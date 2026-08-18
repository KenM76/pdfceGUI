# RESUME — read this, then say "continue"

**Written 2026-08-18, last revised at `2955ab3`.** For a session starting cold
on `D:\Dev\pdfceGUI`.

This file is the **entry point**. `HANDOFF.md` is the long-form institutional
record and is still authoritative for the standing rules, the phase order and
the accumulated findings — read it after this one, or when this one points you
at a section of it.

---

## ★★★ State, as measured at `2955ab3`

**This table is a reading, not a status.** Every row is what a command
printed at that commit; the tree has moved since, and the numbers move with
it. It is here so you know roughly where you are, not so you can quote it.

| | |
|---|---|
| **Measured at** | `2955ab3`, clean tree |
| **Engine** | `D:\Dev\pdfce` local `main`, locked at `bd861a0`, taken as `git = "file:///D:/Dev/pdfce", branch = "main"` |
| **Tests** | 1,851 passing, 0 failing |
| **Gates** | 14 of 14, 0 skipped |
| **`ui-verify`** | ★ **NOT RUN since `6dc6749`.** Two new checks are written and unrun — see the queue below |
| **Latest build** | see `D:\builds\` — the newest `pdfcegui-*` directory; two most recent mirrored to `OneDrive\pdfceGUI1` / `pdfceGUI2` |
| **Requests owed by pdfce** | one `note_*`, filed by us and not blocking — `open/` otherwise empty |

## ★★ The harness queue — needs the operator's go-ahead

`ui-verify` drives the real cursor and keyboard, so it may not run while the
operator is at the machine. Two checks were written this session and **neither
has ever executed**. Under R1 that means both features are **implemented and
unverified**, and they must be reported in exactly those words until the queue
is run.

```bash
cargo run --release -q -p ui-verify -- --exe target/release/pdfce-gui.exe \
  --pdf D:/Dev/temp/pdfce/SW41177.pdf --doc-point 0,300,500
```

| check | asserts |
|---|---|
| `print_paper_changes_the_plan` | choosing a paper size re-plans the job — `paper=` and `sheet=` on one trace line must BOTH move. Also that the Properties… button is drawn |
| `new_document_sizes_the_page` | New from template ▸ A3 ▸ Landscape produces a 420 × 297 mm page, read back **after the re-parse** |

Both are written not to press the two buttons a harness must never press: the
print commit (it consumes paper) and Properties… (a vendor driver's modal —
unpublishable, un-dismissable, and one left standing hangs every check after
it).

### ★ Two things from 2026-08-18 that the harness CANNOT check, and why

Not queue items. Recorded here so nobody adds a check for them and then
wonders why it cannot be made to work.

| change | why no check |
|---|---|
| **the crosshair cursor** | **Windows composites the pointer separately from window contents.** `BitBlt` and `PrintWindow` — the two ways `ui-verify` captures a window — return an image with **no cursor in it**, at any price. There is no pixel oracle. The substitute is a trace line, `cursor-crosshair on px=… / off`, emitted on change; it proves the wiring, not the legibility. Legibility was checked by rendering the glyph onto black, grey and white and looking at it — `evidence/crosshair-32.png`, `-64.png` |
| **the executable's icon** | It is read by the **shell**, from the PE image, without the program running. Nothing a harness that drives the running program can observe. Verified instead by extracting it back out of the built exe with Windows' own `ExtractAssociatedIcon` — `evidence/embedded-icon-check.png` — which is stronger evidence than a screenshot would have been, because it is the same API Explorer uses |

Both are also **loudly visible to the operator on first use**, which is the
other half of the calculus: R1 exists for defects that a green suite hides, and
a white crosshair or a missing icon is not one of those. The print and page-size
work above is.

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

### ★ First: run the harness queue above

Two features shipped this session and neither has been driven. That is the one
outstanding obligation, it needs nothing but the operator's desktop, and under
R1 nothing else should be called done until it is discharged.

### 1. Revision clouds — the last standard revisioning tool

Text box, sticky note and stamp landed 2026-08-18. Clouds are the fourth and
are **still not started upstream**, accepted and scheduled since 2026-08-14.
Verified rather than assumed: `grep Cloud` and `grep BorderEffect` over
`pdfce-core` return nothing, and `MarkupSpec` has `Square`, `Circle`, `Line`,
`Ink`, `Polygon`, `PolyLine`, `TextMarkup` and no cloud.

**Nothing on this side is missing.** If the operator wants the set truly
finished, this is a re-raise in the channel, not work here.

### 2. ★★ The selection model — the piece that unblocks the Format tab

This is the largest thing left and it is worth reading the reasoning before
picking anything else up.

`EditSession::set_markup_style` shipped 2026-08-18 — colour, interior, width,
opacity and arrowheads on an existing annotation, keeping its object id. That
is the Format contextual tab's first real slice, and **this shell cannot reach
it**, because `Selection` is a paint-order index and cannot name an
annotation.

The engine said so explicitly when declining to ship the enumerator we asked
for:

> *"What you actually want is `markup_rects(page_index)` … **Not shipped,
> deliberately**: your own blocker is on your side, and a query with no caller
> is exactly the `[x] core / [ ] gui` drift R151 exists for. **Ask for it with
> your selection change and it lands in the same session.**"*

So the order is fixed and it is not the obvious one: **change `Selection`
first, file `markup_rects` at the same moment, then build the Format tab.**
Filing the request before the selection change would be asking for the thing
they just declined, for the reason they declined it.

They also corrected one of our claims, and it saves work: `bounds_of` applies
the pen half-width at *authoring* time, so the stored `/Rect` already contains
it and **a shell hit-testing `/Rect` is correct today**. We do not need pdfce
to own the hit test.

**★★ And read this before writing a single line of that tab.** A ce dimension
is a `/Line` with `/IT /LineDimension`. It passes every "markup pdfce can
author" test, and restyling one regenerates it as a **bare line — label and
witness lines gone** — from an operator who asked only to recolour it. The
engine refuses by name (`EditError::AnnotationIsCeDimension`) and points at
`set_dimension_style`. **A Format tab must route ce dimensions there.**
`panels::comments::model` already computes the set of ce-dimension object ids
per document, so the routing predicate exists.

### 3. Resize an EXISTING page

`set_media_boxes(indices, rect)` shipped with `set_media_box` and only the
second is used. A page-resize surface belongs in Document ▸ Properties and is
a genuinely different capability from the New chooser: does content move, does
`/CropBox` follow, is shrinking below the content a refusal. None of those
arise for a blank page and all of them arise here, so it is a design question
before it is a coding one.

> **Read `archive/2026-08-18-mediabox-and-markup-reply.md` first.**
> `/MediaBox` is inheritable (§7.7.3.4), so the write is three-way, and *"a
> target equal to the inherited value REMOVES the page's own entry"* is
> load-bearing and invisible to a one-page fixture — **writing to the ancestor
> that supplies the value resizes every sibling.**

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

### From 2026-08-18 (this one): two features, and three drifted claims

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

## The founding rule, and where this session falls short of it

> **Verify by driving the binary, not by a passing test.**

**Both features shipped on 2026-08-18 are implemented and NOT DRIVEN.** The
checks exist; the harness has not run; the operator's desktop was in use. That
is stated plainly rather than softened, because this project was founded on a
commit that said *"analysis-confirmed, NOT empirically verified"* and was
treated as done anyway.

What driving buys, from the session before, in four trace lines:

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
