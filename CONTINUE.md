# CONTINUE — handoff, 2026-08-20 late evening

**Clean tree. 16/16 gates. 1,567 tests. The driven suite RAN, live, on his own
drawing — see §1b.**
**Newest build: `OneDrive\pdfceGUI2`, engine at `e5be7d5d` or later.**
`pdfceGUI1` holds 2026-08-19 17:44 — the fallback is a day old, and that is
now a *known* fact rather than an accident; see §4.

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

## 1. What happened tonight

Four things shipped and one build had to be retracted. In the order they
matter to Ken:

1. **Form-XObject text editing** — the 99 % case on a CAD sheet. His words:
   *"I need that editing capability as it is 99% of the text I will want to
   edit."* The engine shipped it (`Pass 119.0`); this shell's cost was
   **deleting one match arm**, because the guard asked `TextRun::editability()`
   instead of modelling the answer and a `#[deprecated]` attribute pointed
   straight at the line. That is the whole return on a decision made two days
   earlier, demonstrated end to end.
2. **Shift constrains every drag** — aspect on a resize, axis on a move, a
   dimension, a perimeter corner and a Bézier handle. Announced on the status
   row, because the ghost cannot say *"because you are holding Shift"*.
3. **A perimeter corner snaps** — same query, same tolerance, same switch the
   tool that placed it uses. Alt suspends it.
4. **The Print dialog is a real OS window**, with Enter as its default. One
   host, so the other thirteen dialogs are one line each.
5. **Move, resize and rotate ANY object** — `transform_objects` (`Pass 113.0`)
   closes the request he made three times. Three refusals were **deleted** with
   it, each true when written.
6. **The rotate handle** — the ninth grip, a circle on a stem above the box,
   Shift stepping by 15°. Driven: `rotate-commit deg=-90.22`.
7. **Cut, copy and paste of page content** (`Pass 120.0`) — his **oldest** open
   request, from the first week.

### ★★★ And the finding underneath the last one, which explains two reports

`Ctrl+C`, `Ctrl+X` and `Ctrl+V` **had never once reached the keymap.** He said
so twice. They were bound on 2026-08-20, which was necessary and not sufficient:
`egui-winit` pushes `Event::Copy`/`Cut`/`Paste` and **returns before the
`Event::Key` push**, so the chord matcher saw nothing while the manifest, the
unit tests and the menus all agreed the binding was there.

**A keymap lookup is not a keystroke.** `Ctrl+V` was worse — `Event::Paste` is
raised only when the OS clipboard holds non-empty text, so with it empty the
keystroke vanished and whether paste worked depended on what he had last copied
*in another application*.

Filed in `D:/dev/rag/egui/`, because every egui project with a data-driven
keymap has this and does not know it. The general rule is in the last line of
that file: **any shortcut a toolkit gives special semantic treatment — clipboard,
IME, Tab, Escape — may never reach a generic handler, and every test written
against the binding table will agree it works.**

### ★ The seam to hold on to, because it paid twice in one evening

Both of tonight's big unblocks cost almost nothing on this side, and for the
same reason: **the shell had asked the engine rather than modelling it.**
`TextRun::editability()` started answering `Editable` and a `#[deprecated]`
attribute pointed at the one line to delete; `transform_objects` took a slice
and three refusals became false at once. A hand-rolled guard would have outlived
both. That is decision 058 earning its keep, twice, inside forty-eight hours.

### ★★ And the retraction, which is the thing to internalise

The 20:09 build was published and withdrawn within the hour. It linked an
engine revision whose reflow could shift **1,676 labels** on one four-character
edit — 34,059 changed pixels across a whole sheet — because reflow walked
forward to a `Td` boundary and **a CAD stream never emits one**.

Three separate lessons, all live:

- **`cargo update` immediately before packaging, and read the output.** The
  packager runs its own `cargo build --release`, which re-resolves the git
  dependency. It warned *"the engine MOVED and --verify was not passed"* and
  the warning was right.
- **The engine's own advice, taken:** *"if you show one number from an edit
  report beyond the disclosures, make it `followers_repositioned`."* It is on
  the diagnostic channel now, and a driven check fails above 64.
- **A fix can silence a falsifier.** The engine's repair left `proof.rs`'s
  fixture unable to exhibit the hazard at all, so both falsifying assertions
  went quiet — and a quiet falsifier is a test that has stopped measuring. The
  fixture grew a fourth block (two runs on one baseline) and the two old
  controls were **inverted** into guards on the engine's fix.

---

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

## 2. What to do next, in his stated order

He approved a three-item list; items 1–3 are done. What is left of it:

1. **The other thirteen dialogs**, now that the host exists. Each is: hold a
   `Host`, take it out for the draw, call `Host::buttons` in the footer. Watch
   for the borrow — `Host::show` needs `&mut` on the host while the body needs
   `&mut` on the dialog, so the field is an `Option` and is taken for the
   duration, exactly as `canvas::interact` does with the selection.
2. **The rest of O14**: unfilled-shape hit testing (only ce dimensions carry a
   real shape), grapheme clusters in the caret, selection inside a draft,
   right-click to add/remove a perimeter point (both engine verbs exist), the
   zero-travel guard on three of four drag paths.
3. **The transform preflight**, which is a named gap in `canvas::resizing`:
   an object whose own CTM is singular cannot be transformed at all and the
   engine says *do not offer a handle*. `transform_preview` is the predicate and
   it **decomposes the page** (~4 s debug on the benchmark), so it needs a cache
   keyed on `(page, epoch, selection)` shaped like `app::cache::FormRunCache`.
4. **The clipboard's two remaining halves**, both named in `OPERATOR_REQUESTS.md`
   O2: a private Windows clipboard format so a paste works **across two pdfce
   windows**, and the engine's `Pass 120.2` (selection → standalone one-page
   PDF) so a paste works **into another program**. Neither blocks anything he
   has reported.
5. **Re-run the driven checks** Three features shipped tonight with checks
   written and **not run** — `shift_constrains_a_resize`, the snap assertion in
   `measure_perimeter_traces_and_closes`, and the two new assertions in
   `text_edit_on_a_real_drawing`. The harness takes the real cursor; ask.

---

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
