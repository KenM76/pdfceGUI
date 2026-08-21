//! # canvas — the page on screen, what is selected on it, and the gestures that move both
//!
//! The one place a rasterized page is drawn and the one place canvas input is
//! read. The navigation gestures — **wheel to scroll, Ctrl+wheel to zoom about
//! the cursor, middle-drag to pan**, and from Phase 3 the **hand tool with
//! space-to-pan**, **anchored discrete zoom**, **zoom to selection** and
//! **marquee zoom to region** — and, from stage S4, the **selection model**:
//! click, Shift+click, double-click to descend, Escape to ascend, rubber-band
//! marquee, eight grips plus move, **dragging a selection to move it**, and
//! Delete.
//!
//! ## Where the selection model lives
//!
//! | module | subject |
//! |---|---|
//! | [`mapping`] | the ONE screen⟷page conversion, and the hit tolerance |
//! | [`target`] | the provider seam, and the trait re-attached to the salvaged decomposition |
//! | [`selection`] | selection as identity; the level ladder; re-resolution |
//! | [`forms`] | filling a form where it is drawn: why it is not a tool, why its hit test takes no tolerance, and what its editor cannot promise |
//! | [`gesture`] | press / drag / release, the clear that must not happen on a press, Escape's abort, and the one rubber band's two intents |
//! | [`moving`] | which move verb each rung reaches, the canvas→page delta, and the ghost's honesty rule |
//! | [`handles`] | eight grips plus move, and the cursor over each |
//! | [`textsel`] | selecting **text**: why it needs no capability, why it is offered exactly where content selection is not, and the single pass that makes the highlight and the copy one value |
//! | [`menus`] | the right-click: which of the two canvas menus opens, and the select-first rule that makes it about the thing you pointed at |
//! | [`overlay`] | what all of it looks like — and what rule 4 forbids it looking like |
//! | [`geometry`] | the pan and zoom-anchor arithmetic |
//! | [`keys`] | Escape and Delete, and which of Escape's three claimants gets it |
//! | [`tool`] | select or hand, and the space bar that borrows the hand |
//! | [`zoom`] | **the anchor rule**, the two-frame handshake, and the five zoom paths that route through it |
//! | [`interact`](mod@interact) | **what the operator just did, and what happens as a result** — the pointer frame, the gesture application, the right-click, the keys, the re-resolve, the cursor |
//!
//! Everything above is pure except this file, [`interact`](mod@interact),
//! [`overlay`], and [`moving`]'s one wiring function ([`moving::drag`], which
//! is the only thing there that touches the live object model — its rules are
//! pure). That is the point: `PROJECT_PLAN.md`'s split is driven by
//! testability, and the
//! selection invariants are exactly the kind of property a unit test can hold
//! and a running window cannot be trusted to demonstrate.
//!
//! ## What is in this file, and what is next door in [`interact`](mod@interact)
//!
//! This file is **composition**. [`show`] and [`show_in`], the `ScrollArea`,
//! the strip of page rectangles and the raster or the state sentence drawn into
//! each, the fit resolved against this frame's viewport, the placeholder a
//! document with no pages gets, the [`CanvasGeometry`] the rulers are then
//! painted against, and the layout trace. It needs a live `Ui`, and it answers
//! one question: **where does everything go on screen?**
//!
//! [`interact`](mod@interact) is **interaction**. The pointer frame, what a
//! press would land on, the gesture machine's outcome and its application, the
//! right-click, the keys, the re-resolve of the selection, the overlay draw and
//! the cursor. It needs this frame's input, and it answers the other question:
//! **what did the operator just do, and what happens as a result?**
//!
//! The two change for different reasons — a page-display mode is a layout
//! question, a new tool is an interaction one — which is the seam rule R2
//! forced this file along when it reached 1,526 lines. The invariants that
//! belong to the second half travel with it, and are written up in
//! [`interact`](mod@interact)'s header rather than here: **selection survives
//! navigation**, and the two closed seams (where the selection lives, and the
//! one shared decomposition).
//!
//! ## Actions, not mutations — and the two documented exceptions
//!
//! The project's strongest structural invariant is that **no code path runs
//! from a widget to a document**: everything an operator does becomes an
//! [`Action`] that is applied *after* the frame is drawn. It is why the old
//! GUI's undo log is coherent, and it is established here at S0 — with two
//! actions and one widget — because retrofitting it later is expensive.
//!
//! [`show`] therefore takes `&mut OpenDoc` but is permitted to write
//! exactly three fields, none of them document state and none of them
//! expressible as an action. The first two are **frame bookkeeping about the
//! view**, and are impossible to defer for the same reason:
//!
//! 1. **`last_scroll_offset`** — the offset the scroll area settled on this
//!    frame. It is only readable *after* the area is built, and the next
//!    frame's pan needs it *before* the area is built. Storing it is what
//!    lets a pan track the hand instead of lagging it by a frame.
//! 2. **`zoom_anchor`** — which page point a zoom step must hold still, and
//!    where. It has to span two frames because the new zoom is not known when
//!    the step is asked for: the zoom is an [`Action`], applied after the UI
//!    is built, and it *clamps*. Recording the inputs and solving next frame
//!    avoids predicting a clamp we do not control. [`zoom`] owns both ends —
//!    which point ([`zoom::anchor_point`]) and when to spend it
//!    ([`zoom::anchor_step`]).
//!
//! The third is **`selection`**, which arrived here when seam 1 above was
//! closed, and it does not weaken the invariant: a selection *names* parts of
//! a document and changes nothing a save would write. It is settled during the
//! frame, from input that only exists during the frame, so deferring it would
//! make a click land a frame after the operator made it — the same argument as
//! the two above. The line is unmoved and visible in [`canvas_keys`]: Delete
//! removes nothing here, it raises [`VectorAction::DeleteSelection.into()`] carrying the
//! operand list, applied after the frame through the one funnel. Nothing that
//! touches `EditSession` runs from a widget.
//!
//! A fourth value is derived rather than stored: [`crate::viewer::ViewState::apply_fit`]
//! is called inline, because a fit mode is a pure function of this frame's
//! viewport and turning it into an action would apply it one frame late —
//! the page would visibly lag every window resize.
//!
//! ## Input conventions, and why breaking them feels wrong
//!
//! - **Plain wheel scrolls; Ctrl+wheel zooms.** egui routes these apart at
//!   the input-state level: a wheel event carrying the zoom modifier
//!   becomes `zoom_delta` and contributes *nothing* to
//!   `smooth_scroll_delta`, so the scroll area cannot pan and zoom off the
//!   same gesture. Breaking this is the single most common way a
//!   from-scratch viewer feels wrong.
//! - **Middle-drag pans** — the CAD / Inkscape / Illustrator / browser
//!   convention, requested by the operator on 2026-08-04. It is implemented
//!   against the scroll offset directly rather than by enabling
//!   `ScrollSource.drag`, because that knob is button-agnostic: turning it
//!   on would also make a *left*-drag pan, and the left button is reserved
//!   for the selection marquee that arrives at S4.
//! - **Panning triggers no re-raster.** It moves the viewport over an
//!   existing texture.
//!
//! ## The zoom anchor — decided once, in [`zoom`]
//!
//! `DEFECTS.md`'s "Not defects" table records that *"zoom buttons pin the
//! page's top-left, not the centre or the cursor"*. The wheel path was fixed
//! at S0; Phase 3.1 closes the rest, and the rule that governs all five zoom
//! paths (wheel, in, out, actual size, and the two framing commands) lives in
//! [`zoom`]'s header and in [`zoom::anchor_point`] — **the pointer when it is
//! over the canvas, the viewport's centre when it is not**. This file no
//! longer decides an anchor; it arms one ([`zoom::arm_anchor`]) and consumes
//! one ([`zoom::consume_anchor`]).

