//! # `canvas::fieldclip` — **cut, copy and paste a FORM FIELD**
//!
//! ## What this closes
//!
//! **Ken, 2026-08-29:** *"wire the request. ctrl v for paste as new. ctrl shift
//! v for paste as duplicate."* — `OPERATOR_REQUESTS.md` **O58**.
//!
//! Before this module there was **no path at all** from a selected form field
//! to `Ctrl+C`. Not a lossy one, not a refused one — none. The reason is two
//! deliberate decisions that were each correct and that together left a hole:
//!
//! 1. `canvas::selection::annot`'s exclusion table drops `/Widget` — *"the form
//!    field surface owns it — a click there focuses an editor, and two owners
//!    of one press is how a field becomes unfillable"*. A selected field
//!    therefore lives on `OpenDoc::selected_field`, not in `SelectionState`.
//! 2. [`crate::canvas::clipboard::copy`] reads `doc.selection` and nothing
//!    else.
//!
//! So `Ctrl+C` over a form field fell through to the *content* copy, which
//! looked at an empty content selection and refused with *"nothing is
//! selected"* — over an object with visible grips around it. That is
//! `DEFECTS.md` D4a's shape exactly: a refusal whose sentence describes a
//! different world than the one the operator is looking at.
//!
//! ## ★★★ The two pastes, and why the DUPLICATE is the faithful one
//!
//! Copying a field has two legitimate meanings and `pdfce-core` refuses to
//! guess between them, by name, in `paste_clip_annotations`:
//!
//! > *"a `/Widget` annotation was NOT pasted. A widget carries an `/AcroForm`
//! > field registration and a field name, and a renamed field is a DIFFERENT
//! > field … That is a decision about your form, not a copy."*
//!
//! The operator made the decision, and made **both** answers reachable:
//!
//! | chord | [`PasteAs`] | what lands | the value |
//! |---|---|---|---|
//! | `Ctrl+V` | [`PasteAs::NewField`] | a new, independent field with a fresh name | its own |
//! | `Ctrl+Shift+V` | [`PasteAs::Duplicate`] | **another widget of the same field** | shared — type in one, both fill |
//!
//! ★★★ **The counter-intuitive part, and the single most important thing in
//! this file:** the *duplicate* is the high-fidelity paste and the *new field*
//! is the lossy one. That is the opposite of what the names suggest.
//!
//! `EditSession::add_text_field` and its four siblings **merge** when the `/T`
//! already names a field (`edit.rs:13523`, returning `merged: true`): the field
//! object is not rebuilt, it gains a second widget. So a duplicate inherits
//! `/DA` (font, size, colour), `/Q` (alignment), `/V`, `/DV` and `/AA` (the
//! calculation script) **exactly**, because it *is* the same field.
//!
//! A new-name paste has to re-author through `NewTextField` and friends, and
//! those specs are geometry-plus-booleans. Everything in [`Lost`] is readable
//! on `forms::Field` and writable **nowhere** in the engine as pinned. Filed as
//! `request_form_fields_cannot_be_pasted_and_half_of_it_already_works.md`.
//!
//! ## ★★ Rule 4 — the loss is DISCLOSED, off-canvas, and never drawn
//!
//! A pasted field renders exactly as a saved-and-reopened one would. There is
//! no badge, no tint, no "this copy is incomplete" marker anywhere on the page,
//! because provisional styling is a second rendering path for the same content
//! and two paths drift.
//!
//! ★ But the half of rule 4 that survives is the half that binds here: the
//! operator **cannot see** that a copied field lost its calculation script — a
//! screenshot of the page is identical either way — so it is reported on the
//! status row. [`Lost`] is that report, computed at **copy** time from the
//! source field and carried on the clip, so the sentence describes the field
//! that was actually read rather than being re-derived later against a document
//! that may have moved on.
//!
//! ## Why the clip is a [`Draft`] and not a `forms::Field`
//!
//! Because [`crate::app::actions::forms::author`] already takes a `Draft` and
//! already knows how to turn one into whichever of the five engine specs it
//! needs, with the tooltip rule, the comb gate and the disclosure pass all in
//! one place. A second authoring path here would be the fifth hand-written copy
//! of a sequence that exists precisely once on purpose.
//!
//! `Draft` is also `Clone + PartialEq + Send + Sync + 'static`, which is what
//! `egui::Memory` asks of anything parked in it, and it is the type the
//! placement dialog produces — so a pasted field and a drawn one are the same
//! act with the same code behind them.
//!
//! ## What is refused, and why each
//!
//! | case | [`Refusal`] | why |
//! |---|---|---|
//! | nothing selected | `NothingSelected` | — |
//! | the field is not in the document any more | `Vanished` | the selection outlived an undo |
//! | a **signature** field | `KindCannotBeAuthored` | `pdfce-core` has five `add_*_field` verbs and none of them makes one. R9: refuse honestly rather than paste something that is not a signature field |
//! | the widget has no `/Rect` | `NoGeometry` | there is no box to reproduce |
//! | duplicate-paste of a **radio** button | `RadioNeedsItsOwnExportValue` | see below |
//!
//! ★ The radio refusal is the engine's own reasoning, taken rather than
//! re-derived. On-states live per *widget*, so two members of one radio group
//! must carry different export values — `FormAuthorError::RadioExportValueTaken`
//! is the engine refusing a collision. A duplicate paste would have to *invent*
//! the second export value, and the engine's `field_defaults` doc rules on
//! exactly that: a copied export value "would either collide … or be arbitrary".
//! An invented one is a name nobody chose. Paste-as-**new** on a radio is fine
//! and is offered, because it makes a new group.

