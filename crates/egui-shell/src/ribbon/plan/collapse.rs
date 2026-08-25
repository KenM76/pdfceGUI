//! # `ribbon::plan::collapse` — the collapse ladder
//!
//! **S3 of `RIBBON_SCALING.md`.** When a band does not fit, this decides
//! *which groups give up their rows*, and in what order, before anything is
//! pushed into the overflow menu at all.
//!
//! ## What Word actually does, measured rather than remembered
//!
//! The specification for this file is a series of photographs in
//! `evidence/word-ribbon/`, taken at twelve window widths, because a ribbon's
//! scaling rules are implemented inside the Office UI framework and exposed
//! through no API — `Application.CommandBars` is the 2003 toolbar surface and
//! says nothing about any of this. Three readings decide everything here:
//!
//! | width | what the Home tab shows |
//! |---:|---|
//! | 1300 | Clipboard, Font, Paragraph, Styles, Editing … all expanded, Font and Paragraph on two rows |
//! | 800 | **Font and Paragraph are single captioned buttons with a chevron.** Clipboard is unchanged |
//! | 460 | Font, Paragraph, Styles, Editing all collapsed; **Clipboard still expanded**; a `›` scroll arrow at the band's right edge |
//!
//! Three facts follow, and all three are counter-intuitive enough to be worth
//! stating:
//!
//! 1. **Groups do not re-wrap onto more rows as the window narrows.** The row
//!    split is a property of the group's own content — see
//!    [`super::GROUP_WRAP_WIDTH`] — and it does not change with the window.
//!    Narrowing collapses whole groups instead. This is the answer to *"will
//!    it wrap tools in their sections onto second lines when the window is
//!    resized?"*: it already wraps, by content, and Word does not re-wrap by
//!    window either.
//! 2. **A group can decline to collapse, and the one that declines is not the
//!    small one.** Clipboard is wider than Editing and outlives it at every
//!    width. The property being selected on is *importance*, which is
//!    editorial, which is why it is [`crate::manifest::Group::collapse`] in
//!    the manifest and not a heuristic here.
//! 3. **Collapsing comes before scrolling, and before the overflow menu.** The
//!    `›` arrow appears at 460 and not at 800, by which point four groups have
//!    already collapsed. A surface that scrolls first would be hiding commands
//!    while the space to show them existed.
//!
//! ## ★★★ The two `plan_band` invariants, restated
//!
//! `RIBBON_SCALING.md` §6 names this stage's real cost: two invariants that
//! `super::plan_band` has held since it was written both have to be re-stated
//! for a world in which a group can **shrink** instead of vanishing.
//!
//! **`the_visible_groups_are_a_prefix_and_nothing_is_lost`.** Still true, and
//! now says something stronger. The shown groups remain a prefix of the
//! manifest order; what changes is that a group in that prefix may be present
//! in *collapsed* form. Nothing is lost either way — a collapsed group's items
//! are all reachable through its own popup, exactly as a hidden group's are
//! through the overflow menu. The count that must be conserved is therefore
//! `shown + hidden == n` regardless of how many of the shown are collapsed,
//! and [`collapse_to_fit`] never changes `n`.
//!
//! **`widening_the_band_never_hides_a_group_that_was_visible`.** This is the
//! one that needed real care, because the obvious implementation breaks it.
//! Monotonicity now has **two** rungs rather than one, and both must hold:
//!
//! * widening never collapses a group that was expanded, and
//! * widening never hides a group that was visible **or** collapsed.
//!
//! [`collapse_to_fit`] gets this by construction rather than by testing for
//! it: it always starts from *everything expanded* and collapses forward in a
//! fixed priority order until it fits. It never starts from the previous
//! frame's answer. So the set collapsed at width `w` is a pure function of
//! `w`, and it shrinks as `w` grows — there is no state to drift and no
//! hysteresis to tune.
//!
//! ★ That is also why this is a free function over slices rather than a method
//! on something that remembers. **A layout that remembers what it did last
//! frame is how a ribbon acquires a width at which it flickers**, and this
//! project has already paid once for a measurement fed back into a size (see
//! `dialogs::host`'s growth budget).

/// One group, as the ladder needs to see it.
///
/// Deliberately not the manifest's `Group`: the ladder needs three numbers and
/// nothing else, and keeping it that way is what makes it testable without
/// building a manifest, a registry and a font.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Candidate {
    /// What the group costs with all its rows drawn.
    pub expanded: f32,
    /// What it costs as a single captioned button with a chevron.
    ///
    /// Always the smaller of the two in practice; the ladder does not require
    /// it to be, and simply gains nothing from collapsing a group whose
    /// collapsed form is no narrower.
    pub collapsed: f32,
    /// Its rung on the ladder, or `None` for a group that never collapses.
    ///
    /// Lower collapses first. See [`crate::manifest::Group::collapse`].
    pub priority: Option<u32>,
}

