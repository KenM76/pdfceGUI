//! # `canvas::dimdrag` — **dragging a ce dimension to where it should be drawn**
//!
//! ## The operator's report, verbatim
//!
//! > *"I need to be able to move the dimension after it has been laid down,
//! > and there should be a preview of the dimensioning lines as I lay it down
//! > and click to position it when it is created and after the fact."*
//!
//! Two halves. The first half — *as I lay it down* — already shipped:
//! [`crate::canvas::measure`]'s third click places a new ce dimension and
//! previews it through `measure::pick::dimension_preview_segments`. This
//! module is the second half, *after the fact*, and it reuses that same
//! preview function for the same reason: a preview derived a second way is a
//! preview that can disagree with what commits.
//!
//! ## ★★ What "move a dimension" means, and why it is NOT `move_dimension`
//!
//! `pdfce-core` offers two verbs and picking the wrong one is the whole design
//! decision here:
//!
//! | verb | what it changes | what the number does |
//! |---|---|---|
//! | `EditSession::move_dimension` | translates the **measured points** with the drawing | unchanged (a rigid motion preserves a distance) — but the dimension leaves the feature it was measuring |
//! | `EditSession::place_dimension` | writes `offset` and `text_along` only | **cannot** change, by construction: the value function does not read either field |
//!
//! Dragging a dimension is `place_dimension`. That is what SolidWorks does —
//! the attachment points stay on the geometry and the extension lines stretch
//! — and it is what the engine's own doc comment says the verb exists for:
//! *"This, not `move_dimension`, is what dragging a dimension does."*
//!
//! The consequence worth stating out loud, because it is the property that
//! makes this gesture safe enough to be the *default* action on a press:
//! **no drag, however far, can alter the printed number.** An operator can drag
//! a dimension across the sheet and back and the document's measurements are
//! unchanged. `move_dimension` has no such guarantee — it would take a
//! dimension off the feature it annotates — so it is deliberately not wired to
//! a drag and remains available only where the operator has said they mean it.
//!
//! ## The delta is resolved in the dimension's OWN frame
//!
//! A page-space delta is projected onto the two axes `axis_frame` gives:
//!
//! ```text
//! offset'     = offset     + delta · n    (perpendicular — how far the line stands off)
//! text_along' = text_along + delta · u    (parallel      — where the number sits along it)
//! ```
//!
//! **A delta, not `placement_from_point`.** Both were available and the
//! absolute form is shorter, but it resolves the placement from wherever the
//! *pointer* is, which means the dimension jumps on the first frame of the drag
//! so that its anchor lands under the cursor. A delta preserves the grab: the
//! dimension moves exactly as far as the hand does, and whatever part of it the
//! operator grabbed stays under their finger. The absolute form is the right
//! one for authoring — where there is no grab to preserve, because the
//! dimension does not exist yet — and that is precisely where `canvas::measure`
//! uses it.
//!
//! ## Why only linear dimensions may be dragged (and why that is not a stub)
//!
//! `place_dimension` accepts angular dimensions too, but its two arguments mean
//! something different there: `offset` is an **arc radius** from the apex and
//! `text_along` is in **degrees**. Adding a dot product measured in points to a
//! quantity measured in degrees is not a smaller version of the right answer,
//! it is arithmetic on mismatched units, so [`placed`] refuses.
//!
//! The refusal is honoured at the *press*, not at the release: [`grab_box`]
//! returns `None` for anything it cannot drag, so the press falls through to
//! the ordinary marquee and no gesture is ever started that could not finish.
//! That is this project's no-placeholders invariant applied to a gesture rather
//! than to a widget — an inert drag is a visible control that silently does
//! nothing, which is the exact failure class `DEFECTS.md` is made of.
//!
//! Angular placement by drag is worth having and is written up rather than
//! faked; see the TODO note on [`placed`].
//!
//! ## What this module does NOT decide
//!
//! * **Whether the press is a drag at all.** `canvas::gesture` owns that. This
//!   module only supplies the hit box that makes the press mean *move*.
//! * **Where the dimension is drawn once committed.** `pdfce-render` draws the
//!   annotation; this module draws only the in-flight preview, and only from
//!   the same segment function a committed dimension is previewed from.
//! * **Undo granularity.** `place_dimension` is one command, so one drag is one
//!   undo entry, decided by the engine.

use egui::{Rect, Vec2};
use pdfce_core::dimension::{DimensionId, DimensionKind};
use pdfce_core::page_tree::Page;
use pdfce_core::vector::Point;

use crate::app::actions::Action;
use crate::app::actions::dimensions::DimensionAction;
use crate::app::state::OpenDoc;
use crate::canvas::gesture::Phase;
use crate::canvas::mapping::PageMapping;
use crate::canvas::selection::{AnnotKind, SelectionState};

/// The trace channel a driven check reads to prove a placement committed.
///
/// An in-flight placement and a committed one are the same screenshot at the
/// moment of release, which is defect 8's lesson: the harness needs a line that
/// distinguishes *"the preview followed the pointer"* from *"the release
/// reached the verb"*, and no pixel can carry that.
pub const TRACE: &str = "dimension-place"; // ui-text-exempt: diagnostic trace name

