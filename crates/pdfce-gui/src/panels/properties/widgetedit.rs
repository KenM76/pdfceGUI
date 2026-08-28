//! # `panels::properties::widgetedit` — the **box** a form field is drawn in,
//! as opposed to the field itself
//!
//! `Pass 134.0`'s `EditSession::edit_widget`, consumed 2026-08-27.
//!
//! ## ★★★ Why this is a second file and not four more rows in [`super::fieldedit`]
//!
//! Because the engine has two verbs, and it has two verbs because Acrobat's own
//! scripting model has two scopes. Taken verbatim from the design brief: some
//! properties *"apply to all widgets that are children of that field"*, others
//! *"are specific to individual widgets"*.
//!
//! | scope | verb | properties |
//! |---|---|---|
//! | **field** — one write, every placement | `edit_field` | required, read-only, tooltip, and the type flags |
//! | **widget** — per placement | `edit_widget` | rect, border, visibility, caption |
//!
//! > **Getting this backwards is invisible on the ordinary one-widget field and
//! > wrong on every radio group** — where "the border" can only sensibly mean
//! > one button and "required" can only sensibly mean the group.
//!
//! A single file holding both would make that distinction a comment. Two files
//! make it the module boundary, and the pane draws them as two headed sections
//! so an operator meets it as well.
//!
//! ## ★★★ Moving is free and resizing is not. Both, and the difference shows
//!
//! §12.5.5 derives a widget's appearance matrix from the appearance box's
//! corners and the `/Rect` corners. A **pure translation** makes that matrix a
//! pure translation, so the baked artwork moves with the box, exactly and for
//! nothing — which is why `move_widget` regenerates no appearance and is right
//! not to.
//!
//! A changed **extent** puts the same algorithm to work as a *scale*. A text
//! field dragged twice as wide would render its text twice as wide rather than
//! gaining room for more text. So `edit_widget` compares the **extent, not the
//! corners**, and rebuilds only when it changed. `WidgetEditOutcome::resized`
//! reports which happened, and the pane says so, because *"the box moved"* and
//! *"the box was resized and its contents were redrawn"* are different things
//! to have done to a file.
//!
//! ★★ **`appearance_stale` is the one an operator will see and misread.** A
//! resize that could not rebuild the artwork — a push button's baked caption, a
//! signature — leaves the widget rendering **distorted**. The engine names it;
//! this pane prefixes the engine's own string with what it means on screen.
//!
//! ## ★★ What is NOT here, and it is a boundary report rather than a deferral
//!
//! `WidgetEdit` carries four properties and this pane offers **two**.
//!
//! * **rect** — `forms::Widget::rect`. Readable, writable. Here.
//! * **caption** — `forms::Widget::caption` (`/MK` `/CA`). Readable, writable.
//!   Here.
//! * **border** (`/BS` — style and width) — writable, and **not readable**.
//!   `annot_author::read_border_width` is private, `border_style` is a *writer*,
//!   and `forms::Widget` models no border at all. Grepped, 2026-08-27.
//! * **visibility** (`/F`) — writable, and readable only by a detour:
//!   `annot::page_annotations` returns `Annotation::flags` and a widget would
//!   have to be matched to its annotation by `ObjId`.
//!
//! ⇒ The last two are **absent rather than offered**, and the reason is not
//! effort. A properties control has to show the current value; a border control
//! that could not read one would display an invented default, and the first
//! press would write that invention into the operator's file. That is strictly
//! worse than no control — it is the "shows an approximation and then writes it
//! back" failure `panels::properties::text`'s colour swatch refuses for CMYK.
//!
//! Filed to the request channel rather than worked around, per decision 058:
//! *anything the GUI has to work around is a place the crate boundary was drawn
//! wrong.*
//!
//! ## Rule 4
//!
//! Nothing here marks the canvas. A moved box renders exactly where the saved
//! file will render it, and every disclosure — resized, stale artwork, siblings
//! untouched — lands in the status bar.

use egui::Ui;
use pdfce_core::edit::WidgetEdit;
use pdfce_core::forms::{Field, Widget};
use pdfce_core::page_tree::Rect;

use crate::app::actions::Action;
use crate::app::actions::forms::FieldAction;
use crate::panels::PanelsState;
use crate::text::panels::formfield as t;

