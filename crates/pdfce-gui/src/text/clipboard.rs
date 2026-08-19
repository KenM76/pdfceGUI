//! # `text::clipboard` — the four sentences the object clipboard can say
//!
//! Each is a **refusal on the status row**, and each exists because the
//! alternative is a keystroke that does nothing and says nothing. That is how
//! the operator experienced the absence of cut, copy and paste in the first
//! place — *"the standard copy/paste … aren't implemented"* — and a build that
//! implemented them and stayed silent when it could not act would read
//! identically.
//!
//! ## ★ Why two of the four name the ENGINE and two name the selection
//!
//! Because they are different kinds of "no" and an operator's next move differs:
//!
//! - *nothing selected* / *nothing copied* — **select something, or copy
//!   something.** The operator's own next act fixes it.
//! - *a path is selected* — **nothing the operator can do fixes it today.**
//!   `EditSession` has no verb that puts page content back on a page, so there
//!   is no sequence of clicks that would make the copy work.
//!
//! The second kind has to say so, or the operator spends the afternoon trying
//! different objects. `NO_SURFACE.md` §1c's rule about dated citations applies
//! to the sentence as much as to the code comment: it says *what pdfce cannot
//! do*, not *that something went wrong*.

use crate::canvas::clipboard::Refusal;

/// The sentence for a refusal.
#[must_use]
pub const fn refusal(reason: Refusal) -> &'static str {
    match reason {
        Refusal::NothingSelected => {
            "Nothing is selected. Click a comment or a markup shape on the page first."
        }
        // ★ Names the boundary and does not apologise for it. An operator who
        // reads "pdfce cannot copy page content yet" stops trying; one who reads
        // "copy failed" tries four more shapes.
        Refusal::ContentNotAnnotation => {
            "That is page content — a line, a shape or a piece of text. pdfce can copy comments \
             and markup, but it cannot yet put page content back onto a page, so copying one \
             would offer a paste that could never happen."
        }
        Refusal::Unreadable => {
            "That annotation is not one pdfce authors — a link, a form field or an attachment — \
             so there is nothing for it to copy."
        }
        Refusal::NothingCopied => {
            "Nothing has been copied yet. Select a comment or markup and \
                                   press Ctrl+C first."
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★ **Every refusal says what to do next, or why there is nothing to do.**
    ///
    /// Asserted as a length floor rather than by matching words, because the
    /// property is *"this is a sentence, not a label"*. A four-word refusal is
    /// the failure this whole module exists to prevent, and it is the shape a
    /// future edit would most plausibly introduce while "tidying".
    #[test]
    fn every_refusal_is_a_sentence() {
        for reason in [
            Refusal::NothingSelected,
            Refusal::ContentNotAnnotation,
            Refusal::Unreadable,
            Refusal::NothingCopied,
        ] {
            let s = refusal(reason);
            assert!(
                s.len() > 40,
                "{reason:?} is too short to be an explanation: {s:?}"
            );
            assert!(s.ends_with('.'), "{reason:?} must be a sentence: {s:?}");
        }
    }
}
