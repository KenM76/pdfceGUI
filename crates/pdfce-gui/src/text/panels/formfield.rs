//! # `text::panels::formfield` — the words in the form-field properties section
//!
//! Covers [`crate::panels::properties::formfield`], the section that appears
//! when an operator clicks a form field on the page in Edit mode.
//!
//! ## ★★★ The sentence this file exists for
//!
//! [`not_editable_note`]. `pdfce-core` has four verbs for an existing field —
//! rename, delete, delete-a-widget, fill — and **none** for its flags. Required,
//! read-only, multiline, comb, the border and the tooltip are settable only when
//! the field is created.
//!
//! A panel that showed those as facts and offered no way to change them would
//! read as unfinished, and an operator would spend real time looking for the
//! control. So the limit is **stated**: what cannot be changed, and what to do
//! instead. Saying it costs one line; not saying it costs a search that ends in
//! the operator concluding the program is broken.
//!
//! ★★ It is worded as a **statement about pdfce**, not about PDF. The format
//! permits changing every one of them; it is this engine that has no verb yet.
//! Blaming the format would be a false claim, and the kind that is never
//! corrected because nobody can check it.
//!
//! ## Vocabulary
//!
//! The same rule [`crate::text::formfield`]'s header sets: the word the
//! operator's other programs use, not the spec's. A `/Ch` is a drop-down, a
//! `/Btn` is a check box or a button, `/TU` is a tooltip. The one place the
//! spec's vocabulary survives is the word **box** for a widget, because there
//! is no better one — "widget" is jargon and "annotation" is wrong.

use pdfce_core::forms::{Field, FieldType};

/// The section heading.
#[must_use]
pub fn heading() -> String {
    "Form field".to_owned()
}

/// The label for the field's name.
#[must_use]
pub fn label_name() -> String {
    "Name".to_owned()
}

/// The label for the field's type.
#[must_use]
pub fn label_type() -> String {
    "Type".to_owned()
}

/// The label for the page the clicked box is on.
#[must_use]
pub fn label_page() -> String {
    "Page".to_owned()
}

/// The label for the box count.
#[must_use]
pub fn label_boxes() -> String {
    "Boxes".to_owned()
}

/// The label for the field's current value.
#[must_use]
pub fn label_value() -> String {
    "Value".to_owned()
}

/// The label for the set flags.
#[must_use]
pub fn label_flags() -> String {
    "Set".to_owned()
}

/// A 1-based page number.
///
/// ★ 1-based, because that is what the page strip shows and what the operator
/// would say out loud. Every 0-based index in this shell stops at the boundary
/// with the person using it.
#[must_use]
pub fn page_number(one_based: usize) -> String {
    format!("{one_based}")
}

/// How many boxes the field is drawn as, and which one was clicked.
///
/// ★★ Shown only when there is more than one, and it is a **disclosure** rather
/// than a statistic: a field drawn in three places can be changed from three
/// pages, and the two the operator is not looking at change with it. Nothing on
/// the page says so.
#[must_use]
pub fn box_count(total: usize, clicked: usize) -> String {
    format!("{total} — you clicked #{}", clicked.saturating_add(1))
}

/// What kind of control this is, in the operator's vocabulary.
///
/// ★ A `/Btn` is three different controls and the spec tells them apart by
/// flag bits, not by type. Reporting all three as "button" would be accurate
/// about the format and useless to a person: a check box and a push button have
/// nothing in common from where they sit.
#[must_use]
pub fn field_type(field: &Field) -> String {
    use pdfce_core::forms::ButtonKind;
    match field.field_type {
        Some(FieldType::Text) => "Text field".to_owned(),
        Some(FieldType::Button) => match field.button_kind {
            Some(ButtonKind::Check) => "Check box".to_owned(),
            Some(ButtonKind::Radio) => "Radio button".to_owned(),
            Some(ButtonKind::Push) => "Button".to_owned(),
            None => "Button".to_owned(),
        },
        Some(FieldType::Choice) => "Drop-down list".to_owned(),
        Some(FieldType::Signature) => "Signature".to_owned(),
        // ★★ Not "unknown" and not blank. A field with no `/FT` is a real and
        // specific defect — no viewer knows how to fill it — and it is exactly
        // the shape `EditSession::adopt_widget` produces from a bare kid that
        // lost its `/Parent`. Saying so here is the same disclosure
        // `text::status::adopted` makes at the moment one is created.
        None => "No type — no viewer knows how to fill it".to_owned(),
    }
}

