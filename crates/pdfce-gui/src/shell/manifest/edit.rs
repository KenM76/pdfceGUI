//! The **Edit** tab — *what am I changing about content that is already
//! there?*
//!
//! `RIBBON_IA.md` §5.4. Five groups: Content, Insert, Clipboard, Forms,
//! Protect.
//!
//! # Three renames that are the point of the tab
//!
//! The salvage source's Content group carried three buttons labelled
//! `Aa`, `I⁺ Aa` and `Obj`. `Obj` is not a word, and the first two
//! returned the *same string literal* — two adjacent buttons
//! distinguishable only by icon and tooltip. These are the primary
//! content-editing tools and they were the least legible controls in the
//! application. They are now **Edit text**, **Add text** and **Edit
//! objects**, with the icons kept.
//!
//! # The `Editing on` master toggle is gone
//!
//! Operator decision, 2026-08-12: *"make it work the same way other
//! programs do."*
//!
//! No mainstream editor has a global editing switch. Acrobat, Bluebeam,
//! Word and Illustrator all work the same way: selection and Delete are
//! always live, and picking a tool arms *that tool* until Escape or
//! another tool. There is no state in which a click does nothing without
//! the application saying so.
//!
//! So there is no `Mode` group on this tab and no `edit.editing_enabled`
//! command. `RIBBON_IA.md` §7's migration map has a row for it — `Edit ▸
//! ContentTools ▸ Editing on` → `Edit ▸ Mode` — which §5.4 then
//! supersedes; §5.4 is the later and more specific statement and it is the
//! one implemented. The command is deliberately **not** in
//! [`super::PLANNED`] either, because it is not planned: it is deleted.
//!
//! This matters for how the Read/Review/Edit modes must behave. The rule
//! that makes those safe — *a mode changes what is **visible**; it never
//! makes a visible control silently inert* — is precisely the rule the
//! master toggle broke. A mode **removes** the tools it disables, so
//! there is no click that mysteriously fails. Reintroducing a global
//! enable flag under any name would undo that.
//!
//! # Redact arrives here
//!
//! From Tools ▸ Protect. One of the three moves a returning user will
//! notice, and the reasoning is the same as for the other two: a user
//! editing a document looks under Edit for the command that removes
//! content from it. Tools is for jobs that run across *other* files.
//!
//! The pair is kept together and in this order — mark, then apply —
//! because the asymmetry between them is the dangerous part: marking is
//! reversible and applying is not, and both tooltips say so.
//!
//! # What is absent
//!
//! The whole **Arrange** group (align, distribute, bring forward, send
//! backward, group, ungroup, flip) is **N**, so it is not here. So is the
//! object clipboard — cut, copy, paste, paste in place — which is why the
//! Clipboard group holds only the two text-copying commands that moved
//! here from File. `Shape ⌄` and `Sanitise…` are **N**.

use super::{command, group};
use crate::text::ribbon;
use egui_shell::manifest::Tab;

/// The Edit tab.
pub(super) fn tab() -> Tab {
    Tab::new("edit", ribbon::tab_edit())
        .with_question(ribbon::question_edit())
        .with_groups([
            // ---------------------------------------------------------------
            // Content — the three primary editing tools, relabelled.
            // ---------------------------------------------------------------
            group(
                "content",
                ribbon::group_edit_content(),
                [
                    command("edit.text"),
                    command("edit.add_text"),
                    command("edit.objects"),
                ],
            ),
            // ---------------------------------------------------------------
            // Insert — new content onto an existing page.
            //
            // Placing an image works today only by drag and drop, which is
            // a gesture with no discoverable equivalent: there is nothing
            // on screen that tells an operator it is possible. A command
            // is the affordance.
            //
            // `Shape ⌄` is **N**.
            // ---------------------------------------------------------------
            group(
                "insert",
                ribbon::group_edit_insert(),
                [command("edit.insert_image")],
            ),
            // ---------------------------------------------------------------
            // Clipboard — the two commands that moved off File.
            //
            // Copying text out of a document is a content operation, not a
            // file operation. That is the whole argument, and it is why
            // these two are the first entries in a group whose eventual
            // first four entries are cut/copy/paste/paste-in-place for the
            // object clipboard (**N**).
            // ---------------------------------------------------------------
            group(
                "clipboard",
                ribbon::group_edit_clipboard(),
                [
                    command("edit.copy_page_text"),
                    command("edit.copy_document_text"),
                ],
            ),
            // ---------------------------------------------------------------
            // Forms — one band where the salvage source had two.
            //
            // `Forms` (fill) and `Build Form` (author) were separate
            // groups on the same tab, which asks the operator to already
            // know which side of that line they are on before they can
            // find the control. All four steps of a form's life —
            // fill it, create a field, manage the fields, flatten the
            // result — sit together.
            //
            // `Flatten` moves out of the Forms pane for the same reason
            // `Export form data` moved to File ▸ Export: a command buried
            // in a panel is reachable only by someone who already opened
            // the panel.
            // ---------------------------------------------------------------
            group(
                "forms",
                ribbon::group_edit_forms(),
                [
                    command("edit.form_fill"),
                    command("edit.form_create_field"),
                    command("edit.form_manage_fields"),
                    command("edit.form_flatten"),
                ],
            ),
            // ---------------------------------------------------------------
            // Protect — mark, then apply. See the module header.
            // ---------------------------------------------------------------
            group(
                "protect",
                ribbon::group_edit_protect(),
                [command("edit.redact"), command("edit.redact_apply")],
            ),
        ])
}
