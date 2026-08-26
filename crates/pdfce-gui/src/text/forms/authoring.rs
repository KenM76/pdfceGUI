//! # `text::forms::authoring` — the words for **making** a form field
//!
//! [`super`] covers **filling** an `/AcroForm` that already exists. This file
//! covers the opposite direction, added 2026-08-26 on the operator's request to
//! place form controls from the ribbon: the five kinds' nouns, the placement
//! dialog's labels, and the disclosures the engine's `FieldAuthorOutcome`
//! obliges.
//!
//! Its own file rather than more of `super`, for the reason `super`'s header
//! gives about itself: *"the reviewer of a disclosure sentence is reading a file
//! that contains nothing but disclosure sentences"*. It also keeps both files
//! comfortably inside R2 — `super` is already at 1,265 lines.
//!
//! ## ★★★ The four disclosures, and why they are status lines
//!
//! `FieldAuthorOutcome` reports four things about a field that has just been
//! authored, and **not one of them is visible on the rendered page**. That is
//! the exact condition rule 4's surviving half describes: an inference the
//! operator cannot see still owes an off-canvas report. Decision 059 settled
//! *where* — the status line, never a mark on the canvas — so a screenshot of
//! the page with a merged field and a screenshot with an independent one are
//! identical, which is correct, and the status bar is what tells them apart.
//!
//! The sharpest is [`form_field_merged`]. A name that matches an existing field
//! does not make a second field; it makes a second **view** of the first, so
//! typing in one changes the other. An operator who meant to place two
//! independent boxes has placed one, and nothing about the page says so.
//!
//! ## What is deliberately NOT here
//!
//! The field-name stems (`Text`, `Check Box`, `Group`, …). Those are `/T`
//! strings written into the file, are what a form-filling script and an FDF
//! import key on, and translating them would rename every field for an operator
//! running a different language — invisibly, until the import failed. They live
//! on `FormFieldKind::name_prefix` as literals with that reasoning attached.

/// The noun for a text field, in a sentence.
#[must_use]
pub fn form_noun_text() -> String {
    "Text field".to_owned()
}

/// The noun for a check box, in a sentence.
#[must_use]
pub fn form_noun_check_box() -> String {
    "Check box".to_owned()
}

/// The noun for a radio button, in a sentence.
#[must_use]
pub fn form_noun_radio() -> String {
    "Radio button".to_owned()
}

/// The noun for a drop-down or list box, in a sentence.
///
/// ★ "Drop-down list" rather than "choice field", which is the PDF spec's word
/// (`/Ch`) and means nothing to anyone who has not read it. The operator's
/// standing tie-breaker — *make it work the way other programs do* — applies to
/// vocabulary as much as to behaviour, and every program calls this a drop-down.
#[must_use]
pub fn form_noun_choice() -> String {
    "Drop-down list".to_owned()
}

/// The noun for a push button, in a sentence.
#[must_use]
pub fn form_noun_push_button() -> String {
    "Button".to_owned()
}

/// **The field was authored.** The one line every placement produces.
///
/// It names the kind rather than saying "field added", because five commands
/// place five different things and a generic confirmation cannot tell an
/// operator that the button they pressed was not the one they meant.
#[must_use]
pub fn form_field_added(noun: &str) -> String {
    format!("{noun} added.")
}

/// ★★★ **The name matched an existing field, so this widget joined it.**
///
/// The single most important sentence in this file, and the one with the least
/// visible cause. In PDF a fully-qualified name *is* the field's identity: two
/// widgets carrying the same one are one field with two appearances on the
/// page, and typing into either changes both.
///
/// The page looks exactly as it would if they were independent. So this is
/// stated plainly, with what it means rather than with the word "merged" —
/// which is the engine's word and describes the mechanism, not the consequence.
#[must_use]
pub fn form_field_merged() -> String {
    "That name already existed, so this control shows the same value as the \
     other one — typing in either changes both. Give it a different name if you \
     wanted two separate fields."
        .to_owned()
}

/// **No tooltip was given**, and what that costs.
///
/// Not a scolding and not a warning: leaving it blank is a legitimate decision
/// and the engine accepts it as one. What the operator may not know is the
/// consequence, which is entirely invisible on screen — a screen reader has
/// nothing to announce for this control but its type.
#[must_use]
pub fn form_field_no_tooltip() -> String {
    "It has no tooltip, so a screen reader will announce only what kind of \
     control it is."
        .to_owned()
}

/// **A drop-down with no options in it.**
///
/// Authorable, and empty. Worth saying because an empty list renders as a
/// control that opens and shows nothing, which reads as a broken field rather
/// than an unfinished one.
#[must_use]
pub fn form_field_no_options() -> String {
    "It has no options yet, so it will open empty.".to_owned()
}

/// **The document is tagged, and this control is not in the tag tree.**
///
/// Covers both `tagged_document` and `structure_tab_order` in one sentence,
/// deliberately: they are two symptoms of one situation, and an operator who
/// gets two lines about the same thing reads the second as a separate problem.
///
/// ★ It says what is true rather than what to do, because pdfce cannot yet fix
/// it and a line that recommended an action it does not offer would be worse
/// than one that reports a fact.
#[must_use]
pub fn form_field_tagged_document() -> String {
    "This document is tagged for accessibility, and the new control is not in \
     its structure tree — its reading order will not include this field."
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Every noun is distinct**, because the confirmation line is the only
    /// place the operator learns which of five buttons they actually pressed.
    #[test]
    fn the_five_nouns_are_distinct() {
        let nouns = [
            form_noun_text(),
            form_noun_check_box(),
            form_noun_radio(),
            form_noun_choice(),
            form_noun_push_button(),
        ];
        for (i, a) in nouns.iter().enumerate() {
            for b in nouns.iter().skip(i + 1) {
                assert_ne!(a, b);
            }
        }
    }

    /// ★★ **The merge disclosure explains the consequence, not the mechanism.**
    ///
    /// Asserted rather than left to review because the tempting rewrite — "the
    /// field was merged with an existing one" — is shorter, is what the engine
    /// calls it, and tells an operator nothing about what will happen when they
    /// type. This test fails if the sentence stops saying that both change.
    #[test]
    fn the_merge_disclosure_says_what_it_means_for_the_operator() {
        let line = form_field_merged();
        assert!(
            line.contains("changes both"),
            "the consequence must be stated, not just the fact of merging: {line}"
        );
        assert!(
            !line.contains("merged"),
            "\u{201c}merged\u{201d} is the engine's word for the mechanism: {line}"
        );
    }

    /// **Every disclosure is one sentence an operator could act on or ignore**
    /// — none of them is empty, and none runs past a status line's width.
    #[test]
    fn the_disclosures_are_stated_and_bounded() {
        for line in [
            form_field_merged(),
            form_field_no_tooltip(),
            form_field_no_options(),
            form_field_tagged_document(),
        ] {
            assert!(!line.trim().is_empty());
            assert!(line.len() < 240, "too long for a status line: {line}");
        }
    }
}
