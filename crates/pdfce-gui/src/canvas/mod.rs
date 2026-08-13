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
pub mod handles;
// Escape and Delete, and the precedence between the three things that would
// like Escape. Split from this file along the seam every other split here
// follows: that module is drivable by a headless `egui::Context`, this one
// needs a window.
pub mod keys;
pub mod mapping;
pub mod menus;
// Dragging a selection: which verb each rung reaches, the canvas→page delta,
// and the ghost's honesty rule. Kept out of `selection` deliberately — that
// module is already 1,352 lines and owns *what is selected*, while this owns
// *what happens when you drag it*.
pub mod moving;
pub mod overlay;
pub mod selection;
pub mod target;
// Which pointer tool the canvas is in — select or hand — and the space bar
// that borrows the hand for as long as it is held.
pub mod tool;
// The anchor rule, the two-frame handshake it rides on, and the five zoom
// paths that route through it.
pub mod zoom;

use egui::{Key, PointerButton, Pos2, Rect, Sense, Vec2, scroll_area::ScrollSource, vec2};
use egui_shell::HandlerToken;

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::canvas::gesture::{
    DragKind, GestureOutcome, GestureState, MarqueeIntent, Phase, PointerFrame,
};
use crate::canvas::mapping::PageMapping;
use crate::canvas::selection::{ClickHit, SelectionState};
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

/// `egui::Memory` key for the in-flight pointer gesture.
///
/// ★ **The one thing that stayed in `Memory` when the selection left**, and the
/// distinction is the point rather than an omission. A gesture is genuinely
/// frame-local UI state — the drag that is happening *right now* — not
/// document-scoped state. It has no meaning across a document, and a gesture
/// that survived one would be a drag continuing over a file it did not start
/// on. Keying it here means it cannot: `Memory` is per-`Context`, and every
/// document change starts the next frame with no press in flight.
const GESTURE_MEMORY_KEY: &str = "pdfce-canvas-gesture"; // ui-text-exempt: internal memory id, never displayed

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
struct Frame {
    /// The frame's ONE screen ⟷ canvas map.
    map: PageMapping,
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
#[must_use]
pub fn show(
    ui: &mut egui::Ui,
    doc: &mut OpenDoc,
    host: Option<&MenuHost<'_>>,
    actions: &mut Vec<Action>,
) -> Vec<HandlerToken> {
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
        return Vec::new();
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
    let extent = doc.current_extent();
    let pixels_per_point = ui.ctx().pixels_per_point();
    let max_zoom = viewer::max_zoom_for_page(extent, pixels_per_point);
    doc.view.apply_fit(extent, viewport, max_zoom);

    if let Some(message) = &doc.render_error {
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
        return Vec::new();
    }

    let display_size = vec2(extent.0 * doc.view.zoom, extent.1 * doc.view.zoom);
    let texture = doc.page_texture.as_ref().map(|t| t.texture.clone());

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
    if let Some(offset) = zoom::consume_anchor(ui.ctx(), doc, (display_size.x, display_size.y)) {
        scroll_area = scroll_area.scroll_offset(offset);
    } else if let Some(pan) = pan_delta(ui, active_tool) {
        // Panning subtracts the pointer delta: the content follows the hand,
        // so the page moves WITH the pointer rather than under it.
        let vp = ui.available_size();
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
        // Centre the page MANUALLY rather than with
        // `ui.centered_and_justified`, because that helper returns the
        // JUSTIFIED CONTAINER rect — the whole available area — while
        // drawing the image centred inside it. Taking that rect as
        // `image_rect` makes every page↔screen mapping wrong by the centring
        // margin whenever the page is smaller than the viewport.
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
        // So: reserve `max(page, viewport)` so the ScrollArea still scrolls
        // when the page is larger AND there is a margin to centre within
        // when it is smaller, then place the image at an explicit centred
        // rect. `Ui::put`/`allocate_rect` return a Response whose `.rect` IS
        // that rect, so `image_rect` is the page's true drawn rect by
        // construction rather than by coincidence.
        //
        // S0 draws no overlay, so nothing depends on that rect yet. It is
        // built correctly now because the alternative is discovering at S4
        // that the substrate every overlay sits on has a margin-sized error
        // in it — which is precisely how the old GUI got there.
        let avail = ui.available_size();
        let outer = vec2(display_size.x.max(avail.x), display_size.y.max(avail.y));
        let (outer_rect, _) = ui.allocate_exact_size(outer, Sense::hover());
        let page_rect = Rect::from_center_size(outer_rect.center(), display_size);
        // `click_and_drag`, not `hover`: the page is the selection surface.
        // Both branches must agree — a first frame that reserved the space
        // with a different sense would swallow the click that opened the
        // document, and the operator would experience it as "the first click
        // never works".
        let sense = Sense::click_and_drag();
        let response = if let Some(texture) = texture {
            ui.put(
                page_rect,
                egui::Image::from_texture(&texture)
                    .fit_to_exact_size(display_size)
                    .sense(sense),
            )
        } else {
            // First frame after an open: the texture is made at the END of
            // this frame (see `PdfceApp::update`). Reserve the page's space —
            // same rect, same sense — so nothing jumps when it arrives.
            ui.allocate_rect(page_rect, sense)
        };
        // `avail` rides out with the response because it is the viewport the
        // zoom-to-cursor solve needs, and it is only knowable in here — the
        // same `avail` that decided `outer` above, so the margin the solve
        // reconstructs is the margin this frame actually drew.
        (response, avail)
    });

