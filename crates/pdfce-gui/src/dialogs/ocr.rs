//! # `dialogs::ocr` — the Recognise-text transaction
//!
//! One dialog, three states, and a shape that is chosen rather than
//! conventional. It is the surface for `file.ocr`, and it is also the
//! **enforcement point for two rules** that would otherwise have nowhere to
//! live in this build.
//!
//! ## The three states
//!
//! | state | what the operator sees | what exists |
//! |---|---|---|
//! | **ready** | what OCR does to the page, and one button that starts it | nothing |
//! | **working** | *Recognising…* | a thread |
//! | **answered** | the disclosure, then a Save-as button — or a named refusal | bytes, in memory |
//!
//! ## ★ Why the recognition is disclosed BEFORE it is written, not after
//!
//! This is the whole reason the dialog has a third state instead of running
//! OCR and immediately opening a file picker.
//!
//! Project rule 4 is *"fuzzy, never sneaky"*, and `pdfce-core`'s own OCR
//! header sharpens it for exactly this feature: **every word an OCR layer
//! contains is a guess**, and this engine reports no confidence for any of
//! them. A surface that recognised a page and dropped the finished file in
//! front of the operator would be technically disclosive — the report would be
//! *somewhere* — while being, in practice, a program that silently inserted
//! several hundred unreviewed inferences into a document. `DEFECTS.md` and
//! `HANDOFF.md` both record that this project's characteristic failure is a
//! surface that is *correct* and *unreadable at the moment it matters*.
//!
//! So the order is: recognise, **show what was inferred**, and only then offer
//! to write it. The operator reads the disclosure while holding the one thing
//! that gives it force — the ability to not save. That is not a nicety; it is
//! the difference between a disclosure and a receipt.
//!
//! ## ★ Why the write is a Save-as, in every mode
//!
//! The operator's standing rule, 2026-08-14: *"Read may produce a new
//! document; it may not modify this one"*, with the enforcement at the **save**
//! rather than at the operation.
//!
//! `HANDOFF.md` §3 records that rule as currently **vacuous**, on the grounds
//! that `file.save_copy` never overwrites the original. That was true and it
//! understated the position: when this dialog was built, `file.save_copy` had
//! **no dispatch arm at all** (`app/dispatch.rs` fell through to
//! `command-unimplemented`), so **this shell could not write a file of any
//! kind.**
//!
//! ★ **`file.save_copy` was wired on 2026-08-14** and the rule stays vacuous
//! for the reason it always was: that command asks for a destination too, and
//! `crate::app::save::suggested_path` guarantees the *suggestion* is never the
//! file that was opened, exactly as [`suggested_path`] below does here. The two
//! surfaces now share one picker, `crate::app::files::pick_save_path`, and the
//! only thing that differs between them is the dialog's title.
//!
//! This dialog was nevertheless the first write to disk pdfceGUI performed, and
//! therefore the first place the rule could bite. It bites here in the only way that is
//! honest: the destination is a path the operator names, so the rule holds in
//! Read **and** in Edit **and** in Review by construction rather than by a mode
//! check. Nothing here consults the mode, and nothing here should — the rule is
//! about what a save may overwrite, not about who is asking.
//!
//! What is deliberately **not** done: no second save command, no in-place path,
//! no `Save`-labelled control anywhere. The day in-place `Save` lands it will
//! need its own Read-mode gate, and that gate belongs beside it rather than
//! being invented here in advance against a command that does not exist.
//!
//! ## ★ Why OCR is available in Read, with no capability flag
//!
//! `app::modes::capability` governs **gestures** — what a press on the canvas
//! means. OCR is not a gesture; it is a command with a dialog, and it changes
//! no document that is open. Adding a capability flag for it would put a rule
//! about *saving* into the machinery that decides what a drag does, where the
//! next reader would neither look for it nor believe it.
//!
//! Read is therefore offered OCR exactly as Edit is, and that is the operator's
//! instruction rather than an omission.
//!
//! ## Why this dialog does not push an `Action`
//!
//! [`super`]'s rule: a dialog uses the action funnel when it edits **this**
//! document, and this one never does. The recognised bytes are a *new*
//! document; the open one is untouched, its `edit_epoch` does not move, and
//! there is nothing to order against or to undo. What the funnel's reasoning
//! does still demand is that irreversible work not happen part-way through a
//! layout pass — and it does not: the button sets a flag, and the file is
//! written after the window's closure returns.
//!
//! ## What is document-scoped about it
//!
//! Everything. A recognition is of *this page* of *this file*, so
//! [`super::DialogsState`] holds it in the document-scoped group and closing
//! the document closes it. A finished-but-unsaved recognition is discarded
//! with it, which is the right answer: writing it afterwards would produce a
//! file derived from a document the operator has already put away.

