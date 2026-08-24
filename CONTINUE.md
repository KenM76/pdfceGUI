# CONTINUE — handoff

## 2026-08-24 (later) — O28, O29, O30: three asks, all driven

**Clean tree. 17/17 gates. 1,707 + 385 + 144 tests, 0 failing.** Re-measure
before quoting; the commands are in `RESUME.md`.

The operator, after O26 landed:

> *"If I press the Fit width or fit page button the view should center to the
> width as well or center the page. Adobe has fit height, so add that too.
> Also when in single page view there should be an option on screen near the
> button to scroll or flip through pages, or the current way it is now when the
> scroll wheel is used."*

| | |
|---|---|
| **O28** | a fit now **places the view**, not just the scale. It pins the axes whose extent it decided and keeps the operator's position on the rest, clamped to the page. This is a consequence of O23's pasteboard and the second one: before it, a fitted page had nowhere to be except the middle |
| **O29** | **Fit height**, end to end — mode, `fit_scale` arm, status-bar button, registered command, ribbon item, context-menu entry, icon, opening-fit preference, on-disk token |
| **O30** | the **Flip pages** wheel toggle beside the page buttons. Off by default; not drawn at all under a continuous display (R9); takes effect on the very next notch |

### ★★★ Both new checks were falsified, and both found defects in themselves first

* `a_fit_command_puts_the_page_on_screen` pans thirty notches into the
  pasteboard, **asserts it got there**, then presses each of the three buttons.
  With the placement disabled it reports *"the vertical margins are 261.5 and
  −4.4, so the page is not centred"*. ★ Its first draft demanded the page sit
  flush against the viewport and failed a correct build by exactly
  `CANVAS_MARGIN` — the application had never promised that.
* `the_wheel_turns_pages_when_the_operator_asks_it_to` makes five claims, and
  the first is that the **default is silent**, so a build that flipped
  unconditionally could not pass. ★★ It found two harness defects before it
  found anything else: it **mutates a persisted setting and did not normalise
  at the start**, so its second run accused the shipped default; and its
  absence claim passed an event count to a helper that wants a line number.
  Both are standing RAG lessons in new costumes. The application now publishes
  `wheel=` on its status line so the check can read what it is about to change.

Both pass twice in a row, which is the property the first defect cost.

### R2 forced four splits

`viewer::fit`, `app::status::fit`, `app::dispatch::zoom`, `canvas::fit` and
`canvas::offset`. The last is the one worth reading: *who decides where the
view is this frame*, six ranked sources with the argument for each rank, which
had grown inside `canvas::show` where nobody could see them together.

★ **A caution from doing it**: `git checkout --` on four files, meant as an
inspection, threw away an hour of uncommitted work that had to be retyped.
Commit before restructuring.

---

## 2026-08-24 — O26: zoom out, and the seven reasons the page ended up in a corner

**Clean tree. 17/17 gates. 1,691 + 385 + 144 tests, 0 failing.** Re-measure
before quoting; the commands are in `RESUME.md`.

The operator, 2026-08-24: *"zoom in works flawlessly now. The panning works.
Zoom out has a small bug where it sometimes seems to reposition the page so
that it is off screen in the far bottom left corner. This happened when I
zoomed back from around 2 million% but seems to happen at other junctions
too."*

**Seven independent causes, all fixed, all with the same symptom.**
`OPERATOR_REQUESTS.md` O26 has each one with its measurement. The short form:

| | |
|---|---|
| **O26a** | `Strip::page_at_view` was fed a **content-space** rect. Usually returned `None` — so scroll-driven current-page tracking has been **inert since the pasteboard landed** and nothing said so. Occasionally returned a page several pitches wrong, which then chose the frame of reference for every position solve. **One wheel notch at 30 % took the view from page 1 to page 8**, confirmed by screenshot |
| **O26b** | Ctrl+wheel gated on the **acting page's** hover, so the pointer over a neighbouring page, a gap, or the pasteboard did nothing. Turned O26a's jump into a **freeze** |
| **O26c** | the acting page's **rect** and the acting page's **extent** were different pages whenever the fallback fired. Invisible on a uniform document; `SW41177.pdf` mixes 1584×1224 and 1224×792 sheets |
| **O26d** | the zoom anchor **did not name its page**, so the frame that solved it added whichever page was current a frame later |
| **O26e** | `CanvasFrame::offset` was reconstructed from a scroll offset the deep tier **forces to zero** — so every deep frame recorded a fictitious "before", and the first zoom back across the threshold put the page's own origin under the pointer |
| **O26f** | the exit from the deep tier is now solved in `f64`, on the frame that leaves it |
| **O26g** | the strip was placed with `Rect::from_center_size`, a catastrophic cancellation at 4.6 × 10⁸ points. Proven equivalent, but **no measured improvement** — do not credit it with one |