    let (image_response, viewport_size) = scroll_output.inner;
    // The offset the area settled on THIS frame: the `offset_before` of any
    // zoom step the operator starts now, and the base the next frame's
    // middle-drag pan moves from.
    doc.last_scroll_offset = scroll_output.state.offset;
    let scroll_offset = scroll_output.state.offset;
    let image_rect = image_response.rect;

    // The frame's ONE screen⟷page map. Built here, immediately after the
    // scroll area has settled and the page's true drawn rect is known, and
    // handed to everything below — nothing past this line divides by the zoom
    // for itself. See `mapping`'s header for why that matters twice over.
    let map = PageMapping::new(image_rect, extent, doc.view.zoom);

    // ★ The frame's geometry, recorded for the commands that arrive with none.
    // A zoom raised from a keyboard chord, the ribbon or the status bar has no
    // `Ui` and no page rect, and it must describe its anchor against the view
    // as it stands BEFORE the zoom is applied — which is exactly this. See
    // [`zoom::CanvasFrame`].
    zoom::remember_frame(
        ui.ctx(),
        zoom::CanvasFrame {
            map,
            extent,
            display: (display_size.x, display_size.y),
            viewport: (viewport_size.x, viewport_size.y),
            viewport_rect: scroll_output.inner_rect,
            offset: (scroll_offset.x, scroll_offset.y),
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
            clip: scroll_output.inner_rect,
            tool: active_tool,
        },
        host,
        actions,
    );

    trace_layout(doc, image_rect, scroll_offset, selected);
    crate::diag::ui_rect(REGION_PAGE, image_rect);
    crate::diag::ui_rect(REGION_CANVAS_VIEWPORT, scroll_output.inner_rect);
    trace_pointer(ui, doc, image_rect, extent);

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