// Filling an interactive form where it is drawn: the boxes, the hit test that
// deliberately takes no tolerance, and the one editor a focused field gets.
pub mod forms;
// pdfce's OWN crosshair bitmap, supplied to the OS as a real cursor. The
// platform's stock crosshair is monochrome and its colour belongs to the
// operator's pointer scheme, which is how it came to be white on white paper.
pub mod cursor;
pub mod geometry;
pub mod gesture;
// Draggable alignment lines: what a guide belongs to, where it lives on disk,
// and why grabbing one cannot also start a marquee.
pub mod guides;
// The drawing grid, in each page's own space. Split from `rulers` under R2
// along the seam that module's header already drew: a ruler is chrome beside
// the canvas that reserves layout space, a grid is chrome over the page that
// reserves none.
/// ★ Dragging a **Bézier handle** — the last Phase 1 row, and one `pdfce`'s
/// own `gui` column ticked `[x]` while nothing here drew a handle at all.
/// `EditSession::move_handle` had existed since Pass 30.1; what was missing was
/// a way to see one and a way to grab one.
/// ★ **Cut, copy and paste on the canvas** — the operator's report of
/// 2026-08-19. Implements the row the engine can express (markup) and records
/// the one it cannot (page content) as a dated citation rather than a promise.
/// ★ **What a click MEANS** — the eight-rung ladder that decides whether a
/// completed click places an anchor, a caret, a vertex, a sticky, a dimension
/// pick, a text sweep, an annotation selection or a content selection. Split
/// out of `interact` under R2 on 2026-08-20; its header carries the order and
/// why each rung sits where it does.
pub mod clicking;
pub mod clipboard;
/// ★ **What Shift does to a drag** - the axis lock and the aspect lock, written
/// down once for the five drags that share them. `ui-conventions/drag-moves.md`
/// D5, found absent from every one of them by the conventions sweep of
/// 2026-08-20. Its header carries why one module rather than five call sites.
pub mod constrain;
/// A placement drag on a selected ce dimension - the operator's report of
/// 2026-08-20, *"I need to be able to move the dimension after it has been laid
/// down"*. Reaches `place_dimension`, never `move_dimension`; its header says
/// why that distinction is the whole design.
pub mod dimdrag;
pub mod grid;
pub mod handledrag;
pub mod handles;
/// ★ **What a press would land on, and what it would mean.** Split out of
/// `interact` under R2; its header carries the four-way precedence between a
/// Bézier handle, an anchor, a resize grip and the selection body — the single
/// most bug-prone rule on this canvas, learned three separate times in one day.
pub mod pressing;
// Reading this frame's pointer — what a click landed on at every rung, which
// of the two panning gestures is in flight, and where the in-flight press is
// kept between frames. Split out under R2 when the rulers landed; see its
// header on why the forced seam is a real one.
pub mod input;
// What the operator just did, and what happens as a result: the seven ordered
// steps of the one gesture function, the `Frame` of settled facts it is handed,
// and the two invariants it is accountable for. Split from this file under R2
// along the seam the two subjects already drew — composition needs a live `Ui`
// and answers *where does everything go?*, interaction needs this frame's input
// and answers *what did the operator just do?* Its items are `pub(super)`: this
// module is the only caller and nothing outside `canvas` can name them.
pub mod interact;
// Escape and Delete, and the precedence between the three things that would
// like Escape. Split from this file along the seam every other split here
// follows: that module is drivable by a headless `egui::Context`, this one
// needs a window.
pub mod keys;
pub mod mapping;
// Drawing a markup annotation where the operator points: the rubber band, the
// four kinds it can author, and the raw endpoints an arrow's head depends on.
pub mod markup;
pub mod measure;
pub mod menus;
// Dragging a selection: which verb each rung reaches, the canvas→page delta,
// and the ghost's honesty rule. Kept out of `selection` deliberately — that
// module is already 1,352 lines and owns *what is selected*, while this owns
// *what happens when you drag it*.
pub mod moving;
pub mod overlay;
/// ★ The application's own colour ROLES — `preview` and `dimension_selected` —
/// built from the resolved theme's palette and published per frame.
///
/// `egui_shell::theme::Overlays` is a generic role map because **R7** forbids
/// the shell learning what a ce dimension is; the roles are pdfce's, exactly as
/// the ribbon manifest's command ids are. Its header carries the mapping
/// argument and the distinctness test the shell says the application owes.
pub mod overlays;

