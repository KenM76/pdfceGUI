//! # `app::status::decline` — the worded decline: saying that a command did
//! *not* run
//!
//! `FEATURES.md`'s Phase 3 row read *"traced and greyed but never worded"*.
//! `crate::canvas::zoom` has always returned
//! [`ZoomOutcome::NoBounds`]/[`ZoomOutcome::NoCanvas`] and always traced them,
//! and `crate::app::dispatch` has always dropped the value on the floor. This
//! module is the half that was missing: a store, a retirement rule, and one
//! line in the status bar's left half.
//!
//! ## ★ The distinction this module exists to hold: a decline is not a
//! disclosure
//!
//! The bar's left half already carries two rule-4 sentences
//! ([`super::fill_disclosure`], [`super::edit_disclosure`]) and they are a
//! different **speech act** from this one:
//!
//! | | says | is true because |
//! |---|---|---|
//! | disclosure | *this happened, and here is the part you cannot see* | a document changed |
//! | decline | *this did not happen* | a document did **not** change |
//!
//! They share the *place* and the *discipline* — the same
//! [`super::disclosure_line`], the same named-region publication, the same
//! R128 fixed row — and they share nothing else. In particular they must not
//! share a store, and the wording must diverge too: *"Nothing to zoom to"* is
//! not *"About your last edit: …"*. One slot and one wording for both would
//! make a completed gesture and a refused one wear the same sentence in the
//! same place, which is **worse than the trace-only state this replaces**.
//!
//! ## ★★ Why this is NOT keyed on [`crate::app::state::OpenDoc::edit_epoch`]
//!
//! The epoch key is what makes the two disclosures safe, and it is exactly
//! what would make this one wrong. Three independent reasons, any one of them
//! sufficient:
//!
//! 1. **A decline changes no document, so the epoch never moves.** The
//!    disclosures retire because the *next edit* bumps the epoch past them,
//!    with no code remembering to clear anything. A decline produces no edit,
//!    so an epoch-keyed decline would never retire — it would still read
//!    "Nothing to zoom to" forty gestures later, which is the precise inverse
//!    of the property that makes the edit disclosure safe.
//! 2. **A decline must be repeatable.** Pressing the chord twice with nothing
//!    selected is **two events**, and the operator needs the second to
//!    register. An epoch key cannot express a repeat, because by construction
//!    nothing changed between the two — the key is identical, so the second
//!    press is indistinguishable from the first never having been retired.
//!    `crate::canvas::zoom::trace_outcome` makes the same ruling on the trace
//!    channel and states it in the same words: *"two identical zoom commands
//!    are two events, and a gate that silenced the second would make a harness
//!    unable to tell a command that ran twice from one that ran once."*
//! 3. **They are different speech acts** — see the table above.
//!
//! ## ★ The precedent that IS right: `page_box`'s clamp note
//!
//! [`super::page_box`] already has a note that is retired **by the operator's
//! next act** rather than by an epoch. Its rule is *"the note is true while
//! you are still where it put you"*, and its test is
//! `page_box::tests::a_clamp_note_is_forgotten_once_the_operator_moves_away`.
//! A decline is that shape, and this module is modelled on it. Two halves,
//! and both are needed:
//!
//! - **[`retire`] at the dispatcher.** `crate::app::dispatch` is the one
//!   choke point every command arrives at, which makes it the one place that
//!   knows *"the operator has just invoked something"*. A decline is retired
//!   there, before the arm for the new command runs — so pressing Ctrl+F, or
//!   clicking Fit page, ends the sentence, and re-pressing the zoom chord ends
//!   it and then raises it again, which is reason 2 above made mechanical.
//! - **[`live`]'s still-true filter at the bar.** Not every act is a command:
//!   *selecting something* is a canvas gesture and reaches no dispatcher. So
//!   the bar draws the sentence only while the reason that produced it is
//!   still true, asked through **the same predicate that produced it**
//!   ([`zoom::can_zoom_to_selection`], [`zoom::last_frame`]) rather than
//!   through a second spelling that could drift. A decline can therefore never
//!   become a lie, only stale — and the dispatcher handles stale.
//!
//! The filter is a *filter* rather than a clear, exactly as
//! [`crate::app::actions::last_edit_disclosure`]'s epoch comparison is: state
//! that must be cleared is state that will one day be shown against the wrong
//! document.
//!
//! ## ★ What this module deliberately does NOT word
//!
//! **The raster-ceiling-clamped region zoom is not a decline — it is a partial
//! grant.** [`ZoomOutcome::Zoomed`] carries both the scale that was asked for
//! and the scale that was pinned, and
//! [`ZoomOutcome::ceiling_changed_the_answer`] reports when they differ. It is
//! tempting to word that here. It would be wrong:
//!
//! - the region **is** framed, centred, at the closest scale the page can go
//!   to — the operator got the honest partial answer, not a refusal;
//! - the clamp **already reports itself**, and does so in the one place an
//!   operator is already looking for a scale: the framing verb raises
//!   `Action::ZoomTo` carrying the *clamped* number, so the zoom readout three
//!   controls to the right states the truth on the same frame.
//!
//! Wording it would word a non-event, and would train the operator to read a
//! decline line that fires when nothing was declined — which is how a surface
//! stops being read. The decision is recorded beside
//! [`ZoomOutcome::ceiling_changed_the_answer`] as well, because that is where
//! the next reader will look.
//!
//! ## Why the store is a thread-local
//!
//! The same answer, for the same reason, as
//! [`crate::app::actions::last_edit_disclosure`]'s `LAST_EDIT` and
//! `crate::panels::forms::edit`'s `LAST_FILL`: it *should* be a field on
//! `OpenDoc`, and `crate::app::state` is not this work's to extend — a
//! **territory boundary rather than a design judgement**, stated here so
//! whoever lifts it knows what the preferred shape is.
//!
//! It is nonetheless sound, and rather more obviously so than its two
//! neighbours: this is not document state at all. It records that a command
//! declined; it cannot change a pixel of the page; nothing reads it except a
//! bar deciding whether to draw a sentence; and `eframe`'s update loop is one
//! thread, so the writer and the reader are the same thread while a test on
//! another thread gets its own empty slot rather than another test's
//! leftovers.
//!
//! One thing it does **not** need that its neighbours do: a document
//! identity. A decline that outlived a document close would be filtered out on
//! the next frame anyway, because a freshly-opened document has drawn no page
//! and has nothing selected — which makes the sentence *true* rather than
//! stale — and the first command the operator invokes retires it.

