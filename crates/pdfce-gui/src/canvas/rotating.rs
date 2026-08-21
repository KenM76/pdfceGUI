//! # `canvas::rotating` — the ninth grip, and the one gesture the eight could
//! never express
//!
//! ## What this closes
//!
//! `ui-conventions/handles.md` H2 — *"the standard set is eight resize grips, a
//! body, and a rotate handle"* — and the operator's own report, which is the
//! sentence that corpus row quotes:
//!
//! > *"unfortunately there was no way to reposition, resize, or rotate it on
//! > the screen. Can I please please please have that too?"*
//!
//! Reposition landed with `canvas::moving`'s transform fork and resize with
//! `canvas::resizing`'s, both on 2026-08-20. **Rotate is the third word in that
//! sentence and it had no affordance at all** — the verb rotates, and nothing
//! on the canvas reached it.
//!
//! ## ★★ Why a rotate is not a resize with different arithmetic
//!
//! The eight grips answer *"how big"*, which is a **distance**, so a resize is
//! a delta in two axes and every one of them has an opposite corner that must
//! not move. A rotation answers *"which way round"*, which is an **angle**, so:
//!
//! | | resize | rotate |
//! |---|---|---|
//! | measured from | the grip's opposite corner | the selection's **centre** |
//! | what the drag reads | a displacement | the change in bearing between two rays |
//! | what the pointer's *distance* means | everything | **nothing** |
//! | modifier | Shift preserves aspect | Shift snaps to 15° |
//!
//! The third row is the one that decides the module boundary. A rotate drag
//! must ignore how far the pointer is from the centre entirely — an operator
//! swinging a long arc for precision is doing exactly what the gesture invites,
//! and a build that scaled with radius would shrink the object as they did it.
//!
//! ## The handle sits ABOVE the box, on a stem
//!
//! Which is PowerPoint, Illustrator, Figma, Inkscape, Visio and Konva's
//! `Transformer`. The offset is what makes it reachable on a selection whose
//! top edge is crowded with the north grip and whatever is behind it, and the
//! stem is what says the two belong together — without it the handle reads as
//! an unrelated dot floating over the page.
//!
//! ★ **It is drawn as a circle**, not a square. Every square on this canvas
//! resizes; a shape that resized in one place and rotated in another would be
//! a private convention the operator has to learn, which is
//! `handles.md` H2's stated failure mode.
//!
//! ## conventions: drag-moves
//!
//! Corpus: `ui-conventions/drag-moves.md`. The handle rows are answered by
//! `canvas::handles`, which owns the grip.
//!
//! - D1 live-preview: the ghost is the selection's outlines rotated about the
//!   same centre by the same angle the release commits.
//! - D2 derived-from-commit: [`angle`] is called once per frame and its result
//!   feeds the ghost and the commit; there is no second derivation.
//! - D3 escape-cancels: the gesture machine drops it and nothing is written
//!   before `Complete`.
//! - D4 one-undo-entry: one `transform_objects` call over every selected index.
//! - D5 modifiers-constrain: **Shift snaps to 15°**, which is the rotate
//!   flavour of the convention rather than the axis lock — see [`STEP_DEGREES`]
//!   — and it is announced on the status row like every other constraint.
//! - D6 snapping: WAIVED — there is nothing on a page to snap an angle to. A
//!   future "match that line's angle" is a different feature with a target.
//! - D7 no-op-is-not-an-edit: a release at exactly the bearing it began at
//!   raises nothing; see [`is_travel`].
//! - D8 grab-point: WAIVED, and it is the one row that genuinely does not
//!   apply. There is no point under the pointer to preserve — the pointer is
//!   holding a *bearing*, and the handle stays on its stem at the top of the
//!   box because that is where the box's top is.
//! - D9 disclosure: WAIVED — a rotation changes no value pdfce authored, and
//!   the new orientation is visible.

use egui::{Pos2, Vec2};

/// How many degrees a constrained rotation snaps to.
///
/// Fifteen, which is PowerPoint's, Illustrator's, Inkscape's and Figma's. It
/// divides 90 and 360 exactly, so the four right angles and the four diagonals
/// are all reachable — which is what the operator actually wants from the key,
/// and what a value like 10° would give them for 90 and take away for 45.
pub const STEP_DEGREES: f32 = 15.0;

/// The smallest rotation, in degrees, that counts as a gesture rather than a
/// twitch.
///
/// `drag-moves` D7: a drag that moves nothing is not an edit. A tenth of a
/// degree over a 200 pt box is a quarter of a pixel at the corner — invisible,
/// and not worth an undo entry for somebody who thought better of it.
const MIN_TRAVEL_DEGREES: f32 = 0.1;