/// The field's current value, if it has one worth showing.
///
/// `None` for an empty field, which draws no row at all rather than an empty
/// one — R9 applied to a fact instead of to a control.
#[must_use]
pub fn field_value(field: &Field) -> Option<String> {
    use pdfce_core::forms::FieldValue;
    if matches!(field.value, FieldValue::Absent) {
        return None;
    }
    // ★★ `display_text` and not a decode of our own. A `/V` is raw bytes, and
    // turning them into characters is §7.9.2 / Annex D.3 text-string decoding —
    // PDFDocEncoding or UTF-16BE with a BOM, not UTF-8. `String::from_utf8_lossy`
    // over a UTF-16 value produces interleaved NULs and replacement characters,
    // which is a wrong answer that looks like a corrupt document rather than
    // like a bug in this panel.
    //
    // The engine's own helper is explicit that it is for display and that
    // export uses the raw bytes, which is exactly this use.
    let text = field.value.display_text();
    (!text.trim().is_empty()).then_some(text)
}

/// The flags that are set, named, or `None` when none are.
///
/// ★ Only the ones that are **set**. A list of every flag with yes/no beside it
/// would be six rows of "No" on a typical field, and the reader has to search
/// it to learn anything. Naming the exceptions is what a person would do.
#[must_use]
pub fn field_flags(field: &Field) -> Option<String> {
    use pdfce_core::forms::FieldFlags;
    let f = field.flags;
    let mut set: Vec<&str> = Vec::new();
    if f.read_only() {
        set.push("read-only");
    }
    if f.required() {
        set.push("required");
    }
    if f.no_export() {
        set.push("not exported");
    }
    // ★ The type-specific bits are read only for the type they belong to,
    // because the SAME BIT means different things on different field types —
    // bit 18 is `Edit` on a choice and `DoNotSpellCheck`-adjacent territory
    // elsewhere. Reporting a text flag on a check box would be inventing a
    // property from a bit that is not about it.
    if matches!(field.field_type, Some(FieldType::Text)) {
        if f.has(FieldFlags::MULTILINE) {
            set.push("multi-line");
        }
        if f.has(FieldFlags::PASSWORD) {
            set.push("hidden as typed");
        }
        if f.has(FieldFlags::COMB) {
            set.push("equal cells");
        }
    }
    if matches!(field.field_type, Some(FieldType::Choice)) {
        if f.has(FieldFlags::COMBO) {
            set.push("drop-down");
        }
        if f.has(FieldFlags::EDIT) {
            set.push("free text allowed");
        }
        if f.has(FieldFlags::MULTI_SELECT) {
            set.push("multi-select");
        }
    }
    (!set.is_empty()).then(|| set.join(", "))
}

/// The label above the rename box.
///
/// ★ It asks for the **short** name and says so, because `rename_field` takes a
/// partial name and rebuilds the qualified one from the parent chain. An
/// operator who copied `Address.Line1` out of the row above and pasted it here
/// would author a `/T` containing a dot, which nothing can address again.
#[must_use]
pub fn rename_label() -> String {
    "Rename — its short name, with no dots".to_owned()
}

/// The rename button.
#[must_use]
pub fn rename_button() -> String {
    "Rename".to_owned()
}

/// Why Rename is greyed.
#[must_use]
pub fn rename_disabled() -> String {
    "Type a name with no dots in it. A dot separates a field from its parent, \
     so a name containing one cannot be addressed."
        .to_owned()
}

/// The delete-the-field button.
#[must_use]
pub fn delete_field() -> String {
    "Delete field".to_owned()
}

/// What deleting the field will take with it.
///
/// ★★ Names the count in the hover rather than after the fact, because that is
/// where it can still change the operator's mind. A confirmation that said
/// "deleted from 3 pages" afterwards is a report; this is a warning.
#[must_use]
pub fn delete_field_hover(widgets: usize) -> String {
    if widgets <= 1 {
        "Removes the field from the form and the box from the page.".to_owned()
    } else {
        format!(
            "Removes the field from the form and all {widgets} of its boxes, on \
             every page they are drawn on."
        )
    }
}

/// The delete-this-box button.
#[must_use]
pub fn delete_box() -> String {
    "Delete this box".to_owned()
}