### ★★★ The two lessons, and they outrank the fixes

1. **A check that travels in one direction tests one direction.**
   `zoom_keeps_place` climbs to 10¹² % one notch at a time with a tolerance
   fine enough to catch a hundredth of a point, and had **never once rolled
   the wheel the other way**. Its own header calls the tier hand-over *"half
   of what this check is for"* — true of the upward crossing only. The new
   `zooming_back_out_keeps_the_view` is the inverse.
2. **A check that may decline to judge cannot fail.** A "record instead of
   assert above the measured noise floor" hatch, with a careful written
   argument for why it was a boundary on the subject rather than a loosened
   tolerance, **recorded 1,161 pt — the whole page — and reported PASS on its
   first driven run**, hiding O26d. Removed. Both zoom checks are RED on the
   O27 residual and that is the correct state.

### ⚠️ Both zoom checks are RED, deliberately — see O27

With all seven fixed, an anchored notch still moves the view **10–35 screen
pixels** on the `f32` scroll tier above ~130,000 %. On the `deep` tier the same
measurement is **±0.05 px**. It is bounded jitter, not drift: sixteen readings
at ~10⁶ % oscillated in a 43 px band and did not accumulate. Cause **not
established** — the predicted `f32` accumulation is ~±2 px, so something an
order of magnitude larger is unfound. `OPERATOR_REQUESTS.md` O27.

### What is NOT done

* **The full driven suite has not been run** since these fixes. Only the two
  zoom checks were driven; the operator returned to the machine. Everything
  else is unverified against this tree.
* **Not published.** No `package-portable.py` run — the standing rule says
  publish after every keeper build, and this one has not been through the
  suite.
* `canvas/mod.rs` split three ways to hold R2 (`canvas::deep`, plus the
  current-page tracker into `canvas::strip`). Largest file is now
  `canvas/markup.rs` at exactly 1,500.

---

## 2026-08-21 evening

**Type `continue` and start at §2.** Everything above it is state; §2 is the
work queue, in order.

**Clean tree. 17/17 gates. 1,650 + 385 tests**, measured at `376fc15`.
Re-measure before quoting any of that; the commands are in `RESUME.md`.

⚠️ **The driven figure is deliberately not quoted here, because it is stale and
cannot currently be refreshed.** The last full run — 55 passed, 1 failed, 12
skipped — predates the three O24 pieces and the engine bump, and driving is
blocked on an idle desktop (see the box below). **Do not carry that number
forward as if it described this tree.** Run it when the machine is attended.

★ The one failure in that run was `multi_node_move_moves_every_picked_anchor`,
which has never passed on any build — an unbuilt path, not a regression, and
the selection model underneath it is now proven by two unit tests.

**Published: `OneDrive\pdfceGUI2`, built 2026-08-21 16:0x**, with the Select
popup working. `pdfceGUI1` holds the 15:47 build as the fallback.

★ **The published build predates the two new driven checks and O22's finding.**
Nothing in it is wrong that was not wrong before — the checks are harness
work, and O22 is a defect it already had — but it is not the tip.

## ★★★ STATE, MEASURED 2026-08-21 LATE

**Driven: 55 passed, 1 failed, 12 skipped.** Up one pass from the evening's
54, and the one failure is **newly exercised rather than newly broken** — read
the box below before treating it as a regression.

**O23's first half is IN and driven.** Free navigation works: one whole
viewport of pasteboard on every side, so any corner of the page reaches any
point of the screen. It took four attempts and the cause of the first three
was a single missing coordinate conversion —
`geometry::scroll_to_strip`, absent from `visible_rect`.

★★★ **The lesson, and it is the biggest one of the day: READ WHAT THE
APPLICATION SAYS ABOUT ITSELF, FIRST.** Every failing trace from the first
attempt onward carried

```
pdfce-diag canvas-unavailable reason=nothing-visible
```

