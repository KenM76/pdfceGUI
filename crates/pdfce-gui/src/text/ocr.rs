//! # `text::ocr` — every word the Recognise-text surface says
//!
//! Consumed by [`crate::dialogs::ocr`] (the dialog that runs recognition and
//! reports what it inferred) and by [`crate::find::bar`] (the offer that
//! appears when a search found nothing on a page that has no text to find).
//!
//! ## ★ Why this catalog is unusually careful, and it is not house style
//!
//! **OCR is the single largest inference pdfce makes.** `pdfce-core`'s own
//! `ocr::layer` header says it in those words — *"every word here is a
//! guess"* — and project rule 4 (*"fuzzy, never sneaky"*) therefore binds this
//! surface harder than any other in the program. Two of its clauses bite here
//! and they pull in different directions, which is why the copy is written the
//! way it is:
//!
//! 1. **The result must look normal.** The operator asked for exactly that:
//!    *"I want OCRed stuff to look normal when the command is executed too."*
//!    Mode 3 is not a compromise, it is the whole mechanism — nothing visible
//!    is added, the page renders pixel-identically, and there is **no**
//!    highlighting of doubtful words baked into the document. So none of the
//!    copy below promises a visible mark, and none of it should ever grow one.
//! 2. **The uncertainty must be stated anyway**, off-canvas, before the
//!    recognition becomes a file. That is what this dialog is for.
//!
//! ## ★ The one fact this surface exists to carry
//!
//! **`ocrs` reports no confidence at all.** Not "low confidence", not
//! "confidence pending" — its output type is a character and a rectangle, and
//! there is no score on a character, a word, a line or the page.
//! `OcrsEngine::reports_confidence()` returns `false` and
//! `pdfce_core::ocr::engine_ocrs`'s header is explicit that this is *"a fact
//! about the world"* rather than a stub awaiting improvement.
//!
//! The consequence for copy is sharp, and it is the reason [`no_confidence`]
//! is worded as a negation of a specific wrong reading rather than as a
//! neutral note: **an absent score and a high score must never look the
//! same.** A dialog that reported "0 words need review" would be true of an
//! engine that scores nothing and would read as a clean bill of health. So
//! this surface never says that, and `OcrPage::words_needing_review` already
//! encodes the same principle on the other side by counting an unscored word
//! as needing review.
//!
//! ## Where the *engine's* sentences come from, and why they are not here
//!
//! [`crate::dialogs::ocr`] renders `OcrLayerReport::disclosures()` — a
//! `Vec<String>` built inside `pdfce-core` — as a list, verbatim, beneath the
//! headings below. That is deliberate and it is the engine's own instruction:
//! the disclosures are built *"here rather than at each call site so the GUI
//! and the CLI cannot disagree about what was disclosed."*
//!
//! They are therefore **data at run time**, not literals in this crate, and
//! `tools/gates/check-ui-strings.sh` is untouched by them. What lives here is
//! the shell's own framing — the headings, the buttons, the refusals — which
//! is exactly the split rule R1 is about: a catalog owns the words this
//! program chose, not the words another crate reported.
//!
//! ## Conventions
//!
//! [`crate::text`]'s, unchanged: sentence case and no trailing period on a
//! label, full sentences with punctuation for prose, an ellipsis on a control
//! that asks a question before acting. One addition — **no sentence here
//! makes a claim about accuracy.** pdfce has never measured this engine
//! against a real scan (`FEATURES.md` records that its only test documents are
//! vector PDFs that already contain text), so "accurate", "reliable" and
//! "high quality" are words this surface is not entitled to.

// ---------------------------------------------------------------------------
// The dialog
// ---------------------------------------------------------------------------

/// The dialog's title.
#[must_use]
pub fn title() -> &'static str {
    "Recognise text"
}

