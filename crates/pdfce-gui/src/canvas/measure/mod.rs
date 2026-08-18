//! # `canvas::measure` — the dimensioning tools
//!
//! Phase 7. Placed under `canvas/` rather than at `tools/measure/` (which is
//! where `SALVAGE.md` §"Class C" pencilled it in) to follow the precedent this
//! shell actually set: [`crate::canvas::markup`] is the other on-canvas
//! authoring tool, it lives here, and a measure tool is the same kind of thing
//! — a gesture that reads the page and raises an `Action`.
//!
//! ## The parts
//!
//! | module | what it holds | egui? |
//! |---|---|---|
//! | [`pick`] | the pick state machines — linear, circular, two-line | **no** |
//! | [`circular`] | the radius/diameter tool: its pick, its **two endings**, its outlines | yes |
//! | [`scale`] | the scale-entry and dimension-group model — salvaged, **not yet reachable**; see the note on `MeasureKind` | **no** |
//! | [`state`] | the container built on tool entry, and what it discards | **no** |
//! | this file | the canvas hosting: picks, preview, overlay, disclosure | yes |
//!
//! [`circular`] is the newest row and the only one that hosts a single tool.
//! **R2** forced it out of this file when the radius/diameter tool was armed
//! (1,617 lines), but the line count only says that *something* had to move;
//! that module's own header carries which subject was separable and why —
//! briefly, it is the only measure gesture that does not end itself, so the
//! machinery for *saying when it is over* has nothing corresponding to it in
//! the other two tools.
//!
//! [`state`] is a third row rather than the two the salvage planned, and the
//! reason is **R2**: the old `measure_tool.rs` is 2,044 lines, and the planned
//! two-way split still leaves the pick half about twenty lines over the
//! 1,500-line limit. It was cut once more at a seam the original had already
//! drawn for itself — a `// ---` banner separating the three tools' individual
//! pick machines from the single container that owns all of them — rather than
//! by shaving doc comments to fit a threshold, which is the incentive
//! `tools/gates/check-file-size.sh` says in its own header it exists to refuse.
//! That module's own docs carry the full argument.
//!
//! ★ **`pick`, `scale` and `state` never see an `egui` type**, and that is carried
//! across deliberately from the old shell's `measure_tool.rs`, whose header
//! makes the argument: every transition is unit-testable without a live frame.
//! It is also what let the whole file be salvaged rather than rewritten.
//!
//! ## ★ This module owns no geometry
//!
//! Every load-bearing computation is a call into the already-shipped
//! `pdfce-core::dimension` / `pdfce-core::vector` — the Taubin best-fit circle,
//! the axis-constrained projection, the measured length, the scale back-calc,
//! and `author_from_two_lines`. The rule the old shell stated and this one
//! keeps: **reuse, never reimplement**, so a dimension authored on the canvas
//! is byte-for-byte the one `pdfce-cli dimension-add` writes.

pub mod circular;
pub mod pick;
pub mod scale;
pub mod state;

/// **Which dimensioning tool is armed.**
///
/// Carried on one [`crate::canvas::tool::CanvasTool::Measure`] variant rather
/// than becoming four `CanvasTool` entries, for the argument
/// [`crate::canvas::markup::MarkupKind`] settled and which applies here
/// unchanged: the operator is placing exactly one kind of dimension, so a type
/// that can say `Linear` and `Circular` at once — which four variants plus a
/// "which is active" rule spread across call sites can — is a type whose
/// illegal states must be prevented by discipline. Carrying the kind makes them
/// unrepresentable, and it makes every rule this enum owns
/// ([`crate::canvas::tool::CanvasTool::cursor`], the press decision in
/// [`crate::canvas::gesture::press_kind`]) written once for measure as a whole.
///
/// ★ **The old shell had four separate `CanvasTool` variants for these**
/// (`MeasureLinear`, `MeasureCircular`, `MeasureScale`) plus an `is_measure()`
/// predicate and three `tool_builds_measure_*` functions to ask which. That is
/// the shape this enum exists to avoid, and it is the one place this salvage
/// deliberately departs from the source: the old arrangement needed five
/// helpers to answer questions the kind answers by being a value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MeasureKind {
    /// Point A → point B → a third click for the standoff. The ordinary
    /// dimension, and the one an operator reaches for first.
    Linear,
    /// **Toggle objects into a best-fit circle**, then say when the set is
    /// complete. Authors a [`pdfce_core::dimension::DimensionKind::Circular`]
    /// — a radius or a diameter, which is one display toggle on one fit rather
    /// than two tools (decision 011's value model: `diameter = 2 × radius`, the
    /// same stored geometry).
    ///
    /// # ★ This variant was deleted once, and what brought it back
    ///
    /// Phase 7 shipped Linear and Two-line and deliberately left this one
    /// unarmed. The reason was recorded in three documents and it was a real
    /// one rather than an unfinished corner: **the gesture has no natural
    /// end.** Linear knows it is finished at three clicks and two-line at two,
    /// because both are picks of a fixed arity. A best-fit circle is finished
    /// when the operator says it is — an arc drawn as four separate polyline
    /// objects needs four picks, the same arc drawn as one needs one, and
    /// nothing in the geometry can tell pdfce which the operator meant. The
    /// only surface available for saying *"done"* was an accept box floating
    /// over the canvas, and decision 024 retired exactly that at the operator's
    /// instruction.
    ///
    /// **The operator settled it on 2026-08-14: give it two endings.** A
    /// **double-click** on the canvas, which is what every polyline-ish tool in
    /// every drawing package uses to say "that was the last one" and is
    /// therefore what *"make it work the way other programs do"* asks for; and
    /// a ribbon command, `measure.finish`, for an operator who does not know
    /// the double-click or whose last pick was awkward to double-click on.
    /// Neither is a floating box, and both reach `circular::commit` — one
    /// commit path, so the two endings cannot author different dimensions.
    ///
    /// # ★ Its pick set is the tool's own and is never the selection
    ///
    /// [`pick::CircularPick`]'s docs are explicit (ui-spec §3.1) and this
    /// hosting keeps the line: the objects toggled into a fit live on the pick,
    /// not in [`crate::canvas::selection::SelectionState`]. A half-assembled
    /// circle fit is not a selection — no verb on the Format tab means anything
    /// applied to it, Delete least of all — and borrowing the selection to hold
    /// it would arm a destructive control over a set the operator assembled for
    /// a completely different purpose.
    Circular,
    /// Pick two lines on the page; the engine authors the dimension between
    /// them. The gesture pdfce's own ledger marks as shipped, whose caller was
    /// missing on this side — see `SALVAGE.md`'s correction of 2026-08-14.
    TwoLine,
    /// **Pick two points on the drawing to say what its scale is.**
    ///
    /// The calibration gesture, and the one the operator asked for by name on
    /// 2026-08-17: *"set the scale by selecting two lines or points and
    /// defining what that distance represents."*
    ///
    /// # ★ It authors no dimension, which is why it is a kind and not a verb
    ///
    /// Every other variant ends in `Action::CommitDimension`. This one ends in
    /// a **dialog**: two picks measure a reference length in PDF points, and
    /// the operator then says what that length *is* on the real thing. The
    /// scale falls out of the two, and `EditSession::set_group_scale` records
    /// it against the group.
    ///
    /// It is nevertheless a `MeasureKind` rather than a separate tool, because
    /// everything about the *gesture* is a measure pick: it snaps to content,
    /// it honours the H/V/aligned constraint, it Tab-cycles candidates, and it
    /// clears on page navigation. [`crate::canvas::measure::scale::ScalePick`]
    /// reuses `LinearPick` **verbatim** for exactly that reason — the reference
    /// line is a linear pick that happens not to be authored.
    ///
    /// # ★ Deliberately absent from [`Self::ALL`]
    ///
    /// `ALL` is the list of kinds the **Measure tab arms with a command**, and
    /// this one is armed from inside the Set-scale dialog instead. See `ALL`'s
    /// own docs for why that distinction is worth the exception, and
    /// `tests::every_variant_is_either_offered_or_deliberately_excluded` for
    /// what stops the exception becoming a hole.
    Scale,
}