use std::path::{Path, PathBuf};

use egui_shell::theme::Theme;

use crate::app::state::{OpenDoc, Status};
use crate::ocr::{self, Job, Recognised, Refusal, Request};
use crate::text::ocr as t;

// ---------------------------------------------------------------------------
// Named regions
//
// `crate::diag::ui_rect` publishes where a control actually got drawn, so
// `tools/ui-verify` can aim a real click at it. These names are matched
// LITERALLY by `tools/ui-verify/src/checks/ocr.rs`, so renaming one silently
// un-aims the check that measures it.
//
// ★ Why a dialog needs them at all, when the ribbon's controls get theirs for
// free: `egui_shell::ribbon` declares a rect per band control centrally, and
// nothing does that for a window this crate draws itself. Without these, the
// only way a harness could reach the Recognise button would be to guess a
// fraction of a centred window -- which goes stale the first time a sentence
// in the dialog wraps differently.
// ---------------------------------------------------------------------------

/// The whole window.
const REGION_DIALOG: &str = "ocr-dialog"; // ui-text-exempt: trace region name, never displayed

/// The control that starts recognition.
const REGION_RUN: &str = "ocr-run"; // ui-text-exempt: trace region name, never displayed

/// The control that writes the recognised copy.
///
/// Declared **only while it exists**, which is itself the assertion a harness
/// wants: this control is drawn if and only if there are recognised bytes to
/// write, so its absence from the trace is evidence that nothing was
/// recognised rather than that a click missed.
const REGION_SAVE: &str = "ocr-save"; // ui-text-exempt: trace region name, never displayed

/// Where one Recognise-text transaction has got to.
///
/// A state machine rather than three `Option`s, because the states are
/// mutually exclusive and an `Option` triple has five nonsense combinations
/// that would all compile.
#[derive(Debug, Default)]
enum Phase {
    /// Nothing has been asked for yet.
    #[default]
    Ready,
    /// A thread is recognising.
    Working(Job),
    /// Recognition finished and produced a document that is not saved.
    Recognised(Box<Recognised>),
    /// Recognition did not happen, for a named reason.
    Refused(Refusal),
    /// The bytes were written to this path.
    Saved(PathBuf),
}

/// The Recognise-text dialog.
#[derive(Debug)]
pub struct OcrDialog {
    /// The page this transaction is about, captured when the dialog opened.
    ///
    /// ★ **Captured, not read per frame**, and that is a correctness
    /// requirement rather than an optimisation. The operator can page the
    /// document while the dialog is open; a `Save` that read the *current*
    /// page index would label bytes recognised from page 3 as belonging to
    /// whatever page they had scrolled to. The recognition is of one page and
    /// the dialog remembers which.
    page_index: usize,
    /// The document's own path, for suggesting a name to save under.
    source: PathBuf,
    /// The transaction's state.
    phase: Phase,
    /// Set by the Close button, consumed by [`Self::show`].
    ///
    /// The two-step every dialog here uses: a widget drawn from the state
    /// cannot drop the state it is being drawn from, so it records the request
    /// and the caller acts after the closure returns.
    close_requested: bool,
    /// Set by the Save button; the write happens after the closure returns.
    ///
    /// Same two-step, for a stronger reason: this is the irreversible half.
    /// [`super`]'s header requires that a dialog's irreversible work not run
    /// part-way through a layout pass, and a `rfd` modal opened from inside an
    /// `egui::Window` closure would block the frame it is drawn in.
    save_requested: bool,
}

impl OcrDialog {
    /// Build the dialog for the page `doc` is showing.
    ///
    /// Nothing is recognised yet: opening the dialog is free, and the several
    /// seconds of work start on a press the operator makes after reading what
    /// the operation does. A dialog that started recognising on open would
    /// spend that time before the operator had decided they wanted it.
    #[must_use]
    pub(super) fn open(doc: &OpenDoc) -> Self {
        Self {
            page_index: doc.view.page_index,
            source: doc.path.clone(),
            phase: Phase::Ready,
            close_requested: false,
            save_requested: false,
        }
    }