/// The sentence at the top of the dialog, before anything has been run.
///
/// Says what the operation *does to the page*, because that is the first
/// question an operator has about a tool that rewrites a document they may
/// have to defend the provenance of. The answer — nothing visible changes, the
/// image is not re-encoded — is `ocr::layer`'s own guarantee and is worth
/// leading with rather than burying under a progress bar.
#[must_use]
pub fn intro() -> &'static str {
    "Reads the words in the page image and adds them as invisible text behind it, so Find and copy work. The page still looks exactly the same, and the scan itself is never re-encoded."
}

/// The label on the control that starts recognition.
#[must_use]
pub fn run() -> &'static str {
    "Recognise this page"
}

/// Its tooltip.
///
/// Names the cost in the operator's terms. There is no measured figure to
/// quote — see the module header on what this surface is not entitled to
/// claim — so it says *seconds* and says which page, which are both true and
/// checkable.
#[must_use]
pub fn run_tooltip() -> &'static str {
    "Runs the recogniser over the page you are looking at. It takes a few seconds and the window will not respond while it does."
}

/// Shown while the recogniser is working.
#[must_use]
pub fn working() -> &'static str {
    "Recognising…"
}

/// The heading above the engine's own disclosure lines.
#[must_use]
pub fn what_was_inferred() -> &'static str {
    "What was recognised, and what that is worth"
}

/// ★ **The confidence sentence, and the most load-bearing string here.**
///
/// Worded to refuse a specific wrong reading rather than to state a neutral
/// fact, because the wrong reading is the one a reader arrives with: a page of
/// recognised text with no warnings on it looks checked. It is not checked. It
/// was never scored either way.
///
/// The engine emits its own version of this through
/// `OcrLayerReport::disclosures()`, and the two are deliberately both present:
/// that one appears in the list of disclosures beside the counts, this one is
/// the dialog's own heading-level statement, and the operator reads the second
/// before they read the list. Duplication is the point — this is the one fact
/// that must not be missed by someone who skims.
#[must_use]
pub fn no_confidence() -> &'static str {
    "This recogniser reports no confidence score for any word, so nothing here has been checked — that is not the same as everything being right. Read the text before you rely on it."
}

/// The sentence that says the recognition is not in a document yet.
///
/// The dialog's whole shape rests on this being true: recognition happens,
/// the operator reads what it inferred, and only then do they choose where it
/// goes. Nothing is written until they name a file.
#[must_use]
pub fn not_saved_yet() -> &'static str {
    "Nothing has been written yet. Choose where to save the recognised copy."
}

/// The label on the control that writes the recognised document.
///
/// ★ **"Save as", never "Save".** The operator's standing rule is that Read
/// may produce a new document and may not modify this one, and the enforcement
/// point is the save rather than the operation. This is the only write to disk
/// this shell performs, so it is also the first place that rule can bite —
/// and it bites the same way in every mode, because a destination the operator
/// names cannot be the file they opened unless they say so.
#[must_use]
pub fn save_as() -> &'static str {
    "Save recognised copy as…"
}

/// Its tooltip.
#[must_use]
pub fn save_as_tooltip() -> &'static str {
    "Writes a new PDF with the invisible text layer added. The document you opened is not changed."
}

/// The title on the system file-save dialog.
#[must_use]
pub fn save_dialog_title() -> &'static str {
    "Save recognised copy"
}

/// The suffix appended to the original file's stem to suggest a name.
///
/// A suggestion, not a rule — the operator can type anything. It exists so the
/// default answer is never the file they opened, which is the same protection
/// the label spells out in words.
#[must_use]
pub fn suggested_suffix() -> &'static str {
    "-recognised"
}

/// Confirmation, once the bytes are on disk. `path` is what they chose.
#[must_use]
pub fn saved(path: &str) -> String {
    format!("Saved to {path}")
}

/// The button that closes the dialog.
#[must_use]
pub fn close() -> &'static str {
    "Close"
}

// ---------------------------------------------------------------------------
// Named refusals
//
// Every one of these is a specific, actionable cause. The engine's own error
// type refuses by name for the same reason, and folding them into one
// "OCR failed" would throw away the half of the message the operator can act
// on.
// ---------------------------------------------------------------------------

