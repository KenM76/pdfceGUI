//! # `panels::tool::armed` — what is armed, how far through the gesture is,
//! and how to put it down
//!
//! The frame every armed tool shares, and the per-family bodies inside it.
//!
//! ## ★ The frame, and why its order is not interchangeable
//!
//! | # | row | kind | present |
//! |---|---|---|---|
//! | 1 | **Identity** — the tool's name, its chord, its ribbon home | STATUS | always |
//! | 2 | **Stage** — one fixed slot: the instruction before a gesture, the live stage during one | STATUS | always |
//! | 3 | **Options** | OPTIONS | per tool; several families have none |
//! | 4 | **Put the tool down** | verb | always |
//!
//! **Identity first** because it is the literal complaint — *"no side bar area
//! showing what tool is active"* — and because it is the only row true in every
//! state of every tool.
//!
//! **Stage second, and it is the volatile row.** A row that changes must sit
//! where its change cannot move anything the operator is aiming at. Put an
//! option control above it and every option shifts vertically each time a
//! vertex lands — a control that moves is a control you cannot aim at, which is
//! this project's layout defect wearing yet another set of clothes.
//!
//! **The stage is one fixed slot that is never empty and is never a
//! placeholder.** It holds the instruction when idle and the stage when live —
//! same slot, same height, no *"nothing in progress"* line. That single choice
//! is what makes the armed frame R9-clean without a single greyed control.
//!
//! ## ★★ The identity row reads the REGISTRY, never a string of its own
//!
//! `MenuHost::label` and `MenuHost::chord`. A second copy of a label compiles,
//! reads identically the day it is written, and drifts the first time either is
//! reworded — invisibly, because nothing renders both at once. `NO_SURFACE.md`
//! §1 records that failure with a colour and, worse, records the test that
//! failed to catch it: it *"asserted the literal triple against a function
//! returning the literal triple. Two copies of one constant cannot disagree."*
//!
//! The chord comes from the operator's own keymap for the same reason. A panel
//! that hard-coded `Ctrl+E` would be telling somebody to press a key their
//! manifest may not bind — and a chord that does not work reads as the
//! *feature* not working, which is exactly the report that produced this panel.
//!
//! ## What is deliberately absent from the armed frame
//!
//! - **The markup pen's colour and width.** They belong here. `Panel::show`
//!   cannot reach the pen — see [`super`]'s closing section — and a swatch that
//!   accepts a click and discards it is the control `panels::properties`
//!   refused to ship.
//! - **Any sibling kind.** Arming Rectangle shows Rectangle. An "or try
//!   Ellipse" row is the second ribbon.
//! - **Anything about a PLACED annotation.** This panel is about the *next*
//!   gesture; `panels::properties` and the Format tab are about the placed
//!   thing, and `RIBBON_IA.md` §5.5 draws that line explicitly.

use egui::Ui;

use crate::canvas::tool::CanvasTool;
use crate::shell::menus::MenuHost;
use crate::text::tool as t;

/// Draw the armed block.
pub(super) fn block(
    ui: &mut Ui,
    ctx: &egui::Context,
    tool: CanvasTool,
    host: Option<&MenuHost<'_>>,
) {
    ui.label(t::armed_heading());
    crate::diag::ui_rect(super::REGION_ARMED, ui.min_rect());

    identity(ui, tool, host);
    ui.add_space(4.0);
    stage(ui, ctx, tool);
    ui.add_space(6.0);
    put_down(ui, ctx);
}

/// Row 1 — what is armed, and where it came from.
///
/// Falls back to nothing at all when the command is not registered or there is
/// no validated manifest, for [`super::idle`]'s reason: a name with no control
/// behind it is worse than no name.
fn identity(ui: &mut Ui, tool: CanvasTool, host: Option<&MenuHost<'_>>) {
    let Some(id) = command_for(tool) else {
        return;
    };
    let Some(host) = host else {
        return;
    };
    let Some(label) = host.label(id) else {
        return;
    };
    ui.label(label);
    let tab = tab_for(tool);
    ui.label(
        egui::RichText::new(t::row_home(tab, host.chord(id).as_deref()))
            .small()
            .weak(),
    );
}

