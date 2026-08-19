//! # `text::resizing` — every sentence the resize grips show
//!
//! Six refusals and one disclosure, for [`crate::canvas::resizing`].
//!
//! ## ★★ Why a refusal here is worth more than the feature it refuses
//!
//! The eight grips have been drawn, cursored and drag-consuming since S4 and
//! have **committed nothing** for the whole life of this shell. An operator
//! aiming at one got a resize cursor, a drag that felt like it was doing
//! something, and no change — which is the exact shape of `DEFECTS.md` D4a,
//! the defect that began this project: *the old shell's answer to a caret it
//! could not place was a boolean and a keyboard that stopped responding.*
//!
//! So the sentences below are not decoration on a new feature. **Three of the
//! six describe cases the new feature still cannot do**, and saying so is the
//! difference between a limit and a bug. A drag on a text run will still change
//! nothing; what changes today is that the operator is told why in one
//! sentence, and told what would work instead.
//!
//! ## The rule every sentence follows
//!
//! **Name the thing the operator can see, never the thing pdfce models.** They
//! can see a line of text, a picture, and a shape made of corners; they cannot
//! see a "show operator", a "path object" or a "node". `canvas::textedit`'s
//! catalogue makes the same choice in the same words, and for the same reason:
//! a refusal phrased in the file format's vocabulary is a refusal that reads as
//! an internal error.

use crate::canvas::resizing::Refusal;

/// The sentence for a refusal to resize.
///
/// One function over the enum rather than one per variant, so a variant added
/// to [`Refusal`] is a compile error here instead of a drag that refuses
/// silently — which is what the grips did for their whole life until
/// 2026-08-19.
#[must_use]
pub const fn refusal(reason: Refusal) -> &'static str {
    match reason {
        Refusal::NothingSelected => {
            "Select something first. Click a shape on the page, then drag one of the squares \
             around it to resize it."
        }
        // ★ Names the alternative that DOES work, which is what makes it a
        // refusal rather than a shrug: moving a group is supported, and an
        // operator who wanted to resize several things can still do them one at
        // a time.
        Refusal::ManyObjects => {
            "pdfce resizes one shape at a time. Select just the one you want and drag again — \
             several shapes can be moved together, but not resized together."
        }
        // ★★ The commonest of the six on this operator's documents, and the one
        // whose wording carries the most. It says what pdfce will not do and
        // WHY in the operator's terms — not "a text object has no nodes".
        Refusal::NotAPath => {
            "pdfce cannot resize text or pictures — only shapes drawn out of lines and curves. \
             Resizing text would mean changing its size, which is a different edit and is not \
             built yet."
        }
        Refusal::NoObjectModel => {
            "pdfce could not read this page's shapes, so it will not guess at what a resize \
             would do to them."
        }
        // Refused rather than clamped, and the sentence says which direction to
        // go so the operator does not simply try the same drag again.
        Refusal::Degenerate => {
            "That would flatten the shape to nothing or turn it inside out. Drag back the other \
             way to make it smaller without collapsing it."
        }
        Refusal::NoNodes => {
            "This shape has no corners to move, so there is nothing for a resize to act on."
        }
    }
}

/// ★ The disclosure a completed resize owes — **line weight does not scale**.
///
/// # Why this is disclosed rather than fixed, and rather than ignored
///
/// A path scaled by moving its nodes keeps its original `w`, so a box dragged
/// to twice the size has the same stroke width it started with.
///
/// That is **usually right and never chosen**, and both halves matter. On a CAD
/// drawing a line weight is a *drafting standard* — 0.25 mm is 0.25 mm whatever
/// size the detail is drawn at — so scaling it would be wrong far more often
/// than keeping it, and every drafting package this operator uses keeps it.
///
/// But it is a decision pdfce made and he did not, and he cannot see that it
/// was made: the shape looks right, and only a measurement would show that its
/// outline is now proportionally thinner than it was. Rule 4's surviving half —
/// *an inference the operator cannot see still owes an off-canvas report* — so
/// it is said once, off the canvas, in the same channel every other edit
/// disclosure uses.
///
/// **Two sentences and no more**, because it shares the status row with
/// everything else and R128 forbids that row growing.
#[must_use]
pub const fn line_weight_disclosure() -> &'static str {
    "layout: the shape changed size and its line thickness did not, which is how a drawing \
     standard works. If you wanted the outline heavier too, that is a separate change pdfce \
     cannot make yet."
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★ **Every refusal has a real sentence, and none of them is empty.**
    ///
    /// The whole point of the module. The grips' answer to every case they
    /// could not handle was silence, for the entire life of the shell.
    #[test]
    fn every_refusal_says_something() {
        for r in [
            Refusal::NothingSelected,
            Refusal::ManyObjects,
            Refusal::NotAPath,
            Refusal::NoObjectModel,
            Refusal::Degenerate,
            Refusal::NoNodes,
        ] {
            let s = refusal(r);
            assert!(s.len() > 40, "{r:?} needs a real sentence, got {s:?}");
            assert!(
                s.ends_with('.'),
                "{r:?} is prose and prose is punctuated: {s:?}"
            );
        }
    }

    /// ★★ **No sentence uses a word from the file format.**
    ///
    /// The rule in the module header, mechanised. An operator can see a shape,
    /// a line of text and a picture; they cannot see a *node*, a *path object*
    /// or a *show operator*, and a refusal phrased in those terms reads as an
    /// internal error rather than as a limit.
    ///
    /// "corners" is the word this catalogue uses where the engine says "nodes",
    /// and it is checked for rather than merely permitted — the `NoNodes`
    /// sentence has to say *something* about them.
    #[test]
    fn nothing_is_phrased_in_the_file_formats_vocabulary() {
        for r in [
            Refusal::NothingSelected,
            Refusal::ManyObjects,
            Refusal::NotAPath,
            Refusal::NoObjectModel,
            Refusal::Degenerate,
            Refusal::NoNodes,
        ] {
            let s = refusal(r).to_lowercase();
            for word in [
                "node",
                "operator",
                "path object",
                "subpath",
                "content stream",
            ] {
                assert!(
                    !s.contains(word),
                    "{r:?} says {word:?}, which names nothing an operator can see: {s:?}"
                );
            }
        }
        assert!(
            refusal(Refusal::NoNodes).contains("corners"),
            "the empty-shape refusal must still say what is missing, in a word that means \
             something on a page"
        );
    }

    /// ★ **The two refusals that have a working alternative name it.**
    ///
    /// A refusal that does not say what to do instead is a shrug with a capital
    /// letter — `text::commands`' own rule. These two are the ones where
    /// something *does* work: select one shape, or move rather than resize.
    #[test]
    fn the_refusals_with_an_alternative_offer_it() {
        assert!(
            refusal(Refusal::ManyObjects).contains("one at a time")
                || refusal(Refusal::ManyObjects).contains("just the one")
        );
        assert!(refusal(Refusal::NothingSelected).contains("Click"));
    }

    /// The line-weight disclosure names both what happened and what it means.
    ///
    /// Not "the stroke width was preserved", which states a fact about the file
    /// and leaves the operator to work out whether that was on purpose.
    #[test]
    fn the_line_weight_disclosure_explains_itself() {
        let s = line_weight_disclosure();
        assert!(s.contains("line thickness"));
        assert!(
            s.contains("drawing standard"),
            "it must say WHY this is the right default, or it reads as pdfce failing to scale \
             something: {s:?}"
        );
    }
}