impl MeasureKind {
    /// Every variant the **Measure tab offers as a ribbon control**, in the
    /// order it offers them.
    ///
    /// A kind listed here and not given a command fails a test rather than
    /// shipping as a tool nothing can arm — the same contract
    /// [`crate::canvas::markup::MarkupKind::ALL`] carries.
    ///
    /// # ★ It is no longer exhaustive over the enum, and that is deliberate
    ///
    /// [`Self::Scale`] is absent. It is armed from a button inside the
    /// Set-scale dialog rather than from the ribbon, because the dialog is
    /// where an operator is already standing when they discover they need it —
    /// they opened it to set a scale, and the button says *there is a better
    /// way to do this than typing a ratio*. A second ribbon control would put
    /// the two halves of one decision on two different surfaces.
    ///
    /// The cost of the exception is that `ALL` stops being a complete
    /// inventory, and an inventory with a silent exception is how a future
    /// kind ships armed by nothing.
    /// `tests::every_variant_is_either_offered_or_deliberately_excluded` pays
    /// that cost: it matches **exhaustively** over the enum, so a new variant
    /// does not compile until it is either put in `ALL` or added to the
    /// excluded list with a reason.
    pub const ALL: &'static [Self] = &[Self::Linear, Self::Circular, Self::TwoLine];

    /// The kinds that are deliberately **not** on the ribbon, with the surface
    /// that arms each one instead.
    ///
    /// Read only by the exhaustiveness test. Its value is the second column:
    /// "excluded" with no destination is indistinguishable from "forgotten".
    #[cfg(test)]
    const ARMED_ELSEWHERE: &'static [(Self, &'static str)] = &[(
        Self::Scale,
        // ui-text-exempt: a test-only note naming the surface that arms this
        // kind. Never displayed — it exists so an excluded variant carries its
        // destination, and the test asserts it is non-empty.
        "the Set-scale dialog's calibrate button",
    )];
}

// ===========================================================================
// The canvas hosting
// ===========================================================================

use egui::{Pos2, Rect, Ui};
use pdfce_core::dimension::TwoLinePlacement;
use pdfce_core::vector::Point;
use pdfce_core::vector::linepick::{ParallelPolicy, pick_line_in_page};
use pdfce_core::vector::snap::{SnapCandidate, SnapConfig, snap_candidates};

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::canvas::mapping::PageMapping;
use crate::canvas::snap;
use crate::canvas::target::CanvasTargetProvider;
use crate::viewer;

use state::{ClickOutcome, MeasureState};

/// Where the in-progress pick lives between frames.
///
/// `egui::Memory`, beside the armed tool and the gesture machine, and for the
/// same reason [`crate::canvas::tool`]'s header gives: this is **transient UI
/// state**, not document state. It must not sit on `OpenDoc`, because a
/// half-finished pick is not part of the document and a document saved
/// mid-gesture must not carry one.
// ui-text-exempt: an `egui::Id` source string, never displayed.
const MEASURE_MEMORY_KEY: &str = "pdfce-measure-state";

/// Read the measure state, building one that already agrees with `kind` if
/// there is none.
fn load(ctx: &egui::Context, page_index: usize, kind: MeasureKind) -> MeasureState {
    let id = egui::Id::new(MEASURE_MEMORY_KEY);
    let mut st = ctx
        .data_mut(|d| d.get_temp::<MeasureState>(id))
        .unwrap_or_else(|| MeasureState::for_kind(page_index, kind));
    // ★ Two synchronisations, and the order matters.
    //
    // The kind first, because `set_kind` is what knows which picks a kind
    // change invalidates. Then the page: a gesture begun on one sheet means
    // nothing on the next, and navigating away is not a reason to keep half a
    // dimension alive (ui-spec §1.3, carried across from the old shell).
    st.set_kind(kind);
    if st.page_index != page_index {
        st.page_index = page_index;
        st.clear_gesture();
    }
    st
}

/// Write the measure state back.
fn store(ctx: &egui::Context, st: MeasureState) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(MEASURE_MEMORY_KEY), st));
}

/// **Advance the snap cycle**, reporting whether there was anything to advance.
///
/// Tab's claimant while a measure tool is armed. Two candidates a few pixels
/// apart — an endpoint and the midpoint of the segment it ends — are
/// indistinguishable by pointing, so the operator needs a way to say *"the
/// other one"*. That is what the cycle index is, and
/// [`snap::next_snap_index`] is the salvaged rule for advancing it.
///
/// # Why it does not need the candidate list
///
/// It advances the index and lets [`snap::active_snap_candidate`] wrap it
/// against whatever list the next frame produces, rather than reading the list
/// here to bound it. That is deliberate: the list is rebuilt every frame from
/// the live pointer, so a bound taken now would be a bound on a list that no
/// longer exists by the time it is used. `active_snap_candidate` already takes
/// the index modulo the list it is given, which is the only place the two are
/// guaranteed to be the same list.
///
/// Returns `false` when no measure state exists, so Tab falls through to
/// whatever else wants it rather than being silently eaten by a tool that is
/// not armed.
pub fn cycle_snap(ctx: &egui::Context) -> bool {
    let id = egui::Id::new(MEASURE_MEMORY_KEY);
    let Some(mut st) = ctx.data_mut(|d| d.get_temp::<MeasureState>(id)) else {
        return false;
    };
    // `usize::MAX` as the length: the advance is unbounded here on purpose —
    // see above — and `next_snap_index` is what decides the step.
    st.snap_cycle = snap::next_snap_index(st.snap_cycle, usize::MAX);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("measure-snap-cycle index={}", st.snap_cycle)
    });
    store(ctx, st);
    true
}