/// **Dropping pages onto the page view** — the caret between two sheets, and
/// the release that inserts or reorders there.
///
/// The operator's request of 2026-08-19: *"…or onto the canvas to add pages
/// and insert them in between the pages we've dragged to"*. The drag itself
/// lives in [`crate::pagedrag`], which is what lets a gesture that began in a
/// panel — possibly in another document — end here.
pub mod pagedrop;
/// ★ Everything the canvas draws, once everything is decided — lifted out of
/// [`interact`] when that file crossed R2's ceiling. Its header carries the
/// layer order and the argument for each position in it.
mod painting;
/// ★★ **What a click is ALLOWED to land on** — the operator's selection
/// filter, and the eleven classes it switches.
///
/// `OPERATOR_REQUESTS.md` O17. This is the replacement for Edit ▸ Content's
/// declare-your-intention-then-point model, and its header carries the whole
/// argument: why a filter belongs on the status bar rather than the ribbon,
/// why it is **subtractive only** (so `default()` reproduces today's behaviour
/// and R6 holds by construction), and why it composes with
/// [`crate::app::modes::capability::Capabilities`] as an `AND` rather than an
/// override.
///
/// Pure: no egui, no pointer, no document. Which is exactly why the popup that
/// drives it still has to be driven before any of it counts — R1.
pub mod pick;
/// ★★ The eight resize grips, finally committing — built out of `move_nodes`
/// because `pdfce-core` has no scale verb, which was re-derived against its
/// source rather than taken from a note.
pub mod resizing;
/// ★★ **The ninth grip** — the rotate handle above the selection box, and the
/// one gesture the eight could never express. `ui-conventions/handles.md` H2,
/// and the third word of the operator's *"reposition, resize, or rotate"*. Its
/// header carries why a rotation is not a resize with different arithmetic:
/// the pointer's DISTANCE from the centre must mean nothing.
pub mod rotating;
// The ruler gutters, the 1-2-5 tick ladder they and the grid share, and what
// unit the whole thing reads in. Its header carries the three decisions this
// feature turns on: the unit, the space the grid lives in, and why the
// reservation it takes out of the viewport is a constant (R128).
pub mod rulers;
pub mod selection;
// The GUI half of snapping: the zoom-invariant catch radius, the master/Alt
// gates, the Tab cycle, the two-click confirm, and the indicator glyph.
pub mod snap;
// Which page the frame is about, in what order the rest should be drawn, and
// where a navigated-to page lands. The canvas's half of Phase 4's strip.
pub mod strip;
pub mod target;
// Selecting TEXT on the page, and copying it: the mode gate that needs no
// capability, the interaction decisions and which of Acrobat / Inkscape /
// SolidWorks each came from, and the one derivation that makes what is
// highlighted and what is copied the same value.
/// The three markup kinds that carry WORDS — text box, sticky note and
/// stamp. A different gesture (place, then type) and a different engine spec
/// from the seven geometric kinds; its header carries the argument.
pub mod textannot;
pub mod textsel;
// EDITING the page's own words, and placing new ones: the caret, the draft, and
// — in its `disposition` submodule — the two cases `DEFECTS.md` D4b records as
// wrong on commit, where the engine had the mechanism and the old GUI never
// selected it.
pub mod textedit;
// The `PDFCE_DIAG` lines the canvas writes, and the shape contract
// `tools/ui-verify` reads them under.
pub mod trace;
// Which pointer tool the canvas is in — select or hand — and the space bar
// that borrows the hand for as long as it is held.
pub mod tool;
// The anchor rule, the two-frame handshake it rides on, and the five zoom
// paths that route through it.
pub mod zoom;

use egui::{PointerButton, Pos2, Rect, Sense, scroll_area::ScrollSource, vec2};
use egui_shell::HandlerToken;

use crate::app::actions::Action;
use crate::app::modes::Capabilities;
use crate::app::state::OpenDoc;
use crate::canvas::input::pan_delta;
// The interaction half, next door. `Frame` is this frame's settled facts on the
// way in; `interact` is everything that follows from them. Imported by name
// rather than called as `interact::interact(…)` so the one call site below
// reads exactly as it did before the split.
use crate::canvas::interact::{Frame, interact};
use crate::canvas::mapping::PageMapping;
use crate::canvas::rulers::CanvasGeometry;
use crate::shell::menus::MenuHost;
use crate::viewer;

/// Padding, in points, left around the page inside the canvas so the page
/// does not sit flush against the panel edges under a fit mode.
///
/// Subtracted from the viewport *before* the fit scale is derived rather
/// than added as a layout margin afterwards, so "fit page" really does fit
/// with the gap visible instead of fitting exactly and then being clipped
/// by the gap.
///
/// `pub` because [`zoom::zoom_to_rect`] fits a *region* by the identical rule
/// and must leave the identical gap — a framing command that pressed the
/// region flush against the panel edges while "Fit page" left 16 points would
/// read as two different ideas of what fitting means.
pub const CANVAS_MARGIN: f32 = 16.0;