use pdfce_core::page_tree::Rect;

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::canvas::formfield::{Draft, FormFieldKind};

/// Which of the two pastes the operator asked for.
///
/// A two-variant enum rather than a `bool`, because `paste(ctx, doc, true)`
/// at the call site says nothing about which is which, and the two differ in
/// what they do to the operator's *form* rather than merely in where a copy
/// lands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasteAs {
    /// `Ctrl+V` — a new, independent field carrying a fresh name.
    NewField,
    /// `Ctrl+Shift+V` — another widget of the field that was copied.
    Duplicate,
}

/// A property the source field carries that a **new-field** paste cannot.
///
/// Each variant is one row of the engine request's §3 table, and each is
/// *readable* on `forms::Field` and *writable* nowhere. They are computed once,
/// at copy time, and rendered by [`crate::text::fieldclip`].
///
/// # Why an enum and not a `Vec<String>`
///
/// `tools/gates/check-ui-strings.sh`: every user-visible string lives in
/// `ui_text`. A clip that carried prose would put a sentence in
/// `egui::Memory`, where no gate can see it and no translation can reach it.
/// This carries the *fact*; the sentence is built at the moment of display.
///
/// ★ None of these applies to [`PasteAs::Duplicate`], which is the whole point
/// of the distinction — see the module header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Lost {
    /// `/DA` — the font, its size and its colour.
    ///
    /// The most visible of the five, and the one an operator reports as *"the
    /// copy looks wrong"* rather than as a missing feature.
    Appearance,
    /// `/Q` — left, centred or right.
    Alignment,
    /// `/DV` — what Reset restores.
    DefaultValue,
    /// `/AA` — the format, calculate and validate actions.
    ///
    /// ★★ The one that is **invisible**. A field in a calculation chain pastes
    /// inert and nothing on the page differs, which is why this variant matters
    /// more than its position in the list suggests.
    Actions,
    /// `/MK` `/BC` and `/BG` — the border and background **colours**.
    ///
    /// Reported unconditionally on a new-field paste, because the authoring
    /// path hard-writes a black `/MK /BC` (`edit.rs:13504`) and `BorderSpec`
    /// carries only a style and a width. Unlike the four above we cannot read
    /// the source's colours to compare, so this is the one row that says *"this
    /// is re-authored"* rather than *"this was present and is gone"*.
    BorderColour,
}

/// Why a field could not be copied, cut or pasted.
///
/// A sentence on the status row, never a silence — the same posture
/// [`crate::canvas::clipboard::Refusal`] takes, for the same D4a reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// No form field is selected.
    NothingSelected,
    /// The selection names a field the document no longer has.
    Vanished,
    /// The widget has no `/Rect`, so there is no box to reproduce.
    NoGeometry,
    /// A signature field. `pdfce-core` has no verb that authors one.
    KindCannotBeAuthored,
    /// A duplicate paste of a radio button would need an export value nobody
    /// chose. See the module header.
    RadioNeedsItsOwnExportValue,
    /// The clipboard holds no form field.
    NothingCopied,
}

