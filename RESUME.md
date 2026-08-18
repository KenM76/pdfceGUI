# RESUME — read this, then say "continue"

**Written 2026-08-18 at `6dc6749`.** For a session starting cold on
`D:\Dev\pdfceGUI`.

This file is the **entry point**. `HANDOFF.md` is the long-form institutional
record and is still authoritative for the standing rules, the phase order and
the accumulated findings — read it after this one, or when this one points you
at a section of it.

---

## ★★★ State, as measured at `6dc6749`

**This table is a reading, not a status.** Every row is what a command
printed at that commit; the tree has moved since, and the numbers move with
it. It is here so you know roughly where you are, not so you can quote it.

| | |
|---|---|
| **Measured at** | `6dc6749`, clean tree. Doc-only commits follow it |
| **Engine** | `D:\Dev\pdfce` local `main`, locked at `ac15158`, taken as `git = "file:///D:/Dev/pdfce", branch = "main"` |
| **Tests** | 1,839 passing, 0 failing |
| **Gates** | 14 of 14, 0 skipped |
| **`ui-verify`** | 25 passed, 0 failed, 2 skipped |
| **Latest build** | `D:\builds\pdfcegui-20260818-0536-ac15158-82605b5\`, mirrored to `OneDrive\pdfceGUI2` |
| **Requests owed by pdfce** | **NONE — `open/` is empty.** Seven filed, all seven answered inside a day |

**Re-measure before you rely on any of it.** Prose drifting from a number is a
defect this project has spent six corrections on — the gate runner's own
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

### 1. The print UI — asked for THREE times, now fully unblocked

The operator, 2026-08-18:

> *"acrobat reader has those options, and pretty much every program I have ever
> seen lets you press a properties button beside the selected printer in the
> drop-down menu to open the printer options."*

`pdfce-print` shipped the entire filing on 2026-08-18 (`c75a17c`, `c5d3ae8`,
`cdac4e7`). Everything is now available:

| now exists | for |
|---|---|
| `printer_forms(printer) -> Vec<PaperForm>` | the paper dropdown |
| `printer_configuration(printer) -> PrinterConfiguration` | the driver's real DEVMODE |
| a properties call taking `start_from: Option<&PrinterConfiguration>` | the **Properties…** button |
| `DeviceSettings::paper: PaperSelection` | `DeviceDefault` / `Form(id)` / `Custom { … }` |
| `spool(…, config: Option<&PrinterConfiguration>)` | carrying it into the job |

The GUI passes `PaperSelection::DeviceDefault` today, in
`dialogs/print/spooler.rs`'s `to_engine_settings` — unchanged behaviour. The
work is a dropdown and a button beside the printer combo, plus threading a
`PrinterConfiguration` through the dialog's state.

**★ One disclosure you will owe the operator, and it is not obvious from the
API.** The engine's reply reports that **two drivers were found silently
ignoring a paper request.** So a chosen paper is a *request*, not a guarantee,
and the dialog must be able to say so. Read
`archive/2026-08-18-print-devmode-reply.md` before designing the surface —
that finding is the reason it exists.

### 2. Revision clouds — the last standard revisioning tool

Text box, sticky note and stamp all landed on 2026-08-18. Clouds are the fourth
and are **still not started upstream**, accepted and scheduled since
2026-08-14. Verified rather than assumed: `grep Cloud` and `grep BorderEffect`
over `pdfce-core` return nothing, and `MarkupSpec` has `Square`, `Circle`,
`Line`, `Ink`, `Polygon`, `PolyLine`, `TextMarkup` and no cloud.

**Nothing on this side is missing.** If the operator wants the set truly
finished, this is a re-raise in the channel, not work here.

### 3. Two more things the engine unblocked that nobody has picked up

**`set_media_box` / `set_media_boxes` + `pdfce_core::paper`** (A0–A6, Letter,
Legal, Tabloid, Executive, ANSI A–E). Closes `NO_SURFACE.md`'s *"New blank page
size — A4, baked-in template, none"*, which matters because the operator's
sheets are A1 and A3.

> **Read `archive/2026-08-18-mediabox-and-markup-reply.md` first.** `/MediaBox`
> is inheritable (§7.7.3.4), so the write is three-way, and *"a target equal to
> the inherited value REMOVES the page's own entry"* is load-bearing and
> invisible to a one-page fixture. `paper` lives in **core**, not in a shell, so
> the size chooser and the CLI cannot disagree about what A1 is.

**`set_markup_style`** — colour, interior, width, opacity and arrowheads on an
existing annotation, keeping its object id. The Format tab's first real slice.

> **★★ The refusal to read before touching it.** A ce dimension is a `/Line`
> with `/IT /LineDimension`. It passes every "markup pdfce can author" test, and
> restyling one would regenerate it as a **bare line — label and witness lines
> gone** — from an operator who asked only to recolour it. The engine refuses by
> name (`EditError::AnnotationIsCeDimension`) and points at
> `set_dimension_style`. **A Format tab must route ce dimensions there.**

That reply also **corrects one of our claims**: the request argued pdfce should
own the hit test. It should not — `bounds_of` applies the pen half-width at
*authoring* time, so `/Rect` already contains it and a shell hit-testing
`/Rect` is correct today. What is actually wanted is `markup_rects(page_index)`,
deliberately **not shipped** because our own blocker is on our side:
`Selection` is a paint-order index and cannot name an annotation. **Ask for it
together with the selection change and it lands in the same session.**

---

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
2. **Mirror the last two builds to OneDrive**, `pdfceGUI1` / `pdfceGUI2`, newest
   replacing the older slot. Automated. `userdata/` is preserved across a
   rotation, because the operator can run the exe straight out of a slot.
3. **Put engine work through the channel**, and the other session picks it up in
   parallel.

---

## ★★ What the last session found — the part worth carrying

Four operator reports. Every one turned up something worse than the thing
reported.

**Every significant find was a PREDICATE that was too coarse, not a broken
mechanism** — and in three of four cases the *harness* was what was wrong,
while looking entirely confident:

| reported as | actually |
|---|---|
| *"synthetic keyboard input does not reach the window"* | only **chords** failed. `keybd_event` posts asynchronously and egui drains once per frame, so modifier-down and key-down in the same microsecond deliver an **unmodified** key. Three 12 ms sleeps. `find_opens_and_finds` passed for the first time in the project's life |
| *"18 controls laid out outside the window"* | the `ui-rect` trace is a **change log** and could not report that a control stopped being drawn. The ribbon overflow had correctly swallowed them. Fixed at source with `ui-rect-gone` |
| *"selection is not taking the hit test's result"* | six doc-points across a dense sheet all reported `hit 0 objects`. **A hit test that misses everywhere is a gate, not a hit test** — the check had never left Read mode |
| *"three headings illegible"* | three headings **not on screen**. A `ScrollArea` lays out below-the-fold children before clipping them |

**Two mistakes of mine, kept in the docs because they looked reasonable:**

- I invented seven stamp label strings. `TextAnnotSpec::Stamp` takes a
  `StampName` — ISO Table 181's standard set — so every stamp would have carried
  `/Name /Draft` whatever it read, and any reader but pdfce would show *Draft*
  beneath a stamp saying APPROVED. **An annotation disagreeing with its own
  appearance**, invisible from inside the GUI.
- I left the UI-scale check's injected preference at 1.8 "on purpose", reasoning
  that tidying up would hide inherited state. Next full run: **20/0/4 →
  3/1/21.** The distinction I had missed is **who owns the state** — application
  side-effects stay, harness-injected inputs get restored.

**`--verify` had never worked, for a reason nobody had diagnosed.**
`HANDOFF.md` §7 said for weeks that *"a spawned bash does not inherit
`~/.cargo/bin`"*. Wrong, and no PATH fix could have helped:
`subprocess.run(["bash", …])` resolves `System32\bash.exe` — **the WSL
launcher** — before Git Bash. It also explains a CRLF symptom filed separately,
since WSL bash rejects CRLF scripts. One root cause, two unrecognisable
symptoms. **A workaround written against a wrong diagnosis outlives the problem
and hides it.**

**R2 came due and three files were split** — `tool.rs` (what a tool *is* vs how
one is *chosen*), `actions/apply.rs` (the redaction arms, **with their
comments**), `gesture/mod.rs` (the machine vs the vocabulary it speaks in). The
seam test: *do these two change for different reasons?*

---

## The founding rule, restated because it earned its keep again

> **Verify by driving the binary, not by a passing test.**

Everything shipped in the last session was driven. The one that shows why:

```
Markup > Text box armed the text-annotation tool
the page carries 0 annotation(s) before the drag
the release authored nothing — still 0 — and opened the dialog instead
Accept authored: the page went from 0 to 1
```

That middle line is the whole feature. A build where the release authored
directly passes **every** unit test in `canvas::textannot` — the spec builder
is pure and correct either way — and puts an empty box on the operator's
drawing every time they let go of the mouse. Nothing in the workspace can see
it; only a running window can.

---

## ★ One thing to know about the copy, added after this file's first draft

`tools/gates/check-string-gaps.sh` is new, and so is the defect class it
hunts. **Rust's line continuation eats the newline and the next line's
indentation; lose the trailing backslash and the indentation ships.** The
literal still compiles and still passes every test that does not compare it to
a hand-written expectation.

`pdfce-core` reported six of these in its own error messages. The same grep
here found **36 across 22 files, eight of them in `crates/pdfce-gui/src/text/`
— copy the operator reads on screen**, including every sentence of the
Set-scale dialog. All repaired; the gate now runs in `run-all.sh`.

**Why it matters to you specifically:** it is invisible in a diff. Reviewing
the source you see a wrapped sentence and the spaces read as indentation, which
is what your eye is trained to skip. If you write operator copy this session,
you will not catch it by looking — run the gate.

**Stated limitation:** the gate asserts the *source*. No driven check reads the
Set-scale copy, so the repair is verified at the literal and not yet in a
rendered window. If you touch that dialog, that is a `ui-verify` assertion
worth adding while you are in there.

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
