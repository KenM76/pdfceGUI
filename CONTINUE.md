# CONTINUE — start here, then keep going

**Rewritten 2026-08-19 at `f06ee2d`, clean tree, gates 14/14, 1,431 tests.**
**Newest portable build: `OneDrive\pdfceGUI2`.**

You are the `pdfce-gui-engineer`. This file is the entry point when the
operator types **“continue”** and nothing else. Read it, read the three files
in §1, then start at the top of §3 without asking which item to do.

---

## 0. The one rule that governs everything

> **`D:\Dev\pdfce\` is READ-ONLY.**

Read it constantly — it is the engine and the salvage source. Write to it
never. If the shell needs something `pdfce-core` does not have, **file it in
the request channel** (§6) and carry on with something else. A parallel session
works that repo and answers within the hour; four requests were filed and four
shipped inside a day on 2026-08-19.

---

## 1. Read these before touching anything

| File | Why |
|---|---|
| `RESUME.md` | The long state document: every surface, every check, the harness's history and its false failures. **§“Standing operator instructions” is not optional.** |
| `.claude/agents/pdfce-gui-engineer.md` | The role: R1–R9, the rules that produced every design decision here |
| `D:\dev\rag\egui\index.md` | Empirical findings from this codebase. Four of them were written on 2026-08-19 and **three describe defects you will otherwise re-create** |

`D:\Dev\pdfce\docs\core-api\index.md` before calling any engine verb.
`D:\Dev\pdfce\docs\FEATURES.md`'s `gui` column is the acceptance criteria.

---

## 2. The operator's own list, verbatim, and what is left of it

He raised these on 2026-08-19 with *“I bring them up over and over again and
they are still not dealt with.”* He was right. **This list outranks anything
you or I think is more interesting.**

| # | His words | State |
|---|---|---|
| 1 | *“the I cursor turns white for text selection so I cant see it on a white background”* | ✅ **done**, `277a040`. Two-tone I-beam, same fix the crosshair got |
| 2 | *“the measuring tools don't give me any indication of what is being selected … hover over a line or node”* | ✅ **done**, `ae5d0d4`. He confirmed he wants **both** entity and node |
| 3 | *“the groups editor popup … too long for some screens so can't close it … should come up in the side bar and be scrollable and each section should be able to fold up like the settings one”* | ✅ **done**, `cbb3469`. `panels::dimension_groups`, six folds, five shut. **Not driven** — he was on the machine |
| 4 | *“no side bar area showing what tool is active and its options”* | ⬜ **NEXT.** §3.2. Fixes #5 too |
| 5 | *“no text editing or adding text on the canvas”* | ⚠ **they exist and he cannot find them** — see §4.1. #4 is the fix |
| 6 | *“still no revision cloud tool”* | ✅ **done**, `c972dfd`. `MarkupKind::Cloud`, `/BE /I 1.0`, its own glyph, ribbon row after Polygon. **Not driven** |

---

## 3. What to do next, in order, without asking

### 3.1 ✅ Dimension groups — done, `cbb3469`

`panels/dimension_groups/`, mounted in Review and Edit, six foldable sections
of which **one** starts open. Its module header carries the fold policy and is
worth reading before building any other panel: the rule is *what does an
operator need to READ without asking*, not *what do they most often change* —
and the second question is the trap that made this surface taller than his
screen in the first place.

`measure.manage_groups` is the panel's command, with **no** second
`view.panel_*` id. Redact's precedent: one surface, one id, and the mode
taxonomy does the gating for free.

⚠ **Not verified by driving.** `dimension_groups_panel_makes_a_group` was
rewritten to press fold headings and to open-then-shut the Appearance section,
and it compiles and has never been run.

### 3.2 ⬜ A tool panel — what is armed, and its options — **NEXT**

The biggest single win left and it fixes two complaints at once (#4 and #5).
There is still no surface anywhere that says which canvas tool is armed.

- `crate::canvas::tool::CanvasTool` already knows.
- `MODES_AND_PANELS.md` and `RIBBON_IA.md` are the spec — **do not improvise
  the IA.**
- `panels::dimension_groups` is now the worked example of a panel with folds;
  copy its layout discipline, not its content.
- Get a UX critique before designing it. The `pdfce-ui-specialist` agent is
  **not on this session's roster** — dispatch `general-purpose` with the
  specialist's framing instead.

★ The hard part is **the empty state**, which is what the application opens in
and where the operator spends most of their time. An empty panel teaches people
to close it; a panel of *“nothing selected”* placeholders breaks R9. Solve that
first, not last.

### 3.3 ✅ Revision clouds — done, `c972dfd`

`MarkupKind::Cloud`, `markup.cloud` at token 507, `shape-cloud.svg`, ribbon row
directly after Polygon. It joins `is_vertex` and nowhere else in that impl;
`markup::action` takes Polygon and Cloud through one arm; the whole difference
is `/BE /I 1.0` in `spec`, which is Acrobat's default cloud.

`markup_shapes` gained **phase F**, which asserts one field — `kind=Cloud` on
`markup-commit`. A build whose control armed `Polygon` would pass every other
assertion in that check.

⚠ **Not verified by driving.**

### 3.4 ⬜ Three things this session left on the floor, in order of size

1. **Run the driven suite.** Three checks changed today and none has been run:
   `dimension_groups_panel_makes_a_group` (rewritten for the panel),
   `markup_freehand_and_vertex_kinds` (phase F), and everything the new
   Dimension-groups tab might have shifted in the dock for other panels'
   coordinates. **Ask before driving — it needs the machine.**
2. **A Format-tab surface for a selected markup.** `set_markup_style` shipped in
   the engine and takes `/C`, `/IC`, `/BS /W`, `/CA` and `/LE`. This shell has
   no surface for any of it. That is now the largest engine capability with no
   route from this GUI.
3. **`NO_SURFACE.md` §1c wants a sweep.** Every remaining blocker in that file
   that names `pdfce-core` is a claim this project cannot re-check. Two were
   found false in one session. Re-derive the rest, and rewrite each as a dated
   citation.

---

## 4. Two things that are true and surprising

### 4.1 Text on canvas EXISTS and he cannot find it

`edit.text` and `edit.add_text` are registered, on the Edit tab, bound to
`Ctrl+E` and `Ctrl+Shift+E`, and **two driven checks pass on them**
(`add_text_takes_real_keystrokes`, `text_edit_pins_an_aligned_tail`).

So this is not a missing feature. It is a **discoverability defect**, which is
this project's founding failure wearing different clothes, and I marked it
green. §3.2 is the fix; do not “build text editing”.

### 4.2 The suite is not deterministic

The last full run was 35 passed · 1 failed · 4 skipped, and **all three
non-passes passed in isolation**, with messages pointing at pointer injection
and window activation rather than the application.

> **A full-suite red is not a defect report until the member has been re-run
> alone.**

---

## 5. How to work

```bash
# ALWAYS first — the engine repo moves several times a day
cargo update -p pdfce-core -p pdfce-render -p pdfce-print

