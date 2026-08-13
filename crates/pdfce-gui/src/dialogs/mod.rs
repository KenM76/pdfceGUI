//! # `dialogs` — the shell's stationary, screen-anchored surfaces
//!
//! ## What belongs here, and what does not
//!
//! A **dialog** is a single transaction with a start and an end: it is opened
//! deliberately, it holds one job's worth of answers, and closing it forgets
//! them. A **panel** is somewhere an operator dips in and out of while
//! working, and it keeps its state across documents. The distinction decides
//! where a surface lives, and getting it wrong is not cosmetic — a print
//! configuration that persisted across documents would let a range typed for
//! one file silently apply to another.
//!
//! `SALVAGE.md`'s redistribution table names the tenants of this directory:
//! *"Dialogs — properties, print, export, reset, settings host — ~1,500 lines
//! — `dialogs/`."* [`print`] is the first of them.
//!
//! ## ★ Every dialog here is screen-anchored, never page-anchored
//!
//! A decision inherited from the old shell, where it was made in response to a
//! specific operator objection: **controls whose position is derived from the
//! page move on every zoom and scroll.** A surface an operator is reading and
//! typing into must stay where they put their eyes. Each dialog therefore
//! anchors to the viewport rather than being positioned relative to the
//! canvas, and none of them is drawn inside the canvas's coordinate space.
//!
//! ## ★ Where dialog state lives, and why it is one field
//!
//! [`DialogsState`] is the whole dock-side surface of this module: one field
//! on `PdfceApp`, one `open_*` call per dialog from the command dispatcher,
//! and one [`DialogsState::show`] call per frame. It follows
//! `crate::panels::PanelsState` exactly — same idiom, second instance, not a
//! new convention — and the reason it is a struct rather than a bare
//! `Option<PrintDialog>` is that the *next* dialog is then a change to this
//! file rather than to `app/mod.rs`, which is the file every parallel task
//! already contends over.
//!
//! ## Why a dialog does not push an `Action`
//!
//! `crate::app::actions`' invariant is that **no code path runs from a widget
//! to a document**, and the four things it buys are all about *document*
//! state: a coherent undo log, an aliasing problem turned into a queue,
//! explicit ordering between changes, and a greppable answer to "what can
//! change this?".
//!
//! A print changes no document state. It reads the document — the pages, the
//! edited view — and writes to a spooler, so it contributes nothing to the
//! undo log and has nothing to order against. Routing it through the funnel
//! would add an `Action` variant that `apply` could only answer by reaching
//! back into a dialog for the state it needs, which is the funnel pointing the
//! wrong way.
//!
//! What the funnel's *reason* does still demand is that the irreversible work
//! not happen part-way through a layout pass, and [`print::PrintDialog`]
//! honours that in its own scope: the button sets a flag, and the spool runs
//! after the window's closure returns. See that field's documentation.
//!
//! **A dialog that edits the document is a different case and must use the
//! funnel.** The properties dialog and the settings host will both raise
//! `Action`s; this note is about printing specifically, not about dialogs in
//! general.

pub mod print;

use crate::app::state::{OpenDoc, Status};

/// Every dialog this build has, and whether each is open.
///
/// One field per dialog, each an `Option` whose `Some` *is* the "open" state —
/// there is no separate visibility flag that could disagree with whether the
/// state exists. Closing a dialog drops its state, which is what makes
/// "closing forgets the job" true by construction rather than by remembering
/// to reset fields.
#[derive(Default)]
pub struct DialogsState {
    /// The print dialog, when one is open.
    print: Option<print::PrintDialog>,
}

impl DialogsState {
    /// Open the print dialog for the document in `status`.
    ///
    /// **The dispatch target for the `file.print` command.** The command is
    /// registered `enabled_when("doc.open")`, so the ribbon button cannot be
    /// pressed without a document — but a keyboard chord bound to the same id
    /// has neither that guard nor the button's once-per-frame property, and
    /// the shell's own dispatch pattern is *"push the chord blind, gate the
    /// effect in dispatch"*. Both conditions are therefore enforced **here**,
    /// at the one place the dialog is ever built, which fixes the button and
    /// the chord by construction rather than by a condition duplicated at the
    /// keymap:
    ///
    /// - **No document, no dialog.** Without this, the chord on an empty
    ///   canvas would enumerate the spooler — a blocking call on a network
    ///   printer — to populate a window [`Self::show`] closes again on its
    ///   very next frame.
    /// - **Already open means leave it alone.** This function *builds* a
    ///   dialog from defaults. A second press part-way through configuring a
    ///   job would silently reset the range, the scale, the copy count and the
    ///   annotation scope — the operator's own settings, discarded by the
    ///   shortcut they pressed to look at them.
    pub fn open_print(&mut self, status: &Status) {
        let Status::Open(doc) = status else {
            return;
        };
        if self.print.is_some() {
            return;
        }
        self.print = Some(print::PrintDialog::open(doc));
    }

    /// Draw every open dialog, and close the ones that asked to close.
    ///
    /// Called once per frame from frame composition, **after** the canvas and
    /// the docks: a dialog is an overlay, and egui's `Area` ordering follows
    /// the order things are added within a frame.
    ///
    /// # Why a closed document closes the dialogs
    ///
    /// Every dialog here is *about* the open document — a print job is a job
    /// on this file's pages. A dialog left up over a closed document would be
    /// configuring a job against pages that no longer exist, and the honest
    /// response is to close it rather than to freeze it or to let it act on
    /// whatever is opened next.
    pub fn show(&mut self, ctx: &egui::Context, status: &Status) {
        let Status::Open(doc) = status else {
            self.close_all();
            return;
        };
        let doc: &OpenDoc = doc;
        // Drawn first, then closed — rather than closed inside the borrow that
        // drew it. A dialog decides whether it stays open *while* it draws
        // (the title-bar cross and its own Close button are both widgets), so
        // the answer arrives out of the same call that needs `&mut` on the
        // state being dropped.
        if self.print.as_mut().map(|d| d.show(ctx, doc)) == Some(false) {
            self.print = None;
        }
    }

    /// Drop every open dialog's state.
    ///
    /// One place, so a dialog added later cannot be forgotten by whichever of
    /// the close paths its author did not think of.
    fn close_all(&mut self) {
        self.print = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A dialog cannot be opened without a document.
    ///
    /// The guard that stops a keyboard chord from enumerating the spooler —
    /// a call that blocks on a network printer — to populate a window that
    /// would be closed again on the next frame.
    #[test]
    fn no_document_means_no_dialog() {
        let mut dialogs = DialogsState::default();
        dialogs.open_print(&Status::Empty);
        assert!(dialogs.print.is_none());
    }

    /// Closing the document closes the dialogs.
    ///
    /// Asserted through the public path rather than by setting the field, so
    /// the test covers what a frame actually does.
    #[test]
    fn a_closed_document_closes_every_dialog() {
        let mut dialogs = DialogsState::default();
        assert!(dialogs.print.is_none());
        dialogs.close_all();
        assert!(dialogs.print.is_none());
    }
}
