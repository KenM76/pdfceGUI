//! # `text::files` — the copy the open/close/recent surface owns
//!
//! The strings [`crate::app::files`] and [`crate::app::recent`] show:
//! the file dialog's own title and filter names, and everything the Recent
//! control draws.
//!
//! ## Why these are not in [`crate::text::commands`]
//!
//! That catalog holds one thing: the **label and tooltip of a registered
//! command**, paired, because the tooltip's job is to say what the label
//! cannot fit. Nothing here is that. A dialog title is a string handed to the
//! operating system; a menu row is a file name the operator chose long ago
//! and this catalog only frames; "No recent documents" is a *state*, not a
//! verb. Putting them in `commands` would mean that file no longer answered
//! one question.
//!
//! ## ★ The dialog strings cross a shell boundary
//!
//! [`open_dialog_title`], [`filter_pdf`] and [`filter_all`] are interpolated
//! into a PowerShell script (see [`crate::app::files`] for why that script
//! exists and what replaces it). They are quoted there with **single quotes**,
//! which PowerShell does not interpolate, so the only character that could
//! break out is a single quote itself.
//!
//! **No string in this module may contain `'`.**
//! [`tests::the_dialog_strings_cannot_break_out_of_the_script`] enforces it,
//! so an apostrophe added to "pdfce's documents" fails the suite rather than
//! producing a parse error inside a child process nobody is watching. English
//! copy here has no need of one, and the day it does — a translation, most
//! likely — the fix is the `rfd` call the module header already carries,
//! which has no shell in it at all.

use std::path::Path;

/// The file dialog's title bar.
///
/// Names what is being asked for rather than the verb, because the verb is
/// already on the dialog's own accept button ("Open") and repeating it says
/// nothing. "PDF" appears because the filter defaults to PDFs and an operator
/// looking for a DWG should learn that here rather than from an empty file
/// list.
#[must_use]
pub fn open_dialog_title() -> &'static str {
    "Open a PDF document"
}

/// The name of the dialog's PDF filter. The pattern (`*.pdf`) is appended by
/// the caller, which is the convention every platform picker follows.
#[must_use]
pub fn filter_pdf() -> &'static str {
    "PDF documents"
}

/// The name of the dialog's everything filter.
///
/// Offered because a PDF with the wrong extension is a real thing an operator
/// hits — a file saved as `.pdf.txt` by a mail client, a drawing exported
/// without an extension at all — and pdfce reads a file by its bytes, not by
/// its name. A picker that could only offer `*.pdf` would make those files
/// unopenable through the only surface that opens files.
#[must_use]
pub fn filter_all() -> &'static str {
    "All files"
}

// ---------------------------------------------------------------------------
// ★ The Recent control's own LABEL and TOOLTIP are deliberately not here.
//
// It is a control for a registered command — `file.recent` — and a command's
// words live in `crate::text::commands`, whichever surface draws it. The
// custom item reads `crate::text::commands::file_recent()` for exactly the
// reason `crate::shell::menus`' header gives for a context-menu row reading
// its command's text: "a second copy of 'Delete' is a second copy that can
// drift". What IS here is everything the command's text cannot cover — the
// rows, which are file names, and the empty state, which is not a verb.
// ---------------------------------------------------------------------------

/// Shown inside the Recent menu when it has nothing to offer.
///
/// Two states share this sentence deliberately: nothing has ever been opened,
/// and everything that was is on a drive that cannot be reached right now.
/// The distinction is real but it is not one the operator can act on
/// differently — in both cases the answer is `Open…` — and a menu that
/// explained its own bookkeeping would be talking about itself.
#[must_use]
pub fn recent_empty() -> &'static str {
    "No recent documents"
}

/// One row of the Recent menu: the file's name.
///
/// The name alone, because a ribbon menu holding ten full paths is a menu as
/// wide as the window. The path is on hover — see [`recent_entry_tooltip`] —
/// which is where two files that share a name are told apart.
///
/// Falls back to the whole path when there is no file name to take, which
/// `Path::file_name` reports for a bare root or a path ending in `..`. A row
/// that rendered as an empty string would be a live control the operator
/// cannot see.
#[must_use]
pub fn recent_entry_label(path: &Path) -> String {
    path.file_name()
        .unwrap_or(path.as_os_str())
        .to_string_lossy()
        .into_owned()
}

/// One row of the Recent menu, on hover: where the file actually is.
///
/// The full path, unedited. Two drawings called `Sheet 1.pdf` in two job
/// folders are the ordinary case in this trade, and the folder is the only
/// thing that distinguishes them.
#[must_use]
pub fn recent_entry_tooltip(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// ★ **No dialog string can break out of the PowerShell script.**
    ///
    /// See the module header. The interim picker single-quotes these into a
    /// script; a `'` inside one would end the literal and the child process
    /// would fail to parse a program nobody can see. This is the mechanical
    /// half of that rule, and it fails at `cargo test` rather than at the
    /// operator's next click.
    #[test]
    fn the_dialog_strings_cannot_break_out_of_the_script() {
        for text in [open_dialog_title(), filter_pdf(), filter_all()] {
            assert!(
                !text.contains('\''),
                "`{text}` carries an apostrophe, which ends the single-quoted literal it is \
                 interpolated into (see this module's header)"
            );
            assert!(!text.is_empty());
        }
    }

    /// A row shows the file's name and hovers its whole path.
    #[test]
    fn a_row_names_the_file_and_hovers_where_it_is() {
        let path = PathBuf::from("D:\\jobs\\4471\\Sheet 1.pdf");
        assert_eq!(recent_entry_label(&path), "Sheet 1.pdf");
        assert_eq!(recent_entry_tooltip(&path), "D:\\jobs\\4471\\Sheet 1.pdf");
    }

    /// A path with no file name still renders something the operator can see.
    #[test]
    fn a_path_without_a_file_name_still_draws_a_row() {
        let label = recent_entry_label(Path::new("D:\\"));
        assert!(!label.is_empty(), "an empty row is an invisible control");
    }
}
