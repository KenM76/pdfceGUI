//! # `canvas::tool` — which pointer tool the canvas is in, and the space bar that borrows it
//!
//! ## What this module is for
//!
//! `GUI_ROADMAP.md` Phase 3.2: *"There is no hand tool at all; panning is
//! middle-drag only."* This is the hand tool, and the space bar that borrows
//! it for as long as it is held.
//!
//! It owns exactly one question — **what does the primary button mean right
//! now?** — and answers it as a pure function of two inputs: the tool the
//! operator *chose* ([`selected`]) and whether the space bar is *down*
//! ([`space_held`]). Everything else in `canvas/` reads [`active`] and
//! branches on the answer.
//!
//! ## ★ Why the space override is derived and never stored
//!
//! The requirement is *"space held = temporary pan, releasing returns to the
//! previous tool"*, and the obvious implementation — remember the previous
//! tool on key-down, restore it on key-up — is the one that fails. It fails
//! in the ordinary way (an interrupted key-up: the window loses focus mid-pan,
//! the operator alt-tabs, a dialog steals the release) and the failure is
//! *sticky*: the canvas is left in a hand tool the operator never chose and
//! cannot leave except by choosing something else. Every application that has
//! ever shipped a modal space-pan has shipped that bug at least once.
//!
//! So there is **no stored override and nothing to restore**. [`selected`] is
//! the only persistent value; the space bar is read fresh from
//! [`egui::InputState`] on every frame and composed with it by [`resolve`].
//! "Returning to the previous tool" is then not an action that can be missed —
//! it is what the next frame computes when the key is no longer down. A lost
//! key-up costs one frame of pan, not a stuck mode.
//!
//! ## ★ The text-field guard is not optional
//!
//! Space is a *character*. A canvas that panned on any Space keypress would
//! pan while the operator typed a page number into the status bar's page box
//! or a value into the Properties panel. The guard is
//! [`egui::Context::text_edit_focused`] — the same predicate, for the same
//! reason, as `DEFECTS.md` D1's Delete-key fix, and deliberately **not**
//! `egui_wants_keyboard_input()`, which is true whenever *any* widget has
//! focus and would therefore disable space-pan after a single click on the
//! canvas (the canvas takes focus on click, which is exactly how D1 happened).
//!
//! ## ★ This header used to say there would never be a third variant. What
//! changed
//!
//! Until the markup substrate landed, [`CanvasTool`]'s own doc comment read:
//!
//! > Deliberately two variants and not a general "tool" enum with markup,
//! > measure and text members. **Those are *modes* that arm a whole authoring
//! > surface and they will arrive with their own state**; this enum answers the
//! > narrow navigation question — does a primary drag select, or does it move
//! > the paper?
//!
//! That was right, and it is not being overturned — **its condition has been
//! met.** The sentence set a bar for admission ("arrives with their own
//! state"), and markup now clears it: it arrives with [`markup::MarkupKind`],
//! with a `DragKind` and a `GestureOutcome` of its own in
//! [`crate::canvas::gesture`], with a rubber band, a commit path and an
//! `Action`. What it does *not* have — and this is the part that decided the
//! shape — is any state that outlives a frame except **which kind is armed**,
//! which is precisely one enum value and is exactly the kind of thing this
//! module already stores.
//!
//! So the enum grows by one variant *carrying* the kind, rather than by four,
//! and the question it answers grows by one word: **does a primary drag select,
//! move the paper, or draw?** The two rules that made the old sentence true are
//! both still enforced here rather than at call sites —
//! [`CanvasTool::pans_with_primary`] is still the single predicate the pan and
//! gesture-suppression paths share, and [`CanvasTool::cursor`] is still the
//! single place a tool's cursor is decided.
//!
//! Measure and text are **still** outside, and for the original reason rather
//! than by inertia: `MEASURE`'s tool is a two-point pick with a snap indicator
//! and a live readout, and text editing is a caret in a re-laid-out box. Neither
//! is "what does the primary button mean"; both would drag a whole subsystem's
//! state through this type. When one of them arrives, whoever writes it should
//! have to make the same argument this paragraph makes, in this file.
//!
//! ## Where the state lives, and why `egui::Memory` is right here when it was
//! wrong for the selection
//!
//! `canvas/mod.rs`'s seam 1 records the selection being *moved out* of
//! `egui::Memory` because it is **document-scoped**: closing a document must
//! forget it, and `Memory` outlives documents. A tool is the opposite — it is
//! **application-scoped**, like the ribbon tab or the theme. An operator who
//! picks the hand tool, opens another drawing and finds themselves back in the
//! select tool would report that as a bug. So the tool stays in `Memory`
//! precisely *because* `Memory` outlives documents, which is the property that
//! disqualified it for the selection.

