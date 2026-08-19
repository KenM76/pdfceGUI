//! # `app::actions::forms` — registering a form control the document lists but
//! no field claims
//!
//! One verb, and the smallest sibling of [`super::dimensions`],
//! [`super::pages`] and [`super::export`]. It exists as its own module for the
//! reason those do: `super::apply` is at the far end of R2's file-size budget,
//! and a verb whose *disclosure and refusal wording* is the substantial part of
//! it would put a hundred lines of judgement in the file every other task
//! already contends over.
//!
//! ## What an unclaimed widget is, and why the shell can produce one
//!
//! A `/Widget` annotation in a page's `/Annots` that no entry of the document's
//! `/AcroForm` `/Fields` reaches. It **draws** — border, background, the whole
//! appearance stream — and nothing can fill it, because every filling verb
//! addresses a field by its fully qualified name and this box is in no field.
//!
//! This project's recurring failure mode, a visible control that is silently
//! inert, arriving through a **document** rather than through a ribbon. The
//! operator clicks it, types, and nothing happens.
//!
//! ★ pdfce makes them itself. `EditSession::insert_pages` copies everything
//! reachable from a page, and a page's `/Annots` reaches its widgets — but
//! `/AcroForm` is document-level and is not merged, so a source with 12 fields
//! inserted into a blank document produces 13 widgets and no form at all. The
//! engine measured exactly that (`examples/orphan_probe.rs`, pdfbox corpus) and
//! now returns the count in `InsertOutcome::orphaned_widgets`.
//!
//! ## ★★ Two shapes, and only one of them can be put back
//!
//! The engine's measurement is the reason this module has two refusal arms
//! rather than a success path and a shrug:
//!
//! | shape | of 13 measured | carries | registering it |
//! |---|---|---|---|
//! | **merged field-widget** (§12.7.3.1) | 11 | its own `/FT`, `/T`, `/V`, `/DA` | **recovers the field exactly** |
//! | **bare kid** (a radio group's member) | 2 | nothing at all | **creates a new, empty field** |
//!
//! The second row is `insert_pages` dropping `/Parent` from every dictionary it
//! copies. For a page that is correct — following it would drag the source's
//! whole page tree across. For a widget, `/Parent` **is** its link to its
//! identity, so those two arrived having lost the name `GroupOption`, the type
//! `/Btn`, the radio flags `0xC000` and the value `Option2`. Nothing in the
//! target document holds any of it.
//!
//! An operator cannot see which shape a box is, and the difference decides
//! whether pressing Register restores something or invents something. That is
//! why [`crate::text::status::adopt_declined_no_name`] refuses to use the word
//! *restore*, and why it names re-inserting from the source as the only route
//! that gets the original back.
//!
//! ## Why this uses the funnel
//!
//! `adopt_widget` writes `/AcroForm` and `/T`. It is a document edit with one
//! undo entry, so it goes through [`super::apply::vector_edit`] like every
//! other one — the render worker stopped, the mutation, the epoch bumped, the
//! page invalidated. Nothing here is special except the wording.

use pdfce_core::object::ObjId;

use crate::app::state::OpenDoc;
use crate::app::status::decline::{self, Declined};
use crate::text::status as t;

/// Register one unclaimed widget into the document's `/AcroForm`.
///
/// `name` is `None` when the operator left the box blank, which is the common
/// and correct answer: a merged field-widget carries its own `/T` and typing a
/// name would **override** it rather than supply something missing.
///
/// # ★ Why the refusal is inspected here and the error is still returned
///
/// [`super::apply::vector_edit`] takes `Display` and does one thing with an
/// `Err`: it traces it and leaves the document alone. That is right, and it is
/// not enough for this verb, because two of `adopt_widget`'s five refusals are
/// **things the operator can fix in the next three seconds** — retype the name,
/// or supply one. A refusal an operator can act on that reaches only
/// `PDFCE_DIAG` is a control that does nothing when pressed.
///
/// So the closure records a decline on the way past and then hands the error
/// back unchanged. Both halves matter:
///
/// - **recording, not returning a message**, because `crate::app::status::decline`
///   already owns the store, the retirement rule and the one line in the bar,
///   and a second mechanism beside it would be the one that forgot to retire
///   itself — that module's own header says so;
/// - **returning the error anyway**, so the trace still carries the engine's own
///   `Display` prose. The decline is a sentence for an operator; the trace is
///   the record for whoever is debugging, and they are not the same text and
///   must not become each other. `check-ui-strings.sh`'s exclusion 3 is explicit
///   that an error type's prose is not permission to route UI text through it.
///
/// The three refusals with no arm are unreachable from this surface rather than
/// unhandled — see [`decline::record_adopt_refusal`], which carries the table.
pub(super) fn adopt(doc: &mut OpenDoc, page: usize, widget: ObjId, name: Option<String>) {
    super::apply::vector_edit(doc, "adopt-widget", page, 1, |session| {
        match session.adopt_widget(widget, name.as_deref()) {
            Ok(outcome) => Ok(vec![t::adopted(
                &outcome.name,
                outcome.field_type.is_some(),
                outcome.acroform_created,
            )]),
            Err(error) => {
                if let Some(declined) = correctable(&error) {
                    decline::record_adopt_refusal(declined);
                }
                Err(error)
            }
        }
    });
}