cargo fmt --all
cargo test -q -p pdfce-gui                 # 1,432 at this commit
bash tools/gates/run-all.sh                # 14/14, all must pass

# Drive it. Needs the operator off the machine — ASK, unless he has said go.
cargo build --release -q -p pdfce-gui -p ui-verify
cargo run --release -q -p ui-verify -- \
  --exe target/release/pdfce-gui.exe \
  --pdf D:/Dev/temp/pdfce/SW41177.pdf --doc-point 0,300,500 \
  --check <name>                            # `--check`, NOT `--only`

# And publish. Standing rule, restated by the operator 2026-08-19.
python tools/package-portable.py
```

`package-portable.py` alternates `OneDrive\pdfceGUI1` / `pdfceGUI2` itself and
preserves each slot's `userdata/`. **Say which slot in your report** — the name
carries no version. At this commit the newest is **`pdfceGUI1`**.

44 checks are declared. `page_ops_round_trip` needs
`D:\Dev\pdfce\fixtures\synthetic\pageops\four-pages.pdf` (the standard fixture
has 36 `/Rotate` entries and the evidence would be indistinguishable).

---

## 6. The request channel

`D:\Dev\FeatureRequests\pdfce_FeatureRequests\open\`

`request_<topic>.md` goes out, `note_<topic>.md` answers, replies come back as
`YYYY-MM-DD-*.md`. **One topic per file.** Read it at the start of every
session — things land unprompted.

★ **The `gui` column of `D:\Dev\pdfce\docs\FEATURES.md` is officially ours as
of 2026-08-19** — `2026-08-19-the-gui-column-is-yours-now-officially.md`. It is
now a report on *this* build, and the engine session has asked to be told when a
row it ticked is not actually reachable here. It has adopted this project's
ticking bar: *a row is ticked only when an operator can reach it in a real
build.* And it wants ⛔ rows that are blocked on `pdfce-core` **filed as
requests**, so a core gap is visible from their side rather than showing as an
empty box.

★★ **A blocker naming `D:\Dev\pdfce\` cannot fail a test here.** Two were found
false on 2026-08-19 — `markup.cloud`'s PLANNED entry and `NO_SURFACE.md`'s
opacity row — and the first cost the operator three weeks of asking for a tool
whose only blocker had already shipped. Write every external blocker as a
**dated citation**, never as a verdict, and re-derive before acting on one. It
is one `grep`. `NO_SURFACE.md` §1c and `D:\devagust\` carry the whole
argument.

Open at this commit: everything from the insert-pages request is **shipped**
except `Pass 102.1` (carry field definitions across `insert_pages`), which the
engine will start unless told otherwise. It reduces `orphaned_widgets` and can
never zero it, so the Register rows are permanent.

★ **Report every workaround, even successful ones** (pdfce decision 058). A
workaround is a finding about where the crate boundary sits.

---

## 7. ★★ The lesson of 2026-08-19, which cost a day

**Read the trace before believing the check.**

Driving the binary found four application defects that 1,432 passing tests
could not see. It also produced **five confident, specific, entirely wrong
defect reports about working code**:

| the check said | the truth |
|---|---|
| the drop caret was “never published” | its rectangle was in the trace four lines above the release — a gesture overlay is always retired before an out-of-process harness can look |
| “1 row before, 2 after the delete” | the delete worked; the counting helper collects every name ever seen, and its own doc said *“used only for SKIP reasons”* |
| the shortcuts list was “0.0 pt high” | a region published as the first statement in a `ScrollArea` closure, over `ui.min_rect()`, before anything was laid out |
| the measure hover “was missing” at a computed point | the point landed 135 pt away on blank paper |
| the highlight “never retires” over blank paper | a CAD sheet has a drawing border; the corner is not blank |

Two rules fell out of that and both are now in the RAG:

1. **A harness assertion is a claim about the program *and* about the harness,
   and only one of them is under test.**
2. **An instrument that can only return one answer cannot detect the thing it
   was added to detect.** Put diagnostics at the *entry* of a function with
   early returns, naming each gate — one at the bottom emits nothing and tells
   you only that the function did not finish.

And the application-side pattern, which recurred **four times in one day**:

> **A control that must be reachable cannot be placed after an unbounded
> `ScrollArea`, and reserve-and-hope is the same defect with a tuning
> parameter.**

Bookmarks' authoring row, the Manage-groups Add button, the Register rows and
the Forms panel's whole body were each unreachable in the exact state that
needed them. Grep for `ScrollArea::vertical()` and check what follows it.

---

## 8. Session shutdown

1. `RESUME.md` and this file reflect **reality**, not intent.
2. Findings go to `D:\dev\rag\egui\` or `D:\dev\rag\rust\` — write the lesson,
   do not ask whether to.
3. Anything needing a change in `D:\Dev\pdfce\` is a hand-off, never applied.
4. Package and say which slot.
