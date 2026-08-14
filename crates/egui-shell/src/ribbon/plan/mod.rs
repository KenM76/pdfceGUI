//! Ribbon layout arithmetic — the part of the ribbon that has no `egui` in
//! it.
//!
//! # What is planned here, and in which file
//!
//! Two rows, one rule.
//!
//! | Function | File | Plans | Reservation it protects |
//! |---|---|---|---|
//! | [`plan_band`] | this one | the band's groups | the band's "⏷ N more" affordance |
//! | [`plan_strip_row`] | [`row`] | the tab-strip row's three regions | the tab area itself |
//! | [`plan_tab_strip`] | [`row`] | the tabs within that area | the strip's own "⏷ N more" affordance **and the active tab** |
//!
//! Both are re-exported here, so every call site says `plan::…` and the
//! split is an organisational fact rather than something a caller has to
//! know. [`row`]'s header explains why the row is a different problem from
//! the band despite looking like the same one — the short version is that
//! everything a *band* hides is still reachable through its menu, and the
//! one thing a *strip* must never hide is the very tab its menu cannot
//! reach.
//!
//! What the two share is the greedy fill: [`plan_tab_strip`] calls
//! [`plan_band`] to place the tabs it has not pinned, so the monotonicity
//! rule — *widening never hides something that was visible* — exists in
//! one place rather than two.
//!
//! # Why this is a separate module with no `Ui` in its signatures
//!
//! Everything in this file is a pure function over `f32`. That is not
//! tidiness; it is the only way the *overflow* invariant can be tested at
//! all.
//!
//! `MODES_AND_PANELS.md` Part 2 lists twelve failure modes observed in a
//! shipping application, and number eight is the one this module exists
//! to make impossible:
//!
//! > **Tab overflow has no escape** — past ~6 tabs the overflow *button
//! > itself* gets hidden, leaving no route to the hidden tabs. → *The
//! > overflow affordance is reserved space, never the first thing
//! > squeezed out.*
//!
//! That defect is a *layout arithmetic* defect. It happens when the
//! overflow control is emitted **after** the content, into whatever space
//! the content did not take — which is the obvious immediate-mode
//! spelling, and which yields nothing at all once the content takes
//! everything. Reading such code does not reveal the bug; the code says
//! "draw the groups, then draw the overflow button", and that sentence
//! sounds correct.
//!
//! So the arithmetic is lifted out, and the reservation is made the
//! **first** subtraction rather than the last:
//!
//! ```text
//! budget_for_groups = available − overflow_width − separator
//! ```
//!
//! computed before a single group is measured against it. A group can
//! then only ever consume `budget_for_groups`, and the overflow control's
//! width is not in that number. There is no ordering of the group loop
//! that can reach it.
//!
//! `plan_band` returns that budget, and [`super::band`] hands the
//! groups a `Ui` whose maximum width **is** that budget — so the
//! reservation is enforced twice: once by this arithmetic, and once by
//! `egui`'s own clipping, which cannot be talked out of it.
//!
//! # Why widths are estimated rather than measured
//!
//! Immediate mode has a genuine ordering problem: the width a group will
//! occupy is known only after it is drawn, and the decision about whether
//! to draw it must be made before. There are three ways out.
//!
//! 1. **Draw, measure, and re-lay-out next frame.** Correct widths, and a
//!    visible one-frame flicker every time the window is resized — which
//!    is exactly when the operator is looking at the ribbon.
//! 2. **Draw into a scratch layer and discard.** Correct widths, double
//!    the work, and every side effect (hover, click, focus) has to be
//!    suppressed on the discarded pass or it fires twice.
//! 3. **Estimate analytically from the item list**, using the same font
//!    metrics `egui` will use to lay the text out. Cheap, single-pass,
//!    and exact to within the padding constants.
//!
//! This module is option 3. `ItemWidths` is fed measured galley widths
//! by [`super::band`] — `egui` memoizes galleys, so asking for the width
//! of a label that is about to be drawn costs a hash lookup — and adds
//! the padding constants the renderer will actually apply.
//!
//! The estimate can be wrong. A [`crate::manifest::Item::Custom`] is
//! drawn by the application and the shell cannot know how wide it will
//! be, so it is budgeted at `CUSTOM_ITEM_WIDTH`. **An estimate that is
//! too small costs a clipped group; it cannot cost the overflow
//! control**, because the overflow control's width was subtracted from
//! the total before the estimate was consulted. That asymmetry is the
//! whole reason the reservation is made first, and it is why a rough
//! estimate is an acceptable input to an invariant this strict.
//!
//! # Minimum widths, and why they matter to the tests
//!
//! Every control is at least `MIN_ITEM_WIDTH` wide. That is a real
//! design rule — a control narrower than it is tall reads as a rendering
//! fault — and it has a second effect worth naming: it makes this
//! module's arithmetic meaningful **even when no font is installed**.
//!
//! This crate depends on `egui` with `default-features = false`, so a
//! test process has no font data and every galley measures near zero. A
//! layout that derived its widths from text alone would collapse to zero
//! in exactly the environment its tests run in, and the overflow tests
//! would be asserting against a band that never overflows. The floor
//! keeps the numbers honest headlessly.
//!
//! ## ★ And why the floor is not enough — read this before trusting a
//! ## width test in this crate
//!
//! The floor keeps the arithmetic *meaningful*; it does not make a
//! zero-width-text test *equivalent* to a real one, and treating the two
//! as equivalent cost this module two defects.
//!
//! The font situation is not merely "absent in tests". It is **decided by
//! whichever sibling crate is in the build**:
//!
//! ```text
//! cargo test -p egui-shell --lib   egui alone            → no fonts, widths ≈ 0
//! cargo test --workspace           pdfce-gui → eframe    → egui/default_fonts,
//!                                                          real widths
//! ```
//!
//! Cargo unifies features across a workspace build, so the same assertions
//! measure different text under the two commands, and the *narrower*
//! command — the one a developer working on the shell reaches for — is
//! the one that measures nothing. Everything below that compares a width
//! against another width was, for the whole of this module's life,
//! trivially satisfied under that command.
//!
//! Two consequences were real defects, both of which only appear with
//! metrics: the overflow affordance being positioned from a `Ui` whose
//! `max_rect` a sibling row had grown (see [`super::band`]), and
//! [`overflow_width`] reserving for the label with the most *characters*
//! rather than the most *width*.
//!
//! `super::width_tests` closes that hole by installing a synthetic
//! proportional face this crate builds itself, so the width-sensitive
//! paths are exercised with real advances under **both** commands and
//! under any future workspace membership. A new width rule added to this
//! module belongs there as well as here.

