//! # `app::status` — the status bar: the narrator on the left, the constant controls on the right
//!
//! `RIBBON_IA.md` §6 specifies this surface in one paragraph, and the
//! paragraph contains the whole design:
//!
//! > **Status bar** — Find toggle, actual size, fit width, fit page, zoom
//! > −/%/+, page ◀ n/N ▶, and a **new editable page-number box**. These are
//! > the controls a user touches constantly; they belong where they never
//! > disappear behind a tab change. The current render-diagnostics text
//! > moves behind a disclosure (see `DEFECTS.md` §5).
//!
//! Two halves, two arguments.
//!
//! ## The left half: the narrator, demoted
//!
//! `DEFECTS.md`'s "Not defects" table records the old shell opening with a
//! substitute-glyph census:
//!
//! > The first thing a user reads is the app talking about itself. Excellent
//! > information, wrong prominence — put it behind the disclosure triangle
//! > that is already there.
//!
//! So the render diagnostics — which glyphs were substituted, which images
//! were skipped, which content streams the file does not actually contain —
//! are still here, still complete, and **closed by default**. The word
//! "Render notes" stays visible so the report is discoverable; only the
//! report itself is one click away.
//!
//! That table's other entry that lands on this surface is the zoom anchor:
//! *"Zoom buttons pin the page's top-left, not the centre or the cursor."*
//! The − and + buttons below raise the same `ZoomIn`/`ZoomOut` actions the
//! ribbon and the keyboard raise, so they inherit that anchor exactly. It is
//! **not** fixed here, and it must not be fixed here: the anchor is
//! `crate::canvas`' scroll arithmetic (`zoom_anchor_offset`), and a status
//! bar that anchored zoom differently from the ribbon would be a second
//! zoom model. Recorded so the next reader knows the omission is a decision.
//!
//! ## The right half: the controls that must never move
//!
//! Everything on the right exists because it is reached constantly and
//! because a tab change must not take it away. They are **mirrors**, and
//! amendment P1a is what makes mirroring legal:
//!
//! > *the QAT and the status bar are shortcut surfaces, not tabs. A command
//! > may appear on exactly one tab and additionally on the QAT and/or the
//! > status bar.*
//!
//! So `Actual size · Fit width · Fit page` appear here *and* on View ▸ Zoom,
//! and `RibbonTab::groups()` remains the single source of truth for tab
//! ownership because this surface is outside its domain.
//!
//! ## ★ The editable page box is the point of the exercise
//!
//! `GUI_ROADMAP.md` 3.3 states the problem in one line: *"Reaching page 37
//! of 42 currently means the thumbnail rail or 36 keystrokes."* Type `37`,
//! press Enter, arrive. Four properties make that usable rather than
//! merely present, and each is implemented deliberately:
//!
//! 1. **Commit on Enter or focus loss, never per keystroke.** Someone typing
//!    `42` passes through `4`, and a box that navigated per keystroke would
//!    take them to page 4, re-render a CAD sheet, and then take them to page
//!    42 — with the intermediate render wasted and the operator's eye
//!    already moved. See [`page_box`].
//! 2. **Out of range clamps, and says so.** `99` in a 42-page document goes
//!    to page 42 *and reports that it did*
//!    ([`crate::text::status::page_clamped_note`]). A silent clamp is
//!    indistinguishable from a box that ignored what was typed, and an
//!    operator who cannot tell those apart stops trusting the control.
//! 3. **Non-numeric input is refused without discarding it.** The text stays
//!    in the box with a note beside it. Wiping an operator's typing to
//!    "helpfully" restore the current page destroys the evidence of what
//!    they meant.
//! 4. **★ It suppresses the unmodified keyboard bindings while focused, and
//!    that is defect D1 from the other end.** `crate::app::keyboard` guards
//!    those bindings with `ctx.text_edit_focused()` — *not*
//!    `egui_wants_keyboard_input()`, which means "any widget has focus" and
//!    cost the operator the Delete key and all keyboard page navigation from
//!    the first canvas click onward. `text_edit_focused()` resolves the
//!    focused id and asks whether a `TextEditState` exists **for that id**,
//!    so this control has to be a real [`egui::TextEdit`] with a stable id
//!    for the guard to see it. A `DragValue`, a custom painted field, or a
//!    label-plus-popup would all typecheck and all silently re-open D1 in
//!    the reverse direction: `PageDown` would step the page while the
//!    operator was halfway through typing a page number.
//!    `page_box::tests::typing_a_digit_into_the_page_box_does_not_also_step_the_page`
//!    is the regression test, and it asserts the failing condition is really
//!    present before asserting the fix — the shape the D1 post-mortem says
//!    the original test was missing.
//!
//! ## ★ The bar has a FIXED height, and this is measured rather than tidy
//!
//! `D:/dev/rag/egui/bottom_panel_height_change_retriggers_fit_to_viewport_zoom.md`,
//! pdfce standing rule **R128**: *a panel whose size feeds a
//! fit-to-viewport computation has a fixed size.*
//!
//! The loop is real and it was measured. A content-driven status panel takes
//! space from the central panel; `FitMode::Page`/`FitMode::Width` recompute
//! their zoom from the canvas viewport **every frame they are active**
//! ([`crate::viewer::ViewState::apply_fit`]); so one extra status line on
//! frame N produces a smaller fit scale on frame N+1. On pdfce that showed
//! up as a page that visibly shrank across three frames (230 % → 224 % →
//! 215 %) and, worse, as click coordinates that went stale between the frame
//! they were captured on and the next render. The canonical symptom is *"the
//! page jumped when I clicked an object"* — which reads as a selection bug
//! and gets investigated in the selection code, where nothing is wrong.
//!
//! Two defences, and this module carries both ends:
//!
//! - **The caller pins the panel.** [`HEIGHT_PTS`] exists to be passed to
//!   `egui::Panel::bottom(..).exact_size(..)`. Only `exact_size` closes the
//!   loop: `default_height` is a starting value that content still overrides,
//!   `min_height`/`max_height` bound a *range* the panel still varies inside,
//!   and `resizable(false)` only stops the operator dragging the edge.
//! - **The content cannot grow anyway.** [`show`] lays everything out inside
//!   one allocated row of [`ROW_HEIGHT_PTS`], and — the part that actually
//!   takes discipline — **opening the disclosure does not add a line.** The
//!   render notes are drawn *on the same row*, to the right of the triangle,
//!   elided if they are long, with the full text on hover. That is why
//!   §6 says "one line" and why there is no [`egui::CollapsingHeader`]
//!   anywhere in this file: a collapsing header's entire behaviour is to
//!   change its own height, which is the one thing this surface may not do.
//!   [`tests::the_bar_is_exactly_as_tall_open_as_closed`] pins it.
//!
//! ## ★ Two defects this surface surfaced, both since fixed
//!
//! Recorded because the *reasoning* is the durable part, not the bug.
//!
//! **`Actual size` did not produce actual size.** It raised
//! `Action::Fit(FitMode::None)`, which only stops the per-frame re-fit and
//! leaves `zoom` wherever it was — so a control whose tooltip promises
//! *"one PDF point per screen point"* pinned 73 % at 73 %. It was mirrored
//! here **as-is on purpose** rather than worked around: the status bar is a
//! *shortcut surface* for a tab command (P1a), and a mirror that behaved
//! differently from the control it mirrors would be a second zoom model,
//! which is worse than a shared defect. Fixed 2026-08-13 by
//! `Action::ZoomTo`, dispatched from `view.zoom_actual` and from here —
//! one change, both surfaces. Deliberately not `ZoomBy(1.0 / zoom)`, which
//! lands on the right number but routes a discrete command through the
//! wheel path's 150 ms settle.
//!
//! **`Ctrl+0` had two owners.** The manifest keymap bound it to
//! `view.zoom_actual` while `crate::app::keyboard` bound it to
//! `Fit(FitMode::Page)` and got there first, so this bar's Actual-size
//! tooltip had to advertise no chord at all. Fixed 2026-08-13, and fixed
//! *structurally*: `keyboard` no longer knows what a manifest chord means —
//! it spells the key, looks it up in the keymap, and returns a command id
//! that goes through the same dispatcher a ribbon click reaches. The
//! tooltip names its chord again, and `no_chord_has_two_owners` fails
//! naming the chord and both claimants if the conflict returns.
//!
//! ## What is NOT drawn, and why that is not an oversight
//!
//! **The Find toggle.** §6 lists it first, and it is absent. `Find` has no
//! entry in `crate::shell::commands` — the registry holds 74 ids and none of
//! them is a find — and `crate::shell::manifest`'s keymap says so itself:
//! *"Ctrl+F — Find lives in the status bar, which this manifest does not
//! describe."* There is no find panel, no search over the page's text, and
//! no action to raise. Under `PROJECT_PLAN.md` §3's no-placeholders
//! invariant an unavailable capability renders **nothing** — not a greyed
//! button, not a toggle that opens an empty pane — so the control is omitted
//! entirely and the catalog carries no strings for it either. It lands with
//! the find panel, and `crate::text::status` gains its two entries then.
//!
//! **The page controls, on a document with no pages.** `/Count 0` is legal
//! PDF. A page box over a document with no pages is a control whose every
//! input is out of range, so the group is dropped and the zoom controls stay.
//!
//! **The whole left half, before the first raster.** The render notes are a
//! property of a *drawn page*; there is nothing to disclose until a page has
//! been drawn, and `page_texture` is `None` only before the first render and
//! after a render failure (which the canvas already reports in words).
//!
//! ## Actions, not mutations
//!
//! Every control here pushes an [`Action`] and mutates nothing.
//! `crate::app::actions`' header states the invariant — *"No code path runs
//! from a widget to a document"* — and this module honours it including for
//! the page box, whose commit raises `Action::GoToPage` rather than touching
//! `view.page_index`. The only state this module writes is its own widget
//! state (the draft text, the note, the disclosure flag), which lives in
//! `egui`'s per-id store and describes the *control*, not the document.
//!
//! ### Why that state lives in `egui::Memory` when `crate::app::state`
//! argues against it
//!
//! `OpenDoc`'s docs record moving the canvas selection *off* `egui::Memory`,
//! because a selection is document-scoped and `Memory` outlives documents —
//! which forced a synthetic document identity, and an address is not an
//! identity. None of that applies here, for a reason rather than by luck:
//!
//! - The draft is a **text-editing buffer**. `egui` already keeps one for
//!   this very widget (`TextEditState`, keyed by the same id), so the draft
//!   sits beside its own cursor and selection rather than in a second place
//!   with a different lifetime.
//! - It is **discarded on focus loss**, always: the box shows the current
//!   page whenever it is not being edited and no note is outstanding. A
//!   value that cannot survive a click elsewhere cannot survive a document
//!   open either, so there is nothing to key on and no staleness to detect.
//! - Neither this module nor the parent may add a field: `app/mod.rs`,
//!   `app/state.rs` and `app/actions.rs` are owned elsewhere, and inventing
//!   a parallel owner for four bytes of widget state would be a worse
//!   structural change than using the store egui provides for exactly this.
//!
//! ## Where the page box went
//!
//! Into [`page_box`], because this file reached standing rule R2's
//! 1,500-line ceiling and the box is the one subject here that separates
//! cleanly: it has its own state, its own vocabulary, its own pure decision
//! function and its own hazard (defect D1's keyboard guard), none of which
//! the rest of the bar shares. What is left answers *"how is the bar laid
//! out, and what does each group show?"*; that module answers *"what did the
//! operator mean by what they typed?"*. Its header carries the commit rule,
//! the three outcomes, and the D1 argument in full.

