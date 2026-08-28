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

// ---------------------------------------------------------------------------
// Deleting a selected annotation
// ---------------------------------------------------------------------------

/// ★ **What went with it** — the collateral of deleting one annotation.
///
/// # Why a deletion needs to say anything at all
///
/// Because the operator named **one** annotation and the engine may legitimately
/// remove or alter more. `AnnotationDeletion` reports three such cases and each
/// is a fact about the file rather than about pdfce:
///
/// * a `/Popup` companion goes with its parent — §12.5.6.14 is a `shall`, a
///   pop-up *"shall not appear alone but is associated with a markup
///   annotation"*, so leaving it would be a clause violation. The spec
///   requiring it is a reason, **not a licence to stay quiet**;
/// * replies hanging off it as `/IRT` targets are **orphaned**, not deleted —
///   the thread survives and its root does not;
/// * group members are **promoted** when the group's primary goes.
///
/// Rule 4, in its second clause: pdfce did something the operator did not ask
/// for, so pdfce says so, off-canvas, in words.
///
/// # ★ What this deliberately does NOT say
///
/// **That the content is gone from the file.** It is not: deleting an
/// annotation removes an entry from `/Annots` and does not touch page content,
/// and the previous revision is still in the file after an incremental save.
/// `docs/core-api/03-capabilities.md` §3.4 states the rule this observes —
/// *"delete is not redaction"* — and the redaction surface is where that
/// distinction is made loudly. Saying "removed" here would be the exact wording
/// `crate::text::redact`'s header forbids.
///
/// Returns `None` when nothing but the named annotation was affected, which is
/// the ordinary case: a disclosure that fires on every delete is one nobody
/// reads by the third time.
#[must_use]
pub fn deleted_collateral(
    popup_removed: bool,
    parent_popup_cleared: bool,
    replies_orphaned: usize,
    group_members_promoted: usize,
) -> Option<String> {
    let mut parts: Vec<String> = Vec::new();
    if popup_removed {
        parts.push("its pop-up note went with it, which the PDF specification requires".to_owned());
    }
    if parent_popup_cleared {
        parts.push("the annotation it belonged to no longer refers to it".to_owned());
    }
    if replies_orphaned == 1 {
        parts.push("1 reply is left without the comment it replied to".to_owned());
    } else if replies_orphaned > 1 {
        parts.push(format!(
            "{replies_orphaned} replies are left without the comment they replied to"
        ));
    }
    if group_members_promoted == 1 {
        parts.push("1 grouped annotation is now on its own".to_owned());
    } else if group_members_promoted > 1 {
        parts.push(format!(
            "{group_members_promoted} grouped annotations are now on their own"
        ));
    }
    if parts.is_empty() {
        return None;
    }
    Some(format!("Deleted — {}.", parts.join("; ")))
}

/// Disclosure: the annotation moved and its pop-up note did not.
///
/// ★★★ **The one consequence of a move that this program cannot show.** §12.5.6.14
/// makes a pop-up a separate annotation with its own placement and leaves to the
/// reader whether it follows; `pdfce-core` reports the object it left behind and
/// says the decision is the shell's.
///
/// This shell does not draw pop-ups, so a stranded one is invisible here and
/// perfectly visible in Acrobat — which is Rule 4's surviving half in its
/// purest form: render normally, report separately, both.
///
/// ★ It says pdfce did **not** move it, rather than offering to. Moving it
/// would be a second undo entry for something the operator cannot see, and a
/// gesture that produces two entries is one `Ctrl+Z` away from a state nobody
/// can explain.
#[must_use]
pub fn popup_left_behind() -> String {
    "The note attached to this markup stayed where it was. pdfce does not show those \
     notes, so you will only see it in a reader that does."
        .to_owned()
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
