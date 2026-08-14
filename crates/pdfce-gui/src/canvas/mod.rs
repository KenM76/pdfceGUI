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
//! | [`gesture`] | press / drag / release, the clear that must not happen on a press, Escape's abort, and the one rubber band's two intents |
//! | [`moving`] | which move verb each rung reaches, the canvas→page delta, and the ghost's honesty rule |
//! | [`handles`] | eight grips plus move, and the cursor over each |
//! | [`menus`] | the right-click: which of the two canvas menus opens, and the select-first rule that makes it about the thing you pointed at |
//! | [`overlay`] | what all of it looks like — and what rule 4 forbids it looking like |
//! | [`geometry`] | the pan and zoom-anchor arithmetic |
//! | [`keys`] | Escape and Delete, and which of Escape's three claimants gets it |
//! | [`tool`] | select or hand, and the space bar that borrows the hand |
//! | [`zoom`] | **the anchor rule**, the two-frame handshake, and the five zoom paths that route through it |
//!
//! Everything above is pure except this file, [`overlay`], and [`moving`]'s
//! one wiring function ([`moving::drag`], which is the only thing there that
//! touches the live object model — its rules are pure). That is the
//! point: `PROJECT_PLAN.md`'s split is driven by testability, and the
//! selection invariants are exactly the kind of property a unit test can hold
//! and a running window cannot be trusted to demonstrate.
//!
//! ## ★ Selection survives navigation — the invariant this stage is accountable for
//!
//! `GUI_ROADMAP.md` Phase 1 states it and names three ways it is lost.
//! [`selection`]'s header carries the full table; the wiring's share of it is
//! visible right here in [`show`]:
//!
//! - the selection is **read, resolved and stored** on every frame, and the
//!   resolve does no work unless `(page, edit epoch)` moved — so a zoom, a
//!   pan, a fit change, a view rotation, a page-display change and a ribbon-tab
//!   change all cost one comparison and change nothing;
//! - the only thing that can clear it is [`gesture::GestureOutcome::Click`],
//!   which is raised on a **completed click with no drag**;
//! - a decomposition is built **only when a gesture needs one or the epoch
//!   moved** — never per frame, and never merely because the view changed.
//!
//! **A move does not threaten the invariant, and that is measured rather than
//! hoped.** `move_*` rewrites operator operands in place, adds and removes no
//! operator, and therefore renumbers nothing — `pdfce-core`'s
//! `object_identity_across_edits.rs` decomposes, edits and decomposes again to
//! prove it. So a committed move leaves every `Selection` untouched and only
//! the *outlines* have to catch up, which the epoch bump already handles. The
//! `delete_*` family is the one that renumbers. See [`moving`]'s header for the
//! full table.
//!
//! ## ★ The two seams that were wiring, and how they were closed
//!
//! Both were recorded here as *"one-line changes once the field they want
//! exists"*. The field exists; both are closed. They are written up rather
//! than deleted because each closed a live hazard, and the next person to
//! consider reopening one should have to read what it cost.
//!
//! 1. **The selection was held in `egui`'s `Memory`.** It is document-scoped
//!    state, and `Memory` outlives documents — so the canvas had to *detect*
//!    a document change with a `DocumentToken` built from the
//!    `Arc<EditSession>`'s **allocation address** mixed with the page count,
//!    compared once per frame. That is the same address-as-identity key
//!    `panels::DocKey` was deleted for earlier in this stage, with the same
//!    ABA hazard.
//!
//!    It now lives on [`crate::app::state::OpenDoc::selection`], and
//!    `DocumentToken` and `SelectionState::sync_document` are **deleted**.
//!    `OpenDoc::new`'s own argument does the work: constructing fresh state
//!    per document is what makes *"a cached value can never refer to a
//!    previous file"* true by construction, on every frame, with nothing to
//!    compare. The canvas still owns the selection in the sense that matters —
//!    it is the only writer — and the *edit* a Delete leads to is still an
//!    [`Action`] applied after the frame.
//!
//! 2. **The decomposition was built transiently, per gesture.** The Objects
//!    panel's cache moved onto `OpenDoc` earlier in this stage; until this
//!    change the canvas could not reach it, so `build_targets` called
//!    `ObjectModelProvider::build` for itself on every click and every marquee
//!    release — a **second** decomposition of a page the panel had already
//!    decomposed, which is the *"two decompositions quietly diverge"* failure
//!    decision 011 names.
//!
//!    [`interact`] now reads [`crate::app::state::OpenDoc::page_objects`], and
//!    `build_targets` is deleted. One decomposition per `(page, epoch)`, shared
//!    by the panel, the Properties row, the `objects n=` trace and the hit
//!    test — the same value, not two values that agree. [`show`] needs no
//!    provider argument, because the document it already takes carries it.
//!
//!    The gating is kept and still matters: the cache is asked for **only**
//!    when a gesture needs a hit test or when `(page, epoch)` has moved, so a
//!    zoom or a pan with the Objects panel closed still decomposes nothing.
//!    Drawing needs no provider at all — the outlines are cached in canvas
//!    space, which is zoom-independent.
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
//! removes nothing here, it raises [`Action::DeleteSelection`] carrying the
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

