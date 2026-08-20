//! # `dialogs::unsaved` — the question `file.close` has been promising to ask
//! since it shipped
//!
//! ## The defect
//!
//! Found **2026-08-19**, while auditing this build against `pdfce`'s
//! capability register. `file.close`'s shipped tooltip — an operator-visible
//! string, on the ribbon, in every mode — reads:
//!
//! > *"Close the document. **You are asked what to do about unsaved edits
//! > first.**"*
//!
//! Nothing asked. [`crate::app::actions::Action::Close`] consulted
//! `PdfceApp::save_pending`, which is permanently `false` by design, and then
//! called `close_document()`, which sets `Status::Empty` and drops the
//! `EditSession`. **Every edit made since the file was opened was discarded,
//! silently, with no prompt and no undo.** The same held for
//! [`crate::app::actions::Action::Open`], `New` and `NewSized`, each of which
//! replaces the open document.
//!
//! It is the worst defect this project has found: it destroys work, it destroys
//! it on the operator's own instruction so it never looks like a crash, and the
//! surface **told them it would not happen**.
//!
//! ## ★ Why `save_pending` was not the bug, and must not become the fix
//!
//! The obvious repair is to make `save_pending` return `edit_epoch != 0`. That
//! would be wrong, and `crate::app::lifecycle`'s own header says why in
//! advance: `save_pending` asks *"is a save **in flight**"* — is there a moment
//! at which the bytes on disk are a partial revision and the `EditSession` the
//! writer is reading from must not be dropped. `file.save_copy` is
//! **synchronous**, entered and finished inside one `apply` call with no frame
//! drawn in between, so that state genuinely cannot occur and the honest answer
//! genuinely is `false`.
//!
//! *"Are there unsaved edits?"* is a **different question with a different
//! answer**, and conflating them would have broken the live consumer that
//! module names: `dialogs::ocr`'s `UnsavedEdits` refusal reads `edit_epoch != 0`
//! directly, for the good reason that a successful save-a-copy leaves the
//! document exactly as unsaved as it was — *the copy went somewhere else*.
//!
//! So this is a **second** predicate beside the first, not a redefinition of it,
//! and the two guards compose: a save in flight declines outright; unsaved edits
//! ask. See [`PendingIntent`].
//!
//! ## ★★ The button that is not "Save"
//!
//! Every three-way close prompt an operator has ever seen offers *Save · Don't
//! save · Cancel*, and this one **cannot**, because this build has no Save.
//! `file.save` is in `crate::shell::manifest::PLANNED`, blocked on autosave and
//! crash recovery; the only writer is `file.save_copy`, which writes a **new
//! file somewhere else** and leaves the open document untouched and still
//! unsaved.
//!
//! Labelling that button *Save* would be the same class of lie as the tooltip
//! that started this: an operator would press it, see a file-save dialog, name
//! a file, press Close, and find that the document they were editing still has
//! its original contents on disk. They would have lost nothing — the copy is
//! real — but they would believe something false about which file their work is
//! in, which is worse than losing it *and knowing*.
//!
//! So the button says **"Save a copy…"**, and the sentence beside it says what
//! that means for the file they came from. When `file.save` lands, a fourth
//! button joins it and this paragraph gets shorter; nothing else here changes.
//!
//! ## Why cancelling the file picker cancels the whole thing
//!
//! [`Outcome::SaveCopy`] runs the save and **only resumes the intent if a file
//! was actually written**. A cancelled picker means the operator changed their
//! mind mid-transaction, and the least surprising reading of that is *"leave my
//! document alone"* — not *"close it anyway, unsaved"*, which would be a
//! destructive act reached by pressing Cancel.
//!
//! ## Why the dialog is not a `Modal`
//!
//! egui 0.35 has `egui::Modal`, and this is the one surface in the crate with a
//! genuine claim on it. It is deliberately not used, for the reason every
//! dialog in [`super`] is a `Window`: this crate has exactly one dialog idiom,
//! `ui-verify` drives all of them the same way, and a second idiom introduced
//! for one surface is a second set of layout, focus and escape behaviours to
//! get right. What matters here is not modality but that **the destructive
//! action does not happen until a button is pressed**, which is a property of
//! the control flow, not of the window.

use egui::Ui;

use crate::app::state::Status;
use crate::text::unsaved as t;