/// **Read the measure state without creating one.**
///
/// The half of [`load`] that has no side effects, for the two callers that must
/// not manufacture a state merely by asking: the ribbon's enabled-when
/// predicate ([`finishable`]) is evaluated on **every frame**, and the Finish
/// command's arm may be reached by a chord with no tool armed at all. `load`
/// would build a fresh `MeasureState` for either, which would then be written
/// back by the next `store` and leave the canvas holding a state for a tool
/// nobody armed.
/// The group the next dimension will join, for a surface that needs to name
/// it — today, the Set-scale dialog.
///
/// # Why `Option`, and why the caller does not get a default
///
/// `None` means the measure tool has never been entered this session, so there
/// is no state in `egui::Memory` and no group has been chosen. The caller could
/// substitute `DEFAULT_GROUP_ID` — and must not, silently: calibrating the
/// default group when the operator has been working in a named one is a wrong
/// answer that looks exactly like a right one, because both are "a group got a
/// scale".
///
/// `crate::app::dispatch`'s arm therefore falls back **deliberately and with a
/// trace line**, so a run where the fallback fired is distinguishable from one
/// where it did not.
#[must_use]
pub fn active_group(ctx: &egui::Context) -> Option<pdfce_core::dimension::GroupId> {
    read(ctx).map(|st| st.group)
}

fn read(ctx: &egui::Context) -> Option<MeasureState> {
    ctx.data_mut(|d| d.get_temp::<MeasureState>(egui::Id::new(MEASURE_MEMORY_KEY)))
}
/// The radius/diameter tool's two public entrances, re-exported so that every
/// caller outside `canvas/` names one module.
///
/// `app::dispatch` calls [`finish`] for the `measure.finish` command and
/// `app::conditions` calls [`finishable`] to publish `measure.finishable`. Both
/// live in [`circular`] because both are about the gesture's **ending**, which
/// is that tool's subject alone; they are named here because a dispatcher
/// reaching two modules deep for one verb is a dispatcher that knows more about
/// the canvas than it should.
pub use circular::{finish, finishable};

/// **Abandon a pick in progress**, reporting whether there was one.
///
/// Escape's claimant. It sits *below* the drag-in-flight rung and *above*
/// retiring the tool — see [`crate::canvas::tool::disarm_measure`], which
/// carries the argument for why those are two separate presses.
/// **Take the completed calibration line's measured length, once.**
///
/// Returns `Some(points)` on the single frame after the two-point pick
/// completes, and `None` on every other frame — the length is cleared from the
/// state as it is read.
///
/// # ★ Read-and-clear, because the alternative re-opens the dialog forever
///
/// `ScalePick::drawn_pdf_length` stays `Some` for as long as the pick holds a
/// completed line; that is what keeps the reference line drawn on the page
/// while the operator types. A caller that merely *observed* it would therefore
/// see it `Some` on every subsequent frame and re-open the Set-scale dialog
/// sixty times a second, discarding whatever had been typed into it each time.
///
/// Clearing here rather than asking the caller to remember is the same choice
/// `ScaleDialog::take_calibrate_request` makes: an edge that has to be reset by
/// discipline is one that eventually is not.
///
/// The whole pick is cleared, not just the length, so the tool is left ready
/// for another calibration rather than holding a line the dialog has already
/// consumed.
pub fn take_completed_scale_line(ctx: &egui::Context) -> Option<f64> {
    // Read the stored state DIRECTLY rather than through `load`, deliberately.
    // `load` synchronises the kind and the page and will happily build a fresh
    // state when there is none — which is exactly wrong for a question that
    // must answer "no" when nothing has happened. Building state here would
    // also make merely *asking* create a gesture, which is the hazard
    // `MeasureState::set_kind`'s docs name.
    let id = egui::Id::new(MEASURE_MEMORY_KEY);
    let mut st = ctx.data_mut(|d| d.get_temp::<MeasureState>(id))?;
    let measured = st.scale.drawn_pdf_length?;
    st.scale.clear();
    store(ctx, st);
    Some(measured)
}

pub fn abandon(ctx: &egui::Context) -> bool {
    let id = egui::Id::new(MEASURE_MEMORY_KEY);
    let Some(mut st) = ctx.data_mut(|d| d.get_temp::<MeasureState>(id)) else {
        return false;
    };
    if !st.gesture_in_progress() {
        return false;
    }
    st.clear_gesture();
    store(ctx, st);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        "canvas-escape outcome=AbandonedMeasurePick".to_owned()
    });
    true
}

/// **Resolve a raw pointer position to the point the pick will actually
/// commit**, and say which candidate it came from.
///
/// This is the call the salvaged [`crate::canvas::snap`] primitives were
/// waiting for. Until it existed every pick was the raw pointer position,
/// which on a CAD sheet is the difference between a dimension that measures a
/// line and one that measures *near* a line — and the second is worse than no
/// dimension, because it is wrong by an amount nobody can see.
///
/// # The four gates, in order, and what each is for
///
/// 1. **The master toggle and the Alt override**, through
///    [`snap::snap_query_enabled`]. Alt is the operator saying *"not this
///    time"* — it is what makes the offer refusable, and it is why the catch
///    radius can afford to be generous ([`PageMapping::snap_tolerance`]).
/// 2. **A decomposition must exist.** No model, no candidates, raw point. This
///    is a real case rather than a defensive one: the model is built only when
///    something needs it, and a measure click is one of the things that asks.
/// 3. **The query**, `pdfce_core::vector::snap_candidates` — the engine's, not
///    ours. `SnapConfig::new` leaves intersections off and axes on, which is
///    the shipped default; the grid is `None` because the canvas grid is a
///    *view* aid drawn in page space and snapping to it would be snapping to
///    something the document does not contain.
/// 4. **The Tab cycle**, through [`snap::active_snap_candidate`], which is what
///    lets the operator choose between an endpoint and a midpoint that are
///    within a few pixels of each other.
///
/// Returns the raw point unchanged when any gate declines, with `None` for the
/// candidate — so a caller never has to distinguish "snapping is off" from
/// "nothing was near", because neither changes what it does next.
fn snapped(
    st: &MeasureState,
    raw: Point,
    alt_held: bool,
    targets: Option<&dyn CanvasTargetProvider>,
    page_index: usize,
    map: &PageMapping,
) -> (Point, Option<SnapCandidate>) {
    if !snap::snap_query_enabled(st.snap_master, alt_held) {
        return (raw, None);
    }
    let Some(model) = targets.and_then(|t| t.page_objects_model(page_index)) else {
        return (raw, None);
    };
    let config = SnapConfig::new(map.snap_tolerance());
    let candidates = snap_candidates(raw, &config, model);
    match snap::active_snap_candidate(&candidates, st.snap_cycle) {
        Some(c) => (c.point, Some(c)),
        None => (raw, None),
    }
}

