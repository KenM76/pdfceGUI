//! # `text::fieldclip` — the sentences the FORM-FIELD clipboard can say
//!
//! Two families, and they are different kinds of statement:
//!
//! 1. **[`refusal`] — why nothing happened.** Same posture as
//!    `text::clipboard`: a keystroke that does nothing and says nothing is
//!    indistinguishable from a broken keyboard.
//! 2. **[`loss_note`] — what happened, and what did not come along.** This one
//!    is not a refusal. The paste *worked*; the sentence exists because part of
//!    the source field could not travel and **the operator cannot see which
//!    part**.
//!
//! ## ★★★ Why the loss note is a sentence and not a mark on the canvas
//!
//! Rule 4, both halves, and they pull in opposite directions here.
//!
//! The pasted field **renders exactly as a saved-and-reopened one would**. No
//! badge, no tint, no dashed outline, no "incomplete copy" layer. The
//! operator's own ruling: *"the nagging and red flagging in the original GUI
//! made for a lot of extra bugs in the visibility when editing."* Provisional
//! styling is a second rendering path for the same content and two paths drift.
//!
//! And the half that survives is precisely this case: a field that lost its
//! **calculation script** looks identical to one that kept it. A screenshot
//! cannot show the difference; the file behaves differently. So it is reported
//! — off-canvas, on the status row, not blocking, not positioned relative to
//! the document.
//!
//! ⇒ *Render normally; report separately.* **Both.**
//!
//! ## Why the sentence names the property and not the PDF key
//!
//! *"the font and colour"*, not *"`/DA`"*. The operator is a draughtsman, not a
//! spec reader, and a sentence he has to look up is a sentence he skips. The
//! keys live in the doc comments on [`crate::canvas::fieldclip::Lost`], where a
//! future engineer needs them and he does not.

use crate::canvas::fieldclip::{Lost, Refusal};

/// The sentence for a refusal.
#[must_use]
pub const fn refusal(reason: Refusal) -> &'static str {
    match reason {
        Refusal::NothingSelected => "No form field is selected. Click a field on the page first.",
        // ★ Not "an error occurred". The document changed underneath the
        // selection — an undo, a deletion from the Forms panel — and the
        // operator's next act is to click the field again, so the sentence
        // says that rather than describing the internal state.
        Refusal::Vanished => {
            "That field is no longer in the document. Click a field on the page again."
        }
        Refusal::NoGeometry => {
            "That field has no box on the page, so there is nothing to copy. \
             Fields like this are reached from the Forms panel."
        }
        // ★★ A dated citation, in the operator's terms, in the shape
        // `NO_SURFACE.md` §1c asks for: it says what pdfce cannot do rather
        // than that something went wrong, so he stops trying instead of
        // trying four more signature fields.
        Refusal::KindCannotBeAuthored => {
            "Signature fields cannot be copied. pdfce can create text boxes, \
             check boxes, radio buttons, drop-downs and buttons, and a signature \
             field is not one of them."
        }
        // ★★★ The engine's own reasoning, in his terms. Two radio buttons in
        // one group must have different export values -- that is what makes
        // them alternatives rather than the same answer twice -- and pdfce will
        // not invent the second one. Ctrl+V is offered in the same breath
        // because it works and is almost certainly what was wanted.
        Refusal::RadioNeedsItsOwnExportValue => {
            "A radio button needs its own export value, so it cannot share a \
             field with the one you copied. Press Ctrl+V to paste it as a new \
             group instead."
        }
        Refusal::NothingCopied => "Nothing has been copied. Select a field and press Ctrl+C.",
    }
}