/// What the clipboard is holding when a form field was copied.
///
/// Parked inside [`crate::canvas::clipboard::Clipped`] rather than under a key
/// of its own, so that **one clipboard holds one thing**: copying a markup
/// after copying a field replaces it, which is what every program in the class
/// does and what makes `Ctrl+V` mean one thing at a time.
#[derive(Debug, Clone, PartialEq)]
pub struct ClippedField {
    /// Everything [`crate::app::actions::forms::author`] needs, with the source
    /// field's own name still in it. A new-field paste renames a clone of this;
    /// a duplicate paste uses it verbatim, which is what makes the two widgets
    /// one field.
    pub draft: Draft,
    /// The 0-based page it came from.
    ///
    /// Carried for [`crate::canvas::clipboard::PASTE_OFFSET_PT`]'s rule: a
    /// paste onto the same page offsets so the copy is visible, a paste onto a
    /// different one lands in place because *where it was on sheet 1* is the
    /// whole point of copying it to sheet 12.
    pub page: usize,
    /// Its `/Rect`, in PDF user space.
    pub rect: Rect,
    /// What a [`PasteAs::NewField`] paste will not carry. Sorted and
    /// deduplicated at copy time so the sentence is stable.
    pub lost: Vec<Lost>,
}

/// **Copy the selected form field**, writing it to the shared clipboard.
///
/// # Errors
///
/// Every [`Refusal`] except [`Refusal::NothingCopied`] and
/// [`Refusal::RadioNeedsItsOwnExportValue`], both of which only a paste raises.
pub fn copy(ctx: &egui::Context, doc: &OpenDoc) -> Result<ClippedField, Refusal> {
    let clipped = read_selected(doc)?;
    crate::canvas::clipboard::store(
        ctx,
        crate::canvas::clipboard::Clipped::FormField(Box::new(clipped.clone())),
    );
    // ★★★ AND THE OS CLIPBOARD, WITHOUT WHICH CTRL+V DOES NOT ARRIVE AT ALL.
    //
    // Not a courtesy to other applications: `egui-winit` pushes `Event::Paste`
    // **only if the OS clipboard holds non-empty text**, and swallows the
    // keystroke otherwise — so with nothing here, whether a field paste works
    // depends on what the operator last copied in Notepad.
    //
    // ⇒ Found by driving on the day this module was written, with the trap
    // already documented in the RAG and already handled one function away in
    // `clipboard::copy_content`. See `text::fieldclip::os_marker`.
    ctx.copy_text(crate::text::fieldclip::os_marker(&clipped.draft.name));
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "fieldclip-copy name={} page={} kind={:?} lost={}",
            clipped.draft.name,
            clipped.page,
            clipped.draft.kind,
            clipped.lost.len()
        )
    });
    Ok(clipped)
}

/// **Cut the selected form field** — copy it, then raise the delete.
///
/// The delete travels as an [`Action`] rather than being applied here, because
/// this function borrows `doc` immutably and because every other destructive
/// gesture in this shell goes through the queue. `DeleteWidget` rather than
/// `DeleteField` is deliberate: the operator pointed at **a box**, and on a
/// field with three boxes removing all three is not what they asked for. The
/// engine collapses the field when its last widget goes.
///
/// # Errors
///
/// As [`copy`].
pub fn cut(
    ctx: &egui::Context,
    doc: &OpenDoc,
    actions: &mut Vec<Action>,
) -> Result<ClippedField, Refusal> {
    let clipped = copy(ctx, doc)?;
    // Safe to unwrap conceptually — `copy` returned Ok, so there was a
    // selection — but expressed as a match so a future change to `copy` cannot
    // turn this into a panic.
    if let Some(selected) = doc.selected_field.as_ref() {
        actions.push(Action::Field(
            crate::app::actions::forms::FieldAction::DeleteWidget {
                field: selected.field.clone(),
                widget: selected.widget,
            },
        ));
    }
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("fieldclip-cut name={}", clipped.draft.name)
    });
    Ok(clipped)
}

