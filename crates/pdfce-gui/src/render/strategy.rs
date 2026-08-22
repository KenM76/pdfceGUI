//! # `render::strategy` — whole page, or just the window?
//!
//! `OPERATOR_REQUESTS.md` **O24**. One decision, made in one place, from
//! numbers rather than from a mode flag: **at this zoom, on this page, do we
//! rasterize the whole sheet or only what is on screen?**
//!
//! ## ★★★ The constraint that shaped this, in the operator's words
//!
//! > *"I don't want to lose our capability to pan around a page and still see
//! > high detail as we pan. I don't want the affect that other readers have
//! > where you always have to wait for detail to render after panning to a new
//! > area."*
//!
//! He is describing the cost of region rendering, and he is right to refuse it
//! as a general answer:
//!
//! | | whole page | region |
//! |---|---|---|
//! | rasterized once per | **zoom** | **position** |
//! | what a pan costs | nothing — the texture exists and the view moves over it | a new raster, every time |
//! | what he sees while panning | full detail, immediately | blur or blank until it lands |
//!
//! Panning at full detail is a *property of rasterizing the whole page*, and it
//! is free precisely because the raster does not depend on where he is looking.
//! So region rendering may not simply replace it.
//!
//! ## The tiers, and why nothing is taken away to pay for anything
//!
//! | tier | when | panning |
//! |---|---|---|
//! | [`Strategy::WholePage`] | while the page's raster fits `MAX_PIXMAP_EDGE` | **free, full detail** |
//! | [`Strategy::Region`] | only above that | free within the overscan; a re-raster on leaving it |
//!
//! ★★ **The tier he works in does not change at all.** On an A1 sheet the
//! whole-page raster survives to about 1,034 %, and today `MAX_ZOOM` stops him
//! at 800 % first — so every zoom he has ever used keeps exactly the behaviour
//! he has, *by construction rather than by tuning*. There is no low-zoom
//! performance question to answer here, because at low zoom this module returns
//! [`Strategy::WholePage`] and nothing downstream is different.
//!
//! ★ And the region tier only ever engages where the zoom is currently
//! **unavailable**. It cannot regress anything, because there is nothing there
//! to regress.
//!
//! ## What this module is NOT
//!
//! It does not render, does not touch a cache, and does not know what a texture
//! is. It is arithmetic over four numbers, which is what lets the interesting
//! question — *where exactly does the switch happen, and does it move when the
//! window resizes?* — be answered by a unit test rather than by watching a
//! canvas.

/// How much extra to rasterize around the viewport, as a fraction of the
/// viewport on **each** side.
///
/// ★★ This is the dial the operator's constraint turns on, and its cost is
/// quadratic, so it is written down rather than tuned by feel:
///
/// | overscan | pixels | pans that cost nothing |
/// |---|---|---|
/// | `0.0` | 1× | none — every pixel of movement crosses the edge |
/// | `0.5` | **4×** | up to half a screen in any direction |
/// | `1.0` | 9× | up to a full screen |
///
/// `0.5` is the shipped value. At the zooms where the region tier engages the
/// viewport is a few hundred thousand pixels, so 4× of it is small in absolute
/// terms — which is the entire point of the region tier: **the raster stops
/// scaling with the zoom**, so a constant multiple of the window is affordable
/// where a constant multiple of the page would not be.
pub const OVERSCAN: f64 = 0.5;

/// What to hand the renderer for one page at one zoom.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Strategy {
    /// Rasterize the whole page. Today's path, and the one that makes panning
    /// free — see the module header.
    WholePage,
    /// Rasterize only the visible rectangle, plus [`OVERSCAN`] on each side.
    ///
    /// Carries the scale the caller should use; the rectangle itself is the
    /// canvas's to compute, because only it knows where the operator is
    /// looking. This module decides *whether*, not *where*.
    Region,
}

