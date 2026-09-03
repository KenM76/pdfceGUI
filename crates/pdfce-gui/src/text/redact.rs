//! # `text::redact` — every word the redaction surface says
//!
//! Consumed by [`crate::panels::redact`] (mark and review) and
//! [`crate::dialogs::redact`] (the apply transaction and its report).
//!
//! Carried across from the old shell's `ui_text.rs:6876-7410` on 2026-08-15,
//! with the wording rules that governed it. Those rules are the most valuable
//! part of the salvage and they are reproduced here in full, because they are
//! the reason the strings below read as they do:
//!
//! > The wording rules here are stricter than anywhere else in this catalog,
//! > because **this is the one feature where a comfortable sentence is a
//! > security defect.** Three of them, and binding on anyone editing these
//! > strings:
//! >
//! > 1. **Never say "removed" without qualification when anything was left.**
//! >    A residual is named in the SAME sentence as the success, never in a
//! >    footnote the operator can miss.
//! > 2. **Never say "verified" unless a verification step actually ran.** One
//! >    does — [`crate::redact::prepare_redaction_apply`] greps the finished
//! >    bytes — so the word is earned; but [`verified_line`] is the only place
//! >    it may appear, and only from a clean
//! >    [`crate::redact::AbsenceVerification`].
//! > 3. **Never put the word "Undo" near a post-apply state.** Every OTHER edit
//! >    in pdfce teaches the operator that undo is available until save; this
//! >    is the one moment that learned expectation is wrong, so the copy
//! >    corrects it on screen instead of leaving it to be assumed.
//!
//! ## ★ The one distinction every string here has to keep alive
//!
//! `crate::text::commands::edit_redact`'s shipped tooltip states it in four
//! words — ***"Marking is reversible; applying is not"*** — and
//! `crate::shell::manifest::edit` explains why the two commands sit together in
//! that order: *"the asymmetry between them is the dangerous part."*
//!
//! The single most-cited real-world redaction failure is an operator who
//! believes marking **is** redacting and ships the marked file. So the marking
//! copy never says "removed", the review count says the content is *still
//! there*, and the apply copy leads with permanence rather than burying it.
//!
//! ## ★ A departure from the source, and it is about this shell rather than
//! about copy
//!
//! The old shell's permanence statement already deviated from its own ui-spec,
//! because apply there wrote a **new file** and left the open document alone.
//! That is still true here and is now enforced structurally as well as worded
//! (`crate::redact` §4), so the sentence is carried across essentially
//! unchanged. What is new is that this shell's *ordinary* save is incremental
//! and promises so on `file.save_copy`'s tooltip — which makes
//! [`single_revision_note`] carry more weight here than it did there: it is the
//! one place an operator is told that this write does **not** behave like the
//! save they already know.
//!
//! ## Conventions
//!
//! [`crate::text`]'s, unchanged: sentence case and no trailing period on a
//! label, full sentences with punctuation for prose, an ellipsis on a control
//! that asks a question before acting.
//!
//! One addition of this module's own: **`⚠` is the residual mark and the only
//! non-ASCII character used here.** It is measured drawable — `DEFECTS.md` D12
//! records the correction that established it — and
//! `crate::icons::glyphs::tests::every_glyph_the_catalog_draws_has_a_glyph`
//! sweeps this file with the rest of the catalog, so a decorative glyph added
//! later that the bundled fonts cannot draw fails a test rather than shipping
//! as a box. That matters more on this surface than on any other: a residual
//! line whose first character is a broken box reads as a rendering failure, and
//! an operator who has decided a surface is broken stops reading it.

// ---------------------------------------------------------------------------
// The panel — marking and review
// ---------------------------------------------------------------------------

/// The panel's own heading.
#[must_use]
pub fn panel_title() -> &'static str {
    "Redact"
}

/// The sentence at the top of the panel.
///
/// States the whole two-phase model in one line, because an operator who
/// believes marking IS redacting is the single most-cited real-world redaction
/// failure. Carried verbatim from the old shell's `redact_panel_intro`.
#[must_use]
pub fn panel_intro() -> &'static str {
    "Mark content, then apply to permanently remove it. Marking is reversible and changes nothing in the file; applying writes a new file with the marked content gone, and cannot be undone."
}

/// The heading over the marking controls.
#[must_use]
pub fn mark_heading() -> &'static str {
    "Mark content for removal"
}

/// The control that marks the whole of the page on screen.
#[must_use]
pub fn mark_whole_page() -> &'static str {
    "Mark whole page"
}

/// Its tooltip.
///
/// Names the reversibility explicitly, since marking an entire page in one
/// click is easy to do by accident.
#[must_use]
pub fn mark_whole_page_tooltip() -> &'static str {
    "Mark this entire page for redaction. Nothing is removed until you apply, and you can take the mark off again from the list below or with Undo."
}

/// ★★★ Why *Mark whole page* is greyed — `OPERATOR_REQUESTS.md` O77.
///
/// The control's only gate is `page_count > 0`, so there is exactly one reason
/// and it can be stated flatly. It is a document with no pages, which is legal
/// PDF (`/Count 0`) and which pdfce opens rather than refusing — so this is a
/// real state an operator can be in, not a defensive branch.
///
/// ★ It names the cause rather than the remedy, and that is the right way
/// round here: there is no action he can take inside this panel to give the
/// document a page, so *"add a page first"* would be advice about somewhere
/// else. Where a remedy exists — the search button beside it — the sentence
/// names the remedy instead.
#[must_use]
pub fn mark_whole_page_disabled() -> &'static str {
    "This document has no pages to mark."
}

