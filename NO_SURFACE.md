# NO_SURFACE.md — shipped behaviour with a hard-coded value and no control

**What this is.** Every tunable in `crates/pdfce-gui/src/` that an operator
would plausibly want to change and cannot: a compiled-in constant, a `Default`
impl, or a field only settable from code. One row each, with `file:line`.

**Why it exists.** The operator's report of 2026-08-17 was *"I tried a lot of
the features that have been added only to find there is no surface for changing
or editing the settings for them."* Answering that needed an inventory, and an
inventory that lives only in a session is one somebody re-derives. This is the
inventory, kept.

**How to read a row.** *Belongs* is the destination `RIBBON_IA.md` names, or —
where it names none — the honest place for a value of that kind. A row is not a
commitment to build it; several of these should stay compiled-in forever, and
saying which is part of the value.

---

## Status, 2026-08-17

This file was compiled from a sweep that began at `f794e27` and finished after
four commits had landed under it. **The markup section below is the part that
moved**, and it is corrected here rather than deleted, because the correction is
the useful bit:

| the sweep found | now |
|---|---|
| *"Markup ▸ Style renders an empty captioned band"* — `colour_swatch` declared and never drawn, so colour, width, fill and opacity all had **zero** surface | ✅ **Fixed in `4035b64`.** Two colour swatches and a width. `RIBBON_IA.md` §5.5's *"partial G — colour only"* is now true of this build, having described the **old** shell |
| Pen colour, highlight colour, stroke width | ✅ Settings are live — `canvas::markup::pen`, edited from the ribbon's Style group |
| Fill | ⛔ **design decision, not a gap.** `spec` passes `interior: None` with a note: *"a filled comment shape hides the drawing it is a comment about, which on a CAD sheet is the whole content under it."* Reversing that is the operator's call |
| Opacity | ⛔ **blocked on the engine.** Annotation transparency is `/CA` and `pdfce-core` does not write it |
| Arrowheads, ink tolerance, note text | ⬜ still open — the rows below stand |

Everything in sections 2 to 5 was verified after those commits and stands as
written — **except one sentence in §4's widget census, which did not survive
its own re-read** and is struck through and explained there rather than
deleted. The correction is more useful than the row was: it says what a
*measurement* in a document owes the reader that a *finding* does not.

---

## 1. Markup — what is left after the Style group landed

**★ The headline finding of the sweep was fixed while the sweep ran**, and it is
kept here because the *shape* of it recurs: the manifest declared
`Item::custom("colour_swatch")` at `shell/manifest/markup.rs:175` and **no
renderer ever matched the kind**, so the shell reserved the item's space, the
application declined to draw it, and the group rendered a caption over nothing.
Nothing could see it — the manifest test asserts the item is *declared* and
passed correctly, and a `Custom` item carries no command id so the reachability
check is blind to it.

Colour and width are live as of `4035b64`. The rows below are what is left.

| Tunable | Value | Defined | Surface | Belongs (per RIBBON_IA) |
|---|---|---|---|---|
| ~~Pen colour, geometric kinds~~ | `(0.85, 0.16, 0.16)` red | now `canvas/markup/pen.rs` | ✅ **Markup ▸ Style** | — |
| ~~Highlight colour~~ | `(1.0, 1.0, 0.0)` | now `canvas/markup/pen.rs` | ✅ **Markup ▸ Style**, a second swatch | — |
| ~~Underline / StrikeOut / Squiggly colour~~ | ~~`(0.85, 0.16, 0.16)`~~ | now `pen.ink` | ✅ **Markup ▸ Style** — see below | — |
| ~~Stroke width~~ | `2.0` pt | now `Pen::width_pts`, range 0.25–12 | ✅ **Markup ▸ Style** | — |
| Fill | never authored — `border` only | `canvas/markup.rs` | ⛔ **design decision** — a filled comment hides the drawing it comments on | operator call |
| Opacity | never authored at all | — | ⛔ **blocked on the engine** — `/CA` is not written | Markup ▸ Style, when it lands |
| ~~Arrow head length~~ | `HEAD_LEN_PX = 14.0` | `canvas/markup/band.rs` | ⛔ **MIS-FILED — it is the cursor, not the mark** | ~~Format ▸ Arrowheads~~ |
| ~~Arrow head angle~~ | `HEAD_ANGLE = 0.42` rad | `canvas/markup/band.rs` | ⛔ same | ~~Format ▸ Arrowheads~~ |
| ~~Ink simplification tolerance~~ | ~~0.5 pt fixed~~ | now `Pen::simplify_tolerance_pts` | ✅ **already settable — it follows the pen width**, and it was a live defect until 2026-08-18 |  — |
| Preview band alpha | 90 | `canvas/markup/band.rs:311` | **none** — the cursor again | — |
| Ellipse tessellation | 48 segments | `canvas/markup/band.rs:183` | **none** — the cursor again | — |
| Author / subject / note text | not authored at all | — | **none** | Format ▸ Note text |

