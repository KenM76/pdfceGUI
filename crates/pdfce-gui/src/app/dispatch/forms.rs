//! # `app::dispatch::forms` — what the five form-field commands do
//!
//! One arm of [`super::PdfceApp::dispatch_command`], lifted into its own file.
//! The dispatcher is a routing table and this is a route; what it gained by
//! moving is that the **reasoning** below is no longer sitting in the middle of
//! ninety-eight unrelated arms, and R2 stopped being breached by four lines.
//!
//! ## The whole of what a form command does: it arms a tool
//!
//! Nothing is authored here, and — unusually — nothing is authored on release
//! either. The placing gesture raises `Action::BeginFormField`, which opens
//! `crate::dialogs::formfield`, and the field exists once the operator presses
//! Add. `crate::canvas::formfield`'s header argues why that indirection is the
//! feature rather than an extra step: a stray form field is invisible on a
//! printed page and swallows every keystroke aimed near it, so a mis-drag must
//! cost nothing.
//!
//! ## ★★★ Why the push button is refused here, in words
//!
//! This file exists to hold one finding, and it is worth the space.
//!
//! `edit.form_push_button` is `enabled_when("forms.push_button_runnable")`, a
//! condition nothing sets, so its ribbon item is greyed. That much is measured
//! rather than assumed — `egui_shell::ribbon::ctx::condition_holds` answers
//! `false` for an unset name.
//!
//! **But `egui` refusing a click on a disabled widget is the entire mechanism
//! of greying.** Every other route into the dispatcher — a keyboard chord, the
//! QAT, a context menu, the `PDFCE_DIAG_INVOKE` harness seam — never touches
//! the ribbon at all. Driving the release binary with that id armed the tool
//! and traced `form-tool-armed kind=PushButton`. Ninety-nine commands carry an
//! `enabled_when`; the greying on all of them was a drawing, not a rule.
//!
//! ### ★★ The obvious repair was written, and the test suite refused it
//!
//! One guard at the top of `dispatch_command`: refuse any command whose
//! `enable` predicate is false. Ninety-nine controls fixed in six lines. It
//! compiled, and two tests failed — one of which carries the argument in its
//! own header:
//!
//! > *"the dispatcher must not consult one. `undo.available` greys the control
//! > and the apply arm declines an empty stack **in words** — both of which are
//! > somebody else's job."*
//!
//! That is right, and it is the more important half of the rule. **Greying is a
//! hint; the worded decline is the answer.** A choke point that swallowed the
//! command would have made `Ctrl+Z` on an empty stack do nothing at all *and*
//! say nothing at all — strictly worse than the status line it produces today,
//! and the exact shape of the silent-control defect this project keeps finding.
//!
//! So enforcement lives where the words can live, which is the arm. What that
//! costs is one branch per command that needs it; what it buys is that an
//! operator who reaches a greyed capability by some other route is told why
//! rather than left pressing a key that does nothing.

use super::PdfceApp;
use crate::canvas::formfield::FormFieldKind;

/// Arm the placement tool for `kind`, or decline in words.
///
/// `id` is carried only for the trace — it is recoverable from `kind`, but a
/// trace line that printed a reconstructed id would be a second opinion about
/// what the operator invoked, and the whole value of a trace is that it is a
/// record rather than an inference.
pub(super) fn arm(app: &PdfceApp, ctx: &egui::Context, id: &str, kind: FormFieldKind) {
    if !app.capabilities().edit_content {
        // ★ `edit_content`, not `author_markup`: a form field is a change to
        // the document's own content rather than an annotation over it, so
        // Review mode places no controls. Pairing it with markup would let a
        // reviewer author interactive controls, which is not a review activity.
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("command-declined id={id} reason=mode-cannot-edit-content")
        });
        return;
    }
    if !kind.is_useful_once_placed() {
        // ★ The predicate is `is_useful_once_placed`, **not** the condition
        // string. The condition is how the RIBBON asks; this is how the code
        // asks; and the two are welded by `only_the_push_button_is_inert` and
        // by the catalog test that pins the `enabled_when`. On the day pdfce
        // runs PDF actions, one predicate flips and both surfaces follow.
        crate::app::status::decline::record_push_button_inert();
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("command-declined id={id} reason=push-button-not-runnable")
        });
        return;
    }
    let _ = crate::canvas::tool::arm_form(ctx, kind);
}
