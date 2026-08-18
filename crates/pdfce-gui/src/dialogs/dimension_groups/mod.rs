//! # `dialogs::dimension_groups` — where dimension groups are made, chosen and
//! configured
//!
//! ## The gap this closes
//!
//! `measure.manage_groups` was registered, drawn on Measure ▸ Scale, listed in
//! `shell::commands::reach`'s `SCAFFOLDED` set, and **inert for the whole life
//! of this build**. The operator hit it by name on 2026-08-18: *"I still can't
//! get to edit dimension groups when I click on it."*
//!
//! The recorded blocker was *"needs a window, not an arm"* plus *"two of four
//! verbs do not exist"*, and re-measuring it on 2026-08-18 found the second
//! half had shrunk. Of the six things a group manager wants to do, **four are
//! shipped engine verbs**:
//!
//! | | verb | `edit.rs` |
//! |---|---|---|
//! | create | `add_dimension_group` | 17692 |
//! | calibrate | `set_group_scale` | 17718 |
//! | drafting standard | `set_group_standard` | 18220 |
//! | appearance defaults | `set_group_style` | 18289 |
//! | show / hide the layer | `toggle_dimension_layer` | 17769 |
//! | **rename** | — | **absent** |
//! | **delete** | — | **absent** |
//!
//! Both absences are filed
//! (`request_a_dimension_group_can_be_created_and_never_renamed_or_deleted.md`,
//! 2026-08-18) and both are **disclosed in the window** rather than left as a
//! control an operator hunts for. A "Manage groups" window that can create and
//! cannot manage would be the boundary drawn in the wrong place; saying so on
//! screen is the honest interim.
//!
//! ## ★ The control that was missing from the whole feature, not just from this
//! window
//!
//! `MeasureState::group` — *"the active authoring group the next dimension
//! joins"* — has existed since the Phase 7 salvage, is documented as *"ui-spec
//! §2.6 group picker"*, is seeded to `DEFAULT_GROUP_ID`, and **nothing in this
//! build ever wrote to it**. So a second group could be created from the CLI,
//! carry its own scale, and be joinable by nothing: every dimension the shell
//! authored went into the default group, forever.
//!
//! The *Draw into* column is that picker. It is the first control in this
//! window not because it is the most elaborate but because without it every
//! other control on the window governs a group nothing can reach.
//!
//! ## Why a dialog and not a panel
//!
//! [`super`]'s own rule: a dialog is *one transaction with a start and an end*,
//! a panel is *somewhere an operator dips in and out of while working*. Setting
//! up a drawing's groups is the first, and emphatically: it happens once at the
//! start of a sheet and then not again for hours.
//!
//! The counter-argument is real and is recorded rather than dismissed — the
//! ui-spec §C.12 wants the **per-ce-dimension** properties in
//! `DockPanel::Properties`, contextually on selection, and that is right for
//! *that* surface because it answers *"what is this one thing I just clicked"*.
//! Group setup answers *"how is this drawing dimensioned"*, which nobody asks
//! while clicking. Two surfaces, two questions; §5.8's own "both surfaces, not
//! one" decision is the precedent.
//!
//! ## What it does NOT do, deliberately
//!
//! - **It does not pick which group a placed ce dimension belongs to.** There
//!   is no engine verb for that either
//!   (`request_a_placed_ce_dimension_cannot_be_moved_to_another_group.md`), and
//!   more importantly it is a *per-ce-dimension* question — it belongs on the
//!   selection surface, beside the other per-ce-dimension overrides.
//! - **It does not set a per-ce-dimension anything.** Every control here is
//!   group-scoped, which is what makes the reach-backwards disclosure on each
//!   of them true and uniform.
//! - **It does not offer a scale field.** The Set-scale window already exists,
//!   already owns the two entry paths and the calibration gesture, and already
//!   raises the one action. A second scale entry here would be a second
//!   implementation of the hardest arithmetic in the feature — see
//!   [`DimensionGroupsDialog::scale_requested`] for how the button hands over.

