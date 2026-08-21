//! # `dialogs::host` — a dialog is an OS WINDOW
//!
//! ## The operator's report, 2026-08-20
//!
//! > *"Print dialogue box doesn't pop up in its own movable window. It is
//! > locked within the boundaries of the program's window. Like, I just assume
//! > you've been trained on a million lines of code and software that pops it
//! > up in its own window."*
//!
//! He is right, and the last sentence is the diagnosis. `ui-conventions/dialogs.md`
//! G1 states the rule and states why immediate-mode toolkits get it wrong:
//!
//! > **Why immediate-mode toolkits get this wrong.** Their in-viewport "window"
//! > widget is the path of least resistance — it looks like a dialog and is one
//! > function call. A real OS window needs a viewport/second-window API that is
//! > newer and more awkward. The default is wrong and nothing pushes back.
//!
//! Every retained toolkit makes the OS window the **default** and the inline
//! panel the special case — `QDialog`, `NSWindow`, `ContentDialog` vs `Window`.
//! That default ordering is itself the guidance. This module restores it here:
//! calling [`Host::show`] is now as easy as calling `egui::Window::new`, so the
//! path of least resistance and the right answer are the same path.
//!
//! ## ★ What an OS window buys, concretely
//!
//! Not aesthetics. Four things an operator does with a print dialog:
//!
//! 1. **Move it off the document** to read the page underneath while choosing a
//!    range. An in-viewport window can be dragged to the edge and no further.
//! 2. **Put it on the second monitor**, which is what a two-screen desk is for.
//! 3. **Find it in the taskbar / Alt-Tab** when it has gone behind something.
//! 4. **Resize it past the application window**, which matters for the print
//!    preview specifically: the preview is the point of the dialog and it is
//!    the first thing the 520 pt floor squeezes.
//!
//! ## ★★ It degrades, and that is deliberate rather than incidental
//!
//! `Context::show_viewport_immediate` falls back to an **embedded** window —
//! literally the `egui::Window` this replaces — when the backend has no
//! multi-viewport renderer, which is the case on **web**. `MODES_AND_PANELS.md`
//! records the web fork as a live target, so a dialog host that only worked
//! natively would be a surface that vanishes on one of the two platforms.
//!
//! The fallback is egui's, not ours: one code path, two renderings, and
//! [`Frame::class`] says which so a caller that genuinely must know can ask.
//! Nothing in this module branches on it except the position memory, which has
//! nothing to remember when the OS is not placing the window.
//!
//! ## G4 — Enter accepts, Escape cancels, and the default is VISIBLY the default
//!
//! The second half of the operator's item, and the failure mode is the one
//! everybody has met:
//!
//! > *"The operator types into the last field, presses Enter out of habit, and
//! > nothing happens — or worse, something other than what they expected."*
//!
//! [`Host::buttons`] draws the pair and owns all three obligations, because a
//! caller that had to remember them would forget one:
//!
//! - **Enter** activates the affirmative action — but **not while a text field
//!   has focus and wants the key**, which is why the check asks
//!   `ctx.text_edit_focused()` first. A multi-line field would otherwise lose
//!   the ability to type a newline the moment it sat in a dialog.
//! - **Escape** is equivalent to Cancel *and* to the close button, so all three
//!   routes out are one outcome.
//! - The affirmative button is **drawn** as the default, from the theme's own
//!   selection fill, so the operator knows what Enter will do before pressing
//!   it. A default nobody can see is not a default; it is a surprise.
//!
//! ## G6 — it remembers where it was left
//!
//! Position is held per dialog, keyed on the [`Host::id`] it was constructed
//! with, and re-applied on the next open through
//! `ViewportBuilder::with_position`. **Nothing is remembered in the embedded
//! fallback**, because egui places that window and the OS does not.
//!
//! ★ It is stored in memory rather than on disk, deliberately. A position that
//! survived a restart would have to be validated against the *current* monitor
//! layout — G6 says so in the same breath — and a dialog that opens on a
//! monitor which is no longer attached is worse than one that opens centred.
//! Persisting it is a real feature with a real check to write; the session-long
//! version is the nine-tenths of it that costs nothing and cannot strand
//! anybody.
//!
//! ## What this does NOT fix, said so it is a decision
//!
//! - **G3, owned by the application window.** `eframe 0.35`'s
//!   `ViewportBuilder` has no owner or parent option — `grep with_` over
//!   `egui/src/viewport.rs` returns thirty builders and none of them is one —
//!   and `egui-winit` never passes the parent relationship egui itself tracks
//!   in `viewport_parents` down to `winit`. So the dialog can fall behind the
//!   main window, which is *the* classic Windows bug and the reason G3 exists.
//!   Mitigating it with `with_always_on_top` was considered and refused: this
//!   project's own RAG records an always-on-top window swallowing the driven
//!   harness's clicks with `SetForegroundWindow` still reporting success, and
//!   trading a rare confusion for a class of undiagnosable harness failure is
//!   a bad trade. It needs a `SetWindowLongPtrW(GWLP_HWNDPARENT)` on the child
//!   window, and eframe does not expose the child's handle either. Filed as a
//!   gap rather than papered over.
//! - **G5, focus trapping and tab order.** An OS window gets keyboard focus of
//!   its own, which is most of what G5 asks for and is strictly better than the
//!   in-viewport version had. Ordered tab traversal and a focus trap are not
//!   asserted by anything and are still a gap.
//!
//! ## ★ The diagnostic channel had to learn about viewports, and why
//!
//! `crate::diag::ui_rect` publishes a named region's rectangle so a driven
//! check can aim at a control without guessing. Those rectangles are **relative
//! to the viewport that drew them**, and until this module existed there was
//! only one viewport, so the harness could add the main window's client origin
//! and be right.
//!
//! A dialog in its own window breaks that silently — the coordinates stay
//! plausible and land somewhere else entirely, which is the exact shape of
//! defect `D:/dev/rag/egui/` records twice already. So [`Host::show`] publishes
//! `viewport-inner`, the child's own client rectangle in **desktop**
//! coordinates, and `ui_rect` tags every region it publishes with the viewport
//! that drew it. The harness then has both halves and can convert; it is also
//! the only way a check can *tell* that a dialog opened in its own window,
//! which is what makes G1 assertable rather than a matter of looking.

