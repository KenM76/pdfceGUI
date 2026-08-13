#!/usr/bin/env python3
"""Build the pdfceGUI portable distribution into ``D:\\builds``.

WHY THIS EXISTS
===============

Operator request, 2026-08-13: *"When complete compile integrated with
pdfce in d:\\builds as a single exe like pdfce does. ... Basically I want
this to get to where I can use it to replace acrobat reader first."*

"Integrated with pdfce" needed no fold-in, and that is the single most
important fact about this script. `crates/pdfce-gui` depends on
`pdfce-core` and `pdfce-render` **by path** into `D:\\Dev\\pdfce` (see the
workspace root manifest, and PROJECT_PLAN.md §2). Rust links those
statically. So `cargo build --release -p pdfce-gui` in THIS workspace
already produces one self-contained executable carrying pdfce's engine
— the integration the request asks for is a property of the dependency
graph, not a merge that has to happen first.

That matters because the alternative reading — fold the new shell into
`D:\\Dev\\pdfce` and package from there — would have shipped a REGRESSION.
`FEATURES.md` § "Not salvaged yet" lists measure, redaction, settings
and text editing as still living only in the old shell. Replacing that
shell today to get one exe would trade four working capabilities for a
packaging convenience. Shipping from here costs nothing and keeps the
old build intact and installable beside it.

WHAT IT PRODUCES
================

``D:\\builds\\pdfcegui-<YYYYMMDD-HHMM>-<engine>-<src>[-enginedirty]\\``

    pdfce-gui.exe            the new shell, with pdfce's engine linked in
    LICENSE                  MIT
    README.md                the project README
    FEATURES.md              what works today and what does not
    BUILD-INFO.txt           identity, verification state, what to try

`FEATURES.md` ships deliberately. The operator's constraint on the
pdfce builds was *"so I can try it out while you are working"*, and
"try it out" is answerable only if the operator knows what is finished.
Here that matters more than it did there, because this build is a shell
mid-rebuild: without the feature list, a missing Measure tab reads as a
bug rather than as a row that is not ticked yet.

``userdata/`` is deliberately NOT created, for the same reason as pdfce's
packager: the app makes it at first run, so "replace the folder" stays a
safe update procedure precisely because payload and state are
distinguishable.

TWO IDENTITIES, BECAUSE THERE ARE TWO SOURCES
=============================================

pdfce's packager names a build after one commit. It cannot here, and the
difference is not cosmetic — **this binary is built from two trees**:

* the **engine**, `D:\\Dev\\pdfce`, which IS a git repository and which
  another session edits live; and
* the **shell**, this workspace, which is **not under version control at
  all**.

So the name carries both. `<engine>` is pdfce's short HEAD. `<src>` is a
12-hex digest over this workspace's build-affecting source, computed by
`source_digest()` below. The digest is not a substitute for git — it
cannot tell you *what* the code was, only whether two builds were built
from identical code — but that is the question a bug report actually
asks ("is this the build I was running?"), and answering it is strictly
better than a name that answers nothing.

If this workspace is ever put under git, replace `source_digest()` with
the same `git rev-parse --short HEAD` the engine uses and delete this
paragraph. The digest exists because of an absence, not because it is
the better design.

THE `-enginedirty` SUFFIX IS LOAD-BEARING
=========================================

`D:\\Dev\\pdfce` is read-only *to this project*, not to its own session,
which means the engine tree can and does move underneath a build. If it
carried uncommitted changes under `crates/`, the linked engine is **not**
the commit in the folder name, and the name would otherwise be a claim
the operator could reasonably rely on and that is false.

The test is deliberately narrow, copied from pdfce's packager and for the
reason recorded there: changes under `docs/`, `fixtures/` and `.claude/`
cannot reach a compiler, so a package built while a documentation agent
was writing is still exactly the named commit. Flagging those produces a
warning that fires when nothing is wrong, and a warning that fires when
nothing is wrong is one that gets ignored when something is. Non-build
changes are still REPORTED in `BUILD-INFO.txt`, just not as a claim about
the payload.

THE NAME MUST NOT START WITH ``pdfce-``
=======================================

`D:\\Dev\\pdfce\\tools\\package-portable.py` finds its previous build with
``dest.glob("pdfce-*")`` and reads that folder's `BUILD-INFO.txt` to pick
the commit its changelog diffs against. A build of THIS project named
`pdfce-gui-2026...` would match that glob, and pdfce's next package would
diff its history against a commit hash belonging to a different tree —
producing a silently wrong changelog in a file whose entire purpose is to
tell the operator what changed.

`pdfcegui-` does not match `pdfce-*` (the glob needs a literal hyphen at
that position). One absent hyphen is the whole safeguard, and a safeguard
that thin cannot rest on a reader noticing it — so ``--self-test``
asserts it against a real temporary directory and pdfce's own glob
string, following the self-test convention `tools/gates/check-ui-strings.sh`
established in this project after a gate shipped that could not catch its
own bug.

EXIT CODES
==========

``0`` build written; ``1`` a prerequisite is missing (no cargo, no engine
tree, no binary after a build attempt, or `--verify` failed); ``2`` the
destination exists and is non-empty, which should be impossible given the
timestamp and means something else is writing there.
"""