/// The models are not where this build looks for them.
///
/// `searched` is the engine's own list of every directory it tried, in order.
/// It is part of the message rather than a detail: *"models not found"* is
/// unactionable, and the list is what tells an operator either where to put
/// the files or — just as often — that they put them somewhere pdfce never
/// looks.
///
/// ★ It takes a **list**, not a pre-joined string, and the separator below is
/// why: a comma and a space between two paths is punctuation an operator reads,
/// so it is copy and belongs in this file rather than at the call site.
/// `tools/gates/check-ui-strings.sh` caught exactly that `", "` sitting in
/// `dialogs::ocr::sentence`, and it was right to.
#[must_use]
pub fn models_missing(searched: &[String]) -> String {
    let list = searched.join(", ");
    format!(
        "The recognition models are not installed. They ship in the models\\ocrs folder beside \
         pdfce-gui.exe; this build looked in: {list}"
    )
}

/// This build was compiled without the recogniser.
///
/// A named refusal rather than a greyed control, and distinct from
/// [`models_missing`] on purpose: *"cannot look for text"* and *"could not
/// find the files to look with"* call for completely different actions, and
/// the engine's own feature block insists the two never collapse into one
/// answer.
#[must_use]
pub fn engine_absent() -> &'static str {
    "This build was made without the text recogniser, so it cannot read words from an image. A standard pdfce build can."
}

/// The document has unsaved edits, which recognition would not carry.
///
/// ★ **A refusal, not a warning, and the reasoning is worth keeping.**
/// `pdfce_core::ocr::layer::add_ocr_layer` writes an incremental revision on
/// top of the document **as it was opened** — that is what makes the scan
/// byte-identical and the round trip clean — so a recognised copy taken while
/// edits are pending would silently be a copy of the *original*, with the
/// operator's work missing and nothing on screen to say so. Producing that
/// file quietly is the sneaky half of rule 4; declining to is the honest half.
#[must_use]
pub fn unsaved_edits() -> &'static str {
    "This document has unsaved changes. Recognition writes from the file as it was opened, so the recognised copy would not contain them."
}

/// Recognition ran and found no word it could place.
///
/// Distinct from a failure: the engine worked, the page simply had nothing on
/// it a recogniser could read. Blank paper and a photograph of a wall both
/// land here, and so does a page whose ink is too faint.
#[must_use]
pub fn nothing_recognised() -> &'static str {
    "No text was recognised on this page. There may be nothing readable on it, or the image may be too small or too faint."
}

/// The recogniser or the layer writer refused, carrying the engine's reason.
///
/// The engine's own sentence is appended rather than replaced. `pdfce-core`'s
/// error types name specific causes — an encrypted document, a page index past
/// the end, a model file the runtime rejected — and paraphrasing them here
/// would produce a second, vaguer account of a diagnosis that was already
/// precise.
#[must_use]
pub fn failed(reason: &str) -> String {
    format!("Recognition did not finish: {reason}")
}

// ---------------------------------------------------------------------------
// The Find offer
// ---------------------------------------------------------------------------

/// ★ **The sentence the Find bar shows when the page has no text at all.**
///
/// It reports the *page*, not the search. That distinction is the whole rule
/// and the operator stated it: the trigger is *"this document is images"*, and
/// it is **not** *"this search had no matches"*. A search for a word that
/// simply is not in a text PDF is an ordinary empty result, and offering to
/// recognise it would be nonsense — so this sentence says what was actually
/// established, which is that there is no text on this page for any search to
/// have found.
#[must_use]
pub fn offer() -> &'static str {
    "This page has no text on it — only an image."
}

/// The control beside it.
///
/// Ellipsis, because it opens the dialog rather than recognising on the spot.
/// A search bar is the wrong place to start several seconds of work from a
/// single click.
#[must_use]
pub fn offer_action() -> &'static str {
    "Recognise text…"
}