### ★ Three of those rows were wrong, and in two different ways — 2026-08-18

**The two arrowhead rows asked for a setting that must not exist.** They are
screen-space **preview** constants, and `band.rs`'s own doc comment says so
before the code does:

> the head is part of the *cursor* … The committed annotation's own `/LE` head
> is drawn by the appearance stream at whatever size the engine chooses; **this
> is not a promise about that size, it is a statement about direction.**

Exposing them would let an operator tune the arrowhead on the *rubber band* and
change nothing about the arrow they author — a control whose readback is real
and whose effect is imaginary. Worse, it would break the one thing the preview
is for: the head is fixed in **screen** points precisely so it does not shrink
to nothing at 25 % zoom and stop saying which end is the head.

`Belongs: Format ▸ Arrowheads` was the mis-reading. `RIBBON_IA.md`'s Arrowheads
group means the **annotation's** `/LE` style — open, closed, diamond, none — a
document property that this shell does not author and that has nothing to do
with these two numbers. Two rows that look identical in a table were a real gap
and a category error.

The same verdict covers the **preview band alpha** and the **ellipse
tessellation** below them, for the same reason, which is why they are annotated
rather than left blank.

**The ink tolerance row was the opposite mistake: it was not a missing setting,
it was a live defect.** `SIMPLIFY_TOLERANCE_PTS` was a `const` derived from
`PEN_WIDTH_PTS`, the pen's *default* width — which was right until the pen
became an operator control on 2026-08-17 and then silently stopped being right.
At a 0.25 pt pen the fixed 0.5 pt tolerance is **four times** the stroke's
half-width, so the simplification could move the drawn centreline outside the
stroke and author a curve the operator did not draw. `ink.rs` §3.2 had written
down the rule for that exact day — *"if the pen ever becomes an operator
control, the tolerance follows it"* — and having written it down was not enough,
because a `const` cannot follow anything. It now derives from the live pen and
is asserted at both ends of the operator's range.

**What to take from all three:** a row in this file says *a value is
hard-coded*. It does **not** say the value should become a control, and it does
not say the value is even correct. Three of these rows needed three different
answers — *must not exist*, *already fixed by making it follow something else*,
and *blocked* — and the only way to tell was to read what consumes the value.

`canvas/markup.rs` named its own seam before the build: *"the seam for a real
pen control is exactly this function: give it a colour and a width from the
document's markup state and nothing else in the module changes."* That
prediction was exactly right — `spec` and `action` gained a `Pen` parameter and
nothing else in the module moved. **A doc comment that names its own seam is the
cheapest refactoring aid this project has**, and it is worth writing one when a
constant is left in place of a control.

> ### ★★ And then the same habit failed, in the file directly below that row
>
> **Corrected 2026-08-17 in `09ca9a8`.** The row above this note used to read
> *"Underline / StrikeOut / Squiggly colour — `(0.85, 0.16, 0.16)` —
> `canvas/markup/text.rs:353` — surface: **none**"*, filed alongside the others
> as a control nobody had built yet. **It was not that.** It was a *stale
> duplicate of a control that already existed*, and the difference is the whole
> finding: the operator could set the pen to blue, draw a rectangle and get
> blue, then underline a word and get red — in the build shipped by the commit
> that answered *"I can't change a markup's colour"*.
>
> `text.rs` had named its seam too, in the same style and just as clearly:
> *"there is no pen control in this shell yet … a real pen replaces exactly
> this function."* The real pen arrived in `4035b64` and **did not replace
> it**, because the person filling the seam was working in a different file and
> nothing pointed from one to the other.
>
> So the praise above needs its other half, and this is it: **a doc comment
> naming its own seam is an asset only if something checks the seam when it is
> filled.** Prose is a note to a human, and the human who fills a seam is
> usually not the one who named it. What closes the gap is a *test that asserts
> the two paths agree* — and the test that was there could not, because
> `the_pen_is_the_visible_one` asserted the literal triple against a function
> returning the literal triple. Two copies of one constant cannot disagree.
>
> The replacement, `the_ink_reaches_every_text_kind`, asserts a **relation and
> not a magnitude** — whatever the pen holds is what is authored — and was
> proved by planting the old constant and watching it fail. The rule worth
> carrying out of this file: **when you leave a constant in place of a control,
> write the test that will fail when the control arrives, not the comment that
> asks someone to notice.**

