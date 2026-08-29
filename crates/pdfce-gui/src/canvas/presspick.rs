//! # `canvas::presspick` — **the selection catches up with the pointer, at
//! press time**
//!
//! One step, run once per frame, immediately before [`crate::canvas::pressing`]
//! is asked what a press would mean.
//!
//! ## Why it is its own file, and not part of `pressing`
//!
//! `pressing`'s header opens with *"nothing here changes anything"*, and that
//! sentence is load-bearing — it is what lets that module be read as a pure
//! answer to *"what is under the pointer?"*. This step **mutates the
//! selection**. Putting it there would have made a stated contract false, which
//! is worse than having two small files.
//!
//! R2 forced the question on 2026-08-27 (`interact.rs` reached 1,570 lines) and
//! the seam was already drawn: look, then decide, then act. This is the "act"
//! that has to happen before the "decide".
//!
//! ## What it is for
//!
//! Read [`at_press`]. The short version: selection used to happen on the
//! **click**, which in egui means on release, so a press-and-drag on an
//! unselected object could not move it — the operator got a marquee across the
//! thing they were dragging. Every graphics editor selects on press, and the
//! operator said so in those words.

use crate::app::modes::Capabilities;
use crate::app::state::OpenDoc;
use crate::canvas::mapping::PageMapping;
use crate::canvas::pick::PickFilter;
use crate::canvas::selection::SelectionState;
use crate::canvas::tool::CanvasTool;

/// Select whatever an unselected press landed on.
///
/// Called from `canvas::interact` step 1b, before `pressing::look`, so that the
/// grip test on the very next statement sees the selection this made.
///
#[allow(clippy::too_many_arguments)]
pub(super) fn at_press(
    ctx: &egui::Context,
    doc: &OpenDoc,
    selection: &mut SelectionState,
    map: &PageMapping,
    page_index: usize,
    active_tool: CanvasTool,
    caps: Capabilities,
    pick: PickFilter,
    shift: bool,
) {
    // ★★★ **1b. A press on an unselected object selects it — before the
    // gesture machine is asked what the press means.**
    //
    // The operator, 2026-08-26: *"if I add an image I Expect to click on it to
    // resize but dragging doesn't resize […] Editing should work like 99% of
    // the graphics programs out there."*
    //
    // # What was actually wrong
    //
    // Selection happened on the **click**, which in egui means on *release*.
    // So a press-and-drag on an object that was not already selected never
    // selected anything: `pressing::look` saw an empty selection, found no grip
    // under the origin, and `press_kind` fell to
    // `(None, None) => Marquee(Select)` — the operator got a rubber band across
    // the thing they were trying to drag, and on release it selected. Two
    // gestures to do what every other editor does in one.
    //
    // # Why it is fixed HERE rather than in the gesture machine
    //
    // Because the gesture machine's answer was never wrong. *"No grip under the
    // origin, so marquee"* is correct — the fault was that the selection had not
    // caught up with the pointer yet. Selecting at press time makes
    // `pressing::look` (called on the very next statement, in this same frame)
    // find `Grip::Move` and produce `DragKind::Move` through the path that
    // already existed and is already tested.
    //
    // That is why this is nine statements rather than a new `DragKind`, a new
    // gesture phase, and an audit of every arm that reads one.
    // `INTERACTION_GAP.md` priced this item as the most invasive of the
    // unblocked set on the assumption it had to be done in the machine.
    //
    // # The four things it must not disturb, and how each is held off
    //
    // 1. **A press on empty paper still marquees.** No object under the origin
    //    means no selection is made and nothing below changes.
    // 2. **A press on the CURRENT selection still moves it without
    //    re-selecting.** `grip_box` already contains the origin in that case, so
    //    the guard declines and the existing move runs untouched. This matters
    //    for a *multiple* selection: re-selecting would silently drop it to one
    //    object mid-gesture.
    // 3. **An armed tool keeps its press.** Only the plain Select tool reaches
    //    here — the pen, the caret, the measure tools and the Node tool all own
    //    their press by this codebase's standing rule.
    // 4. **An armed region zoom outranks it**, on the same argument the text
    //    row makes: it is a one-shot the operator armed deliberately from the
    //    ribbon, and it is spent on the next press.
    //
    // ★ And `Shift` declines, because a Shift-press is the *extend* gesture and
    // the click path owns it. Selecting on press would replace the selection the
    // operator was adding to — the same mid-gesture loss as case 2, arrived at
    // from the other direction.
    if press_selects(ctx, active_tool, caps, shift)
        && let Some(origin) = ctx.input(|i| i.pointer.press_origin())
        && !covers(ctx, doc, map, selection, origin)
    {
        let point = map.to_page(origin);
        let hit = doc.page_objects().and_then(|provider| {
            crate::canvas::input::topmost(&*provider, page_index, point, map, pick)
        });
        if let Some(object) = hit {
            selection.select_only(page_index, object, "press");
        }
    }
}

/// Whether a press this frame may select what is under it.
///
/// Four conditions, each with its own reason in [`interact`]'s step 1b:
/// the primary button went down **this frame**, the plain Select tool is armed,
/// the mode may edit content, and no region zoom is waiting to be spent.
///
/// `Shift` declines here rather than at the call site so that the whole
/// predicate reads in one place — a reader asking *"when does a press select?"*
/// gets one answer rather than a function plus a condition beside it.
fn press_selects(ctx: &egui::Context, tool: CanvasTool, caps: Capabilities, shift: bool) -> bool {
    matches!(tool, CanvasTool::Select)
        && caps.edit_content
        && !shift
        && !crate::canvas::zoom::region_zoom_armed(ctx)
        && ctx.input(|i| i.pointer.button_pressed(egui::PointerButton::Primary))
}