/// Page navigation and the editable page-number box. See this module's
/// header for the seam, and that one's for the control.
mod page_box;

use egui::{Align, Id, Layout, Vec2};

use crate::app::actions::Action;
use crate::app::state::{OpenDoc, Status};
use crate::text::status as t;
use crate::viewer::FitMode;

// ---------------------------------------------------------------------------
// Geometry — see the ★ R128 section of the module docs
// ---------------------------------------------------------------------------

/// The exact outer height, in egui points, the status panel must be given.
///
/// **Pass this to `egui::Panel::bottom(..).exact_size(..)`, not to
/// `default_height`.** The difference is rule R128: `exact_size` pins the
/// panel's outer size so its content cannot perturb the central region at
/// all, and every other sizing API leaves the fit-to-viewport feedback loop
/// open. The module docs carry the measured case.
///
/// [`ROW_HEIGHT_PTS`] plus egui's own `Frame::side_top_panel` inner margin
/// (2 pt above and below) plus a little room for the panel's separator
/// stroke. Generous rather than tight: a bar whose content is clipped by one
/// point is a legibility defect, and the cost of the slack is four pixels of
/// canvas that never change size.
pub const HEIGHT_PTS: f32 = 30.0;

/// The height of the single row every control is laid out inside.
///
/// [`show`] allocates exactly this, so the bar's content height is a
/// constant rather than a function of what there is to say — which is the
/// second half of the R128 defence and the reason the disclosure draws its
/// line *beside* the triangle rather than beneath it.
pub const ROW_HEIGHT_PTS: f32 = 24.0;

