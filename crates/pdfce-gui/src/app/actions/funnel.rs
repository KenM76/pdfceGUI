//! # `app::actions::funnel` — **the one place an edit passes through**
//!
//! `vector_edit`, and nothing else. Every verb in this shell that changes a
//! document calls it, and what it owns is the four-step protocol each of them
//! would otherwise write out again: cancel the render worker, take the session,
//! run the edit, and on success bump the epoch, invalidate the caches and carry
//! the disclosures to the status row.
//!
//! ## Why it is its own file
//!
//! R2, and the seam is real rather than convenient. `super::apply` answers
//! *"which module handles this action?"* — it is a router, and it grows when a
//! verb is added. This answers *"what happens around any edit?"* — it is a
//! protocol, and it grows when the protocol changes, which is roughly never.
//!
//! Two subjects, two rates of change. `apply.rs` crossed 1,500 lines on
//! 2026-08-30 when the third redaction-marking route arrived, and the honest
//! response was not to shorten a comment: it was to notice that the file held a
//! router *and* the thing every router arm calls.
//!
//! ## ★ Nothing moved but its address
//!
//! The function, its documentation and its generic bound are unchanged. It is
//! still `pub(super)`, so every call site still says `super::apply::vector_edit`
//! — no, it says `apply::vector_edit` from a sibling and that path still
//! resolves, because `apply` re-exports it. Callers did not learn this file
//! exists.

// ★ Listed rather than glob-imported from `super::apply`, and the difference is
// worth knowing: a `use` is a PRIVATE import, so `use super::apply::*` brings
// across that module's public items and none of the names it imported for
// itself. `canvas::present` could use a glob because it moved to a CHILD of the
// module it left; this moved to a SIBLING.
use std::sync::Arc;

use pdfce_core::edit::EditSession;

use super::{EditDisclosure, record_edit_disclosure};
use crate::app::state::OpenDoc;

pub(super) fn vector_edit<E: std::fmt::Display>(
    doc: &mut OpenDoc,
    label: &str,
    page: usize,
    operands: usize,
    edit: impl FnOnce(&mut EditSession) -> Result<Vec<String>, E>,
) {
    doc.render_worker.cancel_and_wait();
    let Some(session) = Arc::get_mut(&mut doc.session) else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("{label}-refused page={page} n={operands} reason=session-borrowed")
        });
        return;
    };
    match edit(session) {
        Ok(disclosures) => {
            doc.edit_epoch = doc.edit_epoch.wrapping_add(1);
            // ★★ Stamped in the SAME statement group as the bump, because the
            // two are one fact — *the document changed, at this moment* — and a
            // second place that set one without the other would produce a
            // "catching up" line measured from the wrong edit. `OPERATOR_REQUESTS.md`
            // O63; see `OpenDoc::page_is_catching_up`.
            doc.last_edit_at = Some(std::time::Instant::now());
            // ★ The texture is NOT dropped here — the fix for 2026-08-18's
            // *"the page goes blank and flashes after every change."*
            //
            // `doc.page_texture = None` did two jobs: it made `render::settle`
            // notice the edit, and it took the picture off the screen. Only the
            // first was wanted; the second put an empty page in front of the
            // operator between every edit and its raster.
            //
            // `OpenDoc::page_texture_epoch` now carries the third term the
            // strip cache always had, so settle gets its "no" from the epoch
            // and the stale raster stays up until the new one lands — which
            // `OpenDoc::rasterize`'s docs already promised for a slow render.
            //
            // A page-SET change is different: there the stale raster is a
            // picture of another sheet, and `pages::resync` drops it on exactly
            // that condition.
            // ★ **Step 5, added when the page verbs landed** — see
            // `super::pages`' header, which carries the whole argument and the
            // table of what each kind of edit invalidates.
            //
            // Here rather than in the four page arms, because `Action::Undo`
            // and `Action::Redo` come through this same function and run those
            // same engine commands **backwards**: an undone page delete puts
            // sheets back, and an arm-side resync could not see it. This is the
            // one place every document change already passes through, which is
            // `HANDOFF.md` §6's rule applied to a consequence rather than to a
            // dispatch.
            //
            // It is self-describing rather than told — it compares the page
            // vector it has against the one the session now reports — so an
            // edit that touched no page costs one page-tree walk and one `Vec`
            // comparison, per operator gesture, and does nothing else.
            super::pages::resync(doc);
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "{label} page={page} n={operands} epoch={} disclosures={}",
                    doc.edit_epoch,
                    if disclosures.is_empty() {
                        // ui-text-exempt: diagnostic trace, never displayed in the UI
                        "none".to_owned()
                    } else {
                        disclosures.join(" | ")
                    }
                )
            });
            // ★ Surfaced as well as traced — see this function's "The
            // disclosures" section. Stamped with the epoch bumped above: the
            // revision on screen from now until the next edit, so an undo
            // retires the sentence by moving past it.
            //
            // AFTER the trace, which is what lets the list travel by MOVE
            // rather than by clone: `crate::diag::trace` runs its closure only
            // when `PDFCE_DIAG` is set, and that closure only *borrows*
            // `disclosures` to join it. Recording first would have meant
            // cloning a vec on every edit to keep both readers fed.
            record_edit_disclosure(if disclosures.is_empty() {
                // The overwhelmingly common case: the surgery expressed the
                // operator's request without changing anyone's form, so there
                // is nothing to disclose and the previous edit's sentence —
                // already stale by its epoch — is dropped outright.
                None
            } else {
                Some(EditDisclosure {
                    epoch: doc.edit_epoch,
                    notes: disclosures,
                })
            });
        }
        // A refusal is the engine's, and it is structured. Reporting it and
        // leaving the document alone is still the whole response here — and
        // as of 2026-08-14 that is a *scope* statement rather than the "there
        // is nowhere to say it" this comment used to make. There is now
        // somewhere: `app::status` draws the `Ok` arm's disclosure list.
        //
        // A refusal is deliberately not routed to it, because the two are
        // different acts. A disclosure is **after the fact** — the edit
        // happened, and the operator is owed the part they cannot see. A
        // refusal is a **decline**: nothing happened, and the sentence has to
        // arrive while the operator still believes it did. Sharing one slot
        // would mean an undone gesture and a completed one wearing the same
        // wording in the same place, which is worse than the trace-only state
        // it replaced. That is `FEATURES.md`'s "Worded decline" row, which
        // wants its own decision about wording and placement; this arm is
        // where it lands when it is taken.
        //
        // Note also that `EditError` is `Display` output — diagnostic prose an
        // error writes about itself — and `check-ui-strings.sh`'s exclusion 3
        // says in as many words that this exclusion "is not permission to
        // route UI text through an error type". So wording a decline is
        // catalog work in `text/`, not a `format!` of this value.
        Err(error) => crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("{label}-refused page={page} n={operands} detail={error}")
        }),
    }
}

// ---------------------------------------------------------------------------
// Undo and redo — one function, one direction parameter
// ---------------------------------------------------------------------------