/// The label beside the search field.
#[must_use]
pub fn search_label() -> &'static str {
    "Text:"
}

/// The control that searches and marks every hit.
#[must_use]
pub fn search_button() -> &'static str {
    "Find & mark"
}

/// Its tooltip, in both states.
///
/// The disabled form explains what would enable it, rather than leaving a dead
/// control unexplained.
#[must_use]
pub fn search_button_tooltip(has_query: bool) -> &'static str {
    if has_query {
        "Finds this text on every page and adds a mark over each match, for you to review before applying."
    } else {
        "Type the text you want marked first — there is nothing to search for."
    }
}

/// The label on the literal-versus-pattern switch.
#[must_use]
pub fn match_mode_label() -> &'static str {
    "Match:"
}

/// The literal half of that switch.
#[must_use]
pub fn match_literal() -> &'static str {
    "Exact text"
}

/// Its tooltip.
#[must_use]
pub fn match_literal_tooltip() -> &'static str {
    "Find this text exactly as typed, ignoring upper and lower case."
}

/// The pattern half of that switch.
#[must_use]
pub fn match_pattern() -> &'static str {
    "Pattern"
}

/// Its tooltip.
///
/// Leads with the example rather than the syntax: the operator's actual thought
/// is *"redact every social security number"*, not *"I would like a wildcard
/// language."*
#[must_use]
pub fn match_pattern_tooltip() -> &'static str {
    "Find every run SHAPED like what you type — for example ###-##-#### marks every social-security number on every page in one action. Use this when you know the shape but not the values."
}

/// ★ **The hint under the search field, in whichever mode is selected.**
///
/// The scanned-page caveat is mandatory rather than decorative, and it is
/// carried in **both** modes deliberately. A silent zero-match result on a
/// scanned page is a named real-world failure: an operator reads *"no matches"*
/// as *"nothing sensitive here"* rather than as *"nothing SEARCHABLE here"*,
/// and dropping the warning from one of two hints is how it stops being read.
///
/// The pattern form states the whole syntax, because it is two characters long
/// and an operator who has to go looking for it will type a literal instead and
/// get nothing.
#[must_use]
pub fn search_hint(pattern: bool) -> &'static str {
    if pattern {
        "# matches any digit, ? matches any single character, everything else is literal. So ###-###-#### finds phone numbers and A?-#### finds A1-2345. Marks are added for you to review before applying. This can only find text pdfce can extract — on a scanned page with no text layer it will find nothing, which is not the same as there being nothing sensitive there."
    } else {
        "Finds this exact text on every page and adds a mark over each match, for you to review before applying. It can only find text pdfce can extract — on a scanned page with no text layer it will find nothing, which is not the same as there being nothing sensitive there."
    }
}

/// ★ **The census line above the mark list.**
///
/// Zero is a distinct sentence rather than "0 marks", because *"no marks"* is a
/// state an operator reads as an answer while *"0 pending redaction mark(s)"*
/// is one they read as a counter.
///
/// The non-zero form is the load-bearing one, and the shouted clause is
/// deliberate: this is the sentence standing between a marked document and an
/// operator who is about to email it.
#[must_use]
pub fn marks_count(count: usize) -> String {
    if count == 0 {
        "No redaction marks in this document. Nothing is marked, and nothing has been removed."
            .to_owned()
    } else {
        format!(
            "{count} pending redaction mark(s) — the content underneath them is STILL IN THIS FILE until you apply."
        )
    }
}

/// One row in the mark list: which page, and how big the marked region is.
///
/// The size is shown because two marks on one page are otherwise
/// indistinguishable in a list, and *"which one is the one I mis-marked?"* is
/// the question the list exists to answer.
#[must_use]
pub fn mark_row(page_number: usize, size: Option<(f64, f64)>) -> String {
    match size {
        Some((w, h)) => format!("Page {page_number} — region {w:.0} × {h:.0} pt"),
        None => format!("Page {page_number} — region (no stored size)"),
    }
}

/// The tooltip on a mark row.
///
/// Says what clicking it does; the row is a navigation control, not a
/// selection.
#[must_use]
pub fn mark_row_tooltip() -> &'static str {
    "Go to this page so you can see what the mark covers."
}

/// The control that takes one mark off.
///
/// ★ **A word rather than a `✕` glyph.** The old shell's note is carried
/// because the measurement behind it is: a decorative Unicode glyph outside the
/// bundled font chain ships as a tofu box, and U+2715 is one of the codepoints
/// `DEFECTS.md` D12's corrected table lists as having **no supporting face** at
/// all. `crate::icons` exists for controls that need a mark; a list row does
/// not.
#[must_use]
pub fn mark_remove() -> &'static str {
    "Remove"
}

/// Its tooltip.
///
/// The second half is the whole point: removing a MARK is not undoing a
/// redaction, because no redaction happened.
#[must_use]
pub fn mark_remove_tooltip() -> &'static str {
    "Remove this mark. It was never applied, so nothing in the document changes and nothing is recovered — the content it covers was there all along."
}