which states the cause exactly. It was never grepped for, because each search
was for the SYMPTOM — no pointer input, stalled frames, a page rect that
looked right — and those sent the diagnosis to the arithmetic, then the
allocation, then the seeding mechanism, then the offset magnitude. All four
were innocent. **Grep the trace for what the program is reporting before
grepping it for what you are seeing.**

### ⚠️ THE ONE FAILING CHECK, AND WHY IT IS NOT (EVIDENTLY) A REGRESSION

`multi_node_move_moves_every_picked_anchor` **FAILS**, reproducibly, in
isolation as well as in the suite.

On the baseline it **SKIPPED**, with *"the subpath has one anchor. Aim
--doc-point at a polyline."* With the pasteboard it finds **two** anchors,
clicks both — the second with Shift — and reports that **one** ends up
selected.

So the check now reaches a rung it could never reach before and fails there.
That is a different operand, not a changed behaviour, and the pasteboard is
the likely reason the operand changed: a different scroll position makes a
different page current in a continuous strip.

★★ **NARROWED 2026-08-21: the model is correct, so the fault is downstream.**
Two unit tests were written for the rung nothing had ever covered —
`shift_picking_a_second_anchor_adds_it_rather_than_replacing` and
`shift_picking_a_selected_anchor_removes_it` — and **both pass**.
`SelectionState::pick_within` adds a Shift-picked anchor and toggles one that
is already picked, and `normalise` only collapses entries that span different
objects, which two anchors on one subpath do not.

So it is **not a pasteboard regression** (the check had never run before) and
**not a selection-model defect** (now proven, and guarded). What is left is
the driven path: either the harness's Shift+click is not delivering the
modifier, or the shift is not reaching `pick_within` from the real event, or
the second click resolved to the same anchor. All three are in
`canvas::clicking` / the harness, not in the model.

★ Worth knowing before chasing it: the check has **never passed**, on any
build. There is no evidence this ever worked, so treat it as an unbuilt path
rather than a broken one.
### ★★★ AND DRIVING FOUND THE DEFECT KEN IS BLOCKED ON — `O22`

**An object near the top of the view cannot be rotated. Its handle is drawn
nine pixels above the canvas.** `rotate_handle_turns_a_selection` passes at
`0,300,500` and fails at `0,1211,1021`, while `resize_scales_a_shape` passes at
both. The numbers are in `O22`; the short version is that
`Grip::Rotate.anchor()` is `bounds.top() - 20.0`, and a selection flush with
the top of the viewport puts that outside the clip.

It is not about text, which is what `O20` assumed. **Any selection within 24 pt
of the top of the view**, whatever it is made of.

★ **The fix is settled and was attempted.** Ken answered the convention
question on 2026-08-21 and asked for more than `O22` proposed — any corner of
the page reachable to anywhere on screen — which is `O23`. The pasteboard was
built that evening, passed 1,634 tests and 17 gates, **broke selection on the
real application, and was reverted.** `O23` carries the three things measured
on the way. ⚠️ Its first diagnosis — *"the page jumps a frame after opening"* —
was **wrong and was never measured**; it compared two builds. Re-measured, the
pasteboard layout is *steadier* than today's (one stable rect against today's
two). What actually breaks is that **the canvas receives no pointer input at
all**, while the page is centred, visible and correctly drawn.

★★★ **Bisected to two literals.** It is not the arithmetic, not the
allocation, and not the seeding mechanism: `.scroll_offset(vec2(484, 492))` —
a hard-coded constant, no pasteboard anywhere — reproduces it, while
`vec2(100, 100)` does not. **A large applied scroll offset costs the canvas
its pointer input**, leaving layout, drawing and the published rects correct.

★ `O23` §3f names the one experiment that decides what this is: drive the
WHEEL to a similar offset on today's unmodified shell and click. If input dies
there too, this is a pre-existing defect the operator meets whenever they
scroll far, and it outranks O23 entirely. Run that before building anything.

⚠️ Do not plan around this row's first claim that the eight resize grips share
the defect. **They do not** — their centres sit ON the box edge, so their inner
half is always inside the canvas and always grabbable. Only the rotate handle's
centre is outside the box. The correction, with the geometry, is in `O22`.

★★ **The transferable half: one `--doc-point` passing is what hid this for a
day.** A driven check aimed at a single point proves the gesture works *there*.

### Two harness lessons from the same evening, both now in the RAG

Both new checks accused the application before they were right, and both times
the harness was at fault:

