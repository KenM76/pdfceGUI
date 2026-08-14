//! # `canvas::input` — reading one frame's pointer: what it landed on, what it is panning, and where the gesture is kept
//!
//! ## Why this is a module rather than four functions at the bottom of [`super`]
//!
//! Rule R2's 1,500-line ceiling forced a split when the rulers landed, and
//! this is the seam it forced — the same way it produced [`super::trace`] when
//! Phase 4 added the strip, and [`super::strip`] alongside it. Both of those
//! headers record that the forced seam turned out to be a real one, and so
//! does this.
//!
//! Everything here answers **"what is the pointer doing this frame?"**, and
//! every one of them is a question with a single, local answer:
//!
//! | function | question |
//! |---|---|
//! | [`probe`] | what a click landed on, at every rung of the selection ladder at once |
//! | [`pan_delta`] | whether *either* of the two panning gestures is in flight, and how far it moved |
//! | [`load_gesture`] / [`store_gesture`] | where the in-flight press lives between frames |
//!
//! What is left behind in [`super`] answers a different question — *how is the
//! frame composed?* — and it is a question about layout, the scroll area, the
//! strip and the order the overlay is painted in. Nothing here needs any of
//! that: [`probe`] needs a provider and a mapping, [`pan_delta`] needs an
//! input state and a rect, and the two `Memory` accessors need a `Context`.
//!
//! ## The one thing that is still in `egui::Memory`, and why
//!
//! [`GESTURE_MEMORY_KEY`]. The selection moved off `Memory` and onto
//! `OpenDoc` at stage S4 because it is **document-scoped** state and `Memory`
//! outlives documents; the argument, and the address-as-identity hazard that
//! came with the workaround, are in [`crate::app::state::OpenDoc::selection`].
//!
//! A gesture is the opposite case and it is worth being explicit about why.
//! The drag that is happening *right now* is genuinely frame-local UI state.
//! It has no meaning across a document, and a gesture that survived one would
//! be a drag continuing over a file it did not start on. Keying it in `Memory`
//! means it cannot: `Memory` is per-`Context`, and every document change
//! starts the next frame with no press in flight — by construction, with
//! nothing to compare and nothing to forget.

use egui::{Pos2, Vec2};

use crate::canvas::gesture::GestureState;
use crate::canvas::mapping::PageMapping;
use crate::canvas::selection::{ClickHit, SelectionState};
use crate::canvas::target::CanvasTargetProvider;
use crate::canvas::tool::CanvasTool;

/// `egui::Memory` key for the in-flight pointer gesture.
///
/// ★ **The one thing that stayed in `Memory` when the selection left**, and
/// the distinction is the point rather than an omission — see this module's
/// header.
const GESTURE_MEMORY_KEY: &str = "pdfce-canvas-gesture"; // ui-text-exempt: internal memory id, never displayed

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
pub(super) fn probe(
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
pub(super) fn load_gesture(ctx: &egui::Context) -> GestureState {
    let id = egui::Id::new(GESTURE_MEMORY_KEY);
    ctx.data_mut(|d| d.get_temp::<GestureState>(id).unwrap_or_default())
}

/// Write the in-flight pointer gesture back.
pub(super) fn store_gesture(ctx: &egui::Context, gestures: GestureState) {
    let id = egui::Id::new(GESTURE_MEMORY_KEY);
    ctx.data_mut(|d| d.insert_temp(id, gestures));
}

/// The pointer movement of an in-progress pan over this canvas, or `None` when
/// no pan is happening.
///
/// **Two buttons, one path.** The middle button always pans — the CAD /
/// Inkscape / Illustrator / browser convention, requested on 2026-08-04 — and
/// the primary button pans as well while the hand tool is active, whether the
/// operator chose it or is borrowing it with the space bar. They share this
/// function and therefore share [`super::geometry::pan_offset`], its clamp and
/// its cursor: `GUI_ROADMAP` 3.2 asks for a hand tool, not for a second
/// panning implementation that rounds differently at the edges of the scroll
/// range.
///
/// Gated on the pointer being over the canvas so a drag that began on some
/// other surface does not yank the page sideways.
///
/// ★ **`ui` is the canvas's own child `Ui`**, whose `max_rect` is the region
/// *inside* the ruler gutters — see [`super::rulers::Gutters::content_ui`].
/// That is what stops a drag begun on a ruler from also panning the page: the
/// gutter is outside this rect, so `over` is false there.
pub(super) fn pan_delta(ui: &egui::Ui, tool: CanvasTool) -> Option<Vec2> {
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
