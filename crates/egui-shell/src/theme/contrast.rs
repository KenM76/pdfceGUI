//! The rendered-pair contrast gate.
//!
//! # Why this module exists, and what it is a reaction to
//!
//! `DEFECTS.md` D2: every collapsible section heading in the settings
//! dialog and both dock tab labels rendered near-white on light grey. At
//! 1× they were simply not readable. The cause was one unassigned field —
//! `widgets.active.bg_fill` was never given the accent, while
//! `widgets.active.fg_stroke` was given a near-white plate colour — so
//! any widget that paints with `bg_fill` rather than `weak_bg_fill` got a
//! near-white foreground on `egui`'s stock light background.
//!
//! **Two theme tests sat directly adjacent to it and neither could have
//! caught it**, which is the finding worth carrying forward:
//!
//! - One compared `text` against `surface` and `panel`. The foreground
//!   that failed was neither of those.
//! - One asserted the plate colour stays light. That is correct for its
//!   stated purpose, and it therefore **agreed with the defect**.
//!
//! Both are *palette-vs-palette* tests: they compare two colours a human
//! deliberately wrote down beside each other. The defect was not in the
//! palette. It was in the **assignment** — which palette entry ends up as
//! a foreground and which as a background on the `egui::Style` that
//! actually gets painted. The pair that rendered was never a pair anyone
//! wrote down, so no palette test could contain it.
//!
//! A structural gate could not see it either. The project's
//! `check-theme-colors.sh` bans raw `Color32` literals outside the theme
//! module — a real and useful rule that says nothing about whether the
//! named colours are legible together. *The gate was structural, not
//! perceptual.*
//!
//! # What this module does instead
//!
//! It enumerates the **render surface**, not the author's intentions.
//! [`pairs`] walks `egui`'s five widget states × two background fills and
//! reports the ten `(fg_stroke.color, bg_fill)` pairs `egui` will paint,
//! read back from a real `egui::Style`. [`check`] measures each and
//! returns every failure.
//!
//! The consequence that matters: a fill somebody *forgets* to assign in a
//! future preset is caught by the same assertion that catches a fill
//! somebody assigns *wrongly*, because an unassigned field still has a
//! value and that value still gets painted. A test written as a list of
//! pairs to check would have needed somebody to think of the missing one
//! — which is precisely what did not happen.
//!
//! # What this module deliberately is not
//!
//! It is **not a WCAG conformance check**. It computes a crude
//! relative-luminance gap on a 0–255 scale, not a contrast ratio against
//! the sRGB transfer function, and it makes no accessibility claim. That
//! is the salvage source's own reasoning about its coarser sibling and it
//! applies with equal force here:
//!
//! > a coarse check that always fires beats a precise one nobody runs.
//!
//! The failure mode being guarded against is not "3.9:1 where 4.5:1 was
//! wanted". It is "white on white", and a crude measure catches that
//! every time, with no colour-science dependency and no argument about
//! which standard applies to a 1 px stroke.
//!
//! An application that wants a real WCAG gate should build one on top of
//! [`pairs`], which is the part that is hard to get right.

use egui::{Color32, Style};

/// The luminance gap below which a rendered pair is considered
/// unreadable.
///
/// 90 on a 0–255 crude luminance scale. The figure is inherited from the
/// salvaged palette-level text test so that both gates agree about what
/// "readable" means; a theme that satisfies one and not the other would
/// produce two contradictory failures for one edit.
///
/// The shipped presets clear it comfortably. The tightest real pair is
/// `on_accent` on `accent` in the dark preset, at roughly 125.
pub const READABLE_LUMA_GAP: f32 = 90.0;

/// Which of `egui`'s five widget states a pair came from.
///
/// Mirrors `egui::style::Widgets`' fields. A local enum rather than a
/// re-export because the point of it is to be *named in a failure
/// message* — "the Active state's bg_fill" is the sentence that points at
/// the line to change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetState {
    /// Not interactive at all: labels, separators, panel frames.
    NonInteractive,
    /// Interactive, at rest.
    Inactive,
    /// Under the pointer.
    Hovered,
    /// Being pressed, or currently the selected/active one.
    Active,
    /// An open menu, combo box or collapsing header.
    Open,
}

