//! # `canvas::overlay` — what the selection looks like, and what it must never look like
//!
//! ## ★ Rule 4 is the whole design constraint of this file
//!
//! `D:\Dev\FeatureRequests\pdfce_FeatureRequests\README.md`, second and fourth
//! clauses of the disclosure rule:
//!
//! > **Applied content renders exactly as saved content will render.** No
//! > badge, red flag, dashed outline or "provisional" layer drawn into the
//! > page view. […] **A pre-commit affordance is not content marking.** A
//! > snap indicator, a hover highlight, a rubber-band, a selection handle —
//! > these are the *cursor*; they describe what is about to happen and they
//! > are welcome.
//!
//! Everything this module paints is in the second category and nothing is in
//! the first. Outlines, grips, rubber-bands and a move ghost all describe
//! *what the operator is about to act on*, and all of them disappear the
//! instant the selection does. Nothing here is keyed on a property of the
//! **content** — not "this text was OCRed", not "this bound is approximate",
//! not "this font was substituted". Those are inferences, they owe an
//! off-canvas report, and `panels`' own header records where the old shell's
//! dashed-outline version of one of them used to live and where its
//! replacement now is (a sentence in the Properties panel).
//!
//! The one-line test, from the same source: *would a screenshot of the
//! editing canvas differ from a screenshot of the same document saved and
//! reopened?* With nothing selected, this module paints **nothing at all**,
//! so the answer is no by construction.
//!
//! ## Colours come from the theme, never from a literal
//!
//! Every colour here is read from [`egui::Visuals`] — `selection.stroke` for
//! the outline and grips, `selection.bg_fill` for the rubber-band's wash.
//! A hard-coded colour would be correct in one theme and invisible or
//! shouting in the other, and `panels`' scroll-bar note records that exact
//! failure already measured once in this project: a control that was present,
//! opaque, correctly sized and invisible in a capture.
//!
//! ## Why the outline is grown before it is drawn
//!
//! [`visible_outline_rect`], salvaged with its reasoning. A horizontal rule
//! has a real, finite page bbox that is **exactly zero high**; it hit-tests,
//! selects and lists correctly, and its outline puts nothing on the screen.
//! The operator's click was right, the selection state was right, and the
//! feedback was a blank page — a correct action with no feedback is
//! indistinguishable from a broken one.

use egui::{Color32, CornerRadius, Painter, Rect, Stroke, StrokeKind, Visuals};

use crate::canvas::handles;
use crate::canvas::mapping::PageMapping;
use crate::canvas::selection::SelectionState;

/// The minimum on-screen extent, in egui logical points, that a selection
/// outline is guaranteed to have on each axis.
///
/// Sized to be unmistakably visible without materially misreporting where the
/// object is: at 6 pt a horizontal rule's outline reads as a thin band centred
/// on the rule. The Properties panel states the object's true size, so the
/// enlargement can never be mistaken for the object's real extent — which is
/// what keeps this a legibility fix rather than a silent widening (rule 4 is
/// satisfied by disclosure, not by declining to draw).
pub const MIN_OUTLINE_EXTENT_PX: f32 = 6.0;

/// Grow a degenerate outline rect, about its own centre, until it is at least
/// `min_extent` on both axes — **the fix for a selection that is correct and
/// paints nothing.**
///
/// # The bug this closes
///
/// A horizontal rule (`100 200 m 300 200 l S`) has the page bbox
/// `100,200 → 300,200`: real, finite, and exactly zero high. `rect_stroke`
/// with `StrokeKind::Inside` then has no interior band to fill and puts
/// nothing on the screen.
///
/// # Why in SCREEN space, and why symmetric
///
/// Applied after the canvas→screen projection, so the guaranteed thickness is
/// a constant number of on-screen points at every zoom — the same
/// zoom-invariance discipline
/// [`crate::canvas::mapping::screen_tolerance_to_page`] applies to the catch
/// radius. Growing symmetrically about the centre keeps the band straddling
/// the rule rather than sitting to one side of it, so the outline still says
/// truthfully *the object is here*.
///
/// A non-finite rect is returned unchanged: there is no meaningful centre to
/// grow about, and a NaN box is a bug to leave visible upstream rather than
/// repair here.
#[must_use]
pub fn visible_outline_rect(rect: Rect, min_extent: f32) -> Rect {
    if !rect.min.x.is_finite()
        || !rect.min.y.is_finite()
        || !rect.max.x.is_finite()
        || !rect.max.y.is_finite()
        || !min_extent.is_finite()
        || min_extent <= 0.0
    {
        return rect;
    }
    // Normalise: the canvas→screen projection is handed rects the provider
    // built by bounding a mapped quad, so `min` is not guaranteed to be the
    // smaller corner by the time it arrives.
    let rect = Rect::from_two_pos(rect.min, rect.max);
    let grow = |lo: f32, hi: f32| -> (f32, f32) {
        let extent = hi - lo;
        if extent >= min_extent {
            return (lo, hi);
        }
        let pad = (min_extent - extent) / 2.0;
        (lo - pad, hi + pad)
    };
    let (x0, x1) = grow(rect.min.x, rect.max.x);
    let (y0, y1) = grow(rect.min.y, rect.max.y);
    Rect::from_min_max(egui::pos2(x0, y0), egui::pos2(x1, y1))
}

