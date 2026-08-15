# Salvage inventory — what carries over from the old GUI

**Source:** `D:\Dev\pdfce\crates\pdfce-gui\src\`, measured 2026-08-12.
**Total:** 49,837 code lines + 12,273 test lines across 21 files.

This is not a from-scratch rewrite. **Roughly 45 % of the code comes
across with little or no change**, and most of what is rebuilt is one
file. The headline problem is `main.rs` at 25,005 code lines — half the
crate — and the rebuild is mostly about giving its contents somewhere
better to live.

---

## Summary

| Class | Code lines | Share | Meaning |
|---|---:|---:|---|
| **A — Lift** | 9,895 | 20 % | Comes across nearly as-is. Engine-facing, tested, correct. |
| **B — Lift and rework** | 12,507 | 25 % | Good bones, needs adapting to the new IA or new structure. |
| **C — Restructure** | 25,005 | 50 % | `main.rs`. Most of its *content* moves; its *shape* does not survive. |
| **D — Rebuild** | 2,430 | 5 % | Ribbon and dock — superseded by `RIBBON_IA.md`. |

Tests follow their subjects. The 12,273 test lines are salvaged with the
code they cover, and are a floor rather than a ceiling — see rule **R1**
in the agent charter.

---

## Class A — Lift nearly as-is

These are the parts of the old GUI that are **good**, and several are
things no competing product has. They come across with their doc
comments and tests intact. "Nearly as-is" still means each gets a
read-through and a `ui-verify` assertion before it is trusted.

| File | Code | Tests | Why it survives | Change needed |
|---|---:|---:|---|---|
| `print_flow.rs` | 1,854 | 168 | Three-tab print dialog with a zoomable live preview of real page content. Self-contained, works, nothing like it needs re-deriving. | Add imposition once `pdfce-print` shares the sheet composition (a **C**-row in `FEATURES.md`). |
| `icons.rs` | 1,747 | 383 | SVG path data rasterized at physical pixel size rather than pre-baked PNGs. Mostly data. | New icons for the new commands. |
| `measure_tool.rs` | 1,230 | 814 | Dimension **groups** with shared scale and drafting standard — better than the comparison product has. Taubin best-fit circle. Snapping. **`TwoLinePick` (`:361`) is here, built and tested** — see the note below. | Add Area, Angular. **Carry the Two-line gesture across**; it does not need wiring, it needs salvaging. |
| `diag.rs` | 819 | 144 | The `PDFCE_DIAG` key=value channel. Off by default, one atomic load, never load-bearing. This is what made the Delete-key and render analyses possible. | Extend with page complexity per `BENCHMARK.md` §"instrument before optimising". |
| `settings_panel.rs` | 800 | 114 | The spec-ambiguity settings model — each row states what the standard leaves open and how well-founded the default is. A genuine differentiator. | Add the **Render** group (strategy, raster scale, settle delay, thin lines, antialias). Fix the heading contrast — `DEFECTS.md` D2. |
| `object_provider.rs` | 694 | 313 | Front-to-back page object decomposition. Feeds the Objects panel, which is the single strongest thing pdfce has. | Serve more than the current page, for continuous mode (Phase 4). |
| `object_summary.rs` | 520 | 276 | Per-object descriptions — type, text, font, colour, width, node count, winding. | Row text must not clip; the old panel truncated with no horizontal scroll. |
| `viewer.rs` | 509 | 413 | Zoom ladder with provable reversibility, fit modes re-derived per frame, per-page raster ceiling accounting for `pixels_per_point`. Well tested. | Cursor-anchored zoom (Phase 3.1); page *range* not `page_index` (Phase 4.1). |
| `render_worker.rs` | 466 | 116 | Generation counter + between-operator cancellation. **Measured**: six rapid zoom steps start six generations and complete one. Do not touch the design. | Add a thread pool for thumbnails and adjacent-page prerender (`BENCHMARK.md`). |
| `theme.rs` | 464 | 137 | Palette/preset model, chrome-vs-document colour separation enforced by CI. Sound design. | **Fix `widgets.active.bg_fill`** and add a rendered-pair contrast test — `DEFECTS.md` D2. |
| `redact_apply.rs` | 429 | 280 | Runtime-verified true-removal proof, forced full rewrite. **★ This file is currently the ONLY place the proof exists** — see below. | Canvas drag-to-mark (currently panel-driven only). |
| `raster.rs` | 363 | 0 | Premultiplied alpha handled correctly; stale texture scaled `LINEAR` during settle. This is *why* zoom feels smooth. | None. |

**Subtotal: 9,895 code lines, 3,158 test lines.**

### ★ Correction, 2026-08-14 — the Two-line gesture is *built*, not pending

The `measure_tool.rs` row used to say *"wire the Two-line gesture whose core
is already done"*, and four other documents said the same thing in stronger
words: **"the canvas gesture has no caller"** (`FEATURES.md`, `HANDOFF.md`,
`RIBBON_IA.md` §5.6 twice, and `shell/manifest/mod.rs`'s `PLANNED` entry for
`measure.two_line`).

**It was false, and it was false when written.** In the old shell:

| what | where |
|---|---|
| the `pick_line_in_page` call | `main.rs:23564` |
| the pick itself | `main.rs:23592` — `st.two_lines.offer_line(h, parallel_epsilon)` |
| hover highlight before any click | `main.rs:23574-23587` |
| picked-pair overlay, verdict disclosure, Escape to clear | `:23597-23604`, `:23175-23187`, `:23857` |
| the state type | `measure_tool.rs:361` `TwoLinePick`, tests at `:1717-2040` |

pdfce's own `docs/FEATURES.md:104` marks that row **`gui [x]`**; the `[ ]` in
it is the *Acrobat* column. The gesture landed in their commit `c4ec3f5`,
2026-08-12 — the same day this file's survey was taken, which is the most
likely reason it was missed. The probable textual origin is a misread of
pdfce's `ROADMAP.md:2778`, which explains why `pick_line_in_page` exists, one
paragraph above the commit heading that added its caller.

**What it changes.** The missing caller is *ours*, not theirs. This shell has
no measure tool at all: `canvas/tool.rs` has two `CanvasTool` variants, no
`measure.*` command has a dispatch arm, and `crates/pdfce-gui` contains zero
occurrences of `linepick`, `PickedLine` or `author_from_two_lines`. So the
work is this row — carry `measure_tool.rs` across — plus the ~900 lines of
Class C canvas hosting at `main.rs:23100-23900`. The *"cheapest real feature
in the backlog"* claim that rode on the false premise is withdrawn.

**The lesson is the same one the `deletion_refusal` filing taught**, pointed
the other way: that time a claim about *their* code was wrong because it was
checked against the wrong function; this time a claim about *our own*
backlog was wrong because it was never checked at all, and it then travelled
into four more documents by being quoted. **A status word in a table is a
claim, and it decays.**

---

### ★ `redact_apply.rs` is load-bearing in a way the table understates

Flagged by the core team in the request channel, 2026-08-13:

> **`Pass 72.0` — the redaction true-removal proof is not in
> `pdfce-core`.** It lives in `crates/pdfce-gui/src/redact_apply.rs:269`,
> i.e. in the shell being replaced. **A shell calling
> `redact::apply_redactions` directly and writing the bytes ships an
> unverified redaction and will not know.** … `pdfce-cli`'s
> `redact-apply` does exactly that at HEAD and exits `SUCCESS` on a file
> it never verified. **Do not build a redaction UI against core's
> current surface**; wait for the verdict type to land in core.

Two consequences for this project:

1. **Salvaging this file is not optional and not merely convenient** —
   deleting it, or reimplementing redaction against core's current API,
   would ship an unverified redaction. It comes across whole, with its
   proof intact, and the proof is re-verified by a test before the
   redaction UI is reachable.
2. **When core lands the verdict type, this becomes a deletion**, not a
   parallel implementation. Two proofs that can disagree is worse than
   one in the wrong crate. Watch the channel for the `note_` that says
   Pass 72.0 closed, and file the migration as its own task rather than
   keeping both.

---

## Class B — Lift and rework

Good material whose hosting or structure changes.

| File | Code | Tests | Disposition |
|---|---:|---:|---|
| `ui_text.rs` | 7,912 | 3,913 | **The string catalog — 1,193 `pub fn` entries.** A large asset and the reason pdfce's copy is as good as it is. Most strings survive verbatim; ribbon/tab/panel labels change with the IA. **Fix `shortcuts_reference()` — it omits six live bindings (`DEFECTS.md` D5) — and derive it from the keyboard map so it cannot drift again.** Split into modules by area; at 7,912 lines it breaks R2. |
| `canvas.rs` | 1,893 | 1,244 | The `CanvasTool` enum, dispatch, and the escape ladder are sound concepts. The *selection layer* is where Phase 1 lands — handles, context menus, `/Rect` move-and-resize — so this becomes several modules under `canvas/`. |
| `panels_structure.rs` | 1,807 | 0 | Bookmarks, Layers, Signatures, Fonts panel bodies. The bodies keep; the hosting changes, and Fonts moves to **File ▸ Document** per the IA. Note this file ships **zero tests** and three of its panels shipped with no operator-reachable control at all. |
| `canvas_overlay.rs` | 749 | 0 | Overlay drawing — theme-invariant by design because the page beneath is white regardless of chrome. Mostly keeps; grows for selection handles and marquee. |
| `vector_edit_tool.rs` | 146 | 91 | Node/handle editing. Keeps. Carries a measured hot spot at 6,681 anchors in one path object — re-check as selection gets richer. |

**Subtotal: 12,507 code lines, 5,248 test lines.**

---

## Class C — Restructure: `main.rs`

**25,005 code lines + 3,579 test lines.** Half the crate.

The *architecture inside it is good* and must be preserved:

- **Actions, not mutations.** No code path runs from a widget to a
  `Document`; everything is an `Action` applied after the frame draws.
  This is why the undo log is coherent, and it is the single best
  structural decision in the old GUI. **Keep it exactly.**
- **One `EditSession` command log**, 44 `CommandKind` variants, depth
  bounded at 256, undo tooltips naming the specific operation.
- **The five-rung Escape ladder** with documented precedence.
- **Fixed-height status and find panels**, because content-driven
  heights re-fit the page on every click — a measured defect, already
  solved.

What does not survive is the *file*. Its contents redistribute roughly:

| Content | Approx. lines | Goes to |
|---|---:|---|
| Form filling, field authoring, FDF/XFDF/CSV | ~1,600 | `panels/forms/` |
| Text editing — runs, caret, formatting, reflow host | ~3,500 | `tools/text/` |
| Vector object editing, node/handle | ~1,200 | `tools/vector/` |
| Measure/dimension hosting | ~900 | `tools/measure/` |
| Page ops, thumbnail rail, selection action bar | ~1,400 | `panels/pages/` |
| Batch pane — merge, split, insert, font folders | ~700 | `panels/batch/` |
| Redaction hosting | ~600 | `panels/redact/` |
| Object tree panel | ~800 | `panels/objects/` |
| Frame composition, panel order, dock hosting | ~900 | `app/frame.rs` |
| Keyboard map, action dispatch, status narration | ~1,800 | `app/{keyboard,actions,status}.rs` |
| App state, open/save/close, password prompt, parked docs | ~2,200 | `app/state.rs` |
| Canvas hosting, hit-test dispatch, pan/zoom input | ~2,000 | `canvas/` |
| Dialogs — properties, print, export, reset, settings host | ~1,500 | `dialogs/` |
| Find | ~600 | `app/find.rs` |
| Remaining glue, helpers, types | ~5,300 | distributed |

**Honest framing:** perhaps 60 % of this moves with edits rather than
being rewritten. The genuinely *new* work is the selection model,
context menus, the properties panel, and the ribbon — and those are
additions, not replacements.

---

## Class D — Rebuild

| File | Code | Tests | Why |
|---|---:|---:|---|
| `ribbon.rs` | 666 | 47 | Tab/group model and the ownership test. The *mechanism* is good — one source of truth, a test asserting every group has exactly one owning tab. The *content* is superseded by `RIBBON_IA.md`: seven tabs plus a contextual Format tab, P1a amended so the QAT and status bar may mirror. Rebuild around the same invariants. |
| `ribbon_ui.rs` | 1,187 | 0 | Band rendering. Group-caption enforcement via one closure is worth keeping. Everything else follows the new IA. |
| `dock.rs` | 577 | 241 | Two independent `egui_tiles` trees, deliberately unbridgeable. Reconsider: the new panel set is larger (properties panel, comments, forms), layout must **persist** (Phase 3.6), and the two-pane-max constraint was a workaround for `egui_tiles` 0.16 hiding overflow tabs. Its constraints-as-tests approach is worth carrying forward regardless of the outcome. |

**Subtotal: 2,430 code lines, 288 test lines.**

---

## What is NOT salvaged, and is not lost either

Things the old GUI does not have. Listed so they are not mistaken for
salvage:

- Context menus — `grep context_menu` across the old crate returns
  **zero hits**.
- Move or resize anything carrying a `/Rect` — the one `FEATURES.md`
  row that gates markup, form widgets, redaction marks, links and
  dimensions all at once.
- A properties panel of any kind.
- Recent files, session restore, autosave, in-place save.
- Hand tool, rulers, grid, guides, go-to-page box.
- Six of ten markup kinds, and revision clouds.
- Page image export, attachments panel, canvas text selection — all
  **C**-rows: present in core or CLI, no GUI surface.

---

## Salvage procedure

For each file, in this order:

1. **Read it in full**, including the doc comments. They explain the
   defects that shaped it, and that reasoning is the most valuable thing
   being transferred.
2. **Copy it across with its tests and its documentation**, into its new
   module home.
3. **Apply the known fixes** for that file from `DEFECTS.md`, and add
   the regression test the defect implies.
4. **Split if it exceeds 1,500 lines** (R2) — find the seam, do not
   raise the limit.
5. **Assert it in `ui-verify`** before calling it done (R1). A green
   unit test is the floor.
6. **Record it here** — move the row to a "landed" state with the date
   and what changed. This file tracks reality, not intent.

Never salvage a file by pasting a snippet out of it. The old GUI's
value is disproportionately in its doc comments; a snippet leaves those
behind and the next engineer re-derives a decision that was already
made and already paid for.

---

## Landed

Step 6 of the procedure above. This section tracks **reality**: what has
actually been moved, where it now lives, and what changed on the way.

### Stage S0 — 2026-08-13

Built against `D:\Dev\pdfce` as of 2026-08-13 (`pdfce-render` 0.5.3).
All of the below builds, `cargo test -p pdfce-gui` is green (56 tests),
`cargo fmt -p pdfce-gui --check` and
`cargo clippy -p pdfce-gui --all-targets -- -D warnings` are clean, and
the binary renders the 5.6 MB CAD benchmark drawing.

| Source (old crate) | New home | State | What changed |
|---|---|---|---|
| `viewer.rs` (509 + 413 test) | `src/viewer/mod.rs` (945) | **complete** | `use eframe::egui` → `use egui`. `zoom_percent`, `page_to_screen`, `pdf_space_to_canvas` carry `#[allow(dead_code, reason = …)]` naming the stage of their first consumer. No arithmetic changed; every test carried across and passing. |
| `render_worker.rs` (466 + 116 test) | `src/render/worker.rs` (595) | **complete, three keys deferred** | Generation counter, `RenderCancel` token, `IN_FRAME_BUDGET` and the single-slot design untouched. `RenderKey` compares **two** keys (page, raster scale) rather than five: `annotations`, `font_env_generation` and `layers_generation` land **with the surfaces that vary them** (S2/S3) — the module docs tabulate all three, the defect each prevents, and the rule that the key ships in the same commit as its control. `cmyk_intent`/`fonts`/`view_magnification` left to `RenderOptions`' defaults for the same reason. New: a `render-spawn gen=N page=P scale=S` trace line, so "six zoom steps start six generations and complete one" is checkable from outside the process. |
| `raster.rs` (363) | `src/render/raster.rs` (214) | **page half complete** | `pixmap_to_color_image`, `PageTexture`, `texture_from_pixels` carried with the premultiplied-alpha and LINEAR-filtering sections verbatim. `ThumbnailCache` deliberately left behind — it belongs with the Pages panel (S3). `texture_from_pixmap` left behind — it exists for the print preview (S5). The stale *"Why rendering is synchronous"* section was replaced by an accurate one that **keeps the original prediction on the record** and notes it was vindicated. New: two unit tests pinning the premultiplied read, which the original had none of. |
| `canvas.rs` — `pan_offset`, `zoom_anchor_offset` (Class B) | `src/canvas/geometry.rs` (334) | **these two complete** | Lifted verbatim with all eight tests. The rest of `canvas.rs` (tool dispatch, selection, escape ladder) stays behind for S4/S5. |
| `diag.rs` (819 + 144 test) | `src/diag.rs` (119) | **trace channel only** | `enabled()`/`trace()` and the full header rationale. The `PDFCE_DIAG_SCRIPT` harness grammar (`Step`, `ScriptTool`, …) lands with `tools/ui-verify` at S1 — a script language with no interpreter is not salvage. New: a test that a disabled trace never builds its message. |
| `main.rs` — eframe bootstrap, `ViewportBuilder`, `PDFCE_DIAG_VIEWPORT`, `configure_context`, `open_path`, `settle_and_rasterize`, `is_unsupported_structure`, the canvas `ScrollArea` (Class C) | `src/main.rs` (149), `src/app/{mod,state,actions,keyboard}.rs`, `src/canvas/mod.rs` | **the S0 slice** | The three-way open-failure distinction, both staleness policies, the `ZOOM_SETTLE` debounce, the discrete-command bypass, the manual page centring (and the ~105 px selection-offset defect its comment records) all carried with their reasoning. `main.rs` is 149 lines against the old 25,005. |
| — (new) | `src/text/mod.rs` (205) | **new** | The ui-string catalog, a directory from the first commit so the old `ui_text.rs`'s 7,912-line R2 breach cannot recur as a migration. |