/// The control that opens the apply report.
///
/// The label promises a **report**, not an apply, because the click that opens
/// it must not feel like the click that commits.
#[must_use]
pub fn review_and_apply() -> &'static str {
    "Review & apply redactions…"
}

/// Its tooltip, in both states.
#[must_use]
pub fn review_and_apply_tooltip(can_apply: bool) -> &'static str {
    if can_apply {
        "Opens a report of exactly what will be permanently removed, and of anything pdfce could not remove. Nothing is written until you confirm there."
    } else {
        "Mark at least one region first — there is nothing to apply."
    }
}

// ---------------------------------------------------------------------------
// The apply dialog — the report, the acknowledgements, the write
// ---------------------------------------------------------------------------

/// The dialog's title.
#[must_use]
pub fn apply_title() -> &'static str {
    "Apply redactions — permanent removal"
}

/// The heading above the report body.
#[must_use]
pub fn report_heading() -> &'static str {
    "What applying will do"
}

/// ★★ **The permanence statement — the first thing in the dialog body, never
/// abbreviated, never softened.**
///
/// It says what this operation does *in this shell*, which is not what a
/// generic redaction warning would say. Apply does not mutate the open
/// document: it writes a new file and leaves the session exactly as it is,
/// marks and all (`crate::redact` §3). A sentence about "you cannot undo this
/// once you save" would describe a save that never happens.
///
/// The clause about the open document is not reassurance filler. It is the
/// answer to the question an operator asks immediately afterwards — *"so what
/// happened to the thing I was working on?"* — and getting it wrong in either
/// direction is expensive: believing the open document was redacted is the
/// worse error, and believing nothing happened at all is the one that makes
/// people press the button twice.
#[must_use]
pub fn permanence_statement() -> &'static str {
    "Applying writes a NEW file with the marked content permanently removed. It is a full rewrite, not an edit: nothing in that file can bring the removed content back — not Undo, not a previous revision, not any recovery tool. The document you have open is left exactly as it is now, marks and all."
}

/// The heading for the affirmative half of the report.
#[must_use]
pub fn will_remove_heading() -> &'static str {
    "Will be permanently removed:"
}

/// ★ **The removal summary — the measured centrepiece of the report.**
///
/// These are measurements, not predictions:
/// [`crate::redact::prepare_redaction_apply`] performs the whole removal in
/// memory before this dialog can show anything, so every number here describes
/// what actually happened to the bytes that will be written on confirm.
///
/// "character(s)", never "glyphs": the engine counts character codes removed
/// from content streams, and *glyph* is a typesetting word an operator has no
/// reason to know.
#[must_use]
pub fn removal_summary(regions: u64, pages: usize, glyphs: u64, streams: u64) -> String {
    format!(
        "{regions} marked region(s) across {pages} page(s): {glyphs} character(s) deleted from {streams} page content stream(s), and the marks themselves removed."
    )
}

/// The annotation line, shown only when the count is non-zero.
///
/// ★ **Worded as a TOTAL, not as an overlap count**, and the old shell's
/// correction is carried with it because the mistake is easy to repeat: the
/// engine's `annotations_removed` counts the redaction marks themselves *plus*
/// any annotation that overlapped a marked region, and the two are not reported
/// separately. An earlier draft attributed the whole figure to overlaps, which
/// on a three-mark fixture read as "3 annotations overlapping a marked region
/// will also be removed" when all three were the marks.
///
/// Overstating collateral damage is a smaller sin than understating it, and
/// still a lie — and this is the one feature whose entire value is that its
/// report can be believed. The overlap fact is still disclosed, because an
/// operator whose highlight silently vanished is owed the reason before it
/// happens rather than after.
#[must_use]
pub fn annotations_removed(count: u64) -> String {
    format!(
        "{count} annotation object(s) will be removed in total: the redaction marks themselves, plus any annotation that overlapped a marked region — an annotation sitting over redacted content can carry a copy of it in its own appearance or text."
    )
}

/// The document-information line, shown only when the count is non-zero.
#[must_use]
pub fn info_scrubbed(count: u64) -> String {
    format!(
        "{count} document-information entr(y/ies) contained the redacted text and will be scrubbed of it."
    )
}

/// The object-stream line, shown only when the count is non-zero.
///
/// ISO 32000-1 §7.5.7: a removed object must not survive compressed inside a
/// container.
#[must_use]
pub fn containers_decomposed(containers: u64, promoted: u64) -> String {
    format!(
        "{containers} compressed object container(s) will be taken apart ({promoted} object(s) moved out of them), so no removed object can survive inside one."
    )
}

/// ★ **The single-revision line — engine rule R35 in operator language.**
///
/// It carries more weight in this shell than it did in the one it came from.
/// `file.save_copy`'s shipped tooltip promises that an ordinary save *"appends
/// the edits as an update so the previous version stays intact inside the
/// file"* — which is exactly the property a redaction must not have, and which
/// an operator has by then been taught to expect. This is the one sentence that
/// tells them this write is different.
#[must_use]
pub fn single_revision_note() -> &'static str {
    "The new file will be a single revision. Any earlier revision of this document — which would still hold the un-redacted content — is not carried into it."
}

