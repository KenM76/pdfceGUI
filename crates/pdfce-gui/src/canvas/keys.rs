//! # `canvas::keys` — the keys the canvas owns, and who gets Escape
//!
//! Escape, Delete and **Tab**, and nothing else. They are split out of
//! `canvas/mod.rs`
//! along a real seam rather than for line count: everything else in that file
//! is *wiring* — it needs an `egui::Ui`, a laid-out scroll area and a live
//! document, and cannot be exercised without a window — whereas this is a
//! decision about keys that a headless `egui::Context` can drive end to end.
//! Its tests came with it, which is the test for whether a split was along a
//! seam.
//!
//! ## Tab, and why it is guarded on the armed tool rather than on a mode
//!
//! Tab advances the **snap cycle** while a measure tool is armed
//! ([`crate::canvas::measure::cycle_snap`]) — the operator's way of saying
//! *"the other candidate"* when an endpoint and a midpoint are a few pixels
//! apart and pointing cannot separate them.
//!
//! Tab is `egui`'s focus key, so the guard matters more here than for the
//! other two. It is the **armed tool**, not a mode and not a capability:
//! with no measure tool armed the branch is false, the key is untouched, and
//! keyboard navigation of every panel and dialog behaves exactly as it did.
//! A mode-shaped guard would have been wrong in both directions — Review and
//! Edit both offer the measure tools, and in neither of them should Tab stop
//! moving focus when the operator is not measuring.
//!
//! ## ★ Six claimants for Escape, one press, one effect
//!
//! Decision 025's L1 is that Escape ascends **exactly one rung** rather than
//! collapsing the ladder, and the same discipline governs everything else that
//! would like the key. By Phase 6 there are six claimants, and the precedence
//! is *"retire the most transient thing first"*:
//!
//! | # | claimant | who decides | how it says it took the key |
//! |---|---|---|---|
//! | 0 | a **form field being typed into** | [`crate::canvas::forms`] | [`crate::canvas::forms::escape_spent`]: `true` when a draft was abandoned |
//! | 1 | a **drag in flight**, including a markup band | [`crate::canvas::gesture::GestureState::update`] — the only thing that knows whether there is one | [`crate::canvas::gesture::GestureOutcome::Cancelled`], arriving here as `escape_consumed` |
//! | 2 | a **guide drag in flight** | [`crate::canvas::guides::cancel_drag`] | its return value: `true` when there was one |
//! | 3a | a **measure pick** or a **markup vertex run** in progress | [`crate::canvas::measure::abandon`] / [`crate::canvas::markup::vertex::abandon`] | its return value: `true` when there was one |
//! | 3b | an **armed markup or measure tool** | [`crate::canvas::tool::disarm_markup`] / [`crate::canvas::tool::disarm_measure`] | its return value: `true` when there was one armed |
//! | — | ★ the **armed text tool** is deliberately **not** a claimant — see below | — | — |
//! | 4 | an **armed region zoom** | [`crate::canvas::zoom::disarm_region_zoom`] | its return value: `true` when there was something to retire |
//! | 5 | the **selection ladder**, or the **text selection** | [`crate::canvas::selection::SelectionState::escape`] / clearing [`crate::canvas::textsel::TextSelection`] | it is last, so it acts only when none above did |
//!
//! ### ★ Why the text selection shares rung 5 rather than taking a sixth
//!
//! The original argument was that it is not a precedence decision at all:
//!
//! > **The two occupants of rung 5 can never both be present.**
//! > `canvas::textsel::takes_the_press` gives a press its text meaning exactly
//! > when `Capabilities::edit_content` is absent, and the content selection is
//! > reachable exactly when it is present — one flag, two mutually exclusive
//! > branches […] there is no frame in which both could claim the key.
//!
//! ★ **That is no longer true, and the rung is right anyway.** Since
//! [`crate::canvas::tool::CanvasTool::Text`] landed (2026-08-14) an operator in
//! **Edit** can marquee some objects with the select tool, arm the text tool,
//! sweep a line, and hold both selections at once — `takes_the_press` gained a
//! disjunct, so the two are no longer decided by one flag in opposite senses.
//! `canvas::textsel` §3 records that move from exclusivity-by-construction to
//! exclusivity-by-precedence in full.
//!
//! So there **is** an ordering now, and it is the one the code already had:
//! **the text selection first.** Three reasons, in weight order:
//!
//! 1. **It is the more transient**, which is this table's own rule. A text range
//!    is made by the gesture the operator is performing right now and is
//!    destroyed by the very next edit anyway (`textsel` §7's epoch rule); an
//!    object selection survives edits, navigation, zoom and the mode change that
//!    hid it.
//! 2. **It is what the operator just did.** Reaching this state requires arming
//!    a tool and sweeping, and the press that follows means the thing that press
//!    could plausibly be about.
//! 3. **The content ladder has rungs and a text range has none**, which is the
//!    asymmetry the paragraph below already describes: clearing the text
//!    selection costs the operator one re-sweep, while ascending the object
//!    ladder loses the sub-path or node they had descended into. Given a
//!    choice, spend the press on the cheaper loss.
//!
//! A sixth rung is **still** not the answer, and now for a different reason than
//! before. It would put a *precedence* between two things that are the same act
//! — "clear what is selected" — expressed twice because this shell has two kinds
//! of selectable thing; and it would make a mode in which only one of them can
//! exist (Read, Review) look as though it had two rungs to climb. The rung is
//! "the selection"; which selection is answered by what is actually there.
//!
//! Both branches obey L1 identically: one press clears, and a **second** press
//! then does nothing, because there is nothing left. That is deliberately
//! unlike the content ladder, whose first press *ascends* and whose second
//! clears — a text range has no rungs to ascend, since there is no larger unit
//! than the sweep the operator made and no smaller one they have descended into.
//!
//! ### ★ Why the armed TEXT tool takes no rung at all
//!
//! Rung 3b retires an armed markup or measure tool, so the obvious symmetry is a
//! third call beside them. It is deliberately absent, and the decision is
//! recorded here because its absence looks exactly like an omission.
//!
//! **What rung 3b is actually for.** Both tools it retires paint a **crosshair**
//! that promises a gesture which *writes to the document*, and a mis-armed pen is
//! costly enough that the operator needs a universal way out of it. The text tool
//! paints an I-beam and promises a selection; it authors nothing (see
//! [`crate::canvas::tool::retire_forbidden`], which permits it in every mode for
//! the same reason). Nothing about it needs an emergency exit.
//!
//! **What the rung would collide with.** Escape at rung 5 already means *clear
//! the selection* while this tool is armed, because clearing a text selection is
//! what rung 5's first branch does. A rung *below* that — which is where the
//! transience rule would put it, the tool being less transient than the range —
//! would make the second press silently move the operator from **sweeping text**
//! to **marqueeing objects** in Edit: a change of what the primary button means,
//! delivered by the key they pressed to clear something. One press, one effect
//! is satisfied; *one press, one **expected** effect* is not.
//!
//! **What the reference applications do**, under `HANDOFF.md` §3's standing
//! instruction: Inkscape's Escape in the text tool deselects and **stays in the
//! tool**; Acrobat's Escape changes no tool; only SolidWorks exits the active
//! command. Two of three keep the tool, which is what ships — and it is also the
//! answer that needs no code.
//!
//! The route out is the control that armed it: `view.tool_text` is a toggle, so
//! pressing it again returns to the select tool
//! ([`crate::canvas::tool::toggle_text`]). That is the same *the button is
//! pressed, so pressing it un-presses it* rule the four markup buttons follow —
//! rung 3b is the **extra** affordance those tools get, not their only one.
//!
//! ### ★ Why a measure pick is TWO rungs rather than one
//!
//! A markup **band** is a drag, so abandoning it and retiring the pen are
//! already separated by the table: the drag is claimant 1, the tool is claimant
//! 3b. A measure pick is a sequence of **clicks**, so there is no drag for
//! claimant 1 to cancel — and yet a linear dimension with point A taken and
//! point B not is unmistakably a gesture in flight. Without 3a, one Escape
//! would put the tool down *and* silently discard that pick: two effects from
//! one press, which is exactly what decision 025's L1 forbids.
//!
//! ★ **The same argument admitted a second occupant to 3a on 2026-08-14**, and
//! the fact that it needed no new reasoning is the point. PolyLine and Polygon
//! are also gestured by clicks
//! ([`crate::canvas::markup::vertex`]), so a run of three vertices with the
//! fourth not yet placed is in exactly the position a half-taken linear pick is.
//! [`crate::canvas::markup::vertex::abandon`] therefore sits beside
//! [`crate::canvas::measure::abandon`] rather than taking a rung of its own:
//! the two cannot both be in progress, because a measure tool and a markup tool
//! cannot both be armed, so this is **one claimant expressed as two calls** —
//! which is precisely the arrangement rung 3b already uses for the two `disarm`
//! functions.
//!
//! It also means the sentence *"a markup is a drag"* is now only three-quarters
//! true, and the quarter that is not is why the rung was needed. The band kinds
//! and Ink are drags; the two vertex kinds are not.
//!
//! So the pick is retired first and the tool second, which is also the order
//! the transience rule gives — a half-taken pick is the more transient of the
//! two, and it is the thing the operator is most likely to have meant.
//! Pressing Escape twice puts the tool down; pressing it once corrects a
//! mis-aimed first click without leaving the tool.
//!
//! ### ★ Why a focused form field is rung 0, and why its rung is unlike every
//! other
//!
//! It is numbered 0 rather than 1 because it does not merely *outrank* the
//! others — while it is live, **none of them can see the key at all**. A
//! focused field is an `egui::TextEdit`, so `Context::text_edit_focused` is
//! true, and that predicate is the first line of this very function
//! (`DEFECTS.md` D1's guard) as well as the gate `interact` builds the gesture
//! machine's `cancel` flag from. So the exclusion is **mechanical**, not
//! ordered, and there is no version of this table in which a marquee and a
//! half-typed field compete for one press.
//!
//! What the rung is actually for is the *other* direction, and it is a real
//! hazard rather than a formality. `egui`'s own `TextEdit` surrenders focus on
//! Escape, and it does so **before** this function runs — so by the time the
//! guard above is asked, `text_edit_focused` is already false and the key
//! falls straight through to the selection ladder. One press would abandon the
//! draft *and* ascend a rung: exactly the double effect L1 forbids, and exactly
//! the shape the `escape_consumed` flag was invented for one row down.
//! [`crate::canvas::forms::escape_spent`] is the same report-rather-than-
//! re-derive contract, read once and cleared by the reading.
//!
//! ### ★ Where the markup tool sits, and why the transience rule does not
//! settle it
//!
//! Row 1 needed **no change at all**, and that is the first thing to notice: a
//! markup band is a [`DragKind`](crate::canvas::gesture::DragKind), so a markup
//! drag in flight is already the *drag in flight* claimant, cancelled by the
//! existing branch, with no new mechanism and no second rule. Abandoning it
//! authors nothing — the annotation is only written by
//! [`crate::app::actions::Action::CommitMarkup`], which the release raises and
//! a cancellation never does.
//!
//! Retiring the armed **tool** is a different act, and it needed a row. Its
//! placement is the one judgement call here, because the table's own rule does
//! not decide it: an armed markup tool is *less* transient than an armed region
//! zoom, not more. The zoom arming is a one-shot, spent by the very next drag;
//! the markup tool is a mode the operator stays in while they draw five
//! rectangles. On transience alone it would sit **below** the zoom.
//!
//! It sits above it, and the deciding argument is the one the guide-versus-zoom
//! row already makes in the paragraph below: *"there is no reading of that press
//! under which they meant a zoom they armed earlier and have not used."* That
//! is true here twice over, and the second reason is mechanical rather than a
//! matter of intent — **while the markup tool is armed, the region zoom cannot
//! be reached at all.** [`crate::canvas::gesture::press_kind`] gives an armed
//! markup tool the primary drag unconditionally, so the armed zoom is inert for
//! as long as the pen is down. Spending the operator's Escape on retiring
//! something inert, while they are looking at an armed Rectangle button that
//! does not un-press, is one press with no visible effect — and the effect it
//! *does* have is on a control that lives on a different ribbon tab, where they
//! cannot see it.
//!
//! So the ordering rule is unchanged and is simply not the one that applies to
//! this pair. What applies is the rule underneath it: **retire the thing the
//! operator could have meant.**
//!
//! ### Why a guide drag outranks an armed region zoom
//!
//! Both are "something in flight", so the tie is broken by the rule itself:
//! **retire the most transient thing first.** A guide drag ends the moment
//! the pointer is released, and it is following the pointer *now*; an armed
//! region zoom persists across frames waiting for a drag that has not started.
//! An operator dragging a guide who presses Escape means the guide, and there
//! is no reading of that press under which they meant a zoom they armed
//! earlier and have not used.
//!
//! A guide drag also does **not** reach [`crate::canvas::gesture`] — see that
//! module's header for why: a drag that did would move a guide *and* the
//! selection. So claimant 1 cannot speak for it, which is exactly why it needs
//! a row of its own rather than folding into `escape_consumed`.
//!
//! Each claimant reports whether it took the key rather than the caller
//! guessing, because the caller cannot know: whether a drag exists is the
//! gesture machine's private state and whether a zoom is armed is the zoom
//! module's. A version that re-derived either here would be the version that
//! cancels a drag **and** ascends a rung — which is the defect the whole
//! arrangement exists to prevent, and which an operator experiences as losing
//! the sub-path they were editing every time they abandon a mis-aimed drag.

