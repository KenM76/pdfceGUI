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

## O21 — Move, resize and rotate ANY object; click nodes, select several, move them — all with live preview

**Asked:** 2026-08-21 — *"I think pdfce implemented the capability to move and
resize and rotate any object. you'll have to confirm, but that is what I want. I
should be able to click individual nodes, or select several at once and move
them too, with live preview of everything if possible."*

**Status:** ★ **ENGINE CONFIRMED 2026-08-21 against `D:\Dev\pdfce` source.**
You were right, with two boundaries worth knowing. Asking for it to be
confirmed rather than assumed was the correct instinct and it paid: the
confirmation also caught **a claim in this very file that was false**, and I
had re-published it an hour earlier — see `O20`.

### What the engine actually does

**`EditSession::transform_objects`** (`crates/pdfce-core/src/edit.rs:7512`) is
**genuinely kind-agnostic**: one verb, one undo entry, doing move, scale,
rotate, shear and mirror on **paths, text objects, image XObjects, form
XObjects and inline images**. It is kind-agnostic *by construction* rather than
by a match — it wraps each object's byte span in `q … cm … Q` and never reads
an operand (`vector/edit.rs:996`). So *"any object"* is true for page content,
and it is true of text specifically.

**Three places it stops being true**, and they are worth knowing because each
is something you might reasonably try:

| | |
|---|---|
| **Annotations** — markup, form fields, ce dimensions | no transform verb at all. Translate only, or nothing. And a `/Rect`-based markup **cannot express a rotation**: the engine's own words, *"a rotated one has no spelling"* |
| **Below whole-object level** — subpaths, nodes, Bézier handles | **translate only.** There is no rotate or scale for a node selection |
| **Inside a placed block (form XObject)** | not addressable. The decomposer treats it as one object and does not recurse, so you can rotate the block but nothing within it |

### Nodes — better than expected

**`move_nodes`** (`edit.rs:8486`) moves **several anchors in one call**, each
to its own destination, as one command and one undo entry. ★ And a loop of
single moves would have been *wrong*, not merely slow — all four corners of an
`re` rectangle are the same four operands, so the second call would plan
against byte offsets the first had already replaced.

**Bézier control points are separately addressable** (`move_handle`,
`edit.rs:8542`), and it refuses a straight segment by name rather than quietly
turning a line into a curve.

★ The shell already has multi-node selection and already calls `move_nodes`
(`canvas/moving.rs:560`, `app/actions/vector.rs:469`). Whether you can *build*
that selection by pointing — marquee inside an object, Shift+click to add — is
the open question, not whether the verb exists.

### Live preview — already there for all three gestures

`canvas::overlay` draws a move ghost, a **rotate ghost** (four transformed
corners as a quadrilateral, deliberately not a growing rectangle) and a resize
ghost. They are outlines rather than re-rendered content, which is the correct
trade and is documented as such.

### ★★ The one real gap the confirmation found

**`transform_preview` is never called.** It is the engine's preflight and it
distinguishes two refusals the UI must treat differently: `DegenerateCtm`
means *this object can never be transformed — do not offer a handle*, and
`SingularTransform` means *this particular drag collapses it — offer the
handle, refuse on release*.

`canvas/resizing.rs:172` admits it in the source, in these words:

> *"A handle is currently offered for an object that can never be transformed,
> and the operator finds out by dragging it. That is a real gap."*

It is unbuilt for a measured reason rather than an oversight: the preview
**decomposes the whole page** — about 4 seconds in a debug build on your
benchmark drawing — so it cannot be asked per frame. It needs a cache keyed on
`(page, edit epoch, selection)`.

### And one thing to watch in the file you send on

Every `transform_objects` call adds a fresh `q`/`cm`/`Q` wrapper **per object,
per gesture**, and nothing folds them together. Forty nudges nest forty
wrappers and the file grows monotonically. The shell already dodges this for
the common case — an all-path move takes the lighter `move_objects`, which
rewrites coordinates and adds no bytes — but a rotate or a resize cannot.

This row supersedes nothing. It **subsumes** `O20`'s rotate half and `O11`'s
rotate paragraph, both of which say the same thing more narrowly: the verb
exists and there is no grip to reach it with.

### The four things asked for, separated because they are in different states

| | what he wants | state |
|---|---|---|
| 1 | **Move / resize any object** | move and resize ship (`O11`, `O12`) — *"any"* is the part to verify, not the verb |
| 2 | **Rotate any object** | the engine verb rotates. **There is no rotate grip on the canvas**, for any object kind. Shell work, unblocked since 2026-08-20 |
| 3 | **Click individual nodes; select several and move them** | the Node tool (`A`) and a multi-node move both exist. Whether a *multi-node selection* can be built by pointing is the open question |
| 4 | **Live preview of everything** | move, resize and rotate ghosts exist in `canvas::overlay`. Whether every path has one is the open question |

