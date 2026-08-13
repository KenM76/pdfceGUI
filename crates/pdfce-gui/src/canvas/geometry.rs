//! # `canvas::geometry` — the pure arithmetic behind panning and zooming
//!
//! **Salvaged from `D:\Dev\pdfce\crates\pdfce-gui\src\canvas.rs`** (Class
//! B, `SALVAGE.md`: *"the `CanvasTool` enum, dispatch, and the escape
//! ladder are sound concepts… this becomes several modules under
//! `canvas/`"*). This is the first of those modules: the two scroll-offset
//! solves, lifted with their documentation and their entire test suite.
//! The tool dispatch, the selection layer and the escape ladder stay behind
//! until the stages that need them (S4, S5).
//!
//! ## Why these are pure functions in their own file
//!
//! Both answer the same shape of question — *given where the view is and
//! what the operator just did, where should the scroll offset be?* — and
//! both are wrong in ways that are invisible in a screenshot and obvious in
//! use: a pan that rubber-bands, a zoom that slides the detail out from
//! under the pointer by an amount proportional to how far off-centre you
//! were pointing. Neither can be unit-tested through a `ScrollArea`; both
//! are trivially testable as arithmetic. So they are arithmetic, and the
//! widget code that calls them ([`super::show`]) is wiring.

/// The scroll offset a middle-drag pan should move to, clamped to what the
/// canvas can actually show.
///
/// # Why the clamp is not optional
///
/// The offset is subtracted, so the content follows the hand. Without a clamp
/// an unscrollable canvas — the page fitted inside the viewport, offset pinned
/// at zero — still accepts a negative target for one frame, so the page slides
/// with the pointer and then snaps back the instant the drag ends. Observed
/// exactly that on 2026-08-04: a 50 px slide and a 50 px jump back. Refusing to
/// move at all is the honest response to "there is nothing to pan to".
///
/// # Known limitation, deliberately left
///
/// This clamps to the PAGE, so the page edges cannot be dragged inward past the
/// viewport edge. The operator asked to "navigate beyond the page's edges",
/// which needs reserved space around the page rather than a different clamp —
/// a change to how the canvas reserves its content area, with a visible
/// consequence (scrollbars present at every zoom). That is a UX call, and this
/// function is the one place it would need to change.
#[must_use]
pub fn pan_offset(
    last: (f32, f32),
    pan: (f32, f32),
    display: (f32, f32),
    viewport: (f32, f32),
) -> (f32, f32) {
    fn axis(last: f32, pan: f32, d: f32, v: f32) -> f32 {
        if !(last.is_finite() && pan.is_finite() && d.is_finite() && v.is_finite()) {
            return last;
        }
        (last - pan).clamp(0.0, (d - v).max(0.0))
    }
    (
        axis(last.0, pan.0, display.0, viewport.0),
        axis(last.1, pan.1, display.1, viewport.1),
    )
}

/// The centring margin on one axis: half the slack when the page is smaller
/// than the viewport, zero once it is larger.
///
/// Lifted out of [`zoom_anchor_offset`] when [`anchor_screen_pos`] and
/// [`offset_holding_anchor_at`] were split out of it, so that all three read
/// the *same* margin. The margin is not a refinement — see
/// [`zoom_anchor_offset`]'s derivation — and two spellings of it would drift
/// apart in exactly the case that matters, the fit-page zoom an operator
/// starts from.
#[must_use]
fn margin(display: f32, viewport: f32) -> f32 {
    (display.max(viewport) - display) / 2.0
}