/// Its tooltip.
#[must_use]
pub fn offer_tooltip() -> &'static str {
    "Opens Recognise text, which reads the words in the page image and adds them as invisible text so Find can see them."
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **This surface never claims accuracy.**
    ///
    /// The module header's rule, asserted rather than trusted. pdfce has never
    /// run this engine against a real scan, so any adjective implying measured
    /// quality would be a claim with nothing behind it — and marketing
    /// adjectives are exactly what a copy pass adds without thinking.
    #[test]
    fn nothing_here_claims_the_recognition_is_accurate() {
        let forbidden = [
            "accurate",
            "accuracy",
            "reliable",
            "reliably",
            "high quality",
            "high-quality",
            "precise",
            "correctly",
            "perfect",
        ];
        let prose: Vec<String> = vec![
            title().to_owned(),
            intro().to_owned(),
            run().to_owned(),
            run_tooltip().to_owned(),
            working().to_owned(),
            what_was_inferred().to_owned(),
            no_confidence().to_owned(),
            not_saved_yet().to_owned(),
            save_as().to_owned(),
            save_as_tooltip().to_owned(),
            engine_absent().to_owned(),
            unsaved_edits().to_owned(),
            nothing_recognised().to_owned(),
            offer().to_owned(),
            offer_action().to_owned(),
            offer_tooltip().to_owned(),
        ];
        for line in &prose {
            let lower = line.to_lowercase();
            for word in forbidden {
                assert!(
                    !lower.contains(word),
                    "`{line}` claims {word:?}; this surface has no measurement behind such a claim"
                );
            }
        }
    }

    /// ★ **The confidence sentence says the absence is not a clean bill.**
    ///
    /// The one string on this surface that must not be softened. It is here as
    /// a test rather than only as a doc comment because "no confidence
    /// reported" reads as neutral, and a future copy pass tidying it into
    /// something neutral would delete the disclosure while leaving a sentence
    /// in its place.
    #[test]
    fn the_confidence_sentence_refuses_the_wrong_reading() {
        let text = no_confidence().to_lowercase();
        assert!(
            text.contains("no confidence"),
            "it must state the absence outright: {text}"
        );
        assert!(
            text.contains("not the same"),
            "…and must say that the absence is not the same as everything being right, which is \
             the reading it exists to refuse: {text}"
        );
    }

    /// **The write is a Save as, in words as well as in behaviour.**
    ///
    /// The operator's rule is enforced at the save, so the label is part of the
    /// enforcement: a control saying `Save` would promise in-place saving that
    /// this shell has no path for and that Read forbids outright.
    #[test]
    fn the_write_control_offers_a_new_file_and_never_an_overwrite() {
        assert!(save_as().contains("as…"), "it must read as a prompt");
        assert!(
            save_as_tooltip().contains("not changed"),
            "the tooltip must say the opened document is untouched"
        );
        assert!(
            suggested_suffix().starts_with('-'),
            "the suggested name must differ from the original's stem"
        );
    }

    /// ★ **The Find offer talks about the page, not about the search.**
    ///
    /// The trap the operator named, pinned. A sentence mentioning matches
    /// would be the collapse of *"the document is images"* into *"this search
    /// found nothing"* — the two the specification insists must not be one.
    #[test]
    fn the_find_offer_reports_the_page_rather_than_the_search() {
        let text = offer().to_lowercase();
        assert!(
            text.contains("no text") && text.contains("page"),
            "the offer must state what is true of the page: {text}"
        );
        for word in ["match", "search", "result", "found"] {
            assert!(
                !text.contains(word),
                "the offer must not mention {word:?} — the trigger is that the page is an image, \
                 not that a search came back empty: {text}"
            );
        }
    }

    /// Two different absences produce two different sentences.
    #[test]
    fn a_missing_engine_and_missing_models_are_not_the_same_message() {
        assert_ne!(engine_absent(), models_missing(&["x".to_owned()]));
        assert!(
            models_missing(&["C:\\a".to_owned(), "C:\\b".to_owned()]).contains("C:\\a, C:\\b"),
            "the searched paths are the actionable half and must survive into the message — \
             and so must the separator between them, which is why this function joins the \
             list rather than taking one already joined"
        );
    }
}