from __future__ import annotations

import argparse
import hashlib
import re
import shutil
import subprocess
import sys
from datetime import datetime
from pathlib import Path

# Windows consoles default to a code page that cannot encode this file's
# arrows and dashes; see the same block in pdfce's packager and in the
# Python gates.
if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

REPO = Path(__file__).resolve().parent.parent
ENGINE = Path("D:/Dev/pdfce")
DEFAULT_DEST = Path("D:/builds")

#: Payload files copied verbatim from the repo root, in the order they are
#: listed in `BUILD-INFO.txt`. Deliberate rather than alphabetical:
#: licence first, then the two documents an operator opens.
PAYLOAD_DOCS = ["LICENSE", "README.md", "FEATURES.md"]

#: The one shipped binary. Unlike pdfce's packager there is no CLI here:
#: `pdfce-cli` is a member of the ENGINE workspace, and building it would
#: mean invoking cargo inside `D:\Dev\pdfce` — a write to a tree this
#: project holds read-only (it would create/refresh `target/`). Operators
#: who want the CLI take it from a pdfce build, which is where it is
#: maintained.
BINARIES = ["pdfce-gui.exe"]

#: Prefixes under the ENGINE tree that can reach a compiler. Generous on
#: purpose: under-reporting here produces the false reassurance the
#: `-enginedirty` suffix exists to prevent, while over-reporting costs
#: only an unnecessary suffix.
BUILD_AFFECTING = (
    "crates/",
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain",
    ".cargo/",
    "build.rs",
)

#: What `source_digest()` hashes. Mirrors BUILD_AFFECTING for this tree.
#: `tools/` is EXCLUDED — this file lives there, and a packager edit is
#: not a change to the shipped program. `tools/ui-verify` is a workspace
#: member but produces no shipped artifact.
SOURCE_GLOBS = ("crates/**/*.rs", "crates/**/Cargo.toml")
SOURCE_FILES = ("Cargo.toml", "Cargo.lock")


def git(*args: str, cwd: Path = ENGINE) -> str:
    """Run a git command in `cwd` and return its stripped stdout.

    Defaults to the ENGINE tree, since this workspace has no repository.

    `encoding="utf-8"` is not optional, and the reason is recorded in
    pdfce's packager: `text=True` alone decodes with the LOCALE codec,
    cp1252 on this machine, and pdfce's commit subjects are full of
    em-dashes — so a changelog rendered `Pass 54.1 â€” a group can be
    deleted`. `errors="replace"` so a stray non-UTF-8 byte in some future
    commit degrades one character instead of failing a build.
    """
    return subprocess.run(
        ["git", *args],
        cwd=cwd,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        check=False,
    ).stdout.strip()