**`DEFECTS.md` fixes applied at salvage time (procedure step 3):**

- **D1 — the keyboard guard.** `src/app/keyboard.rs` uses
  `ctx.text_edit_focused()`, never `ctx.egui_wants_keyboard_input()`. The
  module header records the whole causal chain and the egui line numbers.
  The regression test
  `a_focused_non_text_widget_does_not_suppress_unmodified_keys` drives a
  real `Context` through two frames, **asserts that
  `egui_wants_keyboard_input()` is genuinely `true`** (so it cannot pass
  vacuously the way the old single test did), and then asserts the
  unmodified bindings still fire.
- **D1 "Not defects" — zoom anchor.** Ctrl+wheel now anchors on the
  **cursor**, via the salvaged `zoom_anchor_offset`. The *discrete* zoom
  commands (Ctrl+Plus/Minus/0) are still unanchored and carry a TODO in
  `src/canvas/mod.rs` naming `GUI_ROADMAP` Phase 3.1, where the zoom
  buttons and zoom-to-selection/region land and the anchor rule can be
  decided once for all four.

**Still owed on the S0 salvage (procedure step 5):** every one of the
above needs a `ui-verify` assertion, which cannot be written until the
harness exists at S1. A green unit test is the floor, not the ceiling.

### Phase 7 — the measure salvage, 2026-08-14

