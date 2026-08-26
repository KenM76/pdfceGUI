//! # `canvas::formfield` — placing a new form field on the page
//!
//! **Operator request, 2026-08-26:**
//!
//! > *"get all the form buttons on the ribbon working next along with adding
//! > all the form feature buttons. when I click one I should be able to click
//! > on the canvas to place the position or drag a box for size then a pop up
//! > lets me set the details for the feature."*
//!
//! That is exactly the interaction the existing command's own tooltip has
//! promised since it was written — *"Click where you want it, or drag out the
//! exact size."* The design was specified and never built.
//!
//! ## ★★★ Why it was never built, and why that reason was wrong
//!
//! `shell::commands::reach::register` recorded `edit.form_create_field` as
//! blocked on *"core's STRUCTURAL certification gate"*. **There is no such
//! gate.** Probed on 2026-08-26 against a real drawing:
//! `EditSession::add_text_field` authors a field and returns its id.
//!
//! What the engine refuses is a spec whose tooltip is `Undecided` —
//! `TooltipDecisionRequired`, an **accessibility** requirement rather than a
//! permission. A form control owes a screen reader a name and the engine will
//! not invent one silently. So the entire blocker is a field of the very dialog
//! this feature needs. `app::actions::forms::authoring_is_available` is the
//! standing test, and it asserts both halves so it cannot rot into a
//! tautology.
//!
//! ★★ Fourth stale blocker in this project. The standing rule that produced the
//! probe: **a backlog row is a record, not evidence.**
//!
//! ## The shape, and why it borrows from markup rather than inventing
//!
//! Placing a field is *geometrically* the same act as drawing a markup
//! rectangle: arm a tool, put a rectangle on a page, commit once. So it reuses
//! that machinery rather than growing a second one —
//! [`crate::canvas::markup::band`]'s two-phase drag, the same page-space
//! conversion, the same single-`Action` release.
//!
//! It differs in exactly one way, and the difference is the feature: **the
//! release does not author anything.** It opens a dialog. Nothing exists in the
//! document until the operator presses OK, which is what makes Escape free and
//! what stops a mis-drag leaving a stray field behind.

pub mod draft;

pub use draft::{Draft, Remembered};

use crate::canvas::markup::MarkupKind;

/// The five kinds of form control pdfce can author.
///
/// ★ Exactly the five `pdfce-core` has verbs for — `add_text_field`,
/// `add_check_box`, `add_radio_button`, `add_choice_field`, `add_push_button`.
/// The list is not a design choice here and must not become one: a sixth entry
/// would be a button with nothing behind it, which is the placeholder R9
/// forbids.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FormFieldKind {
    /// A box the operator types into.
    Text,
    /// A single on/off box.
    CheckBox,
    /// One of a group, where choosing one clears the others.
    ///
    /// ★ The only kind whose meaning depends on a *group* rather than on the
    /// field alone: radio buttons that share a name are one control. The dialog
    /// therefore asks for the group name, and two radios placed with the same
    /// name become alternatives rather than two independent buttons.
    Radio,
    /// A drop-down or list of options.
    Choice,
    /// A button that performs an action.
    ///
    /// ★★ **Authored, but inert — and it is on the ribbon GREYED rather than
    /// absent**, on the operator's ruling of 2026-08-26.
    ///
    /// That is R9-correct rather than an exception to it. R9 reserves greying
    /// for a *temporarily* unavailable capability that is explained on hover,
    /// and this is precisely that: `add_push_button` works, so the control is
    /// authorable — what pdfce cannot yet do is **run** what a button would do,
    /// because it executes no PDF actions. A button placed today would look
    /// right and do nothing, and a control that silently does nothing is worse
    /// than one that says why it is unavailable.
    PushButton,
}

impl FormFieldKind {
    /// Every kind, in the order they appear on the ribbon.
    pub const ALL: [Self; 5] = [
        Self::Text,
        Self::CheckBox,
        Self::Radio,
        Self::Choice,
        Self::PushButton,
    ];

    /// Whether pdfce can do anything useful with this kind **once placed**.
    ///
    /// ★ Distinct from "can it be authored", and the distinction is the whole
    /// of the push button's greying: authoring works for all five. This asks
    /// whether the resulting control does anything, which for a push button
    /// means running a PDF action, and pdfce runs none.
    #[must_use]
    pub const fn is_useful_once_placed(self) -> bool {
        !matches!(self, Self::PushButton)
    }

