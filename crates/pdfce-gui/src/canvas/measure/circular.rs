//! # `canvas::measure::circular` — the radius/diameter tool, and the gesture
//! the operator has to end
//!
//! The canvas hosting for [`MeasureKind::Circular`] alone: what one click does
//! to the fit set, what the two endings do, and what has to be resolved out of
//! the decomposition so the set can be drawn. [`super`] hosts the other two
//! tools and everything the three share — the memory, the snap resolution, the
//! preview painting.
//!
//! ## ★ Why this is a file of its own, and what the seam actually is
//!
//! **R2** (no `.rs` file over 1,500 lines) forced a split when the tool was
//! armed: [`super`] reached 1,617 lines. But the line count only says *that*
//! something had to move; it does not say what, and `tools/gates/check-file-size.sh`
//! says in its own header that shaving prose to fit a threshold is the
//! behaviour it exists to refuse. So the question was which subject was
//! separable, and this one is, for a reason none of the other tools give:
//!
//! > **Linear and two-line gestures end themselves. This one does not.**
//!
//! A linear dimension is finished at its third click and a two-line dimension
//! at its second, because both are picks of a **fixed arity** — the pick
//! machine in [`super::pick`] knows it is done, and [`super::click`] simply
//! raises whatever the machine hands back. A best-fit circle has no such
//! number. An arc drawn as four separate polyline objects needs four picks; the
//! same arc drawn as one needs one; nothing in the geometry can tell pdfce
//! which the operator meant. So the operator says when, and the machinery for
//! *saying when* — two entrances, one commit path, a predicate the ribbon reads
//! every frame to decide whether the control is even live — is a subject the
//! other two tools have nothing corresponding to.
//!
//! That is the seam. Everything here answers *"when is this gesture over, and
//! what does ending it do?"*; everything left in [`super`] answers *"where did
//! that click land?"*.
//!
//! ## The two endings, and why there is exactly one commit path
//!
//! | ending | entrance | why it exists |
//! |---|---|---|
//! | **double-click** on the canvas | [`click`], via [`super::click`]'s `double` flag | what every drawing package's multi-pick tool uses; the standing *"make it work the way other programs do"* tie-breaker |
//! | **`measure.finish`** on the ribbon | [`finish`], via `app::dispatch` | discoverable without knowing the double-click, and reachable when the last picked arc sits somewhere awkward to double-click |
//!
//! Both call [`commit`] and nothing else raises a circular
//! `Action::CommitDimension`. Two arms that each assembled a `DimensionKind`
//! would be two derivations of one answer: they would agree on the day they
//! were written, diverge at the first change to either, and **the operator
//! would have no way to see it** — a circle fitted from the same points looks
//! the same whichever code drew it.
//!
//! Neither ending is an accept box floating over the canvas, which is what
//! decision 024 retired at the operator's instruction and what kept this tool
//! unarmed through Phase 7.
//!
//! ## This module owns no geometry either
//!
//! The fit is [`pdfce_core::dimension::fit_circle_taubin`], reached through
//! [`super::pick::CircularPick`]; the authored value is `pdfce-core`'s own
//! `DimensionKind`. Nothing here computes a centre, a radius or a residual.
//! What it owns is *composition and lifetime*: which objects are in the set,
//! when the set becomes a dimension, and when it is emptied.

use egui::{Pos2, Rect};

use super::{MeasureKind, MeasureState, read, store};
use crate::app::actions::Action;
use crate::app::actions::dimensions::DimensionAction;
use crate::canvas::mapping::PageMapping;
use crate::canvas::target::CanvasTargetProvider;