pub(crate) mod row;

// Re-exported so every call site reads `plan::…` and the split between
// this file and [`row`] stays an organisational fact rather than
// something a caller has to know.
pub(crate) use row::{RowDemand, StripPlan, plan_strip_row, plan_tab_strip};

/// The narrowest a control may be drawn, in points, before the theme's
/// padding is added.
///
/// A control narrower than its own height reads as a clipping artefact
/// rather than as a button. This is also the floor that keeps the
/// arithmetic meaningful with no fonts installed — see the module header.
pub(crate) const MIN_ITEM_WIDTH: f32 = 20.0;

/// The width budgeted for a [`crate::manifest::Item::Custom`].
///
/// The shell does not draw custom items and cannot measure them: the
/// application is handed a `Ui` and draws a colour swatch, a zoom slider
/// or a gallery into it. Four control-widths is a generous guess at a
/// compound control.
///
/// Being wrong here costs a clipped group, never a lost overflow
/// affordance — see the module header on why that asymmetry is the point.
pub(crate) const CUSTOM_ITEM_WIDTH: f32 = 96.0;

/// Horizontal padding inside a group, either side of its content.
///
/// # ⚠ Budgeted here, not drawn by [`super::band`] — recorded 2026-08-13
///
/// [`super::band::captioned_group`] lays a group out as a bare
/// `ui.vertical` with no horizontal inset, so the 12 pt this adds to every
/// group's planned width is space the renderer never uses. Measured
/// against the synthetic face: the three View-tab groups plan at 213.9,
/// 174.4 and 165.6 pt and draw at 202.4, 165.1 and 157.5 pt.
///
/// The discrepancy is in the **safe** direction — the band hides a group
/// slightly earlier than it must, rather than clipping one — so it is left
/// as it is rather than "fixed" in whichever direction a passing reader
/// happens to prefer. It is written down because it is not free:
///
/// - It is an accidental 12 pt-per-group safety margin, and margins that
///   nobody knows about are the ones that get spent. `super::width_tests`'
///   `a_band_that_claims_to_fit_really_does_fit` cannot see an
///   under-estimate smaller than it.
/// - Resolving it is a *design* decision, not an arithmetic one: either a
///   ribbon group has internal padding (in which case the renderer should
///   draw it, and the plan is already right) or it does not (in which case
///   this constant should be zero). `RIBBON_IA.md` does not say, and a
///   renderer change here alters the look of every band.
pub(crate) const GROUP_PADDING: f32 = 6.0;