/// ★★ **The verification line — the ONLY place in this catalog permitted to
/// use the word "verified".**
///
/// Rule 2 of the module header. It is shown only from a clean
/// [`crate::redact::AbsenceVerification`], and what licenses it is that a real
/// search ran over the real output bytes: `crate::redact::proof` re-parses the
/// finished document, decodes every stream in it, and greps both the decoded
/// content and the raw buffer.
///
/// The three clauses at the end are the three places it looked, named
/// individually rather than summarised, because *"we checked"* is a claim and
/// *"we looked in the page content, in every other stream, and in the raw
/// bytes"* is a description someone could go and repeat.
#[must_use]
pub fn verified_line(strings_checked: usize) -> String {
    format!(
        "Verified: pdfce searched the finished file for all {strings_checked} distinct piece(s) of removed text and found none of them — not in the page content, not in any other stream, not in the raw bytes."
    )
}

/// The verification line's honest companion when some removed strings were too
/// short for a whole-file byte search to mean anything.
///
/// [`crate::redact::proof::MIN_VERIFIABLE_LEN`] is the four this names. A proof
/// that quietly skipped these would be claiming a completeness it does not
/// have.
#[must_use]
pub fn verification_limit_line(too_short: usize) -> String {
    format!(
        "{too_short} removed piece(s) were too short (under 4 characters) for a whole-file byte search to say anything useful, so those were checked against the decoded page content only."
    )
}

/// **The mark covers a raster image, so applying will be refused** —
/// `OPERATOR_REQUESTS.md` O103.
///
/// # ★★★ Said at MARK time, which is the whole point of the sentence
///
/// The refusal itself is the engine's and is correct: it cannot destroy image
/// pixels, and clipping or masking would leave them recoverable. What was wrong
/// was *when* the operator found out. Apply is all-or-nothing for the document,
/// so twelve careful marks and one that grazed a logo produced a single refusal
/// naming no region — after the work, not during it.
///
/// # Why it names the count and the consequence, and offers no remedy
///
/// Because there is no remedy pdfce can offer yet. Telling him to "move the
/// rectangle" would be advice we cannot check — on a title block the value and
/// the logo may genuinely overlap — and telling him to delete the mark would be
/// deciding on his behalf that it is not worth having when the engine may gain
/// the capability. So it states the fact and stops.
///
/// ★ The mark is still authored. This blocks nothing.
#[must_use]
pub fn mark_covers_image(images: usize) -> String {
    format!(
        "This region covers {images} image(s). pdfce cannot yet remove image pixels, so applying redactions to this document will be refused until no marked region touches an image — the mark itself is fine and is still here."
    )
}

/// The heading for the residual section — the part that makes the feature
/// honest.
#[must_use]
pub fn residual_heading() -> &'static str {
    "⚠  pdfce could NOT remove the following — read this before continuing:"
}

/// One residual line for a carrier the engine detected and could not scrub.
#[must_use]
pub fn residual_carrier_line(carrier: &str) -> String {
    format!(
        "⚠  {carrier}: present in this document, and pdfce cannot scrub it in this build. Whatever it holds will still be in the saved file — check it by hand."
    )
}

/// ★ **One residual line for a removed string that still occurs in the raw
/// output bytes while occurring in no decoded stream.**
///
/// Worded so it claims exactly what pdfce knows and nothing more: the byte run
/// is there; whether it is a real leftover copy or an unrelated coincidence is
/// not something pdfce can decide. See `crate::redact::proof`'s table for why
/// this is disclosed rather than refused.
#[must_use]
pub fn raw_residual_line(text: &str) -> String {
    format!(
        "⚠  The removed text “{text}” no longer appears in any page content, but that same byte sequence still occurs somewhere in the saved file. It may be an unrelated coincidence, or a copy in a carrier pdfce does not recognise — pdfce cannot tell which, so it is reported rather than claimed removed."
    )
}

/// One residual line when materialising the operator's unsaved edits had to
/// promote objects out of an object stream (engine rule R38).
#[must_use]
pub fn promotion_line(count: usize) -> String {
    format!(
        "⚠  {count} object(s) had to be moved out of a compressed container in order to write your unsaved edits, and the container keeps its own copy of their previous value. Page content is never stored that way, so this cannot hold redacted text — but it is a leftover of your edits and is reported rather than passed over."
    )
}

/// The scope reminder — what redaction does **not** touch.
///
/// Named so an operator does not read "redacted" as "sanitised".
#[must_use]
pub fn scope_reminder() -> &'static str {
    "This removes what your marks cover. It does not sweep the document for unrelated hidden data: metadata history, embedded files, scripts or hidden layers that no mark touches are not part of this operation."
}

/// ★ **The extra acknowledgement, shown ONLY when the report has a residual
/// section.**
///
/// Distinct from [`confirm_checkbox`] on purpose: a partial redaction must
/// never be mistaken for a complete one. Showing it always would make it a box
/// operators tick without reading, which is the failure mode that makes every
/// other acknowledgement in the program worthless.
#[must_use]
pub fn residual_acknowledgement_checkbox() -> &'static str {
    "I have read the items above, I understand they will NOT be removed, and I still want to apply the redactions that can be completed."
}

/// The mandatory confirmation.
///
/// Its wording targets the exact misunderstanding this feature exists to
/// prevent.
#[must_use]
pub fn confirm_checkbox() -> &'static str {
    "I understand this permanently removes the underlying content, not just the visible marks."
}

/// ★ **The confirm control. The label IS the consequence** — never "OK", never
/// "Yes", never "Apply" alone.
#[must_use]
pub fn confirm_button() -> &'static str {
    "Permanently remove & save as…"
}