/// Row 2 — the fixed slot: the instruction, or the stage of a live gesture.
///
/// # ★ One slot, two contents, never absent
///
/// The whole of this panel's stability. An operator's eye settles on this line
/// and the controls below it must not move when the line's *content* changes —
/// so the instruction and the live stage share it rather than the stage
/// appearing beneath the instruction.
fn stage(ui: &mut Ui, ctx: &egui::Context, tool: CanvasTool) {
    let text = match tool {
        CanvasTool::Select => return,
        CanvasTool::Hand => {
            ui.label(egui::RichText::new(t::hand_instruction()).small());
            ui.label(egui::RichText::new(t::hand_borrow()).small().weak());
            return;
        }
        CanvasTool::Text => {
            ui.label(egui::RichText::new(t::text_select_instruction()).small());
            // ★ Rendered only where it is TRUE. In Read and Review the select
            // tool already swept text, so arming this takes nothing away and
            // the sentence would be describing a change that did not happen.
            // Absent rather than reworded — R9 applied to a sentence.
            if crate::canvas::tool::capabilities(ctx).edit_content {
                ui.label(
                    egui::RichText::new(t::text_select_takes_the_press())
                        .small()
                        .weak(),
                );
            }
            return;
        }
        CanvasTool::Markup(kind) => {
            // The live count for a run of clicks, the instruction otherwise.
            // `vertex::read` answers `None` when nothing is in progress, which
            // is the idle case and gets the instruction — one slot, two
            // contents.
            match crate::canvas::markup::vertex::read(ctx) {
                Some(run) if run.kind == kind && run.in_progress() => {
                    t::vertices_placed(run.vertices.len())
                }
                _ => t::markup_instruction(kind).to_owned(),
            }
        }
        CanvasTool::TextAnnot(kind) => {
            ui.label(egui::RichText::new(t::text_annot_instruction(kind)).small());
            // ★★ The sentence that stops a working tool reading as broken.
            //
            // `CanvasTool` was split for exactly this: *"A markup band authors
            // on release, from geometry alone. These cannot: releasing produces
            // an empty box, and an empty box is not an annotation."* An
            // operator who drags one of these out, lets go, and sees nothing
            // land has met a release that authored nothing — which is the same
            // failure shape as the text-editing complaint that produced this
            // whole panel.
            ui.label(egui::RichText::new(t::text_annot_release()).small().weak());
            return;
        }
        CanvasTool::TextEdit(kind) => {
            // Live when there is a caret, the instruction before there is one.
            match crate::canvas::textedit::read(ctx) {
                Some(draft) if draft.kind == kind => t::text_edit_live().to_owned(),
                _ => t::text_edit_instruction(kind).to_owned(),
            }
        }
        CanvasTool::Measure(kind) => t::measure_instruction(kind).to_owned(),
    };
    ui.label(egui::RichText::new(text).small());
}

/// Row 4 — put the tool down.
///
/// # ★ It is NOT a Close button, and the distinction is one an operator can
/// lose a panel to
///
/// [`super::dimension_groups`]' rule stands: a panel has no Close button,
/// because the dock tab carries one. This is a different verb — it retires the
/// **tool** and leaves the panel exactly where it was — and it says so in its
/// own label rather than relying on position. Two controls a click apart that
/// both read as *closing something* is how somebody shuts a surface they
/// wanted.
///
/// # ★ It writes the armed tool DIRECTLY, and that is the house idiom rather
/// than an exception
///
/// The armed tool is not document state. It lives in `egui::Memory` beside the
/// gesture machine, `canvas::tool`'s own header argues why, and **every other
/// retirement path in the crate writes it the same way**: `disarm_markup`,
/// `disarm_measure` and `retire_forbidden` are all one `select(ctx,
/// CanvasTool::Select)`. The Dimension-groups panel writes the measure tool's
/// authoring group from a panel body on the identical argument — *"it changes
/// no document; it says where the next gesture's product will go."*
///
/// So this is not the funnel being bypassed. `crate::app::actions`' invariant
/// is about **document** state, and there is none here: putting a tool down
/// contributes nothing to the undo log and has nothing to order against.
/// Routing it through an `Action` would add a variant `apply` could only answer
/// by writing the same memory slot, which is the funnel pointing the wrong way
/// — the argument `crate::dialogs`' header makes about printing.
///
/// Returns to `Select` rather than to `Hand`, matching every other retirement
/// path: `Select` is the enum's `#[default]`, and a control that silently
/// swapped in a *different* tool would be a second surprise on top of the one
/// the operator asked for.
fn put_down(ui: &mut Ui, ctx: &egui::Context) {
    let response = ui.button(t::put_down_button());
    // ★ The hint names the key, and the key is `Escape` — which this build
    // handles in `canvas::keys` rather than through the manifest keymap, so
    // `MenuHost::chord` would answer `None` for it. Written here rather than
    // derived, and that is a deliberate exception to this panel's
    // read-the-registry rule with a narrow justification: Escape is not a
    // *binding*, it is a rung on a ladder (`canvas::keys`), and a keymap
    // lookup for it would be asking the wrong question rather than getting an
    // unlucky answer.
    let response = response.on_hover_text(t::put_down_hint());
    if response.clicked() {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            "tool-panel-put-down".to_owned()
        });
        crate::canvas::tool::select(ctx, CanvasTool::Select);
    }
}