/// The measured pieces of one item, before padding.
///
/// Separated from the total so [`item_width`] has one place to apply the
/// padding rule and the tests have something to assert against.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ItemWidths {
    /// Width of the item's icon, or `0.0` if it has none.
    pub icon: f32,
    /// Width of the item's visible text, or `0.0` if it is icon-only.
    pub text: f32,
    /// Gap between icon and text, applied only when both are present.
    pub gap: f32,
    /// Padding applied inside the control, both sides together.
    pub padding: f32,
}

impl ItemWidths {
    /// The width this item will occupy, floored at [`MIN_ITEM_WIDTH`].
    pub(crate) fn total(self) -> f32 {
        let gap = if self.icon > 0.0 && self.text > 0.0 {
            self.gap
        } else {
            0.0
        };
        (self.icon + self.text + gap + self.padding).max(MIN_ITEM_WIDTH)
    }
}

/// The width of a whole group: its item row, its caption, and its
/// padding.
///
/// The caption is part of the *width*, not only of the height, and that
/// is deliberate. A group whose caption is wider than its single control
/// — "Page display" over one icon button — is as wide as its caption, and
/// planning it at the control's width would overflow the band by the
/// difference. `RIBBON_IA.md` §5 is full of two-word captions over
/// one-glyph controls, so this is the common case rather than the corner.
pub(crate) fn group_width(item_widths: &[f32], gutter: f32, caption_width: f32) -> f32 {
    let row: f32 = if item_widths.is_empty() {
        0.0
    } else {
        item_widths.iter().sum::<f32>() + gutter * (item_widths.len() as f32 - 1.0)
    };
    row.max(caption_width) + GROUP_PADDING * 2.0
}

/// How a band's groups are split between the visible band and the
/// overflow menu.
///
/// Returned by [`plan_band`]. Every field is a decision the renderer then
/// obeys without re-deriving anything, so that the arithmetic exists in
/// exactly one place and can be tested without a window.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct BandPlan {
    /// How many leading groups are drawn in the band itself.
    ///
    /// Always a *prefix* of the group list: the manifest's order is the
    /// operator's order, and a plan that dropped a group from the middle
    /// would make the visible band's order depend on the window width.
    pub shown: usize,
    /// How many trailing groups moved into the overflow menu.
    pub hidden: usize,
    /// The width the shown groups may occupy, in points.
    ///
    /// **This number already excludes the overflow control's width.** It
    /// is what [`super::band`] uses as the maximum width of the `Ui` the
    /// groups are drawn into, which is the second half of the enforcement
    /// — see the module header.
    pub group_budget: f32,
    /// The width reserved for the overflow affordance, or `0.0` when
    /// nothing overflowed.
    ///
    /// Zero when [`Self::hidden`] is zero, and non-zero whenever it is
    /// not. That biconditional is asserted by
    /// `the_overflow_affordance_is_reserved_exactly_when_it_is_needed`.
    pub overflow_width: f32,
}

impl BandPlan {
    /// Whether the overflow affordance is to be drawn.
    pub(crate) fn has_overflow(self) -> bool {
        self.hidden > 0
    }
}

