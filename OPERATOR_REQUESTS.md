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

## O44 — Four things the first COMPLETE driven run found. Two were real, two were the test.

**Found:** 2026-08-26, by the first `ui-verify` run in this project's history in
which every declared check actually launched. **All four resolved the same day**
— and the honest tally is that **two were defects in the program and two were
defects in the tests**, which is worth stating plainly rather than counting four
fixes.

Evidence: `evidence/ui-verify-run-2026-08-26-rotated.txt` for the discovery run.
Every claim below was driven, and every driven claim was falsified first.

### ★★ O44a — The status bar's controls went off the window at a large UI scale. **REAL. FIXED.**

At `ui_scale = 1.80` the zoom control, the fit buttons, Find and the selection
filter were **off the left edge of the window at negative coordinates**, and the
left-hand notes were drawn underneath the fit group. Two points of the zoom
stepper and Find were also clipped off the *bottom*, at **every** scale
including 100 %.

Two independent causes, both now fixed:

* **The bar had no narrowing behaviour at all.** It now sheds, the way Word's
  and VS Code's do — biggest and least essential first. ★ The clause that makes
  that legitimate is enforced rather than promised: a control may only be shed
  if it has another home, checked against the real command registry. **That
  check immediately refused the obvious design** — which would have dropped the
  selection filter first, and the filter has no ribbon command, no menu entry
  and no shortcut. It exists only on that bar. Only the fit buttons and Find may
  go, and dropping the fit group alone is enough.
* **The bar was two points shorter than its own controls**, because its height
  was a constant written for 24-point controls and the shipped theme's are 28.
  It is taken from the theme now.

★ **A finding for you rather than a fix:** the selection filter and the zoom
stepper are reachable **only** from the status bar. If you would like either on
the ribbon, say so — it is one line each, and it would let the bar shed more
gracefully on a small window.

### ★★★ O44b — The Apply button for typed sizes could not be reached at all. **REAL, AND WORSE THAN FILED. FIXED.**

Filed as *"typing a width and pressing Apply does nothing"*. That was wrong in
an important way: **Apply was never pressed**, by anybody, because it could not
be seen.

The Properties panel drew its sections straight into the panel with **no scroll
area around them** — only the read-only metadata rows at the very bottom had
one, nested deep inside. So with an object selected you got Left, Bottom, Width
and Height, and Apply was **below the window edge with no scrollbar and no
gesture that would reach it**. The whole typed-geometry feature was complete,
wired, tested — and unusable.

★ It took a **screenshot** to see it. The coordinates said so all along and
three readings of them still reached the wrong conclusion. One scroll area round
the whole panel, and the driven check now scrubs the Width field, scrolls to
Apply, presses it, and watches the resize reach the engine.

### O44c — Shift-dragging pages between documents. **NOT A DEFECT. RETRACTED.**

It works. The run that reported it was given a **one-page** second document, and
a one-page document cannot be moved out of — by design, and no build could
change that. Re-run against a four-page document it passes end to end: the
target gains the page and the source loses it.

★ My error, not the program's: the check said so in its own skip message and the
first run reported the wrong half of it. The suite's invocation now uses a
multi-page second document.

### O44d — The Settings window published no headings. **REAL, AND NOT WHAT IT LOOKED LIKE. FIXED.**

It looked like a tracing bug. It was not. **Opening Settings showed nothing but
the list of rendering standards** — the presets section had grown to ten radios
and filled the entire window, so every group and every setting was below the
fold. The window was reporting no headings because there were none on screen,
which was exactly right.

The presets are now a collapsible group like every other section — still first,
because a preset sets all of them, but closed. Opening Settings shows the
groups again.

★ **Not changed to a dropdown**, though that is what most applications use for a
preset and it would cost one row instead of one click. You have just been given
the radio list and reported on it; swapping the control while fixing a layout
defect would be improvising. Worth proposing separately.

### ★ And two failures that were the TEST being wrong

Both would have gone on reporting red for ever on your own drawings, which
trains a reader to skip the section.

* **`blend_space`** claimed *"the page's colours have changed and nothing on
  screen says so"* about `SW41177.pdf` — a drawing with no transparency on it,
  which never asks for the colour buffer and is owed no disclosure. It computed
  the crossing from the page's **dimensions**, which says the buffer *would* be
  refused if the page asked for one.
* **`dimension_groups`** reported *"the panel declares no
  `dimension-groups.heading.add` region"* and then, in its very next sentence,
  **"Headings declared: dimension-groups.heading.add."** It contradicted itself
  in two consecutive lines: the region was there, and the dock was still
  settling so its position never held still long enough to click. A
  self-contradicting failure is worse than a silent one — it names a defect in
  the program for a condition that is entirely the harness's.

## O43 — Vertical text should behave like vertical text## O43 — Vertical text should behave like vertical text

**Asked:** 2026-08-26. **Shipped the same day. Driven check written and NOT YET
RUN** — see the status below, which says so in those words.

> *"I have text placed vertically on the bottom left corner of the SW41177.pdf.
> In Adobe when I hover over it the I cursor re-orients itself to match the text
> orientation, and when I select the text it shades each letter as part of the
> same block. when I copy and paste into notepad, I get the text on one line as
> expected. I need pdfcegui to have the same behaviour. as it is now the I
> cursor doesn't reorient and it pastes each letter onto its own line."*
>
> *"The last page has the vertical text."*

### ★★★ Three symptoms, one cause, and the cause is in the engine

`pdfce-core` places every glyph by the §9.4.4 text rendering matrix and then
publishes four numbers out of it: the origin `x`, `y` — exact — plus the
`advance` and the `size`, **both of which are lengths**. The two basis
*vectors* are reduced to their magnitudes, so **which way the text runs is never
published at all**. Its own rustdoc still calls the advance *"horizontal"*.

Everything downstream then assumes the missing vector is `(1, 0)`:

| symptom | mechanism |
|---|---|
| *"pastes each letter onto its own line"* | the extraction breaks a line whenever the baseline y moves. Text advancing in **y** changes baseline at every glyph, so it inserts a line break between every letter — **71 of them** in your stamp |
| *"shades each letter as part of the same block"* not happening | a glyph box is taken as `x … x+advance` across. For 90° text that is the right size turned the wrong way and hung off the wrong corner, so the wash sits *beside* the letters |
| the I-beam does not turn | nothing the cursor can ask knows the direction |

There is a fourth you did not report and would have hit next: **clicking on a
vertical letter lands on the wrong one.** The engine's hit-test boxes are built
the same wrong way, so a press in the middle of a letter is outside every box
and the nearest-line fallback decides. Found by driving it: a sweep down a
six-letter string selected five, and a sweep along an upside-down one selected
nothing at all.

### What was done here, and what was asked of the engine

The direction is **recovered from the glyphs themselves**, and the measurement
is exact rather than a guess: a *chain* of three or more consecutive glyphs each
sitting exactly one advance from the last, along a common direction off the
page's x axis, is not a coincidence available to horizontal text — within a
line every step is horizontal, and the jumps *between* lines are separated by a
whole line, so they can never be consecutive.

★ **A page with no rotated text on it never reaches any of the new code.** That
is structural, not incidental: the direction census comes back empty and every
branch is keyed on it. Asserted against a real drawing sheet.

The engine request is filed —
`open/request_extraction_drops_the_writing_direction.md` — asking for
direction-aware segmentation and for the direction to be published. When that
lands, the shell-side recovery becomes a fallback and then deletes.

### Status, stated honestly

| | |
|---|---|
| your own file | **verified.** The stamp on page 36 of `SW41177.pdf` comes back as one line: `W:\Engineering\Products\SAM\SW41177 Toyota Pick up ROPS\SW41177-WELDED FOPS.SLDDRW`. Run it yourself: `cargo test -p pdfce-gui --lib the_operators_own_vertical_stamp -- --ignored --nocapture` |
| unit tests | 22 new, against real extractions of a real fixture at 0°, 90°, 180°, 270° and 30°. Falsified in both directions before being quoted |
| **the driven check** | **WRITTEN, NOT RUN.** `ui-verify rotated_text_selects_and_copies_as_one_line` drives the release binary, sweeps the string, asserts `chars=6 quads=1`, asserts the cursor traced `deg=90`, and reads the OS clipboard from outside the process. It needs the pointer and the foreground, and you were using the PC. **Say when and it runs.** |
| the 30° case | the band it *marks* is a true parallelogram; the wash it *paints* is that band's bounding box, so it over-covers at the corners. Named rather than discovered. Quadrant rotations — every one a CAD exporter emits — are exact |

## O42 — Let me set the colour-blending buffer size myself

**Asked:** 2026-08-26. **Measured and filed with the engine the same day; needs
one change there before the setting can exist.**

> *"can the size of the buffer be increased? Allow the user to set the size up
> to the maximum possible?"*

**Shipped, 2026-08-26.** Settings ▸ Colour ▸ **"Colours changing when you
zoom"**. Type `default`, or a size — `512mib`, `1.5gb`, or a plain number of
bytes. It is uncapped, with no guard and no preflight, exactly the treatment you
chose for the zoom limit; the window states the cost and does not prevent the
choice.

★ **And you will very likely never need it**, because O41 below is fixed as
well: pdfce now stops asking for a page image bigger than the colour buffer can
handle, so the colours no longer change with the zoom at all. The setting is
there for the case the automatic fix cannot cover — a very large monitor, where
even the visible-part render can exceed the default.

**Driven:** `ui-verify blend_space`, and the funnel that carries the number to
the renderer has its own test — falsified by breaking the one line that carries
it, because a settings field that saves a number and changes nothing would be
worse than not offering it.

**What it would cost, measured** — corrected 2026-08-26, see below:

| you want correct colours up to… | one buffer |
|---|---|
| 579 % (the zoom you were testing) | 302 MB — barely above today's 256 MB |
| 800 % | 641 MB |
| 1035 % | 1.0 GB |
| 1200 % | 1.4 GB |
| every zoom pdfce allows on A4 (1946 %) | **4.0 GB**, plus the page image beside it |

**★ Two corrections to what I told you earlier, and both are mine.**

**(1) The percentages were labelled A4 and were not.** The page I measured on is
`596 × 791 pt`, which is neither A4 (`595 × 842`) nor Letter (`612 × 792`). The
mechanism and the bisection stand exactly as measured; only the label moves. On
real A4 the cap is reached at **518 %**, not 534 %, and the top of the whole-page
tier is **1946 %**, not 2071 %.

**(2) *"About 5 GB is the maximum"* is too low, which is the dangerous
direction.** That figure is for **one** buffer. A page with nested transparency
can hold several page-sized ones at once, so **peak memory can be about four
times the number you choose**. Pick with that in mind — the Settings control
will say so on its own line rather than leaving you to find out.

It also costs **about 50 % more time**: measured on the same page at the same
pixel count, blending in print colours took 1.4 s against 0.9 s. Correct colours
are slower, which is the actual trade.

**★★ And a finding that changes what I told you earlier.** I said the better fix
was for pdfce to render only the visible part above that limit, which needs no
extra memory. That is true on a small screen and **not true on yours if it is
1440p or bigger**: the visible part plus the margin pdfce renders around it
already needs 281 MB at 1440p and 633 MB at 4K — both over today's cap. So the
cap has to grow *as well*, or a big monitor gets approximate colours at every
zoom. Both changes are needed, not one.


## O41 — Colours change with the zoom level

**Asked:** 2026-08-26. **Cause found and disclosed the same day; the real fix is
filed with the engine.**

> *"seems I get different results depending on Zoom level. The [shading] boxes
> for example on zoom out the colors between our rendering and the references
> don't match, but they do when I am zoomed in. up to 474% they are mismatched,
> but at 579% they match. There's little problems like this in the rendering in
> others too, so probably all of them are related to one bug hopefully."*

**Your hunch was right — it is one bug, and your bracket contained it.**

pdfce blends a page that uses transparency in *print* colours (CMYK), which is
the correct way to do it. That takes a big working buffer, and the engine caps
it at 256 MB. Past the cap it falls back to blending in screen colours instead.
On an A4 page the cap is reached at **zoom 534 %** — between the 474 % where you
saw a mismatch and the 579 % where you saw agreement. Measured: crossing it
moves those patches by up to 16 levels out of 255.

**What you get today:** the status bar now tells you when it has happened —
*"Colours are approximate at this zoom … zoom out to see the exact colours."*
Nothing is marked on the page itself.

**FIXED, 2026-08-26, and the colours no longer depend on the zoom.**

pdfce should never have asked for a page image that big. Above a *different*,
much higher limit it already renders just the visible part instead — and a
visible-part render stays under the colour cap at any zoom. So the switch-over
now happens at the **colour** cap as well, and the page keeps its ink all the way
up.

**Driven, on the file that shows it.** At 801 % zoom — well past the 534 % where
this same page used to lose its ink — the trace now reads
`cmyk_buffer=true refused=0`. Before the change it read `refused=1` and the
status bar apologised. `ui-verify blend_space` asserts it, and its assertion was
falsified by disabling the mechanism and watching it go red.

### ★★★ The part worth knowing: it does NOT apply to your drawings

The obvious version of this fix applies the colour cap to every page, and it
would have been a serious regression **for you specifically**:

* on your own D-size sheet (1584 × 1224 pt) the cap falls at **263 % zoom** —
  well inside the range you work in;
* and that sheet is line work with **no transparency on it at all**, so it never
  asks for the colour buffer and nothing whatever would have been gained;
* about **0.4 %** of real documents use the buffer at all — 15 of 4,012 in the
  engine's own corpus.

So pdfce **learns** instead: the renderer reports, on every page image, whether
it blended in ink, and only a page that has been seen doing so gets the lower
cap. Your drawings keep free panning at every zoom, exactly as they do today,
and a print-ready file gets its colours fixed. Asserted both ways.

★ **`534 %` in the paragraph above was mislabelled as A4.** On real A4 it is
**518 %**. The bracket you gave — mismatch at 474 %, agreement at 579 % — still
contains it, and nothing about the diagnosis changes.

**Verified:** driven — `ui-verify blend_space` zooms past the crossing and
checks the line appears, and that it is absent below it.

## O40 — Only one standard was selectable in Settings

**Asked:** 2026-08-26. **Shipped:** 2026-08-26.

> *"in the settings for the standards compatibility I can only select
> (ISO15930-1, -4). I want to be able to select all of them and especially
> PDF/X-4 (ISO 15930-7)."*

**You were describing it exactly.** The control worked out which preset was
selected by comparing your settings against each one — and all eight of the
PDF/X and PDF/A presets set *identical* rendering answers. So whichever you
clicked, it matched PDF/X-1a first and the dot jumped back there. All nine are
selectable now.

★ **But it will not change what you see, and the window now says so.** Those
standards differ in what they require of the *file* — embedded fonts, an output
intent, whether transparency is allowed — which is a preflight question. What
they ask of a *renderer* is the same, so pdfce gives them the same answers.
Switching between them changes nothing on screen. Worth knowing before you use
it to compare against the conformance tests.


## O39 — All the form buttons working, and clicking a field shows its properties

**Asked:** 2026-08-26. **Shipped:** 2026-08-26.

> *"can you get all the form buttons on the ribbon working next along with
> adding all the form feature buttons. when I click one I should be able to
> click on the canvas to place the position or drag a box for size then a pop up
> lets me set the details for the feature."*
>
> *"remember last settings and leave push buttons on the ribbon but greyed out
> for now. also don't forget that when I click on an existing form field on the
> page it's properties should come up in our side pane for editing it's
> properties."*

**What you get.** Five buttons on Edit ▸ Forms — text field, check box, radio
button, drop-down, button. Click one, then click the page to place it at its
usual size, or drag a box for an exact one. A window asks for its details.
Nothing is added until you press Add, so a mis-drag costs nothing. The settings
you accept carry over to the next field you place. Click a field that is already
on the page and its properties appear in the Properties pane, where you can
rename it or delete it.

**The push button is greyed**, as you asked. If you reach it another way — a
keyboard shortcut, say — it now tells you why instead of doing nothing.

**Three things worth knowing:**

1. **In Edit mode, clicking a field selects it rather than filling it.** That is
   how every program that both fills and authors forms behaves, and it is the
   only way one click can mean both things. Filling on the page still works in
   Read and Review, and the Forms panel fills in every mode.
2. **Names must be different, and radio buttons are the exception.** Two fields
   sharing a name are ONE field with two boxes — type in either and both change.
   pdfce numbers new fields so that cannot happen by accident. Radio buttons in
   one set are *supposed* to share a name, so those keep theirs and get
   different values instead.
