//! # `render::region` — turning "what is on screen" into "what to rasterize"
//!
//! `OPERATOR_REQUESTS.md` **O24**, and the failure that forced it into the
//! canvas on 2026-08-22:
//!
//! > *"I got a requested raster size 14580x18868 is empty or exceeds
//! > MAX_PIXMAP_EDGE when I got to 2382% zoom."*
//!
//! A US Letter page at 2382 % is 18,868 device pixels tall against a 16,384
//! cap. The whole-page raster cannot be made, and the answer he proposed is the
//! right one: *"reducing the raster sized area to around the cursor zoomed area
//! to what will fit"*.
//!
//! ## The two conversions, and why they are here rather than in the canvas
//!
//! | | |
//! |---|---|
//! | [`page_region`] | the visible part of a page, in the **PDF's** coordinates, ready for `render_page_region` |
//! | [`region_on_screen`] | where that rectangle's raster belongs on screen |
//!
//! They are exact inverses of each other and are the only place this shell
//! crosses between screen space and PDF space for a *raster*. Keeping them
//! together is what lets the round trip be a unit test — and a round trip is
//! precisely the property that matters, because getting one of the two slightly
//! wrong produces a page that is drawn *almost* in the right place, which reads
//! as a rendering bug rather than as a coordinate one.
//!
//! ## ★★ The y flip, which is the part that goes wrong
//!
//! Canvas space is y-**down** from the page's top-left; PDF user space is
//! y-**up** from its bottom-left. `render_page_region` documents its rectangle
//! as *"page space, pre-scale — the same coordinate system as `Page::crop_box`,
//! y-up"*, so the flip happens here, once, in both directions.
//!
//! ★ A flip that is applied twice is the identity, and a flip that is missed
//! shows the operator the *opposite end* of the page from the one they are
//! pointing at — which at 2382 % is a uniform field of whatever happens to be
//! there, and looks exactly like a blank raster.

use pdfce_core::page_tree::Rect;

/// The visible part of a page in **PDF user space**, y-up, ready for
/// `pdfce_render::render_page_region`.
///
/// * `visible_canvas` — what the operator can see of this page, in canvas
///   points from the page's top-left, y-down.
/// * `page_pts` — the page's own size in points, as
///   [`crate::viewer::page_extent_pts`] reports it.
///
/// The rectangle is quantised by [`super::strategy::region_for`] first, so a
/// small pan asks for the rectangle already rasterized — see that function for
/// why that is the difference between panning smoothly and waiting for a redraw
/// on every pixel of movement.
#[must_use]
pub fn page_region(visible_canvas: (f32, f32, f32, f32), page_pts: (f32, f32)) -> Rect {
    let (x0, y0, x1, y1) = super::strategy::region_for(visible_canvas);
    let height = f64::from(page_pts.1);
    // y-down from the top becomes y-up from the bottom, so the two y values
    // also swap ends: the canvas's smaller y is the PDF's larger one.
    Rect::from_corners(
        f64::from(x0),
        height - f64::from(y1),
        f64::from(x1),
        height - f64::from(y0),
    )
}