/// **The angle a rotate drag has turned through, in radians.**
///
/// Positive is the direction the pointer went. Both rays are measured from
/// `centre`, and the pointer's *distance* from it is discarded — see the module
/// header for why that is the whole shape of the gesture rather than a detail.
///
/// # ★ Screen space in, screen space out, and the sign survives the hop
///
/// `centre`, `from` and `at` are all screen positions, where y runs **down**.
/// `atan2` therefore answers a bearing in a left-handed frame, so a clockwise
/// drag comes back positive. PDF user space is y-**up**, and
/// `Matrix::rotate(θ)` turns anticlockwise in it — so the caller negates once,
/// at the one place it converts, exactly as `canvas::mapping` does for every
/// other quantity that crosses.
///
/// Doing the flip here would put a page-space fact in a function that has never
/// seen a page, which is how a preview and a commit come to disagree about
/// which way round something went.
///
/// `None` when either ray is degenerate — the pointer exactly on the centre —
/// because a bearing from a zero-length ray is not a number and
/// `atan2(0.0, 0.0)` quietly answers zero rather than saying so.
#[must_use]
pub fn angle(centre: Pos2, from: Pos2, at: Pos2, constrain: bool) -> Option<f32> {
    let a = from - centre;
    let b = at - centre;
    if a.length() < f32::EPSILON || b.length() < f32::EPSILON {
        return None;
    }
    let delta = b.y.atan2(b.x) - a.y.atan2(a.x);
    // ★ Normalised into (-π, π] so a drag that crosses the ray behind the
    // centre turns the short way rather than jumping a full turn. Without it a
    // pointer moving smoothly through 180° makes the object spin the other way
    // round in one frame — a real defect in every naive implementation of this
    // gesture, and one that looks like a physics bug rather than an arithmetic
    // one.
    let delta = normalise(delta);
    Some(if constrain { snap(delta) } else { delta })
}

/// Wrap a radian difference into `(-π, π]`.
///
/// Its own function so the property has somewhere to be tested. See [`angle`]
/// for the defect it prevents.
#[must_use]
pub fn normalise(mut radians: f32) -> f32 {
    let turn = std::f32::consts::TAU;
    while radians > std::f32::consts::PI {
        radians -= turn;
    }
    while radians <= -std::f32::consts::PI {
        radians += turn;
    }
    radians
}

/// Round a radian angle to the nearest [`STEP_DEGREES`].
///
/// ★ It snaps the **total turn**, not the increment. Accumulating snapped
/// increments would let a slow drag through 90° arrive at 87°, because each
/// frame's small delta rounds to zero — the classic error, and the reason this
/// takes the whole angle rather than a per-frame one.
#[must_use]
pub fn snap(radians: f32) -> f32 {
    let step = STEP_DEGREES.to_radians();
    (radians / step).round() * step
}

/// Whether a rotation is big enough to be an edit — `drag-moves` D7.
#[must_use]
pub fn is_travel(radians: f32) -> bool {
    radians.abs() >= MIN_TRAVEL_DEGREES.to_radians()
}

/// Rotate a screen point about a screen centre, for the ghost.
///
/// ★ The ghost is drawn from **this** function and the commit from
/// `Matrix::rotate(θ).about(centre)`, which are the same map in two spaces —
/// and that is the one duplication in this module. It is not avoidable: the
/// preview must be drawn in screen space before the page conversion, and the
/// commit must be expressed in the engine's own type. What makes it safe is
/// that both take **the same θ from [`angle`]**, so the two can differ only in
/// the y-flip, which is a sign a unit test can pin.
#[must_use]
pub fn rotate_about(centre: Pos2, p: Pos2, radians: f32) -> Pos2 {
    let (s, c) = radians.sin_cos();
    let v = p - centre;
    centre + Vec2::new(v.x * c - v.y * s, v.x * s + v.y * c)
}