3. **Required, read-only, the tooltip and the border can only be set when a
   field is placed.** The engine has no way to change them afterwards yet. The
   Properties pane says so rather than leaving you hunting, and the request is
   filed.

**Verified:** driven. `ui-verify form_field` launches the real program, arms the
tool, clicks the page, watches the field get created, then clicks an existing
field and checks the Properties pane actually drew.


## O38 — A rendering preset for PDF/X-4 (ISO 15930-7) conformance, and a standards selector

**Asked:** 2026-08-25. **Investigated, not yet built.**

> *"I'd like a preset setting for rendering things to what the [print
> conformance suite] page needs to render correctly. We can't call it [that],
> but since it is for conformance to PDF/X-4 (ISO 15930-7)... I noticed touching
> some of our presets caused some test to show up as failed... maybe we should
> have a dropdown to select view options between the different standards."*

### ✅ Done immediately: the suite is no longer named here

We were naming it in two places. Both scrubbed, and
`tools/check-suite-name-absent.py` now fails the build if it comes back —
carried across from the engine, which had already made the same ruling. 18 gates.

### ★★★ MEASURED: the rendering settings change this file on every page

Not a theoretical concern. Rendering all six pages twice, changing **one**
setting — how images are sampled when drawn smaller than their pixel grid:

| page | pixels differing by >8 | worst channel delta |
|---:|---:|---:|
| 1 | 0.04 % | 95 |
| 2 | 0.27 % | 98 |
| 3 | 0.93 % | 139 |
| 4 | 0.31 % | 99 |
| 5 | 0.19 % | 64 |
| 6 | 1.02 % | 100 |

**Every page differs.** And the shape of the difference is the diagnostic part:
a *small area* changing by a *large amount*. That is not anti-aliasing spread
thinly over a page — it is specific patches shifting colour, which is exactly
what you described.

★ **Disclosure: I changed that setting today.** Image minification went from
point-sampling to smoothing this morning (O35), on your instruction, and this
measurement says that change moves every page of this file. Your report that
"touching some of our presets caused some test to show up as failed" may be
about your own change or about mine — the numbers above cannot tell us which,
but they do say the effect is real and worth pinning. **If you want it back the
way it was, it is one control in Settings ▸ Images and it stays changed.**

### What a preset is, and why it is the right shape

Not a new rendering mode — a **named bundle of settings that already exist**.
About seven of the twenty-three settings have a *render* radius, and each one
exists because the standard is genuinely silent and pdfce had to choose. A
preset says: *for this standard, choose these.* Everything stays individually
editable afterwards.

Your "dropdown to select view options between the different standards" is the
same mechanism with more than one entry, and that is how it should be built:

- **pdfce (recommended)** — today's defaults, including the two you ruled on
  personally (neutral black, and now image smoothing)
- **PDF/X-4 (ISO 15930-7)** — the conformance answers

### ✅ The mechanism is BUILT (2026-08-25), on your instruction to proceed

Settings ▸ top of the window, above every group because it sets all of them.
One entry today — **pdfce recommended** — which is the half of your request that
was never blocked: you had changed several settings while investigating and
wanted a way back.

★ It restores the two answers **you** ruled on personally rather than reverting
to the engine's defaults: neutral black for line art (2026-08-08) and smoothing
shrunk pictures (2026-08-25). A "recommended" preset that quietly undid your own
decisions would be resetting, not restoring, and there is a test that fails on
the day the engine adopts either — so the restatement can be removed rather than
silently becoming a no-op.

★★ **PDF/X-4 appears by adding one entry to a list, and nothing else.** No
control to write, no layout to touch. Until its values exist it is *absent*
rather than greyed — R9 — because a greyed row labelled with a standard's name
would carry that standard's authority with none of its content.

Verified on screen, offscreen: the row publishes `settings.presets` at
`614 × 117 pt` at the top of the window.

### ✅ SHIPPED 2026-08-25 — ten standards, and each says how much it can back up

The engine answered within the hour, and answered better than asked: not a table
of six values but an API, with **every value graded for evidence quality**.

**Ten choices** now: pdfce's own answers, plus PDF/X-1a, X-3, X-4, X-5g, X-6,
PDF/A-1, A-2, A-4 and PDF/UA-1.

★★★ **The important part is not the dropdown — and here is what it says.**
Choosing a standard now tells you how much of itself it can actually back up:

| standard | stated by the standard | inferred | chosen by pdfce |
|---|---:|---:|---:|
| PDF/X-1a | 4 | 0 | 2 |
| **PDF/X-4** | **1** | 2 | 3 |
| PDF/A-2 | 1 | 0 | 5 |
| PDF/UA-1 | *sets nothing — that is its answer* | | |

**Exactly one of PDF/X-4's six answers is stated by the standard it is named
after.** Anyone pressing a button marked ISO 15930-7 would reasonably assume
six. It also names what each standard does *not* reach — in the same words as
the controls further down the window, so you can go and look.

 Only **one** of PDF/X-4's six
answers is a claim about the standard at all, and even that one is *implied*
rather than *sourced*. So choosing a standard also shows what it does **not**
say — by name, not blank — and any disclosure it owes you, quoted from the
standard rather than paraphrased. A row that showed the name and hid the grading
would be exactly the over-claim this request was careful to avoid.

★★ **Your black-generation question turned out to be the wrong question, in a
useful way.** I filed it as *contentious* — your 2026-08-08 ruling versus a
conformance render. The engine's answer: **no setting of it is conformant**,
because every PDF/X level guarantees a measured definition of ink and this
control picks among fixed built-in tables. So the two were never in tension.
It is one control standing in for something pdfce cannot do yet, and the preset
says so on screen rather than leaving a colour conversion that silently did not
happen.

★ **PDF/UA is listed and correctly changes nothing** — measured, not assumed:
zero rendering requirements across all 197 of its rules. Listed rather than
hidden, because *"nothing, and here is the measurement"* cannot be mistaken for
unfinished work, whereas a missing entry can.

And the image-smoothing change from this morning is **gone as a special case**:
the engine adopted it as its own default, so the one-time migration deleted
itself exactly as designed.

## M1 — The PC starts pdfce unreliably. The laptop does not. It is the PC.

**★ SETTLED 2026-08-26 by your laptop test, and the conclusion is the useful
part: pdfce is exonerated.** The same portable build, the same files, works
normally on the laptop and fails roughly one launch in three on the PC. That is
a machine difference, not a program defect, and no more of my time goes on it.

**What this costs, and it is worth knowing rather than rediscovering:** the
automated test suite launches a fresh copy of pdfce for every check, so on the
PC about a third of them cannot start. Those show up as skips that look like
failures. Any future session driving the suite **on this PC** should expect that
and not go hunting.

★★ And my earlier diagnosis was **wrong**, which is worth stating plainly rather
than quietly dropping. I found OneDrive holding 404,000 file handles, established
by controlled test that my publishing was feeding it, restarted it at your
request, and watched the count fall to 1,179 — and the crashes **carried on at
the same rate**. So the handle leak was real and worth fixing, and it was not the
cause. Correlation, measured carefully, and still the wrong mechanism.

The publishing rule stays regardless: 27,000 handles per published build is a
genuine cost whether or not it crashes anything, and the rule is in the packaging
tool with the measurement beside it.

### Original report — the handle leak, which was real but was not the cause

**Found 2026-08-26 while testing. Not a pdfce bug — but it bites pdfce.**

Roughly a third of my automated tests could not start the program at all. It
dies before showing a window, with a Windows error about **"not enough memory
resources"** coming from the accessibility layer.

**Measured cause:**

| process | open handles |
|---|---:|
| **OneDrive** | **349,208** |
| Outlook | 51,751 |
| Explorer | 12,206 |

349,000 handles in one process is roughly a hundred times normal. Windows starts
refusing to hand out the resources a new window needs, and pdfce is simply the
next program that asks.

★ **It is intermittent, not constant** — three launches in a row gave two
successes and one crash, and pausing between them helped. So you may have seen
pdfce fail to open occasionally and put it down to bad luck. It probably was not.

★★ **What it costs you:** any program can hit this, not just pdfce. A restart of
OneDrive (or of the machine) will clear it. I have not touched it — that is your
sync and your call.

### ★★★ It IS me, measured — and I have changed what I do

I said I might be contributing. I tested it rather than leaving it as a guess,
by taking a reading, publishing nothing for half an hour, and taking another:

| period | publishes | handles gained |
|---|---:|---:|
| ~2 hours | 2 | **+55,000** |
| 32 minutes | **0** | **+6** |

Four orders of magnitude apart. **Each build I mirror to OneDrive costs your
machine roughly 27,000 handles, and OneDrive never gives them back.**

**So I have stopped publishing everything.** From now on a build goes to OneDrive
only when there is something you would actually notice — a fix you can feel, a
feature you asked for. Documentation, tests, refactors and engine re-pins with no
visible difference are commits only. The rule is written into the packaging tool
itself, with the measurement beside it, so it survives me.

★ **This does not undo what has leaked.** The 404,000 handles already taken stay
taken until OneDrive is restarted — which is worth doing, because at this level
roughly **one program launch in three fails**, and not only pdfce's.

★★ And the error message is actively misleading, which is why this was never
going to be reported as a pattern: Windows says *"not enough memory resources"*
while the machine has plenty of memory. It is **handles**, not memory — measured
at 404,179 handles against a 15 MB working set.

## E3 — OCR put every word in the wrong place on rotated pages

**Not asked — found by the engine and fixed.** 2026-08-26, commit `fe087c4`.

Scanned pages are usually rotated by the *scanner driver* writing a rotation
flag rather than by turning the pixels. pdfce honoured that flag when drawing
the page and **not** when placing the recognised words — so on any quarter-turn
page, every word ended up on the wrong axis at the wrong scale.

★ You could never have reported this, because there is nothing to see. The text
layer OCR adds is invisible by design, so a page with every word misplaced looks
identical to a page with every word right. The only symptom is that searching or
selecting picks the wrong thing — which anyone would blame on the recognition,
not the geometry.

**★ RE-MEASURED 2026-08-26, and the answer is better than the retracted one.**
Against the benchmark drawing's own text: 72 → 56.5 %, 100 → 56.7 %, 150 →
54.5 %, 200 → 53.9 %, 300 → **35.1 %**. So the headline you were given
originally is *confirmed* — **more scanning resolution makes OCR worse, and the
conventional 300 DPI is the worst of the five** — but the "150 is the sweet
spot" part was noise. The truth is that anything from 72 to 200 performs the
same, and then it falls off a cliff. pdfce's setting sits inside that flat
range, so nothing needed changing; it was right for the wrong reason and is now
right for a measured one.

These are still dense CAD drawings, which are the hardest thing to read. On
ordinary text the engine now reads a blurred, skewed, noisy scan at **47 of 47
words**.

**The original OCR accuracy figures were withdrawn.** The engine's bundled
text-detection model had never worked. The numbers I reported — including "150
DPI is the sweet spot at 44.7 %" — were measurements of noise, not of pdfce.
They are marked as retracted rather than quietly corrected, because the
*reasoning* behind them is probably still sound even though the values are not.
For scale, the fixed engine now reads a realistic synthetic scan at **47 of 47
words**. A proper re-measurement is outstanding.

## E2 — "Redact every match" could report success and leave the text in the file

**Not asked — found by the engine and fixed.** 2026-08-26, commit `a2518e5`.

The sibling of E1 below, and the dangerous one. Some PDFs store text with no
record of which letters it is — it renders and prints perfectly, and nothing can
search it. Ask pdfce to redact every occurrence of a name in such a file and it
would mark nothing, report success, and leave the name in the document. Then you
send it.

The redact panel now says so, in the strongest wording anywhere in pdfce: how
many fonts could not be read, and that any matches inside them **were not marked
and are still in the file**.

★ It is worded as a consequence rather than a mechanism, because on this one
operation there is no undo and no second chance to notice.

## E1 — Find said "no matches" over text it could never have searched

**Not asked — found by the engine and acted on.** 2026-08-25, commit `9f6ec1b`.

**The defect you would never have reported as a bug**, because it does not look
like one. A search can return "No matches" for a word that is plainly on the
page. Two situations produce that identical answer: the word really is not
there, or **the document stores its text in a way that records no letters** —
so nothing could ever have matched. The text renders perfectly. It prints. It
simply cannot be searched, and Find used to answer that with a confident "No
matches".

Find now says how many fonts in the document store unsearchable text, with a
hover explaining what that means and that recognising the page fixes it.

★ **Acrobat has exactly the same limit** and says nothing at all. This is a gap
in the *file*, not in pdfce, and the wording says so — calling a file's own gap
a tool limitation would send you looking for a better tool that does not exist.

★★ It appears in the Find bar, never as a mark on the page. Marking content that
renders correctly would be a second way of drawing the same thing, and two ways
of drawing one thing drift apart.

Engine re-pinned to v0.11.0 for it.

## O37 — All the font tools Word has

**Asked:** 2026-08-25. **Planned**, not started. `RIBBON_SCALING.md` §6c.

> *"We should also have all the font tools available that Word does."*

Deliberately not started alongside the scaling work, because it is a
**capability** question and not a layout one.

### ✅ Step 1 done — the inventory, read out of the engine source

★★★ **The headline, and it decides the whole shape of this request: pdfce can
choose how text looks when it is CREATED, and cannot change how existing text
looks at all.** `EditSession`'s text verbs are `add_text`, `edit_text` and
`delete_text_run` — and `edit_text` is find-and-replace that **re-encodes into
the run's existing font**. There is no restyle verb. (`set_font` exists, but it
is a low-level content-stream writer, not a session verb.)

| Word ▸ Home ▸ Font | on NEW text | on EXISTING text |
|---|---|---|
| Font name | ✅ 14 built-in faces, or embed any donor font | ❌ |
| Font size | ✅ | ❌ |
| Grow / shrink font | ✅ (arithmetic on the above) | ❌ |
| **Bold** | ✅ — as a *face*: Helvetica-Bold, Times-Bold, Courier-Bold | ❌ |
| *Italic* | ✅ — likewise: Oblique / Italic faces | ❌ |
| Bold + italic together | ✅ — the four combined faces exist | ❌ |
| Font colour | ✅ | ❌ |
| Alignment (L/C/R/Justified) | ✅ per text block | ❌ |
| Line spacing | ✅ (leading) | ❌ |
| **Change case** | ✅ | ★ **✅ — and this one is free** |
| Underline | ❌ as a text attribute | ✅ **as an annotation**, already shipped |
| Strikethrough | ❌ as a text attribute | ✅ **as an annotation**, already shipped |
| Highlight colour | ❌ as a text attribute | ✅ **as an annotation**, already shipped |
| Superscript / subscript | ❌ | ❌ |
| Character spacing / kerning | ❌ | ❌ |
| Text effects (shadow, outline, glow) | ❌ | ❌ |
| Clear formatting | ❌ | ❌ |

### Three findings worth acting on separately

**1. Change case is shippable today, with no engine work.** It is a string
transform followed by `edit_text` — and `edit_text` re-encoding into the run's
*existing* font, which is a limitation everywhere else on this table, is
exactly what makes case changing work: the glyphs stay in the same face. UPPER,
lower and Sentence case are the three worth having. This is real Word parity
for one afternoon.

**2. Three of Word's buttons already exist here, as something else.**
Underline, strikethrough and highlight are **annotations** in pdfce
(`markup.underline`, `markup.strikeout`, `markup.highlight`) rather than
character attributes. ★ That is not a lesser answer for a review tool — it is
arguably the right one, since an annotation is reviewable, attributable and
removable without touching the page content. But it means *"we should have all
the font tools Word does"* is already **half true in a way the Font group would
hide**: putting an Underline button in a Format tab that authored a text
attribute would create a second, incompatible underline. **Recommend: do not
add these.** They are on the Markup tab, where they belong.

**3. The real gap is one capability, not fourteen buttons.** Everything marked
❌-on-existing-text is the same missing verb: *restyle a selected run*. Bold,
italic, size, face and colour on existing text are one engine feature with five
front ends. Filing five requests would misdescribe it.

### Still to do

- ✅ **Step 2 done** — the engine hand-off is filed as
  `request_restyle_an_existing_text_run.md`, deliberately as ONE request. It
  asks the two questions only the engine can answer: whether a restyle is
  representable for an arbitrary run at all (swapping Helvetica for
  Helvetica-Bold changes every advance width, so the run reflows or overruns),
  and whether the honest scope is narrower — restyling only text pdfce itself
  authored, where the metrics are already known. If it is narrower, we would
  rather disclose a narrow capability than ship a wide-looking one.
- **Step 3** — the IA amendment. pdfce's text lives under Edit ▸ Content and the
  contextual **Format** tab; there is no Home tab, and the Format tab is the
  natural home for anything acting on a selection.

