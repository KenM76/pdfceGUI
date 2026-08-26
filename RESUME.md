# RESUME — read this, then say "continue"

> ★★ **If the operator typed "continue" and nothing else, read**
> **[`CONTINUE.md`](CONTINUE.md) first.** It is the short path: what is next,
> in order, and the two facts that are true and surprising. This file is the
> long state document it points into.



**Written 2026-08-18, last revised 2026-08-25 at `496d1db`.** For a session
starting cold on `D:\Dev\pdfceGUI`.

This file is the **entry point**. `HANDOFF.md` is the long-form institutional
record and is still authoritative for the standing rules, the phase order and
the accumulated findings — read it after this one, or when this one points you
at a section of it.

---

## ★★★ State, as measured at `496d1db` (2026-08-25)

**This table is a reading, not a status.** Every row is what a command printed
at that commit; the tree has moved since, and the numbers move with it. It is
here so you know roughly where you are, not so you can quote it.

| | |
|---|---|
| **Tests** | 1,707 (`pdfce-gui`) + 397 (`egui-shell`) + 144 (`ui-verify`), 0 failing |
| **Gates** | **17 of 17**, 0 skipped |
| **`ui-verify`** | **77 checks declared.** Last full driven run: **66 passed · 3 failed · 8 skipped** — see `evidence/ui-verify-run.txt`, committed deliberately because it is the reason a release decision was made |
| **The three failures** | `zooming_does_not_throw_away_where_the_operator_panned` and `zooming_back_out_keeps_the_view` are both **O27**, the f32 scroll-tier residual above ~130,000 %, deliberately left RED. `multi_node_move_moves_every_picked_anchor` is pre-existing and filed. **None is new and none blocks a release** |
| **Panels** | **12.** Pages · Bookmarks · Layers · Signatures · Fonts · Objects · Properties · Forms · Comments · Redact · Dimension groups · Tool |
| **Engine** | `D:\Dev\pdfce` local `main`, taken as a **git** dependency, pinned at `81e5aab`. **Read `Cargo.lock`, not this row** — it moves nearly every session, and `package-portable.py` re-resolves it unless you pass `--no-update` |
| **Latest build** | `OneDrive\pdfceGUI1`, published 2026-08-25 from shell `db88cc8` / engine `81e5aab`, **with `--verify`**. `pdfceGUI2` holds the previous build as the fallback. Open **About** and read the Build block rather than trusting this row |

### ⚠ ON THIS PC, pdfce FAILS TO START ABOUT ONE LAUNCH IN THREE

**It is the machine, not the program** — settled 2026-08-26 by the operator
testing the identical portable build on his laptop, where it is fine. Do not
diagnose it again.

The symptom is a panic before any window appears, from `accesskit_windows`,
reporting `HRESULT 0x80070008 "Not enough memory resources"` on a machine with
plenty of memory.

★ **What it costs you:** `ui-verify` launches a fresh process per check, so on
this PC roughly a third of the suite cannot start and reports SKIPPED. Those are
environmental, not product defects — read the skip reason before chasing one. A
run on this machine is therefore always partial, and reporting it as a pass would
be false.

★★ A red herring on the way: OneDrive was found holding 404,000 handles, and
publishing builds was measurably feeding it (~27,000 per build, established with
a do-nothing control). Restarting OneDrive dropped it to 1,179 — **and the crash
rate did not change.** Real leak, real fix, wrong mechanism.

### ★★ Two things that will otherwise cost you an hour each

**1. Always publish with `--verify`.** The 2026-08-24 23:04 publish omitted it,
so its tests and gates never ran, and the engine's new `page_blend_space_source`
setting arrived as a surprise instead of as a packaging failure. Pass
`--no-update --verify` when the build should be the one you already verified —
plain `--verify` re-resolves the engine first and can move it under you.

**2. When a driven check says it could not raise the window, read WHICH window
is holding the foreground** — the harness now prints its class, title and pid.
On 2026-08-25 nine checks skipped with the "no foreground rights" message and
the cause was a stray `OpenWith.exe` dialog on the desktop: not the harness and
not the application. `D:/dev/rag/egui/` carries the finding.

### ★★★ READ THIS FIRST: R1's debt was paid on 2026-08-19, and re-incurred the same day

The morning of 2026-08-19 paid it. The operator handed over the machine, all 38
checks then declared were driven, and **four application defects came out that
1,417 passing tests could not see** — the table further down keeps them, because
their shapes recur.

**Then he took the machine back, and six more features shipped without being
driven at all.**