/// **Apply one frame of a rotate drag: preview it, or commit it.**
///
/// Mirrors [`crate::canvas::resizing::drag`] deliberately, down to the return
/// type, so a reader who has understood one has understood both. What it hands
/// back is the **angle** for the ghost, where the resize hands back two factors
/// and the move hands back a displacement.
///
/// Returns `Some(radians)` only when a ghost should be drawn — which, by this
/// project's honesty contract, is exactly when a release would commit.
///
/// # ★★ The negation, and it happens exactly once
///
/// [`angle`] measures in **screen** space, where y runs down, so a clockwise
/// drag comes back positive. `Matrix::rotate` turns anticlockwise in PDF user
/// space, where y runs **up**. The single `-` below is that crossing, and it
/// lives here rather than in `angle` for the reason `canvas::mapping`'s header
/// gives about every other conversion: one place, or the preview and the commit
/// eventually disagree about which way round something went — which is a defect
/// that looks like a deliberate feature.
///
/// The ghost is drawn from the **un-negated** angle, in screen space, by
/// `overlay::draw_rotate_ghost`. Both come from one call to [`angle`], so the
/// only thing that can differ between them is this sign.
pub fn drag(
    ctx: &egui::Context,
    frame: Frame<'_>,
    selection: &crate::canvas::selection::SelectionState,
    actions: &mut Vec<crate::app::actions::Action>,
) -> Option<f32> {
    let Frame {
        from,
        at,
        phase,
        bounds,
        page_index,
        constrain,
        map,
        page,
    } = frame;
    let bounds = bounds?;
    // ★ The pivot is the CENTRE, taken from `Grip::pivot` rather than from
    // `bounds.center()` here. Same number today; one statement of "what does a
    // rotate turn about", so the ghost, the commit and any future third reader
    // cannot drift.
    let centre = crate::canvas::handles::Grip::Rotate.pivot(bounds);
    let map = map?;
    // Screen space throughout, because that is what `bounds` is: the two rays
    // and the centre have to be in one frame, and the grip box is the frame the
    // handle was drawn in.
    let theta = angle(centre, map.to_screen(from), map.to_screen(at), constrain)?;
    if constrain {
        crate::canvas::constrain::announce(ctx, crate::canvas::constrain::Lock::Angle);
    }
    if phase == crate::canvas::gesture::Phase::InFlight {
        return Some(theta);
    }

    // ---- commit ------------------------------------------------------
    if !is_travel(theta) {
        // `drag-moves` D7. Silent: a release at the bearing it began at is an
        // operator who thought better of it, and a sentence about it would be
        // reporting their change of mind back to them.
        return None;
    }
    let objects = selection.object_indices_on(page_index);
    if objects.is_empty() {
        return None;
    }
    let page = page?;
    // The SAME two hops every other commit on this canvas takes, in the same
    // order, through the same two functions — screen → canvas → PDF user space.
    let pdf = crate::viewer::canvas_to_pdf_space(map.to_page(centre), page)?;
    let pivot = pdfce_core::vector::Point::new(f64::from(pdf.x), f64::from(pdf.y));
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        //
        // ★ It carries the ANGLE IN DEGREES and the pivot, which is what a
        // wrong build gets wrong: one that turned the other way, one that
        // pivoted about a corner instead of the centre, and one that snapped
        // when it should not are all `rotate-commit` otherwise.
        format!(
            "rotate-commit deg={:.2} px={:.2} py={:.2} objects={} constrained={}",
            (-theta).to_degrees(),
            pivot.x,
            pivot.y,
            objects.len(),
            u8::from(constrain),
        )
    });
    actions.push(
        crate::app::actions::VectorAction::TransformObjects {
            page: page_index,
            objects,
            // ★★ NEGATED here, and nowhere else. See this function's header.
            matrix: pdfce_core::vector::Matrix::rotate(f64::from(-theta)).about(pivot),
        }
        .into(),
    );
    None
}