/// The panel must be taller than the row it contains, or the bar's own
/// content is clipped by the frame that is supposed to hold it.
///
/// Checked at **compile time** rather than in a test: the relationship
/// between the two constants is a property of the constants, and a test
/// would only re-discover at run time what the compiler can refuse outright.
const _: () = assert!(
    HEIGHT_PTS > ROW_HEIGHT_PTS,
    // ui-text-exempt: compile-error text, never displayed in the UI
    "the panel height must leave room for its own inner margin"
);

/// A fixed width for the zoom readout.
///
/// `8%` and `800%` are different widths, and without a reserve the − button
/// would step sideways every time the operator clicked +. Wide enough for
/// four characters, which is the whole range [`crate::viewer::ZOOM_LADDER`]
/// can produce.
const ZOOM_READOUT_WIDTH_PTS: f32 = 46.0;

/// The share of the bar the render-notes line may occupy before eliding.
///
/// The notes are the *least* urgent thing on this surface (`DEFECTS.md`:
/// "excellent information, wrong prominence"), so on a narrow window they
/// yield to the navigation controls rather than squeezing them. The full
/// text is always available on hover, so nothing is lost — only deferred.
const NOTES_WIDTH_FRACTION: f32 = 0.45;

// ---------------------------------------------------------------------------
// Named regions — see `crate::diag::ui_rect` for the contract and the naming
// rule ("a stable, lowercase, hyphenated noun for the thing an operator
// would point at"). These names are matched literally by `tools/ui-verify`,
// so renaming one silently un-aims whatever check was measuring it.
// ---------------------------------------------------------------------------

