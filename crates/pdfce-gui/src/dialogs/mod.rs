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

pub mod about;
/// The render report `tools.render_diagnostics` opens — what the renderer did
/// with the page currently on the canvas, with the room the status bar's one
/// elided line does not have.
pub mod diagnostics;
pub mod ocr;
pub mod print;

use crate::app::state::{OpenDoc, Status};

/// Every dialog this build has, and whether each is open.
///
/// One field per dialog, each an `Option` whose `Some` *is* the "open" state —
/// there is no separate visibility flag that could disagree with whether the
/// state exists. Closing a dialog drops its state, which is what makes
/// "closing forgets the job" true by construction rather than by remembering
/// to reset fields.
///
/// ## ★ The fields are in two groups, and the split is load-bearing
///
/// A **document-scoped** dialog is about the open file: a print job is a job
/// on *these* pages. An **application-scoped** dialog is about pdfce itself
/// and is meaningful with nothing loaded.
///
/// Until 2026-08-14 every dialog here was document-scoped and
/// [`DialogsState::show`] could take the shortcut of dropping all of them the
/// moment the document went away. [`about::AboutDialog`] broke that: an
/// operator who has just launched pdfce and wants to know what version they
/// are running, or under what terms, has no document — and a control that did
/// nothing in that state would be the placeholder `HANDOFF.md` §6 forbids.
///
/// So the two groups are drawn separately rather than the rule being softened
/// for everything. Print still closes with its document; About does not, and
/// cannot be made to without breaking the command that opens it.
#[derive(Default)]
pub struct DialogsState {
    // --- document-scoped: closed when the document closes -----------------
    /// The print dialog, when one is open.
    print: Option<print::PrintDialog>,

    /// The Recognise-text dialog, when one is open.
    ///
    /// Document-scoped, and firmly so: a recognition is of one page of one
    /// file. ★ It is the first dialog here that can hold **unsaved bytes**, and
    /// closing the document discards them — which is the right answer rather
    /// than a loss. Writing them afterwards would produce a file derived from a
    /// document the operator has already put away, and offering to do that is
    /// how a program ends up with two ideas about what "the document" means.
    ocr: Option<ocr::OcrDialog>,

    /// The Render-diagnostics report, when one is open.
    ///
    /// Document-scoped: it describes *this page of this file*, and a window
    /// left up over a closed document would be reporting measurements of a
    /// raster that no longer exists. It holds no configuration, so closing it
    /// forgets nothing — but it must still close, for the same reason print
    /// does.
    diagnostics: Option<diagnostics::DiagnosticsDialog>,

    // --- application-scoped: survives an empty canvas ---------------------
    /// The About dialog, when one is open.
    ///
    /// Carries the attribution surface — see [`about`] and
    /// [`crate::text::about`] for why a shipped `LICENSE` file is not enough
    /// once a CC-BY-SA-4.0 asset is in the package.
    about: Option<about::AboutDialog>,
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

    /// Open the Recognise-text dialog for the document in `status`.
    ///
    /// **The dispatch target for the `file.ocr` command**, and it applies the
    /// same two guards [`Self::open_print`] documents, for the same two
    /// reasons: the ribbon control is gated on `doc.pages` and a chord bound to
    /// the same id is not, so both are fixed here at the one place the dialog
    /// is built.
    ///
    /// The already-open guard is the stronger of the two here. A second press
    /// while a recognition is running would abandon a live worker thread and
    /// start another beside it, and a second press *after* one finished would
    /// discard recognised bytes the operator has not saved yet — several
    /// seconds of work and an unwritten document, thrown away by the shortcut
    /// they pressed to look at it.
    pub fn open_ocr(&mut self, status: &Status) {
        if self.ocr.is_some() {
            return;
        }
        self.ocr = ocr::open_for(status);
    }

