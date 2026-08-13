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

/// `egui::Memory` key for the operator's chosen pointer tool.
const TOOL_MEMORY_KEY: &str = "pdfce-canvas-tool"; // ui-text-exempt: internal memory id, never displayed

/// What the primary button does over the page.
///
/// Deliberately two variants and not a general "tool" enum with markup,
/// measure and text members. Those are *modes* that arm a whole authoring
/// surface and they will arrive with their own state; this enum answers the
/// narrow navigation question — **does a primary drag select, or does it move
/// the paper?** — which is the only question the pan and marquee paths need
/// settled, and settling it here keeps them from inventing two different
/// answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CanvasTool {
    /// Click selects, drag rubber-bands. The shipped behaviour, and the
    /// default.
    #[default]
    Select,
    /// Click does nothing, drag moves the paper under the viewport.
    Hand,
}

impl CanvasTool {
    /// Whether a primary-button drag pans the view rather than reaching the
    /// gesture machine.
    ///
    /// The whole branch, in one predicate, so the pan path and the
    /// gesture-suppression path cannot disagree about which tool pans — a
    /// disagreement whose symptom would be a drag that pans **and** marquees,
    /// which is one of the two things this stage must not ship.
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
    #[must_use]
    pub fn cursor(self, dragging: bool) -> Option<CursorIcon> {
        match self {
            Self::Select => None,
            Self::Hand if dragging => Some(CursorIcon::Grabbing),
            Self::Hand => Some(CursorIcon::Grab),
        }
    }
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
        CanvasTool::Select => CanvasTool::Hand,
    };
    select(ctx, next);
    next
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

    /// Only the hand pans, and only the hand shows a cursor — the two halves
    /// of the branch, asserted together so a future third tool cannot answer
    /// one and forget the other.
    #[test]
    fn only_the_hand_pans_and_only_the_hand_paints_a_cursor() {
        assert!(!CanvasTool::Select.pans_with_primary());
        assert!(CanvasTool::Hand.pans_with_primary());
        assert_eq!(CanvasTool::Select.cursor(false), None);
        assert_eq!(CanvasTool::Select.cursor(true), None);
        assert_eq!(CanvasTool::Hand.cursor(false), Some(CursorIcon::Grab));
        assert_eq!(CanvasTool::Hand.cursor(true), Some(CursorIcon::Grabbing));
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