use std::cell::RefCell;

use crate::app::state::OpenDoc;
use crate::canvas::zoom::{self, ZoomOutcome};
use crate::text::status as t;

/// Named region: the worded decline, when one is live.
///
/// Named for the same reason its two disclosure siblings are: the whole
/// requirement of a decline is that it is **on screen and legible**, and
/// `ui-verify` can only assert that about a rect the application published.
/// Matched literally by `tools/ui-verify`, so renaming it silently un-aims
/// whatever check was measuring it.
const REGION_DECLINE: &str = "status-group:decline"; // ui-text-exempt: trace region name, never displayed

// ---------------------------------------------------------------------------
// What was declined
// ---------------------------------------------------------------------------

/// A framing zoom that did not happen, and why.
///
/// A *narrower* type than [`ZoomOutcome`] on purpose: that enum's third
/// variant is a zoom that **did** happen (possibly clamped, which is a partial
/// grant and not a decline — see the module docs), and a store that could hold
/// it would be a store a future edit could word. This one cannot represent a
/// grant at all.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Declined {
    /// Nothing on the page resolved to a box to frame.
    ///
    /// [`ZoomOutcome::NoBounds`], which `crate::canvas::zoom` raises for three
    /// situations it rules are one from the operator's side — nothing
    /// selected, the selection on another page, or a selection that no longer
    /// resolves after an edit.
    NothingToFrame,
    /// The canvas had not drawn a page, so there was no viewport to frame
    /// anything into. [`ZoomOutcome::NoCanvas`].
    CanvasNotDrawn,
    /// **`file.save_copy` was given a destination and produced no file.**
    ///
    /// The first decline here that is not about zoom, which is why this enum's
    /// two constructors are now [`Declined::of`] (zoom's) and
    /// [`record_save_failure`] (the save's) rather than one. It joins rather
    /// than getting a store of its own for the reason the module header gives
    /// about the two disclosures: a second mechanism beside this one would give
    /// the bar two ways to learn the same *kind* of thing, and the second would
    /// be the one that forgot to retire itself.
    ///
    /// # ★ It is retired by the operator's next act, and by nothing else
    ///
    /// [`Self::still_true`] answers `true` for it unconditionally, and that is a
    /// decision rather than a gap. Its two neighbours have a live predicate to
    /// re-ask — *is something framable now?*, *has the page drawn?* — because
    /// their reasons can stop being true on their own, with the operator doing
    /// nothing. A failed write has no such state: the folder is not going to
    /// reappear on the next frame, and if it did, the sentence would still be a
    /// true report of what happened when the operator pressed Save.
    ///
    /// So it lives exactly as long as [`retire`] lets it — until the next
    /// command — which is `super::page_box`'s clamp-note rule and the one the
    /// module header names as the right precedent. Pressing `Ctrl+S` again
    /// retires it and then records it again, so two failed saves are two events.
    ///
    /// The engine's own reason is **not** carried. It goes to the trace, from
    /// `crate::app::save`; see [`crate::text::status::save_copy_failed`] for why
    /// a `Display` impl's prose is not operator copy.
    SaveFailed,
    /// **The Settings window's Save wrote nothing.**
    ///
    /// # ★ Why this is not [`Self::SaveFailed`], although both are failed writes
    ///
    /// Because the two sentences have to say opposite things about what
    /// happened to the operator's work.
    ///
    /// A failed `file.save_copy` produced **no file**: the operator asked for
    /// something and got nothing, and there is no partial state to explain.
    ///
    /// A failed settings save is the opposite shape. The application **adopted
    /// the configuration anyway** — deliberately, because the operator asked
    /// for it and a disk that refuses should not cost them a choice they made —
    /// so what is true is *"this is in force now and will be gone when pdfce
    /// restarts"*. Reusing the save-a-copy sentence would tell them their
    /// choice did not take, which is false, and they would make it again.
    ///
    /// Sharing a variant would also make the two indistinguishable in the one
    /// place it matters: an operator who pressed Save in two different windows
    /// in one minute.
    ///
    /// # Retired by the operator's next command, like its neighbour
    ///
    /// [`Self::still_true`] answers `true` unconditionally, for the same reason
    /// `SaveFailed` does: the folder is not going to become writable on the
    /// next frame, and if it did, the sentence would still be a true report of
    /// what happened when Save was pressed.
    ///
    /// The engine's own reason is **not** carried — it goes to the trace from
    /// `crate::app::settings_window`, which is where the store location is also
    /// recorded. A `Display` impl's prose is not operator copy.
    SettingsNotSaved,
    /// **`edit.undo` was invoked with an empty command log.**
    ///
    /// # ★ Why this is worded when the control that raises it is greyed
    ///
    /// Because the control is not the only route, and the other route is the
    /// one where greying explains nothing. `edit.undo` is gated on
    /// `undo.available`, so the quick-access button is un-pressable with an
    /// empty log — and it is *also* bound to `Ctrl+Z`, and
    /// [`crate::app::modes::capability::offers_command`] lets it through in
    /// **every** mode because it sits on no tab. So the reachable case is a
    /// chord, fired by an operator whose eyes are on the page rather than on an
    /// 18 pt icon in the title bar.
    ///
    /// That is [`Self::NothingToFrame`]'s *"reached by a chord"* argument with
    /// the reflexes of a whole industry behind it: `Ctrl+Z` is the keystroke an
    /// operator presses without deciding to, and answering the commonest
    /// keystroke in editing with nothing at all is the exact "the button does
    /// nothing" state this project was founded on.
    ///
    /// # It has a live predicate, and it is the one that produced it
    ///
    /// [`Self::still_true`] re-asks `EditSession::can_undo` — the same question
    /// `PdfceApp::conditions` publishes `undo.available` from and the same one
    /// the apply arm declined on. So authoring anything at all retires the
    /// sentence on the next frame, without the operator invoking a command,
    /// which is [`Self::NothingToFrame`]'s shape exactly: *the remedy happened,
    /// so the sentence is history.*
    /// **A widget could not be registered: another field already has the name.**
    ///
    /// `EditError::FieldNameTaken`, raised by `EditSession::adopt_widget`.
    ///
    /// # ★ Why this is refused rather than auto-renamed, which is the engine's
    /// ruling and this shell agrees with it
    ///
    /// ISO 32000-2 SS12.7.3.1 makes the **fully qualified name the field's
    /// identity**. Two top-level fields called `Address` are not two fields —
    /// they are *one field with two widgets*, so typing in either fills both.
    /// No viewer reports this. The operator discovers it by typing into one box
    /// and watching another change, which is the worst possible way to learn it.
    ///
    /// `pageops::assemble` auto-renames on merge because it has nobody to ask.
    /// This surface **does** have somebody to ask, and the engine put it
    /// plainly: *"`Address_2` is a name nobody chose."* So the edit declines and
    /// the operator retypes, with what they typed still in the box in front of
    /// them.
    ///
    /// The clashing name is **not** carried into the sentence. It is a `Copy`
    /// enum, and more to the point the name is already on screen — the operator
    /// typed it seconds ago and it is still in the field they typed it into.
    FieldNameTaken,
    /// **A widget could not be registered because it carries no name of its
    /// own, and none was supplied.**
    ///
    /// `EditError::WidgetHasNoFieldIdentity`.
    ///
    /// # ★ What this actually means, and why the sentence must not say
    /// "recovered"
    ///
    /// It is a **bare kid**: a widget whose `/Parent` pointed at its field, in a
    /// document where that `/Parent` is gone. The engine measured a real form
    /// and found 2 of 13 in this shape after an insert, and named its own cause
    /// — `insert_pages` drops `/Parent` from every dictionary it copies, which
    /// is correct for a page and destroys a widget's only link to its identity.
    ///
    /// What was lost is not just the name. It was the name **and** the field
    /// type, the radio flags and the value. Nothing in this document holds any
    /// of it, so a name typed here **creates a new field**; it does not recover
    /// the old one. The sentence says so, because an operator told they had
    /// "restored" a radio button would go looking for its group.
    WidgetHasNoName,
    NothingToUndo,
    /// **`edit.redo` was invoked with an empty redo stack.**
    ///
    /// Distinct from [`Self::NothingToUndo`] for the reason the module header
    /// gives about the disclosures and the declines generally — the operator
    /// gets **one** line, and these two describe different states with
    /// different remedies. An empty undo log means nothing has been changed at
    /// all; an empty redo stack is the ordinary state of a document that has
    /// been edited and never undone, and it is *also* what a fresh edit after an
    /// undo produces, because `EditSession::commit` clears the redo stack when a
    /// new command is recorded (*"the redone future no longer exists once
    /// history diverges"*).
    ///
    /// Its live predicate is `EditSession::can_redo`, for
    /// [`Self::NothingToUndo`]'s reason and asked the same way.
    NothingToRedo,
}

