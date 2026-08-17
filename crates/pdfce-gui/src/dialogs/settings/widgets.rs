//! # `dialogs::settings::widgets` — the three shapes every setting is made of
//!
//! Seven group modules draw thirteen settings, and every one of them is built
//! from the three functions here. That is deliberate: a settings window whose
//! entries are hand-laid-out drifts into thirteen slightly different layouts
//! within a year, and the reader notices the inconsistency before they notice
//! the content.
//!
//! ## ★ [`header`]'s signature is where obligation 2 and 3 are enforced
//!
//! `crate::dialogs::settings`' header names three things this window must show
//! that a conventional settings screen omits. Two of them are properties of
//! *every* setting, and rather than trusting each group module to remember
//! them, they are **required arguments**:
//!
//! ```text
//! header(ui, title, silence, radius)
//!               │       │       └── which way costs what
//!               │       └────────── what the standard leaves open
//!               └────────────────── what the setting is
//! ```
//!
//! A setting cannot be added without answering all three, because the code
//! does not compile otherwise. `crate::text::settings` mirrors the shape —
//! `*_title`, `*_silence`, `*_radius` for all thirteen — and its own tests
//! assert none of the answers is empty.
//!
//! Obligation 1 — *whether the default is a guess* — is **not** enforceable
//! this way: it belongs to one option rather than to the setting, and only
//! some options have anything to say. It is pinned by a test over the catalog
//! instead.

use egui::{RichText, Ui};

/// One collapsible subject group.
///
/// ## ★ Plain text, not `.strong()` — `DEFECTS.md` D11
///
/// Both this and [`header`] used `RichText::strong()` in their first draft, and
/// both were **near-invisible on screen**: pale grey on pale grey, while the
/// radio labels under them read normally. Found by capturing the running
/// window, which is the only oracle for this class of defect and is why the
/// check that opens this dialog exists.
///
/// The mechanism is `egui`'s, and D11 sets it out: there is **no separate role
/// for emphasised text** — `strong_text_color()` returns
/// `widgets.active.fg_stroke`, the foreground of the *accent-filled* state. In
/// any theme whose active state is accent-filled, which is all three of this
/// project's, `.strong()` on an ordinary panel resolves to a colour chosen to
/// sit on the accent. It also survives `override_text_color`.
///
/// D11 states the rule — *"do not use `RichText::strong()` in this
/// application"* — and prescribes the fix five other panels already took:
/// render as plain text, because *"the emphasis they were asking for was
/// invisible"*. The hierarchy that emphasis was reaching for is still there and
/// is carried by layout rather than by weight: a heading has a disclosure
/// triangle beside it, and a setting's title is the only line of the three that
/// is **not** `.small().weak()`.
///
/// The two legitimate uses in the workspace both take the colour back
/// explicitly on the next line — `egui-shell`'s ribbon and dock tab labels,
/// which are drawn on an accent fill and pair `.strong()` with
/// `.color(palette.on_accent)`. That pairing is what
/// `tools/gates/check-strong-text.sh` allows and a bare `.strong()` is what it
/// refuses, so this cannot be got wrong a third time by remembering.
///
/// ## The rest of this control
///
/// A `CollapsingHeader` rather than a `heading` size: this window is a list of
/// thirteen things and a true heading at each of seven would make it read as
/// seven documents. `default_open` is passed rather than remembered, because
/// which group is expanded is a statement about which symptom is most likely —
/// see the module header — and not a preference of the operator's to be
/// persisted.
/// `key` is the stable identifier the heading's rect is published under —
/// `settings.heading.<key>` — and it is deliberately **not** derived from the
/// caption. The caption is operator copy and may be reworded or translated; a
/// check aimed at a region named after it would then silently stop finding its
/// subject and report a heading that is not there rather than a heading that is
/// illegible. Those are different verdicts and only one of them is true.
pub fn group(
    ui: &mut Ui,
    key: &str,
    heading: &str,
    open_by_default: bool,
    body: impl FnOnce(&mut Ui),
) {
    let response = egui::CollapsingHeader::new(RichText::new(heading))
        .default_open(open_by_default)
        .show(ui, body);
    // ★ The HEADER's rect, not the whole collapsible's.
    //
    // `CollapsingHeaderResponse::header_response` is the row carrying the text;
    // the outer rect would include the expanded body, and a contrast check
    // measuring that would sample a hundred lines of prose and average the
    // heading away. D2 was a defect in one row of pixels, and it measured about
    // 1.1:1 — a figure only obtainable from the row itself.
    crate::diag::ui_rect(
        &format!("{}{key}", super::REGION_HEADING_PREFIX),
        response.header_response.rect,
    );
    ui.add_space(2.0);
}