/// Decide the strategy for one page at one raster scale.
///
/// `page_pts` is the page's own size in PDF points, longest edge first or not —
/// both are examined. `raster_scale` is device pixels per PDF point, which is
/// the operator's zoom already multiplied by the display scale.
///
/// # Why the pixmap ceiling is the switch, and not a zoom percentage
///
/// A zoom threshold would be wrong on exactly the documents this shell is for.
/// The whole-page raster fails when `page × scale` exceeds
/// `MAX_PIXMAP_EDGE` — so a small page survives to a far higher zoom than a
/// large one, and an A0 sheet reaches the ceiling while an A5 is still
/// comfortable. Switching on the thing that actually fails means the operator
/// keeps free panning for as long as it is physically available, on every page
/// size, without anybody choosing a number per document class.
///
/// ★ It also means the switch **moves with the display scale**, which is
/// correct and would be easy to get wrong: `raster_scale` already includes
/// `pixels_per_point`, so a 150 % display reaches the ceiling at two-thirds the
/// zoom, exactly as it should.
#[must_use]
pub fn for_page(page_pts: (f32, f32), raster_scale: f32) -> Strategy {
    let longest = page_pts.0.max(page_pts.1);
    if !longest.is_finite() || longest <= 0.0 || !raster_scale.is_finite() || raster_scale <= 0.0 {
        // A degenerate page or scale cannot be reasoned about, and the
        // whole-page path already refuses it safely. Never answer `Region` on
        // bad input: that would send a nonsense rectangle to the renderer.
        return Strategy::WholePage;
    }
    #[allow(
        clippy::cast_precision_loss,
        reason = "MAX_PIXMAP_EDGE is 16384; f32 is exact to 2^24" // ui-text-exempt: clippy lint justification, never displayed
    )]
    let ceiling = (pdfce_render::MAX_PIXMAP_EDGE - 1) as f32;
    if longest * raster_scale <= ceiling {
        Strategy::WholePage
    } else {
        Strategy::Region
    }
}

/// The rectangle to rasterize for [`Strategy::Region`]: the visible rect grown
/// by [`OVERSCAN`] on each side.
///
/// Takes and returns page-space points. Growing here rather than at the call
/// site is what keeps the overscan one number in one place — a second caller
/// that grew it differently would produce a cache that never hits, because two
/// requests for the same view would ask for different rectangles.
#[must_use]
/// ★★ `f64`, since 2026-08-22 — see [`region_for`] for what `f32` cost here.
pub fn overscanned(visible: (f64, f64, f64, f64)) -> (f64, f64, f64, f64) {
    let (x0, y0, x1, y1) = visible;
    let w = (x1 - x0).abs();
    let h = (y1 - y0).abs();
    let dx = w * OVERSCAN;
    let dy = h * OVERSCAN;
    (x0 - dx, y0 - dy, x1 + dx, y1 + dy)
}

