# pdfceGUI — GUI design and remediation workspace

The **GUI rebuild** for **`D:\Dev\pdfce`**: a new `pdfce-gui` crate that
will replace `D:\Dev\pdfce\crates\pdfce-gui\` when complete, built on a
new reusable `egui-shell` crate that knows nothing about PDF and will be
extracted for use by other projects.

Started 2026-08-12 as a design workspace, after a comparison of pdfce
against **Open PDF Studio 1.82.0** (openaec / OpenAEC Foundation, LGPL,
Tauri 2 + SolidJS over a multi-process PDFium worker pool) installed on
this machine. It became the code on 2026-08-13.

**➡ New session? Read `HANDOFF.md` first.**

```
cargo test --workspace         # 1,530 tests, 0 failed
bash tools/gates/run-all.sh    # 10 passed, 0 failed, 0 skipped
```

Stages **S0–S5** are complete, along with **Phase 3** (viewer conventions,
Find, thumbnails, rulers/grid/guides) and **Phase 4** (page display
modes). The application opens documents, navigates, selects, edits,
measures nothing yet, prints, fills forms and finds text. `FEATURES.md`
is the authoritative list, and a row there is ticked **only when an
operator can reach it in a real build**.

Next is **Phase 5 — text editing**, which is the defect that began the
project. It has not been started.

`PROJECT_PLAN.md` §4 has the eight-stage plan.

## Builds

```
python tools/package-portable.py --verify --note "what this milestone added"
```

Writes `D:\builds\pdfcegui-<stamp>-<engine>-<shell>\` — one folder per
build, never an overwrite, because on Windows a running executable
cannot be replaced and a half-updated folder is worse than either
version.

**"Integrated with pdfce as a single exe" needs no fold-in.**
`crates/pdfce-gui` depends on `pdfce-core` and `pdfce-render` **by path**
into `D:\Dev\pdfce`, and Rust links them statically — so the release
binary already carries pdfce's engine. Integration here is a property of
the dependency graph, not a merge that has to happen first.

That is worth stating because the alternative — folding this shell into
`D:\Dev\pdfce` and packaging from there — would ship a **regression**
today: `FEATURES.md` § "Not salvaged yet" still lists measure,
redaction, the settings dialog and text editing as living only in the
old shell. Shipping from here costs nothing and leaves a pdfce build
installable beside it. Fold-in happens when `FEATURES.md` says nothing
regresses, per `PROJECT_PLAN.md` §5.

**Two identities, because there are two source trees.** `<engine>` is
`D:\Dev\pdfce`'s short HEAD; `<shell>` is this workspace's. Either gets
a `-dirty` / `-enginedirty` marker when its tree carries changes that
can reach a compiler — narrowly defined, so a documentation edit does
not raise a warning about the binary.

A **source digest** is recorded alongside them, and joins the folder
name when the shell tree is dirty. It exists because commits here are
taken at milestones while several agents write concurrently, so the tree
is dirty more often than not — and for a dirty tree the commit names the
last checkpoint, not what was compiled. The digest names what was
compiled. It cannot say *what* the code was, only whether two builds
came from identical bytes, which is the question a bug report actually
asks.

`--verify` runs the workspace tests and the CI gates **before** building,
so a failure costs nothing and leaves no folder behind, and records the
result in `BUILD-INFO.txt`. When it is not run the file says so in those
words — an omitted line would read as "nothing to report", when it means
nobody checked.

`python tools/package-portable.py --self-test` asserts the script's own
two invisible invariants: that the folder name is **not** caught by
pdfce's own `pdfce-*` glob (it would otherwise diff *its* changelog
against a commit from this tree), and that the source digest is
deterministic and moves on a rename. Both failures package successfully
and run fine — the damage lands elsewhere, later — which is why they are
asserted rather than reasoned about.

## Licensing, and the material this binary carries that is not ours

pdfceGUI is **MIT** — see `LICENSE`, `Copyright (c) 2026 Ken Mantle`.
That covers everything in this repository, including the icon set, which
is the operator's own art (`crates/pdfce-gui/src/icons/assets/PROVENANCE.md`).

It does **not** cover everything `pdfce-gui.exe` contains. The binary
links `pdfce-core` and `pdfce-render` statically, and those crates embed
third-party font faces and data tables with `include_bytes!` — so this
program redistributes work whose licences require their notices to
travel with it.

Two surfaces carry those notices, and they are not redundant:

| Surface | Carries | Reached by |
|---|---|---|
| `THIRD_PARTY_LICENSES.md`, copied into every build | every licence **text**, in full | anyone who opens the package folder |
| **File ▸ pdfce ▸ About pdfce**, in the program | the **attribution** — who made it, what it is, on what terms, and whether pdfce changed it | anyone who runs it |

`THIRD_PARTY_LICENSES.md` is generated: `cargo about generate about.hbs
-o THIRD_PARTY_LICENSES.md`, from this workspace's real `Cargo.lock`.
Never edit it by hand. Its `accepted` list in `about.toml` is
permissive-only, so a copyleft dependency entering this workspace makes
generation **fail and name the crate** — that failure is the licence
audit. `about.hbs`'s static epilogue carries the non-Cargo assets, which
no generator can see.

`tools/gates/check-shipped-assets.py` enforces the whole arrangement: a
`PROVENANCE.md` beside every redistributed asset directory, a citation
in both notice surfaces unless the asset is our own work, the notice
present in the packager's payload, and the generated file not stale
relative to its template.

**When the OCR feature lands** it brings the `ocrs` model weights, which
are **CC-BY-SA-4.0** and which the operator accepted into this MIT
package on 2026-08-14. Shipping them *unmodified* is distribution of a
verbatim work in a collection and leaves pdfce's own licence untouched.
**Modifying them — fine-tuning, retraining, quantizing to shrink the
file, or converting them to another runtime's format — creates Adapted
Material, and the result must be released under CC-BY-SA-4.0 or a
compatible licence.** That is an engineering constraint, not a footnote:
see `crates/pdfce-gui/src/text/about.rs`.

## Version control

Under git since `2a504ef` (2026-08-13). `.gitattributes` **predates that
commit deliberately**: `core.autocrlf` is true globally on this machine,
and pdfce's 2026-08-02 finding records that CRLF normalization of PDF
fixtures lands **in the index at add time**, not only at checkout. A
PDF's cross-reference table stores absolute byte offsets, so a
normalized fixture is a corrupt one, and 18 evidence PNGs would have
gone the same way. The first `git add` here was made without the file,
noticed, and unwound with `git rm -r --cached` before anything was
committed.

The rebuild is owned by the **`pdfce-gui-engineer`** agent
(`.claude/agents/pdfce-gui-engineer.md`). Its governing rule:
`D:\Dev\pdfce\` is **read-only** until fold-in day, so the working
program keeps working for the whole life of the project.

## Read in this order

| Document | What it is |
|---|---|
| **`HANDOFF.md`** | **Start here in a new session.** Current state, the standing instructions, how the parallel agent work was actually run, the five obligations of registering a command, and what is left in order. It carries what the other documents cannot: the working agreements and the judgement calls. |
| **`FEATURES.md`** | **What works today, and what is next in order.** A row is ticked only when an operator can reach it in a real build — not when the code exists or a test passes. Start here for status. |
| **`PROJECT_PLAN.md`** | The charter: topology, module architecture, eight build stages, the fold-in procedure, risks, open questions. **Review this first.** |
| **`SALVAGE.md`** | What carries over from the old GUI's 49,837 code lines, file by file, in four classes. ~45 % comes across with little change. |
| **`DEFECTS.md`** | What is broken today, with `file:line` for every claim. Start here — it contains the diagnosis of the Delete key and of text editing. |
| **`GUI_ROADMAP.md`** | Phased plan, Phase 0 through Phase 7, plus a standing shell-only backlog and five open questions. |
| **`RIBBON_IA.md`** | The full information architecture: seven tabs plus one contextual tab, every command assigned, every current command migrated. |
| **`MODES_AND_PANELS.md`** | The **Read / Review / Edit** selector and the Inkscape-class flexible panel system. Operator additions, 2026-08-13. A mode is a named workspace layout, which is why the two are one system. |
| **`SHELL_FRAMEWORK.md`** | `egui-shell` — the reusable application shell. The ribbon, dock, modes and keymap are a **serializable manifest**, not code, which delivers cross-project reuse and operator ribbon-customization from one mechanism. |
| **`mockups/ribbon.html`** | Open in a browser. Interactive — click a tab to see its band. Colour-coded by whether each command exists today, exists in core/CLI only, or is new. |
| **`mockups/app.html`** | Six full-window scenes: object selected (Format tab + properties panel + context menu), View ▸ Render options, in-place text editing, the Pages tab, Measure, and placing a revision cloud. |
| **`mockups/modes.html`** | The same document rendered in Read, Review and Edit, with the selector at the far right of the tab row. |
| **`BENCHMARK.md`** | Measured rendering performance on a real 5.6 MB CAD site plan, head to head with the competitor. This is the evidence that overturned an earlier, unmeasured claim about whole-page rendering. |
| **`evidence/`** | Screenshots backing every observational claim, plus `bench-gui-diag.txt`, the raw `PDFCE_DIAG` trace. |

## Evidence index

| File | What it shows |
|---|---|
| `pdfce_max.png` | pdfce, maximised, the shared test drawing |
| `ops_max.png` | Open PDF Studio, same window size, same drawing |
| `crop_settings.png` | pdfce Settings dialog at 3× — the invisible section headings (D2) |
| `crop_tabs_left.png`, `crop_tabs_right.png` | Dock tab labels at 3× — same defect |
| `pdfce_settings.png`, `pdfce_panels.png` | Settings dialog and the Objects + Fonts panels in place |
| `ribbon_*.png` | Every pdfce ribbon tab: edit, review, measure, tools, view |
| `ops_*.png` | Every Open PDF Studio ribbon tab: view, drawing, annotation, edit & combine, settings |

The shared test document is
`C:\Program Files\Open PDF Studio\kaders\grootformaat_a1_liggend.pdf` —
an A1 landscape title-block frame, chosen because both products target
drawing work and it exercises vector paths, subset fonts and
annotations.

## Headline findings

**The Delete key is broken by one line.** `main.rs:13777` guards the
unmodified-key bindings with `ctx.egui_wants_keyboard_input()`, which in
egui 0.35 means *any widget has focus* — not *a text field has focus* —
and the canvas takes focus on the very click that selects an object. The
same guard also kills PageUp/PageDown, Home/End and `[` / `]`. Fix is
`ctx.text_edit_focused()`. Full chain in `DEFECTS.md` D1.

**Section headings and dock tab labels are invisible** in the default
Quiet theme: `widgets.active.fg_stroke` is set to a near-white
`label_backdrop` while `widgets.active.bg_fill` is never assigned the
accent. `DEFECTS.md` D2.

**Three of the six ribbon tabs are underfilled, page operations are not
on the ribbon at all, and the View tab contains no view controls.**
`RIBBON_IA.md` §3.

**Text editing has three distinct problems**, not one: the edit unit is
a single PDF show-text operator rather than a visual box; nothing
re-lays-out while you type, and aligned or rotated text is moved wrongly
on commit; and reflow is blocked behind three gates including one open
filed defect. `DEFECTS.md` D4.

**Two of these were invisible to a green test suite.** `GUI_ROADMAP.md`
proposes a `tools/ui-verify/` harness that drives the real binary as the
highest-leverage item in the plan.

## Decisions taken, 2026-08-13

| Decision | Effect |
|---|---|
| **Pages belongs in Review mode** | Reviewing a set means rotating a sheet to read it and extracting the pages you were asked about. The stance that matters is *the content is not yours to alter*, and page operations do not alter content. |
| **"Nothing floats over the canvas" becomes two settings, not an invariant** | **Floating panels** (Off · Allowed, default Allowed) governs whether *you* may tear a panel out. **App initiative** (Never · Ask · Allowed, **default Never**) governs whether pdfce may float something *on its own*. The second carries the original complaint and its default preserves today's behaviour — as a choice rather than a law. Both under View ▸ Window. |
| **The shell becomes a reusable crate** | `egui-shell` — ribbon, dock, modes, layout persistence, theme, command registry — knowing nothing about PDF, enforced by a CI gate, extracted to its own repo at fold-in. |
| **The ribbon becomes data** | Tabs, groups, commands, modes and keymap are a serializable manifest. This is what makes the shell reusable *and* the ribbon customizable — one mechanism for both. Retires the deferral at `ribbon.rs:42-52`, whose objection was about persistence. |

## Decisions taken, 2026-08-12

| Decision | Effect |
|---|---|
| **Continuous scroll is an option, not a replacement** | Single page stays the default — it is the right model for drafting review. Four page-display modes sit together on the View tab, persisted per document. |
| **Whole-page rendering stays the default** | Now **measured** (`BENCHMARK.md`): six rapid zoom steps started six render generations and completed exactly one, at the destination — 1.9 s instead of ~11 s. The generation counter and settle debounce already solve what a tile cache would have been built to solve. pdfce also uses 2.5× less memory than the competitor on the same file. Tiled progressive becomes an opt-in in a new **View ▸ Render** group. This corrects an earlier draft that called whole-page a weakness on architectural grounds without measuring it. |
| **The `Editing on` master toggle is removed** | *"Make it work the same way other programs do."* Selection and Delete are always live; tools arm and disarm. Supersedes defect D6. |
| **Format tab and properties panel both ship** | Panel first — it holds the full property set including editable X/Y/W/H, and the tab's contents are a subset. Context menus are the third surface. |