/// The section's rect, for `ui-verify`.
///
/// ★ Plain [`crate::diag::ui_rect`], not the visibility-gated form, for the
/// reason [`super::fieldedit`]'s own note records at length: a **section** rect
/// answers *"did this draw?"* and *"where do I scroll?"*, and gating it on
/// 60 % visibility deletes it exactly when the section is taller than its dock
/// slot. The per-control regions below take the gated form, because a check
/// clicks those.
// ui-text-exempt: trace region name, never displayed
pub const REGION: &str = "properties.widget_edit";
/// The four geometry spinners' shared region prefix.
// ui-text-exempt: trace region name, never displayed
pub const GEOMETRY_REGION: &str = "properties.widget_edit.geometry";
/// The Apply button — the one control a driven check presses.
// ui-text-exempt: trace region name, never displayed
pub const APPLY_REGION: &str = "properties.widget_edit.apply";

/// How fast a drag on one of the four spinners moves it, in points per pixel.
///
/// ★ A quarter of a point, matching `super::geometry`'s `SPEED`, and the
/// reason is the same: these are **drafting** numbers on a drawing sheet, where
/// a whole point of drift is visible. An operator who wants a big move types
/// the number.
const SPEED: f64 = 0.25;

/// Draw the selected widget's own properties, or nothing.
///
/// Returns whether it drew. `false` when the selection names a widget the
/// field no longer has — reachable through undo, which does not clear a
/// selection, and the right answer is silence rather than a pane describing a
/// box that is not there.
pub fn section(
    ui: &mut Ui,
    field: &Field,
    fqn: &str,
    widget_index: usize,
    state: &mut PanelsState,
    epoch: u64,
    actions: &mut Vec<Action>,
) -> bool {
    let Some(widget) = field.widgets.get(widget_index) else {
        return false;
    };
    let Some(rect) = widget.rect else {
        // ★ A widget with no readable `/Rect` renders nothing this pane could
        // describe, and a zero-area rect is *intentional* invisibility for a
        // signature field (§12.7.4.5) rather than a defect — so `None` here is
        // the malformed case only. Silence: four spinners seeded from nothing
        // would invite a press that writes an invented box.
        return false;
    };

    let draft = state.widget_props_mut();
    draft.read(widget, rect, fqn, widget_index, epoch);

    ui.label(t::widget_heading());
    // ★ Said only when there is more than one placement, because that is the
    // only state in which the scope distinction is visible — and it is
    // precisely the state in which an operator would otherwise expect this
    // section to behave like the one above it.
    if field.widgets.len() > 1 {
        ui.small(t::widget_scope_note(field.widgets.len()));
    }
    ui.add_space(2.0);

    geometry_rows(ui, draft, actions, fqn, widget_index);
    ui.add_space(4.0);
    caption_row(ui, draft, actions, fqn, widget_index);

    crate::diag::ui_rect(REGION, ui.min_rect());
    true
}

/// The four typed numbers and the button that commits them.
///
/// # ★★ Why an Apply button and not commit-on-release
///
/// [`super::fieldedit`]'s max-length spinner commits on release, and this one
/// deliberately does not — the difference is that **these four are one edit**.
/// A box is moved by changing X *and* Y; committing each on release would
/// author two `edit_widget` calls, two undo entries, and an intermediate state
/// in which the box has moved sideways and not down. `super::geometry` reached
/// the same conclusion for the same reason and this follows it, including the
/// button's placement.
///
/// ★ The button is **greyed when nothing was typed**, which is R9's temporarily
/// unavailable case: there is a capability and no operand, and the hover says
/// so.
fn geometry_rows(
    ui: &mut Ui,
    draft: &mut WidgetPropsDraft,
    actions: &mut Vec<Action>,
    fqn: &str,
    widget_index: usize,
) {
    let spinner = |ui: &mut Ui, label: &str, key: &str, value: &mut f64| {
        ui.horizontal(|ui| {
            ui.label(label);
            let response = ui.add(egui::DragValue::new(value).speed(SPEED).fixed_decimals(2));
            crate::diag::ui_rect_visible(
                &format!("{GEOMETRY_REGION}.{key}"),
                response.rect,
                ui.clip_rect(),
            );
        });
    };
    // ui-text-exempt: trace region keys, never displayed.
    spinner(ui, t::label_widget_x(), "x", &mut draft.x);
    spinner(ui, t::label_widget_y(), "y", &mut draft.y);
    spinner(ui, t::label_widget_w(), "w", &mut draft.w);
    spinner(ui, t::label_widget_h(), "h", &mut draft.h);

    let changed = draft.differs();
    let apply = ui.add_enabled(changed, egui::Button::new(t::widget_apply()));
    crate::diag::ui_rect_visible(APPLY_REGION, apply.rect, ui.clip_rect());
    let apply = if changed {
        // ★ The hover names which of the two acts is about to happen, because
        // the consequences differ and the operator has already decided: a move
        // keeps the baked artwork exact, a resize rebuilds it and may fail to.
        apply.on_hover_text(t::widget_apply_hover(draft.resizes()))
    } else {
        apply.on_disabled_hover_text(t::widget_apply_disabled())
    };
    if apply.clicked() {
        actions.push(
            FieldAction::EditWidget {
                field: fqn.to_owned(),
                widget: widget_index,
                // ★ `from_corners`, not a literal `Rect { .. }`: §7.9.5 lets a
                // `/Rect`'s corners arrive in any order and normalises them, and
                // an operator who types a width of -20 has expressed something
                // the standard has an answer for. Building the rect any other
                // way would either refuse a legal input or author a
                // denormalised box.
                edit: WidgetEdit::new().with_rect(Rect::from_corners(
                    draft.x,
                    draft.y,
                    draft.x + draft.w,
                    draft.y + draft.h,
                )),
                // ui-text-exempt: a control name carried for a refusal message.
                touched: "the box",
            }
            .into(),
        );
    }
}