★ The target remains the capability list, not the pixel layout: *"everything
Word lets me do to text, pdfce lets me do to text"* — not a copy of two combos
and fourteen buttons onto a different selection model.

## O36 — Sections re-wrap onto more rows, and the scroll arrow is authorised

**Asked:** 2026-08-25. **Planned**, not started. `RIBBON_SCALING.md` §6a, §6b.

> *"put it in the plan to update so that tools within sections will re-wrap
> onto more rows when I resize, and do the scroll like Word. BTW the Font
> section in Word will wrap tools onto 3 lines when the window is narrowed
> enough, and other tools wrap in a similar way too."*

★★★ **He corrected a factual claim of mine, and he was right.** I had written —
in this file's O33 row, in a module header and in a commit message — that Word
does not re-wrap groups by window width. It does: the Font group is 2 rows at
1900 pt and 3 rows at 1000 pt, and **both photographs were already in
`evidence/word-ribbon/`** when I wrote the opposite. I had compared 1300 with
800, and by 800 the group has already collapsed, so the reflow appears in
neither frame. Sampling either side of a transition and concluding there is no
transition. O33's answer is corrected on the record rather than quietly edited.

**Scroll arrow: settled, no longer a question.** It replaces the `⏷ N more`
dropdown rather than joining it. Sequenced *after* the re-wrap, because
re-wrapping will move the width at which the dropdown appears again and there
is no sense tuning a scroll step against a threshold that is about to change.

## O35 — Image quality worse than Acrobat on normal pages

**Asked:** 2026-08-25. **Shipped:** 2026-08-25.

> *"there was also an update to an image quality setting to discard smaller
> details than the screen sees a while ago that I think has been enabled by
> default because image quality is a little worse on normal pages than it was
> whereas before it was on par with acrobat reader — this setting should be an
> option in our settings and disabled by default."*

**You named the mechanism exactly.** Images drawn smaller than their own pixel
grid were point-sampled: one texel per output pixel, the rest discarded. That
is every scan and every CAD raster at anything under 1:1, and the engine's own
note on it says *"aliasing, shimmer, dropped hairlines"*.

**One half of the report was a hypothesis and it was wrong, which is worth
saying because it would have sent me hunting.** It was not enabled recently —
it has been the shipped default all along, and wiring the setting through the
GUI changed nothing, because the render options and the settings file carry the
*same* default. So there is no regression commit to find; there is a default to
decide, and that decision is yours and is now made.