    /// Open the Render-diagnostics report for the document in `status`.
    ///
    /// **The dispatch target for the `tools.render_diagnostics` command**, and
    /// it applies the same two guards [`Self::open_print`] documents, for the
    /// same two reasons: the ribbon control is gated on `doc.open` and a chord
    /// bound to the same id is not.
    ///
    /// The no-document guard is the sharper of the two here. Without it a chord
    /// on an empty canvas would build a window that [`Self::show`] closes again
    /// on its very next frame — a control that visibly flickers rather than one
    /// that visibly declines, which is the harder of the two to diagnose.
    ///
    /// The already-open guard costs nothing (there is no configuration to
    /// discard) and is kept for About's reason: rebuilding would move the
    /// window back to the centre and the findings list back to the top, which
    /// for an operator half-way down a census reads as the program losing their
    /// place.
    ///
    /// ★ Note what it does **not** guard on: whether anything has been
    /// rasterized. `doc.open` is the registered predicate, and a document with
    /// no texture yet is precisely when an operator asks what the renderer did
    /// — so the dialog opens and *says* that nothing has been drawn, rather
    /// than the command silently doing nothing.
    pub fn open_diagnostics(&mut self, status: &Status) {
        if !matches!(status, Status::Open(_)) {
            return;
        }
        if self.diagnostics.is_some() {
            return;
        }
        self.diagnostics = Some(diagnostics::DiagnosticsDialog::open());
    }

    /// Open the About dialog.
    ///
    /// **The dispatch target for the `file.about` command.** Unlike
    /// [`Self::open_print`] it takes no [`Status`], because it needs none:
    /// About describes the application, and the application is always there.
    /// The command is registered with no `enabled_when` for the same reason.
    ///
    /// The already-open guard is kept, and for a slightly different reason
    /// than print's: this dialog holds no configuration to discard, so
    /// rebuilding it would lose nothing — but it would *move* the window back
    /// to the centre and reset its scroll position, which for an operator
    /// half-way down the attribution list reads as the program losing their
    /// place.
    pub fn open_about(&mut self) {
        if self.about.is_some() {
            return;
        }
        self.about = Some(about::AboutDialog::open());
    }

    /// Draw every open dialog, and close the ones that asked to close.
    ///
    /// Called once per frame from frame composition, **after** the canvas and
    /// the docks: a dialog is an overlay, and egui's `Area` ordering follows
    /// the order things are added within a frame.
    ///
    /// # Why a closed document closes the DOCUMENT-SCOPED dialogs
    ///
    /// A print job is a job on this file's pages. A dialog left up over a
    /// closed document would be configuring a job against pages that no
    /// longer exist, and the honest response is to close it rather than to
    /// freeze it or to let it act on whatever is opened next.
    ///
    /// # ★ …and why About is drawn either way
    ///
    /// It is about pdfce, not about a document. Closing it when the document
    /// closes would make `file.about` — a command every mode offers, with no
    /// `enabled_when` — open a window that vanished on the same frame
    /// whenever the canvas was empty. That is a control that does nothing,
    /// and it would look exactly like a bug in the command dispatch rather
    /// than like a rule about dialog lifetime.
    ///
    /// The early return therefore covers only the first group. Both are drawn
    /// first and closed after, rather than closed inside the borrow that drew
    /// them: a dialog decides whether it stays open *while* it draws (the
    /// title-bar cross and its own Close button are both widgets), so the
    /// answer arrives out of the same call that needs `&mut` on the state
    /// being dropped.
    pub fn show(&mut self, ctx: &egui::Context, status: &Status) {
        // Application-scoped first, so that an empty canvas cannot skip it.
        // Ordering is the whole guard here: putting this after the early
        // return below is a one-line edit that would silently restore the old
        // behaviour, which is why it is above it rather than beside it.
        if self.about.as_mut().map(|d| d.show(ctx)) == Some(false) {
            self.about = None;
        }

        let Status::Open(doc) = status else {
            self.close_document_scoped();
            return;
        };
        let doc: &OpenDoc = doc;
        if self.print.as_mut().map(|d| d.show(ctx, doc)) == Some(false) {
            self.print = None;
        }
        if self.ocr.as_mut().map(|d| d.show(ctx, doc)) == Some(false) {
            self.ocr = None;
        }
        if self.diagnostics.as_mut().map(|d| d.show(ctx, doc)) == Some(false) {
            self.diagnostics = None;
        }
    }

