//! The **Format** tab — contextual, appearing only while something is
//! selected.
//!
//! `RIBBON_IA.md` §5.8. One group: Selection.
//!
//! # What a contextual tab is, in this manifest
//!
//! It lives in `contextual_tabs` rather than `tabs`, and it carries a
//! `visible_when` condition — `"selection.any"` — that the application
//! publishes each frame in its `egui_shell::commands::ConditionSet`. The
//! separation is not cosmetic: a **mode** names a fixed tab set, and a
//! contextual tab's whole nature is that its presence is decided by
//! application state rather than by configuration. `egui-shell` refuses a
//! mode that names one, by design.
//!
//! It is therefore present in **all three modes** and in none of their tab
//! lists. Selecting a markup in Review mode shows the Format tab exactly
//! as selecting one in Edit mode does. That is correct: Review is the
//! stance in which you place and adjust *your own* markup, and a reviewer
//! who cannot recolour a cloud they just drew has been given half a tool.
//!
//! # Why this tab is nearly empty, and why it ships anyway
//!
//! §5.8 calls the contextual tab *"the single largest usability change
//! proposed here"* and then sets the build order:
//!
//! > Build order: **panel first, tab second.** The panel is the harder
//! > half and the tab's contents are a subset of it, so building the tab
//! > first would mean writing the property editors twice.
//!
//! Every property editor the tab is eventually made of — colour, fill,
//! line width, line style, opacity, arrowheads, note text, dimension
//! group, scale, precision, units, standard, witness lines, size,
//! position, crop, stroke, winding rule, node tools, font, spacing,
//! alignment — is therefore **N**, and under P3 absent. Twenty-four
//! entries in [`super::PLANNED`] come from this one section.
//!
//! What is left is the row that appears in *every* selection type's list
//! in §5.8's table, and that works today: **Delete**. An unarmed canvas
//! already does modeless select-and-delete — that is what the removal of
//! the `Editing on` master toggle relies on — so a Delete command on a
//! surface that only appears when something is selected is real, not a
//! stub.
//!
//! Shipping the tab with one command rather than deferring the tab
//! entirely is a deliberate choice and worth defending, because it looks
//! like exactly the placeholder P3 forbids and is not:
//!
//! - The tab **appears on selection**, which is itself the affordance
//!   §5.8 credits it with. That behaviour is the feature, and it is
//!   testable and demonstrable now.
//! - The command in it **does something**.
//! - The alternative — no contextual tab until the property editors land —
//!   means the appear-on-selection behaviour, the mode interaction and the
//!   one-command-one-tab consequences all get their first exercise at the
//!   same moment as twenty-four new controls.
//!
//! # The other two surfaces
//!
//! §5.8 is explicit that the contextual tab and a persistent **properties
//! panel** both ship, and that they answer different questions: the tab
//! carries what a user changes *while working*, the panel carries
//! everything including read-only facts and the editable X/Y/W/H geometry.
//! A **context menu** carries the same commands again for the user who
//! right-clicks — currently there is not one anywhere in the application.
//! Neither is a manifest concern at this stage; both are recorded here so
//! that "Format is nearly empty" is not read as "Format is all there will
//! be".

use super::{command, group};
use crate::text::ribbon;
use egui_shell::manifest::Tab;

/// The condition, published by the application each frame, under which the
/// Format tab appears.
///
/// Named rather than inlined because the same string is the enable
/// predicate of the command inside it: a tab that appears when something
/// is selected, holding a command that is available when something is
/// selected. Two spellings of one condition would be a defect that only
/// shows up as a tab containing one greyed control.
pub(super) const VISIBLE_WHEN: &str = "selection.any";

/// The Format tab.
pub(super) fn tab() -> Tab {
    Tab::new("format", ribbon::tab_format())
        .with_question(ribbon::question_format())
        .with_visible_when(VISIBLE_WHEN)
        .with_groups([group(
            "selection",
            ribbon::group_format_selection(),
            [command("format.delete")],
        )])
}