/// One setting's three lines: what it is, what is open, and what it costs.
///
/// Always in this order, and the order is the argument the window makes. The
/// operator reads *what this is*, then *why they are being asked* — which is
/// the sentence that stops a pdfce/Acrobat difference being read as a bug —
/// and then *what changing it will do to their file*, which is the one they
/// need before touching a radio rather than after.
///
/// `.small().weak()` for the second and third: they are context for the choice
/// rather than the choice, and at the same weight as the title they would make
/// every setting look like three settings.
///
/// ★ The title is **plain text**, not `.strong()` — see [`group`] for the
/// screenshot that found the difference and `DEFECTS.md` D11 for why no theme
/// this project ships can render `.strong()` legibly on a panel. Being the only
/// one of the three lines that is not small and weak is the whole of its
/// emphasis, and it is enough.
pub fn header(ui: &mut Ui, title: &str, silence: &str, radius: &str) {
    ui.label(RichText::new(title));
    ui.label(RichText::new(silence).small().weak());
    ui.label(RichText::new(radius).small().weak());
    ui.add_space(2.0);
}

/// One radio option, with an optional gloss under it.
///
/// # Why the note is an `Option`
///
/// A few labels are self-describing — *"Carriage return then newline"* needs
/// no gloss — and padding them out to match their neighbours would be noise.
/// The rule this window inherits about tooltips applies one layer down: text
/// that says nothing trains the reader to stop reading the text that does.
///
/// Exactly two of the thirteen settings' options pass `None`, and both are in
/// the *Saving files* group where the label names a byte sequence.
pub fn option<T: PartialEq>(
    ui: &mut Ui,
    current: &mut T,
    value: T,
    label: &str,
    note: Option<&str>,
) {
    ui.radio_value(current, value, label);
    if let Some(note) = note
        && !note.is_empty()
    {
        ui.label(RichText::new(note).small().weak());
    }
}

/// One switch, with an optional gloss under it.
///
/// # ★ The fourth shape, and why a two-option radio group was refused
///
/// This module's header opens *"the three shapes every setting is made of"*,
/// and a fourth arriving needs a better reason than convenience. It has one: a
/// **switch is not a choice between named alternatives**.
///
/// [`option`] draws a radio, which is the right control when the operator is
/// picking one of several *named things* — `Nearest sample`, `Average the
/// area` — and the names carry the content of the choice. A visibility toggle
/// has no such names. Rendering it as a radio group would mean inventing the
/// pair *"Shown" / "Hidden"*, which says nothing the checkbox's own label does
/// not, and it would draw **six** controls for the three overlays where three
/// belong. Worse, three adjacent two-radio groups read as though the six were
/// somehow related — a reader scanning them has to work out that they are three
/// independent switches and not one six-way choice.
///
/// # Why the label is on the checkbox rather than in a [`header`]
///
/// Because these are the sub-parts of **one** setting rather than settings in
/// their own right. The Drawing-the-page group's overlay control has a single
/// header — one title, one silence line, one radius line — and three switches
/// under it, because the three interlock: a guide is dragged out of a ruler, so
/// switching guides on without rulers places nothing. Giving each its own
/// header would print that explanation three times, or once, in a place two of
/// the three readers would not look.
///
/// `note` is `Option` for the same reason it is on [`option`]: a label that
/// needs no gloss should not get a padded one, because text that says nothing
/// trains the reader to stop reading the text that does.
pub fn toggle(ui: &mut Ui, value: &mut bool, label: &str, note: Option<&str>) {
    ui.checkbox(value, label);
    if let Some(note) = note
        && !note.is_empty()
    {
        ui.label(RichText::new(note).small().weak());
    }
}

/// A sentence the operator must see but that belongs to the **setting**, not to
/// any one of its options.
///
/// `.small()` and deliberately **not** `.weak()`, which is the whole point of
/// its existing separately from [`option`]'s note. There are exactly three of
/// these in the window and each is a disclosure rather than a description:
///
/// - the CMYK intent group's *"pdfce's default deliberately differs from
///   Acrobat here"*, which the person reading that radio group is precisely the
///   person who needs;
/// - the replacement-text group's bound, which applies **whichever option is
///   chosen** and would be misread as an argument for one of them if it sat
///   inside a note;
/// - the unknown-theme sentence, which explains why none of the three radios is
///   selected.
///
/// Weak-grey is for context. A disclosure that pdfce owes the operator is not
/// context, and greying it would be the quiet version of not saying it.
pub fn disclosure(ui: &mut Ui, text: &str) {
    ui.add_space(2.0);
    ui.label(RichText::new(text).small());
}
