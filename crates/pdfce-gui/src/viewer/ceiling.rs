//! # `viewer::ceiling` — how far this page can actually be zoomed
//!
//! `OPERATOR_REQUESTS.md` **O24**. Three limits bind at three different
//! depths, and keeping them in one file is what stops a caller reconciling
//! them differently from another:
//!
//! | limit | what it is | where it bites |
//! |---|---|---|
//! | [`max_zoom_for_page`] | the whole-page raster exceeds `MAX_PIXMAP_EDGE` | ~1,000 % on a large sheet |
//! | [`SUB_PIXEL_CONTENT_EXTENT`] | the `f32` scroll offset can no longer place the view to the pixel | ~1,000,000 % |
//! | the operator's own setting | whatever he asked for | wherever he says |
//!
//! ★★ The first is not a limit at all once the region tier can render past
//! it — the raster becomes window-sized and the page's size stops entering
//! the arithmetic. The second is, and is the one that decides what the shell
//! can honestly offer today; `viewer::deep::DeepAnchor` is what raises it,
//! and it is built, unit-tested and not yet wired.
//!
//! ## Why this is its own file
//!
//! R2's 1,500-line ceiling forced the split when the positional cap landed,
//! and the seam is real: everything here answers one question — *how far may
//! this page be magnified?* — where the rest of [`super`] answers *where is
//! the view and what is it showing?*

// ★ `max_zoom_for_page` stays in [`super`] with the rest of the raster-side
// arithmetic and its own tests, and is imported rather than moved: it answers
// a question about a PIXMAP, where everything in this file answers one about
// how far the operator may go. Moving it would have dragged its tests across a
// seam they do not belong on.
use super::{MAX_ZOOM, MIN_ZOOM, max_zoom_for_page};
/// The largest strip extent — `page × zoom`, in points — at which a page is
/// still drawn.
///
/// `2^35`, chosen inside a measurement rather than derived: driving to the top
/// of the setting on US Letter drew at a strip extent of 4.1×10^10 points and
/// did not at 6.1×10^11. This sits between them, nearer the confirmed end.
///
/// ★ It is what remains after tier 3. The `f64` anchor took the POSITION off
/// the `f32` scroll offset and moved the usable ceiling from 2.1 million
/// percent to 6.7 billion; the strip's own size is still `f32`, and every
/// page's placement is measured inside it.
pub(super) const MAX_STRIP_EXTENT: f32 = 34_359_738_368.0;

/// The largest content extent at which an `f32` scroll offset still positions
/// the view to within one screen pixel — `2^24`, the last integer `f32`
/// represents exactly.
///
/// One unit of content space is one screen pixel, so the spacing between
/// representable offsets **is** the positioning error. Past this the view
/// judders; well past it, it stops being drawn at all.
///
/// ★ Measured rather than assumed: driving to the top of the setting on a US
/// Letter page drew at a content extent of 20.5 billion — a 2,048 px step —
/// and stopped at 41 billion. Drawing is therefore NOT the limit that matters;
/// usability gives out four orders of magnitude earlier, and this is that point.
pub(super) const SUB_PIXEL_CONTENT_EXTENT: f32 = 16_777_216.0;

/// The highest zoom this page can reach **when the region tier is
/// available** — `OPERATOR_REQUESTS.md` O24.
///
/// # ★★ Why this is a different function rather than a flag on the old one
///
/// [`max_zoom_for_page`] answers a question about a **pixmap**: how far can
/// this page be magnified before its whole-page raster exceeds
/// `MAX_PIXMAP_EDGE`? That question is real and its answer is a genuine
/// ceiling — *for the whole-page tier*.
///
/// It is simply **not the question** once the renderer can be asked for a
/// region. There the pixmap is the size of the window, so the page's own
/// size stops entering the arithmetic at all and the only remaining limit is
/// whatever the operator has said they want. Two different questions with
/// two different answers are two functions; adding a boolean to the first
/// would have produced one function whose name describes only half of what
/// it does.
///
/// # ★ It is dormant, and deliberately so
///
/// Nothing calls this yet. It lands ahead of the canvas change that will,
/// so that the arithmetic can be reviewed and tested while it cannot affect
/// a running build — the same staging the render worker's `region` field
/// took.
///
/// `limit` is the operator's own maximum, which becomes a setting. Clamped
/// to at least [`MIN_ZOOM`] so a nonsensical stored value cannot make the
/// document unzoomable.
#[must_use]
pub fn max_zoom_with_regions(limit: f32) -> f32 {
    if limit.is_finite() && limit >= MIN_ZOOM {
        limit
    } else {
        MIN_ZOOM
    }
}