It was already an option (Settings ▸ Images, *"Shrinking a large image to
fit"*). What was wrong was the default. Changing only the default would have
fixed **nobody**: every real installation already contains an explicit
`image_minify = point_sample`, written by our own save into the engine's
generated template — both of your settings files did. So it ships as a
**one-time migration** with a marker in `preferences.txt`: flipped once,
recorded, and if you ever set it back it stays back.

**Verified by driving** three cases: unmarked installation flips and records;
second launch does nothing; marker present plus a deliberate `point_sample`
survives untouched.

**Engine hand-off filed** — `pdfce-core` grades its own default "a guess" and
names the exact evidence that would flip it: a viewer-behaviour comparison.
You just supplied one, against Acrobat, on your own drawings.

## O34 — The print dialog grows for ever after printing

**Asked:** 2026-08-25. **Shipped:** 2026-08-25, commit `deb9853`.

> *"the print dialogue has a bug that when I press print, instead of closing
> after printing it just keeps expanding its size in little steps to infinity."*

It did. The footer drew its buttons first and the *"Sent N pages"* message
after — and the button pair uses a right-to-left layout, which anchors to the
right edge of whatever width it is offered whether it needs the room or not.
Anything placed after it lands past that edge. The dialog host then grows a
window whose content is wider than it is, the wider window offers a wider row,
the message lands past the new edge, and round it goes.

**Measured:** in a 400 pt row the old ordering produced 481.9 pt and the fixed
ordering produces exactly 400.0. That 81.9 pt was the step, once per frame, for
as long as the dialog stayed open.

The message now draws first — status left, actions right, which is the Windows
arrangement anyway. Separately, the dialog host gained a **growth budget**: any
dialog that asks to grow more than three times stops and records why. Two
existing guards were satisfied throughout this bug and neither helped, because
a guard against *repetition* cannot see monotonic creep — creep never repeats.

**Verified:** four unit tests including one that reproduces the overflow in a
real laid-out frame and fails on the old ordering. **NOT driven** — reproducing
it end-to-end would mean sending a job to your printer.

## O33 — Does the ribbon get the scroll arrow, and do groups re-wrap?

**Asked:** 2026-08-25. **Partly shipped** 2026-08-25, commit `10877a1`.
**One decision open — yours.**

Two questions, and they have different answers.

**"Will it wrap tools in their sections onto second lines when the window is
resized?"** They already wrap onto a second row, but by the *group's own
content width*, not by the window — and **Word does not re-wrap on resize
either**. Its Font group keeps the same two rows at 1900 pt and at 1300 pt.
What Word does instead is collapse whole groups, and that is what shipped
today: at 1600 one group collapses, at 1100 four do.

**"Does this include replacing the ⏷ N more dropdown with an arrow that shifts
the ribbon horizontally?"** That is S4 and it is *not* built. It is still the
plan and Word does exactly it — a `›` at the band's right edge, which appears
at 460 pt and not before. But the collapse ladder changed the argument, and the
number is the reason this row exists:

| window width | `⏷ N more` before today | after |
|---:|---|---|
| 1600 | yes | **no** |
| 1200 | yes | **no** |
| 1100 | yes | **no** |
| 1050 | yes | yes |

**The dropdown used to appear at 1600 and every width below. It now appears
only below about 1100.** So S4 would replace an affordance you will rarely see
— and it would replace it with something *less* informative, because a menu
names what is hidden and an arrow makes you hunt for it. Against that: the
arrow is the convention, you have asked for it twice, and a band you can scroll
never hides a group's caption.

**What I need from you:** say the word and the arrow replaces the dropdown. It
touches six tests and one driven check, so I would rather do it in a session
where I can drive the running application to prove it, which needs the machine.

## O32 — The commands whose tab was decided by mode exposure, not by subject

**Asked:** 2026-08-25 —

> *"the current commands for each are fine as is for now. there were just some
> commands you made a decision to put in a different tab than where they would
> normally go because exposure was tab based and not command based."*

**Status:** **FOUND AND LISTED. The operator's decision, per command — nothing
moved.** The mechanism that forced them is gone; whether each *should* move is
a `RIBBON_IA.md` question and the IA is his.

### The mechanism, and exactly what changed about it

A mode names **tabs**: Read is `["file", "view"]`. So a command on a tab Read
does not show is not merely inconvenient there, it is **unreachable** — no tab,
no band, no control, and `modes::capability::offers_command` refuses its chord
too. Four commands were therefore homed on File or View instead of where their
subject says they belong, and the codebase names the pattern and calls it a
rule:

> *"a command refused in a mode where the operator plainly needs it is evidence
> that the command's tab is wrong, not that the mode gate needs an exception."*
> — `RIBBON_IA.md` §5.7

★★★ **What changed, precisely:** `visible_when` (O31) hides an **item**. It
cannot make a **tab** appear. So on its own it does not undo any of these. What
undoes them is the pair — `visible_when` plus *a tab with nothing left to show
is not shown* — which together turn `Mode::tabs` from *"which tabs exist here"*
into *"which tabs may appear here"*. A mode can now be given a tab generously
and shown only the part of it that applies.

So the move is buildable now. It was not before, and the cost is no longer
"Read gains batch merge, split and font embedding" — those would simply be
hidden.

### The four, and what each would cost to move back

| command | where its subject says | why it is where it is | moving it back would |
|---|---|---|---|
| **`file.ocr`** | Tools ▸ Recognise (`RIBBON_IA.md` §5.7) | operator: *"if in read mode ocr should still be available"* | give **Read a Tools tab** showing one command |
| **`file.copy_page_text`**, **`file.copy_document_text`** | Edit ▸ Clipboard (§5.1, §5.4, §7) | `Ctrl+Shift+C` was refused in Read, *"a mode whose whole standard is Acrobat Reader, which copies text"* | give **Read an Edit tab** showing two commands |
| **`view.panel_forms`** (was `edit.form_fill`) | Edit ▸ Forms | operator: Read fills forms, because Acrobat Reader does | needs the command **re-invented**: it stopped being a verb and became a *panel toggle*, and panels live on View |
| **`view.tool_text`** | — | placed on View ▸ Navigate **pre-emptively**, to avoid being the fourth instance | nothing: it is a pointer tool beside select, node and hand, which is a coherent group on its own terms |

### ★ The recommendation, per command — and it is not "move them all"

Because on inspection **each landed somewhere defensible**, and two of them are
now better placed than the specification's original:

* **`file.ocr` — leave, and amend the spec.** File ▸ Recognise sits beside the
  verbs that make a document exist and write one out, and OCR's product is a
  new file. Tools answers *"what do I run **across files**, or configure
  once?"* — and OCR runs on **this** file. The spec's Tools placement is the
  weaker of the two, and it was written before the tab questions were.
* **`file.copy_*` — leave.** *Copying is not authoring*: it reads the page and
  writes to the clipboard, and cannot change a byte. File ▸ Export groups it
  with the other verbs whose destination is outside the document. ★★ And the
  alternative is worse than it sounds: a mode called **Read** showing a tab
  called **Edit** contradicts the stance the mode exists to state, even if the
  tab holds nothing but Copy.
* **`view.panel_forms` — leave.** Not a tab move to undo. It is a panel
  toggle, and every panel toggle is on View ▸ Panels.
* **`view.tool_text` — leave.** A pointer tool, in the group of pointer tools.
  `edit.text` (change text that is already there) is a different verb and is
  correctly on Edit.

### ★★★ What is genuinely wrong and should be fixed either way

**`RIBBON_IA.md` is internally inconsistent.** It records the OCR move in §5.7
and names the rule — and §5.1, §5.4 and §7 were never updated for the two
earlier ones. The spec still says today:

> §5.1: *"**Moved off this tab:** `Copy this page's text` and `Copy the whole
> document's text` go to **Edit ▸ Clipboard**"*
> §5.4: `| | Copy page text · Copy document text | **G** *(from File)* |`
> §5.4: `| **Forms** | Fill form | **G** |`
> §7: `| Edit ▸ Forms ▸ Fill Form | Edit ▸ Forms |`

Three sections route commands to a tab they have not been on since 2026-08-14.
Whatever is decided about moving them, the specification should say where they
**are** — a settled document that disagrees with the build is worse than an
unsettled one, because it is read as authority.

### One inverse case, for completeness

`markup.underline`, `markup.strikeout`, `markup.squiggly` are the **opposite**
problem and are already recorded as an accepted inversion: they are on their
natural tab and were permanently greyed in Edit, *"not fixable by hiding them,
because the Markup tab is in both Review and Edit and a command has one tab"*.
★ That is now fixable — `visible_when` is exactly the missing mechanism — but
the tension closed on its own when `CanvasTool::Text` landed, so there is
nothing left to fix.

## O31 — Improve the ribbon: learn from Word

**Asked:** 2026-08-24 —

> *"can you improve the ribbon bar? if you can learn how word handles when to
> have text labels, organization on two rows for some commands, and how it
> handles narrowing the window. for one thing it puts an arrow at the end to
> press to move over if there isn't room for all commands. also we should have
> flexibility to show or hide and commands and shift the space used depending
> on what exists. this would allow greater flexibility of where to place
> commands for read, review and edit modes, as what remains shown can be mixed
> on tabs. if you can, drive word as it is installed on this machine."*

**Status:** **RESEARCHED AND STAGED. S1 and S2 done 2026-08-24; S3 and S4
designed and not built.** The whole of it is `RIBBON_SCALING.md`.

### Word was driven, and it had to be photographed rather than asked

Word's ribbon scaling rules are **not in its object model** — `CommandBars` is
the 2003 toolbar surface and says nothing about the ribbon, which is RibbonX
compiled into the product with its behaviour inside the Office UI framework.
So `tools/word-ribbon-study.ps1` sets a window width, waits for the re-layout
and captures: twelve widths, 1,884 down to 444, largest first, because Word
re-lays-out incrementally and a growing series would photograph the *recovery*
path. `tools/our-ribbon-study.ps1` is its twin, pointed at our own build.

### ★★★ The measurement that decided the work

Groups reachable on the band without opening a menu:

| client width | Word | pdfce, before |
|---:|---:|---:|
| 884 | **10** | **3** |
| 604 | **7** + a scroll chevron | **1** |

**Our overflow was not the problem.** The `⏷ N more` affordance is the arrow he
is describing, it works, and it is tested at every width. It was starting far
too early, because every control in the band was icon-plus-label and a group
had no way to give up space except to vanish.

### What landed

* **Three item sizes** — Large (icon above label, spans the rows), Medium
  (today's), Small (icon only). Declared per item in the manifest, defaulting
  to Medium, so a manifest that says nothing renders identically.
* **Small is earned**: it needs an icon, a tooltip *and* an installed painter,
  or it falls back to labelled. The tooltip is the icon's accessible name.
* **`visible_when` on an item** — hidden **before measurement**, so the space
  is reclaimed and the group re-flows; a group with nothing left is not drawn
  at all, separator included. That is his second paragraph, and it is what will
  let one tab definition serve Read, Review and Edit.
* Applied to pdfce's manifest: icon-only for the four page displays, four
  pointer tools, five display toggles, two page rotations, cut/copy/paste, four
  text markups and seven markup shapes; Large for the six one-item groups.

★★ **The File tab is deliberately unchanged**, and that is the finding worth
keeping. Its commands are *named things* — "Export form data…", "Save a
copy…" — not iconic ones, and `band.rs`'s original argument was right about
exactly that case. Driving Word showed the argument is about **the command**,
not about the band.

### What is designed and not built

`RIBBON_SCALING.md` §5.2 and §6: **per-group collapse in an authored order** —
each group in turn becoming a single captioned button with its full layout one
click away, which is what Word actually does — and **scrolling** as the last
resort beneath it rather than the first. Both touch `plan_band`'s invariants
(`the_visible_groups_are_a_prefix_and_nothing_is_lost`), which is why they are
staged rather than rushed.

★ One open question for the operator, and it is his to answer, not this
project's: **which commands should differ between Read, Review and Edit?**
`visible_when` is built and tested and nothing uses it yet, because deciding
what appears where is `RIBBON_IA.md`'s territory and the IA is settled.

## O28 — A fit control must place the view, not only set the scale

**Asked:** 2026-08-24 —

> *"If I press the Fit width or fit page button the view should center to the
> width as well or center the page."*

**Status:** **FIXED 2026-08-24**, and driven —
`a_fit_command_puts_the_page_on_screen` pans thirty notches into the pasteboard
before pressing each button, asserts it got there, and then measures the page's
drawn rect against the canvas's. Measured after Fit page: page
`296,272 .. 764,633` in a canvas of `288,143 .. 772,762` — margins of 8 and 8
horizontally, 128.5 and 128.6 vertically. **Falsified**: with the placement
disabled the same run reports *"part of the page is outside the canvas; the
vertical margins are 261.5 and −4.4, so the page is not centred"*.

★ **This is a consequence of O23's pasteboard and it is the second one.** Before
the pasteboard, a page smaller than the viewport had nowhere to be except the
middle, so "fit" and "centred" were the same act and nobody had to decide which
one the button meant. The pasteboard added a whole viewport of slack on every
side — deliberately, so any corner of the page can be brought to any point of
the screen — and with it the state the operator is reporting: **the scale is
right and the page is not on screen.**

So `Action::Fit` sets the scale and must now also **place the view**:

| | |
|---|---|
| **Fit page** | centred on both axes. The page fits, so there is exactly one honest position for it |
| **Fit width** | centred horizontally; the vertical position is kept but clamped to the page's own range, so you do not lose your place in a long sheet and cannot be left looking at pasteboard |
| **Fit height** | the mirror: centred vertically, horizontal kept and clamped |

★ Keeping the other axis rather than resetting it to the top is deliberate.
"Fit width" on page 12 of a drawing set is a *scale* request; throwing the
operator back to the top of the sheet would be a navigation they did not ask
for. Clamping is what makes "kept" safe.

## O29 — Fit height, because Acrobat has it

**Asked:** 2026-08-24 — *"Adobe has fit height, so add that too."*

**Status:** **FIXED 2026-08-24**, and driven in the same check. Measured after
Fit height on the 1584 × 1224 sheet: page `288,151 .. 1068,754` in a canvas of
`288,143 .. 772,762` — the full height on screen, and the width overflowing by
296 points, which is the mode doing exactly what it is for.

A third mode beside Fit page and Fit width: recompute the zoom each frame so
the page's full **height** is visible. On a landscape CAD sheet in a portrait
window it is the useful one, and it is the mode this build has been missing
every time the operator wanted to read a title block down the right-hand edge.

Scope, taken as the whole expected behaviour rather than the sentence:

* the mode itself, recomputed on every window resize like its two siblings;
* the **status bar** control beside Fit width and Fit page;
* a **registered command** so it appears wherever the other two do — the ribbon
  included — because R8 makes registering the command the only way the shell is
  allowed to learn a capability exists;
* the **opening-fit preference**, so a document can be opened at fit-height the
  same way it can be opened at fit-width;
* the on-disk id, its round trip, and the exhaustive-variant tests that would
  otherwise pass while silently not covering it.

## O30 — In single-page view, choose what the wheel does

**Asked:** 2026-08-24 —

> *"when in single page view there should be an option on screen near the
> button to scroll or flip through pages, or the current way it is now when the
> scroll wheel is used."*

**Status:** **FIXED 2026-08-24**, and driven —
`the_wheel_turns_pages_when_the_operator_asks_it_to` makes five separate
claims, in order: the default is **silent** (a build that flipped
unconditionally could not pass), the toggle is on screen beside the page
buttons, the **very next** notch turns a page, rolling back returns to the page
before, and under a continuous display the control is **not drawn at all**.

★ Two defects were found by writing it, and neither was in the feature:
the check **mutates a persisted setting and did not normalise at the start**,
so its second run inherited its first run's toggle and accused the shipped
default; and its absence claim used `declared_since` with an event count where
that helper wants a line number, reporting a control as drawn when it was not.
Both are the standing lessons in a new costume. The application now publishes
`wheel=` on its status line so the check can read the state it is about to
change.

Two behaviours, chosen by a control **next to the page navigation buttons** in
the status bar:

| | |
|---|---|
| **Scroll the page** | today's behaviour: the wheel moves within the sheet and never leaves it |
| **Flip pages** | the wheel turns to the next or previous page |

★ **The control renders only where it means something** — R9. Under a
continuous display mode the wheel scrolls the whole document by definition and
there is no choice to offer, so nothing is drawn rather than a disabled stub.
Under Single and Facing it appears beside the page box, which is where the
operator is already looking when they are thinking about pages.

★ It is an operator setting and therefore persisted, like every other view
preference: a choice that resets on the next launch is a choice the operator
has to keep making.

## O26 — Zoom out throws the page off screen into a corner

**Asked:** 2026-08-24 —

> *"the zoom in function works flawlessly now. The panning works. Zoom out has
> a small bug where it sometimes seems to reposition the page so that it is off
> screen in the far bottom left corner. This happened when I zoomed back from
> around 2 million% but seems to happen at other junctions too."*

**Status:** **SEVEN CAUSES FOUND AND FIXED, 2026-08-24**, in two clusters:
O26a-d below, which relocate the page at ordinary zooms and were never about
zooming out in particular, and O26e-g, the missing hand-over out of the `f64`
position tier. Every one of them moves the page by a whole page or more.
Driven, with pixels for the first. A residual is filed separately as O27.

★★★ *"Seems to happen at other junctions too"* was the load-bearing half of the
report and it was right. The 2,000,000 % crossing was **one** of seven
independent faults with the same symptom, and it was the least often reached —
three of the other six are reachable at 30 %.

### O26a — one wheel notch at 30 % took the view from page 1 to page 8

**★ Found by pixels, in the first thirty seconds of driving**, and it is the
worst of the four. `Strip::page_at_view` takes a **strip-space** rect. It was
being handed `scroll_output.state.offset` — a **content-space** offset, which
since O23's pasteboard sits a whole viewport above and to the left of the
strip's origin.

**This is the second site of the omission O23 spent four attempts on.**
`geometry::scroll_to_strip` was added then, for `visible_rect`, and nobody
swept for the other callers.

Two failure modes, and the silent one had been shipping for longer:

* **No page at all.** The horizontal error is a whole viewport and the strip is
  only as wide as its widest page, so the displaced box usually misses the
  strip entirely, `page_at_view` returns `None`, and the branch never runs.
  **Scroll-driven current-page tracking — Phase 4.3, the whole reason the block
  exists — has been inert since the pasteboard landed.** Nothing said so; the
  page number simply stopped following the scroll.
* **The wrong page**, whenever the strip grows wide enough for the displaced
  box to clip its right-hand edge. That is a function of the zoom, so it
  arrives at one particular magnification and not the ones either side of it.
  **That is the operator's "other junctions".**

And a mis-reported page is not cosmetic, because `current_origin` — the frame
of reference every single-page solve in `canvas::zoom` and `find::reveal` is
handed — is *that page's* origin in the strip. Set it to page 7 and the next
anchored zoom converts its answer back through page 7's origin, so the view
moves by seven page pitches in one wheel notch.

Measured on `SW41177.pdf` at 30 %, one Ctrl+wheel notch: `page` 0 → 7,
`off` [484, 490] → [514, 2767], and the status bar read `8 / 36`. Screenshots
before and after are the evidence; no trace field says *"the wrong page"*.

### O26b — and then the wheel stopped zooming altogether

`if image_response.hovered()` gated Ctrl+wheel on the **acting page's** own
response. Three ordinary positions were therefore inert: the pointer over a
*different* visible page, the pointer in the gap between two, and the pointer
over O23's **pasteboard** — a whole viewport of it on every side, added
deliberately so any page corner can be brought to any point of the screen, and
therefore a position the operator is now *expected* to be in.

★★ It is also what turned O26a's catapult from a lurch into a **freeze**. Once
the tracker had thrown `page_index` seven pages down the strip, the acting page
was off screen, nothing under the pointer was it, and every subsequent
Ctrl+wheel did nothing at all — five further notches produced a byte-identical
trace. A view that jumps is a bug; a view that jumps and then will not zoom
back is what gets reported.

The gate is now the scroll area's own content response, which covers pages,
gaps and pasteboard, and which — being a real `Response` — still lets a
floating window over the canvas swallow the wheel. A `rect.contains` test would
not have.

### O26c — the acting page's rect and the acting page's extent were different pages

`acting` was `doc.view.page_index`, decided *before* the fallback that picks
`drawn.first()` when the current page is not among the drawn ones — and then
never revisited. The next two lines paired **that page's rect** with **the
current page's extent**.

On a document whose sheets are all one size the mismatch is invisible.
`SW41177.pdf` mixes 1584 × 1224 sheets with 1224 × 792 ones, and the trace
caught it exactly:

```text
canvas rect=[[-5634238.0 681671.0] - [5515170.0 7895993.0]] zoom=9108.99
canvas-pos … ext=1584.000,1224.000
```

11,149,400 × 7,214,300 is 1224 × 792 at that zoom while `ext` says 1584 × 1224.
`PageMapping` is built from both, so the pointer mapped to a page point that
was not where the pointer was — the same frame reported `page=(618.59, −74.79)`
for a pointer well inside the sheet — the anchor's `frac` came from that, and
the next solve asked for an offset far outside the range.

`acting` is now taken from the page that was actually chosen, so the rect and
the extent always describe the same sheet.

### O26d — the zoom anchor did not name its page

A page-local offset is measured from **one** page's top-left, and converting it
back into a strip offset means adding **that** page's origin. The canvas added
whichever page was current on the frame the anchor was *consumed* — and an
anchor is armed on frame N, while `show` runs and the wheel is seen, and solved
on frame N+1, once the zoom has landed. The current page tracks the scroll in
between.

When they disagree the answer is wrong by whole page pitches. At 900,000 % a
pitch is 1.1 × 10⁷ points, so the offset lands far outside the scrollable
range, `strip_offset` clamps it to zero — **and zero is the content's top-left
corner.** Driven, descending 970,851 % → 814,325 %: the page point under the
viewport centre went from 1164.82 to **−0.04**, the page's own top edge, and
stayed there for the rest of the descent.

`ZoomAnchor` and `CanvasFrame` now carry `page`, and the conversion uses it.
Under `PageDisplay::Single` there is one page at the strip's origin and this is
the identity it always was.

## O26e / O26f / O26g — the hand-over back out of the `f64` tier

**Status:** **FIXED 2026-08-24**, and driven. The operator's *"from around
2 million %"* is the same number as O24f's, and it is not a number he picked
either time: `SUB_PIXEL_CONTENT_EXTENT / page_height` is where the position
hands over between the `f32` scroll offset and the `f64` `DeepAnchor`.

### ★★★ O24f fixed the hand-over IN. There was never one OUT.

A hand-over is two functions. Seeding the anchor from the scroll offset on the
way in was written; converting it back on the way out was not. Coming down, the
anchor was discarded and the `f32` machinery resumed from the zero the deep
tier forces every frame.

**Measured before the fix**, descending through 1,185,799 %: the page point
under the viewport centre went from (791.93, 1152.34) to **(−0.02, −0.03)** —
the corner of the sheet, with twelve million pixels of drawing off screen.
1,152 pt of movement, or about eleven million screen pixels.

★★ **The suite could not see it because `zoom_keeps_place` climbs.** It climbs
to the ceiling, one notch at a time, with a tolerance fine enough to catch a
hundredth of a point, and then the run ends without ever rolling the wheel the
other way. Its own header calls the hand-over *"half of what this check is
for"*; that sentence was true of the **upward** crossing only. **A check that
travels in one direction tests one direction.**

Three pieces:

* **O26e — `CanvasFrame::offset` was a lie at the deep tier.** It was
  reconstructed from the scroll offset, which that tier **forces to zero**, so
  every deep frame recorded "the page is centred in the pasteboard". Nothing
  consumed the lie while the tier held; the first zoom that crossed back did,
  because `offset_before` is that field. It is now **measured from the drawn
  rect** — `geometry::offset_from_drawn`, `margin − (page_min − viewport_min)`
  — which is algebraically the same number below the threshold (asserted by a
  unit test over the same inputs) and the truth above it, because it never
  mentions the scroll offset at all.
* **O26f — the exit is solved in `f64`.** `offset_from_drawn` alone took the
  descent from 1,152 pt out to 0.005 pt out, but 0.005 pt at a million percent
  is fifty screen pixels: every term being subtracted has a magnitude near 10⁷,
  where an `f32`'s step is a whole pixel. `DeepAnchor::page_local_offset` forms
  `page × zoom` in `f64` and narrows once, on the frame that leaves the tier —
  and re-states the anchor about the pointer first, so the last notch out of
  deep zoom is not the one notch that fails to hold the cursor.
* **O26g — the strip is placed from the content's origin, not its centre.**
  `Rect::from_center_size(outer_rect.center(), display_size)` is the same
  rectangle and is a catastrophic cancellation: in a continuous mode the strip
  is `pages × page_height × zoom`, which on a 36-page set at a million percent
  is 4.6 × 10⁸ points, where an `f32`'s step is **32 points**. It formed
  `centre − strip/2`, two numbers near 2.3 × 10⁸ whose difference is about 619.
  `geometry::strip_origin_offset` evaluates the same quantity symbolically — a
  centring margin that is exactly zero once the strip exceeds the viewport,
  plus one viewport of pasteboard — so no large intermediate is formed. Proven
  equivalent by a unit test wherever the plain expression is still exact.

  ★ Honest note: **the measured jitter did not change with this one.** It is
  justified by the arithmetic and by the equivalence proof, not by an
  improvement anyone observed. See O27.

## O27 — The `f32` scroll tier jitters above about 100,000 %

**Found:** 2026-08-24, while driving O26. **Not reported by the operator.**

With all four O26a-d causes and all three O26e-g pieces fixed, an anchored zoom
notch still moves the view by **10–35 screen pixels** on the `scroll` tier
above roughly 130,000 %. On the `deep` tier the same measurement is **±0.05
px** across four readings — exact.

It is **bounded jitter, not drift**: sixteen consecutive readings at ~10⁶ %
oscillated within a band of 43 px and did not accumulate. The view shimmies; it
does not walk away.

★★ **Both zoom checks are RED on this, deliberately.** An earlier draft gave
them a "record instead of assert" hatch above a measured jitter zoom, with a
written argument for why that was a boundary on the subject rather than a
loosened tolerance. **On its very first driven run the hatch recorded a
movement of 1,161 pt — the whole page — and reported PASS**, hiding O26d on its
first outing. The hatch is gone. Two red checks that name a real residual beat
two green checks that swallowed a page.

★ Cause not established. The predicted `f32` accumulation in the anchor solve
is about ±2 px, so something an order of magnitude larger is in the chain and
has not been found; the candidates left are the acting page's own strip origin
(up to 4.5 × 10⁸ on this document, step 32) and `egui`'s own scroll-area
arithmetic at that content size. The structural remedy is probably to make
`viewer::deep_position_needed` test the **view's** magnitude rather than one
page's — the strip exceeds `f32`'s exact range earlier than the page does, by
exactly the page count — but that widens the deep tier considerably and this
canvas has three times been broken by a change that meant to affect only deep
zoom. Not attempted.

## O25 — Panning far, or zooming out, leaves the new area blank

**Asked:** 2026-08-23 — *"zoom is working amazing, and panning is fast, but if
I pan to far to one side when I am beyond 800% zoom it doesn't always render
the new exposed area, and the same thing happens usually when I zoom out."*

**Status:** **FIXED 2026-08-23.** Driven, and the check fails on a build with
the defect restored.

### ★★★ One missing comparison, and it explains both halves

Above the pixmap ceiling a raster covers the **visible region** rather than the
page, so two textures of the same page at the same scale can be pictures of
*different places*. `render::settle`'s staleness test asked two questions —
has a **discrete input** changed (page, annotations, layers), and has the
**scale** changed — and **the region was in the cache key without being in
either**.

So a pan that changed nothing but which part of the page is on screen was not
stale by any measure it applied, and **no render was ever requested**. The
picture he had kept being drawn correctly at its own region and simply slid
off, leaving the newly exposed area blank for as long as he cared to look at
it.

★ The zoom-out half is the same fault by a different route. A zoom *does*
change the scale, so a render is requested — but the request is built from
whatever region was current when it spawned, and by the time it lands the
gesture has moved on. Once the scale settles, nothing notices the region it
arrived with is the wrong one. **"Usually"** in his sentence is the tell: it
depends on whether the gesture outran the render.

### Where the new term went, and why not with the discrete ones

`stale_region` is grouped with the **scale**, on the same debounce. A region
changes under a continuous gesture, and a render started on every frame of a
drag would be cancelled by the next one — the worker is single-slot — so the
operator would pan for a second and receive nothing at the end of it.

★ It is already rate-limited in a way the scale is not:
`render::strategy::region_for` snaps to a half-viewport grid, so a region
changes at most once per half-screen of travel however smoothly the pointer
moves. The debounce is the second limiter, not the only one — which is why the
settle interval can stay tuned for zoom without making a pan feel slow.

### ★★ The check could not see it, and the reason is worth more than the fix

The first version of `panning_past_the_overscan_renders_the_new_area` watched
`region=` — the region the pixels on screen are a picture of. On the defective
build **that field never changes**: no render is requested, no new texture
arrives, so the field describing the texture stands still. The check read *"the
view did not move"* and reported **SKIP** against a binary with the defect
deliberately restored.

The trace now carries `want=` beside it — the region the shell wants next,
which moves the instant the view does. **The gap between the two is the
defect, and it takes two fields to measure a gap.** With `want=` the check
fails on the defective build, naming the cause, and passes three runs of three
on the fixed one.

### What was measured

| | |
|---|---|
| pan, 40 wheel notches at 4,155 % | wanted region moves; **2 renders complete**; canvas shows 45–46 distinct tones |
| then zoom out, 6 Ctrl+wheel notches | wanted region moves; **1 render completes**; canvas shows 46 distinct tones |
| the same, with `stale_region` removed | wanted region moves; **0 renders**; check FAILS naming `RenderKey::same_region` |

★ The zoom-out half is asserted separately rather than assumed fixed by the
pan case. They share a cause and they do not share a code path, and *"it is
probably the same bug"* is how the second half of a two-part report gets
shipped broken.

### Why nothing else caught it

`panning_at_deep_zoom_stays_where_it_was_put` asks whether the view **moves**
and whether the pixels are **placed** correctly — both were perfect throughout.
`the_page_still_renders_at_every_decade_of_zoom` photographs after a **zoom**,
which changes the scale and therefore does request a render. **Nothing in the
suite panned far enough to leave the overscan and then looked at the screen.**

---

## O24i / O24j — Screenshots at maximum zoom, and the two defects they found

**Asked:** 2026-08-22 — *"Can you confirm that rendering on screen is actually
happening at maximum zoom? zoom in on one of the michocondria structures and
post screenshots here to confirm. start with the full page first to confirm it
renders."*

**Status:** **CONFIRMED, and it was not confirming before he asked.**

### ★★★ Why nothing already in the suite could answer this

`zooming_does_not_throw_away_where_the_operator_panned` proves the view stays
where it is put to a trillion percent. `zooming_past_the_pixmap_ceiling_still_
renders` proves no raster is refused. **Neither looks at the screen**, and a
canvas can satisfy both while drawing blank paper: the arithmetic would be
perfect, the rasters would complete, and the operator would see nothing.

That is exactly what was happening.

### O24i — the region path narrowed to `f32`, and detail stopped at ~10⁷ %

`render::strategy::region_for` snaps the region's origin to a half-view grid:

```rust
snapped_x = (x0 / step_x).floor() * step_x
```

At a trillion percent `step_x` is about 2 × 10⁻⁸ pt while `x0` is an ordinary
page coordinate near 540. Their quotient is **2 × 10¹⁰** — past `f32`'s last
exactly representable integer of 2²⁴ ≈ 1.7 × 10⁷ — so `.floor()` was applied to
a number that had already lost its integer part.

**Measured before the fix:** from about 10⁷ % the region stopped shrinking and
floored at 2.4414 × 10⁻³ × 3.0213 × 10⁻³ pt, **fifty thousand times** the
4.8 × 10⁻⁸ × 6.2 × 10⁻⁸ the viewport was showing. Its raster was then painted
18,998,834 window points off the viewport. `drawn=1` was still traced and no
render failed, so every check passed; the operator saw a fraction of one texel
stretched across the window.

The whole path is `f64` now — `region_for`, `overscanned`, `page_region`,
`OVERSCAN`, and the canvas call site that used to cast `DeepAnchor::
visible_rect`'s `f64` result straight down to `f32`. That cast was the one
narrowing left in a path whose every other stage was already `f64`, and it was
narrowing the value the tier exists to compute.

★ The real ceiling of the fixed design, since it is worth knowing: the extent
is computed as `(x0 + w) − x0` at an absolute position near 540, where an
`f64` ULP is 1.1 × 10⁻¹³. At the maximum zoom `w` is 10⁻⁹ pt — about **8,800
ULPs**, or eighteen representable steps per screen pixel. Comfortable. The tier
below it ran out at 2²⁴; this one has room left.

### O24j — the status bar showed `4294967295%`

`ViewState::zoom_percent` returned a `u32` and `as u32` saturates, so past
about 42,949,672 % the readout showed **u32::MAX presented as a measurement**.
Seen in the screenshot gallery, which is the only instrument that reads the
number an operator reads.

★ The type was right when `MAX_ZOOM` was 8.0 and every reachable value fitted
in three digits. O24 raised the ceiling to 10¹² and did not revisit it — the
recurring shape of this whole request: **a limit lifted in one place while a
narrower type downstream keeps enforcing the old one silently.**

★★ It now reports **999999995904%** at the top, not 1000000000000%, and that
is correct rather than a rounding failure: `ViewState::zoom` is an `f32`, so
that is the nearest representable value and it is what the view actually is.
Pinned exactly, so a future change that starts rounding the display instead of
reporting it has to be deliberate.

### The gallery

`the_page_still_renders_at_every_decade_of_zoom` — new. Opens the document,
parks the pointer on a **document coordinate**, climbs by Ctrl+wheel and
photographs the window at each tier of the fixture's own scale chain. At every
step it asserts three things, because any two can hold while the third fails:

| assertion | rules out |
|---|---|
| the **canvas region** is not near-uniform | a blank page |
| the canvas traced `drawn ≥ 1` | space reserved with no raster in it |
| no `outcome=failed` render | a refused rasterization the shell swallowed |

★★ *The canvas region*, not the window. The first version asked
`capture::window_to_png` — which refuses a near-uniform **window**, and a
window always contains a ribbon and two panels, so it can never fire for the
reason this check cares about. It passed a screenshot of blank white paper on
that technicality. Third instance this session of an assertion aimed at the
wrong surface.

★ It also **re-aims between tiers**, from the `f64` position line. Zoom-to-
cursor holds the point to about half a per-notch tolerance, which is excellent
per notch and still accumulates over the ~120 notches to the ceiling — so
without it the run wanders off a 3 µm mitochondrion and photographs cytoplasm.
`CanvasMapping` cannot do the re-aiming: it converts through the `f32` `rect=`,
whose spacing at the ceiling is half a million points.

### What the screenshots show

| zoom | on screen | distinct tones in the canvas |
|---|---|---|
| 114 % | the whole banana | 318 |
| 2,785 % | the two cells | 293 |
| 13,794 % | cell labels, organelles | 487 |
| 45,799 % | labelled organelles, the easter egg | 707 |
| 504,845 % | one mitochondrion in cytoplasm, cell wall behind | 144 |
| 3,730,330 % | mitochondria with cristae | 238 |
| 41,120,084 % | mtDNA nucleoid, mitoribosomes, ATP synthase heads along a crista | 222 |
| 999,999,995,904 % | mitochondrial matrix — a 0.02 nm field, smaller than an atom | 15 |

★ The last row is a solid fill and that is **correct**: at the ceiling the
viewport spans 6 × 10⁻⁸ pt, and the fixture has nothing smaller than the 10 nm
ATP synthase heads. Rendering is still happening; there is simply nothing left
to draw.

### And an easter egg

`gen_banana.py` gained `easter_egg.py` today. Inside the pulp cell, readable
from about 100,000 %: **KEN ♡ EMILY — HAPPY 7TH ANNIVERSARY 2026.**

---

## O24h — "Can you test up to maximum zoom please?"

**Asked:** 2026-08-22 — *"can you test up to maximum zoom please? If you find
issues that probably can't be resolved it is ok at that point to say good
enough. that level of zoom is unheard of in any pdf software commonly available
and the performance is amazing."*

**Status:** **DONE 2026-08-22. Nothing had to be called good enough.**

### The result

Both driven zoom checks now climb until the application **saturates**, rather
than to a depth chosen in advance. Measured on `banana.pdf`, whose two cells
are drawn at life size and are the only thing on the sheet worth magnifying:

| | |
|---|---|
| ceiling reached | **1,000,000,000,000 %** — the configured maximum, exactly |
| stages to get there | 16, of 8 Ctrl+wheel notches each (128 notches) |
| notches that advanced | 117 of 128; the rest are the tail after saturation |
| tiers crossed | `scroll` → `deep` |
| worst per-notch drift of the point under the cursor | **54 % of tolerance** |
| panning at the ceiling | +960 px asked, +960 px moved, held for 90 frames |
| renders refused | none |

★ The saturation test asks the **application** where its ceiling is rather than
comparing against a constant. The maximum is an operator setting, so a check
that hard-coded 10¹² % would silently stop testing the ceiling the day he
changed it — the same silently-inert control this whole request began with.

### ★★★ Two harness faults the climb exposed, both of the same kind

Neither was an application defect, and both would have been reported as one.

**1. The instrument ran out before the application did.** `held()` derived the
page point from the `canvas` line's `rect=` and `zoom=`. At 41,000,000 % a
Letter page's rect holds a magnitude near 2.5 × 10⁸, where an `f32`'s spacing
is 32 — so the reading resolved to about 8 × 10⁻⁵ pt while the tolerance at
that zoom was 3 × 10⁻⁵. The check failed with *"moved 0.0000 pt, where 0.0000
is the tolerance"* against a build holding the point perfectly.

The tempting fix is to widen the tolerance, which would have hidden a real
defect at every zoom below that. The `canvas-pos` line already carries the same
quantity in `f64` — added for O24b for exactly this reason — so the fix was to
**read the instrument that can still see**. `RESOLUTION_FLOOR` now stops the
proportional tolerance from ever dropping below what any instrument here can
resolve: a floor where the proportional tolerance would be smaller, not a
widening where it is meaningful. Those are different changes and only one of
them is honest.

**2. A guard phrased as "every notch advances".** The ceiling is reached
partway through a stage, so the tail of that stage and the whole of the next
legitimately stand still. The guard exists to catch a wheel that is *panning*
instead of zooming — which advances on **zero** notches — so it is three
quarters now, with room to spare.

### And one more, in a check that was not part of this

`measure_hover_shows_what_it_will_take` failed on this fixture, having first
printed *"legitimate"* about the very condition that made it fail: the sweep
landed on the banana's outline, a curve has no endpoint to snap to, and the
assertion below can only be met by a straight run. It now SKIPs with the
finding named. **A check that fails on correct behaviour is worse than an
absent one, because its red gets quoted.**

### Full suite

`ui-verify` on `banana.pdf`: **36 verified, 0 failed, 36 skipped** — the skips
are checks needing a `--doc-point` or a fixture this sheet cannot provide.

---

## O24e / O24f / O24g — Zoom throws the view away, twice, and `−` undoes a hundredfold

**Asked:** 2026-08-22, one message, three separate faults:

> *"there is a little bug where if I am zoomed out to about page size, pan the
> cells to the center of the screen, then start to zoom, the page snaps back to
> near the center position. … I do lose the view at 2000000% magnification.
> Also clicking the negative button to zoom back snaps me back to 800% when I
> am over 800%."*

**Status:** **ALL THREE FIXED 2026-08-22.** Driven, and the driven check fails
on a build with the defects present.

### O24g — the `−` button was not the inverse of `+`

`ladder_step_up` grew a doubling branch when O24 raised the ceiling.
`ladder_step_down` did not, so a plain reverse search found the highest named
rung below the current zoom — **8.00** — from anywhere above it. One press
discarded a hundred-fold magnification.

★ The asymmetry is the defect, more than the snap. `viewer`'s own header
promises *"zoom-in/zoom-out exactly reversible"*, and two controls that
disagree about what a step is break the one property an operator relies on to
explore without losing their place. Pinned as a **round trip** — up then down
returns to where it started — rather than against fixed numbers, which would
keep passing if both were changed together in a way that broke it.

The ladder is now its own module (`viewer::ladder`), which R2 forced when
`viewer/mod.rs` reached 1,540 lines and which is a real seam: everything in it
answers *what is the next zoom?* and none of it knows what a page is.

### O24e — a stale clamp, in the wrong space, against the wrong extent

`geometry::zoom_anchor_offset` clamped its answer to `display − viewport`: the
range a page has when the scroll content is the page and **nothing else**. The
pasteboard (O23) made that false — `content_extent` now adds a viewport of
slack on every side, so the real range is larger and the page is only part of
it.

★★ The damage was worst exactly where he found it. At a fit-page zoom the page
is no **larger** than the viewport, so `display − viewport` is zero or
negative, the clamp range collapsed to `[0, 0]`, and every zoom forced the
offset to zero — which after the strip conversion is the centred position. Not
*"near"* the centre by accident: it **is** the centre.

The clamp has moved to `strip_offset`, the one place that knows the real
range and the only value actually handed to the `ScrollArea`. That is the
division of labour the module already stated and had stopped observing.

★ Two tests had to change with it, and both were pinning the pre-pasteboard
constraint — including one asserting that framing a region at the page's
corner *cannot* be centred, *"there is no page to the left of or above the
origin to scroll to"*. O23 was the request to make exactly that possible. **A
test that pins last year's constraint is how a stale clamp survives the
feature designed to remove it.**

### O24f — the deep tier's hand-over, in three parts

2,000,000 % is not a number he picked. The threshold is
`SUB_PIXEL_CONTENT_EXTENT / page_height` = 16,777,216 / 792 ≈ **2,118,000 %**
on a Letter sheet. Three faults met there:

1. **The seed read the previous frame's scroll offset.** Dividing it by the
   *new* zoom asks where a point is using one frame's distance and the next
   frame's scale. The zoom anchor is now consumed at this tier too, and the
   seed uses the offset it solved for.
2. **Nothing called `DeepAnchor::zoomed_about`.** The module exists for that
   one operation and it had no callers: the anchored page point stayed nailed
   to the viewport's top-left and everything the operator was looking at
   expanded off screen. It is now called on every zoom while deep, about the
   pointer — or the viewport centre when the pointer is elsewhere, matching
   what `+`, `−` and Ctrl+0 anchor on at every other zoom.
3. **★★★ The scroll area held its old offset for one frame.** The content is
   the viewport at this tier so zero is the only valid offset, and egui does
   clamp to it — one frame late. On the frame the tier flips, `outer_rect.min`
   is still displaced by the stale offset, so the anchor placed the strip
   relative to a displaced origin and the page landed at roughly **twice** the
   intended distance.

Measured at the hand-over, 2,047,244 % → 2,181,987 %: the position line said
the page origin belonged 6,676,376 px left of the viewport; it was drawn
12,940,650 px left. The difference is 6,264,274 — the stale scroll offset, to
four significant figures. The offset is now assigned rather than left to the
clamp, because the raster region is computed from the same placement: the
frame was not merely misplaced, it rendered a different part of the page.

### What was measured

`ui-verify --check zooming_does_not_throw_away_where_the_operator_panned`.
Pans off-centre, then Ctrl+wheels **one notch at a time** with the pointer on
the viewport centre, following the page point under that centre:

| stage | zoom | tier | worst per-notch drift |
|---|---|---|---|
| 0 | 76 % → 377 % | scroll | 43 % of tolerance |
| 1–5 | 377 % → 1,123,552 % | scroll | 52 % |
| 6 | 1,123,552 % → 5,564,985 % | **deep** | 52 % |

Before the fix it failed at notch 3 of stage 6 — the hand-over — with the
centred page point moving 556.5 pt against a tolerance of 0.0004.

★ Three properties of that check are deliberate and were each learned the hard
way today: it **pans first** (the centred position is what O24e snapped *to*,
so a check that skipped the pan would watch the view "stay" where the bug was
about to put it); it reads **per notch** (once per eight-notch stage compared
accumulated rounding against a one-notch budget and failed a correct build);
and it **refuses to pass** a run that never crossed the tier boundary.

---

## O24c / O24d — The page lurches backwards mid-pan, and bounces when zoomed

**Asked:** 2026-08-22, two reports minutes apart, and they are **one defect**.

> *"As I drag using the middle mouse button the pan will follow and work, but
> if I pan a little too far it jumps back in the opposite direction I was
> moving the mouse towards … It isn't exactly in the same place as it started.
> When I zoom in the image does seem to disappear from the screen sometimes …
> if I pan the other direction and cross the same area where I experienced the
> jump the pan location jumps back to being correct."*

> *"Up to 800% things work perfect. Over that … it seems to refresh the image
> zoom, then reposition to the cursor location, which causes the image to
> bounce around a bit before settling under the cursor."*

**Status:** **FIXED 2026-08-22.** Unit-tested. ★ Driven confirmation still
owed — the operator was at the machine and `ui-verify` cannot take the
foreground from him.

### ★★★ The cause, and the second report is what proves it

The current page's texture is served from its slot **without a staleness
check**, deliberately: that is what shows the last good picture during a pan
instead of blank paper, and it is his own requirement — *"I don't want the
affect that other readers have where you always have to wait for detail to
render after panning to a new area."*

But the destination rectangle was computed from `OpenDoc::region_for` — the
region the shell wants **next** — while the pixels were still the previous
region's. `render::strategy::region_for` quantises the wanted region to a
half-viewport grid, so the instant a pan crossed a grid line the destination
jumped a whole grid step while the picture did not change. Every detail
follows:

| his words | the mechanism |
|---|---|
| *"follows for a bit, then jumps back"* | smooth within a grid cell, one step at the boundary |
| *"the opposite direction I was moving"* | the grid steps against the pan |
| *"isn't exactly in the same place as it started"* | the step is the grid, not the drag |
| *"cross the same area and it jumps back to being correct"* | pure function of position — re-enter the cell, get the cell's rect |
| *"the image does seem to disappear"* | two steps at once, at a zoom where the grid is most of the window |

### ★★★ RETRACTED: "up to 800 % things work perfect" is not evidence

The first version of this entry claimed that row as the confirmation, on the
reasoning that 800 % is where the whole-page raster gives out and the region
tier takes over. **That is false and the trace says so.**

The region tier engages at `MAX_PIXMAP_EDGE / page_height` — 16,383 / 792 =
**about 2,070 %** on a Letter sheet. Below it the raster is whole-page and
`region=none`. 800 % is nothing but the **old maximum zoom**, and the plain
reading of his sentence is *"the range that existed before is fine; the new
range is not"* — a statement about what he had already tested, not about a
mechanism.

★★ This matters beyond the correction. The driven check was tuned to land at
1,867 %, just under the real threshold, so every run traced `region=none`, the
placement cross-check had nothing to compare, and **the check reported PASS
twice against a binary with the defect deliberately put back in**. The wrong
reading of the 800 % sentence is what made that look like agreement instead of
like a check that could not fail.

A sentence was promoted to a measurement because it agreed with a theory. The
theory happened to be right; the evidence for it was not evidence.

### The actual proof

With the zoom raised to land at **4,155 %** — inside the band where the region
tier is engaged and the position is still on the `scroll` tier, which is the
only place this defect can exist — the check:

* **FAILS** on a build with the placement reverted to the wanted region, by
  **309.5 points** at mid-roll 1, and
* **PASSES** three runs out of three on the fixed build.

`ui-verify` now refuses to report PASS on a run in which no reading described
a region raster (`REGION_TIER_REQUIRED`). A check that cannot fail is not
evidence, and this one was being quoted as evidence.

And the zoom bounce is the same thing seen on a different transient: a zoom
changes the wanted region wholesale, so the held texture was thrown to a
completely different rect for as long as the new raster took — *"bounces
around a bit before settling"*. His guess in the message (*"maybe what you are
doing now will fix that behaviour"*) was right.

### ★★ Why rejecting the stale texture would have been the wrong fix

It is the obvious fix and it is worse. Blanking the page on every grid
crossing is precisely the behaviour he ruled out by name. The fix is to draw
the stale pixels **where they belong**, so they slide with the page as the pan
continues and the new raster replaces them in place.

`RenderKey` already carried the region; nothing ever read it back. It does
now (`RenderKey::region`), the texture and its region travel together to the
paint site, and the placement asks the pixels rather than the request.

### What was added to make it checkable

`canvas-pos` gained `paint=`, `region=` and `ext=`. `ui-verify` recomputes
`region_on_screen` from those **independently** and compares — so a future
change back to the wanted region is caught rather than merely absent from the
tests. `RenderKey::region`'s round-trip is pinned bit-exact.

★ The check also now zooms with Ctrl+wheel **at the two cells** rather than
pressing `+`, on his correction: *"Right now you are just zooming into a blank
area on the canvas."* Zoom-to-cursor keeps them under the pointer, so one aim
covers every rung.

---

## O24b — "Can the huge intermediate be fixed? Is that why panning jumped back?"

**Asked:** 2026-08-22 — *"can the huge intermediate be fixed? is that the
challenge I was running into trying to pan over a little bit at high zoom, but
it would jump back to it's original location I panned from because I couldn't
pan to the next point?"* — with the clarification that he meant the release
**before** the deep-zoom one, not the build published on 2026-08-22.

**Status:** **ANSWERED AND MEASURED, 2026-08-22.**

### Both halves, answered

**The intermediate is fixed.** `render::region::region_on_screen_deep` computes
the drawn rect from the `f64` anchor, so the page's ~10¹²-pixel screen rect is
never formed. That is the change that took the ceiling from about two million
percent to a trillion.

**And yes, that is consistent with what he described.** On the previous
release the view's position lived entirely in an `f32` scroll offset over a
content space of `page × zoom` where one unit is one screen pixel. Past about
2²⁴ content points that `f32` can only address every second pixel, then every
fourth, and so on — so a small pan computes `last - delta`, rounds to the
nearest representable value, and lands back on `last`. The view does not
move. From the operator's seat that is indistinguishable from *"it jumped back
to where I panned from"*, and it is exactly *"I couldn't pan to the next
point"*.

★ Stated as consistent rather than as proven, deliberately: the build he saw
it on has been published over and cannot be driven any more. What **is**
measured is the current one.

### What was measured

`ui-verify --check panning_at_deep_zoom_stays_where_it_was_put`, on
`banana.pdf`, rolling the wheel three notches and reading the position again
ninety frames later:

| zoom | tier | before → after → settled |
|---|---|---|
| 102,400 % | `scroll` | 405304.625 → 405424.625 → 405424.625 |
| 999,999,995,904 % | `deep` | 1981027832031.25 → 1981027832151.25 → 1981027832151.25 |

**+120 px asked, +120 px moved, and it stayed** — the same 120 pixels at a
trillion percent as at a hundred thousand. In `f32` that second row is
impossible: the representable spacing near 2 × 10¹² is 262,144, so a 120-pixel
move could not be written down at all, let alone survive ninety frames.

### ★★ A harness defect found on the way, and worth more than the answer

The check's first version drag-panned with the **primary** button and reported
the view as stuck at 102,400 %. That was wrong. `canvas::input::pan_delta`
pans on the middle button always and on the primary button only under the hand
tool; the default tool is Select, so a primary drag correctly rubber-band
selected and correctly moved nothing. The harness had measured a gesture the
application never offered and blamed the application for not honouring it.

It is written down because it is the third instance this month of the same
shape — a measurement of the wrong surface, whose verdict line is
indistinguishable from a real defect. **Ask what the check sampled before
asking what is broken.**

### What this added to the shell

`canvas::trace::position` — a `canvas-pos at=… tier=…` line carrying the pan
position in `f64`. The existing `canvas` line's `rect=` and `off=` are both
`f32`, and at these depths their own representable spacing exceeds the pan, so
**neither can measure this**: a check reading either would report a stuck view
against a perfectly working build. `tier=` names which mechanism produced the
number, so a failure points at one file rather than two.

---

## O24 — A setting for the maximum zoom

**Asked:** 2026-08-21 — *"add a setting so the user can set the maximum zoom.
the pdfce engine has been updated to handle at least 1,000,000,000,000%. I'm not
concerned about the practicality of offering such a high zoom. it is up to the
user to determine how much of a performance hit they want to take."*

**Status:** **RECORDED 2026-08-21. NOT STARTED.** ★ The engine claim is
**verified** — see below — and it changes what this row is. The setting is the
small half.

### The engine really does do it, and the commits say how

`D:\Dev\pdfce`, both landed since this shell's current lock:

```
71f7055  Deep zoom now holds its viewport to a trillion percent, and the fix
         is one subtraction moved into f64
bd9844d  render-page --region: the flag that makes deep zoom a viewport
         question instead of a page-size one
```

That second title is the whole architecture of this row, stated by the engine
itself: **deep zoom is a viewport question, not a page-size one.**

### ★★★ Why raising a constant will not do it, and where it stops

`viewer::MAX_ZOOM` is `8.0`, and raising it moves the ceiling only until a
harder one binds. `viewer::max_zoom_for_page` computes:

```rust
let ceiling = (pdfce_render::MAX_PIXMAP_EDGE - 1) as f32 / (longest * ppp);
ceiling.clamp(MIN_ZOOM, MAX_ZOOM)
```

`MAX_PIXMAP_EDGE` is **16,384** and is an engine constant. For an A1 sheet
(~1,584 pt on its long edge) at 1 device pixel per point that ceiling is
**≈ 1,034 %**. So today `MAX_ZOOM = 800 %` binds first and the raster ceiling is
just behind it.

**A setting alone therefore buys about one more doubling and then stops
dead** — and it stops *silently*, because `max_zoom_for_page` clamps rather
than refusing, so the operator would set 100,000 % and watch the zoom stop at
roughly a thousand with nothing said.

★ That is the shape this project keeps finding: a control that is drawn,
accepted, persisted, and then quietly overruled downstream. Shipping the
setting without the mechanism behind it would be exactly that.

### ★★ THE RELEASE INSTRUCTION — 2026-08-21

> *"when you complete the step 2 zoom release to git and put on OneDrive."*

**On completion of step 2 — the `f64` viewport — and not before:**

| | |
|---|---|
| **1** | `git push origin main` |
| **2** | `python tools/package-portable.py`, which mirrors to the older `OneDrive\pdfceGUI*` slot and leaves the other as the fallback |

★ **This is the first push of the project.** `origin` is
`github.com/KenM76/pdfceGUI.git` and the local branch is **253 commits
ahead**; the last tag is `v0.3.0`. So the push is not a routine increment —
it publishes the whole of this shell's history at once, and it is worth
doing deliberately rather than as a step in a script.

★★ **Preconditions, because a release is the worst place to discover any of
them.** Every one has bitten this project already:

1. **Clean tree, 17/17 gates, all tests.** The gates include the four self-
   tests that prove the gates can still fail.
2. **The full driven suite**, on his own drawing, with **both** `--doc-point`s
   — `0,300,500` and `0,1211,1021`. One point passing is what hid `O22` for a
   day.
3. **`cargo update -p pdfce-core -p pdfce-render -p pdfce-print` first**, then
   rebuild and re-run. `O24` depends on two engine commits (`71f7055`,
   `bd9844d`) that this shell's lock predates, so a release built on a stale
   pin would ship without the thing it is a release of.
4. **`FEATURES.md` re-measured against the build**, because he reads it to
   know what he has, and it has carried a false claim before.
5. **Anything still failing is named in the release note**, not omitted. At
   the time of writing that is `multi_node_move_moves_every_picked_anchor`,
   which has never passed on any build and is an unbuilt path rather than a
   regression.

★ **Not before step 2.** He named the trigger precisely, and step 1 landing
on its own is not it — the point of the release is the higher zoom
capability, and shipping the tiering without the tier that needs it would be
a release of preparation.

### ★★★ THE CONSTRAINT THAT DECIDES THE DESIGN — 2026-08-21

> *"can you build the first step and build the second one and put it as an
> option to use instead for higher zoom capability? that way I can test out
> both in case there are performance issues introduced at lower zoom. I don't
> want to lose our capability to pan around a page and still see high detail
> as we pan. I don't want the affect that other readers have where you always
> have to wait for detail to render after panning to a new area."*

★★★ **That sentence rules out the obvious implementation, and it is right
to.** Region rendering applied everywhere would produce *exactly* the defect
he is describing — and it would be a regression, not a trade.

Here is why, stated plainly because it is the whole design:

| | today | naive region rendering |
|---|---|---|
| what is rasterized | the **whole page**, once per zoom | the **visible rectangle**, once per *position* |
| what a pan costs | nothing. The texture already exists; the view moves over it | **a new raster every time**, because the rectangle changed |
| what the operator sees while panning | full detail, immediately | blur, or blank, until the new raster lands |

So the thing he values — *"pan around a page and still see high detail as we
pan"* — is a **property of rasterizing the whole page**, and it is free
precisely because the raster does not depend on where you are looking.

#### The design that keeps it: tiers, each used only where the last cannot work

| tier | when | how | panning |
|---|---|---|---|
| **A — whole page** | while `page × zoom` fits `MAX_PIXMAP_EDGE` (16,384) | today's path, unchanged | **free, full detail** |
| **B — region + overscan** | above that, to ~1,000,000 % | rasterize the viewport **plus a margin**, so small pans are already covered | free within the margin; a re-raster only when you leave it |
| **C — f64 viewport** | above ~1,000,000 % | the visible page rect in `f64` becomes the position | as B |

★★ **Tier A is where he lives, and it does not change at all.** On an A1
sheet the whole-page raster works to about 1,034 %; today `MAX_ZOOM` stops it
at 800 % first. So every zoom he uses now, and one more doubling beyond it,
keeps exactly the panning behaviour he has — **by construction, not by
tuning.** There is no low-zoom performance question to test, because at low
zoom nothing is different.

★ And tier B only ever engages **where today the zoom is simply unavailable**.
Nothing is taken away to pay for it. The worst case is that deep zoom pans
less smoothly than shallow zoom — which is true of every reader, and is the
cost he explicitly said is his to accept.

#### The overscan is the part to get right

Rasterizing exactly the viewport means every pixel of pan crosses the edge.
Rasterizing the viewport **plus half a viewport on each side** costs 4× the
pixels and makes any pan up to half a screen free. That is the dial, and it
should be a **named constant with its cost written next to it**, not a magic
number:

```text
overscan 0.0  →  1.0x pixels, every pan re-rasters
overscan 0.5  →  4.0x pixels, pans up to half a screen are free
overscan 1.0  →  9.0x pixels, pans up to a full screen are free
```

At tier B the viewport is a few hundred thousand pixels, so 4× is cheap in
absolute terms — the whole point of tier B is that the raster no longer
scales with the zoom.

#### What the option actually is

He asked for the second step *"as an option to use instead"* so he can
compare. Given the tiering, the honest control is **the threshold, not a
mode**: the setting says how far the whole-page path is allowed to go before
the region path takes over. Set it low and he is testing tier B at ordinary
zooms; set it high and he never leaves tier A.

★ That gives him exactly the comparison he asked for, **and** it is the same
control as the maximum-zoom setting rather than a second one — which is
better than a checkbox, because a checkbox would have to be explained and a
threshold explains itself.

### ★★★ HOW IT GETS THERE — asked 2026-08-21

> *"how do we get to the insanely high limit? … I've seen readers hit over
> 4000%, and none are limited to a mere 1000%. You should be able to have a
> new algorithm take over for bigger zooms?"*

**Yes, and it is two changes rather than one — with two different ceilings
behind them.** The first gets from ~1,000 % to roughly a million percent and
needs no new position model at all. Only the second needs the *"new
algorithm"*, and it is not about how pixels are made.

#### Step 1 — render the WINDOW, not the page. Ceiling: ~1,000,000 %

Today the shell rasterizes the whole page and lets the scroll area show part
of it, so the pixmap grows with the zoom and hits `MAX_PIXMAP_EDGE` at about
1,034 % on an A1 sheet. **Every reader that reaches 4,000 % does the other
thing:** it rasterizes only the visible rectangle, so the pixmap is always
about window-sized and the zoom does not enter its size at all.

★ The engine has already done its half, and its own measurement is the proof
— commit `71f7055`, a requested 800×600 viewport:

```
zoom factor      zoom %        raster before    raster after
          1         100          800x600          800x600
    100,000  10,000,000          800x592          800x600
  1,000,000 100,000,000          800x640          800x600
```

*"the fix is one subtraction moved into `f64`"* — at deep zoom the region's
device origin is a few billion while the region itself is 800 points, so the
difference vanishes in `f32`. The large magnitudes now exist only inside
`f64` and are subtracted out before anything is handed back.

So on the engine side this is **done and measured to 100,000,000 %**. The
shell has simply never called `render_page_region`.

#### Step 2 — stop letting the scroll area own the position. Ceiling: none

This is the *"new algorithm"*, and it is about **where the viewport's
position is stored**, not about rendering.

Today the position is an `egui::ScrollArea` offset into a content rectangle
of `page × zoom`, and those offsets are `f32`. `f32` carries 24 bits of
mantissa, so it can address about 16.7 million distinct units before the
spacing between representable values exceeds one:

| content size | smallest addressable step |
|---|---|
| 16,700,000 pt | 1 pt |
| 1,600,000,000 pt | ~128 pt |
| 16,000,000,000,000 pt | ~1,000,000 pt |

An A1 sheet is ~1,584 pt on its long edge, so the content reaches 16.7
million at a zoom of about **10,500× — roughly 1,050,000 %**. Past that the
scroll offset cannot express where you are: panning would jump in steps of
hundreds and then thousands of points, and the view would judder and then
stick.

★ **Computed, not estimated.** The three steps above were produced by taking
the actual `f32` successor of each value: `1.00`, `128.00` and `1,048,576.00`
points respectively. The threshold for a 1,584 pt page is `16,777,216 / 1584`
= **10,543×**, i.e. 1,054,300 %.

What is *not* yet measured is the behaviour — that panning really does judder
and stick there. Worth driving before it is relied on, but the arithmetic is
not in doubt.

**So above about a million percent the source of truth changes**: the visible
**page-space rectangle in `f64`** becomes the position, panning adds to that
rectangle, and the scroll area stops being the thing that remembers where you
are. That is exactly the shape the engine's `--region` commit describes —
*"a viewport question instead of a page-size one"* — carried one layer up.

#### What this means for the order of work

| | delivers | needs |
|---|---|---|
| **1** | 1,000 % → ~1,000,000 % | region rendering in the canvas. No new position model |
| **2** | ~1,000,000 % → whatever he types | the viewport rect in `f64` as the position |
| **3** | the setting | (1) at minimum, or it is a control the shell cannot honour |

★★ **Step 1 alone already beats every reader he has seen.** 4,000 % is
inside it by two and a half orders of magnitude, and it is the smaller and
far less invasive of the two changes — it touches the render worker and the
raster cache, not the canvas's coordinate model. **If only one thing is
built, build that one.**

★ And it is not speculative: `crates/pdfce-gui/src/render/offpage.rs` already
drives `render_page_region` and asserts the pixmap matches the region asked
for. Those tests were written for `O23` and this is the same mechanism.

### What actually delivers it

**Render the viewport, not the page.** `pdfce_render::render_page_region` takes
an arbitrary page-space rect, and at deep zoom the visible rect is a *tiny*
fraction of the page — so the pixmap stays small however large the zoom is.
That is what the engine's `--region` commit means.

★★ This shell **has never called `render_page_region`.** Established
2026-08-21 while answering `O23`: it appears twice in
`crates/pdfce-gui/src/`, both times in prose explaining that a tiled path does
not exist. The render worker uses `render_page_with_view`, whole-page, every
time.

★ And it is already de-risked. `crates/pdfce-gui/src/render/offpage.rs` drives
the region path with regions off, straddling and enclosing the page, and
asserts the pixmap matches the region asked for rather than its overlap with
the page. Those four tests were written for `O23` and they are the same
mechanism this row needs.

### So the row is two pieces, and they ship in this order

| | | |
|---|---|---|
| 1 | **Region rendering in the canvas** | the real work. The render worker asks for the visible rect at the current zoom instead of the whole page. Wants `display_list::record_page` + `replay_region` rather than N region renders, because a region render re-interprets the whole content stream and a moving view would pay that per frame |
| 2 | **The setting** | small, and honest only once (1) exists |

★ Doing (2) first is possible and is **not** recommended: it would ship a
control that accepts a number the shell cannot honour, which is the defect
class above.

### Two consequences to decide when it is built, not after

- **The zoom readout is 46 pt wide**, sized for four characters
  (`ZOOM_READOUT_WIDTH_PTS`, with a comment saying so) because
  `ZOOM_LADDER` tops out at `800%`. `1000000000000%` is fourteen. The readout
  needs a format — `1e12 %`, or `1.0 Tx` — decided rather than allowed to
  stretch the status bar.
- **`ZOOM_LADDER` is a fixed array** the `+`/`−` buttons step through, ending
  at `8.00`. Beyond it the ladder has to become generated — presumably
  multiplying by a constant factor per step — or the buttons stop working
  exactly where the setting starts mattering.

★ Neither is hard. Both are the kind of thing that gets discovered by an
operator rather than decided by an engineer if they are not written down first.

### And one thing that is genuinely free

Ken's *"it is up to the user to determine how much of a performance hit they
want to take"* removes the question this would otherwise turn on. The setting
does **not** need a guard, a warning, or a preflight. It needs to be honest
about what it does, and to actually do it.


## O23 — Free navigation: any part of the page to anywhere on screen, and objects off the page still reachable

**Asked:** 2026-08-21 — *"also objects should still be reachable even if they are
off the page. I should also be able to move the view of the corner of the page
to the center of the screen, or even all the way vertically to the opposite
corner if I want to."*

**Status:** **RECORDED 2026-08-21. NOT STARTED.** ★ This **answers `O22`'s open
convention question** — the pasteboard is what he wants — and then asks for more
than `O22` proposed. `O22`'s three candidate fixes are settled by this row:
candidate 3, sized as below.

### Two requirements, and they are not the same job

| | |
|---|---|
| **A — free scrolling** | any point of the page can be brought to any point of the screen |
| **B — off-page content is reachable** | an object whose geometry lies outside the `/MediaBox` can still be seen and selected |

A is a scroll-extent change. B is about what the canvas draws and hit-tests at
all. They are filed together because he asked for them together and because A is
a precondition for B — there is no point being able to select something you
cannot scroll to — but they will not be one change.

### A — how much pasteboard, derived from his own words rather than guessed

He gave two levels, and the second is the requirement because it subsumes the
first:

1. *"the corner of the page to the center of the screen"* → needs **half a
   viewport** of margin on each side.
2. *"even all the way vertically to the opposite corner"* → needs **a full
   viewport** of margin on each side. To bring the page's top-left corner to the
   screen's bottom-right, the content must extend one whole viewport past the
   page on the top and left.

★ So: **pad = one viewport extent on every side**, recomputed as the viewport
changes rather than fixed in points. A constant number of points would be too
small on a large monitor and absurd on a small one, and it would silently stop
satisfying his sentence the first time he resized the window.

That is also the standard approximation of an infinite canvas — it is what
Illustrator, Figma and every CAD package give you, and none of them makes the
operator think about it.

### ★★ The risk, named up front because it has bitten this project before

Everything in `canvas::geometry` that today treats the **strip's** size as the
**scroll content's** size becomes wrong, because those stop being the same
number. `strip_offset`, `page_local_offset` and `pan_offset` all take a
`display`/`strip` extent and use it both to compute the centring margin and to
clamp the scroll range.

The failure mode is not hypothetical and it is recorded in `canvas::mod`'s own
source: in the old GUI, a centring-margin error made selection outlines draw
**~105 px** from the object they outlined, and clicking directly on a visible
object missed it — worst at exactly the zoom an operator uses to see a whole
page, and invisible at high zoom where the margin is zero.

So the change is: introduce the **content extent** as a value distinct from the
**strip extent**, and audit every consumer. Not "add a pad to `outer`".

### B — off-page objects

Not yet investigated. What has to be established first, against source:

1. ~~Does the decomposition **include** objects outside the `/MediaBox`?~~
   **ANSWERED 2026-08-21 against `pdfce-core` source: YES, with no culling of
   any kind.** The decomposer is never even *told* what the page box is —
   `decompose_page` has the `&Page` in hand and reads only its content
   stream, resources and fonts. Grepping `media_box|crop_box|page_box` across
   the whole `vector` module returns two hits, both in `clip.rs` and both
   about the synthetic clipboard PDF's own box. An object drawn at
   `(-5000, -5000)` is in `PageObjects::objects` with a truthful negative
   `page_bbox`.

   ★ Adjacent, and worth knowing before designing anything: **clipping is
   ignored in general, not just the page box.** The painting-operator
   dispatch has no `W`/`W*` arm at all, so a path used only as a clip is
   emitted as an ordinary object, and an object whose paint is entirely
   clipped away still arrives with its full unclipped geometry.
   `PaintStyle::is_invisible` exists so a caller can tell — nothing drops
   them. So *"everything the model contains"* is a larger set than
   *"everything the operator can see"*, by more than just the off-page
   objects.
2. ~~Does the hit test accept a canvas point **outside the page rect**?~~
   **ANSWERED 2026-08-21 — the hit test would, and it never gets the chance.**

   | | |
   |---|---|
   | the screen→canvas conversion | `canvas::mapping::to_page` is pure arithmetic with **no clamp** (`mapping.rs:189`), so a point past the page's edge maps to a canvas point past its extent, and would hit-test as ordinary geometry |
   | the pasteboard area | allocated `Sense::hover()` (`canvas/mod.rs:662`) — it senses the pointer and **cannot be clicked** |
   | each page | allocated `Sense::click_and_drag()` (`canvas/mod.rs:688`) |
   | where a press comes from | `response.clicked_by(..)` / `drag_started_by(..)` on **the current page's** response (`canvas/interact.rs:372-375`) |

   ★ So the gate is not the hit test and not the mapping — **it is the input
   surface.** A press only becomes a canvas gesture if it landed inside a
   page's rectangle. Content painted outside the `/MediaBox` sits over the
   area that senses hover and refuses clicks, so it can be pointed at and
   never pressed.

   ★★ That is worth knowing before any of part B is designed, because it
   means B is **not** a hit-test change. It is a change to what the canvas
   allocates as clickable — which is the same code the pasteboard touches,
   and is why A and B belong in one row even though they are two jobs.

   Hover, by contrast, is already unbounded: `interact` falls back to
   `ctx.pointer_latest_pos()` (`interact.rs:352`) and asks `over_canvas`
   against the scroll **viewport** rather than the page (`interact.rs:1310`).
3. ~~Is such an object **painted**?~~ **ANSWERED: no, and the engine already
   has the way to make it so — this shell has never called it.**

   The whole-page entry point sizes the pixmap to the **CropBox** and there
   is no explicit clip anywhere; the clipping is purely *implicit*, because
   geometry outside the pixmap is culled by the rasteriser. That is why the
   escape hatch works at all:

   **`pdfce_render::render_page_region(doc, page, scale, region, options)`**
   takes an arbitrary page-space rect and **never clamps or intersects it
   with the crop box**. A region starting left of or below the page produces
   a negative origin and is translated into view. The only limits are
   finite-and-non-empty and `MAX_PIXMAP_EDGE` (16,384) applied to the region.

   ★ **Nothing in `RenderOptions` selects a box.** "Render a bigger area" is a
   different *function*, not a setting — so this is not a matter of passing a
   flag.

   ⚠ **Two caveats, both load-bearing:**

   - ~~**No test exercises a region outside the crop box.**~~ ★★ **CLOSED
     2026-08-21 — one now does, in this repository.**
     `crates/pdfce-gui/src/render/offpage.rs` drives `render_page_region` with
     a region entirely off the page, one straddling its left edge, and one
     containing the whole page plus a margin. **All three rasterize, and the
     pixmap is sized to the region asked for rather than to its overlap with
     the page.** So the escape hatch is proven rather than merely
     unrejectable.

     ★ It lives here rather than in `pdfce-render` for two reasons: that crate
     is read-only to this project, and a consumer asserting the contract it
     depends on is the right shape anyway — if an engine bump starts clamping
     the region, the failure lands on the shell that cared, naming the feature,
     instead of presenting as a blank canvas.
   - **A region render re-interprets the whole content stream.** N tiles cost
     N interpretations. For a view that moves, `display_list::record_page` +
     `DisplayList::replay_region` is the intended path and is documented as
     landing on byte-identical pixels.

4. **And the measurement that makes the whole thing tractable:**
   `PageObjects::page_bbox()` returns the union of every object's bounds —
   which, because of (1), **includes the off-page ones**. So
   `model.page_bbox().union(crop)` is a ready-made *"what must I be able to
   scroll to in order to reach everything"*, and it feeds straight into both
   the scrollable extent and `render_page_region`.

   Precision caveats it carries: text boxes are approximate (and say so, via
   `TextBoundsBasis`), stroke width is **not** included, and clip-only paths
   inflate the union because of the finding in (1).

### ★★★ The conclusion: NO ENGINE CHANGE IS REQUIRED

Off-page content is already fully present and fully selectable in the model.
The decomposer keeps it, `page_bbox` measures it, `hit_test_point_all` will
select it. **The only place it disappears is the raster**, and only because
the whole-page entry point sizes the pixmap to the crop box.

So both halves of this row are shell work:

| | |
|---|---|
| **A** | the scroll extent and the seeding — `O23`'s attempt above |
| **B** | make the canvas allocate the off-page area as clickable, and render it through `render_page_region` |

★ Verified against **this** shell rather than assumed: `render_page_region`
appears twice in `crates/pdfce-gui/src/`, both times in **prose** explaining
that a tiled-progressive path does not exist. It has never been called. The
render worker uses `render_page_with_view`, i.e. whole-page-at-crop-box.

★★ **No feature request to the engine session is owed for this row.** That
was worth establishing rather than assuming: the reflex on hitting a wall
like *"the raster stops at the page edge"* is to file it as an engine gap,
and it is not one.

★ Rule 4 applies to the answer: if pdfce can see content the operator cannot,
that owes an **off-canvas** report. It must not be marked on the page.

### ★★★ ATTEMPTED 2026-08-21 AND BACKED OUT. What it cost, and what it taught

The whole change was built, all 1,634 unit tests passed, all 17 gates passed
— **and it broke selection on the real application.** It was reverted the
same evening rather than left in the tree, because a build where clicking an
object does nothing is worse than one that cannot rotate near the top edge.

Nothing below is speculation. Every item was measured.

#### 1. There are TWO offset spaces and only one of them has a pasteboard

| space | origin | margin |
|---|---|---|
| the scroll offset egui is given | the content's top-left, pasteboard included | padded |
| the page-local offset the view stores | the page's own top-left | **plain** |

`strip_offset` and `page_local_offset` convert between them and must use
**one of each**, or the pad cancels and vanishes.

★ The trap: `anchor_screen_pos` and `offset_holding_anchor_at` look like
scroll-space functions and are **page-local** — `canvas::mod` converts before
building the `CanvasFrame`. Padding them doubles the pad, and the symptom is
*"zoom-to-cursor flies off"*, worst on a large window.

#### 2. ★★ The pasteboard must be measured against the OUTER viewport (R128)

The obvious `ui.available_size()` is measured **inside** the scroll area, so
it depends on whether the scrollbars are showing — and the pasteboard is what
makes them show. Feeding it back is a loop: content grows, scrollbars appear,
available shrinks, content changes.

Measured symptom when it happened: `ui-rect-gone name=canvas-viewport` — the
canvas region retired entirely and no page was drawn. That is R128 in a new
place, the same shape as the status bar that drifted 230 % → 224 % → 215 %.

#### 3. ⚠️ THE DIAGNOSIS THIS ROW GAVE FIRST WAS WRONG, AND IT WAS NOT MEASURED

This section said the page *"MOVES, one frame later, as the offset settles"*,
and gave numbers: the page's rect going from `y=143.0` to `y=269.7`.

**Those two numbers came from two different builds.** 143.0 was the shell
without a pasteboard; 269.7 was the shell with one. Comparing them and
calling the difference a per-frame transient is the same unsound inference
this file has now corrected three times in one day — and it was written
here, as a measurement, hours after the rule was recorded.

**Re-measured properly on 2026-08-21, within a single run**, by counting
distinct `canvas rect=` lines (the trace is a change log, so one line means
one stable value):

| build | distinct rects during startup |
|---|---|
| without the pasteboard | **two** — `y=139.0` then `y=143.0`, a 4 pt settle |
| with the pasteboard | **one** — stable from the first frame |

★ So the pasteboard does not merely fail to cause a jump; the layout it
produces is *steadier* than today's. The seeding works.

#### 3b. ★★★ WHAT ACTUALLY BREAKS, stated as what was observed

**The canvas stops receiving pointer input entirely.** Not a mis-aimed
click, not a coordinate error — no input at all.

| observation | value |
|---|---|
| `canvas-pointer` events in a driven run | **0**, at both `--doc-point`s tried |
| the page's published rect | `[[296.0 269.7] - [764.0 631.3]]` |
| the canvas viewport | `[[288.0 139.3] - [772.0 762.0]]` |
| where the page sits in it | **centred on both axes, wholly inside, fully visible** |
| rendering | unaffected — an offscreen run reaches `drawn=14` exactly as the baseline does |

So the geometry is right, the page is where it should be, it is drawn, and a
click computed from the application's own published rect lands inside it —
and the canvas never sees a pointer.

★★ **That is a much sharper clue than the one this row gave first**, and it
points somewhere entirely different: at input and widget allocation, not at
coordinates. The two candidates worth starting from, neither confirmed:

- The scroll content is allocated as one rect with `Sense::hover()`
  (`canvas/mod.rs:662`) and the pages are placed inside it with `ui.put` /
  `allocate_rect`. Before the pasteboard, that outer rect was exactly the
  strip; now it is larger than the strip on every side. Whether that changes
  which widget egui resolves a press against is the first thing to test.
- `visible_rect` is built from `doc.last_scroll_offset` — **the previous
  frame's** offset — and on the frames right after seeding that is still
  zero, which now names the far corner of the pasteboard rather than the top
  of the strip. Whether the pages allocated on those frames are the ones the
  pointer is over is the second thing to test.

★ Both are cheap to test and neither was tested, because the first
diagnosis was believed. **Test the input path before touching the
arithmetic again** — the arithmetic is not what is wrong.
#### 3c. ★★★ BISECTED 2026-08-21. One suspect cleared, the other located

Three driven runs, each changing **one** thing from the last, all at
`--doc-point 0,300,500` with `resize_scales_a_shape`:

| # | what was applied | result | `canvas-pointer` events |
|---|---|---|---|
| 1 | scroll content **+200 pt** each axis. No seeding, no arithmetic change | **PASS** | 19 |
| 2 | scroll content **+ a whole viewport** each axis. Still no seeding | SKIP | 9 |
| 3 | the full change: pasteboard **and** seeding | SKIP | **0** |

**Run 1 clears the allocation suspect outright.** Enlarging the
hover-sensing content rect so it is no longer exactly the strip does not
cost the canvas its pointer input — the check passes and the gesture
completes. Whatever is wrong is not that the pages stopped being the widget
egui resolves a press against.

**Run 2 explains itself and is not a defect.** With a full pasteboard and no
seeding, the scroll offset is still zero, which now names the far corner of
the pasteboard. Measured: the page's rect was
`[[780.0 761.7] - [1248.0 1123.3]]` against a viewport of
`[[288.0 143.3] - [772.0 762.0]]` — **the page is entirely outside the view**,
exactly one pasteboard away, which is precisely what seeding exists to fix.

★ It also turned up something a harness author needs to know: **the
application publishes `canvas rect=` for a page that is off-screen.** The
rect is the page's *allocated* rect, not its visible one. A check that maps a
document point through it will compute a screen point outside the window and
click on whatever is there — which is what run 2 did, landing at page
coordinates of `-2529`.

#### 3d. So the remaining mystery is narrow, and it is not geometry

In run 3 the seeding **works**: the page's rect is
`[[296.0 269.7] - [764.0 631.3]]` inside a viewport of
`[[288.0 139.3] - [772.0 762.0]]` — centred on both axes, wholly visible.
And the canvas receives **nothing**.

★★ The new clue, which run 2 makes visible by contrast: in run 3 the trace
carries **one** `canvas` line and `drawn=0` for the whole run, where run 2
climbs through `drawn=1 … 10`. **The application barely advances.** An
offscreen smoke launch with the same seeded build reaches `drawn=14`
normally, so it is not that seeding freezes the shell — it is something
about the seeded build *in a driven run*.

So the question to start from next time is **not** "where is the page" but:
*why does a seeded build stop advancing frames when the window is raised and
driven?* Candidates worth trying, cheapest first:

1. Call `.scroll_offset()` on **every** frame from the stored view rather
   than once behind a flag, and see whether input returns. If it does, the
   one-shot is interacting with `ScrollArea`'s own state rather than seeding
   it.
2. Check whether anything is requesting a repaint after the seed. `drawn=0`
   with pages visible means rasters were requested and never arrived, which
   is a repaint question, not a layout one.
3. Seed by writing `doc.view`'s stored offset **before** the scroll area is
   built, so the existing offset path carries it and no `.scroll_offset()`
   override is needed at all.

★ Candidate 3 is the one to try first on design grounds: it removes the
override entirely rather than tuning it, and the override is the only thing
run 3 has that run 2 does not.

#### 3e. ★★★ BISECTED FURTHER 2026-08-21. It is the SCROLL OFFSET ITSELF

Four more driven runs. Each changes one thing; all at
`--doc-point 0,300,500` with `resize_scales_a_shape`.

| # | seeding | offset that resulted | frames advance? | `canvas-pointer` | verdict |
|---|---|---|---|---|---|
| 4 | write egui's `scroll_area::State` before the area is built | **`[0,0]`** — did not apply | yes, `drawn=10` | 11 | page off-screen |
| 5 | `.scroll_offset(vec2(100, 100))` | `[100,100]` — applied | yes | 9 | page still off-screen |
| 6 | `.scroll_offset(vec2(484, 492))` — the magnitude the real seed produces | `[484,492]` — applied | eventually | **0** | **page ON-screen, input dead** |

**Run 4 kills the nominated fix.** Pre-writing `scroll_area::State` does not
take — egui reports `off=[0,0]` — almost certainly because it clamps a
restored offset against a content size it does not know on the first frame.
So *"seed the state instead of overriding"* is not available, and the
override is not avoidable that way.

**Run 5 clears the override mechanism.** `.scroll_offset(..)` with a small
value applies cleanly, the application advances, and the canvas keeps its
pointer input. Nothing about forcing the offset is inherently harmful.

**Run 6 is the whole defect, reproduced from a HARD-CODED CONSTANT.** No
pasteboard arithmetic is involved in choosing it — it is two literals. The
page's rect settles at `[[296.0 269.7] - [764.0 631.3]]`, one stable value,
wholly inside a viewport of `[[288.0 139.3] - [772.0 762.0]]`. A click
computed from that rect lands at roughly `(385, 484)`, comfortably inside
both. **The canvas receives nothing.**

★★ So the cause is neither the arithmetic, nor the allocation, nor the
seeding mechanism. **A large applied scroll offset costs the canvas its
pointer input**, while leaving layout, drawing and the published rects
entirely correct. That is a much smaller and much stranger problem than any
of the three this row has previously blamed.

#### 3f. ★★★ ANSWERED 2026-08-21: it is a SEQUENCING bug, not a shell defect

The experiment was run, as a permanent check —
`scrolling_far_keeps_the_canvas_its_pointer_input`:

```
[PASS] scrolling_far_keeps_the_canvas_its_pointer_input
       before scrolling: 1 pointer event(s)
       scrolled to an offset of 1600 pt
       after scrolling: 20 pointer event(s)
```

**1600 pt — more than three times the offset that killed input when it was
forced — reached with the wheel, and the canvas keeps its pointer.**

So today's shell is fine, the operator is not meeting this, and O23 was not
being blamed for somebody else's defect. Both good outcomes.

★★ **Which settles the diagnosis by elimination.** It is not the magnitude of
the offset, not the arithmetic, not the allocation, and not the override
mechanism. **It is forcing an offset on the frame the content is first laid
out**, before egui knows how big that content is.

That also explains run 4's failure to take: pre-writing `scroll_area::State`
was clamped away against an unknown content size. Same cause, other symptom.

#### 3g. The fix, now specific

**Seed one frame late.** Let the first frame lay the content out with egui's
own offset, and apply the seed on the second, when the content size is known
and the offset will neither be clamped nor arrive mid-layout.

The cost is one frame showing the unseeded view. At a full-viewport pasteboard
that frame shows blank paper, which is visible — so the seed wants to be
**silent**: either the canvas skips its first paint, or the pasteboard starts
at zero and grows on the second frame. The second is cheaper and has no
flicker, because a content size that grows under a correct offset moves
nothing on screen.

★ `canvas_offset_seeded` becomes a small counter rather than a flag, and the
row's earlier three candidates are all retired.
#### 4. What survived, and what it is worth

The pure arithmetic was written and proven before it was reverted, and it is
reconstructible in minutes from this row:

- `pasteboard(viewport) = viewport × 1.0` — the fraction comes from his two
  sentences: half a viewport reaches the screen's centre, a whole one reaches
  the opposite corner. **A fraction, never a constant number of points**, or
  it stops satisfying the requirement when the window is resized.
- `content_extent(display, viewport) = display.max(viewport) + 2 × pasteboard`
- `strip_margin = margin + pasteboard`, and `margin` stays as it is
- every scroll clamp moves from `display − viewport` to
  `content_extent − viewport`
- `strip_to_scroll(in_strip, strip, viewport)` for callers that already have a
  strip-space position — `strip::page_scroll_offset` is the one that exists

Five unit tests changed, each pinning the old unpadded model, and each
correctly. **That is the useful signal**: they are the inventory of what the
pasteboard changes.

### What it needs, in order

1. **A** first, because **B** is unreachable without it.
2. The `canvas::geometry` audit, with its unit tests extended to cover
   `content != strip` — that arithmetic is pure and is exactly what a unit test
   is good for.
3. A driven check per page **edge**, as a regression guard once the pasteboard
   lands. ⚠️ **Not** because the resize grips share the defect — they do not;
   see `O22`'s correction. Their centres sit ON the box edge, so their inner
   half is always inside the canvas and always grabbable. Only the rotate
   handle's centre is outside the box.
4. Re-run `rotate_handle_turns_a_selection` at **both** `--doc-point`s. One
   point passing is what hid `O22` for a day.
5. **B**, as its own piece of work, starting with the three questions above
   answered against `pdfce-core` source rather than assumed.


## O22 — An object near the top of the view cannot be rotated: its handle is off-canvas

**Found:** 2026-08-21, by driving `rotate_handle_turns_a_selection` at a second
`--doc-point`. **This is the cause of Ken's *"I also can't drag and rotate text
on the screen yet"*** (`O20`), and it is not about text.

**Status:** **CONFIRMED BY DRIVING, WITH NUMBERS. NOT FIXED.** The fix is a
convention question and is not being improvised.

### The evidence

```
--doc-point 0,300,500    rotate_handle_turns_a_selection   PASS
--doc-point 0,1211,1021  rotate_handle_turns_a_selection   FAIL
--doc-point 0,1211,1021  resize_scales_a_shape             PASS
```

Resize passes at the same point that rotate fails, so the object is selected,
the outline is drawn and the eight resize grips are reachable. Only the ninth is
not.

### The arithmetic, from the application's own trace

| | |
|---|---|
| the canvas viewport | `rect=[[296.0 143.0] - [764.0 504.6]]` |
| the selection outline | `rect=[[614.1 150.2] - [753.9 224.0]]` |
| `ROTATE_STEM_PX` | `20.0` (`canvas/handles.rs:335`) |
| `GRIP_SIZE_PX` | `8.0` (`canvas/handles.rs:100`) |

`Grip::Rotate.anchor()` is `(mid.x, bounds.top() - ROTATE_STEM_PX)` —
`handles.rs:271` — so the handle's centre is at **y = 150.2 − 20 = 130.2**, and
its square spans **126.2 → 134.2**.

**The canvas begins at y = 143.0.** The whole handle is 9 pixels above the top
of the canvas.

### Why it fails twice over

- **It is not visible.** The painter draws into the canvas's clip rect, so the
  handle is clipped away entirely. The operator sees eight grips and no ninth,
  and reasonably concludes rotate does not exist — which is precisely what Ken
  concluded, and what three of our own documents also said.
- **It is not reachable.** The press never arrives at the canvas widget at all;
  it lands on whatever occupies that strip of the window, which is the ribbon.

★ So this is convention `handles.md` **H7** — *a handle that cannot act is not
drawn* — failing in its more dangerous direction: the handle is not drawn **and
cannot act**, while the feature is present and correct everywhere else. That is
why it reads as "rotate is missing" rather than as "rotate is broken".

### The general shape

**Any selection whose top edge is within `ROTATE_STEM_PX + GRIP_SIZE_PX / 2` —
24 pt — of the top of the viewport cannot be rotated.** Nothing to do with what
kind of object it is. On a CAD sheet scrolled to the top, that is the title
block, the sheet number and the top row of a BOM: exactly the things an operator
reaches for first.

It is also why `O20` looked like a text problem. The BOM row that
`--doc-point 0,1211,1021` names happens to sit at the top of the sheet.

### ~~The fix is a convention question — do not improvise it~~ — ANSWERED

★ **Ken settled it on 2026-08-21: `O23`.** He asked for the pasteboard and
then for more of it than was proposed here — *"I should also be able to move
the view of the corner of the page to the center of the screen, or even all the
way vertically to the opposite corner"*. Candidate 3 below, sized at **one
viewport on every side**. The analysis is kept because the two rejected
candidates and their reasons are still the record of why.

Three candidates, and the standing rule is *use the conventional interaction,
never invent one*:

1. **Flip the handle below the box when there is no room above.** Cheap and
   local. ★ But it is an **invention** for this gesture: no program in the class
   flips a rotate handle, and an operator who learned "the rotate handle is
   above" would find it moving for reasons they cannot see.
2. **Clamp the handle inside the viewport.** Rejected on sight — it detaches the
   handle from the box it belongs to, breaking convention **C7** (*the drawn
   outline and the live target are the same shape*) to fix an H7 violation.
3. **★ Give the canvas scroll padding, so the page can always be scrolled away
   from the viewport edge.** This is what Illustrator, Acrobat and Inkscape all
   do — you can scroll past the edge of the page, and the pasteboard is why a
   handle at the extreme edge of the sheet is always reachable. It fixes a whole
   class of edge problems rather than this one symptom, including the eight
   resize grips on an object flush with the left or bottom edge, which have the
   same defect and no check yet.

**Recommendation: 3.** It is the conventional answer, it is the only one that
fixes the resize grips too, and it needs no new rule for the operator to learn.
The trace shows `off=[0.0 0.0]` — the canvas is scrolled hard against its own
top with nowhere further to go.

### ★ The check now names this cause instead of three wrong ones

Its first failure listed `Grip::is_resize`, `gesture::meaning` and
`needs_targets` — three real hazards, none of which is what happened, all three
inside the application, and all three in the ROUTING when the defect is in the
LAYOUT. A reader would have gone looking in the wrong file, with a specific and
plausible instruction to do so.

It now measures the handle against the canvas's own declared rect first, and
says so with numbers:

```
★★ THE ROTATE HANDLE IS OFF-CANVAS - defect O22, and NOT a routing problem.
   The selection's top edge is at y=150.2, the handle therefore spans from
   y=126.2, and the canvas begins at y=143.3. The handle is 17.1 point(s)
   above the top of the canvas ...
```

It still **passes** at `0,300,500`, so this is a diagnosis rather than a
blanket failure.

★★ The general rule it earned, and it is the third instance in one evening:
**a confident, specific, wrong accusation is worse than a vague one**, because
it is actionable and it aims somebody at the wrong file. A check that can rule
a cause OUT should.

### ⚠️ CORRECTION 2026-08-21: the resize grips do NOT have this defect

This row claimed, twice, that *"the eight resize grips have the same latent
defect on the left and bottom edges"*. **That is wrong, and it was written
here without being checked** — hours after this same file recorded the rule
that a claim about what the code does is verified against source, not
asserted. It was then repeated to the operator and promoted into
`CONTINUE.md` as scheduled work.

The geometry, from `canvas/handles.rs`:

| affordance | where its CENTRE sits | how far outside the box |
|---|---|---|
| the eight resize grips | **on** the box's edge or corner (`anchor`, `handles.rs:260-267`) | half a grip — `GRIP_SIZE_PX / 2` = **4 pt** |
| the rotate handle | `bounds.top() - ROTATE_STEM_PX` | **16 – 24 pt**, entirely outside |

`handles.rs:269` says it in the source, in as many words:

> *"Above the top edge, centred, by the stem's length. **The one grip whose
> centre is OUTSIDE the box**, which is what the offset is for."*

So a resize grip's centre — and its whole inner half — is inside the
selection box, and therefore inside the canvas whenever the object is
visible at all. **It can always be grabbed.** The rotate handle has no part
inside the box and can be entirely off-canvas, which is why it and only it
disappears.

★ **What is left of the claim, stated accurately**, because there is a
residual and it is cosmetic rather than functional: against a viewport edge
the outer half of a grip is clipped, so it is drawn as a 4 pt sliver rather
than an 8 pt square, and its effective target shrinks from 12 pt (8 + 2 slack
each side) to about 6. Harder to hit, never impossible.

**Consequence for the plan:** the per-edge driven check drops from *"needed
to cover a latent defect"* to *"a reasonable regression guard once the
pasteboard lands"*. `O23` is the whole of the work; there is no second
defect waiting on the left and bottom edges.

★★ The shape, for the third time in one day: **an unverified claim about an
ABSENCE or a DEFECT costs nothing at the moment it is written and is
expensive later**, because nothing fails when it is wrong — it just quietly
shapes a plan. Analysis, not driving: no fixture point flush with a page
edge was available to aim at, and this is labelled as reasoning from the
constants rather than as a measurement.

### What it needs

1. Scroll padding around the page in the canvas's scroll area, sized so every
   affordance of a selection flush with any page edge is reachable.
2. A driven check per edge — top, bottom, left, right — because the resize grips
   have the same problem and nothing has ever aimed at them there.
3. ★ Re-run `rotate_handle_turns_a_selection` at **both** `--doc-point`s
   afterwards. One point passing is what hid this for a day.


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

★★ **ANSWERED 2026-08-21 by driving: see `O22`.** It is not about text at all.
The rotate handle sits 20 pt ABOVE the selection box, and the object he was
aiming at is near the top of the sheet — so the handle is drawn 9 pixels above
the top of the canvas, where it is clipped away and where a press lands on the
ribbon instead. Any selection within 24 pt of the top of the view has this,
whatever kind of object it is.

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

★★ **And it is now DRIVEN**, 2026-08-21, on the real application with the real
Windows clipboard:

```
ctrl_c_copies_text_to_the_os_clipboard   PASS
  the sweep selected 10 character(s); the clipboard holds 10 after Ctrl+C
  clipboard begins "- 22 - 250"
```

The check reads the **operating system's** clipboard from outside the process,
which is the only oracle that can see this defect: the failure was never in a
function's return value but in which of two handlers reached the OS last, and a
trace cannot see that either. It **clears the clipboard first** — without that,
*"the application did nothing"* and *"the application copied correctly"* are
the same observation whenever an earlier run left the right text behind.

The row still stays open until you close it.

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

★★ **And parts A and C are now DRIVEN**, 2026-08-21:

```
select_filter_changes_what_a_click_hits   PASS
  with every class on, the click selects
  with every class off, the same click selects nothing
  switching them back on restores it
```

Deliberately **not** *"the popup opens"*. That is already a unit test, and it
is also the one claim that stays true of an inert control — which is precisely
what this popup was for an hour this morning.

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