Class A `measure_tool.rs` and the Pass 12.M1 snap primitives, carried across
in one pass. `cargo test -p pdfce-gui --lib` green, all eight gates green.

| Source (old crate) | New home | State | What changed |
|---|---|---|---|
| `measure_tool.rs` (1,230 + 814 test) | `src/canvas/measure/pick.rs` (1,290), `scale.rs` (607), `state.rs` (377) | **complete** | Every `///` and `//!` paragraph carried verbatim; **all 36 tests carried, none dropped** — verified by diffing the complete function-name set (public, private and test) old against new: identical. Both load-bearing CLI-equivalence tests pass, so a canvas-authored `DimensionKind` is still byte-for-byte the one `pdfce-cli dimension-add` builds. **No `pdfce-core` API had moved** — every one of the ~25 imported items checked against the engine at this workspace's path dependency, unchanged signatures, no adaptation invented. Three adaptations, all documented in the files: `CanvasTool::MeasureLinear` and `GestureInterrupt` became prose (neither exists here), and cross-module doc links were repointed. |
| `canvas.rs:1584-1892` + tests at `:3046-3136` (12.M1 snap) | `src/canvas/snap.rs` (587) | **complete, not yet queried** | The zoom-invariant catch radius, the master/Alt gate, the Tab cycle, the two-click confirm, the indicator glyph. `#[allow(dead_code, reason = …)]` kept where the item is still unused, with **the reason rewritten** to name this shell's consumer — an inherited reason pointing at a pass in another repo is a stale claim. `screen_tolerance_to_page` deliberately **not** salvaged: `canvas/mapping.rs` already has it, and that module's header states there is no second place in `canvas/` that divides by zoom. |