use egui::{Pos2, Vec2, ViewportBuilder, ViewportClass, ViewportId};

/// How far in from the application window a dialog opens when it has no
/// remembered position.
///
/// Not centred on the parent, and not at the OS's own default. Centring puts a
/// dialog exactly over the thing it is asking about, which is the one place it
/// must not be for a *print* dialog whose preview the operator is comparing
/// against the page behind it. A small inset reads as "this belongs to that
/// window" without covering its middle.
const OPEN_INSET_PT: f32 = 48.0;

/// One dialog's window: what it is called, how big it opens, and where the
/// operator last left it.
///
/// Held by the dialog it belongs to, so its lifetime is the dialog's — which is
/// what makes the position memory correct without anything having to clear it.
/// A dialog that is closed and reopened gets a fresh `Host` and therefore opens
/// where the platform puts it; a dialog that stays open across frames keeps the
/// position it has been dragged to.
pub struct Host {
    /// The viewport id, stable for this dialog across frames.
    ///
    /// ★ Derived from a caller-supplied string rather than counted, because
    /// `ViewportId` is what egui keys the OS window on: two dialogs sharing one
    /// would be two dialogs sharing one window, and a counter would give a
    /// dialog a different window depending on what else was open when it was
    /// created.
    id: ViewportId,
    /// The window's title bar text. Owned rather than `&'static str` because it
    /// may carry a document name.
    title: String,
    /// The size it opens at.
    default_size: Vec2,
    /// The smallest it may be dragged to.
    ///
    /// A floor, not a preference — the reason is `print`'s and it generalises:
    /// a resizable window with no floor can be dragged down to a title bar and
    /// a scrollbar, which is a state with no way back except closing the
    /// dialog and losing what was typed into it.
    min_size: Vec2,
    /// Where the operator last left it, in **desktop** coordinates, or `None`
    /// before it has been placed. See the module header for why this is
    /// session-scoped.
    left_at: Option<Pos2>,
}

