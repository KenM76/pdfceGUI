//! # `canvas::mapping` — the ONE screen↔page conversion, and the tolerance
//!
//! ## Why this file exists at all
//!
//! `GUI_ROADMAP.md` Phase 1 names three ways a selection model loses the
//! *"selection survives navigation"* invariant. The first is **selection
//! stored in screen coordinates**, and it has a twin that is easier to miss:
//!
//! > *"Every hit-test and snap `tolerance` is a PAGE-space radius, and
//! > nothing checks it. Pass raw screen pixels and it compiles, runs, and
//! > merely drifts with zoom"* (`hit.rs:118-120`, quoted in
//! > `D:\Dev\FeatureRequests\pdfce_FeatureRequests\README.md`).
//!
//! Both failures are the same mistake — *a screen number used where a page
//! number was meant* — and both are silent. So this module is the **single
//! boundary**: everything crossing it in one direction is screen space,
//! everything crossing it in the other is page space, and there is no second
//! place in `canvas/` that divides by `zoom`.
//!
//! Concretely: [`PageMapping`] holds the frame's page rect, extent and zoom,
//! and every conversion the selection layer needs is a method on it. A caller
//! that has a `PageMapping` cannot accidentally convert a point with this
//! frame's zoom and a tolerance with last frame's, because there is one zoom
//! and it is inside the mapping.
//!
//! ## "Page space" here means CANVAS space, and that is deliberate
//!
//! Three frames are in play and conflating any two of them is the classic
//! silent defect (`viewer`'s own header sets out the first two):
//!
//! | frame | Y | origin | who speaks it |
//! |---|---|---|---|
//! | **screen** | down | window top-left | egui, the pointer, the painter |
//! | **canvas** | down | page top-left, `/Rotate` applied | this module, [`crate::panels::objects::provider::ObjectModelProvider`]'s public surface, the raster |
//! | **PDF user** | **up** | un-rotated CropBox lower-left | the object model's *internals*, every `pdfce-core` authoring verb |
//!
//! [`PageMapping`] converts **screen ⟷ canvas** and stops there. The
//! canvas → PDF-user hop is the provider's own business
//! ([`crate::viewer::canvas_to_pdf_space`] is the per-point sibling), and it
//! is left there on purpose: it needs the page's device transform, it is
//! already implemented once by inverting the *renderer's* own transform, and
//! a second implementation here would be a second chance to get the Y-flip
//! backwards. **PDF user space is y-UP; canvas and screen are y-DOWN.** The
//! failure is silent — the page looks perfect until someone selects a line
//! and gets a different one.
//!
//! ## Why the tolerance is a distance and not a rect
//!
//! Canvas space at zoom 1.0 is *distance-preserving* with respect to PDF user
//! space: `page_device_geometry(page, 1.0)`'s transform is a rotation, a
//! Y-flip and a translation, none of which change lengths. So a radius of
//! *n* canvas units **is** a radius of *n* PDF units, and one number can
//! serve both — which is exactly why
//! [`crate::panels::objects::provider::FALLBACK_SELECT_TOLERANCE`] can be
//! documented as "canvas space, and in effect page space" without a
//! conversion. If the canvas ever gained a non-uniform scale that would stop
//! being true, and this paragraph is where it would have to be revisited.

use egui::{Pos2, Rect};

use crate::viewer;

/// The screen-space catch radius for **object selection**, in egui logical
/// points, converted to a canvas/page-space tolerance per query by
/// [`screen_tolerance_to_page`].
///
/// # Why 6, and why a screen number rather than a page number
///
/// Salvaged verbatim from the old shell, with its reasoning, because the
/// reasoning is the valuable part. The behaviour it replaced was a fixed
/// `3.0` **canvas-space** value, which is `3.0 × zoom` pixels on screen: 3 px
/// at 100%, 1.5 px at 50%, 0.75 px at 25%. Objects were effectively
/// unclickable at exactly the zoom an operator uses to see a whole drawing.
///
/// Deliberately a *sibling* of the snap radius rather than the same constant:
/// snapping and selection answer different questions and are allowed to drift
/// apart. Selection is set **tighter** because a snap that grabs a nearby
/// vertex is a helpful correction the operator can see and cycle through,
/// whereas a selection that grabs a neighbouring object is a silent wrong
/// answer. The failure modes are not symmetric, so the tolerances should not
/// be either.
///
/// # This constant lives HERE and nowhere else
///
/// `panels::objects::provider`'s salvage note records that one test did not
/// come across —
/// `screen_tolerance_keeps_the_on_screen_catch_radius_constant` — because
/// *"re-declaring those constants here to keep a test green would put the
/// tolerance in two places, which is the cause of the defect the test guards,
/// not a way to guard it."* This module is where the constant landed, and
/// [`tests::screen_tolerance_keeps_the_on_screen_catch_radius_constant`] is
/// that test, restored.
pub const SELECT_SCREEN_TOLERANCE_PX: f32 = 6.0;