**Three departures from the source, each deliberate:**

1. **One `CanvasTool::Measure(MeasureKind)` variant**, where the old shell had
   three variants plus `is_measure()` plus three `tool_builds_measure_*`
   predicates. Five helpers replaced by a value. This is the one place the
   salvage deliberately improves on its source rather than carrying it.
2. **A third file.** The planned two-way split leaves `pick.rs` about twenty
   lines over R2. Rather than shave prose to fit a threshold — the incentive
   `check-file-size.sh` says in its own header it refuses to build in — the cut
   was made at a seam **the original had already drawn for itself**, its own
   `// ---` banner separating the three pick machines from the container that
   owns them.
3. **No Accept/Reject box.** The old hosting held a completed pick in
   `MeasureState::pending` and waited for an explicit Accept in a property bar.
   The third click commits instead. `pending` survives on the type, with its
   tests, for a future property surface that is not a floating box.

**One collision the salvage surfaced, and how it was resolved.** The old shell
had **two** axes — a `CanvasTool` *and*, inside the linear tool, a
`LinearPickMode` — so `set_linear_pick_mode`'s discard guarded one and the tool
switch guarded the other. This shell has **one**: `MeasureKind`, with two-line
as a kind rather than a mode. Had arming become the axis while the discard
stayed attached to the old one, a half-finished point pick would have survived
into two-line mode — and the original's own docs warn that this surfaces not as
an error but as *"something strange"* on the operator's **next** click.
`MeasureState::set_kind` is that rule restated over the axis this shell
actually has, delegating to `set_linear_pick_mode` for the pair it already
owns.

**Still owed:** a `ui-verify` assertion (procedure step 5) for the placed
dimension, and the snap candidate query, which is the one thing standing
between the salvaged snap primitives and a pick that snaps.