    /// Draw one frame. Returns `false` when the dialog should close.
    pub(super) fn show(&mut self, ctx: &egui::Context, doc: &OpenDoc) -> bool {
        self.poll_worker();

        // ★ ITS OWN OS WINDOW as of 2026-08-21. OCR is the longest-running
        // thing in this program — a job an operator starts and then goes back
        // to work while it runs — and a progress window locked inside the
        // application frame is a window that has to be closed to keep working.
        //
        // ★ The dialog region is published from INSIDE the callback now. It
        // used to come from the `egui::Window` response rect, which no longer
        // exists; `ui.max_rect()` is the same rectangle in the coordinates the
        // harness converts, and `dialogs::host` tags it with this viewport.
        let (frame, ()) = crate::dialogs::host::Host::new(
            "ocr", // ui-text-exempt: a viewport key, never displayed.
            t::title(),
            egui::vec2(560.0, 420.0),
            // A floor, on the print and About dialogs' own reasoning: a
            // resizable window with no minimum can be dragged down to a title
            // bar and a scrollbar, which is a state with no way out but
            // closing.
            egui::vec2(420.0, 260.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_DIALOG, ui.max_rect());
            self.body(ui, doc);
        });
        let open = !frame.closed;

        // The irreversible half, after the closure. See `save_requested`.
        if std::mem::take(&mut self.save_requested) {
            self.save();
        }
        open && !std::mem::take(&mut self.close_requested)
    }

    /// Move a finished job into the phase that describes its answer.
    ///
    /// Separate from [`Self::body`] so the transition happens once per frame
    /// regardless of what the window drew, and so that a dialog scrolled out
    /// of view still notices its own worker finishing.
    fn poll_worker(&mut self) {
        let page_index = self.page_index;
        let Phase::Working(job) = &mut self.phase else {
            return;
        };
        let Some(outcome) = job.poll() else {
            return;
        };
        self.phase = match outcome {
            Ok(recognised) => {
                crate::diag::trace(|| {
                    format!(
                        // ui-text-exempt: diagnostic trace, never displayed.
                        //
                        // ★ `recognised=` beside `written=`, on `HANDOFF.md`
                        // §2's own advice about the ink trail: a build whose
                        // placement silently dropped every word would emit an
                        // otherwise identical line, and the pair is what makes
                        // the two numbers comparable from a trace alone.
                        // `confidence_available=` is here because it is the
                        // one field whose value decides what the operator is
                        // told, and a build that started claiming `true` would
                        // be invisible in every other field.
                        "ocr-recognised page={page_index} recognised={} written={} skipped={} \
                         substituted={} clamped={} confidence_available={} dpi={:.0} bytes={}",
                        recognised.words_recognised,
                        recognised.report.words_written,
                        recognised.report.words_skipped,
                        recognised.report.words_substituted,
                        recognised.report.words_scale_clamped,
                        recognised.report.confidence_available,
                        recognised.effective_dpi,
                        recognised.bytes.len(),
                    )
                });
                Phase::Recognised(recognised)
            }
            Err(refusal) => {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    format!("ocr-refused reason={refusal:?}")
                });
                Phase::Refused(refusal)
            }
        };
    }

    /// Everything inside the window.
    fn body(&mut self, ui: &mut egui::Ui, doc: &OpenDoc) {
        let theme = Theme::of(ui.ctx());
        ui.label(t::intro());
        ui.add_space(10.0);
        ui.separator();
        ui.add_space(6.0);

        match &self.phase {
            Phase::Ready => self.ready(ui, doc),
            Phase::Working(_) => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(t::working());
                });
            }
            Phase::Recognised(recognised) => {
                let disclosures = recognised.report.disclosures();
                let saveable = !recognised.bytes.is_empty();
                Self::answered(ui, &theme, &disclosures);
                if saveable {
                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(6.0);
                    ui.label(t::not_saved_yet());
                    ui.add_space(6.0);
                    let save = ui.button(t::save_as()).on_hover_text(t::save_as_tooltip());
                    crate::diag::ui_rect(REGION_SAVE, save.rect);
                    if save.clicked() {
                        self.save_requested = true;
                    }
                }
            }
            Phase::Refused(refusal) => {
                ui.label(sentence(refusal));
            }
            Phase::Saved(path) => {
                ui.label(t::saved(&path.display().to_string()));
            }
        }

        ui.add_space(10.0);
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button(t::close()).clicked() {
                self.close_requested = true;
            }
        });
    }

    /// The pre-run state: one button, and the refusals that can be answered
    /// without running anything.
    ///
    /// ★ The order of the checks is the order of the questions, and it is not
    /// arbitrary. *Can this build look at all* comes before *are the files
    /// there to look with*, because asking them the other way round would
    /// report a missing model directory in a build that has no recogniser to
    /// use it — a true statement and the wrong diagnosis.
    fn ready(&mut self, ui: &mut egui::Ui, doc: &OpenDoc) {
        if let Some(refusal) = Self::preflight(doc) {
            ui.label(sentence(&refusal));
            return;
        }
        let run = ui.button(t::run()).on_hover_text(t::run_tooltip());
        crate::diag::ui_rect(REGION_RUN, run.rect);
        if run.clicked() {
            self.start(doc);
        }
    }

    /// Everything that can be refused before a thread is spawned.
    ///
    /// Returns `None` when recognition may proceed. Pulled out of [`Self::ready`]
    /// so that the decision is a pure function of the document and is therefore
    /// reachable from a test — the button and the window are not.
    fn preflight(doc: &OpenDoc) -> Option<Refusal> {
        if !ocr::engine_compiled_in() {
            return Some(Refusal::EngineAbsent);
        }
        // ★ Refused, not disclosed. `add_ocr_layer` writes an incremental
        // revision over the document AS OPENED, so a recognised copy taken now
        // would silently omit the operator's edits. See `crate::ocr`'s header.
        if doc.edit_epoch != 0 {
            return Some(Refusal::UnsavedEdits);
        }
        match ocr::resolve_models(ocr::exe_dir().as_deref(), user_data_dir().as_deref()) {
            Ok(_) => None,
            Err(e) => Some(Refusal::ModelsMissing(e.searched)),
        }
    }

    /// Spawn the worker.
    fn start(&mut self, doc: &OpenDoc) {
        let Ok(source) = ocr::resolve_models(ocr::exe_dir().as_deref(), user_data_dir().as_deref())
        else {
            // Unreachable behind `preflight`, and answered rather than
            // ignored: a button that did nothing would be indistinguishable
            // from a recognition that produced no words.
            self.phase = Phase::Refused(Refusal::ModelsMissing(Vec::new()));
            return;
        };
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed.
                "ocr-started page={} models={} source={}",
                self.page_index,
                source.path().display(),
                source.token()
            )
        });
        self.phase = Phase::Working(Job::spawn(Request {
            session: std::sync::Arc::clone(&doc.session),
            page_index: self.page_index,
            model_dir: source.path().to_path_buf(),
        }));
    }

    /// Ask where the recognised copy goes, and write it there.
    ///
    /// ★ **The first write to disk this program performed**, and since
    /// 2026-08-14 one of two — `crate::app::save::save_copy` is the other. It
    /// asks first, every time, and the suggested name is never the file that was
    /// opened — see [`suggested_path`]. There is no "save over the original"
    /// branch to find because there is none to write.
    fn save(&mut self) {
        let Phase::Recognised(recognised) = &self.phase else {
            return;
        };
        let suggested = suggested_path(&self.source);
        // The dialog's own title, not `file.save_copy`'s: both surfaces ask the
        // same operating-system question through the same picker, about
        // different things. See `crate::app::files::pick_save_path`.
        let crate::app::files::Picked::Path(target) =
            crate::app::files::pick_save_path(&suggested, crate::text::ocr::save_dialog_title())
        else {
            // Cancelled, or a build with no picker. Either way the bytes are
            // still in hand and the button is still there: nothing is lost and
            // nothing is said, because a cancelled save is a complete and
            // uninteresting outcome.
            return;
        };
        match std::fs::write(&target, &recognised.bytes) {
            Ok(()) => {
                crate::diag::trace(|| {
                    format!(
                        // ui-text-exempt: diagnostic trace, never displayed.
                        "ocr-saved path={} bytes={}",
                        target.display(),
                        recognised.bytes.len()
                    )
                });
                self.phase = Phase::Saved(target);
            }
            Err(e) => {
                self.phase = Phase::Refused(Refusal::Engine(e.to_string()));
            }
        }
    }

    /// The disclosure block: the confidence statement, then the engine's own
    /// lines.
    ///
    /// ★ **The confidence sentence is drawn first and separately, above the
    /// list.** `OcrLayerReport::disclosures()` already contains a sentence
    /// making the same point, and this is deliberate duplication rather than an
    /// oversight: the engine's version sits fourth in a list of counts, and the
    /// one fact that must survive a skim is that **nothing here was scored**.
    /// A reader who takes in one line takes in that one.
    ///
    /// Drawn in the plain text role, never `.strong()` — `DEFECTS.md` D11
    /// records that role as unusable in this theme, and a named palette exists
    /// so a surface written later does not rediscover it.
    fn answered(ui: &mut egui::Ui, theme: &Theme, disclosures: &[String]) {
        ui.label(t::what_was_inferred());
        ui.add_space(6.0);
        ui.label(t::no_confidence());
        ui.add_space(8.0);
        egui::ScrollArea::vertical()
            .auto_shrink([false, true])
            // The same negative-height trap the About dialog documents: a
            // window shorter than its own header makes `available_height()`
            // minus a reservation go negative, and a negative `max_height` is
            // a scroll area that silently draws nothing rather than an error.
            .max_height((ui.available_height() - FOOTER_RESERVE).max(LIST_FLOOR))
            .show(ui, |ui| {
                for line in disclosures {
                    ui.label(egui::RichText::new(line).color(theme.palette.text_muted));
                    ui.add_space(4.0);
                }
            });
    }
}

