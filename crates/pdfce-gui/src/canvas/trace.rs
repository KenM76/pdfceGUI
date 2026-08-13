//! # `canvas::trace` — what the canvas says on the `PDFCE_DIAG` channel
//!
//! Three lines, one subject: *where is the canvas, what is on it, and what did
//! the operator just do to the selection?*
//!
//! ## Why this is a module rather than three functions at the bottom of [`super`]
//!
//! Rule R2's 1,500-line ceiling forced a split when Phase 4 added the strip,
//! and this is the seam it forced — which turns out to be a real one. Every
//! function here exists to serve a **consumer outside the process**:
//! `tools/ui-verify`, which drives the binary and reads its stderr. That gives
//! them a property nothing else in `canvas/` has: **their output shape is a
//! contract.** A field renamed here breaks a harness that does not compile
//! against this crate and will therefore not fail to build — it will fail to
//! find what it is looking for, at run time, in a check whose subject is
//! something else entirely.
//!
//! Keeping them together is what makes that contract reviewable in one place.
//! `PROJECT_PLAN.md` §4.3's three requirements are all discharged by the
//! functions below, and each one's doc comment carries the requirement it
//! answers and the failure it was written after.
//!
//! ## The de-duplication slots, and why each line has its own
//!
//! [`crate::diag::trace_changed`] emits a line only when it differs from the
//! last one written to the same slot. The slots are separate because the lines
//! answer different questions on different timescales — the pointer moves
//! constantly while the layout does not — and sharing one would make each
//! silence the other.

use egui::{Rect, Vec2};

use crate::app::state::OpenDoc;
use crate::canvas::selection::SelectionState;
use crate::viewer;

/// Report a selection-changing gesture on the `PDFCE_DIAG` channel.
///
/// De-duplicated on the rendered line, so a marquee dragged across a sheet
/// does not bury the events around it — the lesson `canvas-pointer` taught
/// when a stationary pointer emitted fifty identical lines in nine seconds.
/// The count and the level are on the line because they are what a harness
/// asserts on: *"the click landed"* is `sel=` moving, and *"the ladder
/// descended"* is `level=` moving.
pub(super) fn selection_event(selection: &SelectionState, kind: &str, modifier: bool) {
    crate::diag::trace_changed(super::SELECTION_SLOT, || {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI.
            // Placed directly above the literal — see `trace_layout`.
            "canvas-selection via={kind} mod={modifier} sel={} level={:?}",
            selection.len(),
            selection.level(),
        )
    });
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
/// # ★ `display=`, `visible=` and `drawn=` — added at Phase 4, at the END
///
/// The five original fields keep their names, their order and their meaning,
/// because `ui-verify`'s `CanvasMapping` parses them and `rect=` is still the
/// **acting page's** rect — the thing `viewer::screen_to_page` is the inverse
/// of, and the one a click has to be aimed against. Under a continuous mode
/// several pages are on screen and `rect=` names one of them; `page=` says
/// which, exactly as it always did.
///
/// The three new fields answer what a strip made askable and are appended so
/// no existing parser moves:
///
/// * `display=` — the page-display mode's id (`single`, `continuous`,
///   `facing`, `facing-continuous`). Without it a trace cannot distinguish
///   "one page is on screen because the operator chose Single" from "one page
///   is on screen because that is all that fits", and those need opposite
///   responses.
/// * `visible=` — how many pages this frame drew. The number a scroll check
///   watches move, and the number that says whether the strip is doing
///   anything at all.
/// * `drawn=` — how many of those had a raster. `drawn < visible` is the
///   honest statement that the renderer is behind, which is exactly what the
///   undrawn pages are saying on screen; `drawn == visible` is a settled
///   strip. A check that measured only `visible` could not tell a filled strip
///   from an empty one.
pub(super) fn layout(
    doc: &OpenDoc,
    image_rect: Rect,
    scroll_offset: Vec2,
    selected: usize,
    visible: usize,
    with_raster: usize,
) {
    crate::diag::trace_changed(super::LAYOUT_SLOT, || {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI.
            // This comment sits directly above the literal, not above the
            // enclosing call: the gate's scope is the line, and rustfmt is
            // free to reflow a call's arguments out from under a comment
            // placed further up.
            "canvas rect={image_rect:?} zoom={:.4} page={} pages={} off={scroll_offset:?} sel={selected} display={} visible={} drawn={with_raster}",
            doc.view.zoom,
            doc.view.page_index,
            doc.pages.len(),
            doc.view.display.id(),
            visible,
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
pub(super) fn pointer(ui: &egui::Ui, doc: &OpenDoc, image_rect: Rect, extent: (f32, f32)) {
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
    crate::diag::trace_changed(super::POINTER_SLOT, || {
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