- one asked whether a `ui-rect` region had been published. That channel is a
  **change log**, so a region that stopped being drawn is still in the trace —
  it reported *"THE FILTER IS DECORATIVE"* about a build whose own trace said
  `sel=0`;
- the other watched `text-selection`, which no build has ever emitted (it is
  `canvas-text-selection`), and reported SKIPPED blaming the fixture.

**Read the trace before believing the check.**

And one that is about harness design: **a driven check that mutates persisted
state must establish that state at the START.** Restoring it at the end only
runs when the check passed, which is the case that did not need it — the filter
check failed at step 2, left every class switched off on disk, and its next run
blamed `--doc-point`.
---

## ⚠️ DRIVEN RUNS NEED AN ATTENDED MACHINE — found 2026-08-22

After several hours with no human input, **every driven check began reporting**

```
the window ... could not be brought to the front. Windows refuses
SetForegroundWindow to a process without foreground rights
```

No code change between the working runs and the failing ones; unit tests and
all 17 gates stayed green. `SetForegroundWindow` is granted only to a process
that is already foreground, received the last input, or has recent user input
behind it — and on an idle desktop a background harness has none of those.

★ **So the idle machine that looks like the ideal time to run the suite is the
one time it cannot run.** Schedule driven runs when he hands the machine over,
not overnight. And read a window-activation SKIP as an ENVIRONMENT verdict, not
an application one — it survives re-running, so it reads as a defect.

★ Clean up orphans first: a killed harness run leaves the application running,
and that instance competes for the foreground. `taskkill //IM pdfce-gui.exe
//F`. Note `taskkill //PID` against a pid from `ps -W` fails — that column is
Git Bash's pid, not Windows'.

★★ **How much is left unattended, measured rather than assumed: `--no-input`
verifies 4 checks of 68.** Nineteen skip on "this check clicks a mode segment"
alone. An input-disabled run is not a reduced suite, it is a different and much
smaller thing.

The refinement worth having: the four that ran include capture and dialog
checks, so **capture does not need foreground rights — only synthesised input
does.** The harness is not disabled on an idle desktop; its clicking is.

So the unattended set is: unit tests, the gates, an offscreen launch
(`PDFCE_DIAG_VIEWPORT`) asserting on the trace, and those four. Everything that
asserts *a gesture produces a result* needs an attended machine.

## ★★ A RELEASE IS OWED WHEN O24's STEP 2 LANDS

Ken, 2026-08-21: *"when you complete the step 2 zoom release to git and put on
OneDrive."*

**`git push origin main`, then `tools/package-portable.py`.** ★ This would be
the project's **first push** — `origin` is `github.com/KenM76/pdfceGUI.git` and
the local branch is 253 commits ahead of it, last tag `v0.3.0`. Not a routine
increment.

`O24`'s release section carries the five preconditions, every one of which has
bitten this project already. The two most easily forgotten: **bump the engine
first** (`O24` depends on two commits this lock predates, so a stale pin ships
a release without the thing it is a release of), and **drive the suite at BOTH
`--doc-point`s**, because one point passing is what hid `O22` for a day.

★ **Not on step 1.** He named the trigger precisely.

## 0. ★★★ READ THESE FIRST, EVERY SESSION

### `OPERATOR_REQUESTS.md` — the backlog, and the only truth about it

Every ask goes in that file **the moment it is made**. **Only Ken closes a
row.** A status is evidence or the words NOT VERIFIED. A blocked row names the
request file. Nothing is silently rescoped.

★ Row 13b is a **withdrawn** defect report — read it. It was written up in good
faith, it was wrong, and the retraction is left standing beside what it
retracts. That shape is the standard for this file.

### `D:\dev\rag\ui-conventions\` — and the gate behind it

Five gesture classes, each a numbered list carrying where the rule comes from
and the failure mode when it is absent. `tools/gates/check-conventions.sh`
makes every registered surface answer every row in its own source. **It cannot
check behaviour and does not pretend to** — it checks that the question was
asked, which is the whole of the problem.

Of the fourteen gaps its first run found, item 11 (no selection inside a text
draft) closed on 2026-08-21, keyboard and pointer. The rest are O14.

### `D:\Dev\FeatureRequests\pdfce_FeatureRequests\open\`

Read it every session; **empty means nothing is owed**. The engine session runs
in parallel and answers within minutes. The live entries are `note_*` replies
awaiting consumption, not asks.

---

## 1. What happened this session (2026-08-21)