impl Declined {
    /// The decline in an outcome, if it is one.
    ///
    /// `None` for [`ZoomOutcome::Zoomed`] **including the clamped case**. See
    /// the module docs: a clamped framing zoom is a partial grant that already
    /// reports itself through the zoom readout, and wording it here would word
    /// a non-event.
    #[must_use]
    pub(crate) fn of(outcome: ZoomOutcome) -> Option<Self> {
        match outcome {
            ZoomOutcome::NoBounds => Some(Self::NothingToFrame),
            ZoomOutcome::NoCanvas => Some(Self::CanvasNotDrawn),
            ZoomOutcome::Zoomed { .. } => None,
        }
    }

    /// Whether this decline still describes the application in front of the
    /// operator.
    ///
    /// **Pure, and that is the point** — the project's standing split
    /// (`crate::viewer`'s header: *"this module is unit-testable and the widget
    /// code is not"*). Every property of the retirement rule that can be wrong
    /// is decided here and asserted headlessly; [`live`] adds only "go and ask
    /// the two questions".
    ///
    /// The facts are named as booleans rather than taken as a `&OpenDoc`
    /// so that the caller is forced to state *which* question it asked. All
    /// are asked through the same predicates that produced the decline in the
    /// first place, which is what stops a second spelling of "is there
    /// anything to frame?" drifting away from the first.
    ///
    /// # ★ Why a fourth parameter rather than a `&OpenDoc`
    ///
    /// [`History`] arrived with the undo wiring and needed a third fact — *is
    /// there anything on the stack now?* — which is where the temptation to
    /// collapse the list into the document it is all read from is strongest.
    /// The list stays, for the reason it was a list to begin with: a
    /// `&OpenDoc` here would make this function able to ask **any** question,
    /// and the one property that makes it worth testing is that every question
    /// it asks was asked by the code that produced the decline. The parameters
    /// are the contract; [`live`] is the only place allowed to go and get them.
    ///
    /// The two history variants take their fact as *one* [`History`] pair
    /// rather than as two more booleans, so a caller cannot transpose them —
    /// and each arm below names the field it reads, so neither can read the
    /// other's stack.
    #[must_use]
    fn still_true(self, has_bounds: bool, canvas_has_drawn: bool, history: History) -> bool {
        match self {
            // The operator has selected something framable: the sentence is
            // now history, and a stale explanation beside a live control is
            // worse than none — it attaches a refusal to a state that would
            // not produce one.
            Self::NothingToFrame => !has_bounds,
            // The page has drawn. The remedy happened on its own, without the
            // operator doing anything, which is exactly what the sentence
            // promised ("…has not finished drawing").
            Self::CanvasNotDrawn => !canvas_has_drawn,
            // ★ Neither fact is about this one, and there is no third fact to
            // add. A write that failed stays failed until the operator does
            // something about it, and what they do about it is a *command* —
            // which `retire` catches. See the variant's own docs; the two
            // parameters are deliberately ignored rather than being joined by a
            // third that would always be `true`.
            Self::SaveFailed | Self::SettingsNotSaved => true,
            // ★ Same ruling, third and fourth cases. A name is not going to
            // stop being taken, and a widget is not going to grow a `/T`,
            // between one frame and the next. Both are corrected by the
            // operator doing something — typing a different name and pressing
            // Register again — and pressing Register is a command, which
            // `retire` catches.
            Self::FieldNameTaken | Self::WidgetHasNoName => true,
            // ★ The stack filled up. Something was authored — or, for redo,
            // something was undone — and the sentence is now history, exactly
            // as `NothingToFrame` is once something is selected. The operator
            // reaches this without invoking any command, which is why the
            // filter is needed at all: `retire` would not have run.
            Self::NothingToUndo => !history.can_undo,
            Self::NothingToRedo => !history.can_redo,
        }
    }

