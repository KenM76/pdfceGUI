//! # `app::actions::crossdoc` — pages dragged out of one open document and
//! into another
//!
//! One arm, and it is here rather than in [`super::pages`] because it is the
//! only edit in the application that reads **two documents at once**. Every
//! other verb takes a `&mut OpenDoc` and is done; this one needs a
//! `DocumentView` over a *parked* session while it holds `&mut` on the active
//! one, which is a different shape with a different set of things that can go
//! wrong.
//!
//! ---
//!
//! ## 1. The gesture
//!
//! The operator presses on a page tile in the Pages panel, drags — spring-
//! loading a document tab on the way if the destination is another document
//! ([`crate::app::doctabs`] §3) — and releases over the destination's page
//! list or page view at a caret between two sheets. See [`crate::pagedrag`]
//! for the state that survives the document switch in the middle.
//!
//! ## 2. ★ It is a COPY, and the reason is undo
//!
//! [`crate::text::doctabs::drag_landing_other`] carries the argument in the
//! words the operator reads. The engineering form:
//!
//! A cross-document *move* is two edits — an insert into the target and a
//! delete from the source — recorded on **two independent undo stacks**,
//! because `EditSession` owns one command log per document and this
//! application has one session per document. There is no ordering of those two
//! commands under which a single Ctrl+Z means *"undo what I just did"*: undo
//! goes to whichever document has focus, so the operator gets half of their
//! gesture reversed and no indication that the other half is still applied.
//!
//! Half-undone is worse than not-undone, and much worse on a drawing set,
//! where the evidence is a page count nobody checks.
//!
//! Windows Explorer reaches the same conclusion from a different direction and
//! copies between volumes by default. Acrobat's Insert Pages is a copy. So this
//! is a copy, and the caption says so before the operator releases the button.
//!
//! ## 3. What the source document is guaranteed
//!
//! **Nothing is written to it and nothing is read out of it destructively.**
//! The engine takes a `DocumentView` — a read-only projection — and copies
//! every object it needs at fresh object numbers in the *target*. The source's
//! `EditSession` is not borrowed mutably, its undo stack is untouched, its
//! `is_modified` answer does not change, and its tab does not acquire the
//! unsaved marker.
//!
//! That is worth stating because it is the property that makes the gesture
//! safe to try. An operator who drags the wrong sheet has changed one document
//! and can undo it there.
//!
//! ## 4. What does not come across, and why the operator is told at the moment
//! it happens
//!
//! Exactly what [`super::pages::insert_from_view`] reports for an insert from
//! a file, through the same [`crate::text::pages::inserted`] sentence: page
//! content, resources, fonts and XObjects arrive; the source's **document-level**
//! structures — outlines, the AcroForm field tree, named destinations, page
//! labels — do not, because merging those rewrites objects an incremental save
//! exists in order not to touch.
//!
//! R8b rule 4's surviving half is the reason this is disclosed rather than
//! left to be discovered: *"inferences the operator cannot see … still owe an
//! off-canvas report"*. A form field whose widget arrived without its
//! definition looks exactly like a form field until it is filled in.

use crate::app::PdfceApp;
use crate::app::state::Status;

impl PdfceApp {
    /// `Action::InsertPagesFromOpenDocument` — take `pages` out of the
    /// document in tab position `source_slot` and put copies of them into the
    /// document on screen, at `position`.
    ///
    /// **The target is always the active document**, and is not carried by the
    /// action. That is not an omission: the drop landed on a surface, the
    /// surface was showing the active document, and a slot carried from the
    /// press would name whatever was active when the *drag started* — which,
    /// with spring-loading, is precisely the document it is not.
    ///
    /// # The three ways it declines, and why each is silent or spoken
    ///
    /// | condition | what happens |
    /// |---|---|
    /// | the source tab has gone, or never held an open document | traced, nothing said — unreachable without a close mid-drag, and there is no remedy to offer |
    /// | nothing is open to drop into | traced, nothing said — same |
    /// | the source **is** the target | traced and **refused**; a same-document drag is a reorder and reaches a different arm entirely |
    /// | the engine refuses the insert | `vector_edit`'s own decline path, which puts the engine's reason on the status row |
    pub(super) fn apply_insert_from_open_document(
        &mut self,
        source_slot: usize,
        pages: &[usize],
        position: pdfce_core::pageops::InsertPosition,
    ) {
        if source_slot == self.active_slot {
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "page-drop-refused slot={source_slot} reason=same-document"
                )
            });
            return;
        }

        // ★ The parked index, which is NOT the slot.
        //
        // `crate::app::documents` §1: `parked` holds the open documents in tab
        // order **with the active one removed**, so every slot above the
        // active one is one place earlier in the vector. Getting this wrong
        // inserts the wrong document's pages, silently and plausibly, which is
        // why it is written once here rather than at each of the two borrows
        // below.
        let parked_index = if source_slot < self.active_slot {
            source_slot
        } else {
            source_slot.wrapping_sub(1)
        };

        // Two disjoint FIELD borrows, taken as two statements. `self.parked`
        // and `self.status` are different fields, so the borrow checker splits
        // them; routing either through `self.slot(..)` — a method on `&self` —
        // would borrow the whole application and make the second borrow
        // impossible. `crate::app::documents`' header explains why the
        // encoding is two fields at all, and this is the one place that
        // benefits from it.
        let Some(Status::Open(source)) = self.parked.get(parked_index) else {
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "page-drop-refused slot={source_slot} reason=source-not-open"
                )
            });
            return;
        };
        let view = source.session.view();
        let source_path = source.path.clone();

        let Status::Open(target) = &mut self.status else {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "page-drop-refused reason=no-target".to_owned()
            });
            return;
        };

        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "page-drop from={source_path:?} pages={} into={:?} position={position:?}",
                pages.len(),
                target.path,
            )
        });

        // Everything past this point is an ordinary insert, and deliberately
        // the *same* one an Insert-from-file performs. See that function's
        // docs for the disclosure and for why the view goes to what arrived.
        super::pages::insert_from_view(target, &view, pages, position);
    }
}
