//! # `app::actions::export` — writing part of the document out as something
//! else
//!
//! ## Why this is its own file
//!
//! The sixth sibling of [`super::apply`], drawn along the same seam the other
//! five are: *what class of thing does this verb act on?* — pages there,
//! annotations in `annots`, the dimensioning model in `dimensions`, page
//! content in `apply`, redaction marks in `redact`. This is **what leaves the
//! document**.
//!
//! It is a real subject rather than a size-driven cut, and the evidence is the
//! property every verb here shares and no verb elsewhere does: **none of them
//! changes the document at all.** No `vector_edit`, no undo entry, no epoch
//! bump, no cache invalidation. They read the open file and write a different
//! one, which makes every rule the mutation funnel enforces irrelevant to them
//! and every rule about *file* handling — a save picker, an overwrite, a
//! partial write — apply instead.
//!
//! `super::pages::extract` is the same shape and stayed in `pages` because its
//! subject is a page set. If a third export lands, that is the moment to move
//! it here.
//!
//! ## ★ Why an export is an `Action` at all
//!
//! `super::apply`'s header answers it for `SaveCopy` and the answer is the same
//! here: **a native file dialog must not open inside a layout pass.** It is a
//! modal OS window that blocks the thread, and opening one from a widget's
//! `clicked()` branch means egui is part-way through building a frame that will
//! not finish until the operator has answered.
//!
//! Nothing about the document is being ordered — there is nothing to order —
//! so the funnel's *invariant* does not apply. Its **reason** does.

use crate::app::state::OpenDoc;

/// Write one page's vector geometry as an ASCII DXF.
///
/// ## What this owes the operator, and why it is not optional
///
/// `DxfOutcome` is the disclosure half, and two of its counts are the reason
/// this feature is worth having over any generic converter:
///
/// - **`skipped_images`** — DXF has no raster entity, so a picture on the page
///   is simply not in the file. The engine's own words for why that must be
///   said: *"an operator whose drawing was half annotation gets a DXF that
///   looks like the geometry went missing, and 'the labels are not in this
///   file' is a sentence they need **before** they open it in SOLIDWORKS, not
///   after."*
/// - **`unreadable_text`** — text pdfce could not decode, kept apart from
///   `skipped_text` (which the operator asked for) because one is a choice and
///   the other is a fact about the source PDF. Rolling them together would let
///   the second hide inside the first.
///
/// ## ★ Why the geometry is fetched here and not carried in the action
///
/// `PageObjects` is a decomposition of a whole page — every path, every text
/// run, every image placement — and the shell already holds one, cached, keyed
/// on `(page, epoch)`. Carrying it through the action queue would clone it for
/// a value the apply phase can borrow, and a **stale** clone at that: the queue
/// drains after the frame, and an edit raised earlier in the same frame would
/// leave the export describing the page as it was.
///
/// Fetching it here means the export sees the document as it stands when the
/// export runs, which is the only reading that can be defended.
pub(super) fn dxf(doc: &mut OpenDoc, page: usize, options: &pdfce_core::export::dxf::DxfOptions) {
    // The decomposition, from the cache the canvas and the Objects panel share.
    // `None` is reachable — a page still being read, or one whose content
    // streams could not be resolved — and is a decline rather than a failure:
    // nothing was written and the sentence says which.
    let Some(dxf) = doc
        .page_objects()
        .map(|provider| pdfce_core::export::dxf::write_dxf(provider.page_objects(), options))
    else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!("export-dxf-declined page={page} reason=no-decomposition")
        });
        super::record_note(
            doc.edit_epoch,
            crate::text::export_dxf::no_geometry().to_owned(),
        );
        return;
    };
    let (text, outcome) = dxf;

    // ★ The picker AFTER the write, not before.
    //
    // The write is pure and cannot fail — `write_dxf` returns no `Result`, and
    // its doc says why: *"the writer cannot fail on well-formed input, and
    // malformed input is skipped and counted rather than refused."* So doing it
    // first costs nothing and buys the property that matters: the operator is
    // never asked where to put a file that turns out to be empty. If a future
    // slice gives the writer a refusal, this ordering is what lets the refusal
    // be reported before a save dialog has been opened.
    let suggested = suggested_path(doc);
    let crate::app::files::Picked::Path(target) =
        crate::app::files::pick_save_path(&suggested, crate::text::export_dxf::save_dialog_title())
    else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!("export-dxf-cancelled page={page}")
        });
        return;
    };

    match std::fs::write(&target, text.as_bytes()) {
        Ok(()) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "export-dxf page={page} bytes={} polylines={} circles={} arcs={} \
                     splines={} skipped_text={} skipped_images={} unreadable_text={}",
                    text.len(),
                    outcome.polylines,
                    outcome.circles,
                    outcome.arcs,
                    outcome.splines,
                    outcome.skipped_text,
                    outcome.skipped_images,
                    outcome.unreadable_text
                )
            });
            // ★ Recorded through `record_note` rather than returned from a
            // `vector_edit` closure, because there is no edit to ride in on —
            // the same case `canvas::interact` records for a caret that cannot
            // be placed. Stamped with the CURRENT epoch, so the sentences stand
            // until the next real edit moves past them.
            //
            // The list is joined rather than recorded one at a time: the slot
            // holds one disclosure, and the last writer would win.
            let notes = crate::text::export_dxf::exported(&target.display().to_string(), &outcome);
            super::record_edit_disclosure(Some(super::EditDisclosure {
                epoch: doc.edit_epoch,
                notes,
            }));
        }
        Err(error) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!("export-dxf-failed page={page} detail={error}")
            });
            super::record_note(
                doc.edit_epoch,
                crate::text::export_dxf::export_failed(&error.to_string()),
            );
        }
    }
}

/// Where the save dialog opens, and what it calls the file.
///
/// Beside the document, named after it, with a `.dxf` extension. The same rule
/// `super::pages::suggested_path` follows for an extract, and for its reason: a
/// picker that opens in the last-used directory of some other application is a
/// picker that makes the operator navigate back to their own project every
/// time.
fn suggested_path(doc: &OpenDoc) -> std::path::PathBuf {
    let mut path = doc.path.clone();
    let stem = path
        .file_stem()
        .map_or_else(|| "export".to_owned(), |s| s.to_string_lossy().into_owned());
    path.set_file_name(stem);
    // `set_extension` rather than pushing a string: a document called
    // `plan.rev2.pdf` has a stem of `plan.rev2`, and appending would produce
    // `plan.rev2.dxf` either way — but a document with no extension at all
    // would gain one only through this call.
    path.set_extension("dxf"); // ui-text-exempt: a file extension, never displayed as prose
    path
}