| shipped 2026-08-19 afternoon | driven? |
|---|---|
| `panels::dimension_groups` — the window became a **dock panel**, six folds | ✗ |
| `MarkupKind::Cloud` — the revision cloud, its glyph, its ribbon row | ✗ |
| `panels::tool` — the Tool panel, in its own dock stack in two modes | ✗ |
| `dialogs::unsaved` — the close/open/new confirmation | ✗ |
| the settings-coverage gate, and `quad_point_order`'s control | ✗ |
| the two-tone I-beam and the pre-first-click measure hover *(morning, before handover)* | ✗ |

That is the largest verification debt this project has carried, and it is on
exactly the class of change R1 exists for: **two new panels, a new dialog on the
close path, a new markup kind, and a new ribbon row.** `CONTINUE.md` §3.0 is the
queue and it outranks everything else in this file.

★ **Three checks were rewritten and never run**, and the Tool panel took the top
stack in the right dock of Review and Edit — so **every other panel's coordinates
moved**. A red from any right-dock check is more likely to be that than a defect.

★★ **And the check the day most needs does not exist.** The Tool panel was built
to make `edit.text` and `edit.add_text` findable, and *"it renders"* is not that
claim — asserting it renders would repeat the original failure exactly, because
the commands rendered on the ribbon all along. The honest check is a **first
frame with zero clicks**: launch, open the fixture, enter Edit, screenshot, and
assert both labels are on screen inside the panel's rect.

### The four defects driving found on 2026-08-19, kept because their shapes recur

| defect | how it presented |
|---|---|
| **A `Window` with a `default_width` and no HEIGHT around a `ScrollArea` grows ~38 pt every frame** | two dialogs walked off the screen; the Manage-groups Add button was laid out at y=1114 in a body ending at y=676 — drawn, positioned, unclickable. ★ **The panel move retired the condition rather than tuning it**: a dock panel's height is the dock's, decided before the body draws |
| **The Bookmarks authoring row sat after an unbounded scroll** | `add_outline_item`, wired that morning, was **unreachable on any document with a real outline** — the 122-bookmark fixture pushed the row 129 pt below the panel |
| **A region published at the TOP of a `ScrollArea` closure over `ui.min_rect()`** | reported `0.0 pt high` for ever — an instrument that can only return one answer cannot detect what it was added to detect |
| **The Manage-groups Add button was below its own settings block** | it acts on the LIST and was positioned under a different group's settings, so it made the wrong claim about what it acts on even when visible. ★ The panel fixed the **claim**, not just the reach: Add now sits directly under the list |

★ And **three harness defects that produced confident, wrong failure reports
about working code** — see `D:/dev/rag/egui/a_ui_rect_change_log_produces_confident_wrong_failures_in_BOTH_directions.md`.
The drop caret was reported *"never published"* over a trace containing it.
**Read the trace before believing the check.**

⚠ **The suite is not deterministic.** The final full run had 35 passed · 1
failed · 4 skipped; all three of the non-passes then passed in isolation, with
messages pointing at pointer injection and window activation rather than at the
application. Per-check runs are authoritative; a full-suite red needs the member
re-run before it is believed.

Two skips are legitimate and not defects: `ocr` (no models in this build) and
`page_ops` on `SW41177.pdf` (the fixture carries 36 `/Rotate` entries, so the
evidence would be indistinguishable from its own furniture — it PASSES against
`D:\Dev\pdfce\fixtures\synthetic\pageops\four-pages.pdf`).

### ★★ Two defects found by AUDIT rather than by driving, and both were worse

Answering `pdfce`'s capability-register questions found two things no test and
no driven run would have reached, because neither has a symptom on screen.

| | |
|---|---|
| **Close destroyed unsaved edits, silently** | `file.close`'s tooltip promised *"You are asked what to do about unsaved edits first"* since the day it shipped, and nothing asked. Open and New too. ★ Why it survived: **the guard that should have caught it existed, was well argued, was correct, and was answering a different question** — `save_pending` asks *is a save in flight*, which is permanently `false` here by design. Fixed |
| **`Document::recovery()` is never called** | a document whose cross-reference table pdfce **rebuilt by scanning** opens with no indication at all. `last_wins_collisions` means two definitions of one object existed and pdfce chose between them: the operator is looking at one of two possible documents and has not been told there was a choice. Blocked on nothing. **Still open** — `NO_SURFACE.md` §3b |

The transferable half: **driving finds what an operator can see. It cannot find
a promise nobody kept, or a report nobody rendered.** Both of these were sitting
in plain source, and what surfaced them was another project reading our
documentation and asking a question about it.

### ★★★ And four recorded claims turned out to be false, in one day

| the claim | the truth | cost |
|---|---|---|
| `markup.cloud` — *"the ONLY markup kind still absent for an ENGINE reason"* | `MarkupSpec::Cloud` had shipped | the operator asked **three times** over ~3 weeks |
| `NO_SURFACE.md` — *"Opacity: blocked on the engine, `/CA` is never written"* | `set_markup_style` writes it, tests both ways | a capability inventory reporting a false blocker |
| `FEATURES.md` — *"the theme preset is not yet choosable"* | choosable since 2026-08-17 | found by the **other project** reading our file |
| the shared `gui` column | read two incompatible ways for weeks, neither side seeing a contradiction | seven rows re-based down |