/// Decide how many groups fit, reserving the overflow affordance's width
/// **before** any group is measured against the remainder.
///
/// # Arguments
///
/// - `available` — the band's usable width in points. A negative,
///   infinite or NaN value is treated as zero; a layout pass can hand out
///   `f32::INFINITY` for available width inside an unbounded container,
///   and `INFINITY - overflow_width` is still infinity, which would
///   silently disable overflow rather than show everything.
/// - `group_widths` — each group's planned width, in manifest order.
/// - `separator` — the width of the vertical rule drawn between adjacent
///   groups, including its own spacing.
/// - `overflow_width` — the width the overflow affordance needs. Computed
///   for the **widest label it could ever show** (see
///   [`overflow_label`]), so the reservation can never turn out to be too
///   small once the hidden count is known.
///
/// # The algorithm, and the one line that matters
///
/// 1. If everything fits, show everything and reserve nothing. An
///    overflow control that took space when there was nothing to overflow
///    into it would be a permanent tax on the band.
/// 2. Otherwise `group_budget = available − overflow_width − separator`,
///    **clamped at zero**. This is the line the whole module exists for.
///    The separator is subtracted too, because a rule is drawn between
///    the last visible group and the overflow control.
/// 3. Fill `group_budget` greedily from the front.
///
/// Step 3 can place **zero** groups — at a width narrower than one group
/// plus the reservation, `shown` is 0 and every group is in the menu.
/// That is the correct answer and it is the case the reservation exists
/// for: the band degrades to a single "⏷ N more" control that still
/// reaches everything, rather than to a band of clipped groups with no
/// route to the rest.
pub(crate) fn plan_band(
    available: f32,
    group_widths: &[f32],
    separator: f32,
    overflow_width: f32,
) -> BandPlan {
    // A non-finite or negative width is not a width. Treating it as zero
    // makes the degenerate case the *safe* one (everything in the menu)
    // rather than the dangerous one (infinite budget, no overflow).
    let available = if available.is_finite() {
        available.max(0.0)
    } else {
        0.0
    };
    let separator = separator.max(0.0);
    let overflow_width = overflow_width.max(0.0);

    let n = group_widths.len();
    if n == 0 {
        return BandPlan {
            shown: 0,
            hidden: 0,
            group_budget: available,
            overflow_width: 0.0,
        };
    }

    let total: f32 = group_widths.iter().sum::<f32>() + separator * (n as f32 - 1.0);
    if total <= available {
        return BandPlan {
            shown: n,
            hidden: 0,
            group_budget: available,
            overflow_width: 0.0,
        };
    }

    // ★ THE RESERVATION. Subtracted before a single group is considered.
    let group_budget = (available - overflow_width - separator).max(0.0);

    let mut used = 0.0_f32;
    let mut shown = 0_usize;
    for (i, w) in group_widths.iter().enumerate() {
        let step = if i == 0 { *w } else { separator + *w };
        if used + step <= group_budget {
            used += step;
            shown += 1;
        } else {
            break;
        }
    }

    BandPlan {
        shown,
        hidden: n - shown,
        group_budget,
        overflow_width,
    }
}

/// The overflow affordance's label for `hidden` hidden groups.
///
/// The chevron is part of the label rather than a separate glyph so the
/// control is one measurable string, and so a build with no icon set
/// still shows an affordance rather than an empty button.
///
/// ## ★ The chevron is `⏷` U+23F7, and the obvious choices are all tofu
///
/// This read `⌄` (U+2304, DOWNWARDS ARROWHEAD) until 2026-08-14 and
/// **rendered as an empty box in every shipped build** — `□ 1 more`,
/// `□ 2 more` — because egui's bundled font stack (Ubuntu-Light +
/// NotoEmoji + emoji-icon-font) has no face for it.
///
/// It is worth naming the near misses, because every one of them is what
/// somebody reaches for first and **four of them were already known to be
/// missing** by a test in the consuming application:
///
/// | codepoint | in the font? |
/// |---|---|
/// | `⌄` U+2304 | **no** — what this was |
/// | `▾` U+25BE, `▼` U+25BC, `⌃` U+2303, `˅` U+02C5 | **no** |
/// | `⏷` U+23F7 | **yes** — and its siblings `⏴` U+23F4 / `⏵` U+23F5 are already in use |
///
/// Measured with `Fonts::has_glyph`, not assumed.
///
/// **Why this crate cannot test it and the application must.** `cargo test
/// -p egui-shell` compiles without egui's `default_fonts`, so `has_glyph`
/// here would answer about a font set that does not exist in any real
/// build — the test would pass, vacuously, for the whole life of the
/// defect. The assertion therefore lives with the fonts, in the
/// application, beside the two that already guard the status bar and the
/// find bar. That is also why this shipped: the crate that owns the string
/// is structurally unable to check it.
pub(crate) fn overflow_label(hidden: usize) -> String {
    format!("⏷ {hidden} more")
}