/// The frame's facts about a rotate drag in flight.
///
/// The `Frame` shape `canvas::resizing` and `canvas::handledrag` already use,
/// and for the reason they give: the members are read-only facts about one
/// frame, so grouping them says what they are and removes the failure a long
/// parameter list invites — `from` and `at` are both `Pos2` in the same space
/// and swapping them would compile and turn the object backwards.
#[derive(Clone, Copy)]
pub struct Frame<'a> {
    /// Canvas-space position of the press — the first ray.
    pub from: Pos2,
    /// Canvas-space position of the pointer now — the second ray.
    pub at: Pos2,
    /// Draw the ghost, or commit.
    pub phase: crate::canvas::gesture::Phase,
    /// The selection's grip box in **screen** space, or `None` if there is no
    /// outline to have grabbed.
    pub bounds: Option<egui::Rect>,
    /// The page the drag is on.
    pub page_index: usize,
    /// Whether Shift is down **this frame** — snap to [`STEP_DEGREES`].
    pub constrain: bool,
    /// The frame's screen ⟷ canvas mapping.
    pub map: Option<&'a crate::canvas::mapping::PageMapping>,
    /// The page itself, for the canvas → PDF hop.
    pub page: Option<&'a pdfce_core::page_tree::Page>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn deg(d: f32) -> f32 {
        d.to_radians()
    }

    /// ★ **A quarter turn clockwise on screen reads as +90°.**
    ///
    /// The base case, and the one whose sign is easy to get backwards: screen y
    /// is DOWN, so a pointer moving from due-east to due-south of the centre has
    /// gone clockwise, and `atan2` in a y-down frame calls that positive. The
    /// caller negates once when it crosses into page space; getting the sign
    /// wrong here would rotate the object the other way and look like a
    /// perfectly deliberate feature.
    #[test]
    fn a_clockwise_quarter_turn_on_screen_is_positive() {
        let c = Pos2::new(100.0, 100.0);
        let got = angle(c, Pos2::new(200.0, 100.0), Pos2::new(100.0, 200.0), false).expect("rays");
        assert!((got - deg(90.0)).abs() < 1e-4, "got {}", got.to_degrees());
    }

    /// ★★ **The pointer's DISTANCE from the centre changes nothing.**
    ///
    /// The property that separates this gesture from a resize, asserted rather
    /// than assumed: an operator swinging a long arc for precision is doing what
    /// the gesture invites, and a build that let the radius in would shrink or
    /// grow the object while they did it.
    #[test]
    fn the_radius_does_not_matter() {
        let c = Pos2::new(0.0, 0.0);
        let near = angle(c, Pos2::new(10.0, 0.0), Pos2::new(0.0, 10.0), false).expect("rays");
        let far = angle(c, Pos2::new(900.0, 0.0), Pos2::new(0.0, 4.0), false).expect("rays");
        assert!((near - far).abs() < 1e-4, "{near} vs {far}");
    }

    /// ★★ **A drag past 180° turns the SHORT way rather than spinning back.**
    ///
    /// Without the normalisation a pointer moving smoothly through the ray
    /// behind the centre makes the object jump a full turn in one frame. It
    /// looks like a physics bug and it is an arithmetic one.
    #[test]
    fn crossing_the_far_ray_does_not_spin_a_whole_turn() {
        assert!((normalise(deg(190.0)) - deg(-170.0)).abs() < 1e-4);
        assert!((normalise(deg(-190.0)) - deg(170.0)).abs() < 1e-4);
        assert!((normalise(deg(179.0)) - deg(179.0)).abs() < 1e-4);
    }

    /// Shift lands on the right angles and the diagonals, which is what 15°
    /// divides both of.
    #[test]
    fn the_constraint_reaches_the_angles_anybody_wants() {
        for want in [0.0, 15.0, 45.0, 90.0, 180.0] {
            let near = deg(want + 4.0);
            assert!(
                (snap(near) - deg(want)).abs() < 1e-4,
                "{want}° was not reachable from {}°",
                near.to_degrees()
            );
        }
    }

    /// ★ **The TOTAL turn snaps, not each increment.**
    ///
    /// Accumulating snapped increments lets a slow drag through 90° arrive at
    /// 87°, because each frame's small delta rounds to zero. Asserted as the
    /// property rather than by simulating frames: `snap` is called on the whole
    /// angle and there is nowhere for an increment to be rounded.
    #[test]
    fn a_slow_drag_still_reaches_the_step() {
        // Half a degree at a time is what a careful hand produces; each one
        // snaps to zero, and their sum does not.
        assert!((snap(deg(0.5))).abs() < 1e-6, "one small step is no turn");
        assert!(
            (snap(deg(88.0)) - deg(90.0)).abs() < 1e-4,
            "the accumulated angle snaps to the step it is nearest"
        );
    }

    /// A twitch is not an edit.
    #[test]
    fn a_twitch_is_not_travel() {
        assert!(!is_travel(deg(0.05)));
        assert!(is_travel(deg(1.0)));
    }

    /// The pointer exactly on the centre has no bearing, and says so rather
    /// than answering zero.
    #[test]
    fn a_degenerate_ray_has_no_angle() {
        let c = Pos2::new(50.0, 50.0);
        assert!(angle(c, c, Pos2::new(60.0, 50.0), false).is_none());
        assert!(angle(c, Pos2::new(60.0, 50.0), c, false).is_none());
    }

    /// ★ **The ghost's rotation agrees with the angle that produced it.**
    ///
    /// `rotate_about` is the one duplication in this module — the ghost is drawn
    /// from it and the commit from `Matrix::rotate`. This pins the half that can
    /// be checked without a document: a point rotated by the angle measured
    /// between two rays lands on the second ray.
    #[test]
    fn the_ghost_map_agrees_with_the_measured_angle() {
        let c = Pos2::new(0.0, 0.0);
        let from = Pos2::new(100.0, 0.0);
        let at = Pos2::new(0.0, 100.0);
        let theta = angle(c, from, at, false).expect("rays");
        let moved = rotate_about(c, from, theta);
        assert!((moved.x - at.x).abs() < 1e-3, "x {moved:?}");
        assert!((moved.y - at.y).abs() < 1e-3, "y {moved:?}");
    }
}
