# CONTINUE — start here, then keep going

**Updated 2026-08-20, clean tree, gates 14/14, 1,523 + 384 + 144 tests,
53 driven checks — 45 verified, 6 skipped for a stated reason, 0 failed, and
★ **two written that have NEVER BEEN RUN**.**
**All six of the operator's standing complaints are closed AND driven.**
**Phase 1 is complete but for the object clipboard. Phase 5 is complete but for
live re-layout.**

---

## 0a. ★★★ WHAT LANDED ON 2026-08-20, AND THE ONE THING IT STILL OWES

The operator asked for **several PDFs open at once, and pages dragged between
them**. It is built: a document tab strip, spring-loaded tabs, a page drag that
survives the document switch, a drop caret on the page view, and a
cross-document insert that says *copy* before the button is released.

**It has not been driven.** The harness takes the pointer, the keyboard and the
display, and `feedback_ui_verify_competes_for_the_machine` says that needs a
go-ahead. So the two checks written for it — `two_documents_get_two_tabs` and
`a_page_dragged_between_documents_is_copied` — exist and have never executed.

★ **This is the exact state this project was founded to stop treating as done.**
Report it in those words. The queue:

```bash
cargo run --release -q -p ui-verify --   --exe target/release/pdfce-gui.exe   --pdf D:/Dev/temp/pdfce/SW41177.pdf   --second-pdf D:/Dev/pdfce/fixtures/synthetic/pageops/four-pages.pdf   --doc-point 0,300,500   --check two_documents_get_two_tabs   --check a_page_dragged_between_documents_is_copied
```

`--second-pdf` must be a **different file** from `--pdf`; both checks SKIP with
that sentence rather than falling back, because opening an already-open path
activates its tab by design and passing one file twice would make them assert
the opposite of what they are for.

### The map, for reading the new code

| file | subject |
|---|---|
| `app/documents.rs` | the tab arithmetic — one active `Status` plus `parked`, and why it is not `Vec<Status>` |
| `app/doctabs.rs` | the strip, and the **spring-loaded** tab that makes a cross-document drag possible |
| `egui-shell/src/tabstrip/` | the reusable strip. Knows nothing about documents; refuses to (R7) |
| `pagedrag.rs` | the drag in flight, in `egui::Memory` — because switching documents resets `PanelsState` |
| `canvas/pagedrop.rs` | the caret between two sheets on the page view, and the release |
| `app/actions/crossdoc.rs` | the only edit that reads two documents at once, and why it is a **copy** |

### ⚠ Two things this change did NOT do, and one it made worse

1. **Quitting with unsaved documents still asks nothing.** It was already
   unguarded before this; it is now unguarded across *N* documents instead of
   one. `PendingIntent::Open` / `::New` / `::NewSized` are now unconstructed —
   they are the shape a quit guard would use, and are deliberately left in
   place rather than deleted for that reason.
2. **Document tabs cannot be reordered by dragging.** Every tabbed application
   allows it and nobody asked; it is the obvious next thing an operator will
   try.
3. **A parked document keeps its page texture and its strip cache.** Deliberate
   — `BENCHMARK.md` measures 877 ms for one full-page render of the benchmark
   drawing, so dropping them would make every tab switch a visible stall. It is
   memory spent on purpose and it is unbounded; if it ever needs bounding,
   bound *how many parked documents keep rasters*, not *whether they do*.

---

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

## 1b. ★★★ READ THIS BEFORE DESIGNING ANYTHING

The most expensive finding of this project, and it arrived on 2026-08-19 after
six features had shipped that day:

> *"The selector should be predictable like other programs. It seems a lot of
> ideas are getting invented instead of just using the LLM weighting that would
> have produced the most common method expected."*

He was reporting a canvas he could not edit text on, add text on, or move points
on. **Every one of those features already worked.** What was invented was
*reaching* them — four ribbon steps to type a character, a two-double-click
descent to reach an anchor with nothing drawn saying a deeper level existed.

Each decision behind that had a written justification and most were locally
sound. **You do not catch this by reviewing the decision.** You catch it by
noticing the result resembles no program the operator has used.

So, standing, before any interaction is designed:

1. **What do Illustrator, Inkscape, Acrobat, Word and the OLD shell do?** If
   they agree, that is the answer. The convergence is the specification.
2. **Two tells that the model is wrong rather than the docs thin:** the feature
   works and nobody can find it; and *you are proud of the model*. Elegance in
   an interaction is a warning sign — its value is almost entirely in how little
   has to be learned.
3. **A usability complaint that is cheap to fix is evidence the DESIGN was
   wrong**, not the implementation. This one took hours because the machinery
   was all there.

Full write-up: `D:\dev\rag\egui\an_invented_interaction_model_is_a_defect_even_when_every_part_of_it_works.md`.

---

## 2. The operator's own list, verbatim, and what is left of it