### ★ On "any object", which is the word to be careful with

*Any* is the operator's word and it is the right requirement. It is also the one
most likely to be quietly false in a specific place, and this project has the
shape on record already: `O11` shipped move-and-resize while
`transform_objects` refused a **degenerate placement matrix**, and the engine
said in as many words *"do not offer a handle"* — so the shell offers one and
the operator finds out by dragging it.

So the confirmation must answer, per verb, **which object kinds it refuses and
why**, not merely whether the verb exists.

### ★★ Live preview is a standing expectation, not a per-feature request

> *"I've never seen a program that doesn't live preview any change, and yet here
> I am having to ask for all the minute details as if you'd never been trained
> on it."*

He has now reported the absence on three separate features. Treat *"with live
preview of everything if possible"* as the default requirement for every drag
this row touches, not as an optional fourth item — a rotate that only shows its
result on release is not finished.

★ And `ui-conventions/handles.md` H1 already says it: *selecting something shows
how to manipulate it*, before any drag.

### What this needs, in order

1. **Confirm the engine, per verb and per object kind**, with `file:line`
   against `D:\Dev\pdfce` source rather than against `docs/core-api/index.md`,
   which is a dated snapshot. Report what it refuses.
2. **The ninth grip** — rotate — painted and hit-tested from **one** predicate.
   `ui-conventions/handles.md` H7 and C7, and the trap this shell fell into
   once already: vertex handles painted for a selected dimension that could not
   be grabbed, because the painter asked about the selection and the hit test
   asked about a capability the mode lacked.
3. **Multi-node selection by pointing** — marquee inside an entered object, and
   Shift+click to add a node — feeding the multi-node move that already exists.
4. **A preview on every one of those drags**, and a check that each one
   *renders* rather than merely being constructed.
5. **The degenerate-matrix preflight**, so a grip that cannot act is not drawn
   (H7). Named in `O11` as needing a page decomposition cached per selection.

★ **Nothing here may be reported as working on the strength of a passing test.**
The Select button was green on 1,628 tests, 17 gates and a smoke launch on the
same day it reached him doing nothing at all.


## O20 — Dragging and rotating TEXT on the canvas

**Asked:** 2026-08-21 — *"I also can't drag and rotate text on the screen yet."*

**Status:** **RECORDED 2026-08-21. NOT STARTED.** Two separate things behind one
sentence, and they are in very different states, which is why they are written
out rather than merged.

### ~~Rotate — nothing to grab~~ — ⚠️ WRONG, AND WRITTEN BY ME, TODAY

**The rotate grip exists and has since 2026-08-20** (`560280a`). This section
was written on 2026-08-21 by reading `O11` and `O14` row 5 and trusting them
instead of the source. Both were stale; the claim was three weeks' worth of
true and one day's worth of false, and I re-published it as current.

★ **The lesson, which is the same one this file already carries twice:** a row
in the backlog is a record of what was true when it was written. It is not
evidence. `git log -S` and the source are evidence, and they cost a minute.

So the operator's *"I also can't drag and rotate text"* is **not** explained by
an absent grip. It needs driving. The candidates, none confirmed:

- **Grips are drawn at the Object rung only** (`overlay.rs:220`). If a click
  descended into the text object — to a run, or a caret — the box and its nine
  grips are gone, correctly, and there is nothing to grab.
- **A click on text may be arming the caret rather than selecting the object**
  — rung 2 of `clicking.rs`'s ladder beats rung 8.
- **The mode.** Content selection needs `edit_content`; Read and Review have
  no grips because they have no content selection.

The superseded text follows.

~~`O11` and `O14` row 5 both already say this and neither has been actioned:~~

> The verb rotates. **There is no rotate handle on the canvas to reach it
> with.** … it needs a ninth grip above the selection box, a drag that measures
> an angle rather than a distance, and a preview.

★ **Nothing is blocked.** The engine verb shipped 2026-08-20. This is entirely
shell work and has been since then. It applies to *everything* selectable, not
only text — a picture, a shape and a text run all have the same absent grip —
so the sentence *"I can't rotate text"* is really *"nothing on the canvas can be
rotated by pointing at it"*.

`ui-conventions/handles.md` H2 already specifies the shape: eight resize grips,
a body, and a rotate handle above the box. This is the "use the conventional
interaction, never invent one" case with the convention already written down in
this repository.

### Drag — claimed as shipped, never verified on text specifically

`O12` says text became draggable on 2026-08-20, through the same verb as an
image, and its own row ends:

> **NOT YET DRIVEN** on a text object specifically — the driven checks aim at a
> shape, because that is what the fixture's `--doc-point` names.

So there are two possibilities and this row does **not** guess between them:

1. It works and he has not found the gesture — which would make it a
   discoverability defect rather than a functional one, and those are fixed
   differently.
2. It does not work on text, and the row that claimed it did was claiming a
   verb rather than a behaviour.

★ Given the afternoon of 2026-08-21 — a Select button that did nothing at all,
green on 1,628 tests and 17 gates — **possibility 2 gets no benefit of the
doubt.** The first action is to drive it on a text run, not to reason about the
verb.

### What it needs, in order

1. **Drive a drag on a text object** at a `--doc-point` that names one, and find
   out which of the two above is true. Cheap, and it decides everything else.
2. **The ninth grip.** Painted from the same predicate that hit-tests it —
   convention C7 and H7, and the trap this shell has already fallen into once:
   *"a set of vertex handles was painted for a selected dimension and could not
   be grabbed, because the painter asked about the selection and the hit test
   asked about a capability the mode did not have."*
3. **A live preview during the rotate drag**, because he has reported the
   absence of live preview on three separate features now, and *"I've never seen
   a program that doesn't live preview any change"* is a standing expectation
   rather than a per-feature request.
4. The **degenerate-matrix preflight** named in `O11`: an object whose own
   placement matrix is collapsed cannot be transformed and the engine says *do
   not offer a handle*. Offering a grip that cannot act is exactly what H7
   forbids.


## O19 — In single-page mode, an option to turn the page when you scroll past its end

**Asked:** 2026-08-21 — *"also in single page mode I'd like a little checkbox
below that option to go to the next page when scrolling, and unchecked it keeps
its current behaviour."*

**Status:** **RECORDED 2026-08-21, NOT STARTED.**

A checkbox positioned **below the Single page option** in the page-display
group, so it reads as a qualifier on that mode rather than as an independent
setting — which is what it is: it has no meaning in Continuous, Facing or
Facing-continuous, where scrolling already crosses page boundaries.

- **Checked** — scrolling past the bottom of the page moves to the next page,
  and past the top moves to the previous one.
- **Unchecked** — today's behaviour exactly, which is that the scroll stops at
  the page's edge. **This is the default**, because it is what the shell does
  now and changing what an existing control does without being asked is the
  regression R6 forbids.

### The parts that are not decided, and are not to be improvised

These are the questions the class has already answered and they should be
checked against a real program rather than guessed:

- **Does the new page arrive at its top or at its bottom?** Scrolling *down*
  onto page 4 should land at page 4's **top**; scrolling *up* onto page 3 should
  land at page 3's **bottom**. Anything else teleports the reader.
- **Is there resistance at the boundary?** Every reader in the class makes you
  reach the edge and then scroll *again* rather than sliding straight through,
  so that a fast flick down a long page does not overshoot into the next one.
  Acrobat, Preview and every browser PDF viewer do this.
- **Does it interact with zoom?** At a zoom where the page is narrower than the
  viewport there is no vertical travel at all, so the first scroll event is
  already at the boundary. The resistance rule above is what stops that
  becoming "one wheel click skips a page".

### Where it lives

The page-display controls are `View` ▸ page display, mirrored by
`viewer::PageDisplay`. The checkbox belongs beside them and **not** in the
Settings window: it is a view control the operator changes while reading, in
the same group as the mode it qualifies.

## O18 — Ctrl+C on selected TEXT puts "1 object copied from pdfce" on the clipboard

**Asked:** 2026-08-21 — *"in the build from 9:50 this morning if I select text
in read mode, or edit and select text in an edit box in the canvas in edit
mode, and press ctrl+c to copy, then try to paste in notepad, it doesn't work.
I get a notice to paste it back into pdfc to place it."*

**Status:** ★ **CONFIRMED BY THE OPERATOR, 2026-08-21** — *"copy paste now
works!"* Fixed in all three places.

The driven check still does not exist, and the row stays open until you say
it can close. His confirmation is worth more than the check would be — he is
the oracle the check approximates — but the check is what stops it breaking
again without anyone noticing, and `CONTINUE.md` §2 item 1 still carries it.

What changed: `textsel::clipboard::pending_key` reads `Event::Copy` instead of
a key event that never arrives; `canvas/textedit/` gained Copy, **Cut and
Paste**, which it had none of; and `canvas::clipboard::text_owns_the_chord`
makes that module's oldest claim — *"text wins"* — actually true, so the object
path stands aside instead of writing its marker over the text.

`tools/gates/check-clipboard-chords.sh` now fails the build on any source file
that asks about `C`, `X` or `V` as a **key**. That gate is the part that
outlives this row: the real failure here was not the egui-winit quirk, which
had already been found and written up in capitals a day earlier, but that
nobody asked who else read the same broken signal.

