# CONTINUE — session handoff, 2026-08-17

**Why this file exists:** the operator's mouse became unusable mid-task and the
session was restarted. Type `continue` in a new session and point the agent at
this file. It is written to be self-sufficient: everything below was verified by
reading source, and every claim carries a `file:line` so nothing has to be
re-derived on trust.

**Status at pause:** the inventory task described in §2 is **substantially
complete** (§4–§9 are the deliverable). What remains is re-verification against a
tree that moved three times during the analysis (§3), and the four parallel
sweeps that were still in flight when the session ended (§10).

---

## 1. First — the mouse

The operator restarted because the mouse became unusable. **This session never
diagnosed that and has no theory about it.** A suspicion about "two things
interfering with the mouse" was raised by the operator as something a *previous*
context had said; the agent writing this file did not say it and could not
identify what was meant.

For the record, the processes visible at 14:50 on 2026-08-17 were:

| PID | Process | Started | CPU |
|---|---|---|---|
| 31056 | `cargo` | 14:49 | 0.52 |
| 38600 | `cargo` | 14:49 | 0.03 |
| 40904 | `cargo-clippy` | 14:49 | 0.03 |
| 32576 | `python` | 06:36 | 7.14 |
| 27776 | `pythonw` | 08:15 | 6.77 |

The two long-lived Python processes (06:36 and 08:15) are the only plausible
"two things". **Nothing was killed** — killing processes on a guess was outside a
read-only remit, and the restart clears them anyway. No `pdfce-gui.exe` was
running, so the application itself was not grabbing the pointer.

**If the mouse is still bad after the restart**, that rules out anything this
session started and the two Python processes are the first thing to look at.

---

## 2. The task, as given

> READ-ONLY analysis in `D:\Dev\pdfceGUI\crates\pdfce-gui\src\`. Build an
> exhaustive inventory of every tunable/parameter the operator would plausibly
> want to change which currently has **no UI surface at all** — a hard-coded
> constant, a `Default` impl, or a field only settable from code. For each:
> feature area, tunable, current value, `file:line`, whether it is changeable
> from the UI, and where a surface would belong per `RIBBON_IA.md`. Mark
> partial surfaces distinctly from zero surfaces. Separately list every
> registered command with no dispatch arm.

Driving complaint, in the operator's words: *"I tried a lot of the features that
have been added only to find there is no surface for changing or editing the
settings for them."*

**Nothing was edited.** The analysis was read-only throughout. This file is the
only thing this session wrote.

---

## 3. ★ The tree moved three times during the analysis — re-verify before acting

This is the most important thing on the page. `git status` was **clean at
`f794e27`** when the task began. By the time it paused:

| Checkpoint | HEAD | Note |
|---|---|---|
| Task start | `f794e27` | "Phase 5: aligned and rotated text stop moving the wrong way" |
| Mid-analysis | `980971f` | "Set scale: the dimensions were measuring the paper" — plus 22 modified, 2 untracked |
| At pause | `29cdc31` | "The seven view settings: two built, five deleted" |

At pause, working tree held only:

```
 M FEATURES.md
 M crates/pdfce-gui/src/shell/manifest/format.rs
```

No stashes. **Nothing is at risk from the restart** — the settings work was
committed before the pause.

Another session was building *exactly this brief* while the inventory was being
taken. `dialogs/settings/mod.rs:20-25` states the motive in the operator's own
words: *"the operator's report was that features had been added with 'no surface
for changing or editing the settings for them.'"*

**Consequence: treat every count and every line number below as of ~14:50 on
2026-08-17 and re-verify before planning work on it.** Re-derive the moving
numbers with:

```bash
cd /d/Dev/pdfceGUI
git log --oneline -5
grep -n "total, \|p3, " crates/pdfce-gui/src/shell/commands/reach.rs
awk '/^        "[a-z_]+\.[a-z_]+",$/{print NR": "$0}' \
    crates/pdfce-gui/src/shell/commands/reach/register.rs