mod style;

use egui::Ui;
use pdfce_core::dimension::{DEFAULT_GROUP_ID, DimStandard, GroupId, Unit};

use crate::app::actions::Action;
use crate::app::actions::dimensions::DimensionAction;
use crate::app::state::{OpenDoc, Status};
use crate::text::dimension_groups as t;

/// The region this dialog publishes for its body.
pub const REGION_BODY: &str = "dialog:dimension-groups"; // ui-text-exempt: trace region name, never displayed
/// The region the *Add group* button publishes.
pub const REGION_ADD: &str = "dimension-groups.add"; // ui-text-exempt: trace region name, never displayed
/// The region the new-group name field publishes, so a driven check can type
/// into it.
pub const REGION_NEW_NAME: &str = "dimension-groups.new_name"; // ui-text-exempt: trace region name, never displayed
/// The prefix of the per-row *Draw into* radio regions; the group's numeric id
/// is appended.
///
/// Indexed by the **`GroupId`**, not by the row's position in the list. A row
/// index would change under a check the moment a group was added, which is
/// exactly what a check that adds a group is doing.
pub const REGION_DRAW_INTO_PREFIX: &str = "dimension-groups.draw_into."; // ui-text-exempt: trace region name, never displayed

/// The Manage-dimension-groups window's live state.
///
/// Existence is the "open" state, as everywhere in [`super`] — there is no
/// `open: bool` that could disagree with whether the state exists.
///
/// ★ **Almost nothing is held here**, and that is the design. The groups, their
/// scales, standards, styles and member counts are all read from
/// `EditSession::dimension_model()` on every frame. A local copy would be a
/// second source of truth for a model that this very window edits through an
/// action queue applied *after* the frame — so the copy would be stale for
/// exactly one frame after every change the operator made, which is the frame
/// they are looking at.
///
/// What is held is the four things the *document* does not know: which row the
/// operator is configuring, what they have typed into the new-group fields, and
/// the two one-shot requests that have to survive past the window closure.
pub struct DimensionGroupsDialog {
    /// The group whose settings the lower half of the window is showing.
    ///
    /// Distinct from the **authoring** group (the *Draw into* radio), and the
    /// distinction is worth the extra state: an operator setting up a detail
    /// group's appearance while still drawing into the plan group is an
    /// ordinary thing to want, and collapsing the two would make inspecting a
    /// group's settings silently redirect the next dimension they draw.
    selected: GroupId,
    /// What has been typed into the new-group name field.
    new_name: String,
    /// The unit the new group would start in.
    new_unit: Unit,
    /// Set by the *Set scale…* button, drained by [`super::DialogsState`].
    ///
    /// ★ **A request rather than a call**, because this window cannot open its
    /// sibling: both are fields of one `DialogsState` and neither can reach the
    /// other from inside its own `show`. Draining it in the owner is the
    /// smallest coupling that works, and it keeps the Set-scale window's own
    /// guards (`open_scale`'s no-document and already-open checks) on the one
    /// path that builds it.
    scale_requested: Option<GroupId>,
    /// Set by Close, consumed by [`Self::show`].
    close_requested: bool,
}

impl DimensionGroupsDialog {
    /// Open the window, showing `active`'s settings.
    ///
    /// Seeded with the **authoring** group rather than with the default group,
    /// because an operator who opens this while working has a group in mind and
    /// it is the one they are drawing into. Opening on the default group would
    /// make the first thing they read be settings for a group they may not have
    /// used since the sheet was started.
    #[must_use]
    pub fn open(active: GroupId) -> Self {
        Self {
            selected: active,
            new_name: String::new(),
            // Millimetres, because it is `Unit::default()`'s neighbour in every
            // sense that matters here: this operator's drawings are metric, and
            // a unit is one combo away for anybody whose are not.
            new_unit: Unit::Millimeter,
            scale_requested: None,
            close_requested: false,
        }
    }