★ **What to try, and what would still fail.** Sweep text in Read and Ctrl+C;
select inside a text box in Edit and Ctrl+C, Ctrl+X, Ctrl+V. A multi-line paste
will arrive as **one line** — the draft is single-line until O15, and that is
named here rather than left for you to find.

### The sentence he is seeing, and where it comes from

`crate::text::clipboard::os_marker` — *"1 object copied from pdfce. Paste it
back into pdfce to place it."* It is written to the operating system's
clipboard deliberately, by the **object** copy path, and it is not a bug in
itself: `egui-winit` synthesises a paste event only when the OS clipboard holds
non-empty text, so without *something* there, whether Ctrl+V works inside pdfce
would depend on what the operator last copied in another application.

The defect is that this sentence is reaching the clipboard when the operator
copied **text**, which should have put the text there.

### Case 2 — inside a text edit box. CONFIRMED, cause known

**`Key::C` is handled in exactly one place in the whole canvas**:
`canvas::textsel::clipboard::pending_key`. That function opens with

> *"A canvas draft claims these chords too … Ctrl+C mid-word must not copy the
> page's text selection: the operator is composing, and the selection they made
> before the caret landed is not what those two keys mean any more."*

and returns `None` whenever `canvas::textedit::composing()` is true.

That reasoning was right about what Ctrl+C must **stop** doing and never
supplied what it must **start** doing. `canvas/textedit/` has no Ctrl+C
handler of any kind — no `Key::C` appears anywhere in it. So inside a draft the
chord falls straight through to the ribbon keymap, which binds it to
`edit.copy`, which is the **object** clipboard, which writes the marker.

★ Note what this means precisely: **selecting text inside an edit box and
pressing Ctrl+C has never copied that text.** Not since the draft selection
shipped on 2026-08-21. The gesture is new; the gap arrived with it.

The shape is worth recording because it is a recurring one: a guard was added
to stop a chord doing the *wrong* thing, and stopping it was treated as the
whole of the job. The chord then had no owner at all, and fell through to
whatever claimed it next.

### Case 1 — a text sweep in Read mode. ROOT CAUSE FOUND, and it is the same one

The first draft of this row listed three candidates and said the convenient
answer was the one to distrust. It was right to, and all three were wrong.

**`Ctrl+C` never reaches `textsel` at all, in the real application, and never
has.** `canvas::textsel::clipboard::pending_key` asks
`InputState::key_pressed(egui::Key::C)`. In a real window that is permanently
false, because of fifteen lines of `egui-winit-0.35.0/src/lib.rs`:

```rust
if is_cut_command(modifiers, active_key)   { events.push(Event::Cut);   return; }
if is_copy_command(modifiers, active_key)  { events.push(Event::Copy);  return; }
if is_paste_command(modifiers, active_key) { … events.push(Event::Paste(contents)); return; }
events.push(Event::Key { … });
```

**The `return` comes before the `Event::Key` push.** So `Ctrl+C` produces
`Event::Copy` and *no key event whatsoever*. A function asking "was C pressed
with Ctrl held" can never be told yes.

★★ **This project already knew.** `app::keyboard` carries that exact quotation
under a heading reading *"CTRL+C, CTRL+X AND CTRL+V NEVER ARRIVE AS KEY EVENTS,
AND THAT IS WHY THEY HAVE NEVER WORKED"*, written on 2026-08-20 after the
operator reported the chords dead twice. That module was fixed: it translates
`Event::Copy` through the keymap, which is why `edit.copy` fires at all.

`canvas::textsel::clipboard` was not, and nobody noticed the second reader of
the same broken signal. So the finding was recorded, the general lesson was
written down, and the sweep's own copy went on being dead beside it.

### Why the two cases produce the marker rather than silence

With the text path dead, the surviving handler is `app::keyboard`'s: it turns
`Event::Copy` into the keymap's `edit.copy`, which is the **object** clipboard,
which writes `os_marker` — the sentence Ken pasted into Notepad.

So there is one defect wearing two faces:

| | why the text is not copied | what writes the marker instead |
|---|---|---|
| sweep, Read | `pending_key` reads a key event that does not exist | `Event::Copy` → keymap → `edit.copy` |
| draft, Edit | no `Ctrl+C` handler exists in `canvas/textedit/` at all | the same |

### ★ The transferable lesson, which is the expensive half

**A finding recorded in one module is not a fix applied to its siblings.** The
question that was never asked on 2026-08-20 is *who else reads this signal?* —
and the answer was one grep away: `Key::C` appears in exactly one other file.

The gate this suggests is a real one: **nothing in this crate may ask
`key_pressed` about `C`, `X` or `V`**, because the answer is always false. That
is checkable by a script, unlike the behaviour it protects.

### What it needs

- `canvas::textsel::clipboard::pending_key` reads `Event::Copy` (and `Cut`)
  rather than `Key::C`. ★ Its unit tests inject `Event::Key { key: C }` and
  **pass**, which is how this survived — they must be changed to inject what
  winit actually sends, or they will keep certifying a dead path.
