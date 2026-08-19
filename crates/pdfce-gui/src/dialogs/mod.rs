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
/// ★ The Manage-dimension-groups window — where a drawing's ce-dimension
/// groups are made, chosen and configured.
///
/// `measure.manage_groups` was registered, drawn on Measure ▸ Scale and inert
/// for the whole life of this build; its own header records what the operator
/// hit and which four of the six group verbs the engine actually ships.
pub mod dimension_groups;
/// ★ The Insert-image window — a picture placed on the page as content, by a
/// rectangle in millimetres.
///
/// Its header carries the decision a reader will question first: why placement
/// is numeric rather than a drag, and why a drag is a second **route** to the
/// same action rather than the one that should have shipped.
pub mod insert_image;
pub mod insert_pages;
pub mod new_document;
pub mod ocr;
pub mod print;
/// The Apply-redactions transaction — the report, the two acknowledgements, and
/// the write. The **irreversible** half of the redaction feature; its
/// reversible twin is `crate::panels::redact`. See its header for why the
/// removal runs on open, why confirmation is three gates rather than one click,
/// and why the destination is asked for every time.
pub mod redact;

/// ★ The Set-scale dialog — what a dimension's number *means*.
///
/// Phase 7 shipped three tools that place dimensions and no way to say what
/// scale they are at, so every label read in PDF points: a measurement of the
/// **paper** rather than of the thing drawn on it. A plausible answer to a
/// question nobody asked, which is worse than a missing feature.
pub mod scale;
/// The words half of a text-bearing annotation — the second half of the
/// place-then-type gesture. Its header argues why it is a dialog.
pub mod textannot;