/// Where the page point at `anchor_frac` currently sits **relative to the
/// viewport's top-left**, in logical points.
///
/// The forward half of the pair this module's zoom solves are built from:
///
/// ```text
///     screen = margin(display, viewport) + anchor_frac * display - offset
/// ```
///
/// Not clamped and not guarded, deliberately. It is a *measurement* of where
/// something is, and a value outside `0 ..= viewport` is the true answer for a
/// point that has been scrolled off the edge of the view — clamping it would
/// silently claim the anchor was visible when it was not.
#[must_use]
pub fn anchor_screen_pos(
    anchor_frac: (f32, f32),
    offset: (f32, f32),
    display: (f32, f32),
    viewport: (f32, f32),
) -> (f32, f32) {
    fn axis(u: f32, off: f32, d: f32, v: f32) -> f32 {
        margin(d, v) + u * d - off
    }
    (
        axis(anchor_frac.0, offset.0, display.0, viewport.0),
        axis(anchor_frac.1, offset.1, display.1, viewport.1),
    )
}

/// The exact inverse of [`anchor_screen_pos`]: the scroll offset that would
/// put the page point at `anchor_frac` at viewport-relative position
/// `target`.
///
/// **Unclamped, and that is the point.** Two callers need the raw solve for
/// two different reasons and a clamp inside here would spoil both:
///
/// * [`zoom_anchor_offset`] applies the scrollable-range clamp *itself*, after
///   composing this with [`anchor_screen_pos`], because the clamp belongs to
///   the offset that is actually handed to a `ScrollArea` and not to an
///   intermediate;
/// * [`crate::canvas::zoom`] uses it to *fabricate a before-state* — "the
///   offset at which the anchor would have been sitting where we want it to
///   end up" — which is a hypothetical, not a scroll position, and clamping a
///   hypothetical into the current page's range would quietly change the
///   framing it describes.
///
/// A non-finite axis yields `0.0` on that axis. There is no honest answer to
/// "where would the offset have been" when one of the inputs is NaN, and `0.0`
/// is the one value guaranteed to be a legal scroll offset for any page — the
/// same "fail to a finite, harmless value" discipline `viewer` applies to a
/// degenerate zoom.
#[must_use]
pub fn offset_holding_anchor_at(
    anchor_frac: (f32, f32),
    target: (f32, f32),
    display: (f32, f32),
    viewport: (f32, f32),
) -> (f32, f32) {
    fn axis(u: f32, target: f32, d: f32, v: f32) -> f32 {
        let off = margin(d, v) + u * d - target;
        if off.is_finite() { off } else { 0.0 }
    }
    (
        axis(anchor_frac.0, target.0, display.0, viewport.0),
        axis(anchor_frac.1, target.1, display.1, viewport.1),
    )
}

// ---------------------------------------------------------------------------
// The strip ⟷ page-local bridge  (Phase 4)
// ---------------------------------------------------------------------------
//
// ★ **Why this pair exists, and what it buys.**
//
// Before Phase 4 the scroll area's content was one page, so a scroll offset and
// a page-relative offset were the same number. Under a continuous mode the
// content is a *strip* of pages and they are not — which threatens two solves
// that are deliberately owned elsewhere and must not be reimplemented here:
//
// * [`crate::canvas::zoom`] anchors every zoom against `ZoomAnchor`, whose
//   fields are a page fraction, a "before" offset and a "before" drawn size;
// * `crate::find::reveal::take_reveal_offset` scrolls a search hit into the
//   middle of the viewport from a page fraction and a page drawn size.
//
// Neither module is this work's to edit, and neither should be: the anchor
// rule and the reveal handshake are correct and are each asserted by their own
// suite. What they need is for the world to keep looking the way they expect —
// **one page, at the origin of the scroll content** — and that is exactly what
// these two functions provide. The canvas converts the real strip offset into
// the offset those solves would see if the current page were the only thing in
// the scroll area, hands it over, and converts the answer back.
//
// The conversion is exact, not an approximation, and
// [`tests::the_strip_bridge_preserves_where_a_page_point_lands_on_screen`]
// proves it the only way that matters: by asserting that a page point lands at
// the same screen position measured either way.
//
// One consequence is worth naming rather than discovering. `zoom_anchor_offset`
// clamps its answer to *the page's own* scroll range before the conversion
// back, so under a continuous mode an anchored zoom cannot scroll further than
// the current page's own extent in a single step. That is the same behaviour
// single-page mode has always had — the clamp is what stops an anchor near an
// edge from scrolling blank space into view — and applying it per page keeps
// a zoom about the cursor from throwing the operator onto a different sheet.