> **A blocker that names a repository this project does not build cannot fail a
> test.** Nothing compiles differently, nothing lints, and CI stays green
> *precisely because the feature is still absent*.

Every external blocker here is now a **dated citation** rather than a verdict.
`NO_SURFACE.md` §1c and `D:\dev\rag\rust\` carry the argument. This is the
single most expensive pattern this project has found, measured in weeks of the
operator asking for something that was not blocked.

## ★★ The harness — last run 2026-08-18, and what it found

`ui-verify` drives the real cursor and keyboard, so it may not run while the
operator is at the machine. Last run 2026-08-18 with the operator off the PC:
**28 passed, 1 failed, 3 skipped.**

★★ **It CAN type, and for months this project believed it could not.** Three of
the four new checks press keys: `add_text_takes_real_keystrokes`,
`text_annot_takes_the_keyboard_unclicked`, `every_declared_chord_dispatches`.
The fourth is `about_reports_the_build`, the first driven check of the About
window — which had no declared region, so nothing could find it, while it
carries the third-party attributions and their legal obligation.

See the founding-rule section near the end of this file. The false belief is
the single most expensive thing recorded in this repository so far, and the
shape of how it survived matters more than the fix.

### ★ The two skips, and why neither is a gap in the application

- `page_ops_round_trip` — the fixture already carries 36 `/Rotate` entries, so
  the check's evidence (find `/Rotate 90` in the saved copy and not in the
  source) would be ambiguous on this document. Wants a fixture with none.
- `ocr_recognises_a_page_and_writes_a_new_file` — needs a model present.

### ★★ Two checks were reporting FALSE failures, and both were believed

`print_dialog_reaches_the_spooler` stood at FAIL and `print_paper_changes_the_plan`
at SKIP, both saying the File tab declares no `ribbon.item.file.print`. True,
and false: at the harness's 1100 pt window the ribbon had correctly folded
Print into the **overflow**. It was written up as a harness gap and left.

`driving::declared_or_in_overflow` looks in both places, and both checks pass.

**That is the second false-failure-believed of the same day** — the other being
"this machine cannot send synthetic keys", which cost the keyboard. A harness
that cries wolf gets believed the way any worn-out alarm does, and the cost is
not the noise: it is that the next real failure reads as more noise. **When a
check reports something absent, ask what else could make it absent before
writing the limitation down.**

```bash
cargo run --release -q -p ui-verify -- --exe target/release/pdfce-gui.exe \
  --pdf D:/Dev/temp/pdfce/SW41177.pdf --doc-point 0,300,500 > evidence/ui-verify-run.txt 2>&1
