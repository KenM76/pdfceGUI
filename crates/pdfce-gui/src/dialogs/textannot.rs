//! # `dialogs::textannot` — the words half of a text-bearing annotation
//!
//! The second half of the place-then-type gesture. The canvas has taken a
//! rectangle (or a point); this asks what goes in it, and **nothing reaches
//! the document until Accept.**
//!
//! ## ★ Why a dialog, when markup authors on release
//!
//! `crate::dialogs`' header draws the line: *"a dialog is a single transaction
//! with a start and an end… a panel is somewhere an operator dips in and out
//! of while working."* Typing a callout is unmistakably the first — it begins
//! when the box is drawn, it ends when the words are accepted or abandoned,
//! and there is nothing to dip back into afterwards.
//!
//! The alternative — an in-place editor drawn over the page, the way a word
//! processor would — was rejected on the standing rule that **nothing floats
//! over the canvas** except the Find bar, which is a documented exception the
//! operator granted for one surface. It would also have needed a caret, a
//! selection and a hit test over text this shell does not own, which is a text
//! editor rather than a dialog.
//!
//! ## ★ It is deliberately NOT modal to the document
//!
//! The reference line stays drawn, the page stays where it was, and the dialog
//! is `default_pos` rather than anchored so it can be dragged aside. An
//! operator writing a callout is usually looking at the thing they are calling
//! out, and a window pinned over it would make them close the window to read
//! what they were annotating.
//!
//! ## The three kinds meet three different questions
//!
//! | kind | what this asks |
//! |---|---|
//! | text box | *what should it say?* — a multi-line field, because a callout wraps |
//! | sticky note | *what is the note?* — the same field; the words live in a popup rather than on the page, and the window says so |
//! | stamp | *which stamp?* — a gallery, and **no text field at all** |
//!
//! The stamp's absence of a field is the important one. `manifest/markup.rs`
//! recorded the blocker as *"a stamp with no chooser has no operand"*, and the
//! converse is just as true: a stamp with a free-text field is a text box with
//! a border, and offering both would be two controls for one feature with no
//! way for an operator to tell which they wanted.

use egui::Ui;
use pdfce_core::annot_author::StampName;
use pdfce_core::page_tree::Rect;

use crate::app::actions::Action;
use crate::canvas::textannot::{DEFAULT_STAMP, MAX_TEXT_CHARS, STAMPS, TextAnnotKind};
use crate::text::textannot as t;

/// The region the whole window publishes.
pub const REGION_BODY: &str = "dialog:text-annot"; // ui-text-exempt: trace region name, never displayed
/// The region the text field publishes, so a driven check can type into it.
pub const REGION_TEXT: &str = "text-annot.text"; // ui-text-exempt: trace region name, never displayed
/// The region the Accept control publishes.
pub const REGION_ACCEPT: &str = "text-annot.accept"; // ui-text-exempt: trace region name, never displayed

/// One open text-annotation dialog.
pub struct TextAnnotDialog {
    /// The page the annotation will land on, captured when the gesture
    /// completed.
    ///
    /// **Not re-read per frame.** The operator drew a box on the sheet they
    /// were looking at; a page change while this window is open must not
    /// redirect the annotation, which is the same rule the Set-scale dialog
    /// applies to its group.
    page: usize,
    /// Which kind is being authored.
    kind: TextAnnotKind,
    /// The rectangle, in PDF user space, captured with the page.
    rect: Rect,
    /// What the operator has typed.
    text: String,
    /// The stamp selected in the gallery. Meaningless for the other kinds and
    /// carried anyway — see `Action::CommitTextAnnot`'s field of the same name.
    stamp: StampName,
    /// Set by Accept, consumed after the window's closure returns.
    accept_requested: bool,
    /// Set by Cancel, consumed by [`Self::show`].
    close_requested: bool,
    /// Whether the text field has been focused yet.
    ///
    /// ★ One-shot, and it exists because a dialog that asks a question should
    /// put the caret where the answer goes. Without it the operator draws a
    /// box, a window appears asking what it should say, and they have to click
    /// into the field before they can type — which is a step the window itself
    /// created.
    focused_once: bool,
}