/// **The circular pick set that is ready to become a dimension**, or `None`.
///
/// The single derivation behind both halves of the Finish control:
/// [`finishable`] asks whether to enable it and [`finish`] asks what to do when
/// it is pressed. Two spellings of "is there something to finish?" would
/// eventually disagree, and the way they would disagree is the worst available
/// — an enabled control that does nothing when pressed, which is precisely the
/// placeholder the no-placeholders invariant forbids.
///
/// Three conditions, and each rules out a state that really occurs:
///
/// 1. **The radius/diameter tool is armed.** The pick set outlives disarming
///    (`disarm_measure` puts the tool down; it does not discard work — see its
///    docs, and Escape's two rungs), so without this the ribbon would offer
///    Finish for a set the operator can no longer see being outlined.
/// 2. **A state exists.** Nothing has been picked on this page since the tool
///    was armed, so there is nothing to finish.
/// 3. **The fit is not degenerate** — [`super::pick::CircularPick::author`]
///    returns `None` for fewer than three usable points or a numerically
///    singular set, and its own docs say that is when Accept must not be
///    offered. One or two picked objects whose anchors lie on a line is the
///    ordinary way to reach it, not a pathological one.
fn pending(ctx: &egui::Context) -> Option<MeasureState> {
    if crate::canvas::tool::selected(ctx).measure_kind() != Some(MeasureKind::Circular) {
        return None;
    }
    let st = read(ctx)?;
    st.circular.author().map(|_| st)
}

/// **Is there a circle fit waiting to be committed?** — the application state
/// behind the `measure.finishable` condition.
///
/// Published by `crate::app::PdfceApp::conditions` and read by
/// `measure.finish`'s `enabled_when`. See [`pending`] for what the three
/// conditions are and why each one is needed.
#[must_use]
pub fn finishable(ctx: &egui::Context) -> bool {
    pending(ctx).is_some()
}

/// **End the gesture: author the dimension and empty the pick set.**
///
/// ★ **The one commit path**, reached by both endings — see the module header
/// for the argument, which is the reason this function exists rather than two
/// arms that each build a `DimensionKind`.
///
/// Pure over the state and the action list — no `egui`, no context, no memory —
/// which is what makes both endings assertable without a window.
///
/// Returns `false` and raises nothing when the fit is degenerate. That is the
/// same refusal [`super::pick::CircularPick::author`] states: an inference
/// pdfce cannot make is not made silently on the operator's behalf.
pub(super) fn commit(st: &mut MeasureState, page_index: usize, actions: &mut Vec<Action>) -> bool {
    let Some(kind) = st.circular.author() else {
        return false;
    };
    actions.push(Action::Dimension(DimensionAction::Commit {
        page: page_index,
        group: st.group,
        kind,
        // Nothing to disclose: a best-fit circle's output is the circle the
        // operator assembled, and its residual is already on screen through the
        // live preview. See `DimensionAction::Commit`'s field.
        disclosures: Vec::new(),
    }));
    // Emptied, not left standing. The next dimension starts from nothing, the
    // same way `LinearPick` resets on its placing click — otherwise a second
    // Finish would author the same circle again from a set the operator
    // believes they have already spent.
    st.circular.clear();
    true
}

/// **The `measure.finish` command's whole effect**, reporting whether it did
/// anything.
///
/// The second entrance to [`commit`], and the only thing it adds is the trip
/// through `egui::Memory`: read the state, run the one commit path, write it
/// back. The page comes from the **state**, not from the current view, because
/// the pick was made on that page and a state whose page has been left behind
/// is cleared by `super::load` on the next frame anyway — reading
/// `doc.view.page_index` here would be a second source of truth for a fact the
/// state already carries.
///
/// Returns `false` when there is nothing to finish, so the dispatcher can say
/// which kind of nothing happened rather than tracing a success it did not
/// have.
pub fn finish(ctx: &egui::Context, actions: &mut Vec<Action>) -> bool {
    let Some(mut st) = pending(ctx) else {
        return false;
    };
    let page_index = st.page_index;
    if !commit(&mut st, page_index, actions) {
        return false;
    }
    store(ctx, st);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI.
        //
        // The `add-dimension` line the engine traces proves the edit landed;
        // this one proves which of the two endings asked for it, which a
        // screenshot cannot distinguish and neither can the engine.
        format!("measure-finish via=command page={page_index}")
    });
    true
}