use egui::Key;

use crate::app::actions::Action;
use crate::app::modes::Capabilities;
use crate::canvas::selection::{SelectionLevel, SelectionState};
use crate::canvas::zoom;

/// Escape and Delete, for the canvas selection.
///
/// # ★ `DEFECTS.md` D1, from the other end
///
/// D1 is *"I can't even click on an object and delete it by hitting the
/// delete key."* Its cause was `ctx.egui_wants_keyboard_input()` — which means
/// *any* widget has focus, not *a text field has focus* — combined with a
/// canvas that takes focus on click. `app::keyboard` already carries the fix
/// (`ctx.text_edit_focused()`) and the regression test for it. This is the
/// **verb** the fixed key now reaches: without it, D1 would be fixed and
/// Delete would still do nothing, because there was no selection to delete
/// and nothing to delete it with.
///
/// The same guard is applied here rather than inherited, because this reads
/// the key itself. It has to: `app::keyboard::collect` runs before any widget
/// is built, and although the selection now lives on `OpenDoc` and is
/// therefore *reachable* from there (module docs, seam 1), moving these two
/// bindings into the keymap is a change to `app::keyboard`'s key table and its
/// tests — a separate change with its own argument about chord precedence,
/// not a consequence of this one. What the move already bought is the ribbon's
/// Delete: `PdfceApp::dispatch_token` can now read the selection, so
/// `format.delete` raises the same action this does, from the same rule
/// ([`SelectionState::deletable_objects_on`]).
///
/// Backspace is bound alongside Delete because a laptop keyboard without a
/// dedicated Delete key is the common case, and every editor accepts both.
///
/// # `escape_consumed`, and why the gesture gets first refusal on the key
///
/// A drag in flight may be abandoned with Escape ([`gesture::GestureOutcome::Cancelled`]),
/// and that is the *same press* the ladder would otherwise read. One press must
/// have one effect — decision 025's L1, which is why Escape ascends exactly one
/// rung rather than collapsing the ladder — so when the gesture machine has
/// already spent the key, this is told and leaves it alone. The flag travels as
/// an argument rather than being re-derived here because the machine is the only
/// thing that knows whether there *was* a drag under the press; an Escape with an
/// idle pointer arrives here untouched and ascends, as it always did.
///
/// # `text_selection` — rung 5's other occupant
///
/// Passed as `&mut Option<_>` rather than being read back off the document,
/// because `canvas::interact` has taken it out by value for the duration of the
/// frame (the same move it makes for the object selection, and for the same
/// borrow reason). Clearing it needs nothing but the field: the *making* of a
/// text selection needs the page's extraction, which is why Ctrl+A and Ctrl+C
/// live in `canvas::textsel::keys` and only Escape lives here. See this module's
/// header on why it shares a rung with the ladder instead of taking a sixth.
pub(super) fn canvas_keys(
    ctx: &egui::Context,
    selection: &mut SelectionState,
    text_selection: &mut Option<crate::canvas::textsel::TextSelection>,
    page_index: usize,
    caps: Capabilities,
    actions: &mut Vec<Action>,
    escape_consumed: bool,
) {
    // ★ Claimant 0, and it is read BEFORE the D1 guard rather than after it.
    //
    // That order is the whole point: `egui`'s `TextEdit` has already
    // surrendered focus by the time this runs, so the guard below no longer
    // sees a focused field and would let the press reach the ladder. Reading
    // the flag here — and clearing it, which `escape_spent` does — is what
    // makes one press one effect. See the header's own section on this rung.
    //
    // With nothing focused it is `false` and costs one map lookup, exactly as
    // an un-armed `disarm_region_zoom` costs one.
    let form_abandoned = crate::canvas::forms::escape_spent(ctx);

    // ★ D1: `text_edit_focused()`, NEVER `egui_wants_keyboard_input()`.
    if ctx.text_edit_focused() {
        return;
    }
    let (escape, delete, tab) = ctx.input(|i| {
        (
            i.key_pressed(Key::Escape),
            i.key_pressed(Key::Delete) || i.key_pressed(Key::Backspace),
            i.key_pressed(Key::Tab),
        )
    });

    // ★ Tab advances the snap cycle, and ONLY while a measure tool is armed.
    //
    // Tab is egui's focus key, so taking it unconditionally would break
    // keyboard navigation of every panel and every dialog. The guard is the
    // armed tool rather than a mode or a capability: with no measure tool
    // armed, `cycle_snap` reports `false` and the key falls through untouched,
    // so this costs one map lookup on a canvas that is not measuring.
    //
    // It returns early rather than falling through to Escape and Delete
    // because a frame carrying Tab is not carrying either of those, and
    // continuing would only re-read two keys that cannot be pressed.
    if tab && crate::canvas::tool::active(ctx).measure_kind().is_some() {
        crate::canvas::measure::cycle_snap(ctx);
        return;
    }

    // ★ Escape retires the most transient thing first, and exactly one thing.
    //
    // The precedence is: a focused form field (spent above) → a drag in flight
    // (spent at step 3, and reported here as `escape_consumed`) → a guide drag
    // → an armed markup tool → an armed region zoom → the selection ladder.
    // Every rung obeys the same rule,
    // decision 025's L1: **one press, one effect.** An operator who
    // arms a marquee zoom and changes their mind presses Escape once and is
    // back in the select tool — with the rung they were working in intact,
    // because this returns before the ladder is touched.
    //
    // `disarm_region_zoom` reports whether there was anything armed, so an
    // Escape on an un-armed canvas falls straight through and still ascends,
    // exactly as it did before this branch existed.
    let escape_available = escape && !escape_consumed && !form_abandoned;
    if form_abandoned {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "canvas-escape outcome=AbandonedFormDraft".to_owned()
        });
    }

    // Claimant 2: a guide being dragged right now. Ahead of the region zoom
    // because it is the more transient of the two — it is following the
    // pointer this frame, while an armed zoom is waiting for a drag that has
    // not started. Abandoning it leaves nothing behind: the drag holds a
    // *proposed* position and the committed set only changes on release, so
    // there is no half-applied state to undo.
    let guide_cancelled = escape_available && crate::canvas::guides::cancel_drag(ctx);
    if guide_cancelled {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "canvas-escape outcome=CancelledGuideDrag".to_owned()
        });
    }

    // Claimant 3a: a measure pick in progress. Above the tool rung below it,
    // so one Escape corrects a mis-aimed pick and a second puts the tool down
    // — see the header's own section on why this needs two rungs where markup
    // needs one.
    let measure_abandoned =
        escape_available && !guide_cancelled && crate::canvas::measure::abandon(ctx);

    // …and a **markup vertex run** in progress, on the same rung and for the
    // identical reason: PolyLine and Polygon are gestured by clicks, so there is
    // no drag for claimant 1 to cancel, and yet a polygon with three vertices
    // taken and the fourth not is unmistakably a gesture in flight. Without this,
    // one Escape would discard the run **and** put the pen down — two effects
    // from one press, which is decision 025's L1 broken.
    //
    // One claimant expressed as two calls rather than two claimants, exactly as
    // 3b below is: a measure tool and a markup tool cannot both be armed, so a
    // measure pick and a vertex run cannot both be in progress.
    let vertex_abandoned = escape_available
        && !guide_cancelled
        && !measure_abandoned
        && crate::canvas::markup::vertex::abandon(ctx);

    // Claimant 3b: an armed markup tool. Above the region zoom deliberately —
    // see the header's own section on why the transience rule does not settle
    // that pair and what does. Note this is `disarm`, not "cancel a markup
    // drag": a drag in flight was already spent at claimant 1, and by the time
    // control reaches here `escape_available` is false in that case, so one
    // press can never both abandon the band AND put the pen down.
    let markup_disarmed = escape_available
        && !guide_cancelled
        && !measure_abandoned
        && !vertex_abandoned
        && crate::canvas::tool::disarm_markup(ctx);
    if markup_disarmed {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "canvas-escape outcome=DisarmedMarkupTool".to_owned()
        });
    }

    // …and the measure tool, on the same rung: they cannot both be armed, so
    // this is one claimant expressed as two calls rather than two claimants.
    let measure_disarmed = escape_available
        && !guide_cancelled
        && !measure_abandoned
        && !vertex_abandoned
        && !markup_disarmed
        && crate::canvas::tool::disarm_measure(ctx);
    if measure_disarmed {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "canvas-escape outcome=DisarmedMeasureTool".to_owned()
        });
    }

    // Claimant 4.
    let disarmed = escape_available
        && !guide_cancelled
        && !measure_abandoned
        && !vertex_abandoned
        && !markup_disarmed
        && !measure_disarmed
        && zoom::disarm_region_zoom(ctx);
    if disarmed {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "canvas-escape outcome=DisarmedRegionZoom".to_owned()
        });
    }

    // Claimant 5, and only if none of the four above took the key.
    if escape_available
        && !guide_cancelled
        && !measure_abandoned
        && !vertex_abandoned
        && !markup_disarmed
        && !measure_disarmed
        && !disarmed
    {
        // ★ Rung 5's two occupants, and they cannot both be here — see the
        // header. The text branch is tested first because it is the one that
        // can be non-empty in a mode where the other is *structurally* empty:
        // in Read the ladder has nothing to ascend, so calling `escape()` first
        // would consume the press on a no-op and leave the wash on the page.
        if text_selection.take().is_some() {
            crate::canvas::trace::text_selection(page_index, None, "escape");
        } else {
            let outcome = selection.escape();
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "canvas-escape outcome={outcome:?} sel={}",
                    selection.len()
                )
            });
        }
    }

    if !delete {
        return;
    }

    // ★ A mode that cannot edit content has no Delete.
    //
    // Escape is deliberately **above** this line and ungated: every one of its
    // claimants — a form draft, a guide drag, an armed region zoom — is
    // reachable in Read, and a mode that swallowed Escape would trap the
    // operator inside the gesture it had just let them start.
    //
    // In practice this is unreachable, because entering such a mode clears the
    // selection (`PdfceApp`'s mode-change arm) and no gesture can build a new
    // one, so `deletable_objects_on` would return an empty list two lines
    // below and refuse anyway. It is written explicitly regardless: *"a test
    // that checks a relation rather than a magnitude is satisfied by any
    // absurdity in the right direction"* (`HANDOFF.md` §2), and "Delete is
    // safe because nothing can be selected" is exactly that shape of argument
    // — it holds only for as long as the other half does, and the other half
    // is in a different file.
    if !caps.edit_content {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "canvas-delete-declined reason=mode-cannot-edit-content".to_owned()
        });
        return;
    }

    // ★ Delete acts at the OBJECT rung only, and the guard is not caution —
    // without it this is a destructive wrong action. The rule itself lives on
    // [`SelectionState::deletable_objects_on`], which carries the full
    // argument, because the ribbon's `format.delete` needs the identical
    // answer and a rule stated in two places is a rule that drifts.
    //
    // What is decided *here* is only how to report a refusal: a rung with no
    // delete verb is a distinct event from an empty selection, and a harness
    // reading `canvas-delete-declined` is entitled to know which happened.
    let objects = selection.deletable_objects_on(page_index);
    if objects.is_empty() {
        if selection.level() != SelectionLevel::Object {
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "canvas-delete-declined level={:?} reason=no-verb-for-rung",
                    selection.level()
                )
            });
        }
        return;
    }

    // The selection is NOT cleared here. The delete is an action, applied
    // after this frame; the epoch it bumps makes `SelectionState::resolve`
    // drop exactly the entries whose objects no longer exist, on the next
    // frame. Clearing here as well would be a second mechanism for the same
    // outcome, and the two would disagree the first time the engine refused
    // the edit.
    actions.push(Action::DeleteSelection {
        page: page_index,
        objects,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::selection::ClickHit;
    use crate::canvas::target::TargetId;
    use egui::{Context, Event, Modifiers, RawInput};

    /// A selection holding one whole object on page 0.
    fn object_selected() -> SelectionState {
        let mut selection = SelectionState::default();
        selection.click(
            0,
            ClickHit {
                object: Some(TargetId(3)),
                ..ClickHit::default()
            },
            false,
            false,
        );
        selection
    }

    /// …and the same one, descended a rung into part 1.
    fn part_entered() -> SelectionState {
        let mut selection = object_selected();
        selection.click(
            0,
            ClickHit {
                object: Some(TargetId(3)),
                part: Some(1),
                node: None,
            },
            false,
            true,
        );
        selection
    }

    /// `RawInput` carrying one unmodified key press.
    fn key(key: Key) -> RawInput {
        RawInput {
            events: vec![Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
            ..Default::default()
        }
    }

    /// Run [`canvas_keys`] for one frame against a real `egui::Context`.
    fn keys_for(input: RawInput, selection: &mut SelectionState) -> Vec<Action> {
        let ctx = Context::default();
        let mut actions = Vec::new();
        let mut text_selection = None;
        let _ = ctx.run_ui(input, |ui| {
            canvas_keys(
                ui.ctx(),
                selection,
                &mut text_selection,
                0,
                Capabilities::FULL,
                &mut actions,
                false,
            );
        });
        actions
    }

    /// ★ **Click, then Delete — the sequence `DEFECTS.md` D1 is about.**
    ///
    /// D1's own words: *"I can't even click on an object and delete it by
    /// hitting the delete key."* `app::keyboard` proves the key survives a
    /// canvas click; this proves the key now reaches a verb.
    #[test]
    fn delete_with_an_object_selected_raises_the_delete_action() {
        let mut selection = object_selected();
        assert_eq!(
            keys_for(key(Key::Delete), &mut selection),
            vec![Action::DeleteSelection {
                page: 0,
                objects: vec![3],
            }]
        );

        // Backspace is bound too — a laptop without a Delete key is the
        // common case.
        let mut selection = object_selected();
        assert_eq!(keys_for(key(Key::Backspace), &mut selection).len(), 1);
    }

    /// With nothing selected, Delete raises nothing rather than an empty
    /// batch the engine would have to refuse.
    #[test]
    fn delete_with_nothing_selected_raises_nothing() {
        let mut selection = SelectionState::default();
        assert!(keys_for(key(Key::Delete), &mut selection).is_empty());
    }

    /// ★ **Delete inside an object deletes NOTHING, rather than the object.**
    ///
    /// The destructive wrong action this stage must not ship: the selection
    /// names a subpath, the only wired verb removes whole objects, and one
    /// measured CAD export holds an entire drawing view as a single path
    /// object with 1,194 subpaths. "They can undo it" is not an answer to a
    /// keypress that removes a drawing.
    #[test]
    fn delete_inside_an_object_refuses_rather_than_deleting_the_object() {
        let mut selection = part_entered();
        assert_eq!(selection.level(), SelectionLevel::Part);
        assert!(
            keys_for(key(Key::Delete), &mut selection).is_empty(),
            "the Part rung has no delete verb wired, and must not borrow the Object rung's"
        );
        assert_eq!(selection.len(), 1, "and the selection is left alone");
    }

    /// ★ **An Escape already spent cancelling a drag does not also ascend a
    /// rung.** One press, one effect: an operator who abandons a move drag
    /// must still be standing where they were, or cancelling costs them the
    /// part they were working in as well as the drag.
    #[test]
    fn an_escape_spent_on_a_drag_leaves_the_rung_alone() {
        let mut selection = part_entered();
        let ctx = Context::default();
        let mut actions = Vec::new();
        let mut text_selection = None;
        let _ = ctx.run_ui(key(Key::Escape), |ui| {
            canvas_keys(
                ui.ctx(),
                &mut selection,
                &mut text_selection,
                0,
                Capabilities::FULL,
                &mut actions,
                true,
            );
        });
        assert_eq!(selection.level(), SelectionLevel::Part);
        assert!(actions.is_empty());
    }

    /// ★ **Escape retires an armed region zoom instead of ascending a rung —
    /// and only one of the two happens.**
    ///
    /// The rule this must not break is already in the file above: *"there is
    /// already an Escape rule that must not both cancel a drag and ascend a
    /// selection rung."* Phase 3.4 inserts a third claimant between them, so
    /// the same discipline is asserted for the new pair: an operator who arms
    /// a marquee zoom and changes their mind gets out of the tool **and keeps
    /// the part they were working in**.
    #[test]
    fn escape_retires_an_armed_region_zoom_before_it_touches_the_ladder() {
        let ctx = Context::default();
        let mut selection = part_entered();
        let mut actions = Vec::new();
        let mut text_selection = None;
        zoom::arm_region_zoom(&ctx);

        let _ = ctx.run_ui(key(Key::Escape), |ui| {
            canvas_keys(
                ui.ctx(),
                &mut selection,
                &mut text_selection,
                0,
                Capabilities::FULL,
                &mut actions,
                false,
            );
        });

        assert!(
            !zoom::region_zoom_armed(&ctx),
            "the zoom tool must be retired"
        );
        assert_eq!(
            selection.level(),
            SelectionLevel::Part,
            "and the rung must be left exactly where it was"
        );
        assert!(actions.is_empty());
    }

    /// …and the *next* Escape, with nothing armed, ascends exactly as it
    /// always did. Without this the test above would pass on a build where
    /// Escape had stopped reaching the ladder altogether.
    #[test]
    fn escape_reaches_the_ladder_again_once_nothing_is_armed() {
        let ctx = Context::default();
        let mut selection = part_entered();
        let mut actions = Vec::new();
        let mut text_selection = None;
        assert!(!zoom::region_zoom_armed(&ctx));

        let _ = ctx.run_ui(key(Key::Escape), |ui| {
            canvas_keys(
                ui.ctx(),
                &mut selection,
                &mut text_selection,
                0,
                Capabilities::FULL,
                &mut actions,
                false,
            );
        });

        assert_eq!(selection.level(), SelectionLevel::Object);
        assert_eq!(selection.len(), 1, "leaving a rung does not clear");
    }

    /// ★ **An Escape already spent cancelling a drag leaves the armed zoom
    /// alone too.**
    ///
    /// The one-press-one-effect rule runs in both directions: a cancelled
    /// zoom-marquee drag must not *also* disarm the tool, or an operator who
    /// mis-drags a zoom box has to re-arm it before they can try again.
    #[test]
    fn an_escape_spent_on_a_drag_leaves_the_armed_zoom_alone() {
        let ctx = Context::default();
        let mut selection = part_entered();
        let mut actions = Vec::new();
        let mut text_selection = None;
        zoom::arm_region_zoom(&ctx);

        let _ = ctx.run_ui(key(Key::Escape), |ui| {
            canvas_keys(
                ui.ctx(),
                &mut selection,
                &mut text_selection,
                0,
                Capabilities::FULL,
                &mut actions,
                true,
            );
        });

        assert!(
            zoom::region_zoom_armed(&ctx),
            "the drag consumed the key; the arming must survive for the retry"
        );
        assert_eq!(selection.level(), SelectionLevel::Part);
    }

    /// Escape ascends one rung and raises no action — the ladder is canvas
    /// state, not a document change.
    #[test]
    fn escape_ascends_a_rung_and_raises_no_action() {
        let mut selection = part_entered();
        assert!(keys_for(key(Key::Escape), &mut selection).is_empty());
        assert_eq!(selection.level(), SelectionLevel::Object);
        assert_eq!(selection.len(), 1, "leaving a rung does not clear");

        assert!(keys_for(key(Key::Escape), &mut selection).is_empty());
        assert!(selection.is_empty(), "the next press clears");
    }

    /// ★ **A focused text field keeps its Delete key** — the guard D1 is
    /// about, asserted in the direction that matters for correctness.
    ///
    /// `app::keyboard`'s regression test proves the *other* direction: a
    /// focused NON-text widget must not suppress the key. Both are needed.
    /// This one builds a real `TextEdit` and focuses it, because
    /// `text_edit_focused()` resolves the focused id and looks for a
    /// `TextEditState` under it — a hand-requested focus on a bare id would
    /// pass vacuously.
    #[test]
    fn a_focused_text_field_keeps_delete_for_itself() {
        let ctx = Context::default();
        let mut buffer = String::from("x");
        let mut selection = object_selected();
        let mut actions = Vec::new();
        let mut text_selection = None;

        // Frame 1: build the field and take focus.
        let _ = ctx.run_ui(RawInput::default(), |ui| {
            ui.add(egui::TextEdit::singleline(&mut buffer))
                .request_focus();
        });
        // Frame 2: the field holds focus; Delete belongs to it.
        let mut typing = false;
        let _ = ctx.run_ui(key(Key::Delete), |ui| {
            ui.add(egui::TextEdit::singleline(&mut buffer));
            typing = ui.ctx().text_edit_focused();
            canvas_keys(
                ui.ctx(),
                &mut selection,
                &mut text_selection,
                0,
                Capabilities::FULL,
                &mut actions,
                false,
            );
        });

        assert!(
            typing,
            "the test is vacuous unless a TEXT field really holds focus"
        );
        assert!(
            actions.is_empty(),
            "a focused text field must keep Delete for itself"
        );
        assert_eq!(selection.len(), 1);
    }

    /// ★ **Escape retires an armed markup tool before it touches the region
    /// zoom or the ladder — and retires exactly one thing.**
    ///
    /// Both are armed at once deliberately, for the reason the guide-versus-zoom
    /// test below states: asserting the markup tool is retired would pass on a
    /// build that retired everything. Asserting the zoom **survives** is what
    /// makes it a precedence test.
    #[test]
    fn escape_retires_the_markup_tool_before_the_region_zoom() {
        use crate::canvas::markup::MarkupKind;
        use crate::canvas::tool;

        let ctx = Context::default();
        let mut selection = part_entered();
        let mut actions = Vec::new();
        let mut text_selection = None;
        zoom::arm_region_zoom(&ctx);
        tool::arm_markup(&ctx, MarkupKind::Rectangle);

        let _ = ctx.run_ui(key(Key::Escape), |ui| {
            canvas_keys(
                ui.ctx(),
                &mut selection,
                &mut text_selection,
                0,
                Capabilities::FULL,
                &mut actions,
                false,
            );
        });

        assert_eq!(
            tool::selected(&ctx),
            tool::CanvasTool::Select,
            "the pen must be put down"
        );
        assert!(
            zoom::region_zoom_armed(&ctx),
            "the armed zoom must SURVIVE: one press, one effect"
        );
        assert_eq!(
            selection.level(),
            SelectionLevel::Part,
            "and the selection rung must be untouched"
        );
        assert!(actions.is_empty());
    }

    /// ★ **An Escape already spent abandoning a markup band does NOT also put
    /// the pen down.**
    ///
    /// The sharpest form of one-press-one-effect for this feature: an operator
    /// who mis-drags a rectangle and cancels it is still holding the rectangle
    /// tool, so their next drag draws a rectangle. Retiring the tool as well
    /// would make every abandoned drag cost a trip back to the ribbon.
    #[test]
    fn an_escape_spent_on_a_markup_drag_leaves_the_tool_armed() {
        use crate::canvas::markup::MarkupKind;
        use crate::canvas::tool;

        let ctx = Context::default();
        let mut selection = part_entered();
        let mut actions = Vec::new();
        let mut text_selection = None;
        tool::arm_markup(&ctx, MarkupKind::Ellipse);

        let _ = ctx.run_ui(key(Key::Escape), |ui| {
            // `true`: the gesture machine already spent the key cancelling the
            // band, exactly as `canvas::interact` reports it.
            canvas_keys(
                ui.ctx(),
                &mut selection,
                &mut text_selection,
                0,
                Capabilities::FULL,
                &mut actions,
                true,
            );
        });

        assert_eq!(
            tool::selected(&ctx),
            tool::CanvasTool::Markup(MarkupKind::Ellipse),
            "the drag consumed the key; the tool must survive for the retry"
        );
        assert_eq!(selection.level(), SelectionLevel::Part);
    }

    /// …and with no markup armed, Escape reaches the zoom and then the ladder
    /// exactly as it did before this claimant existed. Without this, the two
    /// tests above would pass on a build where the markup claimant swallowed
    /// every Escape.
    #[test]
    fn escape_still_reaches_the_zoom_and_the_ladder_with_no_markup_armed() {
        let ctx = Context::default();
        let mut selection = part_entered();
        let mut actions = Vec::new();
        let mut text_selection = None;
        zoom::arm_region_zoom(&ctx);

        for _ in 0..2 {
            let _ = ctx.run_ui(key(Key::Escape), |ui| {
                canvas_keys(
                    ui.ctx(),
                    &mut selection,
                    &mut text_selection,
                    0,
                    Capabilities::FULL,
                    &mut actions,
                    false,
                );
            });
        }

        assert!(
            !zoom::region_zoom_armed(&ctx),
            "the first press took the zoom"
        );
        assert_eq!(
            selection.level(),
            SelectionLevel::Object,
            "and the second reached the ladder"
        );
    }

    /// ★ **A guide drag outranks an armed region zoom, and only one of the
    /// two is retired.**
    ///
    /// The tie-break the precedence table states: retire the most transient
    /// thing first. Both are "in flight", and the guide is the one following
    /// the pointer *this frame* while the zoom is waiting for a drag that has
    /// not started.
    ///
    /// Both are armed at once deliberately. Asserting the guide is cancelled
    /// would pass on a build that cancelled everything; asserting the zoom
    /// SURVIVES is what makes it a precedence test rather than a "something
    /// happened" test.
    #[test]
    fn escape_abandons_a_guide_drag_before_it_touches_the_region_zoom() {
        let ctx = Context::default();
        let mut selection = part_entered();
        let mut actions = Vec::new();
        let mut text_selection = None;
        zoom::arm_region_zoom(&ctx);
        crate::canvas::guides::plant_drag_for_test(&ctx);

        let _ = ctx.run_ui(key(Key::Escape), |ui| {
            canvas_keys(
                ui.ctx(),
                &mut selection,
                &mut text_selection,
                0,
                Capabilities::FULL,
                &mut actions,
                false,
            );
        });

        assert!(
            zoom::region_zoom_armed(&ctx),
            "the armed zoom must SURVIVE: one press, one effect"
        );
        assert_eq!(
            selection.level(),
            SelectionLevel::Part,
            "and the selection rung must be untouched"
        );
        assert!(
            actions.is_empty(),
            "an abandoned drag holds a proposal, so it raises no action"
        );
    }

    /// ★ **A circular pick set is abandoned by the FIRST Escape and the tool
    /// by the second** — the two rungs, over the one tool that most needs
    /// them.
    ///
    /// The radius/diameter gesture has no natural end, so a pick set can sit
    /// there for as long as the operator keeps toggling arcs into it. That
    /// makes the two-rung rule load-bearing rather than tidy: an operator who
    /// has picked four arcs and catches a fifth by mistake presses Escape to
    /// correct it and must find themselves still holding the tool, with the
    /// set cleared — not back in the select tool with everything gone.
    ///
    /// A region zoom is armed throughout, and asserting it **survives** both
    /// presses is what makes this a precedence test rather than a "something
    /// happened" test: a build that retired everything on the first press would
    /// pass the first two assertions.
    #[test]
    fn escape_abandons_a_circle_fit_before_it_puts_the_measure_tool_down() {
        use crate::canvas::measure::{self, MeasureKind};
        use crate::canvas::tool;

        let ctx = Context::default();
        let mut selection = part_entered();
        let mut actions = Vec::new();
        let mut text_selection = None;
        zoom::arm_region_zoom(&ctx);
        tool::arm_measure(&ctx, MeasureKind::Circular);
        measure::circular::plant_pick_for_test(&ctx, 0);
        assert!(
            measure::finishable(&ctx),
            "the fixture must be a real, finishable pick set"
        );

        // Press 1: the pick set, and nothing else.
        let _ = ctx.run_ui(key(Key::Escape), |ui| {
            canvas_keys(
                ui.ctx(),
                &mut selection,
                &mut text_selection,
                0,
                Capabilities::FULL,
                &mut actions,
                false,
            );
        });
        assert_eq!(
            tool::selected(&ctx),
            tool::CanvasTool::Measure(MeasureKind::Circular),
            "the tool must survive: one press corrects a mis-picked arc"
        );
        assert!(
            !measure::finishable(&ctx),
            "and the pick set is the thing that went"
        );
        assert!(zoom::region_zoom_armed(&ctx), "one press, one effect");
        assert_eq!(selection.level(), SelectionLevel::Part);

        // Press 2: the tool.
        let _ = ctx.run_ui(key(Key::Escape), |ui| {
            canvas_keys(
                ui.ctx(),
                &mut selection,
                &mut text_selection,
                0,
                Capabilities::FULL,
                &mut actions,
                false,
            );
        });
        assert_eq!(
            tool::selected(&ctx),
            tool::CanvasTool::Select,
            "the second press puts the tool down"
        );
        assert!(zoom::region_zoom_armed(&ctx), "and still not the zoom");
        assert_eq!(
            selection.level(),
            SelectionLevel::Part,
            "two presses, two effects — and neither of them the ladder"
        );
        assert!(
            actions.is_empty(),
            "abandoning a pick authors nothing: the dimension only exists once \
             one of the two endings raises it"
        );
    }

    /// ★ **A markup vertex run is abandoned by the FIRST Escape and the pen by
    /// the second** — rung 3a's second occupant, asserted the same way its first
    /// is.
    ///
    /// Written as a near-copy of the circle-fit test above **on purpose**, and
    /// the copy is the point rather than duplication: the two gestures have the
    /// same problem (a run of clicks with no natural end), were given the same
    /// answer (two endings, one commit path), and now share a rung — so a build
    /// that got the precedence right for one and wrong for the other is exactly
    /// what a near-copy catches and a shared helper would hide.
    ///
    /// The operator's case is concrete: someone clicking out a polygon round a
    /// detail catches a seventh corner by mistake, presses Escape to correct it,
    /// and must find themselves **still holding the pen** with the run cleared —
    /// not back in the select tool with everything gone and the tool to re-arm.
    ///
    /// A region zoom is armed throughout, and asserting it **survives both
    /// presses** is what makes this a precedence test rather than a "something
    /// happened" test: a build that retired everything on the first press would
    /// pass the first two assertions.
    #[test]
    fn escape_abandons_a_vertex_run_before_it_puts_the_markup_tool_down() {
        use crate::canvas::markup::{MarkupKind, vertex};
        use crate::canvas::tool;

        let ctx = Context::default();
        let mut selection = part_entered();
        let mut actions = Vec::new();
        let mut text_selection = None;
        zoom::arm_region_zoom(&ctx);
        tool::arm_markup(&ctx, MarkupKind::Polygon);
        vertex::plant_run_for_test(&ctx, 0, MarkupKind::Polygon);
        assert!(
            vertex::finishable(&ctx),
            "the fixture must be a real, finishable run"
        );

        // Press 1: the run, and nothing else.
        let _ = ctx.run_ui(key(Key::Escape), |ui| {
            canvas_keys(
                ui.ctx(),
                &mut selection,
                &mut text_selection,
                0,
                Capabilities::FULL,
                &mut actions,
                false,
            );
        });
        assert_eq!(
            tool::selected(&ctx),
            tool::CanvasTool::Markup(MarkupKind::Polygon),
            "the pen must survive: one press corrects a mis-clicked corner"
        );
        assert!(
            !vertex::finishable(&ctx),
            "and the run is the thing that went"
        );
        assert!(zoom::region_zoom_armed(&ctx), "one press, one effect");
        assert_eq!(selection.level(), SelectionLevel::Part);

        // Press 2: the pen.
        let _ = ctx.run_ui(key(Key::Escape), |ui| {
            canvas_keys(
                ui.ctx(),
                &mut selection,
                &mut text_selection,
                0,
                Capabilities::FULL,
                &mut actions,
                false,
            );
        });
        assert_eq!(
            tool::selected(&ctx),
            tool::CanvasTool::Select,
            "the second press puts the pen down"
        );
        assert!(zoom::region_zoom_armed(&ctx), "and still not the zoom");
        assert_eq!(
            selection.level(),
            SelectionLevel::Part,
            "two presses, two effects — and neither of them the ladder"
        );
        assert!(
            actions.is_empty(),
            "abandoning a run authors nothing: the annotation only exists once \
             one of the two endings raises it"
        );
    }

    /// …and a second Escape then retires the zoom, so nothing is stranded.
    ///
    /// Without this, the test above would pass on a build where a guide drag
    /// permanently swallowed Escape — which is a worse bug than the one being
    /// fixed, because it would leave the operator unable to leave any tool.
    #[test]
    fn a_second_escape_retires_the_zoom_the_guide_drag_protected() {
        let ctx = Context::default();
        let mut selection = part_entered();
        let mut actions = Vec::new();
        let mut text_selection = None;
        zoom::arm_region_zoom(&ctx);
        crate::canvas::guides::plant_drag_for_test(&ctx);

        for _ in 0..2 {
            let _ = ctx.run_ui(key(Key::Escape), |ui| {
                canvas_keys(
                    ui.ctx(),
                    &mut selection,
                    &mut text_selection,
                    0,
                    Capabilities::FULL,
                    &mut actions,
                    false,
                );
            });
        }

        assert!(
            !zoom::region_zoom_armed(&ctx),
            "the second press must reach the zoom"
        );
        assert_eq!(
            selection.level(),
            SelectionLevel::Part,
            "two presses, two effects — and neither of them the ladder"
        );
    }
}