/// The width to reserve for the overflow affordance, given how many
/// groups the band holds **in total**.
///
/// # Why every reachable label is measured, not just the largest count
///
/// The hidden count is not known until [`plan_band`] has run, and
/// [`plan_band`] needs this number as an input. The circularity has to be
/// broken by reserving for a label that has not been chosen yet, and the
/// only safe direction is the worst case.
///
/// An earlier version measured `"⏷ N more"` for `N = total_groups` alone,
/// reasoning that more hidden groups means a longer string. That is true
/// of the *character count* and false of the *width*: with no font
/// installed every label measures zero and the two agree, but with real
/// metrics `"⏷ 8 more"` is wider than `"⏷ 9 more"` in any face whose
/// digits are not tabular, and a band of nine groups showing one would
/// then draw a control wider than the space reserved for it — the
/// affordance overhanging the band's right edge, which is failure mode #8
/// with the control present but partly unclickable.
///
/// So the reservation is `max` over every label the control can ever
/// display: `1..=total_groups`. That makes the claim *"the drawn control
/// never exceeds its reservation"* a proof rather than an argument about
/// digit shapes. The cost is one memoized galley lookup per group per
/// frame — `egui` caches layout jobs, and a band has single-digit numbers
/// of groups.
///
/// `measure` is the caller's text-measuring function, so this stays free
/// of `egui`.
pub(crate) fn overflow_width(
    total_groups: usize,
    padding: f32,
    measure: impl Fn(&str) -> f32,
) -> f32 {
    let widest = (1..=total_groups.max(1))
        .map(|n| measure(&overflow_label(n)))
        .fold(0.0_f32, f32::max);
    (widest + padding).max(MIN_ITEM_WIDTH)
}
#[cfg(test)]
mod tests {
    use super::*;

    /// Ten groups of 100 pt each, the shape most of these tests want.
    fn widths(n: usize, each: f32) -> Vec<f32> {
        vec![each; n]
    }

    /// An item is never narrower than [`MIN_ITEM_WIDTH`], and the
    /// icon/text gap applies only when both are present.
    ///
    /// The second half is what makes an icon-only control the same width
    /// as a square rather than a square plus a gap to nothing.
    #[test]
    fn an_item_is_floored_and_only_gapped_when_it_has_both_halves() {
        let both = ItemWidths {
            icon: 16.0,
            text: 40.0,
            gap: 4.0,
            padding: 8.0,
        };
        assert_eq!(both.total(), 68.0);

        let icon_only = ItemWidths { text: 0.0, ..both };
        assert_eq!(icon_only.total(), 24.0, "no gap when there is no text");

        let text_only = ItemWidths { icon: 0.0, ..both };
        assert_eq!(text_only.total(), 48.0, "no gap when there is no icon");

        let tiny = ItemWidths {
            icon: 0.0,
            text: 1.0,
            gap: 4.0,
            padding: 2.0,
        };
        assert_eq!(
            tiny.total(),
            MIN_ITEM_WIDTH,
            "a control narrower than it is tall reads as a clipping artefact"
        );
    }

    /// **A group is as wide as its caption when its caption is the wider
    /// half.**
    ///
    /// `RIBBON_IA.md` §5 is full of two-word captions over one-glyph
    /// controls — "Page display" over a single icon button — so this is
    /// the common case, not the corner. Planning such a group at its
    /// control's width would overflow the band by the difference, every
    /// time, and the symptom would be a clipped caption rather than
    /// anything that looks like a layout bug.
    #[test]
    fn a_group_is_as_wide_as_its_caption_when_the_caption_is_wider() {
        let one_narrow_button = [24.0];
        let wide_caption = 90.0;
        assert_eq!(
            group_width(&one_narrow_button, 4.0, wide_caption),
            wide_caption + GROUP_PADDING * 2.0
        );

        let wide_row = [60.0, 60.0];
        assert_eq!(
            group_width(&wide_row, 4.0, 20.0),
            60.0 + 4.0 + 60.0 + GROUP_PADDING * 2.0
        );

        assert_eq!(
            group_width(&[], 4.0, 0.0),
            GROUP_PADDING * 2.0,
            "an empty group is its padding, not a negative number"
        );
    }

