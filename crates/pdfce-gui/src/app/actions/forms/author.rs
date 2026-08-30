//! # `app::actions::forms::author` — making a form control from the operator's choices
//!
//! One function. It is the path the **placement dialog** commits through, and
//! its sibling [`super::paste`] is the path a **clipboard** commits through.
//!
//! ## ★★ Why those are two paths and not one
//!
//! `paste`'s own header carries the long form; in a sentence: `New*Field` is a
//! **spec** — geometry plus a dozen booleans — so authoring from it can only
//! carry what the spec can *express*, and that is exactly right when the
//! operator has just chosen every value in a dialog. It is exactly wrong when
//! the values came from an existing field, which is why the paste route carries
//! a clip instead.
//!
//! Split out of `super` under R2 on 2026-08-30, when widget rotation took that
//! file past 1,500 lines for the second time in one session.

use crate::app::state::OpenDoc;

pub(in crate::app::actions) fn author(
    doc: &mut OpenDoc,
    page: usize,
    rect: pdfce_core::page_tree::Rect,
    draft: &crate::canvas::formfield::Draft,
) {
    use crate::canvas::formfield::FormFieldKind as K;
    use pdfce_core::edit::{
        BorderSpec, BorderStyle, ChoiceOption, NewCheckBox, NewChoiceField, NewPushButton,
        NewRadioButton, NewTextField, TooltipChoice,
    };

    // ★★★ REFUSE A NAME THAT WOULD SWALLOW AN EXISTING FIELD, before anything
    // is written. See `group_is_a_field` — this is a shim for an engine gap and
    // it guards unrecoverable data loss, so it runs first.
    if let Some(victim) = super::group_is_a_field(doc, draft.name.trim()) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("add-form-field-refused reason=group-is-a-field victim={victim}")
        });
        crate::app::actions::record_note(
            doc.edit_epoch,
            crate::text::fieldclip::name_would_swallow(&victim),
        );
        return;
    }

    let name = draft.name.trim().to_owned();
    // ★ Empty means DECLINED, not undecided. See the header — this one line is
    // the difference between a feature and a nine-day blocker.
    let tooltip = if draft.tooltip.trim().is_empty() {
        TooltipChoice::Declined
    } else {
        TooltipChoice::Text(draft.tooltip.trim().to_owned())
    };
    // A zero width is how PDF spells "no border", so the operator's choice
    // travels as a number rather than as a second boolean that could disagree
    // with it.
    let border = BorderSpec {
        style: BorderStyle::Solid,
        width: draft.border_width.max(0.0),
    };
    let kind = draft.kind;
    // ★★★ The epoch BEFORE, so the selection below is set only if the field was
    // actually authored. `vector_edit` bumps it on success and leaves it alone
    // on a refusal, which is the one signal available here -- the closure's
    // `Result` is consumed inside the funnel.
    let before = doc.edit_epoch;
    let placed_name = draft.name.trim().to_owned();

    crate::app::actions::apply::vector_edit(doc, "add-form-field", page, 1, |session| {
        let outcome = match kind {
            K::Text => {
                let mut spec = NewTextField::new(page, name, rect);
                spec.value = draft.value.clone();
                spec.max_len = draft.max_len;
                spec.tooltip = tooltip;
                spec.multiline = draft.multiline;
                spec.password = draft.password;
                // ★ Gated on `comb_ok` rather than on the flag alone, so the
                // dialog's rule and the authored field cannot disagree: comb
                // divides the width into `max_len` cells, and without a
                // maximum there is nothing to divide by.
                spec.comb = draft.comb && draft.comb_ok();
                spec.read_only = draft.read_only;
                spec.required = draft.required;
                spec.border = border;
                session.add_text_field(&spec)
            }
            K::CheckBox => {
                let mut spec = NewCheckBox::new(page, name, rect);
                spec.on_state = draft.export_value.clone();
                spec.checked = draft.checked;
                spec.tooltip = tooltip;
                spec.read_only = draft.read_only;
                spec.required = draft.required;
                spec.border = border;
                session.add_check_box(&spec)
            }
            K::Radio => {
                let mut spec = NewRadioButton::new(page, name, rect, draft.export_value.clone());
                spec.selected = draft.checked;
                spec.tooltip = tooltip;
                // `no_toggle_to_off` and `radios_in_unison` are left at the
                // engine's defaults rather than exposed: they are properties of
                // a GROUP, not of the widget being placed, so offering them
                // per-widget would let two members of one group carry
                // contradictory answers. They belong on a group editor, which
                // is the properties pane's business.
                spec.read_only = draft.read_only;
                spec.required = draft.required;
                spec.border = border;
                session.add_radio_button(&spec)
            }
            K::Choice => {
                // Export value and display text the same, deliberately — which
                // is what `ChoiceOption::plain` means. They differ only when a
                // form is submitted to a system that wants a code rather than a
                // label, which is a second column this dialog does not offer
                // and must not guess at.
                let options: Vec<ChoiceOption> = draft
                    .options()
                    .into_iter()
                    .map(ChoiceOption::plain)
                    .collect();
                let mut spec = NewChoiceField::new(page, name, rect, options);
                spec.combo = draft.combo;
                spec.editable = draft.editable;
                spec.multi_select = draft.multi_select;
                spec.sort = draft.sort;
                spec.tooltip = tooltip;
                spec.read_only = draft.read_only;
                spec.required = draft.required;
                spec.border = border;
                session.add_choice_field(&spec)
            }
            K::PushButton => {
                let mut spec = NewPushButton::new(page, name, rect, draft.caption.clone());
                spec.tooltip = tooltip;
                spec.read_only = draft.read_only;
                spec.border = border;
                session.add_push_button(&spec)
            }
        };
        outcome.map(|o| super::disclosures(&o, kind))
    });

    // ★★★ SELECT WHAT WAS JUST PLACED. `OPERATOR_REQUESTS.md` **O53**.
    //
    // Every program in this class leaves a newly drawn object selected --
    // Acrobat, Word, PowerPoint, Visio, Illustrator, Inkscape -- and they
    // disagree about whether the TOOL stays armed. So the arming is a taste
    // question with a convergent default (`dialogs::formfield` takes Acrobat's)
    // and this is not: it is the half none of them differ on.
    //
    // ★★ It is what makes the operator's next gesture work. He drew a checkbox
    // and reported *"I can't select it on the canvas to move or resize"*; with
    // the tool put down AND the field selected, the grips are already there and
    // the drag is already live. Requiring a click to select something he just
    // created is a step no other editor asks for.
    //
    // ★ Widget 0, because a field authored here has exactly one -- `add_*_field`
    // places a single widget. A field with several is one that grew later,
    // through `merge_document` or a hand-edited file, and there is no "the new
    // one" to name in that case.
    if doc.edit_epoch != before && !placed_name.is_empty() {
        doc.selected_field = Some(crate::app::state::SelectedField {
            field: placed_name,
            widget: 0,
            page,
        });
    }
}