/// **Strip offset → the offset a single-page solve expects.**
///
/// `page_origin` is the current page's top-left in strip space (from
/// [`crate::viewer::strip::Strip::rect_of`]); `strip` is the strip's whole
/// drawn size; `page_display` is the current page's drawn size. Under
/// [`crate::viewer::PageDisplay::Single`] the origin is `(0,0)` and `strip`
/// equals `page_display`, so this is the identity — which is the mechanical
/// form of "the single-page path is untouched", asserted by
/// [`tests::the_strip_bridge_is_the_identity_for_a_single_page`].
///
/// Not clamped, deliberately: the result is fed to solves that do their own
/// clamping ([`zoom_anchor_offset`]) or that are measuring a hypothetical
/// (`offset_holding_anchor_at`), and a clamp here would quietly change what
/// they were asked.
#[must_use]
pub fn page_local_offset(
    strip_offset: (f32, f32),
    page_origin: (f32, f32),
    strip: (f32, f32),
    page_display: (f32, f32),
    viewport: (f32, f32),
) -> (f32, f32) {
    fn axis(off: f32, origin: f32, strip: f32, page: f32, v: f32) -> f32 {
        let out = off - origin - margin(strip, v) + margin(page, v);
        if out.is_finite() { out } else { 0.0 }
    }
    (
        axis(
            strip_offset.0,
            page_origin.0,
            strip.0,
            page_display.0,
            viewport.0,
        ),
        axis(
            strip_offset.1,
            page_origin.1,
            strip.1,
            page_display.1,
            viewport.1,
        ),
    )
}

/// **The exact inverse of [`page_local_offset`]: back to a strip offset.**
///
/// Clamped to the strip's scrollable range, because *this* is the value that
/// is handed to a `ScrollArea` — the same division of labour
/// [`offset_holding_anchor_at`] and [`zoom_anchor_offset`] already observe
/// between them, where the raw solve is unclamped and the offset that actually
/// reaches the widget is not.
#[must_use]
pub fn strip_offset(
    page_local: (f32, f32),
    page_origin: (f32, f32),
    strip: (f32, f32),
    page_display: (f32, f32),
    viewport: (f32, f32),
) -> (f32, f32) {
    fn axis(off: f32, origin: f32, strip: f32, page: f32, v: f32) -> f32 {
        let out = off + origin + margin(strip, v) - margin(page, v);
        if out.is_finite() {
            out.clamp(0.0, (strip - v).max(0.0))
        } else {
            0.0
        }
    }
    (
        axis(
            page_local.0,
            page_origin.0,
            strip.0,
            page_display.0,
            viewport.0,
        ),
        axis(
            page_local.1,
            page_origin.1,
            strip.1,
            page_display.1,
            viewport.1,
        ),
    )
}