    tokens
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
    actions: &mut Vec<Action>,
) -> (usize, Vec<HandlerToken>) {
    // Destructured through the reference, so `map` stays a borrow — it is
    // handed on to the overlay and the probe, both of which take one.
    let Frame {
        map,
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
    // The marquee's INTENT is sampled here, at press time, alongside what the
    // press landed on — see `gesture`'s header on why a release must not
    // re-read it. A grip drag is never a zoom: the grips belong to a selection,
    // and a region zoom is about the paper.
    let grip_box = overlay::grip_box(map, &selection);
    let hovered_grip = grip_box
        .zip(screen_pos)
        .and_then(|(bounds, p)| handles::grip_at(bounds, p));
    let press_kind = match hovered_grip {
        Some(grip) if grip.is_resize() => DragKind::Resize(grip),
        Some(_) => DragKind::Move,
        None if zoom::region_zoom_armed(&ctx) => DragKind::Marquee(MarqueeIntent::Zoom),
        None => DragKind::Marquee(MarqueeIntent::Select),
    };

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
            trace_selection_event(&selection, "click", double);
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
            trace_selection_event(&selection, "marquee", shift);
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
    overlay::draw_selection(&painter, ui.visuals(), map, &selection);
    if let Some(rect) = marquee {
        overlay::draw_marquee(&painter, ui.visuals(), map, rect);
    }
    // The ghost sits ON TOP of the real outline, and both stay visible: the
    // pair is what states the displacement. `ghost` is `Some` only when
    // `moving::drag` has already established that the release will commit — a
    // preview of a move that will be refused is the thing rule 4 and the
    // no-placeholders invariant both forbid.
    if let Some(delta) = ghost {
        overlay::draw_move_ghost(&painter, ui.visuals(), map, &selection, delta);
    }

    // The cursor states what the gesture is, and an in-flight gesture keeps
    // its own cursor even once the pointer has wandered off the thing it
    // started on — otherwise a drag that outruns its object looks like it
    // stopped working.
    //
    // ★ The hand comes first, and it is the whole of "the cursor must change,
    // and must change back". It changes because this branch is taken while the
    // tool is active; it changes back because the branch is re-evaluated every
    // frame from `tool::active` and there is no stored cursor to restore. A
    // dropped key-up costs one frame of hand, not a canvas stuck showing a
    // grab cursor over a select tool.
    //
    // Measured against `clip` — the scroll VIEWPORT — rather than the page's
    // own rect, because the hand pans the grey surround as readily as the
    // paper, and a hand tool that shows no hand over half the canvas reads as
    // a tool that is not armed.
    let hand_dragging = ctx.input(|i| i.pointer.primary_down() || i.pointer.middle_down());
    let over_canvas = ctx.pointer_latest_pos().is_some_and(|p| clip.contains(p));
    if let Some(icon) = active_tool
        .cursor(hand_dragging)
        .filter(|_| over_canvas || hand_dragging)
    {
        ctx.set_cursor_icon(icon);
    } else if let Some(kind) = gestures.active() {
        ctx.set_cursor_icon(match kind {
            // One crosshair for both intents: the band is the same band and
            // `gesture`'s header refuses a second set of pixels for it. What
            // tells the operator a zoom is armed is the ribbon control that
            // armed it, off-canvas, where a mode indicator belongs.
            DragKind::Marquee(_) => egui::CursorIcon::Crosshair,
            DragKind::Move => egui::CursorIcon::Grabbing,
            DragKind::Resize(grip) => grip.cursor(),
        });
    } else if response.hovered()
        && let Some(grip) = hovered_grip
    {
        ctx.set_cursor_icon(grip.cursor());
    }

    let count = selection.len();
    // Back onto the document, by value. Moved rather than cloned: a marquee
    // over a dense sheet can select thousands of entries, and cloning that per
    // frame at 60 Hz would be a real cost for no reason.
    doc.selection = selection;
    store_gesture(&ctx, gestures);
    (count, tokens)
}

/// Ask the provider what is under a click, at every rung at once.
///
/// # Why the part and node queries are scoped to the ENTERED object
///
/// Because that is what makes the deeper rungs predictable. A node query
/// against an object's whole flat anchor list is the hazard decision 028 found
/// already shipped: one measured CAD object holds **6,681 anchors**, so "the
/// nearest anchor to the press" can easily belong to a subpath the operator is
/// not pointing at, with nothing drawn beforehand to say which.
///
/// When nothing is entered yet, the subject is the object under the pointer —
/// which is what a double-click needs, since it descends into whatever it
/// landed on.
fn probe(
    targets: &dyn CanvasTargetProvider,
    selection: &SelectionState,
    page_index: usize,
    point: Pos2,
    map: &PageMapping,
) -> ClickHit {
    // ONE tolerance, converted once, in page units. Passing
    // `SELECT_SCREEN_TOLERANCE_PX` here would compile, run, and merely drift
    // with zoom — see `mapping`.
    let tolerance = map.tolerance();
    let object = targets.hit_test(page_index, point, tolerance);

    let subject = selection
        .entered_object()
        .map(|e| e.object)
        .or(object)
        .and_then(|t| usize::try_from(t.0).ok());
    let (part, node) = match subject {
        Some(index) => {
            let part = targets
                .part_hits(page_index, index, point, tolerance)
                .first()
                .copied();
            let node =
                part.and_then(|p| targets.nearest_node(page_index, index, p, point, tolerance));
            (part, node)
        }
        None => (None, None),
    };
    ClickHit { object, part, node }
}

/// Read the in-flight pointer gesture.
fn load_gesture(ctx: &egui::Context) -> GestureState {
    let id = egui::Id::new(GESTURE_MEMORY_KEY);
    ctx.data_mut(|d| d.get_temp::<GestureState>(id).unwrap_or_default())
}

/// Write the in-flight pointer gesture back.
fn store_gesture(ctx: &egui::Context, gestures: GestureState) {
    let id = egui::Id::new(GESTURE_MEMORY_KEY);
    ctx.data_mut(|d| d.insert_temp(id, gestures));
}

/// Report a selection-changing gesture on the `PDFCE_DIAG` channel.
///
/// De-duplicated on the rendered line, so a marquee dragged across a sheet
/// does not bury the events around it — the lesson `canvas-pointer` taught
/// when a stationary pointer emitted fifty identical lines in nine seconds.
/// The count and the level are on the line because they are what a harness
/// asserts on: *"the click landed"* is `sel=` moving, and *"the ladder
/// descended"* is `level=` moving.
fn trace_selection_event(selection: &SelectionState, kind: &str, modifier: bool) {
    crate::diag::trace_changed(SELECTION_SLOT, || {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI.
            // Placed directly above the literal — see `trace_layout`.
            "canvas-selection via={kind} mod={modifier} sel={} level={:?}",
            selection.len(),
            selection.level(),
        )
    });
}