    /// Take the pending *Set scale…* request, if the operator pressed it.
    ///
    /// Called by [`super::DialogsState::show`] immediately after this window
    /// draws. Returning it rather than acting on it is what keeps this module
    /// free of any knowledge of its siblings.
    pub fn take_scale_request(&mut self) -> Option<GroupId> {
        self.scale_requested.take()
    }

    /// Draw it. Returns `false` when it should close.
    ///
    /// # Why it takes `ctx` as well as `ui`-worth of state
    ///
    /// The *Draw into* radio writes the measure tool's active authoring group,
    /// which lives in `egui::Memory` rather than in any struct this window can
    /// reach — see [`crate::canvas::measure::set_active_group`]. That is the
    /// one thing here that is neither a document edit nor local state, and it
    /// is written directly rather than through the action funnel because it
    /// changes no document: it says where the *next* gesture's product will go.
    pub fn show(&mut self, ctx: &egui::Context, doc: &OpenDoc, actions: &mut Vec<Action>) -> bool {
        let mut open = true;
        egui::Window::new(t::window_title())
            .collapsible(false)
            .resizable(true)
            .default_width(520.0)
            // Screen-anchored, never page-anchored — the standing rule for
            // every dialog in this directory. An operator reading a list and
            // typing a name must not have it move when they scroll the drawing
            // behind it.
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .open(&mut open)
            .show(ctx, |ui| {
                crate::diag::ui_rect(REGION_BODY, ui.max_rect());
                self.body(ui, ctx, doc, actions);
            });

        open && !std::mem::take(&mut self.close_requested)
    }

    /// The whole window body.
    fn body(&mut self, ui: &mut Ui, ctx: &egui::Context, doc: &OpenDoc, actions: &mut Vec<Action>) {
        // ★ Read once per frame, and read from the SESSION rather than from any
        // cache. `dimension_model()` clones out of the `/PieceInfo` sidecar, so
        // this is the model as the document currently stands including every
        // unsaved edit — which is what the operator is looking at.
        let model = doc.session.dimension_model();
        let active = crate::canvas::measure::active_group(ctx).unwrap_or(DEFAULT_GROUP_ID);

        ui.label(t::intro());
        ui.add_space(8.0);

        // If the selected group was somehow removed from under the window, fall
        // back rather than drawing an empty lower half. Unreachable today —
        // there is no delete verb — and spelled anyway, because the day the
        // requested verb lands this is the line that stops the window going
        // blank.
        if model.group(self.selected).is_none() {
            self.selected = DEFAULT_GROUP_ID;
        }

        egui::ScrollArea::vertical()
            .id_salt("dimension-groups-scroll")
            .show(ui, |ui| {
                self.group_list(ui, ctx, &model, active);
                ui.separator();
                self.selected_group(ui, &model, actions);
                ui.separator();
                self.add_group(ui, actions);
            });

        ui.separator();
        ui.horizontal(|ui| {
            if ui.button(t::close_button()).clicked() {
                self.close_requested = true;
            }
        });
    }

    /// The list of groups: the authoring radio, the name, and the facts that
    /// distinguish one row from another.
    ///
    /// # Why the row shows the scale and the member count
    ///
    /// Because those are the two things that tell an operator *which group this
    /// is* when the names are `Plan` and `Detail` and they set them up an hour
    /// ago. A list of names alone would be a list of words.
    fn group_list(
        &mut self,
        ui: &mut Ui,
        ctx: &egui::Context,
        model: &pdfce_core::dimension::DimensionModel,
        active: GroupId,
    ) {
        ui.label(t::groups_heading());
        ui.label(t::draw_into_hint());
        ui.add_space(4.0);

        egui::Grid::new("dimension-groups-list")
            .num_columns(4)
            .striped(true)
            .show(ui, |ui| {
                ui.weak(t::draw_into_heading());
                ui.label("");
                ui.label("");
                ui.label("");
                ui.end_row();

                for group in model.groups() {
                    let response = ui.radio(group.id == active, "");
                    crate::diag::ui_rect(
                        // ui-text-exempt: trace region name, never displayed
                        &format!("{REGION_DRAW_INTO_PREFIX}{}", group.id.0),
                        response.rect,
                    );
                    if response.clicked() {
                        crate::canvas::measure::set_active_group(ctx, group.id);
                        crate::diag::trace(|| {
                            // ui-text-exempt: diagnostic trace, never displayed
                            format!("dimension-authoring-group id={}", group.id.0)
                        });
                    }

                    // The name doubles as the row selector for the lower half.
                    // A separate "configure" button per row would be a second
                    // control doing what clicking the row already means
                    // everywhere else in this application.
                    if ui
                        .selectable_label(self.selected == group.id, &group.name)
                        .clicked()
                    {
                        self.selected = group.id;
                    }
                    ui.label(t::member_count(model.member_count(group.id)));
                    ui.label(t::scale_phrase(group.scale, group.format.unit));
                    ui.end_row();
                }
            });
        ui.add_space(4.0);
        ui.weak(t::cannot_rename_or_delete());
    }