/// The command that arms `tool`, if one does.
///
/// # ★ Derived from the existing id maps, never written a second time
///
/// `shell::commands::markup_command` and `measure_for_command`'s inverse are
/// the single binding between an id and a kind, exactly as
/// `Panel::from_command_id` is for panels. Re-listing them here would be a
/// second table to keep in step, and the failure when it drifted would be an
/// identity row naming the wrong tool — which is the one thing this panel
/// exists to get right.
fn command_for(tool: CanvasTool) -> Option<&'static str> {
    match tool {
        // ui-text-exempt: command ids, never displayed
        CanvasTool::Select => None,
        CanvasTool::Hand => Some("view.tool_hand"),
        CanvasTool::Text => Some("view.tool_text"),
        CanvasTool::Markup(kind) => Some(crate::shell::commands::markup_command(kind)),
        // ★ The empty string is `MeasureKind::Scale`'s id, and it is not a
        // command — that kind is armed from inside the Set-scale window and
        // deliberately maps to nothing. `MenuHost::label` would answer `None`
        // for it anyway, but filtering here says the reason: there is no
        // ribbon control to name, so the identity row is absent rather than
        // blank.
        CanvasTool::Measure(kind) => {
            let id = crate::shell::commands::measure_command(kind);
            (!id.is_empty()).then_some(id)
        }
        // ★ `TextAnnotKind::command` rather than a table here. It is the
        // single binding between one of these kinds and its id — the same
        // shape `markup_command` and `measure_command` have — and
        // `TextAnnotKind::from_command` is its inverse, which is what the
        // dispatcher uses. A third spelling in this file would be the second
        // table this function's own doc comment refuses.
        CanvasTool::TextAnnot(kind) => Some(kind.command()),
        CanvasTool::TextEdit(crate::canvas::textedit::TextEditKind::Edit) => Some("edit.text"),
        CanvasTool::TextEdit(crate::canvas::textedit::TextEditKind::Add) => Some("edit.add_text"),
    }
}

/// The ribbon tab `tool`'s command lives on.
///
/// The half of the identity row that teaches the ribbon. It is a `match` on the
/// tool rather than a lookup from the manifest because a manifest lookup would
/// answer *"which tab declares this item"* — which is the same answer today and
/// would silently follow a customized manifest that moved the control, telling
/// an operator to look somewhere their ribbon does not have. This says where
/// `RIBBON_IA.md` puts it, which is the promise the built-in shell keeps.
fn tab_for(tool: CanvasTool) -> &'static str {
    match tool {
        CanvasTool::Select | CanvasTool::Hand | CanvasTool::Text => crate::text::ribbon::tab_view(),
        CanvasTool::Markup(_) | CanvasTool::TextAnnot(_) => crate::text::ribbon::tab_markup(),
        CanvasTool::Measure(_) => crate::text::ribbon::tab_measure(),
        CanvasTool::TextEdit(_) => crate::text::ribbon::tab_edit(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::markup::MarkupKind;
    use crate::canvas::textedit::TextEditKind;

    /// ★ Every armable tool names a command, and `Select` names none.
    ///
    /// `Select` is the unarmed state — [`super`]'s body treats it as such and
    /// never draws this block for it — so a command here would be a claim that
    /// there is a control called "Select", which there is not.
    #[test]
    fn every_armed_tool_names_its_command_and_select_names_none() {
        assert_eq!(command_for(CanvasTool::Select), None);
        for tool in [
            CanvasTool::Hand,
            CanvasTool::Text,
            CanvasTool::Markup(MarkupKind::Rectangle),
            CanvasTool::Markup(MarkupKind::Cloud),
            CanvasTool::TextEdit(TextEditKind::Edit),
            CanvasTool::TextEdit(TextEditKind::Add),
        ] {
            assert!(
                command_for(tool).is_some(),
                "{tool:?} is armable and names no command, so the identity row would be \
                 blank for a tool the operator has in their hand"
            );
        }
    }

    /// The two text tools name **different** commands.
    ///
    /// The pair the operator confuses. An identity row that named the same
    /// command for both would make the panel unable to tell him which one he
    /// had armed — which is the state he is already in, and the state this
    /// panel exists to end.
    #[test]
    fn the_two_text_tools_are_told_apart() {
        assert_ne!(
            command_for(CanvasTool::TextEdit(TextEditKind::Edit)),
            command_for(CanvasTool::TextEdit(TextEditKind::Add))
        );
    }

    /// Every markup kind's command is the one `shell::commands` owns.
    ///
    /// Asserted as a **relation** to the id map rather than against literals,
    /// which is the whole point: two copies of one constant cannot disagree,
    /// so a test written against literals would pass on a build where this
    /// module had its own stale table.
    #[test]
    fn the_markup_identity_reads_the_id_map() {
        for kind in MarkupKind::ALL.iter().copied() {
            assert_eq!(
                command_for(CanvasTool::Markup(kind)),
                Some(crate::shell::commands::markup_command(kind))
            );
        }
    }
}
