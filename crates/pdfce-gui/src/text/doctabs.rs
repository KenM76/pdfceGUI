//! # `text::doctabs` — what a document tab says, and what a page drag says it
//! is about to do
//!
//! Two families of string, and they are here together because they are read in
//! the same gesture: the operator drags a page out of one document, reads the
//! tab strip to find the other, and reads the caption to check where it will
//! land.
//!
//! ## ★ The unsaved marker is a PREFIX, and that is not a style choice
//!
//! A tab is truncated from the right with an ellipsis when the strip is
//! crowded — which is exactly when several documents are open, which is
//! exactly when knowing which of them has unsaved work matters most. A
//! trailing marker is the first thing the ellipsis eats. Word, Bluebeam and
//! Notepad++ all put theirs after the name and all three are showing a name
//! that has room; a strip of nine drawings is not.
//!
//! So it goes in front, where truncation cannot reach it.
//!
//! ## ★ And the tooltip is the whole path, always
//!
//! `SW41177.pdf` and `SW41177.pdf` are two different drawings when they are in
//! two different job folders, and a CAD office has that situation constantly.
//! The label is the file name because that is what fits; the tooltip is the
//! location because that is what disambiguates. Neither on its own is enough.

use std::path::Path;

/// The **unsaved marker**, in front of the name.
///
/// An asterisk rather than a bullet, a dot or a coloured label: it is ASCII, so
/// no font in any fallback chain can fail to draw it (this project has been
/// bitten by a codepoint that rendered as a substitution box in a sentence
/// whose whole job was to give directions), and it is the oldest and most
/// widely understood "there are unsaved changes here" marker in desktop
/// software.
const UNSAVED_MARKER: char = '*';

/// The label on a document's tab.
///
/// The file name, prefixed with [`UNSAVED_MARKER`] when the document has edits
/// that no save has taken. See this module's header for why the marker leads.
///
/// A path with no file-name component — which `Path::new("")` and a bare root
/// both are — falls back to the whole path rather than to an empty tab. An
/// empty tab is indistinguishable from a rendering failure.
#[must_use]
pub fn tab_label(path: &Path, unsaved: bool) -> String {
    let name = path.file_name().map_or_else(
        || path.display().to_string(),
        |n| n.to_string_lossy().into(),
    );
    if unsaved {
        format!("{UNSAVED_MARKER}{name}")
    } else {
        name
    }
}

/// The hover text on an **open** document's tab: where it is, and whether it
/// has unsaved work.
///
/// Two sentences rather than one, because they answer two different questions
/// and an operator scanning a strip of tabs is usually asking only one of them.
#[must_use]
pub fn tab_tooltip_open(path: &Path, unsaved: bool) -> String {
    let where_it_is = path.display();
    if unsaved {
        format!("{where_it_is}\nThis document has edits that have not been saved.")
    } else {
        where_it_is.to_string()
    }
}

/// The hover text on a **created** document's tab.
///
/// ★ It says the document has never been written, which the path cannot: a
/// created document's path is a *name*, so showing it as a location would
/// assert that a file exists at `Untitled 2.pdf` in whatever the operator reads
/// as the current directory.
#[must_use]
pub fn tab_tooltip_created(name: &Path) -> String {
    format!(
        "{} — made in this session and never saved to a file.",
        name.display()
    )
}

/// The hover text on a tab whose file would not open, whichever of the three
/// ways it failed.
///
/// The reason travels because the tab itself has room only for a name, and a
/// tab that says a file's name with no indication of why it is unreadable is a
/// tab the operator will click repeatedly.
#[must_use]
pub fn tab_tooltip_unopened(path: &Path, reason: &str) -> String {
    format!("{}\n{reason}", path.display())
}

/// The reason line for a tab waiting on a password.
///
/// Its own sentence rather than the `Failed` message, because §7.6 makes this a
/// third state and not a failure — pdfce *can* read this document and has not
/// been told how.
#[must_use]
pub const fn tab_reason_needs_password() -> &'static str {
    "This document is encrypted and pdfce has not been given the password."
}