/// What one frame of a hosted dialog reported back.
pub struct Frame {
    /// Whether egui drew a real OS window or fell back to an embedded one.
    ///
    /// Carried rather than hidden because it is the honest answer to *"did G1
    /// actually happen"*, and because the embedded case has no position to
    /// remember. No caller is expected to branch on it.
    pub class: ViewportClass,
    /// The operator asked to close it — the OS close button, or Escape.
    ///
    /// Both, together, deliberately: G4 says Escape *is* Cancel and is *is* the
    /// close button, so a caller that treated them differently would give one
    /// of the three routes out a different meaning from the other two.
    pub closed: bool,
}

impl Host {
    /// A dialog window.
    ///
    /// `id` must be unique and stable per dialog — `"print"`, `"insert-image"`.
    /// It keys the OS window, and it is also what the diagnostic channel
    /// publishes, so a driven check names the same string the code does.
    #[must_use]
    pub fn new(id: &str, title: impl Into<String>, default_size: Vec2, min_size: Vec2) -> Self {
        Self {
            id: ViewportId::from_hash_of(id),
            title: title.into(),
            default_size,
            min_size,
            left_at: None,
        }
    }

    /// **Draw one frame of this dialog in its own OS window.**
    ///
    /// `add` is handed a `Ui` inside the window and may do anything an
    /// `egui::Window`'s closure could. What it returns comes back untouched, so
    /// a dialog that computes something while drawing does not need a field to
    /// carry it out.
    ///
    /// # ★ Why the close signal comes back rather than through an `&mut bool`
    ///
    /// `egui::Window::open(&mut bool)` is the idiom this replaces, and it has a
    /// property worth losing: the flag is written *during* the draw, so a
    /// caller reading it afterwards cannot tell whether the operator closed the
    /// window or the caller's own code did. [`Frame::closed`] is a report about
    /// this frame only, and the caller decides what closing means — which for
    /// a dialog that is mid-transaction is not always "stop".
    pub fn show<R>(
        &mut self,
        ctx: &egui::Context,
        add: impl FnOnce(&mut egui::Ui) -> R,
    ) -> (Frame, R) {
        // ★ `show_viewport_immediate` takes `FnMut`, because egui reserves the
        // right to call a viewport's callback more than once. `add` is `FnOnce`
        // — the honest signature for a dialog body, which draws once per frame
        // and may consume what it captures — so it is moved into an `Option`
        // and taken. A second call would `expect` here rather than silently
        // drawing nothing, because "the dialog was blank" is a symptom nobody
        // could trace back to this line.
        let mut add = Some(add);
        let mut builder = ViewportBuilder::default()
            .with_title(self.title.clone())
            .with_inner_size(self.default_size)
            .with_min_inner_size(self.min_size)
            // ★ No maximize and no minimize. A dialog is one transaction; the
            // operator finishes it or abandons it, and a minimised dialog is a
            // transaction that has been left open with no surface saying so.
            // Every platform's dialog chrome makes the same choice.
            .with_minimize_button(false)
            .with_maximize_button(false)
            // It IS in the window list, deliberately, and that is the half of
            // the operator's report that a borderless window would not fix:
            // *"find it when it has gone behind something"*. With G3
            // unavailable (see the module header) this is the only route back
            // to a dialog that has fallen behind the parent.
            .with_taskbar(true);
        builder = match self.left_at {
            // G6: back where it was left.
            Some(at) => builder.with_position(at),
            // First open: inset from the application window rather than centred
            // on it — see `OPEN_INSET_PT`.
            None => match ctx.input(|i| i.viewport().outer_rect) {
                Some(parent) => builder.with_position(parent.min + Vec2::splat(OPEN_INSET_PT)),
                // No parent rect means egui has not been told where the
                // application window is, which happens on the first frame and
                // in a headless harness. Letting the platform place it is the
                // right answer and not a fallback: it is what every dialog does
                // when nothing better is known.
                None => builder,
            },
        };

        let mut frame = Frame {
            // ★ `EmbeddedWindow`, not `Root`, as the value before egui
            // answers. It is the CONSERVATIVE default: it claims the fallback
            // rather than the OS window, so a path that somehow never reaches
            // the callback reports "G1 did not happen" instead of asserting it
            // did. A default that over-claims is how a gate goes green on a
            // build that regressed.
            class: ViewportClass::EmbeddedWindow,
            closed: false,
        };
        let result = ctx.show_viewport_immediate(self.id, builder, |ui, class| {
            frame.class = class;
            let child = ui.ctx().clone();

            // ★ Remember where the OS has put it, every frame, so a drag is
            // captured without a drag handler. `inner_rect` is desktop
            // coordinates; `with_position` takes the OUTER position, so the
            // outer rect is what is stored — using the inner one would walk the
            // window up-left by the title bar's height on every reopen.
            if class == ViewportClass::Immediate {
                let (outer, inner) =
                    child.input(|i| (i.viewport().outer_rect, i.viewport().inner_rect));
                if let Some(outer) = outer {
                    self.left_at = Some(outer.min);
                }
                // ★★ The child's own client rectangle, in DESKTOP coordinates,
                // for the harness. See the module header: every `ui-rect` this
                // dialog publishes is relative to THIS origin and not to the
                // application window's, and the two are plausible-looking
                // numbers that differ by hundreds of pixels.
                if let Some(inner) = inner {
                    crate::diag::viewport_inner(self.id, inner);
                }
            }

            // ★★ Every `ui-rect` this dialog publishes is tagged with THIS
            // viewport for the rest of the callback. See
            // `crate::diag::ViewportScope`: without it the harness reads the
            // dialog's rectangles as if they were the application window's,
            // and they are plausible numbers naming a different place on the
            // desktop.
            let _regions = crate::diag::ViewportScope::enter(self.id);

            // G4's Escape half, read from the CHILD's input. Reading the
            // parent's would answer about a key pressed into the application
            // window, which is a different window and, once G3 lands, a
            // different focus.
            //
            // typing-guard-exempt: this asks whether a WIDGET holds Escape, not
            // whether anybody is composing. A canvas draft is not reachable from
            // inside a dialog.
            let escape =
                !child.text_edit_focused() && child.input(|i| i.key_pressed(egui::Key::Escape));
            frame.closed = escape || child.input(|i| i.viewport().close_requested());

            // egui may in principle call a viewport's callback more than once
            // in a frame; a dialog body is `FnOnce` and may consume what it
            // captures, so the second call panics rather than silently drawing
            // nothing. "The dialog was blank" is a symptom nobody could trace
            // back to this line.
            //
            // ui-text-exempt: a panic message for an egui contract violation,
            // never displayed to an operator and never reachable from one.
            let draw = add.take().expect("viewport callback ran twice");
            draw(ui)
        });
        (frame, result)
    }