/// The strip the bar's content occupies.
const REGION_BAR: &str = "status-bar"; // ui-text-exempt: trace region name, never displayed

/// The disclosure triangle, plus its one line of render notes when open.
const REGION_NOTES: &str = "status-group:notes"; // ui-text-exempt: trace region name, never displayed

/// `Actual size · Fit width · Fit page`.
const REGION_FIT: &str = "status-group:fit"; // ui-text-exempt: trace region name, never displayed

/// `−  ⟨percent⟩  +`.
const REGION_ZOOM: &str = "status-group:zoom"; // ui-text-exempt: trace region name, never displayed

/// Trace slot for the bar's steady state, de-duplicated on the rendered line.
const STATUS_SLOT: &str = "status"; // ui-text-exempt: trace slot name, never displayed

// ---------------------------------------------------------------------------
// Widget-state ids
// ---------------------------------------------------------------------------

/// Whether the render-notes disclosure is open.
const NOTES_OPEN_ID: &str = "pdfce-status-notes-open"; // ui-text-exempt: widget id, never displayed

// ---------------------------------------------------------------------------
// The bar
// ---------------------------------------------------------------------------

/// Draw the status bar.
///
/// Call it inside a bottom panel pinned to [`HEIGHT_PTS`]:
///
/// ```ignore
/// egui::Panel::bottom("status")
///     .exact_size(crate::app::status::HEIGHT_PTS)
///     .show(ui, |ui| crate::app::status::show(ui, &self.status, &mut actions));
/// ```
///
/// Composition order matters, and the rule is already written down in
/// `crate::app`'s header: *a full-width bar must be added **before** any side
/// panel, or it starts at the side panel's edge instead of spanning the
/// window. A status bar that does not span the window is not a status bar.*
/// So this belongs with the ribbon, above the docks, and the `CentralPanel`
/// stays last because it takes whatever is left.
///
/// Raises actions and mutates nothing — see the module docs.
pub fn show(ui: &mut egui::Ui, status: &Status, actions: &mut Vec<Action>) {
    // ★ One allocated row, of a height that does not depend on what there is
    // to show. R128; see the module docs for the measurement.
    let row = Vec2::new(ui.available_width(), ROW_HEIGHT_PTS);
    let bar = ui.allocate_ui_with_layout(row, Layout::left_to_right(Align::Center), |ui| {
        // ★ Claim the whole row even when nothing is drawn into it.
        //
        // `allocate_ui_with_layout` advances its parent by the child's
        // *min_rect* — what the content actually used — not by the size that
        // was asked for (`egui-0.35.0/src/ui.rs:1330`). Without this line a
        // bar with no document, or with the disclosure closed, would consume
        // less height than one with them, and the R128 loop would be open
        // again through the one path the panel's `exact_size` does not cover:
        // a caller who forgot to use it. Two independent defences, and this
        // is the one that lives in the code being defended.
        ui.set_min_height(ROW_HEIGHT_PTS);

        // With nothing open there is no page to number, no zoom to report
        // and no raster to have notes about. The bar still occupies its
        // height — that is the whole point of pinning it — but it draws no
        // control, because a control that cannot work is the placeholder the
        // project's invariants forbid.
        let Status::Open(doc) = status else {
            return;
        };

        // Left: the narrator, demoted behind a disclosure.
        notes(ui, doc);

        // Right: the controls that must never move.
        //
        // ★ Laid out RIGHT-TO-LEFT, so the group added FIRST is drawn
        // RIGHTMOST. The reading order on screen is therefore the reverse of
        // the call order below:
        //
        //     screen:  fit  │  zoom  │  page          (left → right)
        //     calls:   page │  zoom  │  fit           (first → last)
        //
        // The alternative — measuring the cluster and left-aligning it at a
        // computed offset — is the pattern `egui-shell`'s own dock notes call
        // out as fragile (`right − width` goes negative the moment the bar is
        // narrower than its content). A right-to-left layout cannot get that
        // wrong; it simply runs out of room, and the notes on the left are
        // what yields.
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            page_box::group(ui, doc, actions);
            ui.separator();
            zoom_group(ui, doc, actions);
            ui.separator();
            fit_group(ui, doc, actions);
        });
    });

    // The content strip, not the panel: a legibility check wants the pixels
    // the bar actually drew into. See `crate::diag::ui_rect` on why the
    // application measures this rather than the harness computing a fraction
    // of the window.
    crate::diag::ui_rect(REGION_BAR, bar.response.rect);

    if let Status::Open(doc) = status {
        crate::diag::trace_changed(STATUS_SLOT, || {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "status page={} pages={} zoom={} fit={:?}",
                doc.view.page_index,
                doc.pages.len(),
                doc.view.zoom_percent(),
                doc.view.fit,
            )
        });
    }
}