    /// The sentence, from the catalog.
    ///
    /// The mapping is the whole of this module's contribution to the copy;
    /// every word an operator reads is [`crate::text::status`]'s, under rule
    /// R1.
    #[must_use]
    fn line(self) -> &'static str {
        match self {
            Self::NothingToFrame => t::zoom_declined_no_selection(),
            Self::CanvasNotDrawn => t::zoom_declined_not_drawn(),
            Self::SaveFailed => t::save_copy_failed(),
            Self::SettingsNotSaved => t::settings_not_saved(),
            Self::NothingToUndo => t::undo_declined_empty(),
            Self::NothingToRedo => t::redo_declined_empty(),
            Self::FieldNameTaken => t::adopt_declined_name_taken(),
            Self::WidgetHasNoName => t::adopt_declined_no_name(),
        }
    }
}

/// **What the command log says right now** — the fact
/// [`Declined::NothingToUndo`] and [`Declined::NothingToRedo`] are retired by.
///
/// A pair rather than two parameters because they are read together, from one
/// borrow of one session, and because a caller that had to pass two loose
/// booleans in the right order would eventually pass them in the wrong one —
/// and the symptom would be a sentence that retires when the *other* stack
/// fills, which reads exactly like a sentence that retires correctly.
///
/// Both are asked through `EditSession`'s own predicates, which is the same
/// pair `crate::app::conditions` publishes `undo.available`/`redo.available`
/// from and the same pair `crate::app::actions`' history arm declines on. Three
/// readers, one derivation: the control cannot be greyed while the sentence
/// says the opposite.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct History {
    /// `EditSession::can_undo` — something has been changed and not taken back.
    pub(crate) can_undo: bool,
    /// `EditSession::can_redo` — something has been taken back and not
    /// re-applied, and no command has been recorded since.
    pub(crate) can_redo: bool,
}

impl History {
    /// What the open document's session currently says.
    ///
    /// The one derivation, so the bar cannot learn this from a different
    /// question than the one that produced the sentence.
    #[must_use]
    fn of(doc: &OpenDoc) -> Self {
        Self {
            can_undo: doc.session.can_undo(),
            can_redo: doc.session.can_redo(),
        }
    }
}

thread_local! {
    /// The most recent declined command, waiting to be read by the status
    /// bar. See the module docs for why a thread-local, and why that is sound
    /// rather than smuggled.
    static LAST: RefCell<Option<Declined>> = const { RefCell::new(None) };
}

// ---------------------------------------------------------------------------
// The store — written by the dispatcher, read by the bar
// ---------------------------------------------------------------------------

/// Record what a framing zoom did, so the bar can say so if it declined.
///
/// **Written unconditionally**, including for a grant, which stores `None`.
/// That mirrors `crate::app::actions::record_edit_disclosure`'s discipline:
/// the slot never holds a sentence whose only defence against being shown is a
/// filter somewhere else. A zoom that *worked* silences a decline immediately,
/// on the same call, rather than relying on [`retire`] having been reached.
///
/// Called from `crate::app::dispatch`'s `view.zoom_selection` arm — which is
/// still routing rather than computing: it hands over the value the verb
/// returned and decides nothing about it.
pub(crate) fn record(outcome: ZoomOutcome) {
    let declined = Declined::of(outcome);
    LAST.with_borrow_mut(|slot| *slot = declined);
}

/// Record that `file.save_copy` was given a destination and produced no file.
///
/// Called from `crate::app::save::write_and_report`, which is in the **apply**
/// phase rather than in the dispatcher — the one difference from [`record`]'s
/// call site, and it is why this is a separate entry point rather than a second
/// argument to that one. [`retire`] runs at the top of `dispatch_command`, so a
/// sentence recorded during the apply of the *same* frame survives it: the
/// order is dispatch (retire, raise the action) → apply (write, record) → next
/// frame (the bar draws it) → the operator's next command (retire).
///
/// Unconditional, and there is deliberately no matching "the save worked" call
/// that stores `None`. A successful save-a-copy produces a file at a path the
/// operator typed into a dialog they were looking at, which is the most visible
/// confirmation this application has; adding a sentence for it would narrate
/// what they just did. Two saves in a row, one failing and one succeeding, are
/// still handled — the second press retires the first's sentence through
/// [`retire`] before its own arm runs.
pub(crate) fn record_save_failure() {
    LAST.with_borrow_mut(|slot| *slot = Some(Declined::SaveFailed));
}

