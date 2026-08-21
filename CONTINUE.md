# CONTINUE — handoff, 2026-08-21 morning

**Clean tree. 16/16 gates. 1,592 tests. Driven suite: 53 passed, 0 failed,
12 skipped** on his own drawing at `--doc-point 0,300,500`.

**Newest published build: `OneDrive\pdfceGUI2`, 2026-08-21 07:53**, engine
`10de389`, shell `6b20ab3`. `pdfceGUI1` holds 2026-08-20 06:03 as the fallback.

★ **Slot 2 again, and it was not the rotation's choice.** The packager wanted
`pdfceGUI1` — the older slot — and OneDrive held it locked across two attempts
(`WinError 32`, a failed rename, so nothing was moved and it still holds its
build in full). Forcing slot 2 replaces yesterday's build and leaves the
2026-08-20 fallback intact, which is the right trade and the same one made on
2026-08-20. **If slot 1 is locked again next session, that is now twice: worth
asking whether something is holding it open.**

The previous edition of this file is in git history at `5221e61`.

---

## 0. ★★★ READ THESE FIRST, EVERY SESSION

### `OPERATOR_REQUESTS.md` — the backlog, and the only truth about it

Every ask goes in that file **the moment it is made**. **Only Ken closes a
row.** A status is evidence or the words NOT VERIFIED. A blocked row names the
request file. Nothing is silently rescoped.

### `D:\dev\rag\ui-conventions\` — and the gate behind it

Five gesture classes, each a numbered list carrying where the rule comes from
and the failure mode when it is absent. `tools/gates/check-conventions.sh`
makes every registered surface answer every row in its own source. **It cannot
check behaviour and does not pretend to** — it checks the question was asked,
which is the whole of the problem.

Thirteen surfaces registered. Three of the fourteen gaps it found on its first
run were closed tonight (§1); the rest are O14 in the backlog.

### `D:\Dev\FeatureRequests\pdfce_FeatureRequests\open\`

Read it every session; **empty means nothing is owed**. The engine session runs
in parallel and answers within minutes.

---

## 1. What happened this session (2026-08-21)

Six things shipped, one defect was found and fixed the same hour, and one
is written up as still broken. In the order they matter to Ken:

1. **The navigation keys walk the page, not the fragment you clicked.** Down
   and Up move to the line below and above **including into the next block of
   text**; End and Home reach the ends of the line he can *see*, however many
   show operators drew it — four or five on a CAD title-block row. Salvage:
   the old shell asked `caret_up`, `caret_down` and `line_range_at` and that
   was its entire contribution; the reassembly was always `pdfce-core`'s.
   Driven on `SW41177.pdf`: `text-caret-step dir=Down from_run=232 to_run=240`,
   `text-caret-line end=true from_run=232 to_run=236`.
2. **Every dialog is its own OS window**, not just Print. All thirteen:
   title bar, taskbar entry, second monitor. Driven on eight of them through
   the headless invoke seam, so it needs no pointer.
3. **There is a selection inside a text draft.** Shift+arrows, Shift+Home/End,
   Ctrl+A; typing replaces it, Backspace and Delete remove it, and any move
   without Shift drops it. Closes the conventions sweep's item 11.
4. **A dialog asks for the keyboard when it opens**, which it stopped doing
   the moment it became an OS window.
5. **The harness learned the program has more than one window** — six checks
   were failing and six skipping, every one clicking hundreds of pixels from
   the control it named.
6. **A defect that had shipped: every dialog drew on a BLACK background.**
   Dark text on near-black. Nothing caught it — the window opened, every
   control was where it said it was, the driven check for *"it opens in its
   own OS window"* passed on all eight. **A screenshot showed a black
   rectangle.**

### ★★ The three findings worth carrying forward

- **A guard that only stops repetition does not stop creep.** `Host::fit`
  grows a dialog to fit its body; its first version padded the measurement
  before comparing, so every frame asked for eight more pixels than the last
  and the once-per-size guard never fired *because every size was a new one*.
  About opened at 560x480 and was **1624x746** a few frames later.