use egui::{CursorIcon, Key};

use crate::canvas::markup::MarkupKind;

/// `egui::Memory` key for the operator's chosen pointer tool.
const TOOL_MEMORY_KEY: &str = "pdfce-canvas-tool"; // ui-text-exempt: internal memory id, never displayed

/// What the primary button does over the page.
///
/// **Does a primary drag select, move the paper, or draw?** — the only question
/// the pan, marquee and markup paths need settled, and settling it here keeps
/// them from inventing three different answers.
///
/// Three variants, not five: [`Self::Markup`] carries **which** shape is being
/// drawn rather than there being one variant per shape. See that variant's docs
/// for the argument, and the module header for what changed since this enum
/// said it would stay at two.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CanvasTool {
    /// Click selects, drag rubber-bands. The shipped behaviour, and the
    /// default.
    #[default]
    Select,
    /// Click does nothing, drag moves the paper under the viewport.
    Hand,
    /// Drag authors a markup annotation of the carried kind.
    ///
    /// # ★ One variant carrying a kind, not one variant per shape
    ///
    /// The old shell settled this and its reasoning is carried across intact
    /// (`D:\Dev\pdfce\crates\pdfce-gui\src\canvas.rs:232-244`):
    ///
    /// > All markup kinds live in `MarkupToolState::kind` rather than becoming
    /// > separate `CanvasTool` entries […] Separate entries would put
    /// > mutually-exclusive states into a type that can express all their
    /// > combinations.
    ///
    /// That last clause is the whole argument, and it is a statement about
    /// *types* rather than about tidiness: the operator is drawing exactly one
    /// shape, so a type that can say `Rectangle` and `Ellipse` at once — which
    /// four booleans, or four variants plus a "which is active" rule spread
    /// across call sites, both can — is a type whose illegal states have to be
    /// prevented by discipline. Carrying the kind makes them unrepresentable.
    ///
    /// It also makes the *tool* branch total: every rule this enum owns —
    /// [`Self::pans_with_primary`], [`Self::cursor`], the press-kind decision in
    /// [`crate::canvas::gesture::press_kind`] — is written once for markup as a
    /// whole and cannot be written four times and forgotten once.
    ///
    /// **Changing the kind mid-drag is not possible here**, where the old shell
    /// had to discard an in-progress gesture on a kind change. Arming is a
    /// command, commands are dispatched between frames, and a drag in flight is
    /// owned by [`crate::canvas::gesture::GestureState`], which carries the kind
    /// it started with on its own `DragKind` — so a kind change mid-drag cannot
    /// reach the drag at all. The property the old shell had to enforce, this
    /// one gets from the gesture machine's existing "a drag keeps the kind it
    /// started with" rule.
    Markup(MarkupKind),
}

impl CanvasTool {
    /// Whether a primary-button drag pans the view rather than reaching the
    /// gesture machine.
    ///
    /// The whole branch, in one predicate, so the pan path and the
    /// gesture-suppression path cannot disagree about which tool pans — a
    /// disagreement whose symptom would be a drag that pans **and** marquees,
    /// which is one of the two things this stage must not ship.
    ///
    /// The markup tool answers `false`, which is what makes a markup drag reach
    /// the gesture machine at all: `canvas::interact` hands that machine a
    /// **blank** frame whenever this is `true`. Space-to-pan still works over
    /// the markup tool, because [`resolve`] composes the held space bar *before*
    /// this is asked — so a held space bar borrows the hand out of the markup
    /// tool exactly as it does out of the select tool, and releasing it hands
    /// the markup tool back with nothing stored and nothing to restore.
    #[must_use]
    pub fn pans_with_primary(self) -> bool {
        matches!(self, Self::Hand)
    }