/// **Where the pointer would pick, resolved once for the frame.**
///
/// ★ The indicator and the click read *this same value*, which is the whole
/// reason it exists as a type rather than as two calls to [`snapped`]. A
/// preview that re-ran the query would be a second derivation of the same
/// answer, and the two would agree right up until they did not — the operator
/// aiming at a marker drawn over an endpoint and committing a point somewhere
/// else. `pdfce_FeatureRequests/README.md` rule 4 is explicit that a
/// pre-commit affordance must describe *what is about to happen*; one
/// derivation is how that stays true rather than being maintained.
#[derive(Debug, Clone, Copy)]
pub(super) struct Resolved {
    /// The point a click would commit — snapped, or the raw pointer.
    pub at: Point,
    /// Which candidate produced it, if any. `None` means the raw pointer, and
    /// nothing is drawn.
    pub candidate: Option<SnapCandidate>,
}

/// Resolve the pointer for this frame, while the decomposition is still
/// borrowed.
///
/// Called from `canvas::interact` **before** it drops the provider, which is
/// the constraint that shaped this API: the draw happens after the drop, so the
/// query cannot happen there.
pub(super) fn resolve_hover(
    ctx: &egui::Context,
    doc: &OpenDoc,
    page_index: usize,
    pointer: Option<Pos2>,
    targets: Option<&dyn CanvasTargetProvider>,
    map: &PageMapping,
) -> Option<Resolved> {
    let (pointer, page) = (pointer?, doc.current_page()?);
    let pdf = viewer::canvas_to_pdf_space(pointer, page)?;
    let raw = Point {
        x: f64::from(pdf.x),
        y: f64::from(pdf.y),
    };
    let st = ctx.data_mut(|d| d.get_temp::<MeasureState>(egui::Id::new(MEASURE_MEMORY_KEY)))?;
    if st.page_index != page_index {
        return None;
    }
    let alt_held = ctx.input(|i| i.modifiers.alt);
    let (at, candidate) = snapped(&st, raw, alt_held, targets, page_index, map);
    Some(Resolved { at, candidate })
}

/// **Everything one measure click is resolved against.**
///
/// A struct rather than seven parameters, and not merely to satisfy a lint: the
/// five that describe *where* the click landed — the document, the page, the
/// pointer, the decomposition and the mapping — are only meaningful together.
/// Any call site that had six of them and reached for the seventh from
/// somewhere else would be resolving a click against a page it did not come
/// from, which is the class of defect `canvas::mapping`'s header exists to make
/// unavailable.
pub(super) struct Pick<'a> {
    /// Where the in-progress pick is stored between frames.
    pub ctx: &'a egui::Context,
    /// The open document, for the page transform.
    pub doc: &'a OpenDoc,
    /// The page the click landed on.
    pub page_index: usize,
    /// Which measure tool is armed.
    pub kind: MeasureKind,
    /// The click, in canvas space.
    pub canvas_point: Pos2,
    /// **This was the second click of a double-click.**
    ///
    /// Carried rather than re-read from `egui` for the same reason every other
    /// field is: the whole of one click's meaning arrives in one value, so a
    /// future arm cannot resolve half of a click against this frame and half
    /// against the input state as it is by the time the arm runs.
    ///
    /// Only the circular tool reads it, and what it means there is *"the set is
    /// complete"* — see [`click`]'s own section on the two endings. The linear
    /// and two-line picks ignore it: their gestures end at a known click count,
    /// so a double-click on a linear tool is simply two picks, which is what an
    /// operator hurrying through point A and point B actually means.
    pub double: bool,
    /// The page's decomposed geometry, for the two-line pick and the circular
    /// object pick. `None` when no decomposition was built this frame.
    pub targets: Option<&'a dyn CanvasTargetProvider>,
    /// The screen↔page mapping, for the pick tolerance.
    pub map: &'a PageMapping,
}