- **A `Key` event's own modifiers can be EMPTY while the frame's are right.**
  Measured: `ev=Modifiers::NONE frame=Modifiers { shift: true }`, three
  presses, Shift held throughout. The exact inverse of the chord-matcher
  finding filed three days earlier, and **both are true** — choose by what the
  modifier MEANS. A chord is a command and must use the event's alone; a held
  qualifier must use `event || frame`, because over-reading costs one step and
  under-reading destroys accumulated state.
- **A measurement of the wrong surface is indistinguishable from a
  measurement of a broken one.** Twice in one afternoon: a contrast check
  capturing the wrong WINDOW (reporting 1.51:1 about two headings that render
  at 15.07:1), and `ui_rect_visible` publishing the wrong PART of the right
  one (a heading two points inside a scroll area's bottom edge, measured off
  the anti-aliased top rows of clipped glyphs).

### ⚠ What LOOKED broken, and was not — the sharpest lesson of the session

`text_annot_takes_the_keyboard_unclicked` failed all evening with *"drag out
a note box, type without clicking the field, and the words go nowhere"*. It was
written up for Ken as a live defect. **It was not one.**

The check clicked its Accept button through the APPLICATION window's
coordinates while the dialog had its own. The typing worked the whole time —
the dialog took the characters, the field held focus, the draft was right — and
the click that should have committed it landed on a page. One call site,
converted to `frame_of` like the other six, and it passes.

★★★ **Chasing the wrong culprit, the program was changed four times to hold the
keyboard harder, and every change appeared to help.** The dialog visibly held
the foreground while the new code was asking for it and lost it the instant it
stopped. Real, repeatable, and beside the point. `FOCUS_FRAMES` went
1 → 8 → 40 → 120 on exactly that evidence and is back at 8.

> **A measurement that moves when you turn a knob is not proof the knob is the
> subject.**

Two of the four changes were kept because they are right on their own terms:
**G3 is closed** (a dialog is now OWNED by the window it belongs to, so it can
no longer fall behind it — the unsafe quarantined in a `native-window` crate so
two `#![forbid(unsafe_code)]` claims survive), and a dialog's **position is
asserted once**, on the pass it opens, rather than re-asserted every frame from
a value read back out of the window.

### Published — and on the ENGINE PIN THAT WAS VERIFIED, not the newest one

Packaged to OneDrive at Ken's request: *"fix the regression and release the
latest version."*

★★ **The engine pin was bumped and then deliberately put back.** `cargo update`
moved core/render/print from `10de389` to `05ba72a`, eleven commits — and four
of them are `Pass 97`, the compositing formula, non-isolated group backdrops,
knockout groups and soft masks applied once per group instead of once per
object. **That is the class of change that alters how every page RASTERIZES**,
and the driven suite could not be re-run against it because Ken had come back
to the keyboard: `SetForegroundWindow` is refused to a background process, so
55 of 65 checks correctly declined to click rather than clicking into whatever
he was using.

A green unit suite is not a substitute for a rendered page. So the release ships
the pin the driven suite was green on, and the bump is the next session's first
job — with `Pass 120.2/120.4` in it, which is the engine half of the clipboard
work already on the list.

## 1b. The driven suite, run live

He said *"I'm not using the PC so you can thoroughly test everything live"*, so
it ran against `SW41177.pdf` with `--doc-point 0,300,500`.

**First run: 49 passed, 3 failed, 7 skipped. All three failures were the
harness**, and each in a different way — which is worth reading, because two of
them were *confident, specific, wrong accusations about working code*:

| check | what it said | what was true |
|---|---|---|
| `insert_image_places_a_picture` | *"THE PAGE DID NOT CHANGE"* | the picture was on the page and the sharp oracle had already counted it. The pixel floor was **one in five hundred of the page** — a statement about the FIXTURE. On a 1584 × 1224 pt sheet at 0.3× a 64 × 16 pt picture is 19 × 5 screen pixels, so a *correct* insert changes 90 of 169,416 |
| `a_shift_drag_between_documents_moves_the_pages` | *"the sheets are now in BOTH documents"* | `--second-pdf` was a **one-page** file. The engine refused to remove its only page, by name and correctly, and the shell had already worded that |
| `read_mode_hides_the_chrome` | full screen did not restore | **A REAL DEFECT, and I called it flaky first.** See below |

Both real checks are repaired: the pixel floor is now derived from the
**operand** (the fixture's size in points through the page rect against the
`/MediaBox`), and the page-drag check asks for the refusal line before accusing
and SKIPS with the engine's sentence quoted.