/// Draw the page, read the canvas gestures, and attach the canvas context
/// menus.
///
/// Operator intent leaves by two routes, and the split is not arbitrary:
///
/// * **`actions`** carries what the canvas itself decides — a zoom step, a
///   fit, a Delete raised by the Delete key. These are already `Action`s
///   because the canvas knows what they mean.
/// * **the return value** carries `egui_shell::HandlerToken`s: the commands
///   the operator chose from a context menu. The canvas must *not* translate
///   those, because translating them is what `PdfceApp::dispatch_token`
///   does for the ribbon, and a second translation is how the two surfaces
///   start disagreeing about what `format.delete` means. Handing the token
///   on unchanged is what makes `RIBBON_IA.md` §5.8's *"carries the same
///   commands again"* literally true.
///
/// `host` is `None` when the application has no validated shell — see
/// [`MenuHost`] — in which case no menu is attached and a right-click does
/// nothing, which is the correct behaviour for a build with no menu
/// document rather than a disabled feature.
///
/// Beyond those two, everything this function decides lands in the three
/// documented bookkeeping fields (see the module docs). The document itself
/// is never touched.
///
/// # ★ The rulers wrap this, and the wrapping is three statements
///
/// [`rulers::reserve`] takes a **constant** bite out of `ui` before anything
/// measures the viewport (rule R128 — see that module's header §3), the whole
/// of the canvas is then drawn into a child `Ui` covering what is left, and
/// the gutters are painted afterwards from the geometry [`show_in`] hands
/// back. Painting them *after* is what puts a guide preview over the page
/// rather than under it, and what lets the ruler mark the page's own edges —
/// neither of which is knowable until the scroll area has settled.
#[must_use]
pub fn show(
    ui: &mut egui::Ui,
    doc: &mut OpenDoc,
    host: Option<&MenuHost<'_>>,
    find: &crate::find::FindState,
    caps: Capabilities,
    pen: crate::canvas::markup::pen::Pen,
    actions: &mut Vec<Action>,
) -> Vec<HandlerToken> {
    let gutters = rulers::reserve(ui, doc.view.rulers && !doc.pages.is_empty());
    let mut content = gutters.content_ui(ui);
    let (tokens, geometry) = show_in(&mut content, doc, host, find, caps, pen, actions);
    rulers::draw(ui, doc, gutters, geometry.as_ref());
    // Starting a guide drag needs a ruler to drag out of; *finishing* one does
    // not, because it may have started on the canvas. So the two halves are
    // separate calls and only the first is conditional — see `guides::settle`.
    guides::ruler_drag(ui, doc, gutters);
    guides::settle(ui, doc, geometry.as_ref(), actions);
    tokens
}