He raised these on 2026-08-19 with *“I bring them up over and over again and
they are still not dealt with.”* He was right. **This list outranks anything
you or I think is more interesting.**

★ All six are closed **and driven** as of 2026-08-19. He raised **nine** more
that day and all nine are closed: the three below, plus *"can't select and edit
end points"*, *"can't type when I click text"*, *"can't make new text where I
click"*, *"standard copy/paste and cut aren't implemented"*, *"can't drag and
drop a jpg"*, and *"the insert image button doesn't insert it either"* — the last
of which was **false**, and is the finding in §1b's companion RAG entry: the
button worked, and the silent drop failure beside it took its credibility. He
raised three earlier the same day — *“text editing on canvas still doesn't work”*, *“no
cloud revision tool either”*, *“increase cache to maximum for page view”* — and
all three are closed too. The first was a **real defect the driven checks
missed** because they drove fixtures this repository generates; the second was
already built and he was on an older build; the third was a cache that held
only the visible page set, which is a frame buffer with extra steps. §7.

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

### 3.0 The verification debt is PAID. Do not re-open it.

51 driven checks; 45 verified and 6 skipped on the last full run. Run it before
believing anything:

```bash
cargo update -p pdfce-core -p pdfce-render -p pdfce-print
cargo build --release -q
cargo run --release -q -p ui-verify -- \
  --exe target/release/pdfce-gui.exe \
  --pdf D:/Dev/temp/pdfce/SW41177.pdf --doc-point 0,300,500
```

Two checks want the **node fixture** rather than the real drawing, and skip
honestly on `SW41177.pdf` because its paths are two-anchor lines:

```bash
python tools/gen-node-fixture.py     # if fixtures/polyline-nodes.pdf is absent
cargo run --release -q -p ui-verify -- \
  --exe target/release/pdfce-gui.exe \
  --pdf fixtures/polyline-nodes.pdf --doc-point 0,200,320 \
  --check multi_node_move_moves_every_picked_anchor \
  --check bezier_handle_drag_changes_a_curve
```

★ **`PDFCE_UIV_IMAGE` points the two image checks at a real file** instead of a
PNG the harness encodes itself. Use it — that seam exists because
`insert_image_places_a_picture` passed for weeks on harness-authored PNGs while
the operator was reporting **jpg**. Same fixture trap that hid the text-editing
defect:

```bash
PDFCE_UIV_IMAGE=D:/Dev/temp/pdfce/fixture.jpg cargo run --release -q -p ui-verify -- \
  --exe target/release/pdfce-gui.exe --pdf D:/Dev/temp/pdfce/SW41177.pdf \
  --check insert_image_places_a_picture \
  --check a_dropped_image_reaches_the_placement_window
```

**One phase is a permanent SKIP with a written reason**:
`dimension_groups_panel_makes_a_group`'s folds. Three harness fixes did not make
them work and each would have been a wrong report about the program. The account
is in the function. Do not "fix" it with a retry loop.

### 3.1 🔨 The object clipboard — half built, and the other half is blocked

Cut, copy and paste ship for **markup and comments** (`spec_from_dict` out,
`add_markup` back). **Page content cannot be pasted** — 157 verbs in `edit.rs`
and none inserts any, re-measured 2026-08-19.

**Do not file a request for the content half yet.** Nothing is measured about
how the operator wants it to behave and the request would be inventing the
requirement. The note in the channel says exactly that, so the engine is not
waiting.

★ Worth re-reading before adding anything: `add_image` **exists**, so pasting an
*image* may be expressible — what is missing is an accessor that reads one back
out of a page. That is a small, well-shaped engine ask if the operator ever
wants it.

### 3.2 ⬜ `pages.merge_into` — unblocked, still unwired

`EditSession::merge_document`: one undo entry, session intact, fields arriving
**fillable**, collisions **renamed** rather than refused. Pass 106.1 (engine
commit `af12b31`) added the **navigation carry — destinations first**, so the
"not carried yet" list has shrunk; re-read `2026-08-19-merge-that-keeps-your-undo-log.md`
and `464e306` before writing the disclosure sentence.

**`fields_renamed > 0` must be disclosed.** A renamed field breaks any script,
FDF or calculation keyed on the old name, and there is no other way to learn it.

### 3.3 ⬜ Re-derive the fourteen `SCAFFOLDED` entries

The engine's standing ask, worth more than a feature request: *"if you have
buttons parked on `command-unimplemented` because a verb exists but is the
wrong SHAPE for an editor, those are ours. Send them."*

★★ **And do it with `D:\dev\rag\rust\a_missing_verb_is_often_an_existing_verb_you_did_not_decompose_the_operation_into.md`
open.** Three features shipped on 2026-08-19 whose recorded blockers were
**never true** — resize ("no scale verb": true, and irrelevant, because
`move_nodes` takes per-node deltas and a scale is a list of them), multi-run
text editing (the guard was refusing a shared *line*, which `FollowerDisposition::Pin`
had always handled), and Bézier handles (`move_handle` had shipped in Pass
30.1). Each blocker was produced by grepping the API for a verb whose **name**
matched the operation. Read the whole verb list and the **enums**; both misses
lived in parameter types.

