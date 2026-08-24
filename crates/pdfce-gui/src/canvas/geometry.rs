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
        (last - pan).clamp(0.0, (content_extent(d, v) - v).max(0.0))
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

/// The pasteboard, as a multiple of the viewport. O23: half a viewport puts
/// a page corner at the screen's centre, a whole one puts it at the opposite
/// corner, and the operator asked for the second.
const PASTEBOARD_FRACTION: f32 = 1.0;

/// The pasteboard on one axis, in logical points. Zero for a degenerate
/// viewport so a frame measured before layout cannot produce a NaN extent.
#[must_use]
fn pasteboard(viewport: f32) -> f32 {
    if viewport.is_finite() && viewport > 0.0 {
        viewport * PASTEBOARD_FRACTION
    } else {
        0.0
    }
}

/// The **scroll content's** extent: the strip plus a pasteboard each side,
/// never smaller than the viewport. This is what `display.max(viewport)` used
/// to be at every call site, back when the strip and the content were the
/// same rectangle.
#[must_use]
pub fn content_extent(display: f32, viewport: f32) -> f32 {
    let out = display.max(viewport) + 2.0 * pasteboard(viewport);
    if out.is_finite() {
        out
    } else {
        display.max(viewport)
    }
}

/// How far the **strip's** origin sits from the **content's**: the centring
/// margin plus the pasteboard.
///
/// ★★ Not the same function as [`margin`], and must not become it. Two
/// offset spaces exist and only one is padded — the scroll offset egui is
/// given is measured from the content's origin, the page-local offset the
/// view stores is measured from the page's. [`strip_offset`] and
/// [`page_local_offset`] therefore call **one of each**; using the same
/// margin for both makes the pad cancel and a stored offset of zero scrolls
/// to blank paper.
///
/// `anchor_screen_pos` and `offset_holding_anchor_at` look like scroll-space
/// functions and are **page-local** — `canvas::mod` converts before building
/// the `CanvasFrame` — so they keep [`margin`]. Padding them doubles the pad.
///
/// ★ `pub` since O26g, because `canvas::show` must place the strip from it
/// **symbolically** rather than by subtracting two large rectangles. See
/// [`strip_origin_offset`].
#[must_use]
pub fn strip_margin(display: f32, viewport: f32) -> f32 {
    margin(display, viewport) + pasteboard(viewport)
}

/// **How far the strip's top-left sits from the scroll content's, on one
/// axis** — the number `canvas::show` adds to `outer_rect.min` to place the
/// strip.
///
/// # ★★★ Why this is not `(outer − display) / 2`
///
/// It *is* that, algebraically. Evaluated that way in `f32` it is a
/// catastrophic cancellation, and at deep zoom in a continuous mode it is the
/// **dominant source of error in the whole canvas**.
///
/// The strip's height is `pages × page_height × zoom`. On the operator's
/// 36-page drawing set at 1,045,114 % that is 4.6 × 10⁸ logical points, where
/// an `f32`'s representable step is **32 points**. `Rect::from_center_size`
/// forms `content_centre − strip/2` — two numbers near 2.3 × 10⁸ whose
/// difference is about 619 — so the strip's origin, and therefore every page
/// rect derived from it, and therefore the zoom anchor's `frac`, the raster
/// region and the pointer mapping, were all quantised to 32 points.
///
/// ★★ Measured, and the arithmetic predicts the measurement: an anchored zoom
/// notch slid the view 10 points at 292,415 % (strip 1.3 × 10⁸, step 8) and
/// 16 points at 1,045,114 % (strip 4.6 × 10⁸, step 32). That is why zooming
/// deep in a *multi-page* document creeps while the same zoom on a single page
/// does not: `viewer::deep_position_needed` measures the **page's** magnitude,
/// and it is the **strip's** that overflows `f32`'s exact range — earlier by
/// exactly the page count.
///
/// # The symbolic form
///
/// `outer` is `content_extent(display, viewport).max(avail)` per axis, so
///
/// * when the content wins — every case that matters, because a strip taller
///   than the window is what "scroll" means — the difference is
///   [`strip_margin`]: a centring margin that is **exactly zero** once the
///   display exceeds the viewport, plus a pasteboard that is one viewport.
///   Both are small, both are exact, and no large intermediate is formed at
///   all;
/// * when `avail` wins — a document smaller than the window, where every
///   magnitude is a few hundred points — the plain expression is used, and its
///   precision is not in question there.
///
/// The two agree to the last bit wherever the first branch is taken, which
/// [`tests::the_strip_origin_is_the_plain_expression_wherever_that_expression_is_exact`]
/// asserts.
#[must_use]
pub fn strip_origin_offset(display: f32, viewport: f32, avail: f32) -> f32 {
    let content = content_extent(display, viewport);
    let out = if content >= avail {
        strip_margin(display, viewport)
    } else {
        (avail - display) / 2.0
    };
    if out.is_finite() { out } else { 0.0 }
}

