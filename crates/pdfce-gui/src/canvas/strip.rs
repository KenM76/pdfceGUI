//! # `canvas::strip` — wiring the laid-out strip to the frame
//!
//! The canvas's half of Phase 4, kept out of [`super`] because it answers a
//! different question and because that file is against rule R2's 1,500-line
//! ceiling.
//!
//! Three modules now carry the word *strip*, and the split between them is the
//! project's standing one — the unit-testable decision on one side, the wiring
//! on the other:
//!
//! | module | subject |
//! |---|---|
//! | [`crate::viewer::strip`] | **where** every page sits, as pure geometry |
//! | [`crate::render::strip`] | **what** each page's picture is, or says instead |
//! | this module | **which** page the frame is about, and in what order the rest should be drawn |
//!
//! Everything here is a decision the frame has to make once the scroll area has
//! settled and before the gesture layer runs, and every one of them is a pure
//! function of numbers the frame already has — which is exactly why they are
//! here rather than inline in [`super::show`], where a `Response` in the way
//! would make them untestable.

use egui::{Rect, Vec2, vec2};

use crate::canvas::mapping::PageMapping;

use crate::app::state::OpenDoc;
use crate::viewer;

/// One page the canvas drew this frame, and the widget that senses it.
///
/// Every visible page gets a `click_and_drag` response, not only the current
/// one, because pressing on a page is how the operator moves to it under a
/// continuous mode — see [`show`]. The response of exactly one of them becomes
/// the frame's interaction response.
pub(crate) struct DrawnPage {
    /// The 0-based page index.
    pub(crate) page: usize,
    /// Its rect on screen, in window logical points.
    pub(crate) rect: Rect,
    /// Its own sensing widget.
    pub(crate) response: egui::Response,
    /// Whether a raster was painted into it, as opposed to a state sentence.
    ///
    /// Recorded here rather than re-derived for the trace, because the caches
    /// are asked once — during the draw — and a second reading afterwards
    /// could disagree with what the operator is looking at.
    pub(super) has_raster: bool,
}

/// One page and its screen ⟷ canvas map.
///
/// The Find wash's unit of work. Separate from [`DrawnPage`] because it
/// outlives the scroll-area closure and holds no `Response`, and because
/// [`Frame`] is `Copy` while a `Response` is not.
#[derive(Debug, Clone, Copy)]
pub(super) struct PageView {
    /// The 0-based page index.
    pub(super) page: usize,
    /// Its screen ⟷ canvas map.
    pub(super) map: PageMapping,
}

/// What the **current** page has instead of a raster, if it has none.
///
/// The strip cache answers this for every other page ([`OpenDoc::strip_page_state`]);
/// the current page's answer comes from its own two fields, because its raster
/// lives in its own slot. See [`crate::render::strip`]'s header on why the
/// split exists.
///
/// `None` is unreachable in practice from the one call site — it is only asked
/// when there is no texture — and is answered as "waiting" rather than by
/// panicking, because the honest thing to draw on a page with no picture and
/// no stated reason is that it has not been drawn.
pub(super) fn current_page_state(doc: &OpenDoc) -> Option<crate::render::strip::PageState> {
    use crate::render::strip::PageState;
    if let Some(detail) = &doc.render_error {
        return Some(PageState::Refused(detail.clone()));
    }
    if doc
        .render_worker
        .rendering_key()
        .map(|k| k.page())
        .is_some_and(|p| p == doc.view.page_index)
    {
        return Some(PageState::Drawing);
    }
    Some(PageState::Waiting)
}