/// The pointer movement of an in-progress pan over this canvas, or `None` when
/// no pan is happening.
///
/// **Two buttons, one path.** The middle button always pans — the CAD /
/// Inkscape / Illustrator / browser convention, requested on 2026-08-04 — and
/// the primary button pans as well while the hand tool is active, whether the
/// operator chose it or is borrowing it with the space bar. They share this
/// function and therefore share [`geometry::pan_offset`], its clamp and its
/// cursor: `GUI_ROADMAP` 3.2 asks for a hand tool, not for a second panning
/// implementation that rounds differently at the edges of the scroll range.
///
/// Gated on the pointer being over the canvas so a drag that began on some
/// other surface does not yank the page sideways.
fn pan_delta(ui: &egui::Ui, tool: CanvasTool) -> Option<Vec2> {
    let rect = ui.max_rect();
    ui.input(|i| {
        let over = i.pointer.latest_pos().is_some_and(|p| rect.contains(p));
        let panning =
            i.pointer.middle_down() || (tool.pans_with_primary() && i.pointer.primary_down());
        if panning && over {
            let delta = i.pointer.delta();
            (delta != Vec2::ZERO).then_some(delta)
        } else {
            None
        }
    })
}

/// Report where the canvas is, at what magnification, on the `PDFCE_DIAG`
/// channel — **unconditionally**, not only when something happens.
///
/// # The deadlock this removes
///
/// `PROJECT_PLAN.md` §4.3 requirement 1, discovered by building
/// `tools/ui-verify` at S1 rather than by reading code:
///
/// > The old binary traces it only on pointer events, so the harness cannot
/// > aim until it clicks and cannot click until it can aim.
///
/// The old shell's canvas line fires on `pressed || released || down ||
/// zoom`. A freshly opened document is none of those, so it reports no canvas
/// rect at all — and without a canvas rect there is no document-to-window
/// mapping, and without that mapping there is no click that can be aimed. The
/// harness worked around it with one documented *layout-probe* click at the
/// client-area centre (`ui-verify`'s `WindowFrame::layout_probe_point`),
/// whose only purpose was to make the application speak.
///
/// The workaround was safe but not free: it rests on the assumption that the
/// centre of the client area is the canvas, it fires a real OS click into a
/// document before any assertion has been made, and every check that used it
/// had to count the events it produced so they were not mistaken for the
/// check's own. All of that goes away if the application simply says where
/// its canvas is.
///
/// # When this emits
///
/// Every frame builds the line; [`crate::diag::trace_changed`] emits it only
/// when it differs from the last one. So in practice:
///
/// * **once per document open** — the first frame of a new document finds an
///   empty gate (see [`crate::diag::reset_change_gates`], called from the open
///   path), so there is always a line before any input is delivered;
/// * **again on every layout change** — a window resize, a panel resize, a
///   zoom step, a fit-mode re-derivation, a page change, a scroll;
/// * **not at all** on the frames in between, which is what keeps a
///   several-minute driven run from burying its own evidence.
///
/// # The line, field by field
///
/// ```text
/// pdfce-diag canvas rect=[[240.0 96.0] - [1560.0 968.0]] zoom=1.5000 page=0 pages=3 off=[0.0 0.0]
/// ```
///
/// * `rect=` — the **page raster's** rect in window logical points, printed
///   as `egui::Rect`'s own `Debug`. Not the viewport, not the panel: the
///   thing `viewer::screen_to_page` is the inverse of. `ui-verify`'s
///   `CanvasMapping` computes `window = rect.min + canvas_point * zoom`, so
///   handing it anything else would be a confidently wrong click.
/// * `zoom=` — logical points per PDF user-space unit, the same number
///   `viewer::screen_to_page` divides by. Four decimals because a fit scale
///   is rarely round and two would quantise a 1320 pt page by a whole point.
/// * `page=` — the 0-based page index `rect` shows. `ui-verify` refuses to
///   convert a document point against a mapping for a different page, and it
///   can only do that if the application says which page it drew.
/// * `pages=` — the document's page count, so a check that walked off the end
///   can tell "no such page" from "the application ignored the command".
/// * `off=` — the scroll offset the area settled on. Reported because
///   `ui-verify`'s `coords` module documents an **unverified assumption**
///   that `rect=` already accounts for scrolling, names the experiment that
///   would settle it, and holds a `scroll` correction at zero until someone
///   runs it. It cannot be run against a binary that does not report the
///   offset, so this field is what makes the assumption falsifiable.
///
/// # `sel=` — added here, in the commit that gave it something to count
///
/// The old binary's canvas line carries `sel=`, the current selection size,
/// and `ui-verify` reads it as a fallback when a click produced no event of
/// its own. Stages S0–S3 deliberately did **not** emit it, with the reason
/// recorded rather than the field silently omitted: there was no hit test and
/// no selection set, so `sel=0` would have been a measurement of something
/// that did not exist, and it would have turned
/// `delete_key_after_canvas_click` from an honest SKIP (*"the harness cannot
/// tell whether the click landed"*) into a FAIL blaming a subsystem nobody
/// had written. The stated condition for adding it was *"in the same commit
/// as the selection model, at S4"* — this is that commit.
///
/// It is counted **after** the frame's gesture has been applied (see the call
/// site), so a click and the `sel=` that describes it appear on the same
/// frame rather than one apart.
fn trace_layout(doc: &OpenDoc, image_rect: Rect, scroll_offset: Vec2, selected: usize) {
    crate::diag::trace_changed(LAYOUT_SLOT, || {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI.
            // This comment sits directly above the literal, not above the
            // enclosing call: the gate's scope is the line, and rustfmt is
            // free to reflow a call's arguments out from under a comment
            // placed further up.
            "canvas rect={image_rect:?} zoom={:.4} page={} pages={} off={scroll_offset:?} sel={selected}",
            doc.view.zoom,
            doc.view.page_index,
            doc.pages.len(),
        )
    });
}

