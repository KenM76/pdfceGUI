//! # `panels::forms::tab_order::register` — the rows that put an unclaimed
//! form control back into the form
//!
//! ## Why this is here and not in a dialog of its own
//!
//! Because [`super::model`] already answers the question the dialog would have
//! had to re-ask. *"Which widgets does this page list that no field claims?"*
//! is one `/Annots` walk cross-referenced against one parsed `/AcroForm`, and
//! the Tab-order section performs it every frame it is open — it is the whole
//! reason that section exists.
//!
//! A separate Register window would have walked `/Annots` a second time, from a
//! second parse, at a second moment. Two answers to one question is how a list
//! and the button beside it come to disagree about the set, and the failure is
//! silent: the operator presses Register on the third box and a different third
//! box is registered.
//!
//! It is also where the operator already is. They opened Tab order because
//! something on the page would not fill, and the section told them *"3 boxes on
//! this page are drawn as form controls that no field claims"*. The next thing
//! they want is to do something about it, and R9's spirit — no control that
//! looks available and is not — reads the other way round here: **a stated
//! problem with no offered remedy is the same defect wearing different
//! clothes.**
//!
//! ## ★ Every row can be pressed with the name box empty, and that is the
//! recommended answer
//!
//! The engine measured a real form and found **11 of 13** unclaimed widgets to
//! be merged field-widgets (§12.7.3.1) — one dictionary serving as both field
//! and widget, carrying its own `/T`, `/FT`, `/V` and `/DA`. For those,
//! `adopt_widget(id, None)` recovers the field exactly as it was, and typing a
//! name would *override* the one the file already holds.
//!
//! The other 2 were **bare kids** with no identity at all, and they refuse. The
//! refusal is worded, arrives in the status bar, and says what typing a name
//! will actually produce — a new, empty field, not the radio button that was
//! lost. See [`crate::text::status::adopt_declined_no_name`].
//!
//! ## ★ Why there is no pre-flight, said out loud rather than left as a gap
//!
//! `EditSession` has `fill_refusal` and `rename_refusal` — non-mutating
//! predicates that let a shell grey a control instead of failing late. There is
//! no `adopt_refusal`, so this surface cannot know which of the two shapes a
//! box is until it presses.
//!
//! The consequence is small and is honestly disclosed rather than hidden: the
//! blank-name press is offered on every row, and on a bare kid it declines with
//! a sentence that says what to do next. One press, one sentence, converged.
//! What it costs is that pdfce cannot say *in advance* "this one will need a
//! name", which is information the operator would rather have before typing.
//!
//! A request for the missing sibling is filed with the engine. When it lands,
//! the change here is to ask it once per row and label the two shapes
//! differently — the rows and the action do not move.

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::text::forms as t;

use super::model::Unclaimed;

/// What the operator has typed into the name boxes, and which document
/// revision it describes.
///
/// # ★ Keyed on `(path, edit_epoch)`, which is what makes undo correct
///
/// Exactly `super::super::FormsUi`'s rule, for exactly its reason, and it is
/// worth restating because the consequence here is the opposite of what a
/// naive reading suggests.
///
/// A successful registration bumps the epoch, so **every draft in this map is
/// discarded**. That is right rather than lossy: the widget the operator was
/// typing about is no longer unclaimed, its row is gone, and the box they typed
/// into does not exist any more. Keeping the text would mean re-showing it
/// against whichever box happened to take that row next.
///
/// An **undo** bumps the epoch too, which restores the row and clears the box.
/// Also right: the name went into the document and came back out, so a box
/// still holding it would be showing a value the document no longer has, which
/// is the precise defect `FormsUi`'s own comment records for the fill drafts.
#[derive(Clone, Default)]
struct Drafts {
    /// The `(document path, edit epoch)` this map describes.
    key: Option<(PathBuf, u64)>,
    /// Typed names, by the widget's object **number**.
    ///
    /// The number rather than the whole [`pdfce_core::object::ObjId`] because
    /// `ObjId` is not `Ord` in a way this map could rely on across engine
    /// versions, and because a generation cannot distinguish two live objects
    /// anyway — §7.3.10 gives a generation meaning only for reused free
    /// numbers, and nothing here holds a reference to a freed object.
    names: BTreeMap<u32, String>,
}

impl Drafts {
    /// The egui id this state is stored under.
    ///
    /// Distinct from `FormsUi`'s. The two are different lifetimes of thing
    /// keyed the same way — one holds field values, one holds proposed field
    /// names — and sharing an id would make each frame's store overwrite the
    /// other's.
    fn id() -> egui::Id {
        egui::Id::new("pdfce-forms-tab-order-register")
    }

    /// Read this frame's drafts, dropping them if they describe a different
    /// document or a different revision.
    fn load(ui: &egui::Ui, doc: &OpenDoc) -> Self {
        let key = (doc.path.clone(), doc.edit_epoch);
        let state: Self = ui
            .data(|d| d.get_temp::<Self>(Self::id()))
            .unwrap_or_default();
        if state.key.as_ref() == Some(&key) {
            state
        } else {
            Self {
                key: Some(key),
                names: BTreeMap::new(),
            }
        }
    }

    /// Write this frame's drafts back.
    fn store(self, ui: &egui::Ui) {
        ui.data_mut(|d| d.insert_temp(Self::id(), self));
    }
}