/// Where the canvas must be scrolled to so the page point under the pointer
/// stays under the pointer across a zoom step — "zoom to cursor".
///
/// # Why this exists
///
/// Ctrl+wheel previously called `zoom_by` and nothing else. The scroll offset
/// was left alone, so the *viewport centre* was the fixed point of the zoom and
/// whatever the operator was pointing at slid away — worse the further from
/// centre they were pointing, which is exactly where a person zooms in on a
/// drawing detail. Every other application that zooms a canvas (browsers, CAD,
/// Inkscape, Office) anchors on the cursor, and the operator reported the old
/// behaviour as "jarring" on 2026-08-04 for that reason.
///
/// # The geometry
///
/// The page is drawn at `display` pixels inside a scroll-area content box of
/// `outer = max(display, viewport)` — the `max` is what lets the area still
/// scroll when the page is bigger AND centre the page when it is smaller (see
/// the reservation comment in [`super::show`]). So the page's top-left sits at
/// `margin = (outer - display) / 2` in content coordinates, and a point at
/// fraction `anchor_frac` of the page appears on screen at
///
/// ```text
///     screen = viewport_origin + margin + anchor_frac * display - offset
/// ```
///
/// Holding `screen` fixed across the step and solving for the new offset gives
///
/// ```text
///     offset₁ = offset₀ + anchor_frac * (display₁ - display₀) + (margin₁ - margin₀)
/// ```
///
/// which needs no knowledge of where the viewport is on screen — only sizes.
/// The margin term is not a refinement: while the page is smaller than the
/// viewport the offset is pinned at zero and *all* of the movement is the
/// margin shrinking, so dropping it would make zoom-to-cursor do nothing at
/// precisely the "fit page" zoom an operator starts from.
///
/// # Contract
///
/// - `anchor_frac` is the pointer's position as a fraction of the page's drawn
///   size, `(pointer - page_top_left) / display₀`. Values outside `0..=1` are
///   meaningful (the pointer may be in the centring margin) and are not clamped.
/// - The result is clamped to the scrollable range `0 ..= max(0, display₁ -
///   viewport)`, so a caller may hand it straight to `ScrollArea::scroll_offset`
///   without producing an offset the area would fight back against.
/// - Non-finite inputs yield `offset_before` unchanged: refusing to move is the
///   only safe answer, since a NaN offset would blank the canvas.
///
/// # ★ Expressed as "measure, then re-place", and why that is not a
/// refactor for its own sake
///
/// The body below is literally *"find where the anchor is on screen
/// ([`anchor_screen_pos`]), then find the offset that puts it back there at
/// the new size ([`offset_holding_anchor_at`])"*, and the composition is
/// algebraically identical to the closed form in the derivation above —
/// [`tests::the_split_solve_is_the_closed_form_it_replaced`] asserts that
/// against the original expression rather than trusting the algebra.
///
/// It is written this way because **zoom-to-region and zoom-to-selection need
/// the same solve with a different target**: not "back where it was" but "at
/// the centre of the viewport". With the two halves named, framing a rect is
/// the *same arithmetic with one substitution* rather than a second solve
/// living beside this one — and two independently-maintained scroll solves is
/// how the discrete zoom commands ended up anchoring the page's top-left while
/// the wheel anchored the cursor, which is the defect Phase 3.1 exists to fix.
#[must_use]
pub fn zoom_anchor_offset(
    offset_before: (f32, f32),
    display_before: (f32, f32),
    display_after: (f32, f32),
    viewport: (f32, f32),
    anchor_frac: (f32, f32),
) -> (f32, f32) {
    let finite = [
        offset_before.0,
        offset_before.1,
        display_before.0,
        display_before.1,
        display_after.0,
        display_after.1,
        viewport.0,
        viewport.1,
        anchor_frac.0,
        anchor_frac.1,
    ]
    .iter()
    .all(|f| f.is_finite());
    if !finite {
        return offset_before;
    }

    let held = anchor_screen_pos(anchor_frac, offset_before, display_before, viewport);
    let solved = offset_holding_anchor_at(anchor_frac, held, display_after, viewport);
    // The scrollable-range clamp, applied to the offset that is actually
    // handed to the `ScrollArea` — see the contract above, and
    // `anchoring_past_the_page_edge_saturates_rather_than_scrolling_into_blank_space`
    // for the one case where it visibly wins over the anchor.
    (
        solved.0.clamp(0.0, (display_after.0 - viewport.0).max(0.0)),
        solved.1.clamp(0.0, (display_after.1 - viewport.1).max(0.0)),
    )
}

#[cfg(test)]
#[allow(clippy::float_cmp, reason = "exact f32 arithmetic on exact literals")] // ui-text-exempt: clippy lint justification, never displayed
mod tests {
    use super::*;

    // ---- middle-drag pan ----------------------------------------------