/// ★★★ Why *Permanently remove & save as…* is greyed —
/// `OPERATOR_REQUESTS.md` O77's sweep.
///
/// The most consequential of the seven dead explanations found on 2026-08-31:
/// this is the last control before content is destroyed, it was greyed, and it
/// had **no hover explanation at all**. An operator who could not press it had
/// no way to find out why.
///
/// # ★★ It names WHICH box, because *"tick the box"* is ambiguous here
///
/// Two checkboxes gate this button and they do not both appear. The
/// acknowledgement is always shown; the residual acknowledgement is shown only
/// when the engine reported content it could not prove was removed — which is
/// precisely the situation in which an operator is reading carefully and is
/// least able to afford a vague refusal.
///
/// Four states, three of them reachable: the pair is only ever consulted when
/// the button is off, so `(true, true)` cannot be seen here. It is answered
/// anyway rather than left to a `_` arm, because a sentence that cannot be
/// reached is better than a panic and better than a wrong one, and because the
/// day the gate grows a third term this arm is where the omission shows.
#[must_use]
pub fn confirm_disabled(acknowledged: bool, residuals_acknowledged: bool) -> &'static str {
    match (acknowledged, residuals_acknowledged) {
        (false, false) => "Tick both boxes above to confirm you have read what will be removed.",
        (false, true) => "Tick the box above to confirm you have read what will be removed.",
        (true, false) => {
            "Tick the second box above to confirm you have read what pdfce could not prove it removed."
        }
        (true, true) => "Ready.",
    }
}

/// The control that closes the dialog without applying.
///
/// Phrased as a deferral rather than a refusal, because cancelling here loses
/// nothing — the marks survive.
#[must_use]
pub fn cancel_button() -> &'static str {
    "Don't apply yet"
}

/// ★ **The no-shortcut disclosure, in the dialog's footer.**
///
/// Visible text rather than an omission an operator has to notice: this shell
/// binds `Ctrl+Z`, `Delete` and the whole `Ctrl` chord family to
/// destructive-but-reversible actions everywhere else, so the ABSENCE of a
/// chord on the one irreversible action is a deliberate asymmetry worth
/// stating.
#[must_use]
pub fn no_shortcut_note() -> &'static str {
    "There is deliberately no keyboard shortcut for this button. It is the one action in pdfce that cannot be undone, so it takes a deliberate click."
}

/// The title on the system file-save dialog.
#[must_use]
pub fn save_dialog_title() -> &'static str {
    "Save redacted copy"
}

/// ★ **The suffix appended to the original file's stem to suggest a name.**
///
/// A suggestion, not a rule — the operator can type anything. It exists so the
/// default answer is never the file they opened, which on this operation is the
/// difference between a copy and the destruction of the only remaining source
/// of the content being removed. `crate::dialogs::ocr::suggested_suffix` and
/// `crate::text::files::save_copy_suffix` enforce the identical rule for the
/// two milder writes.
#[must_use]
pub fn suggested_suffix() -> &'static str {
    "-redacted"
}

// ---------------------------------------------------------------------------
// Outcomes
// ---------------------------------------------------------------------------

/// The line shown once a clean redaction is on disk.
///
/// "Verified" is earned here — the absence proof ran on these exact bytes, and
/// ran again between the buffer and the write. The last clause corrects the
/// learned Undo expectation rather than leaving it to be assumed (rule 3).
#[must_use]
pub fn applied_clean(file_name: &str, regions: u64, pages: usize) -> String {
    format!(
        "Redacted and saved to {file_name} — {regions} region(s) across {pages} page(s) removed, and verified absent from the saved file. That file cannot be un-redacted; the document you still have open is unchanged."
    )
}

/// The line shown once a redaction that had acknowledged residuals is on disk.
///
/// Never shortened, never omitted, and never allowed to borrow the clean form's
/// wording: an operator who acknowledged a residual in a dialog and then closed
/// it is still owed a standing record of what remains. This is rule 1 —
/// **the residual is named in the same sentence as the success.**
#[must_use]
pub fn applied_with_residuals(file_name: &str, regions: u64, residuals: usize) -> String {
    format!(
        "⚠  Redacted and saved to {file_name} — {regions} region(s) removed, but {residuals} item(s) could NOT be removed and are still in that file. Do not treat it as fully redacted; see the report you acknowledged for what and why."
    )
}

/// The sentence for a refusal that happened before anything was written.
///
/// Every variant of [`crate::redact::RedactApplyRefusal`] gets its own sentence
/// rather than one "redaction failed", for `crate::text::ocr`'s reason: the
/// engine refuses by name because the causes have different remedies, and
/// folding four named causes into one message throws that away at the last
/// step.
#[must_use]
pub fn refusal_message(refusal: &crate::redact::RedactApplyRefusal) -> String {
    use crate::redact::RedactApplyRefusal as R;
    match refusal {
        R::NothingToApply => "Nothing to apply — this document has no redaction marks.".to_owned(),
        R::FullRewriteUnavailable { reason } => format!(
            "Redaction refused — this document cannot be rewritten in full, and nothing was written. Applying a redaction requires rewriting the entire file as one revision: an incremental save would leave the un-redacted content sitting in the file's previous revision, where anyone could recover it, so pdfce will not fall back to one. The writer's reason: {reason}"
        ),
        R::MaterialisedDocumentUnreadable { reason } => format!(
            "Redaction refused — pdfce rewrote your unsaved edits but could not read the result back, so it could not apply the redactions to them. Nothing was written. This is a fault in pdfce, not in your document. The parser's reason: {reason}"
        ),
        R::CoreRefused { reason } => {
            format!("Redaction refused, and nothing was written: {reason}")
        }
        // ★ The one refusal that is a report of a pdfce defect rather than of a
        // property of the operator's document, and the sentence says so. The
        // last clause is deliberately the strongest instruction in this
        // catalog: if the removal and the proof disagree, no file derived from
        // this document can be trusted, including ones written earlier.
        R::VerificationFailed { survivors } => format!(
            "Redaction refused — pdfce applied the removal, then searched the finished file and found {} piece(s) of the supposedly-removed text still in it. Nothing was written. Do not use any file produced from this document until this is investigated.",
            survivors.len()
        ),
    }
}

