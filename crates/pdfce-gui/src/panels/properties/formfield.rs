//! # `panels::properties::formfield` — the properties of a form field clicked
//! on the page
//!
//! **Operator request, 2026-08-26:** *"don't forget that when I click on an
//! existing form field on the page it's properties should come up in our side
//! pane for editing it's properties."* This is the side pane's half; the click
//! is `crate::canvas::forms`'s.
//!
//! ## ★★★ What can be changed, what can only be read, and why the difference
//! is disclosed rather than hidden
//!
//! `pdfce-core` has exactly four verbs for a field that already exists:
//! `rename_field`, `delete_field`, `delete_widget` and `fill_text_field`. It has
//! **none** for a field's flags — required, read-only, multiline, comb, the
//! border, the tooltip. Those are settable only at authoring time, through the
//! five `add_*` specs.
//!
//! So this panel offers rename, delete and delete-this-box, and **shows the
//! rest as read-only facts**. That is a real limitation and it is stated in the
//! panel rather than expressed as an absence, because an absence is
//! indistinguishable from an oversight:
//! [`crate::text::panels::formfield::not_editable_note`] says which properties
//! cannot be changed after placing and what to do instead.
//!
//! ★★ It is also written up as an engine request rather than worked around.
//! The standing rule is *report every workaround, even a successful one* —
//! anything the GUI has to work around is a place the crate boundary was drawn
//! wrong. A properties panel that can show a flag and not change it is exactly
//! that shape.
//!
//! ## ★★ Why this is not `SelectionState`, and why the panel says so
//!
//! `crate::app::state::SelectedField`'s doc carries the argument: a form field
//! is a document-level entry with a **name** for identity and possibly several
//! widgets on several pages, where everything `SelectionState` holds has
//! paint-order indices, a bounding box and drag handles. Merging them would arm
//! the Format tab's Delete over a field and hand the resize grips a rectangle
//! nothing can move.
//!
//! The visible consequence, which this panel is careful about: **a field is not
//! "selected" in the sense the rest of the shell means.** No handles appear, the
//! Format tab does not open, and Delete on the keyboard does not remove it. The
//! delete controls are *in this panel*, labelled, and there are two of them
//! because "remove this box" and "remove this field" are different requests.
//!
//! ## The rename box is a draft, not a live write
//!
//! Typing into a `TextEdit` bound straight to the field would rename on every
//! keystroke — `Address` would pass through `A`, `Ad`, `Add`, each of them a
//! real rename of a real field, each undoable separately, and any of them
//! capable of colliding with an existing name. So the box holds a draft in
//! `PanelsState` and a button commits it, which is the same shape
//! `super::geometry` uses for the same reason.

use crate::app::actions::forms::FieldAction;
use egui::Ui;

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::panels::PanelsState;
use crate::text::panels::formfield as t;

/// The section's rect, for `ui-verify`.
///
/// ★ A published region name is a cross-repo stability contract: the harness
/// asserts on it by string, so renaming one turns a check into a skip rather
/// than a failure.
const REGION: &str = "properties.form_field";