    #[test]
    fn panning_moves_the_content_opposite_the_offset_so_the_page_follows_the_hand() {
        // Page twice the viewport, so there is room to move.
        let out = pan_offset(
            (500.0, 500.0),
            (30.0, -20.0),
            (1600.0, 1600.0),
            (800.0, 800.0),
        );
        assert_eq!(
            out,
            (470.0, 520.0),
            "dragging right must DECREASE the offset, or the page moves against the hand"
        );
    }

    #[test]
    fn an_unscrollable_canvas_refuses_to_pan_rather_than_rubber_banding() {
        // The fit-page case: page smaller than the viewport, offset pinned.
        // Before the clamp this returned -50 and the page visibly slid, then
        // snapped back when the drag ended.
        let out = pan_offset((0.0, 0.0), (50.0, 50.0), (600.0, 600.0), (800.0, 800.0));
        assert_eq!(out, (0.0, 0.0));
    }

    #[test]
    fn panning_stops_at_the_far_edge() {
        let out = pan_offset(
            (700.0, 0.0),
            (-500.0, 0.0),
            (1000.0, 1000.0),
            (800.0, 800.0),
        );
        assert_eq!(out.0, 200.0, "must not scroll past the end of the page");
    }

    // ---- zoom to cursor -----------------------------------------------

    /// The whole point, stated as the invariant rather than as an offset:
    /// re-derive where the anchored page point lands on screen after the step
    /// and assert it has not moved.
    ///
    /// Screen position is `margin + frac * display - offset`, which is the same
    /// expression the doc comment solves — so this checks the solve, not merely
    /// that the code agrees with itself about arithmetic (assert the outcome,
    /// not the intent).
    fn anchored_screen_x(off: f32, d: f32, v: f32, u: f32) -> f32 {
        (d.max(v) - d) / 2.0 + u * d - off
    }

    #[test]
    fn the_point_under_the_cursor_stays_under_the_cursor() {
        // A page larger than the viewport, pointer three quarters across —
        // i.e. far from centre, where the old centre-anchored behaviour was
        // most visibly wrong.
        let (v, u) = (800.0_f32, 0.75_f32);
        let (d0, d1) = (1200.0_f32, 1800.0_f32); // a 1.5x zoom in
        let off0 = 300.0_f32;
        let before = anchored_screen_x(off0, d0, v, u);

        let off1 = zoom_anchor_offset((off0, 0.0), (d0, d0), (d1, d1), (v, v), (u, u)).0;
        let after = anchored_screen_x(off1, d1, v, u);

        assert!(
            (after - before).abs() < 0.01,
            "the anchored point moved {} px across the zoom (before {before}, after {after})",
            after - before
        );
    }

    #[test]
    fn zooming_in_from_fit_page_moves_the_view_even_though_the_offset_starts_pinned() {
        // The case the margin term exists for: at "fit page" the page is
        // SMALLER than the viewport, so offset is 0 and cannot go lower.
        // Zooming past the viewport must start scrolling toward the anchor.
        let (v, u) = (800.0_f32, 0.9_f32); // pointer near the right edge
        let (d0, d1) = (600.0_f32, 2000.0_f32);
        let off1 = zoom_anchor_offset((0.0, 0.0), (d0, d0), (d1, d1), (v, v), (u, u)).0;
        assert!(
            off1 > 0.0,
            "zooming in past the viewport with the pointer off-centre must scroll toward it, \
             got {off1}"
        );
        let before = anchored_screen_x(0.0, d0, v, u);
        let after = anchored_screen_x(off1, d1, v, u);
        assert!(
            (after - before).abs() < 0.01,
            "the anchored point moved {} px",
            after - before
        );
    }

