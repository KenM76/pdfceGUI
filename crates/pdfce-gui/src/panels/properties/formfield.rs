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
//! ## ★★★ Rename and Delete are OFFERED ONLY WHERE THEY WOULD WORK (R83)
//!
//! Added 2026-08-28, closing a gap an audit of `EditSession`'s public surface
//! found: **nothing in this shell consulted `deletion_refusal`**, a pure query
//! that has existed for the whole life of the crate and whose own doctest
//! spells out this call site. It appeared here only inside comments in
//! `crate::panels::forms`, arguing about which query *Flatten* should ask, while
//! Delete asked none.
//!
//! The consequence, on the ordinary real-world certified fillable form: three
//! live controls — Rename, Delete field, Delete this box — every press of which
//! returned a refusal to the trace and nothing at all to the operator. That is
//! the failure this project is named after in miniature: a visible control that
//! is silently inert.
//!
//! Both gates are now asked in [`section`], **before anything is drawn**, and
//! each control asks **its own question**: `rename_refusal` for the rename box,
//! `deletion_refusal` for the two delete buttons. They compute the same answer
//! today and are deliberately separate functions on core's side; that
//! reasoning, and why borrowing one for the other is a silent-failure waiting
//! on a spec nuance, is quoted in full at the call site.
//!
//! Where a gate refuses, the controls **are not drawn at all** and a sentence
//! takes their place. R9: greying is for the temporarily unavailable and must
//! explain itself on hover; a certification signature is neither temporary nor
//! arguable. And a sentence rather than a silence, because a panel that simply
//! omits half its controls looks half-drawn.
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
/// The **Rename** control's rect, published only when the control is drawn.
///
/// ★★★ "Only when drawn" is the whole value of it. On a document that refuses a
/// rename this section draws a sentence and no control at all (R9), so the
/// region's *absence* is the evidence a driven check reads — and it is
/// admissible evidence only because [`TRACE_GATES`] is written on every frame
/// either way, so a check can tell "the control was withheld" from "the section
/// never drew". `crate::checks`' rule 4 in the harness states the same
/// obligation from the other side: never treat an absence as evidence unless
/// you have shown the thing that would have produced it was working.
const REGION_RENAME: &str = "properties.form_field.rename";
/// The **Delete field** control's rect, published only when the control is
/// drawn. See [`REGION_RENAME`].
const REGION_DELETE: &str = "properties.form_field.delete";
/// The per-frame census of what the two structural gates answered.
///
/// Written whether or not either control is drawn, which is what makes the
/// regions above readable as evidence rather than as noise.
const TRACE_GATES: &str = "form-field-gates";

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

    // ★★★ R83 — ASKED HERE, ONCE, BEFORE EITHER CONTROL IS DRAWN, AND EACH
    // CONTROL ASKS ITS OWN QUESTION.
    //
    // Both are **pure queries**: they read the signature census and the trailer
    // and mutate nothing, so they are safe to call every frame from a UI, and
    // core says so in as many words.
    //
    // # Why two calls and not one, when the two answers are identical today
    //
    // They are. `rename_refusal` and `deletion_refusal` both delegate to
    // `structural_form_refusal`, and core's doc comment says outright that a
    // shell *"could call that one and be correct"* — and then says it should
    // not, in terms this file is the exact instance of:
    //
    // > the two gates *happen* to be computable together and are answers to
    // > different questions, and a call site that asks the wrong question is
    // > correct only until the answers diverge — at which point it is wrong
    // > silently, in a control that stays enabled while its verb refuses.
    //
    // > A GUI disabling a Rename button through a method named
    // > `deletion_refusal` is that hazard with the name spelled out at the call
    // > site.
    //
    // The coupling is explicit on core's side precisely so that if a future
    // spec nuance separates renaming from deletion, the split happens THERE,
    // once, and every caller keeps asking its own question. Two lines here is
    // the entire price of that.
    //
    // The same panel already carries the measured version of this argument the
    // other way round: `crate::panels::forms` gates its Flatten control on
    // `flatten_refusal` after a period of borrowing `deletion_refusal`, because
    // flatten additionally creates page content and carries a guard deletion
    // does not — two checks of three, which works until it does not, on
    // documents that are not exotic.
    let rename_refusal = doc.session.rename_refusal();
    let delete_refusal = doc.session.deletion_refusal();
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI.
        // Written EVERY frame this section draws, refused or not — see
        // `REGION_RENAME` for why the regions are only readable as evidence
        // when this line is unconditional.
        format!(
            "{TRACE_GATES} rename_refused={} delete_refused={}",
            u8::from(rename_refusal.is_some()),
            u8::from(delete_refusal.is_some()),
        )
    });

    // No `.strong()` — R84 / DEFECTS.md D11: no theme this project ships
    // renders it legibly on a panel.
    ui.label(t::heading());
    ui.add_space(4.0);

    facts(ui, field, &selected);
    ui.add_space(6.0);
    ui.separator();
    ui.add_space(6.0);
    rename_row(ui, state, &selected, rename_refusal.is_some(), actions);
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
    delete_row(ui, field, &selected, delete_refusal.is_some(), actions);
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
///
/// # ★★★ On a document that refuses a rename, this draws a SENTENCE and no box
///
/// `refused` is `EditSession::rename_refusal`'s answer, asked in [`section`]
/// before anything was drawn. When it is true the operator gets one line saying
/// the document forbids it, and **no text field and no button** — which is R9's
/// ruling rather than a preference:
///
/// - Greying is for a capability that is *temporarily* unavailable, and is
///   always explained on hover. A certification signature is not temporary and
///   cannot be argued out of.
/// - A permanently-refused capability renders **nothing**, or a sentence saying
///   where the thing actually lives. Here there is no elsewhere, so it is the
///   sentence.
///
/// And a sentence rather than silence, because the section around it is full of
/// controls: an operator who finds the rename box missing with no explanation
/// has found a panel that looks half-drawn.
///
/// ★★ What this replaces is worse than either. Before this, the box and the
/// button were drawn unconditionally, the operator typed a new name, pressed
/// Rename, and the engine refused **after** the typing — with the refusal
/// reaching the trace and nothing else. That is the shape R83 exists to remove:
/// discovery by pressing, on a control the program already knew would refuse.
fn rename_row(
    ui: &mut Ui,
    state: &mut PanelsState,
    selected: &crate::app::state::SelectedField,
    refused: bool,
    actions: &mut Vec<Action>,
) {
    if refused {
        ui.label(t::rename_refused());
        return;
    }
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
    // ★ Published only on the path where the control exists — see
    // `REGION_RENAME`. Greying is still correct HERE: "you have not typed a
    // usable name yet" is exactly the temporary, operator-fixable condition R9
    // reserves greying for, and it is explained on hover two lines down.
    crate::diag::ui_rect(REGION_RENAME, commit.rect);
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
///
/// # ★★★ `refused` — the second half of a finding, and this is what it cost
///
/// `EditSession::deletion_refusal` has existed for the whole life of this
/// shell, carries a doctest that spells out this exact call site, and was
/// **consulted by nothing**. It appeared in this crate only inside comments
/// — three of them, in `panels::forms`, arguing correctly about which query
/// Flatten should ask and never noticing that Delete asked none at all.
///
/// So both of these buttons were drawn live on every document, including a
/// certified one, and every press of them returned the same refusal to the
/// trace and nothing to the operator. R83's whole subject.
///
/// The remedy is the same as the rename box's, for the same reason: the
/// controls are **not drawn**, and a sentence takes their place. Deleting a
/// field and deleting one of its boxes are both structural, so they share one
/// gate and one sentence — unlike rename, which asks its own query in
/// [`section`] because it is a different question that happens to have the same
/// answer today.
fn delete_row(
    ui: &mut Ui,
    field: &pdfce_core::forms::Field,
    selected: &crate::app::state::SelectedField,
    refused: bool,
    actions: &mut Vec<Action>,
) {
    if refused {
        ui.label(t::delete_refused());
        return;
    }
    ui.horizontal(|ui| {
        let remove = ui
            .button(t::delete_field())
            .on_hover_text(t::delete_field_hover(field.widgets.len()));
        crate::diag::ui_rect(REGION_DELETE, remove.rect);
        if remove.clicked() {
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