/// The sentence for a write that was attempted and produced no file.
///
/// Distinct from [`refusal_message`] because the two happen at different
/// moments and mean different things to the operator: a refusal happens when
/// they press *Review & apply* and nothing has been decided yet, and this
/// happens after they have confirmed, named a destination and expect a file to
/// be there.
#[must_use]
pub fn write_failed(reason: &crate::redact::WriteRefusal) -> String {
    use crate::redact::WriteRefusal as W;
    match reason {
        // Unreachable from the dialog, whose confirm control is disabled until
        // the box is ticked — and worded rather than left to a panic, because
        // `crate::redact::write_to`'s gate is the mechanism and a control being
        // greyed is not.
        W::ResidualsNotAcknowledged { .. } => {
            "Nothing was written: the items pdfce could not remove have not been acknowledged.".to_owned()
        }
        W::VerificationFailed { survivors } => format!(
            "Nothing was written. pdfce checked the finished bytes one last time before writing them and found {} piece(s) of the supposedly-removed text still present. Do not use any file produced from this document until this is investigated.",
            survivors.len()
        ),
        W::FileSystem(_) => {
            "The redacted file could not be written. Check that the folder still exists and that you can write to it, then try again — nothing has been lost, and the marks are still in the document.".to_owned()
        }
    }
}

/// The status-bar note appended to an ordinary save while marks are still
/// pending.
///
/// ★ Fires in ADDITION to the save's own outcome, never instead of it: the save
/// genuinely succeeded, and the operator also needs to know what it did not do.
/// This is the sentence that stands between `file.save_copy` and a marked file
/// leaving the building.
#[must_use]
pub fn save_kept_pending_marks(count: usize) -> String {
    format!(
        "That save kept {count} pending redaction mark(s) in the file — the marked content is still there. Marking does not remove anything; nothing is removed until you apply."
    )
}

// ===========================================================================
// Appearance — what an applied redaction LOOKS like
// ===========================================================================
//
// ★ Unblocked 2026-08-17. These three values shipped as hard-coded `None`s
// for the life of the project, and the reason was not neglect: `fill` could
// not reach marks made by Find & mark, and `overlay_text` was written into
// the PDF and never drawn. Both were filed and both came back fixed the same
// day. See `panels::redact::appearance` for the sequence and for the one
// finding that outlives it.

/// The appearance group's heading.
#[must_use]
pub const fn appearance_heading() -> &'static str {
    "How a redaction will look"
}

/// What the heading means, said once so no control below has to repeat it.
///
/// ★ It says **applied**, twice over, because that is the distinction the
/// whole panel turns on. Nothing chosen here changes anything until the
/// operator applies — a mark is a red outline whatever fill is set — and an
/// operator who expected the swatch to recolour their marks would otherwise
/// conclude the control does nothing.
#[must_use]
pub const fn appearance_intro() -> &'static str {
    "Marks are outlined in red while you review them. These settings are what \
     replaces the content when you apply, and they are recorded on each mark \
     as you make it — so changing them affects the next mark, not the ones \
     already in the list."
}

/// The fill control's label.
#[must_use]
pub const fn fill_label() -> &'static str {
    "Cover with"
}

/// One fill's name.
#[must_use]
pub const fn fill_option_label(fill: crate::panels::redact::appearance::Fill) -> &'static str {
    use crate::panels::redact::appearance::Fill as F;
    match fill {
        F::Black => "Black",
        F::White => "White",
        F::Custom(..) => "Colour…",
        F::Transparent => "Nothing",
    }
}

/// ★ What "Nothing" actually does, which is not what it sounds like.
///
/// The one fill an operator can misread as "do not redact". It removes the
/// content exactly as the others do; what it omits is the box that says so.
/// Said at the control rather than in a tooltip, because a tooltip is not
/// read before a choice is made and this is the choice with a surprise in it.
#[must_use]
pub const fn fill_transparent_note() -> &'static str {
    "The content is still removed. Nothing is drawn over the gap, so the page \
     will not show where it was."
}

/// The caption field's label.
#[must_use]
pub const fn overlay_label() -> &'static str {
    "Write on it"
}

/// The caption field's placeholder-ish hint.
#[must_use]
pub const fn overlay_hint() -> &'static str {
    "Optional. Left empty, the box is plain."
}