/// `/MK` `/CA` — the widget's caption.
///
/// ★★ **Not cosmetic on a push button**, which is why the engine models this
/// one key out of `/MK` and none of the other ten. A push button has no `/V` at
/// all (§12.7.4.2.2), so the caption is the only thing distinguishing *Submit*
/// from *Reset* to anyone reading the field list.
///
/// ★ Empty commits `Some("")`, which **removes** it. That is the engine's
/// spelling and it is unambiguous, unlike the tooltip's three-state choice —
/// there is no "leave it alone" to express here, because not touching the
/// control is how you leave it alone.
fn caption_row(
    ui: &mut Ui,
    draft: &mut WidgetPropsDraft,
    actions: &mut Vec<Action>,
    fqn: &str,
    widget_index: usize,
) {
    ui.label(t::label_caption());
    let response = ui.add(
        egui::TextEdit::singleline(&mut draft.caption)
            .desired_width(f32::INFINITY)
            .hint_text(t::label_caption_hint()),
    );
    let typed = draft.caption.trim().to_owned();
    if response.lost_focus() && typed != draft.caption_stored {
        actions.push(
            FieldAction::EditWidget {
                field: fqn.to_owned(),
                widget: widget_index,
                edit: WidgetEdit::new().with_caption(typed),
                // ui-text-exempt: a control name carried for a refusal message.
                touched: "the caption",
            }
            .into(),
        );
    }
}

/// The typed box and caption, and the widget they were read for.
#[derive(Default)]
pub struct WidgetPropsDraft {
    /// `(field name, widget index, edit epoch)` the values below were read at.
    ///
    /// ★ The **widget index** is in the stamp where [`super::fieldedit`]'s
    /// carries only a name, and it has to be: one field can be drawn in three
    /// places with three different boxes, and a draft keyed on the name alone
    /// would carry the first box's numbers onto the second placement. On a
    /// radio group that is the ordinary case rather than the exotic one.
    stamp: Option<(String, usize, u64)>,
    /// Lower-left x, in PDF user space.
    x: f64,
    /// Lower-left y.
    y: f64,
    /// Width.
    w: f64,
    /// Height.
    h: f64,
    /// The four as the document holds them, so Apply can tell whether the
    /// operator changed anything and `resizes()` can tell which act it is.
    stored: (f64, f64, f64, f64),
    /// The caption being typed.
    caption: String,
    /// The caption as the document holds it.
    caption_stored: String,
}

impl WidgetPropsDraft {
    /// Pull the values off a real widget, and sync.
    fn read(&mut self, widget: &Widget, rect: Rect, fqn: &str, widget_index: usize, epoch: u64) {
        let caption = widget
            .caption
            .as_deref()
            .map(|raw| String::from_utf8_lossy(raw).into_owned())
            .unwrap_or_default();
        self.sync(
            (rect.llx, rect.lly, rect.urx - rect.llx, rect.ury - rect.lly),
            caption,
            fqn,
            widget_index,
            epoch,
        );
    }