/// Convert a **scroll offset** back into **strip space** — the inverse of
/// [`strip_to_scroll`], and the one every consumer that thinks in strip
/// coordinates needs.
///
/// ★★★ Its absence was O23's whole failure, through three attempts. The
/// canvas builds its visible-region rect from `last_scroll_offset`, which is a
/// **content-space** offset, and then intersects it with the strip's own
/// layout. Before the pasteboard those were the same space and the omission
/// was invisible. With one, the rect lands a whole pasteboard past the end of
/// the strip, `layout.visible()` returns nothing, and the application draws
/// **no canvas at all** — it says so itself, as
/// `canvas-unavailable reason=nothing-visible`.
///
/// Every symptom chased for three attempts followed from that one line: no
/// pointer input, because there was no canvas to point at; a page rect that
/// looked correct, because it was published before the region went; and
/// `drawn=0`, because nothing was visible to raster.
#[must_use]
pub fn scroll_to_strip(scroll: f32, strip: f32, viewport: f32) -> f32 {
    let out = scroll - strip_margin(strip, viewport);
    if out.is_finite() { out } else { 0.0 }
}

/// Convert a position in **strip space** into the **scroll offset** that puts
/// it at the viewport's top-left, clamped to what can be reached.
#[must_use]
pub fn strip_to_scroll(in_strip: f32, strip: f32, viewport: f32) -> f32 {
    let out = in_strip + strip_margin(strip, viewport);
    if out.is_finite() {
        out.clamp(0.0, (content_extent(strip, viewport) - viewport).max(0.0))
    } else {
        0.0
    }
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

/// **The page-local offset, MEASURED from where the page was actually drawn.**
///
/// # ★★★ Why this exists beside [`page_local_offset`], which computes the same
/// number
///
/// `page_local_offset` *reconstructs* the offset from the scroll area's own
/// offset. That is correct exactly while the scroll offset is where the view
/// is — and above the deep-position threshold it is **not**. There the content
/// is taken down to the viewport, the scroll offset is forced to `(0, 0)` so
/// egui has nothing to round, and the position is held by
/// [`crate::viewer::deep::DeepAnchor`] in `f64`. Reconstructing from a forced
/// zero yields "the page is centred in the pasteboard", which is a statement
/// about a page nobody is looking at.
///
/// ★★★ **That lie was `OPERATOR_REQUESTS.md` O26e.** `CanvasFrame::offset` is
/// the `offset_before` of the next zoom, so every frame spent at deep zoom
/// recorded a fictitious "before". Nothing went wrong while the tier held —
/// the deep branch does not consult it — but the moment a zoom-out crossed
/// back, [`zoom_anchor_offset`] solved against it and put the page's **origin**
/// under the pointer. Driven, 2026-08-24: descending through the boundary at
/// 1,185,799 % moved the page point under the viewport centre from
/// (791.93, 1152.34) to **(−0.02, −0.03)** — the corner of the sheet, with
/// twelve million pixels of drawing off screen. The operator's report was
/// *"zoom out … repositions the page so that it is off screen in the far
/// bottom left corner … from around 2 million %"*.
///
/// # The measurement
///
/// ```text
///     page_top_left_on_screen = viewport_origin + margin(display, viewport) - offset
/// ```
///
/// which is [`anchor_screen_pos`] at `anchor_frac = 0`, rearranged. So the
/// offset is `margin − (page_min − viewport_min)`, and every term is a rect
/// this frame really drew. **It cannot disagree with the pixels, because it is
/// derived from them.**
///
/// ★ It is not an approximation of [`page_local_offset`] and not a second
/// spelling of it: on the shallow tier the two are *algebraically identical*,
/// which [`tests::measuring_the_offset_from_the_drawn_rect_matches_the_solved_one`]
/// asserts against the same inputs rather than trusting this paragraph. What
/// it buys is that the identity survives the tier change, because it never
/// mentions the scroll offset at all.
///
/// # Arguments
///
/// * `page_min` — the current page's top-left **on screen** (`image_rect.min`).
/// * `viewport_min` — the scroll viewport's top-left on screen
///   (`inner_rect.min`), which is where a viewport-relative position is
///   measured from.
/// * `display` — the page's drawn size, the same one the solve is handed.
/// * `viewport` — the viewport's size, the same measurement the margin term is
///   derived against.
///
/// Non-finite inputs yield `(0.0, 0.0)`: a `NaN` here would propagate into the
/// next zoom's `offset_before` and blank the canvas, and "centred" is the only
/// safe fiction when the true answer is unrepresentable.
#[must_use]
pub fn offset_from_drawn(
    page_min: (f32, f32),
    viewport_min: (f32, f32),
    display: (f32, f32),
    viewport: (f32, f32),
) -> (f32, f32) {
    fn axis(page_min: f32, viewport_min: f32, display: f32, viewport: f32) -> f32 {
        let out = margin(display, viewport) - (page_min - viewport_min);
        if out.is_finite() { out } else { 0.0 }
    }
    (
        axis(page_min.0, viewport_min.0, display.0, viewport.0),
        axis(page_min.1, viewport_min.1, display.1, viewport.1),
    )
}

/// **Where a fit command puts the view** — `OPERATOR_REQUESTS.md` O28.
///
/// # The report, and why a fit is now a position as well as a scale
///
/// > *"If I press the Fit width or fit page button the view should center to
/// > the width as well or center the page."*
///
/// Before O23's pasteboard a page no larger than the viewport had nowhere to
/// be except the middle, so *fit* and *centred* were the same act and the
/// button never had to choose. The pasteboard added a whole viewport of slack
/// on every side, and with it the state the operator is describing: the scale
/// is right and the page is not on screen.
///
/// # The rule, per axis
///
/// * **Pinned** — the fit has just decided this axis's extent, so there is one
///   honest position for it and the answer is **zero**. Zero is not an
///   arbitrary choice: [`anchor_screen_pos`] at `frac = 0` places the page's
///   top-left at `margin - offset` from the viewport's, and [`margin`] is
///   *half the slack when the page is smaller than the viewport and exactly
///   zero once it is larger*. So a page-local offset of zero means **centred
///   if it fits, flush if it does not** — which is fit-page's answer on both
///   axes and fit-width's on the horizontal, without a special case for
///   either.
/// * **Unpinned** — the operator is still navigating this axis, so their
///   position is *kept*, clamped to the page's own range `0 ..= display -
///   viewport`. Keeping it is why "Fit width" on page twelve of a drawing set
///   does not throw them back to the top of the sheet; clamping it is what
///   stops "kept" meaning "still looking at pasteboard".
///
/// ★ The clamp collapses to `[0, 0]` whenever the page is no larger than the
/// viewport on that axis — the fit-page case, and the landscape-sheet case —
/// and `0` is centred there, so the two rules agree at the boundary rather
/// than fighting over it.
///
/// `current` is the page-local offset the view is at now. Non-finite input
/// yields the pinned answer, because a `NaN` position is not one worth
/// preserving.
#[must_use]
pub fn fit_placement_offset(
    pinned: (bool, bool),
    current: (f32, f32),
    display: (f32, f32),
    viewport: (f32, f32),
) -> (f32, f32) {
    fn axis(pinned: bool, current: f32, display: f32, viewport: f32) -> f32 {
        if pinned || !current.is_finite() {
            return 0.0;
        }
        current.clamp(0.0, (display - viewport).max(0.0))
    }
    (
        axis(pinned.0, current.0, display.0, viewport.0),
        axis(pinned.1, current.1, display.1, viewport.1),
    )
}

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
        let out = off - origin - strip_margin(strip, v) + margin(page, v);
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
        let out = off + origin + strip_margin(strip, v) - margin(page, v);
        if out.is_finite() {
            out.clamp(0.0, (content_extent(strip, v) - v).max(0.0))
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
    // ★★★ RETURNED UNCLAMPED — `OPERATOR_REQUESTS.md` O24e.
    //
    // This used to clamp to `display_after - viewport`: the range a page has
    // when the scroll content is the page and nothing else. **The pasteboard
    // made that false.** `content_extent` now adds a viewport of slack on
    // every side (O23, so an object off the page is still reachable), so the
    // real range is `content_extent(strip, viewport) - viewport` and the page
    // is only part of it.
    //
    // The damage was worst exactly where the operator found it. At a fit-page
    // zoom the page is no LARGER than the viewport, so `display_after -
    // viewport` is zero or negative, the clamp range collapses to `[0, 0]`,
    // and every zoom forced the offset to zero — which after
    // `strip_offset`'s conversion is the centred position. His report,
    // 2026-08-22:
    //
    // > *"if I am zoomed out to about page size, pan the cells to the center
    // > of the screen, then start to zoom, the page snaps back to near the
    // > center position."*
    //
    // Not "near" by accident: it is the centre, and it is the centre because
    // zero page-local offset means the page sits centred in the pasteboard.
    //
    // ★ The clamp is not gone, it has moved to the one place that can do it
    // correctly. [`strip_offset`] already clamps to
    // `content_extent(strip, v) - v` — the true range, pasteboard included —
    // and it is the value actually handed to the `ScrollArea`. That is the
    // division of labour this module's header states: *the raw solve is
    // unclamped and the offset that reaches the widget is not*. Clamping
    // here as well was a second clamp in the wrong space against the wrong
    // extent, and the two were not equivalent the moment the pasteboard
    // existed.
    offset_holding_anchor_at(anchor_frac, held, display_after, viewport)
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

    /// ★★ **The far edge is now a whole viewport PAST the page**, which is
    /// `OPERATOR_REQUESTS.md` O23 stated as a number.
    ///
    /// This test asserted `200.0` — `display - viewport` — from the day it was
    /// written until 2026-08-21, and it was right to: the clamp stopped at the
    /// page's own edge and the module's header recorded that as a known
    /// limitation waiting on a UX call. The call was made:
    ///
    /// > *"I should also be able to move the view of the corner of the page to
    /// > the center of the screen, or even all the way vertically to the
    /// > opposite corner if I want to."*
    ///
    /// With a one-viewport pasteboard the content is `1000 + 2×800 = 2600`
    /// wide, so the last offset that still shows anything is `2600 − 800 =
    /// 1800`. The pan asks for `700 + 500 = 1200`, which is now inside the
    /// range and is therefore granted in full.
    #[test]
    fn panning_stops_a_whole_viewport_past_the_page_edge() {
        let out = pan_offset(
            (700.0, 0.0),
            (-500.0, 0.0),
            (1000.0, 1000.0),
            (800.0, 800.0),
        );
        assert_eq!(
            out.0, 1200.0,
            "the pasteboard makes this pan reachable; it used to clamp at 200"
        );

        // …and the clamp still exists, one viewport further out.
        let far = pan_offset(
            (1800.0, 0.0),
            (-500.0, 0.0),
            (1000.0, 1000.0),
            (800.0, 800.0),
        );
        assert_eq!(
            far.0,
            content_extent(1000.0, 800.0) - 800.0,
            "there is still an end; it is the end of the PASTEBOARD, not of the page"
        );
    }

    /// ★★★ **O23, asserted as the operator's own two sentences.**
    ///
    /// The pasteboard's size is not a taste; it is whatever makes these two
    /// true. If `PASTEBOARD_FRACTION` is ever reduced, this fails and says
    /// which sentence stopped holding.
    #[test]
    fn any_page_corner_can_be_brought_to_the_centre_and_to_the_opposite_corner() {
        // A page smaller than the window — the hard case, because there is no
        // scrolling to be had from the page's own size.
        let (d, v) = (200.0_f32, 800.0_f32);
        let range = (content_extent(d, v) - v).max(0.0);

        // Where the strip's own origin sits inside the content.
        let origin = strip_margin(d, v);

        // "the corner of the page to the center of the screen": the offset that
        // puts the strip's top-left half a viewport in from the view's left.
        let to_centre = origin - v / 2.0;
        assert!(
            (0.0..=range).contains(&to_centre),
            "a page corner must reach the centre of the screen: {to_centre} not in 0..={range}"
        );

        // "even all the way … to the opposite corner": the strip's top-left
        // pushed to the far edge of the view.
        let to_far_corner = origin - v;
        assert!(
            (0.0..=range).contains(&to_far_corner),
            "a page corner must reach the opposite corner: {to_far_corner} not in 0..={range}"
        );

        // And the mirror: the page's BOTTOM-RIGHT corner brought back to the
        // view's top-left, which is the same freedom in the other direction.
        let bottom_right_to_origin = origin + d;
        assert!(
            (0.0..=range).contains(&bottom_right_to_origin),
            "the far corner must reach the near one: {bottom_right_to_origin} not in 0..={range}"
        );
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

    /// ★★ **The offset handed to the scroll area never leaves its range** —
    /// and after O24e that range is the pasteboard's, not the page's.
    ///
    /// The assertion used to be made against [`zoom_anchor_offset`], which
    /// clamped to `display - viewport`. That is the range a page has when the
    /// scroll content is the page and nothing else, and it stopped being true
    /// when the pasteboard landed — see that function for what it cost.
    #[test]
    fn the_offset_never_leaves_the_scrollable_range() {
        // Zooming OUT far enough that the page no longer fills the viewport.
        // The solve itself is handed back raw…
        let out = zoom_anchor_offset(
            (900.0, 900.0),
            (2000.0, 2000.0),
            (400.0, 400.0),
            (800.0, 800.0),
            (0.1, 0.1),
        );
        // …and whatever it says, the value that reaches the widget is inside
        // the content. Both ends checked, because a negative offset is the
        // failure the original test was written for and it must still be
        // impossible.
        let (v, d) = (800.0_f32, 400.0_f32);
        let range = content_extent(d, v) - v;
        for probe in [out.0, -5000.0, 0.0, 5000.0] {
            let reached = strip_offset((probe, probe), (0.0, 0.0), (d, d), (d, d), (v, v));
            assert!(
                reached.0 >= 0.0 && reached.0 <= range,
                "{probe} reached {reached:?}, outside [0, {range}]"
            );
            assert!(reached.1 >= 0.0 && reached.1 <= range);
        }

        // And never past the far edge, however extreme the anchor fraction.
        // ★ Against the CONTENT's range, not the page's — that substitution is
        // the whole of O24e.
        let (v2, d2) = (800.0_f32, 1000.0_f32);
        let solved =
            zoom_anchor_offset((900.0, 0.0), (500.0, 500.0), (d2, d2), (v2, v2), (5.0, 0.0)).0;
        let reached = strip_offset((solved, 0.0), (0.0, 0.0), (d2, d2), (d2, d2), (v2, v2)).0;
        let range2 = content_extent(d2, v2) - v2;
        assert!(
            reached <= range2 + 0.01,
            "offset {reached} exceeds the maximum scroll {range2}"
        );
    }

    /// ★★★ **The anchor solve is unclamped; the SCROLL OFFSET is clamped** —
    /// and the two are different values in different spaces.
    ///
    /// This test used to assert that [`zoom_anchor_offset`] saturated at
    /// `display - viewport`, the range a page has when the scroll content is
    /// the page and nothing else. The pasteboard (O23) made that false, and
    /// the stale clamp became `OPERATOR_REQUESTS.md` **O24e**: at a fit-page
    /// zoom the page is no larger than the viewport, the range collapsed to
    /// `[0, 0]`, and every zoom threw away whatever the operator had panned to.
    ///
    /// ★ The behaviour the old test was protecting is real and still wanted —
    /// an anchor near an edge must saturate rather than scroll into nothing.
    /// It just belongs to the value that reaches the widget. So the assertion
    /// moved to [`strip_offset`], which clamps against `content_extent`, the
    /// pasteboard included.
    #[test]
    fn the_scroll_offset_saturates_at_the_pasteboard_edge_not_at_the_page_edge() {
        let (v, u) = (800.0_f32, 0.9_f32);
        let (d0, d1) = (600.0_f32, 1000.0_f32);

        // 1. The raw solve is handed back whole, over-range and all.
        let solved = zoom_anchor_offset((0.0, 0.0), (d0, d0), (d1, d1), (v, v), (u, u)).0;
        let unclamped = 0.9 * (d1 - d0) - 100.0; // 260
        assert!(
            unclamped > d1 - v,
            "this case must actually be over-range for the page to test anything"
        );
        assert_eq!(
            solved, unclamped,
            "the anchor solve must not clamp: it does not know the real range"
        );

        // 2. And 260 is REACHABLE, because the pasteboard extends the range
        //    well past the page's own 200. This is the whole point: the old
        //    clamp was discarding positions the operator can legitimately be
        //    at.
        // ★ Compared against the PAGE's range, not against `unclamped`:
        // `strip_offset` also applies the strip↔page-local conversion, so the
        // number it returns is in a different space and is not expected to
        // equal the solve. What matters is that it was not truncated to the
        // page's 200 — the position the operator panned to is still reachable.
        let reached = strip_offset((solved, 0.0), (0.0, 0.0), (d1, d1), (d1, d1), (v, v)).0;
        assert!(
            reached > d1 - v,
            "reached {reached}, which is inside the page's own range of {} — the pasteboard \
             position was discarded",
            d1 - v
        );

        // 3. The saturation itself still happens — at the pasteboard's edge.
        let far = content_extent(d1, v) * 4.0;
        let limit = strip_offset((far, 0.0), (0.0, 0.0), (d1, d1), (d1, d1), (v, v)).0;
        assert_eq!(
            limit,
            content_extent(d1, v) - v,
            "an absurd offset must saturate at the end of the scrollable content"
        );
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
    /// over-range case that used to be clamped here — so a future edit to either
    /// cannot silently change what Ctrl+wheel does.
    #[test]
    fn the_split_solve_is_the_closed_form_it_replaced() {
        fn closed_form(off0: f32, d0: f32, d1: f32, v: f32, u: f32) -> f32 {
            let margin = |d: f32| (d.max(v) - d) / 2.0;
            // ★ No clamp: O24e moved it to `strip_offset`, which is the only
            // caller that knows the pasteboard-extended range. See
            // `zoom_anchor_offset`.
            off0 + u * (d1 - d0) + (margin(d1) - margin(d0))
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
            //
            // ★ UNCHANGED by O23's pasteboard, and that is itself the assertion:
            // this function works in PAGE-LOCAL space, where there is no
            // pasteboard. If a future edit makes this 120, it has padded a
            // page-local function — see `strip_margin`'s note.
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
    fn the_strip_bridge_is_a_pure_pasteboard_shift_for_a_single_page() {
        let v = (800.0_f32, 600.0_f32);
        for &page in &[(400.0_f32, 300.0_f32), (1600.0, 2400.0), (800.0, 600.0)] {
            let pad = (pasteboard(v.0), pasteboard(v.1));
            let range = (
                (content_extent(page.0, v.0) - v.0).max(0.0),
                (content_extent(page.1, v.1) - v.1).max(0.0),
            );
            for &off in &[(0.0_f32, 0.0_f32), (120.0, 55.0), (900.0, 1800.0)] {
                // ★ Going OUT: the page-local offset gains exactly one
                // pasteboard and nothing else. With one page the strip IS the
                // page, so the two centring margins are equal and cancel — the
                // only surviving term is the pad, which is the whole of what
                // O23 added. Anything else here would mean the centring margin
                // had leaked into a space that does not have one.
                let expected = (
                    (off.0 + pad.0).clamp(0.0, range.0),
                    (off.1 + pad.1).clamp(0.0, range.1),
                );
                assert_eq!(
                    strip_offset(off, (0.0, 0.0), page, page, v),
                    expected,
                    "page {page:?} offset {off:?}"
                );

                // …and coming BACK the pad is removed again, so a round trip
                // through both legs is the identity wherever the clamp did not
                // bite. **This is the property that matters** — the pad must
                // not accumulate, or every frame would drift one viewport
                // further into blank paper.
                let back = page_local_offset(expected, (0.0, 0.0), page, page, v);
                if expected.0 > 0.0 && expected.0 < range.0 {
                    assert!(
                        (back.0 - off.0).abs() < 0.001,
                        "x round trip: {off:?} -> {expected:?} -> {back:?}"
                    );
                }
                if expected.1 > 0.0 && expected.1 < range.1 {
                    assert!(
                        (back.1 - off.1).abs() < 0.001,
                        "y round trip: {off:?} -> {expected:?} -> {back:?}"
                    );
                }
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
                            // ★ The strip's origin inside the CONTENT — its
                            // centring margin plus the pasteboard. Spelled out
                            // rather than calling `strip_margin`, because a
                            // test that reuses the function under test agrees
                            // with it by construction, including when wrong.
                            let truth = (
                                (strip.0.max(v.0) - strip.0) / 2.0
                                    + v.0 * PASTEBOARD_FRACTION
                                    + origin.0
                                    + frac.0 * page.0
                                    - off.0,
                                (strip.1.max(v.1) - strip.1) / 2.0
                                    + v.1 * PASTEBOARD_FRACTION
                                    + origin.1
                                    + frac.1 * page.1
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

    /// ★★★ **[`offset_from_drawn`] is [`page_local_offset`], on the shallow
    /// tier — the same number, from the pixels instead of from the offset.**
    ///
    /// This is the claim O26e's fix rests on, and it is the claim that makes
    /// the change safe: `canvas::show` swapped one for the other on **every**
    /// frame, not only deep ones, so if they disagreed anywhere below the
    /// threshold the fix would have traded a rare catastrophe for a constant
    /// small one.
    ///
    /// The shallow tier's geometry is reconstructed here exactly as `show`
    /// builds it — content origin, strip centring margin, pasteboard, the
    /// page's place inside the strip, the scroll offset — and then the page's
    /// screen rect and the viewport's screen rect are handed to
    /// `offset_from_drawn` the way `show` hands it `image_rect.min` and
    /// `inner_rect.min`. Spelled out rather than calling `strip_margin`,
    /// for the reason the sibling test above states: a test that reuses the
    /// function under test agrees with it by construction, including when
    /// both are wrong.
    ///
    /// ★ What this does **not** claim, deliberately: that they agree at the
    /// deep tier. They do not, and that is the whole point — there the scroll
    /// offset is forced to zero and `page_local_offset` describes a page
    /// nobody is looking at, while `offset_from_drawn` describes the one on
    /// screen. There is no assertion to write for "one of these is a lie",
    /// only a driven check: `zooming_back_out_keeps_the_view`.
    #[test]
    fn measuring_the_offset_from_the_drawn_rect_matches_the_solved_one() {
        let v = (800.0_f32, 600.0_f32);
        // Somewhere non-zero, so an implementation that forgot the viewport
        // origin cannot pass by it being (0, 0).
        let viewport_min = (137.0_f32, 91.0_f32);
        for &strip in &[(612.0_f32, 4000.0_f32), (1200.0, 500.0), (300.0, 200.0)] {
            for &page in &[(612.0_f32, 792.0_f32), (300.0, 200.0)] {
                for &origin in &[(0.0_f32, 0.0_f32), (0.0, 1200.0), (294.0, 2400.0)] {
                    for &off in &[(0.0_f32, 0.0_f32), (100.0, 900.0)] {
                        // The content's origin on screen: the viewport's, less
                        // however far the area has been scrolled.
                        let content_min = (viewport_min.0 - off.0, viewport_min.1 - off.1);
                        // The page's top-left on screen: the content's origin,
                        // plus the strip's centring margin and the pasteboard,
                        // plus the page's own place inside the strip.
                        let page_min = (
                            content_min.0
                                + (strip.0.max(v.0) - strip.0) / 2.0
                                + v.0 * PASTEBOARD_FRACTION
                                + origin.0,
                            content_min.1
                                + (strip.1.max(v.1) - strip.1) / 2.0
                                + v.1 * PASTEBOARD_FRACTION
                                + origin.1,
                        );
                        let solved = page_local_offset(off, origin, strip, page, v);
                        let measured = offset_from_drawn(page_min, viewport_min, page, v);
                        assert!(
                            (measured.0 - solved.0).abs() < 1e-2
                                && (measured.1 - solved.1).abs() < 1e-2,
                            "strip={strip:?} page={page:?} origin={origin:?} off={off:?}: measured \n                             {measured:?} vs solved {solved:?}"
                        );
                    }
                }
            }
        }
    }

    /// A non-finite rect cannot poison the next zoom's `offset_before`.
    ///
    /// ★ Zero rather than the previous value, because this function has no
    /// previous value to return — it is a measurement, not a step. "Centred"
    /// is the safe fiction; a `NaN` propagates into `zoom_anchor_offset` and
    /// blanks the canvas, which is the one outcome worse than a wrong offset.
    #[test]
    fn a_non_finite_drawn_rect_measures_as_centred_rather_than_as_nan() {
        let out = offset_from_drawn(
            (f32::NAN, f32::INFINITY),
            (0.0, 0.0),
            (612.0, 792.0),
            (800.0, 600.0),
        );
        assert_eq!(
            out,
            (0.0, 0.0),
            "a non-finite rect must not yield a NaN offset"
        );
    }

    /// [`strip_origin_offset`] is `(outer − display) / 2` wherever that
    /// expression can still be evaluated exactly — which is the claim that
    /// makes replacing one with the other safe.
    ///
    /// ★ The magnitudes here are deliberately ordinary. The whole point of the
    /// symbolic form is that it agrees with the plain one where the plain one
    /// is trustworthy and continues to be right where it is not, and only the
    /// first half of that is assertable in `f32` arithmetic — the second half
    /// is what `zooming_back_out_keeps_the_view` drives.
    #[test]
    fn the_strip_origin_is_the_plain_expression_wherever_that_expression_is_exact() {
        for &vp in &[600.0_f32, 619.0, 1000.0] {
            for &display in &[100.0_f32, 599.0, 600.0, 1200.0, 40_000.0] {
                for &avail in &[400.0_f32, 600.0, 5_000.0] {
                    let outer = content_extent(display, vp).max(avail);
                    let plain = (outer - display) / 2.0;
                    let symbolic = strip_origin_offset(display, vp, avail);
                    assert!(
                        (plain - symbolic).abs() < 1e-3,
                        "display={display} vp={vp} avail={avail}: plain {plain} vs symbolic \n                         {symbolic}"
                    );
                }
            }
        }
    }

    // ---- the fit placement (O28) ---------------------------------------

    /// ★★★ **A pinned axis lands centred when the page fits, and flush when it
    /// does not** — the property the whole of O28 rests on, and the reason the
    /// pinned answer can be the single constant `0.0`.
    ///
    /// Asserted through [`anchor_screen_pos`] rather than by re-stating the
    /// arithmetic: what matters is not that the function returns zero, it is
    /// **where the page's top-left ends up on screen** when it does. A test
    /// that checked for zero would keep passing if [`margin`] were changed
    /// underneath it, which is exactly the coupling this pins.
    #[test]
    fn a_pinned_axis_centres_a_page_that_fits_and_sits_flush_with_one_that_does_not() {
        let viewport = (800.0_f32, 600.0_f32);
        // Smaller than the viewport on both axes: fit-page's own case.
        let small = (500.0_f32, 400.0_f32);
        let placed = fit_placement_offset((true, true), (123.0, 456.0), small, viewport);
        let corner = anchor_screen_pos((0.0, 0.0), placed, small, viewport);
        assert!(
            (corner.0 - (viewport.0 - small.0) / 2.0).abs() < 1e-3
                && (corner.1 - (viewport.1 - small.1) / 2.0).abs() < 1e-3,
            "a page that fits must be centred, not merely at offset zero: {corner:?}"
        );

        // Exactly the viewport's width, which is what fit-width produces.
        let wide = (800.0_f32, 2000.0_f32);
        let placed = fit_placement_offset((true, false), (600.0, 300.0), wide, viewport);
        let corner = anchor_screen_pos((0.0, 0.0), placed, wide, viewport);
        assert!(
            corner.0.abs() < 1e-3,
            "a page exactly as wide as the viewport must sit flush to its left edge, so the full width shows and no pasteboard does: {corner:?}"
        );
    }

    /// An unpinned axis keeps where the operator was — and cannot keep them in
    /// the pasteboard.
    ///
    /// ★ Both halves in one test, because they are one rule. Keeping the
    /// position is what stops "Fit width" throwing the operator back to the
    /// top of a long sheet; clamping it is what stops "kept" meaning "still
    /// looking at nothing".
    #[test]
    fn an_unpinned_axis_is_kept_but_clamped_to_the_page() {
        let viewport = (800.0_f32, 600.0_f32);
        let tall = (800.0_f32, 2000.0_f32);
        assert_eq!(
            fit_placement_offset((true, false), (0.0, 900.0), tall, viewport).1,
            900.0,
            "a position inside the page must survive a fit untouched"
        );
        assert_eq!(
            fit_placement_offset((true, false), (0.0, 5000.0), tall, viewport).1,
            tall.1 - viewport.1,
            "a position out in the pasteboard must be pulled back onto the page"
        );
        assert_eq!(
            fit_placement_offset((true, false), (0.0, -900.0), tall, viewport).1,
            0.0,
            "and so must one above the page's top, which the pasteboard also allows"
        );
    }

    /// A page shorter than the viewport has a clamp range of `[0, 0]`, so the
    /// "kept" rule and the "centred" rule agree rather than fighting.
    ///
    /// This is the landscape-sheet-under-fit-width case, and the one where a
    /// `max(0.0)` on the wrong side would leave the page pinned to the top of
    /// the window with a gap underneath.
    #[test]
    fn an_unpinned_axis_on_a_page_smaller_than_the_viewport_still_centres() {
        let viewport = (800.0_f32, 600.0_f32);
        let short = (800.0_f32, 300.0_f32);
        let placed = fit_placement_offset((true, false), (0.0, 250.0), short, viewport);
        let corner = anchor_screen_pos((0.0, 0.0), placed, short, viewport);
        assert!(
            (corner.1 - (viewport.1 - short.1) / 2.0).abs() < 1e-3,
            "a page shorter than the viewport must be centred on the free axis too: {corner:?}"
        );
    }

    /// A non-finite position cannot survive a fit.
    #[test]
    fn a_non_finite_position_falls_back_to_the_pinned_answer() {
        let out = fit_placement_offset(
            (false, false),
            (f32::NAN, f32::INFINITY),
            (800.0, 2000.0),
            (800.0, 600.0),
        );
        assert_eq!(out, (0.0, 0.0));
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
        // ★ The ceiling is the CONTENT's range, not the strip's — O23. On x this
        // used to be 0.0, because a strip narrower than the viewport had nowhere
        // to scroll; there is now a pasteboard either side of it.
        assert_eq!(
            out,
            (
                content_extent(strip.0, v.0) - v.0,
                content_extent(strip.1, v.1) - v.1
            )
        );
        assert!(out.0 > 0.0, "a narrow page must still be pannable sideways");
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
