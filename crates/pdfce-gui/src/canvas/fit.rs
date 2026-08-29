//! # `canvas::fit` — spending a fit command's request to place the view
//!
//! ## The request
//!
//! `OPERATOR_REQUESTS.md` O28, 2026-08-24:
//!
//! > *"If I press the Fit width or fit page button the view should center to
//! > the width as well or center the page."*
//!
//! ## ★★★ Why a fit is now a position as well as a scale
//!
//! Before O23's pasteboard a page no larger than the viewport had nowhere to
//! be except the middle, so *fit* and *centred* were the same act and the
//! button never had to choose between them. The pasteboard added a whole
//! viewport of slack on every side — deliberately, so any corner of the page
//! can be brought to any point of the screen — and with it the state the
//! operator is reporting: **the scale is right and the page is not on
//! screen.**
//!
//! ## The two-frame handshake, and why it is the same one the zoom anchor uses
//!
//! `Action::Fit` cannot place the view itself: the re-fitted zoom is computed
//! by `ViewState::apply_fit` from a viewport the action funnel cannot see, so
//! the page's new drawn size is not known until the canvas next runs. So the
//! action records the request on [`crate::app::state::OpenDoc::fit_placement`]
//! and this module spends it on the following frame, by which time
//! `apply_fit` has run near the top of `show_in` and `current_display` is the
//! page's **new** size.
//!
//! That is exactly the shape [`crate::canvas::zoom`]'s anchor uses, for
//! exactly the same reason, and the resemblance is not a coincidence worth
//! collapsing: an anchor says *"hold this page point where it is"* and a fit
//! says *"decide where the page goes"*, which on a pinned axis is the
//! statement that there is no previous position worth holding.
//!
//! ## Where the rules live, and why not here
//!
//! * *Which axes does this mode pin?* — [`crate::viewer::FitMode::pinned_axes`],
//!   beside `fit_scale`, because the two are one decision: an axis is pinned
//!   exactly when the fit has just decided its extent.
//! * *What offset does a pinned or unpinned axis get?* —
//!   [`crate::canvas::geometry::fit_placement_offset`], beside the other
//!   offset solves and the `margin` term whose definition makes the pinned
//!   answer a single constant.
//!
//! This file is the **frame plumbing** between them: read where the view is
//! now, ask the two rules, hand back an offset. Keeping it separate is what
//! stops either rule being restated in `canvas::show`.

use egui::{Rect, Vec2, vec2};

use crate::app::state::OpenDoc;
use crate::canvas::geometry;