// ---------------------------------------------------------------------------
// Left — the narrator
// ---------------------------------------------------------------------------

/// The render-notes disclosure, and its one line when open.
///
/// Drawn only when a page has actually been rasterized: the notes describe a
/// raster, and `page_texture` is `None` only before the first render and
/// after a failure the canvas already reports in words.
///
/// **Opening this does not make the bar taller.** The line is drawn beside
/// the triangle, inside the same row, elided at [`NOTES_WIDTH_FRACTION`] of
/// the bar with the whole text on hover. See the ★ R128 section of the
/// module docs for why that is a requirement rather than a layout preference.
fn notes(ui: &mut egui::Ui, doc: &OpenDoc) {
    let Some(texture) = doc.page_texture.as_ref() else {
        return;
    };
    let id = Id::new(NOTES_OPEN_ID);
    let mut open = ui
        .ctx()
        .data_mut(|d| d.get_temp::<bool>(id))
        .unwrap_or(false);

    let rect = ui
        .scope(|ui| {
            let toggle = ui
                .selectable_label(open, t::diagnostics_toggle(open))
                .on_hover_text(t::diagnostics_tooltip());
            if toggle.clicked() {
                open = !open;
            }
            if !open {
                return;
            }
            let line = notes_line(&texture.diagnostics);
            // A bounded sub-region, so a page with eight findings cannot
            // squeeze the navigation controls off the right of the bar.
            let width = (ui.available_width() * NOTES_WIDTH_FRACTION).max(0.0);
            ui.allocate_ui_with_layout(
                Vec2::new(width, ROW_HEIGHT_PTS),
                Layout::left_to_right(Align::Center),
                |ui| {
                    ui.add(
                        egui::Label::new(egui::RichText::new(&line).small().weak())
                            // Elide rather than wrap: wrapping is how a
                            // one-row bar becomes a two-row bar, which is
                            // the R128 loop with extra steps.
                            .truncate(),
                    )
                    .on_hover_text(line.clone());
                },
            );
        })
        .response
        .rect;

    crate::diag::ui_rect(REGION_NOTES, rect);
    ui.ctx().data_mut(|d| d.insert_temp(id, open));
}

/// One count from the renderer's report, paired with the catalog entry that
/// puts it into words.
///
/// A named type rather than an inline tuple so [`notes_line`]'s table reads
/// as a table. The `fn(usize) -> String` half is a plain function pointer
/// rather than a closure on purpose: it names a *catalog entry*, so the
/// pairing is a lookup that can be read down the page, and a reviewer
/// checking that every reported field has a sentence has one place to look.
type NoteEntry = (usize, fn(usize) -> String);