/// **Take one click for the armed measure tool.**
///
/// The whole of the tool's input. Called from `canvas::interact`'s `Click` arm
/// when [`crate::canvas::tool::CanvasTool::measure_kind`] says a measure tool
/// is armed — which is the same arm that would otherwise hit-test for a
/// selection, so a measure click and a selecting click are mutually exclusive
/// by construction rather than by a guard either could forget.
///
/// # The commit happens on the placing click, and there is no accept box
///
/// The old shell held a completed pick in `MeasureState::pending` and waited
/// for an explicit Accept drawn in a property bar. That is deliberately not
/// carried across, and the reason is the operator's own: decision 024 and
/// `shell-redesign.md` §2.4 exist because they disliked *"a separate accept /
/// reject box somewhere on the screen"*, and `MODES_AND_PANELS.md` now makes
/// application-initiated floating surfaces default to **Never**.
///
/// So the third click — the one that says where the dimension sits — **is** the
/// commit. That is also what SolidWorks does, which is what the standing
/// *"make it work the way other programs do"* tie-breaker asks for, and the
/// corrective for a mis-placed dimension is undo, exactly as it is for a
/// mis-drawn markup.
///
/// `pending` is therefore never set here. It stays on [`MeasureState`] because
/// it is salvaged state with its own tests, and because a future property
/// surface that is *not* a floating box would use it.
///
/// # ★ The circular tool is the exception, and it has two endings
///
/// Every sentence above is about a gesture whose *arity* ends it. The
/// radius/diameter tool has none — see [`MeasureKind::Circular`] — so the
/// operator ends it, in one of two ways:
///
/// | ending | where it is taken | why it exists |
/// |---|---|---|
/// | **double-click** on the canvas | this function, the `double` flag on [`Pick`] | what every drawing package's multi-pick tool uses; the standing *"make it work the way other programs do"* tie-breaker |
/// | **`measure.finish`** on the ribbon | [`finish`], through the dispatcher | discoverable without knowing the double-click, and reachable when the last pick sits somewhere awkward to double-click |
///
/// Both call [`circular::commit`] and nothing else raises a circular
/// `Action::CommitDimension`. Neither is a floating accept box, so decision 024
/// stands.
///
/// **The first click of a double-click is still a pick**, and that is
/// deliberate: it toggles whatever it lands on, exactly as a single click
/// would, and then the second click finishes. The alternative — swallowing the
/// pair — would make the operator's last object need a separate click *and* a
/// double-click somewhere harmless. It is also the convention this canvas
/// already follows: [`crate::canvas::selection::SelectionState::click`] takes
/// the same flag and gives the *second* click its own meaning (descend a rung)
/// rather than repeating the first's.
pub(super) fn click(pick: Pick<'_>, actions: &mut Vec<Action>) {
    let Pick {
        ctx,
        doc,
        page_index,
        kind,
        canvas_point,
        double,
        targets,
        map,
    } = pick;
    let Some(page) = doc.current_page() else {
        return;
    };
    // The one conversion, through the renderer's own transform — never a
    // hand-written Y-flip. The rule `canvas::markup::endpoints` states.
    let Some(pdf) = viewer::canvas_to_pdf_space(canvas_point, page) else {
        return;
    };
    let picked = Point {
        x: f64::from(pdf.x),
        y: f64::from(pdf.y),
    };

    let mut st = load(ctx, page_index, kind);

    // ★ The circular tool is handled BEFORE the snap resolution, and that
    // ordering is a decision rather than convenience.
    //
    // Everything between here and the `match` below resolves a *point*: it
    // moves the click onto a nearby endpoint, and it makes a click on a
    // **derived** candidate cost two presses so an inference is confirmed
    // rather than assumed. The circular pick commits no point. It toggles an
    // **object** into a fit set, and the object under the pointer is the same
    // object whether or not there is a midpoint six pixels away.
    //
    // Running it through that machinery anyway would take the second rule and
    // apply it where it means nothing: an operator picking an arc that happens
    // to lie near an inferred centerline would find their first click did
    // nothing at all, with the disclosure explaining a confirmation they were
    // never asking for. That is a real state, not a contrived one — derived
    // centerlines are inferred from exactly the kind of drawn geometry a
    // radius dimension is placed on.
    if kind == MeasureKind::Circular {
        let before = actions.len();
        circular::click(
            &mut st,
            page_index,
            canvas_point,
            double,
            targets,
            map,
            actions,
        );
        trace_pick(kind, &st, actions.len() > before);
        store(ctx, st);
        return;
    }

    // ★ The pick is the SNAPPED point, not the pointer — and it is the point
    // the indicator was drawn over, because both read one `Resolved`.
    //
    // Falls back to the raw pick when the frame resolved nothing, which is the
    // honest answer: no decomposition, or snapping off, or nothing near.
    let alt_held = ctx.input(|i| i.modifiers.alt);
    let (p_raw, candidate) =
        match resolve_hover(ctx, doc, page_index, Some(canvas_point), targets, map) {
            Some(r) => (r.at, r.candidate),
            None => snapped(&st, picked, alt_held, targets, page_index, map),
        };

    // The fuzzy-never-sneaky gate (rule 4). A **derived** candidate — a
    // centerline pdfce inferred rather than one the file states — is not
    // committed by the click that finds it: the first click promotes it and a
    // second click on the same point confirms. That is the whole reason
    // `resolve_click` takes this flag, and passing a constant `false` (which is
    // what this call site did until the query above existed) quietly turned an
    // inference into a commitment.
    let is_derived = candidate.is_some_and(|c| snap::snap_commit_clicks(c.kind) > 1);
    let ClickOutcome::Commit(p) = st.resolve_click(p_raw, is_derived) else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "measure-pick outcome=Promoted reason=derived-candidate-needs-confirm".to_owned()
        });
        store(ctx, st);
        return;
    };
    st.snap_cycle = 0;

    let before = actions.len();
    match kind {
        MeasureKind::Linear => {
            if let Some(authored) = st.linear.commit_point(p) {
                actions.push(Action::CommitDimension {
                    page: page_index,
                    group: st.group,
                    kind: authored,
                });
            }
        }
        // Taken above, before the point resolution this arm is unreachable
        // from. Spelled rather than wildcarded so that a fifth kind added to
        // the enum fails to compile here instead of quietly falling into a
        // branch written for something else.
        MeasureKind::Circular => {}
        // ★ The calibration pick. Two points measure a reference length; the
        // dialog then asks what that length IS on the real thing.
        //
        // It raises NO action on the picks themselves, which is the difference
        // from every arm above. `ScalePick::commit_point` returns `true` on the
        // click that completes the line, and the application notices
        // `dialog_open()` on the next frame and puts the Set-scale dialog up
        // with the measured length in it. Nothing is authored until the
        // operator says what the distance represents and accepts — the
        // fuzzy-never-sneaky rule, applied to the one gesture whose output is
        // a number every later dimension is multiplied by.
        MeasureKind::Scale => {
            if st.scale.commit_point(p) {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    format!(
                        "measure-scale-line pdf_length={:.3}",
                        st.scale.drawn_pdf_length.unwrap_or(0.0)
                    )
                });
            }
        }
        MeasureKind::TwoLine => {
            // The pick is against the page's own geometry, so it needs the
            // decomposition — which is why `needs_targets` asks for one when a
            // measure tool is armed even in a mode that cannot select. Picking
            // a line is *reading* the page, not selecting it.
            let Some(model) = targets.and_then(|t| t.page_objects_model(page_index)) else {
                store(ctx, st);
                return;
            };
            if let Some(line) = pick_line_in_page(model, p, map.tolerance()) {
                let epsilon = ParallelPolicy::default().epsilon_degrees;
                st.two_lines.offer_line(line, epsilon);
                if let Some(Ok(authoring)) =
                    st.two_lines.authoring(epsilon, TwoLinePlacement::default())
                {
                    actions.push(Action::CommitDimension {
                        page: page_index,
                        group: st.group,
                        kind: authoring.kind,
                    });
                    st.two_lines.clear();
                }
            }
        }
    }

    trace_pick(kind, &st, actions.len() > before);
    store(ctx, st);
}