Seven things shipped. In the order they matter to Ken:

1. **The navigation keys walk the page, not the fragment you clicked.** Down
   and Up move to the line below and above **including into the next block of
   text**; End and Home reach the ends of the line he can *see*, however many
   show operators drew it — four or five on a CAD title-block row. **Salvage**:
   the old shell asked `caret_up`, `caret_down` and `line_range_at`, and that
   was its entire contribution; the reassembly was always `pdfce-core`'s.
2. **Every dialog is its own OS window**, not just Print. All thirteen: title
   bar, taskbar entry, second monitor.
3. **A dialog is OWNED by the window it belongs to** — G3, which had been filed
   as impossible. It can no longer fall behind the application.
4. **There is a selection inside a text draft.** Shift+arrows, Shift+Home/End,
   Ctrl+A, and — later the same day — **drag to select, double-click for a
   word**. Closes conventions item 11.
5. **A defect that had shipped: every dialog drew on a BLACK background.** Dark
   text on near-black. Nothing caught it — the windows opened, every control
   was where it said it was, the driven check for *"it opens in its own OS
   window"* passed on all eight. **A screenshot showed a black rectangle.**
6. **The harness learned the program has more than one window.** Six checks
   were failing and six skipping, every one clicking hundreds of pixels from
   the control it named.
7. **The engine moved 14 commits**, including `Pass 97` compositing work and
   `Pass 120.2/120.4`, the engine half of the clipboard.

### ★★ The four findings worth carrying forward

- **A guard that stops repetition does not stop creep.** `Host::fit` grows a
  dialog to fit its body; its first version padded the measurement before
  comparing, so every frame asked for eight more pixels than the last and the
  once-per-size guard never fired *because every size was a new one*. About
  opened at 560×480 and was **1624×746** a few frames later.
- **A `Key` event's own modifiers can be EMPTY while the frame's are right.**
  Measured: `ev=Modifiers::NONE frame=Modifiers { shift: true }`, three
  presses, Shift held throughout. The exact **inverse** of the chord-matcher
  finding filed three days earlier, and **both are true** — choose by what the
  modifier MEANS. A chord is a command and must use the event's alone; a held
  qualifier must use `event || frame`, because over-reading costs one step and
  under-reading destroys accumulated state.
- **A measurement of the wrong surface is indistinguishable from a measurement
  of a broken one.** Twice in one afternoon: a contrast check capturing the
  wrong WINDOW (1.51:1 reported about two headings that render at 15.07:1), and
  `ui_rect_visible` publishing the wrong PART of the right one (a heading two
  points inside a scroll area's bottom edge, measured off the anti-aliased top
  rows of clipped glyphs).
- **A measurement that moves when you turn a knob is not proof the knob is the
  subject.** See §1b.

### ⚠ §1b — What LOOKED broken and was not, which cost the most time

`text_annot_takes_the_keyboard_unclicked` failed for hours with *"drag out a
note box, type without clicking the field, and the words go nowhere"*. It was
written up for Ken as a live defect. **It was not one.**

The check clicked its Accept button through the APPLICATION window's
coordinates while the dialog had its own. The typing worked the whole time —
the dialog took the characters, the field held focus, the draft was right — and
the click that should have committed it landed on a page. One call site,
converted to `frame_of` like the other six, and it passes.

★★★ Chasing the wrong culprit, the program was changed four times to hold the
keyboard harder, and **every change appeared to help**: the dialog visibly held
the foreground while the new code was asking for it and lost it the instant it
stopped. Real, repeatable, beside the point. `FOCUS_FRAMES` went 1 → 8 → 40 →
120 on exactly that evidence and is back at 8, with the story written into its
doc comment so the next reader does not repeat it.

Two of the four changes were kept because they are right on their own terms:
**G3** (above) and **a dialog's position is asserted once**, on the pass it
opens, rather than re-asserted every frame from a value read back out of the
window one frame earlier.

---

## 1c. ★★ WHAT IS PUBLISHED AND NOT VERIFIED

Two things are in the operator's hands and have **not** been checked against a
running binary, because he came back to the keyboard and the harness takes the
cursor:

1. **Drag-select and double-click-a-word inside a text draft.** Unit-tested
   against the real galley, three tests. The driven step is written — step 6 of
   `shift_arrows_select_text` — and has never been run.
