#!/usr/bin/env python3
"""verb-coverage.py — which of `EditSession`'s verbs this shell actually calls.

WHY THIS EXISTS
===============

On 2026-08-28 the operator asked a question this project could not answer from
its own documents:

    "confirm that you have built every editable surface into the GUI that has
     been implemented in pdfce"

`FEATURES.md` describes what the GUI does. `NO_SURFACE.md` lists tunables with
no control. Neither is keyed on the ENGINE's verb list, so neither could answer
*"is there a verb pdfce-core implements that nothing here calls?"* — and the
answer turned out to be yes, repeatedly, including for capabilities the engine
had shipped IN ANSWER TO THIS SHELL'S OWN REQUESTS and this shell had then not
consumed.

★★★ The failure mode this closes is specific and this project has now recorded
**seven** instances of it: a blocker written into a doc comment or a backlog
row, true on the day it was written, false within days, and re-read by nobody.
A blocker's reason is prose, and no test can check prose. This is the
instrument that makes the question mechanical instead.

WHAT IT MEASURES, AND WHAT THAT MEASUREMENT IS WORTH
====================================================

For every `pub fn` declared inside an `impl EditSession` block in
`pdfce-core/src/edit.rs`, whether the identifier appears anywhere in
`crates/pdfce-gui/src`.

★★ It is a **grep**, and its limits are worth stating plainly because a number
from a tool reads as authoritative:

  - A hit means the NAME appears, not that a reachable operator route calls it.
    A call site behind a condition nothing sets is a hit here and dead in the
    running program. `tools/ui-verify` is the instrument for that question and
    this one cannot answer it.
  - A miss is stronger: no occurrence of the identifier means nothing here
    calls it, full stop. **The miss list is the useful output.**
  - A miss is not automatically a gap. Roughly a third of the misses are
    builder methods on other types that happen to sit in the same file, or
    `*_with` variants of a verb this shell calls in its plain form. The
    register at `EDITABLE_SURFACES.md` carries a hand-written reason per miss;
    this tool produces the list that register must account for.

USAGE
=====

    python tools/verb-coverage.py                 # the miss list, one per line
    python tools/verb-coverage.py --all           # every verb with its count
    python tools/verb-coverage.py --markdown      # a table for the register

Exit code is 0 always: this is an instrument, not a gate. Making it a gate
would require a checked-in allow-list of "misses that are fine", which is
another prose blocker list, which is the thing it exists to replace.
"""

from __future__ import annotations

import argparse
import pathlib
import re
import sys

ENGINE = pathlib.Path("D:/Dev/pdfce/crates/pdfce-core/src/edit.rs")
GUI = pathlib.Path("crates/pdfce-gui/src")

IMPL = re.compile(r"^impl(?:<[^>]*>)?\s+([A-Za-z0-9_]+)")
# Four-space indent only: a `pub fn` nested deeper is inside a nested item or a
# test module, and `EditSession`'s own verbs are all at one level.
METHOD = re.compile(r"^    pub (?:const )?(?:unsafe )?fn ([a-z0-9_]+)")


def engine_verbs(path: pathlib.Path) -> list[str]:
    """Every `pub fn` declared at one indent inside an `impl EditSession`.

    The scan is a state machine over `impl` headers rather than a parse: the
    file is 34,000 lines and holds dozens of impls, so "every `pub fn` in the
    file" — which is what the first cut of this measurement did — sweeps in
    `MarkupNote::new`, `NewTextField::with_value` and forty other builders and
    reports them as uncalled engine verbs.
    """
    current = None
    out: list[str] = []
    for line in path.read_text(encoding="utf-8", errors="replace").splitlines():
        m = IMPL.match(line)
        if m:
            current = m.group(1)
            continue
        if current != "EditSession":
            continue
        m = METHOD.match(line)
        if m:
            out.append(m.group(1))
    return sorted(set(out))


def gui_hits(names: list[str], root: pathlib.Path) -> dict[str, int]:
    """How many times each identifier appears across the shell's sources.

    One pass over the tree holding every file in memory once, rather than a
    grep per name: 177 names over ~400 files is 70,000 file reads the naive
    way, which took long enough that the first version of this script was
    unusable on Windows.
    """
    blobs = [
        p.read_text(encoding="utf-8", errors="replace")
        for p in root.rglob("*.rs")
    ]
    counts = {}
    for name in names:
        pattern = re.compile(r"\b" + re.escape(name) + r"\b")
        counts[name] = sum(len(pattern.findall(b)) for b in blobs)
    return counts


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--all", action="store_true", help="every verb, with counts")
    ap.add_argument("--markdown", action="store_true", help="a table for the register")
    args = ap.parse_args()

    if not ENGINE.exists():
        print(f"engine not found at {ENGINE}", file=sys.stderr)
        return 0
    verbs = engine_verbs(ENGINE)
    counts = gui_hits(verbs, GUI)
    missing = [v for v in verbs if counts[v] == 0]

    if args.markdown:
        print(f"| verb | occurrences in `crates/pdfce-gui/src` |")
        print("|---|---|")
        for v in verbs:
            print(f"| `{v}` | {counts[v]} |")
    elif args.all:
        for v in verbs:
            print(f"{counts[v]:5d}  {v}")
    else:
        for v in missing:
            print(v)

    print(
        f"\n{len(verbs)} EditSession verbs, {len(verbs) - len(missing)} named "
        f"somewhere in the shell, {len(missing)} named nowhere.",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