/// **Which groups must collapse for the band to fit.**
///
/// Returns a mask parallel to `groups`: `true` means *draw this one
/// collapsed*. The mask is a pure function of its inputs, which is the whole
/// of the monotonicity argument in this module's header.
///
/// # The algorithm, and why it stops where it does
///
/// 1. Measure everything expanded. If it fits, collapse nothing — a band that
///    collapsed a group it had room for would be hiding commands for no gain.
/// 2. Otherwise collapse the next group in priority order, re-measure, and
///    repeat.
/// 3. Stop when it fits, **or when every group that may collapse has**. The
///    second exit is not a failure: what is left over is handed to
///    [`super::plan_band`], whose overflow menu is the next rung and was
///    always going to be the last resort.
///
/// ★ Step 2 re-measures rather than subtracting a precomputed saving, because
/// the two are not the same once separators are involved and the difference is
/// exactly the kind of one-group-too-many error that shows up only at one
/// window width.
///
/// # Ties
///
/// Equal priorities collapse in manifest order — left to right — because that
/// is what a reader watching the band narrow would predict. `sort_by_key` is
/// stable, so this needs no special handling, but it is asserted rather than
/// assumed.
pub(crate) fn collapse_to_fit(groups: &[Candidate], available: f32, separator: f32) -> Vec<bool> {
    let mut mask = vec![false; groups.len()];
    if groups.is_empty() {
        return mask;
    }

    // A non-finite or negative width is not a width. Zero makes the degenerate
    // case the safe one — collapse everything that may collapse — rather than
    // the dangerous one, an infinite budget that collapses nothing and clips.
    let available = if available.is_finite() {
        available.max(0.0)
    } else {
        0.0
    };
    let separator = if separator.is_finite() {
        separator.max(0.0)
    } else {
        0.0
    };

    if width_of(groups, &mask, separator) <= available {
        return mask;
    }

    // The ladder, resolved once. `(priority, index)` with a stable sort gives
    // manifest order within a priority for free.
    let mut ladder: Vec<(u32, usize)> = groups
        .iter()
        .enumerate()
        .filter_map(|(i, g)| g.priority.map(|p| (p, i)))
        .collect();
    ladder.sort_by_key(|(p, _)| *p);

    for (_, i) in ladder {
        mask[i] = true;
        if width_of(groups, &mask, separator) <= available {
            break;
        }
    }
    mask
}

/// What the band costs with this collapse mask applied.
///
/// `n` groups carry `n − 1` separators, and a collapsed group still occupies a
/// slot — it is narrower, not absent. Conflating "collapsed" with "gone" is
/// the mistake that would break the conservation half of
/// `the_visible_groups_are_a_prefix_and_nothing_is_lost`.
fn width_of(groups: &[Candidate], mask: &[bool], separator: f32) -> f32 {
    let sum: f32 = groups
        .iter()
        .zip(mask)
        .map(|(g, &c)| if c { g.collapsed } else { g.expanded })
        .sum();
    sum + separator * (groups.len().saturating_sub(1)) as f32
}