/// **What a paste did NOT carry** — the status-row sentence, or `None`.
///
/// # ★★★ Why this does not mention the merge, and used to
///
/// The first version took a `merged: bool` and opened with *"Pasted as another
/// box for the same field — typing in either one fills both."* A driven run on
/// 2026-08-29 showed the engine saying the same thing one line later, in its own
/// `FieldAuthorOutcome` disclosure, which `actions::forms::author` already puts
/// on the status row:
///
/// > *"That name already existed, so this control shows the same value as the
/// > other one — typing in either changes both. Give it a different name if you
/// > wanted two separate fields."*
///
/// Two sentences for one fact, in two wordings, from two files that will drift.
/// This shell's standing rule is the opposite — *one fact, one wording* — and
/// the engine's version is the better one anyway: it is authoritative (it
/// reports what the merge **did**, not what the shell **intended**) and it ends
/// with the remedy.
///
/// ⇒ So this function says only what the engine **cannot**: which properties the
/// re-authoring left behind. It returns `None` when there is nothing to add,
/// which is the duplicate paste's case — and the silence is correct, because the
/// engine's own line has already spoken.
///
/// ★ A `NewField` paste always has something to say, because
/// [`Lost::BorderColour`] is unconditional. That is not an accident of this
/// function; it is the authoring path hard-writing a black `/MK /BC`.
#[must_use]
pub fn loss_note(lost: &[Lost]) -> Option<String> {
    if lost.is_empty() {
        return None;
    }
    let parts: Vec<&str> = lost.iter().map(|l| property(*l)).collect();
    Some(format!(
        "Pasted as a new field. Not carried over: {}.",
        join_and(&parts)
    ))
}

/// One property, named the way an operator would name it.
#[must_use]
const fn property(lost: Lost) -> &'static str {
    match lost {
        Lost::Appearance => "the font, its size and its colour",
        Lost::Alignment => "the text alignment",
        Lost::DefaultValue => "the default value",
        // ★ Spelled out rather than called "actions", because "actions" is a
        // PDF word and "calculation" is what a person put in the field.
        Lost::Actions => "any calculation or formatting script",
        Lost::BorderColour => "the border and background colours",
    }
}

/// **What a field copy leaves on the OPERATING SYSTEM's clipboard.**
///
/// ★★★ This exists because of a toolkit constraint, not a design wish, and
/// without it `Ctrl+V` does not work at all. `egui-winit-0.35.0` synthesises
/// `Event::Paste` **only when the OS clipboard holds non-empty text**, and
/// swallows the keystroke entirely otherwise — no key event, no paste event,
/// nothing. So a paste of something pdfce holds in its own memory would depend
/// on whether the operator had recently copied text in some other application.
///
/// ⇒ **It was found by driving, not by reading**, on 2026-08-29: the whole
/// field clipboard was written, unit-tested, gate-clean and shipped-looking, and
/// the first driven run reported `fieldclip-copy` present and `fieldclip-paste`
/// absent with nothing between them. The RAG entry
/// `egui_winit_swallows_ctrl_c_x_v_so_a_keymap_binding_for_them_is_dead_on_arrival.md`
/// predicted exactly this and it still happened, because the field path is a
/// *new* copy path and the marker lives at each copy site rather than in one
/// place. That is the finding worth keeping: **a documented platform trap does
/// not protect a code path that did not exist when it was documented.**
///
/// # The wording
///
/// For a human who pastes into a text editor and wonders what they got. It names
/// the field, because a form has many and *"a form field"* would not say which.
/// It names both chords, because the second one is the whole feature and an
/// operator who reads this sentence in an email has just been taught it.
#[must_use]
pub fn os_marker(field: &str) -> String {
    format!(
        "The form field “{field}” was copied from pdfce. Paste it back into pdfce \
         with Ctrl+V for a new field, or Ctrl+Shift+V for another box that fills with \
         the same value."
    )
}