/// **Take one click for the radius/diameter tool** — a toggle, or the ending.
///
/// The whole of the gesture's input. Called from [`super::click`] before that
/// function's point-resolution machinery runs, and its own docs carry the
/// argument for that ordering: this pick commits no *point*, so the snap query
/// and the derived-candidate two-click confirm have nothing to contribute and
/// would cost the operator a click for nothing.
///
/// # The order of the two questions
///
/// A **double**-click finishes and picks nothing further. The first click of
/// the pair has already been through here as an ordinary click and has already
/// toggled whatever it landed on — see [`super::click`]'s section on why that
/// is the right reading rather than an accident of how `egui` reports the pair.
///
/// Otherwise it is a pick, and it is resolved through
/// [`CanvasTargetProvider::hit_test`] — **the same hit test a selecting click
/// uses**, at the same tolerance, so the object the operator gets is the object
/// they would have selected had the tool not been armed. A second rule for
/// "which object is under the pointer" is exactly the drift
/// [`crate::canvas::target`]'s header exists to prevent.
///
/// # ★ An object with no fit geometry is refused, not toggled in
///
/// [`crate::panels::objects::provider::ObjectModelProvider::object_sample_points`]
/// contributes nothing for a text, image or form object — they carry no
/// anchors, which is the same exclusion the snap engine applies. Adding one to
/// the set anyway would be worse than doing nothing: the object would be
/// outlined as though it were part of the fit, the count in the pick set would
/// go up, and the fitted circle would not move. An affordance that says *"this
/// is in the fit"* about something that is not in the fit is the placeholder
/// rule broken from the inside.
///
/// A pick that is **already in the set** is toggled out regardless, because
/// removing what you added must always work — an object cannot have got in
/// without samples, so this costs nothing and closes the case where the
/// document changed underneath.
pub(super) fn click(
    st: &mut MeasureState,
    page_index: usize,
    canvas_point: Pos2,
    double: bool,
    targets: Option<&dyn CanvasTargetProvider>,
    map: &PageMapping,
    actions: &mut Vec<Action>,
) {
    if double {
        if !commit(st, page_index, actions) {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "measure-finish via=double-click outcome=declined reason=degenerate-fit".to_owned()
            });
            return;
        }
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("measure-finish via=double-click page={page_index}")
        });
        return;
    }
    // No decomposition means no object under the pointer — the honest answer,
    // and a real case rather than a defensive one: the model is built only when
    // something asks for it, and `canvas::interact` asks on every frame a
    // measure tool is armed precisely so that this is `Some` when it matters.
    let Some(targets) = targets else {
        return;
    };
    let Some(target) = targets.hit_test(page_index, canvas_point, map.tolerance()) else {
        return;
    };
    let Ok(index) = usize::try_from(target.0) else {
        return;
    };
    if st.circular.object_indices().any(|i| i == index) {
        st.circular.toggle_object(index, Vec::new());
        return;
    }
    let samples = targets.object_sample_points(page_index, index);
    if samples.is_empty() {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("measure-pick-declined object={index} reason=no-fit-geometry")
        });
        return;
    }
    st.circular.toggle_object(index, samples);
}

/// **The pick set's object outlines, in canvas space.**
///
/// Called from `canvas::interact` **before** it drops the decomposition, which
/// is the constraint that shaped this API — the identical constraint
/// [`super::resolve_hover`] exists under, and the same answer: resolve while
/// the borrow is live, draw from the resolved value afterwards.
///
/// Empty for every other tool, so an armed Linear or Two-line pays one
/// comparison and no queries. Empty, too, for an object the provider no longer
/// knows: `bounds` returns `None` after an edit renumbered the page, and the
/// correct response is to draw one outline fewer rather than to panic in the
/// frame that is trying to draw.
pub(in crate::canvas) fn pick_outlines(
    ctx: &egui::Context,
    page_index: usize,
    kind: MeasureKind,
    targets: Option<&dyn CanvasTargetProvider>,
) -> Vec<Rect> {
    if kind != MeasureKind::Circular {
        return Vec::new();
    }
    let (Some(targets), Some(st)) = (targets, read(ctx)) else {
        return Vec::new();
    };
    if st.page_index != page_index {
        return Vec::new();
    }
    st.circular
        .object_indices()
        .filter_map(|index| u64::try_from(index).ok())
        .filter_map(|id| targets.bounds(page_index, crate::canvas::target::TargetId(id)))
        .collect()
}