    /// Re-read when the stamp has moved; otherwise keep what is on screen.
    ///
    /// Takes the values rather than a `&Widget`, for the reason
    /// [`super::fieldedit::FieldPropsDraft::sync`] does: `forms::Widget` has no
    /// `Default`, so a unit test cannot build one without a document, and this
    /// function reads exactly five things off it.
    fn sync(
        &mut self,
        rect: (f64, f64, f64, f64),
        caption: String,
        fqn: &str,
        widget_index: usize,
        epoch: u64,
    ) {
        let stamp = (fqn.to_owned(), widget_index, epoch);
        if self.stamp.as_ref() == Some(&stamp) {
            return;
        }
        self.stamp = Some(stamp);
        self.stored = rect;
        (self.x, self.y, self.w, self.h) = rect;
        self.caption_stored = caption;
        self.caption.clone_from(&self.caption_stored);
    }

    /// Whether any of the four numbers has been typed away from the document's.
    ///
    /// ★ An epsilon rather than `!=`, because the spinners round to two
    /// decimals for display and a `/Rect` read out of a file routinely carries
    /// more. Without it the Apply button would be live the moment the pane
    /// opened, on every widget whose box is not exactly hundredths — which is
    /// most of them, and which reads as the program thinking the operator has
    /// unsaved changes they never made.
    fn differs(&self) -> bool {
        let (x, y, w, h) = self.stored;
        !near(self.x, x) || !near(self.y, y) || !near(self.w, w) || !near(self.h, h)
    }

    /// Whether committing would change the **extent**, which is what decides
    /// between a free translation and an appearance rebuild.
    ///
    /// The engine makes the same comparison and its answer is authoritative;
    /// this one exists only so the Apply button's hover can say which act the
    /// operator is about to perform, **before** they perform it.
    fn resizes(&self) -> bool {
        let (_, _, w, h) = self.stored;
        !near(self.w, w) || !near(self.h, h)
    }
}

/// Two values within display precision of each other.
///
/// Half a hundredth: the spinners show two decimals, so anything closer than
/// that is a difference the operator cannot see and did not type.
fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 0.005
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★★★ **A draft is re-seeded when the WIDGET changes, not only when the
    /// field does.**
    ///
    /// The failure this stamp's middle term exists for, and it is invisible on
    /// every one-widget field: a radio group is one field with several boxes,
    /// so a draft keyed on the name alone would carry the first button's
    /// geometry onto the second, and pressing Apply would move a box the
    /// operator was not looking at.
    #[test]
    fn a_draft_follows_the_widget_and_not_just_the_field() {
        let mut draft = WidgetPropsDraft::default();
        draft.sync((10.0, 20.0, 100.0, 30.0), String::new(), "Group", 0, 0);
        assert!((draft.x - 10.0).abs() < 1e-9);

        // The same field, a different placement.
        draft.sync((200.0, 400.0, 60.0, 12.0), String::new(), "Group", 1, 0);
        assert!(
            (draft.x - 200.0).abs() < 1e-9,
            "the second button's box must replace the first's"
        );
    }

    /// **Apply is dead until something is typed**, and a `/Rect` carrying more
    /// than two decimals does not count as typed.
    ///
    /// ★ The second half is the one worth testing. Without the epsilon the
    /// button would be live the moment the pane opened on any widget whose box
    /// is not exactly hundredths — which reads as unsaved changes the operator
    /// never made, on most real documents.
    #[test]
    fn apply_is_dead_until_a_number_actually_moves() {
        let mut draft = WidgetPropsDraft::default();
        draft.sync((10.0016, 20.0, 100.0, 30.0), String::new(), "F", 0, 0);
        assert!(
            !draft.differs(),
            "a sub-display-precision difference is not a change"
        );

        draft.x = 12.0;
        assert!(draft.differs());
        assert!(!draft.resizes(), "moving is not resizing");

        draft.w = 140.0;
        assert!(draft.resizes(), "and changing the extent is");
    }

    /// **A move and a resize are told apart**, which is what the Apply hover
    /// promises before the press and the status line reports after it.
    #[test]
    fn a_pure_translation_is_never_reported_as_a_resize() {
        let mut draft = WidgetPropsDraft::default();
        draft.sync((10.0, 20.0, 100.0, 30.0), String::new(), "F", 0, 0);
        draft.x += 50.0;
        draft.y -= 12.5;
        assert!(draft.differs());
        assert!(
            !draft.resizes(),
            "both corners moved by the same amount, so the extent is unchanged"
        );
    }
}