    /// Everything fits: no overflow control, and no width taken for one.
    ///
    /// The second half matters — a reservation that persisted when
    /// nothing was hidden would be a permanent tax on every band that
    /// fits.
    #[test]
    fn a_band_that_fits_reserves_nothing() {
        let plan = plan_band(1000.0, &widths(3, 100.0), 8.0, 60.0);
        assert_eq!(plan.shown, 3);
        assert_eq!(plan.hidden, 0);
        assert_eq!(plan.overflow_width, 0.0);
        assert!(!plan.has_overflow());
    }

    /// **★ Failure mode #8: at a width too narrow for even one group, the
    /// overflow affordance is still planned and still has its width.**
    ///
    /// This is the invariant the whole module exists for. The observed
    /// defect it guards against is `MODES_AND_PANELS.md` Part 2, #8: past
    /// a certain count the overflow button *itself* gets hidden, leaving
    /// no route to what it was hiding. The plan must degrade to "no
    /// groups, one working affordance", never to "some clipped groups, no
    /// affordance".
    ///
    /// Checked at three widths on the way down, because the interesting
    /// failure is not at zero — it is at the width where a naive
    /// implementation still has *just* enough room for a group and
    /// therefore spends the overflow control's space on it.
    #[test]
    fn the_overflow_affordance_survives_a_band_too_narrow_for_any_group() {
        let groups = widths(6, 100.0);
        for available in [0.0_f32, 1.0, 40.0, 60.0, 99.0, 100.0, 140.0] {
            let plan = plan_band(available, &groups, 8.0, 60.0);
            assert!(
                plan.has_overflow(),
                "at {available} pt every group must be reachable through the menu"
            );
            assert_eq!(plan.overflow_width, 60.0, "at {available} pt");
            assert_eq!(
                plan.hidden,
                groups.len() - plan.shown,
                "at {available} pt: every group is either shown or reachable"
            );
            assert!(
                plan.group_budget + plan.overflow_width <= available.max(60.0),
                "at {available} pt the groups were allowed into the reserved space"
            );
        }

        // The specific shape of the degenerate case.
        let plan = plan_band(10.0, &groups, 8.0, 60.0);
        assert_eq!(plan.shown, 0, "no group fits beside the reservation");
        assert_eq!(plan.hidden, 6, "so all six are in the menu");
        assert_eq!(plan.group_budget, 0.0, "and the groups get nothing");
    }

    /// **The reservation is subtracted before any group is placed.**
    ///
    /// Stated as arithmetic rather than as an outcome, because this is
    /// the property that cannot be reintroduced by a later edit to the
    /// group loop: whatever the loop does, it is filling a budget that
    /// never contained the overflow control's width.
    ///
    /// The equality below is exact and is the whole rule:
    ///
    /// ```text
    /// group_budget == max(0, available − overflow_width − separator)
    /// ```
    ///
    /// Note that this is *not* the same as "budget + reservation ≤
    /// available". Once the band is narrower than the reservation itself,
    /// the reservation deliberately exceeds the band — the affordance
    /// keeps its width and the groups get nothing, which is exactly the
    /// degenerate case failure mode #8 is about. Writing the assertion
    /// the other way would have demanded the opposite behaviour.
    #[test]
    fn the_group_budget_never_contains_the_reservation() {
        const SEP: f32 = 8.0;
        const RESERVE: f32 = 60.0;
        for n in 1..12_usize {
            for available in (0..900).step_by(17).map(|w| w as f32) {
                let groups = widths(n, 100.0);
                let plan = plan_band(available, &groups, SEP, RESERVE);
                if plan.has_overflow() {
                    assert_eq!(
                        plan.group_budget,
                        (available - RESERVE - SEP).max(0.0),
                        "n={n} available={available}: the group budget was not \
                         the band minus the reservation"
                    );
                    assert!(
                        plan.group_budget <= available,
                        "n={n} available={available}: the groups were budgeted \
                         more than the whole band"
                    );
                }
            }
        }
    }

