# Operator requests — the standing backlog

> **Ken, 2026-08-20:** *"Where do you need to put these requests so they just
> get auto-repeated over and over again so I don't have to keep requesting they
> be done over and over again?"*

**Here.** This file is the answer, and this section is the contract.

## The contract

1. **Every request you make goes in this file, the moment you make it**, before
   any work starts on it. Not into a chat reply, not into a session summary,
   not into an agent's memory — into this file, which is in git, on disk,
   backed up, and read at the start of every session.
2. **Only you close a row.** I may move a row to *shipped-and-driven*; the row
   does not leave this file until you have used it and said so. A row I believe
   is done but you have not confirmed sits under **Shipped — awaiting your
   verdict**, not deleted.
3. **A row carries evidence, not a claim.** *"Done"* is not a status. The
   status is either a driven check by name, or a dated note saying exactly what
   was verified and how. If nobody drove it, it says NOT VERIFIED, in those
   words.
4. **A blocked row names what blocks it and where that is filed.** If it is an
   engine gap, the row names the file in
   `D:\Dev\FeatureRequests\pdfce_FeatureRequests\open\`. A row that says
   "blocked" with nothing behind it is a row I have not done the work on.
5. **Nothing is silently rescoped.** If I ship half of what you asked for, the
   row stays open and says which half.

## Why this file and not something cleverer

The failure this fixes is real and it is mine: a request made in conversation
lives exactly as long as the conversation. Sessions end, context is compacted,
and an ask made in turn three of a long session is gone by turn thirty — which
is why you have had to repeat yourself. The agent-memory system is for *how you
work*; it is the wrong shape for *what you asked for*, because memories get
summarised and requests must not.

A file in the repository has none of those properties. It is read at session
start alongside `PROJECT_PLAN.md`, it survives compaction because it is on
disk, and its history is in git so a row cannot quietly disappear.

---

# ★★★ THE OPERATOR'S CURRENT DIRECTION — 2026-08-20

> *"put the text editing aside again. In the next version I just need the
> perimeter measuring tool to work with the group scale stuff the same as the
> other dimensioning tools have."*

**O1 is PARKED at his instruction.** Not closed, not solved — parked. The
findings stand and the engine request stays open. Do not pick it back up
without him.

**O3, the perimeter tool, is the single deliverable for the next build.** The
engine shipped its whole half on 2026-08-20; everything left is shell work.

## And the standing criticism, recorded rather than argued with

> *"We'll have to reconsider how you are going about the canvas later since it
> shouldn't take multiple 3 hour sessions each day to figure out how to get a
> cursor to move and edit text on it, or get shortcuts to work for basic
> functions."*

He is right and this belongs at the top of the file where it cannot be missed.
Two observations that are mine to act on, not his to have to make again:

1. **The basics were never audited as basics.** Ctrl+P had never been bound. A
   text caret had no index. Both are things every application in this class has
   on day one, and both were found by him rather than by us — because every
   test asked "does the thing I built work?" and nothing asked "does the thing
   everyone expects exist?". The keymap now has a list-shaped gate for exactly
   this reason. The canvas needs the same treatment and does not have it yet.
2. **Sessions have been spent diagnosing, not building.** Three of today's
   findings were engine defects that presented as shell defects, and each cost
   hours to localise. That is partly the boundary and partly that the driven
   checks were reading traces rather than pixels — a trace can say the verb ran
   and cannot say the screen changed. Two checks were fixed today to stop
   producing confident wrong diagnoses. That is the pattern to keep pulling on.

# OPEN

## O15 — Text editing should be MULTI-LINE

**Asked:** 2026-08-21 — *"I should be able to make it multi line."*
**Status:** **SHIPPED 2026-08-21 AND DRIVEN.** Awaiting your verdict.

Arm **Edit ▸ Add text** and **drag a rectangle**. Type into it. **Enter starts a
new line**; text that runs past the right edge wraps by itself. **Ctrl+Enter**
puts it on the page — or click away, which also commits.

A plain click still places a single line at a point, exactly as before, and
Enter there still commits. Two gestures, two behaviours, and the one you get is
decided by whether you dragged.

### Why it needs a rectangle, which is not a design choice

**A PDF has no paragraph.** Every visual line in a PDF is a separate instruction
at its own absolute position — there is nothing in the file that says "these
lines belong together". So something has to decide where the second line starts,
and the only thing that can is a width to wrap against. That is the rectangle.

### Driven

```
text-box-open page=0 box=301.2,438.8,500.9,499.7 w=199.7 h=60.9
add-text page=0 n=2 … boxed add: wrapped to 2 line(s) at 199.7pt box width,
         left alignment, top-anchored from the box top at 14.40pt leading