    /// The cursor this tool shows, or `None` to leave the cursor to whatever
    /// else the canvas is doing with it (a grip, a marquee, a move drag).
    ///
    /// `Grab` when the hand is available and `Grabbing` while it is closed, in
    /// the direction every browser, CAD package and image editor uses. The
    /// pair matters: the requirement is that the cursor *changes and changes
    /// back*, and a single hand cursor for both states would leave an operator
    /// unable to tell a hand tool that is working from one that has run out of
    /// scroll range — the exact ambiguity the middle-drag path's own
    /// `Grabbing` was added to remove.
    ///
    /// `Select` returns `None` rather than `Default`: returning a cursor here
    /// would overwrite the grip cursors that [`crate::canvas::handles`] sets
    /// for the eight resize handles, and a resize grip that loses its cursor
    /// is a grip nobody can find.
    ///
    /// `Markup` returns `Crosshair` in **both** states, and the sameness is
    /// deliberate where the hand's pair is deliberately different. The hand
    /// needs to distinguish "available" from "closed" because a pan that has
    /// run out of scroll range is otherwise indistinguishable from a pan that
    /// is not working; a markup drag has no such failure — the band under the
    /// pointer is the feedback, and a cursor that changed under it would
    /// compete with the thing it is describing. What the crosshair says is
    /// *"this canvas draws now"*, which is true from the moment the tool is
    /// armed until it is retired, and returning it also **suppresses the grip
    /// cursors** — correctly, because a markup drag over a selected object
    /// draws a shape rather than resizing anything.
    #[must_use]
    pub fn cursor(self, dragging: bool) -> Option<CursorIcon> {
        match self {
            Self::Select => None,
            Self::Hand if dragging => Some(CursorIcon::Grabbing),
            Self::Hand => Some(CursorIcon::Grab),
            Self::Markup(_) => Some(CursorIcon::Crosshair),
        }
    }

    /// Which markup kind is armed, if any.
    ///
    /// The accessor `crate::app::PdfceApp::conditions` needs in order to render
    /// exactly one Markup button pressed, and the accessor
    /// [`crate::canvas::gesture::press_kind`] needs in order to decide what a
    /// press means. Both would otherwise write the same `if let` — which is how
    /// a canvas ends up drawing one shape while the ribbon says another.
    #[must_use]
    pub fn markup_kind(self) -> Option<MarkupKind> {
        match self {
            Self::Markup(kind) => Some(kind),
            _ => None,
        }
    }
}

/// **What the pointer looks like this frame** — the whole precedence, in one
/// pure function.
///
/// Lifted out of `canvas::interact` when the markup tool arrived, along the
/// same seam [`crate::canvas::gesture::press_kind`] was: the first rung of this
/// decision was already [`CanvasTool::cursor`], so the remaining three rungs
/// were the rest of one question living in the wiring, where they could not be
/// tested and where a fourth tool would have had to be remembered.
///
/// # The order is the rule
///
/// 1. **The armed tool**, when the pointer is over the canvas or a button is
///    down. This rung is the whole of *"the cursor must change, and must change
///    back"*: it changes because this branch is taken while the tool is active,
///    and it changes back because the answer is recomputed every frame from
///    [`active`] with nothing stored to restore. A dropped key-up costs one
///    frame of hand, not a canvas stuck showing a grab cursor over a select
///    tool.
/// 2. **A gesture in flight**, which keeps its own cursor even once the pointer
///    has wandered off the thing it started on — otherwise a drag that outruns
///    its object looks like it stopped working.
/// 3. **A hovered grip**, which is how the eight resize handles are findable at
///    all.
/// 4. **Nothing**, leaving the cursor to whatever else set it.
///
/// `pointer_down` is *any* button, because a middle-drag pan must show the
/// closed hand too; `over_canvas` is measured against the scroll viewport
/// rather than the page, because the hand pans the grey surround as readily as
/// the paper and a hand tool that shows no hand over half the canvas reads as a
/// tool that is not armed.
#[must_use]
pub fn cursor_for(
    tool: CanvasTool,
    gesture: Option<crate::canvas::gesture::DragKind>,
    hovered_grip: Option<crate::canvas::handles::Grip>,
    pointer_down: bool,
    over_canvas: bool,
) -> Option<CursorIcon> {
    use crate::canvas::gesture::DragKind;

    if let Some(icon) = tool
        .cursor(pointer_down)
        .filter(|_| over_canvas || pointer_down)
    {
        return Some(icon);
    }
    if let Some(kind) = gesture {
        return Some(match kind {
            // One crosshair for both marquee intents: the band is the same band
            // and `gesture`'s header refuses a second set of pixels for it. What
            // tells the operator a zoom is armed is the ribbon control that
            // armed it, off-canvas, where a mode indicator belongs. A markup
            // band answers the same way, and is stated rather than wildcarded
            // even though rung 1 already claimed it — a drag cannot be in flight
            // without the tool that started it, so this is unreachable today and
            // spelling it keeps the two answers one answer if that changes.
            DragKind::Marquee(_) | DragKind::Markup(_) => CursorIcon::Crosshair,
            DragKind::Move => CursorIcon::Grabbing,
            DragKind::Resize(grip) => grip.cursor(),
        });
    }
    hovered_grip.map(crate::canvas::handles::Grip::cursor)
}

