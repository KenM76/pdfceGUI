//! # text — the operator-visible string catalog
//!
//! **Every string a human can read in this application is defined here and
//! nowhere else.** That is a standing convention carried across from the
//! old crate (`ui_text.rs`, 7,912 lines and 1,193 entries), and it is
//! enforced mechanically rather than by review: a CI gate scans the module
//! tree for string literals outside the catalog and fails the build.
//!
//! ## Why a catalog rather than literals at the call site
//!
//! Three reasons, in order of weight:
//!
//! 1. **Copy is a design surface with its own quality bar.** pdfce's error
//!    prose distinguishes "your file is damaged" from "pdfce is not
//!    finished yet" from "this page would not draw", and it does that
//!    consistently because all three sentences are visible in one file
//!    next to each other. Scattered literals drift into three different
//!    voices within a month.
//! 2. **Translation, when it comes, is a mechanical job or an impossible
//!    one.** Which it is was decided the day the first literal was written.
//! 3. **It makes "no placeholders" checkable.** A label that says `TODO`
//!    or `Panel` is visible in the catalog in a way it never is inline.
//!
//! ## Why this is a directory, not a file
//!
//! The old catalog broke the project's 1,500-line ceiling by a factor of
//! five. It is split by AREA here from the first commit — `mod.rs` holds
//! shell-wide strings, and each future surface (ribbon, panels, dialogs,
//! tools) gets a sibling module — so the split never has to be done as a
//! migration. At S0 there is exactly one area, which is why `mod.rs` is
//! currently the whole catalog.
//!
//! ## Conventions
//!
//! - **Sentence case, no trailing period on labels; full sentences with
//!   punctuation for prose.** A label is a name; a message is a statement.
//! - **Name the thing that went wrong and what the operator can do.**
//!   "Failed to open" is not a message; it is a shrug with a capital F.
//! - **Never apologise, never blame the file without evidence.** The three
//!   open-failure functions below exist precisely so the shell does not
//!   have to guess which of "your document is broken" and "pdfce cannot do
//!   this yet" is true — `pdfce-core` returns structured errors that say.
//!
//! ## Areas
//!
//! | Module | Surface |
//! |---|---|
//! | `mod.rs` (this file) | shell-wide strings: window title, canvas states, the three open failures |
//! | [`ribbon`] | the ribbon's *structural* strings — tab labels, the one-line question each tab answers, group captions, mode labels |
//! | [`commands`] | the label and tooltip of every ribbon command |
//! | [`files`] | the open/close/recent surface: the file dialog's title and filters, and everything the Recent control draws |
//! | [`menus`] | the copy a **context menu** owns rather than borrows. Empty by construction — a menu row's words are its command's — and its header is the argument for why |
//! | [`panels`] | every string the dock's panel bodies show — Bookmarks, Layers, Signatures, Fonts, Objects, Properties |
//! | [`print`] | the print dialog — three tabs, the preview, the device refusals, and the commit button whose label carries the clip count |
//! | [`status`] | the status bar — the render-notes disclosure, the fit/zoom mirrors, and the editable page box |
//!
//! The split between `ribbon` and `commands` follows the seam in the data
//! itself: `crate::shell::manifest` consumes [`ribbon`] and
//! `crate::shell::commands` consumes [`commands`], so a change to one file
//! has one reviewer and one consumer. [`panels`] follows the same rule one
//! surface over — `crate::panels` is its sole consumer — and is itself a
//! directory, because six panel bodies' worth of copy is more than one file
//! should hold and the 1,500-line ceiling is not raised for catalogs.

/// The label and tooltip of every ribbon command. Consumed by
/// `crate::shell::commands`.
pub mod commands;
/// The copy the open/close/recent surface owns — the file dialog's title and
/// filter names, and every string the Recent control draws. Consumed by
/// `crate::app::files` and `crate::app::recent`.
pub mod files;
/// Every string the Forms panel shows. Consumed by `crate::panels::forms`.
pub mod forms;
/// The copy the **context-menu** surface owns, as distinct from the copy
/// its rows borrow from [`commands`]. Currently empty by construction; its
/// header carries the argument and the list of what would land there.
pub mod menus;
/// Every string the dock's panel bodies show. Consumed by `crate::panels`.
pub mod panels;
/// Every word the print dialog shows. Consumed by `crate::dialogs::print`.
pub mod print;
/// The ribbon's structural strings: tab labels and questions, group
/// captions, mode labels. Consumed by `crate::shell::manifest`.
pub mod ribbon;
/// Every string the status bar shows. Consumed by `crate::app::status`.
pub mod status;

use std::path::Path;

/// The window title.
///
/// Just the product name at S0. Once a document can be open, the
/// convention every document application follows is `<file> — pdfce`, and
/// that belongs here rather than at the `ViewportBuilder` call site.
#[must_use]
pub fn window_title() -> &'static str {
    "pdfce"
}

/// Shown on the canvas when nothing is open.
///
/// ★ **This sentence changed when `file.open` was wired**, and the change is
/// the rule rather than an edit. It used to read *"No document open. Start
/// pdfce with a PDF path, for example: pdfce-gui drawing.pdf"*, because at S0
/// there was no Open command and *"a message that names a control the
/// operator cannot find is worse than no message."* The command exists now —
/// on the File tab, on the quick-access toolbar, and on Ctrl+O — so the
/// message names it. The old wording would have been the same defect in
/// reverse: telling an operator to restart the application to do something
/// there is a button for.
///
/// The command line stays in the sentence because it is still true and is
/// still how a file association or a shell "Open with" reaches pdfce.
#[must_use]
pub fn canvas_no_document() -> &'static str {
    "No document open. Choose File ▸ Open, press Ctrl+O, or start pdfce with a PDF path."
}