/// Record that the Settings window's Save reached no disk.
///
/// Called from `crate::app::settings_window::save_settings`, and **only** on
/// the failure path — there is deliberately no matching "the settings saved"
/// call. A successful settings save is not narrated for the same reason a
/// successful save-a-copy is not: the operator pressed a button in a window
/// they were looking at, the window closed, and a sentence telling them so
/// would narrate what they just did.
///
/// # ★ What must be true at the call site before this is reached
///
/// The configuration has **already been adopted**. That ordering is the whole
/// meaning of [`Declined::SettingsNotSaved`]'s sentence, and calling this
/// before the adoption — or instead of it — would make the sentence a lie in
/// the more damaging direction: the operator would be told their choice is
/// in force for this session when it is not.
pub(crate) fn record_settings_not_saved() {
    LAST.with_borrow_mut(|slot| *slot = Some(Declined::SettingsNotSaved));
}

/// Record that `edit.undo` or `edit.redo` arrived with an empty stack.
///
/// Called from `crate::app::actions::apply`'s history arm — the **apply**
/// phase, exactly as [`record_save_failure`] is, and for the same reason: the
/// arm that can tell is the one holding the session, and [`retire`] runs at the
/// top of `dispatch_command`, so a sentence recorded during the apply of the
/// same frame survives it.
///
/// # ★ Why the dispatcher does not decide this
///
/// It could: `PdfceApp` has the session, and `view.zoom_selection` sets the
/// precedent of a dispatch arm recording an outcome. It must not, because the
/// dispatcher's arms **route** (`HANDOFF.md` §6), and "is there anything to
/// undo?" is a question about the document that the apply phase has to ask
/// anyway before it touches the session. Asking it in both places is how the
/// greyed control and the sentence come to disagree.
///
/// **`Declined` is the parameter rather than a `bool`**, so the call site reads
/// as the state it is reporting and a third stack would not silently become a
/// fourth meaning of `true`. Only the two history variants are constructible
/// here in practice; passing anything else records a decline the arm did not
/// mean, which is why the two callers are one arm apart in one function.
pub(crate) fn record_history_empty(declined: Declined) {
    LAST.with_borrow_mut(|slot| *slot = Some(declined));
}

/// Record that `adopt_widget` refused, and which of its two correctable
/// refusals it was.
///
/// Called from `crate::app::actions::forms`, the apply phase, exactly as
/// [`record_history_empty`] is and for the same reason: the arm holding the
/// session is the one that can tell.
///
/// # ★ Why only two of the engine's five refusals reach here
///
/// `adopt_widget` refuses five ways. Three of them cannot happen from this
/// surface and wording them would be wording states the operator cannot be in:
///
/// | refusal | why it is unreachable here |
/// |---|---|
/// | `NotAWidget` | the ids come from `page_annotations(..).is_widget()`, in this document |
/// | `WidgetAlreadyOwned` | the ids are exactly the ones no field claimed, from the same walk |
/// | `FieldNameEmpty` | the box is trimmed and an empty one sends `None`, not `Some("")` |
///
/// They still reach the trace through [`super::super::actions::apply::vector_edit`]'s
/// error branch, which is where an impossible refusal belongs: visible to
/// whoever is debugging, absent from the status bar an operator reads.
pub(crate) fn record_adopt_refusal(declined: Declined) {
    LAST.with_borrow_mut(|slot| *slot = Some(declined));
}

/// Forget any live decline — **the operator's next act**.
///
/// Called at the top of `crate::app::dispatch::PdfceApp::dispatch_command`,
/// before the arm for the new command runs. That placement is the whole
/// retirement rule and it is deliberate on both counts:
///
/// - **the dispatcher**, because it is the one choke point that knows an
///   operator has invoked *something*, and "the next thing you did" is the
///   only honest lifetime for a sentence about a gesture. See the module docs
///   for why an epoch cannot serve here;
/// - **before the arm**, so that re-pressing the declining chord retires the
///   old sentence and then [`record`]s a new one. Two presses are two events
///   (module docs, reason 2), and this is where that becomes mechanical rather
///   than aspirational.
///
/// Idempotent and free: one `Option` write per *invoked command*, which is an
/// operator click, not a frame.
pub(crate) fn retire() {
    LAST.with_borrow_mut(|slot| *slot = None);
}

/// The live decline, if there is one and it still describes what the operator
/// is looking at.
///
/// The bar's read. Both facts are gathered from the modules that own them —
/// [`zoom::can_zoom_to_selection`] is the same predicate `view.zoom_selection`
/// is gated on and the same one [`zoom::zoom_to_selection`] declines from, and
/// [`zoom::last_frame`] is the same record the framing verbs check for
/// [`ZoomOutcome::NoCanvas`]. Asking the producing predicate rather than an
/// equivalent-looking one (`doc.page_texture.is_some()`, say, which is a
/// *different* question by one frame) is what keeps the retirement rule from
/// drifting away from the decline it retires.
///
/// Filters rather than clears; see the module docs.
#[must_use]
pub(super) fn live(ctx: &egui::Context, doc: &OpenDoc) -> Option<Declined> {
    let has_bounds = zoom::can_zoom_to_selection(doc);
    let canvas_has_drawn = zoom::last_frame(ctx).is_some();
    let history = History::of(doc);
    LAST.with_borrow(|slot| {
        slot.filter(|d| d.still_true(has_bounds, canvas_has_drawn, history))
            .to_owned()
    })
}

// ---------------------------------------------------------------------------
// The line
// ---------------------------------------------------------------------------