def source_digest(repo: Path) -> str:
    """A 12-hex digest identifying this workspace's build-affecting source.

    Stands in for the commit hash this project does not have. The contract
    is narrow and worth stating exactly, because a digest invites more
    trust than it earns: **identical digests mean identical source; a
    different digest means the source differs, somewhere.** It cannot say
    what differs, and it is not a version — 12 hex digits sort
    meaninglessly.

    Determinism is the whole value, so both inputs are normalised:

    * **Paths** are relative and forward-slashed, so the digest does not
      change if the workspace is moved or built from a different shell.
    * **Order** is sorted by that relative path, because `Path.glob` order
      is filesystem-dependent — the same tree on a different volume would
      otherwise digest differently.

    The path is hashed alongside the bytes so that RENAMING a file changes
    the digest. Hashing contents alone would call a rename a no-op, and a
    rename absolutely changes the program.
    """
    h = hashlib.sha256()
    paths: set[Path] = set()
    for pattern in SOURCE_GLOBS:
        paths.update(p for p in repo.glob(pattern) if p.is_file())
    for name in SOURCE_FILES:
        p = repo / name
        if p.is_file():
            paths.add(p)
    for p in sorted(paths, key=lambda q: q.relative_to(repo).as_posix()):
        h.update(p.relative_to(repo).as_posix().encode("utf-8"))
        h.update(b"\0")
        h.update(p.read_bytes())
    return h.hexdigest()[:12]


def path_of(line: str) -> str:
    """The path out of one `git status --porcelain` line.

    `XY <path>`, or `XY <old> -> <new>` for a rename; the destination is
    what exists on disk now.

    Parsed with a regex rather than `line[3:]`, and the reason is a bug
    pdfce's packager hit: the status field is two columns and EITHER may
    be a space, while `git()` strips its output — which removes the
    leading space from the first line only. A fixed offset therefore ate
    one character of exactly one path per run, printing
    `claude/agent-memory/...` for `.claude/...`. One wrong entry in a list
    of correct ones is the worst version of this bug: the list looks
    right, so the reader trusts the entry that is not, and a path that
    does not exist reads as a deleted file rather than as a parse error.
    """
    m = re.match(r"^\s*\S{1,2}\s+(.*)$", line)
    if not m:
        return ""
    return m.group(1).split(" -> ")[-1].strip().strip('"')


def previous_build(dest: Path) -> tuple[str, str] | None:
    """`(engine_commit, source_digest)` of the most recent build of THIS
    project in `dest`, or `None`.

    Read out of that build's own `BUILD-INFO.txt` rather than parsed from
    the folder name: the name is for humans and can be renamed, the file
    is the record.

    The glob is `pdfcegui-*`, which excludes pdfce's own builds. Reading
    one of those would produce a changelog diffing this tree's history
    against a commit from a build of a different program.
    """
    builds = sorted(
        (p for p in dest.glob("pdfcegui-*") if p.is_dir()),
        key=lambda p: p.stat().st_mtime,
        reverse=True,
    )
    for b in builds:
        info = b / "BUILD-INFO.txt"
        if not info.is_file():
            continue
        engine = src = ""
        for line in info.read_text(encoding="utf-8", errors="replace").splitlines():
            if line.startswith("Engine:"):
                engine = line.split(":", 1)[1].strip().split()[0]
            elif line.startswith("Shell source:"):
                src = line.split(":", 1)[1].strip().split()[0]
        if engine:
            return engine, src
    return None