/// The page-space rectangle to rasterize for [`Strategy::Region`], **quantised**
/// so that small pans reuse the same raster.
///
/// `visible` is what the operator can see, in page points. The returned rect is
/// grown by [`OVERSCAN`] on each side and then snapped to a grid, and both
/// halves matter:
///
/// | | |
/// |---|---|
/// | **the overscan** | gives margin, so a pan inside it needs no new raster |
/// | **the quantisation** | makes the SAME view produce the SAME rect, so the cache hits |
///
/// ★★ Without the snap the rect would change on every pixel of movement, every
/// request would be a cache miss, and the operator would wait for a redraw
/// continuously — which is precisely the *"wait for detail to render after
/// panning"* he refused. The snap turns that into at most one redraw per half
/// viewport of travel.
///
/// ★ The grid step is half the visible extent rather than a constant: a
/// constant in page points would be a different distance on screen at every
/// zoom, so the redraw cadence would vary with magnification for no reason the
/// operator could see.
/// # ★★★ `f64`, and this is the arithmetic that forces it
///
/// `OPERATOR_REQUESTS.md` **O24i**. The snap divides a page coordinate by the
/// grid step:
///
/// ```text
/// snapped_x = (x0 / step_x).floor() * step_x
/// ```
///
/// At a trillion percent the visible extent is about 5 × 10⁻⁸ pt, so `step_x`
/// is 2 × 10⁻⁸ — while `x0` is an ordinary page coordinate near 540. Their
/// quotient is **2 × 10¹⁰**, and an `f32`'s last exactly representable integer
/// is 2²⁴ ≈ 1.7 × 10⁷. The `.floor()` is then applied to a number that has
/// already lost its integer part, and the snapped origin comes back quantised
/// to tens of `f32` ULPs.
///
/// ★ Measured before it was fixed: from about 10⁷ % the region stopped
/// shrinking and floored at 2.4414 × 10⁻³ × 3.0213 × 10⁻³ pt — **fifty
/// thousand times** the 4.8 × 10⁻⁸ × 6.2 × 10⁻⁸ the viewport actually showed.
/// The raster was still produced and `drawn=1` was still traced, so every
/// existing check passed; what the operator saw was a fraction of one texel
/// stretched across the window, which reads as blank paper.
///
/// ★★ The magnitudes here are the reason. This function mixes an **absolute
/// page position** with a **relative extent**, and at deep zoom those differ by
/// ten orders of magnitude — which is exactly the shape `f32` cannot hold. The
/// rest of the region path was already `f64` (`page_region` returns a `f64`
/// rect, `RenderKey` stores `f64` bits); this was the one narrowing left, and
/// it was narrowing the value the whole tier exists to compute.
#[must_use]
pub fn region_for(visible: (f64, f64, f64, f64)) -> (f64, f64, f64, f64) {
    let (x0, y0, x1, y1) = visible;
    let w = (x1 - x0).abs();
    let h = (y1 - y0).abs();
    if !(w.is_finite() && h.is_finite()) || w <= 0.0 || h <= 0.0 {
        return visible;
    }
    // Snap the ORIGIN to a half-viewport grid, then grow from there. Snapping
    // after growing would move the margin around instead of the window.
    let step_x = w * 0.5;
    let step_y = h * 0.5;
    let snapped_x = (x0 / step_x).floor() * step_x;
    let snapped_y = (y0 / step_y).floor() * step_y;
    overscanned((snapped_x, snapped_y, snapped_x + w, snapped_y + h))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The A1 sheet this project's benchmark and fixtures are built from.
    const A1_LONG_PT: f32 = 1584.0;

    /// ★★★ **The zoom the operator uses today does not change tiers.**
    ///
    /// `viewer::MAX_ZOOM` is 8.0 — 800 % — and at one device pixel per point an
    /// A1 sheet's whole-page raster is comfortably inside the ceiling there. If
    /// this ever fails, the shipped zoom range has started taking the region
    /// path, and the panning behaviour he asked to keep has silently changed.
    #[test]
    fn every_zoom_the_shell_offers_today_still_rasterizes_the_whole_page() {
        for zoom in crate::viewer::ZOOM_LADDER {
            assert_eq!(
                for_page((A1_LONG_PT, 1100.0), *zoom),
                Strategy::WholePage,
                "zoom {zoom} left the whole-page tier on an A1 sheet"
            );
        }
    }

    /// …and the switch happens where the raster would actually fail, rather
    /// than at a number somebody chose.
    #[test]
    fn the_switch_is_the_pixmap_ceiling() {
        #[allow(clippy::cast_precision_loss, reason = "16384 is exact in f32")]
        let ceiling = (pdfce_render::MAX_PIXMAP_EDGE - 1) as f32;
        let exact = ceiling / A1_LONG_PT;

        assert_eq!(for_page((A1_LONG_PT, 1100.0), exact), Strategy::WholePage);
        assert_eq!(
            for_page((A1_LONG_PT, 1100.0), exact * 1.01),
            Strategy::Region,
            "just past the ceiling the region tier must take over"
        );
    }

    /// ★ **A smaller page keeps free panning for longer**, which is the reason
    /// the switch is a pixmap size and not a zoom percentage.
    #[test]
    fn a_smaller_page_survives_to_a_higher_zoom() {
        let big = (A1_LONG_PT, 1100.0);
        let small = (306.0_f32, 396.0); // a quarter-letter slip
        let zoom = 20.0_f32; // 2,000 %

        assert_eq!(for_page(big, zoom), Strategy::Region);
        assert_eq!(
            for_page(small, zoom),
            Strategy::WholePage,
            "a small page should not be pushed into the region tier by a zoom it can afford"
        );
    }

    /// ★ The display scale is already inside `raster_scale`, so a 150 % display
    /// reaches the ceiling at two-thirds the zoom. Asserted because it is the
    /// kind of thing that is correct by accident and then broken by a
    /// refactor that "tidies up" the units.
    #[test]
    fn the_display_scale_moves_the_switch() {
        let page = (A1_LONG_PT, 1100.0);
        #[allow(clippy::cast_precision_loss, reason = "16384 is exact in f32")]
        let ceiling = (pdfce_render::MAX_PIXMAP_EDGE - 1) as f32;
        let zoom_at_1x = ceiling / A1_LONG_PT;

        // The same zoom on a 1.5x display is 1.5x the raster scale.
        assert_eq!(for_page(page, zoom_at_1x), Strategy::WholePage);
        assert_eq!(for_page(page, zoom_at_1x * 1.5), Strategy::Region);
    }

    /// Degenerate input never answers `Region`, because a nonsense rectangle
    /// would then be handed to the renderer.
    #[test]
    fn degenerate_input_falls_back_to_the_whole_page() {
        for bad in [f32::NAN, f32::INFINITY, 0.0, -1.0] {
            assert_eq!(for_page((A1_LONG_PT, 1100.0), bad), Strategy::WholePage);
            assert_eq!(for_page((bad, bad), 1.0), Strategy::WholePage);
        }
    }

    /// ★★★ **A small pan reuses the same raster** — the property the operator's
    /// constraint turns on.
    ///
    /// He refused *"the affect that other readers have where you always have to
    /// wait for detail to render after panning"*. A region that changed on every
    /// pixel would do exactly that, because every request would miss the cache.
    #[test]
    fn a_small_pan_asks_for_the_same_rectangle() {
        let base = region_for((1000.0, 1000.0, 1800.0, 1600.0));
        // Move a tenth of a viewport in each direction.
        for (dx, dy) in [(80.0, 0.0), (0.0, 60.0), (40.0, 30.0), (-40.0, -30.0)] {
            let moved = region_for((1000.0 + dx, 1000.0 + dy, 1800.0 + dx, 1600.0 + dy));
            assert_eq!(
                base, moved,
                "a {dx},{dy} pan changed the raster rect, so every pan would redraw"
            );
        }
    }

    /// …and a large pan does ask for a new one, or the operator would be looking
    /// at a raster that no longer covers the window.
    #[test]
    fn a_pan_past_the_margin_asks_for_a_new_rectangle() {
        let base = region_for((1000.0, 1000.0, 1800.0, 1600.0));
        let far = region_for((2000.0, 1000.0, 2800.0, 1600.0));
        assert_ne!(base, far);
    }

    /// ★★ **The raster is bounded by the WINDOW, not by the zoom** — which is
    /// the whole reason the region tier exists and the answer to the operator's
    /// `MAX_PIXMAP_EDGE` failure at 2382 %.
    ///
    /// The region's page-space size is the visible extent, which shrinks as the
    /// zoom rises — so its device size is a constant multiple of the viewport at
    /// every magnification.
    #[test]
    fn the_region_raster_stays_window_sized_at_any_zoom() {
        let viewport_px = 1400.0_f32;
        for zoom in [1.0_f32, 23.82, 1_000.0, 100_000.0] {
            // What the operator can see, in page points, at this zoom.
            let visible_pt = viewport_px / zoom;
            let r = region_for((0.0, 0.0, f64::from(visible_pt), f64::from(visible_pt)));
            let device = (r.2 - r.0) * f64::from(zoom);
            assert!(
                device <= f64::from(pdfce_render::MAX_PIXMAP_EDGE),
                "at {zoom}x the region would be {device} px, past the {} cap",
                pdfce_render::MAX_PIXMAP_EDGE
            );
            // …and it is the same size at every zoom, being 2x the window.
            assert!(
                (device - f64::from(viewport_px) * 2.0).abs() < 1.0,
                "{device} at {zoom}x"
            );
        }
    }

    /// The overscan grows the rect on every side, by the documented fraction.
    #[test]
    fn the_overscan_grows_every_side_by_half_a_viewport() {
        let (x0, y0, x1, y1) = overscanned((100.0, 200.0, 300.0, 400.0));
        // 200 wide, 200 tall; half of each is 100.
        assert!((x0 - 0.0).abs() < 0.001);
        assert!((y0 - 100.0).abs() < 0.001);
        assert!((x1 - 400.0).abs() < 0.001);
        assert!((y1 - 500.0).abs() < 0.001);
    }

    /// ★★ **The overscanned rect is a pure function of the visible rect**, so
    /// two requests for the same view ask for the same rectangle.
    ///
    /// That is what makes a raster cache possible at all. A caller that grew
    /// the rect itself, by a slightly different amount, would produce a cache
    /// that never hits — every pan a miss, which is exactly the "wait for
    /// detail" the operator refused.
    #[test]
    fn the_same_view_always_asks_for_the_same_rectangle() {
        let view = (12.5, 33.25, 812.5, 633.25);
        assert_eq!(overscanned(view), overscanned(view));
    }

    /// ★★★ O24i — **the region must keep shrinking all the way to the
    /// ceiling.**
    ///
    /// The snap divides an absolute page coordinate by the grid step. At a
    /// trillion percent the step is about 2 × 10⁻⁸ pt and the coordinate is an
    /// ordinary ~540, so the quotient is 2 × 10¹⁰ — past `f32`'s last exact
    /// integer of 2²⁴, which made `.floor()` meaningless and floored the
    /// region at **fifty thousand times** the size the viewport showed.
    ///
    /// The raster was still produced and still traced `drawn=1`, so every
    /// existing check passed while the operator saw a fraction of one texel
    /// stretched across the window — blank paper.
    ///
    /// ★ Asserted as a RATIO against the visible extent rather than against
    /// absolute sizes: what matters is that the rect stays proportional to
    /// what is on screen, at every depth, and a test of fixed numbers would
    /// have to be rewritten the next time `OVERSCAN` moves.
    #[test]
    fn the_region_stays_proportional_to_the_view_at_every_depth() {
        // A page coordinate far from the origin, which is the whole
        // difficulty: near zero even `f32` would cope.
        let at = 540.158_756_f64;
        for zoom in [1.0e3_f64, 1.0e5, 1.0e7, 1.0e9, 1.0e10, 1.0e12] {
            let w = 484.0 / zoom;
            let h = 619.0 / zoom;
            let r = region_for((at, at, at + w, at + h));
            let got_w = r.2 - r.0;
            let want_w = w * (1.0 + 2.0 * OVERSCAN);
            assert!(
                // ★★ A part in a thousand, and the slack is `f64`'s own.
                //
                // The extent is computed as `(x0 + w) - x0` at an absolute
                // position near 540, where an `f64` ULP is 1.1e-13. At a
                // trillion percent `w` is 1e-9 pt — about 8,800 ULPs — so the
                // subtraction returns a relative error near 1e-4 and no
                // implementation can do better while the position is absolute.
                //
                // ★ Which is also the real ceiling of this design, worth
                // stating: 8,800 representable steps across a 484-pixel
                // viewport is 18 per pixel, so the arithmetic is still
                // comfortable at the maximum zoom the shell offers. The tier
                // below it ran out at 2^24; this one has room left.
                (got_w / want_w - 1.0).abs() < 1e-3,
                "at zoom {zoom:e} the region is {got_w:e} pt wide, {:.1}x the {want_w:e} the \
                 overscanned view needs",
                got_w / want_w
            );
        }
    }

    /// ★★ …and the snapped origin must stay WITHIN one grid step of the view.
    ///
    /// The size test above would pass on an implementation that returned a
    /// correctly-sized rect somewhere else entirely — which is close to what
    /// the `f32` version did, since a meaningless `.floor()` corrupts the
    /// origin rather than the extent. Measured before the fix: the raster was
    /// placed 18,998,834 window points from the viewport at a trillion
    /// percent.
    #[test]
    fn the_snapped_origin_stays_next_to_the_view_at_every_depth() {
        let at = 540.158_756_f64;
        for zoom in [1.0e3_f64, 1.0e5, 1.0e7, 1.0e9, 1.0e10, 1.0e12] {
            let w = 484.0 / zoom;
            let h = 619.0 / zoom;
            let r = region_for((at, at, at + w, at + h));
            // The snap floors to a half-view grid and the overscan then grows
            // by half a view, so the origin can legitimately sit one and a
            // half views below the view's own. Anything beyond that is the
            // origin having been corrupted rather than quantised.
            let slack = w * 1.5 + h * 1.5;
            assert!(
                (at - r.0).abs() <= slack && (at - r.1).abs() <= slack,
                "at zoom {zoom:e} the region starts at ({:e}, {:e}), {:e} pt from the view at \
                 {at} — the snap has lost the coordinate rather than quantised it",
                r.0,
                r.1,
                (at - r.0).abs().max((at - r.1).abs())
            );
        }
    }
}