```

★ **Redirect to a file.** The first attempt piped through `tail`, which threw
away the failure detail and cost a second run of the operator's window. The
second run was skipped wholesale as STALE because a source file was edited
while it was in flight — **finish the edits, rebuild, then run.**

### The one failure, and it is the harness rather than the application

`print_dialog_reaches_the_spooler` reported that the File tab declares no
`ribbon.item.file.print`. It does not — at **1100 pt**, the harness's window
width, the ribbon has correctly folded the **Print, Document and pdfce groups
into the overflow**, and `ribbon.overflow` is declared in the same frame.

Two things are true and both matter:

1. **The check has a gap.** `settings_headings_legible` meets the same
   condition in the same run and handles it — *"the Settings control is not on
   the ribbon band at this window width … Opening the overflow to reach it."*
   The print check needs the same step. `print_paper_changes_the_plan` skips
   for the same reason and defers to it, correctly.
2. **`file.new_from_template` made the File band one item wider** on
   2026-08-18, which is what tipped it over at that width. On a wide window
   Print is still on the band; it was not verified at the operator's own window
   size.

**Fix the check first**, and only then decide whether the size chooser has
earned a place on the band — that is an IA question for the operator, not a
build session's.

### The three skips are honest and named

| check | why |
|---|---|
| `page_ops_round_trip` | the fixture already carries 36 `/Rotate` entries, so the evidence would be indistinguishable from the document's own furniture. Point `--pdf` at `D:\Dev\pdfce\fixtures\pageops\four-pages.pdf` |
| `ocr_recognises_a_page_and_writes_a_new_file` | needs the `models/ocrs` weights beside the exe, i.e. a **packaged** build |
| `print_paper_changes_the_plan` | ★ FIXED — both now look in the ribbon overflow |

### Still not written

★★ **A first-frame discoverability check, and it is the most important missing
one in the project.** Launch, open the fixture, enter Edit, screenshot with
**zero clicks**, assert the strings `Add text` and `Edit text` are on screen and
inside the Tool panel's rect. The panel was built because the operator could not
find two commands that were on the ribbon all along, so a check asserting the
panel *renders* would repeat the original failure exactly — the commands
rendered too. Pair it with: arm `edit.text`, click blank paper, assert
`Refusal::NoRun`'s sentence is on screen in the panel. That second one **fails
today** for want of anywhere to put the sentence, which is the point.

An **annotation-selection** check — click a stamp, assert `annot-select`, press
Delete, assert one fewer annotation. Every trace line it needs already exists,
and Delete can now be pressed, because the keyboard works.

An **unsaved-edits** check — make an edit, press Close, assert the confirmation
appeared **and that the document is still open**; then press *Close without
saving* and assert it is not. The second half is the one that matters: a
confirmation that appears and is ignored is worse than none, and until
2026-08-19 that path destroyed the operator's work silently while its own
tooltip promised otherwise.

★ **A check that types for real is DONE** — three of them, in fact
(`add_text_takes_real_keystrokes`, `text_annot_takes_the_keyboard_unclicked`,
`every_declared_chord_dispatches`). The text-EDITING check still seeds its
draft through `PDFCE_DIAG_TYPE`, which bypasses the event loop; that seam is
now a convenience rather than a workaround, and the link it skips is covered
by `add_text`.

**Re-measure before you rely on any of it.** Prose drifting from a number is a
defect this project has spent seven corrections on — the gate runner's own
header spent months saying "Three gates carry one" while four ran, and the
README claimed 1,530 tests against an actual 1,839. Both were fixed by
deleting the count, not by updating it. Do the same here if you find yourself
tempted to edit a number rather than re-run the command.

```bash
git log --oneline -1
cargo test --workspace
bash tools/gates/run-all.sh
cargo run --release -q -p ui-verify -- --exe target/release/pdfce-gui.exe \
  --pdf D:/Dev/temp/pdfce/SW41177.pdf --doc-point 0,300,500