- `canvas/textedit/` gains Copy, **Cut and Paste** for a draft's selection. All
  three are missing, not just the one reported: a text box you cannot paste
  into is the next report.
- A driven check per case that asserts **the operating system's clipboard**
  holds the expected text, having cleared it first. A trace line cannot see the
  clipboard, and the clipboard is the thing that is wrong.
- A gate forbidding `key_pressed` on `C`/`X`/`V` anywhere in the crate.

## O17 — Selection is governed by a FILTER on the status bar, not by two menus at the top

**Asked:** 2026-08-21 — *"Can we change how editing works? On the bottom bar I
want a filter menu that pops up with all the options of what to enable
selecting of — text, points, lines, etc — all the object types (glyphs beside
each option). … We should put a view one beside it that allows the changing of
what objects show bounding boxes around them on screen. … This is to replace
the wonky content edit text and edit objects menu at the top. … we should also
add a right click feature to select other for objects that are under another
object."*

**Status:** **PARTS A AND C BUILT 2026-08-21. THE POPUP SHIPPED BROKEN THE
FIRST TIME AND IS FIXED.** Parts B and D not started.

| part | state |
|---|---|
| **A** — the Select filter popup | built; **shipped inert, fixed 2026-08-21** — see below |
| **B** — the View twin (bounding boxes, node markers) | not started |
| **C** — what a click means per mode | built: the filter gates the hit test in all three modes |
| **D** — right-click *Select other* | not started |

### ★★★ The first build of part A did nothing at all, and how that happened

The operator, within minutes of opening it: *"I see a Select button, but this
should be a menu that pops up to choose what I can select on the screen and
edit in editor mode."*

The button was drawn, in the right place, and clicking it did nothing.
`egui::Popup::menu` is defined as `from_toggle_button_response`, which
**already toggles the popup open on click**. The code called
`Popup::toggle_id` as well, against the same id, in the same frame — so the
popup opened and closed before it could be drawn. From outside, a popup
toggled twice in one frame is indistinguishable from one that was never
wired up.

★ **What makes this worth writing down is everything that was green.** 1,628
unit tests. 17 of 17 gates. An offscreen smoke launch that confirmed the
button's rect was published at the exact intended spot on the status bar. All
of them observed the **button**, and the button was never the broken part.
This is R1 stated as an incident rather than as a rule: *the tests pass* is
not a report of working software, and it reached you because nothing had ever
opened the popup before you did.

There are now four tests that click the button and assert the popup's open
flag directly. **They were checked by re-introducing the defect and watching
two of them fail**, which is the only way to know a regression test tests
anything.

### The specification, unchanged

### What is being replaced, and why he calls it wonky

The two menus at the top ask the operator to **declare an intention before
pointing at anything** — *I am now editing text*, *I am now editing objects* —
and then hit-testing obeys the declaration rather than the drawing. That is the
wrong end of the gesture. It means the same click on the same pixel does
different things depending on a control the operator is not looking at while
they click, and it means reaching two levels into a ribbon to make a line
selectable.

Every program in the class solved this the other way round: **a persistent,
always-visible filter that says what is pickable, parked where it can be
glanced at without leaving the page.** AutoCAD's object snap and selection
filter, Illustrator's layer lock column, Inkscape's "Select Same" plus its
per-layer locks, Acrobat's own object-type restriction — the mechanism differs,
the shape does not. Ken's placement (status bar, popup on click, glyph per row)
is the CAD convention exactly, and per the standing rule the convergence of the
product class IS the specification.

### A — The Selection Filter popup

Lives on the **bottom status bar**. Click opens a popup listing **every object
class pdfce can hit-test**, each with a glyph and a checkbox. Enabled = that
class accepts a click. Disabled = clicks pass straight through it to whatever is
behind, in every mode, with no exception.

The class list must be derived from what the hit test can actually distinguish
today, not invented — text, glyph/character, path, line segment, node/vertex,
image, shape/annotation, ce dimension, form field, markup, link, and whatever
else the selection enum already carries. **Every class in the enum gets a row;
a class with no row is a class the operator cannot reach.**

Needs, at minimum: All / None, and the state must persist across sessions like
any other operator preference.

### B — The View popup, beside it

Same placement, same shape, different question: **what is drawn as a bounding
box or a node marker while unselected.**

- Text **off** = renders exactly as it renders. Text **on** = a box around each
  text run, always, selected or not.
- Objects **on** = a box around each object.
- Nodes **on** = the vertices of paths shown as markers, so they can be seen
  before they are aimed at.