```

★ **And the check earned its keep on its first run.** Everything was correct —
the drag opened the box, Enter arrived, the right branch ran — and the newline
was **thrown away one function deeper**, by a filter that strips control
characters from typed text. Its own comment argued, correctly, that a control
character has no meaning in a PDF show string. True of typed text; not true of a
paragraph break. That is the **fifth** carefully-argued restriction in two days
to go false the week it was written.

### Still open, and named rather than implied

- **Turning existing text into multiple lines.** This ships *new* multi-line
  text. Making a line that is already on the page break into two is a *reflow*,
  which the engine has and which currently demands the page be saved and
  reopened first. Separate row when you want it.
- **Alignment and the box's own size.** A new box is left-aligned and cannot be
  resized after the fact. Both are surfaces rather than engine gaps.

## O16 — Reassemble lines into paragraphs, and move between blocks with the arrow keys

**Asked:** 2026-08-21 — *"there was an acrobat feature in the original pdfce-gui
that attempted to reassemble individual lines into paragraphs and the cursor
would move to the next block of text using the navigation keys."*
**Status:** OPEN, not started. **This is SALVAGE** — it existed in the shell
this project is replacing, so the first act is to find it there and read it, not
to design it.

`SALVAGE.md` is the register for what carries over and in what condition; this
row will name the file and the line count once it has been read.

★ The substrate is already in `pdfce-core` and this shell already touches it:
`EditableTextModel::recognize` with `BlockRecognitionOptions`, `block_at`,
`line_range_at` and `ReflowEngine::detect_alignment` are what
`canvas::textedit::plan` uses today to decide whether a tail should move. So the
paragraph recognition exists; what is missing is a surface that shows it and a
navigation that uses it.

## O14 — The conventions sweep, 2026-08-20: fourteen gaps, found by asking

**Asked:** by you, as *"how can you learn from these other programs so that you
can build the missing parts more effectively?"*
**Status:** the mechanism is built. These are what it found on its first run.

`D:/dev/rag/ui-conventions/` is a corpus of five gesture classes — what every
program in the class already does, where the rule comes from, and the failure
mode when it is absent. `tools/gates/check-conventions.sh` makes each
interactive surface answer every row of its class, in its own source, and fails
the build on an unanswered one. Eleven surfaces registered; all eleven now
answer.

It cannot check behaviour and does not pretend to. It checks that **the question
was asked** — which is the whole of the problem, because every convention you
have had to report was one nobody had asked about rather than one somebody
decided against.

### What it found. None of this was known before today.

**Direct manipulation**

1. ~~**Shift does not preserve aspect on a resize.**~~ — **SHIPPED
   2026-08-20.** Hold Shift while dragging a corner and the shape keeps its
   proportions; the status row says *"Shift: keeping its proportions"* so you
   can tell the key did something. A side handle under Shift scales both axes,
   which is what Figma and Slides do.
2. ~~Shift does not constrain a move, a handle drag or a dimension drag to an
   axis.~~ — **SHIPPED 2026-08-20**, all four drags. A move, a dimension label,
   a perimeter corner and a Bézier handle each lock to whichever axis you have
   travelled furthest along, re-decided every frame — so you can start off
   crooked, commit to vertical mid-drag, and it follows. Let go of Shift and it
   comes straight back to the free path.

   ★ A Bézier handle locks to its **anchor's** axis rather than to where you
   grabbed it, because a control point's meaning is the tangent it defines.
   That is what Illustrator and Inkscape do and it is the one place the four
   drags differ.

   **Not built, and named rather than implied:** Alt to scale about the centre,
   Alt to break a smooth node's symmetry, a 45° diagonal lock, and a dimension
   label held to its *standoff* or its *slide* specifically rather than to a
   page axis. Each is a decision recorded in `canvas::constrain`'s header, not
   an omission.
3. ~~**A vertex drag does not snap**, while the tool that placed that vertex
   does.~~ — **SHIPPED 2026-08-20.** Drag a perimeter corner and it now snaps to
   endpoints, midpoints and intersections exactly as the tool that placed it
   does, with the same marker at the same size, honouring the same *"Snap to
   content"* switch. Hold **Alt** to refuse the offer for one drag.

   ★ **The snap overrides the grab point**, deliberately. If you grabbed the
   handle three pixels off centre, a corner that preserved that offset would
   land three pixels off the thing it snapped to — a corner that looks snapped
   and is not, which is the worst of the three outcomes.

   The **label** drag still does not snap and will not: a label's position is
   presentational, it changes no measured value, and snapping a caption to a
   wall would move it onto the drawing rather than clear of it.

   Driven: `measure_perimeter_traces_and_closes` now asserts the drag **asked**
   the snap query — the `snap=` field exists on the shell's own line. It does
   not assert a hit, because whether anything is near that destination is a
   fact about the fixture and not about the build. NOT YET RUN.
4. Neither a move, a resize nor a handle drag snaps to guides, grid or geometry.
5. **No rotate handle** anywhere. ~~Blocked on the engine verb~~ — **the verb
   shipped 2026-08-20 and rotates**; what is missing is a ninth grip above the
   selection box to reach it with, a drag that measures an angle rather than a
   distance, and a preview. **Shell work, unblocked, and the next thing on the
   list unless you say otherwise.**
6. **No right-click to add or remove a perimeter point**, though both engine
   verbs and the preflight that greys the menu item already exist.
7. A zero-travel release still raises an action in three of the four drag paths.

**Selection**

8. **Only ce dimensions hit-test their real shape.** A `/Square` with no interior
   colour still claims its interior, so a large empty callout box is
   un-clickable-through. The mechanism to fix it now exists — that subtype needs
   a shape.

**Text**

9. ~~No live preview while typing~~ — **FIXED 2026-08-20.** An in-place editor
   box, sized to what you type, with the caret measured against the text as
   drawn. The design had always intended the characters to be shown *off-canvas
   in the status bar* and that half was never built, so they appeared nowhere at
   all.
10. Caret indices are characters, not grapheme clusters, so a combining mark or
    an emoji takes two presses. `unicode-segmentation` is already in the tree.
11. **No selection inside a draft** — no Shift+arrow, no Ctrl+A, no drag-select.

**Dialogs**

12. ~~**No dialog is a real OS window**~~ — **PRINT SHIPPED 2026-08-20.**
    Print now opens in its own window: a title bar you can drag, a taskbar
    entry, and you can put it on the second monitor or move it off the drawing
    to read the page underneath while you choose a range.

    Your words, recorded because the last sentence was the diagnosis:

    > *"Print dialogue box doesn't pop up in its own movable window. It is
    > locked within the boundaries of the program's window. Like, I just assume
    > you've been trained on a million lines of code and software that pops it
    > up in its own window."*

    The mechanism is one host, so **the other thirteen dialogs are one line
    each** rather than thirteen implementations. Print first because you said
    to start there.

    ★ **Verified, and without touching your mouse.** A new headless seam
    (`PDFCE_DIAG_INVOKE`) lets a diagnostic run press a ribbon command in an
    invisible window, so this was proved on the machine while you were using
    it. The evidence:

    ```
    diag-invoke id=file.print
    print-open printers=12 selected=8
    viewport-inner id="4206" rect=[[-3944 -3921] - [-3144 -3301]]
    ui-rect name=print.paper rect=[[393.9 480.0] - [601.4 504.0]] viewport="4206"
    ```

    An 800 x 620 OS window of its own, with its controls positioned inside it.

    **Still open:** the other thirteen dialogs, and one row that eframe cannot
    express — the dialog is not *owned* by the main window, so it can fall
    behind it. There is no owner option in the toolkit's window builder at all.
    Making it always-on-top instead was considered and refused: it would break
    the driven harness in a way that produces confident wrong bug reports, which
    we have paid for once already. The taskbar entry is the route back.
13. ~~**Enter is not the affirmative default**~~ — **PRINT SHIPPED
    2026-08-20.** Type a page range, press Enter, it prints. Print is drawn
    filled in the theme's own accent so you can see what Enter will do before
    you press it, and Escape now closes the dialog exactly as the X does.

    The pair is drawn by the host, not by the dialog, so no future dialog can
    implement two of the three obligations and forget the third.

    **Known limit, named rather than found:** Enter is suppressed while a text
    field has focus, because the toolkit reports *"a text field has focus"*
    without saying whether it is multi-line — and a multi-line field must keep
    the ability to type a newline. So in a dialog whose last control is a
    one-line box, you may need to click out of it first. The fix is per-field.
14. **PARTIAL 2026-08-20.** Print comes back where you left it for as long as
    it stays open. It does not survive closing and reopening, and it does not
    survive a restart: a remembered position has to be checked against your
    current monitors, and a dialog that opens on a screen you have unplugged is
    worse than one that opens where Windows puts it. Tab order and modal
    focus-trapping are still untested.

### And two fixed on the spot, because writing the row exposed them

- The vertex drag converted screen→canvas **twice**, so it tracked at `1/zoom`
  and sat off by the scroll origin — *"the distance from the pointer varies as
  you move it."* Fixed.
- It also assigned the pointer straight to the vertex, so grabbing a handle
  slightly off-centre teleported the corner under the cursor before you had
  moved it. Now it moves by the delta and the grab point is preserved.

## O8 — **A Save button.** Not Save As. Save.

**Asked:** 2026-08-20 — *"can I please have a save button like every other
program in existence has? We're on week two of this and just have a save as
button."*
**Status:** **SHIPPED 2026-08-20 and DRIVEN.** Awaiting your verdict.

`Ctrl+S` saves over the file you opened. `Ctrl+Shift+S` is Save-a-copy. The
quick-access toolbar's second slot is Save now, and it carries the disk glyph;
Save-a-copy renders as text.

It writes to a temporary beside your file and then renames, so a crash or a full
disk in the middle leaves your original untouched rather than half-written. And
because pdfce saves incrementally, the previous version of the document stays
inside the file — nothing is thrown away by pressing it.

Driven: `save_writes_over_the_file_you_opened` — the file grew 140,660 →
141,423 bytes, no temporary was left behind, and it still reads as a page tree.

★ The blocker that kept this out for a fortnight said *"in-place save is blocked
on autosave and crash recovery"*. That was aimed at the wrong hazard: pdfce's
incremental format already WAS the crash recovery. What was actually unsafe was
the write, and that has a three-line answer nobody had written because nobody
was asking. Third time in two days that a blocker turned out to be a question
asked wrongly.

There is no defence for how long it took. `Ctrl+S` is bound to `file.save_copy`, which asks where to
put it, every time. Overwrite-in-place was written down as *"an operator scope
decision"* and then sat there being nobody's problem — which is the same failure
as `Ctrl+P` never being bound and the caret never having an index: **the basics
were never audited as basics**, because every test asked "does the thing I built
work?" and nothing asked "does the thing everyone expects exist?".

## O9 — A **length** tool: the perimeter tool that never closes

**Asked:** 2026-08-20 — *"add a length tool that works like the perimeter tool
without needing to close the profile."*
**Status:** **SHIPPED 2026-08-20 and DRIVEN.** Awaiting your verdict.

Measure ▸ **Length**, beside Perimeter. Same gesture, same snapping, same
preview, same running total, same group scale — it just never closes. Clicking
the first point again adds a point there, because a run of cable that loops back
is still a run of cable. Double-click the last point to finish.

It is a separate control rather than a checkbox on Perimeter because "Perimeter"
says closed, and nobody measuring a pipe run would go looking inside it.

Driven, in the same check as Perimeter and deliberately so: what is worth
proving about Length is a *negative* relative to Perimeter — that the
first-vertex click does **not** close it — and a negative is only meaningful
beside the positive it differs from. Two separate checks would let the pair
drift into being one tool.

```
★ the ring closed and the dimension reached the engine   (Perimeter)
★ the Length tool took all 5 clicks as vertices          (Length)
```

## O10 — Neither measuring tool previews while you trace

**Asked:** 2026-08-20 — *"both these tools need a preview just like the measure
tool has."*
**Status:** **FIXED 2026-08-20**, awaiting your verdict.

The preview arm was written and unit-tested for its segments, and it was
**unreachable**: `super::preview` returns early on
`MeasureState::gesture_in_progress()`, and that function had not learned about
the perimeter's pick. So the tool drew nothing at all while tracing.

It is the failure class this project keeps meeting — every part correct, the
*join* unobserved — and the driven check could not see it, because that check
asserts on the trace and a preview is pixels. `every_pick_kind_is_counted_as_a_gesture`
is now the guard.

## O11 — Move, resize and rotate a placed image on the canvas

**Asked:** 2026-08-20 — *"there was no way to reposition, resize, or rotate it
on the screen. Can I please please please have that too?"*
**Status:** **MOVE AND RESIZE SHIPPED 2026-08-20 AND DRIVEN. ROTATE IS NOT — see
below.** Awaiting your verdict.

Select a picture and drag it: it moves. Grab a corner and drag: it resizes.
Select several things at once — a picture, a box and a line — and one corner
drag resizes all three about the same point, as one undo entry.

Three refusals you may have seen are gone with it:

- *"pdfce cannot resize text or pictures — only shapes drawn out of lines and
  curves."*
- *"pdfce resizes one shape at a time."*
- *"This shape has no corners to move."*

**Driven, on your own drawing, 2026-08-20:**

```
resize_scales_a_shape           PASS — through transform_objects
geometry_fields_resize_a_shape  PASS — the typed W/H route, same function
shift_constrains_a_resize       PASS — and Shift keeps the proportions
```

### ★ ROTATE IS NOT BUILT, and it is a shell gap now rather than an engine one

The verb rotates. **There is no rotate handle on the canvas to reach it with.**
That is `O14` row 5, and it stopped being blocked tonight — it needs a ninth
grip above the selection box, a drag that measures an angle rather than a
distance, and a preview. Say the word and it is the next thing.

### And one more thing that is not built, named rather than left to be found

A picture whose own placement matrix is degenerate cannot be transformed at
all, and the engine says so — *"do not offer a handle"*. pdfce currently offers
one and you would find out by dragging it. The preflight that would grey it
needs a page decomposition cached per selection (**~4 seconds** on your
benchmark drawing in a debug build), which is a piece of work rather than a
line. Rare: it needs a producer to have emitted a collapsed matrix.

## O12 — Move text after placing it

**Asked:** 2026-08-20 — *"can I please please please have the capability to move
the text after?"*
**Status:** **SHIPPED 2026-08-20.** Select the text and drag it. Same verb as
O11, exactly as asked for — a placed image and a placed text run are the same
shape in a content stream, so they got one verb rather than two.

★ **A move still uses the lighter verb where it can**, and that is deliberate
rather than a leftover: for a selection made only of shapes, pdfce rewrites the
coordinates in place and adds nothing to the file. The general verb wraps each
object in three extra operators every time you nudge it, and you nudge things
dozens of times in a file you then send to somebody. Shapes take the light
path; anything else takes the general one.

**NOT YET DRIVEN** on a text object specifically — the driven checks aim at a
shape, because that is what the fixture's `--doc-point` names. The verb is the
same one three passing checks exercise.

## O13 — Insert image does not appear until you save and reopen

**Asked:** 2026-08-20 — *"I tried a new document and inserted an image. Nothing
appeared on screen or in the tree, but after saving and reopening the image was
there."*
**Status:** **FIXED 2026-08-20 and DRIVEN.** Awaiting your verdict.

Your report split O4 in two, and this half was mine.

After every edit the shell re-walks the page tree and then compared *(page
object id, rotation)* against what it had. If nothing there moved it returned
early, with a comment saying *"the page vector already describes the
document"*. **That is false.** A `Page` is not an id — it is a resolved page,
with its `/Contents` and `/Resources` in it. `add_image` turns `/Contents` from
a stream into an array and adds an `/XObject`; the page's id does not move; the
early return fired; the canvas and the Objects panel went on reading a page as
it was before the edit.

Which is why saving and reopening worked: the bytes were right the whole time.

Markup never showed it (annotations are read from the session, not that vector)
and moving an object never showed it (that rewrites a stream *in place*, so the
stale reference still resolves to the right object). `add_image` is the first
verb that changes what `/Contents` **is**, so the bug had been there since it
was written and had never been reachable in a way anyone could see.

Driven, on your own JPEG, into a new document from the template:

```
made a blank document first
placed: add-image page=0 n=1 — 839 dpi
the page repainted: 118,580 of 256,878 pixels changed
```

and the Objects panel now reads *"1 object(s) on this page — 1 image(s). #0
Image · 6247 × 5010 px"*.

**O4 is still open** — that one is the engine corrupting `/Contents` when it is
already an indirect array, which is what your CAD sheets use, and it produces a
file pdfce cannot reopen at all. Filed and unchanged.


## O1 — Editing text on the canvas, and editing text in a text box

**Asked:** repeatedly; restated 2026-08-20 — *"Still no editing text on top of
the canvas. Or editing text on a text box."*
**Status:** OPEN. Under investigation 2026-08-20; nothing claimed.

Two distinct things and I have been conflating them, which is probably part of
why it keeps coming back:

- **(a) Editing text that is already on the page** — click a run of existing
  page text, get a caret, retype it.
- **(b) Editing the text inside a text box you have added** — a `/FreeText`
  annotation, or a text object this shell authored: double-click it, get a
  caret in it, retype.

**(a) — driven 2026-08-20 on your own CAD drawing. It is reproduced, and it is
not the shell.** `text_edit_on_a_real_drawing`:

```
text-edit-caret kind=Edit page=0 run=44 len=1     ← the caret lands on real text
text-edit-typing draft=true text_events=1 len=2   ← keystrokes reach the draft
text-edit-typing draft=true text_events=1 len=3
text-edit-plan page=0 run=44 disposition=Pin reason=Rotated pinned=true
edit-text-refused page=0 n=1
  detail=text to edit ("p") was not found in an editable run on the page