/// The screen-space box the grips are laid out on, or `None` when nothing is
/// selected.
///
/// Shared by the painter and the hit test so the drawn grips and the live
/// grips are the same squares. Two derivations of one box is how an operator
/// ends up aiming at a handle and getting a marquee.
#[must_use]
pub fn grip_box(mapping: &PageMapping, selection: &SelectionState) -> Option<Rect> {
    let union = selection.outline_union()?;
    Some(visible_outline_rect(
        mapping.rect_to_screen(union),
        MIN_OUTLINE_EXTENT_PX,
    ))
}

/// The move ghost's alpha, out of 255.
///
/// High enough to read as a *second* outline over dense linework — the whole
/// point is that the operator can see where the object is going — and low
/// enough that it never competes with the real outline, which is still on
/// screen showing where the object still is. Both boxes are visible during a
/// drag on purpose: the pair states the displacement, which one box alone
/// cannot.
const GHOST_ALPHA: u8 = 150;

/// Paint the selection: one outline per entry, plus the grips.
///
/// # The move ghost lives next door, and why it came back
///
/// A first draft of this function drew a translucent copy of the outline
/// offset by an in-flight move drag, and it was removed before it shipped:
/// `pdfce-core` has no resize verb, the move drag was not wired to one either,
/// and *"a pre-commit affordance that describes something which does not
/// happen is not an affordance, it is a lie with a low alpha. It returns in
/// the same change as the verb."*
///
/// This is that change. [`draw_move_ghost`] is the ghost, and the condition
/// under which it may be drawn is exactly the one that note demanded: only
/// when [`crate::canvas::moving::eligible`] has already established that the
/// release will reach a real verb on real operands. **Resize is still not
/// wired**, and the grips still commit nothing — there is no scale verb — so
/// no ghost is offered for a grip drag either.
pub fn draw_selection(
    painter: &Painter,
    visuals: &Visuals,
    mapping: &PageMapping,
    selection: &SelectionState,
) {
    if selection.is_empty() {
        return;
    }
    let stroke = Stroke::new(1.5, visuals.selection.stroke.color);

    for (_, page_rect) in selection.outlines() {
        let screen =
            visible_outline_rect(mapping.rect_to_screen(*page_rect), MIN_OUTLINE_EXTENT_PX);
        painter.rect_stroke(screen, CornerRadius::ZERO, stroke, StrokeKind::Middle);
    }

    if let Some(box_) = grip_box(mapping, selection) {
        draw_grips(painter, visuals, box_);
    }
}

/// Paint the eight resize grips around a screen-space box.
///
/// Filled with the theme's window background and stroked in the selection
/// colour: a filled square reads as a handle at any zoom and against any page
/// content, where an outline-only square disappears over dense linework —
/// which is precisely the document class pdfce is for.
pub fn draw_grips(painter: &Painter, visuals: &Visuals, bounds: Rect) {
    let stroke = Stroke::new(1.0, visuals.selection.stroke.color);
    for (_, rect) in handles::grip_rects(bounds) {
        painter.rect(
            rect,
            CornerRadius::ZERO,
            visuals.window_fill,
            stroke,
            StrokeKind::Middle,
        );
    }
}