2. **The engine, 14 commits forward** to `cbb1ede`. Four are `Pass 97`: the
   compositing formula, non-isolated group backdrops, knockout groups, and soft
   masks applied once per group instead of once per object. **That is the class
   of change that alters how every page rasterizes.**

**They are published anyway, and that is his instruction:**

> *"no it doesn't matter if it has been checked or not. I always want the
> latest build there."* — 2026-08-21

★★ **Understand the correction, do not merely obey it.** A release was held
back earlier the same day on exactly the opposite reasoning, and the reason
that was wrong is already built into the tool: **the other slot holds the
previous build.** He has a fallback by construction, so the cost of a bad build
is a folder swap — while the cost of withholding is that he does not have the
work at all. Driven verification gates *claiming a feature works*, not *putting
the binary where he can reach it*. The two were being conflated.

Disclosure moves rather than disappears: it goes in the report and in the
build's own `BUILD-INFO.txt` (`--note`).

---

## 2. What to do next

His standing instruction is *"continue looping through other tasks"*. In the
order that returns the most:

1. ★★★ **Fix `O22` — the rotate handle is off-canvas near the top of the
   view.** This is what Ken is blocked on, it is confirmed by driving with
   numbers, and it is the only item here that a person is currently waiting on.

   ★ Ken settled the approach on 2026-08-21 and asked for MORE than was
   proposed — `O23` supersedes the fix half of this row. It was attempted the
   same evening, broke selection, and was reverted; `O23` carries the three
   things that were measured, and the one that stopped it is that the page
   JUMPS a frame after opening. Solve the seeding before rebuilding the rest.

   ⚠️ And do not plan around this row's old claim that the resize grips share
   the defect — **they do not**, and the correction is in `O22`. Their centres
   sit ON the box edge; only the rotate handle's is outside it.

   ★ Re-run `rotate_handle_turns_a_selection` at **both** `--doc-point`s
   afterwards. One point passing is what hid this.
2. **Run the full driven suite**, and fix whatever the engine
   bump moved:
   ```
   ./target/release/ui-verify.exe --pdf D:/Dev/temp/pdfce/SW41177.pdf \
       --doc-point 0,300,500
   ```
   Not to gate a release — that has already happened — but because he is
   running unverified code and the sooner that stops being true the better.
   **`0,300,500` is the calibrated point**; `0,1211,1021` aims at a BOM row and
   is right for the text checks and wrong for `rotate_handle_turns_a_selection`.
3. **The three gesture-only dialogs nobody has driven** — Insert pages, Set
   scale, and the unsaved-changes question. They are OS windows now and nothing
   has clicked them. `frame_of` and the driver's focus tracking are in place, so
   each is a check rather than an investigation.
4. **`Pass 120.2/120.4` is in the tree now** — selection to a standalone
   one-page PDF, and the clipboard's cross-application half. That closes
   `OPERATOR_REQUESTS.md` O2's remaining rows; the shell side is a paste target
   and a private Windows clipboard format.
5. **The rest of O14**: unfilled-shape hit testing (only ce dimensions carry a
   real shape), grapheme clusters in the caret, right-click to add or remove a
   perimeter point (both engine verbs exist), the zero-travel guard on three of
   four drag paths.
6. **The transform preflight**, a named gap in `canvas::resizing`: an object
   whose own CTM is singular cannot be transformed and the engine says *do not
   offer a handle*. `transform_preview` is the predicate and it decomposes the
   page, so it needs a cache keyed on `(page, epoch, selection)` shaped like
   `app::cache::FormRunCache`.
7. **Turning existing page text into multiple lines** — O15's remainder. That
   is a reflow, which the engine has and which currently demands the document
   be saved and reopened first.

---

## 3. Blocked on the engine

Nothing on the list above is engine-blocked. The two that were —
`transform_objects` and the object clipboard — both shipped.

The live entries in the request channel are `note_*` **replies awaiting
consumption**, not asks. Read them; several report that a blocker this project
filed was never real.

---

## 4. Environment gotchas

- **`ui-verify` takes the real cursor and keyboard.** If Ken is at the machine,
  every check that clicks will SKIP with a foreground refusal — which is
  correct, not a failure. **51 of 65 skipped once this session** for that.
- **★★ Do not edit source while the suite runs.** The staleness guard fires and
  every check refuses: a file edited mid-run makes every trace describe code
  that is not the code under test. One or the other, never both.