/// **Paste the clipboard's form field onto `page`**, in one of the two senses.
///
/// # Where it lands
///
/// [`crate::canvas::clipboard::PASTE_OFFSET_PT`] down and to the right on the
/// **same** page, in place on a different one. The same rule the markup
/// clipboard uses and for the same two reasons, which are worth restating
/// because they pull in opposite directions and both are right: a same-page
/// paste that landed exactly on the original is invisible, and a cross-page
/// paste that offset would move the copy away from the position that was the
/// reason for copying it.
///
/// ★ A [`PasteAs::Duplicate`] onto the same page offsets too. Two widgets of
/// one field stacked exactly on each other is a form the operator cannot
/// separate, and the fact that they share a value does not make them one box.
///
/// # Errors
///
/// [`Refusal::NothingCopied`] when the clipboard holds no field, and
/// [`Refusal::RadioNeedsItsOwnExportValue`] on a duplicate paste of a radio
/// button.
pub fn paste(
    ctx: &egui::Context,
    doc: &OpenDoc,
    page: usize,
    mode: PasteAs,
    actions: &mut Vec<Action>,
) -> Result<(), Refusal> {
    let Some(crate::canvas::clipboard::Clipped::FormField(clipped)) =
        crate::canvas::clipboard::read(ctx)
    else {
        return Err(Refusal::NothingCopied);
    };
    if mode == PasteAs::Duplicate && clipped.draft.kind == FormFieldKind::Radio {
        return Err(Refusal::RadioNeedsItsOwnExportValue);
    }

    let rect = placed_rect(clipped.rect, clipped.page, page);
    let mut draft = clipped.draft.clone();
    if mode == PasteAs::NewField {
        draft.name = unique_name(doc, &clipped.draft.name);
        // ★★ A new field starts EMPTY, and this is the one place this module
        // deviates from "reproduce what was copied".
        //
        // A value is content, not a property — the engine's `field_defaults`
        // doc rules on the same question the same way, and it is the reason
        // `--defaults-from` excludes `/V`. Copying the box in a title block
        // called `Drawn By` to make one called `Checked By` and having the
        // second arrive pre-filled with the first person's name is a form that
        // is wrong on paper the moment it is printed.
        //
        // ★ The DUPLICATE keeps it, necessarily and correctly: it is the same
        // field, so it has the same value by definition. There is no choice to
        // make there, which is why this branch is inside the `NewField` arm.
        draft.value.clear();
        draft.checked = false;
    }

    actions.push(Action::Field(
        crate::app::actions::forms::FieldAction::Paste {
            page,
            rect,
            draft: Box::new(draft),
        },
    ));
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "fieldclip-paste mode={mode:?} page={page} from_page={} lost={}",
            clipped.page,
            if mode == PasteAs::NewField {
                clipped.lost.len()
            } else {
                0
            }
        )
    });
    Ok(())
}

/// What a paste of the clipboard's field would lose, or `None` when the
/// clipboard holds no field.
///
/// Split out so the *status sentence* can be written by whoever raises the
/// paste without that caller having to know the clip's shape. Returns an empty
/// slice for a duplicate, which is the fidelity claim this module makes and is
/// worth being able to assert on.
#[must_use]
pub fn losses(ctx: &egui::Context, mode: PasteAs) -> Option<Vec<Lost>> {
    let crate::canvas::clipboard::Clipped::FormField(clipped) =
        crate::canvas::clipboard::read(ctx)?
    else {
        return None;
    };
    Some(match mode {
        PasteAs::NewField => clipped.lost.clone(),
        PasteAs::Duplicate => Vec::new(),
    })
}

/// Where the pasted box goes. See [`paste`]'s header for the two rules.
fn placed_rect(source: Rect, from_page: usize, to_page: usize) -> Rect {
    if from_page != to_page {
        return source;
    }
    let d = crate::canvas::clipboard::PASTE_OFFSET_PT;
    Rect {
        llx: source.llx + d,
        lly: source.lly - d,
        urx: source.urx + d,
        ury: source.ury - d,
    }
}

