# CONTINUE — handoff, 2026-08-21 morning

**Type `continue` and start at §2.** Everything above it is state; §2 is the
work queue, in order.

**Clean tree. 16/16 gates. 1,595 tests.** Driven suite, last full green run:
**53 passed, 0 failed, 12 skipped**, on his own drawing at
`--doc-point 0,300,500` — but read §1c, because two things have landed since
that are **published and unverified**.

**Published: `OneDrive\pdfceGUI2`, built 2026-08-21 09:50**, engine `cbb1ede`,
shell `622f74b`. `pdfceGUI1` holds 2026-08-19 17:44 as the fallback.

---

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

1. **Run the full driven suite**, machine free, and fix whatever the engine
   bump moved:
   ```
   ./target/release/ui-verify.exe --pdf D:/Dev/temp/pdfce/SW41177.pdf \
       --doc-point 0,300,500
   ```
   Not to gate a release — that has already happened — but because he is
   running unverified code and the sooner that stops being true the better.
   **`0,300,500` is the calibrated point**; `0,1211,1021` aims at a BOM row and
   is right for the text checks and wrong for `rotate_handle_turns_a_selection`.
2. **The three gesture-only dialogs nobody has driven** — Insert pages, Set
   scale, and the unsaved-changes question. They are OS windows now and nothing
   has clicked them. `frame_of` and the driver's focus tracking are in place, so
   each is a check rather than an investigation.
3. **`Pass 120.2/120.4` is in the tree now** — selection to a standalone
   one-page PDF, and the clipboard's cross-application half. That closes
   `OPERATOR_REQUESTS.md` O2's remaining rows; the shell side is a paste target
   and a private Windows clipboard format.
4. **The rest of O14**: unfilled-shape hit testing (only ce dimensions carry a
   real shape), grapheme clusters in the caret, right-click to add or remove a
   perimeter point (both engine verbs exist), the zero-travel guard on three of
   four drag paths.
5. **The transform preflight**, a named gap in `canvas::resizing`: an object
   whose own CTM is singular cannot be transformed and the engine says *do not
   offer a handle*. `transform_preview` is the predicate and it decomposes the
   page, so it needs a cache keyed on `(page, epoch, selection)` shaped like
   `app::cache::FormRunCache`.
6. **Turning existing page text into multiple lines** — O15's remainder. That
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
