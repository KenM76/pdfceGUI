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
| **Commands** | 90 registered · 88 declared-and-deferred |
| **Latest build** | `D:\builds\pdfcegui-20260813-2248-b943ea1-a748414\` |
| **Requests owed by pdfce** | **none.** Four filed and answered on 2026-08-14 — revision clouds, markup note text, markup opacity, tab-order authoring. Three are **accepted and scheduled, none started**; they block *items within* Phase 6, not Phase 6 itself. From the fourth, **`widget_rects(page_index)` shipped immediately** (engine `e8e9881`) and canvas form filling uses it. Tab-order *writing* stays blocked on their F4 — see §8. |

> **★ The table above is from 2026-08-13 and is superseded twice over.**
> A 2026-08-14 session landed the Read-mode gate and Phase 7; a
> 2026-08-15 session landed Phase 5's one-run text editor. Measured at
> the end of the latter: **1,744 tests passing, 0 failing · 10 of 10
> gates, 0 skipped · 101 commands registered · 31 groups · 31
> scaffolded.** `FEATURES.md`'s own header carries the same figures.
> Re-measure rather than quoting either table if any time has passed;
> the numbers above are the ones a test pins, and prose drifting from
> them is a defect this project has now had four times.

### ★★ 2026-08-17 — the operator's report, and what it turned up

> *"I tried a lot of the features that have been added only to find there is no
> surface for changing or editing the settings for them. please add the ones
> that are missing for all of the features currently supported in the gui. also
> port the settings dialog from the pdfce gui. also the print dialogue didn't
> work."*

Three commits. Every one of them found something worse than the thing reported,
and all three failures are the **same shape** — wiring that no test in the
workspace could see:

| # | reported | found |
|---|---|---|
| `6d790db` | print didn't work | `pdfce-print` was linked into every shipped binary and the adapter's four calls still returned `NotLinked`. **A green test held it there**: `every_hole_refuses_rather_than_guessing` asserted all four refused, which was correct while unlinked and became a lock the moment it was not — doing the right thing would have turned the suite red |
| `87b4f3d` | no settings surface | `file.settings` inert for the whole project — and, measured against the old shell, **nine of its thirteen settings were persisted, shown, edited and never read by anything**, discarded at every call site that built its own option struct. `app::settings` is now the one funnel and a `syn` check enforces it. D10's second half closed, proved in pixels |
| `4035b64` | can't change a markup's colour | Markup ▸ Style declared `Item::custom("colour_swatch")` since S2 and **no renderer ever matched the kind**, so the group shipped as a caption over an empty band. The manifest test asserted the item was *declared* and passed correctly |

**The generalisation is worth carrying**: there are now three distinct ways for
a control to ship inert, and each defeats a different guard.

1. **A command with no dispatch arm.** Caught by `shell::commands::reach`.
2. **A linked crate with a refusing adapter.** Caught by nothing — the
   adapter's own tests asserted the refusal.
3. **A declared `Item::Custom` with no renderer.** Caught by nothing, and it is
   the quietest: a `Custom` item carries no command id, so it is invisible to
   every check built on `command_references()`, and the manifest test that
   *does* see it is asserting the manifest, which was right.

Only one oracle sees all three: driving the binary and reading what it declared.

**D11 was also broken again, three days after being written, by someone who had
read it** — the settings window's headings used `.strong()` and rendered pale
grey on pale grey. `tools/gates/check-strong-text.sh` now enforces it, found a
third latent instance in `egui-shell` on its first run, and had to learn to
measure its window in *code* lines rather than source lines so a well-commented
fix would not fail a gate a terse one passes.

### ★ The three follow-ups, resolved — two built, one measured and blocked

| | outcome |
|---|---|
| **Measure ▸ Set scale** | ✅ `980971f`. The model was salvaged whole in Phase 7 and only a window was missing. **Manage groups stays deferred and its reason was rewritten**: it waited on "the same absent dialog", the dialog landed, and the entry stays — because rename and delete are *not in the shipped `EditSession` surface* and a management window missing half its verbs is worse than none |
| **The seven `view.*` settings** | ✅ `29cdc31`. **Two built, five deleted.** Four named capabilities that do not exist; the fifth, `app_initiative`, existed to switch off a behaviour pdfce does not have. All seven unregistered on R8, the empty Render group deleted, 32 groups → 31 |
| **The Format contextual tab** | ⛔ `3784cca`. **Blocked, not unbuilt** — see below |

**The Format tab is the finding worth carrying.** §5.8 specifies twenty-four
property editors across six selection types and the tab can carry **one**,
`Delete`, which it already has. Two independent blockers:

1. **`EditSession` has no verb that modifies an annotation.** `add_markup`,
   `add_text_annotation`, `delete_annotation` and two deletion predicates is the
   whole surface. Delete-and-re-add is not a workaround — it loses the
   annotation's identity and with it its `/NM`, its z-order in `/Annots`, and
   any reply thread hung off it as `/IRT`. The **one** exception is the ce
   dimension row: dimensions have a style model and nothing else does.
2. **The canvas selection cannot address an annotation.** `Selection` is
   `page + object + subpath + node`, a paint-order index into page *content* —
   which is what makes it immune to zoom, and also means a markup or a dimension
   is not selectable at all.

The second is ours. The first is filed, along with a request for an
`annotation_at(page, point)` sibling of the `widget_rects` query that unblocked
canvas form filling — the exact precedent, and it worked.

### Four requests open to pdfce, all filed 2026-08-17

`open/` was empty before this session. It now holds four, three of them from
one operator question about print:

| request | finding |
|---|---|
| `devicesettings_pick_tray_is_never_read` | the field is declared, documented, plumbed through `spool`'s signature, and **read nowhere**. The GUI shipped a checkbox for it; the checkbox is removed |
| `orientation_auto_is_per_job_not_per_page` | documented as per-page in a heading that says so, implemented per-job — `build_devmode` is called once with `first_page_pt` |
| `no_paper_size_selection_in_the_print_path` | no paper list, no way to request one, no route to the driver's properties dialog. The dialog now **discloses** which paper the job is planned against and that pdfce cannot change it |
| `no_verb_modifies_an_existing_annotation` | the Format-tab blocker above |

### What is still missing, from the inventory sweep

Not asked for, and worth having on the record — every one is a shipped feature
with a hard-coded value and no surface:

- **Redaction**: fill colour, overlay text and quadding are three `None`s at
  `panels/redact.rs:418-420`. The engine takes all three.
- **Snap and drafting**: snap tolerance (10 px), selection tolerance (6 px),
  grid alphas, guide catch radius, ruler pitch and thickness, zoom min/max,
  default fit mode. All compiled-in, all preference-shaped, and `app::prefs`
  now exists as their home.
- **New document**: `file.new` always makes an A4 from a baked-in template with
  no chooser. `file.new_from_template` is `PLANNED`.
- **Markup, the rest of the Style group**: arrowhead length and angle, ink
  simplification tolerance. Fill and opacity stay blocked (design decision and
  engine respectively).
- **No UI-scale or base-font-size control anywhere**, which is an accessibility
  gap rather than a preference.

---

**Everything the operator asked for is shipped.** Phase 3 and Phase 4 are
complete, along with Print, Forms-fill, Icons, Open/Recent/Close and Find.
Every loose end the build agents reported has been closed.

**The operator gave the order on 2026-08-14**, and it is:

> **Phase 6 (markup) → Phase 7 (measure) → the three small unblocked items
> → OCR → Phase 5 (text editing).**

### ★ Where that order has got to, as of 2026-08-14 (second session)

| | |
|---|---|
| **Read mode is genuinely read-only** | Asked for mid-session and built. Capability is derived from the **mode's tab list in the manifest**, never from the string `"read"`, so the ribbon and the canvas read one sentence. Closes **`DEFECTS.md` D6**. Proven by `ui-verify` driving the real window, not by tests. |
| **Phase 7 — measure** | **The salvage landed 2026-08-14 and three tools place dimensions**: Linear (three clicks — what, to what, where), Two-line, and **Radius / diameter**. `measure_tool.rs` came across whole into `canvas/measure/{pick,scale,state}.rs`, the 12.M1 snap primitives into `canvas/snap.rs`, 45 tests carried, **no engine API had moved**. ★ **The radius/diameter blocker is closed, by operator decision.** This row used to say the gesture had no natural end and the only place to say "done" was an accept box decision 024 retired — true, and the operator's answer on 2026-08-14 was to give it **two** endings that are not boxes: a **double-click** on the canvas and a registered **`measure.finish`** command, both routed through one commit path in `canvas/measure/circular.rs` so they cannot author different dimensions. Finish is gated on a new condition, `measure.finishable` (the tool armed *and* a non-degenerate fit), because a Finish that is always enabled is a control that does nothing on almost every press. The snap query is also wired now. What remains: **Set scale** still has no dialog to ask the length in; Area and Count still need engine changes; Angular is core-complete with no tool. See `SALVAGE.md`'s Phase 7 entry for the three deliberate departures from the source and the axis collision it surfaced. |
| **The three small unblocked items** | Two are done — the **edit-disclosure surface** and the **chord/mode gate**. Panel toggles are the third and the operator has chosen the semantics. |
| **Four operator decisions taken** | 2026-08-14: chords gate on tab membership; radius/diameter gets **both** a Finish command and a double-click; an open panel's control **closes** it; and `⚠` was to be fixed by **adding font coverage**. |
| **★ …and the fourth decision was answered by a measurement instead** | The operator chose to bundle a font for `⚠`. **No font was needed: `⚠` was never missing and renders correctly.** The broken thing was the *gate's predicate* — `epaint`'s `Fonts::has_glyph` asks "is this drawn by a face other than the one supplying the substitution mark?", so it answers `false` for every codepoint whose first supporting face is that one. The unanswerable demonstration is that **`has_glyph(Monospace, 'A')` is `false`**. So `DEFECTS.md` D12's measured lists were an artefact of the instrument, and four of its thirteen "shipped tofu" sentences were fine all along. **No dependency, no font data, zero added bytes.** The lesson is the general one: *a measurement is only as good as the predicate behind it*, and D12 is rewritten with the wrong claim kept visible. The gate was then widened to every `text/` module and **found two real tofu boxes on its first run** — both now fixed; see D12. |

**Two taxonomy questions were open and are the operator's**, both of the same
shape as the `edit.form_fill` → `view.panel_forms` move that is already in
this file. ★ **The first is now closed — answered and shipped on 2026-08-14 —
and is kept here struck through rather than deleted**, because the pattern is
the useful part: a chord refused in a mode where the operator plainly needs it
is evidence that the *command's tab* is wrong, not that the gate needs an
exception. That is twice this has happened and twice the fix has been a tab
move.

1. ~~**`edit.copy_page_text` sits on the Edit tab**~~ — **CLOSED by the
   operator on 2026-08-14: the destination is File ▸ Export.** The question
   was that `Ctrl+Shift+C` was refused in Read while Acrobat Reader copies
   text, which is the standard Read is measured against. Copying is not
   authoring, so both verbs left the authoring tab: `edit.copy_page_text` →
   **`file.copy_page_text`**, `edit.copy_document_text` →
   **`file.copy_document_text`**, tokens 122 and 123, the chord following the
   command in the manifest keymap. Export rather than a new Clipboard band,
   because an export is content written out to somewhere that is not this
   document and the destination — clipboard rather than path — is what the
   label says. **Edit ▸ Clipboard was deleted rather than shipped empty**, so
   the group count is **31**, not 32; that number is quoted in six places and
   all six moved together. One test now stands under the restored property,
   `both_text_copy_commands_are_offered_by_every_mode`, because nothing else
   would notice a revert. (At the time that test was written neither command
   had a dispatch arm; **both were wired on 2026-08-14** by the canvas
   text-selection work, and both read the same page extraction the canvas
   does, so ribbon-copy and selection-copy cannot disagree.)
2. ~~**Worded decline**~~ **— CLOSED 2026-08-14, built as specified.** `ZoomOutcome::NoBounds`/`NoCanvas` are worded in the status bar through `app/status/decline.rs`; the ceiling-clamped region zoom is deliberately left unworded as a partial grant. One thing came back with it: no chord binds `view.zoom_selection`, and its ribbon control is greyed exactly when it would decline. **Settled 2026-08-14 under the reference-application instruction** (§3 item 4): SolidWorks and Acrobat both reach zoom-to-selection by right-click and only Inkscape binds a key, so it joined the `canvas.object` context menu and **no chord was invented** — Inkscape's key is a bare digit, this shell's chords are `Ctrl`-modified by construction, and `Ctrl+1/2/3` are the mode selector. A menu on an object implies a selection, so the decline sentence stays **race-only**, which is the right shape: it is a safety net for the case where bounds evaporate between the frame that drew the enabled control and the frame that applied it.

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
reading its trace or its pixels has reached **eleven**:

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
9. The page-text extraction was paid **at open** rather than on the gesture
   that needs it — 392 ms on the benchmark sheet, charged to an operator who
   had touched nothing. **The suite was green and the cache was working**:
   exactly one extraction happened, which is all a test can ask about. What
   was wrong was *when*, and the only thing that carries a when is a
   timestamped trace line. Found by reading `page-text` in a driven run and
   noticing it sat beside `open` instead of beside the first sweep.
10. The freehand ink trail was read **after** the gesture machine had already
    cleared it. `GestureState::update` drops its own drag on the frame it
    reports `Complete`, and `ink::sync` was called after it — so on exactly
    the frame the release arrived the trail answered `None`, and every
    freehand stroke authored **two points**. Every unit test passed: they
    call `drag` directly, and **none of them can see the order
    `canvas::interact` calls two functions in.** Found from the trace line
    `markup-commit kind=Ink raw=2 kept=2` on a drag that was hundreds of
    points long — which is also why that line carries `raw=` beside `kept=`,
    since a build whose simplification did nothing emits an otherwise
    identical line.
11. The redaction panel's apply control was laid out **below the bottom of
    its own pane** — declared at `y = 801.7` inside a body ending at
    `y = 770.0`, on the shipped window size, with a mark already made. Every
    unit test passed. `MODES_AND_PANELS.md` already records the rule this
    proves twice over: *layout and clipping defects have exactly one oracle,
    a rendered screenshot* — and the control it hid was the one that applies
    an irreversible edit.

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

   **★ Sharpened 2026-08-14, and this is the most useful instruction in the
   file**: *"make your best educated guesses to match what inkscape, acrobat,
   and SolidWorks do."*

   Three named reference applications, and they are named for reasons that
   cover the product between them: **Acrobat** is what pdfce replaces,
   **Inkscape** is the vector editor whose docking and tool model this shell
   already benchmarks against (`MODES_AND_PANELS.md` Part 2), and
   **SolidWorks** is where the operator's drawings come from and therefore
   where their muscle memory lives.

   What it changes in practice: **do not ask the operator how an interaction
   should behave.** Look at what those three do, pick, and *record which one
   you followed and why*, so the guess is auditable rather than merely made.
   Where they disagree, say so and say which won — that disagreement is
   usually the interesting part of the decision.

   Worked example, from the day it was given: *how is zoom-to-selection
   reached?* SolidWorks and Acrobat both put it on the right-click menu;
   only Inkscape binds a key (bare `3`, in its `1`–`6` zoom family). Two of
   three said menu, so it went on `canvas.object` — and no chord was
   invented, because Inkscape's family is *unmodified digits* while this
   shell's manifest chords are `Ctrl`-modified by construction and its
   `Ctrl+1`/`2`/`3` are the mode selector. Transposing `3` onto `Ctrl+4`
   would have matched the letter of neither convention and the muscle memory
   of nobody. The whole argument is at the registration site in
   `shell::menus`.

   **A second worked example, and it sharpens the rule.** *How does a
   polyline or polygon end?* All three double-click, so that half was not a
   judgement call. The disagreement was the *other* way out: Inkscape and
   SolidWorks both close a shape by clicking the first vertex, and **Acrobat
   does not**. Acrobat won — two against one — because `/Polygon` closes back
   to `/Vertices[0]` by ISO 32000-1 §12.5.6.13, so pdfce is in Acrobat's
   position, and a click-the-first-vertex rule would author a duplicate
   vertex and a zero-length closing segment. **The majority had never faced
   the surface.** SolidWorks' Escape-to-end was refused outright on a
   different ground: in this shell Escape means *abandon*, and committing on
   the key an operator presses to say "no" is the least recoverable reading
   available.

   So the rule has two halves, and the second is the one that keeps being
   load-bearing: **match what they do, but first ask which of them actually
   has the surface you are deciding about.** That test has now decided three
   cases — zoom-to-selection, the Edit text tool, and this one — and in two
   of the three it overturned the head-count.

   The instruction does **not** license guessing about *claims* — refunds,
   licensing, what the engine does, what a file format permits. Those are
   still verified. It licenses guessing about **behaviour**, where a
   defensible convention beats a blocked question.
5. **★ Read may produce a new document; it may not modify this one.**
   Operator instruction, 2026-08-14, given as a rule about OCR:

   > *"if in read mode ocr should still be available, but it will prompt to
   > save changes as save as instead of save."*

   Recorded in its **general** form, because that is the form that decides
   future cases rather than one. It explains the two exceptions Read already
   carries — **form filling** (2026-08-14) and now **OCR** — as exceptions
   rather than inconsistencies: neither changes the document the operator was
   handed. It also settles in advance every capability of the same shape that
   is still to come: flatten, redact-apply, PDF/A convert, page export.

   The line Read's gate actually draws is therefore **not** "no writes". It
   is *no writes to **this** file*, and the enforcement point is the **save**,
   not the operation. That is worth knowing before anyone tries to make the
   canvas gate cover it: `app::modes::capability` governs *gestures*, and OCR
   is not a gesture.

   **Two things about it that are true today and easy to get wrong:**

   - **It is currently vacuous.** There is exactly one save command,
     `file.save_copy`, and pdfce never overwrites the original unless the
     operator picks it — so every mode already behaves this way. The rule
     becomes load-bearing **the day in-place `Save` lands**, which is
     precisely when someone will be least likely to remember it. That is why
     it is written here against a command that does not exist yet.
   - **OCR is still blocked, and not on anything in this repo.** The blocker
     is shipping a CC-BY-SA-4.0 model in an MIT repo, plus the engine's own
     note that recognition quality is unproven. Both are the operator's to
     settle. See `FEATURES.md`'s OCR section for the Find-offers-OCR trigger
     rule, which has one trap in it: *"the document is images"* is not
     *"this search had no matches."*

6. **Dispatch subagents freely; do not ask permission.** See the global
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

## 5. Registering a command has six obligations

Every one has a test that fails loudly. This is the single most common way
to break the build.

1. The registry count in `shell/commands.rs`.
2. The group count in `shell/manifest/mod.rs`.
3. Removal from the `PLANNED` list if the command is named there.
4. Regenerate the RON:
   `cargo test -p pdfce-gui --lib rewrite_built_in_ron -- --ignored`
5. A `KNOWN` entry for any new `enabled_when` condition name.
6. **★ A dispatch arm — or an argued entry in
   `shell::commands::reach::SCAFFOLDED`.** Added 2026-08-15, and it is the
   only one of the six that asks whether the command *does anything*.
   `file.save_copy` passed the other five for the whole life of the project
   while doing nothing: registered, drawn on the quick access toolbar, bound
   to `Ctrl+S`, printed in its own tooltip and in the shortcuts reference.
   `edit.undo`, `edit.redo` and every page verb shipped in v0.1.0 the same
   way.

   The check parses `app/dispatch.rs` with `syn` — a `match` is not a regular
   language, so it is not grepped — and it does not parse the guard arms at
   all: it extracts *which function* each guards on and then **calls the real
   one** against the real registry. A set-equality test welds the two halves,
   so a new guard fails by name and a deleted one stops vouching for its
   family. `include_str!` makes a moved dispatcher a compile error, so
   "scanned nothing" cannot pass as "found nothing".

   **The list has already gone down twice**, which is the outcome this
   obligation exists to produce rather than a register that only grows:
   `SCAFFOLDED` 38 → 33 and its `★ P3` subset 11 → 8, by wiring three
   controls that had five surfaces and no behaviour. A fourth,
   `view.show_points`, was investigated and **stayed** — with its reason
   upgraded from *"no recorded reason anywhere"* to a cited blocker, which
   is the other honest outcome and the more common one. Then **33 → 31**
   on 2026-08-15 when `edit.text` and `edit.add_text` got real dispatch
   arms; `★ P3` unchanged at 8.

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
| **Phase 5 — text editing** | **Started 2026-08-15 on the operator's explicit instruction, and partly landed.** Of D4's three problems: **D4b's two wrong cases are FIXED** — aligned tails are pinned and rotated text is no longer shifted along the wrong axis (`canvas::textedit::disposition`), proved by a `ui-verify` check that re-opens the saved copy in a second process and asserts the untouched line's `Tm` survived, with the old `EditOptions::default()` build planted twice to confirm the check fails against it. **Not done:** per-keystroke re-layout — measured at 102.77 ms on a SolidWorks sheet and blocked on the engine, which keeps `plan_edit` `pub(crate)` so there is no dry run; **D4a's cross-run edit**, which needs a multi-run request core does not have and now refuses in a sentence rather than by a dead keyboard; **D4c's three gates**, untouched. `edit.add_text` is wired and unit-tested but not driven, and has no font/size/colour surface. `DEFECTS.md` D4 carries the measurement table and the honest single-line limit. |
| **Phase 6 — markup** | **In progress, and larger than this row used to imply.** The new shell has *no markup placement at all*: all eight `markup.*` commands draw and fall through to `command-unimplemented`, `CanvasTool` has two variants, and there is no `canvas/markup.rs`. So it is *build the tool substrate, then ten kinds*, plus the Comments panel (which does not exist here either). **Three items needed engine changes; all three were filed and answered on 2026-08-14, accepted and scheduled, none started.** Revision clouds land as `MarkupSpec::Cloud` plus `Square { border_effect }` — and the *rectangular* cloud ships first, being the gesture people actually reach for. Note text lands as `/Contents` + `/T` + `/M` together, `/M` engine-stamped and `/T` optional with **no invented placeholder**. Opacity is `/CA` **alone** — writing `/ca` into the appearance stream would encode a pdfce render bug into the file format; see **`DEFECTS.md` D9**, which is the more urgent half of that exchange and is about *viewing*, not authoring. Polyline, polygon, ink, underline, strikeout, squiggly, width and fill are engine-ready and blocked on nothing. |
| **Phase 7 — measure** | **Three tools place dimensions**: Linear (three clicks — what, to what, where), Two-line, and **Radius / diameter**. `measure_tool.rs` came across whole into `canvas/measure/{pick,scale,state}.rs`, the 12.M1 snap primitives into `canvas/snap.rs`, 45 tests carried, **no engine API had moved**. ★ **This row used to name three remaining decisions and two of them are taken.** *Radius/diameter had no natural end to its gesture and the only place to say "done" was an accept box decision 024 retired* — the operator's answer on 2026-08-14 was **two** endings that are not boxes, a double-click and `measure.finish`, through one commit path in `canvas/measure/circular.rs`; the Finish control is gated on a new `measure.finishable` condition so it is live only when there is a non-degenerate fit to commit. *The snap query is unwired* — it is wired. What is left is **Set scale**, which still has no dialog to ask the length in. Area and Count still need engine changes; Angular is core-complete with no tool. See `SALVAGE.md`'s Phase 7 entry for the three deliberate departures from the source and the axis collision it surfaced. |
| **Salvage remaining** | Redaction (its true-removal proof exists **only** in the old shell), and the settings dialog. |
| **S6 — deep zoom** | ⛔ Blocked on the reusable parsed handle, which pdfce has scheduled as `Pass 75.0`. Do not build tiling: measured as a 9× regression. |

Smaller, unblocked, and recorded in `FEATURES.md`:

- ~~Panel toggle semantics~~ — **done 2026-08-14.** An open panel's control
  closes it; `file.properties` and `markup.comments` deliberately do **not**
  toggle, because they answer *"tell me about this thing"* rather than *"is
  this panel open?"*. See `app/panels.rs`.
- ~~The **edit-disclosure surface**~~ — **done 2026-08-14**, and the two
  things that were waiting on it are settled: the zoom decline is built
  (`app/status/decline.rs`, same surface, *different* store), and the
  guide-count refusal can now follow the same pattern.
- ~~**A text tool for Edit**~~ — **done 2026-08-14.** `CanvasTool::Text`,
  armed by `view.tool_text` beside the hand tool in View ▸ Navigate. It
  closed **two** things: Edit could not sweep text, and the three
  text-markup controls were drawn on the Markup tab in Edit and could never
  enable — a live P3 tension, now observed closed by `ui-verify`'s
  `text_tool_selects_and_marks_in_edit`.

  ★ **The reference applications disagreed, and how that was resolved is the
  reusable part.** Acrobat and SolidWorks resolve text-versus-object
  *contextually inside one tool*; only **Inkscape** uses a separate Text
  tool. Inkscape won and **not by head-count**: an object marquee over
  vector content is a surface Acrobat does not have at all, so its
  contextual answer was not an answer to this conflict. The deciding
  argument was concrete rather than taxonomic — a contextual press would
  make a marquee over a region containing text unpredictable, and that is
  the commonest gesture in Edit. **When the three references disagree, ask
  which of them actually has the surface in question**; a majority that has
  never faced the problem is not a majority.

  One consequence worth knowing before touching the gesture layer: the new
  rung sits **above** the `caps.edit_content` branch, so text-versus-content
  exclusivity moved from *construction* to *precedence*. An object selection
  and a text selection can now both be non-empty, which is why
  `canvas::keys`' Escape ladder had to be re-argued rather than merely
  extended.

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

   **★ The operator asked for exactly that on 2026-08-14** — *"in read mode
   the document shouldn't allow editing"* — and the answer to the sentence
   above turned out to be **no, `canvas::forms` stays out.** Filling is not
   authoring; it is the primary reason most form documents exist, and
   Acrobat Reader fills forms in its default view. What the gate covers is
   the canvas *gestures* (`app::modes::capability`, `app::gating`), derived
   from the mode's **tab list** rather than from the id `"read"` — so the
   ribbon and the canvas cannot disagree about what a mode is. `forms.rs`
   was left untouched, deliberately, and its header's argument for that is
   now load-bearing rather than incidental.
2. **Per-mode memory of the page-display choice.** Deliberately not built:
   it is a second axis that collides with per-document, which is what was
   actually asked for.

---

## 10. Things that will bite you

- **★★ Registration is not implementation, and five surfaces will lie about
  it.** `file.save_copy` was registered, drawn on the **quick access
  toolbar**, bound to `Ctrl+S`, listed in the shortcuts reference, and
  printed "(Ctrl+S)" in its own tooltip — with **no dispatch arm**. Nothing
  this shell built could be written to disk, for the whole life of the
  project, and it was within an hour of being released that way.

  An audit afterwards found the same shape in **`edit.undo`/`edit.redo`**
  (QAT, three chords) and in **every page operation**, six of which the
  Pages context menu offers while `panels/pages/select.rs` maintains a
  multi-select model to feed them.

  The audit is one command and is worth running before any release:

  ```bash
  # every registered id, against the ids dispatch.rs actually names
  # (remember the guard arms: markup_for_command, measure_for_command,
  #  Panel::from_command_id, page_display_for_command, chrome_for_command)
  ```

  **A `command-unimplemented` trace is the only honest signal**, and nothing
  reads it. The durable fix is a test asserting that every registered command
  is reachable by *some* arm — literal or guard — with an explicit,
  argued allow-list for the ones deliberately scaffolded. Until that exists,
  audit by hand.
- **★ The conventional value can be the worst one, and only measurement
  tells you.** OCR shipped with `OCR_DPI = 300` — the number every scanning
  guide gives. Measured against `SW41177.pdf` using its own vector text as
  ground truth: 72 DPI → 34.8 %, 100 → 20.0 %, **150 → 44.7 %**, 200 → 27.5 %,
  **300 → 3.3 %**. The conventional answer was the worst of the five by an
  order of magnitude, because `ocrs` resizes every image to its model's fixed
  input — so **pixel count governs, not resolution**, and 300 DPI on an A1
  sheet throws away almost everything in the downscale. The constant is now
  `TARGET_PIXELS`, with the table in its doc comment. **Before trusting a
  parameter because it is standard, ask what the standard was measured on.**
- **★ The RON has now been found stale three times in one day**, by five
  separate changes that each missed it. See §5 obligation 4. Nothing about
  this is going to improve by asking people to remember; it wants either a
  non-`--ignored` test that regenerates and fails on a diff, or a pre-commit
  hook. Until then, run it after **every** manifest touch, and re-run it last
  when several sessions are landing at once.
- **★ A test fixture that is not themed like the running application hides
  spacing bugs.** The two-row ribbon's first cut padded rows by
  `rows×height + (rows−1)×spacing` — one gap short, because egui advances the
  cursor past *every* rect including the last. **Every test in the crate
  passed**, because `width_tests`' context installs a font but no theme, so
  egui's default `interact_size.y` (18 pt) sat 6 pt under the theme's
  `control_height` (24 pt) and the slack swallowed the error. It was visible
  only in the running binary's trace, as one group 68 pt tall beside another
  at 64. `height_tests::context()` now applies the theme, and
  `the_fixture_is_themed_like_the_running_application` guards it. **A layout
  fixture must be built like the thing it stands in for**, or its slack is
  the bug's hiding place.
- **★ A gate can be satisfied by a comment saying the thing is missing.**
  The shipped-assets gate's first self-test plant — declaring the OCR weights
  redistributed before writing their notice — **was not caught**, because
  `about.hbs`'s epilogue names that directory inside an HTML comment
  explaining it is deliberately absent, and a presence check found the string.
  Fixed by stripping comments before the check, and pinned by a sixth
  self-test case. The general form is worth carrying: **a check that greps for
  a string is a check on the file's *text*, not on its *output*** — render
  first, then assert.
- **★ Attribution is a shipped artefact, not a source-tree one.** Building
  the OCR prerequisite found **three third-party works this shell had been
  redistributing with no notice at all** — the Foxit CFF faces, the Adobe
  Core-14 AFM metrics and the Adobe Glyph List, all compiled into the binary
  by the engine crates. Nothing detected it for the whole life of the project,
  because `cargo-about` sees Cargo dependencies and these are `include_bytes!`
  payloads. **If it is in the binary and someone else wrote it, it needs a
  notice**, and the only thing that finds those is a gate that reads what
  packaging actually copies.
- **★ The RON regeneration is the obligation that silently rots**, and it has
  now rotted twice. Of the five obligations in §5, four fail loudly — a
  count assertion, a group assertion, a `PLANNED` disjointness test, a
  `KNOWN` lookup. **Obligation 4 has no compiler behind it and no failure
  until someone else runs the round-trip**, so a session that forgets it
  leaves `shell/ron/built_in.ron` describing a ribbon the build does not
  have. On 2026-08-14 it was found stale by *five* separate changes at once
  — the text-copy move to File ▸ Export, Edit ▸ Clipboard's deletion, three
  text-markup commands, two measure commands, and a context-menu entry —
  none of which had written it back. Run it, every time, even when your
  change "obviously" did not touch the manifest:

  ```bash
  cargo test -p pdfce-gui --lib rewrite_built_in_ron -- --ignored
  ```

  The round-trip test `the_ron_file_and_the_rust_agree` is what eventually
  catches it, which means the person who pays is whoever next touches the
  manifest rather than whoever broke it. **It has now been found stale twice
  in one day**, by two different sessions, which is the strongest available
  argument that this should not depend on anyone remembering.
- **★ A fixture can flatter the thing it measures, and the numbers will look
  fine.** The ink simplification was first measured against a synthetic trail
  whose disturbances were applied *along the arc's tangent* — so both of them
  only re-spaced samples along a path whose shape never changed. It reported
  17 points kept at a 0.5 pt tolerance and 33 at 0.125: a suspiciously flat
  response to a 16× change, and the tell. Recomputing retention independently
  exposed it; the fixture now offsets **radially** and carries an assertion
  that the worst deviation actually exceeds half the tolerance, so a future
  fixture that stops exercising the bound fails rather than flattering it.
  This is `HANDOFF.md` §2's grid lesson wearing different clothes: **a
  measurement that moves in the right direction is not evidence that it
  measures the right thing.**
- **Prose that quotes a number drifts from the number.** This has now
  happened five times: the command count in two module headers, the group
  count in six places, the test count in two documents, and — caught
  2026-08-14 — the icon coverage split, which read *"82 of 93 named, 12
  refused"* when 82 + 12 = 94 ≠ 93. Each was true when written. The fix that
  works is a **test that asserts the arithmetic**, not a comment asking the
  next reader to keep it current; `the_icon_coverage_split_adds_up_to_the_registry`
  is the current example.
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
- **★ An `Options` flag that defaults off will silently neuter a correct
  decision function, and every unit test will stay green.** The text-edit
  disposition chooser is pure — it reads a text matrix, a CTM and an
  alignment, and picks `Pin` or `Reflow`. It was correct, and it was
  about to be permanently dead: `ExtractOptions::capture_provenance`
  **defaults to `false`**, and the shell's shared `page_text()` cache is
  built with `default()`. Fed from that cache the chooser would have
  received a `None` pin and identity matrices — so the rotation guard
  could never fire, on any document, while its own tests passed against
  hand-built matrices. `plan` now runs its own provenance-capturing
  extraction once per commit.

  The general shape: **a pure function's tests prove the function, not
  its inputs.** When the input arrives through a cache someone else
  configured, assert on a real document that the discriminating field is
  actually populated — or the feature is decorative.
- **Two files are now at the R2 ceiling**: `canvas/tool.rs` (1,487) and
  `shell/commands/reach.rs` (1,498), against a limit of 1,500. The next
  edit to either must split it first. `reach.rs` in particular grows
  with every scaffolded command that gets an argued reason, so it will
  hit the wall on ordinary work, not on a rewrite.

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