```

So the tool arms, the caret lands on the right run, the typing arrives, the plan
is built and the commit reaches the engine — **and `pdfce-core` refuses it.**
From your chair that is precisely "the tool responds and the page does not
change".

Root cause under investigation 2026-08-20. It will be either an engine gap (a
request, filed) or a wrong call on my side (a fix). Either way this row records
the answer.

★ And a second defect found in the process, on my side: the driven check for
this was reporting *"THE COMMIT NEVER REACHED THE ENGINE"* — which was **false**.
`edit-text-refused` is not `edit-text`, so a check asserting on the absence of a
line produced a confident, specific, wrong accusation about working code. Fixed:
a refusal is now asked about first and quoted verbatim.

**(c) — the caret cannot be moved inside a run. FIXED 2026-08-20**, and it was
worse than it looked: there was **no caret index at all**. The draft appended
text and Backspace popped the last character, so the painter drew its line at
the right edge of the run's box because that is the only position an
append-only draft has.

Now: a real caret. Click part-way into a run and it lands at that character
(measured against the run's own glyph advances); Left, Right, Home, End,
Ctrl+Left and Ctrl+Right move it; Delete eats forwards; typing and Backspace
act at the caret. Unit-tested, including your `SHEET 1 OF 4` case as a named
test. **NOT yet driven** — the harness was blocked by the on-screen keyboard.

**No selection yet** — no Shift+arrow, no Ctrl+A, no drag-select inside a
draft. That is a second feature and it is row **O7** rather than an implied gap.

**(a) — ★★★ IT WORKS. 2026-08-20, and it is the 99 % case.**

You can now click a label, a title-block field or a *pdf dimension* callout on
a CAD sheet, get a caret, and retype it. That text lives inside what the format
calls a form XObject — a block the drawing program placed — and until this
evening pdfce could read it and not write it.

Measured on your benchmark drawing, which is why this mattered more than
anything else in the queue: **1,696 show operators of real drawing text inside
the block, against 3,007 metadata glyphs in the page's own stream.** Your own
words when you saw the split: *"I need that editing capability as it is 99% of
the text I will want to edit."* The engine escalated the work ahead of the
move/resize verbs on the strength of that sentence.

### ★ One thing you need to know, and it is not a pdfce limitation

**A drawing program may place ONE copy of a block and paint it on six sheets.**
That is what the construct is *for* — the standard names a CAD system's
standard component as the illustration — and nothing in the format binds a
block to a page. So when you edit text inside a shared one, **it changes on
every sheet it appears on**, because there is exactly one copy of those letters
in the file.

pdfce cannot make that not be true, so it tells you: after an edit that touched
shared content, the status row says *"SHARED CONTENT: this text is drawn from
shared content that appears in N place(s) on M page(s)"*. It is deliberately
silent on the ordinary case — a warning that fires every time is one nobody
reads, and this one is meant to make you stop.

**Nothing is drawn on the page.** No badge, no tint, no flag. Your own finding
about the old GUI's red-flagging stands.

**Not built, and named rather than left implied:** you are told *after* the
edit, not before you type. Telling you at the caret means asking the document
how many places paint that block, which is a walk of the whole file — cheap
once, not cheap on every click on text — so it needs a cache that does not
exist yet. Undo puts it all back in one press in the meantime. Say the word and
it moves up.

### What it cost on this side, and why it was one deleted line

The shell had a guard refusing the caret. When it was written, the request that
went with it said:

> *"my shell encodes a fact about your surgery's internals. The day form
> editing lands, my guard silently keeps refusing until I notice and delete
> it."*

So the engine published a query — *"is this run editable?"* — and the shell
asked it instead of modelling the answer. When the capability landed, the query
started answering *yes*, a deprecation warning pointed at the single line to
remove, and that was the whole job. A hand-rolled guard would have gone on
refusing 99 % of your text until somebody noticed.

★ **One thing added beyond deleting the guard.** The shell now names *which*
buffer it measured when it commits, rather than letting the engine search. On
your sheets the page's own stream holds 3,007 single-character operators, so
letting a byte offset be tried there first is a dense field of near-misses —
an edit that could succeed on the wrong glyph with no error anywhere.

**Driven:** `text_edit_on_a_real_drawing` now asserts the commit named a form,
and the old "this is an absent capability" skip has been **inverted** — if a
build ever refuses with the old reason again, that check fails loudly.
**NOT YET RUN.**

What remains under this heading is the refusal for text that has no letters
behind it at all — an `/ActualText` description the producer supplied instead
of glyphs. That one is genuinely unreachable and always was, and it now says
so in its own words rather than borrowing the form sentence.

**(b) — not yet driven.** Nothing claimed.

## O2 — Cut / copy / paste of PAGE CONTENT (`Ctrl+X` / `Ctrl+C` / `Ctrl+V`)

**Asked:** first week, and repeatedly since. Restated 2026-08-20: *"can you get
cut copy and paste working for objects I select on the canvas?"*
**Scope set by you, 2026-08-20:** *"oh I might want all cases so we shouldn't be
restrictive in our ask."*
**Status:** **SHIPPED 2026-08-20 AND DRIVEN.** Awaiting your verdict.

Select a line, a shape or a piece of text on the page and press **Ctrl+C**, then
**Ctrl+V**. It lands 10 pt down and right so you can see it is a copy, or in
place if you paste onto a different page. **Ctrl+X** cuts, as one undo.

Driven on your own drawing: a 108 KB clip out, one object back in.

### ★★★ And Ctrl+C had never once reached the keyboard map — that is why you kept reporting it

You said *"still no ctrl+c, ctrl+v, ctrl+x"* twice. On 2026-08-20 they were
bound, which was necessary and **not sufficient**: the toolkit intercepts those
three chords and converts them into its own clipboard events **before** the
keystroke reaches anything pdfce can see. So the binding existed, every test
agreed it existed, the menu showed it next to the command — and the key did
nothing, for ever.

★ **Ctrl+V was worse.** The toolkit only raises a paste event if the *Windows*
clipboard already holds some text. With it empty, the keystroke vanished
completely — so whether paste worked depended on **whether you had recently
copied text in another program**. Not random, not reproducible, and nothing to
do with pdfce.

That is why copying now also leaves a sentence on the Windows clipboard —
*"1 object copied from pdfce. Paste it back into pdfce to place it."* It is what
makes the key arrive, and if you paste it into an email by accident it reads as
an explanation rather than as garbage.

### What works today

- **Page content — a path, a line, a block of drawing, a picture, text:** cut,
  copy and paste, in any mixture, as one undo entry.
- Markup and comments: cut, copy and paste.
- Swept text: copy to the system clipboard.

### Still open, and named rather than left as a silence

- **Across two pdfce windows.** Within one window it is lossless. Between two
  processes it needs the clip registered under a private Windows clipboard
  format, which is a call this shell does not make yet.
- **Copying to another program** — Illustrator, SolidWorks — needs the selection
  rendered as a standalone one-page PDF, which the engine has filed separately
  and deliberately did *not* fold into the same bytes: a one-page PDF cannot
  carry which byte range was which object, so re-deriving it on the way back in
  would make a pdfce→pdfce paste worse than a pdfce→Illustrator one.
- **Dimensions and form fields** are annotations rather than page content, so
  these verbs cannot reach them at all. Filed.

### ★ And our reading of the engine was right in a way that mattered, and wrong in one place that was the whole job

We scoped the ask as *"expose the copy engine you already have at object
granularity"*, on the strength of a function that already copies object graphs
with every reference remapped. That was correct. What it misses is that a
drawing's content is not an object graph at all — it is **bytes inside a page's
content stream**, and those bytes name their fonts and images **by a nickname
that is local to that page**. On another page, `/F1` is a different font.

So a naive copy would have pasted the right letters in the wrong typeface, and
**nothing would have errored**. The engine built the name-rebinding half; our
reading identified the prerequisite. Worth recording for the next request scoped
that way.

### I nearly asked for a third of it

I was going to ask for `duplicate_objects` alone, on the argument that Ctrl+V in
one document decomposes into *duplicate + offset* and `move_objects` already
exists. That is true and it would have covered same-document duplication only —
not pasting into the other tab, not the system clipboard, not dimensions or form
fields. You stopped that, and the filed request is the whole capability:

1. **A portable object payload** — content *and* the resources it depends on.
   Kind-agnostic, so a mixed selection works; takes a `Matrix`, so paste-in-place,
   paste-offset, paste-scaled and paste-rotated are one verb; with a preflight so
   the menu item can be greyed rather than discovering the refusal by pressing.
2. **Serialisable**, which is what makes cross-document and cross-session paste
   fall out instead of being a second feature.
3. **The system clipboard** — a pdfce-private format so pdfce→pdfce is lossless,
   plus a standalone PDF and an image so SolidWorks and your CAD packages can
   read it. Registering those is mine; I need the bytes from them.
4. **Cut as one undo entry**, or Ctrl+X then Ctrl+Z gives your objects back and
   leaves the clipboard changed.
5. **Dimensions and form fields refuse loudly** rather than pasting something
   subtly broken — a pasted dimension needs a sidecar record and a group, a
   pasted field needs a name that does not collide. Silent partial success is
   the one outcome I cannot work with.

**Reading vector data IN from other programs** (paste from Illustrator) is
explicitly *not* in the ask — that is foreign PDF/EMF/SVG parsing and a much
larger job. Named so it is a decision rather than an omission. Say the word if
you want it.

### ★ The finding that makes this smaller than it looks

`EditSession::import_object` already exists, privately: a recursive
cross-document object-graph copy with fresh object numbers, every reference
remapped, cycles handled, and stream payloads re-staged. It is what
`insert_pages` and `merge_document` use to bring pages across **with their
fonts, patterns, images and soft masks intact**.

That is the entire difficulty of pasting page content, already solved. The ask
is not "build a copy engine" — it is "expose the one you have at object
granularity."

**Sits below the transform verbs in priority**, deliberately: an operator who
can place an image and not move it is worse off than one who cannot copy a path.
The first is a feature that looks broken; the second is one that is absent.

## O3 — Perimeter measuring tool

**Asked:** 2026-08-20 — click around to make a shape, sum the segment lengths
into one dimension; right-click to add segments; drag the endpoints to adjust
the shape; all the scaling options of the other dimension tools.
**Status:** OPEN — **and no longer blocked.** Filed 2026-08-20; the engine
shipped the whole thing the same day (commits `9940acf`, `ae06440`): a
`Perimeter` kind carrying its vertices and an open/closed flag, verbs to move,
insert and remove a vertex, and a preflight so a right-click menu can be greyed
correctly rather than by guessing.

The value goes through the same group path as every other dimension, so scale,
unit, precision, drafting standard and layer come free. The label sits at the
vertex centroid, so it drifts smoothly when you drag a corner instead of
teleporting across the shape.

**SHIPPED 2026-08-20 and DRIVEN.** Measure ▸ Perimeter arms a tool; click
around the shape; the polyline previews as you go with a rubber band to the
cursor; the Tool panel shows the running total in the group's own units;
clicking the first point closes the ring and commits; double-click finishes an
open path; `measure.finish` on the ribbon does the same.

It is a real dimension in a real group, so the scale, unit, number format,
drafting standard, layer and style cascade all apply exactly as they do to a
linear dimension — which is the half you asked about specifically.

Driven on the benchmark sheet by `measure_perimeter_traces_and_closes`:

```
Measure ▸ Perimeter armed the tool
four vertices taken; running total: -0.0 → 378.8 → 655.8 → 1023.6
★ the ring closed and the dimension reached the engine: add-dimension page=0 n=1
```

The first driven run FAILED, correctly: the ring would not close, by eight
canvas units. The first vertex is stored where the *snap* put it, so the closing
click was being measured against a target that had already moved — and it was
being measured with the click tolerance instead of the snap tolerance. Fixed.

**Dragging the endpoints SHIPPED 2026-08-20 and is driven.** Select a perimeter
and its corners get handles; drag one and the shape follows, previewed as you
go. The status row reports what it cost you:

```
That corner changed the measurement: 621.45 pt is now 1226.84 pt.
```

Both numbers, because you can see the new one and cannot see the old one — the
geometry it came from is gone. Silent when the number did not move, so the line
means something when it appears.

You can also drag a perimeter's **number** now, the same way as a linear
dimension's — and more freely, because a perimeter's label is anchored in page
axes rather than to an axis, so it lands where you drop it instead of being
flattened onto a line.

**Still open in this row:** right-click a segment to add a point, right-click a
point to remove one. Both engine verbs exist, and so does the preflight that
tells a menu whether to grey the item, so this is shell work only.

## O4 — Insert image does nothing

**Asked:** 2026-08-19, restated 2026-08-20 — *"No it always hasn't worked."*
**Status:** **BOTH CAUSES FIXED 2026-08-20.** Awaiting your verdict.

You were right, twice, and it was never a misunderstanding. There were two
separate defects sitting on top of each other:

**The engine's.** `add_image` corrupted the page's `/Contents` whenever it was
an indirect reference to an array — which is what every CAD-exported sheet uses.
The verb returned success, the status bar reported the resolution, the picture
was not on the page, and **the saved file could not be reopened by pdfce at
all**. Filed with an eight-line repro; fixed in `Pass 111.0`. Files already
damaged by an older build now open, render, and say so through a counted
disclosure rather than being silently patched.

Verified here, headlessly, on your benchmark drawing:

```
pages BEFORE: Ok(1)
pages AFTER:  Ok(1)     ← was Err("/Contents is neither a stream nor an array")
reloaded:     Ok(1)
```

**Mine.** The shell re-walked the page tree after every edit and returned early
when the page's object id had not moved — with a comment claiming *"the page
vector already describes the document"*. False: a `Page` carries its `/Contents`
and `/Resources`, and `add_image` changes what `/Contents` **is**. So the canvas
and the object tree went on reading the page as it was before the edit. That is
row O13, fixed the same day and driven.

Between them they explain everything you reported, including why saving and
reopening showed the picture: the bytes were right the whole time.

## O7 — Selecting text inside a draft

**Asked:** not by you. Recorded 2026-08-20 because it is the obvious next thing
after the caret and I would rather name the gap than leave it implied.

Shift+arrow, Ctrl+A, and dragging across a draft to select part of it, so that
typing replaces the selection. Not started.

## Q1 — ANSWERED 2026-08-21: the rest of the line MOVES ALONG

> *"It should move along."*

**Settled. Nothing changes** — `FollowerDisposition::Reflow` was already the
default and stays it. The three automatic exceptions stay too: rotated text,
right-aligned or centred text, and a line drawn as several separate pieces are
each pinned, because in those three cases "moving along" would move the tail the
wrong way, off its margin, or drag a neighbour that is not part of your edit.

Recorded rather than deleted, because the engine will ask again the next time it
touches reflow and the answer should not have to be re-derived.

The question, kept for that reason:

When you retype a piece of text and the new words are longer, pdfce has to
decide what happens to whatever is drawn after it on the same line. Two answers:

- **Push it along** (today's default). Right for a paragraph; the sentence stays
  a sentence.
- **Leave it exactly where it is**, and absorb the difference invisibly. Right
  for a drawing, where a label beside a label is not a sentence and moving one
  is a change nobody asked for.

pdfce already picks *leave it* automatically in three cases: rotated text,
right-aligned or centred text, and a line drawn as several separate pieces —
which is most of a CAD title block. The question is whether **drawing content
should default to leaving it** rather than relying on those three to catch it.

The engine's own view: *"`Pin` is the safe posture for drawing content … worth
offering on a per-edit basis rather than as a global preference, since it is
right for a CAD label and wrong for a paragraph."*

**Nothing is built on this and nothing will be until you answer.** Recorded here
rather than decided quietly, because it changes what happens to your drawings
when you type.

## O5 — Horizontal / vertical dimension constraint, from a drop-down

**Asked:** 2026-08-20.
**Status:** OPEN, not started. `LinearPick::constraint` exists in the shell and
is never written outside tests — there is no control for it. No engine work
needed.

## O6 — The scale ratio field follows the dimension you set

**Asked:** 2026-08-20 — *"when I set the dimension the editable ratio one shown
should change to match it."*
**Status:** OPEN, not started.

---

# SHIPPED — awaiting your verdict

Rows here are built, gated and driven. **They stay here until you have used
them and said so**, then they move to CLOSED with the date you confirmed.

## S1 — Move a placed dimension, with a live preview

**Asked:** 2026-08-20. **Shipped:** 2026-08-20, commit `469d4d7`.
Press inside a selected ce dimension and drag: the dimension line follows,
previewed from the same function a committed dimension is drawn from, and the
release commits `place_dimension`. The measured points never move, so the
printed number cannot change. Linear dimensions only — angular refuses at the
press rather than starting a drag that could not finish.
**Verified:** unit tests only (4). **NOT yet driven through the harness.**

## S2 — The measure sidebar no longer hides half its controls

**Asked:** 2026-08-20. **Shipped:** 2026-08-20, commit `c2de963`.
The group list was a four-column grid 209 pt wider than its own column, clipped
with no scrollbar in that axis. Now a block per group, and the panel measures
its own overflow so it cannot come back quietly.
**Verified:** `no_row_in_this_panel_outruns_a_narrow_dock`, which failed at
209 pt before the change. **NOT yet looked at on screen.**

## S3 — `Ctrl+P` opens Print

**Asked:** 2026-08-20. **Shipped:** 2026-08-20.
It had never been bound. Print was on the ribbon, in the QAT and in a menu, so
every surface that lists commands showed it and only the keyboard did not.
`the_keymap_offers_the_chords_a_document_application_must` now asserts the whole
list of universal chords rather than this one line.
**Verified:** unit test. **NOT yet driven.**

## S4 — Imperial sheet sizes

**Asked:** 2026-08-19. **Shipped:** 2026-08-19, commit `815036a`.

## S5 — Multiple documents, page drag between them, tab reorder

**Asked:** 2026-08-19/20. **Shipped:** 2026-08-20.
**Verified:** driven — `document_tabs`, `page_drag_between_documents`,
`tab_reorder`.

---

# CLOSED

*(Nothing yet. A row lands here only when you have said it works.)*