    /// Drop the state of every dialog that is about the open document.
    ///
    /// One place, so a document-scoped dialog added later cannot be forgotten
    /// by whichever of the close paths its author did not think of.
    /// Application-scoped dialogs are deliberately absent — see
    /// [`Self::show`].
    fn close_document_scoped(&mut self) {
        self.print = None;
        self.ocr = None;
        self.diagnostics = None;
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

    /// Closing the document closes the document-scoped dialogs.
    ///
    /// Asserted through the public path rather than by setting the field, so
    /// the test covers what a frame actually does.
    #[test]
    fn a_closed_document_closes_every_document_scoped_dialog() {
        let mut dialogs = DialogsState::default();
        assert!(dialogs.print.is_none());
        dialogs.close_document_scoped();
        assert!(dialogs.print.is_none());
        assert!(dialogs.ocr.is_none());
        assert!(dialogs.diagnostics.is_none());
    }

    /// The render report cannot be opened without a document either, and the
    /// guard is the one that matters most for it.
    ///
    /// Its command is gated on `doc.open`, so the ribbon cannot reach this
    /// state — but a chord can, and without the guard the dialog would be built
    /// and then closed by [`DialogsState::show`] on the very next frame. A
    /// window that flickers is harder to diagnose than one that never appears.
    #[test]
    fn no_document_means_no_diagnostics_dialog() {
        let mut dialogs = DialogsState::default();
        dialogs.open_diagnostics(&Status::Empty);
        assert!(dialogs.diagnostics.is_none());
    }

    /// Pressing Render diagnostics twice does not rebuild the report.
    ///
    /// Nothing would be lost — it holds no configuration, and it reads the
    /// texture live — but the window would jump back to the centre and the
    /// findings list back to the top, which for an operator half-way down a
    /// census is the program losing their place. About's argument, one dialog
    /// over.
    #[test]
    fn opening_the_diagnostics_report_twice_leaves_the_first_one_alone() {
        let mut dialogs = DialogsState::default();
        let status = Status::Open(Box::new(crate::app::state::open_fixture(
            crate::app::state::FOUR_PAGES,
        )));
        dialogs.open_diagnostics(&status);
        let first = std::ptr::from_ref(dialogs.diagnostics.as_ref().expect("open"));
        dialogs.open_diagnostics(&status);
        let second = std::ptr::from_ref(dialogs.diagnostics.as_ref().expect("still open"));
        assert_eq!(first, second, "the second press replaced the dialog");
    }

    /// Recognise text cannot be opened without a document either.
    ///
    /// Same guard as print's, and it matters for a different reason: the
    /// dialog captures the page index and the document path on construction,
    /// so one built against `Status::Empty` would have neither and would be a
    /// window that could only refuse.
    #[test]
    fn no_document_means_no_recognition_dialog() {
        let mut dialogs = DialogsState::default();
        dialogs.open_ocr(&Status::Empty);
        assert!(dialogs.ocr.is_none());
    }

    /// About opens with no document, and survives the document closing.
    ///
    /// ★ The one property that would have been lost by reusing print's shape.
    /// `open_about` takes no `Status` precisely so this cannot regress by
    /// someone adding a guard "for consistency"; the assertion is here so
    /// that if they do, something says why it was not consistent in the first
    /// place.
    #[test]
    fn about_opens_without_a_document_and_survives_one_closing() {
        let mut dialogs = DialogsState::default();
        dialogs.open_about();
        assert!(
            dialogs.about.is_some(),
            "About must open on an empty canvas: it describes pdfce, not a file"
        );
        dialogs.close_document_scoped();
        assert!(
            dialogs.about.is_some(),
            "About is not about the document and must not close with it"
        );
    }

    /// Pressing About twice does not rebuild the dialog.
    ///
    /// Nothing would be *lost* — it holds no configuration — but the window
    /// would jump back to the centre and the attribution list back to the
    /// top, which for an operator reading it is the program losing their
    /// place.
    #[test]
    fn opening_about_twice_leaves_the_first_one_alone() {
        let mut dialogs = DialogsState::default();
        dialogs.open_about();
        let first = std::ptr::from_ref(dialogs.about.as_ref().expect("open"));
        dialogs.open_about();
        let second = std::ptr::from_ref(dialogs.about.as_ref().expect("still open"));
        assert_eq!(first, second, "the second press replaced the dialog");
    }
}