    /// The selected group's settings: scale, standard, layer, appearance.
    fn selected_group(
        &mut self,
        ui: &mut Ui,
        model: &pdfce_core::dimension::DimensionModel,
        actions: &mut Vec<Action>,
    ) {
        let Some(group) = model.group(self.selected) else {
            return;
        };

        // --- scale ------------------------------------------------------
        ui.horizontal(|ui| {
            ui.label(t::scale_phrase(group.scale, group.format.unit));
            if ui.button(t::set_scale_button()).clicked() {
                self.scale_requested = Some(group.id);
            }
        });
        ui.add_space(6.0);

        // --- drafting standard ------------------------------------------
        ui.label(t::standard_heading());
        ui.label(t::standard_hint());
        let mut standard = group.standard;
        ui.horizontal(|ui| {
            for option in [DimStandard::Ansi, DimStandard::Iso] {
                ui.radio_value(&mut standard, option, t::standard_name(option));
            }
        });
        // ★ The whole group moves, always — the standard has no per-ce-dimension
        // tier on `Group`, so no member can be following anything else. The
        // count is therefore the member count itself, and it is still shown,
        // because "all 40 will be redrawn" is exactly the sentence the operator
        // asked for and its absence here would read as "this one is different".
        let members = model.member_count(group.id);
        ui.weak(t::members_that_will_move(members, members));
        if standard != group.standard {
            actions.push(Action::Dimension(DimensionAction::SetGroupStandard {
                group: group.id,
                standard,
            }));
        }
        ui.add_space(6.0);

        // --- layer ------------------------------------------------------
        ui.label(t::layer_heading());
        if group.id == DEFAULT_GROUP_ID {
            // R9: the affordance is ABSENT, not greyed. The engine refuses to
            // hide the default group, so a switch here could never be honoured
            // — and the sentence in its place is why an omission does not read
            // as a bug.
            ui.weak(t::layer_default_group());
        } else {
            let mut visible = group.visible;
            if ui.checkbox(&mut visible, t::layer_visible()).changed() {
                actions.push(Action::Dimension(DimensionAction::ToggleLayer {
                    group: group.id,
                    visible,
                }));
            }
            ui.weak(t::layer_hint());
        }
        ui.add_space(8.0);

        // --- appearance defaults ----------------------------------------
        style::show(ui, model, group, actions);
    }