python tools/package-portable.py --verify --note "what this milestone added"
```

The `ui-verify` skips are honest and named: `page_ops` wants a fixture with no
`/Rotate` furniture, and `ocr` wants the model weights beside the binary.
`markup_style` is intermittent — it skipped in the run before last and passed
in the last one, so treat a third skip as scheduling, not as a regression.

---

## What to do next, in the operator's likely order

> ### ★★★ Read this before picking anything: the order came from the wrong place
>
> On 2026-08-18 the operator said, of a day and a half of work:
>
> > *"I'm still flabbergasted by how the GUI is still not user friendly. …It
> > feels like nothing is moving forward on these things. …Hours and hours,
> > and I click and can't figure out how to enable some of the basic stuff."*
>
> He was right, and the cause was **scheduling, not difficulty**. This list
> used to be ordered by *what the engine most recently unblocked* — which is
> how print paper sizes and a page-size chooser got built while
> **clicking a stamp did nothing at all**. Both were things he had asked for;
> neither was on the path to what he was actually hitting.
>
> **Order this list by what the operator reaches for, not by what just
> arrived in the channel.** The engine's replies are an input to *how* a thing
> gets built, never to *which* thing.

### ★★★ First: DRIVE. `CONTINUE.md` §3.0, and it outranks everything below.

**Six features are shipped and undriven** — two panels, a dialog on the close
path, a markup kind, a ribbon row, and two cursor fixes. Three checks were
rewritten and never run. The Tool panel took the top stack in the right dock of
two modes, so every other right-dock check's coordinates moved.

It needs nothing but the desktop, and the operator off it.

### 1. The Format tab's first slice — restyle a selected annotation

**Selection landed 2026-08-18** (`00ff4c7`): a stamp, note, shape or ce
dimension can be clicked, is outlined, and can be **deleted**. What is still
missing is everything that made the operator ask *"how do I **edit** a stamp"*:
colour, width, opacity.

`EditSession::set_markup_style` is the verb and it is shipped. The routing is
already done for you — `AnnotKind` on the selected target is `Markup` or
`CeDimension`, and the second **must** go to `set_dimension_style` instead.

> ★★ A ce dimension is a `/Line` with `/IT /LineDimension`. It passes every
> "markup pdfce can author" test, and restyling one through `set_markup_style`
> regenerates it as a **bare line — label and witness lines gone** — from an
> operator who asked only to recolour it. The engine refuses by name; the kind
> on the target is what stops the refusal being reached.

Deliberately **not** in that verb, so the tab must not offer them: note text,
**move and resize**, and `/LE` on PolyLine. Move/resize is the next ask after
this one and needs its own engine request.

### 2. Dimension select-and-drag — a REGRESSION, not an unbuilt feature

The **old** GUI does this: `run_dimension_drag` at
`D:\Dev\pdfce\crates\pdfce-gui\src\main.rs:22782`, with
`doc.selected_dimension`, `doc.dimension_drag`, and `dimension_rects`
hit-tested per page. The new shell never called `dimension_rects` at all.

Selection now covers *clicking* one. **Dragging it is still gone**, and R6 says
nothing regresses. The old code exists and is salvageable; 18 references to
`selected_dimension` in that file are the whole feature.

### 3. One open operator report — the other was found and fixed

**★ FIXED, but NOT DRIVEN: "add text types nothing."** The dialog latched on
having **asked** for focus rather than on holding it, so a request that lost
its opening frame was never retried and the field swallowed every keystroke
while looking exactly like a focused one. Losing that frame is the normal
case, not an edge case: the dialog's first draw is the frame *after* the
gesture that opened it, so the pointer release is still being resolved around
the request, and egui keeps the earlier of two requests in one pass. Fixed at
`1b4949f` with a bounded retry and a regression test that was run **both**
ways — it fails on the old implementation and passes on the new.

It has not been confirmed against the operator's own report. **That is the
one thing outstanding on this item**, and it needs the machine.

Two things this cost, both worth knowing before writing the next test of a
window: `RawInput::default()` has no `screen_rect`, so a dialog that sizes
itself from the screen lays out unlike the application; and it has no `time`,
which egui then fills from the **wall clock**, so a multi-frame test flakes
under load. That flake read as "test interference" and sent me looking for a
polluting sibling test that does not exist. Both are in
`D:/dev/rag/egui/rawinput_default_has_no_screen_rect_and_no_time_...md`.

**No context-sensitive panel, and no tool indicator.** *"When I click to use a
tool I have no indicator to tell me what to do next or what tool I am even
using."* Verified: the status bar carries page, find, fit, zoom and
disclosures, and **nothing names the armed tool**. Read `MODES_AND_PANELS.md`
before designing — the flexible panel system is specified there and this is not
a thing to improvise.

### 4. "Highlight fillable fields" — the smallest real win on this list

Form filling **works**, in every mode including Read: `canvas::forms` never
consults the mode. What is missing is that **nothing shows where the fields
are** — Acrobat tints them blue, pdfce paints nothing and only changes the
cursor to an I-beam. That is the whole of *"How do I click on a form to edit
it in the Canvas?"*

`canvas/forms.rs`'s own header names it and declines to build it:

> *"…a weaker one than Acrobat's blue field tint. The honest remedy is a
> **"highlight fillable fields" toggle**, which is a ribbon command; this
> module deliberately adds none and the entry point is reported rather than
> wired."*

It is a view overlay like rulers, the grid and find hits — **not** content
marking, so rule 4 permits it. One command, one condition, one overlay pass.

### 5. Resize an EXISTING page

`set_media_boxes(indices, rect)` shipped with `set_media_box` and only the
second is used. Belongs in Document ▸ Properties, and is a **design** question
before a coding one: does content move, does `/CropBox` follow, is shrinking
below the content a refusal.

> Read `archive/2026-08-18-mediabox-and-markup-reply.md` first. `/MediaBox` is
> inheritable (§7.7.3.4), so the write is three-way, and *"a target equal to
> the inherited value REMOVES the page's own entry"* is load-bearing and
> invisible to a one-page fixture — **writing to the ancestor that supplies
> the value resizes every sibling.**

### ~~Not ours: revision clouds~~ — ★★ this heading cost three weeks

**Struck 2026-08-19. It shipped that day, in about an hour.**

It read: *"Confirmed moving upstream on 2026-08-18 — `EditError::TooFewVertices`
and a `Cloud` subtype are in `D:\Dev\pdfce`'s working tree. The operator:
'don't worry about item 5. It's aware of that one now.'"*

Every word of that was true, and the **heading** was wrong. He meant the
*engine* was aware. This file turned that into *"not ours"*, filed it under a
heading a reader takes as a scheduling decision, and the operator went on asking
for the revision cloud tool while the only thing blocking it — a `MarkupSpec`
variant — sat shipped in a repository one `grep` away.

Kept, struck, because the mis-reading is the finding:

> **"The engine is aware" is not "this is not ours."** An upstream repository
> acknowledging a gap says nothing about whose work the *surface* is, and a
> heading that says otherwise stops anyone re-checking.

And the deeper one, which now governs every blocker in this project: a claim
about a repository you do not build **cannot fail a test**, so it goes on being
read as current until somebody happens to look. `NO_SURFACE.md` §1c.

## What NOT to do

- **Do not start Phase 5 (text editing) early.** It is deliberately last —
  `HANDOFF.md` §8. It is the defect that began this project, and every earlier
  version of that file treated it as the obvious next move. It is not.
- **Do not build S6 deep zoom or tiling.** Measured as a 9× regression.
- **Do not write to `D:\Dev\pdfce`.** Read-only to this project. Engine work
  goes through `D:\Dev\FeatureRequests\pdfce_FeatureRequests\` and lands there
  as its own Pass. That channel answered seven requests in a day — it works.
- **Do not run `ui-verify` without the operator's go-ahead** if they are using
  the machine. It drives the real cursor and keyboard.

---

## Standing operator instructions set in this session

1. **Always `cargo update -p pdfce-core -p pdfce-render -p pdfce-print` before
   building.** Automated as a build step in `package-portable.py`; `--no-update`
   exists for reproducing an exact revision. The engine repo moved 8, then 12,
   then 4, then 6 commits ahead inside one afternoon, and a stale pin already
   cost eighteen missing images on the operator's own file.
2. ★★ **Publish EVERY build worth keeping to OneDrive**, alternating
   `pdfceGUI1` / `pdfceGUI2`, newest replacing the older slot. Restated as a
   standing rule by the operator on 2026-08-19.

   ```bash
   python tools/package-portable.py     # updates the engine, builds, mirrors, rotates
   ```

   The alternation is a property of the tool, not something to track: it picks
   the older slot itself and preserves that slot's `userdata/`, because the
   operator runs the exe straight out of OneDrive on this machine and others.

   The obligation is the part that is not automated. **A build that exists only
   in `target/release/` or `D:uilds\` has not reached the operator.** Run it
   at the end of any session that landed working changes, and immediately after
   any fix he might want to try — without asking, since it writes only to
   `D:uilds\` and the OneDrive slot and never to a repository.

   And **say which slot in the report**, together with which one holds the
   previous build. The slot name carries no version information, so "packaged"
   on its own leaves him opening folders to find out which is which. The reason
   there are two slots at all is the project's own fallback property applied to
   the day-to-day: the previous build stays intact beside the new one, to fall
   back to and to compare against.
3. **Put engine work through the channel**, and the other session picks it up in
   parallel.

### ★ When the engine repo is busy, the packager races itself

`package-portable.py` updates the engine as its **first** step, so on a day the
other session is committing live, every packaging run moves the pin, dirties
the tree, and stamps the artefact `-dirty`. Two consecutive runs on 2026-08-18
produced two dirty builds for a reason that had nothing to do with either of
them — the engine moved 11 commits, then 1, then 1 again, inside twenty
minutes.

The sequence that works, and what `--no-update` is actually for:

```bash
cargo update -p pdfce-core -p pdfce-render -p pdfce-print
git commit Cargo.lock -m "Take the engine to <rev>"
python tools/package-portable.py --no-update --verify --note "…"
```

Standing instruction 1 is still honoured — the engine IS updated immediately
before the build. What is skipped is a **second** update racing the build it is
meant to precede. On a quiet day the default path is fine and this never comes
up.

---

## ★★ What the last two sessions found — the part worth carrying

### ★★★ From 2026-08-18 (latest): fourteen shortcuts had never worked

Found while investigating "add text types nothing", which it does **not**
explain. `app::keyboard::commands` matched the frame's keypress against
`DERIVED`, a hand-written table of eight chord spellings, and refused outright
any chord holding Shift or Alt. The manifest binds twenty-one. So fourteen
bindings were declared in `built_in.ron`, printed in menus and tooltips as
shortcuts, and delivered nothing:

```
Ctrl+Z  undo          Ctrl+S        save a copy      F11   fullscreen
Ctrl+Y  redo          Ctrl+E        edit text        [ ]   rotate
Ctrl+Shift+Z redo     Ctrl+Shift+E  add text         Alt+Up/Down  move page
Ctrl+H  read mode     Ctrl+Shift+C  copy page text   Ctrl+Alt+N   from template
```

**Undo had a keyboard shortcut everywhere except the keyboard.** This is very
likely a large part of *"I click and can't figure out how to enable some of the
basic stuff"* — and it is why the operator has been reaching for the ribbon for
everything.

Three things to carry:

1. **A table kept in step with a manifest by hand falls out of step with it**,
   and the failure is silent from both ends: the entry looks bound, the hint
   looks true, the key does nothing. The table is gone; `parse_chord` reads the
   manifest through `egui::Key::from_name`.
2. **`Modifiers::matches_logically` is permissive** — it asks whether the
   pattern's modifiers are *present*, not whether the extras are *absent*, so
   `Ctrl+Shift+Z` satisfies `Ctrl+Z`. Bound to redo and undo, that makes one
   keypress mean two opposite things with iteration order deciding. Compare the
   three flags exactly. Refusing the extra modifiers outright was the old
   code's answer to the same hazard, and it is what killed `Ctrl+Shift+E`.
3. **★ The meta-lesson, and the one to re-read before writing any gate.** A
   gate *did* exist. Its doc comment stated the general rule — *"a chord this
   module cannot see would then be a keymap entry, a menu hint and a tooltip
   promising something no keypress delivers"* — and its body then said
   `if !is_digit_chord { continue; }`, sweeping seven of twenty-one. `Ctrl+O`
   had already been found dead once and was fixed by adding one row: the
   instance closed, the class left open. **When a gate's prose is general and
   its body has a `continue`, the `continue` is the bug.**

Its replacement presses every chord and asserts the command comes back, which
is a stronger claim than spellability — a spelling test passes on a dispatcher
that spells a chord correctly and then filters it out for holding Shift.

**Not driven.** All four new tests are headless. Pressing Ctrl+Z on a real
document is a thirty-second confirmation and has not been done.

### From 2026-08-18 (earlier the same day): two features, and three drifted claims

**What shipped.** The print dialog grew a **paper list**, the driver's own
**Properties…** button and a restored **tray** checkbox; `file.new_from_template`
grew a **page-size chooser**. Both were engine gaps that had been filed and
answered, and in both cases the shell half was smaller than the reasoning
around it.

**The finding worth carrying is about DISCLOSURES THAT EXPIRE.** Three separate
true-when-written sentences were false by the time they were read:

| the claim | why it expired |
|---|---|
| *"Paper comes from this printer's settings. **pdfce cannot change it.**"* | shipped copy, correct for months, falsified by the control added three lines above it |
| `app::blank` §3a *"the size picker is BLOCKED on the engine"* | correct on 2026-08-17, unblocked on 2026-08-18 |
| `catalog.rs` *"86 of 101 named, 15 refused"* | the registry held 94, of which 85 named and 9 refused. `86 + 15 = 101` is internally consistent, which is why nobody looked twice |

The third is the instructive one: a test **had** been added after the fourth
drift of that pair, and it did not catch the fifth, because it pins the split
against its own literals and the *sentence* was never one of them. Its failure
message said *"update that sentence together"*. **A test that asks a human to do
the thing they just failed to do is a note, not a gate.** The heading no longer
carries numbers.

The repair is now this project's standing move and it has been taken four
times — the gate runner's header, `README.md`'s test count, this heading, and
the paper sentence. **When prose and a measurement disagree, delete the prose's
copy of the measurement rather than correcting it.** Where the prose must state
a limitation instead, there is no gate that can help; the only defence is
noticing at the site of the change that invalidates it.

**Two design decisions worth not re-deriving.**

- **`NotListed` is not `no`.** `pdfce-print` declined our proposal to gate the
  tray control on a `bool`, with a measurement: `DC_BINS` on Microsoft Print to
  PDF returns nothing at all, while that same device's `dmDefaultSource` is
  already `DMBIN_FORMSOURCE`. A bool would have hidden a control from a device
  that was doing the thing by default. R83 forbids offering what the hardware
  *cannot* honour; it does not forbid offering what the driver merely declined
  to advertise.
- **A new document is not an edited document.** `file.new_from_template`
  serializes and re-parses rather than handing over the `EditSession` that
  resized the page. Otherwise a brand-new A1 sheet arrives already modified,
  with `Ctrl+Z` waiting to take it back to A4.

**One of my own tests failed on its first run, correctly.** It asserted *"every
paper size's name differs from its uppercased identifier"* — false for A0
through A6, and rightly so. Restated as *"no name contains a hyphen"*, which is
what actually distinguishes a wrong fallback (`ANSI-D`) from a right one (`A0`).
Same family as a test that pins a refusal: an assertion that is *checkable* is
not the same as one that is *true*.

### From the session before: predicates too coarse, and a harness that lied

| reported as | actually |
|---|---|
| *"synthetic keyboard input does not reach the window"* | only **chords** failed. `keybd_event` posts asynchronously and egui drains once per frame, so modifier-down and key-down in the same microsecond deliver an **unmodified** key. Three 12 ms sleeps fixed it |
| *"18 controls laid out outside the window"* | the `ui-rect` trace is a **change log** and could not report that a control stopped being drawn. The ribbon overflow had correctly swallowed them. Fixed at source with `ui-rect-gone` |
| *"selection is not taking the hit test's result"* | six doc-points across a dense sheet all reported `hit 0 objects`. **A hit test that misses everywhere is a gate, not a hit test** — the check had never left Read mode |
| *"three headings illegible"* | three headings **not on screen**. A `ScrollArea` lays out below-the-fold children before clipping them |

Two mistakes kept in the docs because they looked reasonable: seven invented
stamp label strings (`TextAnnotSpec::Stamp` takes ISO Table 181's `StampName`,
so every stamp would have carried `/Name /Draft` whatever it read), and leaving
the UI-scale check's injected preference at 1.8 "on purpose" — next full run,
**20/0/4 → 3/1/21**. The distinction missed was **who owns the state**:
application side-effects stay, harness-injected inputs get restored.

**`tools/gates/check-string-gaps.sh` came from that session and is worth
knowing about before you write operator copy.** Rust's line continuation eats
the newline *and the next line's indentation*; lose the trailing backslash and
the indentation ships. The literal still compiles and still passes every test
that does not compare it to a hand-written expectation. The same grep found
**36 across 22 files, eight of them in copy the operator reads on screen**. It
is invisible in a diff — you see a wrapped sentence and the spaces read as
indentation, which is what your eye is trained to skip. Run the gate; do not
look.

**`--verify` had never worked, for a reason nobody had diagnosed.**
`subprocess.run(["bash", …])` resolves `System32\bash.exe` — **the WSL
launcher** — before Git Bash, which also explains a CRLF symptom filed
separately. One root cause, two unrecognisable symptoms. **A workaround written
against a wrong diagnosis outlives the problem and hides it.**

---

## ★★★ The founding rule, and the day it paid for itself

> **Verify by driving the binary, not by a passing test.**

**2026-08-18, second half: everything below was driven.** The operator handed
over the machine and the harness ran. What it settled is worth reading before
anything else in this file, because two of the three findings **contradict what
a green test suite said an hour earlier**.

### It falsified my own fix

The morning's `dialogs::textannot` focus fix — latch on `has_focus()` rather
than on having asked — has a headless regression test that fails on the old
implementation and passes on the new. It looked like the operator's bug.

`text_annot_takes_the_keyboard_unclicked` was then run against a binary built
with the **old** latch. **It passed.** The dialog took the keyboard all along;
the race the test constructs does not happen in the real frame. The fix stays,
because asking for focus and holding it really are different facts and the
bounded retry costs nothing — but **it is not the explanation**, and anyone
reading the commit for that story should read this paragraph instead.

### It found a defect no test could have

`app::keyboard::commands` compared a per-frame modifier snapshot
(`i.modifiers`) against a per-event fact (`Event::Key`). On a long frame — the
application rasterizing a dense CAD sheet — a quick `Ctrl+Z` arrives with Ctrl
already up and is silently dropped. It presented as *harness flakiness*: a
different pair of chords dead on each run, and reordering the list moved which.

### And it named what nobody was looking at

Nine module headers in `tools/ui-verify` recorded, as a fact about the machine,
that synthetic keyboard input does not reach the target window. It was inferred
from `Ctrl+E` arming nothing — `Ctrl+E` being one of the fourteen chords the
dispatcher never dispatched. Eight of those headers cited `checks::find_bar` as
the source; `find_bar` **passes**, and its own report says *"control chord
Ctrl+2 arrived, so the input channel works"*. The record contradicted itself in
the same run report for months.

**A constraint inferred about the environment is a reading, not a fact** — the
operator's own standing rule, and this is the second time it has cost this
project real work. A reading that stops people testing something is the
expensive kind: while it stood, no check drove a chord; because none did,
nothing contradicted it; and undo had no keyboard for months.

---

## Where a session can still fall short of it

The morning half of 2026-08-18 shipped two features with checks written and
**not run**, because the operator's desktop was in use. That is the normal
state of this project between hand-overs, and it is stated plainly rather than
softened: this project was founded on a commit that said *"analysis-confirmed,
NOT empirically verified"* and was treated as done anyway.

What driving buys, in four trace lines:

```
Markup > Text box armed the text-annotation tool
the page carries 0 annotation(s) before the drag
the release authored nothing — still 0 — and opened the dialog instead
Accept authored: the page went from 0 to 1
```

That middle line is the whole feature. A build where the release authored
directly passes **every** unit test in `canvas::textannot` — the spec builder is
pure and correct either way — and puts an empty box on the operator's drawing
every time they let go of the mouse.

---

## Where to read next

| file | for |
|---|---|
| `HANDOFF.md` | the standing rules, the phase order, the accumulated findings, and §5's six obligations of registering a command |
| `FEATURES.md` | what works today, row by row. The acceptance contract |
| `NO_SURFACE.md` | every hard-coded value with no control — **and the standing warning that a row here is not automatically a build-the-surface task** |
| `DEFECTS.md` | the defects this project exists to fix, with `file:line` |
| `D:\Dev\FeatureRequests\pdfce_FeatureRequests\` | the channel. `open/` empty means nothing is owed; `INDEX.md` is the memory |
| `D:\dev\rag\egui\`, `D:/dev/rag/rust/` | ecosystem findings — read before non-obvious work, write findings back |