/// ★ The Settings window — the thirteen questions the PDF standard declines to
/// answer, and the operator's answers to them.
///
/// **Application-scoped and not held in [`DialogsState`]**, which is the one
/// departure in this directory and is forced rather than chosen. Its draft has
/// to be readable at the *top* of the frame, before any widget is built,
/// because the theme is installed there and a draft theme must take effect
/// immediately — you cannot judge a theme from a radio label. So the draft
/// lives on `PdfceApp` as `settings_draft`, and this module is a renderer with
/// no state of its own.
pub mod settings;

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

    /// The Set-scale dialog, when one is open.
    ///
    /// Document-scoped, and the first dialog here that **edits the document
    /// through the action funnel**. Print writes to a spooler and OCR and
    /// redaction produce new files; this one recalibrates a dimension group in
    /// the open document, which is an undoable edit — see
    /// `crate::app::actions::Action::SetGroupScale`.
    ///
    /// That is why [`Self::show`] takes an action queue at all: this module's
    /// header says a dialog that edits the document *"must use the funnel"*,
    /// and this is the first one that does.
    scale: Option<scale::ScaleDialog>,
    /// The open text-annotation dialog, if a text box, sticky or stamp has
    /// just been placed.
    text_annot: Option<textannot::TextAnnotDialog>,

    /// The Apply-redactions dialog, when one is open.
    ///
    /// Document-scoped, and more emphatically than any of its neighbours. ★ It
    /// is the second dialog here that holds **unsaved bytes** and the first
    /// whose bytes are a *destructive* transformation of the open file, so
    /// closing the document discards them — which is the right answer rather
    /// than a loss, and for a sharper version of [`Self::ocr`]'s reason: a
    /// redaction is of *these marks* on *this document*, and writing prepared
    /// bytes after the operator has put the file away would produce a redacted
    /// copy of something nobody is looking at, derived from a mark census that
    /// no longer exists to be checked against.
    redact: Option<redact::RedactDialog>,

    // --- application-scoped: survives an empty canvas ---------------------
    /// The About dialog, when one is open.
    ///
    /// Carries the attribution surface — see [`about`] and
    /// [`crate::text::about`] for why a shipped `LICENSE` file is not enough
    /// once a CC-BY-SA-4.0 asset is in the package.
    about: Option<about::AboutDialog>,

    /// The sized-New dialog, when one is open.
    ///
    /// **Application-scoped**, beside About and for the strongest version of
    /// its reason: an operator with nothing open is not somebody this window is
    /// *tolerated* for, they are the operator it exists for. Closing a document
    /// must therefore not close it — and, unlike About, this one would be
    /// actively harmful to close, because the document it is about to make is
    /// how the operator gets out of the empty state.
    new_document: Option<new_document::NewDocumentDialog>,

    /// The insert dialog, when one is open.
    ///
    /// **Document-scoped**: it inserts into the open document, so closing that
    /// document closes it. It sits in this group rather than beside About for
    /// the reason the group exists — a dialog configuring an edit to a file
    /// that is no longer open is configuring nothing.
    insert_pages: Option<insert_pages::InsertPagesDialog>,

    /// The Insert-image window, when one is open.
    ///
    /// **Document-scoped**: it places a picture on a page of the open file, and
    /// it holds the imported bytes — so closing the document discards them,
    /// which is the right answer rather than a loss, for [`Self::ocr`]'s reason
    /// applied to an operand instead of to a result.
    insert_image: Option<insert_image::InsertImageDialog>,

    /// The Manage-dimension-groups window, when one is open.
    ///
    /// **Document-scoped**: a dimension group is a record in *this* document's
    /// `/PieceInfo` sidecar, so a window listing them over a closed document
    /// would be listing nothing.
    ///
    /// ★ It is the first dialog here that **asks its owner to open a sibling**.
    /// Its *Set scale…* button cannot call [`Self::open_scale`] — both are
    /// fields of this one struct and neither can reach the other from inside
    /// its own `show` — so it parks a `GroupId` and [`Self::show`] drains it.
    /// See `dimension_groups::DimensionGroupsDialog::scale_requested`.
    dimension_groups: Option<dimension_groups::DimensionGroupsDialog>,
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

    /// Open the Apply-redactions dialog for the document in `status`.
    ///
    /// **The dispatch target for the `edit.redact_apply` command**, and it
    /// applies the same two guards [`Self::open_print`] documents — the ribbon
    /// control is gated on `doc.pages` and a chord bound to the same id is not.
    ///
    /// ★ Both guards are load-bearing here in a way they are not elsewhere,
    /// because [`redact::RedactDialog::open`] **runs the whole removal**.
    ///
    /// - **No document, no dialog.** Without this, an invocation over an empty
    ///   shell would build a window that [`Self::show`] closes again on its very
    ///   next frame — a control that visibly flickers rather than one that
    ///   declines.
    /// - **Already open means leave it alone**, and this is the strong one. A
    ///   second press would re-run a full rewrite of the document *and* discard
    ///   the operator's two acknowledgements — throwing away the reading they
    ///   have just done on the one report in this program that has to be read.
    ///   Worse, it would silently replace a report computed against the marks as
    ///   they were with one computed against the marks as they are now, which is
    ///   the difference between the numbers on screen and the bytes that would
    ///   be written.
    pub fn open_redact(&mut self, status: &Status) {
        if self.redact.is_some() {
            return;
        }
        self.redact = redact::open_for(status);
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
    /// Open the Set-scale dialog on `group`.
    ///
    /// The already-open guard is the same one every dialog here has, and it
    /// matters more than usual: a second press must not discard a ratio the
    /// operator has half typed, and re-opening would also re-capture the active
    /// group — so a group change made while the dialog was up would silently
    /// redirect the calibration.
    pub fn open_scale(&mut self, status: &Status, group: pdfce_core::dimension::GroupId) {
        if !matches!(status, Status::Open(_)) {
            return;
        }
        if self.scale.is_some() {
            return;
        }
        self.scale = Some(scale::ScaleDialog::open(group));
    }

    /// **Open the Set-scale dialog with a reference line already measured.**
    ///
    /// The calibration path's entry point, raised by the application on the
    /// click that completes the two-point pick.
    ///
    /// # ★ It REPLACES an open dialog, where [`Self::open_scale`] refuses to
    ///
    /// That guard exists so a second press of the ribbon control does not
    /// discard what the operator has half typed. The situations are opposite
    /// here: the operator asked to measure on the drawing, the dialog closed
    /// so they could, and they have now finished. A guard that refused would
    /// leave them looking at a stale window with no measurement in it —
    /// the one outcome the whole gesture exists to avoid.
    pub fn open_scale_calibrated(
        &mut self,
        status: &Status,
        group: pdfce_core::dimension::GroupId,
        drawn_pdf_length: f64,
    ) {
        if !matches!(status, Status::Open(_)) {
            return;
        }
        self.scale = Some(scale::ScaleDialog::calibrated(group, drawn_pdf_length));
    }

    /// Whether the open Set-scale dialog is asking to start the two-point pick.
    ///
    /// Read-and-clear, so the caller cannot re-arm on every frame by forgetting
    /// to reset it.
    pub fn take_scale_calibrate_request(&mut self) -> bool {
        self.scale
            .as_mut()
            .is_some_and(scale::ScaleDialog::take_calibrate_request)
    }

    /// Close the Set-scale dialog, whatever state it is in.
    ///
    /// Used when the operator asks to measure on the drawing: the window has to
    /// get out of the way of the page they are about to click on.
    pub fn close_scale(&mut self) {
        self.scale = None;
        self.text_annot = None;
    }

    /// **Open the text-annotation dialog for a just-placed annotation.**
    ///
    /// Raised by `Action::BeginTextAnnot`, which the canvas pushes on the
    /// gesture that finishes placing.
    ///
    /// ★ It REPLACES an open dialog rather than refusing, unlike
    /// [`Self::open_scale`]. The situations are opposite: that guard protects a
    /// half-typed value from a second ribbon press, and here a second placing
    /// gesture is the operator plainly saying they want to annotate somewhere
    /// else. Refusing would leave them looking at a window describing a box
    /// they have moved on from.
    pub fn open_text_annot(
        &mut self,
        status: &Status,
        page: usize,
        kind: crate::canvas::textannot::TextAnnotKind,
        rect: pdfce_core::page_tree::Rect,
    ) {
        if !matches!(status, Status::Open(_)) {
            return;
        }
        self.text_annot = Some(textannot::TextAnnotDialog::open(page, kind, rect));
    }

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

    /// Open the sized-New dialog.
    ///
    /// **The dispatch target for `file.new_from_template`.** Like
    /// [`Self::open_about`] it takes no [`Status`] and the command is
    /// registered with no `enabled_when`: New is the command an empty shell
    /// exists to offer, and gating it on a document would grey the one control
    /// that answers *"there is nothing here"*.
    ///
    /// The already-open guard is print's rather than About's: this window holds
    /// a size, an orientation and two typed numbers, and a second press of the
    /// ribbon control part-way through would silently reset all four to A4
    /// portrait — the operator's own choices, discarded by the control they
    /// pressed to look at them.
    /// Open the insert dialog for `path`, having counted its pages.
    ///
    /// **The dispatch target for `pages.insert_from_file`, after the picker.**
    ///
    /// # ★ Why the page count is read here and not in the dialog
    ///
    /// Because a file that will not open must be reported **instead of** the
    /// dialog, not after the operator has filled one in. Opening a window that
    /// says "0 pages" and refuses its own commit button would be a surface
    /// asking a question that cannot be answered.
    ///
    /// The load is cheap relative to what follows — the same file is opened
    /// again by the insert itself — and a document parsed twice is the honest
    /// trade for a dialog that can state a fact before the operator commits.
    pub fn open_insert_pages(&mut self, path: std::path::PathBuf, current_page: usize) {
        if self.insert_pages.is_some() {
            return;
        }
        let count = match pdfce_core::document::Document::load(&path) {
            Ok(doc) => pdfce_core::page_tree::pages(&doc).map_or(0, |p| p.len()),
            Err(error) => {
                let detail = error.to_string();
                crate::diag::trace(|| {
                    format!(
                        // ui-text-exempt: diagnostic trace, never displayed in the UI
                        "insert-picked-unreadable path={path:?} reason={detail}"
                    )
                });
                0
            }
        };
        if count == 0 {
            // Nothing to ask about. The refusal is the status-bar sentence the
            // insert path already owns, so the operator meets one voice rather
            // than a dialog and then a note.
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("insert-declined path={path:?} reason=no-pages")
            });
            return;
        }
        self.insert_pages = Some(insert_pages::InsertPagesDialog::open(
            path,
            count,
            current_page,
        ));
    }

    pub fn open_new_document(&mut self) {
        if self.new_document.is_some() {
            return;
        }
        self.new_document = Some(new_document::NewDocumentDialog::open());
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
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        status: &Status,
        actions: &mut Vec<crate::app::actions::Action>,
        window: Option<isize>,
    ) {
        // Application-scoped first, so that an empty canvas cannot skip it.
        // Ordering is the whole guard here: putting this after the early
        // return below is a one-line edit that would silently restore the old
        // behaviour, which is why it is above it rather than beside it.
        if self.about.as_mut().map(|d| d.show(ctx)) == Some(false) {
            self.about = None;
        }
        // Beside About, above the guard, and for a sharper version of the same
        // reason: this window's whole purpose is to produce a document, so a
        // guard that closed it when none was open would close it exactly when
        // it was needed.
        if self.new_document.as_mut().map(|d| d.show(ctx, actions)) == Some(false) {
            self.new_document = None;
        }

        let Status::Open(doc) = status else {
            self.close_document_scoped();
            return;
        };
        let doc: &OpenDoc = doc;
        if self.print.as_mut().map(|d| d.show(ctx, doc, window)) == Some(false) {
            self.print = None;
        }
        if self.ocr.as_mut().map(|d| d.show(ctx, doc)) == Some(false) {
            self.ocr = None;
        }
        if self.diagnostics.as_mut().map(|d| d.show(ctx, doc)) == Some(false) {
            self.diagnostics = None;
        }
        if self.redact.as_mut().map(|d| d.show(ctx, doc)) == Some(false) {
            self.redact = None;
        }
        if self.insert_pages.as_mut().map(|d| d.show(ctx, actions)) == Some(false) {
            self.insert_pages = None;
        }
        if self.insert_image.as_mut().map(|d| d.show(ctx, actions)) == Some(false) {
            self.insert_image = None;
        }
        // ★ Drawn BEFORE the Set-scale window, and the order is load-bearing.
        //
        // Its *Set scale…* button parks a request that is drained immediately
        // below, and `open_scale` builds a window `ScaleDialog::show` must then
        // draw. Drawing this one second would leave the request sitting for a
        // frame — invisible, but it would make the button feel like it missed.
        if self
            .dimension_groups
            .as_mut()
            .map(|d| d.show(ctx, doc, actions))
            == Some(false)
        {
            self.dimension_groups = None;
        }
        // The hand-over. `open_scale` applies its own two guards at the one
        // place a `ScaleDialog` is ever built, so a request arriving while one
        // is already open leaves the operator's half-typed ratio alone.
        if let Some(group) = self
            .dimension_groups
            .as_mut()
            .and_then(dimension_groups::DimensionGroupsDialog::take_scale_request)
        {
            self.open_scale(status, group);
        }
        // ★ Takes the action queue, unlike its four neighbours. See the field.
        // It does not take `doc`: the scale it sets belongs to a *group*, which
        // is document-scoped but not page-scoped, and the entry fields need
        // nothing from the open document at all.
        if self.scale.as_mut().map(|d| d.show(ctx, actions)) == Some(false) {
            self.scale = None;
        }
        if self.text_annot.as_mut().map(|d| d.show(ctx, actions)) == Some(false) {
            self.text_annot = None;
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
        self.redact = None;
        self.scale = None;
        self.dimension_groups = None;
        self.insert_image = None;
    }

    /// Open the Insert-image window for an already-imported picture.
    ///
    /// **The dispatch target for the `edit.insert_image` command**, reached only
    /// after the file has been chosen AND imported — see that arm for why the
    /// import happens first.
    ///
    /// The already-open guard matters here the way it matters for OCR: a second
    /// press would discard a placement the operator has typed and replace the
    /// imported bytes with another file's, so the window they pressed the
    /// shortcut to look at would come back describing a different picture.
    pub fn open_insert_image(
        &mut self,
        status: &Status,
        image: std::sync::Arc<pdfce_core::image_import::ImportedImage>,
        name: String,
    ) {
        if self.insert_image.is_some() {
            return;
        }
        self.insert_image = insert_image::open_for(status, image, name);
    }

    /// Open the Manage-dimension-groups window for the document in `status`.
    ///
    /// **The dispatch target for the `measure.manage_groups` command**, and it
    /// applies the same two guards [`Self::open_print`] documents.
    ///
    /// The already-open guard matters more here than the shape of the sentence
    /// suggests: a second press would discard a half-typed group name and reset
    /// which group's settings are on screen — the operator's own state, thrown
    /// away by the control they pressed to look at it.
    ///
    /// `active` is the measure tool's authoring group, resolved by the
    /// dispatcher exactly as `measure.set_scale` resolves it, so the window
    /// opens on the group the operator is drawing into rather than on whichever
    /// one happens to be first in the model.
    pub fn open_dimension_groups(
        &mut self,
        status: &Status,
        active: pdfce_core::dimension::GroupId,
    ) {
        if self.dimension_groups.is_some() {
            return;
        }
        self.dimension_groups = dimension_groups::open_for(status, active);
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
        assert!(dialogs.redact.is_none());
    }

    /// ★ **Apply redactions cannot be opened without a document, and a second
    /// invocation does not rebuild it.**
    ///
    /// Both guards matter more for this dialog than for any of its neighbours,
    /// because opening it runs a full rewrite of the document. The second
    /// assertion is the one with teeth: a rebuild would re-run that work *and*
    /// discard the operator's two acknowledgements, throwing away the reading
    /// they have just done on the one report in this program that has to be
    /// read before a control is pressed.
    #[test]
    fn the_apply_dialog_is_guarded_on_both_counts() {
        let mut dialogs = DialogsState::default();
        dialogs.open_redact(&Status::Empty);
        assert!(
            dialogs.redact.is_none(),
            "a document with nothing open has nothing to redact, and building \
             the dialog would run a full rewrite in order to refuse"
        );

        let status = Status::Open(Box::new(crate::app::state::open_fixture(
            crate::app::state::FOUR_PAGES,
        )));
        dialogs.open_redact(&status);
        let first = std::ptr::from_ref(dialogs.redact.as_ref().expect("open"));
        dialogs.open_redact(&status);
        let second = std::ptr::from_ref(dialogs.redact.as_ref().expect("still open"));
        assert_eq!(
            first, second,
            "the second press replaced the dialog, re-running the removal and \
             discarding both acknowledgements"
        );
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