/// Height kept clear below the disclosure list for the save and close rows.
const FOOTER_RESERVE: f32 = 96.0;

/// The least height the disclosure list may be given.
///
/// See [`OcrDialog::answered`] — without a floor, a small window produces a
/// list that draws nothing at all and looks like a recognition that disclosed
/// nothing, which is the exact opposite of what this dialog is for.
const LIST_FLOOR: f32 = 48.0;

/// The operator-visible sentence for a refusal.
///
/// One place, so the dialog cannot word a refusal differently from anywhere
/// else that grows a need for it, and so `text::ocr`'s catalog is the only
/// thing that has to be read to know what pdfce says when OCR declines.
fn sentence(refusal: &Refusal) -> String {
    match refusal {
        Refusal::EngineAbsent => t::engine_absent().to_owned(),
        // ★ The paths go to the catalog as a LIST, not as a pre-joined string.
        //
        // The separator between them is copy: it is punctuation an operator
        // reads, and `tools/gates/check-ui-strings.sh` caught a `", "` sitting
        // here, correctly. Joining inside `text::ocr::models_missing` puts the
        // whole sentence — wording, punctuation and all — in the one file that
        // is allowed to decide how pdfce phrases things, which is what rule R1
        // is actually asking for rather than a technicality about where a comma
        // lives.
        Refusal::ModelsMissing(searched) => {
            let paths: Vec<String> = searched.iter().map(|p| p.display().to_string()).collect();
            t::models_missing(&paths)
        }
        Refusal::UnsavedEdits => t::unsaved_edits().to_owned(),
        Refusal::NothingRecognised => t::nothing_recognised().to_owned(),
        // A page index past the end and a page with no area are both
        // structural impossibilities from a dialog opened on a page the canvas
        // is showing. They are worded through the engine's own channel rather
        // than given catalog entries of their own: inventing operator copy for
        // a state nothing can reach is how a catalog fills with sentences
        // nobody has ever seen.
        Refusal::NoSuchPage(i) => t::failed(&(i + 1).to_string()),
        Refusal::EmptyPage => t::failed(&(0).to_string()),
        Refusal::Engine(reason) => t::failed(reason),
    }
}

