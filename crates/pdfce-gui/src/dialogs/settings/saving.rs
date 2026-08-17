//! # `dialogs::settings::saving` — two settings nobody can see
//!
//! Both change the **bytes pdfce writes** and neither changes anything visible.
//! That is stated in both radius lines in the same words, and it is the whole
//! reason they are grouped together rather than filed with the settings whose
//! effects an operator can look at.
//!
//! ## Why a setting nobody can see is worth having
//!
//! Because pdfce's round-trip discipline is a promise about bytes: **objects
//! pdfce did not logically touch are re-emitted byte-identical.** A save that
//! rewrites two bytes on every line of a file's index is a diff of ten thousand
//! bytes in a document nobody edited — invisible in a viewer, and immediately
//! visible to version control, to a checksum, and to anyone diffing a
//! before-and-after.
//!
//! That is a real consequence for the people who use this program, and the
//! window says so rather than treating "nothing visible" as "nothing".

use egui::Ui;
use pdfce_core::settings::{TrailingEol, XrefEntryEol};

use super::{Draft, widgets};
use crate::text::settings as t;

/// How each cross-reference entry's line ends.
///
/// # The default was changed on an operator ruling, and it is the interesting one
///
/// This shipped as a fixed `SP LF` for a long time, on a recommendation the
/// ambiguity register itself flagged as sourceless. The register said the
/// shipped default was *"arguably wrong on pdfce's own invariant"*, and it was:
/// a full rewrite of a `CR LF` file under a fixed `SP LF` changes two bytes in
/// every entry, so a 5,000-object file gets a 10,000-byte diff without anybody
/// editing it.
///
/// The reason it shipped wrong is worth keeping: *"match the source"* needed an
/// **observation of the base file's bytes** that no channel carried. The
/// recommendation was right and unimplementable at the same time. The channel
/// now exists — `xref::observed_entry_eol` reads the form out of the base file
/// — and the operator's 2026-08-08 ruling was to use it.
///
/// # ★ Three legal forms, and the illegal ones are deliberately absent
///
/// §7.5.4 fixes the entry at exactly **20 bytes** and permits three and only
/// three forms for bytes 18–19. `LF CR`, bare `LF`, bare `CR`, `SP SP` and
/// `SP CR LF` are **not legal and are not offered**: a settings file is not a
/// licence to emit a non-conforming file, and a future hand must not
/// "helpfully" add them for completeness.
///
/// # The rendering order is the contract, not the declaration order
///
/// `MatchSource` is drawn **first**, ahead of the fixed forms, and the catalog
/// declares them in a different order. The default's own note says *"picking a
/// fixed form below"*, which is only true because of the order here — so this
/// sequence is load-bearing prose as well as layout, and a reorder would make a
/// shipped sentence wrong.
///
/// # The two options with no note
///
/// *"Space then carriage return"* and *"Carriage return then newline"* describe
/// themselves completely. These are the two entries the `Option<&str>` in
/// [`widgets::option`] exists for; padding them out to match their neighbours
/// would be the noise that trains a reader to stop reading notes.
pub fn xref_entry_eol(ui: &mut Ui, draft: &mut Draft) {
    widgets::header(
        ui,
        t::xref_eol_title(),
        t::xref_eol_silence(),
        t::xref_eol_radius(),
    );
    widgets::option(
        ui,
        &mut draft.working.xref_entry_eol,
        XrefEntryEol::MatchSource,
        t::xref_eol_match_label(),
        Some(t::xref_eol_match_note()),
    );
    widgets::option(
        ui,
        &mut draft.working.xref_entry_eol,
        XrefEntryEol::SpaceLf,
        t::xref_eol_space_lf_label(),
        Some(t::xref_eol_space_lf_note()),
    );
    widgets::option(
        ui,
        &mut draft.working.xref_entry_eol,
        XrefEntryEol::SpaceCr,
        t::xref_eol_space_cr_label(),
        None,
    );
    widgets::option(
        ui,
        &mut draft.working.xref_entry_eol,
        XrefEntryEol::CrLf,
        t::xref_eol_cr_lf_label(),
        None,
    );
}

/// Whether one byte follows the end-of-file marker.
///
/// # Explicitly low-value as a knob, and that is why it exists
///
/// §7.5.1 requires every line to be EOL-terminated; §7.5.5 says the last line
/// *"contains only"* `%%EOF`. **Both readings are self-consistent and the
/// standard does not choose.** In practice it matters to almost nobody: a
/// trailing EOL never breaks a reader's backward `%%EOF` scan, and §7.2.3
/// requires one before a following object on the incremental-append path
/// anyway.
///
/// It is a setting because the choice was previously **hard-coded and labelled
/// in the source as a recorded spec ambiguity** — and an engineer who finds
/// that label will ask where the switch is. A documented ambiguity with no
/// control is a decision pdfce made and hid; the cost of the control is one
/// radio pair.
///
/// # ★ The guess disclosure the old note omitted
///
/// The note read as a plain recommendation. It now says which of the two
/// readings pdfce took and that it took one, which is what the window's own
/// contract requires of a tier-(d) default.
pub fn trailing_eol(ui: &mut Ui, draft: &mut Draft) {
    widgets::header(
        ui,
        t::trailing_eol_title(),
        t::trailing_eol_silence(),
        t::trailing_eol_radius(),
    );
    widgets::option(
        ui,
        &mut draft.working.trailing_eol,
        TrailingEol::Lf,
        t::trailing_eol_lf_label(),
        Some(t::trailing_eol_lf_note()),
    );
    widgets::option(
        ui,
        &mut draft.working.trailing_eol,
        TrailingEol::None,
        t::trailing_eol_none_label(),
        Some(t::trailing_eol_none_note()),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★ Only the three legal entry forms are offered.
    ///
    /// §7.5.4 permits exactly three, and the temptation a future hand will feel
    /// is to add the others "for completeness" — bare `LF` in particular, since
    /// it is what a text editor produces. Every one of them makes the entry the
    /// wrong length and the file non-conforming.
    ///
    /// Asserted by round-tripping each offered value through the engine's own
    /// byte encoding: a form that is not two bytes cannot be legal, and a form
    /// the engine does not know would not compile.
    #[test]
    fn only_the_legal_entry_forms_are_offered() {
        // `resolve` needs a base document to observe; `MatchSource` against an
        // empty base falls back to `SpaceLf`, which is the documented
        // behaviour for a file with no such index.
        for form in [
            XrefEntryEol::MatchSource,
            XrefEntryEol::SpaceLf,
            XrefEntryEol::SpaceCr,
            XrefEntryEol::CrLf,
        ] {
            let bytes = form.resolve(&[]).bytes();
            assert_eq!(
                bytes.len(),
                2,
                "{form:?} does not encode as two bytes and would break the 20-byte entry"
            );
            assert!(
                matches!(bytes, [b' ', b'\n'] | [b' ', b'\r'] | [b'\r', b'\n']),
                "{form:?} encodes as {bytes:?}, which is not one of the three legal forms"
            );
        }
    }

    /// `MatchSource` with nothing to match falls back to a legal fixed form.
    ///
    /// The case the default's own note promises — *"files that have no index of
    /// this kind get a space then a newline"* — and one an operator will hit
    /// without knowing it, because a cross-reference **stream** file has no
    /// entry EOL at all.
    #[test]
    fn matching_nothing_falls_back_to_the_documented_form() {
        assert_eq!(
            XrefEntryEol::MatchSource.resolve(&[]).bytes(),
            [b' ', b'\n']
        );
    }
}