/// The one `measure-pick` trace line, emitted from both click paths.
///
/// A function rather than the `format!` written twice, because the circular arm
/// returns before the tail of [`click`] and a harness reading this channel must
/// not have to know which arm produced its line. Two spellings would drift on
/// the first field anyone added.
fn trace_pick(kind: MeasureKind, st: &MeasureState, committed: bool) {
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI.
            //
            // An armed measure tool and an un-armed one are the same
            // screenshot, and so are a first pick and a second — defect 8's
            // lesson. This line is how a harness proves a click became a pick.
            "measure-pick kind={kind:?} in_progress={} committed={committed}",
            st.gesture_in_progress(),
        )
    });
}

/// **Draw what the next click would commit.**
///
/// Rule 4's pre-commit affordance: *"a snap indicator, a hover highlight, a
/// rubber-band … these are the cursor; they describe what is about to
/// happen."* It is only honest if it is derived from the values the commit will
/// use, which is why the placing preview goes through
/// [`pick::dimension_preview_segments`] — the *same* function a committed
/// dimension is drawn from — rather than drawing a line of its own.
/// # ★ The circular tool's preview is the whole of its feedback
///
/// The other two tools draw something that follows the pointer, so an operator
/// can see the gesture working. The circular tool's pick lands on geometry that
/// is *already drawn* — toggling an arc into the fit changes nothing on screen
/// unless this function says so — and its answer is a circle nobody has drawn
/// at all. Without both halves the operator cannot tell a set of three arcs
/// that fits their hole from one that has accidentally caught the leader line
/// beside it, and the residual is invisible until the dimension is already on
/// the page.
///
/// So two things are drawn, and the second goes through
/// [`pick::dimension_preview_segments`]:
///
/// 1. **The picked objects, outlined** — from `picked`, resolved by
///    [`circular::pick_outlines`] while the decomposition was still borrowed.
/// 2. **The fitted circle**, derived by handing the *same*
///    [`pick::CircularPick::author`] value the commit would use to the *same*
///    segment function a committed dimension is drawn from. That is this
///    module's standing rule and it matters more here than anywhere else: the
///    fit is an inference, and an inference previewed by a second derivation is
///    an inference the operator cannot actually check.
pub(super) struct Preview<'a> {
    /// The open document, for the page transform.
    pub doc: &'a OpenDoc,
    /// The page being drawn on.
    pub page_index: usize,
    /// Which measure tool is armed.
    pub kind: MeasureKind,
    /// The frame's screen ⟷ canvas map. **The projection every mark here goes
    /// through**, and the reason this struct exists — see [`preview`].
    pub map: &'a PageMapping,
    /// Where the pointer would pick, resolved once for the frame.
    pub hover: Option<Resolved>,
    /// The circular pick set's objects, as **canvas-space** rects.
    ///
    /// Resolved in `canvas::interact` while the decomposition is still
    /// borrowed, for the same reason [`Resolved`] is: the draw happens after
    /// the `Ref` is dropped, so the query cannot happen here. Empty for every
    /// other tool.
    pub picked: &'a [Rect],
}

pub(super) fn preview(ui: &Ui, preview: Preview<'_>) {
    let Preview {
        doc,
        page_index,
        kind,
        map,
        hover,
        picked,
    } = preview;
    let Some(page) = doc.current_page() else {
        return;
    };
    let ctx = ui.ctx();
    let Some(st) = read(ctx) else {
        return;
    };
    if st.page_index != page_index {
        return;
    }
    let color =
        snap::snap_indicator_tint(ctx).unwrap_or_else(|| ui.visuals().selection.stroke.color);

    // ★ The picked objects, outlined, and drawn on EVERY frame the set is
    // non-empty — not only while the pointer is over the canvas.
    //
    // A `hover` of `None` means the pointer has left the widget, which for the
    // other two tools means there is nothing to preview against. Here it means
    // the operator has moved to the ribbon to press Finish, which is exactly
    // when they most need to still be able to see what is in the set.
    let painter = ui.painter();
    let stroke = egui::Stroke::new(1.5, color);
    for rect in picked {
        painter.rect_stroke(
            map.rect_to_screen(*rect),
            0.0,
            stroke,
            egui::StrokeKind::Outside,
        );
    }

    // ★ The snap indicator is drawn BEFORE the in-progress check, and that is
    // the point of it.
    //
    // It has to appear while the operator is still deciding *where to click
    // first* — that is when it does its work, by saying "this click will land
    // on that endpoint, not where your pointer is". Gating it on a gesture
    // already being in progress would show it only after the first pick, i.e.
    // everywhere except the place it is needed most.
    //
    // This is also what makes the snap honest rather than sneaky
    // (`pdfce_FeatureRequests/README.md` rule 4): the point is moved, and the
    // operator is told, before anything is committed. A snap that silently
    // relocated a click would be an inference applied without disclosure.
    if let Some(c) = hover.and_then(|h| h.candidate)
        && let Some(screen) = page_to_screen(c.point, page, map)
    {
        // Zoom-invariant: the marker is a screen-space affordance, so its size
        // is in points and does not grow with magnification.
        ui.painter().extend(snap::snap_marker_shapes(
            screen,
            c.kind,
            color,
            SNAP_MARKER_PT,
        ));
    }

    if !st.gesture_in_progress() {
        return;
    }

    let segments: Vec<(Point, Point)> = match kind {
        // ★ The reference line, drawn exactly as the linear tool draws its
        // measuring segment — because it IS one. `ScalePick::line` is a
        // `LinearPick`, so an operator calibrating sees the same constrained
        // A-to-pointer rubber band, the same snap markers and the same
        // H/V/aligned behaviour they already know from placing a dimension.
        //
        // Once both points are picked the segment stops following the pointer
        // and the dialog is up, so the line stays drawn while the operator
        // types what it represents — which is the picture that makes the
        // question answerable: *this* line is how long?
        MeasureKind::Scale => {
            if st.scale.drawn_pdf_length.is_some() {
                // Complete: hold the committed line still while the operator
                // types. `placing_preview` is fed the pointer only so it can
                // answer at all; with both points picked the segment it
                // returns is the picked line and does not follow the pointer.
                hover
                    .and_then(|h| st.scale.line.placing_preview(h.at))
                    .as_ref()
                    .map_or_else(Vec::new, pick::dimension_preview_segments)
            } else {
                let Some(at) = hover.map(|h| h.at) else {
                    return;
                };
                st.scale.line.preview_segment(at).into_iter().collect()
            }
        }
        MeasureKind::Linear => {
            // ★ The placing preview needs the pointer; the measuring one does
            // too. With the pointer off the widget there is nothing honest to
            // draw, so nothing is.
            let Some(at) = hover.map(|h| h.at) else {
                return;
            };
            if let Some(authored) = st.linear.placing_preview(at) {
                // Placing: draw the dimension itself, exactly as it will land.
                pick::dimension_preview_segments(&authored)
            } else {
                // Measuring: the constrained A→pointer segment.
                st.linear.preview_segment(at).into_iter().collect()
            }
        }
        // ★ The fitted circle, from the value the commit would author.
        //
        // `author()` is `None` for a degenerate set — one arc, or three points
        // on a line — and that draws nothing, which is the correct picture: the
        // objects are outlined so the operator can see what is in the set, and
        // no circle appears because there is no circle. Drawing a guess would
        // promise a dimension `circular::commit` is about to refuse.
        MeasureKind::Circular => st
            .circular
            .author()
            .map(|kind| pick::dimension_preview_segments(&kind))
            .unwrap_or_default(),
        // A two-line pick has nothing to preview against the pointer: what it
        // has picked is a *line already on the page*, which the page is already
        // drawing. Highlighting the picked line is the affordance it wants, and
        // that needs the hover query this call site does not yet run — see the
        // module header's note on what remains.
        MeasureKind::TwoLine => Vec::new(),
    };
    if segments.is_empty() {
        return;
    }

    for (a, b) in segments {
        let (Some(sa), Some(sb)) = (page_to_screen(a, page, map), page_to_screen(b, page, map))
        else {
            continue;
        };
        painter.line_segment([sa, sb], stroke);
    }
}