- **★ `--second-pdf` must have MORE THAN ONE PAGE.** A one-page source cannot
  be moved out of — the engine refuses to leave a document with no pages.
  `D:/Dev/temp/pdfce/big.pdf` (5 pages).
- **★ Python heredocs eat the `\` continuation in a Rust string literal**, and
  the result COMPILES: what lands on disk is one long line with the indentation
  baked into the string. `.tmpwork/rewrap.py` repairs a file after the fact and
  `check-string-gaps` catches it at the gate. Use the Edit tool for anything
  with a continuation, or `r"""…"""`, or `chr(92)`.
- **`PDFCE_DIAG_INVOKE=<command.id>` presses one ribbon command once**, in an
  invisible window, through the real dispatcher — the way to verify while he is
  working:
  ```
  PDFCE_DIAG=1 PDFCE_DIAG_VIEWPORT=-4000,-4000,1200,850 \
  PDFCE_DIAG_INVOKE=file.print  target/release/pdfce-gui.exe file.pdf
  ```
  `dialogs_open_in_their_own_window` is built entirely on this and needs no
  pointer at all.
- **`osk.exe` covers the ribbon and swallows synthetic clicks**, UIPI-protected.
  A driven failure on this machine is a harness question before it is an
  application one.
- **`python tools/package-portable.py --verify` after every keeper build**, and
  read the two slot dates it prints. **`--slot <name>` forces the target.**
  ★ `pdfceGUI1` refused the mirror **three times on 2026-08-21** — `WinError 32`
  on the rename, with no process running from that folder, so it is OneDrive's
  own sync client. The failure is safe (a failed rename moves nothing) and the
  cost is that the fallback stops rotating. **If it happens again, find out what
  is holding it.**
- **`cargo update -p pdfce-core -p pdfce-render -p pdfce-print` before every
  build.** The packager does it by default; `--no-update` holds a pin.
- `.tmpwork/edit.py` is the CRLF-safe edit helper.
- **Never `git checkout --` a dirty file.**

---

## 5. Standing rules this project has paid for

- **A trace can say the verb ran. It cannot say the screen changed.** Every
  layout, repaint or clipping defect has exactly one oracle: a rendered
  screenshot. Put a capture on the failure branch of anything that draws. The
  black-dialog defect (§1.5) is the newest instance and the most complete: four
  other oracles said the surface was perfect.
- **A check asserting on an ABSENT line must first ask what else happened.**
- **A fixture that cannot exhibit the hazard proves nothing.** When an upstream
  fix stops your falsifier firing, the assertions beside it stop measuring and
  go on passing. Invert the control or grow the fixture; never just delete it.
- **Two derivations of one position agree at first and separate under use.**
  Five instances now. `egui::Pos2` is screen, canvas, page AND per-viewport
  space, so the compiler cannot object. The newest fix is the right shape:
  `canvas::textedit::hit` publishes the galley that was **drawn**, so the
  pointer hit-tests the same layout the caret is painted from — one derivation,
  two questions.
- **A blocker is a measurement, and the question you measured is part of it.**
- **A predicate with two claimants must exist exactly once.**
- **A knob must not sit at a value chosen to fix something it does not fix.**
- **Registering a command is the only way the GUI may learn a capability
  exists** (R8), and **`egui-shell` never learns what a PDF is** (R7).
- **Unsafe code is quarantined.** `pdfce-gui` and `egui-shell` both
  `#![forbid(unsafe_code)]`; the four `user32` calls that make a dialog owned
  live in their own `native-window` crate, to be deleted rather than ported
  when a toolkit grows an owner option.

---

## 6. His standing criticism — keep it in view

> *"it shouldn't take multiple 3 hour sessions each day to figure out how to get
> a cursor to move and edit text on it, or get shortcuts to work for basic
> functions."*

The largest bucket of this fortnight's defects is **conventions nobody
audited** — not engine gaps and not hard problems. The conventions corpus and
its gate are the structural answer; use them **before** building an interaction,
not after he reports it.

And when he reports something, **believe him and go find it.** Every report this
fortnight was precise and correct, including the ones that sounded at first like
misunderstandings — most recently *"text editing doesn't work"*, which was true
for 99 % of the text on his documents while every driven check was green,
**because the checks drove fixtures this repository authored.**

★ The converse arrived this session and is the harder discipline: **when *you*
report something broken, hold it to the same standard.** Row 13b was a defect
report written from a failing check, and the check was wrong. Before writing up
a defect, prove the measurement was of the thing you named.