/// ★ The warning shown when a caption would be drawn in black on a dark fill.
///
/// The engine hard-codes black text in the `/DA` it authors and said so when
/// it shipped the burn-in: *"Wire a fill-colour picker, let someone choose a
/// dark red, and the caption will be black on dark red — we saw it in our own
/// verification render."* There is no overlay-text colour on the API yet.
///
/// So this is a **disclosure of a known engine limit**, not a style opinion,
/// and it names the two ways out rather than only the problem.
#[must_use]
pub const fn overlay_illegible_warning() -> &'static str {
    "pdfce draws this caption in black, and it will be hard to read on a dark \
     cover. Choose a lighter cover, or leave the caption off."
}

/// The justification control's label.
#[must_use]
pub const fn quadding_label() -> &'static str {
    "Line it up"
}

/// One justification's name.
#[must_use]
pub const fn quadding_option_label(q: pdfce_core::vartext::Quadding) -> &'static str {
    use pdfce_core::vartext::Quadding as Q;
    match q {
        Q::Left => "Left",
        Q::Center => "Centre",
        Q::Right => "Right",
    }
}

/// What the operator should know about the caption before they rely on it.
///
/// Two engine limits in one sentence, both stated when the burn-in shipped:
/// the face is Base-14 Latin, so anything outside it becomes `?`; and the
/// size is auto-chosen within a clamp, so a long caption on a small mark is
/// not going to be readable however it is justified.
#[must_use]
pub const fn overlay_bound() -> &'static str {
    "Captions use a standard Latin font — other alphabets come out as question \
     marks — and pdfce picks the size to fit the box, so a long caption on a \
     small mark ends up tiny."
}

/// What a mark-by-search says when part of the document could not be read.
///
/// ★★★ The strongest wording anywhere in this catalogue, and deliberately so.
/// Every other disclosure in pdfce reports something the operator can take or
/// leave. This one reports that **an operation they believe completed may not
/// have**, on the one action with no undo and the one whose failure they will
/// discover after sending the file.
///
/// It does not say "0 results". It says what was searched and what could not be
/// searched, and it says the consequence in the operator's own terms — the text
/// is still there — rather than in the mechanism's.
#[must_use]
pub fn unreadable_warning(fonts: u64) -> String {
    if fonts == 1 {
        "One font in this document stores text that cannot be searched. Any matches inside it were NOT marked and are still in the file.".to_owned()
    } else {
        format!(
            "{fonts} fonts in this document store text that cannot be searched. Any matches inside them were NOT marked and are still in the file."
        )
    }
}

/// The hover behind [`unreadable_warning`].
#[must_use]
pub const fn unreadable_tooltip() -> &'static str {
    "Some PDFs store text as drawings with no record of which letters they are. It looks and prints normally, but nothing can search it — so searching for a word cannot find it, and marking every match will miss it. Check those areas by eye, or mark them by drawing a box."
}