/// Draw the selected form field's properties, if one is selected.
///
/// Returns whether anything was drawn, so [`super::body`] can decide whether a
/// separator is wanted — the same protocol its three sibling sections use.
///
/// ## ★ Three early returns, and the middle one is the interesting one
///
/// 1. **Nothing selected** — the common case, and the panel says nothing at
///    all rather than "no field selected". R9: an unavailable capability
///    renders nothing.
/// 2. **A selection naming a field the document no longer has.** Reachable
///    despite every verb clearing the selection, because **undo and redo do
///    not**: an operator who deletes a field and presses Ctrl+Z has a document
///    whose form changed under a selection nobody touched. Rendering nothing is
///    right — the alternative is a panel describing a field that is not there.
/// 3. **No `/AcroForm` at all** — the same case one step earlier, after a
///    flatten or an undo past the form's creation.
pub fn section(
    ui: &mut Ui,
    doc: &OpenDoc,
    state: &mut PanelsState,
    actions: &mut Vec<Action>,
) -> bool {
    let Some(selected) = doc.selected_field.clone() else {
        return false;
    };
    let view = doc.session.view();
    let Some(form) = pdfce_core::forms::parse_acroform(&view) else {
        return false;
    };
    let Some(field) = form
        .fields
        .iter()
        .find(|f| f.fully_qualified_name == selected.field)
    else {
        return false;
    };

    let epoch = doc.edit_epoch;
    // No `.strong()` — R84 / DEFECTS.md D11: no theme this project ships
    // renders it legibly on a panel.
    ui.label(t::heading());
    ui.add_space(4.0);

    facts(ui, field, &selected);
    ui.add_space(6.0);
    ui.separator();
    ui.add_space(6.0);
    rename_row(ui, state, &selected, actions);
    ui.add_space(6.0);
    ui.separator();
    ui.add_space(6.0);
    // ★★★ The editable properties — `EditSession::edit_field`, consumed
    // 2026-08-27. Placed between the rename box and the delete buttons on
    // purpose: reading the pane top to bottom now goes "what this field IS",
    // then "what you can change about it", then "how to get rid of it", which
    // is the order of increasing commitment every other surface in this shell
    // uses. Delete was directly under rename before, which put the most
    // destructive control in the middle of the panel.
    //
    // `field.clone()` is deliberately NOT taken: the section reads the field
    // it is handed and raises actions, so the borrow ends with the frame.
    super::fieldedit::section(ui, field, &selected.field, state, epoch, actions);
    ui.add_space(6.0);
    // ★★ The WIDGET half, directly under the field half, in the engine's own
    // scope order: what belongs to the field, then what belongs to this one
    // box. `widget_scope_note` explains the distinction in the one state where
    // it is visible — a field drawn in more than one place.
    super::widgetedit::section(
        ui,
        field,
        &selected.field,
        selected.widget,
        state,
        epoch,
        actions,
    );
    ui.add_space(6.0);
    delete_row(ui, field, &selected, actions);
    ui.add_space(6.0);
    // ★★ What is left out of reach, and it is now the WIDGET half rather than
    // the field half. See `text::panels::formfield::not_editable_note` for the
    // sentence this replaced and why it was worse than a gap.
    ui.small(t::not_editable_note());
    ui.add_space(6.0);
    ui.separator();
    // ★★★ `ui.min_rect()`, at the END — and it was `ui.max_rect()` at the
    // START until 2026-08-27, which is a defect only driving could find.
    //
    // `max_rect` is the space a `Ui` is ALLOWED to use, not the space it took.
    // Published before anything is drawn, it reported
    // `[[786, 465] - [1086, 647]]` on the operator's own layout while this
    // section's own controls were at y = 735 — **a rect naming a different
    // panel entirely**, because the Properties dock's slot begins below the
    // Objects panel and `max_rect` had not been narrowed to it yet.
    //
    // Nothing failed. The region was declared, so every check asking *"did the
    // section draw?"* answered yes and was right. What broke was the second
    // thing a section rect is for: `ui-verify` scrolls **at** it, and a wheel
    // event aimed at that centre landed in the **Objects list** and scrolled
    // that instead — so a check hunting for controls below the fold scrolled
    // six times, moved nothing, and reported the controls missing.
    //
    // ⇒ **A region must name where the thing IS, not where it could have
    // been.** `min_rect` after drawing is the occupied space, which is the only
    // rect that is true of what an operator can see and point at.
    crate::diag::ui_rect(REGION, ui.min_rect());
    true
}