/// ★★★ **Whether the current selection already claims this point** — its body,
/// its eight resize grips, or its rotate handle.
///
/// # The guard that keeps a press on an existing selection from re-selecting
///
/// It asks `handles::grip_at` against the box
/// [`crate::canvas::pressing::grabbable`] answers with — the **same** two
/// functions [`crate::canvas::pressing::look`] asks a moment later — rather
/// than testing the selection's entries. A second opinion computed differently
/// here would disagree with the gesture machine at the margins, and every
/// disagreement is a press that selects when it should have transformed.
///
/// # ★★ Why the grips, and not just the box — found by driving, within the hour
///
/// The first version tested `grip_box.contains(point)` alone, and
/// `rotate_handle_turns_a_selection` failed on the next driven run. **The
/// rotate handle sits OUTSIDE the box** — `handles::rotate_rect` puts it above
/// the top edge — so a press on it is not "covered" by the body, and with any
/// object underneath this function selected that object and the rotate became a
/// select-and-move.
///
/// A working gesture aimed at the wrong verb, which is the failure mode this
/// canvas has now produced **five** separate times. It is worth naming every
/// time because it never *looks* broken from a chair — something moves.
///
/// # ★★★ THE FIFTH INSTANCE WAS IN THIS FUNCTION, AGAIN — 2026-08-28
///
/// The paragraph above records the fourth: `covers` asked the wrong *question*
/// (the box, not the grips). This one is subtler and had the identical symptom:
/// it asked the right question of **the wrong box**.
///
/// `overlay::grip_box` derives its answer from the selection's cached
/// **content** outlines, which `select_annot` clears — an annotation is not
/// content and has nothing decomposed to cache. So the moment a markup or a ce
/// dimension gained a rotate handle (`Pass 155.0` / `Pass 159.0`), this
/// function answered `None` for every press on one, `covers` was **false**, and
/// the press fell into the select-on-press body below — which picks the topmost
/// *content* object at that point and **replaces the annotation selection with
/// it**, twenty points above the shape the operator was aiming at.
///
/// ⇒ Then `pressing::look`, on the very next statement, would find a content
/// selection and the release would rotate a page object. A perfect gesture, on
/// something the operator never selected.
///
/// The fix is `pressing::grabbable`, which is the one function that knows about
/// all four kinds of grabbable box — page content, a markup, a ce dimension and
/// a form field's widget. That is what this doc comment always *claimed* was
/// being asked; it stopped being true when the second kind arrived, and nothing
/// said so.
///
/// ★ **The lesson is in the phrasing, not the diff.** *"The same two functions
/// `pressing::look` asks"* was a claim about a call site somewhere else, held
/// together by nothing. A guard that must agree with another module has to
/// **call that module**, not resemble it.
///
/// # ★★★ THE SIXTH INSTANCE WAS NOT HERE, AND IT WIDENS THE RULE — 2026-08-29
///
/// Recorded here because this is where the rule above lives and where the next
/// person will look for it. On the first ever driven run of
/// `rotating_a_markup_turns_it` the rotate handle was painted, was pressed at
/// the rect the application itself declared, and **committed nothing with
/// nothing said anywhere** — the same symptom as the fifth, produced by a line
/// in a different file.
///
/// The cause was a guard in `canvas::rotating::drag` that neither called
/// `grip_box` nor resembled it: `selection.object_indices_on(page).is_empty()`,
/// standing *in front of* the annotation branch. It counts page **content**,
/// which `select_annot` clears, so it returned before the routing decision was
/// ever reached, on every markup and every ce dimension.
///
/// ⇒ So the rule above is necessary and was not sufficient. Its companion:
/// **a guard written in one destination's vocabulary must stand AFTER the
/// branch that picks the destination, never before it.** Three destinations
/// share the rotate gesture and four share a press; a content-shaped test in
/// front of the fork answers about a subject the gesture may already have
/// routed away from. `canvas::rotating`'s header carries the full account.
///
/// The eight resize grips are inside the box and were never at risk. They are
/// covered by the same call anyway, rather than by an argument that they are
/// safe: an argument is a thing that stops being true.
///
/// # ★ `GripSet::all()` here, where `pressing::look` narrows it
///
/// `look` passes `grabbable`'s own `offer`, which is narrower for three of the
/// four kinds. This passes `all()` unconditionally, and the difference errs
/// toward **declining**: this answers `Some(grip)` for points `look` will call
/// `None`, so the press falls through to the gesture machine unchanged instead
/// of re-selecting under a node the operator is in the middle of editing.
/// Erring the other way would be a press that silently leaves node editing.
///
/// ★★ It matters in the new direction too. A selected **ce dimension** is
/// offered `GripSet::rotate_only()`, so `look` will not call a corner press a
/// resize — but `all()` here still claims that corner for the *existing
/// selection*, which is right: whatever the press turns out to mean, it is
/// about the dimension the operator already has, not about the linework
/// underneath it.
fn covers(
    ctx: &egui::Context,
    doc: &OpenDoc,
    map: &PageMapping,
    selection: &SelectionState,
    point: egui::Pos2,
) -> bool {
    crate::canvas::pressing::grabbable(ctx, doc, map, selection)
        .bounds
        .is_some_and(|b| {
            crate::canvas::handles::grip_at(b, point, crate::canvas::handles::GripSet::all())
                .is_some()
        })
}