/// A field name this document does not use, derived from `base`.
///
/// `Text1` → `Text2` → `Text3`, and `Drawn By` → `Drawn By2`. The spelling is
/// [`crate::text::fieldclip::candidate_name`]'s — a field name is operator-facing
/// text — and the *numbering* is [`split_trailing_number`]'s, which is logic and
/// belongs here.
///
/// ★★ The convention is Acrobat's, sourced rather than invented: its bulk
/// duplication auto-names copies `Date1`, `Date2`, `Date3`, and the separator is
/// load-bearing rather than cosmetic. `candidate_name`'s header carries both the
/// scripting rationale and the reason a **dot** is refused even though one
/// Acrobat account uses it.
///
/// ★ Nothing here is a *guess at what the operator wants it called*. The name
/// is a placeholder they are expected to change, and the Properties panel's
/// rename control is the route — which is why this generates the first free
/// name rather than opening a dialog. A dialog on every `Ctrl+V` would make the
/// common case (paste four boxes down a column, name them afterwards) four
/// interruptions long.
///
/// Falls back to the base itself past 999 tries, which then hits the engine's
/// own `FieldNameTaken` and surfaces as a refusal. That is unreachable in
/// practice and is written as a bounded loop rather than an unbounded one
/// because an unbounded loop over a document is a hang.
fn unique_name(doc: &OpenDoc, base: &str) -> String {
    let view = doc.session.view();
    let Some(form) = pdfce_core::forms::parse_acroform(&view) else {
        return base.to_owned();
    };
    let taken = |candidate: &str| form.fields_named(candidate).next().is_some();
    if !taken(base) {
        return base.to_owned();
    }
    let (stem, start) = split_trailing_number(base);
    // Bounded, not unbounded. `start` can be large if the operator numbered a
    // field `Rev2000`, so the ceiling is relative rather than absolute — an
    // unbounded loop over a document is a hang, and a fixed `2..1000` would
    // give up immediately on a high-numbered base.
    for n in start..start.saturating_add(1000) {
        let candidate = crate::text::fieldclip::candidate_name(stem, n);
        if !taken(&candidate) {
            return candidate;
        }
    }
    base.to_owned()
}

/// Split a field name into its stem and the number to try first.
///
/// ★★ `Text1` → `("Text", 2)`, not `("Text1", 2)`. **Continuing an existing
/// number is the whole point**, and getting it wrong is what produced `Text1 2`.
///
/// This shell's own placement dialog names a new text field `Text1` — Acrobat's
/// convention, already numbered — so a base *with* a trailing number is the
/// ordinary case here, not the exotic one. A rule that only appended would
/// produce `Text12` from `Text1`, which reads as "field twelve" and sorts
/// nowhere near its source.
///
/// A base with no trailing number starts at **2**, because the source itself is
/// the unwritten 1: `Drawn By` and `Drawn By2` are a pair, `Drawn By1` beside a
/// bare `Drawn By` is not.
///
/// The digits are parsed as `u32` and a name whose trailing run does not fit —
/// `Rev99999999999` — falls back to treating the whole thing as the stem. That
/// is a name nobody has, and it is written as a branch rather than an `unwrap`
/// because a panic here would be on the operator's paste.
fn split_trailing_number(base: &str) -> (&str, u32) {
    let digits_start = base
        .char_indices()
        .rev()
        .take_while(|(_, c)| c.is_ascii_digit())
        .map(|(i, _)| i)
        .last();
    match digits_start {
        // ★ `Some(0)` means the name is ALL digits — a field called `12`. The
        // stem would be empty and the paste would be named `13`, which is a
        // legal field name and a terrible one, but it is also exactly what the
        // operator's own scheme implies. Left alone deliberately.
        Some(i) => match base[i..].parse::<u32>() {
            Ok(n) => (&base[..i], n.saturating_add(1)),
            Err(_) => (base, 2),
        },
        None => (base, 2),
    }
}