/// What deleting one box does, and the case where it does more.
#[must_use]
pub fn delete_box_hover() -> String {
    "Removes only the box you clicked. The field stays in the form, drawn \
     wherever else it appears."
        .to_owned()
}

/// ★★★ **What cannot be changed after a field is placed, and what to do
/// instead.**
///
/// The sentence this file exists for. See the header: `pdfce-core` has no verb
/// for a field's flags, so this panel can show them and not change them, and an
/// absence with no explanation is indistinguishable from an oversight.
///
/// It names the remedy — delete and place again — because that remedy actually
/// works and takes about five seconds, which is worth knowing before spending a
/// minute hunting for a control that is not there.
#[must_use]
pub fn not_editable_note() -> String {
    "Required, read-only, the tooltip and the border can only be set when a \
     field is placed. To change one, delete this field and place a new one."
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★★ **The limitation note names both halves: what is fixed, and the way
    /// round it.**
    ///
    /// A note that only said "cannot be changed" would leave the operator
    /// stuck; one that only said "delete and replace" would not say why. Tested
    /// because the tempting edit is to shorten it to the first half.
    #[test]
    fn the_limitation_note_names_the_remedy_as_well_as_the_limit() {
        let note = not_editable_note();
        assert!(note.contains("only be set when"), "the limit: {note}");
        assert!(note.contains("delete this field"), "the remedy: {note}");
    }

    /// ★★ **A field with no type is described as a defect, not as "unknown".**
    ///
    /// A `/FT`-less field is what a bare kid that lost its `/Parent` becomes,
    /// and no viewer can fill it. "Unknown" would read as pdfce failing to
    /// look; this says what is true of the document.
    #[test]
    fn a_typeless_field_is_described_as_unfillable() {
        let mut field = sample();
        field.field_type = None;
        let described = field_type(&field);
        assert!(
            described.contains("No type"),
            "must name the defect: {described}"
        );
        assert!(
            !described.to_lowercase().contains("unknown"),
            "\u{201c}unknown\u{201d} blames pdfce for a property of the file: {described}"
        );
    }

    /// **A `/Btn` is told apart into its three real controls**, because the
    /// spec's one type is three different things to a person.
    #[test]
    fn the_three_button_kinds_are_named_separately() {
        use pdfce_core::forms::ButtonKind;
        let mut described = Vec::new();
        for kind in [ButtonKind::Check, ButtonKind::Radio, ButtonKind::Push] {
            let mut field = sample();
            field.field_type = Some(FieldType::Button);
            field.button_kind = Some(kind);
            described.push(field_type(&field));
        }
        let before = described.len();
        described.sort();
        described.dedup();
        assert_eq!(before, described.len(), "two button kinds share a name");
    }

    /// **A field with nothing set shows no flags line at all** — R9 applied to
    /// a fact: an empty row is worse than no row.
    #[test]
    fn no_flags_means_no_line() {
        assert_eq!(field_flags(&sample()), None);
    }

    /// **The box count discloses the click as well as the total**, because a
    /// field drawn three times can be changed from any of them.
    #[test]
    fn the_box_count_says_which_one_was_clicked() {
        let line = box_count(3, 1);
        assert!(line.contains('3'), "the total: {line}");
        assert!(
            line.contains("#2"),
            "1-based, as the operator counts: {line}"
        );
    }

    /// A plain text field with nothing set.
    fn sample() -> Field {
        use pdfce_core::forms::{FieldFlags, FieldValue};
        use pdfce_core::object::ObjId;
        use pdfce_core::vartext::Quadding;
        Field {
            id: ObjId::new(1, 0),
            fully_qualified_name: "Name".to_owned(),
            partial_name: None,
            alternate_name: None,
            mapping_name: None,
            rich_value: None,
            default_style: None,
            field_type: Some(FieldType::Text),
            button_kind: None,
            flags: FieldFlags(0),
            value: FieldValue::Absent,
            default_value: FieldValue::Absent,
            default_appearance: None,
            quadding: Quadding::Left,
            max_len: None,
            options: Vec::new(),
            top_index: 0,
            selected_indices: Vec::new(),
            widgets: Vec::new(),
            merged: false,
            has_additional_actions: false,
            shares_parent_name: false,
            parent: None,
        }
    }
}
