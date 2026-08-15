//! The **Edit** tab — *what am I changing about content that is already
//! there?*
//!
//! `RIBBON_IA.md` §5.4. **Four** groups: Content, Insert, Forms, Protect.
//!
//! ★ It was five until 2026-08-14, when the operator moved the two text-copy
//! commands to File ▸ Export and the **Clipboard** group — whose only members
//! they were — was deleted rather than left empty. The site of that group
//! carries the full reasoning; the one-line version is that copying is not
//! authoring, so the verb does not belong on the authoring tab, and an empty
//! band is the placeholder `RIBBON_IA.md` P3 forbids.
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
//! object clipboard — cut, copy, paste, paste in place — and with the two
//! text-copy commands gone to File ▸ Export there is nothing left for a
//! Clipboard band to hold, so the band is absent rather than empty.
//! `Shape ⌄` and `Sanitise…` are **N**.

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
            // ★ **Clipboard was here, and it is deleted rather than emptied.**
            //
            // It held exactly two commands — `edit.copy_page_text` and
            // `edit.copy_document_text` — and on 2026-08-14 the operator moved
            // both to File ▸ Export as `file.copy_page_text` and
            // `file.copy_document_text`. Its own note read:
            //
            //     Clipboard — the two commands that moved off File.
            //     Copying text out of a document is a content operation, not a
            //     file operation. That is the whole argument, and it is why
            //     these two are the first entries in a group whose eventual
            //     first four entries are cut/copy/paste/paste-in-place for the
            //     object clipboard (**N**).
            //
            // The premise held; the conclusion did not follow. A content
            // operation is not automatically an **authoring** operation, and
            // copying authors nothing — it reads the page and writes to the
            // clipboard. What made that visible was the chord/mode gate
            // refusing `Ctrl+Shift+C` in Read, a mode measured against Acrobat
            // Reader, which copies text. Same line as `edit.form_fill` →
            // `view.panel_forms`: *filling is not authoring*, and neither is
            // copying.
            //
            // **The group goes with them, and does not stay as a placeholder
            // for the object clipboard it was reserving space for.** P3 is the
            // rule — an unavailable capability renders nothing — and a caption
            // with no controls under it is the emptiest possible stub: a band
            // that promises cut, copy and paste and offers no way to reach any
            // of them. All four object-clipboard ids are **N** in
            // `super::PLANNED`, which is where that reservation belongs, and
            // that is where the group will be rebuilt from on the day one of
            // them ships. `super::manifest`'s documented group count goes
            // 32 → 31.
            // ---------------------------------------------------------------
            // Forms — one band where the salvage source had two, and now
            // one band holding **three** steps of a form's life rather than
            // four.
            //
            // `Forms` (fill) and `Build Form` (author) were separate groups
            // on the same tab, which asks the operator to already know which
            // side of that line they are on before they can find the
            // control. That argument put fill, create, manage and flatten
            // together, and it was right about the operator who is *in this
            // tab*.
            //
            // ★ **Fill left on 2026-08-14, and the argument that moved it is
            // stronger than the one that kept it here.** The operator's
            // answer to `crate::app::modes`' open question is that Read
            // fills forms — Acrobat Reader does, and replacing it is the
            // stated goal. Read is shown `file` and `view` alone, and P1
            // gives a command exactly one tab, so `edit.form_fill` became
            // `view.panel_forms` in View ▸ Panels.
            //
            // What is left is not a remnant. Filling a field is using the
            // document as its author designed it; creating a field, renaming
            // one and flattening the result are changes to the design
            // itself. That line is real, it is the line the mode taxonomy
            // already draws between Review and Edit, and the three verbs
            // that stayed are on the authoring side of it together.
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