/// The name to suggest for the recognised copy.
///
/// ★ **Never the file that was opened.** The suffix is what makes the default
/// answer a new document, so an operator who accepts the suggestion without
/// reading it cannot overwrite their scan. That is the standing rule expressed
/// as a default rather than as a warning — a warning is something to click
/// past.
///
/// The extension is forced to `.pdf` rather than preserved: the bytes are a
/// PDF whatever the source was called, and a recognised copy of `scan.PDF`
/// landing as `scan-recognised.PDF` would be correct but is one more way for a
/// tool downstream to disagree about case.
#[must_use]
pub fn suggested_path(source: &Path) -> PathBuf {
    let stem = source.file_stem().map_or_else(
        || String::from("document"),
        |s| s.to_string_lossy().into_owned(),
    ); // ui-text-exempt: a filename fallback, not operator copy
    let name = format!("{stem}{}.pdf", t::suggested_suffix());
    source
        .parent()
        .map_or_else(|| PathBuf::from(&name), |dir| dir.join(&name))
}

/// Where a durable per-user model directory would live.
///
/// `None` today, and that is a statement rather than a stub: this shell has no
/// user-data directory of its own — `app::persistence` writes its layout beside
/// the executable — so there is no second place to look and reporting one would
/// name a path in a "searched here" list that was never searched.
///
/// It exists as a function because `ocr::resolve_models` takes the parameter
/// and the day a user-data location appears there is one call site to change
/// rather than three.
fn user_data_dir() -> Option<PathBuf> {
    None
}