★★ **The rule, third instance in two days:** *a check asserting on an absence
must first ask whether a different signal explains it*, and *a fact about the
fixture is not a fact about the build* — which must be SKIP, never FAIL.

### ★★★ And the third one was NOT flaky, which is the lesson of the evening

It failed on **two of three runs** with *"the display has been left filled;
close the window to recover it"* — and I wrote it off after the first, because
it passed on re-run and `osk.exe` was up.

`toggle_fullscreen` read `ViewportInfo::fullscreen` to decide what to ask for
next. A `ViewportCommand` is **queued and answered by the backend**, so a second
press before it catches up reads the pre-first-press state and asks for full
screen a second time. It turns on and will not turn off. The run it passed on
was the one with more frames between the presses.

★ **The doc comment directly beneath the bug already stated the cause** — as a
*labelling* concern, explaining why a trace should say `asked=` rather than
`on=`. The fact was known and its consequence was never drawn.

> **An intermittent is a defect with a timing dependency.** Three runs is not a
> sample; it is three observations of a race. The last time something here
> looked like harness flakiness, the conclusion drawn was *"this machine cannot
> type"* — which cost the project its whole keyboard surface for months.

Fixed, four unit tests, and verified PASS three times running. Filed in
`D:/dev/rag/egui/`.

**Use `--second-pdf D:/Dev/temp/pdfce/big.pdf`** (5 pages). A one-page second
document makes the move-between-documents check unrunnable.

---

## 2. What to do next

His standing instruction is *"continue looping through other tasks"*. In the
order that returns the most:

1. **The note-box focus defect above.** It is the only known
   operator-visible regression and it is in a gesture he reaches by drawing.
2. **Drag-select and double-click-a-word inside a text draft.** The keyboard
   half shipped; the pointer half needs the laid-out galley published where
   the click ladder can reach it, because the draft is drawn in an editor box
   in screen space.
3. **The three gesture-only dialogs nobody has driven** — Insert pages, Set
   scale, and the unsaved-changes question. They are converted to OS windows
   and nothing has clicked them. `frame_of` and the driver's focus tracking
   are in place, so each is a check rather than an investigation.
4. **The rest of O14**: unfilled-shape hit testing (only ce dimensions carry
   a real shape), grapheme clusters in the caret, right-click to add or remove
   a perimeter point (both engine verbs exist), the zero-travel guard on three
   of four drag paths.
5. **The transform preflight**, a named gap in `canvas::resizing`: an object
   whose own CTM is singular cannot be transformed and the engine says *do not
   offer a handle*. `transform_preview` is the predicate and it decomposes the
   page, so it needs a cache keyed on `(page, epoch, selection)` shaped like
   `app::cache::FormRunCache`.
6. **The clipboard's two remaining halves** (`OPERATOR_REQUESTS.md` O2): a
   private Windows clipboard format for pasting **across two pdfce windows**,
   and `Pass 120.2` (selection to a standalone one-page PDF) for pasting
   **into another program**.
7. **Turning existing page text into multiple lines** — O15's remainder. That
   is a reflow, which the engine has and which currently demands the document
   be saved and reopened first.

★ **Run the full suite with `--doc-point 0,300,500`.** It is the historical
point and the one most checks are calibrated for; `0,1211,1021` aims at a BOM
row and is right for the text checks and wrong for `rotate_handle_turns_a_selection`.
Last full run at 300,500: **51 passed, 2 failed, 12 skipped**, and both
failures were the note dialog — one of which is now fixed.

## 3. Blocked on the engine