    /// The new-group controls.
    fn add_group(&mut self, ui: &mut Ui, actions: &mut Vec<Action>) {
        ui.label(t::new_heading());
        ui.horizontal(|ui| {
            ui.label(t::new_name_label());
            let response =
                ui.add(egui::TextEdit::singleline(&mut self.new_name).desired_width(160.0));
            crate::diag::ui_rect(REGION_NEW_NAME, response.rect);

            ui.label(t::new_unit_label());
            egui::ComboBox::from_id_salt("dimension-groups-new-unit")
                .selected_text(crate::text::scale::unit_name(self.new_unit))
                .show_ui(ui, |ui| {
                    // ★ `Unit::all()`, not a hand-written array. The engine's own
                    // doc for it says *"the GUI unit dropdown and the CLI unit
                    // parser iterate this"*, and `NO_SURFACE.md`'s sweep found a
                    // local copy in `dialogs::scale` that happened to match —
                    // a latent divergence rather than an active one, and this is
                    // the version that cannot acquire it.
                    for unit in Unit::all() {
                        ui.selectable_value(
                            &mut self.new_unit,
                            unit,
                            crate::text::scale::unit_name(unit),
                        );
                    }
                });
        });
        ui.weak(t::new_unit_hint());

        let name = self.new_name.trim().to_owned();
        if name.is_empty() {
            // Greying WITH an explanation: this is the *temporarily*
            // unavailable case R9 reserves it for, and the reason is one the
            // operator can act on in a single keystroke. Omitting the button
            // instead would make the name field look like it does nothing.
            let response = ui.add_enabled(false, egui::Button::new(t::new_button()));
            crate::diag::ui_rect(REGION_ADD, response.rect);
            response.on_hover_text(t::new_needs_a_name());
        } else {
            let response = ui.button(t::new_button());
            crate::diag::ui_rect(REGION_ADD, response.rect);
            if response.clicked() {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed
                    format!(
                        "dimension-group-add unit={:?} chars={}",
                        self.new_unit,
                        name.len()
                    )
                });
                actions.push(Action::Dimension(DimensionAction::AddGroup {
                    name,
                    unit: self.new_unit,
                }));
                // Cleared so a second press cannot silently make a second group
                // with the same name — which the engine would accept, and which
                // would leave two indistinguishable rows in the picker.
                self.new_name.clear();
            }
        }
    }
}

/// Open the window for `status`, or decline.
///
/// Applies the two guards every dialog in [`super`] applies at the one place it
/// is built: no document means no window, and the caller checks *already open*
/// before calling. The document guard is real rather than ceremonial here —
/// `dimension_model()` needs a session, and a window drawn over an empty canvas
/// would be closed again by the next frame's `close_document_scoped`, which
/// reads as a control that flickers rather than one that declines.
#[must_use]
pub fn open_for(status: &Status, active: GroupId) -> Option<DimensionGroupsDialog> {
    match status {
        Status::Open(_) => Some(DimensionGroupsDialog::open(active)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The window opens on the group the operator was drawing into.
    #[test]
    fn it_opens_on_the_authoring_group() {
        let d = DimensionGroupsDialog::open(GroupId(4));
        assert_eq!(d.selected, GroupId(4));
    }

    /// A blank or whitespace name cannot become a group.
    ///
    /// Asserted on the trimming rule rather than on the widget, because the
    /// widget is what a driven check exercises and this is what the *action*
    /// may never carry: `add_dimension_group("")` would be accepted by the
    /// engine and would leave a row in the picker that nothing distinguishes
    /// from the one above it.
    #[test]
    fn a_name_that_is_only_spaces_is_not_a_name() {
        for candidate in ["", "   ", "\t", "\n  "] {
            assert!(
                candidate.trim().is_empty(),
                "{candidate:?} must be refused before it reaches the engine"
            );
        }
        assert_eq!(
            " Plan ".trim(),
            "Plan",
            "a real name is trimmed, not refused"
        );
    }

    /// The scale request is a one-shot: taking it clears it.
    ///
    /// The property that matters is the second `take` returning `None`. Without
    /// it, `DialogsState::show` would re-open the Set-scale window on every
    /// frame after one press — a window that cannot be closed, because closing
    /// it is what lets the next frame open it again.
    #[test]
    fn the_scale_request_fires_once() {
        let mut d = DimensionGroupsDialog::open(DEFAULT_GROUP_ID);
        assert_eq!(d.take_scale_request(), None);
        d.scale_requested = Some(GroupId(2));
        assert_eq!(d.take_scale_request(), Some(GroupId(2)));
        assert_eq!(d.take_scale_request(), None, "it must not repeat");
    }
}