    #[test]
    fn the_offset_never_leaves_the_scrollable_range() {
        // Zooming OUT far enough that the page no longer fills the viewport
        // must land at 0 rather than at a negative offset the scroll area
        // would silently fight.
        let out = zoom_anchor_offset(
            (900.0, 900.0),
            (2000.0, 2000.0),
            (400.0, 400.0),
            (800.0, 800.0),
            (0.1, 0.1),
        );
        assert_eq!(
            out,
            (0.0, 0.0),
            "zoomed-out offset must clamp to the origin"
        );

        // And never past the far edge.
        let (v, d1) = (800.0_f32, 1000.0_f32);
        let out = zoom_anchor_offset((900.0, 0.0), (500.0, 500.0), (d1, d1), (v, v), (5.0, 0.0));
        assert!(
            out.0 <= d1 - v + 0.01,
            "offset {} exceeds the maximum scroll {}",
            out.0,
            d1 - v
        );
    }

    /// **The clamp wins over the anchor, deliberately.** Documented as its own
    /// test because it is the one case where zoom-to-cursor visibly does not
    /// hold the point still, and a future reader could mistake that for the
    /// bug this feature fixes.
    ///
    /// Anchoring near an edge can demand an offset past the end of the page.
    /// Honouring it would scroll blank space into view; every other canvas
    /// application saturates instead, so the anchored point drifts by exactly
    /// the amount the range was short. Found by a test that first asserted
    /// exact preservation here and failed by 60 px — the assertion was wrong,
    /// not the code.
    #[test]
    fn anchoring_past_the_page_edge_saturates_rather_than_scrolling_into_blank_space() {
        let (v, u) = (800.0_f32, 0.9_f32);
        let (d0, d1) = (600.0_f32, 1000.0_f32);
        let off1 = zoom_anchor_offset((0.0, 0.0), (d0, d0), (d1, d1), (v, v), (u, u)).0;
        let want = 0.9 * (d1 - d0) - 100.0; // 260: the unclamped solve
        let max = d1 - v; // 200: all the range there is
        assert!(
            want > max,
            "this case must actually be over-range to test it"
        );
        assert_eq!(off1, max, "the offset must saturate at the page edge");
    }

    /// ★ **The split solve is the closed form it replaced**, checked against
    /// the original expression rather than against itself.
    ///
    /// `zoom_anchor_offset` used to compute
    /// `off0 + u*(d1-d0) + (margin1 - margin0)` inline. It now composes
    /// [`anchor_screen_pos`] with [`offset_holding_anchor_at`] so that
    /// zoom-to-region can reuse the second half with a different target. This
    /// pins the equivalence over a spread of shapes — including the
    /// page-smaller-than-viewport case the margin term exists for, and the
    /// over-range case where the clamp bites — so a future edit to either half
    /// cannot silently change what Ctrl+wheel does.
    #[test]
    fn the_split_solve_is_the_closed_form_it_replaced() {
        fn closed_form(off0: f32, d0: f32, d1: f32, v: f32, u: f32) -> f32 {
            let margin = |d: f32| (d.max(v) - d) / 2.0;
            (off0 + u * (d1 - d0) + (margin(d1) - margin(d0))).clamp(0.0, (d1 - v).max(0.0))
        }
        for &(off0, d0, d1, v) in &[
            (300.0_f32, 1200.0_f32, 1800.0_f32, 800.0_f32),
            (0.0, 600.0, 2000.0, 800.0), // starts smaller than the viewport
            (900.0, 2000.0, 400.0, 800.0), // ends smaller: clamps to 0
            (0.0, 600.0, 1000.0, 800.0), // anchors past the edge: saturates
            (120.0, 1000.0, 1000.0, 1000.0), // no zoom change at all
        ] {
            for &u in &[0.0_f32, 0.1, 0.5, 0.9, 1.0, 5.0, -2.0] {
                let via_split =
                    zoom_anchor_offset((off0, off0), (d0, d0), (d1, d1), (v, v), (u, u)).0;
                let via_closed = closed_form(off0, d0, d1, v, u);
                assert!(
                    (via_split - via_closed).abs() < 1e-3,
                    "u={u} off0={off0} d0={d0} d1={d1} v={v}: {via_split} vs {via_closed}"
                );
            }
        }
    }