    /// The command id that arms this kind.
    ///
    /// ★ Ids rather than a shared command with a parameter, because R8 says
    /// **registering a command is the only way the GUI learns a capability
    /// exists** — and it is what lets a build without one of these simply not
    /// register it, with the ribbon item disappearing rather than being
    /// special-cased.
    #[must_use]
    pub const fn command_id(self) -> &'static str {
        match self {
            Self::Text => "edit.form_text_field",
            Self::CheckBox => "edit.form_check_box",
            Self::Radio => "edit.form_radio_button",
            Self::Choice => "edit.form_choice",
            Self::PushButton => "edit.form_push_button",
        }
    }

    /// The default size, in points, for a field placed by a single **click**
    /// rather than by a drag.
    ///
    /// ★★ A click has to mean something, and a zero-sized field is not it. The
    /// numbers are per-kind because the kinds are not the same shape: a text
    /// box is wide and one line tall, and a check box is square. Sizing them
    /// alike would make every click need a resize afterwards, which defeats
    /// the point of offering a click at all.
    ///
    /// The text height is one line at the size a form typically uses, and the
    /// square kinds match it so that a check box beside a text field sits on
    /// the same baseline.
    #[must_use]
    pub const fn default_size_pt(self) -> (f64, f64) {
        match self {
            Self::Text | Self::Choice => (160.0, 20.0),
            Self::CheckBox | Self::Radio => (14.0, 14.0),
            Self::PushButton => (80.0, 22.0),
        }
    }

    /// What to call this kind in a sentence to the operator.
    ///
    /// ★ Returns the **text function**, not a string, so the words themselves
    /// stay in `crate::text` where `check-ui-strings.sh` can see them. The
    /// contrast with [`Self::name_prefix`] two functions down is the whole
    /// point and is easy to get backwards: that one is a PDF `/T` written into
    /// the file and must never be translated; this one is prose in a status
    /// line and must always be.
    #[must_use]
    pub fn noun(self) -> String {
        match self {
            Self::Text => crate::text::forms::form_noun_text(),
            Self::CheckBox => crate::text::forms::form_noun_check_box(),
            Self::Radio => crate::text::forms::form_noun_radio(),
            Self::Choice => crate::text::forms::form_noun_choice(),
            Self::PushButton => crate::text::forms::form_noun_push_button(),
        }
    }

    /// The stem an auto-generated field name is built from.
    ///
    /// ★★ **A PDF name, not UI copy, and the distinction is load-bearing** —
    /// which is why these are literals here rather than in `crate::text`. This
    /// string is written into the file as the field's `/T`, is what a
    /// form-filling script keys on, and is what an FDF import matches against.
    /// Translating it would rename every field in a document opened by an
    /// operator running a different language, and the renaming would be
    /// invisible until the data import failed.
    ///
    /// The prefixes are Acrobat's own, so a form authored here and a form
    /// authored there are named alike.
    #[must_use]
    pub const fn name_prefix(self) -> &'static str {
        match self {
            Self::Text => "Text", // ui-text-exempt: a PDF /T field-name stem written into the file
            Self::CheckBox => "Check Box", // ui-text-exempt: a PDF /T field-name stem written into the file
            Self::Radio => "Group",
            Self::Choice => "Dropdown",
            Self::PushButton => "Button",
        }
    }

    /// The markup kind whose drag machinery this borrows.
    ///
    /// Always a rectangle: every form control is a `/Rect`, and none of them
    /// has a second geometry. Returning it explicitly rather than assuming it
    /// at the call site keeps the borrowing visible.
    #[must_use]
    pub const fn drag_shape(self) -> MarkupKind {
        MarkupKind::Rectangle
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Every kind has a distinct command id**, because R8 makes the id the
    /// only route by which the ribbon learns the capability exists — two kinds
    /// sharing one would make a build that strips one strip both.
    #[test]
    fn each_kind_has_its_own_command() {
        for (i, a) in FormFieldKind::ALL.iter().enumerate() {
            for b in FormFieldKind::ALL.iter().skip(i + 1) {
                assert_ne!(a.command_id(), b.command_id(), "{a:?} and {b:?}");
            }
        }
    }

    /// **A click places something with real area**, for every kind.
    ///
    /// ★ The guard that stops a click producing an invisible field. A zero or
    /// negative default would author a control that exists in the document,
    /// cannot be seen, and cannot be clicked to select — the exact shape of the
    /// zero-height Large control this project shipped once before.
    #[test]
    fn a_click_places_something_with_area() {
        for k in FormFieldKind::ALL {
            let (w, h) = k.default_size_pt();
            assert!(w > 1.0 && h > 1.0, "{k:?} would place a {w}x{h} pt field");
        }
    }

    /// ★★ **Exactly one kind is authorable-but-inert**, and it is the push
    /// button.
    ///
    /// Asserted rather than assumed so that adding a sixth kind has to state
    /// which side it falls on. If pdfce ever runs PDF actions, this test fails
    /// and the failure is the prompt to un-grey the button.
    #[test]
    fn only_the_push_button_is_inert() {
        let inert: Vec<_> = FormFieldKind::ALL
            .into_iter()
            .filter(|k| !k.is_useful_once_placed())
            .collect();
        assert_eq!(
            inert,
            vec![FormFieldKind::PushButton],
            "the set of authorable-but-inert kinds changed; if pdfce now runs \
             actions, un-grey the push button and delete this expectation"
        );
    }
}
