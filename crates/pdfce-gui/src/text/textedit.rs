//! # `text::textedit` — every sentence the text-editing tool shows
//!
//! Consumed by `crate::canvas::textedit` and by the `Action::CommitTextEdit`
//! apply arm. Split out of [`super`] rather than added to it for the reason the
//! catalog's header gives: it is split by **area** from the first commit, so the
//! split never has to be done as a migration.
//!
//! ## Two things here are load-bearing rather than cosmetic
//!
//! **★★ [`shares_the_line_note`] is a DISCLOSURE, and it was a refusal until
//! 2026-08-19.** `DEFECTS.md` D4a records that the old shell handled a
//! cross-run selection by setting a flag that *"silently disables the whole
//! typing loop"* — the operator pressed keys and nothing happened. This shell
//! replaced the silence with a sentence, which was the right first move and the
//! wrong final one: **it still refused**, and on a CAD sheet, where a table row
//! is one show operator per cell, it refused nearly every click. The operator
//! reported text editing as not working twice, weeks apart, and was right both
//! times. The refusal is gone; the sentence stayed and changed tense.
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
    }
}

/// ★★ The multi-run **disclosure** — what `spans_runs()` used to refuse.
///
/// Until 2026-08-19 this sentence's ancestor was a *refusal*: a click whose
/// visual line was made of more than one show operator placed no caret at all,
/// and the sentence told the operator to *"click directly on the word you want
/// to change"* — advice that could not work, because the refusal was about the
/// **line**, not about where on it they clicked.
///
/// On a SolidWorks sheet — one show operator per table cell, one per title-block
/// field — that refused nearly every click. The operator reported the feature as
/// not working twice, weeks apart, and **he was right both times**.
///
/// The refusal is gone and **the disclosure is the half that was always
/// useful**. It says the same true thing in the same operator's terms — *a run
/// is not a thing anyone can see on a page; what they can see is that the line
/// is made of separate pieces* — and then says what pdfce is going to do about
/// it instead of stopping.
///
/// Shown when the caret **lands**, not when the edit commits: rule 4's
/// *"announced before it is picked, not after"*, applied to a layout
/// consequence rather than to a geometric inference. The commit-time half is
/// [`pinned_tail_disclosure`], which says the same fact in the past tense.
#[must_use]
pub const fn shares_the_line_note() -> &'static str {
    "This line is drawn as several separate pieces. You are editing the piece you clicked; the \
     pieces beside it will stay exactly where they are, so a longer replacement may overlap \
     them."
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
        // ★ The commonest reason on this operator's documents by a wide margin,
        // and the one whose wording matters most: he is looking at what appears
        // to be one line and pdfce has just edited one piece of it.
        Reason::SharesTheLine => {
            "this line is drawn as several separate pieces and the others are not part of your edit"
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
        for r in [Refusal::NoRun, Refusal::NoText] {
            let s = refusal(r);
            assert!(s.len() > 40, "{r:?} needs a real sentence, got {s:?}");
            assert!(
                s.ends_with('.'),
                "{r:?} is prose and prose is punctuated: {s:?}"
            );
        }
    }

    /// ★★ **The multi-run note says what pdfce WILL DO, not what it refuses.**
    ///
    /// Its ancestor asserted `s.contains("Click directly on the word")` — advice
    /// that could not work, because the refusal was about the *line* and not
    /// about where on it the operator clicked. The property that replaces it is
    /// the one that matters now: the sentence must name **the consequence the
    /// operator cannot otherwise see**, which is that the neighbouring pieces
    /// will not move.
    #[test]
    fn the_multi_run_note_says_what_happens_rather_than_refusing() {
        let s = shares_the_line_note();
        assert!(
            s.contains("stay exactly where they are"),
            "the note must say what happens to the pieces the operator did NOT edit — that is \
             the whole of what they cannot see: {s:?}"
        );
        assert!(
            s.contains("overlap"),
            "and it must name the cost, because a longer replacement growing into the next cell \
             is the one thing this decision can produce that the operator would call a bug: {s:?}"
        );
        // …and it does not use the word the engine uses, which names nothing
        // the operator can see on their page.
        assert!(!s.contains("run"), "'run' is a PDF term, not an operator's");
    }

    /// ★ **Sharing the line PINS**, and that is the property the whole fix
    /// rests on.
    ///
    /// If this reason ever reflowed, editing one cell of a SolidWorks parts
    /// table would slide every cell after it sideways — content the operator did
    /// not touch, moved by an edit that did not mention it. Asserted here rather
    /// than only in `disposition`'s own tests because this module is where the
    /// sentence promising it lives, and a sentence and a behaviour that disagree
    /// is worse than either alone.
    #[test]
    fn sharing_the_line_pins_the_neighbours() {
        assert!(Reason::SharesTheLine.pins_the_tail());
        let s = pinned_tail_disclosure(Reason::SharesTheLine);
        assert!(s.contains("several separate pieces"), "{s:?}");
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