/// Convert a fixed SCREEN-space pixel radius into a **canvas/page-space**
/// tolerance at `zoom` (points per PDF user-space unit).
///
/// This is the exact `1 / zoom` distance law
/// [`crate::viewer::screen_to_page`] uses, proven zoom-invariant by that
/// module's `screen_to_page_distance_scales_as_one_over_zoom` test. A
/// constant on-screen catch radius therefore maps to a *shrinking*
/// page-space tolerance as the operator zooms in, which is what keeps the
/// click target feeling identical at every zoom.
///
/// # Degenerate inputs yield `0.0`, and that is not a silent failure
///
/// A non-finite or non-positive `zoom` (reachable: the page's drawn size is
/// zero for one frame after an open, and a fit scale on a degenerate CropBox
/// can go non-finite) returns `0.0` rather than a NaN or an infinity. `0.0`
/// is then recognised by
/// [`crate::panels::objects::provider`]'s `resolve` as degenerate and
/// replaced with the fixed canvas-space fallback — so a bad zoom makes
/// selection *fussy for one frame*, never *broken*. Returning NaN instead
/// would make every comparison in the hit test false and every query a miss,
/// with nothing to say why.
#[must_use]
pub fn screen_tolerance_to_page(screen_px: f32, zoom: f32) -> f64 {
    if zoom.is_finite() && zoom > 0.0 && screen_px.is_finite() && screen_px >= 0.0 {
        f64::from(screen_px) / f64::from(zoom)
    } else {
        0.0
    }
}

/// The frame's screen ⟷ canvas map: where the page raster is, how big the
/// page is, and at what zoom it is drawn.
///
/// Constructed once per frame in [`crate::canvas::show`], immediately after
/// the scroll area has settled and the page's true drawn rect is known, and
/// then handed to everything that needs a coordinate. **Nothing downstream
/// of it sees a screen coordinate again**, except the overlay, which converts
/// back through [`Self::to_screen`] at the moment of painting.
///
/// `Copy` because it is three small values and passing it by value removes
/// any question of it being stale: a mapping is a fact about one frame, and a
/// borrow that outlived the frame would be a mapping for a page rect that has
/// since moved.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageMapping {
    /// The page raster's own rect in window logical points — the rect every
    /// canvas coordinate conversion is relative to.
    ///
    /// This is the `Response::rect` of the image widget, **not** the scroll
    /// viewport and **not** the justified container. `canvas/mod.rs`'s
    /// centring comment records what taking the wrong one costs: at fit-page
    /// on a page smaller than the viewport, every mapping is wrong by the
    /// centring margin (~105 px on one measured case), selection outlines
    /// draw offset from the object they outline, and clicking directly ON a
    /// visible object misses it.
    image_rect: Rect,
    /// The current page's extent in PDF user-space units, `/Rotate` applied
    /// — [`crate::viewer::page_extent_pts`].
    ///
    /// Consulted only to reject a degenerate page; the mapping itself carries
    /// no rotation branch, because rotation is already baked into this value.
    /// Adding one here as well would double-apply it.
    extent: (f32, f32),
    /// Logical points per PDF user-space unit — [`crate::viewer::ViewState::zoom`].
    zoom: f32,
}

impl PageMapping {
    /// Build the mapping for this frame.
    #[must_use]
    pub fn new(image_rect: Rect, extent: (f32, f32), zoom: f32) -> Self {
        Self {
            image_rect,
            extent,
            zoom,
        }
    }

    /// The page raster's rect on screen.
    ///
    /// Deliberately the only way *out* of this type other than a conversion.
    /// There is no `zoom()` accessor: the zoom's whole job here is to be
    /// divided by, and exposing it would be an invitation to divide by it at
    /// a call site — which is the defect this module exists to make
    /// unavailable. Anything that needs a page-space distance asks
    /// [`Self::tolerance`].
    #[must_use]
    pub fn image_rect(&self) -> Rect {
        self.image_rect
    }

