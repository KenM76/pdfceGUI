//! # `app::actions::redact` — the three arms that mark content for removal
//!
//! Split out of [`super::apply`] on 2026-08-18 under rule R2, and the seam is a
//! real one rather than a line count.
//!
//! These are the only arms whose subject is **marking content for removal**.
//! They share a vocabulary nothing else in the funnel uses — `RedactAppearance`,
//! the mark census, the annotation ids a review surface addresses a mark by —
//! and their comments carry the argument for the one operation pdfce cannot
//! undo. Moving the arms and leaving the reasoning behind would have been
//! exactly the split this project's own R2 note warns against.
//!
//! ## ★ What is NOT here, and that is the point
//!
//! **Nothing in this file removes anything.** Marking is the reversible half of
//! redaction; the irreversible half is `crate::dialogs::redact`, which reaches
//! no arm in this funnel at all — it changes no document through the queue, so
//! it has nothing to order against and no epoch to bump. Routing the one
//! operation that cannot be undone through a queue that replays would be the
//! defect, not the tidiness.

use super::Action;
use super::apply::vector_edit;
use crate::app::state::OpenDoc;

/// Apply one of the three marking actions.
///
/// Takes the whole `Action` rather than destructured fields, so the match here
/// is the same shape as the one it was lifted out of and a reader comparing
/// them sees one dispatch rather than two spellings of it.
///
/// # Panics
///
/// Never. The `_` arm is unreachable — `super::apply` routes only the three
/// redaction variants here — and it is spelled rather than `unreachable!()`
/// because a future fourth variant sent here by mistake should do nothing
/// visible rather than end the process an operator is mid-edit in.
pub fn apply(doc: &mut OpenDoc, action: Action) {
    match action {
        // ===============================================================
        // ★ THE REDACTION MARKING VERBS
        //
        // Three arms, each one call, through the same `vector_edit` funnel
        // every other document change uses — which is the whole reason they
        // are one line each. Marking is an ordinary edit: it authors an
        // annotation, the engine records it as an undoable command, and the
        // page has to re-raster because a `/Redact` mark draws a red
        // outline the operator needs to see.
        //
        // ★ **Nothing here removes anything.** The irreversible half is
        // `crate::dialogs::redact`, which reaches no arm in this file at
        // all: it changes no document, so it has nothing to order against
        // and no epoch to bump, and routing it through here would put the
        // one operation that cannot be undone into a queue that replays.
        //
        // `.map(|_| Vec::new())` on the first two adapts the engine's
        // `Vec<ObjId>`/`ObjId` to the disclosure list `vector_edit` traces,
        // and the empty vec is a statement rather than a placeholder —
        // authoring an annotation rewrites no existing operator, so nothing
        // changed form and rule 4 owes the operator nothing. It is the same
        // adaptation `CommitMarkup` makes one screen up.
        // ===============================================================
        Action::MarkRedactionsBySearch {
            query,
            pattern,
            appearance,
        } => {
            if !query.is_empty() {
                let page = doc.view.page_index;
                // The label distinguishes the two marking modes on the
                // trace, because a pattern that marked nothing and a
                // literal that marked nothing are different diagnoses:
                // one is a query the document does not contain, the other
                // is very often a `#` the operator meant literally.
                let label = if pattern {
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "redact-mark-pattern"
                } else {
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "redact-mark-search"
                };
                let before = crate::panels::redact::mark_ids(&doc.session).len();
                vector_edit(doc, label, page, 1, |session| {
                    // ★ Case-INSENSITIVE, always, and it is not a missing
                    // control. Over-marking is the safe direction of error
                    // on this verb and under-marking is not: a mark the
                    // operator did not want is one row and one click in the
                    // review list, and a mark they did want and did not get
                    // is a name shipped in a document they believe is
                    // redacted. The old shell made the same ruling in the
                    // same words.
                    //  The `_styled` verbs, which arrived on 2026-08-17
                    // (`a7210a4`) in answer to this shell's filing: before
                    // them `author_text_matches` built its spec internally
                    // with `fill: None`, so a fill the operator chose was
                    // discarded on this path and honoured on the whole-page
                    // one. A control honoured on some marks and silently
                    // dropped on others is worse than no control, on the
                    // one operation that cannot be undone.
                    if pattern {
                        session.mark_redactions_by_pattern_styled(&query, true, &appearance)
                    } else {
                        session.mark_redactions_by_search_styled(
                            &query,
                            &pdfce_core::edit::TextSearchOptions::default()
                                .with_case_insensitive(true),
                            &appearance,
                        )
                    }
                    .map(|_| Vec::new())
                });
                // ★ Reported AFTER the edit, from the same census the panel
                // lists from, so the number on the trace and the number of
                // rows on screen cannot disagree. `created=0` is the
                // interesting value: it is a search that found nothing,
                // which on a scanned page is the named real-world failure
                // — `crate::text::redact::search_hint` is the sentence that
                // warns about it, and this is how a reader of a trace sees
                // it happen.
                let after = crate::panels::redact::mark_ids(&doc.session).len();
                crate::diag::trace(|| {
                    format!(
                        // ui-text-exempt: diagnostic trace, never displayed in the UI
                        "redact-marked mode={} created={} total={}",
                        if pattern { "pattern" } else { "literal" },
                        after.saturating_sub(before),
                        after
                    )
                });
            }
        }
        Action::MarkPageForRedaction { page, appearance } => {
            // Resolved here rather than carried on the action because the
            // rectangle is the page's, not the operator's — see the
            // variant's docs. A page index past the end is unreachable from
            // the panel and is answered rather than indexed, because an
            // action is plain data a test can build.
            if let Some(spec) = doc
                .pages
                .get(page)
                .map(|p| crate::panels::redact::whole_page_spec(p, &appearance))
            {
                vector_edit(doc, "redact-mark-page", page, 1, |session| {
                    session.add_redaction(page, &spec).map(|_| Vec::new())
                });
            } else {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    format!("redact-mark-page-declined page={page} reason=no-such-page")
                });
            }
        }
        Action::RemoveRedactionMark { annot_id } => {
            let page = doc.view.page_index;
            vector_edit(doc, "redact-unmark", page, 1, |session| {
                session.delete_redaction_mark(annot_id).map(|()| Vec::new())
            });
        }
        _ => {}
    }
}
