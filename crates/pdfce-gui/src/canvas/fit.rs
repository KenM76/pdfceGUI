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
    let mode = doc.fit_placement.take()?;
    let pinned = mode.pinned_axes()?;
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