★ **This is disclosure furniture, not content marking, and the distinction has
to be held.** Rule 4 forbids styling *applied content* to signal uncertainty.
It does not forbid an operator-controlled overlay that reveals structure — that
is the same category as a CAD program's grid or an editor's whitespace marks:
**off by default, switched on deliberately, drawn as chrome over the page and
never mistaken for ink.** The test still applies: with every View toggle off, a
screenshot of the canvas must be indistinguishable from the saved-and-reopened
document. That is what makes this safe.

### C — What a click MEANS, per mode

| mode | single click on a filtered-in object |
|---|---|
| **Read** | selects it. From that selection: clipboard operations, and form filling |
| **Review** | selects it, and permits editing of the things review owns — markup, comments, form fields |
| **Edit** | selects it, and permits editing of anything |

**In all three modes the filter is authoritative.** A class switched off in the
filter is not selectable in Read, not selectable in Review, not selectable in
Edit. The filter sits *above* the mode, not inside it.

★ **Open question, and it is a convention question rather than a preference
one:** whether entering edit-on-an-object in Edit mode is the single click that
selected it, or a second click / double-click. He named the alternative himself
— *"or double clicking if that is the more common convention"* — which is the
right instinct and the right person to be asked. **The class answer is
double-click**: PowerPoint, Illustrator, Figma, Visio and Acrobat all use
single-click-selects, double-click-enters. Single-click-enters exists mainly in
programs with no selection concept at all. Proposed, for his ruling: **click
selects, double-click enters the object's editor**, with Enter as the keyboard
equivalent on a selection.

### D — Right-click, including "Select other"

The escape hatch for the topmost-wins rule (`click-selects` C3). When objects
stack, the click correctly takes the top one; **Select other** walks the stack
underneath the cursor. The class convention is a submenu listing each candidate
by type and hovering it highlights that candidate on the page before committing
— Illustrator and Visio both do exactly this.

Other entries that make sense on a canvas right-click, to be specified rather
than improvised: Cut / Copy / Paste, Delete, Properties, Bring/Send order where
it applies, and the object's own primary action (Edit text…, Edit dimension…).
Right-click on **empty** page gets its own short menu — Paste, Select All, and
the two filter popups so they are reachable without travelling to the status
bar.

### What this row does NOT decide

- The exact class list — that comes from reading the selection enum, and any
  class the enum cannot currently distinguish is a **finding**, not a row to
  quietly drop.
- Whether the top menus are deleted or left as a redundant path during
  transition. Ken said *replace*; that is read as delete, but it is his call.
- Glyph choices.

### Why this is a bigger change than it reads

Every one of these touches the same function: **the thing that turns a click
into a selection.** Adding a filter, adding a stack-walk, adding mode-dependent
consequences and adding node visibility are four demands on one code path that
currently answers one question. `click-selects` C8 says the priority order must
be *written down in one place and testable* — this row is the reason that rule
now has to be honoured rather than noted, because four new claimants are about
to arrive at the same press.

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
**Status:** **SHIPPED 2026-08-21 AND DRIVEN.** Awaiting your verdict.

Put the caret in any piece of text and the navigation keys now work on **the
page**, not on the fragment you happened to click:

| key | where it goes |
|---|---|
| **↓ / ↑** | the line below / above — **including into the next block of text** |
| **End / Home** | the end / start of the line **you can see**, however many pieces it was drawn in |

A CAD sheet draws one visible row as four or five separate instructions, so
"the end of the line" and "the end of the thing I clicked" are different places.
End now goes to the first of those.

### It was SALVAGE, and the salvaged part was four lines

The old shell asked `pdfce-core` four questions — `caret_up`, `caret_down`, and
`line_range_at`'s two ends — and that was the whole of its contribution. **The
reassembly was always the engine's**: `recognize` groups a page's instructions
into lines and lines into blocks by column band, and `caret_up` walks *lines*.
So a caret on the last line of one paragraph steps into the next without
anything in the shell knowing what a paragraph is.

This shell had not been asking, and at the time that was right: its caret is a
position inside **one** run, and a single run has no line above it. What changed
is not the caret — it is that the *page* is now the thing being navigated.

### Driven

```
text-edit-caret  kind=Edit page=0 run=232 len=18     (a BOM row: "SW41177 - 22 - 250")
text-caret-step  dir=Down from_run=232 to_run=240 to_caret=8
text-caret-line  end=true from_run=232 to_run=236 to_caret=1
```

Down crossed into the row beneath and Up came back; End crossed into the rest of
the same row. Three different runs, all reached with the keyboard.

★ **The first live run failed, and the failure was not a defect.** No caret
movement at all — and the trace could not say whether the keys had been eaten on
the way in (a bug) or whether the model had simply found nothing above or below
(a fact about *where the click landed*, because the engine never crosses a
column band and a lone label has nothing stacked over it). Both look like
silence. The fix was a second trace line for the *nowhere* outcome, so the check
now **skips** on the second cause and accuses only on the first. A trace that
can only report success cannot tell a broken build from an unlucky fixture.