/// **Spend a pending fit request and return where the view should go**, as a
/// page-local offset, or `None` on the overwhelming majority of frames where
/// no fit is pending.
///
/// ★ The request is **taken** whatever happens — including on a frame at the
/// deep-zoom tier, and on one where something else wins the scroll offset. A
/// request left pending would fire on whatever frame the caller's chain next
/// reached it, which the operator experiences as the view jumping for a button
/// they pressed some seconds ago. That is the failure mode `zoom::consume_anchor`'s
/// own `Drop` step exists to prevent, and it is prevented here the same way.
///
/// # Arguments
///
/// * `current_rect` — the acting page's rect **within the strip**, for its
///   origin. Under `PageDisplay::Single` that origin is `(0, 0)` and every
///   conversion below is the identity it always was.
/// * `current_display` — the acting page's drawn size, already re-fitted this
///   frame.
/// * `display_size` — the whole strip's drawn size.
/// * `vp` — the viewport measured before the scroll area was built, the same
///   measurement every margin term in [`geometry`] is derived against.
pub(super) fn placement(
    doc: &mut OpenDoc,
    current_rect: Rect,
    current_display: (f32, f32),
    display_size: Vec2,
    vp: Vec2,
) -> Option<Vec2> {
    // ★★★ **A pending request OR a live fit mode**, and the second half is
    // `OPERATOR_REQUESTS.md` **O55**, 2026-08-28:
    //
    // > *"if the canvas window is resized the pdf should resize to match"*
    //
    // ## What was here before, and the exact half it was missing
    //
    // This read `doc.fit_placement.take()?` alone — a **one-shot**, set by
    // `Action::Fit` and spent on the following frame. So the page was placed
    // when the operator pressed the button, and never again.
    //
    // `ViewState::apply_fit` meanwhile recomputes the **zoom** from the
    // viewport on *every* frame a fit mode is active, which is why a resize
    // already re-scaled correctly. Nothing re-placed it. The page therefore
    // grew or shrank about whatever offset it happened to be sitting at, and
    // drifted off centre — the scale right, the position stale, which is the
    // same pair O28 was about arriving through a different door.
    //
    // ⇒ **A fit is a MODE, so its placement is a mode too.** Recomputing the
    // scale every frame and the position once is the inconsistency the
    // operator was looking at.
    //
    // ## ★★ Why the one-shot survives rather than being replaced
    //
    // Because it is still the thing that fires on a frame where the mode was
    // *already* active — pressing **Fit page** while already fitted to page
    // must still recentre a view the operator has panned away, and the mode
    // alone cannot distinguish that frame from the sixty before it.
    //
    // ★ It is `take`n whatever happens, for the reason this function's own
    // note gives: a request left pending fires on some later frame and reads
    // as the view jumping for a button pressed seconds ago.
    let pending = doc.fit_placement.take();
    // ★★★ **A RESIZE, NOT A FRAME**, and the difference is a regression that
    // was written, run and caught the same hour.
    //
    // Re-placing on **every** frame while a fit is active is the obvious
    // reading of *"a fit is a mode, so its placement is a mode"*, and it is
    // wrong: under **Fit page** both axes are pinned, so the placement returns
    // the page's origin on every frame and **the wheel cannot scroll at all**.
    // In a continuous display that makes the document unnavigable, because the
    // wheel is how the next page is reached.
    //
    // ⇒ Caught by `a_fit_command_puts_the_page_on_screen`'s own precondition,
    // which scrolls into the pasteboard and **asserts it got there** before
    // pressing anything. It reported *"the pan did not move the page"* and
    // SKIPPED — a setup step refusing to proceed rather than a subject
    // failing, which is the shape a precondition is supposed to have and the
    // reason that check was written to establish its own.
    //
    // ★ The operator's sentence says it exactly: *"if the canvas window is
    // **resized** the pdf should resize to match"*. Resized, not redrawn.
    //
    // ★ Compared exactly rather than with a tolerance. A viewport that has not
    // changed produces bit-identical floats — it is the same measurement of the
    // same layout — and a tolerance would only decide how much of a resize is
    // allowed to be ignored, which is a question nobody has.
    let resized =
        doc.view.fit != crate::viewer::FitMode::None && doc.fit_viewport != Some((vp.x, vp.y));
    let mode = match pending {
        Some(requested) => requested,
        None if resized => doc.view.fit,
        None => return None,
    };
    let pinned = mode.pinned_axes()?;
    // Recorded whichever route got here, so the next frame is not a resize.
    // ★ Written even for a PENDING request: pressing Fit page establishes the
    // viewport the fit is now placed against, and without this the frame after
    // the button would read as a resize and re-place a view the operator may
    // have already begun scrolling.
    doc.fit_viewport = Some((vp.x, vp.y));
    // Where the view is now, expressed the way a single-page solve expects.
    // The PREVIOUS frame's settled offset, which is the only one available
    // before this frame's scroll area is built — and the correct one, because
    // nothing has moved the view since.
    let now = geometry::page_local_offset(
        (doc.last_scroll_offset.x, doc.last_scroll_offset.y),
        (current_rect.min.x, current_rect.min.y),
        (display_size.x, display_size.y),
        current_display,
        (vp.x, vp.y),
    );
    let (x, y) = geometry::fit_placement_offset(pinned, now, current_display, (vp.x, vp.y));
    Some(vec2(x, y))
}