/// **A candidate name for a pasted field** — `Text` + `2` -> `Text2`.
///
/// # ★★★ NO SEPARATOR, and above all NO DOT
///
/// Corrected 2026-08-29 from `Drawn By 2` (a space) after reading the Acrobat
/// reference `forms__field_copy_paste_and_duplication.md`. Two things it
/// settles, and the second is a correctness matter rather than a taste one.
///
/// **1. The convention is a plain numeric suffix.** Acrobat's bulk duplication
/// ("Create Multiple Copies") auto-names its copies `Date1`, `Date2`, `Date3`,
/// and the sourced rationale is explicitly about scripting: the suffix exists so
/// a script can loop over every field sharing *"the non-number part of the field
/// name"*. **A space breaks exactly that property** — the non-number part of
/// `Drawn By 2` is `Drawn By ` with a trailing space, which no author would
/// write and every string comparison would trip over. So the separator is not a
/// house style to pick; it is load-bearing, and the convention has a reason.
///
/// **2. A DOT WOULD BE A STRUCTURAL CHANGE, NOT A COSMETIC ONE.** The other
/// sourced account of Acrobat's auto-naming describes dot-child notation —
/// `Text.0`, `Text.1` — and the RAG flags the two accounts as contested. It does
/// not matter which is right, because pdfce must not use the dot either way:
/// **`.` is the fully-qualified-name separator** (§12.7.3.2), so `Drawn By.2`
/// is not a field called *"Drawn By.2"*, it is a **child field named `2` under a
/// parent named `Drawn By`**. That is a third shape — a shared ancestor node
/// with independent terminal children — and it is neither of the two this
/// shell's two chords are for. Adopting it would silently give `Ctrl+V` a
/// hierarchy nobody asked for.
///
/// ⇒ The one place a reference implementation's own convention must be refused
/// is where the format assigns the character a meaning.
///
/// # Why the catalog and not the call site
///
/// A field name is shown to the operator in three places — the Forms panel, the
/// tab-order list and the Properties header — so it is operator-facing text and
/// `check-ui-strings` was right to insist. The numbering itself is *logic* and
/// lives with the caller; only the spelling is here.
///
/// The name is a placeholder the operator is expected to change, which is why a
/// paste generates one rather than opening a dialog. Four boxes down a column is
/// four keystrokes, not four interruptions.
#[must_use]
pub fn candidate_name(stem: &str, n: u32) -> String {
    format!("{stem}{n}")
}

/// `a`, `a and b`, `a, b and c` — the serial comma omitted, matching the rest
/// of this shell's prose.
///
/// Written here rather than reached for from a crate because it is four lines
/// and because the alternative bends the sentence to fit a generic joiner.
fn join_and(parts: &[&str]) -> String {
    match parts {
        [] => String::new(),
        [one] => (*one).to_owned(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★★★ A duplicate says NOTHING here, and the silence is the assertion.
    ///
    /// It is not that a duplicate has no news — it has the most surprising news
    /// of the two chords. It is that the ENGINE reports it, in its own words,
    /// through `FieldAuthorOutcome`, and a second sentence from this file would
    /// be the same fact in two wordings that drift. Falsified by driving on
    /// 2026-08-29, where both sentences reached the status row.
    #[test]
    fn a_duplicate_adds_no_sentence_because_the_engine_already_spoke() {
        assert_eq!(
            loss_note(&[]),
            None,
            "an empty loss list must add nothing to the status row: the engine's merge disclosure is authoritative and this file must not restate it"
        );
    }

    /// A list of three reads as prose, not as a debug dump.
    #[test]
    fn several_losses_join_into_a_sentence() {
        let s = loss_note(&[Lost::Appearance, Lost::Actions, Lost::BorderColour])
            .expect("a new-field paste always has at least the border colour to report");
        // Three separate substrings rather than one long one: a single literal
        // spanning source lines is at rustfmt's mercy, and the first version of
        // this test failed because the formatter reflowed the EXPECTATION rather
        // than the program. The three claims are the ones that matter — the
        // Oxford-comma-free join, and that each property is present.
        assert!(s.contains("the font, its size and its colour,"), "got: {s}");
        assert!(
            s.contains("any calculation or formatting script and"),
            "the last two must join with `and` and no serial comma. Got: {s}"
        );
        assert!(
            s.ends_with("the border and background colours."),
            "got: {s}"
        );
        assert!(
            !s.contains("/DA") && !s.contains("Lost::"),
            "no PDF keys and no Rust identifiers in operator-facing prose. Got: {s}"
        );
    }

    /// One loss takes no joiner.
    #[test]
    fn a_single_loss_does_not_grow_a_stray_and() {
        let s = loss_note(&[Lost::Alignment]).expect("one loss is still a sentence");
        assert!(
            s.ends_with("Not carried over: the text alignment."),
            "got: {s}"
        );
    }
}
