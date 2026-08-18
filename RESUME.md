# RESUME — read this, then say "continue"

**Written 2026-08-18, last revised at `3b40492`.** For a session starting cold
on `D:\Dev\pdfceGUI`.

This file is the **entry point**. `HANDOFF.md` is the long-form institutional
record and is still authoritative for the standing rules, the phase order and
the accumulated findings — read it after this one, or when this one points you
at a section of it.

---

## ★★★ State, as measured at `3b40492`

**This table is a reading, not a status.** Every row is what a command
printed at that commit; the tree has moved since, and the numbers move with
it. It is here so you know roughly where you are, not so you can quote it.

| | |
|---|---|
| **Measured at** | `3b40492`, clean tree |
| **Engine** | `D:\Dev\pdfce` local `main`, locked at `d51e0d9`, taken as `git = "file:///D:/Dev/pdfce", branch = "main"` |
| **Tests** | 1,856 passing, 0 failing |
| **Gates** | 14 of 14, 0 skipped |
| **`ui-verify`** | **25 passed · 1 failed · 3 skipped**, run 2026-08-18 in the operator's lunch window. The one failure is a HARNESS gap — see below |
| **Latest build** | `D:\builds\pdfcegui-20260818-1125-4993559-077a6c2\`, mirrored to `OneDrive\pdfceGUI2`. No `-dirty` suffix — see the packager note below |
| **Requests owed by pdfce** | one `note_*`, filed by us and not blocking — `open/` otherwise empty |

## ★★ The harness — last run 2026-08-18, and what it found

`ui-verify` drives the real cursor and keyboard, so it may not run while the
operator is at the machine. It was run in a lunch window on 2026-08-18:
**25 passed, 1 failed, 3 skipped.**

```bash
cargo run --release -q -p ui-verify -- --exe target/release/pdfce-gui.exe \
  --pdf D:/Dev/temp/pdfce/SW41177.pdf --doc-point 0,300,500 > evidence/ui-verify-run.txt 2>&1