/// Report the pointer's position in **document space** on the `PDFCE_DIAG`
/// channel.
///
/// # Why this is here rather than in a later stage
///
/// `PROJECT_PLAN.md` §4.2 lists three prerequisites that *"belong in S1, not
/// later"*, and the first is: **`ui-verify` scripts document-space
/// coordinates, never absolute screen coordinates.** User-rearrangeable
/// panels make widths arbitrary at runtime, and the project's own RAG
/// records this exact class producing a filed-then-retracted false
/// coordinate-space defect.
///
/// A harness cannot script in document space unless the application will
/// *tell* it where a screen point lands in document space. This is that
/// channel, and it exists from S0 so the harness written at S1 has
/// something to read on its first run rather than needing the canvas
/// reopened to add it.
///
/// Two spaces are reported because the harness needs both and the
/// distinction is exactly where coordinate bugs live:
/// `page=` is **canvas space** (Y-down, origin top-left, `/Rotate` applied),
/// `pdf=` is genuine **PDF user space** (Y-up, un-rotated lower-left origin)
/// — the frame an annotation `/Rect` is written in.
///
/// Costs nothing when tracing is off: [`crate::diag::trace_changed`] takes a
/// closure and never calls it.
///
/// # Why this is gated on movement
///
/// It was not, and that was a real defect: `pointer_latest_pos` returns the
/// **last known** position, not "the position it moved to this frame", so a
/// stationary pointer over the canvas re-reported the same three coordinate
/// pairs on every single frame. Measured on the S1 binary: **50 identical
/// lines in 9 seconds.** A driven run is minutes long, so the events that
/// actually matter — an open, a click, a deletion — end up separated by
/// thousands of lines saying nothing, and `ui-verify` re-parses the whole
/// capture after every settle.
///
/// The gate is [`crate::diag::trace_changed`] rather than a hand-rolled
/// comparison against a stored `Pos2` for a specific reason: the printed line
/// is the thing the consumer reads, so the printed line is the right unit of
/// "changed". A movement too small to alter `{:.2}` is a movement no parser
/// could have seen.
///
/// The line's *shape* is unchanged and must stay so — `screen=`, `page=`,
/// `pdf=` and `zoom=` are the contract, and only how often it is written has
/// been fixed.
fn trace_pointer(ui: &egui::Ui, doc: &OpenDoc, image_rect: Rect, extent: (f32, f32)) {
    if !crate::diag::enabled() {
        return;
    }
    let Some(screen) = ui.ctx().pointer_latest_pos() else {
        return;
    };
    let page = viewer::screen_to_page(screen, image_rect, extent, doc.view.zoom);
    let pdf = doc
        .current_page()
        .and_then(|p| viewer::canvas_to_pdf_space(page, p));
    crate::diag::trace_changed(POINTER_SLOT, || {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI.
            // Placed directly above the literal — see `trace_layout`.
            "canvas-pointer screen=({:.1},{:.1}) page=({:.2},{:.2}) pdf={} zoom={:.4}",
            screen.x,
            screen.y,
            page.x,
            page.y,
            pdf.map_or_else(|| "none".to_owned(), |p| format!("({:.2},{:.2})", p.x, p.y)),
            doc.view.zoom,
        )
    });
}