pub mod geometry;
pub mod gesture;
// Draggable alignment lines: what a guide belongs to, where it lives on disk,
// and why grabbing one cannot also start a marquee.
pub mod guides;
// The drawing grid, in each page's own space. Split from `rulers` under R2
// along the seam that module's header already drew: a ruler is chrome beside
// the canvas that reserves layout space, a grid is chrome over the page that
// reserves none.
pub mod grid;
pub mod handles;
// Reading this frame's pointer — what a click landed on at every rung, which
// of the two panning gestures is in flight, and where the in-flight press is
// kept between frames. Split out under R2 when the rulers landed; see its
// header on why the forced seam is a real one.
pub mod input;
// Escape and Delete, and the precedence between the three things that would
// like Escape. Split from this file along the seam every other split here
// follows: that module is drivable by a headless `egui::Context`, this one
// needs a window.
pub mod keys;
pub mod mapping;
// Drawing a markup annotation where the operator points: the rubber band, the
// four kinds it can author, and the raw endpoints an arrow's head depends on.
pub mod markup;
pub mod menus;
// Dragging a selection: which verb each rung reaches, the canvas→page delta,
// and the ghost's honesty rule. Kept out of `selection` deliberately — that
// module is already 1,352 lines and owns *what is selected*, while this owns
// *what happens when you drag it*.
pub mod moving;
pub mod overlay;
// The ruler gutters, the 1-2-5 tick ladder they and the grid share, and what
// unit the whole thing reads in. Its header carries the three decisions this
// feature turns on: the unit, the space the grid lives in, and why the
// reservation it takes out of the viewport is a constant (R128).
pub mod rulers;
pub mod selection;
// Which page the frame is about, in what order the rest should be drawn, and
// where a navigated-to page lands. The canvas's half of Phase 4's strip.
pub mod strip;
pub mod target;
// The three `PDFCE_DIAG` lines the canvas writes, and the shape contract
// `tools/ui-verify` reads them under.
pub mod trace;
// Which pointer tool the canvas is in — select or hand — and the space bar
// that borrows the hand for as long as it is held.
pub mod tool;
// The anchor rule, the two-frame handshake it rides on, and the five zoom
// paths that route through it.
pub mod zoom;

use egui::{Key, PointerButton, Pos2, Rect, Sense, scroll_area::ScrollSource, vec2};
use egui_shell::HandlerToken;

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::canvas::gesture::{GestureOutcome, MarqueeIntent, Phase, PointerFrame};
use crate::canvas::input::{load_gesture, pan_delta, probe, store_gesture};
use crate::canvas::mapping::PageMapping;
use crate::canvas::rulers::CanvasGeometry;
use crate::canvas::target::CanvasTargetProvider;
use crate::canvas::tool::CanvasTool;
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

/// The de-duplication slot every canvas-layout line shares.
///
/// One slot for all of them — the `canvas` line and both
/// `canvas-unavailable` variants — because they answer **one** question
/// ("where is the canvas, and is there one?"), and a consumer reads the
/// answer as the most recent line about it. Splitting them would let a stale
/// `canvas` line sit after the page stopped rendering, with nothing in the
/// trace to say the situation had changed.
const LAYOUT_SLOT: &str = "canvas"; // ui-text-exempt: trace slot name, never displayed

/// The de-duplication slot for the document-space pointer report.
///
/// Separate from [`LAYOUT_SLOT`]: the pointer moves constantly while the
/// layout does not, and sharing a slot would make each silence the other.
const POINTER_SLOT: &str = "canvas-pointer"; // ui-text-exempt: trace slot name, never displayed

/// Named region: the page raster's own rect, in window logical points.
///
/// This is the rect every canvas coordinate conversion is relative to, so it
/// is the one a screenshot oracle needs in order to crop the page out of a
/// window capture. See [`crate::diag::ui_rect`] on naming.
const REGION_PAGE: &str = "page"; // ui-text-exempt: trace region name, never displayed

/// Named region: the scrollable viewport the page sits inside.
///
/// Distinct from [`REGION_PAGE`], and the difference is exactly where the
/// old GUI's selection-offset defect lived (see the centring comment inside
/// [`show`]): at fit-page on a small page the two rects differ by the
/// centring margin, and a check that measured one while meaning the other
/// would sample the grey surround.
const REGION_CANVAS_VIEWPORT: &str = "canvas-viewport"; // ui-text-exempt: trace region name, never displayed