---

## 2. Zero surface — snap / grid / guides / rulers / zoom

**★ Two rows here were closed on 2026-08-18** and are struck through rather than
deleted, for the reason the markup correction above is kept: the *shape* is what
recurs. Both were **defaults**, not capabilities — the toggle and the fit command
both existed and worked — and both cost the operator something on **every
document they ever open** rather than once. That is the property worth scanning
this list for. A hard-coded value an operator meets once is a preference nobody
misses; a hard-coded value they have to correct by hand at every open is a
feature that has quietly been made their job.

| Tunable | Value | Defined | Surface |
|---|---|---|---|
| Snap tolerance | 10.0 px | `canvas/snap.rs:136` | none |
| Selection tolerance | 6.0 px | `canvas/mapping.rs:93` | none |
| Object fallback tolerance | 3.0 | `panels/objects/provider.rs:153` | none |
| Grid pitch | **no spacing variable** — ladder-derived, floor 8.0 pt | `canvas/grid.rs:73` | none |
| Grid minor / major alpha | 26 / 56 | `canvas/grid.rs:82,92` | none |
| Guide catch radius | 4.0 pt | `canvas/guides.rs:240` | none |
| Guide / discard alpha | 170 / 60 | `canvas/guides.rs:249,259` | none |
| Guides per document | 256 (store cap 200) | `canvas/guides.rs:225,211` | none |
| Ruler thickness | 22.0 pt | `canvas/rulers.rs:235` | none |
| Ruler min major pitch | 76.0 pt | `canvas/rulers.rs:250` | none |
| Ruler major / minor tick | 6.0 / 2.5 pt | `canvas/rulers.rs:263,271` | none |
| Ruler page-span alpha | 40 | `canvas/rulers.rs:280` | none |
| ~~Rulers / grid / guides **default visibility**~~ | ~~all `false`~~ | now `app::prefs::PageChrome` | ✅ **Settings ▸ Drawing the page** — 2026-08-18, one setting with three switches |
| Zoom min / max | 0.10 / 8.0 | `viewer/mod.rs:127,132` | none |
| ~~Default fit mode~~ | ~~`FitMode::Page`~~ | now `app::prefs::OpeningFit` | ✅ **Settings ▸ Drawing the page** — and it gained a third value, *actual size*, which `FitMode` alone could not express |
| Zoom-region min extent | 8.0 px | `canvas/zoom.rs:121` | none |
| Canvas fit margin | 16.0 | `canvas/mod.rs:237` | none |
| Grip size / grab slack | 8.0 / 2.0 px | `canvas/handles.rs:65,74` | none |
| Page row / spread gap | 12.0 / 6.0 | `viewer/strip.rs:98,106` | none |
| Snap marker size | 6.0 pt | `canvas/measure/mod.rs:791` | none |
| Arc preview steps | 24 | `canvas/measure/pick.rs:544` | none |

**Two partials worth separating out:**

- Ruler fallback number format is `NumberFormat::decimal(Millimeter, 2)` —
  **precision 2 is hard-coded** at `canvas/rulers.rs:503`, even though the new
  scale dialog can set a format.
- `canvas/rulers.rs:522-525` states *"the GUI has no group picker yet"*, so all
  measure work lands in the default dimension group. `measure.manage_groups` is
  still inert (§9).
- Measure scale-entry seeds are `Default`-only: real-length unit **Meter**, ratio
  basis **Inch**, ratio 1:100 — `canvas/measure/scale.rs:152-157`. The dialog
  offers a unit combo, but the starting values are not preference-backed.

---

## 3. Zero surface — render / redact / OCR / print / new document