def run_verification(repo: Path) -> tuple[bool, str]:
    """Run the test suite and the CI gates; return `(ok, report)`.

    Opt-in via `--verify` rather than always-on, because a full workspace
    test run is minutes and the common case is packaging a build the
    session just verified.

    But when it is NOT run, `BUILD-INFO.txt` says so in those words. An
    omitted verification line would read as "nothing to report", which is
    the opposite of the truth: it means nobody checked. This project's
    whole verification stance is that a passing test is not evidence an
    operator can reach a feature (`FEATURES.md`), so a build that quietly
    implies it was verified is exactly the wrong artifact to ship.
    """
    lines = []
    ok = True
    for label, cmd in (
        ("tests", ["cargo", "test", "--workspace", "--quiet"]),
        ("gates", ["bash", "tools/gates/run-all.sh"]),
    ):
        print(f"package-portable: {label} ...")
        r = subprocess.run(
            cmd,
            cwd=repo,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            check=False,
        )
        passed = r.returncode == 0
        ok = ok and passed
        tail = (r.stdout or r.stderr or "").strip().splitlines()[-3:]
        lines.append(f"  {label}: {'PASS' if passed else 'FAIL'}")
        lines.extend(f"    {t}" for t in tail)
    return ok, "\n".join(lines)


def self_test() -> int:
    """Assert the two invariants that are otherwise invisible.

    Both are properties nothing else would catch. A wrong folder name
    still packages successfully and still runs; the damage lands in a
    DIFFERENT project's changelog, days later. A non-deterministic digest
    still produces a name; it just quietly stops answering the one
    question it exists to answer. Neither failure mode announces itself,
    which is precisely why they are asserted rather than reasoned about.
    """
    import tempfile

    failures: list[str] = []

    # 1. The name must not be picked up by pdfce's own packager. Asserted
    #    against a real directory and pdfce's literal glob string, not by
    #    comparing prefixes — the whole point is to test the thing that
    #    actually runs over there.
    with tempfile.TemporaryDirectory() as td:
        root = Path(td)
        (root / "pdfcegui-20260813-1200-abc1234-0123456789ab").mkdir()
        (root / "pdfce-20260813-1200-abc1234").mkdir()
        caught = sorted(p.name for p in root.glob("pdfce-*") if p.is_dir())
        if caught != ["pdfce-20260813-1200-abc1234"]:
            failures.append(
                "a pdfceGUI build name matches pdfce's `pdfce-*` glob; "
                f"its packager would read {caught} as its own previous build"
            )
        mine = sorted(p.name for p in root.glob("pdfcegui-*") if p.is_dir())
        if len(mine) != 1:
            failures.append(f"`pdfcegui-*` did not match exactly this project's build: {mine}")

    # 2. The digest must be stable across repeated calls on an unchanged
    #    tree, and must move when a byte does. Run against the real
    #    workspace, because a synthetic tree would not exercise the glob
    #    patterns that decide what is hashed in the first place.
    a = source_digest(REPO)
    b = source_digest(REPO)
    if a != b:
        failures.append(f"source_digest is not deterministic: {a} != {b}")
    if len(a) != 12:
        failures.append(f"source_digest returned {len(a)} chars, expected 12")
    with tempfile.TemporaryDirectory() as td:
        root = Path(td)
        (root / "crates" / "x" / "src").mkdir(parents=True)
        f = root / "crates" / "x" / "src" / "lib.rs"
        f.write_text("fn a() {}", encoding="utf-8")
        before = source_digest(root)
        f.write_text("fn b() {}", encoding="utf-8")
        if source_digest(root) == before:
            failures.append("source_digest did not change when a source byte changed")
        f.rename(root / "crates" / "x" / "src" / "renamed.rs")
        if source_digest(root) == before:
            pass  # already differs; the rename check below is the real one
        moved = source_digest(root)
        (root / "crates" / "x" / "src" / "renamed.rs").rename(f)
        if source_digest(root) == moved:
            failures.append("source_digest ignored a rename; it must hash paths too")

    for msg in failures:
        print(f"package-portable --self-test: FAIL — {msg}")
    if failures:
        return 1
    print("package-portable --self-test: 2 invariants hold (name collision, digest).")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--dest",
        type=Path,
        default=DEFAULT_DEST,
        help="destination root (default: D:/builds)",
    )
    ap.add_argument(
        "--no-build",
        action="store_true",
        help="package the existing target/release binary without rebuilding",
    )
    ap.add_argument(
        "--verify",
        action="store_true",
        help="run the workspace tests and the CI gates, and record the result",
    )
    ap.add_argument(
        "--note",
        default="",
        help="one-line summary of what this milestone added, for BUILD-INFO.txt",
    )
    ap.add_argument(
        "--self-test",
        action="store_true",
        help="assert this script's own invariants and exit; packages nothing",
    )
    args = ap.parse_args()

    if args.self_test:
        return self_test()

    if not (REPO / "Cargo.toml").is_file():
        print(f"package-portable: {REPO} is not the pdfceGUI workspace root.")
        return 1
    if not (ENGINE / "Cargo.toml").is_file():
        # Not a nicety. The path dependencies point here; without it the
        # build below fails with a cargo error about a missing manifest
        # that does not mention why the path matters.
        print(f"package-portable: the engine tree {ENGINE} is missing.")
        print("  crates/pdfce-gui depends on pdfce-core and pdfce-render by path.")
        return 1

    # --- identity, before anything is built --------------------------------
    engine_short = git("rev-parse", "--short", "HEAD") or "unknown"
    engine_subject = git("log", "-1", "--format=%s")
    changed = [path_of(l) for l in git("status", "--porcelain").splitlines() if l.strip()]
    dirty_build = [p for p in changed if p.startswith(BUILD_AFFECTING)]
    dirty_other = [p for p in changed if p not in dirty_build]
    src = source_digest(REPO)

    stamp = datetime.now().strftime("%Y%m%d-%H%M")
    name = f"pdfcegui-{stamp}-{engine_short}-{src}" + ("-enginedirty" if dirty_build else "")
    out = args.dest / name

    if out.exists() and any(out.iterdir()):
        print(f"package-portable: {out} already exists and is not empty.")
        return 2

    # --- verify, BEFORE building -------------------------------------------
    #
    # Order is deliberate: a failed verification should cost the operator
    # nothing and leave no folder behind. Verifying after packaging would
    # produce a written build that the same run then declares unfit.
    verification = "  not run for this package (pass --verify to run it)"
    if args.verify:
        ok, verification = run_verification(REPO)
        if not ok:
            print("\npackage-portable: verification failed; nothing packaged.")
            print(verification)
            return 1

    # --- build --------------------------------------------------------------
    if not args.no_build:
        print("package-portable: cargo build --release -p pdfce-gui ...")
        rc = subprocess.run(
            ["cargo", "build", "--release", "-p", "pdfce-gui"],
            cwd=REPO,
            check=False,
        ).returncode
        if rc != 0:
            print("package-portable: the release build failed; nothing packaged.")
            return 1

    rel = REPO / "target" / "release"
    missing = [b for b in BINARIES if not (rel / b).is_file()]
    if missing:
        print(f"package-portable: missing binaries in {rel}: {', '.join(missing)}")
        return 1

    # --- assemble -----------------------------------------------------------
    out.mkdir(parents=True, exist_ok=True)
    for b in BINARIES:
        shutil.copy2(rel / b, out / b)
    copied_docs = []
    for d in PAYLOAD_DOCS:
        s = REPO / d
        if s.is_file():
            shutil.copy2(s, out / d)
            copied_docs.append(d)
        else:
            print(f"package-portable: WARNING — {d} not found at the workspace root")

    # --- what changed since the last build ----------------------------------
    prev = previous_build(args.dest)
    if prev:
        prev_engine, prev_src = prev
        engine_changes = git("log", "--oneline", "--no-decorate", f"{prev_engine}..HEAD")
        change_header = f"Engine commits since the previous build ({prev_engine}):"
        shell_line = (
            "  the shell source is UNCHANGED since the previous build"
            if prev_src == src
            else f"  the shell source CHANGED since the previous build ({prev_src} -> {src})"
        )
    else:
        engine_changes = git("log", "--oneline", "--no-decorate", "-10")
        change_header = "Recent engine commits (no previous pdfceGUI build to diff against):"
        shell_line = "  no previous build to compare the shell source against"
    engine_changes = (
        "\n".join(f"  {l}" for l in engine_changes.splitlines())
        if engine_changes
        else "  (none)"
    )

    warn = ""
    if dirty_other and not dirty_build:
        listed = "\n".join(f"  {q}" for q in dirty_other[:10])
        more = "\n  ..." if len(dirty_other) > 10 else ""
        warn = (
            "\nEngine tree note: files outside the build were modified when this\n"
            "was packaged (documentation, fixtures, or agent memory). None of them\n"
            "reach the compiler, so the engine linked here IS the commit below:\n"
            f"{listed}{more}\n"
        )
    if dirty_build:
        listed = "\n".join(f"  {q}" for q in dirty_build[:10])
        more = "\n  ..." if len(dirty_build) > 10 else ""
        warn = (
            "\n*** THE ENGINE WAS BUILT FROM AN UNCOMMITTED WORKING TREE ***\n"
            "The commit below identifies D:\\Dev\\pdfce's last COMMIT, not what was\n"
            "linked into this binary. These build-affecting files were modified:\n"
            f"{listed}{more}\n"
        )

    # Hoisted out of the f-string: an expression part may not contain a
    # backslash before Python 3.12, and this machine runs 3.11.
    note_block = f"THIS BUILD\n----------\n{args.note}\n" if args.note else ""

    (out / "BUILD-INFO.txt").write_text(
        f"""pdfceGUI portable build
=======================
{warn}
Built:  {datetime.now().strftime("%Y-%m-%d %H:%M:%S")}
Engine: {engine_short}  {engine_subject}
        (D:\\Dev\\pdfce, branch {git("rev-parse", "--abbrev-ref", "HEAD")})
Shell source: {src}  (sha256 digest — this workspace is not under git)

WHAT THIS IS
------------
The NEW pdfce shell, rebuilt from scratch, with pdfce's engine linked in
statically. One executable, no installer, no registry writes, nothing
outside this folder. Run pdfce-gui.exe.

It is not a replacement for a pdfce build yet, and FEATURES.md ships
beside it so you can tell which is which. Measure, redaction, the
settings dialog and text editing still live only in the OLD shell —
install this alongside a pdfce build rather than instead of one.

WHAT TO TRY
-----------
Open a PDF, then: click an object and press Delete; drag a selection;
press Escape mid-drag; switch Read / Review / Edit with Ctrl+1/2/3; drag
a panel to another dock column and restart to confirm the layout came
back. FEATURES.md's first list is the full set that should work.

YOUR SETTINGS LIVE IN THIS FOLDER
---------------------------------
On first run the app creates `userdata/` here. To update, replace the
binary but KEEP `userdata/`.

VERIFICATION
------------
{verification}

{note_block}
{change_header}
{engine_changes}

Shell source:
{shell_line}

Files in this build: {", ".join(BINARIES + copied_docs)}
""",
        encoding="utf-8",
    )

    total = sum(f.stat().st_size for f in out.rglob("*") if f.is_file())
    print(f"\npackage-portable: wrote {out}")
    for f in sorted(out.iterdir()):
        print(f"  {f.name:<26} {f.stat().st_size:>10,} bytes")
    print(f"  {'TOTAL':<26} {total:>10,} bytes")
    if dirty_build:
        print("\n  WARNING: the engine had uncommitted build-affecting changes.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