    /// The overflow affordance is reserved **exactly** when it is needed:
    /// `hidden > 0` if and only if `overflow_width > 0`.
    ///
    /// A biconditional rather than one implication, because both
    /// directions are real defects. Reserving with nothing hidden wastes
    /// band width forever; hiding with nothing reserved is #8 itself.
    #[test]
    fn the_overflow_affordance_is_reserved_exactly_when_it_is_needed() {
        for available in (0..1200).step_by(13).map(|w| w as f32) {
            let plan = plan_band(available, &widths(5, 100.0), 8.0, 60.0);
            assert_eq!(
                plan.hidden > 0,
                plan.overflow_width > 0.0,
                "at {available} pt: hidden={} reserved={}",
                plan.hidden,
                plan.overflow_width
            );
        }
    }

    /// A wider band never shows fewer groups.
    ///
    /// Monotonicity is what makes a window resize feel like a resize.
    /// A greedy fill has it by construction; a later "pack the widest
    /// first" optimisation would not, and this test is what would refuse
    /// that change.
    #[test]
    fn widening_the_band_never_hides_a_group_that_was_visible() {
        let groups = [40.0, 120.0, 30.0, 200.0, 55.0, 90.0];
        let mut last = 0;
        for available in (0..1400).step_by(3).map(|w| w as f32) {
            let shown = plan_band(available, &groups, 8.0, 60.0).shown;
            assert!(
                shown >= last,
                "widening to {available} pt dropped a group that fitted at a narrower width"
            );
            last = shown;
        }
        assert_eq!(last, groups.len(), "at 1400 pt everything must be visible");
    }

    /// The shown groups are always a **prefix** of the manifest order,
    /// and the two counts always sum to the whole band.
    ///
    /// The prefix property is what stops the visible ordering of the
    /// ribbon from depending on the window width — the manifest's order
    /// is the operator's order, and a plan that dropped a middle group to
    /// fit a later narrow one would rearrange the ribbon as the window
    /// moved.
    #[test]
    fn the_visible_groups_are_a_prefix_and_nothing_is_lost() {
        let groups = [40.0, 200.0, 30.0];
        for available in (0..600).step_by(7).map(|w| w as f32) {
            let plan = plan_band(available, &groups, 8.0, 60.0);
            assert_eq!(plan.shown + plan.hidden, groups.len(), "at {available} pt");
            assert!(plan.shown <= groups.len());
        }
    }

    /// **A non-finite available width degrades to "everything in the
    /// menu", not to "infinite room".**
    ///
    /// `egui` hands out `f32::INFINITY` for available width inside an
    /// unbounded container. `INFINITY - overflow_width` is still
    /// infinity, so an unguarded implementation would conclude that
    /// everything fits and emit no affordance — the #8 defect arriving
    /// through arithmetic rather than through ordering.
    #[test]
    fn a_non_finite_width_degrades_safely() {
        for bad in [f32::INFINITY, f32::NEG_INFINITY, f32::NAN, -50.0] {
            let plan = plan_band(bad, &widths(4, 100.0), 8.0, 60.0);
            assert_eq!(plan.shown, 0, "available={bad}");
            assert_eq!(plan.hidden, 4, "available={bad}");
            assert!(plan.has_overflow(), "available={bad}");
        }
    }

    /// An empty band plans nothing and reserves nothing.
    #[test]
    fn an_empty_band_plans_nothing() {
        let plan = plan_band(500.0, &[], 8.0, 60.0);
        assert_eq!(plan.shown, 0);
        assert_eq!(plan.hidden, 0);
        assert!(!plan.has_overflow());
    }