/// **What was marked from a canvas selection.**
///
/// `OPERATOR_REQUESTS.md` **O60**. The third marking route's disclosure.
///
/// # ★★★ Why it says MARKED and not REDACTED
///
/// Because nothing has been removed, and the difference is the single most
/// important thing about this whole feature. A `/Redact` annotation is a
/// **mark** (§12.5.6.23): it covers nothing, deletes nothing, and is perfectly
/// reversible until *Apply* is pressed.
///
/// An operator who read *"3 objects redacted"* would reasonably believe the
/// content was gone, stop reviewing, and save a document that still contains
/// every word of it. That is the one mistake in this feature that cannot be
/// undone by undoing — because it is a mistake about what to do next.
///
/// ⇒ So the sentence names the state and the next step, in that order.
///
/// # Why it counts OBJECTS and not marks
///
/// One gesture makes one annotation carrying one quad per object, so the mark
/// count is always 1 and would tell the operator nothing. What they chose was
/// objects; what they should be told about is objects.
#[must_use]
pub fn marked_selection(objects: usize) -> String {
    if objects == 1 {
        "Marked for redaction. Nothing has been removed yet \u{2014} press Apply redactions \
         when you have checked the marks."
            .to_owned()
    } else {
        format!(
            "{objects} objects marked for redaction. Nothing has been removed yet \u{2014} \
             press Apply redactions when you have checked the marks."
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★★ **No marking string ever claims content was removed.**
    ///
    /// Rule 1 of the module header, asserted rather than trusted. The failure
    /// this catches is a copy pass tightening *"marked for redaction"* into
    /// *"redacted"* — which reads better, is shorter, and is the exact
    /// misunderstanding that ships marked documents.
    #[test]
    fn nothing_on_the_marking_surface_claims_a_removal() {
        let marking: Vec<String> = vec![
            panel_intro().to_owned(),
            mark_heading().to_owned(),
            mark_whole_page().to_owned(),
            mark_whole_page_tooltip().to_owned(),
            search_button_tooltip(true).to_owned(),
            search_hint(false).to_owned(),
            search_hint(true).to_owned(),
            marks_count(3),
            mark_remove_tooltip().to_owned(),
        ];
        for line in &marking {
            let lower = line.to_lowercase();
            for claim in ["has been removed", "was removed", "is redacted", "now gone"] {
                assert!(
                    !lower.contains(claim),
                    "`{line}` claims {claim:?} on the MARKING surface, where nothing \
                     has been removed. Marking is reversible; applying is not, and \
                     an operator who believes otherwise ships the marked file"
                );
            }
        }
    }

    /// ★★ **"Verified" appears in exactly one place.**
    ///
    /// Rule 2, and it is a test rather than a doc comment because the word is
    /// the single most valuable one on this surface: it is the difference
    /// between a report and a claim, and it costs nothing to sprinkle it
    /// somewhere it is not earned.
    #[test]
    fn only_the_verification_line_and_the_clean_outcome_say_verified() {
        let everything: Vec<(&str, String)> = vec![
            ("panel_intro", panel_intro().to_owned()),
            ("permanence_statement", permanence_statement().to_owned()),
            ("removal_summary", removal_summary(2, 1, 30, 1)),
            ("single_revision_note", single_revision_note().to_owned()),
            ("residual_heading", residual_heading().to_owned()),
            ("raw_residual_line", raw_residual_line("x")),
            ("scope_reminder", scope_reminder().to_owned()),
            ("confirm_checkbox", confirm_checkbox().to_owned()),
            ("confirm_button", confirm_button().to_owned()),
            ("marks_count", marks_count(2)),
            (
                "applied_with_residuals",
                applied_with_residuals("a.pdf", 2, 1),
            ),
        ];
        for (name, line) in &everything {
            assert!(
                !line.to_lowercase().contains("verif"),
                "`{name}` uses the word \"verified\", which only a clean \
                 AbsenceVerification earns: {line}"
            );
        }
        assert!(verified_line(3).to_lowercase().contains("verified"));
        assert!(
            applied_clean("a.pdf", 2, 1)
                .to_lowercase()
                .contains("verified")
        );
    }

    /// ★ **No post-apply sentence offers Undo.**
    ///
    /// Rule 3. Every other edit in this shell teaches the operator that undo is
    /// available until they save; this is the one moment that expectation is
    /// wrong, so the copy has to correct it rather than merely omit it.
    ///
    /// The marking strings are excluded from the sweep on purpose — they say
    /// "Undo" and **should**, because taking a mark off genuinely is undoable
    /// and telling the operator so is what makes marking feel reversible.
    #[test]
    fn no_post_apply_sentence_mentions_undo_as_a_way_back() {
        for line in [
            permanence_statement().to_owned(),
            applied_clean("a.pdf", 1, 1),
            applied_with_residuals("a.pdf", 1, 1),
            confirm_checkbox().to_owned(),
        ] {
            let lower = line.to_lowercase();
            assert!(
                !lower.contains("undo")
                    || lower.contains("not undo")
                    || lower.contains("not be undone"),
                "a post-apply sentence offers Undo, which is the one place in \
                 pdfce that learned expectation is wrong: {line}"
            );
        }
    }

    /// ★ **A residual outcome and a clean one do not share a sentence.**
    ///
    /// Rule 1 mechanically: the residual form must name the leftover count in
    /// the same sentence as the success, and must not be reachable by softening
    /// the clean form.
    #[test]
    fn the_two_outcomes_read_differently_and_the_residual_one_names_its_count() {
        let clean = applied_clean("survey.pdf", 4, 2);
        let residual = applied_with_residuals("survey.pdf", 4, 2);
        assert_ne!(clean, residual);
        assert!(
            residual.contains("NOT be removed") && residual.contains('2'),
            "the residual outcome must name what is left, in the same sentence \
             as the success: {residual}"
        );
        assert!(
            !clean.contains("could NOT"),
            "the clean outcome must not carry the residual wording: {clean}"
        );
    }

    /// The suggested name can never be the file that was opened.
    ///
    /// The suffix is the mechanism; `crate::dialogs::redact` asserts the
    /// resulting path. This asserts the half that lives in the catalog, in the
    /// shape `crate::text::ocr`'s equivalent test established.
    #[test]
    fn the_suggested_name_differs_from_the_original() {
        assert!(suggested_suffix().starts_with('-'));
        assert!(confirm_button().contains("as…"), "it must read as a prompt");
        assert!(
            !confirm_button().to_lowercase().contains("ok"),
            "the label IS the consequence"
        );
    }

    /// Every refusal says something different, and each names its own cause.
    #[test]
    fn each_named_refusal_says_something_different() {
        use crate::redact::RedactApplyRefusal as R;
        let all = [
            R::NothingToApply,
            R::FullRewriteUnavailable {
                reason: "hybrid".to_owned(),
            },
            R::MaterialisedDocumentUnreadable {
                reason: "bad xref".to_owned(),
            },
            R::CoreRefused {
                reason: "page 2 is an image".to_owned(),
            },
            R::VerificationFailed {
                survivors: vec!["x".to_owned()],
            },
        ];
        let mut seen: Vec<String> = Vec::new();
        for refusal in &all {
            let s = refusal_message(refusal);
            assert!(!s.is_empty());
            assert!(
                !seen.contains(&s),
                "{refusal:?} repeats a sentence another refusal already uses"
            );
            seen.push(s);
        }
        assert!(
            refusal_message(&all[3]).contains("page 2 is an image"),
            "the engine's own diagnosis is the actionable half and must survive"
        );
    }

    /// The census reads as an answer at zero and as a warning above it.
    #[test]
    fn the_census_changes_shape_rather_than_only_its_number() {
        assert!(marks_count(0).contains("No redaction marks"));
        assert!(!marks_count(0).contains('0'));
        assert!(marks_count(3).contains("STILL IN THIS FILE"));
    }
}