/// The pages drawn this frame, **nearest the viewport centre first**.
///
/// The order [`crate::render::settle`] fills the strip in. Nearest-first rather
/// than top-down because the operator is looking at the middle of the viewport:
/// filling from the top means the page they are reading is the last one to
/// arrive whenever they have scrolled to a boundary, which is exactly the
/// moment a continuous mode is being used.
///
/// `pages` is `(page index, the page's vertical centre on screen)` and
/// `centre_y` is the viewport's own vertical centre, both in **screen**
/// coordinates — one space, so there is no conversion here to get wrong. The
/// pair is passed rather than the [`DrawnPage`] slice so the ordering rule is a
/// pure function with a test ([`tests::the_render_order_starts_at_the_middle_of_the_viewport`]);
/// a `Response` cannot be constructed headlessly, and an ordering rule that
/// could only be checked by running a window is an ordering rule nobody checks.
pub(super) fn nearest_first(pages: &[(usize, f32)], centre_y: f32) -> Vec<usize> {
    let mut order: Vec<(f32, usize)> = pages
        .iter()
        .map(|&(page, y)| ((y - centre_y).abs(), page))
        .collect();
    // `total_cmp` rather than `partial_cmp().unwrap()`: a NaN distance is
    // reachable from a degenerate rect, and a comparator that panics on one
    // would take the whole frame down over a sort order.
    order.sort_by(|a, b| a.0.total_cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    order.into_iter().map(|(_, page)| page).collect()
}

/// ★ **The scroll offset that brings a navigated-to page onto the strip**, or
/// `None` when nothing navigated.
///
/// The third source of a forced scroll offset, and the one Phase 4 adds. Under
/// a continuous mode a page **command** — Next page, the status bar's page box,
/// a bookmark, a Find hit that landed with no geometry — changes
/// `view.page_index` and nothing else, so without this the operator would press
/// "Next page" and watch nothing happen.
///
/// # The gate is `page_index != tracked_page`, and nothing weaker works
///
/// The canvas writes `page_index` itself on every frame from the scroll
/// position, so "the index changed" is true of every frame of every scroll and
/// cannot be the test. `tracked_page` records what the *canvas* last derived;
/// a difference therefore means something else wrote the field, which is
/// precisely the definition of a navigation. See
/// [`crate::app::state::OpenDoc::tracked_page`].
///
/// # Where the page is put, and why it is the top rather than the centre
///
/// The page's top edge goes to the top of the viewport, less the strip's
/// row gap so the sheet does not sit flush against the edge. Not centred:
/// "Next page" means *show me that page*, and a reader expects to arrive at
/// the top of it and read downwards. Centring a page shorter than the viewport
/// would also scroll the previous page's foot into view above it, which reads
/// as having overshot.
///
/// Returns `None` in the paged modes, where a page command changes what is
/// laid out rather than where it is — there is nothing to scroll to.
pub(super) fn page_scroll_offset(
    doc: &mut OpenDoc,
    strip: &viewer::strip::Strip,
    viewport: (f32, f32),
) -> Option<Vec2> {
    if !doc.view.display.is_continuous() || doc.view.page_index == doc.tracked_page {
        return None;
    }
    let rect = strip.rect_of(doc.view.page_index)?;
    doc.tracked_page = doc.view.page_index;
    let size = strip.size();
    let x = (rect.center().x - viewport.0 / 2.0).clamp(0.0, (size.x - viewport.0).max(0.0));
    let y = (rect.min.y - viewer::strip::ROW_GAP).clamp(0.0, (size.y - viewport.1).max(0.0));
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "canvas-page-scroll page={} offset=({x:.1},{y:.1})",
            doc.view.page_index
        )
    });
    Some(vec2(x, y))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::{FOUR_PAGES, open_fixture};
    use crate::viewer::PageDisplay;
    use crate::viewer::strip::Strip;

    /// ★ **The renderer is pointed at the middle of the viewport first.**
    ///
    /// `render::settle` starts one render per frame and takes the first entry
    /// of this list that has no raster, so the order **is** the fill order. Top
    /// down would mean that whenever the operator has scrolled to a page
    /// boundary — which is when a continuous mode is being used — the page they
    /// are reading is the last one to arrive.
    #[test]
    fn the_render_order_starts_at_the_middle_of_the_viewport() {
        // Pages 3,4,5 on screen, the viewport's centre level with page 4.
        let pages = [(3usize, 100.0_f32), (4, 400.0), (5, 700.0)];
        assert_eq!(nearest_first(&pages, 400.0), vec![4, 3, 5]);
        // Scrolled so the centre sits between 4 and 5.
        assert_eq!(nearest_first(&pages, 560.0), vec![5, 4, 3]);
        // A single page is its own answer.
        assert_eq!(nearest_first(&[(7, 0.0)], 999.0), vec![7]);
        assert!(nearest_first(&[], 0.0).is_empty());
    }

    /// An exact tie takes the lower page index, so the order does not
    /// oscillate between two answers on alternate frames while the operator
    /// sits still.
    #[test]
    fn a_tie_in_the_render_order_is_broken_by_the_page_index() {
        let pages = [(9usize, 300.0_f32), (2, 100.0), (5, 100.0)];
        assert_eq!(nearest_first(&pages, 200.0), vec![2, 5, 9]);
    }

    /// A degenerate centre does not panic the frame over a sort order.
    #[test]
    fn a_non_finite_centre_still_produces_an_order() {
        let pages = [(0usize, 0.0_f32), (1, f32::NAN)];
        assert_eq!(nearest_first(&pages, 0.0).len(), 2);
    }

    /// ★ **A page command scrolls the strip; a scroll does not.**
    ///
    /// The gate this function exists for, from both sides. Without the first
    /// half, "Next page" in a continuous document does nothing at all; without
    /// the second, every frame of every scroll would fight the operator by
    /// snapping back to the page the last frame derived.
    #[test]
    fn a_page_command_scrolls_the_strip_and_a_scroll_does_not() {
        let mut doc = open_fixture(FOUR_PAGES);
        doc.view.display = PageDisplay::Continuous;
        let viewport = (612.0_f32, 400.0_f32);
        let strip = Strip::new(&doc.pages, doc.view.display, 0, 1.0);

        // Nothing has navigated: page_index == tracked_page.
        assert_eq!(page_scroll_offset(&mut doc, &strip, viewport), None);

        // A page command — `Action::NextPage` writes `page_index` and nothing
        // else, exactly as this simulates.
        doc.view.page_index = 2;
        let offset = page_scroll_offset(&mut doc, &strip, viewport)
            .expect("a navigation must move the strip");
        let rect = strip.rect_of(2).expect("page 2 is laid out");
        assert!(
            (offset.y - (rect.min.y - crate::viewer::strip::ROW_GAP)).abs() < 0.01,
            "the page's top must land at the top of the viewport: {offset:?} vs {rect:?}"
        );
        assert_eq!(doc.tracked_page, 2, "the scroll is spent once");

        // …and it is spent: asking again on the next frame moves nothing.
        assert_eq!(page_scroll_offset(&mut doc, &strip, viewport), None);
    }

    /// A paged mode has nothing to scroll to: the page command changes what is
    /// laid out, not where it is.
    #[test]
    fn a_paged_mode_never_scrolls_to_a_page() {
        let mut doc = open_fixture(FOUR_PAGES);
        let viewport = (612.0_f32, 400.0_f32);
        for &display in &[PageDisplay::Single, PageDisplay::Facing] {
            doc.view.display = display;
            doc.view.page_index = 3;
            doc.tracked_page = 0;
            let strip = Strip::new(&doc.pages, display, 3, 1.0);
            assert_eq!(page_scroll_offset(&mut doc, &strip, viewport), None);
        }
    }

    /// The forced offset never leaves the strip's scrollable range, so the
    /// scroll area is not handed a position it would immediately fight.
    #[test]
    fn the_forced_offset_stays_inside_the_scroll_range() {
        let mut doc = open_fixture(FOUR_PAGES);
        doc.view.display = PageDisplay::Continuous;
        // A viewport taller than the whole strip: every offset must be zero.
        let strip = Strip::new(&doc.pages, doc.view.display, 0, 1.0);
        let viewport = (10_000.0_f32, 10_000.0_f32);
        doc.view.page_index = 3;
        assert_eq!(
            page_scroll_offset(&mut doc, &strip, viewport),
            Some(Vec2::ZERO)
        );
    }
}