/// Plant a pick set in memory, for tests in sibling modules.
///
/// Two test modules need one and neither can build it the honest way, so the
/// visibility widens rather than the helper being written twice —
/// `crate::app::state::open_fixture`'s own note makes the identical argument.
/// `canvas::keys` owns Escape's precedence and has to assert that a circular
/// pick set is abandoned one press *before* the tool is put down;
/// `app::conditions` has to assert that a finishable set is still not offered
/// with no document open. Neither can assemble one the real way — that needs a
/// laid-out page, a decomposition and a click inside a drawn object — so the
/// state they must react to is planted directly, exactly as
/// `crate::canvas::guides::plant_drag_for_test` plants a guide drag for the
/// same Escape test.
///
/// `#[cfg(test)]` so it cannot become a second way for production code to build
/// a pick set. The real one is [`click`], and a second entry point is how two
/// code paths come to disagree about what a pick is.
///
/// The four points are a square inscribed in a circle of radius 10 centred at
/// (30, 40) — a **non-degenerate** set, so the planted state is one
/// [`finishable`] answers `true` for. A collinear or too-small set would make
/// every Escape test pass for the wrong reason, since the fit would be `None`
/// and nothing downstream would ever be offered.
#[cfg(test)]
pub(crate) fn plant_pick_for_test(ctx: &egui::Context, page_index: usize) {
    let mut st = MeasureState::for_kind(page_index, MeasureKind::Circular);
    st.circular.toggle_object(0, samples_on_a_circle());
    store(ctx, st);
}

/// A square inscribed in a circle of radius 10 centred at (30, 40) — a
/// four-point set that fits **exactly**, so the residual is 0 and any drift in
/// the authored geometry shows up rather than being absorbed by the fit.
#[cfg(test)]
fn samples_on_a_circle() -> Vec<pdfce_core::vector::Point> {
    use pdfce_core::vector::Point;
    vec![
        Point::new(40.0, 40.0),
        Point::new(30.0, 50.0),
        Point::new(20.0, 40.0),
        Point::new(30.0, 30.0),
    ]
}

#[cfg(test)]
#[allow(clippy::panic, reason = "a test that cannot destructure has failed")] // ui-text-exempt: clippy lint justification, never displayed
mod tests {
    use super::*;
    use crate::canvas::target::StubTargets;
    use crate::canvas::tool::{self, CanvasTool};

    /// A stub page holding one object with [`samples_on_a_circle`] behind it,
    /// plus a second object that carries **no** samples — the text/image case
    /// the pick has to refuse.
    fn targets() -> StubTargets {
        StubTargets::new(
            0,
            [
                egui::Rect::from_min_size(egui::Pos2::new(10.0, 10.0), egui::vec2(20.0, 20.0)),
                egui::Rect::from_min_size(egui::Pos2::new(80.0, 80.0), egui::vec2(20.0, 20.0)),
            ],
        )
        .with_samples(0, samples_on_a_circle())
    }

