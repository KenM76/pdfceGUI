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

/// Hover text for the opacity control.
///
/// # ★★★ Why this sentence names the CAD case rather than describing the slider
///
/// Because the reason to reach for it is specific and is not obvious from a
/// percentage: a comment sits on top of the thing it is about, and on a dense
/// drawing an opaque cloud hides the dimension it is drawing attention to. An
/// operator who has never used annotation transparency has no reason to guess
/// that, and a tooltip reading *"the opacity of the next mark"* would restate
/// the label.
///
/// # ★ It says the mark stays selectable, because faint is not gone
///
/// The bottom of the range is a tenth, deliberately (`canvas::markup::pen`'s
/// `MIN_OPACITY` carries the argument), and at a tenth over dark linework a
/// mark can be hard to find with the eye. Saying it is still there and still
/// listed is the disclosure that stops a faint mark reading as a failed one.
#[must_use]
pub const fn pen_opacity_tooltip() -> &'static str {
    "How much of the drawing shows through the next mark. Below 100% the mark \
     is see-through, which is what lets a cloud or a box sit over a dimension \
     without hiding it. Even the faintest mark is still selectable and still \
     listed in the Comments panel."
}

/// The opacity control's suffix.
///
/// A percent sign, because opacity is the one property in this group an
/// operator already thinks about as a percentage — every other program that
/// offers it says 40%, not 0.4. The value written into `/CA` is the fraction;
/// the conversion happens at the control and nowhere else.
#[must_use]
pub const fn opacity_suffix() -> &'static str {
    "%"
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

/// Disclosure: the border width did not scale with the shape.
///
/// ★★★ The engine asks for this sentence by name — *"an operator who scaled a
/// square 3× and expected a heavier border needs telling it stayed"* — and it is
/// Rule 4's surviving half in its purest form: the shape grew around the border
/// and **nothing on the canvas says the border did not grow with it**.
///
/// ★★ It states the default as a **choice**, not as a limitation, because it is
/// one: on a CAD drawing a line weight is a drafting standard rather than
/// decoration, which is this project's own argument and the one the engine
/// promoted into the rule that decides every future case — *is the property a
/// length in the space being transformed?* An inset is; a line weight is not.
#[must_use]
pub fn stroke_width_unchanged() -> String {
    "The outline's thickness has not changed. pdfce treats a line weight as a drawing standard \
     rather than something that scales with the shape."
        .to_owned()
}

/// Disclosure: a foreign appearance was scaled unevenly and its stroke is now
/// anisotropic.
///
/// ★★★ Not a defect and not pdfce's choice — an arithmetic limit. **Neither PDF
/// nor SVG has a per-axis stroke width**: both are scalars, so a stroke drawn
/// through a matrix applied *after* stroking cannot keep an even thickness under
/// a non-uniform scale. Inkscape closed the identical report **Invalid** and
/// silently produces the distorted stroke.
///
/// ★★ pdfce says so instead, which is the whole difference. The operator can see
/// the result — a border thicker on one axis — and cannot see *why*, so the
/// sentence names the cause and the remedy: drag a corner with Shift held, or
/// accept it.
#[must_use]
pub fn appearance_distorted() -> String {
    "This markup was drawn by another program, and scaling it unevenly has made its outline \
     thicker on one side than the other. Hold Shift while dragging a corner to scale it evenly."
        .to_owned()
}

/// **Disclosure: the words this note used to carry, on the case where a save
/// overwrote them.**
///
/// ★★★ Rule 4's surviving half, and `pdfce-core` commissioned this sentence
/// itself: *"those words are gone from the document and nothing on the page
/// shows that they were ever there"*. A shape does not change when its note
/// does. A sticky's words live in a pop-up window this shell does not draw. So
/// on every subtype an operator can comment on, overwriting a note is an edit
/// with **no visible consequence at all** — which is precisely the class this
/// project's disclosure rule exists for.
///
/// ★★ It carries **the text, not a count**, because the engine chose to return
/// the text and said why: a count lets a shell *mention* the loss, and the text
/// lets it *offer the words back*. They are on the status line for as long as
/// the edit epoch holds, so an operator who overwrote the wrong comment can
/// read what was there and retype it — `Ctrl+Z` restores it outright, and this
/// is the surface that tells them there is something to undo.
///
/// ★ `None` when the annotation had no note, which is the ordinary case for
/// every shape this shell draws: a disclosure that fires on every save is one
/// nobody reads by the third time. Same rule as [`deleted_collateral`].
///
/// # The truncation, and why it is not a formatting decision
///
/// A `/Contents` may legitimately be a paragraph. The status line is one
/// bounded row that elides rather than wraps (`DEFECTS.md` R128), so a long
/// previous note would be cut by the *layout* with no indication that it had
/// been. Cutting it here, with an ellipsis and a stated character count, is the
/// difference between an operator seeing all of a short note and believing they
/// have seen all of a long one.
#[must_use]
pub fn note_replaced(previous: &str) -> Option<String> {
    let previous = previous.trim();
    if previous.is_empty() {
        return None;
    }
    let chars = previous.chars().count();
    const KEEP: usize = 120;
    if chars > KEEP {
        let head: String = previous.chars().take(KEEP).collect();
        return Some(format!(
            "The note that was there has been replaced. It began “{head}…” and ran to {chars} \
             characters. Ctrl+Z restores it."
        ));
    }
    Some(format!(
        "The note that was there has been replaced: “{previous}”. Ctrl+Z restores it."
    ))
}

/// **Disclosure: a note was removed, and what it said.**
///
/// The same argument as [`note_replaced`] at its strongest — a removal leaves
/// the markup on the page looking exactly as it did — so this fires even for a
/// short note and never returns `None` for a note that had words.
///
/// ★ It says the markup itself stayed, because that is the thing an operator
/// pressing a button labelled *Remove note* most reasonably fears they have
/// just done, and the canvas cannot answer it: a shape with a note and the same
/// shape without one are the same picture.
#[must_use]
pub fn note_removed(previous: &str) -> Option<String> {
    let previous = previous.trim();
    if previous.is_empty() {
        return None;
    }
    let chars = previous.chars().count();
    const KEEP: usize = 120;
    let words = if chars > KEEP {
        let head: String = previous.chars().take(KEEP).collect();
        format!("“{head}…”, {chars} characters")
    } else {
        format!("“{previous}”")
    };
    Some(format!(
        "The note has been removed — it said {words}. The markup itself is still on the page, and \
         Ctrl+Z restores the words."
    ))
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