/// **The window title**, from what is open.
///
/// Three forms, and the third is the one this function exists for:
///
/// | open | title |
/// |---|---|
/// | nothing | `pdfce` |
/// | one | `SW41177.pdf — pdfce` |
/// | several | `SW41177.pdf — 3 documents open — pdfce` |
///
/// ★ The count is there because the window title is the **only** place a
/// tabbed application reaches an operator who is not looking at it. Alt-Tab,
/// the taskbar and a screen-reader's window list all read this string and none
/// of them can see the tab strip; an operator who left three drawings marked
/// up and went to answer an email is entitled to be told so by the thing they
/// are about to close.
///
/// The active document leads in every form, because that is what every
/// application in the class does and what a truncated taskbar button keeps.
#[must_use]
pub fn window_title(active: Option<&Path>, count: usize) -> String {
    let base = crate::text::window_title();
    let Some(active) = active else {
        return base.to_owned();
    };
    let name = tab_label(active, false);
    if count > 1 {
        format!("{name} — {count} documents open — {base}")
    } else {
        format!("{name} — {base}")
    }
}

// ===========================================================================
// The page drag
// ===========================================================================

/// **Where a page drag would land, when it lands in the document it came
/// from.** A reorder.
///
/// Kept identical in shape to `crate::text::pages::drag_landing`, whose docs
/// carry the argument for saying it in page numbers as well as drawing a
/// caret: *"a hairline between two near-identical drawing sheets is precise
/// and not checkable"*.
#[must_use]
pub fn drag_landing_here(moving: usize, gap: usize, page_count: usize) -> String {
    crate::text::pages::drag_landing(moving, gap, page_count)
}

/// **Where a page drag would land, when it lands in a DIFFERENT document.**
///
/// ★ It says **copy**, and saying so is the whole point of the sentence.
///
/// Dragging a page from one open document into another does not remove it from
/// the one it came from, and an operator who assumed a move would find out by
/// discovering their source drawing intact tomorrow — or, worse, by assuming it
/// was not and deleting the wrong copy.
///
/// The reason it is a copy rather than a move is not squeamishness. A move is
/// two edits in two documents, and this application has one undo stack per
/// document: Ctrl+Z after a cross-document move would put the page back in the
/// source and leave the copy in the target, or take the copy out and leave the
/// source short, depending which document had focus. There is no ordering of
/// those two edits that makes one Ctrl+Z mean "undo what I just did". Windows
/// Explorer reaches the same conclusion for the same reason and copies between
/// volumes by default.
///
/// ★ It names the **source**, not the target. The operator is looking at the
/// target — it is the panel or the page view the pointer is inside — so the
/// document that is not on screen is the one the sentence has to supply.
#[must_use]
pub fn drag_landing_other(moving: usize, gap: usize, source: &str, page_count: usize) -> String {
    let sheets = if moving == 1 { "sheet" } else { "sheets" };
    if gap >= page_count {
        format!("Copy {moving} {sheets} from {source} to the end.")
    } else {
        format!(
            "Copy {moving} {sheets} from {source} to before page {}.",
            gap + 1
        )
    }
}

/// The drag is over something that is not a drop target.
///
/// Distinct from "it would change nothing", which
/// `crate::text::pages::drag_lands_nowhere` already says: this one means the
/// pointer is not over a page list or a page view at all.
#[must_use]
pub const fn drag_over_nothing() -> &'static str {
    "Drop this on a page list or on the page view to place it."
}

/// The whole document is being dragged and the target is the document it came
/// from — a copy of a document into itself.
///
/// **Refused rather than performed.** It is almost always a mis-drag, the
/// result is a document with every sheet twice, and undoing it is one keystroke
/// the operator has to know to reach for. Acrobat's Insert Pages will do it if
/// you ask in the dialog; nothing does it on a drag.
#[must_use]
pub const fn drag_refused_self_copy() -> &'static str {
    // ★ "the Pages tab" rather than the ribbon-path spelling with a U+25B8
    // in it. `icons::glyphs` refuses that codepoint in operator-visible
    // strings and is right to: the font stack cannot draw it, so it renders as
    // a substitution box — and this sentence's whole job is to give
    // directions. `text::dropped` carries the same note for the same reason.
    "Dragging every page of a document into itself would double it. Pick the sheets you want, \
     or use Insert from file on the Pages tab."
}

/// The document a drag would land in cannot take pages.
///
/// One sentence for the three engine refusals a caller cannot do anything
/// about at drop time — a certified document, an encrypted one, a page tree
/// that will not walk. The engine's own reason follows it, because it is the
/// only part that says which.
#[must_use]
pub fn drag_target_refused(reason: &str) -> String {
    format!("Those pages could not be placed here. {reason}")
}