/// Paint the **move ghost**: the selection's outlines, displaced by an
/// in-flight drag.
///
/// `delta` is in **canvas space** — the same space the cached outlines are in,
/// which is what makes this a translation and nothing more.
///
/// # ★ Why this costs no re-raster and no re-decomposition
///
/// Three facts line up, and the preview is affordable because of all three:
///
/// 1. **The outlines are cached in canvas space.**
///    [`SelectionState`] keys them on
///    `(page, edit epoch)`, neither of which moves during a drag, so no
///    decomposition happens on any frame of the gesture.
/// 2. **Canvas space is zoom-independent**, so translating a cached rect by a
///    canvas-space delta and projecting the result is exact at every
///    magnification — there is no per-frame re-derivation of geometry.
/// 3. **Nothing touches the page texture.** A ghost is two strokes on the
///    painter that is already open. The raster is invalidated by
///    `Action::Move*` on *commit*, once, in `app::actions` — not by the
///    preview, which is the whole reason the preview is a preview.
///
/// A ghost that re-rendered the page per frame would be a different feature
/// wearing the same name: on the CAD sheets pdfce exists for, one raster is
/// tens of milliseconds and a drag is sixty frames a second.
///
/// # Rule 4
///
/// This is a *pre-commit affordance* — the cursor describing what is about to
/// happen — and rule 4 admits those explicitly, alongside the snap indicator,
/// the hover highlight and the rubber-band. What rule 4 forbids is marking
/// content that has **already been applied**, and nothing here survives the
/// release: the ghost exists only while the pointer is down. The one-line test
/// in this module's header still answers no — with nothing being dragged, this
/// paints nothing at all.
pub fn draw_move_ghost(
    painter: &Painter,
    visuals: &Visuals,
    mapping: &PageMapping,
    selection: &SelectionState,
    delta: egui::Vec2,
) {
    let stroke = Stroke::new(1.5, ghost(visuals.selection.stroke.color));
    for (_, page_rect) in selection.outlines() {
        let screen = visible_outline_rect(
            mapping.rect_to_screen(page_rect.translate(delta)),
            MIN_OUTLINE_EXTENT_PX,
        );
        painter.rect_stroke(screen, CornerRadius::ZERO, stroke, StrokeKind::Middle);
    }
}

/// Paint the rubber-band, given its **canvas-space** rect.
///
/// A wash plus an outline. The wash matters on a dense drawing: an
/// outline-only band over a hatched region is hard to see at all, and a
/// rubber-band the operator cannot see is a rubber-band they cannot aim.
pub fn draw_marquee(painter: &Painter, visuals: &Visuals, mapping: &PageMapping, page_rect: Rect) {
    let screen = mapping.rect_to_screen(page_rect);
    painter.rect_filled(screen, CornerRadius::ZERO, wash(visuals.selection.bg_fill));
    painter.rect_stroke(
        screen,
        CornerRadius::ZERO,
        Stroke::new(1.0, visuals.selection.stroke.color),
        StrokeKind::Middle,
    );
}

/// The rubber-band's fill: the theme's selection colour at low alpha.
///
/// Derived from the theme rather than named, so it tracks light and dark
/// without a second literal — and low enough that the content under the band
/// stays readable, because the operator is choosing what to enclose *by
/// looking at it*.
fn wash(base: Color32) -> Color32 {
    Color32::from_rgba_unmultiplied(base.r(), base.g(), base.b(), 48)
}