/// Compose the chosen tool with the space bar — **the rule, and the only
/// place it exists**.
///
/// Space *borrows* the hand; it does not choose it. So this is a `max`, not a
/// swap: holding space over the hand tool changes nothing, and releasing it
/// returns whatever [`selected`] has said all along.
#[must_use]
pub fn resolve(selected: CanvasTool, space_held: bool) -> CanvasTool {
    if space_held {
        CanvasTool::Hand
    } else {
        selected
    }
}

/// The tool the operator chose — the persistent half, unaffected by the space
/// bar.
///
/// This is what a ribbon toggle or a tool palette should render as pressed:
/// showing the *active* tool there would make the button flicker under the
/// operator's thumb every time they held space.
#[must_use]
pub fn selected(ctx: &egui::Context) -> CanvasTool {
    let id = egui::Id::new(TOOL_MEMORY_KEY);
    ctx.data(|d| d.get_temp::<CanvasTool>(id).unwrap_or_default())
}

/// Choose a tool. **The entry point a `view.tool_hand` / `view.tool_select`
/// command calls.**
pub fn select(ctx: &egui::Context, tool: CanvasTool) {
    let id = egui::Id::new(TOOL_MEMORY_KEY);
    ctx.data_mut(|d| d.insert_temp(id, tool));
}

/// Flip between the hand and the select tool. **The entry point a single
/// `view.tool_hand` *toggle* command calls.**
///
/// Returns the tool now chosen, so a caller that wants to report or check the
/// new state does not have to ask again and risk reading a different frame's
/// answer.
pub fn toggle_hand(ctx: &egui::Context) -> CanvasTool {
    let next = match selected(ctx) {
        CanvasTool::Hand => CanvasTool::Select,
        // A markup tool is *left* by pressing Hand, not toggled through — the
        // operator asked for the hand, and returning them to Select would make
        // one press mean "put the pen down" and a second one mean "pick the
        // hand up".
        CanvasTool::Select | CanvasTool::Markup(_) => CanvasTool::Hand,
    };
    select(ctx, next);
    next
}

/// Arm the markup tool with `kind`, or retire it if that kind is already
/// armed. **The entry point every `markup.*` shape command calls.**
///
/// # ★ Why pressing the armed button again retires the tool
///
/// *"Make it work the way other programs do"* is the operator's stated
/// tie-breaker, and every drawing application treats a tool button as a toggle:
/// the button is **pressed**, so pressing it is how you un-press it. The
/// alternative — a button that only ever arms — leaves an operator who armed
/// Rectangle by mistake with no way back to the select tool except Escape,
/// which they have to know about, or arming some other tool, which is not what
/// they want either.
///
/// Choosing a *different* kind is not a toggle; it is a change of kind, and it
/// arms. So the rule is: same kind ⇒ retire, different kind ⇒ re-arm. That is
/// what makes the four Markup buttons behave as a radio you can switch off,
/// which is what they look like once each renders pressed.
///
/// Returns the tool now chosen, so a caller that wants to report or check the
/// new state does not have to ask again and risk reading a different frame's
/// answer — the same contract [`toggle_hand`] honours.
pub fn arm_markup(ctx: &egui::Context, kind: MarkupKind) -> CanvasTool {
    let next = if selected(ctx) == CanvasTool::Markup(kind) {
        CanvasTool::Select
    } else {
        CanvasTool::Markup(kind)
    };
    select(ctx, next);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI.
        //
        // The tool a canvas is armed with is otherwise invisible from outside:
        // a crosshair is a cursor, and a screenshot of an armed canvas and an
        // un-armed one are the same picture — which is defect 8's lesson
        // exactly. This line is how a harness proves the button armed anything.
        format!("markup-tool tool={next:?}")
    });
    next
}