```

★ **Redirect to a file.** The first attempt piped through `tail`, which threw
away the failure detail and cost a second run of the operator's window. The
second run was skipped wholesale as STALE because a source file was edited
while it was in flight — **finish the edits, rebuild, then run.**

### The one failure, and it is the harness rather than the application

`print_dialog_reaches_the_spooler` reported that the File tab declares no
`ribbon.item.file.print`. It does not — at **1100 pt**, the harness's window
width, the ribbon has correctly folded the **Print, Document and pdfce groups
into the overflow**, and `ribbon.overflow` is declared in the same frame.

Two things are true and both matter:

1. **The check has a gap.** `settings_headings_legible` meets the same
   condition in the same run and handles it — *"the Settings control is not on
   the ribbon band at this window width … Opening the overflow to reach it."*
   The print check needs the same step. `print_paper_changes_the_plan` skips
   for the same reason and defers to it, correctly.
2. **`file.new_from_template` made the File band one item wider** on
   2026-08-18, which is what tipped it over at that width. On a wide window
   Print is still on the band; it was not verified at the operator's own window
   size.

**Fix the check first**, and only then decide whether the size chooser has
earned a place on the band — that is an IA question for the operator, not a
build session's.

### The three skips are honest and named

| check | why |
|---|---|
| `page_ops_round_trip` | the fixture already carries 36 `/Rotate` entries, so the evidence would be indistinguishable from the document's own furniture. Point `--pdf` at `D:\Dev\pdfce\fixtures\pageops\four-pages.pdf` |
| `ocr_recognises_a_page_and_writes_a_new_file` | needs the `models/ocrs` weights beside the exe, i.e. a **packaged** build |
| `print_paper_changes_the_plan` | blocked by the failure above, and says so |

### Still not written

An **annotation-selection** check — click a stamp, assert `annot-select`, press
Delete, assert one fewer annotation. Every trace line it needs already exists.
And a check that **types for real**: the text-editing check seeds its draft
through `PDFCE_DIAG_TYPE`, which is the one path that bypasses the event loop,
so it would pass on a build where typing is dead — which is what the operator
reported on 2026-08-18.

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

> ### ★★★ Read this before picking anything: the order came from the wrong place
>
> On 2026-08-18 the operator said, of a day and a half of work:
>
> > *"I'm still flabbergasted by how the GUI is still not user friendly. …It
> > feels like nothing is moving forward on these things. …Hours and hours,
> > and I click and can't figure out how to enable some of the basic stuff."*
>
> He was right, and the cause was **scheduling, not difficulty**. This list
> used to be ordered by *what the engine most recently unblocked* — which is
> how print paper sizes and a page-size chooser got built while
> **clicking a stamp did nothing at all**. Both were things he had asked for;
> neither was on the path to what he was actually hitting.
>
> **Order this list by what the operator reaches for, not by what just
> arrived in the channel.** The engine's replies are an input to *how* a thing
> gets built, never to *which* thing.

### ★ First: run the harness queue below

Three features are shipped and undriven. It needs nothing but the desktop.

### 1. The Format tab's first slice — restyle a selected annotation

**Selection landed 2026-08-18** (`00ff4c7`): a stamp, note, shape or ce
dimension can be clicked, is outlined, and can be **deleted**. What is still
missing is everything that made the operator ask *"how do I **edit** a stamp"*:
colour, width, opacity.

`EditSession::set_markup_style` is the verb and it is shipped. The routing is
already done for you — `AnnotKind` on the selected target is `Markup` or
`CeDimension`, and the second **must** go to `set_dimension_style` instead.

> ★★ A ce dimension is a `/Line` with `/IT /LineDimension`. It passes every
> "markup pdfce can author" test, and restyling one through `set_markup_style`
> regenerates it as a **bare line — label and witness lines gone** — from an
> operator who asked only to recolour it. The engine refuses by name; the kind
> on the target is what stops the refusal being reached.

Deliberately **not** in that verb, so the tab must not offer them: note text,
**move and resize**, and `/LE` on PolyLine. Move/resize is the next ask after
this one and needs its own engine request.

### 2. Dimension select-and-drag — a REGRESSION, not an unbuilt feature

The **old** GUI does this: `run_dimension_drag` at
`D:\Dev\pdfce\crates\pdfce-gui\src\main.rs:22782`, with
`doc.selected_dimension`, `doc.dimension_drag`, and `dimension_rects`
hit-tested per page. The new shell never called `dimension_rects` at all.

Selection now covers *clicking* one. **Dragging it is still gone**, and R6 says
nothing regresses. The old code exists and is salvageable; 18 references to
`selected_dimension` in that file are the whole feature.

### 3. Two open operator reports, both from 2026-08-18 and neither diagnosed

**Add text types nothing.** *"adding text does bring up a window and a prompt,
but it doesn't type anything in the box when I type and nothing gets added."*
A `text-edit-typing` trace was added and reports draft / owns_keyboard /
text_events / len — four facts that kill four hypotheses. **★ The existing
driven check cannot see this**: it seeds the draft through `PDFCE_DIAG_TYPE`,
the one path that bypasses the event loop, so it passes on a build where real
typing is dead. A check that types for real is the first thing to write.

**No context-sensitive panel, and no tool indicator.** *"When I click to use a
tool I have no indicator to tell me what to do next or what tool I am even
using."* Verified: the status bar carries page, find, fit, zoom and
disclosures, and **nothing names the armed tool**. Read `MODES_AND_PANELS.md`
before designing — the flexible panel system is specified there and this is not
a thing to improvise.

### 4. "Highlight fillable fields" — the smallest real win on this list

Form filling **works**, in every mode including Read: `canvas::forms` never
consults the mode. What is missing is that **nothing shows where the fields
are** — Acrobat tints them blue, pdfce paints nothing and only changes the
cursor to an I-beam. That is the whole of *"How do I click on a form to edit
it in the Canvas?"*

`canvas/forms.rs`'s own header names it and declines to build it:

> *"…a weaker one than Acrobat's blue field tint. The honest remedy is a
> **"highlight fillable fields" toggle**, which is a ribbon command; this
> module deliberately adds none and the entry point is reported rather than
> wired."*

It is a view overlay like rulers, the grid and find hits — **not** content
marking, so rule 4 permits it. One command, one condition, one overlay pass.

### 5. Resize an EXISTING page

`set_media_boxes(indices, rect)` shipped with `set_media_box` and only the
second is used. Belongs in Document ▸ Properties, and is a **design** question
before a coding one: does content move, does `/CropBox` follow, is shrinking
below the content a refusal.

> Read `archive/2026-08-18-mediabox-and-markup-reply.md` first. `/MediaBox` is
> inheritable (§7.7.3.4), so the write is three-way, and *"a target equal to
> the inherited value REMOVES the page's own entry"* is load-bearing and
> invisible to a one-page fixture — **writing to the ancestor that supplies
> the value resizes every sibling.**

### Not ours: revision clouds

Confirmed moving upstream on 2026-08-18 — `EditError::TooFewVertices` and a
`Cloud` subtype are in `D:\Dev\pdfce`'s working tree. The operator: *"don't
worry about item 5. It's aware of that one now."*

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