/// Turn the renderer's honesty report into the one line the disclosure shows.
///
/// # What is reported, and what is deliberately not
///
/// Every field here changes **what the operator can see on the page**: text
/// that was not drawn, images that were not drawn, glyphs whose shapes are
/// not the document's, layers that were hidden, content the file does not
/// actually contain. Those are facts an operator can act on — supply a font,
/// turn a layer back on, go and find the missing stream.
///
/// `Diagnostics::tolerated` and `Diagnostics::compat_skipped` are **not**
/// reported. Both count divergences that leave the picture correct: a
/// tolerated structural oddity (an unbalanced `Q`, a mid-path `cm`) is
/// something the renderer absorbed and drew right anyway, and a `BX`/`EX`
/// skip is spec-sanctioned (§7.8.2 Table 32) — the file is *telling* readers
/// to skip it. Listing them would put two numbers that mean "nothing is
/// wrong" in front of the six that mean something is. They remain in the
/// structured data for whoever wants them; this is a status bar, not a
/// report.
///
/// # Order
///
/// Most consequential first: content the file is missing, then whole
/// surfaces that were not drawn, then glyph-level substitution, then the
/// operator's own hidden layers, then operators pdfce has not implemented. A
/// line that opens with "3 unrecognised drawing operators" and buries "text
/// from 2 fonts not drawn" is sorted by the renderer's interest rather than
/// by the reader's.
fn notes_line(d: &pdfce_render::Diagnostics) -> String {
    let entries: [NoteEntry; 9] = [
        (
            d.contents_streams_unresolved,
            t::diagnostics_contents_missing,
        ),
        (d.fonts_unsupported, t::diagnostics_fonts_skipped),
        (d.images_unsupported, t::diagnostics_images_skipped),
        (d.glyphs_notdef, t::diagnostics_glyphs_notdef),
        (d.glyphs_substituted, t::diagnostics_glyphs_substituted),
        (d.glyphs_supplied, t::diagnostics_glyphs_supplied),
        (d.oc_sections_hidden, t::diagnostics_layers_hidden),
        (d.deferred_ops, t::diagnostics_ops_deferred),
        (d.unknown_ops, t::diagnostics_ops_unknown),
    ];
    let parts: Vec<String> = entries
        .into_iter()
        .filter(|(n, _)| *n > 0)
        .map(|(n, render)| render(n))
        .collect();
    if parts.is_empty() {
        // Stated positively. An empty disclosure is indistinguishable from
        // one that failed to fill itself, and the operator who opened it
        // wanted an answer either way.
        t::diagnostics_clean().to_owned()
    } else {
        t::diagnostics_join(&parts)
    }
}

// ---------------------------------------------------------------------------
// Right — fit
// ---------------------------------------------------------------------------

/// `Actual size · Fit width · Fit page`, mirroring View ▸ Zoom under P1a.
///
/// ★ **Two of the three are toggles and one is a button, and that asymmetry
/// is honest rather than sloppy.** `FitMode::Page` and `FitMode::Width` are
/// *modes*: they persist, they re-fit on every window resize, and a control
/// that shows whether you are in one is telling the truth. `FitMode::None`
/// is the absence of a mode, so a "selected" Actual size would light up at
/// any pinned zoom — including 73 % — which is the module docs' ★ defect
/// rendered on screen instead of merely wired. A plain button makes no claim
/// about state.
///
/// Called *last* of the three groups because the layout runs right-to-left;
/// see [`show`].
fn fit_group(ui: &mut egui::Ui, doc: &OpenDoc, actions: &mut Vec<Action>) {
    let fit = doc.view.fit;
    let rect = ui
        .scope(|ui| {
            // Right-to-left: added first is drawn rightmost, so the screen
            // reads `Actual size · Fit width · Fit page`.
            if ui
                .selectable_label(fit == FitMode::Page, t::fit_page())
                .on_hover_text(t::fit_page_tooltip())
                .clicked()
            {
                actions.push(Action::Fit(FitMode::Page));
            }
            if ui
                .selectable_label(fit == FitMode::Width, t::fit_width())
                .on_hover_text(t::fit_width_tooltip())
                .clicked()
            {
                actions.push(Action::Fit(FitMode::Width));
            }
            // ★ Raises exactly what the ribbon's `view.zoom_actual` raises,
            // including its defect. See the module docs: the fix is a new
            // action variant, not a divergent mirror.
            if ui
                .button(t::fit_actual_size())
                .on_hover_text(t::fit_actual_size_tooltip())
                .clicked()
            {
                actions.push(Action::Fit(FitMode::None));
            }
        })
        .response
        .rect;
    crate::diag::ui_rect(REGION_FIT, rect);
}

// ---------------------------------------------------------------------------
// Right — zoom
// ---------------------------------------------------------------------------