/// Where a region's raster belongs on screen, given where the whole page would
/// have been drawn.
///
/// `page_screen` is the rect the page occupies on screen — what the whole-page
/// texture would have filled. The returned rect is the sub-rectangle of it that
/// `region` covers, and it is routinely **larger than the screen and partly
/// negative**, because the region carries overscan beyond the viewport. That is
/// correct and must not be clamped: the texture covers that area, and clamping
/// the destination without cropping the source would stretch the image.
#[must_use]
pub fn region_on_screen(region: Rect, page_pts: (f32, f32), page_screen: egui::Rect) -> egui::Rect {
    let (w, h) = (f64::from(page_pts.0), f64::from(page_pts.1));
    if w <= 0.0 || h <= 0.0 {
        return page_screen;
    }
    let sx = f64::from(page_screen.width()) / w;
    let sy = f64::from(page_screen.height()) / h;
    // The flip again, in the other direction: the PDF's upper y is the
    // canvas's smaller one.
    let left = f64::from(page_screen.min.x) + region.llx * sx;
    let right = f64::from(page_screen.min.x) + region.urx * sx;
    let top = f64::from(page_screen.min.y) + (h - region.ury) * sy;
    let bottom = f64::from(page_screen.min.y) + (h - region.lly) * sy;
    egui::Rect::from_min_max(
        egui::pos2(left as f32, top as f32),
        egui::pos2(right as f32, bottom as f32),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// US Letter, which is the page the operator's failure was on.
    const LETTER: (f32, f32) = (612.0, 792.0);

    /// ★★★ **The two conversions are inverses**, which is the property the
    /// canvas actually depends on.
    ///
    /// Asserted as a round trip rather than against hand-computed numbers: a
    /// pair that agrees with each other draws the raster where it belongs, and
    /// two numbers that each look plausible in isolation can still disagree.
    #[test]
    fn a_region_maps_to_screen_and_back_to_itself() {
        let page_screen = egui::Rect::from_min_size(
            egui::pos2(100.0, 50.0),
            egui::vec2(612.0 * 4.0, 792.0 * 4.0),
        );
        let visible = (200.0, 300.0, 260.0, 345.0);
        let region = page_region(visible, LETTER);
        let on_screen = region_on_screen(region, LETTER, page_screen);

        // Recover the canvas-space rect from the screen rect and compare it to
        // the quantised region the conversion actually used.
        let sx = f64::from(page_screen.width()) / f64::from(LETTER.0);
        let sy = f64::from(page_screen.height()) / f64::from(LETTER.1);
        let back = (
            ((f64::from(on_screen.min.x) - f64::from(page_screen.min.x)) / sx) as f32,
            ((f64::from(on_screen.min.y) - f64::from(page_screen.min.y)) / sy) as f32,
            ((f64::from(on_screen.max.x) - f64::from(page_screen.min.x)) / sx) as f32,
            ((f64::from(on_screen.max.y) - f64::from(page_screen.min.y)) / sy) as f32,
        );
        let wanted = super::super::strategy::region_for(visible);
        for (got, want) in [
            (back.0, wanted.0),
            (back.1, wanted.1),
            (back.2, wanted.2),
            (back.3, wanted.3),
        ] {
            assert!(
                (got - want).abs() < 0.01,
                "round trip lost the rect: {back:?} vs {wanted:?}"
            );
        }
    }

    /// ★★ **The y flip happens, and in the right direction.**
    ///
    /// Looking at the TOP of the page must ask for the page's HIGH y in PDF
    /// space. A missed flip shows the opposite end of the sheet, which at deep
    /// zoom is a uniform field and reads as a blank raster rather than as a
    /// coordinate error.
    #[test]
    fn looking_at_the_top_of_the_page_asks_for_the_pdf_top() {
        // A window near the page's top edge, in canvas space (y-down).
        let region = page_region((0.0, 0.0, 60.0, 45.0), LETTER);
        assert!(
            region.ury > f64::from(LETTER.1) * 0.9,
            "the top of the page is the PDF's high y: {region:?}"
        );
        // …and near the bottom asks for low y.
        let low = page_region((0.0, 747.0, 60.0, 792.0), LETTER);
        assert!(
            low.lly < f64::from(LETTER.1) * 0.2,
            "the bottom of the page is the PDF's low y: {low:?}"
        );
    }

    /// ★ **The screen rect may extend past the page's own**, because the region
    /// carries overscan. Clamping it would stretch the texture, so this pins
    /// that it is left alone.
    #[test]
    fn the_screen_rect_may_reach_outside_the_page() {
        let page_screen = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(612.0, 792.0));
        // A window at the very top-left: the overscan reaches off the page.
        let region = page_region((0.0, 0.0, 60.0, 45.0), LETTER);
        let on_screen = region_on_screen(region, LETTER, page_screen);
        assert!(
            on_screen.min.x < page_screen.min.x || on_screen.min.y < page_screen.min.y,
            "overscan should reach outside the page: {on_screen:?}"
        );
    }

    /// A degenerate page yields the page's own rect rather than a division by
    /// zero — the canvas then draws as it always did.
    #[test]
    fn a_degenerate_page_falls_back_to_the_page_rect() {
        let page_screen = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(100.0, 100.0));
        let region = Rect::from_corners(0.0, 0.0, 10.0, 10.0);
        assert_eq!(
            region_on_screen(region, (0.0, 0.0), page_screen),
            page_screen
        );
    }
}