/// Shown when a document opened successfully but contains no pages.
///
/// This is a real, legal PDF: `/Count 0`. Presenting it as a failure would
/// be a lie about the operator's file, which is why the page-index clamp in
/// [`crate::viewer::clamp_page_index`] maps the empty document to page 0
/// rather than panicking — the "no pages" condition is a *presentation*
/// decision and this is the presentation.
#[must_use]
pub fn canvas_no_pages() -> &'static str {
    "This document has no pages."
}

/// Shown when the current page could not be rasterized.
///
/// The document stays open. One page that will not draw is not a reason to
/// close a file the operator can still navigate, and it is not the same
/// event as a file that would not load — hence a distinct message rather
/// than reusing [`open_failed`].
///
/// `detail` is `pdfce-render`'s own error `Display`, passed through rather
/// than rewritten: the renderer's errors are structured, specific
/// diagnostics ("requested raster size 115200x86400 exceeds
/// MAX_PIXMAP_EDGE"), and replacing one with "an error occurred" throws
/// away the only part of the sentence that helps.
#[must_use]
pub fn canvas_render_failed(detail: &str) -> String {
    format!("This page could not be drawn. {detail}")
}

/// Shown when the background render thread died without reporting.
///
/// Distinguished from [`canvas_render_failed`] because the causes are
/// different in kind: a render *failure* is something about this page, and
/// a stopped worker is something about the process. Conflating them would
/// send an operator looking at their document for a fault that is ours.
#[must_use]
pub fn canvas_render_worker_stopped() -> &'static str {
    "The page renderer stopped unexpectedly. Reopen the document to try again."
}

/// The document could not be read: it is damaged, truncated, or not a PDF.
///
/// One of **three distinct ways to fail, said three distinct ways** — a
/// distinction carried across from the old shell because it is one of the
/// things pdfce does that most viewers do not:
///
/// - this function — *the file is wrong*;
/// - [`open_unsupported`] — *the file is fine and pdfce is not finished*;
/// - [`open_needs_password`] — *the file is encrypted and pdfce has not
///   been told the password*.
///
/// The branch between them is made on **structured error data** from
/// `pdfce-core`, never by matching on a message string. That is what makes
/// the distinction reliable rather than a heuristic that decays.
#[must_use]
pub fn open_failed(path: &Path, detail: &str) -> String {
    format!(
        "{} could not be opened. {detail}",
        path.file_name()
            .unwrap_or(path.as_os_str())
            .to_string_lossy()
    )
}

/// The document is well-formed and uses something pdfce does not implement.
///
/// Saying "failed to open" here would tell the operator a lie about their
/// own file. `pdfce-core` detects such a document and refuses it *cleanly*
/// rather than misparsing it into plausible-looking garbage, and this
/// sentence is the other half of that honesty.
#[must_use]
pub fn open_unsupported(path: &Path, detail: &str) -> String {
    format!(
        "{} uses a PDF feature pdfce does not support yet. {detail}",
        path.file_name()
            .unwrap_or(path.as_os_str())
            .to_string_lossy()
    )
}

/// The document is encrypted with a password pdfce has not been given.
///
/// A third thing: neither damaged nor unsupported. pdfce *can* decrypt this
/// file and has not been told how.
///
/// S0 has no password prompt, and this message says so plainly instead of
/// showing an input the shell would then ignore. That is the "no
/// placeholders" invariant (`PROJECT_PLAN.md` §3): unavailable renders
/// nothing, and greying is for *temporarily* unavailable. The prompt lands
/// with the rest of the open/save surface at stage S2.
#[must_use]
pub fn open_needs_password(path: &Path) -> String {
    format!(
        "{} is password-protected. This build cannot yet prompt for a password.",
        path.file_name()
            .unwrap_or(path.as_os_str())
            .to_string_lossy()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// The three open-failure sentences must be genuinely different.
    ///
    /// Not a tautology test: the whole value of the three-way distinction
    /// is that an operator can tell from the words alone which of "my file
    /// is broken", "pdfce is not finished" and "I need to type a password"
    /// is true. Three functions that produced near-identical prose would
    /// satisfy the type system and defeat the design.
    #[test]
    fn the_three_open_failures_read_differently() {
        let p = PathBuf::from("drawing.pdf");
        let a = open_failed(&p, "unexpected end of file");
        let b = open_unsupported(&p, "hybrid-reference file");
        let c = open_needs_password(&p);
        assert_ne!(a, b);
        assert_ne!(b, c);
        assert_ne!(a, c);
        // And each must name the file, or the operator with two documents
        // open cannot tell which one is complaining.
        for message in [&a, &b, &c] {
            assert!(message.contains("drawing.pdf"));
        }
    }

    /// A path with no file name must still produce a usable sentence.
    ///
    /// `Path::file_name` returns `None` for a bare root or a path ending in
    /// `..`, and an unwrap there would turn a nonsense command line into a
    /// panic instead of a message.
    #[test]
    fn a_path_without_a_file_name_still_names_something() {
        let message = open_failed(Path::new("D:\\"), "not a PDF");
        assert!(message.contains("D:\\"));
    }
}
