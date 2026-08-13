//! The **Tools** tab — *what do I run across files, or configure once?*
//!
//! `RIBBON_IA.md` §5.7. Three groups: Batch, Fonts, Diagnostics.
//!
//! # What this tab became
//!
//! In the salvage source it was three groups of one control each — a batch
//! *panel* toggle, font folders, and redaction — and on a 1936 px window
//! that left well over a thousand pixels of empty band. The reorganisation
//! does two things:
//!
//! 1. **Redact leaves**, to Edit ▸ Protect, where a user editing a
//!    document will actually look for it.
//! 2. **The batch panel's contents surface as commands.** `Merge files…`
//!    and `Split files…` were reachable only by opening a pane and then
//!    choosing within it; the pane stays, and the ribbon becomes the path
//!    that can be found without knowing it is there. The pane's third
//!    job — inserting pages from another file — is not here, because
//!    that one changes *this* document and belongs on Pages.
//!
//! The result is a tab defined by a rule rather than by leftovers: things
//! that either operate on files **other than the open one**, or are
//! configured once and rarely touched.
//!
//! # The Pages/Tools line, restated because it is the one that gets blurred
//!
//! `pages.split` and `tools.split_files` are two commands, not one command
//! twice. So are `pages.merge_into` and `tools.merge_files`. The
//! distinction is which document changes:
//!
//! - **Pages** — this document's page set changes. Undoable. Respects the
//!   thumbnail rail's selection.
//! - **Tools** — new files are produced. This document is untouched. The
//!   inputs are chosen from disk.
//!
//! Both tooltips point at the other, so an operator who reached for the
//! wrong one is told where the right one lives rather than getting a
//! dialog that asks the wrong question.
//!
//! # Render diagnostics belongs here rather than in the status bar
//!
//! It is currently a run of text in the status bar. That surface is for
//! the controls a user touches constantly, and a diagnostic readout is
//! neither a control nor constant — it is a thing you go and look at when
//! something is wrong. Moving it here also gives it room to be more than
//! one line.
//!
//! # What is absent
//!
//! `Batch print…`, the whole **Compare** group, `OCR…` (blocked), and the
//! whole **Validate** group (PDF/A validate & convert, Optimise) are
//! **N**. Compare is the one absence an AEC reviewer will name first, and
//! it is a large build; it is an open question in `RIBBON_IA.md` §8 rather
//! than a scheduled item, and [`super::PLANNED`] records it as such.

use super::{command, group};
use crate::text::ribbon;
use egui_shell::manifest::Tab;

/// The Tools tab.
pub(super) fn tab() -> Tab {
    Tab::new("tools", ribbon::tab_tools())
        .with_question(ribbon::question_tools())
        .with_groups([
            // ---------------------------------------------------------------
            // Batch — jobs that produce new files. `Batch print…` is **N**.
            // ---------------------------------------------------------------
            group(
                "batch",
                ribbon::group_tools_batch(),
                [command("tools.merge_files"), command("tools.split_files")],
            ),
            // ---------------------------------------------------------------
            // Fonts — configured once, rarely touched.
            //
            // Font folders is a session-scoped setting (the folders are
            // remembered for the session only), and embed/unembed act on
            // the open document. They share a band because they are the
            // same subject from the operator's side: what happens when a
            // document needs a typeface.
            //
            // Note that the Fonts *panel* is not here — it moved to File ▸
            // Document, because it describes what is inside the file.
            // These are the two verbs; that is the inventory.
            // ---------------------------------------------------------------
            group(
                "fonts",
                ribbon::group_tools_fonts(),
                [
                    command("tools.font_folders"),
                    command("tools.embed_fonts"),
                    command("tools.unembed_fonts"),
                ],
            ),
            // ---------------------------------------------------------------
            // Diagnostics.
            // ---------------------------------------------------------------
            group(
                "diagnostics",
                ribbon::group_tools_diagnostics(),
                [command("tools.render_diagnostics")],
            ),
        ])
}