    /// **Screen → canvas.** The boundary crossing, inward.
    #[must_use]
    pub fn to_page(&self, screen: Pos2) -> Pos2 {
        viewer::screen_to_page(screen, self.image_rect, self.extent, self.zoom)
    }

    /// **Canvas → screen.** The boundary crossing, outward — used by the
    /// overlay and by nothing else.
    #[must_use]
    pub fn to_screen(&self, page: Pos2) -> Pos2 {
        viewer::page_to_screen(page, self.image_rect, self.extent, self.zoom)
    }

    /// **Screen → canvas** for a rect (the marquee).
    ///
    /// Normalised with [`Rect::from_two_pos`] rather than assembled from
    /// `min`/`max`, because a rubber-band is dragged in any of four
    /// directions and its "min" corner is wherever the press happened to be.
    /// A non-normalised rect has negative width and every containment test
    /// against it silently answers `false`.
    #[must_use]
    pub fn rect_to_page(&self, screen: Rect) -> Rect {
        Rect::from_two_pos(self.to_page(screen.min), self.to_page(screen.max))
    }

    /// **Canvas → screen** for a rect (a selection outline).
    ///
    /// Normalised for the same reason [`Self::rect_to_page`] is: the
    /// screen↔canvas map is a pure scale and translate with no flip, but the
    /// rects it is handed come from the provider, which built them by
    /// bounding a *mapped quad* under a transform that may rotate. Assuming
    /// corner order here would produce an inside-out rect that paints nothing.
    #[must_use]
    pub fn rect_to_screen(&self, page: Rect) -> Rect {
        Rect::from_two_pos(self.to_screen(page.min), self.to_screen(page.max))
    }