`D:\Dev\FeatureRequests\pdfce_FeatureRequests\open\` — 4 open requests.

| what | state |
|---|---|
| **`transform_objects`** — move / resize / rotate an image or text | Accepted. `Pass 112.0` shipped the `Matrix` foundation; **`113.0` is the verb and is NOT built.** The whole shell side is built and waiting: selection model, eight grips, ghost preview, and `canvas::resizing` computing scale factors it can only commit for paths. |
| **Object clipboard** | Filed whole on his instruction. ★ `EditSession::import_object` already does cross-document graph copying with id remapping — the ask is *expose it at object granularity*, not *build a copy engine*. |
| Insert-pages orphaned widgets · markup opacity in one verb | Older, unchanged. |

**Closed and archived tonight:** all three asks of the form-XObject request, with
a note back carrying one finding — **do not use `EditTarget::Auto` for a pinned
request.** A pin is a byte span into one buffer and the provenance names which;
`Auto` searches the page stream first, and on his sheets that stream holds 3,007
single-character show operators. A stray match is an edit on the wrong glyph
with no error anywhere.

---

## 4. Environment gotchas

- **`ui-verify` takes the real cursor and keyboard.** Ask, or verify headlessly.
- **★ `--second-pdf` must have MORE THAN ONE PAGE.** A one-page source cannot be
  moved out of — the engine refuses to leave a document with no pages — so the
  cross-document move check cannot run. `D:/Dev/temp/pdfce/big.pdf` (5 pages).
- **★ Python heredocs eat the `\` continuation in a Rust string literal**, and
  the result COMPILES: what lands on disk is one long line with the indentation
  baked into the string. **Five times tonight**, and `check-string-gaps` missed
  the first four because its character class enumerated `[A-Za-z,.:;)]` and had
  a hole the width of `}` — which in Rust is one of the likeliest characters to
  end a clause. The class is `[^[:space:]("]` now, and widening it found **four
  already-shipped defects**. Use the Edit tool for anything with a continuation,
  or `r"""…"""`, or `chr(92)`.
- **★ NEW: `PDFCE_DIAG_INVOKE=<command.id>` presses one ribbon command once**,
  in an invisible window, through the real dispatcher. That is how the Print
  dialog was verified tonight while he was working:
  ```
  PDFCE_DIAG=1 PDFCE_DIAG_VIEWPORT=-4000,-4000,1200,850 \
  PDFCE_DIAG_INVOKE=file.print  target/release/pdfce-gui.exe file.pdf
  ```
  An offscreen window is genuinely laid out and **OS input cannot reach it**, so
  before this a headless run could read the trace and press nothing.
- **`osk.exe` covers the ribbon and swallows synthetic clicks**, UIPI-protected.
  A driven failure on this machine is a harness question before it is an
  application one.
- **`python tools/package-portable.py --verify` after every keeper build**, and
  read the two slot dates it now prints. ★ It used to pick the target by
  directory **mtime**, which is bumped both by a failed mirror *and by the
  operator running the build* (`userdata/` is written beside the exe) — so the
  fallback aged one build at a time while every run reported success. It reads
  the `Built:` stamp now. **`--slot <name>` forces the target, for retraction**:
  the rotation protects the newest build, which is exactly wrong when the newest
  build is the one being withdrawn.
- **`cargo update -p pdfce-core -p pdfce-render -p pdfce-print` before every
  build**, and again before packaging. The engine shipped four revisions tonight.
- `.tmpwork/edit.py` is the CRLF-safe edit helper. **Python heredocs eat
  backslash-continuations in Rust string literals** — it happened twice tonight
  and `check-string-gaps` caught both. Use the Edit tool for those.
- **Never `git checkout --` a dirty file.**

---

## 5. Standing rules this project has paid for

- **A trace can say the verb ran. It cannot say the screen changed.** Every
  layout, repaint or clipping defect has exactly one oracle: a rendered
  screenshot. Put a capture on the failure branch of anything that draws.
- **A check asserting on an ABSENT line must first ask what else happened.**
- **★ A fixture that cannot exhibit the hazard proves nothing.** New tonight,
  and the sharpest of these: when an upstream fix stops your falsifier firing,
  the assertions beside it stop measuring and go on passing. Invert the control
  or grow the fixture; never just delete it.
- **Two derivations of one position agree at first and separate under use.**
  Four instances now — the newest is a child viewport, whose `ui-rect`s are
  relative to **its** origin and look exactly like the parent's. `egui::Pos2` is
  screen, canvas, page AND per-viewport space. The durable fix is publishing
  the frame beside the coordinate, or typed coordinates (`euclid`, already in
  the tree), not care.
- **A blocker is a measurement, and the question you measured is part of it.**
- **A predicate with two claimants must exist exactly once.**
- **Registering a command is the only way the GUI may learn a capability
  exists** (R8), and **`egui-shell` never learns what a PDF is** (R7).

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