### Still open, and named rather than implied

- **Showing which block the caret is in.** The recognition is there; drawing it
  is a separate surface and R8b rule 4 governs how — off-canvas, never a mark on
  the page.
- **A remembered column.** Three presses down and three back up return the caret
  to where it started only when the lines are of similar length. A true desired
  column survives short lines in between; that is a second piece of state.
- **Up and Down inside a text BOX** still do nothing, deliberately: a box's
  lines are the shell's own wrap, and answering with the page model would throw
  the caret to a run somewhere else on the sheet.

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
5. ~~**No rotate handle** anywhere.~~ ⚠️ **CLOSED 2026-08-20 by commit
   `560280a` and not marked until 2026-08-21.** The ninth grip is painted and
   hit-tested from one predicate, with a ghost and 15° Shift-snapping. The
   struck text below is kept because the row's history is the point:
   ~~**the verb
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
11. ~~**No selection inside a draft** — no Shift+arrow, no Ctrl+A, no
    drag-select.~~ — **SHIPPED 2026-08-21 (keyboard half) AND DRIVEN.**
    Shift+arrows, Shift+Home/End and Ctrl+A select; typing replaces the
    selection, Backspace and Delete remove it, and any move without Shift drops
    it. The highlight is drawn under the text, in the theme's own selection
    colour, measured against the characters you can actually see.

    ★★ **And Shift very nearly did not arrive at all.** The first driven run
    moved the caret and selected nothing. With Shift physically held down
    through three presses, the toolkit reported it as *held* to one half of the
    program and *not held* to the other, on the same frame, three times running:

    ```
    ev=Modifiers::NONE  frame=Modifiers { shift: true }
    ```

    A key event is stamped with the modifier state at the moment it is
    translated, and the modifier state itself arrives as a separate event; when
    the two land together with the key first, the key carries nothing. The shell
    now asks both, and the reason it is safe to is an asymmetry rather than a
    preference: reading Shift as held a moment after it was released extends a
    selection by one character and the next press fixes it, while reading it as
    absent **destroys the selection** and no keypress brings it back.

    ★★ **AND THE POINTER HALF LANDED 2026-08-21 TOO — but it is NOT yet
    driven, and that distinction is the whole of this paragraph.** Drag across
    the text in the editor box and it selects what you crossed; double-click a
    word and it takes the word. Both are unit-tested against the **real** text
    layout — the same one the caret is drawn from, so where the pointer lands
    and where the caret appears cannot disagree — and the driven check that
    sweeps the pointer across a live draft on your own drawing is **written and
    has not been run**, because you came back to the keyboard and the harness
    takes the cursor.

    Until that check runs, this row is *built and unit-tested*, not *verified*.

    ★ Two things it deliberately does: a sweep that **starts** in the box and
    runs off onto the page keeps selecting to the end of the text, the way
    every text field does; and a press that starts on the **page** never
    becomes a text selection however far it is dragged into the box, so a
    marquee that happens to cross the editor is still a marquee.

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

    ★★ **AND THE OTHER THIRTEEN SHIPPED 2026-08-21.** About, Render
    diagnostics, Export to DXF, Insert image, Insert pages, New document,
    Recognise text, Apply redactions, Set scale, Keyboard shortcuts, Settings,
    the note editor and the unsaved-changes question. Every one has a title
    bar, a taskbar entry and can go on the second monitor.

    Three of them are worth naming because the window is the *feature*, not a
    tidy-up: **Apply redactions** lists what will be removed and you could not
    check it against the page it was covering; **Render diagnostics** is read
    while zooming the very document it describes; and the **unsaved-changes**
    question appears in answer to a close, when you have already looked away —
    a modal question hidden behind the main window with no taskbar entry is
    the classic *"the program has frozen"*.

    ★ **Nine of the thirteen had no size to convert.** They were content-sized
    windows, so no number for how big they are existed anywhere — and a
    guessed size that is too small does not look wrong, it clips the bottom
    row, which on a confirmation is the row with the buttons on it. So a
    dialog now measures its own body and grows the window to fit; the declared
    size is an opening bid. The first version of that grew About from 560 px
    to 1,624 px in a few frames, and a driven launch caught it the same hour.

    **Driven:** eight of the thirteen, one launch each, no mouse touched.
    Every one opened at its declared size and none needed to grow.

    ★★ **AND CONVERTING THEM BROKE THE VERIFICATION, WHICH IS WORTH SAYING
    OUT LOUD.** Six driven checks failed and six more skipped on the next full
    run — every one of them clicking hundreds of pixels away from the control
    it named, with no error anywhere, because a dialog in its own window has
    its own coordinates and the harness was still adding the application
    window's. All six are fixed and the harness now knows the program has more
    than one window.

    ★★★ **One of them was a real defect that had shipped: every dialog drew
    on a BLACK background.** Dark text on near-black, legible only as an
    outline. Nothing caught it — the window opened, every control was where it
    said it was, and the driven check for *"a dialog opens in its own OS
    window"* passed on all eight. **A screenshot showed a black rectangle.**
    That is the standing rule earning its place again: a rendering defect has
    exactly one oracle and it is a picture.

    **NOT VERIFIED**, named rather than implied: three of the five reachable
    only by a gesture — Insert pages, Set scale and the unsaved-changes
    question. The note editor is now driven, and see the row below for what
    that found.

    ★★ **AND THE LAST ROW OF THIS ITEM IS CLOSED TOO, 2026-08-21: a dialog
    can no longer fall behind the main window.** It stood open because the
    toolkit has no way to say *"this window belongs to that one"* — thirty
    options in its window builder and not one of them is an owner. pdfce now
    tells Windows directly, which is what every native dialog on your machine
    already does and why none of them has this problem. Confirmed on every
    dialog that opens: `dialog-owned owned=true`.

    Making it always-on-top instead stays refused, and the reason is worth
    keeping: it would break the driven checks in a way that produces confident
    wrong bug reports, and we have paid for one of those already today.
