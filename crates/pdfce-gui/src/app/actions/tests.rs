//! # `app::actions::tests` — the vocabulary's own assertions
//!
//! Split out of [`super`] on 2026-08-19 when that file crossed R2's 1,500-line
//! ceiling, and the seam is the one `tools/gates/check-file-size.sh` asks for
//! rather than a size: [`super`] is the **vocabulary** — one enum, and the
//! argument for every variant in it — and this is what is asserted *about* that
//! vocabulary. A reader looking up what `crate::app::actions::VectorAction::MoveNodes.into()` means never needs
//! this file, and a reader asking whether the dispatch reaches it never needs
//! the other 1,400 lines of prose.
//!
//! ★ It is a file rather than an inline module for one more reason worth
//! stating: `super`'s content is nine-tenths doc comments, so a `#[cfg(test)]`
//! block at the bottom of it is a hundred lines of *code* at the end of a
//! document. That is exactly the shape R2 exists to prevent — the old shell's
//! `main.rs` was 25,005 lines plus 3,579 of tests, and nothing in it could be
//! reasoned about locally.

use super::*;

/// ★ **`edit.undo` and `edit.redo` raise actions rather than falling
/// through to `command-unimplemented`.**
///
/// The dispatch link, and the one this pair spent the whole project
/// missing. It is `crate::app::files`'
/// `the_save_copy_command_raises_the_save_action` for the other two
/// commands that were registered, drawn on the quick-access toolbar, bound
/// to a chord, and wired to nothing — and it is written the same way for
/// the same reason: through `PdfceApp::dispatch_token` with the token the
/// **ribbon** would raise, so a build that renamed the id or reassigned the
/// token fails here rather than shipping a control whose press is traced
/// and discarded.
///
/// # What it deliberately does not assert
///
/// That the actions *do* anything. Two arms that pushed the wrong variant
/// would pass a test written as "some action was raised", which is why the
/// comparison is against the exact vector — and what each variant does when
/// applied is `crate::app::actions::apply`'s
/// `an_undo_is_an_edit_and_moves_the_epoch_like_one`, on a real fixture with
/// a real edit on the log.
///
/// # Why an EMPTY log is the state under test here
///
/// Because the dispatcher must not consult one. `undo.available` greys the
/// control and the apply arm declines an empty stack in words — both of
/// which are somebody else's job — and an arm that checked the session here
/// would be the second place that question is asked. So the action is raised
/// with nothing to undo, exactly as it would be for a `Ctrl+Z` fired at a
/// freshly opened document, and the decline happens downstream.
#[test]
fn the_history_commands_raise_actions() {
    let ctx = egui::Context::default();
    let mut app = crate::app::tests::opened();

    for (id, expected) in [("edit.undo", Action::Undo), ("edit.redo", Action::Redo)] {
        let token = app
            .commands
            .get(id)
            .unwrap_or_else(|| panic!("`{id}` must be registered")) // ui-text-exempt: test panic
            .handler;
        let mut actions = Vec::new();
        app.dispatch_token(&ctx, token, &mut actions);
        assert_eq!(
            actions,
            vec![expected],
            "`{id}` must raise its action rather than falling through to \
                 `command-unimplemented`, which is what it did for the whole life of the project"
        );
    }
}

/// ★★★ **A greyed command that is invoked anyway declines IN WORDS.**
///
/// The regression test for a hole this project carried since the registry
/// gained `enabled_when`: **the greying was drawn, never enforced.** `egui`
/// refuses a click on a disabled widget and that is the whole of what greying
/// does — a chord, the QAT, a context menu or the `PDFCE_DIAG_INVOKE` seam all
/// reach `dispatch_command` without passing the ribbon at all.
///
/// Found by driving the release binary, not by reading:
/// `PDFCE_DIAG_INVOKE=mode.edit,edit.form_push_button` traced
/// `form-tool-armed kind=PushButton`, arming a tool whose control is greyed.
///
/// ## What this test asserts, and the thing it deliberately does NOT
///
/// It asserts **two** facts, and the second is the one worth having:
///
/// 1. the tool is not armed — a greyed command does not do its thing;
/// 2. a decline is **recorded**, so the operator gets a sentence.
///
/// The second is what stopped the obvious repair. A blanket refusal at the top
/// of `dispatch_command` was written, and it satisfied (1) for all ninety-nine
/// `enabled_when` commands at once — and the suite rejected it, because it also
/// removed the words. `the_history_commands_raise_actions` says so in its own
/// header: *"the dispatcher must not consult one … the apply arm declines an
/// empty stack IN WORDS."* Greying is a hint; a sentence is the answer.
///
/// So a test that only checked "nothing happened" would have passed against the
/// change this project rejected, which is the shape of a check that cannot fail
/// for the right reason.
#[test]
fn a_greyed_push_button_declines_in_words_rather_than_arming() {
    let ctx = egui::Context::default();
    let mut app = crate::app::tests::opened();
    // Reached through the dispatcher rather than by writing the field, so the
    // mode is entered exactly as an operator's Ctrl+3 enters it.
    app.dispatch_command(&ctx, "mode.edit", &mut Vec::new());
    crate::app::status::decline::retire();

    let mut actions = Vec::new();
    app.dispatch_command(&ctx, "edit.form_push_button", &mut actions);

    assert!(
        !matches!(
            crate::canvas::tool::active(&ctx),
            crate::canvas::tool::CanvasTool::Form(_)
        ),
        "a greyed command must not arm its tool" // ui-text-exempt: test assertion message
    );
    assert_eq!(
        crate::app::status::decline::recorded_for_test(),
        Some(crate::app::status::decline::Declined::PushButtonInert),
        "…and it must SAY so — a command that silently does nothing is \
         indistinguishable from one with no arm, and the two want opposite fixes"
    );
}

/// **The other four form commands still arm**, so the guard above is a guard
/// and not a blanket refusal.
///
/// ★ The positive control. Without it, a mistake that declined every form
/// command would leave the test above passing and the whole feature dead — the
/// standing rule that a check which cannot fail is not evidence, applied to its
/// own neighbour.
#[test]
fn the_four_useful_form_commands_still_arm() {
    use crate::canvas::formfield::FormFieldKind;
    let ctx = egui::Context::default();
    let mut app = crate::app::tests::opened();
    app.dispatch_command(&ctx, "mode.edit", &mut Vec::new());

    for kind in FormFieldKind::ALL {
        if !kind.is_useful_once_placed() {
            continue;
        }
        let mut actions = Vec::new();
        app.dispatch_command(&ctx, kind.command_id(), &mut actions);
        assert_eq!(
            crate::canvas::tool::active(&ctx),
            crate::canvas::tool::CanvasTool::Form(kind),
            "{} must arm its tool", // ui-text-exempt: test assertion message
            kind.command_id()
        );
    }
}
