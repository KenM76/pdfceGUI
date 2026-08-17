//! # `text::markup` — the words the Markup ▸ Style group shows
//!
//! Three tooltips and one suffix, which is the whole operator-visible surface
//! of `canvas::markup::swatch`. The controls themselves are a colour swatch and
//! a number: neither can carry a label without doubling the width of a ribbon
//! group, so **the tooltip is the only place they say what they are** — which
//! makes these strings load-bearing rather than supplementary.
//!
//! ## Each one answers "what will this change, and when?"
//!
//! Because that is the question a swatch in a ribbon cannot answer by looking
//! like a swatch. Every tooltip here says two things: which markup the setting
//! applies to, and — the half an operator is most likely to get wrong — that it
//! applies to the **next** one rather than to anything already on the page.
//!
//! `RIBBON_IA.md` §5.5 is explicit that these are two different surfaces:
//!
//! > The `Style` group sets defaults for the next markup. Changing an
//! > *existing* markup's style happens on the contextual **Format** tab.
//!
//! The Format tab's property editors are not built yet, so an operator who
//! recolours the swatch expecting the rectangle they just drew to change will
//! be disappointed — and the tooltip is the only thing standing between them
//! and concluding the control is broken. Saying "the next one" is therefore a
//! disclosure and not a nicety.

/// Hover text for the ink swatch.
#[must_use]
pub const fn pen_colour_tooltip() -> &'static str {
    "The colour of the next shape, arrow, line or freehand mark you draw. \
     Marks already on the page keep the colour they were drawn in."
}

/// Hover text for the highlighter swatch.
///
/// A separate control and a separate sentence, because they are separate pens
/// — see `canvas::markup::pen`'s header. An operator who sets the ink to green
/// does not thereby want a green highlight, and a tooltip that said "the
/// markup colour" for both would suggest they had.
#[must_use]
pub const fn highlighter_colour_tooltip() -> &'static str {
    "The colour of the next highlight band. Kept separate from the pen above, \
     so choosing a pen colour does not change your highlighter."
}

/// Hover text for the width control.
///
/// Names the **unit** as well as the effect, because "2" on a ribbon is a
/// number without a scale — and points are what the PDF stores, so it is also
/// the number the operator would see if they opened the file in another
/// program.
#[must_use]
pub const fn pen_width_tooltip() -> &'static str {
    "How thick the next mark's line is, in points — the same unit the document \
     itself uses. A drawing's own linework is often a quarter point, so 2 sits \
     clearly above it without covering it."
}

/// The width control's suffix.
///
/// A separate entry rather than a literal in the widget call, for the reason
/// the settings window's degree sign is: `check-ui-strings.sh` looks for
/// exactly this, and a translator localising the ribbon must be able to see
/// that a unit abbreviation exists.
#[must_use]
pub const fn width_suffix() -> &'static str {
    " pt"
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★ Every tooltip says the setting applies to the NEXT mark.
    ///
    /// The disclosure this module exists for. `RIBBON_IA.md` §5.5 puts
    /// "restyle what is already there" on the contextual Format tab, whose
    /// property editors are not built — so an operator who recolours the swatch
    /// expecting the rectangle they just drew to change has no other way to
    /// learn otherwise, and would reasonably report the control as broken.
    ///
    /// A test rather than a convention, because the natural edit when a tooltip
    /// reads long is to cut its second sentence.
    #[test]
    fn every_style_tooltip_says_it_applies_to_the_next_mark() {
        for tip in [
            pen_colour_tooltip(),
            highlighter_colour_tooltip(),
            pen_width_tooltip(),
        ] {
            assert!(
                tip.contains("next"),
                "a Style tooltip no longer says it applies to the next mark: {tip:?}"
            );
        }
    }

    /// The two colour tooltips are different sentences about different pens.
    ///
    /// They are two controls sitting side by side with no labels, so identical
    /// or near-identical hover text would make them indistinguishable — which
    /// is the state the operator is already in before they hover.
    #[test]
    fn the_two_swatches_are_told_apart_by_their_words() {
        assert_ne!(pen_colour_tooltip(), highlighter_colour_tooltip());
        assert!(highlighter_colour_tooltip().contains("highlight"));
    }

    /// The width suffix names a unit and is not empty.
    #[test]
    fn the_width_carries_its_unit() {
        assert!(width_suffix().contains("pt"));
        assert!(pen_width_tooltip().contains("points"));
    }
}
