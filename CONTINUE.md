# CONTINUE — start here, then keep going

**Rewritten 2026-08-19 at `880354c`, clean tree, gates 14/14, 1,452 tests.**
**Newest portable build: `OneDrive\pdfceGUI1`.**
**All six of the operator's standing complaints are closed. None is driven.**

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
| 4 | *“no side bar area showing what tool is active and its options”* | ✅ **done**, `d33d228`. `panels::tool`, own dock stack, first. **Not driven** |
| 5 | *“no text editing or adding text on the canvas”* | ✅ **addressed by #4**, `d33d228`. They always existed; see §4.1. **Whether the fix works is the one thing driving must check** |
| 6 | *“still no revision cloud tool”* | ✅ **done**, `c972dfd`. `MarkupKind::Cloud`, `/BE /I 1.0`, its own glyph, ribbon row after Polygon. **Not driven** |

---

## 3. What to do next, in order, without asking

### 3.0 ★★ **DRIVE THE SUITE.** Everything below is second.

**Six features shipped on 2026-08-19 and not one of them has been driven.**
The operator was at the machine all day, so R1 — *verify by driving the binary,
not by a passing test* — was suspended for a whole day's work. That is the
largest verification debt this project has ever carried, and it is carrying it
on the exact class of change R1 exists for: two new panels, a new dialog on the
close path, a new markup kind, and a new ribbon row.

Ask first — it needs the machine. Then:

```bash
cargo build --release -q -p pdfce-gui -p ui-verify
cargo run --release -q -p ui-verify -- \
  --exe target/release/pdfce-gui.exe \
  --pdf D:/Dev/temp/pdfce/SW41177.pdf --doc-point 0,300,500
```

Three checks were **rewritten and never run**:

| check | what changed |
|---|---|
| `dimension_groups_panel_makes_a_group` | drives a **panel**, presses fold headings, opens and shuts Appearance |
| `markup_freehand_and_vertex_kinds` | phase F asserts `kind=Cloud` on `markup-commit` |
| everything else in the right dock | the **Tool panel took the top stack in Review and Edit**, so every other panel's coordinates moved |

★ And the one check that does not exist yet is the one this day's work most
needs. **A first-frame discoverability assertion**: launch, open the fixture,
enter Edit, screenshot with **zero clicks**, assert the strings `Add text` and
`Edit text` are on screen and inside the Tool panel's rect. That is the check
that would have caught the defect the Tool panel was built for, and the one that
proves it is fixed. A check asserting the panel *renders* would repeat the
original failure exactly.

Second: arm `edit.text`, click blank paper, assert `Refusal::NoRun`'s sentence
is on screen in the panel. It fails today for want of somewhere to put it, which
is the whole point.

★★ And read the trace before believing any red. §4.2 and §7.

### 3.1 ✅ Everything on the operator's list — done, none driven

`cbb3469` dimension groups · `c972dfd` revision cloud · `d33d228` Tool panel ·
plus the I-beam and the measure hover from the morning.

### 3.2 ⬜ `pages.merge_into` — unblocked yesterday, wire it

`EditSession::merge_document` shipped 2026-08-19: one undo entry, session
intact, incremental save, fields arriving **fillable**, collisions **renamed**
rather than refused. Verb 125, `docs/core-api/02-editing-and-saving.md`.

Two things to surface, both from the engine's own note:

- **`fields_renamed > 0` must be disclosed.** A renamed field breaks any script,
  FDF or calculation keyed on the old name, and the operator has no other way to
  learn it happened.
- **Not carried yet:** outlines, named destinations, page labels,
  `/OCProperties`. They asked which matters most for a Merge UI and guessed
  outlines. **Answer them** — it is a free choice about their schedule.

★ And their standing ask, which is worth more than a feature request: *"if you
have other buttons parked on `command-unimplemented` because an engine verb
exists but is the wrong SHAPE for an editor, those are ours. Send them."*
`SCAFFOLDED` has fourteen entries. Re-derive each one and send the shape
problems.

### 3.3 ⬜ The Format tab — the largest engine capability with no route here

`set_markup_style` takes `/C`, `/IC`, `/BS /W`, `/CA` and `/LE` on a **placed**
annotation. This shell has **zero call sites**. Both of the blockers
`manifest/format.rs` recorded are discharged — the verb landed 2026-08-18 and
annotations became selectable the same day — so what is left is building the
tab, which is work rather than a block.

`RIBBON_IA.md` §5.8 specifies twenty-four property editors and the tab currently
carries two.

### 3.4 ⬜ The disclosure gaps, in order of how badly they matter

1. **`Document::recovery()` is never called** (`NO_SURFACE.md` §3b). A document
   whose cross-reference table pdfce **rebuilt by scanning** opens with no
   indication whatsoever. `last_wins_collisions` means two definitions of one
   object existed and pdfce chose — the operator is looking at one of two
   possible documents and has not been told there was a choice. Blocked on
   nothing; the accessor and every field are `pub`.
2. **11 of the engine's 65 render counters reach anyone** (§3c). Not "add 54
   rows" — most are measurements. But `annotations_without_ap` means *a comment
   is in the file and is not being drawn*, and on a drawing somebody is
   reviewing that is worse than a colour being slightly off.
3. **The pen has no surface in the Tool panel.** `Action::SetPen` is the route;
   `panels::tool`'s header carries why a read-only swatch would be worse than
   none.

## 4. Two things that are true and surprising

### 4.1 Text on canvas EXISTS and he cannot find it

`edit.text` and `edit.add_text` are registered, on the Edit tab, bound to
`Ctrl+E` and `Ctrl+Shift+E`, and **two driven checks pass on them**
(`add_text_takes_real_keystrokes`, `text_edit_pins_an_aligned_tail`).

So this is not a missing feature. It is a **discoverability defect**, which is
this project's founding failure wearing different clothes, and I marked it
green. Do not “build text editing”.

★ **`panels::tool` shipped as the fix on 2026-08-19 (`d33d228`) and nothing has
confirmed it works.** It names both tools, in Edit, in the default arrangement,
above the fold, with their chords and their ribbon tab — and the claim that this
makes them *findable* is exactly the kind of claim this project has been wrong
about before. §3.0's first-frame check is what would settle it.

★★ And the likeliest **actual** cause is one layer down and is also unproven:
`text::textedit::refusal` writes three good sentences, has three passing tests,
and was aimed at a status row its own module says R128 forbids growing.
`Refusal::SpansRuns` is 47 words. On a dense CAD sheet the first click lands
where the operator *wants* text rather than where text *is*, so `NoRun` is the
likely first outcome — and a decline nobody can read teaches somebody the
feature does not exist. The Tool panel's third block is where those sentences
now go. **That** is the thing to drive.

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
cargo test -q -p pdfce-gui                 # 1,452 at this commit
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

⚠ **`package-portable.py` re-resolves the git dependency.** It picked the engine
up twice in one session, four commits apart each time. So the exe it publishes
may not be linked against the `Cargo.lock` in the tree — **re-run the tests and
the gates after packaging, then commit the lock**, because a lock that disagrees
with the binary the operator is running is worse than one that moved unasked.

`package-portable.py` alternates `OneDrive\pdfceGUI1` / `pdfceGUI2` itself and
preserves each slot's `userdata/`. **Say which slot in your report** — the name
carries no version. At this commit the newest is **`pdfceGUI1`**.

42 checks are declared. `page_ops_round_trip` needs
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
is one `grep`. `NO_SURFACE.md` §1c and `D:\dev
ag
ust\` carry the whole
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