impl WidgetState {
    /// Every state, in `egui`'s own order.
    pub const ALL: &'static [WidgetState] = &[
        WidgetState::NonInteractive,
        WidgetState::Inactive,
        WidgetState::Hovered,
        WidgetState::Active,
        WidgetState::Open,
    ];

    /// The `egui` style entry for this state.
    fn visuals(self, style: &Style) -> &egui::style::WidgetVisuals {
        let w = &style.visuals.widgets;
        match self {
            WidgetState::NonInteractive => &w.noninteractive,
            WidgetState::Inactive => &w.inactive,
            WidgetState::Hovered => &w.hovered,
            WidgetState::Active => &w.active,
            WidgetState::Open => &w.open,
        }
    }
}

/// Which of the two backgrounds a widget may paint with.
///
/// # Why both, and why this distinction is the whole defect
///
/// `egui` gives each widget state two background colours and lets each
/// widget choose. `Button` and `SelectableLabel` paint `weak_bg_fill`;
/// `CollapsingHeader` headers, `egui_tiles` tab buttons and several
/// others paint `bg_fill`. A theme that assigns one and not the other has
/// themed an arbitrary subset of its own widgets, and which subset is
/// decided by `egui`'s internals rather than by the theme's author.
///
/// D2 is exactly that: `weak_bg_fill` was assigned the accent, `bg_fill`
/// was not, and the widgets that lost were the ones nobody happened to
/// look at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillKind {
    /// `WidgetVisuals::bg_fill` — used by `CollapsingHeader` headers,
    /// tab buttons, and anything drawing a solid widget body.
    BgFill,
    /// `WidgetVisuals::weak_bg_fill` — used by `Button` and friends.
    WeakBgFill,
}

impl FillKind {
    /// Both kinds.
    pub const ALL: &'static [FillKind] = &[FillKind::BgFill, FillKind::WeakBgFill];

    /// The colour for this kind, from a state's visuals.
    fn of(self, v: &egui::style::WidgetVisuals) -> Color32 {
        match self {
            FillKind::BgFill => v.bg_fill,
            FillKind::WeakBgFill => v.weak_bg_fill,
        }
    }
}

/// One foreground/background pair as `egui` will paint it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pair {
    /// Which widget state this pair belongs to.
    pub state: WidgetState,
    /// Which of the state's two backgrounds this pair uses.
    pub fill: FillKind,
    /// The foreground, i.e. `fg_stroke.color`.
    pub fg: Color32,
    /// The background.
    pub bg: Color32,
    /// The crude relative-luminance gap between the two, after
    /// compositing a translucent foreground over the background.
    ///
    /// Stored rather than recomputed so a caller reporting a failure and
    /// a caller ranking pairs cannot disagree about the number.
    pub gap: f32,
}

/// A pair that failed the gate.
///
/// Carries the whole [`Pair`] plus the threshold it was measured against,
/// because a failure message that says "gap 41" without saying "needed
/// 90" makes the reader go and find the threshold.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContrastFailure {
    /// Which widget state failed.
    pub state: WidgetState,
    /// Which fill failed.
    pub fill: FillKind,
    /// The foreground that was measured.
    pub fg: Color32,
    /// The background it was measured against.
    pub bg: Color32,
    /// The gap that was measured.
    pub gap: f32,
    /// The gap that was required.
    pub threshold: f32,
}

impl std::fmt::Display for ContrastFailure {
    /// A one-line diagnostic naming the state, the fill and both colours.
    ///
    /// **This is diagnostic text, not operator-visible copy.** It is
    /// written for a failing test, a CI log or a verification harness. An
    /// application that wants to surface a theme problem to a user should
    /// render the structured fields itself, in its own string catalogue —
    /// the shell has no business deciding how another project words a
    /// message to its operator.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fill = match self.fill {
            FillKind::BgFill => "bg_fill",
            FillKind::WeakBgFill => "weak_bg_fill",
        };
        write!(
            f,
            "widgets.{:?}.fg_stroke {:?} on widgets.{:?}.{fill} {:?}: \
             luminance gap {:.0}, needs {:.0}",
            self.state, self.fg, self.state, self.bg, self.gap, self.threshold
        )
    }
}