| Tunable | Value | Defined | Surface |
|---|---|---|---|
| Texture cache budget | 48,000,000 texels | `render/strip.rs:136` | none |
| In-frame render budget | 12 ms | `render/worker.rs:166` | none |
| Thumbnail cache | 64 | `panels/pages/thumbnails.rs:243` | none |
| Thumbnail width | 140 pt (tile floor 112 pt, `panels/pages/mod.rs:170`) | `panels/pages/thumbnails.rs:178` | none |
| Thumbnail slow / ceiling | 400 ms / 2 s | `panels/pages/thumbnails.rs:202,227` | none |
| Thumbnail quality | pinned `Normal` | `panels/pages/thumbnails.rs` | none — **deliberate**, argued in the module header; do not "fix" |
| Overlay alphas: ghost / find hit / current hit / text selection | 150 / 40 / 96 / 40 | `canvas/overlay.rs:140,281,315,392` | none — **find highlight colours are unreachable** |
| Redaction fill | `None` | `panels/redact.rs` | ⛔ **blocked** — see below |
| Redaction overlay text | `None` | `panels/redact.rs` | ⛔ **blocked** — see below |
| Redaction quadding | `Left` | `panels/redact.rs` | ⛔ **blocked**, on the row above |
| Redaction min verifiable length | 4 | `redact/proof.rs:80` | none |
| OCR target pixels | 8,400,000 | `ocr/mod.rs:168` | none |
| OCR DPI ceiling / floor | 300 / 50 | `ocr/mod.rs:177,185` | none |
| OCR language | none — single `ocrs` model | `ocr/mod.rs:193` | none |
| Print preview zoom min / max / step | 0.25 / 40 / 1.25 | `dialogs/print/preview.rs:158-163` | none |
| Print preview DPI / max side | 150 / 2200 px | `dialogs/print/preview.rs:133,151` | none |
| Default paper | US Letter portrait 612×792 | `dialogs/print/mod.rs:794` | none |
| **New blank page size** | A4, 595.276 × 841.89, baked-in template | `app/blank.rs` (`TEMPLATE` is `include_bytes!`) | ⛔ **blocked on the engine, 2026-08-18** — nothing in `pdfce-core` writes a `/MediaBox`, so the only shell-side implementation is one asset per size (ten, with landscape) and a custom size is impossible at any count. Filed as `request_no_verb_sets_a_pages_media_box.md`; **neither implementation built** |

### ★ The three redaction rows, resolved 2026-08-18 — and the answer was *do not build it*

The sweep listed them as ordinary gaps: three `None`s against an engine whose
`RedactSpec` has all three fields, documents all three, and **writes two of
them into the PDF**. Following each value to the code that consumes it gave a
different answer.

| field | what actually happens |
|---|---|
| `fill` | Honoured — `/IC` is written (`annot_author.rs:942`), read (`annot_fill`, `redact.rs:1373`) and painted (`build_overlay`, `redact.rs:1026`). **But `EditSession::author_text_matches` hard-codes `fill: None` at `edit.rs:11719`**, so no caller can set it on a mark made by *Find and mark*. A swatch would work on whole-page marks and be silently dropped on searched ones |
| `overlay_text` | **Written and never read.** `gather_page` (`redact.rs:1259`) does not look at `/OverlayText`, `build_overlay` draws filled boxes only, and the annotation carrying the string is **deleted** at `redact.rs:1167`. Type *REDACTED*, apply, get plain black boxes, no report row |
| `quadding` | `/Q` justifies the overlay text. Nothing to justify until it burns in |

Both are filed as separate requests (`request_redaction_fill_is_unreachable_from_the_search_path.md`,
`request_overlay_text_is_recorded_and_then_dropped.md`) and the reasoning is
duplicated into `panels/redact.rs`'s `whole_page_spec`, because the request
folder is explicitly **not** a durable record.

**The lesson generalises past redaction and is the one to carry:** an engine
field that exists, is documented, and is *written into the file* is not
evidence that anything **reads** it. Two of these three reach the PDF today.
Distinguishing *supported* from *accepted and discarded* takes following the
value to its consumer — the same shape as `HANDOFF.md` §10's *"registration is
not implementation"*, one layer down, and it is why a "no surface" row is not
automatically a "build the surface" task.

**A surface on an irreversible operation raises the bar further.** A partly
honoured setting is normally a papercut; on redaction it is an operator
believing the wrong thing about content that no longer exists.