/// Open the dialog for the document in `status`, if there is one.
///
/// The dispatch target for `file.ocr`. Lives here rather than in
/// [`super::DialogsState`] only because it needs [`OcrDialog::open`]'s private
/// constructor; the guard it applies is the one `open_print` documents — the
/// ribbon control is gated on `doc.pages`, a chord bound to the same id is not,
/// and both are fixed by refusing here at the one place the dialog is built.
pub(super) fn open_for(status: &Status) -> Option<OcrDialog> {
    let Status::Open(doc) = status else {
        return None;
    };
    Some(OcrDialog::open(doc))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★ **The suggested name is never the file that was opened.**
    ///
    /// The standing rule as a default. An operator who accepts the suggestion
    /// without reading it must not overwrite their scan, and this is the
    /// assertion that says so — the label and the tooltip say it in words, and
    /// words are not a mechanism.
    #[test]
    fn the_suggested_name_is_never_the_source_file() {
        let source = PathBuf::from("D:\\scans\\survey.pdf");
        let suggested = suggested_path(&source);
        assert_ne!(suggested, source);
        assert_eq!(suggested, PathBuf::from("D:\\scans\\survey-recognised.pdf"));
        assert_eq!(
            suggested.parent(),
            source.parent(),
            "the copy should land beside the original, where the operator will look for it"
        );
    }

    /// A capitalised extension still produces a `.pdf`.
    #[test]
    fn the_suggested_name_always_ends_in_pdf() {
        for name in ["scan.PDF", "scan.pdf", "scan"] {
            let suggested = suggested_path(Path::new(name));
            assert!(
                suggested.to_string_lossy().ends_with(".pdf"),
                "{name} suggested {suggested:?}"
            );
        }
    }

    /// A source with no parent directory still yields a usable name.
    #[test]
    fn a_bare_filename_still_produces_a_suggestion() {
        assert_eq!(
            suggested_path(Path::new("scan.pdf")),
            PathBuf::from("scan-recognised.pdf")
        );
    }

    /// ★ **Every refusal produces a different sentence.**
    ///
    /// The property the whole `Refusal` enum exists for: `pdfce-core`'s error
    /// types refuse by name because "OCR failed" is unactionable, and a shell
    /// that mapped four named causes onto one sentence would throw that away at
    /// the last step.
    #[test]
    fn each_named_refusal_says_something_different() {
        let all = [
            Refusal::EngineAbsent,
            Refusal::ModelsMissing(vec![PathBuf::from("C:\\a"), PathBuf::from("C:\\b")]),
            Refusal::UnsavedEdits,
            Refusal::NothingRecognised,
            Refusal::Engine("the runtime rejected the model".to_owned()),
        ];
        let mut seen: Vec<String> = Vec::new();
        for refusal in &all {
            let s = sentence(refusal);
            assert!(!s.is_empty(), "{refusal:?} produced no sentence");
            assert!(
                !seen.contains(&s),
                "{refusal:?} repeats a sentence another refusal already uses"
            );
            seen.push(s);
        }
    }

    /// The searched paths survive into the message an operator reads.
    ///
    /// `models::ModelsNotFound` carries them precisely so the operator learns
    /// where to put the files; dropping them at the display boundary would
    /// undo that in the last inch.
    #[test]
    fn a_missing_model_directory_names_every_place_that_was_tried() {
        let s = sentence(&Refusal::ModelsMissing(vec![
            PathBuf::from("C:\\app\\models\\ocrs"),
            PathBuf::from("C:\\users\\x\\models\\ocrs"),
        ]));
        assert!(s.contains("C:\\app\\models\\ocrs"));
        assert!(s.contains("C:\\users\\x\\models\\ocrs"));
    }

    /// A dialog opened with nothing loaded is not built at all.
    #[test]
    fn no_document_means_no_dialog() {
        assert!(open_for(&Status::Empty).is_none());
    }
}