/// The read-only facts: what this field is, where it is, and what it holds.
fn facts(
    ui: &mut Ui,
    field: &pdfce_core::forms::Field,
    selected: &crate::app::state::SelectedField,
) {
    row(ui, &t::label_name(), &field.fully_qualified_name);
    row(ui, &t::label_type(), &t::field_type(field));
    // 1-based, because a page number in a UI is what the operator reads off the
    // page strip and every other surface in this shell states it that way.
    row(
        ui,
        &t::label_page(),
        &t::page_number(selected.page.saturating_add(1)),
    );
    // ★ Only when there is more than one, because "1 box" is noise on the
    // overwhelming majority of fields and the number is only interesting as a
    // warning: a field drawn in three places is one an operator can change from
    // three pages without realising.
    let widgets = field.widgets.len();
    if widgets > 1 {
        row(
            ui,
            &t::label_boxes(),
            &t::box_count(widgets, selected.widget),
        );
    }
    if let Some(value) = t::field_value(field) {
        row(ui, &t::label_value(), &value);
    }
    // The flags, as a single line naming only the ones that are set. A grid of
    // greyed checkboxes would look editable and is not, which is the one
    // reading this panel must not invite.
    if let Some(flags) = t::field_flags(field) {
        row(ui, &t::label_flags(), &flags);
    }
}

/// One label-and-value line.
///
/// ★ `truncate()` rather than wrapping, and the value on hover. A
/// fully-qualified name can run to any length, and a panel row that grew to
/// three lines would push everything under it around as the operator clicked
/// between fields — the same restlessness `disclosure_line` exists to prevent
/// in the status bar.
fn row(ui: &mut Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(egui::Label::new(value).truncate())
            .on_hover_text(value);
    });
}

/// The rename draft and its button.
fn rename_row(
    ui: &mut Ui,
    state: &mut PanelsState,
    selected: &crate::app::state::SelectedField,
    actions: &mut Vec<Action>,
) {
    ui.label(t::rename_label());
    // ★ The draft is seeded from the selection and re-seeded when the selection
    // changes, so clicking a second field does not leave the first field's name
    // sitting in the box waiting to be applied to the wrong one. That is the
    // failure this two-field state exists to prevent, and it is why the key is
    // stored beside the draft rather than inferred.
    let draft = state.field_rename_mut(&selected.field);
    let response = ui.add(
        egui::TextEdit::singleline(draft)
            .desired_width(f32::INFINITY)
            .char_limit(crate::canvas::formfield::draft::NAME_MAX),
    );
    // ★★ The partial name, NOT the qualified one. `rename_field` takes a
    // partial name and rebuilds the qualified one from the parent chain, so a
    // dotted string typed here would author a `/T` containing a dot — a field
    // no reader, including pdfce, can address again.
    let typed = draft.trim().to_owned();
    let ready = !typed.is_empty() && !typed.contains('.');
    let commit = ui.add_enabled(ready, egui::Button::new(t::rename_button()));
    let pressed = commit.clicked();
    if !ready {
        commit.on_disabled_hover_text(t::rename_disabled());
    }
    // Enter in the box commits, because a single-field form with a button
    // beside it is the one place an operator always tries Enter first.
    let entered = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
    if ready && (pressed || entered) {
        actions.push(
            FieldAction::Rename {
                from: selected.field.clone(),
                to: typed,
            }
            .into(),
        );
    }
}

/// The two delete controls.
///
/// ★★ Two, not one, and they are different requests. See
/// `Action::DeleteFormField`'s doc: one field may be drawn in several places,
/// so "remove this box" and "remove this field" have different consequences,
/// and offering only one of them makes the other impossible.
///
/// The per-box control renders **nothing** when the field has one widget, which
/// is R9 rather than greying: with one box the two buttons would do the same
/// thing, and a control that duplicates its neighbour is worse than absent.
fn delete_row(
    ui: &mut Ui,
    field: &pdfce_core::forms::Field,
    selected: &crate::app::state::SelectedField,
    actions: &mut Vec<Action>,
) {
    ui.horizontal(|ui| {
        if ui
            .button(t::delete_field())
            .on_hover_text(t::delete_field_hover(field.widgets.len()))
            .clicked()
        {
            actions.push(
                FieldAction::DeleteField {
                    field: selected.field.clone(),
                }
                .into(),
            );
        }
        if field.widgets.len() > 1
            && ui
                .button(t::delete_box())
                .on_hover_text(t::delete_box_hover())
                .clicked()
        {
            actions.push(
                FieldAction::DeleteWidget {
                    field: selected.field.clone(),
                    widget: selected.widget,
                }
                .into(),
            );
        }
    });
}
