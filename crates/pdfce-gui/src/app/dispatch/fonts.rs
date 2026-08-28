//! # `app::dispatch::fonts` — the Tools tab's two font commands
//!
//! Split out of [`super`] under **R2** on 2026-08-28, when that file crossed
//! 1,500 lines for the fifth time.
//!
//! ## ★★ The seam, and why it is a subject rather than a size
//!
//! Both commands here — `tools.embed_fonts` and, when it is wired, its mirror
//! `tools.unembed_fonts` — share a property no other command in this shell has:
//! **their operand is not in the document and not in the dialog either.** It is
//! on the operator's disk, found through a folder list the operator maintains,
//! by a resolver this shell owns outright because `pdfce-core` *"never goes
//! looking"*.
//!
//! That gives them a dispatch shape nothing else here has. Every other
//! window-opening arm is one line — `self.dialogs.open_x(&self.status)` —
//! because a document is the only input. These two need the preference as well,
//! and they can **decline with a sentence**, which is a branch a one-line arm
//! cannot express.
//!
//! ## ★ The harness seam lives here, not in the preference
//!
//! [`folders`] appends a directory named by an environment variable, so
//! `ui-verify` can supply the one input an operator supplies through Settings
//! without the harness rewriting `userdata/preferences.txt` — a file that
//! belongs to whoever is running the build, and that a check has no business
//! editing. It is the same seam `PDFCE_DIAG_SAVE_PATH` is, for the same reason
//! `save_copy`'s header gives: the alternative is a check that mutates
//! persisted state and leaves it mutated.

use std::path::PathBuf;

use crate::app::prefs::Prefs;
use crate::app::state::Status;
use crate::dialogs::DialogsState;

/// A font folder supplied by the harness, in addition to the operator's.
///
/// ★ **Additional, never a replacement.** A variable that *replaced* the
/// preference would let a check pass on a build whose preference plumbing was
/// broken end to end — the harness would be testing its own environment
/// variable. Appending means the operator's folders are still read on the same
/// run, so the check exercises the real path and only adds to its input.
// ui-text-exempt: an environment variable name, never displayed.
const FONT_DIR_ENV: &str = "PDFCE_DIAG_FONT_DIR";

/// Whether this file owns `id`.
///
/// `pub(crate)` for [`super::routes::handles`]' reason: `shell::commands::reach`'s
/// reachability checker must be able to evaluate every guard arm it finds, and
/// a guard it cannot evaluate is a place commands could hide from the check
/// that exists to find them.
#[must_use]
pub(crate) fn handles(id: &str) -> bool {
    // ui-text-exempt: registered command ids, never displayed.
    matches!(id, "tools.embed_fonts")
}

/// Where pdfce may look for a donor font on this run.
///
/// The operator's list first, then anything the harness named. Order is search
/// order and the first match wins — see [`crate::app::prefs::fonts`] — so the
/// operator's own folders take precedence over a harness's, which is the only
/// ordering that keeps a driven run honest about what a real one would do.
#[must_use]
pub(crate) fn folders(prefs: &Prefs) -> Vec<PathBuf> {
    let mut out = prefs.font_folders.clone();
    if let Ok(extra) = std::env::var(FONT_DIR_ENV) {
        // Semicolon-separated, matching the platform's own `PATH` convention
        // rather than inventing one. A colon would be ambiguous with a drive
        // letter on the platform this ships on.
        for part in extra.split(';') {
            if let Some(path) = crate::app::prefs::fonts::parse_one(part) {
                crate::app::prefs::fonts::add(&mut out, &path);
            }
        }
    }
    out
}

/// Dispatch a font command.
///
/// ★★★ **`tools.embed_fonts` — registered, drawn on the Tools tab and inert for
/// the whole life of the project.** Wired 2026-08-28.
///
/// Its `SCAFFOLDED` reason quoted a premise that had expired — *"at S3 `Action`
/// carries zoom and page navigation and nothing else"* — and the entry itself
/// flagged that. Re-deriving it turned up a **second, unrecorded** dependency
/// that was the real one: `EmbedRequest::supplied` is a donor map *"the shell
/// resolved for it"*, and pdfce never goes looking. So the command was blocked
/// on a font-folder preference that did not exist until the same day, and that
/// dependency was in neither register.
///
/// ⇒ **A blocker can be correct for the wrong reason.** It is the least visible
/// of the five ways this project's scaffold list has gone wrong: nothing about
/// such an entry looks stale, and the only thing that finds it is asking what
/// the verb's own *request struct* requires rather than whether the verb exists.
///
/// ## ★ It can decline with a sentence, and the sentence is recorded
///
/// A document whose fonts are all embedded is the **normal** case, not an
/// error, and opening a window to say so would be a modal an operator has to
/// dismiss to learn they did not need it. So the construction declines, and the
/// decline goes to `record_note` — the same channel a refused clipboard cut
/// uses, for the same reason: the operator still believes the gesture worked,
/// and silence is what would leave them believing it.
pub(crate) fn dispatch(id: &str, dialogs: &mut DialogsState, status: &Status, prefs: &Prefs) {
    // ui-text-exempt: registered command ids, never displayed.
    if id != "tools.embed_fonts" {
        return;
    }
    // The epoch is read before the window is built, so the note is stamped with
    // the revision the operator is looking at. Nothing here edits, so it cannot
    // move underneath.
    let Status::Open(doc) = status else {
        return;
    };
    let epoch = doc.edit_epoch;
    let folders = folders(prefs);
    if let Some(note) = dialogs.open_embed_fonts(status, &folders) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "embed-fonts-declined folders={} detail=nothing-to-open",
                folders.len()
            )
        });
        crate::app::actions::record_note(epoch, note);
    }
}
