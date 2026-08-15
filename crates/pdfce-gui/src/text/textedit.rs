//! # `text::textedit` — every sentence the text-editing tool shows
//!
//! Consumed by `crate::canvas::textedit` and by the `Action::CommitTextEdit`
//! apply arm. Split out of [`super`] rather than added to it for the reason the
//! catalog's header gives: it is split by **area** from the first commit, so the
//! split never has to be done as a migration.
//!
//! ## Two things here are load-bearing rather than cosmetic
//!
//! **[`spans_runs`] is a refusal, not a status message.** `DEFECTS.md` D4a
//! records that the old shell handled a cross-run selection by setting a flag
//! that *"silently disables the whole typing loop"* — the operator pressed keys
//! and nothing happened. A sentence is the entire difference between a limit and
//! a bug, and this is that sentence.
//!
//! **[`pinned_tail_disclosure`] is owed under rule 4.** When the follower
//! disposition is `Pin`, the text after the edit does not make room, so a longer
//! replacement grows into it. The engine discloses this for `Reflow` and not for
//! `Pin` — from its side, pinning is what was asked for — so the sentence has to
//! come from here or from nowhere. It names *which* rule pinned, because
//! "right-aligned" and "rotated" are different facts about the operator's
//! document and only one of them is something they chose.

use crate::canvas::textedit::Refusal;
use crate::canvas::textedit::disposition::Reason;
use pdfce_core::text_edit::BlockAlignment;

/// The sentence for a refusal to place a caret.
///
/// One function over the enum rather than one per variant, so a variant added to
/// [`Refusal`] is a compile error here instead of a caret that refuses silently.
#[must_use]
pub const fn refusal(reason: Refusal) -> &'static str {
    match reason {
        Refusal::NoRun => {
            "There is no text where you clicked. Click on a word to put the cursor in it, or use \
             Add text to place new text here."
        }
        Refusal::NoText => {
            "pdfce cannot read any text on this page. If it is a scan, run Tools > OCR first — \
             editing needs real text, not a picture of it."
        }
        // ★ D4a, in words, on the surface, where the old shell went quiet.
        Refusal::SpansRuns => spans_runs(),
    }
}

/// ★ The cross-run refusal — `DEFECTS.md` D4a, said out loud.
///
/// It names the limit, names the reason in the operator's terms rather than in
/// PDF's, and gives them the next move. It does **not** say "one text run",
/// because a run is not a thing anyone can see on a page; what they can see is
/// that the line is made of separate pieces.
#[must_use]
pub const fn spans_runs() -> &'static str {
    "This line is stored as several separate pieces of text, and pdfce edits one piece at a \
     time. Click directly on the word you want to change. Editing a whole paragraph at once is \
     not built yet."
}

/// The disclosure appended when the edit pinned the text after it.
///
/// Two sentences and no more, because it shares the status row with everything
/// else and R128 forbids that row growing. The first says what happened; the
/// second says what to watch for.
#[must_use]
pub fn pinned_tail_disclosure(reason: Reason) -> String {
    let because = match reason {
        Reason::Rotated => {
            "this text is rotated, so moving what follows it sideways would move it the wrong way"
        }
        Reason::Flush(BlockAlignment::Right) => "this text is right-aligned",
        Reason::Flush(BlockAlignment::Center) => "this text is centred",
        Reason::Flush(BlockAlignment::Justified) => "this text is justified",
        // `BlockAlignment` is `#[non_exhaustive]`, so a wildcard is required
        // rather than optional. It answers with the general form of the same
        // fact, which is true of every alignment that is not Left.
        Reason::Flush(_) => "the text after this one is lined up against something",
        // Unreachable — neither of these pins — and answered rather than
        // panicked, because a disclosure is not worth a crash in the frame that
        // is trying to draw. See `Reason::pins_the_tail`, which is the predicate
        // the caller gates on.
        Reason::LeftAligned | Reason::AlignmentUndetectable => {
            "the text after this one was kept \
                                                               in place"
        }
    };
    format!(
        "layout: the text after your edit was left exactly where it was, because {because}. If \
         what you typed is longer than what it replaced, it may now overlap — check the page \
         before saving."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★ **Every refusal has a sentence, and none of them is empty.**
    ///
    /// The whole point of the module: the old shell's answer to the cross-run
    /// case was no sentence at all.
    #[test]
    fn every_refusal_says_something() {
        for r in [Refusal::NoRun, Refusal::NoText, Refusal::SpansRuns] {
            let s = refusal(r);
            assert!(s.len() > 40, "{r:?} needs a real sentence, got {s:?}");
            assert!(
                s.ends_with('.'),
                "{r:?} is prose and prose is punctuated: {s:?}"
            );
        }
    }

    /// **The cross-run refusal names the next move.** A refusal that does not
    /// say what to do instead is a shrug with a capital letter — the catalog
    /// header's own rule.
    #[test]
    fn the_cross_run_refusal_tells_the_operator_what_to_do_instead() {
        let s = spans_runs();
        assert!(s.contains("Click directly on the word"));
        // …and it does not use the word the engine uses, which names nothing
        // the operator can see on their page.
        assert!(!s.contains("run"), "'run' is a PDF term, not an operator's");
    }

    /// ★ **Each pinning reason gets its own explanation.**
    ///
    /// A single generic sentence would be the cheaper implementation and would
    /// be wrong for both cases: "right-aligned" is something the operator's
    /// document is, and "rotated" is something they can see, and the remedy
    /// differs.
    #[test]
    fn each_pinning_reason_explains_itself_differently() {
        let rotated = pinned_tail_disclosure(Reason::Rotated);
        let right = pinned_tail_disclosure(Reason::Flush(BlockAlignment::Right));
        let centre = pinned_tail_disclosure(Reason::Flush(BlockAlignment::Center));
        assert!(rotated.contains("rotated"));
        assert!(right.contains("right-aligned"));
        assert!(centre.contains("centred"));
        assert_ne!(rotated, right);
        assert_ne!(right, centre);
    }

    /// **The disclosure warns about the cost it exists to disclose.** Without
    /// the overlap sentence this would be a note about an internal choice
    /// rather than a warning the operator can act on.
    #[test]
    fn the_disclosure_names_the_cost_and_not_just_the_choice() {
        let s = pinned_tail_disclosure(Reason::Flush(BlockAlignment::Right));
        assert!(s.contains("overlap"), "the cost of a pin is an overlap");
        assert!(
            s.contains("before saving"),
            "and there is a moment to check"
        );
    }
}