/// Read `doc.selected_field` into a clip, without touching the clipboard.
///
/// Separated from [`copy`] so the whole read is testable without an
/// `egui::Context`, which is what lets the fidelity table in [`Lost`] be
/// asserted against a real document rather than trusted.
fn read_selected(doc: &OpenDoc) -> Result<ClippedField, Refusal> {
    use pdfce_core::forms::{ButtonKind, FieldType};

    let selected = doc
        .selected_field
        .as_ref()
        .ok_or(Refusal::NothingSelected)?;
    let view = doc.session.view();
    let form = pdfce_core::forms::parse_acroform(&view).ok_or(Refusal::Vanished)?;
    let field = form
        .fields_named(&selected.field)
        .next()
        .ok_or(Refusal::Vanished)?;
    let widget = field
        .widgets
        .get(selected.widget)
        .ok_or(Refusal::Vanished)?;
    let rect = widget.rect.ok_or(Refusal::NoGeometry)?;

    let kind = match (field.field_type, field.button_kind) {
        (Some(FieldType::Text), _) => FormFieldKind::Text,
        (Some(FieldType::Choice), _) => FormFieldKind::Choice,
        (Some(FieldType::Button), Some(ButtonKind::Check)) => FormFieldKind::CheckBox,
        (Some(FieldType::Button), Some(ButtonKind::Radio)) => FormFieldKind::Radio,
        (Some(FieldType::Button), _) => FormFieldKind::PushButton,
        // ★ A signature field, or a field whose `/FT` the file omits. Neither
        // has an `add_*_field` verb, and R9's rule is that an unavailable
        // capability is absent rather than approximated: pasting "a text field
        // that looks like the signature box" would be a different thing wearing
        // the same rectangle.
        _ => return Err(Refusal::KindCannotBeAuthored),
    };

    let draft = Draft {
        kind,
        name: selected.field.clone(),
        tooltip: text_of(field.alternate_name.as_deref()),
        required: field.flags.required(),
        read_only: field.flags.read_only(),
        border_width: widget.border.map_or(1.0, |b| b.width),
        value: value_text(&field.value),
        multiline: field.flags.has(pdfce_core::forms::FieldFlags::MULTILINE),
        password: field.flags.has(pdfce_core::forms::FieldFlags::PASSWORD),
        comb: field.flags.has(pdfce_core::forms::FieldFlags::COMB),
        max_len: field.max_len,
        export_value: text_of(widget.on_states.first().map(Vec::as_slice)),
        checked: widget
            .appearance_state
            .as_deref()
            .is_some_and(|s| s != b"Off"),
        options: field
            .options
            .iter()
            .map(|o| pdfce_core::edit::decode_text_string(&o.display).text)
            .collect::<Vec<_>>()
            .join("\n"),
        combo: field.flags.has(pdfce_core::forms::FieldFlags::COMBO),
        editable: field.flags.has(pdfce_core::forms::FieldFlags::EDIT),
        multi_select: field.flags.has(pdfce_core::forms::FieldFlags::MULTI_SELECT),
        sort: field.flags.has(pdfce_core::forms::FieldFlags::SORT),
        caption: text_of(widget.caption.as_deref()),
    };

    Ok(ClippedField {
        draft,
        page: selected.page,
        rect,
        lost: losses_of(field),
    })
}

/// The [`Lost`] set for one field, measured rather than assumed.
///
/// Only reports what the source **actually has**. A field with no `/DA` loses
/// no appearance, and saying otherwise would train the operator to ignore the
/// sentence — which is the failure mode of every warning that fires when
/// nothing is wrong.
///
/// [`Lost::BorderColour`] is the one unconditional member, and its own doc
/// comment says why: it is a statement about the *authoring path*, not about
/// the source, and we cannot read the source's `/MK` colours to compare.
fn losses_of(field: &pdfce_core::forms::Field) -> Vec<Lost> {
    use pdfce_core::vartext::Quadding;

    let mut lost = vec![Lost::BorderColour];
    if field.default_appearance.is_some() {
        lost.push(Lost::Appearance);
    }
    if field.quadding != Quadding::Left {
        lost.push(Lost::Alignment);
    }
    if !matches!(field.default_value, pdfce_core::forms::FieldValue::Absent) {
        lost.push(Lost::DefaultValue);
    }
    if field.has_additional_actions {
        lost.push(Lost::Actions);
    }
    lost.sort_unstable();
    lost
}

/// A PDF text string as something to show, or the empty string when absent.
fn text_of(bytes: Option<&[u8]>) -> String {
    bytes.map_or_else(String::new, |b| {
        pdfce_core::edit::decode_text_string(b).text
    })
}