/// The dimension under the selection, if one is selected **and** it is a kind
/// this module can drag.
///
/// Returns the record's id together with its geometry, because every caller
/// needs both and a second lookup could resolve differently after an edit.
///
/// # Why the whole model is walked rather than indexed
///
/// An annotation selection carries an *object* id — the annotation in the file
/// — and a dimension record carries a [`DimensionId`]. The sidecar holds the
/// mapping one way only (`record.annot`), so the reverse lookup is a scan. It
/// is a scan over the dimensions on the document, which is a handful even on a
/// heavily dimensioned sheet, and it runs once per press rather than per frame.
#[must_use]
pub fn selected(doc: &OpenDoc, selection: &SelectionState) -> Option<(DimensionId, DimensionKind)> {
    let annot = selection.annot()?;
    if annot.target.kind != AnnotKind::CeDimension {
        return None;
    }
    let model = doc.session.dimension_model();
    let record = model
        .dimensions()
        .iter()
        .find(|r| r.annot == Some(annot.target.id))?;
    // The gate that keeps an un-draggable kind from ever starting a gesture.
    // See the module header: an angular dimension's placement is a radius and
    // an angle, and this module's delta is in points.
    if !matches!(record.kind, DimensionKind::Linear { .. }) {
        return None;
    }
    Some((record.id, record.kind.clone()))
}

/// The screen-space box a press must land in to mean *move this dimension*.
///
/// The annotation's `/Rect`, projected. That is the same rectangle
/// `canvas::overlay::draw_selection` already strokes when a dimension is
/// selected, which is the property that matters: **the drawn outline and the
/// live target are the same shape.** An operator aims at what they can see.
///
/// # Why this is not `overlay::grip_box`
///
/// That function derives its box from the selection's cached content outlines,
/// which `select_annot` clears — an annotation is not content and has no
/// decomposed outline to cache. So `grip_box` answers `None` over a selected
/// dimension, which is why a press on one used to start a marquee and replace
/// the selection the operator was trying to act on. Keeping the two functions
/// separate rather than teaching `grip_box` about annotations keeps the resize
/// grips out of this: `grip_box`'s box is also what the eight scale handles are
/// laid out on, and a dimension has no scale verb.
#[must_use]
pub fn grab_box(doc: &OpenDoc, map: &PageMapping, selection: &SelectionState) -> Option<Rect> {
    selected(doc, selection)?;
    let annot = selection.annot()?;
    Some(map.rect_to_screen(annot.outline))
}

/// **The rule.** A page-space delta, applied in the dimension's own frame.
///
/// Returns the placed geometry together with the two scalars `place_dimension`
/// takes, so the preview and the commit are derived from one calculation rather
/// than two that could disagree. That pairing is the point of the return type:
/// a caller cannot draw one placement and commit another without going out of
/// its way.
///
/// `None` when there is no frame to resolve against — a non-linear dimension
/// (see the module header) or a degenerate `Aligned` one whose two picks
/// coincide, which has no axis at all and which `axis_frame` refuses rather
/// than fabricating.
///
/// # TODO — angular placement
///
/// `place_dimension` accepts an angular dimension, taking an arc radius and a
/// position in degrees. Dragging one is a genuinely different calculation
/// (radial distance from the apex; angle subtended) rather than this one with
/// different names, and it needs its own preview and its own tests. Filed
/// rather than approximated.
#[must_use]
pub fn placed(kind: &DimensionKind, dx: f64, dy: f64) -> Option<(DimensionKind, f64, f64)> {
    let DimensionKind::Linear {
        a,
        b,
        constraint,
        offset,
        text_along,
    } = *kind
    else {
        return None;
    };
    let (u, n) = kind.axis_frame()?;
    let offset = offset + dx * n.x + dy * n.y;
    let text_along = text_along + dx * u.x + dy * u.y;
    Some((
        DimensionKind::Linear {
            a,
            b,
            constraint,
            offset,
            text_along,
        },
        offset,
        text_along,
    ))
}

/// Everything one frame of a placement drag needs, gathered at the call site.
pub struct Frame<'a> {
    /// How far the pointer has travelled since the press, in canvas space.
    pub delta: Vec2,
    /// Draw the preview, or commit the placement.
    pub phase: Phase,
    /// The page the dimension is on — needed to turn a canvas delta into a
    /// page-space one, which is the only place the y-flip is applied.
    pub page: Option<&'a Page>,
}