    /// ★ **`offset_holding_anchor_at` really is the inverse of
    /// `anchor_screen_pos`** — the property zoom-to-region's framing rests on.
    ///
    /// Framing a rect is "put this page point at the viewport centre", which
    /// is the second function with a chosen target. If the pair ever stopped
    /// being an exact inverse, a marquee zoom would land the region *near* the
    /// centre — an error small enough to look like a rounding artefact and
    /// large enough to be the wrong answer.
    #[test]
    fn placing_an_anchor_and_measuring_it_are_exact_inverses() {
        let v = (800.0_f32, 600.0_f32);
        for &d in &[(400.0_f32, 300.0_f32), (1600.0, 2400.0), (800.0, 600.0)] {
            for &u in &[(0.0_f32, 0.0_f32), (0.5, 0.5), (0.25, 0.9), (1.0, 0.0)] {
                for &target in &[(400.0_f32, 300.0_f32), (0.0, 0.0), (-120.0, 55.0)] {
                    let off = offset_holding_anchor_at(u, target, d, v);
                    let back = anchor_screen_pos(u, off, d, v);
                    assert!(
                        (back.0 - target.0).abs() < 1e-3 && (back.1 - target.1).abs() < 1e-3,
                        "u={u:?} d={d:?} target={target:?} came back as {back:?}"
                    );
                }
            }
        }
    }

    /// A non-finite request for a *hypothetical* offset yields the origin
    /// rather than a NaN that would propagate into a scroll offset and blank
    /// the canvas. The sibling guard to
    /// `a_non_finite_input_refuses_to_move_rather_than_blanking_the_canvas`,
    /// which cannot apply here because this function has no "before" state to
    /// refuse to move from.
    #[test]
    fn a_non_finite_placement_falls_back_to_the_origin() {
        assert_eq!(
            offset_holding_anchor_at((f32::NAN, 0.5), (10.0, 10.0), (100.0, 100.0), (80.0, 80.0)),
            // y: margin(100,80) = 0, so 0 + 0.5*100 - 10 = 40.
            (0.0, 40.0)
        );
    }

    // ---- the strip ⟷ page-local bridge --------------------------------

    /// ★ **Under single page the bridge is the identity.**
    ///
    /// The mechanical form of "continuous is an option, not a replacement":
    /// the default path must not merely *behave* the same, it must compute the
    /// same number. With one page in the strip its origin is `(0,0)` and the
    /// strip's size is the page's, so both terms cancel — at every zoom, and
    /// with the page both larger and smaller than the viewport (the latter is
    /// where the centring margin is non-zero and a sloppy conversion would
    /// show).
    #[test]
    fn the_strip_bridge_is_the_identity_for_a_single_page() {
        let v = (800.0_f32, 600.0_f32);
        for &page in &[(400.0_f32, 300.0_f32), (1600.0, 2400.0), (800.0, 600.0)] {
            for &off in &[(0.0_f32, 0.0_f32), (120.0, 55.0), (900.0, 1800.0)] {
                assert_eq!(page_local_offset(off, (0.0, 0.0), page, page, v), off);
                // The inverse clamps to the strip's range, which for a page
                // smaller than the viewport is zero — so compare against the
                // clamp rather than against the raw input.
                let expected = (
                    off.0.clamp(0.0, (page.0 - v.0).max(0.0)),
                    off.1.clamp(0.0, (page.1 - v.1).max(0.0)),
                );
                assert_eq!(strip_offset(off, (0.0, 0.0), page, page, v), expected);
            }
        }
    }