/// Draw the worded decline into the bar's single row, if one is live.
///
/// Drawn through [`super::disclosure_line`] rather than by hand, which is the
/// point of that function existing: the R128 defence is four small rules that
/// only work together — a bounded sub-region, a fixed row height,
/// `truncate()` rather than wrapping, and the full text on hover — and a third
/// hand-written copy would be a third chance to omit one of them.
///
/// **It does not make the bar taller**, and that matters more here than for
/// its neighbours rather than less. A decline arrives from a *keyboard chord*,
/// which is the gesture during which the operator's hands are furthest from
/// the thing they are looking at; if this line grew the bar, an active
/// `FitMode` would recompute its zoom from a smaller viewport on the very next
/// frame and the page would shrink under a gesture that, by construction,
/// changed nothing. "The page moved when the command did nothing" is the
/// worst-reading symptom on this surface.
/// [`tests::a_worded_decline_does_not_change_the_bar_height`] pins it.
pub(super) fn show(ui: &mut egui::Ui, doc: &OpenDoc) {
    let Some(declined) = live(ui.ctx(), doc) else {
        return;
    };
    super::disclosure_line(ui, REGION_DECLINE, declined.line());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::Status;
    use crate::app::status::test_support::{opened, settled_bar_frame};
    use egui::Context;

    // =======================================================================
    // The retirement rule — pure, so every property is pinned without a window
    // =======================================================================

    /// ★ **A decline is retired by the state that produced it stopping being
    /// true**, and by nothing else.
    ///
    /// The full matrix, both directions on every variant. The "still true"
    /// direction is the one worth stating explicitly: a decline whose reason
    /// still holds must survive, or the sentence would flicker off on the next
    /// frame and the operator would never read it.
    #[test]
    fn a_decline_lives_exactly_as_long_as_its_reason() {
        // An empty command log — the state the two zoom declines were written
        // against, and the one every assertion below that is not about the
        // history is indifferent to.
        let empty = History::default();

        // Nothing to frame: retired the moment something is framable, and
        // indifferent to whether the canvas has drawn.
        assert!(Declined::NothingToFrame.still_true(false, true, empty));
        assert!(Declined::NothingToFrame.still_true(false, false, empty));
        assert!(!Declined::NothingToFrame.still_true(true, true, empty));
        assert!(!Declined::NothingToFrame.still_true(true, false, empty));

        // Canvas not drawn: retired the moment it has, and indifferent to the
        // selection — the remedy arrives without the operator doing anything.
        assert!(Declined::CanvasNotDrawn.still_true(false, false, empty));
        assert!(Declined::CanvasNotDrawn.still_true(true, false, empty));
        assert!(!Declined::CanvasNotDrawn.still_true(false, true, empty));
        assert!(!Declined::CanvasNotDrawn.still_true(true, true, empty));

        // ★ A failed save survives every combination of the two facts, because
        // neither is about it: a folder that could not be written to does not
        // become writable because the operator selected something or because a
        // page finished drawing. It is retired by `retire` — the operator's
        // next command — and by nothing else. Asserted over the whole matrix
        // rather than once, so a future edit that "tidied" this variant into
        // one of the two predicates fails here instead of making the sentence
        // vanish on the next raster.
        for has_bounds in [false, true] {
            for drawn in [false, true] {
                assert!(
                    Declined::SaveFailed.still_true(has_bounds, drawn, empty),
                    "a failed write does not repair itself ({has_bounds}, {drawn})"
                );
            }
        }
    }

    /// ★ **Each history decline is retired by ITS OWN stack filling, and by
    /// the other's it is not.**
    ///
    /// The cross terms are the reason this is a separate test rather than four
    /// more lines in the matrix above. A build whose two arms read the same
    /// field — the mistake a two-field struct makes available, and the reason
    /// [`History`]'s doc comment argues for it over two loose booleans — would
    /// pass every same-stack assertion and fail only here.
    ///
    /// The remedy arriving *without a command* is the whole point: authoring a
    /// rectangle is a canvas gesture that reaches no dispatcher, so [`retire`]
    /// never runs and only this filter can end the sentence. That is
    /// [`Declined::NothingToFrame`]'s property, and it is why both of these
    /// have a live predicate at all rather than [`Declined::SaveFailed`]'s
    /// unconditional `true`.
    #[test]
    fn a_history_decline_is_retired_by_its_own_stack() {
        let empty = History::default();
        let undoable = History {
            can_undo: true,
            can_redo: false,
        };
        let redoable = History {
            can_undo: false,
            can_redo: true,
        };

        // Its own stack is what retires it…
        assert!(Declined::NothingToUndo.still_true(false, true, empty));
        assert!(!Declined::NothingToUndo.still_true(false, true, undoable));
        assert!(Declined::NothingToRedo.still_true(false, true, empty));
        assert!(!Declined::NothingToRedo.still_true(false, true, redoable));

        // …and the OTHER stack is not. An operator who authors something can
        // undo it and still has nothing to redo, so a "nothing to redo"
        // sentence that vanished when the undo stack filled would retire on a
        // state that has not changed for it.
        assert!(
            Declined::NothingToRedo.still_true(false, true, undoable),
            "an undoable change is not something to redo"
        );
        assert!(
            Declined::NothingToUndo.still_true(false, true, redoable),
            "a redoable change is not something to undo"
        );

        // Indifferent to the two zoom facts, in every combination: neither the
        // selection nor the raster has anything to do with a command log.
        for has_bounds in [false, true] {
            for drawn in [false, true] {
                assert!(Declined::NothingToUndo.still_true(has_bounds, drawn, empty));
                assert!(Declined::NothingToRedo.still_true(has_bounds, drawn, empty));
            }
        }
    }

    /// ★ **Undo's and redo's declines are two sentences, recorded by name.**
    ///
    /// [`record_history_empty`] takes the value rather than a `bool`, and the
    /// property that buys is asserted here: pressing `Ctrl+Y` with an empty
    /// redo stack must not leave the bar saying the document has no changes.
    /// The ordering half is [`Declined::SaveFailed`]'s, already pinned above —
    /// both record in the apply phase, after the frame's `retire`.
    #[test]
    fn the_two_history_declines_do_not_share_a_slot_or_a_sentence() {
        retire();
        record_history_empty(Declined::NothingToUndo);
        assert_eq!(
            LAST.with_borrow(|slot| *slot),
            Some(Declined::NothingToUndo)
        );
        retire();
        record_history_empty(Declined::NothingToRedo);
        assert_eq!(
            LAST.with_borrow(|slot| *slot),
            Some(Declined::NothingToRedo)
        );
        assert_ne!(
            Declined::NothingToUndo.line(),
            Declined::NothingToRedo.line(),
            "one line reaches the operator; two states that need different \
             sentences must not share one"
        );
        retire();
    }

    /// ★ **A failed save is recorded, survives a frame, and is retired by the
    /// operator's next command — so two failed saves are two events.**
    ///
    /// The store half of [`Declined::SaveFailed`], and the ordering is the
    /// interesting part: [`retire`] runs at the top of `dispatch_command` while
    /// [`record_save_failure`] runs in the **apply** phase of the same frame,
    /// which is later. A sentence recorded by a save therefore survives the
    /// dispatch that raised it, and is cleared by the *next* command — which is
    /// what makes a second `Ctrl+S` record a second sentence rather than
    /// re-showing the first.
    ///
    /// Reversing those two would be silent: the bar would simply never draw the
    /// line, and a reader of the trace would still see `save-copy-failed`.
    #[test]
    fn a_failed_save_is_recorded_and_retired_by_the_next_command() {
        retire();
        record_save_failure();
        assert_eq!(
            LAST.with_borrow(|slot| *slot),
            Some(Declined::SaveFailed),
            "the failure must reach the store, or the bar has nothing to draw"
        );

        // The frame's own dispatch already ran before the apply that recorded
        // this, so the sentence is still there on the next frame.
        assert!(Declined::SaveFailed.still_true(true, true, History::default()));

        // …and the operator's next command ends it.
        retire();
        assert_eq!(LAST.with_borrow(|slot| *slot), None);

        // Two failures in a row are two events: the second press retires the
        // first sentence through `retire` and then records its own.
        record_save_failure();
        retire();
        record_save_failure();
        assert_eq!(LAST.with_borrow(|slot| *slot), Some(Declined::SaveFailed));
        retire();
    }

    /// ★ **A clamped framing zoom is not a decline.**
    ///
    /// The one case this module is deliberately blind to. A region zoom past
    /// the page's raster ceiling still zooms, still centres what was asked
    /// for, and raises `Action::ZoomTo` carrying the clamped scale — so the
    /// bar's own zoom readout states the truth on the same frame. Wording it
    /// would word a non-event.
    ///
    /// Asserted for the clamped case *and* the exact one, because a store that
    /// happened to reject only the exact case would pass a test written the
    /// obvious way and still ship the sentence nobody wants.
    #[test]
    fn a_partial_grant_is_not_a_decline() {
        let clamped = ZoomOutcome::Zoomed {
            requested: 40.0,
            applied: crate::viewer::MAX_ZOOM,
        };
        assert!(
            clamped.ceiling_changed_the_answer(),
            "the fixture must really be the clamped case, or this proves nothing"
        );
        assert_eq!(
            Declined::of(clamped),
            None,
            "the ceiling reports itself through the zoom readout; a second \
             report in words would fire when nothing was declined"
        );
        assert_eq!(
            Declined::of(ZoomOutcome::Zoomed {
                requested: 2.0,
                applied: 2.0
            }),
            None
        );

        // …and both genuine declines are carried.
        assert_eq!(
            Declined::of(ZoomOutcome::NoBounds),
            Some(Declined::NothingToFrame)
        );
        assert_eq!(
            Declined::of(ZoomOutcome::NoCanvas),
            Some(Declined::CanvasNotDrawn)
        );
    }

    /// Each decline says its own thing, from the catalog.
    ///
    /// Three now rather than two, and asserted pairwise: the operator gets one
    /// line, and "nothing is selected", "the page is still drawing" and "the
    /// copy was not written" have three different remedies. A shared sentence
    /// would be a decline that does not say which command declined.
    #[test]
    fn no_two_declines_share_a_sentence() {
        let all = [
            Declined::NothingToFrame,
            Declined::CanvasNotDrawn,
            Declined::SaveFailed,
        ];
        for (i, a) in all.iter().enumerate() {
            for b in &all[i + 1..] {
                assert_ne!(a.line(), b.line(), "{a:?} and {b:?} read the same");
            }
        }
    }

    // =======================================================================
    // The store — recorded, retired, and repeatable
    // =======================================================================

    /// ★ **Two presses are two events**, and the second one registers.
    ///
    /// This is the property an edit-epoch key **cannot** express, and the
    /// reason this module has a store of its own: a decline changes no
    /// document, so an epoch-keyed sentence would be identical on both presses
    /// and would never retire in between. Here the sequence
    /// *decline → the operator does something else → decline again* puts the
    /// sentence back, which is what makes the second press an answer rather
    /// than a swallowed keystroke.
    #[test]
    fn a_decline_can_be_raised_again_after_the_operator_moves_on() {
        let ctx = Context::default();
        let status = opened();
        let Status::Open(doc) = &status else {
            unreachable!("`opened()` returns an open document")
        };

        record(ZoomOutcome::NoBounds);
        assert_eq!(live(&ctx, doc), Some(Declined::NothingToFrame));

        // The operator's next act — any command at all.
        retire();
        assert_eq!(live(&ctx, doc), None, "the next command ends the sentence");

        // …and pressing the chord again is a second event, not a repeat of a
        // sentence that was never taken down.
        record(ZoomOutcome::NoBounds);
        assert_eq!(
            live(&ctx, doc),
            Some(Declined::NothingToFrame),
            "the second press must register, or the operator has pressed a \
             chord and been told nothing"
        );
    }

    /// A framing zoom that *worked* silences a decline on the spot, rather
    /// than leaving it to be retired by whatever comes next.
    #[test]
    fn a_successful_zoom_takes_the_sentence_down_itself() {
        let ctx = Context::default();
        let status = opened();
        let Status::Open(doc) = &status else {
            unreachable!("`opened()` returns an open document")
        };

        record(ZoomOutcome::NoBounds);
        assert!(live(&ctx, doc).is_some());
        record(ZoomOutcome::Zoomed {
            requested: 2.0,
            applied: 2.0,
        });
        assert_eq!(live(&ctx, doc), None);
    }

    // =======================================================================
    // The wiring — through the real dispatcher
    // =======================================================================

    /// ★ **The dispatcher words the decline, and the next command retires
    /// it.**
    ///
    /// Driven through `PdfceApp::dispatch_command`, which is the same entry
    /// point a ribbon click, a quick-access click and a keyboard chord all
    /// reach — so what is asserted is the real routing rather than a
    /// hand-assembled approximation of it.
    ///
    /// Three steps, and the middle one is the point of the whole module:
    ///
    /// 1. `view.zoom_selection` on a freshly-opened document declines. Nothing
    ///    is selected, so `zoom_to_selection` returns `NoBounds` before it ever
    ///    looks for a canvas frame — which is why the expected variant is
    ///    `NothingToFrame` and not `CanvasNotDrawn` even though the canvas has
    ///    also never drawn here.
    /// 2. The sentence is **live**, which is the thing that used to be
    ///    missing: the outcome reached the bar instead of the floor.
    /// 3. Any other command retires it. Asserted with `view.zoom_actual` — an
    ///    ordinary, unrelated verb — because the rule is "the operator's next
    ///    act", not "an act about zooming".
    ///
    ///    ★ It was `view.zoom_in` until 2026-08-15, when that arm was deleted
    ///    as one of the four `shell::commands::reach::UNREACHED_ARMS` — an arm
    ///    for an id no token names. The assertion would still have passed,
    ///    because `retire()` runs *above* the `match` and an unimplemented id
    ///    reaches the catch-all — which is exactly why it was changed: a test
    ///    whose subject is "any other **command**" must name one that exists,
    ///    or it is quietly asserting something weaker than it says.
    #[test]
    fn the_dispatcher_words_a_decline_and_the_next_command_retires_it() {
        let ctx = Context::default();
        let mut app = crate::app::tests::opened();
        retire();

        app.dispatch_command(&ctx, "view.zoom_selection", &mut Vec::new());
        {
            let Status::Open(doc) = &app.status else {
                unreachable!("the fixture is open")
            };
            assert_eq!(
                live(&ctx, doc),
                Some(Declined::NothingToFrame),
                "the outcome `zoom_to_selection` returned reached the bar; \
                 before this row was built it was dropped on the floor"
            );
        }

        app.dispatch_command(&ctx, "view.zoom_actual", &mut Vec::new());
        let Status::Open(doc) = &app.status else {
            unreachable!("the fixture is open")
        };
        assert_eq!(
            live(&ctx, doc),
            None,
            "a sentence about a gesture must not outlive the gesture after it \
             — that is the failure an edit-epoch key would have shipped"
        );
    }

    // =======================================================================
    // R128 — the height that must not move
    // =======================================================================

    /// ★ **A worded decline does not change the bar's height** — R128 for the
    /// sentence a refused command puts there.
    ///
    /// # Why this needs its own test beside the edit-disclosure one
    ///
    /// Same rule, different arrival, and this arrival is the awkward one. The
    /// edit disclosure follows a drag; this follows a **keyboard chord**, and
    /// a chord is precisely the gesture where the operator is looking at the
    /// page rather than at their hands. If this line grew the bar, an active
    /// `FitMode` would recompute its zoom from a viewport one row smaller on
    /// the next frame, and the page would visibly shrink in response to a
    /// command that **did nothing at all**. R128's measured symptom is *"the
    /// page jumped when I clicked an object"*; this variant would read as
    /// *"the page moved when the command was refused"*, and it would be
    /// investigated in the zoom code, where nothing is wrong.
    ///
    /// # The three assertions, and why none of them is the obvious one
    ///
    /// 1. **A measurement happened at all** (`Some(_)`, never `None`) —
    ///    `HANDOFF.md` §10's rule. `cargo test -p egui-shell` and `cargo test
    ///    --workspace` compile `egui` with different features (no fonts vs
    ///    `default_fonts`), so a layout assertion can be entirely vacuous
    ///    under one of the two commands a developer runs.
    /// 2. **The sentence reached the painter** — more shapes with the decline
    ///    live than without it. Without this, assertion 3 is satisfied just as
    ///    well by a [`show`] that returned early and drew nothing, which is
    ///    true and proves nothing.
    /// 3. **The height did not move.** Asserted as `Some(true)` rather than
    ///    with a bare `assert!`, so a run in which either frame failed to
    ///    measure reads as `None` and fails, rather than reading as agreement.
    ///
    /// [`Declined::NothingToFrame`] is the case tested because it is the one
    /// an operator will actually reach, and because its sentence is the longer
    /// of the two — the defence against a long sentence is eliding inside a
    /// bounded sub-region with the whole text on hover, never wrapping,
    /// because wrapping is how a one-row bar becomes a two-row bar.
    #[test]
    fn a_worded_decline_does_not_change_the_bar_height() {
        let ctx = Context::default();
        let status = opened();
        let Status::Open(doc) = &status else {
            unreachable!("`opened()` returns an open document")
        };

        retire();
        let absent = settled_bar_frame(&ctx, &status);

        record(ZoomOutcome::NoBounds);
        // The precondition, asserted rather than assumed: without it every
        // comparison below measures that an absent line did not change the
        // height, which is true and worthless.
        assert!(
            live(&ctx, doc).is_some(),
            "the recorded decline is not live for this document, so the bar \
             drew no line and everything below proves nothing"
        );

        let present = settled_bar_frame(&ctx, &status);

        let drew = match (absent, present) {
            (Some((_, before)), Some((_, after))) => Some(after > before),
            _ => None,
        };
        assert_eq!(
            drew,
            Some(true),
            "the bar painted no more shapes with a live decline ({present:?}) \
             than without one ({absent:?}); the sentence never reached the \
             painter, so the height comparison would be vacuous. `None` here \
             means a frame did not measure at all, which is the other failure \
             and is not a pass"
        );

        let same_height = match (absent, present) {
            (Some((before, _)), Some((after, _))) => Some((after - before).abs() < 0.01),
            _ => None,
        };
        assert_eq!(
            same_height,
            Some(true),
            "a worded decline changed the bar's height ({absent:?} → \
             {present:?}); that re-fits the page on the frame a command \
             refused to do anything, which is the one gesture that must \
             provably move nothing"
        );

        retire();
    }
}