/// Crude relative luminance of a colour, on a 0–255 scale.
///
/// The Rec. 709 coefficients applied directly to sRGB bytes, with no
/// linearization. This is not photometrically correct and is not trying
/// to be — see the module header on why a coarse measure is the right
/// tool for the failure being guarded against.
///
/// The alpha channel is ignored. Composite first with [`over`] if it
/// matters; [`pairs`] does.
#[must_use]
pub fn luma(c: Color32) -> f32 {
    0.2126 * f32::from(c.r()) + 0.7152 * f32::from(c.g()) + 0.0722 * f32::from(c.b())
}

/// Composite `fg` over `bg` using `fg`'s alpha, returning the opaque
/// result.
///
/// # Why this is necessary rather than fussy
///
/// The colour at the heart of D2 was `rgba(250,250,250,220)` — a
/// *translucent* near-white. Measuring its luminance as if it were opaque
/// overstates its contrast against a dark background and understates it
/// against a light one, and a plate colour used as a foreground is
/// exactly the case where that error is largest. A gate that got this
/// wrong would be wrong specifically about the defect it exists to catch.
///
/// `Color32` in `egui` is premultiplied, so the source channels are
/// already scaled by alpha and the composite is `src + dst·(1−a)`.
#[must_use]
pub fn over(fg: Color32, bg: Color32) -> Color32 {
    let a = f32::from(fg.a()) / 255.0;
    let mix = |s: u8, d: u8| -> u8 {
        let v = f32::from(s) + f32::from(d) * (1.0 - a);
        // Saturating rather than wrapping: a premultiplied source whose
        // channel exceeds its own alpha (which malformed input can
        // produce) must clip to white, never wrap to black.
        v.clamp(0.0, 255.0) as u8
    };
    Color32::from_rgb(
        mix(fg.r(), bg.r()),
        mix(fg.g(), bg.g()),
        mix(fg.b(), bg.b()),
    )
}

/// The luminance gap between a foreground and a background, compositing
/// the foreground's alpha over the background first.
#[must_use]
pub fn gap(fg: Color32, bg: Color32) -> f32 {
    (luma(over(fg, bg)) - luma(bg)).abs()
}

/// Every foreground/background pair a style will render.
///
/// Ten of them: five widget states × two background fills. The
/// enumeration is over `egui`'s own matrix rather than over a list
/// somebody maintained, which is what makes an *unassigned* field as
/// visible to the gate as a *wrongly assigned* one.
#[must_use]
pub fn pairs(style: &Style) -> Vec<Pair> {
    let mut out = Vec::with_capacity(WidgetState::ALL.len() * FillKind::ALL.len());
    for &state in WidgetState::ALL {
        let v = state.visuals(style);
        let fg = v.fg_stroke.color;
        for &fill in FillKind::ALL {
            let bg = fill.of(v);
            out.push(Pair {
                state,
                fill,
                fg,
                bg,
                gap: gap(fg, bg),
            });
        }
    }
    out
}