---

## 4. Zero surface — persistence / panels / shell chrome

| Tunable | Value | Defined | Surface |
|---|---|---|---|
| Recent-file cap | 10 | `app/recent.rs:133` | none |
| Recent presence TTL | 2 s | `app/recent.rs:140` | none |
| Layout autosave settle / max defer | 750 ms / 5 s | `app/persistence.rs:155,164` | none |
| Remembered per-document entries | 200 | `viewer/remembered.rs:134` | none |
| Navigator / inspector default width | 280 / 320 | `app/modes/defaults.rs:253,260` | none |
| Window initial / min size | 1100×800 / 640×480 | `lib.rs:107,114` | none |
| Icon size | 16.0 pt | `icons/mod.rs:171` | ✅ **scales with the UI now** — `Settings ▸ Appearance ▸ Size of pdfce's own menus, buttons and text`, 2026-08-18. The 16 pt is still a constant and is now a size *in points*, which `pixels_per_point` multiplies; ~~no UI-scale or base-font-size control anywhere~~ |
| Icon cache | 512 | `icons/cache.rs:92` | none |
| Status bar height / row | 30.0 / 24.0 pt | `app/status.rs:378,386` | none |
| Object-tree point rows per part | 200 | `panels/objects/mod.rs:197` | none |
| Form editor text ratio / range | 0.62 / 9–22 pt | `canvas/forms/boxes.rs:67,74` | none |
| Max traced form boxes | 64 | `canvas/forms.rs:539` | none |
| Glyph ascent / descent | 0.85 / 0.22 | `canvas/textsel.rs:370,374` | none |

Panels surface almost nothing: only the Pages panel's **previews** checkbox
(`panels/pages/mod.rs:238`) and the Forms rows' field editors.

**★ Crate-wide widget census, re-run 2026-08-17 at `2275ee0`.** Every file in
`crates/pdfce-gui/src/` containing a value-editing widget, by count of
occurrences:

| file | what it edits |
|---|---|
| `panels/forms/rows.rs` (7) | form field values, in the panel |
| `dialogs/print/tabs.rs` (6) · `dialogs/print/mod.rs` (1) | the print job |
| `text/redact.rs` (4) | the redact panel's search field and its two confirm checkboxes |
| `find/bar.rs` (4) | the find query and its match options |
| `dialogs/scale.rs` (4) | the measure scale |
| `canvas/forms.rs` (4) · `canvas/tool.rs` (2) · `canvas/keys.rs` (2) · `canvas/textsel/clipboard.rs` (2) | form fields and text **on the canvas** |
| `dialogs/redact.rs` (3) | the two apply-gate checkboxes |
| `dialogs/settings/{text,measuring,display}.rs` (1 each) | the settings window |
| `panels/pages/mod.rs` (1) | the previews checkbox |
| `canvas/markup/swatch.rs` (1) | **the pen width** — the two colour swatches beside it are `color_edit_button_srgba` and are not counted by the pattern above |
| `app/status/page_box.rs` (1) | the page-number box in the status bar |

> **★ Corrected 2026-08-17, after this file was committed.** The sentence that
> stood here — *"`color_edit_button` has zero hits in the entire crate. There
> is no colour picker in pdfce at all"* — was **true when the sweep began at
> `f794e27` and false by the time the sweep was filed.** `4035b64` landed two
> `Ui::color_edit_button_srgba` calls in `canvas/markup/swatch.rs:96,106`, and
> this file's own *Status* section above says that commit was accounted for.
> It was accounted for in §1, where the finding was; it was not carried into
> the §4 census, which is a different claim about the same commit.
>
> **This is the fifth time in this project a number in prose has drifted from
> the code it describes**, and it is the shape `HANDOFF.md` §1 already warns
> about in its own superseded table. The generalisable part is narrow and
> worth stating: **a census is a measurement with a timestamp, and a
> measurement taken across a moving tree must be re-run at filing time, not
> reconciled section by section.** Reconciling §1 and forgetting §4 is not
> carelessness — it is what happens when a correction is applied where the
> *finding* was rather than everywhere the *measurement* was used.
>
> The census is now a one-line command, so the next reader re-runs it rather
> than trusting this paragraph:
>
> ```bash
> grep -rn "color_edit_button\|DragValue\|Slider\|TextEdit\|checkbox\|ComboBox" \
>   crates/pdfce-gui/src/ --include=*.rs | grep -v "mod tests"
> ```