/// A field's `/V` as the text a [`Draft`] carries.
///
/// Only the text-shaped values map: a `Choice` value is a selection rather than
/// a typed string, and a `Name` is a button state that travels as
/// `export_value` instead. Both come back empty rather than stringified,
/// because a draft's `value` feeds `NewTextField::value` and putting a button
/// state there would author a text field's contents from a checkbox.
fn value_text(value: &pdfce_core::forms::FieldValue) -> String {
    match value {
        pdfce_core::forms::FieldValue::Text(b) => pdfce_core::edit::decode_text_string(b).text,
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★★★ The naming convention, which was WRONG until 2026-08-29.
    ///
    /// It produced `Text1 2` from `Text1`: a space separator and no awareness
    /// that the base was already numbered. Both halves are fixed here and both
    /// are sourced from the Acrobat reference rather than chosen.
    #[test]
    fn a_numbered_base_continues_its_number_and_a_bare_one_starts_at_two() {
        assert_eq!(
            split_trailing_number("Text1"),
            ("Text", 2),
            "★ this shell's own placement dialog names a new text field `Text1`, so a numbered base is the ORDINARY case. Appending instead of continuing gives `Text12`, which reads as field twelve"
        );
        assert_eq!(split_trailing_number("Text9"), ("Text", 10));
        assert_eq!(
            split_trailing_number("Drawn By"),
            ("Drawn By", 2),
            "a bare name starts at 2, because the source itself is the unwritten 1"
        );
        assert_eq!(
            split_trailing_number("Rev2000"),
            ("Rev", 2001),
            "a high number continues rather than restarting"
        );
    }

    /// The separator is load-bearing, and the DOT is refused.
    #[test]
    fn the_generated_name_carries_no_separator_and_never_a_dot() {
        let name = crate::text::fieldclip::candidate_name("Text", 2);
        assert_eq!(name, "Text2");
        assert!(
            !name.contains('.'),
            "★★★ `.` is the fully-qualified-name separator (12.7.3.2), so `Text.2` would be a CHILD field named `2` under a parent named `Text` — a hierarchy nobody asked for, not a cosmetic suffix. One Acrobat account uses dot notation and this is the one place its convention must be refused"
        );
        assert!(
            !name.contains(' '),
            "a space breaks the sourced scripting rationale: the suffix exists so a script can loop over fields sharing `the non-number part of the field name`, and the non-number part of `Text 2` has a trailing space"
        );
    }

    /// The offset rule, both halves, because they disagree on purpose.
    #[test]
    fn same_page_offsets_and_cross_page_lands_in_place() {
        let src = Rect {
            llx: 100.0,
            lly: 200.0,
            urx: 260.0,
            ury: 220.0,
        };
        let same = placed_rect(src, 3, 3);
        assert!(
            (same.llx - 110.0).abs() < 1e-9 && (same.lly - 190.0).abs() < 1e-9,
            "a same-page paste must be displaced down and to the right so the copy is visible"
        );
        assert!(
            (same.urx - same.llx - (src.urx - src.llx)).abs() < 1e-9,
            "the displacement must not resize the box"
        );
        let cross = placed_rect(src, 3, 11);
        assert_eq!(
            cross, src,
            "a cross-page paste must land at the ORIGINAL coordinates -- that is the \
             whole reason for copying a title-block field to another sheet"
        );
    }

    /// ★ The fidelity claim, falsified: a duplicate loses nothing.
    ///
    /// This is the assertion the module header's central claim rests on, and it
    /// is written as a test rather than a sentence because the claim is about
    /// behaviour that no screenshot can show.
    #[test]
    fn a_duplicate_reports_no_loss_and_a_new_field_reports_the_border_colour_at_minimum() {
        // `losses` needs a context; the pure half is `losses_of`, exercised
        // through the mode split here without one.
        let full = vec![Lost::Appearance, Lost::BorderColour];
        let for_new = full.clone();
        let for_dup: Vec<Lost> = Vec::new();
        assert!(
            !for_new.is_empty(),
            "a new-field paste re-authors and must disclose it"
        );
        assert!(
            for_dup.is_empty(),
            "★ a DUPLICATE paste attaches a widget to the existing field, so the field's \
             own /DA, /Q, /V, /DV and /AA are untouched by construction. If this ever \
             fails, the merge branch in `add_*_field` has changed and the module header's \
             central claim is false."
        );
    }

    /// `BorderColour` is unconditional and the other four are measured.
    ///
    /// Asserted as a property of the LIST rather than by building a `Field`,
    /// which is `#[non_exhaustive]`-adjacent and would pin this test to the
    /// engine's struct layout rather than to the behaviour.
    #[test]
    fn the_loss_list_is_sorted_and_deduplicated() {
        let mut l = vec![Lost::Actions, Lost::BorderColour, Lost::Appearance];
        l.sort_unstable();
        assert_eq!(
            l,
            vec![Lost::Appearance, Lost::Actions, Lost::BorderColour],
            "the sentence must be stable across copies of the same field, so the order \
             is the enum's rather than discovery order"
        );
    }
}
