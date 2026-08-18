//! # `app::actions::annots` — the verbs that change an annotation
//!
//! Split out of [`super::apply`] under **R2** on 2026-08-18, when annotation
//! selection landed and took that file past 1,500 lines. The seam is the one
//! [`super::pages`] already draws next door: *what class of thing does this
//! verb act on?* — pages there, annotations here, page **content** in `apply`.
//!
//! ## Why it is worth its own file today rather than when it is bigger
//!
//! Because it is about to get bigger, and for a reason that is already
//! scheduled. `EditSession::set_markup_style` shipped on 2026-08-18 —
//! colour, interior, width, opacity and arrowheads on an existing annotation,
//! keeping its object id — and the Format contextual tab is the surface for
//! it. Every one of those becomes a verb in here.
//!
//! ★ And each of them will carry the same routing obligation `delete` does
//! not: a **ce dimension** is a `/Line` with `/IT /LineDimension`, it passes
//! every "markup pdfce can author" test, and restyling one through
//! `set_markup_style` regenerates it as a bare line with its label and witness
//! lines gone. `pdfce-core` refuses it by name and points at
//! `set_dimension_style`. `canvas::selection::annot::AnnotKind` carries the
//! distinction on the selected target precisely so that routing is a `match`
//! the compiler checks — see its header.
//!
//! ## What is NOT here
//!
//! **Placing** an annotation. `Action::CommitMarkup`, `CommitTextAnnot` and
//! the measure commits stay in `apply`, because their subject is the *gesture*
//! that authored them rather than the annotation afterwards. The line is the
//! same one `pages` draws: this file is what happens to a thing that already
//! exists.

use pdfce_core::object::ObjId;

use crate::app::state::OpenDoc;

/// **Remove one annotation from the document.**
///
/// Reached from `format.delete` and from the canvas's Delete key, both only
/// while an annotation is selected.
///
/// # Why it goes through `vector_edit` like everything else
///
/// So the undo entry, the epoch bump, the cache invalidation and the
/// disclosure happen the one way they happen for every other document change.
/// The closure returns the disclosure list, which is where the **collateral**
/// goes: the operator named one annotation and the engine may legitimately
/// have removed or altered more — a `/Popup` companion (§12.5.6.14 is a
/// `shall`), replies orphaned, group members promoted.
///
/// # `page` is for the message, not for the verb
///
/// `delete_annotation` finds the annotation by id wherever it lives, and it
/// has to: a reply may sit on a different page from the comment it replies to,
/// so a page-scoped delete would miss it.
///
/// # ★ This is not redaction
///
/// It removes an entry from `/Annots`. It does not touch page content, and an
/// incremental save leaves the previous revision in the file.
/// `docs/core-api/03-capabilities.md` §3.4 states that rule, and
/// [`crate::text::markup::deleted_collateral`] observes it in the wording it
/// chooses — never "removed".
pub(super) fn delete(doc: &mut OpenDoc, page: usize, id: ObjId) {
    super::apply::vector_edit(doc, "delete-annotation", page, 1, |session| {
        session.delete_annotation(id).map(|report| {
            crate::text::markup::deleted_collateral(
                report.popup_removed,
                report.parent_popup_cleared,
                report.replies_orphaned,
                report.group_members_promoted,
            )
            .into_iter()
            .collect()
        })
    });
    // The selection named an object that no longer exists. Cleared here rather
    // than left for the next frame to notice: an outline around a deleted
    // annotation promises that a second Delete would do something, and the
    // second Delete would refuse.
    doc.selection.clear_annot();
}