So pdfce **does** have a colour picker, in exactly one place: the two markup
swatches. It has no colour picker for redaction fill (§3), for text-markup
colour (§1), or for anything in the settings window. The idiom for the next one
is set — `color_edit_button_srgba` against an `egui::Color32`, converted at the
seam — and `tools/ui-verify/src/checks/markup_style.rs`'s header records the
one thing that idiom costs a harness: **the picker's popup publishes no
regions, so a check can assert the swatch was drawn and driven but cannot aim
at a hue inside it.** Any new colour control inherits that limit, and should be
paired with a driveable non-popup control (as the Style group pairs its
swatches with a width) so the group's check has something it can actually
move.

---

## 5. Registered commands with no dispatch arm

Source of truth: `shell/commands/reach/register.rs` (the `SCAFFOLDED` list).
Counts pinned by `the_p3_tension_is_counted` at
`shell/commands/reach.rs:1077-1086`. `UNREACHED_ARMS` was **empty**
(`reach.rs:1095`) — no dead arms in the dispatcher.

**22 entries, of which 8 carry a ★ P3 mark** — the list was 31/8 when the sweep
began. The count is not restated here: `shell/commands/reach.rs`'s header quotes
it, a test pins it, and a third copy in a document nothing checks is how a
number that has already drifted four times in this project drifts a fifth. Read
the register.

| # | Command id | ★P3 | Line | Recorded reason (condensed) |
|---|---|---|---|---|
| 1 | `file.export_dxf` | ★ | :81 | **No recorded reason anywhere.** Scaffolded by omission, not by decision |
| 2 | `file.export_form_data` | | :90 | Blocked on an FDF/XFDF/CSV writer that does not exist |
| 3 | `file.shortcuts` | | :97 | Blocked on salvaging `ui_text.rs`; salvaging it unfixed imports `DEFECTS.md` D5 |
| 4 | `view.show_points` | ★ | :145 | **There is nothing for it to show** — this build draws no anchor marks at any rung |
| 5 | `view.sidebar` | ★ | :164 | Only justification on record is provably stale — there is no sidebar rail, there is a dock |
| 6 | `pages.split` | ★ | :176 | Needs a boundary chooser; *"there is no honest default"* |
| 7 | `pages.merge_into` | ★ | :185 | `insert` returns a **new** document; wiring it discards the undo command log |
| 8 | `pages.insert_from_file` | ★ | :193 | Twin of the above, same two blockers |
| 9 | `edit.objects` | ★ | :203 | **No recorded reason anywhere** |
| 10 | `edit.insert_image` | ★ | :211 | **No recorded reason** for the missing arm |
| 11 | `edit.form_create_field` | | :219 | Core's **structural** certification gate, not the fill gate |
| 12 | `edit.form_manage_fields` | | :226 | Same structural gate; its dialog does not exist |
| 13 | `edit.form_flatten` | | :232 | Same gate, plus irreversible — needs a disclosure surface |
| 14 | `markup.text_box` | | :267 | Text-bearing, not geometric: needs place-then-type + `TextAnnotSpec` |
| 15 | `markup.sticky_note` | | :275 | Same row of `canvas::markup`'s table |
| 16 | `markup.stamp` | | :281 | Same, plus **needs a gallery** — *"a stamp with no chooser has no operand"* |
| 17 | `measure.manage_groups` | | :290 | Needs a window, not an arm; must not become a picking tool |
| 18 | `tools.merge_files` | | :309 | Batch pane unsalvaged (`SALVAGE.md`, ~700 lines) |
| 19 | `tools.split_files` | | :317 | Same pane, plus inherits the missing boundary chooser |
| 20 | `tools.font_folders` | | :323 | Same pane; a directory list needs the pane it lives in |
| 21 | `tools.embed_fonts` | | :329 | **Reason expired** — the mutation funnel and undo log landed 2026-08-14. Now closer to unwritten than blocked |
| 22 | `tools.unembed_fonts` | | :338 | Sibling, plus a live reason: three of four consequences are invisible on canvas, needs a confirmation surface |

**Closed during this session** (present at `f794e27`, gone by `980971f`):
`file.settings`, `measure.set_scale`, and all seven
`view.render_*` / `view.floating_panels` / `view.app_initiative` entries.

---
