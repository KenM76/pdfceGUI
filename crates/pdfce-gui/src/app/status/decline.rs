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
    /// The two facts are named as booleans rather than taken as a `&OpenDoc`
    /// so that the caller is forced to state *which* question it asked. Both
    /// are asked through the same predicates that produced the decline in the
    /// first place, which is what stops a second spelling of "is there
    /// anything to frame?" drifting away from the first.
    #[must_use]
    fn still_true(self, has_bounds: bool, canvas_has_drawn: bool) -> bool {
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
            Self::SaveFailed => true,
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
    LAST.with_borrow(|slot| {
        slot.filter(|d| d.still_true(has_bounds, canvas_has_drawn))
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
    /// The full matrix, both directions on both variants. The "still true"
    /// direction is the one worth stating explicitly: a decline whose reason
    /// still holds must survive, or the sentence would flicker off on the next
    /// frame and the operator would never read it.
    #[test]
    fn a_decline_lives_exactly_as_long_as_its_reason() {
        // Nothing to frame: retired the moment something is framable, and
        // indifferent to whether the canvas has drawn.
        assert!(Declined::NothingToFrame.still_true(false, true));
        assert!(Declined::NothingToFrame.still_true(false, false));
        assert!(!Declined::NothingToFrame.still_true(true, true));
        assert!(!Declined::NothingToFrame.still_true(true, false));

        // Canvas not drawn: retired the moment it has, and indifferent to the
        // selection — the remedy arrives without the operator doing anything.
        assert!(Declined::CanvasNotDrawn.still_true(false, false));
        assert!(Declined::CanvasNotDrawn.still_true(true, false));
        assert!(!Declined::CanvasNotDrawn.still_true(false, true));
        assert!(!Declined::CanvasNotDrawn.still_true(true, true));

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
                    Declined::SaveFailed.still_true(has_bounds, drawn),
                    "a failed write does not repair itself ({has_bounds}, {drawn})"
                );
            }
        }
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
        assert!(Declined::SaveFailed.still_true(true, true));

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
    /// 3. Any other command retires it. Asserted with `view.zoom_in` — an
    ///    ordinary, unrelated verb — because the rule is "the operator's next
    ///    act", not "an act about zooming".
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

        app.dispatch_command(&ctx, "view.zoom_in", &mut Vec::new());
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