    /// **★ The reservation covers the widest label by WIDTH, not by
    /// character count.**
    ///
    /// The trap this pins is the one real text springs and zero-width
    /// text cannot: in a face whose digits are not tabular, `"⏷ 8 more"`
    /// can be wider than `"⏷ 9 more"` even though the counts and the
    /// lengths say otherwise. Reserving for `N = total_groups` alone —
    /// which reads as obviously sufficient, and is what this function used
    /// to do — then draws a control wider than the space held for it, and
    /// the affordance overhangs the band's right edge.
    ///
    /// The `measure` below is deliberately perverse about exactly that:
    /// every character costs 7 pt except `8`, which costs 40. A
    /// reservation that consults only the largest count fails here; one
    /// that takes the maximum over every reachable label passes.
    #[test]
    fn the_reservation_covers_the_widest_label_by_width_not_by_digit_count() {
        let measure = |s: &str| {
            s.chars()
                .map(|c| if c == '8' { 40.0 } else { 7.0 })
                .sum::<f32>()
        };
        let reserved = overflow_width(9, 10.0, measure);
        for hidden in 1..=9 {
            let label = overflow_label(hidden);
            assert!(
                reserved >= measure(&label) + 10.0,
                "a band of nine groups reserved {reserved} pt, but with {hidden} \
                 hidden it draws {label:?} at {} pt plus padding",
                measure(&label)
            );
        }
    }

    /// The reservation is sized for the widest label it could ever show,
    /// so it cannot turn out to be too small once the hidden count is
    /// known.
    #[test]
    fn the_reservation_is_sized_for_the_worst_case_label() {
        // A measure that charges 7 pt per character, so the assertion is
        // about the label chosen rather than about a font.
        let measure = |s: &str| s.chars().count() as f32 * 7.0;
        let for_twelve = overflow_width(12, 10.0, measure);
        let for_two = overflow_width(2, 10.0, measure);
        assert!(
            for_twelve >= for_two,
            "a band of twelve groups must reserve at least what a band of two does"
        );
        assert!(
            for_twelve >= measure(&overflow_label(12)),
            "the reservation must fit the widest label the control can show"
        );
        assert_eq!(
            overflow_width(0, 0.0, |_| 0.0),
            MIN_ITEM_WIDTH,
            "even with no text the affordance is a clickable size"
        );
    }

    /// The label says how many are hidden, which is the difference
    /// between an affordance and a mystery chevron.
    #[test]
    fn the_overflow_label_states_the_count() {
        assert_eq!(overflow_label(3), "⏷ 3 more");
        assert_eq!(overflow_label(1), "⏷ 1 more");
    }

    /// ★ **The chevron is the pinned codepoint — one half of a two-sided
    /// pin, and this half cannot check the thing that actually matters.**
    ///
    /// Whether the character is *drawable* is asserted in the consuming
    /// application (`pdfce_gui::shell`'s
    /// `the_ribbon_overflow_chevron_has_a_glyph`), because `cargo test -p
    /// egui-shell` compiles without egui's `default_fonts` — a `has_glyph`
    /// call here would answer about a font set no real build has, and would
    /// pass for the whole life of a defect. It did: `⌄` U+2304 shipped as a
    /// tofu box on every ribbon band and dock tab bar this project has
    /// produced.
    ///
    /// So this test does the half it *can* do honestly: pin the codepoint,
    /// so that changing it is a deliberate act which fails a named test and
    /// sends the next reader to the other half. Both docks and both rows
    /// are covered, because [`crate::dock::plan::overflow_label`] promises
    /// to stay identical and identical wording means identical codepoints.
    #[test]
    fn the_overflow_label_uses_the_pinned_chevron() {
        const PINNED: char = '\u{23F7}';
        let label = overflow_label(2);
        let first = label.chars().next().expect("a non-empty label");
        assert_eq!(
            first, PINNED,
            "the overflow chevron changed to U+{:04X}. That is allowed, but the \
             bundled fonts must be able to draw it — see this module's own note on \
             which near misses are missing, and update \
             `pdfce_gui::shell::tests::the_ribbon_overflow_chevron_has_a_glyph`, \
             which is the only place that can check.",
            first as u32
        );
        assert_eq!(
            crate::dock::plan::overflow_label(2),
            label,
            "the dock and the ribbon must spell the affordance identically; an \
             operator should not have to learn two overflow idioms in one window"
        );
    }
}