    /// ★ **The bridge preserves where a page point lands on screen.**
    ///
    /// The property the whole pair exists for, asserted as an *outcome*: take
    /// a fraction of the current page, work out where it appears on screen
    /// from the real strip geometry, then work it out again through the
    /// page-local view the zoom and reveal solves are handed — and require the
    /// two to agree. A conversion that dropped either margin term would pass
    /// every algebraic check and fail this one at exactly the zoom an operator
    /// starts from.
    #[test]
    fn the_strip_bridge_preserves_where_a_page_point_lands_on_screen() {
        let v = (800.0_f32, 600.0_f32);
        for &strip in &[(612.0_f32, 4000.0_f32), (1200.0, 500.0), (300.0, 200.0)] {
            for &page in &[(612.0_f32, 792.0_f32), (300.0, 200.0)] {
                for &origin in &[(0.0_f32, 0.0_f32), (0.0, 1200.0), (294.0, 2400.0)] {
                    for &off in &[(0.0_f32, 0.0_f32), (100.0, 900.0)] {
                        for &frac in &[(0.0_f32, 0.0_f32), (0.5, 0.5), (1.0, 0.25)] {
                            // Where it really is: the strip's own margin, plus
                            // the page's origin in the strip, plus the point
                            // inside the page, less the scroll offset.
                            let truth = (
                                (strip.0.max(v.0) - strip.0) / 2.0 + origin.0 + frac.0 * page.0
                                    - off.0,
                                (strip.1.max(v.1) - strip.1) / 2.0 + origin.1 + frac.1 * page.1
                                    - off.1,
                            );
                            // Where the single-page solves think it is.
                            let local = page_local_offset(off, origin, strip, page, v);
                            let via_bridge = anchor_screen_pos(frac, local, page, v);
                            assert!(
                                (via_bridge.0 - truth.0).abs() < 1e-2
                                    && (via_bridge.1 - truth.1).abs() < 1e-2,
                                "strip={strip:?} page={page:?} origin={origin:?} off={off:?} \
                                 frac={frac:?}: {via_bridge:?} vs {truth:?}"
                            );
                        }
                    }
                }
            }
        }
    }

    /// The two directions round-trip, so an offset handed to a single-page
    /// solve and brought back is the offset it started as — within the strip's
    /// scrollable range, which the return leg clamps to.
    #[test]
    fn the_strip_bridge_round_trips_within_the_scroll_range() {
        let v = (800.0_f32, 600.0_f32);
        let strip = (612.0_f32, 4000.0_f32);
        let page = (612.0_f32, 792.0_f32);
        let origin = (0.0_f32, 1200.0_f32);
        for &off in &[(0.0_f32, 0.0_f32), (0.0, 1500.0), (0.0, 3400.0)] {
            let local = page_local_offset(off, origin, strip, page, v);
            let back = strip_offset(local, origin, strip, page, v);
            assert!(
                (back.0 - off.0).abs() < 1e-2 && (back.1 - off.1).abs() < 1e-2,
                "{off:?} round-tripped to {back:?}"
            );
        }
    }

    /// The return leg never hands a `ScrollArea` an offset outside its range,
    /// and a non-finite input yields the origin rather than a NaN that would
    /// blank the canvas.
    #[test]
    fn the_return_leg_clamps_and_survives_a_nan() {
        let v = (800.0_f32, 600.0_f32);
        let strip = (612.0_f32, 4000.0_f32);
        let page = (612.0_f32, 792.0_f32);
        let out = strip_offset((99_000.0, 99_000.0), (0.0, 0.0), strip, page, v);
        assert_eq!(out, (0.0, strip.1 - v.1));
        let out = strip_offset((-9_000.0, -9_000.0), (0.0, 0.0), strip, page, v);
        assert_eq!(out, (0.0, 0.0));
        assert_eq!(
            strip_offset((f32::NAN, 100.0), (0.0, 0.0), strip, page, v).0,
            0.0
        );
        assert_eq!(
            page_local_offset((f32::NAN, 100.0), (0.0, 0.0), strip, page, v).0,
            0.0
        );
    }

    #[test]
    fn a_non_finite_input_refuses_to_move_rather_than_blanking_the_canvas() {
        // `anchor_frac` divides by the drawn page size, which is zero for one
        // frame after an open — so NaN really can reach here.
        let off0 = (120.0, 45.0);
        assert_eq!(
            zoom_anchor_offset(
                off0,
                (0.0, 0.0),
                (100.0, 100.0),
                (800.0, 800.0),
                (f32::NAN, 0.5)
            ),
            off0
        );
    }
}
