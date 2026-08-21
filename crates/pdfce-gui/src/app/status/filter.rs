//! # `app::status::filter` — the Select popup: what a click may land on
//!
//! `OPERATOR_REQUESTS.md` O17's first half. The status bar's **Select**
//! button, the eleven-row popup behind it, and the standing line that appears
//! when the operator has left nothing selectable at all.
//!
//! [`crate::canvas::pick`] holds the model — the eleven classes, the
//! subtractive invariant, and why the filter composes with the mode as an
//! `AND` rather than as an override. This file holds only the surface, and the
//! split is the usual one: that module can be asserted about in a unit test,
//! while everything here has to be **driven** before it counts (R1).
//!
//! ## ★ Why this is a file and not a section of [`super`]
//!
//! R2's 1,500-line ceiling forced the split, and — as with
//! [`super::page_box`], [`super::notes`] and [`super::decline`] before it —
//! the forced seam turned out to be a real one. Everything else on the bar
//! answers *what is true about the view*: which page, what zoom, what the last
//! raster contained, why a command declined. Those are all **reports**.
//!
//! This is the one thing on the status bar that is not a report. It changes
//! what the pointer does. That is a different kind of control living on a
//! surface full of readouts, and it earns its own file for the same reason it
//! earns its own position in the layout — see [`show`] on why it sits at the
//! left edge of the fixed cluster rather than inside the zoom group.
//!
//! ## What this module does NOT do
//!
//! It does not persist anything. The caller compares the filter before and
//! after and writes it if it moved — see [`crate::app::frame`]'s status-bar
//! block for why that comparison lives there, and [`crate::app::pickstore`]
//! for why the write is immediate where the dock layout's is debounced.

use crate::canvas::pick::{PickClass, PickFilter};
use crate::text::pick as t_pick;

/// The **Select** button and the popup behind it: what a click on the page is
/// allowed to land on.
///
/// `OPERATOR_REQUESTS.md` O17. This is the replacement for Edit > Content's two
/// ribbon buttons, and the placement is the point rather than a detail — see
/// [`crate::canvas::pick`]'s header for why a filter belongs on a surface that
/// is visible *while you aim* instead of two levels into a ribbon you left
/// thirty seconds ago.
///
/// # Why this mutates rather than raising an [`Action`]
///
/// The bar's standing rule is *raise actions and mutate nothing*, and this is
/// the second deliberate exception beside [`super::find_group`]. The rule exists so
/// that a command's one implementation stays in the dispatcher, where undo,
/// tracing and mode gating are applied uniformly. None of those apply here: a
/// selection filter is not undoable (it is not a change to the document), it is
/// not gated by mode (it composes with the mode as an `AND`, and switching a
/// class off is legal in every mode), and it has no other invocation site to
/// stay consistent with. An `Action` round-trip would add a dispatcher arm
/// whose entire body is one assignment.
///
/// # ★ The caller is what persists it
///
/// This function does not write to disk, and that is not laziness. *"Did the
/// operator change the filter"* is one comparison of a `Copy` value at the call
/// site, which is both cheaper and more obvious than a dirty flag threaded
/// through the bar. See [`crate::app::frame`]'s status-bar block.
pub(super) fn show(ui: &mut egui::Ui, filter: &mut PickFilter) {
    let response = ui
        .button(t_pick::filter_button())
        .on_hover_text(t_pick::filter_button_tooltip());
    crate::diag::ui_rect(super::REGION_FILTER, response.rect);

    if response.clicked() {
        egui::Popup::toggle_id(ui.ctx(), egui::Popup::default_response_id(&response));
    }

    egui::Popup::menu(&response)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| popup(ui, filter));
}