/// The region the dialog body publishes.
pub const REGION_BODY: &str = "dialog:unsaved"; // ui-text-exempt: trace region name, never displayed
/// The region the *Save a copy…* button publishes.
pub const REGION_SAVE: &str = "unsaved.save_copy"; // ui-text-exempt: trace region name, never displayed
/// The region the discard button publishes.
pub const REGION_DISCARD: &str = "unsaved.discard"; // ui-text-exempt: trace region name, never displayed
/// The region the Cancel button publishes.
pub const REGION_CANCEL: &str = "unsaved.cancel"; // ui-text-exempt: trace region name, never displayed

/// What the operator asked for, held until they have answered the question.
///
/// # ★ Four variants because four `Action`s replace the open document
///
/// Not "close", which is how this would have been built if it had been written
/// from the tooltip that exposed the defect. `crate::app::lifecycle`'s
/// `save_pending` doc already names the set — *"an Open, a New or a Close must
/// not proceed while a save is pending"* — and `Action::NewSized` joined it on
/// 2026-08-14 **by reusing that predicate rather than growing a second rule**.
///
/// This type is that same set, and building it as a set rather than as a
/// `bool` on the close path is the whole reason Open cannot quietly keep the
/// defect: an operator who has marked up a drawing and then opens the next one
/// has destroyed exactly as much work as one who pressed Close, and is
/// **more** likely to do it, because opening the next file is what you do all
/// day.
///
/// It carries owned data (`Action::Open`'s `PathBuf`) rather than borrowing,
/// because it outlives the frame that raised it by construction — that is what
/// makes it a *pending* intent.
#[derive(Debug, Clone, PartialEq)]
pub enum PendingIntent {
    /// `Action::Close`, and `Action::CloseDocument` once it has brought the
    /// tab it is closing to the front.
    ///
    /// ★ **The only variant anything constructs, since 2026-08-20.** The three
    /// below are unreachable — see the note under this enum.
    Close,
    /// `Action::Open(path)`.
    ///
    /// ⚠ **Never constructed today.** See the note under this enum.
    Open(std::path::PathBuf),
    /// `Action::New`.
    ///
    /// ⚠ **Never constructed today.** See the note under this enum.
    New,
    /// `Action::NewSized`, with its page box in points.
    ///
    /// ⚠ **Never constructed today.** See the note under this enum.
    NewSized {
        /// Page width in points.
        width_pt: f64,
        /// Page height in points.
        height_pt: f64,
    },
}

// ---------------------------------------------------------------------------
// ★★ THREE OF THE FOUR VARIANTS ARE CURRENTLY UNCONSTRUCTED, AND THAT IS
//    RECORDED RATHER THAN DELETED.
//
// Until 2026-08-20, Open, New and NewSized each REPLACED the open document, so
// each asked this question first. Since the document tab strip landed they
// park what is open and add a tab: nothing is discarded, so there is nothing
// to ask about — and asking anyway would put a sentence in front of the
// operator that is **false**, which is how a confirmation gets dismissed
// unread. `crate::app::actions::document`'s header carries the full argument
// and the table of which arms still guard.
//
// They are kept, with their sentences, for one specific reason that is a real
// gap rather than a hedge: **pdfce still asks nothing when the window is
// closed with unsaved edits.** That was true before this change and is now
// true across N documents instead of one. A quit guard is exactly this
// machinery with a fourth intent, and the three sentences here — *"Open
// another document?"*, *"Make a new document?"* — are the shape the fourth
// would take.
//
// If a quit guard lands and does not use them, delete them then. An enum arm
// nothing can reach is dead code wearing a design pattern, and this crate's
// standing preference is to make unreachable states unrepresentable — the
// exception is bought here by a named, dated, still-open gap.
// ---------------------------------------------------------------------------

impl PendingIntent {
    /// The sentence naming what is about to happen to the open document.
    ///
    /// Four sentences rather than one, and it is worth the four: *"Close this
    /// document?"* and *"Open another document?"* are different questions, and
    /// an operator who pressed Open and is asked about closing will read the
    /// prompt as being about a control they did not touch — which is how a
    /// confirmation gets dismissed unread.
    #[must_use]
    pub fn question(&self) -> &'static str {
        match self {
            Self::Close => t::question_close(),
            Self::Open(_) => t::question_open(),
            Self::New | Self::NewSized { .. } => t::question_new(),
        }
    }

    /// The label of the button that goes ahead without saving.
    ///
    /// ★ Named for **what it does**, never *"Yes"* or *"OK"*. The standing
    /// rule this project inherited: a destructive button says the destructive
    /// thing, so that an operator who reads only the buttons — which is most
    /// operators, most of the time — cannot get it wrong. *"Close without
    /// saving"* is unambiguous in a way that *"Yes"* under a question nobody
    /// finished reading is not.
    #[must_use]
    pub fn discard_label(&self) -> &'static str {
        match self {
            Self::Close => t::discard_close(),
            Self::Open(_) => t::discard_open(),
            Self::New | Self::NewSized { .. } => t::discard_new(),
        }
    }
}