/// **The zoom ceiling in force**, given the operator's configured maximum.
///
/// ★★ The ONE place the two tiers are reconciled, so the two call sites that
/// need a ceiling — `app::actions::apply` and `canvas::zoom` — cannot answer
/// the question differently. Their own comments already note that each derives
/// this per action rather than caching it; deriving it *differently* is the
/// failure that would follow.
///
/// The rule is one sentence: **the whole-page raster limit binds only while the
/// operator has not asked to go past it.** Below their maximum the pixmap
/// ceiling is real and is what stops them; above it, the region tier takes over
/// and the page's size stops entering the arithmetic at all.
///
/// `limit_percent` is [`crate::app::prefs::Prefs::max_zoom_percent`]. Passing
/// the shipped default reproduces the old behaviour exactly, which is what
/// keeps a fresh install unchanged.
#[must_use]
pub fn zoom_ceiling(page_pts: (f32, f32), pixels_per_point: f32, limit_percent: f32) -> f32 {
    let limit = max_zoom_with_regions(limit_percent / 100.0);
    let whole_page = max_zoom_for_page(page_pts, pixels_per_point);
    let longest = page_pts.0.max(page_pts.1);

    // ★★★ THE DEFAULT MUST CHANGE NOTHING, and a plain `max` breaks that.
    //
    // `max_zoom_for_page` can fall BELOW `MAX_ZOOM` on a large page at a high
    // display scale — an A1 sheet at 1.5x tops out at 690 %, not 800 % —
    // because the pixmap ceiling bites first. A plain `limit.max(whole_page)`
    // would then raise that page's ceiling to the operator's default of 800 %
    // and rasterize a pixmap the engine refuses.
    //
    // Caught by `the_default_setting_reproduces_the_old_ceiling_exactly`, which
    // is exactly why that test walks three page sizes and three display scales
    // rather than one of each.
    //
    // So the region tier is only allowed to lift the ceiling when the operator
    // has asked for MORE THAN THE SHIPPED DEFAULT. Below that, nothing about
    // the old behaviour is touched — which is the property that makes this
    // feature safe to land.
    // ★★★ The lift is bounded BELOW by `MAX_ZOOM`, not by the default.
    //
    // The first version compared against `DEFAULT_MAX_ZOOM_PERCENT`, which
    // worked while the default was 800 % and became a no-op the moment the
    // operator raised the default to the maximum — the ceiling would then have
    // been the whole-page limit always, and the setting inert everywhere. A
    // guard phrased in terms of a value that can move is a guard that stops
    // guarding when it moves.
    //
    // `MAX_ZOOM` is the right bound because it is what the SHELL offered before
    // any of this: below it, `max_zoom_for_page`'s pixmap ceiling is a real
    // constraint and must keep binding — an A1 sheet at a 1.5x display tops out
    // at 690 %, and lifting that would ask the engine for a raster it refuses.
    // Above it, the region tier can render and the page's size stops mattering.
    // ★★★ THE POSITIONAL CAP IS GONE, because tier 3 replaced it — O24.
    //
    // It stood here from 2026-08-22 morning until the `f64` anchor was wired,
    // and its reasoning is worth keeping because it is what made the cap
    // correct AT THE TIME: the scroll offset is an `f32` over a content space
    // where one unit is one screen pixel, so past 2^24 content points the view
    // moved in 2-pixel steps, then 512, then 2,048, and above 4×10^10 stopped
    // being drawn at all. Rendering succeeded to a trillion percent throughout —
    // **rendering and working parted company four orders of magnitude apart** —
    // so capping where the picture still appeared would have shipped a range
    // whose top half pans in thousand-pixel jumps.
    //
    // `viewer::deep_position_needed` is now the same constant's other use: it
    // hands the position to `viewer::deep::DeepAnchor` at exactly the point the
    // cap used to refuse. One number, two uses, and they cannot drift.

    // ★★★ AND CAPPED BY THE STRIP'S OWN EXTENT, which is the limit that
    // remains after tier 3 — measured 2026-08-22 by driving to the top.
    //
    // Tier 3 took the POSITION off the `f32` scroll offset, and that moved the
    // usable ceiling from 2.1 million percent to **6.7 billion** on US Letter.
    // What still lives in `f32` is the strip's own size, `page × zoom`, which
    // every page's placement is measured inside. Drawn at a strip extent of
    // 4.1×10^10 points; not drawn at 6.1×10^11.
    //
    // ★ So the cap is a **strip extent**, not a zoom — the same shape as the
    // sub-pixel rule, and page-aware for the same reason: an A0 sheet reaches
    // any extent at a third the zoom of a business card, and a single zoom
    // number would be wrong for one of them.
    //
    // ★★ Removing this means not building the strip in `page × zoom` space at
    // all above the threshold — the move tier 3 made for the offset, one layer
    // out. Until then, offering a zoom that renders cleanly and shows a blank
    // page would be the defect this feature has refused throughout.
    let strip_limited = if longest > 0.0 && longest.is_finite() {
        MAX_STRIP_EXTENT / longest
    } else {
        MAX_ZOOM
    };
    limit.max(whole_page.min(MAX_ZOOM)).min(strip_limited)
}

/// Whether this page at this zoom needs the `f64` position model — O24 tier 3.
///
/// True exactly where an `f32` scroll offset stops placing the view to within a
/// screen pixel, which is [`SUB_PIXEL_CONTENT_EXTENT`]. Below it the scroll
/// area is authoritative and nothing about the canvas changes; above it
/// [`super::deep::DeepAnchor`] is.
///
/// ★ The SAME constant that used to cap the zoom. Before tier 3 was wired the
/// only honest response to passing it was to refuse to go further; now it is
/// the point at which the position model changes hands. One number, two uses,
/// and they cannot drift apart.
#[must_use]
pub fn deep_position_needed(page_pts: (f32, f32), zoom: f32) -> bool {
    let longest = page_pts.0.max(page_pts.1);
    longest.is_finite()
        && zoom.is_finite()
        && longest > 0.0
        && zoom > 0.0
        && longest * zoom > SUB_PIXEL_CONTENT_EXTENT
}