/// `−  ⟨percent⟩  +`.
///
/// The readout is a label rather than a field: there is no action that sets
/// a zoom to a named value (see [`crate::text::status::zoom_percent`]), and
/// a text box in front of nothing is a placeholder. It is given a fixed
/// width so that stepping from `100%` to `75%` does not move the − button
/// out from under the operator's pointer.
fn zoom_group(ui: &mut egui::Ui, doc: &OpenDoc, actions: &mut Vec<Action>) {
    let percent = doc.view.zoom_percent();
    let rect = ui
        .scope(|ui| {
            // Right-to-left: added first is drawn rightmost, so the screen
            // reads `− ⟨percent⟩ +`.
            if ui
                .button(t::zoom_in())
                .on_hover_text(t::zoom_in_tooltip())
                .clicked()
            {
                actions.push(Action::ZoomIn);
            }
            ui.add_sized(
                Vec2::new(ZOOM_READOUT_WIDTH_PTS, ROW_HEIGHT_PTS),
                egui::Label::new(t::zoom_percent(percent)),
            )
            .on_hover_text(t::zoom_percent_tooltip());
            if ui
                .button(t::zoom_out())
                .on_hover_text(t::zoom_out_tooltip())
                .clicked()
            {
                actions.push(Action::ZoomOut);
            }
        })
        .response
        .rect;
    crate::diag::ui_rect(REGION_ZOOM, rect);
}

/// Fixtures the bar's own tests and [`page_box`]'s tests both need.
///
/// A module of its own rather than helpers inside `mod tests`, because two
/// sibling test modules share them and `pub(super)` on a helper buried in one
/// of them would read as "the other module reaches into my tests" rather than
/// as "this is the shared harness". Visible to `crate::app::status` and its
/// descendants, and to nothing else.
#[cfg(test)]
pub(super) mod test_support {
    use super::{Action, Status, show};
    use crate::app::state::{FOUR_PAGES, open_fixture};
    use egui::{Context, Event, Key, Modifiers, RawInput};

    /// An application status with the four-page fixture open.
    ///
    /// Opened through `crate::app::state::open_fixture`, which is the same
    /// three calls `PdfceApp::open_path` makes in the same order — so what
    /// these tests drive is the real state machine rather than a hand-built
    /// approximation of it.
    pub(in crate::app::status) fn opened() -> Status {
        Status::Open(Box::new(open_fixture(FOUR_PAGES)))
    }

    /// Run one frame of the bar and return the actions it raised.
    pub(in crate::app::status) fn frame(
        ctx: &Context,
        status: &Status,
        input: RawInput,
    ) -> Vec<Action> {
        let mut actions = Vec::new();
        let _ = ctx.run_ui(input, |ui| show(ui, status, &mut actions));
        actions
    }