/// The ghost outline's colour: the theme's selection stroke at [`GHOST_ALPHA`].
///
/// Read back through `to_srgba_unmultiplied` rather than through `.r()`/`.g()`
/// /`.b()`. [`Color32`] stores **premultiplied** components, so the plain
/// accessors return a hue already darkened by whatever alpha the source
/// carried; re-premultiplying that at a new alpha darkens it a second time.
/// The selection stroke is opaque in both shipped themes, so the two spellings
/// agree today — which is exactly why the wrong one would go unnoticed until a
/// theme with a translucent selection stroke made the ghost a different colour
/// from the outline it is a copy of.
fn ghost(base: Color32) -> Color32 {
    let [r, g, b, _] = base.to_srgba_unmultiplied();
    Color32::from_rgba_unmultiplied(r, g, b, GHOST_ALPHA)
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Pos2, pos2};

    /// ★ A zero-height rule gets a visible band rather than nothing.
    #[test]
    fn a_degenerate_outline_is_grown_until_it_can_be_seen() {
        // The measured case: `100 200 m 300 200 l S`, projected to screen.
        let rule = Rect::from_min_max(pos2(100.0, 200.0), pos2(300.0, 200.0));
        let out = visible_outline_rect(rule, MIN_OUTLINE_EXTENT_PX);
        assert!(out.height() >= MIN_OUTLINE_EXTENT_PX);
        assert!(
            (out.width() - 200.0).abs() < f32::EPSILON,
            "the axis that was already visible must not be touched"
        );
        assert!(
            (out.center().y - 200.0).abs() < f32::EPSILON,
            "the band must straddle the rule, not sit to one side of it"
        );
    }

    /// A comfortable rect is returned unchanged — the growth is a repair, not
    /// a permanent inflation that would misreport every object's extent.
    #[test]
    fn a_healthy_outline_is_left_alone() {
        let r = Rect::from_min_max(pos2(0.0, 0.0), pos2(100.0, 80.0));
        assert_eq!(visible_outline_rect(r, MIN_OUTLINE_EXTENT_PX), r);
    }

    /// An inside-out rect normalises before it is grown, so a projection that
    /// swapped the corners still paints.
    #[test]
    fn an_inside_out_rect_normalises_before_growing() {
        let backwards = Rect::from_min_max(pos2(300.0, 240.0), pos2(100.0, 200.0));
        let out = visible_outline_rect(backwards, MIN_OUTLINE_EXTENT_PX);
        assert!(out.width() > 0.0 && out.height() > 0.0);
        assert!(out.contains(pos2(200.0, 220.0)));
    }

    /// A non-finite rect is left exactly as it arrived: there is no
    /// meaningful centre to grow about, and repairing it here would hide a
    /// bug that belongs upstream.
    #[test]
    fn a_non_finite_rect_is_returned_unchanged() {
        let nan = Rect::from_min_max(pos2(f32::NAN, 0.0), pos2(10.0, 10.0));
        let out = visible_outline_rect(nan, MIN_OUTLINE_EXTENT_PX);
        assert!(out.min.x.is_nan());
        // And a nonsense minimum is refused rather than shrinking the rect.
        let r = Rect::from_min_max(Pos2::ZERO, pos2(10.0, 10.0));
        assert_eq!(visible_outline_rect(r, -1.0), r);
        assert_eq!(visible_outline_rect(r, f32::NAN), r);
    }

    /// The wash keeps its hue and drops its alpha, so the content under a
    /// rubber-band stays readable.
    ///
    /// Asserted through `to_srgba_unmultiplied` rather than through `.r()`,
    /// and **approximately**. Both halves of that are the point:
    ///
    /// - [`Color32`] stores **premultiplied** components, so a translucent
    ///   blue reads back as `(11, 23, 38)` from the plain accessors and looks
    ///   as though the hue was lost. It was not, and "fixing" that by dropping
    ///   the alpha would be the wrong repair.
    /// - Premultiplying at alpha 48 and dividing back out is lossy — 60
    ///   returns as 58 — so exact equality would be asserting the precision of
    ///   egui's colour storage rather than the property this function has.
    #[test]
    fn the_marquee_wash_is_translucent_and_keeps_the_themes_hue() {
        let base = Color32::from_rgb(60, 120, 200);
        let [r, g, b, a] = wash(base).to_srgba_unmultiplied();
        for (got, want) in [(r, 60u8), (g, 120), (b, 200)] {
            assert!(
                got.abs_diff(want) <= 4,
                "the wash drifted off the theme's hue: {got} vs {want}"
            );
        }
        assert!(a < 64, "a rubber-band must not hide what it encloses");
    }

    /// The ghost keeps the theme's hue and is translucent — visibly a *copy*
    /// of the outline rather than a second, competing selection.
    ///
    /// Asserted through `to_srgba_unmultiplied` for the reason [`ghost`]'s own
    /// docs give, and approximately because premultiplying and dividing back
    /// out is lossy.
    #[test]
    fn the_move_ghost_is_translucent_and_keeps_the_themes_hue() {
        let base = Color32::from_rgb(60, 120, 200);
        let [r, g, b, a] = ghost(base).to_srgba_unmultiplied();
        for (got, want) in [(r, 60u8), (g, 120), (b, 200)] {
            assert!(
                got.abs_diff(want) <= 4,
                "the ghost drifted: {got} vs {want}"
            );
        }
        assert_eq!(a, GHOST_ALPHA);
        assert!(
            a > 64,
            "the ghost must be readable over dense linework, unlike the marquee wash"
        );
    }

    /// A translucent source colour does not get darkened twice — the failure
    /// the accessor choice in [`ghost`] guards against.
    #[test]
    fn a_translucent_theme_colour_keeps_its_hue_through_the_ghost() {
        let translucent = Color32::from_rgba_unmultiplied(60, 120, 200, 90);
        let [r, g, b, _] = ghost(translucent).to_srgba_unmultiplied();
        for (got, want) in [(r, 60u8), (g, 120), (b, 200)] {
            assert!(
                got.abs_diff(want) <= 6,
                "premultiplied components were re-premultiplied: {got} vs {want}"
            );
        }
    }
}