/// How large the snap marker is drawn, in **points**.
///
/// Screen-space rather than page-space on purpose: the marker is an
/// affordance, not content, so it must stay the same apparent size whether the
/// operator is zoomed to a whole A1 sheet or to one dimension line. Carried
/// from the old shell's own indicator sizing.
const SNAP_MARKER_PT: f32 = 6.0;

/// **PDF user space → screen**, both hops, in one place.
///
/// # ★ This function is the fix for a defect, and the defect had shipped
///
/// It replaced a `page_to_canvas` that stopped after the first hop and handed
/// the result straight to `ui.painter()`. Three frames are in play — screen,
/// canvas and PDF user (`crate::canvas::mapping`'s header carries the table) —
/// and `viewer::pdf_space_to_canvas` lands in the **middle** one: y-down, but
/// with its origin at the page's top-left corner and **no zoom applied**,
/// because `page_device_geometry` is asked for scale `1.0`. The painter speaks
/// screen. So every mark this module drew — the snap indicator and the linear
/// preview alike — was offset by wherever the page happened to sit in the
/// window and drawn at 100 % regardless of the actual magnification.
///
/// It is exactly `HANDOFF.md` defect 4's shape (Find's bar drawing 108 pt left
/// of its place) and it is invisible to every test in this file, because the
/// tests that exist are about *which point is picked* and the picking was
/// always right. Only the drawing was wrong, and only in a window.
///
/// The second hop is [`PageMapping::to_screen`] — the canvas's own outward
/// boundary crossing, the one `canvas::overlay` has always used for selection
/// outlines. There is no third conversion here and no arithmetic; if there
/// were, it would be the second place in `canvas/` that divides by zoom, which
/// `crate::canvas::mapping`'s header forbids.
fn page_to_screen(p: Point, page: &pdfce_core::page_tree::Page, map: &PageMapping) -> Option<Pos2> {
    let canvas = viewer::pdf_space_to_canvas(Pos2::new(p.x as f32, p.y as f32), page)?;
    Some(map.to_screen(canvas))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::target::StubTargets;

    /// ★ **Snapping off means the raw pointer, unchanged.**
    ///
    /// The master toggle is the operator's, and a tool that snapped anyway
    /// would be applying an inference they had switched off — which is rule 4's
    /// definition of sneaky rather than fuzzy.
    #[test]
    fn the_master_toggle_off_returns_the_raw_point() {
        let mut st = MeasureState::for_kind(0, MeasureKind::Linear);
        st.snap_master = false;
        let raw = Point { x: 10.5, y: 20.25 };
        let targets = StubTargets::default();
        let map = crate::canvas::mapping::PageMapping::new(
            egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(200.0, 300.0)),
            (200.0, 300.0),
            1.0,
        );
        let (at, candidate) = snapped(&st, raw, false, Some(&targets), 0, &map);
        assert_eq!(at, raw, "the pick is where the pointer was");
        assert!(candidate.is_none(), "nothing to draw an indicator for");
    }

    /// ★ **Alt refuses the snap for one pick**, which is what makes a generous
    /// catch radius affordable — see `PageMapping::snap_tolerance`.
    #[test]
    fn alt_overrides_an_enabled_master_toggle() {
        let st = MeasureState::for_kind(0, MeasureKind::Linear);
        assert!(st.snap_master, "the toggle defaults on");
        let raw = Point { x: 1.0, y: 2.0 };
        let targets = StubTargets::default();
        let map = crate::canvas::mapping::PageMapping::new(
            egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(200.0, 300.0)),
            (200.0, 300.0),
            1.0,
        );
        let (at, candidate) = snapped(&st, raw, true, Some(&targets), 0, &map);
        assert_eq!(at, raw);
        assert!(candidate.is_none());
    }

    /// With no decomposition there is nothing to snap to, and that is a real
    /// case rather than a defensive one: the model is built only when something
    /// asks, and this is one of the things that asks.
    #[test]
    fn no_decomposition_means_no_snap_and_no_panic() {
        let st = MeasureState::for_kind(0, MeasureKind::Linear);
        let raw = Point { x: 3.0, y: 4.0 };
        let map = crate::canvas::mapping::PageMapping::new(
            egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(200.0, 300.0)),
            (200.0, 300.0),
            1.0,
        );
        let (at, candidate) = snapped(&st, raw, false, None, 0, &map);
        assert_eq!(at, raw);
        assert!(candidate.is_none());
    }

    /// ★ **The snap radius is wider than the selection radius, and stays so at
    /// every zoom.**
    ///
    /// Both are screen-pixel constants divided by the same zoom, so the
    /// relation is scale-invariant — asserting it at one zoom would be the
    /// *"relation rather than magnitude"* trap `HANDOFF.md` §2 names, so the
    /// magnitudes are checked too.
    #[test]
    fn the_snap_radius_is_wider_than_the_selection_radius_at_every_zoom() {
        for zoom in [0.05_f32, 0.5, 1.0, 4.0, 32.0] {
            let map = crate::canvas::mapping::PageMapping::new(
                egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(200.0, 300.0)),
                (200.0, 300.0),
                zoom,
            );
            let snap = map.snap_tolerance();
            let select = map.tolerance();
            assert!(
                snap > select,
                "zoom={zoom}: snap {snap} must out-reach selection {select}"
            );
            assert!(
                snap.is_finite() && snap > 0.0,
                "zoom={zoom}: a catch radius of {snap} would snap to everything or nothing"
            );
        }
    }

    // =====================================================================
    // The radius/diameter tool: the pick, and the two endings
    // =====================================================================

    /// A square inscribed in a circle of radius 10 centred at (30, 40) — a
    /// four-point set that fits exactly, so the residual is 0 and any drift in
    /// the authored geometry is visible rather than absorbed.
    fn circle_samples() -> Vec<Point> {
        vec![
            Point::new(40.0, 40.0),
            Point::new(30.0, 50.0),
            Point::new(20.0, 40.0),
            Point::new(30.0, 30.0),
        ]
    }

    /// ★ **Changing tool discards the circular pick set.**
    ///
    /// `MeasureState::set_kind` owns the rule and has its own tests; this
    /// asserts the *hosting* applies it, because `load` is what calls it and a
    /// hosting that skipped the call would carry a fit set into the linear tool
    /// — where it would sit invisible, unfinishable, and would reappear the
    /// moment the operator came back.
    #[test]
    fn arming_another_measure_tool_discards_the_circle_fit() {
        let ctx = egui::Context::default();
        let mut st = MeasureState::for_kind(0, MeasureKind::Circular);
        st.circular.toggle_object(0, circle_samples());
        store(&ctx, st);

        let switched = load(&ctx, 0, MeasureKind::Linear);
        assert!(
            !switched.circular.in_progress(),
            "the fit set means nothing to the linear tool"
        );
    }

    /// ★ **Leaving the page discards it too**, which is `load`'s other
    /// synchronisation and the one an operator reaches by paging through a
    /// drawing set with a tool still armed.
    #[test]
    fn navigating_to_another_page_discards_the_circle_fit() {
        let ctx = egui::Context::default();
        let mut st = MeasureState::for_kind(0, MeasureKind::Circular);
        st.circular.toggle_object(0, circle_samples());
        store(&ctx, st);

        let moved = load(&ctx, 1, MeasureKind::Circular);
        assert_eq!(moved.page_index, 1);
        assert!(
            !moved.circular.in_progress(),
            "a fit assembled on sheet 1 means nothing on sheet 2"
        );
    }

    /// ★ **The preview is drawn in SCREEN space, through the frame's map.**
    ///
    /// The regression test for the defect `page_to_screen`'s own docs describe:
    /// `viewer::pdf_space_to_canvas` lands in **canvas** space — page top-left
    /// origin, no zoom — and the painter speaks screen, so a preview that
    /// stopped after the first hop drew every mark offset by wherever the page
    /// sat in the window and at 100 % whatever the magnification.
    ///
    /// Asserted as a **magnitude**, not a relation: at zoom 2 with the page's
    /// corner at (37, 11), the page-space point that is 50 canvas units in from
    /// the page corner must land 100 screen points in from (37, 11). A test
    /// that merely checked "the two differ" would be satisfied by any wrong
    /// answer in the right direction — `HANDOFF.md` §2's own lesson.
    #[test]
    fn the_preview_projects_page_space_all_the_way_to_the_screen() {
        let origin = egui::Pos2::new(37.0, 11.0);
        let zoom = 2.0_f32;
        let extent = (200.0_f32, 300.0_f32);
        let map = PageMapping::new(
            egui::Rect::from_min_size(origin, egui::vec2(extent.0 * zoom, extent.1 * zoom)),
            extent,
            zoom,
        );
        // Canvas (50, 60) is 50 across and 60 down from the page's top-left.
        let canvas = egui::Pos2::new(50.0, 60.0);
        let screen = map.to_screen(canvas);
        assert!(
            (screen.x - (origin.x + 100.0)).abs() < 1e-3
                && (screen.y - (origin.y + 120.0)).abs() < 1e-3,
            "the second hop must apply the page origin AND the zoom, got {screen:?}"
        );
        // …and the canvas coordinate on its own is neither, which is what made
        // the defect invisible at zoom 1 with the page at the window's origin.
        assert!(
            (canvas.x - screen.x).abs() > 1.0,
            "a canvas coordinate handed straight to the painter is the defect"
        );
    }
}