/// The body of the Select popup — a heading, All/None, and one row per class.
///
/// # ★ `CloseOnClickOutside`, not `CloseOnClick`
///
/// egui's default for a menu is to close as soon as anything in it is clicked,
/// which is right for a list of commands and wrong for a list of checkboxes.
/// Switching four classes off is one operator decision expressed as four
/// clicks; a popup that vanished after the first would charge a reopen for each
/// of the remaining three, and the reopen is the expensive part —
/// `FEATURES.md` records the measured cost of exactly that ritual as the
/// complaint this feature answers.
///
/// This is the convention every filter list in the class follows: AutoCAD's
/// object-snap list, Illustrator's layer locks, a browser's cookie panel. A
/// list you can only make one change to is a menu, not a filter.
fn popup(ui: &mut egui::Ui, filter: &mut PickFilter) {
    // ★ Deliberately NOT `.strong()`. `tools/gates/check-strong-text.sh`
    // rejects it, and the reason is defect D11: egui has no role for
    // emphasised text, so `.strong()` resolves to the ACCENT-FILLED widget
    // state — pale text on a pale background on an ordinary surface, which
    // survives `override_text_color`. Six labels shipped that way. The
    // hierarchy here is carried by position and by the separator underneath,
    // which is what the gate's own guidance recommends and which reads better
    // than the emphasis would have.
    ui.label(t_pick::filter_heading());
    ui.separator();

    ui.horizontal(|ui| {
        // Both are ordinary buttons and neither is ever greyed. "All" when
        // everything is already on is a no-op, and a no-op the operator can see
        // is cheaper than a disabled control they have to reason about — R9
        // reserves greying for *temporarily unavailable*, which this is not.
        if ui.button(t_pick::filter_all()).clicked() {
            *filter = PickFilter::all();
        }
        if ui.button(t_pick::filter_none()).clicked() {
            *filter = PickFilter::none();
        }
    });
    ui.separator();

    for (index, class) in PickClass::ALL.into_iter().enumerate() {
        let mut on = filter.allows(class);
        let row = ui
            .horizontal(|ui| {
                // The glyph sits INSIDE the row rather than beside it, so the
                // whole row is one target. A glyph merely adjacent to a
                // checkbox is a piece of the control the operator can aim at
                // and miss — convention C7's "visible control, silently inert",
                // in miniature.
                let hit = ui.checkbox(&mut on, t_pick::class_label(class));
                ui.add(crate::icons::image(ui, class_icon(class)));
                hit
            })
            .inner
            .on_hover_text(t_pick::class_tooltip(class));

        if on != filter.allows(class) {
            filter.set(class, on);
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "pick-filter-toggled class={} on={on} remaining={}",
                    class.token(),
                    filter.count(),
                )
            });
        }

        // Published per row and keyed by INDEX — see `super::REGION_FILTER_ROW`.
        // Bound to a local because an inline format capture cannot name a
        // path — `{super::REGION_FILTER_ROW}` is a parse error, not a lookup.
        let prefix = super::REGION_FILTER_ROW;
        crate::diag::ui_rect(&format!("{prefix}:{index}"), row.rect);
    }
}

/// The glyph for one class's row.
///
/// Six of the eleven reuse icons the set already had and five were authored for
/// this popup; [`crate::icons::Icon`] carries the argument for each. A `match`
/// rather than a lookup table so that adding a class is a compile error here —
/// a row that silently drew no glyph would be the one row that looked broken.
const fn class_icon(class: PickClass) -> crate::icons::Icon {
    use crate::icons::Icon;
    match class {
        PickClass::Text => Icon::PickText,
        PickClass::Path => Icon::PickPath,
        PickClass::Image => Icon::InsertImage,
        PickClass::FormXObject => Icon::PickFormXObject,
        PickClass::Part => Icon::PickPart,
        PickClass::Node => Icon::ShowPoints,
        PickClass::Markup => Icon::Markup,
        PickClass::CeDimension => Icon::Measure,
        PickClass::FormField => Icon::FormField,
        PickClass::Link => Icon::PickLink,
        PickClass::Characters => Icon::TextSelect,
    }
}

/// The standing line shown when the filter has left **nothing** selectable.
///
/// See [`crate::text::pick::nothing_selectable`] for why this exists: the state
/// is legitimate and its symptom — a canvas that ignores every click — is
/// indistinguishable from a fault. Drawn on the left, with the narration,
/// because it is a statement about the session rather than a control.
///
/// ★ Deliberately **not** a mark on the page. Rule 4: disclosure lives
/// off-canvas.
pub(super) fn empty_note(ui: &mut egui::Ui, filter: PickFilter) {
    if !filter.is_none() {
        return;
    }
    let rect = ui
        .scope(|ui| {
            ui.label(t_pick::nothing_selectable())
                .on_hover_text(t_pick::nothing_selectable_tooltip());
        })
        .response
        .rect;
    crate::diag::ui_rect(super::REGION_FILTER_EMPTY, rect);
}