/// Which refusals the operator can do something about.
///
/// A free function taking `&EditError` so it is testable without an
/// `EditSession`, a document or a frame — the same shape
/// `crate::dialogs::insert_image`'s arithmetic was pushed into, and for the
/// same reason: `pdfce_core::edit::EditError` is `#[non_exhaustive]`, so this
/// match needs a wildcard, and a wildcard inside a closure inside a funnel is
/// a place a future variant goes to be silently ignored.
///
/// Here it is one visible function with a test beside it. The wildcard means
/// *"anything else is a fault, not a chore"*, which is a real distinction and
/// the right default: a new refusal variant appearing in a future engine build
/// reaches the trace with its own words and does not silently acquire one of
/// these two sentences, which would be worse than saying nothing.
fn correctable(error: &pdfce_core::edit::EditError) -> Option<Declined> {
    use pdfce_core::edit::EditError as E;
    match error {
        E::FieldNameTaken { .. } => Some(Declined::FieldNameTaken),
        E::WidgetHasNoFieldIdentity { .. } => Some(Declined::WidgetHasNoName),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfce_core::edit::EditError as E;

    /// The two the operator can fix are worded; the ones they cannot are not.
    ///
    /// ★ The negative half is the half worth asserting. `WidgetAlreadyOwned`
    /// cannot happen from this surface — the ids come from exactly the widgets
    /// no field claimed, on the same `/Annots` walk — so if it ever *did*, a
    /// sentence telling the operator to type a different name would be actively
    /// misleading about a state that indicates the listing and the action have
    /// come to disagree about the set. That is a fault to find in the trace, not
    /// a chore to hand to an operator.
    #[test]
    fn only_the_two_the_operator_can_act_on_are_worded() {
        assert_eq!(
            correctable(&E::FieldNameTaken {
                name: "Address".to_owned()
            }),
            Some(Declined::FieldNameTaken)
        );
        assert_eq!(
            correctable(&E::WidgetHasNoFieldIdentity { id: 12 }),
            Some(Declined::WidgetHasNoName)
        );
        assert_eq!(correctable(&E::WidgetAlreadyOwned { id: 12 }), None);
        assert_eq!(correctable(&E::NotAWidget { id: 12 }), None);
    }

    /// The two sentences are different, and neither claims a recovery.
    ///
    /// The wording rule this module's header argues for, asserted rather than
    /// trusted: an operator told they had *restored* a radio button would go
    /// looking for its group, and there is no group.
    #[test]
    fn neither_refusal_promises_a_recovery() {
        let taken = t::adopt_declined_name_taken();
        let unnamed = t::adopt_declined_no_name();
        assert_ne!(taken, unnamed);
        for text in [taken, unnamed] {
            for promise in ["restore", "recover", "put back", "as it was"] {
                assert!(
                    !text.to_lowercase().contains(promise),
                    "{promise:?} promises something registering cannot do: {text}"
                );
            }
        }
        assert!(
            unnamed.contains("insert the pages again"),
            "the one route that does get the original back must be named"
        );
    }

    /// A registration with no field type says so, and one with a type does not
    /// mention it.
    ///
    /// ★ The `field_type: None` case is the fuzzy-never-sneaky half of this
    /// verb: the registration **succeeded**, the operator will be told so, and
    /// the box is *still* not fillable because a top-level field with no `/FT`
    /// has nothing left to inherit from. That is an inference-shaped absence the
    /// operator cannot see, and rule 4 says it is owed a sentence off-canvas
    /// even though — and precisely because — nothing on the page looks wrong.
    #[test]
    fn a_typeless_field_is_disclosed_and_a_typed_one_is_not_nagged_about() {
        let typed = t::adopted("Address", true, false);
        let typeless = t::adopted("Address", false, false);
        assert!(typed.contains("Address"));
        assert!(!typed.contains("field type"));
        assert!(typeless.contains("no field type"));
        assert!(
            typeless.contains("no viewer knows how to fill it"),
            "the consequence is the part the operator needs: {typeless}"
        );
    }

    /// Creating the document's first `/AcroForm` is disclosed, and only then.
    ///
    /// It changes what *other* software does with the file — a viewer that
    /// finds a form shows a form bar over a drawing that had none — and it is
    /// not something the operator asked for. They asked to register one box.
    #[test]
    fn a_document_gaining_its_first_form_is_told() {
        assert!(t::adopted("A", true, true).contains("had no interactive form"));
        assert!(!t::adopted("A", true, false).contains("had no interactive form"));
    }
}
