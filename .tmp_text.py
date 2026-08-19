p='crates/pdfce-gui/src/text/forms.rs'
s=open(p,encoding='utf-8').read()

old = '''/// Widgets on this page that no listed field claims.
///
/// The final clause points at [`forms_inline_field_roots_note`] without
/// re-counting it: a `/Fields` entry written as a direct dictionary is skipped
/// by the parser, so its widgets arrive here unowned. Two numbers about two
/// things, related out loud rather than added together.
#[must_use]
pub fn tab_order_unclaimed(count: usize) -> String {
    format!(
        "⚠ {count} widget(s) on this page belong to no field this form lists, so no row can name \
         a field for them. If the form declares entries pdfce could not read, these may be theirs."
    )
}'''

new = '''/// Widgets on this page that no listed field claims — the heading over the
/// rows that can now do something about it.
///
/// # ★ This sentence used to end in a guess, and the guess has been replaced
/// by a route
///
/// The old wording finished *"if the form declares entries pdfce could not
/// read, these may be theirs"* — a speculation offered because there was
/// nothing better to offer. Nothing could be done with an unclaimed widget, so
/// the only honest thing left was to speculate about where it came from.
///
/// `EditSession::adopt_widget` shipped 2026-08-19, so an unclaimed widget is
/// now a **chore with a button** rather than a curiosity with a theory. The
/// sentence says what the boxes are and what registering does; the rows below
/// it do the rest.
///
/// The inline-field-roots note is still one line away in
/// [`forms_inline_field_roots_note`] and is still not re-counted here — two
/// numbers about two things, related out loud rather than added together.
///
/// # Why it says "cannot be filled" rather than "are broken"
///
/// Because that is the operator-visible fact and it is the one that surprises.
/// The box **draws**. It has a border, it has a background, it looks exactly
/// like the field beside it. What it does not have is a name any filling verb
/// can address, so clicking it and typing produces nothing and no message.
/// This project's own recurring failure — a visible control that is silently
/// inert — arriving through a document rather than through a ribbon.
#[must_use]
pub fn tab_order_unclaimed(count: usize) -> String {
    let boxes = if count == 1 {
        "1 box on this page is drawn as a form control that no field claims"
    } else {
        return format!(
            "{count} boxes on this page are drawn as form controls that no field claims. \
             They cannot be filled until they are registered."
        );
    };
    format!("{boxes}. It cannot be filled until it is registered.")
}

/// One unclaimed widget's row: where it sits in the tab sequence.
///
/// The position is what an operator uses to **find** it — press Tab that many
/// times and watch the focus ring land — which is the only handle they have,
/// because the thing has no name by definition.
#[must_use]
pub fn tab_order_unclaimed_row(position: usize) -> String {
    format!("Box {position} in the tab order")
}

/// The hint over the name box beside an unclaimed widget.
///
/// ★ Says what an empty box means, because empty is the common and correct
/// answer and a blank field with no hint reads as "required".
///
/// Most unclaimed widgets are **merged field-widgets** (§12.7.3.1): one
/// dictionary serving as both, carrying its own `/T`, `/FT` and `/V`. The
/// engine measured a real form and found 11 of 13 in that shape. For those,
/// registering with no name recovers the field exactly as it was — the name is
/// already in the file and typing one would *override* it.
#[must_use]
pub const fn tab_order_register_name_hint() -> &'static str {
    "Name — leave blank to keep the name the box already carries"
}

/// The button that registers one unclaimed widget.
#[must_use]
pub const fn tab_order_register() -> &'static str {
    "Register";
}'''
assert s.count(old)==1
s=s.replace(old,new,1)
open(p,'w',encoding='utf-8',newline='').write(s)
print("ok")