    /// **Draw a dialog's affirmative and cancelling buttons**, with Enter and
    /// Escape wired and the default drawn as the default.
    ///
    /// Returns `(accepted, cancelled)`. Both can be `false`; neither pair of
    /// them is ever `true` together, because Enter and Escape are different
    /// keys and the two buttons are different rectangles.
    ///
    /// # ★ The order is Cancel then Accept, right-aligned
    ///
    /// Which is Windows' order and the order every dialog on this operator's
    /// machine uses. It is not a preference: a button's meaning is learned by
    /// position long before it is read, and a dialog that reverses the pair is
    /// a dialog whose Cancel gets clicked by muscle memory aimed at OK.
    ///
    /// # ★★ Enter is refused while a text field wants it
    ///
    /// `ctx.text_edit_focused()` — the same predicate `canvas::textedit`
    /// enforces one copy of, and `tools/gates/check-typing-guard.sh` fails the
    /// build on a second. Without it, a dialog with a multi-line field would
    /// accept on the first newline the operator typed, which is worse than
    /// having no Enter at all: it commits a half-written transaction.
    ///
    /// A **single-line** field is the case this deliberately gives up. Enter in
    /// a one-line box should accept the dialog, and here it does not, because
    /// egui reports "a text edit has focus" without saying whether it is
    /// multi-line. Recorded as a known limit rather than guessed at — the fix
    /// is per-field and belongs with the field.
    pub fn buttons(ui: &mut egui::Ui, accept: &str, cancel: &str) -> (bool, bool) {
        let ctx = ui.ctx().clone();
        // ★ THIS ASKS WHETHER A WIDGET IN THIS DIALOG HOLDS THE KEYBOARD, not
        // whether the operator is composing anywhere in the application, and
        // the two genuinely differ here.
        //
        // `crate::canvas::textedit::composing` - the predicate this gate
        // normally requires - answers `true` while a canvas draft is live, and
        // a canvas draft SURVIVES the opening of a dialog: it is committed by
        // clicking away on the page, not by a print window appearing. So using
        // it would mean that an operator who had a caret on the page, opened
        // Print and pressed Enter got nothing, with no surface saying why -
        // which is `dialogs.md` G4's stated failure mode reintroduced by the
        // guard against a different one.
        //
        // The hazard the gate exists for cannot occur here in either
        // direction. A dialog is a separate OS window with its own keyboard
        // focus, so an Enter arriving in it was aimed at it; and nothing in
        // this function can steal a key from the canvas, because the canvas is
        // not being drawn inside this callback.
        //
        // What IS wanted is the half `text_edit_focused` answers: a multi-line
        // field inside the dialog must keep the ability to type a newline. See
        // this function's own docs for the single-line case, which is
        // deliberately given up rather than guessed at.
        //
        // typing-guard-exempt: the four paragraphs above are the reason. In one
        // line: a dialog is a separate OS window with its own focus, so
        // "somebody is composing on the canvas" is not a fact about this key.
        let enter = !ctx.text_edit_focused() && ctx.input(|i| i.key_pressed(egui::Key::Enter));

        let mut accepted = false;
        let mut cancelled = false;
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // ★ Drawn from the theme's own selection fill and the strong text
            // colour it is guaranteed to contrast with — never a literal, which
            // `check-theme-colors.sh` enforces and which defect D2 is about.
            // A palette change moves this button with everything else.
            let visuals = ui.visuals();
            let fill = visuals.selection.bg_fill;
            let text = visuals.strong_text_color();
            let default = egui::Button::new(egui::RichText::new(accept).color(text)).fill(fill);
            if ui.add(default).clicked() || enter {
                accepted = true;
            }
            if ui.button(cancel).clicked() {
                cancelled = true;
            }
        });
        (accepted, cancelled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★ **Two dialogs get two windows**, which is the whole reason the id is
    /// derived from a caller-supplied string rather than counted.
    ///
    /// A shared `ViewportId` is a shared OS window: the second dialog would
    /// draw into the first one's frame, and which one you saw would depend on
    /// draw order. Cheap to assert, and the failure is invisible until two
    /// dialogs are open at once.
    #[test]
    fn each_dialog_gets_its_own_viewport() {
        let a = Host::new("print", "Print", Vec2::splat(100.0), Vec2::splat(10.0));
        let b = Host::new(
            "insert-image",
            "Insert image",
            Vec2::splat(100.0),
            Vec2::splat(10.0),
        );
        assert_ne!(a.id, b.id);
    }

    /// …and the same dialog gets the same window every time, so a reopen is the
    /// same window rather than a second one beside it.
    #[test]
    fn one_dialog_keeps_one_viewport_across_constructions() {
        let a = Host::new("print", "Print", Vec2::splat(100.0), Vec2::splat(10.0));
        let b = Host::new("print", "Print", Vec2::splat(900.0), Vec2::splat(90.0));
        assert_eq!(a.id, b.id, "the id must key on the NAME, not on the size");
    }

    /// A fresh host has nothing to remember, so the first open is placed rather
    /// than restored.
    #[test]
    fn a_fresh_host_remembers_no_position() {
        let h = Host::new("print", "Print", Vec2::splat(100.0), Vec2::splat(10.0));
        assert!(h.left_at.is_none());
    }
}
