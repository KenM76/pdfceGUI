# HANDOFF — read this first, then say "continue"

**Written 2026-08-13, at commit `a748414`.** For a session starting cold on
`D:\Dev\pdfceGUI`.

This file carries what the other documents cannot: the **working
agreements**, the **standing instructions**, and the **judgement calls**
that would otherwise have to be rediscovered. It does not duplicate
`FEATURES.md` (what works), `PROJECT_PLAN.md` (the charter) or
`RIBBON_IA.md` (the information architecture) — it tells you when to read
them.

---

## 1. Where things stand, in one screen

| | |
|---|---|
| **Shell HEAD** | `3e27b79` — clean tree |
| **Engine HEAD** | `D:\Dev\pdfce` at `b943ea1` |
| **Tests** | 1,184 passing, 0 failing |
| **Gates** | 8 of 8, 0 skipped |
| **Commands** | 88 registered · 88 declared-and-deferred |
| **Latest build** | `D:\builds\pdfcegui-20260813-2248-b943ea1-a748414\` |
| **Requests owed by pdfce** | **none.** Four filed and answered on 2026-08-14 — revision clouds, markup note text, markup opacity, tab-order authoring. Three are **accepted and scheduled, none started**; they block *items within* Phase 6, not Phase 6 itself. From the fourth, **`widget_rects(page_index)` shipped immediately** (engine `e8e9881`) and canvas form filling uses it. Tab-order *writing* stays blocked on their F4 — see §8. |

**Everything the operator asked for is shipped.** Phase 3 and Phase 4 are
complete, along with Print, Forms-fill, Icons, Open/Recent/Close and Find.
Every loose end the build agents reported has been closed.

**The operator gave the order on 2026-08-14**, and it is:

> **Phase 6 (markup) → Phase 7 (measure) → the three small unblocked items
> → OCR → Phase 5 (text editing).**

Phase 5 is therefore **last**, not next — which is worth stating plainly,
because it is the defect that began this project (*"text editing is weird
and doesn't just edit the existing box and move the text correctly as you
type plus flow to the next line doesn't work"*) and every earlier version of
this file treated it as the obvious next move. It is not. Do not start it
early.

Two things that order does not tell you, and that cost a day to find out:

- **Phase 6 and Phase 7 are both bigger than their rows implied.** Neither
  is "add kinds"; both begin by building a canvas tool substrate this shell
  does not have. See §8.
- **OCR is a licensing question before it is a GUI one.** The engine can
  recognise text end to end (`ocr::layer` writes the invisible sandwich at
  render mode 3, the `ocrs` weights ship at 12,240,008 B) and **no shell has
  a surface**. But `GUI_ROADMAP.md` records the blocker as *shipping a
  CC-BY-SA-4.0 model in an MIT repo* — "not a GUI problem". Settle that with
  the operator before building anything, and note the engine also says
  recognition quality is **unproven**: its only test documents are vector
  PDFs that already contain text.

---

## 2. The founding rule, which is not a slogan

> **Verify by driving the binary, not by a passing test.**

The project exists because two defects were invisible to a green suite.
Since then the count of defects found *only* by running the program and
reading its trace or its pixels has reached **eight**:

1. `Ctrl+O` printed in a tooltip, in the keymap, bound to nothing.
2. The icon painter existed, was tested, and was never passed to the ribbon
   — the whole ribbon was text buttons.
3. Find's current-hit highlight completely covered the word it highlighted.
4. Find's bar drew 108 pt left of its place for one frame on every open.
5. An undrawn page used a fill that read as blank paper, so a page still
   rendering looked like an empty one.
6. That page's explanatory sentence was centred in the *page* rather than
   in the part of the page on screen — a metre below the window.
7. A newly added panel was invisible to anyone who upgraded.
8. The grid was a tint rather than a grid: a one-point minor step, ~2,450
   lines a frame. **A screenshot could not catch this one** — 2,450
   hairlines and a wash are the same picture. It was found by printing the
   ladder the running app had actually chosen.

Number 8 carries the sharpest lesson available here: the existing test
passed because it asserted the grid was *finer* than the ruler, which it
emphatically was. **A test that checks a relation rather than a magnitude
is satisfied by any absurdity in the right direction.**

How to actually do it:

```bash
cargo build --release -p pdfce-gui
PDFCE_DIAG=1 ./target/release/pdfce-gui.exe "D:\Dev\temp\pdfce\SW41177.pdf"
```

Test documents that matter:

| file | why |
|---|---|
| `D:\Dev\temp\pdfce\ncored-benchmark-cad-drawing.pdf` | A3, 129,758 objects, ~1.2 s per raster. The performance case. |
| `D:\Dev\temp\pdfce\SW41177.pdf` | 36 SolidWorks sheets. The multi-page and mixed-size case. |

---

## 3. Standing instructions from the operator

These were given explicitly and are still in force.

1. **Check `D:\Dev\FeatureRequests\pdfce_FeatureRequests\open\` periodically
   while waiting on anything from pdfce.** Empty means nothing is owed. Read
   `INDEX.md` for the history; `archive/` is not read by default.
2. **Continuous scroll is an option, not a replacement.** Single page stays
   the default outside Read mode — *"the way I move around a page is great
   when working with drafting drawings"*. A change that makes single-page
   feel like a degraded continuous mode has failed regardless of the tests.
3. **Ignore the "nothing floats over the canvas" stance** when placing a
   transient bar. Superseded for Find; the bar floats, and the operator said
   to drop the argument.
4. **Make it work the way other programs do.** This retired the `Editing on`
   master toggle and it is the tie-breaker for interaction questions.
5. **Dispatch subagents freely; do not ask permission.** See the global
   `CLAUDE.md`. `D:\Dev\pdfce` is READ-ONLY to this project.

---

## 4. How the parallel work was actually run

This is the part that is easy to get wrong. Up to six agents ran at once,
and the only thing that prevented collisions was an explicit **territory
partition given in each prompt**.

**The pattern that worked:**

- Give each agent a *write territory* as an explicit list of directories,
  and an explicit **do-not-touch** list naming the other agents' territory.
- **Forbid command registration** to every agent but one. `shell/` is a
  single-writer resource — `commands.rs`, the manifest and the generated
  RON must move together.
- Tell each agent to **report entry points** rather than wire them, and do
  the wiring yourself afterwards.
- Tell them **not to commit**. Take the commits yourself, after verifying.
- Expect the crate not to compile at times. Tell agents to report breakage
  outside their files and carry on rather than fixing it.

**What still went wrong, so you can watch for it:** an agent added an
`Action` variant to `app/actions.rs` outside its territory because the
crate would not compile without it (harmless, and correct); another ran
`cargo build` inside `D:\Dev\pdfce`, which touched only its gitignored
`target/` but violated the read-only rule.

**Coordination beats racing.** When the Pages panel needed five lines in
`shell/` and another agent owned it, the right move was to message that
agent with the patch rather than edit around it.

---

## 5. Registering a command has five obligations

Every one has a test that fails loudly. This is the single most common way
to break the build.

1. The registry count in `shell/commands.rs`.
2. The group count in `shell/manifest/mod.rs`.
3. Removal from the `PLANNED` list if the command is named there.
4. Regenerate the RON:
   `cargo test -p pdfce-gui --lib rewrite_built_in_ron -- --ignored`
5. A `KNOWN` entry for any new `enabled_when` condition name.

Adding a **panel** has its own set — see `panels/mod.rs`, whose header
explains that three panels once shipped with a body, a rail entry and *no
control anyone could click*, passing every verification for their whole
shipped life.

---

## 6. Invariants that are not up for renegotiation

- **Actions, not mutations.** A widget is handed `&OpenDoc` and pushes an
  `Action`; everything is applied after the frame, in one place. This is a
  compile-time fact, not a convention.
- **One choke point for dispatch** (`app/dispatch.rs`). The arms *route*;
  they do not compute. The moment an arm works out *how* to do something,
  that rule exists in two places and only one gets fixed.
- **No placeholders.** A capability that is absent renders **nothing**,
  never a greyed control that explains itself badly. An unknown icon key
  draws a visible "missing" mark rather than a blank — because the label
  fallback is decided *upstream* of the painter.
- **Disclosure lives off-canvas** (Rule 4). The one-line test: *would a
  screenshot of the editing canvas differ from a screenshot of the same
  document saved and reopened?*
- **Every operator-visible string lives in `text/`**; every colour is a
  named role in `egui-shell/src/theme/`. Both have gates, and both gates
  have self-tests, because a grep that stops matching prints exactly what a
  clean run prints.
- **No `.rs` file over 1,500 lines.** `app/mod.rs` has been split twice
  under this rule, into `dispatch.rs` and `conditions.rs`, both at real
  seams. The old GUI reached 25,005 lines in one `main.rs`.
- **`egui-shell` knows nothing about PDF.** Enforced by
  `check-shell-purity.sh`. It reports; the application decides.

---

## 7. Build and package

```bash
cargo test --workspace
bash tools/gates/run-all.sh                 # 8 gates; exit 3 means SKIPPED, which is NOT a pass
python tools/package-portable.py --verify --note "what this milestone added"
```

`--verify` runs tests and gates **before** building, so a failure costs
nothing and leaves no folder. When it is not run, `BUILD-INFO.txt` says so
in those words.

**"Integrated with pdfce as a single exe" needs no fold-in.** `pdfce-gui`
depends on `pdfce-core` and `pdfce-render` **by path**, and Rust links them
statically — the release binary already carries the engine. Folding this
shell into `D:\Dev\pdfce` today would ship a *regression*, because measure,
redaction, the settings dialog and text editing still live only in the old
shell.

**Known environment quirk:** `--verify` may report the gates as skipped
because a spawned bash does not inherit `~/.cargo/bin`. If that happens,
run the tests and gates by hand, then package without `--verify` and state
the results in `--note`.

---

## 8. What is left, in the operator's likely order

| | |
|---|---|
| **Phase 5 — text editing** | The defect that started the project. Three distinct problems, not one: the edit unit is a single show-text operator rather than a visual box; nothing re-lays-out while you type and aligned/rotated text is moved wrongly on commit; reflow is blocked behind three gates. `DEFECTS.md` D4 has the full chain. **Ask before starting.** |
| **Phase 6 — markup** | **In progress, and larger than this row used to imply.** The new shell has *no markup placement at all*: all eight `markup.*` commands draw and fall through to `command-unimplemented`, `CanvasTool` has two variants, and there is no `canvas/markup.rs`. So it is *build the tool substrate, then ten kinds*, plus the Comments panel (which does not exist here either). **Three items needed engine changes; all three were filed and answered on 2026-08-14, accepted and scheduled, none started.** Revision clouds land as `MarkupSpec::Cloud` plus `Square { border_effect }` — and the *rectangular* cloud ships first, being the gesture people actually reach for. Note text lands as `/Contents` + `/T` + `/M` together, `/M` engine-stamped and `/T` optional with **no invented placeholder**. Opacity is `/CA` **alone** — writing `/ca` into the appearance stream would encode a pdfce render bug into the file format; see **`DEFECTS.md` D9**, which is the more urgent half of that exchange and is about *viewing*, not authoring. Polyline, polygon, ink, underline, strikeout, squiggly, width and fill are engine-ready and blocked on nothing. |
| **Phase 7 — measure** | A **salvage**, not a wiring job. This entry used to call the two-line ce dimension "the cheapest real feature in the backlog" because "the canvas gesture has no caller" — false in five documents at once, corrected 2026-08-14; see `SALVAGE.md`'s correction note. The gesture is built and tested in the *old* shell; this build has no measure tool at all, so the work is Class A `measure_tool.rs` (1,230 lines) plus ~900 lines of canvas hosting. Area and Count need engine changes; Angular is core-complete with no tool. |
| **Salvage remaining** | Redaction (its true-removal proof exists **only** in the old shell), and the settings dialog. |
| **S6 — deep zoom** | ⛔ Blocked on the reusable parsed handle, which pdfce has scheduled as `Pass 75.0`. Do not build tiling: measured as a 9× regression. |

Smaller, unblocked, and recorded in `FEATURES.md`:

- Panel toggle semantics (`show_panel` is show-only).
- The **edit-disclosure surface** — several features now trace disclosures
  that Rule 4 says must be *surfaced*, not traced. The guide-count refusal
  and the zoom-to-selection decline both wait on it.
- Scoped reset chooser.
- `ui-verify`'s `find_opens_and_finds` **has never passed here**: synthetic
  keyboard input does not reach the target window from the session that
  wrote it. It reports SKIP rather than blaming Find, on purpose.

  **★ A lead was raised against this on 2026-08-14 and then failed to
  reproduce. Recorded because the next reader will otherwise have it
  again.**

  The canvas form-filling work reported driving typing, Enter *and*
  Escape into the real binary successfully, and attributed it to
  `SetForegroundWindow` on the target PID plus verifying the foreground
  actually changed. That would have made this SKIP a two-line gap in
  `ui-verify` rather than an environment limit, and would have recovered
  every keyboard-blocked check — including the Escape rules for markup
  and for a focused form field, both currently asserted by test alone.

  **I tried to reproduce it directly and could not.** With the foreground
  PID confirmed equal to the target's, `keybd_event` for `Ctrl+2` produced
  no `chord-command` line and no mode change. A **mouse** click sent by
  the same mechanism moments later landed and traced
  `canvas-selection via=click`, so the window was live, the process was
  reading input, and the pointer half of the same API worked — **only the
  keystrokes went nowhere.** Sending a click *first*, on the theory that a
  real click confers something `SetForegroundWindow` does not, changed
  nothing.

  `ui-verify` already does the raise and already checks `is_foreground`
  before typing; its own SKIP text says the window reported itself
  foreground. So the missing ingredient, if there is one, is **not**
  foreground rights and **not** a prior click. Two candidates remain
  untested: the harness's 48-frame wait may be too short, and the
  successful report may have used a different injection API
  (`SendInput` rather than `keybd_event`). Worth an hour if keyboard
  coverage ever becomes the blocker; **not** worth treating as solved.

---

## 9. Two open questions worth putting to the operator

1. ~~**Should Read mode fill forms?**~~ **Answered by the operator on
   2026-08-14: yes.** Acrobat Reader fills forms in its default view and
   replacing it is the stated goal. It cost the taxonomy amendment plus a
   tab move — `edit.form_fill` became `view.panel_forms` on View ▸ Panels,
   because Read is shown File and View alone and a command lives on exactly
   one tab. Edit ▸ Forms kept create, manage and flatten: **filling is not
   authoring** is the line that move draws.

   Canvas filling then arrived the same day with **no mode gate at all**,
   which means Read fills forms on the page as well as in the panel. That
   is the same answer reached twice by different routes, so it stands — but
   note it was reached the second time *by omission* rather than by
   argument, and if anyone ever wants a mode to be genuinely read-only,
   `canvas::forms` is the second place that would have to learn about it.
2. **Per-mode memory of the page-display choice.** Deliberately not built:
   it is a second axis that collides with per-document, which is what was
   actually asked for.

---

## 10. Things that will bite you

- **`core.autocrlf` is true globally.** `.gitattributes` predates the first
  commit for that reason: CRLF normalization of PDF fixtures lands **in the
  index at `git add` time**, and a PDF's xref stores absolute byte offsets.
  Do not remove it. Do not add a binary type to it without `binary`.
- **`cargo test -p egui-shell` and `cargo test --workspace` compile with
  different `egui` features** (no fonts vs `default_fonts`). Layout tests
  can be entirely vacuous under one and not the other. Assert that a
  measurement *happened* (`Some(false)` rather than `None`), not just its
  value.
- **`ui-verify` refuses a stale binary.** That is the guard working; rebuild
  or point `--exe` at a packaged build.
- **Selection is an identity** — page, object, subpath, node — not a
  position. Paint-order indices survive `move_*` and do **not** survive
  `delete_*`.
- **~99 % of render cost is resolution-independent** on dense CAD. A small
  thumbnail is not a cheap thumbnail. A 1×1 *point* region costs 691 ms.

---

## 11. The relationship with `D:\Dev\pdfce`

Another session works that repository live. It is **read-only** here.

The channel is `D:\Dev\FeatureRequests\pdfce_FeatureRequests\`. Five
exchanges have completed, all answered within the hour. Two of them were
defects found from this side; **one of my four claims in a filing was
rejected, correctly** — `deletion_refusal` predicts deletion and matches
its guard exactly; I had compared it against *flatten*, which was my own
next item. Acting on it would have disabled a working Delete control, and
core now carries a test whose stated job is to stop a future reader
"correcting" a correct function on the strength of my report.

The lesson is worth carrying: **verify a claim against their source before
filing it**, and when a filing is wrong, record that it was wrong where the
next reader will find it rather than deleting it.

---

## 12. To resume

Read, in this order: this file, `FEATURES.md`, then whichever of
`PROJECT_PLAN.md` / `RIBBON_IA.md` / `MODES_AND_PANELS.md` /
`SHELL_FRAMEWORK.md` the task touches. `SALVAGE.md` before carrying
anything across from the old shell. `BENCHMARK.md` before making any claim
about rendering performance.

Then check `open/`, confirm the tree is green, and ask what to work on —
unless the operator has already said.