    /// Build a `RawInput` carrying one key press.
    pub(in crate::app::status) fn key_press(key: Key, modifiers: Modifiers) -> RawInput {
        RawInput {
            events: vec![Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers,
            }],
            modifiers,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::opened;
    use super::*;
    use egui::{Context, RawInput};

    // =======================================================================
    // R128 — the height that must not move
    // =======================================================================

    /// Measure the height [`show`] consumes for one frame.
    fn bar_height(ctx: &Context, status: &Status) -> f32 {
        let mut height = f32::NAN;
        let _ = ctx.run_ui(RawInput::default(), |ui| {
            let mut actions = Vec::new();
            height = ui
                .scope(|ui| show(ui, status, &mut actions))
                .response
                .rect
                .height();
        });
        height
    }

    /// ★ **The bar is exactly as tall with the disclosure open as closed —
    /// and as tall with no document as with one.**
    ///
    /// Rule R128, asserted rather than argued. A status panel whose height
    /// varies feeds the fit-to-viewport recompute, and the measured result
    /// on pdfce was a page that shrank 230 % → 224 % → 215 % across three
    /// frames with no zoom input, plus click coordinates that went stale
    /// between the frame they were captured on and the next render. The
    /// symptom reads as a selection bug and gets investigated in the
    /// selection code, where nothing is wrong.
    ///
    /// This is the property that forbids an [`egui::CollapsingHeader`] here:
    /// changing its own height is the entire behaviour of that widget. It is
    /// also what `ui.set_min_height` in [`show`] is for — without it the row
    /// would shrink to whatever the content happened to need.
    #[test]
    fn the_bar_is_exactly_as_tall_open_as_closed() {
        let ctx = Context::default();
        let status = opened();
        let empty = Status::Empty;

        let closed_no_doc = bar_height(&ctx, &empty);

        ctx.data_mut(|d| d.insert_temp(egui::Id::new(NOTES_OPEN_ID), false));
        let closed = bar_height(&ctx, &status);

        ctx.data_mut(|d| d.insert_temp(egui::Id::new(NOTES_OPEN_ID), true));
        let open = bar_height(&ctx, &status);

        assert!(
            (open - closed).abs() < 0.01,
            "opening the render notes changed the bar's height ({closed} → \
             {open}); that is R128's feedback loop, and it is measured in \
             page zoom, not in pixels"
        );
        assert!(
            (closed_no_doc - closed).abs() < 0.01,
            "opening a document changed the bar's height ({closed_no_doc} → \
             {closed}), which re-fits the page on the frame it opens"
        );
        assert!(
            closed <= ROW_HEIGHT_PTS + 0.01,
            "the bar's content ({closed} pt) overflowed its allocated row \
             ({ROW_HEIGHT_PTS} pt); either the row is too short or something \
             here is laying out vertically"
        );
    }

    // =======================================================================
    // Legibility — the labels that are glyphs
    // =======================================================================

    /// ★ **Every glyph the bar draws exists in the bundled font set.**
    ///
    /// `⏴`, `⏵`, `⏷`, `−` and `·` are not decoration: three of them are the
    /// entire visible text of a control. A codepoint the font set cannot
    /// draw renders as a tofu box, which is defect D2's shape — an invisible
    /// label — with the operator's page position behind it.
    ///
    /// **This test has already paid for itself.** The catalog was written
    /// with `◀` `▶` for the page steps and `▸` `▾` for the disclosure, and
    /// all four are missing from egui's bundled fonts (Ubuntu-Light +
    /// NotoEmoji + emoji-icon-font). They would have shipped as four tofu
    /// boxes on the two controls an operator touches most.
    ///
    /// Checked against `FontFamily::Proportional`, which is what every label
    /// and button on this bar resolves to, and run inside a real pass because
    /// egui has no fonts before one.
    #[test]
    fn every_glyph_the_status_bar_draws_has_a_glyph() {
        let ctx = Context::default();
        let labels: Vec<String> = vec![
            t::diagnostics_toggle(false).to_owned(),
            t::diagnostics_toggle(true).to_owned(),
            t::diagnostics_join(&["a".to_owned(), "b".to_owned()]),
            t::zoom_out().to_owned(),
            t::zoom_in().to_owned(),
            t::zoom_percent(100),
            t::fit_actual_size().to_owned(),
            t::fit_width().to_owned(),
            t::fit_page().to_owned(),
            t::prev_page().to_owned(),
            t::next_page().to_owned(),
            t::page_of_total(42),
            t::page_number(37),
            t::page_clamped_note(99, 42, 42),
            t::page_rejected_note().to_owned(),
        ];

        let mut missing = Vec::new();
        let _ = ctx.run_ui(RawInput::default(), |ui| {
            let font = egui::FontId::proportional(14.0);
            ui.ctx().fonts_mut(|f| {
                for label in &labels {
                    for c in label.chars() {
                        if !f.has_glyph(&font, c) {
                            missing.push((label.clone(), c));
                        }
                    }
                }
            });
        });

        assert!(
            missing.is_empty(),
            "these labels contain codepoints the bundled fonts cannot draw, \
             so they would render as tofu boxes: {missing:?}"
        );
    }

    // =======================================================================
    // The narrator
    // =======================================================================

    /// A clean render still says something.
    #[test]
    fn a_clean_render_reports_that_it_is_clean() {
        let d = pdfce_render::Diagnostics::default();
        assert_eq!(notes_line(&d), t::diagnostics_clean());
    }

    /// Every reported field reaches the line, and none of them wraps it.
    ///
    /// The second half is R128 again: the disclosure gets **one** line, so a
    /// page with every finding at once must still produce a single line.
    #[test]
    fn every_reported_finding_reaches_the_one_line() {
        let d = pdfce_render::Diagnostics {
            contents_streams_unresolved: 1,
            fonts_unsupported: 2,
            images_unsupported: 3,
            glyphs_notdef: 4,
            glyphs_substituted: 5,
            glyphs_supplied: 6,
            oc_sections_hidden: 7,
            deferred_ops: 8,
            unknown_ops: 9,
            ..Default::default()
        };

        let line = notes_line(&d);
        for n in 1..=9 {
            assert!(
                line.contains(&n.to_string()),
                "finding {n} is missing from the line: {line}"
            );
        }
        assert!(!line.contains('\n'), "the disclosure gets one line: {line}");
        assert_ne!(line, t::diagnostics_clean());
    }

    /// The two counters that mean "nothing is wrong" stay out of the line.
    ///
    /// A tolerated structural oddity was absorbed and drawn correctly, and a
    /// `BX`/`EX` skip is the file telling readers to skip it (§7.8.2 Table
    /// 32). Reporting either would put reassurance in front of the findings
    /// that need reading.
    #[test]
    fn tolerated_and_compat_skipped_are_not_reported() {
        let d = pdfce_render::Diagnostics {
            tolerated: 11,
            compat_skipped: 13,
            ..Default::default()
        };
        assert_eq!(
            notes_line(&d),
            t::diagnostics_clean(),
            "neither counter describes anything the operator can see or act on"
        );
    }
}