/// Retire the markup tool, returning to [`CanvasTool::Select`], and report
/// whether there was one to retire.
///
/// **Escape's claimant.** Reports rather than being asked twice, for the same
/// reason `zoom::disarm_region_zoom` does: the caller cannot know whether the
/// key was spent here without asking, and a caller that re-derived it would be
/// the version that retires the tool *and* ascends a selection rung. See
/// [`crate::canvas::keys`]'s precedence table for where this sits and why.
///
/// Deliberately reads [`selected`] rather than [`active`]: a held space bar
/// borrows the hand, and Escape pressed mid-space must retire the markup tool
/// underneath it rather than doing nothing because the *active* tool happened
/// to be the hand at that instant.
pub fn disarm_markup(ctx: &egui::Context) -> bool {
    if selected(ctx).markup_kind().is_none() {
        return false;
    }
    select(ctx, CanvasTool::Select);
    true
}

/// Whether the space bar is down **and the canvas is entitled to it**.
///
/// See the module docs on the text-field guard.
#[must_use]
pub fn space_held(ctx: &egui::Context) -> bool {
    !ctx.text_edit_focused() && ctx.input(|i| i.key_down(Key::Space))
}

/// What the primary button means on this frame — [`resolve`] applied to the
/// live context.
///
/// The one call the canvas makes. Everything downstream branches on the
/// result and nothing downstream reads the space bar for itself.
#[must_use]
pub fn active(ctx: &egui::Context) -> CanvasTool {
    resolve(selected(ctx), space_held(ctx))
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Context, Event, Modifiers, RawInput};

    /// ★ **Space borrows the hand and gives it back** — the requirement,
    /// stated as the pure rule it is implemented as.
    ///
    /// The third case is the one that matters: releasing space returns to
    /// `Select`, and it does so without anything having been stored, so there
    /// is no restore step that can be skipped.
    #[test]
    fn space_borrows_the_hand_and_releasing_returns_the_previous_tool() {
        assert_eq!(resolve(CanvasTool::Select, false), CanvasTool::Select);
        assert_eq!(resolve(CanvasTool::Select, true), CanvasTool::Hand);
        assert_eq!(resolve(CanvasTool::Select, false), CanvasTool::Select);
    }

    /// Holding space while the hand tool is already chosen changes nothing,
    /// and releasing it does not drop the operator back into Select.
    #[test]
    fn space_over_the_hand_tool_is_a_no_op_in_both_directions() {
        assert_eq!(resolve(CanvasTool::Hand, true), CanvasTool::Hand);
        assert_eq!(resolve(CanvasTool::Hand, false), CanvasTool::Hand);
    }

    /// Only the hand pans, and each tool's cursor is what it should be — the
    /// two halves of the branch, asserted together so a future fourth tool
    /// cannot answer one and forget the other.
    ///
    /// The markup rows are the ones that matter now: a markup tool that
    /// answered `true` to `pans_with_primary` would be handed a blank pointer
    /// frame by `canvas::interact` and could never draw anything at all — a
    /// tool that arms, shows a crosshair and does nothing, which is the exact
    /// shape of an affordance that looks available and is inert.
    #[test]
    fn only_the_hand_pans_and_each_tool_paints_its_own_cursor() {
        assert!(!CanvasTool::Select.pans_with_primary());
        assert!(CanvasTool::Hand.pans_with_primary());
        assert_eq!(CanvasTool::Select.cursor(false), None);
        assert_eq!(CanvasTool::Select.cursor(true), None);
        assert_eq!(CanvasTool::Hand.cursor(false), Some(CursorIcon::Grab));
        assert_eq!(CanvasTool::Hand.cursor(true), Some(CursorIcon::Grabbing));
        for &kind in MarkupKind::ALL {
            let tool = CanvasTool::Markup(kind);
            assert!(!tool.pans_with_primary(), "{kind:?} must not pan");
            assert_eq!(tool.cursor(false), Some(CursorIcon::Crosshair), "{kind:?}");
            assert_eq!(tool.cursor(true), Some(CursorIcon::Crosshair), "{kind:?}");
            assert_eq!(tool.markup_kind(), Some(kind));
        }
        assert_eq!(CanvasTool::Select.markup_kind(), None);
        assert_eq!(CanvasTool::Hand.markup_kind(), None);
    }

    /// ★ **The cursor precedence**, all four rungs, in one test that would
    /// have caught each of them being reordered.
    ///
    /// This rule was four `if`s in the middle of `canvas::interact` and had no
    /// test at all — it needed a window to reach. Moving it here is what makes
    /// it assertable, and the rungs are asserted **against each other**: each
    /// case supplies a lower rung that would answer differently, so a build
    /// that consulted them in the wrong order fails rather than merely
    /// producing *a* cursor.
    #[test]
    fn the_cursor_precedence_runs_tool_then_gesture_then_grip() {
        use crate::canvas::gesture::{DragKind, MarqueeIntent};
        use crate::canvas::handles::Grip;

        // 1. The armed tool wins over a gesture AND a hovered grip.
        assert_eq!(
            cursor_for(
                CanvasTool::Markup(MarkupKind::Arrow),
                Some(DragKind::Move),
                Some(Grip::SouthEast),
                true,
                true,
            ),
            Some(CursorIcon::Crosshair),
        );
        assert_eq!(
            cursor_for(CanvasTool::Hand, Some(DragKind::Move), None, true, true),
            Some(CursorIcon::Grabbing),
        );
        // …but only while the pointer is over the canvas or a button is down,
        // so the hand does not claim the cursor over the ribbon.
        assert_eq!(cursor_for(CanvasTool::Hand, None, None, false, false), None);

        // 2. With the select tool, a gesture in flight wins over a grip the
        //    pointer happens to be over.
        assert_eq!(
            cursor_for(
                CanvasTool::Select,
                Some(DragKind::Marquee(MarqueeIntent::Select)),
                Some(Grip::SouthEast),
                true,
                true,
            ),
            Some(CursorIcon::Crosshair),
        );
        assert_eq!(
            cursor_for(
                CanvasTool::Select,
                Some(DragKind::Resize(Grip::NorthWest)),
                Some(Grip::SouthEast),
                true,
                true,
            ),
            Some(Grip::NorthWest.cursor()),
            "an in-flight resize keeps ITS grip's cursor, not the hovered one"
        );

        // 3. Then a hovered grip, and 4. then nothing.
        assert_eq!(
            cursor_for(CanvasTool::Select, None, Some(Grip::East), false, true),
            Some(Grip::East.cursor()),
        );
        assert_eq!(
            cursor_for(CanvasTool::Select, None, None, false, true),
            None
        );
    }

    /// ★ **Pressing an armed markup button again retires the tool; pressing a
    /// different one changes kind.**
    ///
    /// Both halves, because a build that only armed would pass a test of the
    /// first press alone — and the operator's complaint would be that the tool
    /// cannot be put down.
    #[test]
    fn arming_a_markup_kind_toggles_that_kind_and_switches_between_kinds() {
        let ctx = Context::default();
        assert_eq!(selected(&ctx), CanvasTool::Select);

        assert_eq!(
            arm_markup(&ctx, MarkupKind::Rectangle),
            CanvasTool::Markup(MarkupKind::Rectangle)
        );
        // A different kind re-arms rather than retiring.
        assert_eq!(
            arm_markup(&ctx, MarkupKind::Arrow),
            CanvasTool::Markup(MarkupKind::Arrow)
        );
        assert_eq!(selected(&ctx), CanvasTool::Markup(MarkupKind::Arrow));
        // The same kind again retires.
        assert_eq!(arm_markup(&ctx, MarkupKind::Arrow), CanvasTool::Select);
        assert_eq!(selected(&ctx), CanvasTool::Select);
    }

    /// ★ **Escape's claimant reports whether it took the key.**
    ///
    /// `false` with nothing armed is the load-bearing half: without it Escape
    /// would be consumed by a tool that was not armed, and the selection ladder
    /// would need two presses to leave a rung.
    #[test]
    fn disarming_markup_reports_whether_there_was_anything_to_disarm() {
        let ctx = Context::default();
        assert!(!disarm_markup(&ctx), "nothing armed: the key is not ours");

        arm_markup(&ctx, MarkupKind::Ellipse);
        assert!(disarm_markup(&ctx));
        assert_eq!(selected(&ctx), CanvasTool::Select);
        assert!(!disarm_markup(&ctx), "and it is not claimed twice");

        // The hand tool is not ours to retire either — Escape must not silently
        // put an operator who chose the hand back into Select.
        select(&ctx, CanvasTool::Hand);
        assert!(!disarm_markup(&ctx));
        assert_eq!(selected(&ctx), CanvasTool::Hand);
    }

    /// ★ **Space borrows the hand out of the markup tool and gives it back.**
    ///
    /// The property the whole "derived, never stored" design exists for,
    /// asserted for the new tool: an operator drawing a rectangle who holds
    /// space to reposition the page must get the rectangle tool back on
    /// release, with its kind intact.
    #[test]
    fn space_borrows_the_hand_out_of_the_markup_tool_and_returns_the_kind() {
        let armed = CanvasTool::Markup(MarkupKind::Rectangle);
        assert_eq!(resolve(armed, true), CanvasTool::Hand);
        assert_eq!(resolve(armed, false), armed);
    }

    /// The chosen tool survives a frame, and the toggle alternates rather
    /// than latching.
    #[test]
    fn the_chosen_tool_persists_and_the_toggle_alternates() {
        let ctx = Context::default();
        assert_eq!(selected(&ctx), CanvasTool::Select);
        assert_eq!(toggle_hand(&ctx), CanvasTool::Hand);
        assert_eq!(selected(&ctx), CanvasTool::Hand);
        assert_eq!(toggle_hand(&ctx), CanvasTool::Select);
        select(&ctx, CanvasTool::Hand);
        assert_eq!(selected(&ctx), CanvasTool::Hand);
    }

    /// ★ **A focused text field keeps the space bar**, so typing a page
    /// number into the status bar does not pan the drawing under the
    /// operator.
    ///
    /// Built against a real `TextEdit` for the same reason
    /// `canvas::tests::a_focused_text_field_keeps_delete_for_itself` is:
    /// `text_edit_focused()` resolves the focused id and looks for a
    /// `TextEditState` under it, so a hand-requested focus on a bare id would
    /// pass vacuously.
    #[test]
    fn a_focused_text_field_keeps_the_space_bar() {
        let ctx = Context::default();
        let mut buffer = String::from("37");

        // Frame 1: build the field and take focus.
        let _ = ctx.run_ui(RawInput::default(), |ui| {
            ui.add(egui::TextEdit::singleline(&mut buffer))
                .request_focus();
        });

        // Frame 2: the field holds focus and space is down.
        let input = RawInput {
            events: vec![Event::Key {
                key: Key::Space,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
            ..Default::default()
        };
        let mut typing = false;
        let mut held = true;
        let _ = ctx.run_ui(input, |ui| {
            ui.add(egui::TextEdit::singleline(&mut buffer));
            typing = ui.ctx().text_edit_focused();
            held = space_held(ui.ctx());
        });

        assert!(
            typing,
            "the test is vacuous unless a TEXT field really holds focus"
        );
        assert!(!held, "a focused text field must keep the space bar");
        assert_eq!(
            resolve(selected(&ctx), held),
            CanvasTool::Select,
            "and the tool must therefore not have changed"
        );
    }

    /// With no text field in the way, a held space bar really does reach the
    /// canvas — the other direction of the guard above, without which the
    /// previous test would pass on a build where space-pan never worked at
    /// all.
    #[test]
    fn a_held_space_bar_reaches_the_canvas_when_nothing_is_typing() {
        let ctx = Context::default();
        let input = RawInput {
            events: vec![Event::Key {
                key: Key::Space,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
            ..Default::default()
        };
        let mut tool = CanvasTool::Select;
        let _ = ctx.run_ui(input, |ui| {
            tool = active(ui.ctx());
        });
        assert_eq!(tool, CanvasTool::Hand);
    }
}