/// Measure every rendered pair in `style` against `threshold`.
///
/// # Errors
///
/// Returns **every** failing pair rather than the first, so one run names
/// the whole problem. A gate that reports one failure at a time turns a
/// theme edit into a sequence of rebuilds, and the second failure is
/// often the one that explains the first.
pub fn check(style: &Style, threshold: f32) -> Result<(), Vec<ContrastFailure>> {
    let failures: Vec<ContrastFailure> = pairs(style)
        .into_iter()
        .filter(|p| p.gap < threshold)
        .map(|p| ContrastFailure {
            state: p.state,
            fill: p.fill,
            fg: p.fg,
            bg: p.bg,
            gap: p.gap,
            threshold,
        })
        .collect();
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The matrix is complete: ten pairs, one per state per fill, with no
    /// state or fill silently missing.
    ///
    /// Worth its own test because the entire value of this module is that
    /// its coverage is defined by `egui`'s matrix rather than by a list.
    /// If a state were dropped from [`WidgetState::ALL`] the gate would
    /// still pass everything it looked at, and nothing else would notice.
    #[test]
    fn the_pair_matrix_covers_every_state_and_both_fills() {
        let pairs = pairs(&Style::default());
        assert_eq!(pairs.len(), 10, "five widget states × two fills");
        for &state in WidgetState::ALL {
            for &fill in FillKind::ALL {
                assert!(
                    pairs.iter().any(|p| p.state == state && p.fill == fill),
                    "{state:?}/{fill:?} is not measured, so a defect there is invisible"
                );
            }
        }
    }

    /// A translucent foreground is composited before being measured.
    ///
    /// The specific number matters: `rgba(250,250,250,220)` is the plate
    /// colour from D2, and treating it as opaque is the mistake that
    /// would make this gate wrong about its own defect.
    #[test]
    fn a_translucent_foreground_is_composited_not_treated_as_opaque() {
        let plate = Color32::from_rgba_unmultiplied(250, 250, 250, 220);
        let on_black = over(plate, Color32::BLACK);
        let on_white = over(plate, Color32::WHITE);
        assert!(
            luma(on_black) < luma(on_white),
            "the same translucent colour must resolve differently over \
             different backgrounds, or the gate is measuring a colour that \
             is never painted"
        );
        // Over black, 86% of a near-white: clearly lighter than mid-grey,
        // clearly not the 250 an opaque reading would give.
        assert!(
            luma(on_black) > 180.0 && luma(on_black) < 230.0,
            "unexpected composite luminance {}",
            luma(on_black)
        );
    }

    /// White on white fails; black on white passes. The floor test.
    #[test]
    fn the_gate_separates_the_obvious_cases() {
        assert!(gap(Color32::WHITE, Color32::WHITE) < READABLE_LUMA_GAP);
        assert!(gap(Color32::BLACK, Color32::WHITE) > READABLE_LUMA_GAP);
        assert!(gap(Color32::WHITE, Color32::BLACK) > READABLE_LUMA_GAP);
    }

    /// Every failure is reported, not just the first.
    ///
    /// A gate that stops at the first failure turns a theme edit into a
    /// sequence of rebuilds. `check`'s contract says all of them; this is
    /// what holds it to that.
    #[test]
    fn check_reports_every_failing_pair_not_only_the_first() {
        let mut style = Style::default();
        // Make the whole thing white-on-white: all ten pairs must fail.
        for w in [
            &mut style.visuals.widgets.noninteractive,
            &mut style.visuals.widgets.inactive,
            &mut style.visuals.widgets.hovered,
            &mut style.visuals.widgets.active,
            &mut style.visuals.widgets.open,
        ] {
            w.fg_stroke = egui::Stroke::new(1.0, Color32::WHITE);
            w.bg_fill = Color32::WHITE;
            w.weak_bg_fill = Color32::WHITE;
        }
        let failures =
            check(&style, READABLE_LUMA_GAP).expect_err("white on white cannot be readable");
        assert_eq!(failures.len(), 10, "all ten pairs must be reported");
    }

    /// The failure message names the state, the fill and both colours.
    ///
    /// The message is the deliverable. A gate that says "contrast failed"
    /// has told the reader to go and re-derive what this function already
    /// knew.
    #[test]
    fn a_failure_names_the_line_to_change() {
        let f = ContrastFailure {
            state: WidgetState::Active,
            fill: FillKind::BgFill,
            fg: Color32::WHITE,
            bg: Color32::WHITE,
            gap: 0.0,
            threshold: READABLE_LUMA_GAP,
        };
        let text = f.to_string();
        assert!(text.contains("Active"), "{text}");
        assert!(text.contains("bg_fill"), "{text}");
        assert!(text.contains("needs 90"), "{text}");
    }
}