/// What the operator chose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Write a copy first; resume only if a file was actually written.
    SaveCopy,
    /// Go ahead and lose the edits.
    Discard,
}

/// The dialog's live state.
///
/// Existence is the "open" state, as everywhere in [`super`]. It holds the
/// intent and one drained answer; there is nothing else to remember, because a
/// confirmation has no draft.
pub struct UnsavedDialog {
    /// What the operator asked for before this question interrupted them.
    intent: PendingIntent,
    /// How many edits are at stake, for the sentence that says so.
    ///
    /// ★ Captured at **open** time rather than read per frame, and the reason
    /// is the dialog's own honesty: this window is the only thing on screen
    /// that can change the document (it cannot), so a live read could only ever
    /// return the same number — but capturing it makes the sentence a statement
    /// about the moment the operator was asked, which is what a confirmation
    /// dialog's text is *for*.
    edits: u64,
    /// Set by a button, drained by the owner.
    outcome: Option<Outcome>,
    /// Set by Cancel and by the window's ✕.
    cancelled: bool,
}

impl UnsavedDialog {
    /// Ask about `intent`, with `edits` unsaved changes at stake.
    #[must_use]
    pub fn new(intent: PendingIntent, edits: u64) -> Self {
        Self {
            intent,
            edits,
            outcome: None,
            cancelled: false,
        }
    }

    /// Take the operator's answer, if they have given one.
    ///
    /// Returns the intent **with** the outcome, because the owner needs both
    /// and holding them apart would let a future edit drain one without the
    /// other — which would resume the wrong intent, silently, on a path whose
    /// failure mode is destroying a document.
    pub fn take_outcome(&mut self) -> Option<(PendingIntent, Outcome)> {
        let outcome = self.outcome.take()?;
        Some((self.intent.clone(), outcome))
    }

    /// Draw it. Returns `false` when it should close.
    pub fn show(&mut self, ctx: &egui::Context) -> bool {
        let mut open = true;
        egui::Window::new(t::title())
            .collapsible(false)
            .resizable(false)
            // ★ Fixed size, no `resizable`, and **no `ScrollArea`**. This is the
            // one dialog in the directory whose content is bounded by
            // construction — three buttons and at most four sentences — so the
            // whole family of reach defects `CONTINUE.md` §7 records cannot
            // arise here. Adding a scroll region "for safety" would create the
            // condition it was meant to prevent.
            .default_size([420.0, 190.0])
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .open(&mut open)
            .show(ctx, |ui| {
                crate::diag::ui_rect(REGION_BODY, ui.max_rect());
                self.body(ui);
            });
        // The ✕ is a Cancel. That is not a convenience: the window's close
        // control must mean the NON-destructive answer, because it is the one
        // an operator presses reflexively to make a surprise go away.
        open && !self.cancelled && self.outcome.is_none()
    }

    /// The body.
    fn body(&mut self, ui: &mut Ui) {
        ui.label(self.intent.question());
        ui.add_space(4.0);
        ui.label(t::edits_at_stake(self.edits));
        ui.add_space(8.0);

        // ★ The buttons are in a fixed left-to-right order and the destructive
        // one is NOT first. Save-a-copy, then discard, then cancel: the reading
        // order runs from the answer that loses nothing to the answer that
        // loses everything, which is the order every application the operator
        // uses puts them in.
        ui.horizontal(|ui| {
            let save = ui.button(t::save_copy_button());
            crate::diag::ui_rect(REGION_SAVE, save.rect);
            if save.clicked() {
                self.outcome = Some(Outcome::SaveCopy);
            }
            let discard = ui.button(self.intent.discard_label());
            crate::diag::ui_rect(REGION_DISCARD, discard.rect);
            if discard.clicked() {
                self.outcome = Some(Outcome::Discard);
            }
            let cancel = ui.button(t::cancel_button());
            crate::diag::ui_rect(REGION_CANCEL, cancel.rect);
            if cancel.clicked() {
                self.cancelled = true;
            }
        });

        ui.add_space(8.0);
        // ★★ The disclosure that makes the first button honest, and it is
        // BELOW the buttons rather than above them on purpose: it is what an
        // operator needs after they have noticed the button says "a copy" and
        // wondered why, and putting it above would make three sentences stand
        // between the question and the answer.
        ui.label(egui::RichText::new(t::save_copy_note()).small().weak());
    }
}

