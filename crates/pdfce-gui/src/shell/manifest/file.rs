//! The **File** tab — *what do I do with the file as a whole, or with
//! pdfce itself?*
//!
//! `RIBBON_IA.md` §5.1. Six groups: File, Save, Export, Print, Document,
//! pdfce.
//!
//! # What this tab stopped being
//!
//! The salvage source's File tab was, in its own document's words, *"a
//! junk drawer"*: Properties, Copy this page's text, Copy the whole
//! document's text, Export DXF, Print, Reset layout, Settings, keyboard
//! shortcuts. Two of those are content operations and one is a view
//! operation. Meanwhile it had no New, no Recent, no Close — and no Open
//! and no Save, because those lived only on the quick-access toolbar and
//! the old reading of the one-command-one-tab rule forbade a tab from
//! mirroring them.
//!
//! Three things therefore happen here:
//!
//! 1. **`Open…` and `Save a copy…` appear on a tab**, under amendment P1a
//!    — *the QAT and the status bar are shortcut surfaces, not tabs; a
//!    command may appear on exactly one tab and additionally on the QAT.*
//!    A user who wants to open a file looks under File, and finding
//!    nothing there teaches them the ribbon is not where commands live.
//! 2. **Copy page text / copy document text leave**, to Edit ▸ Clipboard.
//!    Copying text out of a document is a content operation.
//! 3. **Reset layout leaves**, to View ▸ Window. It resets panel geometry.
//!
//! And one thing arrives: **Fonts**, from View ▸ Panels. The Fonts panel
//! answers *"what is inside this file"*, not *"what is on my screen"*, so
//! it sits with Properties as document-level inspection. `RIBBON_IA.md`
//! §5.1 flags this as a real improvement on the current build rather than
//! a re-parenting — the panel is good and nobody was going to find it
//! under View.
//!
//! # Why the Save group holds one command
//!
//! `Save` — the one that overwrites in place — cannot ship before autosave
//! and crash recovery exist; that dependency predates this document. Under
//! P3 it is therefore **absent**, not greyed with an explanatory tooltip,
//! and `Save a copy…` stands alone in the band. `Revert` is absent for the
//! same reason: it is meaningless without a save point to revert to.

use super::{command, group};
use crate::text::ribbon;
use egui_shell::manifest::Tab;

/// The File tab.
pub(super) fn tab() -> Tab {
    Tab::new("file", ribbon::tab_file())
        .with_question(ribbon::question_file())
        .with_groups([
            // ---------------------------------------------------------------
            // File — getting a document in and out of the application.
            //
            // `New` and `Recent ⌄` are specified here and are **N**; see
            // PLANNED. What is left is the pair that exists, and the pair
            // that exists is the pair a first-time user looks for.
            // ---------------------------------------------------------------
            group(
                "file",
                ribbon::group_file_file(),
                [command("file.open"), command("file.close")],
            ),
            // ---------------------------------------------------------------
            // Save — see the module header on why this band has one item.
            // ---------------------------------------------------------------
            group(
                "save",
                ribbon::group_file_save(),
                [command("file.save_copy")],
            ),
            // ---------------------------------------------------------------
            // Export — writing this document out as something else.
            //
            // `Export form data` moves here from the Forms pane: it writes
            // a file, which makes it an export, and leaving it inside a
            // panel meant only an operator who already had the panel open
            // could find it.
            //
            // Export image (PNG/JPEG/TIFF with a DPI picker) and Export
            // text are **C** — `pdfce-core` does both and neither has a
            // GUI surface. They are the cheapest wins on this tab and they
            // are still absent until the shell exists, because a **C** row
            // is an engine, not a command.
            // ---------------------------------------------------------------
            group(
                "export",
                ribbon::group_file_export(),
                [command("file.export_dxf"), command("file.export_form_data")],
            ),
            // ---------------------------------------------------------------
            // Print. Imposition (n-up / booklet / poster) is **C**.
            // ---------------------------------------------------------------
            group("print", ribbon::group_file_print(), [command("file.print")]),
            // ---------------------------------------------------------------
            // Document — inspection of what is inside the file.
            //
            // `Security` is **N** and would sit third. Its absence is
            // visible in a way worth noting: a band called Document that
            // cannot tell you whether a document is encrypted is doing
            // half its job, and the status bar carries that fact today.
            // ---------------------------------------------------------------
            group(
                "document",
                ribbon::group_file_document(),
                [command("file.properties"), command("file.fonts")],
            ),
            // ---------------------------------------------------------------
            // pdfce — the application's own settings and help. `About` is
            // **N**.
            // ---------------------------------------------------------------
            group(
                "pdfce",
                ribbon::group_file_pdfce(),
                [command("file.settings"), command("file.shortcuts")],
            ),
        ])
}