#[cfg(test)]
mod kind_tests {
    use super::MeasureKind;

    /// ★ **Every variant is either on the ribbon or deliberately excluded.**
    ///
    /// `MeasureKind::ALL` stopped being exhaustive over the enum when
    /// [`MeasureKind::Scale`] arrived — it is armed from the Set-scale dialog,
    /// not from a ribbon control, so listing it there would fail
    /// `every_measure_kind_has_a_registered_command` for a kind that correctly
    /// has no command.
    ///
    /// An inventory with a silent exception is how a future kind ships armed by
    /// nothing, which is the exact failure `ALL` was written to prevent. So the
    /// exhaustiveness is moved here and made a **compile-time** obligation: the
    /// `match` below has no wildcard, so a new variant does not build until
    /// somebody decides which list it belongs in.
    ///
    /// The run-time half then checks the two lists are disjoint and complete,
    /// so a kind cannot be quietly in both or in neither.
    #[test]
    fn every_variant_is_either_offered_or_deliberately_excluded() {
        // No wildcard. This is the assertion; the body is bookkeeping.
        fn classify(k: MeasureKind) -> &'static str {
            match k {
                MeasureKind::Linear | MeasureKind::Circular | MeasureKind::TwoLine => "ribbon",
                MeasureKind::Scale => "elsewhere",
            }
        }

        for k in MeasureKind::ALL {
            assert_eq!(
                classify(*k),
                "ribbon",
                "{k:?} is in ALL, so it must be a ribbon kind"
            );
            assert!(
                !MeasureKind::ARMED_ELSEWHERE.iter().any(|(e, _)| e == k),
                "{k:?} is in BOTH lists — it cannot be armed from the ribbon and not"
            );
        }
        for (k, where_armed) in MeasureKind::ARMED_ELSEWHERE {
            assert_eq!(
                classify(*k),
                "elsewhere",
                "{k:?} is excluded from the ribbon and the classifier disagrees"
            );
            assert!(
                !where_armed.is_empty(),
                "{k:?} is excluded with no surface named — 'excluded' with no                  destination is indistinguishable from 'forgotten'"
            );
        }
        assert_eq!(
            MeasureKind::ALL.len() + MeasureKind::ARMED_ELSEWHERE.len(),
            4,
            "a variant was added to the enum and to neither list, or counted twice"
        );
    }
}