    /// The selection catch radius for this frame, in **canvas/page** units.
    ///
    /// The one call every hit test makes. Passing
    /// [`SELECT_SCREEN_TOLERANCE_PX`] straight to a provider query — which
    /// compiles, and runs, and merely drifts with zoom — is the defect this
    /// method exists to make unavailable.
    #[must_use]
    pub fn tolerance(&self) -> f64 {
        screen_tolerance_to_page(SELECT_SCREEN_TOLERANCE_PX, self.zoom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A mapping for a 200×300 page drawn at `zoom`, with the page's
    /// top-left at a deliberately non-zero screen position — a mapping that
    /// forgot the origin would still pass every *distance* assertion, so the
    /// origin has to be somewhere a bug could show up.
    fn mapping(zoom: f32) -> PageMapping {
        let extent = (200.0_f32, 300.0_f32);
        let rect = Rect::from_min_size(
            Pos2::new(37.0, 11.0),
            egui::vec2(extent.0 * zoom, extent.1 * zoom),
        );
        PageMapping::new(rect, extent, zoom)
    }

    /// ★ **The law this module exists for**, restored from the old shell.
    ///
    /// `panels::objects::provider`'s salvage note §4 records that this test
    /// could not come across with the provider, because asserting it there
    /// would have meant re-declaring the constant — putting the tolerance in
    /// two places, which is the *cause* of the defect it guards. It lands
    /// here, with the constant and the conversion it is about.
    ///
    /// The property: the canvas-space tolerance a click supplies scales as
    /// `1 / zoom`, so the SCREEN-space catch radius is the same number of
    /// pixels at every zoom level. Assert the *outcome* (the on-screen
    /// radius) rather than the intermediate (the page-space number), so this
    /// checks the law and not merely that the code agrees with itself.
    #[test]
    fn screen_tolerance_keeps_the_on_screen_catch_radius_constant() {
        for zoom in [0.10_f32, 0.25, 0.5, 1.0, 2.0, 4.0, 8.0] {
            let page_tol = screen_tolerance_to_page(SELECT_SCREEN_TOLERANCE_PX, zoom);
            // Canvas units × zoom = screen px, by the same distance law
            // `viewer::screen_to_page` uses.
            let screen_px = page_tol * f64::from(zoom);
            assert!(
                (screen_px - f64::from(SELECT_SCREEN_TOLERANCE_PX)).abs() < 1e-6,
                "zoom {zoom}: on-screen catch radius drifted to {screen_px} px"
            );
        }
    }

    /// The same law, asserted through the mapping rather than through the
    /// free function — because the mapping is what call sites actually hold,
    /// and a mapping that forgot to divide would pass the test above.
    #[test]
    fn the_mappings_tolerance_is_the_same_screen_radius_at_every_zoom() {
        for zoom in [0.10_f32, 0.5, 1.0, 3.0, 8.0] {
            let m = mapping(zoom);
            let screen_px = m.tolerance() * f64::from(zoom);
            assert!(
                (screen_px - f64::from(SELECT_SCREEN_TOLERANCE_PX)).abs() < 1e-6,
                "zoom {zoom}: mapping tolerance is {screen_px} screen px"
            );
        }
    }

    /// A degenerate zoom disables the *conversion*, not selection: the
    /// provider recognises `0.0` and falls back. Returning NaN here would
    /// make every hit-test comparison false, i.e. every query a miss, with
    /// nothing anywhere to say why.
    #[test]
    fn a_degenerate_zoom_yields_a_zero_tolerance_rather_than_a_nan() {
        assert!((screen_tolerance_to_page(10.0, 0.0) - 0.0).abs() < f64::EPSILON);
        assert!((screen_tolerance_to_page(10.0, -1.0) - 0.0).abs() < f64::EPSILON);
        assert!((screen_tolerance_to_page(10.0, f32::NAN) - 0.0).abs() < f64::EPSILON);
        assert!((screen_tolerance_to_page(f32::NAN, 1.0) - 0.0).abs() < f64::EPSILON);
        // And the plain arithmetic, so a refactor that "simplified" the
        // guard away is caught by more than the degenerate cases.
        assert!((screen_tolerance_to_page(10.0, 2.0) - 5.0).abs() < f64::EPSILON);
        assert!((screen_tolerance_to_page(10.0, 0.5) - 20.0).abs() < f64::EPSILON);
    }

    /// Screen → canvas → screen is the identity, at every zoom, for points
    /// inside and outside the page rect.
    ///
    /// Outside matters: a marquee is routinely dragged past the page edge,
    /// and a mapping that clamped would silently shrink the rubber-band.
    #[test]
    fn the_boundary_round_trips_in_both_directions() {
        for zoom in [0.10_f32, 0.5, 1.0, 2.5, 8.0] {
            let m = mapping(zoom);
            for p in [
                m.image_rect().min,
                m.image_rect().center(),
                m.image_rect().max,
                Pos2::new(-40.0, -90.0),
                Pos2::new(5_000.0, 5_000.0),
            ] {
                let back = m.to_screen(m.to_page(p));
                assert!(
                    (back.x - p.x).abs() < 1e-2 && (back.y - p.y).abs() < 1e-2,
                    "zoom {zoom}: {p:?} round-tripped to {back:?}"
                );
            }
        }
    }

    /// ★ **A canvas coordinate does not move when the view does.**
    ///
    /// The arithmetic half of the "selection survives navigation" invariant:
    /// the *same object point* has the same canvas coordinate at every zoom
    /// and every scroll position, which is exactly why a selection held in
    /// canvas/identity terms survives navigation and one held in screen
    /// terms cannot.
    ///
    /// Modelled by taking a fixed canvas point, projecting it to screen at
    /// each zoom (with the page rect moving as the scroll area would move
    /// it), and converting back.
    #[test]
    fn a_canvas_point_survives_every_zoom_and_scroll_position() {
        let extent = (200.0_f32, 300.0_f32);
        let subject = Pos2::new(123.0, 45.0); // a point on the page
        for zoom in [0.10_f32, 0.33, 1.0, 2.0, 8.0] {
            for origin in [
                Pos2::new(0.0, 0.0),
                Pos2::new(37.0, 11.0),
                Pos2::new(-900.0, -1_400.0), // scrolled far into a big page
            ] {
                let m = PageMapping::new(
                    Rect::from_min_size(origin, egui::vec2(extent.0 * zoom, extent.1 * zoom)),
                    extent,
                    zoom,
                );
                let on_screen = m.to_screen(subject);
                let back = m.to_page(on_screen);
                assert!(
                    (back.x - subject.x).abs() < 1e-2 && (back.y - subject.y).abs() < 1e-2,
                    "zoom {zoom} origin {origin:?}: the point moved to {back:?}"
                );
            }
        }
    }

    /// A rubber-band dragged up-and-left normalises rather than producing a
    /// negative-width rect that contains nothing.
    #[test]
    fn a_backwards_marquee_normalises() {
        let m = mapping(2.0);
        let dragged_up_left = Rect::from_two_pos(Pos2::new(300.0, 400.0), Pos2::new(100.0, 150.0));
        let page = m.rect_to_page(dragged_up_left);
        assert!(page.width() > 0.0 && page.height() > 0.0);
        assert!(page.contains(m.to_page(Pos2::new(200.0, 300.0))));
    }

    /// ★ **Each page of a strip gets its OWN mapping, and they are not
    /// interchangeable.**
    ///
    /// The failure this pins is the one Phase 4 was most likely to ship
    /// silently: under a continuous mode the Find wash has to be painted for
    /// several pages at once, and painting them all through the *acting*
    /// page's mapping would stack every page's highlights onto one page. The
    /// hits would still be found, the wash would still be drawn, and it would
    /// be drawn in the wrong place — which reads as a highlight bug rather
    /// than a mapping one, and is exactly the class `canvas/mod.rs`'s own
    /// centring comment records the old GUI shipping.
    ///
    /// Asserted as the *outcome*: the same canvas point — the top-left corner
    /// of a page — projects to each page's own screen origin through that
    /// page's mapping, and to somewhere else entirely through its neighbour's.
    /// The second half is what makes this a test rather than a tautology; a
    /// build in which the two mappings were accidentally equal would pass the
    /// first half.
    #[test]
    fn each_page_of_a_strip_has_its_own_mapping() {
        use crate::viewer::PageDisplay;
        use crate::viewer::strip::Strip;
        use pdfce_core::object::{Dict, ObjId};
        use pdfce_core::page_tree::{Page, Rect as PageRect};

        let pages: Vec<Page> = (0..3)
            .map(|_| Page {
                id: ObjId::new(1, 0),
                resources: Dict::new(),
                media_box: PageRect::from_corners(0.0, 0.0, 612.0, 792.0),
                crop_box: PageRect::from_corners(0.0, 0.0, 612.0, 792.0),
                rotate: 0,
                contents: Vec::new(),
                contents_unresolved: 0,
            })
            .collect();
        let zoom = 1.5_f32;
        let strip = Strip::new(&pages, PageDisplay::Continuous, 0, zoom);
        // The strip's own origin on screen, somewhere non-zero so a mapping
        // that forgot it would still pass every *distance* assertion.
        let strip_origin = egui::vec2(37.0, 11.0);
        let extent = crate::viewer::page_extent_pts(&pages[0]);

        let maps: Vec<(usize, PageMapping)> = strip
            .placements()
            .map(|p| {
                (
                    p.page,
                    PageMapping::new(p.rect.translate(strip_origin), extent, zoom),
                )
            })
            .collect();
        assert_eq!(maps.len(), 3, "the strip must lay out every page");

        // A hit at the top-left of its own page lands at that page's own
        // screen origin — through that page's map.
        for (page, map) in &maps {
            let rect = strip
                .rect_of(*page)
                .expect("laid out")
                .translate(strip_origin);
            let landed = map.to_screen(Pos2::ZERO);
            assert!(
                (landed - rect.min).length() < 1e-2,
                "page {page}: {landed:?} is not that page's origin {:?}",
                rect.min
            );
        }

        // …and through the WRONG page's map it lands somewhere else, by a
        // whole page height plus the row gap. This is the defect, measured.
        let wrong = maps[0].1.to_screen(Pos2::ZERO);
        let right = maps[1].1.to_screen(Pos2::ZERO);
        let apart = (right.y - wrong.y).abs();
        assert!(
            apart > 700.0,
            "the two mappings differ by only {apart} pt; a highlight painted \
             through the wrong one would look almost correct, which is worse"
        );
    }

    /// A degenerate page maps everything to the origin rather than to NaN —
    /// `viewer`'s "fail to a finite, harmless value" discipline, inherited
    /// rather than re-implemented.
    #[test]
    fn a_degenerate_page_maps_to_a_finite_point() {
        let m = PageMapping::new(
            Rect::from_min_size(Pos2::ZERO, egui::vec2(10.0, 10.0)),
            (0.0, 100.0),
            1.0,
        );
        assert_eq!(m.to_page(Pos2::new(5.0, 5.0)), Pos2::ZERO);
        assert_eq!(m.to_screen(Pos2::new(5.0, 5.0)), Pos2::ZERO);
    }
}