### 3.4 ⬜ The Format tab — still the largest capability with no route here

`RIBBON_IA.md` §5.8 specifies twenty-four property editors; the tab carries
two. Both recorded blockers are discharged.

★ Note that **markup restyle landed in the Properties panel instead**
(2026-08-19) — colour, line width, opacity through `set_markup_style`. That was
a considered placement, not a shortcut: Properties already appears on a
selection and already reads the annotation. Decide deliberately whether the
Format tab duplicates it, replaces it, or takes only the rows Properties
cannot carry.

### 3.5 ⬜ The disclosure gaps, in order of how badly they matter

1. **`Document::recovery()` is never called** (`NO_SURFACE.md` §3b). A document
   whose cross-reference table pdfce **rebuilt by scanning** opens with no
   indication. `last_wins_collisions` means two definitions of one object
   existed and pdfce chose — the operator is looking at one of two possible
   documents and has not been told there was a choice. Blocked on nothing.
2. **11 of the engine's 65 render counters reach anyone** (§3c).
   `annotations_without_ap` means *a comment is in the file and is not being
   drawn*, which on a drawing under review is worse than a wrong colour.
3. **The markup pen has no surface in the Tool panel.** Note the text pen
   solved the same problem a different way — `canvas::textedit::pen` lives in
   `egui::Memory`, so a panel reaches it through `ui.ctx()` with no plumbing,
   which is exactly what `panels::tool`'s header says the markup pen cannot do
   because it is a field on `PdfceApp`. **Move it, do not plumb it.**

## 4. Two things that are true and surprising

### 4.1 ★★★ "It exists and he cannot find it" was the WRONG diagnosis, twice

This section used to read: *"`edit.text` and `edit.add_text` are registered, on
the Edit tab, bound to `Ctrl+E`, and two driven checks pass on them. So this is
not a missing feature. It is a **discoverability defect** … Do not 'build text
editing'."*

Every fact in that was true. **The conclusion was wrong**, and it was wrong in a
way worth keeping the paragraph to demonstrate.

Calling it a *discoverability* defect frames the remedy as **telling the
operator where the feature is** — which is what the Tool panel did, on
2026-08-19, listing both text tools with their chords and their ribbon tab. He
came back the same day and said he still could not type.

The actual defect was that reaching the feature took **four steps** (Edit mode →
Edit tab → *Edit text* → click) and that the I-beam he saw belonged to a
*different* tool that sweeps text. No amount of signposting fixes a
four-step-ritual; **the ritual is the defect**. The remedy was to delete it: one
text tool, `T`, click to edit, click empty space to start new text.

Two lessons, and the second is the reusable one:

1. **"It works and nobody can find it" is never a documentation problem.** If a
   panel has to explain how to reach something, the route is wrong.
2. **A diagnosis that names a category also names a class of remedy.** Calling
   this "discoverability" made a *labelling* fix feel like the answer for three
   weeks. Naming it "the route is four steps and no other program's is" would
   have produced the right fix on day one.

Both are in `D:\dev\rag\egui\an_invented_interaction_model_is_a_defect_even_when_every_part_of_it_works.md`,
and §1b is the short form.

★ The related half that **was** real and is now fixed: `text::textedit::refusal`
wrote good sentences aimed at a status row `R128` forbids growing, and
`SpansRuns` — 47 words — fired on nearly every click on a CAD title block,
because the guard was refusing a shared *line* rather than a genuine multi-run
edit. That refusal is gone (the neighbours are pinned instead) and the remaining
sentences have the Tool panel's third block to live in.

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

★★ **The evening of the same day did it again, in the other direction and
three times over** — and the whole point is that these were harness faults
pointing at working code:

| the check said | the truth |
|---|---|
| “the width was scrubbed and Apply committed nothing” | the trace showed the object's bounds going 317.87 → 358.00 eleven lines later. The oracle was `resize-commit`, which only the *gesture* route writes |
| “the Node rung was never entered” | `double_click_at` was two calls to a settling `click_at`, putting **390 ms** between the presses — past `egui`'s compiled-in 300 ms threshold |
| “1 anchor selected” after a Shift-pick | descending re-scopes the anchor marks; the trace shows a published rect moving x=393.9 → x=336.4 **because of the harness's own double click**, so the second aim was at a place the anchor had left |

Out of which: **a harness may hold a coordinate for exactly as long as it
performs no act that could move it** — three distinct causes in one day, all
producing confident wrong reports. And: **do not compute a coordinate the
application could publish**; a harness that derives a widget position by
arithmetic can be wrong in the same direction as the code under test. Both are
in `D:\dev\rag\egui\`.

Two rules fell out of the morning and both are now in the RAG:

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