/// Named region: the one-sentence message shown instead of a page.
///
/// Shares a name across the no-pages and render-failed arms on purpose: it
/// is the same region of the screen serving the same purpose, and a
/// legibility check asking "is the canvas's explanatory text readable?"
/// should not have to enumerate every reason the text might be there.
const REGION_PAGE_MESSAGE: &str = "canvas-message"; // ui-text-exempt: trace region name, never displayed

/// Trace slot for what the selection layer did — a click, a marquee, an
/// Escape, a Delete.
///
/// Separate from [`LAYOUT_SLOT`] because the two answer different questions
/// and de-duplicate on different timescales: the layout line reports *where
/// the canvas is*, this one reports *what the operator just did to the
/// selection*. Sharing a slot would let each silence the other.
const SELECTION_SLOT: &str = "canvas-selection"; // ui-text-exempt: trace slot name, never displayed

/// The three facts about *this frame's canvas* that [`interact`] needs, and
/// that every part of it must agree on.
///
/// A struct rather than three parameters, and the reason is the same one that
/// made [`PageMapping`] a struct: these are facts about one frame, they are
/// settled together once the scroll area has laid out, and passing them
/// separately invites a call site to compute one of them for itself. `tool` is
/// the newest member and the most dangerous to re-derive — two readings of
/// "is the hand active?" within one frame that disagreed would be a drag that
/// panned **and** marquee'd, which is exactly what Phase 3.2 must not ship.
#[derive(Debug, Clone, Copy)]
struct Frame<'a> {
    /// The frame's ONE screen ⟷ canvas map, **for the page being acted on**.
    ///
    /// Every gesture, every hit test and every selection outline goes through
    /// this one. Phase 4 did not add a second: a strip shows several pages, but
    /// input is about exactly one of them, and which one is settled before this
    /// struct is built (see [`show`]).
    map: PageMapping,
    /// One map per page this frame drew, for the **Find wash** — the single
    /// thing that is legitimately about pages other than the one being acted
    /// on, because a search describes the whole document and its hits have to
    /// land on the pages they are on.
    ///
    /// Exactly one entry under [`viewer::PageDisplay::Single`], and it is the
    /// same page `map` describes.
    pages: &'a [strip::PageView],
    /// The scroll viewport, which is both the painter's clip rect and the
    /// region "is the pointer over the canvas?" is asked against.
    clip: Rect,
    /// What the primary button means this frame.
    tool: CanvasTool,
}

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
    actions: &mut Vec<Action>,
) -> Vec<HandlerToken> {
    let gutters = rulers::reserve(ui, doc.view.rulers && !doc.pages.is_empty());
    let mut content = gutters.content_ui(ui);
    let (tokens, geometry) = show_in(&mut content, doc, host, find, actions);
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
    actions: &mut Vec<Action>,
) -> (Vec<HandlerToken>, Option<CanvasGeometry>) {
    if doc.pages.is_empty() {
        let placeholder = ui.centered_and_justified(|ui| ui.label(crate::text::canvas_no_pages()));
        // Say so on the trace rather than staying silent. A consumer that
        // finds no `canvas` line otherwise has to guess between "this build
        // does not trace its layout" and "there was no layout to trace", and
        // those need opposite responses. See `trace_layout`.
        crate::diag::trace_changed(LAYOUT_SLOT, || {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "canvas-unavailable reason=no-pages".to_owned()
        });
        crate::diag::ui_rect(REGION_PAGE_MESSAGE, placeholder.inner.rect);
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
        crate::diag::trace_changed(LAYOUT_SLOT, || {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "canvas-unavailable reason=render-failed".to_owned()
        });
        crate::diag::ui_rect(REGION_PAGE_MESSAGE, placeholder.inner.rect);
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
    let raster_scale = viewer::raster_scale(doc.view.zoom, pixels_per_point);

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
        crate::diag::trace_changed(LAYOUT_SLOT, || {
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
            map,
            pages: &page_views,
            clip: scroll_output.inner_rect,
            tool: active_tool,
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
    crate::diag::ui_rect(REGION_PAGE, image_rect);
    crate::diag::ui_rect(REGION_CANVAS_VIEWPORT, scroll_output.inner_rect);
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

/// Read the frame's canvas gestures and keys, update the selection, draw it,
/// and return how many entries are selected (for the trace line) together
/// with any context-menu commands the operator invoked.
///
/// # The order of the steps, and why it is this order
///
/// 1. **Read the pointer**, converting once through `map` — the boundary.
///    Escape rides in on the same frame, so a drag in flight can abort.
/// 2. **Decide what a press would land on**, from the *previous* frame's
///    selection. A grip is a target because it is already on screen.
/// 3. **Advance the gesture machine.** A press produces nothing; only a
///    completed click, a released marquee, a move drag or an Escape-abort
///    produces an outcome.
/// 4. **Build a decomposition only if something needs one.** A click, a
///    released marquee, **a right-click**, **a move drag**, or a
///    `(page, epoch)` that moved. Never merely because the view changed — that
///    is the invariant, and it is this `if`.
/// 5. **Apply the outcome**, then **the right-click**, then **re-resolve**,
///    then **draw** (outlines, then the marquee, then the move ghost).
/// 6. **Keys**, guarded by `text_edit_focused()` — `DEFECTS.md` D1 — and by
///    whether step 3 already spent the Escape on a drag.
///
/// Step 4 sitting *after* step 3 is what makes the whole thing affordable:
/// the expensive work is behind the gesture, not in front of it.
///
/// # ★ Why Escape is read in step 1 and honoured in step 6
///
/// Two things want the key and exactly one may have it per press: a drag in
/// flight wants to abandon itself, and the selection ladder wants to ascend a
/// rung. The gesture machine gets first refusal because it is the only thing
/// that knows whether there *is* a drag under the press — it takes the key only
/// when there is, and reports [`gesture::GestureOutcome::Cancelled`] when it
/// does. Step 6 passes that on to [`canvas_keys`], so a cancelled drag does not
/// also cost the operator the rung they were working in. With no drag in
/// flight, nothing is consumed and Escape ascends exactly as it always did.
///
/// # ★ Where the right-click sits, and why it is not step 5's business
///
/// The secondary button never reaches [`gesture`]: that machine reads
/// `PointerButton::Primary` throughout, deliberately, because the middle
/// button pans and the primary button owns press/drag/release. A right-click
/// is not a gesture with a beginning and an end — it is a single event that
/// asks a question — so it is read straight off the `Response` in step 5b,
/// *after* the primary gesture has been applied and *before* the resolve.
///
/// Before the resolve matters: a right-click over an unselected object
/// **selects it** ([`menus::select_under_right_click`]), and the outlines
/// drawn in step 8 have to be the new selection's or the highlight would lag
/// the menu by a frame — the operator would see a menu about an object that
/// is not yet outlined, which is the "which of these is it about?" ambiguity
/// the select-first rule exists to remove.
///
/// # ★ Why the selection is moved out of the document and back again
///
/// The selection now lives on [`OpenDoc`], and the decomposition it resolves
/// against is a [`std::cell::Ref`] **borrowed from the same `OpenDoc`**. Those
/// two cannot be held at once through one `&mut` — a `Ref` keeps a shared
/// borrow of the whole document alive, and mutating the selection in place
/// would need a mutable one.
///
/// So the selection is taken by value at the top and put back at the bottom.
/// That is not a workaround for the borrow checker so much as an honest
/// statement of what this function does: it computes *the next* selection from
/// the previous one and the frame's input, and stores it. Two further
/// properties fall out, both of which the `egui::Memory` version relied on and
/// documented:
///
/// * **nothing is cloned.** A marquee over a dense sheet can select thousands
///   of entries, and cloning that per frame at 60 Hz would be a real cost for
///   no reason;
/// * **a frame that panicked between the take and the put leaves an empty
///   selection**, not a half-updated one — a state the operator can see and
///   recover from with one click.
///
/// # ★ The hand tool suppresses the whole of step 1, and that is the fix
///
/// When [`tool::active`] reports `Hand` — chosen, or borrowed by a held space
/// bar — the primary button pans, and a `PointerFrame` describing that drag
/// would make it marquee **as well**. So the frame is built **blank** apart
/// from Escape: no press, no drag, no click, no position. Not "the marquee arm
/// checks the tool", not "the selection is restored afterwards" — the gesture
/// simply is not offered, which is the only version of this that cannot leave
/// a half-applied selection behind if a future arm forgets to ask.
///
/// A drag already in flight when the space bar goes down is therefore
/// *interrupted*, and [`gesture::GestureState::update`]'s last branch already
/// knows what to do with that: abandon it, commit nothing. An operator who
/// starts a marquee and then reaches for space has changed their mind, and the
/// worst outcome available is that nothing happened.
fn interact(
    ui: &mut egui::Ui,
    doc: &mut OpenDoc,
    response: &egui::Response,
    frame_ctx: &Frame,
    host: Option<&MenuHost<'_>>,
    find: &crate::find::FindState,
    actions: &mut Vec<Action>,
) -> (usize, Vec<HandlerToken>) {
    // Destructured through the reference, so `map` stays a borrow — it is
    // handed on to the overlay and the probe, both of which take one.
    let Frame {
        map,
        pages,
        clip,
        tool: active_tool,
    } = frame_ctx;
    let (clip, active_tool) = (*clip, *active_tool);
    let ctx = ui.ctx().clone();
    let page_index = doc.view.page_index;

    // Out of the document for the duration of the frame's gesture — see this
    // function's docs. The `&mut doc` borrow ends on this line, which is what
    // lets the decomposition below be borrowed out of `doc` at all.
    //
    // ★ Notice what is NOT here: no document token is compared, because none
    // exists. A selection belongs to the `OpenDoc` it is a field of, so a
    // different document is a different `OpenDoc` carrying its own empty one.
    // A page change is not a document change and neither is an edit — both
    // re-resolve at step 7. See `selection`'s invariant 3.
    let mut selection = std::mem::take(&mut doc.selection);
    let mut gestures = load_gesture(&ctx);

    // ---- 1. the pointer, converted once ------------------------------
    //
    // `interact_pointer_pos` first: during a drag it keeps reporting even
    // after the pointer has left the widget, which is exactly the case a
    // rubber-band dragged off the page depends on. `pointer_latest_pos` is
    // the hover fallback, which is what the grip cursor needs.
    let screen_pos = response
        .interact_pointer_pos()
        .or_else(|| ctx.pointer_latest_pos());
    let shift = ctx.input(|i| i.modifiers.shift);
    // Escape is read whatever the tool is: a hand-tool frame still has to be
    // able to abandon a drag the previous tool left in flight, and it is still
    // the ladder's key when there is no drag. See this function's header.
    let cancel = !ctx.text_edit_focused() && ctx.input(|i| i.key_pressed(Key::Escape));
    let frame = if active_tool.pans_with_primary() {
        PointerFrame {
            cancel,
            ..PointerFrame::default()
        }
    } else {
        PointerFrame {
            // `..._by(PointerButton::Primary)` throughout: the bare forms are
            // button-agnostic, and the middle button is the pan. See `gesture`.
            drag_started: response.drag_started_by(PointerButton::Primary),
            dragging: response.dragged_by(PointerButton::Primary),
            drag_stopped: response.drag_stopped_by(PointerButton::Primary),
            clicked: response.clicked_by(PointerButton::Primary),
            double_clicked: response.double_clicked_by(PointerButton::Primary),
            pos: screen_pos.map(|p| map.to_page(p)),
            // ★ Where the button actually went down, through the frame's ONE
            // map. Without it every drag begins wherever the pointer had got to
            // by the frame egui called it a drag — measured at 94 page points
            // on an A1 sheet. See `gesture::PointerFrame::press_origin`.
            press_origin: ctx
                .input(|i| i.pointer.press_origin())
                .map(|p| map.to_page(p)),
            shift,
            // Escape reaches the gesture machine BEFORE it reaches the ladder,
            // so a drag in flight can abandon itself. The machine consumes the
            // key only if there was a drag to abandon, and says so by returning
            // `Cancelled` — which is what step 6 passes on to `canvas_keys`.
            cancel,
        }
    };

    // ---- 2. what a press would land on -------------------------------
    //
    // The armed tool and the marquee's INTENT are sampled here, at press time,
    // alongside what the press landed on — see `gesture`'s header on why a
    // release must not re-read either. The precedence between the four answers
    // lives in `gesture::press_kind`, which is pure and tested; this is the one
    // call that supplies it with the frame's facts.
    let grip_box = overlay::grip_box(map, &selection);
    let hovered_grip = grip_box
        .zip(screen_pos)
        .and_then(|(bounds, p)| handles::grip_at(bounds, p));
    let press_kind = gesture::press_kind(active_tool, hovered_grip, zoom::region_zoom_armed(&ctx));

    // ---- 3. advance the gesture --------------------------------------
    let outcome = gestures.update(frame, press_kind);

    // ---- 4. the decomposition, only if a HIT TEST needs one -------------
    //
    // ★ `doc.page_objects()` — THE decomposition, the one the Objects panel
    // lists and the `objects n=` trace counts. The canvas used to build its
    // own here (module docs, seam 2); that second `decompose_page` over the
    // same page is gone.
    //
    // The gate is kept even though the value is now cached, and it still buys
    // something real: `page_objects()` builds on first use, so asking for it
    // on a frame that has no hit test to do would decompose the page the first
    // time the operator merely zoomed — with the Objects panel closed, that is
    // a whole content-stream walk bought by a mouse wheel. Drawing needs no
    // provider at all (the outlines are cached in canvas space, which is
    // zoom-independent), so on the overwhelming majority of frames nothing is
    // asked for and nothing is built.
    //
    // Not "if a resolve needs one" — see step 6 for why the resolve's build
    // has to happen after the keys rather than before them.
    //
    // A **right-click** is in this set for the same reason a click is: it has
    // to know what is under the pointer in order to select it, and a menu
    // about the wrong object is worse than no menu. It is a rare event, so
    // the gate still buys what it always bought — a zoom or a pan with the
    // Objects panel closed decomposes nothing.
    // A **move drag** is in the set too, at either phase, and it is the one
    // member that is not a hit test — which is why the flag is named for what
    // it gates rather than for what most of its members do. It needs the model
    // to answer two questions the selection alone cannot: *is every selected
    // object a path* (a non-path refuses the whole move, and a ghost drawn over
    // one would promise a move that gets refused), and, at the Node rung,
    // *where is the anchor now* (`move_node` takes a destination, not a delta).
    //
    // Asking on every frame of an in-flight drag is affordable because the
    // answer is already built: the selection cannot have outlines to drag
    // without a decomposition, so `page_objects()` is a cache hit for the
    // whole gesture. The gate still buys what it always bought — a zoom or a
    // pan with nothing being dragged decomposes nothing.
    //
    // ★ A **zoom** marquee is deliberately NOT in the set. It selects nothing,
    // so it hit-tests nothing, so it decomposes nothing — a region zoom over a
    // 129,758-object drawing costs one scroll offset. That falls out of the
    // intent being carried on the outcome rather than being asked for at the
    // release, and it is the concrete payoff for sampling it at the press.
    let secondary_clicked = response.secondary_clicked();
    let needs_targets = secondary_clicked
        || matches!(
            outcome,
            GestureOutcome::Click { .. }
                | GestureOutcome::Move { .. }
                | GestureOutcome::Marquee {
                    phase: Phase::Complete,
                    intent: MarqueeIntent::Select,
                    ..
                }
        );
    let mut targets = if needs_targets {
        doc.page_objects()
    } else {
        None
    };

    // ---- 5. apply the gesture -----------------------------------------
    let mut marquee = None;
    let mut ghost = None;
    let mut band = None;
    let mut zoom_region = None;
    match outcome {
        GestureOutcome::Click {
            point,
            shift,
            double,
        } => {
            let hit = targets
                .as_ref()
                .map(|t| probe(&**t, &selection, page_index, point, map))
                .unwrap_or_default();
            selection.click(page_index, hit, shift, double);
            trace::selection_event(&selection, "click", double);
        }
        GestureOutcome::Marquee {
            rect,
            shift,
            intent: MarqueeIntent::Select,
            phase: Phase::Complete,
        } => {
            let hits = targets
                .as_ref()
                .map(|t| t.hit_test_rect(page_index, rect))
                .unwrap_or_default();
            selection.marquee(page_index, &hits, shift);
            trace::selection_event(&selection, "marquee", shift);
        }
        // ★ The same rubber band, released with the other intent. **The
        // selection is not touched** — not cleared, not replaced, not even
        // read: a navigation gesture that rearranged the selection would break
        // the invariant this whole stage is accountable for, and the way to not
        // break it is to have no line of code here that could. `zoom_to_rect`
        // takes the band in canvas space, which is the space it already
        // arrived in.
        //
        // Disarmed on release, not on press: the one-shot has to survive the
        // whole drag, or an Escape-cancelled zoom marquee would leave the
        // operator with a disarmed tool and nothing to show for it.
        GestureOutcome::Marquee {
            rect,
            intent: MarqueeIntent::Zoom,
            phase: Phase::Complete,
            ..
        } => {
            zoom::disarm_region_zoom(&ctx);
            // Deferred to step 7b for one structural reason: `targets` is a
            // `Ref` borrowed out of `doc`, and arming an anchor needs `&mut
            // doc`. The same constraint the `drop(targets)` below exists for,
            // and the same answer the marquee and the ghost already use —
            // decide here, act once the borrow has ended.
            zoom_region = Some(rect);
        }
        GestureOutcome::Marquee {
            rect,
            phase: Phase::InFlight,
            ..
        } => marquee = Some(rect),
        // ★ The move. `moving::drag` owns every rule — which verb the rung
        // reaches, whether the operands qualify, the canvas→page delta — and
        // returns the ghost's canvas-space offset when, and only when, the
        // release would commit. Nothing about the move is decided here, on
        // purpose: this arm is wiring, and the rules are unit-tested without a
        // window in `moving`.
        GestureOutcome::Move { delta, phase } => {
            ghost = moving::drag(
                delta,
                phase,
                &selection,
                page_index,
                targets.as_deref(),
                doc.current_page(),
                actions,
            );
        }
        // ★ The markup band. `markup::drag` owns every rule — the canvas→page
        // conversion, the degenerate-drag refusal, which endpoints stay raw —
        // and hands back a band only when the release would commit, which is
        // the same honesty contract the move ghost is held to. Nothing about a
        // markup is decided here: this arm is wiring, and the rules are
        // unit-tested without a window in `markup`. Note what it does NOT need:
        // a decomposition. A markup hit-tests nothing, which is why this
        // outcome is absent from `needs_targets` above.
        GestureOutcome::Markup {
            kind,
            from,
            to,
            phase,
        } => {
            band = markup::drag(
                kind,
                from,
                to,
                phase,
                page_index,
                doc.current_page(),
                actions,
            );
        }
        // A resize drag is CONSUMED and commits nothing. Consuming it is the
        // load-bearing part: without it the drag would fall through to a
        // marquee, so aiming at a grip would replace the selection the operator
        // was trying to act on. `pdfce-core` has no scale verb, so there is
        // nothing to commit and — see `overlay::draw_move_ghost` — nothing to
        // preview either. See `handles` for the whole argument.
        //
        // `Cancelled` lands here too: an abandoned drag draws nothing, commits
        // nothing, and its only remaining effect is to keep Escape away from
        // the ladder at step 6.
        GestureOutcome::Resize { .. } | GestureOutcome::Cancelled | GestureOutcome::Idle => {}
    }

    // ---- 5b. the right-click ---------------------------------------------
    //
    // ★ The hit test is deliberately the OBJECT rung only — `hit_test`, not
    // `probe`. `probe` also asks for the nearest part and node so that a
    // double-click can descend, and a right-click never descends: it names a
    // whole object, because the verbs a context menu offers act on whole
    // objects. Asking for the deeper rungs here would pay for two extra
    // provider queries on every right-click and then discard both.
    //
    // `screen_pos` rather than the gesture's own `pos`: the `PointerFrame`
    // was consumed by the gesture machine above, and re-deriving the page
    // point from the same `screen_pos` through the same `map` is the frame's
    // ONE conversion applied twice, not a second conversion.
    //
    // `attach` is called on EVERY frame, not only on the frame of the click,
    // because `egui` draws an open popup until it is dismissed and a popup
    // only exists while something is attached to the response. On a frame
    // with no secondary click and nothing open it does nothing at all.
    let right_clicked_object = if secondary_clicked {
        targets.as_ref().and_then(|t| {
            screen_pos.and_then(|p| t.hit_test(page_index, map.to_page(p), map.tolerance()))
        })
    } else {
        None
    };
    let tokens = menus::attach(
        response,
        &mut selection,
        page_index,
        right_clicked_object,
        host,
    );

    // ---- 6. keys, BEFORE the resolve -----------------------------------
    //
    // Escape ascends a rung, which changes which box each entry outlines —
    // the part's, or the object's. Running the keys after the resolve would
    // leave the outline one frame behind the rung, visible as the part's box
    // lingering for a frame after the operator stepped out of it.
    //
    // `Cancelled` at step 3 means Escape was spent abandoning a drag, and one
    // press may have one effect: the ladder must not also ascend, or an
    // operator who cancels a move drag finds they have left the rung they were
    // working in as well.
    keys::canvas_keys(
        &ctx,
        &mut selection,
        page_index,
        actions,
        matches!(outcome, GestureOutcome::Cancelled),
    );

    // ---- 7. re-resolve -------------------------------------------------
    //
    // ★ A decomposition is attempted whenever a resolve is due, and `resolve`
    // records the key either way. Both halves matter and the pairing is easy
    // to get wrong:
    //
    // * passing `None` when no decomposition was ATTEMPTED would clear the
    //   outlines and then record the key, so they would never come back —
    //   a selection that is still selected and no longer drawn;
    // * not recording the key on a failed attempt would re-decompose an
    //   undecodable page on every single frame.
    // `!needs_targets` rather than `targets.is_none()`: a build that was
    // already attempted and failed must not be attempted a second time in the
    // same frame.
    if !needs_targets && selection.needs_resolve(page_index, doc.edit_epoch) {
        targets = doc.page_objects();
    }
    selection.resolve(
        targets.as_ref().map(|t| &**t as &dyn CanvasTargetProvider),
        page_index,
        doc.edit_epoch,
    );

    // The decomposition is a `Ref` into `doc`, and `Ref` implements `Drop` —
    // so its borrow does NOT end at its last use, and the store at the bottom
    // of this function needs `&mut doc`. Dropped explicitly rather than by
    // shuffling the code into a block, because the ordering is a real
    // constraint worth naming: nothing below this line may read the
    // decomposition.
    drop(targets);

    // ---- 7b. the released zoom marquee ----------------------------------
    //
    // Here rather than in its arm because arming an anchor writes to `doc` and
    // the decomposition's `Ref` has only just been released. The outcome is
    // reported by `zoom` on the diagnostic channel; there is nothing for this
    // frame to do with it, because a region zoom that hit the raster ceiling
    // still zooms — to the ceiling, centred on the region — and the status
    // bar's readout states the scale that was actually pinned. See
    // [`zoom::ZoomOutcome::ceiling_changed_the_answer`].
    if let Some(rect) = zoom_region {
        let _ = zoom::zoom_to_rect(&ctx, doc, rect, CANVAS_MARGIN, actions);
    }

    // ---- 8. draw --------------------------------------------------------
    let painter = ui.painter().with_clip_rect(clip);
    // ★ The grid goes UNDER everything, including the find wash. It is the
    // only thing painted here that is about the *paper* rather than about
    // something the operator has selected, searched for or is dragging, so
    // anything drawn over it is a statement about the drawing and must win.
    // Draws nothing at all with the toggle off. See `rulers`' header §2 for
    // why it is per page rather than across the viewport.
    if doc.view.grid {
        grid::draw(ui, doc, pages, clip);
    }
    // ★ The find highlights go on FIRST, under everything else.
    //
    // They are a wash over page content — an answer to "where is the text I
    // asked about" — while the selection outline is a statement about what a
    // verb would act on. Painting the wash over the outline would dim the
    // control feedback with a hint; painting it under leaves both readable.
    //
    // `page_highlights` yields nothing at all when the results are not current
    // — a stale epoch, a query the operator has edited, a closed bar — so an
    // edit stops the highlights by supplying an empty iterator rather than by
    // a check here. That is what keeps rule 4: this file cannot paint a mark
    // over content the search no longer describes, because it is never handed
    // one. See `crate::find`'s staleness section.
    //
    // ★ **Once per drawn page, each through its own map** — the one place the
    // canvas is legitimately about pages other than the one being acted on. A
    // search describes the whole document, so under a continuous mode its hits
    // are on several of the pages on screen at once, and painting them all
    // through the acting page's map would stack every page's highlights onto
    // one page. That is the failure this feature was most likely to ship
    // silently: the hits are found, the wash is drawn, and it is drawn in the
    // wrong place — which looks like a highlight bug rather than a mapping one.
    //
    // The loop reduces to exactly the previous call under `Single`, where
    // `pages` holds one entry and it is the acting page.
    for view in *pages {
        overlay::draw_find_hits(
            &painter,
            ui.visuals(),
            &view.map,
            find.page_highlights(view.page, doc.edit_epoch),
        );
    }
    overlay::draw_selection(&painter, ui.visuals(), map, &selection);
    if let Some(rect) = marquee {
        overlay::draw_marquee(&painter, ui.visuals(), map, rect);
    }
    // The ghost sits ON TOP of the real outline, and both stay visible: the
    // pair is what states the displacement. `ghost` is `Some` only when
    // `moving::drag` has already established that the release will commit — a
    // preview of a move that will be refused is the thing rule 4 and the
    // no-placeholders invariant both forbid.
    // The guides sit on TOP of the selection, and the order is the point: a
    // guide is a line the operator has to see while they align something to
    // it, and a selection outline is a box a few points across that a hairline
    // crossing it does not hide. The reverse order would hide a guide behind
    // exactly the object the operator is aligning to it.
    guides::draw(ui, doc, pages, clip);
    if let Some(delta) = ghost {
        overlay::draw_move_ghost(&painter, ui.visuals(), map, &selection, delta);
    }
    // Last, and over everything: the band IS the cursor for as long as it
    // exists, and a guide or an outline drawn over the shape being authored
    // would obscure the one thing the operator is aiming.
    if let Some(band) = band {
        markup::draw_preview(&painter, map, band);
    }

    // ★ The cursor — the whole precedence in one pure function, in `tool`,
    // where the first rung of it already lived. See [`tool::cursor_for`]; this
    // is only the gathering of the four facts that are knowable nowhere but
    // here. `clip` is the scroll VIEWPORT rather than the page's rect, because
    // the hand pans the grey surround as readily as the paper.
    let pointer_down = ctx.input(|i| i.pointer.primary_down() || i.pointer.middle_down());
    let over_canvas = ctx.pointer_latest_pos().is_some_and(|p| clip.contains(p));
    if let Some(icon) = tool::cursor_for(
        active_tool,
        gestures.active(),
        hovered_grip.filter(|_| response.hovered()),
        pointer_down,
        over_canvas,
    ) {
        ctx.set_cursor_icon(icon);
    }

    let count = selection.len();
    // Back onto the document, by value. Moved rather than cloned: a marquee
    // over a dense sheet can select thousands of entries, and cloning that per
    // frame at 60 Hz would be a real cost for no reason.
    doc.selection = selection;
    store_gesture(&ctx, gestures);
    (count, tokens)
}