/// Advance one frame of a placement drag.
///
/// Returns the **page-space segments** the dimension would be drawn as, if the
/// operator released now, or `None` when the drag reaches no verb.
///
/// # The honesty contract, restated because it is the same one everywhere here
///
/// The preview is `Some` if and only if a release would commit, and it is
/// derived from the *same* [`placed`] result the commit uses. So the operator
/// cannot be shown a dimension standing off by 40 points and then get one
/// standing off by something else — the two numbers are literally the same
/// `f64`.
///
/// # Rule 4, and why this preview is allowed to exist at all
///
/// Rule 4 forbids marking *applied* content as provisional. This draws
/// something that has not been applied yet: it is the rubber-band of a drag in
/// flight, which the rule names explicitly as a pre-commit affordance — *"a
/// snap indicator, a hover highlight, a rubber-band … these are the cursor"*.
/// It disappears on release, and what replaces it is the annotation itself,
/// rendered by `pdfce-render` with no marking of any kind.
pub fn drag(
    frame: Frame<'_>,
    doc: &OpenDoc,
    selection: &SelectionState,
    actions: &mut Vec<Action>,
) -> Option<Vec<(Point, Point)>> {
    let Frame { delta, phase, page } = frame;
    let (id, kind) = selected(doc, selection)?;
    let page = page?;
    let d = super::moving::page_delta(delta, page)?;
    let (moved, offset, text_along) = placed(&kind, d.dx, d.dy)?;

    if phase == Phase::Complete {
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "{TRACE} id={} offset={offset:.2} text_along={text_along:.2}",
                id.0
            )
        });
        actions.push(Action::Dimension(DimensionAction::Place {
            dimension: id,
            offset,
            text_along,
        }));
        // Nothing is previewed on the frame that commits: the annotation is
        // about to be regenerated and drawn for real, and a preview left on
        // screen over it would be a second copy of the same line, one frame
        // stale.
        return None;
    }
    Some(super::measure::pick::dimension_preview_segments(&moved))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfce_core::vector::AxisConstraint;

    fn horizontal() -> DimensionKind {
        DimensionKind::Linear {
            a: Point::new(100.0, 200.0),
            b: Point::new(300.0, 200.0),
            constraint: AxisConstraint::Horizontal,
            offset: 0.0,
            text_along: 0.0,
        }
    }

    /// A horizontal dimension's frame is `u = +x`, `n = +y`. So a drag straight
    /// up is pure standoff and a drag sideways is pure text slide — the two
    /// components do not contaminate each other.
    #[test]
    fn a_delta_splits_into_standoff_and_slide_along_the_axis() {
        let (_, offset, along) = placed(&horizontal(), 0.0, 30.0).expect("linear places");
        assert!((offset - 30.0).abs() < 1e-9, "straight up is all standoff");
        assert!(along.abs() < 1e-9, "and none of it slides the text");

        let (_, offset, along) = placed(&horizontal(), 25.0, 0.0).expect("linear places");
        assert!(offset.abs() < 1e-9, "sideways changes no standoff");
        assert!(
            (along - 25.0).abs() < 1e-9,
            "and slides the text by the drag"
        );
    }

    /// ★ The property the whole design rests on: **placement never touches what
    /// is measured.** Whatever the drag, `a` and `b` come out unchanged, so the
    /// printed number cannot move.
    #[test]
    fn no_drag_can_move_the_measured_points() {
        let before = horizontal();
        for (dx, dy) in [(0.0, 0.0), (500.0, -900.0), (-12.5, 7.25), (1e6, 1e6)] {
            let (after, _, _) = placed(&before, dx, dy).expect("linear places");
            let (DimensionKind::Linear { a, b, .. }, DimensionKind::Linear { a: a0, b: b0, .. }) =
                (&after, &before)
            else {
                panic!("both are linear");
            };
            assert!(
                (a.x - a0.x).abs() < 1e-9 && (a.y - a0.y).abs() < 1e-9,
                "point a moved on a drag of {dx},{dy}"
            );
            assert!(
                (b.x - b0.x).abs() < 1e-9 && (b.y - b0.y).abs() < 1e-9,
                "point b moved on a drag of {dx},{dy}"
            );
        }
    }

    /// Placement accumulates: two drags of ten leave the dimension where one
    /// drag of twenty would. The delta form is what makes this true — an
    /// absolute `placement_from_point` would put it wherever the pointer last
    /// was, which is a different gesture.
    #[test]
    fn two_drags_compose_into_one() {
        let (once, _, _) = placed(&horizontal(), 0.0, 10.0).expect("places");
        let (twice, offset, _) = placed(&once, 0.0, 10.0).expect("places");
        let (_, direct, _) = placed(&horizontal(), 0.0, 20.0).expect("places");
        assert!((offset - direct).abs() < 1e-9);
        let DimensionKind::Linear { offset: o, .. } = twice else {
            panic!("linear")
        };
        assert!((o - 20.0).abs() < 1e-9);
    }

    /// An aligned dimension whose two picks coincide has no axis, so there is
    /// nothing to resolve a delta against. Refused rather than fabricated — see
    /// `axis_frame`, which makes the same call one level down.
    #[test]
    fn a_degenerate_dimension_has_no_frame_and_is_refused() {
        let degenerate = DimensionKind::Linear {
            a: Point::new(50.0, 50.0),
            b: Point::new(50.0, 50.0),
            constraint: AxisConstraint::Aligned,
            offset: 0.0,
            text_along: 0.0,
        };
        assert!(placed(&degenerate, 5.0, 5.0).is_none());
    }
}