/// The widths [`super::plan_band`] should be given, once the ladder has run.
///
/// A convenience so the caller does not re-derive the same `if` in a third
/// place — the mask and the widths must agree, and the cheapest way to
/// guarantee that is to produce them together.
pub(crate) fn widths_after(groups: &[Candidate], mask: &[bool]) -> Vec<f32> {
    groups
        .iter()
        .zip(mask)
        .map(|(g, &c)| if c { g.collapsed } else { g.expanded })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Three groups, the middle one narrow when collapsed.
    fn trio() -> Vec<Candidate> {
        vec![
            Candidate {
                expanded: 100.0,
                collapsed: 40.0,
                priority: Some(2),
            },
            Candidate {
                expanded: 200.0,
                collapsed: 40.0,
                priority: Some(1),
            },
            Candidate {
                expanded: 150.0,
                collapsed: 40.0,
                priority: Some(3),
            },
        ]
    }

    /// **A band with room collapses nothing.** The first rung of the ladder is
    /// not standing on it.
    #[test]
    fn a_band_that_fits_collapses_nothing() {
        let g = trio();
        // 100 + 200 + 150 + 2 separators of 8 = 466
        assert_eq!(collapse_to_fit(&g, 500.0, 8.0), vec![false, false, false]);
    }

    /// **Lower priority collapses first**, and only as far as it must.
    #[test]
    fn the_ladder_is_climbed_in_priority_order_and_stops_when_it_fits() {
        let g = trio();
        // Collapsing priority 1 (index 1) saves 160 -> 306. Fits in 320.
        assert_eq!(collapse_to_fit(&g, 320.0, 8.0), vec![false, true, false]);
    }

    /// …and keeps going when one is not enough.
    #[test]
    fn the_ladder_continues_until_it_fits() {
        let g = trio();
        // 306 after the first; collapsing index 0 too saves 60 -> 246.
        assert_eq!(collapse_to_fit(&g, 250.0, 8.0), vec![true, true, false]);
    }

    /// ★★ **A group with no priority never collapses, at any width** — the
    /// Clipboard case, and the whole reason the field is an `Option`.
    ///
    /// Asserted at a width far below what the group alone costs, because that
    /// is the condition under which a well-meaning "collapse everything as a
    /// last resort" fallback would fire.
    #[test]
    fn a_group_that_declines_to_collapse_survives_any_width() {
        let g = vec![
            Candidate {
                expanded: 300.0,
                collapsed: 40.0,
                priority: None,
            },
            Candidate {
                expanded: 200.0,
                collapsed: 40.0,
                priority: Some(1),
            },
        ];
        let mask = collapse_to_fit(&g, 10.0, 8.0);
        assert_eq!(
            mask,
            vec![false, true],
            "a group with no collapse priority must keep its rows even when \
             the band cannot possibly fit — what happens next is the overflow \
             menu's problem, not the ladder's"
        );
    }

    /// **Running out of ladder is not a failure.** Everything collapsible is
    /// collapsed and the remainder is left for the overflow menu.
    #[test]
    fn exhausting_the_ladder_leaves_the_rest_to_the_overflow_menu() {
        let g = trio();
        assert_eq!(collapse_to_fit(&g, 1.0, 8.0), vec![true, true, true]);
    }

    /// **Equal priorities collapse left to right.**
    #[test]
    fn ties_break_on_manifest_order() {
        let g = vec![
            Candidate {
                expanded: 100.0,
                collapsed: 10.0,
                priority: Some(5),
            },
            Candidate {
                expanded: 100.0,
                collapsed: 10.0,
                priority: Some(5),
            },
        ];
        // Needs exactly one collapsed: 100 + 10 + 8 = 118.
        assert_eq!(collapse_to_fit(&g, 120.0, 8.0), vec![true, false]);
    }

    /// ★★★ **Widening never re-collapses, and never hides.** The restated
    /// monotonicity invariant, swept rather than spot-checked.
    ///
    /// For every pair of widths `w1 < w2`, every group collapsed at `w2` must
    /// also have been collapsed at `w1`. Equivalently: the collapsed set only
    /// ever shrinks as the band grows. A hysteresis bug — the classic
    /// consequence of planning from the previous frame's answer instead of
    /// from scratch — shows up here as a width at which the set grows again.
    #[test]
    fn widening_the_band_never_collapses_a_group_that_was_expanded() {
        let g = trio();
        let mut prev: Option<Vec<bool>> = None;
        // Walk from narrow to wide, one point at a time.
        for w in 0..600 {
            let mask = collapse_to_fit(&g, w as f32, 8.0);
            if let Some(prev) = &prev {
                for (i, (&now, &before)) in mask.iter().zip(prev).enumerate() {
                    assert!(
                        !(now && !before),
                        "group {i} collapsed at width {w} but was expanded at \
                         {}, which means the band flickers somewhere between \
                         the two",
                        w - 1
                    );
                }
            }
            prev = Some(mask);
        }
    }

    /// **The mask and the widths agree**, because they are produced together.
    #[test]
    fn widths_after_matches_the_mask() {
        let g = trio();
        let mask = vec![false, true, false];
        assert_eq!(widths_after(&g, &mask), vec![100.0, 40.0, 150.0]);
    }

    /// A degenerate width does not produce a degenerate band: it collapses
    /// everything that may collapse, rather than nothing.
    #[test]
    fn a_nonsense_width_is_treated_as_zero_not_as_infinity() {
        let g = trio();
        assert_eq!(collapse_to_fit(&g, f32::NAN, 8.0), vec![true, true, true]);
    }

    /// No groups, no work, no panic.
    #[test]
    fn an_empty_band_collapses_nothing() {
        assert!(collapse_to_fit(&[], 100.0, 8.0).is_empty());
    }
}