    /// The frame map these tests resolve against — page at the origin, one
    /// point per unit, so a canvas coordinate in a test reads as itself.
    fn unit_map() -> PageMapping {
        PageMapping::new(
            egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(200.0, 300.0)),
            (200.0, 300.0),
            1.0,
        )
    }

    /// ★ **A click toggles an object into the fit set, and a second click on
    /// the same object toggles it out.**
    ///
    /// The whole of the pick, and both halves matter: a build that only added
    /// would pass a test of the first click alone, and the operator's complaint
    /// would be that a mis-picked arc cannot be removed without abandoning the
    /// whole set.
    #[test]
    fn a_click_toggles_an_object_into_the_fit_set_and_out_again() {
        let mut st = MeasureState::for_kind(0, MeasureKind::Circular);
        let (targets, map) = (targets(), unit_map());
        let on_object = egui::Pos2::new(20.0, 20.0);
        let mut actions = Vec::new();

        click(
            &mut st,
            0,
            on_object,
            false,
            Some(&targets),
            &map,
            &mut actions,
        );
        assert_eq!(st.circular.object_count(), 1, "the click picked the object");
        assert_eq!(
            st.circular.samples(),
            samples_on_a_circle(),
            "with its anchors"
        );
        assert!(actions.is_empty(), "a pick authors nothing on its own");

        click(
            &mut st,
            0,
            on_object,
            false,
            Some(&targets),
            &map,
            &mut actions,
        );
        assert_eq!(st.circular.object_count(), 0, "…and toggled it back out");
        assert!(actions.is_empty());
    }

    /// ★ **An object with no fit geometry is refused rather than outlined.**
    ///
    /// A text, image or form object contributes no anchors — the same exclusion
    /// the snap engine applies. Adding one to the set anyway would outline it
    /// as though it were part of the fit and leave the fitted circle exactly
    /// where it was: an affordance saying *"this is in"* about something that
    /// is not, which is the no-placeholders rule broken from inside.
    #[test]
    fn an_object_with_no_anchors_is_refused_rather_than_added_to_the_set() {
        let mut st = MeasureState::for_kind(0, MeasureKind::Circular);
        let (targets, map) = (targets(), unit_map());
        let mut actions = Vec::new();
        // Object 1 exists, is hit, and has no samples.
        click(
            &mut st,
            0,
            egui::Pos2::new(90.0, 90.0),
            false,
            Some(&targets),
            &map,
            &mut actions,
        );
        assert_eq!(st.circular.object_count(), 0);
        assert!(!st.gesture_in_progress(), "and no gesture was begun");
    }

    /// ★ **The pick set is the tool's own and never touches the selection.**
    ///
    /// `CircularPick`'s own docs state it (ui-spec §3.1) and this is the
    /// hosting keeping it: a half-assembled circle fit is not a selection, and
    /// borrowing the selection to hold it would arm the Format tab's Delete
    /// over a set the operator assembled in order to measure with.
    ///
    /// The strongest form of the property is in the **signature** — [`click`]
    /// is handed no `SelectionState` at all — so a future edit that wanted to
    /// touch the selection would have to add a parameter and would land in
    /// front of this comment. What is asserted here is the consequence, in both
    /// directions: picking does not select, and an existing selection is not
    /// consumed by a pick.
    #[test]
    fn the_pick_never_reaches_the_selection() {
        use crate::canvas::selection::{ClickHit, SelectionState};

        let mut selection = SelectionState::default();
        let mut st = MeasureState::for_kind(0, MeasureKind::Circular);
        let (targets, map) = (targets(), unit_map());
        let mut actions = Vec::new();
        click(
            &mut st,
            0,
            egui::Pos2::new(20.0, 20.0),
            false,
            Some(&targets),
            &map,
            &mut actions,
        );
        assert_eq!(st.circular.object_count(), 1);
        assert!(
            selection.is_empty(),
            "picking an object for a circle fit must not select it"
        );

        selection.click(
            0,
            ClickHit {
                object: Some(crate::canvas::target::TargetId(1)),
                ..ClickHit::default()
            },
            false,
            false,
        );
        click(
            &mut st,
            0,
            egui::Pos2::new(20.0, 20.0),
            false,
            Some(&targets),
            &map,
            &mut actions,
        );
        assert_eq!(selection.len(), 1, "the selection is untouched either way");
    }

    /// ★ **The two endings author the same dimension from the same picks.**
    ///
    /// The property the one-commit-path design exists for, asserted the only
    /// way that means anything: run *both* endings over identical states and
    /// compare the actions they raise. Two arms that each built a
    /// `DimensionKind` would agree on the day they were written, drift on the
    /// first change to either, and the operator would have no way to see it — a
    /// circle fitted from the same points looks the same whichever code drew
    /// it.
    #[test]
    fn the_double_click_and_the_command_author_the_same_dimension() {
        // Ending 1: the double-click, taken by the canvas.
        let mut by_click = MeasureState::for_kind(2, MeasureKind::Circular);
        by_click.circular.toggle_object(7, samples_on_a_circle());
        let mut click_actions = Vec::new();
        click(
            &mut by_click,
            2,
            egui::Pos2::ZERO,
            true,
            None,
            &unit_map(),
            &mut click_actions,
        );

        // Ending 2: the ribbon command, through `egui::Memory`.
        let ctx = egui::Context::default();
        tool::select(&ctx, CanvasTool::Measure(MeasureKind::Circular));
        let mut by_command = MeasureState::for_kind(2, MeasureKind::Circular);
        by_command.circular.toggle_object(7, samples_on_a_circle());
        store(&ctx, by_command);
        let mut command_actions = Vec::new();
        assert!(finish(&ctx, &mut command_actions), "the command finishes");

        assert_eq!(
            click_actions, command_actions,
            "the two endings must place the same dimension, on the same page, \
             in the same group"
        );
        assert_eq!(click_actions.len(), 1, "exactly one dimension per ending");
        let Some(Action::Dimension(DimensionAction::Commit { page, kind, .. })) =
            click_actions.first()
        else {
            panic!("a dimension is committed")
        };
        assert_eq!(*page, 2, "on the page the pick was made on, not the view's");
        let pdfce_core::dimension::DimensionKind::Circular { fit, .. } = kind else {
            panic!("a circular dimension")
        };
        assert!(
            (fit.radius - 10.0).abs() < 1e-6 && (fit.center.x - 30.0).abs() < 1e-6,
            "the committed circle is the fitted one: {fit:?}"
        );
    }

    /// ★ **Both endings empty the pick set**, so a second Finish does not place
    /// the same circle twice.
    ///
    /// The failure without it is quiet and expensive: the operator presses
    /// Finish, sees the dimension land, presses it again out of habit or
    /// because they did not see the first, and gets two dimensions stacked
    /// exactly on top of each other — indistinguishable on screen and two undo
    /// steps to remove.
    #[test]
    fn finishing_empties_the_pick_set_so_it_cannot_be_committed_twice() {
        let mut st = MeasureState::for_kind(0, MeasureKind::Circular);
        st.circular.toggle_object(0, samples_on_a_circle());
        let mut actions = Vec::new();

        assert!(commit(&mut st, 0, &mut actions));
        assert_eq!(actions.len(), 1);
        assert!(!st.circular.in_progress(), "the set is emptied");
        assert!(
            !commit(&mut st, 0, &mut actions),
            "a second finish has nothing to commit"
        );
        assert_eq!(actions.len(), 1, "and raises nothing");
    }

    /// ★ **A degenerate set commits nothing, from either ending.**
    ///
    /// `CircularPick::author` returns `None` for fewer than three usable points
    /// or a numerically singular set, and its docs say that is precisely when
    /// Accept must not be offered. Two picked arcs whose anchors happen to lie
    /// on a line is the ordinary way to reach it — not a contrived one — and
    /// the honest response is to place nothing rather than to guess a circle.
    #[test]
    fn a_degenerate_fit_is_refused_by_both_endings() {
        use pdfce_core::vector::Point;

        let collinear = vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(20.0, 0.0),
        ];
        let mut st = MeasureState::for_kind(0, MeasureKind::Circular);
        st.circular.toggle_object(0, collinear);
        assert!(st.circular.author().is_none(), "the fixture is degenerate");

        let mut actions = Vec::new();
        assert!(!commit(&mut st, 0, &mut actions));
        assert!(actions.is_empty(), "nothing is authored");
        assert!(
            st.circular.in_progress(),
            "and the picks survive, so the operator can add another arc"
        );

        // …and the double-click reaches the same refusal rather than its own.
        click(
            &mut st,
            0,
            egui::Pos2::ZERO,
            true,
            None,
            &unit_map(),
            &mut actions,
        );
        assert!(actions.is_empty());
        assert!(st.circular.in_progress());
    }

    /// ★ **`measure.finishable` is true exactly when pressing Finish would do
    /// something** — all five of the states that decide it.
    ///
    /// This is the condition behind a ribbon control, so each `false` row is a
    /// control that would otherwise be live and inert. The fourth row is the
    /// one that is easy to miss: putting the tool down does **not** discard the
    /// pick set (Escape's two rungs, `disarm_measure`'s own docs), so without
    /// the armed-tool check the ribbon would keep offering Finish for a set
    /// nothing is outlining any more.
    #[test]
    fn finish_is_offered_only_when_there_is_a_fit_and_the_tool_is_armed() {
        let ctx = egui::Context::default();

        // 1. Nothing armed, no state.
        assert!(!finishable(&ctx), "an unarmed canvas has nothing to finish");

        // 2. Armed, but nothing picked.
        tool::select(&ctx, CanvasTool::Measure(MeasureKind::Circular));
        store(&ctx, MeasureState::for_kind(0, MeasureKind::Circular));
        assert!(!finishable(&ctx), "an empty pick set is not a circle");

        // 3. Armed with a real fit.
        plant_pick_for_test(&ctx, 0);
        assert!(finishable(&ctx), "four points on a circle are finishable");

        // 4. The same set, with the tool put down.
        tool::select(&ctx, CanvasTool::Select);
        assert!(
            !finishable(&ctx),
            "a set nothing is outlining must not keep offering Finish"
        );
        let mut actions = Vec::new();
        assert!(
            !finish(&ctx, &mut actions),
            "…and the command refuses it too, by the same predicate"
        );
        assert!(actions.is_empty());

        // 5. A *different* measure tool armed is not this tool's ending.
        tool::select(&ctx, CanvasTool::Measure(MeasureKind::Linear));
        assert!(!finishable(&ctx));
    }

    /// ★ **Asking whether Finish is available does not manufacture state.**
    ///
    /// [`finishable`] runs on every frame, for every document, armed or not. If
    /// it went through `super::load` — which builds a `MeasureState` when there
    /// is none — the ribbon merely *drawing itself* would leave a measure state
    /// in memory for a tool nobody armed, and the next `store` would persist
    /// it.
    #[test]
    fn asking_whether_finish_is_available_creates_no_measure_state() {
        let ctx = egui::Context::default();
        tool::select(&ctx, CanvasTool::Measure(MeasureKind::Circular));
        assert!(!finishable(&ctx));
        assert!(
            read(&ctx).is_none(),
            "the question must not answer itself into existence"
        );
    }

    /// ★ **The outlines drawn are the objects picked**, and they come from the
    /// provider rather than from anything this module remembers.
    ///
    /// The operator's only way to see what is in the fit. A build that returned
    /// an empty list would draw nothing, and toggling an arc in would change
    /// the screen not at all — a gesture with no feedback, which for a tool
    /// whose whole output is an inference is worse than no tool.
    #[test]
    fn the_pick_sets_outlines_are_resolved_from_the_decomposition() {
        let ctx = egui::Context::default();
        let targets = targets();
        plant_pick_for_test(&ctx, 0);

        let outlines = pick_outlines(&ctx, 0, MeasureKind::Circular, Some(&targets));
        assert_eq!(outlines.len(), 1, "one picked object, one outline");
        assert_eq!(
            outlines[0],
            egui::Rect::from_min_size(egui::Pos2::new(10.0, 10.0), egui::vec2(20.0, 20.0)),
            "and it is that object's own bounds"
        );

        // Another tool asks for nothing, so an armed Linear runs no queries.
        assert!(pick_outlines(&ctx, 0, MeasureKind::Linear, Some(&targets)).is_empty());
        // A page this state does not target contributes nothing rather than
        // outlining another sheet's objects.
        assert!(pick_outlines(&ctx, 1, MeasureKind::Circular, Some(&targets)).is_empty());
    }
}
