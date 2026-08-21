# CONTINUE — handoff, 2026-08-20 late evening

**Clean tree. 16/16 gates. 1,565 tests. 61 driven checks (not run tonight).**
**Newest build: `OneDrive\pdfceGUI2`, 2026-08-20 21:06, engine `65f6a36`.**
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
3. **Run the driven checks.** Three features shipped tonight with checks
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
