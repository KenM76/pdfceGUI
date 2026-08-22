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
    // ★★★ AND CAPPED WHERE THE POSITION STOPS BEING EXPRESSIBLE — measured
    // 2026-08-22 by driving to the top of the setting.
    //
    // The region tier removes the raster limit entirely: rendering succeeded at
    // 999,999,995,904 % with no failures. What does not survive is the SCROLL
    // OFFSET, which egui keeps in `f32` over a content space of `page × zoom`
    // where one unit is one screen pixel:
    //
    //     content extent      f32 step = positioning error
    //         16,777,216            1 px      <- 2^24, the last exact one
    //      4,294,967,296          512 px
    //     20,535,312,384        2,048 px      <- still DREW, at 3.3 billion %
    //     41,070,624,768        4,096 px      <- stopped drawing
    //
    // ★ So "it renders" and "it works" part company long before the raster
    // does. Capping where drawing stops would ship a zoom range whose top half
    // pans in thousand-pixel jumps — a control that accepts a number and then
    // misbehaves, which is the defect this feature has guarded against
    // throughout.
    //
    // The cap is the last extent at which positioning is SUB-PIXEL. On US
    // Letter that is ~2,700,000 %, on an A1 sheet ~1,050,000 % — against the
    // 2,382 % that failed outright before this work.
    //
    // ★★ Raising it is `viewer::deep::DeepAnchor`'s job — the `f64` position
    // model, built and unit-tested and NOT yet wired to the canvas. When it is,
    // this cap is what should be deleted, and nothing else here changes.
    let longest = page_pts.0.max(page_pts.1);
    let positional = if longest > 0.0 && longest.is_finite() {
        SUB_PIXEL_CONTENT_EXTENT / longest
    } else {
        MAX_ZOOM
    };
    limit.max(whole_page.min(MAX_ZOOM)).min(positional)
}