/// Ask about `intent` if the document in `status` has unsaved edits.
///
/// Returns `None` when there is nothing to ask about — no document, or a
/// document nobody has edited — and the caller then proceeds as before. That
/// shape is deliberate: the guard is **one call at the top of an arm** whose
/// `None` answer is the unchanged path, so adding it to a fifth
/// document-replacing action later is one line rather than a new rule.
#[must_use]
pub fn ask_for(status: &Status, intent: PendingIntent) -> Option<UnsavedDialog> {
    let Status::Open(doc) = status else {
        return None;
    };
    // ★ `edit_epoch`, and it is the SAME predicate `dialogs::ocr`'s
    // `UnsavedEdits` refusal already uses. Two independent notions of "this
    // document has been edited" would eventually disagree, and the way they
    // would disagree is that one surface refuses to run OCR on a document
    // another surface is happy to throw away.
    if doc.edit_epoch == 0 {
        return None;
    }
    Some(UnsavedDialog::new(intent, doc.edit_epoch))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An unedited document is closed without a question.
    ///
    /// The property that keeps this from being a nag. `edit_epoch == 0` is the
    /// state a document is in from the moment it opens until the first edit
    /// lands, which is most of the time an operator spends with a file open —
    /// and a confirmation on every close of an unread drawing is exactly the
    /// "nagging" the operator named as having made the old shell worse.
    #[test]
    fn an_unedited_document_is_not_asked_about() {
        assert!(ask_for(&Status::Empty, PendingIntent::Close).is_none());
    }

    /// Each intent asks its own question and offers its own destructive label.
    ///
    /// Asserted as a **relation** rather than against the literals: the point
    /// is that no two intents share a sentence, because an operator who pressed
    /// Open and is asked about closing will read the prompt as being about a
    /// control they did not touch. Comparing against the strings themselves
    /// would pass just as well if all four returned the same one.
    #[test]
    fn the_four_intents_do_not_share_a_sentence() {
        let close = PendingIntent::Close;
        let open = PendingIntent::Open(std::path::PathBuf::from("a.pdf"));
        let new = PendingIntent::New;
        assert_ne!(close.question(), open.question());
        assert_ne!(close.question(), new.question());
        assert_ne!(open.question(), new.question());
        assert_ne!(close.discard_label(), open.discard_label());
        assert_ne!(close.discard_label(), new.discard_label());
        assert_ne!(open.discard_label(), new.discard_label());
    }

    /// A sized New asks the same question as a plain New.
    ///
    /// They are one act with two entry points — `dialogs::new_document` is the
    /// size chooser in front of the same replacement — so asking two different
    /// questions about them would be describing a distinction the operator
    /// cannot see.
    #[test]
    fn both_kinds_of_new_ask_one_question() {
        let sized = PendingIntent::NewSized {
            width_pt: 595.0,
            height_pt: 842.0,
        };
        assert_eq!(sized.question(), PendingIntent::New.question());
        assert_eq!(sized.discard_label(), PendingIntent::New.discard_label());
    }

    /// The answer is a one-shot, and it comes back with its own intent.
    ///
    /// The second `take` returning `None` is what stops the owner resuming the
    /// intent on every frame after one press — which on `PendingIntent::Open`
    /// would re-open the same file forever, and on `Close` would fight anything
    /// the operator opened next.
    #[test]
    fn the_answer_fires_once_and_carries_its_intent() {
        let mut d = UnsavedDialog::new(PendingIntent::Close, 3);
        assert_eq!(d.take_outcome(), None);
        d.outcome = Some(Outcome::Discard);
        assert_eq!(
            d.take_outcome(),
            Some((PendingIntent::Close, Outcome::Discard))
        );
        assert_eq!(d.take_outcome(), None, "it must not repeat");
    }

    /// Cancelling closes the window and answers nothing.
    ///
    /// The two have to be separable: a window that closed *and* answered would
    /// make the ✕ destructive, and the ✕ is the control an operator presses
    /// reflexively to make a surprise go away.
    #[test]
    fn cancelling_answers_nothing() {
        let mut d = UnsavedDialog::new(PendingIntent::Close, 1);
        d.cancelled = true;
        assert_eq!(d.take_outcome(), None);
    }
}