impl TextAnnotDialog {
    /// Open for a placed annotation.
    #[must_use]
    pub fn open(page: usize, kind: TextAnnotKind, rect: Rect) -> Self {
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "text-annot-open kind={kind:?} page={page} w={:.2} h={:.2}",
                rect.urx - rect.llx,
                rect.ury - rect.lly
            )
        });
        Self {
            page,
            kind,
            rect,
            text: String::new(),
            stamp: DEFAULT_STAMP,
            accept_requested: false,
            close_requested: false,
            focused_once: false,
        }
    }

    /// Draw one frame. Returns `false` when it should close.
    pub fn show(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) -> bool {
        let screen = ctx.input(egui::InputState::content_rect);
        let size = egui::vec2(420.0_f32.min(screen.width() - 40.0), 240.0);
        // Centred horizontally and a THIRD of the way down, not half — the
        // same placement the Set-scale dialog uses, and for the same reason: a
        // window centred vertically sits exactly over the middle of the page,
        // which on a drawing sheet is where the content is.
        let pos = egui::pos2(
            ((screen.width() - size.x).max(0.0) / 2.0).max(0.0),
            ((screen.height() - size.y).max(0.0) / 3.0).max(0.0),
        );
        let mut open = true;
        egui::Window::new(t::title(self.kind))
            .collapsible(false)
            .resizable(false)
            .default_size(size)
            .default_pos(pos)
            .open(&mut open)
            .show(ctx, |ui| {
                crate::diag::ui_rect(REGION_BODY, ui.max_rect());
                self.body(ui);
            });

        if self.accept_requested {
            self.accept_requested = false;
            actions.push(Action::CommitTextAnnot {
                page: self.page,
                kind: self.kind,
                rect: self.rect,
                text: std::mem::take(&mut self.text),
                stamp: self.stamp,
            });
            return false;
        }
        // ★ The window's own close button counts as Cancel, and authors
        // nothing. That is the honest reading: the operator dismissed a
        // question, and a dismissed question is not an answer.
        !(self.close_requested || !open)
    }

    /// The field or the gallery, then the two buttons.
    fn body(&mut self, ui: &mut Ui) {
        ui.label(t::intro(self.kind));
        ui.add_space(8.0);

        if self.kind.uses_gallery() {
            self.gallery(ui);
        } else {
            self.field(ui);
        }

        ui.add_space(10.0);
        ui.separator();
        ui.horizontal(|ui| {
            // ★ Accept is greyed when there is nothing to author, with the
            // reason on hover. That is the one place this shell greys rather
            // than hides: the control is *temporarily* unavailable — a
            // keystroke makes it live — which is exactly what greying is
            // reserved for.
            let ready = self.kind.uses_gallery() || !self.text.trim().is_empty();
            let accept = ui.add_enabled(ready, egui::Button::new(t::accept()));
            crate::diag::ui_rect(REGION_ACCEPT, accept.rect);
            if accept.clicked() {
                self.accept_requested = true;
            }
            if !ready {
                accept.on_disabled_hover_text(t::accept_disabled(self.kind));
            }
            if ui.button(t::cancel()).clicked() {
                self.close_requested = true;
            }
        });
    }

    /// The free-text field, for the two kinds whose words the operator writes.
    fn field(&mut self, ui: &mut Ui) {
        let response = ui.add(
            egui::TextEdit::multiline(&mut self.text)
                .desired_rows(4)
                .desired_width(f32::INFINITY)
                .hint_text(t::hint(self.kind))
                .char_limit(MAX_TEXT_CHARS),
        );
        crate::diag::ui_rect(REGION_TEXT, response.rect);
        // One-shot focus — see the field's docs. `request_focus` every frame
        // would fight anything else the operator clicked, including Cancel.
        if !self.focused_once {
            response.request_focus();
            self.focused_once = true;
        }
        ui.label(egui::RichText::new(t::bound(self.kind)).small().weak());
    }

    /// The stamp gallery, for the one kind whose words come from `/Name`.
    fn gallery(&mut self, ui: &mut Ui) {
        // A vertical list of radios rather than a combo box: seven entries is
        // a set an operator reads at a glance, and a combo would hide six of
        // them behind a click for no saving — this window has the room.
        for stamp in STAMPS {
            ui.radio_value(&mut self.stamp, *stamp, t::stamp_label(*stamp));
        }
        ui.add_space(4.0);
        ui.label(egui::RichText::new(t::stamp_bound()).small().weak());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect() -> Rect {
        Rect {
            llx: 0.0,
            lly: 0.0,
            urx: 100.0,
            ury: 40.0,
        }
    }

    /// A fresh dialog carries no words and the gallery's default.
    #[test]
    fn a_fresh_dialog_is_empty_and_defaulted() {
        let d = TextAnnotDialog::open(0, TextAnnotKind::TextBox, rect());
        assert!(d.text.is_empty(), "no words are invented for the operator");
        assert_eq!(d.stamp, DEFAULT_STAMP);
        assert!(!d.accept_requested);
    }

    /// ★ The page and the rect are captured, not re-read.
    ///
    /// The property that stops a page change under an open window redirecting
    /// the annotation. Asserted on the stored values because there is nothing
    /// else to assert it on — the whole point is that nothing re-reads them.
    #[test]
    fn the_page_and_rect_are_captured_at_open() {
        let d = TextAnnotDialog::open(7, TextAnnotKind::Sticky, rect());
        assert_eq!(d.page, 7);
        assert!((d.rect.urx - 100.0).abs() < f64::EPSILON);
    }

    /// ★ Accept is live for a stamp with no typed text, and dead for the
    /// others.
    ///
    /// The readiness rule, which is the gallery exception stated once more at
    /// the control that depends on it. A stamp whose Accept required typing
    /// could never be authored; a callout whose Accept did not would author an
    /// empty box.
    #[test]
    fn readiness_follows_the_gallery_rule() {
        let ready = |d: &TextAnnotDialog| d.kind.uses_gallery() || !d.text.trim().is_empty();

        let stamp = TextAnnotDialog::open(0, TextAnnotKind::Stamp, rect());
        assert!(ready(&stamp), "a stamp needs no typed words");

        let mut box_ = TextAnnotDialog::open(0, TextAnnotKind::TextBox, rect());
        assert!(!ready(&box_), "an empty callout must not be authorable");
        box_.text = "   ".to_owned();
        assert!(!ready(&box_), "whitespace is not words");
        box_.text = "note".to_owned();
        assert!(ready(&box_));
    }

    /// **The oracle for *"it doesn't type anything in the box when I type"*.**
    ///
    /// Every test above asserts on the struct's fields, which is exactly the
    /// blind spot `DEFECTS.md` D1 was: they all pass on a build whose window
    /// accepts no keystrokes, because none of them ever draws one. This drives
    /// a real `egui::Context` through two frames — one to build the field and
    /// take its one-shot focus, one carrying a real `Event::Text` — and asserts
    /// the words arrived.
    #[test]
    fn typing_into_the_open_window_reaches_the_draft() {
        let ctx = egui::Context::default();
        let mut d = TextAnnotDialog::open(0, TextAnnotKind::TextBox, rect());
        let mut actions = Vec::new();

        // Frame 1: the field is created and requests focus.
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            d.show(ui.ctx(), &mut actions);
        });

        // Frame 2: a real keystroke, the way a keyboard delivers one.
        let mut input = egui::RawInput::default();
        input.events.push(egui::Event::Text("h".to_owned()));
        let _ = ctx.run_ui(input, |ui| {
            d.show(ui.ctx(), &mut actions);
        });

        assert_eq!(d.text, "h", "the window took the keystroke");
    }
}