/// [`show`]'s body, drawn into the canvas region the rulers left.
///
/// Returns the context-menu tokens *and* what the frame learned about where
/// its pages ended up — see [`CanvasGeometry`] on why that has to travel
/// outwards rather than be read again.
#[must_use]
fn show_in(
    ui: &mut egui::Ui,
    doc: &mut OpenDoc,
    host: Option<&MenuHost<'_>>,
    find: &crate::find::FindState,
    caps: Capabilities,
    pen: crate::canvas::markup::pen::Pen,
    actions: &mut Vec<Action>,
) -> (Vec<HandlerToken>, Option<CanvasGeometry>) {
    if doc.pages.is_empty() {
        let placeholder = ui.centered_and_justified(|ui| ui.label(crate::text::canvas_no_pages()));
        // Say so on the trace rather than staying silent. A consumer that
        // finds no `canvas` line otherwise has to guess between "this build
        // does not trace its layout" and "there was no layout to trace", and
        // those need opposite responses. See `trace_layout`.
        crate::diag::trace_changed(trace::LAYOUT_SLOT, || {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "canvas-unavailable reason=no-pages".to_owned()
        });
        crate::diag::ui_rect(trace::REGION_PAGE_MESSAGE, placeholder.inner.rect);
        return (Vec::new(), None);
    }

    // The viewport the fit modes are measured against, minus the margin the
    // page sits inside. Clamped to at least one point because a window
    // dragged to nothing would otherwise produce a zero or negative
    // viewport, and `fit_scale` would fall back to actual size on a window
    // the operator is still resizing — a visible jump at the end of a drag.
    let viewport = (
        (ui.available_width() - CANVAS_MARGIN).max(1.0),
        (ui.available_height() - CANVAS_MARGIN).max(1.0),
    );

    // Resolve a fit mode against THIS frame's viewport. Under
    // `FitMode::None` this is a no-op, so it is safe to call always — and
    // calling it always is what makes "Fit page" a mode rather than a
    // one-shot: resize the window and the page re-fits.
    //
    // ★ Against the current **row**, not the current page, and the ceiling
    // against the row's tightest page. Under Single and Continuous a row is
    // one page and both reduce to exactly what they were; under a facing mode
    // a row is the spread, and fitting one half of a spread would leave the
    // other half off screen from a control called "Fit page". See
    // [`viewer::strip::row_metrics`] for why the row is measured without
    // laying the strip out — the strip's geometry depends on the zoom this
    // produces, so it cannot be the source of it.
    let pixels_per_point = ui.ctx().pixels_per_point();
    //
    // ★ `fit_metrics`, NOT `row_metrics`, and the difference is a closed
    // feedback loop. Under a continuous mode `page_index` is derived from the
    // scroll, so fitting the current row makes the zoom depend on the scroll
    // and the scroll depend on the zoom — measured oscillating between
    // `page=0 zoom=1.4773` and `page=1 zoom=0.9559` on a mixed-size document
    // for as long as the wheel was moving. `fit_metrics` fits the tightest row
    // under a continuous mode and the current row otherwise; on a document of
    // one page size the two are identical.
    let row = viewer::strip::fit_metrics(
        &doc.pages,
        doc.view.display,
        doc.view.page_index,
        pixels_per_point,
    );
    doc.view.apply_fit(row.extent, viewport, row.max_zoom);

    // ★ The whole-canvas render failure is the **single-page** answer, and it
    // stays exactly that.
    //
    // With one page on screen, "this page would not draw" is the only thing
    // there is to say and a sentence in the middle of the canvas is the right
    // way to say it. With several, replacing the entire strip with one
    // sentence would hide thirty-nine sheets that drew perfectly because one
    // did not — so every other mode draws the refusal **in the failing page's
    // own rectangle** instead. See [`crate::render::strip::draw_page_state`].
    if doc.view.display == viewer::PageDisplay::Single
        && let Some(message) = &doc.render_error
    {
        let text = crate::text::canvas_render_failed(message);
        let placeholder =
            ui.centered_and_justified(|ui| ui.colored_label(ui.visuals().error_fg_color, text));
        // Same argument as the no-pages arm: there is genuinely no page rect
        // this frame, and saying that is more useful than silence. `reason=`
        // is a fixed token, not the operator-facing message — the message can
        // be reworded and a consumer keying on it would break.
        crate::diag::trace_changed(trace::LAYOUT_SLOT, || {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "canvas-unavailable reason=render-failed".to_owned()
        });
        crate::diag::ui_rect(trace::REGION_PAGE_MESSAGE, placeholder.inner.rect);
        return (Vec::new(), None);
    }

    // ★ **Where every page this view shows sits.** One page under `Single`,
    // whose rect is `(0,0)..display_size` — so everything below is the
    // arithmetic it already was. See [`viewer::strip`].
    let layout = doc.strip();
    let display_size = layout.size();
    let current = doc.view.page_index;
    // The current page's placement, which is the frame of reference every
    // single-page solve in `canvas::zoom` and `find::reveal` is handed. Falls
    // back to the whole strip for the degenerate case where the current page is
    // not laid out at all, which the page-index clamp normally prevents.
    let current_rect = layout
        .rect_of(current)
        .unwrap_or_else(|| Rect::from_min_size(Pos2::ZERO, display_size));
    let current_origin = (current_rect.min.x, current_rect.min.y);
    let current_display = (current_rect.width(), current_rect.height());
    // The scale every page on screen is rasterized at. Derived once, here, and
    // used to look each visible page's raster up: deriving it a second time
    // inside the draw loop is how a page could be *drawn* against one key while
    // being *requested* against another, which would show as a page that never
    // stops saying it is not drawn yet.
    let raster_scale =
        viewer::raster_scale(doc.view.zoom, pixels_per_point, doc.prefs.render_quality);

    // `ScrollSource::ALL` = scroll bars + plain mouse wheel + drag-to-pan.
    // Ctrl+wheel never reaches this, because egui routes a modified wheel
    // event to `zoom_delta` instead of to the scroll delta.
    //
    // `drag` is switched OFF: egui's drag-to-scroll is button-agnostic, and
    // the primary button belongs to the S4 selection marquee. Panning is
    // the middle button, implemented below against the offset directly.
    let mut scroll_source = ScrollSource::ALL;
    scroll_source.drag = egui::scroll_area::DragScroll::Never;

    let mut scroll_area = egui::ScrollArea::both()
        .id_salt("page-canvas") // ui-text-exempt: internal widget id, never displayed
        .scroll_source(scroll_source);

    // Which tool the primary button is in this frame — the select tool, or the
    // hand (chosen, or borrowed for as long as the space bar is down). Read
    // once, here, and passed down: two readings could disagree within a frame
    // and the disagreement would be a drag that panned AND marquee'd.
    let active_tool = tool::active(ui.ctx());

    // Zoom to the anchor, half two: a zoom step was armed on an earlier frame
    // and the new zoom is now known (post-clamp), so solve for the offset that
    // keeps the anchored page point where the rule says it belongs, and force
    // it onto the area before it lays out. `consume_anchor` owns the gate that
    // decides whether the zoom has actually landed yet — see [`zoom`]'s header
    // on the two-frame handshake, and on why an unconditional `take()` here
    // made every *command*-driven zoom silently unanchored.
    //
    // ★ Three sources of a forced scroll offset, and the order between them is
    // a precedence rather than a coincidence:
    //
    // 1. **a zoom anchor**, because a zoom has just landed and the whole point
    //    of the anchor is that one page point does not move as it does;
    // 2. **a find reveal**, because the operator asked to be taken somewhere
    //    and a one-shot navigation outranks nothing else in flight;
    // 3. **a middle-drag pan**, which is a live gesture — and a live gesture is
    //    LAST here for the reason it wins anyway: it re-arms itself on the next
    //    frame, while both of the others are spent once.
    //
    // ★ A **fourth** source arrives with Phase 4 — a page *command* under a
    // continuous mode, which has to scroll the strip to the page it named —
    // and it sits third, below the two one-shots and above the live gesture,
    // by the same reasoning: it is a one-shot the operator asked for, and a
    // live gesture re-arms itself while the one-shots are spent once.
    //
    // ★ Two of the three offsets below are solved by code this work does not
    // own — `canvas::zoom`'s anchor handshake and `find::reveal`'s two-frame
    // reveal — and both are written for a scroll area whose content is **one
    // page at the origin**. Rather than teach either about a strip, the canvas
    // converts: `geometry::page_local_offset` presents the world the way those
    // solves expect, and `geometry::strip_offset` converts their answer back.
    // The conversion is exact, and under `Single` it is the identity. See
    // `geometry`'s header for the whole argument.
    let vp = ui.available_size();
    let to_strip = |local: (f32, f32)| {
        let (x, y) = geometry::strip_offset(
            local,
            current_origin,
            (display_size.x, display_size.y),
            current_display,
            (vp.x, vp.y),
        );
        vec2(x, y)
    };
    if let Some(offset) = zoom::consume_anchor(ui.ctx(), doc, current_display) {
        scroll_area = scroll_area.scroll_offset(to_strip((offset.x, offset.y)));
    } else if let Some(offset) = crate::find::take_reveal_offset(doc, current_display, (vp.x, vp.y))
    {
        // The other half of `Action::Find`'s navigation: the page change was
        // applied after the frame that asked for it, and this is the first
        // frame that is actually showing that page — so it is the first frame
        // on which the page's real drawn size is known and the offset can be
        // solved. `crate::find` owns both the gate and the solve; nothing
        // about a search is decided here.
        //
        // The reveal's gate is `reveal.page == view.page_index`, so the page it
        // solves against is always the current one — which is exactly the page
        // `to_strip` converts for. A reveal therefore lands on the right page
        // of a continuous strip without `find::reveal` knowing a strip exists.
        scroll_area = scroll_area.scroll_offset(to_strip((offset.x, offset.y)));
        doc.tracked_page = doc.view.page_index;
    } else if let Some(offset) = strip::page_scroll_offset(doc, &layout, (vp.x, vp.y)) {
        scroll_area = scroll_area.scroll_offset(offset);
    } else if let Some(pan) = pan_delta(ui, active_tool) {
        // Panning subtracts the pointer delta: the content follows the hand,
        // so the page moves WITH the pointer rather than under it.
        let (x, y) = geometry::pan_offset(
            (doc.last_scroll_offset.x, doc.last_scroll_offset.y),
            (pan.x, pan.y),
            (display_size.x, display_size.y),
            (vp.x, vp.y),
        );
        scroll_area = scroll_area.scroll_offset(vec2(x, y));
        // The gesture has to look like what it is. Without a cursor change a
        // pan that hits the end of the scroll range is indistinguishable from
        // a pan that is not working.
        ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
    }

    let scroll_output = scroll_area.show(ui, |ui| {
        // Centre the STRIP manually rather than with
        // `ui.centered_and_justified`, because that helper returns the
        // JUSTIFIED CONTAINER rect — the whole available area — while
        // drawing the image centred inside it. Taking that rect as
        // `image_rect` makes every page↔screen mapping wrong by the centring
        // margin whenever the content is smaller than the viewport.
        //
        // The symptom in the old GUI was severe and specific: at "Fit page"
        // on a page narrower/shorter than the canvas, selection outlines
        // drew offset from the object they outlined (~105 px on one measured
        // case — exactly the vertical margin), and clicking directly ON a
        // visible object MISSED it. At high zoom, where the page exceeds the
        // viewport and the margin is zero, the same click landed perfectly.
        // That is the giveaway: the error scaled with the margin, not with
        // the zoom — and it was worst at exactly the zoom an operator uses
        // to see a whole page.
        //
        // So: reserve `max(strip, viewport)` so the ScrollArea still scrolls
        // when the content is larger AND there is a margin to centre within
        // when it is smaller, then place each page at an explicit rect
        // derived from the strip's own. `Ui::put` and `allocate_rect` return
        // a Response whose `.rect` IS that rect, so every page's screen rect
        // is its true drawn rect by construction rather than by coincidence.
        let avail = ui.available_size();
        let outer = vec2(display_size.x.max(avail.x), display_size.y.max(avail.y));
        let (outer_rect, _) = ui.allocate_exact_size(outer, Sense::hover());
        // The strip's own rect on screen. Every page's rect is this origin
        // plus its strip-space placement, which is what makes the strip the
        // single owner of "where is page N".
        let strip_rect = Rect::from_center_size(outer_rect.center(), display_size);
        let strip_origin = strip_rect.min.to_vec2();

        // The viewport, expressed in strip space — what decides which pages
        // are drawn at all. `last_scroll_offset` is the previous frame's
        // settled offset, which is the best estimate available *before* this
        // frame's is known; a page that appears one frame late during a fast
        // fling is the cost, and it is bounded by one frame.
        let visible_rect = Rect::from_min_size(
            Pos2::new(doc.last_scroll_offset.x, doc.last_scroll_offset.y),
            avail,
        );

        // `click_and_drag`, not `hover`, on EVERY page — not only the current
        // one. A press on a page the operator is not currently "on" is how
        // they move to it under a continuous mode, and a page that did not
        // sense the press would swallow the click entirely: the operator would
        // have to click twice, once to arrive and once to act, with nothing on
        // screen to say why. Both branches must also agree with each other —
        // a first frame that reserved the space with a different sense would
        // swallow the click that opened the document, experienced as "the
        // first click never works".
        let sense = Sense::click_and_drag();
        let mut drawn: Vec<strip::DrawnPage> = Vec::new();
        for placement in layout.visible(visible_rect) {
            let rect = placement.rect.translate(strip_origin);
            let key = doc.render_key_for(placement.page, raster_scale);
            // The current page's raster lives in its own slot; every other
            // page's lives in the strip cache. See `render::strip`'s header
            // for why the split exists and why the rule is enforced rather
            // than remembered.
            let texture = if placement.page == current {
                doc.page_texture.as_ref().map(|t| t.texture.clone())
            } else {
                doc.strip_page_texture(placement.page, key)
                    .map(|t| t.texture.clone())
            };
            let has_raster = texture.is_some();
            let response = match texture {
                Some(texture) => ui.put(
                    rect,
                    egui::Image::from_texture(&texture)
                        .fit_to_exact_size(rect.size())
                        .sense(sense),
                ),
                None => {
                    // No raster. Reserve the same rect with the same sense so
                    // nothing jumps when one arrives, then SAY what is
                    // happening rather than leaving white paper — see
                    // `render::strip::draw_page_state` on why a blank
                    // rectangle would be a placeholder and this is not.
                    let response = ui.allocate_rect(rect, sense);
                    let state = if placement.page == current {
                        strip::current_page_state(doc)
                    } else {
                        doc.strip_page_state(placement.page, key)
                    };
                    if let Some(state) = state {
                        crate::render::strip::draw_page_state(
                            ui.painter(),
                            ui.visuals(),
                            rect,
                            // The scroll viewport in SCREEN terms — which
                            // inside a `ScrollArea` is exactly the `Ui`'s clip
                            // rect. It is passed so the state sentence is
                            // centred in the part of the page the operator can
                            // actually see: a page whose top edge is showing
                            // and whose middle is a metre below the window
                            // would otherwise draw as a silent empty
                            // rectangle. See `draw_page_state`.
                            ui.clip_rect(),
                            placement.page + 1,
                            &state,
                        );
                    }
                    response
                }
            };
            drawn.push(strip::DrawnPage {
                page: placement.page,
                rect,
                response,
                has_raster,
            });
        }

        // `avail` rides out with the pages because it is the viewport the
        // zoom-to-cursor solve needs, and it is only knowable in here — the
        // same `avail` that decided `outer` above, so the margin the solve
        // reconstructs is the margin this frame actually drew.
        (drawn, avail, strip_rect)
    });

    let (drawn, viewport_size, strip_rect) = scroll_output.inner;
    // The offset the area settled on THIS frame: the `offset_before` of any
    // zoom step the operator starts now, and the base the next frame's
    // middle-drag pan moves from.
    doc.last_scroll_offset = scroll_output.state.offset;
    let scroll_offset = scroll_output.state.offset;

    // ★ **Which page this frame's input is about**, and the two ways it is
    // decided. Both write `view.page_index`, which is the fourth item of
    // per-frame view bookkeeping the canvas is permitted to write (see the
    // module header): a scroll position cannot be deferred into an `Action`,
    // because the action would be applied after the frame that has already
    // drawn from it.
    //
    // 1. **the scroll**, under a continuous mode: the page with the greatest
    //    visible area, per `Strip::page_at_view`. This is `GUI_ROADMAP.md`
    //    Phase 4.3's scroll-driven current-page tracking, and it is what makes
    //    the status bar's page box, the Objects panel and the `objects n=`
    //    trace describe the sheet the operator is actually reading.
    // 2. **a press**, in any mode: pressing on a page makes it current, so a
    //    click on the page below acts on the page below rather than missing.
    //    A press outranks the scroll because it is deliberate, and it is read
    //    from the pages' own responses so it costs nothing on a frame with no
    //    input.
    let view_rect = Rect::from_min_size(Pos2::new(scroll_offset.x, scroll_offset.y), viewport_size);
    if doc.view.display.is_continuous()
        && let Some(page) = layout.page_at_view(view_rect)
    {
        doc.view.page_index = page;
        doc.tracked_page = page;
    }
    if let Some(page) = drawn
        .iter()
        .find(|d| {
            d.response.drag_started_by(PointerButton::Primary)
                || d.response.clicked_by(PointerButton::Primary)
                || d.response.dragged_by(PointerButton::Primary)
                || d.response.secondary_clicked()
        })
        .map(|d| d.page)
    {
        doc.view.page_index = page;
        doc.tracked_page = page;
    }

    // ★ **A page drag from somewhere else, landing here.**
    //
    // Here rather than inside the scroll-area closure, because that is the
    // first point at which every visible page's *screen* rectangle is known —
    // and the caret has to be drawn over the pages rather than under them,
    // which in an immediate-mode painter is a matter of call order.
    //
    // It reads no `Response`, which is the point: the press that started this
    // drag happened in a panel — possibly on another document's page list —
    // so no widget on this canvas has ever seen it. See `canvas::pagedrop`.
    //
    // Costs one `egui::Memory` lookup on a frame with no drag in flight, which
    // is every frame but the handful the operator is carrying something.
    pagedrop::offer(ui, doc, &drawn, scroll_output.inner_rect, actions);

    // The page being acted on: whatever the pointer acted on, else the current
    // page. Its response and its rect are what everything below reads, which is
    // what keeps `interact` a single-page function — the selection, the hit
    // test and the decomposition all describe one page, and that page is this
    // one.
    let acting = doc.view.page_index;
    let Some(active) = drawn
        .iter()
        .find(|d| d.page == acting)
        .or_else(|| drawn.first())
    else {
        // Nothing was laid out at all: a strip whose visible window fell
        // outside every page, which happens for one frame after a mode change
        // before the scroll area has settled. Say so, and let the next frame
        // sort it out rather than inventing a rect for a page nobody drew.
        crate::diag::trace_changed(trace::LAYOUT_SLOT, || {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "canvas-unavailable reason=nothing-visible".to_owned()
        });
        return (Vec::new(), None);
    };
    let image_response = active.response.clone();
    let image_rect = active.rect;
    let extent = viewer::page_extent_pts(&doc.pages[acting]);

    // ★ **Publish what the renderer should work on**, nearest the viewport
    // centre first — the order `render::settle` fills the strip in, and the
    // whole of why a scroll feels like it is keeping up rather than starting
    // from the top every time. Only knowable here, once the scroll area has
    // settled, which is the same reason `last_scroll_offset` is stored.
    let centres: Vec<(usize, f32)> = drawn.iter().map(|d| (d.page, d.rect.center().y)).collect();
    doc.strip_visible = strip::nearest_first(&centres, scroll_output.inner_rect.center().y);

    // The frame's screen⟷page map for the page being acted on. Built here,
    // immediately after the scroll area has settled and the page's true drawn
    // rect is known, and handed to everything below — nothing past this line
    // divides by the zoom for itself. See `mapping`'s header for why that
    // matters twice over.
    let map = PageMapping::new(image_rect, extent, doc.view.zoom);
    // …and one map per drawn page, for the Find wash, which has to land on
    // whichever page its hits are on rather than on the one being acted upon.
    // See `interact`'s step 8.
    let page_views: Vec<strip::PageView> = drawn
        .iter()
        .map(|d| strip::PageView {
            page: d.page,
            map: PageMapping::new(
                d.rect,
                viewer::page_extent_pts(&doc.pages[d.page]),
                doc.view.zoom,
            ),
        })
        .collect();

    // ★ The guides' catch bands, registered AFTER every page widget and before
    // the gesture layer runs. The order is the whole mechanism: a later widget
    // in the same layer is the topmost one under the pointer, so a press on a
    // guide never reaches the page's `Response` and therefore never reaches
    // the gesture machine — a guide drag cannot also rubber-band a selection.
    // See `guides`' header §3. Registers nothing when the toggle is off or the
    // document has no guides.
    guides::canvas_drag(ui, doc, &page_views, actions);

    // ★ Form filling on the page, registered in that same layer and for the
    // same reason: the focused field's editor must be topmost, and nothing is
    // registered for an unfocused one. See `forms`' header §4.
    forms::overlay(ui, doc, &page_views, &drawn, active_tool, actions);

    // ★ The frame's geometry, recorded for the commands that arrive with none.
    // A zoom raised from a keyboard chord, the ribbon or the status bar has no
    // `Ui` and no page rect, and it must describe its anchor against the view
    // as it stands BEFORE the zoom is applied — which is exactly this. See
    // [`zoom::CanvasFrame`].
    //
    // ★ The offset recorded is the **page-local** one, not the strip's. Every
    // consumer of this record — the anchor rule, both framing verbs — is
    // written for a scroll area holding one page at the origin, and converting
    // here is what lets all of them keep working unchanged over a strip. Under
    // `Single` the conversion is the identity. See `geometry`'s header.
    zoom::remember_frame(
        ui.ctx(),
        zoom::CanvasFrame {
            map,
            extent,
            display: (image_rect.width(), image_rect.height()),
            viewport: (viewport_size.x, viewport_size.y),
            viewport_rect: scroll_output.inner_rect,
            offset: geometry::page_local_offset(
                (scroll_offset.x, scroll_offset.y),
                (
                    image_rect.min.x - strip_rect.min.x,
                    image_rect.min.y - strip_rect.min.y,
                ),
                (display_size.x, display_size.y),
                (image_rect.width(), image_rect.height()),
                (viewport_size.x, viewport_size.y),
            ),
        },
    );

    // Selection, before the layout trace: the trace reports `sel=`, and a
    // count taken before the frame's click was applied would describe the
    // previous frame.
    let (selected, tokens) = interact(
        ui,
        doc,
        &image_response,
        &Frame {
            pen,
            map,
            pages: &page_views,
            clip: scroll_output.inner_rect,
            tool: active_tool,
            caps,
        },
        host,
        find,
        actions,
    );

    trace::layout(
        doc,
        image_rect,
        scroll_offset,
        selected,
        drawn.len(),
        drawn.iter().filter(|d| d.has_raster).count(),
    );
    crate::diag::ui_rect(trace::REGION_PAGE, image_rect);
    crate::diag::ui_rect(trace::REGION_CANVAS_VIEWPORT, scroll_output.inner_rect);
    trace::pointer(ui, doc, image_rect, extent);

    // Ctrl+wheel over the canvas: multiply the zoom. Gated on hover so a
    // Ctrl+wheel aimed at some other surface does not zoom the page out from
    // under the operator.
    if image_response.hovered() {
        let factor = ui.ctx().input(|i| i.zoom_delta());
        if (factor - 1.0).abs() > f32::EPSILON {
            // Zoom to cursor, half one: remember WHERE on the page the
            // pointer is before the zoom lands. Anchoring on the viewport
            // centre instead (which is what happens when nothing records
            // this) drags the detail being inspected out from under the
            // operator, worse the further off-centre they point — reported
            // as "jarring" on 2026-08-04.
            //
            // ★ Through [`zoom::arm_anchor`], the same call the discrete
            // commands make — which is what "the rule is decided once for all
            // four" means in code. The wheel used to build its own `ZoomAnchor`
            // inline from the pointer position; that inline version WAS the
            // rule, in a place no command could reach, and duplicating it at
            // three more call sites is how the four would have drifted apart.
            //
            // The pointer guard that used to live here has moved with it: a
            // pointer off-window (a trackpad pinch can produce exactly that)
            // falls back to the viewport centre rather than to nothing, and a
            // zero drawn size can no longer produce a NaN because
            // `zoom::frac_of` divides by the page EXTENT, which is finite and
            // positive for any page that drew at all.
            zoom::arm_anchor(ui.ctx(), doc);
            actions.push(Action::ZoomBy(factor));
        }
    }

    // ★ What the frame learned, handed outwards so the rulers can be drawn
    // against it. Only knowable here — after the scroll area has settled — for
    // the same reason `last_scroll_offset` is stored and `strip_visible` is
    // published during layout. See [`CanvasGeometry`].
    (
        tokens,
        Some(CanvasGeometry {
            pages: page_views,
            current: acting,
            viewport: scroll_output.inner_rect,
        }),
    )
}