13. ~~**Enter is not the affirmative default**~~ — **PRINT SHIPPED
    2026-08-20**, and the pair is the host's, so every dialog converted on
    2026-08-21 inherited it. Type a page range, press Enter, it prints. Print is drawn
    filled in the theme's own accent so you can see what Enter will do before
    you press it, and Escape now closes the dialog exactly as the X does.

    The pair is drawn by the host, not by the dialog, so no future dialog can
    implement two of the three obligations and forget the third.

    **Known limit, named rather than found:** Enter is suppressed while a text
    field has focus, because the toolkit reports *"a text field has focus"*
    without saying whether it is multi-line — and a multi-line field must keep
    the ability to type a newline. So in a dialog whose last control is a
    one-line box, you may need to click out of it first. The fix is per-field.
13b. ~~⚠ **A note box may not take your typing until you click it.**~~ —
    **WITHDRAWN the same session, 2026-08-21. There was no such defect.**

    It was written up here in good faith and it was wrong, so the whole of it
    is left standing rather than deleted: this file is a record of what you
    were told, and a retraction that hides what it retracts is worth less than
    the mistake.

    **What was reported:** drag out a Text box or Sticky note, type without
    clicking the field, and the words go nowhere — with no message, because
    Accept is only enabled once the field has something in it.

    **What was actually true:** *the test* was clicking Accept through the
    main window's coordinates while the dialog had its own. You type, the
    dialog takes the characters correctly, and then the click that should
    commit them lands on the page instead. Converting that one line made the
    check pass. Typing into an unclicked note box has worked the whole time,
    and is now driven end to end — the check authors an annotation and
    confirms it changed the page.

    ★★ **The lesson, because it nearly cost a lot more.** Chasing the wrong
    culprit, the program was changed four times to hold the keyboard harder,
    and **each change appeared to help**: the dialog visibly held focus while
    the new code was asking for it and lost it the moment it stopped. That was
    real, repeatable, and completely beside the point. *A measurement that
    moves when you turn a knob is not proof the knob is the subject.*

    Two of those four changes were kept because they are right on their own
    terms — see item 12's *"the dialog can fall behind"* line, now closed —
    and the two that were only ever tuning were undone.

14. **PARTIAL, improved 2026-08-21.** Every dialog comes back where you left
    it, and it now **survives closing and reopening** — the position moved out
    of the dialog and into the application's own memory on the same pass that
    converted the other thirteen. It still does not survive a restart: a remembered position has to be checked against your
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


> ⚠️ **CORRECTION, 2026-08-21. THE ROTATE GRIP EXISTS AND HAS SINCE
> 2026-08-20.** Commit `560280a`, *"The ninth grip - you can turn things now,
> which was the third word all along"*, added `canvas/rotating.rs` (424 lines),
> `Grip::Rotate` (`canvas/handles.rs:175`), its hit test
> (`handles.rs:412`, ahead of the eight resize grips), its painter
> (`overlay.rs:222` via `draw_grips`), a rotate **ghost**
> (`overlay.rs:612`), Shift-snapping to 15°, and the commit through
> `transform_objects` (`canvas/rotating.rs:273`).
>
> The paragraph below was true when written and was **never updated**. It was
> then re-quoted, in good faith, into two rows written on 2026-08-21 —
> propagating a false claim rather than checking it against the source. That
> is the failure this file exists to prevent, committed inside this file.

~~The verb rotates. **There is no rotate handle on the canvas to reach it with.**~~
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