```

---

## 4. What GAINED a surface on 2026-08-17 (do not re-plan these)

| Tunable | Value | Defined | Surface |
|---|---|---|---|
| Theme preset | Quiet / Airy / Dark, default Quiet | `egui-shell/src/theme/mod.rs:277`; installed `app/mod.rs:764` | **Settings ▸ Appearance** (`dialogs/settings/appearance.rs`) — closes `DEFECTS.md` D10 |
| Render quality | Faster 0.75× / Normal 1.0× / Sharper 1.5× | `app/prefs.rs:119-127` | **Settings ▸ Display** (`dialogs/settings/display.rs:67`) |
| Zoom settle | 150 ms default, range 20–1000 | `app/prefs.rs:163-178`; read at `render/settle.rs` `zoom_settle()` | **Settings ▸ Display** slider (`display.rs:108`) |
| Dimension scale, unit, fraction mode | — | `dialogs/scale.rs`; units `dialogs/scale.rs:344-350` | **Measure ▸ Scale ▸ Set scale** |
| 13 spec-ambiguity settings (CMYK intent, mask resampling, word gap, parallel tolerance, separations, line endings, …) | — | `dialogs/settings/{colour,images,text,measuring,pages,saving}.rs` | **Settings window** |
| Find: case / whole-word / wildcards / word boundary | `false / false / false / Alphanumeric` | `find/mod.rs:286-291` | **Find bar** (`find/bar.rs:774-790`) — fully surfaced |
| Print: range, copies, scale %, collate, reverse, tray, printer, max DPI | — | `dialogs/print/tabs.rs:187-390` | **Print dialog** |

**Also deleted rather than surfaced** (`dialogs/settings/display.rs:17-33`):
`view.render_strategy`, `view.render_thin_lines`, `view.render_antialias`,
`view.floating_panels`, `view.app_initiative`. Five of the seven `DIRECTED`
render controls had **nothing behind them in the engine** — `RenderOptions` has
no thin-lines or antialias field, the dock has no floating mode, and nothing in
the build opens a surface unasked. The rows were removed from the list rather
than wired, which is what `DIRECTED`'s own doc comment prescribed.

New home for preferences that are *not* answers to a silent standard:
`app/prefs.rs` → `userdata/preferences.txt`, beside the engine's `settings.txt`.
**Any new operator preference should go here**, not into `pdfce_core::settings`.

---

## 5. Zero surface — markup (the largest remaining gap)

**★ Headline finding: Markup ▸ Style renders an empty captioned band.**

The manifest declares `Item::custom("colour_swatch")` at
`shell/manifest/markup.rs:175`, but `app/mod.rs:478` records that an unknown
custom kind *"draws **nothing** and returns `None`, which is why the manifest's
unbuilt `colour_swatch` leaves a gap rather than a mystery widget."*

So `RIBBON_IA.md` §5.5's "**partial G** — colour only" is **not true of this
build**. Colour has no surface either. Markup style is a **zero**, not a partial.

| Tunable | Value | Defined | Surface | Belongs (per RIBBON_IA) |
|---|---|---|---|---|
| Pen colour, geometric kinds (Rect / Ellipse / Arrow / PolyLine / Polygon / Ink) | `(0.85, 0.16, 0.16)` red | `canvas/markup.rs:417-431` (`fn rgb`) | **none** | Markup ▸ Style + Format tab |
| Highlight colour | `(1.0, 1.0, 0.0)` | `canvas/markup.rs:429` | **none** | same |
| Underline / StrikeOut / Squiggly colour | `(0.85, 0.16, 0.16)` | `canvas/markup/text.rs:353` | **none** | same |
| Stroke width, **every kind** | `PEN_WIDTH_PTS = 2.0` pt | `canvas/markup.rs:444` | **none** — `markup.line_width` PLANNED at `shell/manifest/mod.rs:745` | Markup ▸ Style |
| Fill | never authored — `border` only | `canvas/markup.rs:640-687` | **none** — `markup.fill` PLANNED at `manifest/mod.rs:751` | Markup ▸ Style |
| Opacity | never authored at all | — | **none** — `markup.opacity` PLANNED at `manifest/mod.rs:753` | Markup ▸ Style |
| Arrow head length | `HEAD_LEN_PX = 14.0` | `canvas/markup/band.rs:193` | **none** | Format ▸ Arrowheads |
| Arrow head angle | `HEAD_ANGLE = 0.42` rad | `canvas/markup/band.rs:195` | **none** | Format ▸ Arrowheads |
| Ink simplification tolerance | 0.5 pt (`PEN_WIDTH_PTS / 4`) | `canvas/markup/ink.rs:214` | **none** | Settings ▸ (new) Markup |
| Preview band alpha | 90 | `canvas/markup/band.rs:311` | **none** | — |
| Ellipse tessellation | 48 segments | `canvas/markup/band.rs:183` | **none** | — |
| Author / subject / note text | not authored at all | — | **none** | Format ▸ Note text |

`canvas/markup.rs:394-402` names the seam in its own words: *"the pen is a
**default**… the seam for a real pen control is exactly this function: give it a
colour and a width from the document's markup state and nothing else in the
module changes."* **One function signature is the whole build.**

---

## 6. Zero surface — snap / grid / guides / rulers / zoom

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
| Rulers / grid / guides **default visibility** | all `false` | `viewer/mod.rs:297-306` | toggles exist (View ▸ Display); **the default is not settable** |
| Zoom min / max | 0.10 / 8.0 | `viewer/mod.rs:127,132` | none |
| Default fit mode | `FitMode::Page` | `viewer/mod.rs:299` | fit commands exist; **default not settable** |
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

## 7. Zero surface — render / redact / OCR / print / new document

| Tunable | Value | Defined | Surface |
|---|---|---|---|
| Texture cache budget | 48,000,000 texels | `render/strip.rs:136` | none |
| In-frame render budget | 12 ms | `render/worker.rs:166` | none |
| Thumbnail cache | 64 | `panels/pages/thumbnails.rs:243` | none |
| Thumbnail width | 140 pt (tile floor 112 pt, `panels/pages/mod.rs:170`) | `panels/pages/thumbnails.rs:178` | none |
| Thumbnail slow / ceiling | 400 ms / 2 s | `panels/pages/thumbnails.rs:202,227` | none |
| Thumbnail quality | pinned `Normal` | `panels/pages/thumbnails.rs` | none — **deliberate**, argued in the module header; do not "fix" |
| Overlay alphas: ghost / find hit / current hit / text selection | 150 / 40 / 96 / 40 | `canvas/overlay.rs:140,281,315,392` | none — **find highlight colours are unreachable** |
| Redaction fill | `None` | `panels/redact.rs:418` | none |
| Redaction overlay text | `None` | `panels/redact.rs:419` | none |
| Redaction quadding | `Left` | `panels/redact.rs:420` | none |
| Redaction min verifiable length | 4 | `redact/proof.rs:80` | none |
| OCR target pixels | 8,400,000 | `ocr/mod.rs:168` | none |
| OCR DPI ceiling / floor | 300 / 50 | `ocr/mod.rs:177,185` | none |
| OCR language | none — single `ocrs` model | `ocr/mod.rs:193` | none |
| Print preview zoom min / max / step | 0.25 / 40 / 1.25 | `dialogs/print/preview.rs:158-163` | none |
| Print preview DPI / max side | 150 / 2200 px | `dialogs/print/preview.rs:133,151` | none |
| Default paper | US Letter portrait 612×792 | `dialogs/print/mod.rs:794` | none |
| **New blank page size** | A4, 595.276 × 841.89, baked-in template | `app/blank.rs:172-175` (`TEMPLATE` is `include_bytes!`) | **none** — `file.new_from_template` PLANNED at `manifest/mod.rs:468` |

---

## 8. Zero surface — persistence / panels / shell chrome

| Tunable | Value | Defined | Surface |
|---|---|---|---|
| Recent-file cap | 10 | `app/recent.rs:133` | none |
| Recent presence TTL | 2 s | `app/recent.rs:140` | none |
| Layout autosave settle / max defer | 750 ms / 5 s | `app/persistence.rs:155,164` | none |
| Remembered per-document entries | 200 | `viewer/remembered.rs:134` | none |
| Navigator / inspector default width | 280 / 320 | `app/modes/defaults.rs:253,260` | none |
| Window initial / min size | 1100×800 / 640×480 | `lib.rs:107,114` | none |
| Icon size | 16.0 pt | `icons/mod.rs:171` | none — **no UI-scale or base-font-size control anywhere** |
| Icon cache | 512 | `icons/cache.rs:92` | none |
| Status bar height / row | 30.0 / 24.0 pt | `app/status.rs:378,386` | none |
| Object-tree point rows per part | 200 | `panels/objects/mod.rs:197` | none |
| Form editor text ratio / range | 0.62 / 9–22 pt | `canvas/forms/boxes.rs:67,74` | none |
| Max traced form boxes | 64 | `canvas/forms.rs:539` | none |
| Glyph ascent / descent | 0.85 / 0.22 | `canvas/textsel.rs:370,374` | none |

Panels surface almost nothing: only the Pages panel's **previews** checkbox
(`panels/pages/mod.rs:238`) and the Forms rows' field editors.

**★ Crate-wide widget census.** The only value-editing widgets that exist
anywhere are in the print dialog, the redact dialog, the find bar, the forms
rows, the Pages previews checkbox, and (as of 2026-08-17) the settings window.
**`color_edit_button` has zero hits in the entire crate.** There is no colour
picker in pdfce at all.

---

## 9. Registered commands with no dispatch arm

Source of truth: `shell/commands/reach/register.rs` (the `SCAFFOLDED` list).
Counts pinned by `the_p3_tension_is_counted` at
`shell/commands/reach.rs:1077-1086`. `UNREACHED_ARMS` was **empty**
(`reach.rs:1095`) — no dead arms in the dispatcher.

**At ~14:40 the list was 22, of which 8 carried a ★ P3 mark.** It was 31/8 at
task start. `29cdc31` landed after this was read, so **re-verify the count
first** (command in §3).

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

## 10. What was still in flight when the session ended

Four parallel read-only sweeps had been dispatched and **had not reported** when
the session paused. They died with the session. Their assignments were:

1. **Markup tunables** — `canvas/markup*`, per-kind defaults, partial-vs-zero classification.
2. **Measure / snap / grid / guides / rulers / zoom** — `canvas/measure*`, `snap.rs`, `grid.rs`, `guides.rs`, `rulers.rs`, `zoom.rs`.
3. **Render / redact / OCR / print** — `render/*`, `redact/*`, `ocr/*`, `dialogs/print/*`, `dialogs/redact.rs`, `app/blank.rs`.
4. **Panels / persistence / find / egui-shell theme** — `panels/*`, `app/persistence.rs`, `app/recent.rs`, `viewer/remembered.rs`, `find/*`, `crates/egui-shell/src/`.

**They do not need re-running.** Everything in §4–§9 was independently verified
by direct reads with `file:line` citations before the pause; the sweeps would
only have added second-order detail. Re-dispatch only if a specific area needs
more depth than the tables above.

---

## 11. Recommended order of work

1. **`colour_swatch`** — Markup ▸ Style currently draws an empty caption. This is
   the most visible instance of the operator's complaint, and `canvas/markup.rs:394`
   says the change is one function signature.
2. **Markup line width + fill + opacity** — all three are PLANNED ids with engine
   support already present. Only the controls are missing.
3. **Snap tolerance, ruler/grid/guides default visibility, zoom min/max,
   recent-file cap** — trivially preference-shaped, and `app/prefs.rs` now exists
   as the correct home for exactly this class of value.
4. **Redaction fill / overlay text / quadding** — three `None`s at
   `panels/redact.rs:418-420`. The engine takes them; nothing offers them.
5. **New-document page size** — `file.new` always produces an A4 from a baked-in
   template with no chooser.
6. **`edit.objects` and `edit.insert_image`** — the two ★P3 commands whose reason
   for being inert is *nothing at all*, which `reach.rs` flags as the worst
   category on the list.

---

## 12. Resuming

Say `continue` and hand the agent this file. Suggested opening move:

```bash
cd /d/Dev/pdfceGUI && git log --oneline -5 && git status --short
grep -n "total, \|p3, " crates/pdfce-gui/src/shell/commands/reach.rs
```

Reconcile §3's checkpoint table and §9's count against what that prints, then
pick up at §11. The inventory itself does not need redoing.

**Standing constraint from the original task:** it was a READ-ONLY analysis.
If the new session is meant to *implement* rather than continue analysing, that
is a change of remit and should be confirmed with the operator first.