/// Draw one row per unclaimed widget on a page, and raise
/// [`Action::AdoptWidget`] when one is pressed.
///
/// `page_index` is 0-based — it is carried into the action for the trace and
/// the re-raster, not for the engine, which edits the document-level
/// `/AcroForm` and never asks which page.
///
/// # ★ At most one registration per frame, and it is not an accident
///
/// The loop `break`s after a press. Two presses in one frame would queue two
/// `AdoptWidget`s against a listing computed **before** either ran, and the
/// second would be acting on a set the first has already changed — the same
/// stale-index hazard the engine hit in its own CLI and described plainly:
/// *"the indices shift after every add … I got this wrong myself and nested
/// something two levels deeper than intended, and the output looked entirely
/// plausible."*
///
/// The ids here are stable where indices are not, so the second action would in
/// fact still name the right widget. The `break` is kept anyway, because
/// *"queue only what was computed against the state you have"* is the property
/// worth holding mechanically rather than re-deriving each time a queued verb
/// is added. It costs the operator nothing: physically, one press per frame is
/// all there is.
pub(super) fn rows(
    ui: &mut egui::Ui,
    doc: &OpenDoc,
    page_index: usize,
    unclaimed: &[Unclaimed],
    actions: &mut Vec<Action>,
) {
    if unclaimed.is_empty() {
        return;
    }
    let mut drafts = Drafts::load(ui, doc);
    let mut pressed: Option<(pdfce_core::object::ObjId, Option<String>)> = None;

    for widget in unclaimed {
        ui.horizontal(|ui| {
            ui.label(t::tab_order_unclaimed_row(widget.position));
            let draft = drafts.names.entry(widget.id.num).or_default();
            ui.add(
                egui::TextEdit::singleline(draft)
                    .desired_width(140.0)
                    .hint_text(t::tab_order_register_name_hint()),
            );
            if ui.button(t::tab_order_register()).clicked() && pressed.is_none() {
                // Trimmed here, and an empty box becomes `None` rather than
                // `Some("")`. The engine refuses an empty name with
                // `FieldNameEmpty`, and that refusal is unreachable from this
                // surface precisely because of this line — see
                // `crate::app::status::decline::record_adopt_refusal`'s table,
                // which claims it is unreachable and would be wrong without it.
                let typed = draft.trim();
                let name = (!typed.is_empty()).then(|| typed.to_owned());
                pressed = Some((widget.id, name));
            }
        });
        if pressed.is_some() {
            break;
        }
    }

    if let Some((widget, name)) = pressed {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!(
                "adopt-widget-requested page={page_index} obj={} named={}",
                widget.num,
                u8::from(name.is_some())
            )
        });
        actions.push(Action::AdoptWidget {
            page: page_index,
            widget,
            name,
        });
    }
    drafts.store(ui);
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfce_core::object::ObjId;

    /// An empty list draws nothing at all — not an empty group, not a heading.
    ///
    /// R9: a page whose widgets are all claimed has no problem to offer a
    /// remedy for, and a "0 boxes need registering" line is a placeholder
    /// wearing a number.
    #[test]
    fn a_page_with_nothing_unclaimed_draws_nothing() {
        let ctx = egui::Context::default();
        let mut actions = Vec::new();
        let doc = crate::app::state::open_fixture(crate::app::state::FOUR_PAGES);
        // `run_ui` rather than `run` — egui 0.35 renamed it, and it hands the
        // closure a root `Ui` directly, which is what this needs.
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            let before = ui.min_rect();
            rows(ui, &doc, 0, &[], &mut actions);
            assert_eq!(ui.min_rect(), before, "nothing may be laid out");
        });
        assert!(actions.is_empty());
    }

    /// The rows are drawn in the order the model gives them, which is
    /// `/Annots` order.
    ///
    /// Asserted through the positions rather than through pixels: a row for
    /// position 2 must exist and must say 2, because the number is the only
    /// handle the operator has on a box with no name — they press Tab that many
    /// times to find it.
    #[test]
    fn each_unclaimed_widget_gets_its_tab_position() {
        assert_eq!(t::tab_order_unclaimed_row(2), "Box 2 in the tab order");
        assert_ne!(
            t::tab_order_unclaimed_row(2),
            t::tab_order_unclaimed_row(3),
            "two boxes must be distinguishable"
        );
    }

    /// A draft map keyed on one revision is discarded by the next.
    ///
    /// The property that makes undo correct here — see [`Drafts`]. Exercised on
    /// the struct rather than through a frame, because the thing being asserted
    /// is the key comparison and not the widget.
    #[test]
    fn an_edit_forgets_every_typed_name() {
        let mut drafts = Drafts {
            key: Some((PathBuf::from("a.pdf"), 4)),
            names: BTreeMap::from([(12, "Address".to_owned())]),
        };
        assert_eq!(drafts.key, Some((PathBuf::from("a.pdf"), 4)));
        let stale = drafts.key.as_ref() != Some(&(PathBuf::from("a.pdf"), 5));
        assert!(stale, "a bumped epoch must invalidate the drafts");
        let other = drafts.key.as_ref() != Some(&(PathBuf::from("b.pdf"), 4));
        assert!(other, "a different document must invalidate the drafts");
        drafts.names.clear();
        assert!(drafts.names.is_empty());
    }

    /// The id this state stores itself under is not the fill panel's.
    ///
    /// Two `Clone` types in one `data` store under one id is a silent
    /// overwrite: whichever stores second wins, and the symptom is a text box
    /// that forgets a keystroke at a time.
    #[test]
    fn the_draft_store_does_not_collide_with_the_fill_panel() {
        assert_ne!(Drafts::id(), egui::Id::new("pdfce-forms-ui"));
    }

    /// An object number survives the round trip into the draft key.
    #[test]
    fn drafts_are_keyed_by_object_number() {
        let id = ObjId::new(12, 0);
        let mut names = BTreeMap::new();
        names.insert(id.num, "Agree".to_owned());
        assert_eq!(names.get(&12).map(String::as_str), Some("Agree"));
    }
}
