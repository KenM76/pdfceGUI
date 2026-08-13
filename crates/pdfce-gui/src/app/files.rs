//! # `app::files` — how a path gets from an operator (or a harness) to
//! [`crate::app::actions::Action::Open`]
//!
//! One question, asked in one place: **which document does the operator want
//! to open?** Everything downstream of the answer — loading, the three-way
//! failure split, forgetting the previous document's panel state, recording
//! the file in the recent list — is [`crate::app::PdfceApp::open_path`]'s and
//! is reached through the action funnel, never from here.
//!
//! ## ★ Rule 1: substitute the dialog's ANSWER, never its interaction
//!
//! `D:\dev\rag\egui\native_file_dialog_is_a_hard_wall_substitute_the_answer_via_env_var.md`
//! records this as a **pattern in this project**, promoted after its second
//! independent instance (`diag::font_dirs`, then `PDFCE_DIAG_EXPORT_DIR`):
//!
//! > A native file/folder picker hands control to the OS shell, outside
//! > egui's own event loop. Neither `eframe::App::raw_input_hook` synthetic
//! > events nor OS-level `SendInput`/`PostMessage` automation can drive the
//! > native dialog's own widget tree — it is a separate top-level window
//! > owned by the shell, not an `egui::Window`. […] Don't try to script the
//! > dialog. Check an environment variable BEFORE opening it; if set, use its
//! > value as the dialog's result and skip opening the dialog at all.
//!
//! So [`pick_document`] checks [`DIAG_OPEN_PATH`] first, and the seam
//! replaces exactly **one** call: everything the harness is actually testing
//! — the action, the load, the failure classification, the recent list, the
//! panels forgetting the previous document — runs through the identical code
//! path a real click produces. The RAG's own instruction for anything new is
//! followed here rather than rediscovered: *"any future `rfd` call added to a
//! scripted-driven GUI should get the same `PDFCE_DIAG_<PURPOSE>` seam from
//! the start, not after the harness fails to reach it."*
//!
//! | `PDFCE_DIAG_OPEN_PATH` | [`pick_document`] returns | For |
//! |---|---|---|
//! | unset | whatever the native picker says | the operator |
//! | a path | [`Picked::Path`] — no dialog opens | a harness opening a second document |
//! | set but **empty** | [`Picked::Cancelled`] — no dialog opens | a harness exercising the *cancel* path, which is the one that must change nothing |
//!
//! ## ★ Rule 2: `rfd` is the right dependency and this work may not add it
//!
//! The old shell picks files with [`rfd`](https://docs.rs/rfd) 0.17.2 — it is
//! in `D:\Dev\pdfce\crates\pdfce-gui\Cargo.toml` and therefore already in
//! pdfce's lockfile, licence-vetted, with no C dependency. It is **not** in
//! this crate's manifest, and this crate's manifest is not this work's to
//! edit, so the dependency is not added silently. The one line, verbatim,
//! under `[dependencies]`:
//!
//! ```toml
//! # rfd: native file-open/save dialogs, no C dependency (docs/PRIOR_ART.md). MIT.
//! rfd = "0.17.2"
//! ```
//!
//! and [`native_pick`] then becomes, in full:
//!
//! ```ignore
//! rfd::FileDialog::new()
//!     .set_title(crate::text::files::open_dialog_title())
//!     .add_filter(crate::text::files::filter_pdf(), &["pdf"])
//!     .add_filter(crate::text::files::filter_all(), &["*"])
//!     .pick_file()
//!     .map_or(Picked::Cancelled, Picked::Path)
//! ```
//!
//! Nothing else in this module, in `app`, or in `shell` changes when that
//! happens: the seam, the action, the command and the dirty-document rule are
//! all on this side of the call.
//!
//! ## What runs until then: the interim Windows picker
//!
//! A command that is drawn, enabled and inert is defect **D1**'s exact shape,
//! and *"a reader that cannot open a second file is not a reader"*. So
//! [`native_pick`] does not shrug: on Windows it asks the operating system
//! for its own picker through `powershell.exe`, which is present on every
//! supported Windows and needs no crate at all.
//!
//! What that costs, stated plainly rather than discovered later:
//!
//! - **It is Windows-only.** Every other platform gets
//!   [`Picked::Unavailable`], because a `zenity`/`kdialog`/`osascript` arm
//!   written on a Windows machine could not be compiled here, let alone run —
//!   and untested code behind a `cfg` that this build never evaluates is
//!   worse than an honest gap. `rfd` covers all three properly.
//! - **The dialog is not owned by the pdfce window**, so it does not travel
//!   with it and does not centre on it. `rfd` takes a parent handle.
//! - **It costs a process launch** (tens of milliseconds) before the dialog
//!   appears, and it blocks the UI thread while the dialog is open — which
//!   `rfd::FileDialog::pick_file` also does, so only the first half is new.
//! - **The path comes back hex-encoded.** See [`native_pick`]; console code
//!   pages mangle non-ASCII file names and a mangled path names a different
//!   file, or none, while looking exactly like a real answer.
//!
//! ## ★ Rule 3: no test may dispatch `file.open`
//!
//! On the machine this is built on, dispatching `file.open` opens a **real
//! modal dialog** and blocks until a human dismisses it. A `cargo test` that
//! did that would hang the suite with an invisible window behind the
//! terminal. So the tests here cover [`from_env`], which is pure, and
//! `crate::app`'s tests cover [`PdfceApp::open_picked`]
//! ([`crate::app::PdfceApp`]) — the translation from a [`Picked`] to an
//! action — with all three variants supplied directly. The only untested
//! millimetre is the `env::var_os` read itself, and it cannot be tested:
//! `std::env::set_var` is `unsafe` in edition 2024 and this crate is
//! `#![forbid(unsafe_code)]`.
//!
//! ## ★ The dirty-document rule, stated where it will be needed
//!
//! [`crate::app::actions`]' header has always said an Open must not proceed
//! while a save is pending. **There is no save in this build** — `file.save`
//! is in `crate::shell::manifest::PLANNED` blocked on autosave and crash
//! recovery, and `file.save_copy` has no arm — so there is nothing to be
//! pending and no dialog is built for a condition that cannot occur. What
//! exists instead is one predicate,
//! [`crate::app::PdfceApp::save_pending`], consulted by both
//! [`crate::app::actions::Action::Open`] and
//! [`crate::app::actions::Action::Close`], returning `false` with the whole
//! rule written above it. The day a save lands, that function reads its state
//! and the two arms grow a confirmation — in one place, already wired, rather
//! than in two arms somebody has to remember to find.

use std::ffi::OsString;
use std::path::PathBuf;

/// The environment variable that answers the dialog instead of opening it.
///
/// `PDFCE_DIAG_*` is this project's established prefix for a
/// diagnostics-only seam — `PDFCE_DIAG`, `PDFCE_DIAG_VIEWPORT`,
/// `PDFCE_DIAG_EXPORT_DIR` — and the naming is part of the pattern rather
/// than decoration: a reader who finds one of them knows what kind of thing
/// the others are.
pub const DIAG_OPEN_PATH: &str = "PDFCE_DIAG_OPEN_PATH"; // ui-text-exempt: an environment variable name, never displayed

/// What asking for a document produced.
///
/// Three answers rather than `Option<PathBuf>`, because the third one is not
/// a refinement of "no path": **cancelled** is the operator saying no, and
/// **unavailable** is this build having no way to ask. They call for
/// different behaviour (silence versus a trace naming a build gap) and
/// conflating them is how "the button does nothing" becomes indistinguishable
/// from "the operator changed their mind".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Picked {
    /// The operator (or the diagnostic seam) named this file.
    Path(PathBuf),
    /// The operator dismissed the dialog. Nothing happens, and nothing is
    /// traced beyond the fact — a cancelled Open is a complete, correct,
    /// uninteresting outcome.
    Cancelled,
    /// This build has no way to ask. See the module header: on Windows this
    /// means `powershell.exe` could not be run at all, which is close to
    /// unreachable; elsewhere it is the honest state of the interim picker.
    Unavailable,
}

/// **Ask for a document to open.**
///
/// The diagnostic seam first, the platform picker second. See the module
/// header for why that order is the whole point.
///
/// Blocks while a dialog is open, exactly as `rfd::FileDialog::pick_file`
/// does: the caller is the command dispatcher, which runs between frames, and
/// a picker that returned asynchronously would need a state machine to hold
/// the half-finished intent across frames — machinery worth building for a
/// dialog pdfce draws itself, not for one the OS owns.
#[must_use]
pub fn pick_document() -> Picked {
    if let Some(answer) = from_env(std::env::var_os(DIAG_OPEN_PATH)) {
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed.
                "open-picked source=env answer={answer:?}"
            )
        });
        return answer;
    }
    let answer = native_pick();
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed.
            "open-picked source=native answer={answer:?}"
        )
    });
    answer
}

/// Read the diagnostic seam, if it is set. Pure, so it can be tested.
///
/// `None` means "the variable is not set, go and ask properly". `Some` is a
/// complete answer that the dialog is then never opened for — which is the
/// property the harness depends on, because a dialog that opened *as well*
/// would still block it.
#[must_use]
pub fn from_env(value: Option<OsString>) -> Option<Picked> {
    let value = value?;
    if value.is_empty() {
        // A deliberate, reachable answer rather than an oversight: it is how
        // a harness drives the branch in which the operator says no, without
        // which "Open changed nothing" cannot be distinguished from "Open was
        // never reached".
        return Some(Picked::Cancelled);
    }
    Some(Picked::Path(PathBuf::from(value)))
}

/// Ask the platform for its own file picker.
///
/// Replaced wholesale by four lines of `rfd` the moment this crate's manifest
/// may carry it — see the module header, which holds both the manifest line
/// and the replacement body.
#[cfg(windows)]
fn native_pick() -> Picked {
    use std::os::windows::process::CommandExt;
    use std::process::{Command, Stdio};

    /// `CREATE_NO_WINDOW`. Without it the child gets a console that flashes
    /// on screen in front of the operator's document for as long as
    /// PowerShell takes to start — which is exactly long enough to see.
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    // ui-text-exempt: this is a PowerShell program, not operator copy. Every
    // string an operator can READ inside it — the dialog title and the two
    // filter names — comes from `crate::text::files` and is interpolated
    // below, which is the rule R1 actually states.
    //
    // Three details are load-bearing and none is decoration:
    //
    //   -STA          WinForms will not show a dialog from an MTA thread.
    //                 `powershell.exe` (Windows PowerShell 5.1, present on
    //                 every supported Windows) is STA by default; passing it
    //                 explicitly means a machine where `pwsh` shadows the
    //                 name still works.
    //   BitConverter  The chosen path is returned as ASCII hex, NOT as text.
    //                 A redirected PowerShell stdout is encoded with the
    //                 console code page, so `C:\Zeichnungen\Übersicht.pdf`
    //                 arrives mojibake — and a mojibake path names a
    //                 different file, or none, while looking exactly like a
    //                 real answer. Hex is immune to every encoding question
    //                 and decodes in ten lines.
    //   no output     on cancel, so empty stdout IS the cancel signal and
    //                 needs no second channel.
    let script = format!(
        "$ErrorActionPreference='Stop';\
         Add-Type -AssemblyName System.Windows.Forms;\
         $d=New-Object System.Windows.Forms.OpenFileDialog;\
         $d.Title='{title}';\
         $d.Filter='{pdf} (*.pdf)|*.pdf|{all} (*.*)|*.*';\
         $d.Multiselect=$false;\
         $d.CheckFileExists=$true;\
         if($d.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK){{\
         [Console]::Out.Write([BitConverter]::ToString(\
         [Text.Encoding]::UTF8.GetBytes($d.FileName)).Replace('-',''))}}",
        title = crate::text::files::open_dialog_title(),
        pdf = crate::text::files::filter_pdf(),
        all = crate::text::files::filter_all(),
    );

    let output = Command::new("powershell.exe")
        .args(["-NoProfile", "-STA", "-Command"])
        .arg(&script)
        .creation_flags(CREATE_NO_WINDOW)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let hex = String::from_utf8_lossy(&out.stdout);
            let hex = hex.trim();
            if hex.is_empty() {
                return Picked::Cancelled;
            }
            match decode_hex(hex) {
                // The bytes are UTF-8 by construction — the script encoded
                // them that way — so a failure here means the output was not
                // the script's, which is a broken picker rather than a
                // cancelled one.
                Some(bytes) => match String::from_utf8(bytes) {
                    Ok(path) => Picked::Path(PathBuf::from(path)),
                    Err(_) => Picked::Unavailable,
                },
                None => Picked::Unavailable,
            }
        }
        // A non-zero exit or a launch failure both mean the same thing to the
        // caller: this build could not ask. The reason goes to the trace,
        // where whoever is looking at a machine they cannot see will find it.
        Ok(out) => {
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed.
                    "open-picker-failed reason=exit status={:?}",
                    out.status.code()
                )
            });
            Picked::Unavailable
        }
        Err(error) => {
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed.
                    "open-picker-failed reason=spawn error={error}"
                )
            });
            Picked::Unavailable
        }
    }
}

/// Ask the platform for its own file picker.
///
/// There is no interim picker outside Windows, and the module header says why
/// an untested `zenity` arm would be worse than this. `rfd` is the fix, and it
/// is one manifest line away.
#[cfg(not(windows))]
fn native_pick() -> Picked {
    Picked::Unavailable
}

/// Decode an even-length ASCII hex string into bytes.
///
/// `None` for anything that is not one, which is how output that did not come
/// from the script above is refused rather than turned into a plausible path.
/// Written here rather than pulled in because it is ten lines and this crate
/// may not grow a dependency.
#[cfg(windows)]
fn decode_hex(text: &str) -> Option<Vec<u8>> {
    if text.len() % 2 != 0 {
        return None;
    }
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len() / 2);
    for pair in bytes.chunks_exact(2) {
        let hi = char::from(pair[0]).to_digit(16)?;
        let lo = char::from(pair[1]).to_digit(16)?;
        out.push(u8::try_from(hi * 16 + lo).ok()?);
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★ **The diagnostic seam answers the dialog, in all three shapes.**
    ///
    /// This is the whole harness contract, and every row of the table in the
    /// module header is asserted: unset defers to the picker, a value is a
    /// path, and an *empty* value is a cancel — the third being the one a
    /// reader would otherwise assume was an accident.
    #[test]
    fn the_diagnostic_seam_answers_the_dialog() {
        assert_eq!(from_env(None), None, "unset must not answer at all");
        assert_eq!(
            from_env(Some(OsString::from("D:\\drawings\\sheet.pdf"))),
            Some(Picked::Path(PathBuf::from("D:\\drawings\\sheet.pdf")))
        );
        assert_eq!(
            from_env(Some(OsString::new())),
            Some(Picked::Cancelled),
            "an empty value is how a harness drives the cancel path without a dialog"
        );
    }

    /// A path with a space, and one that is not ASCII, both survive the seam.
    ///
    /// `OsString` rather than `String` throughout for the same reason
    /// `main.rs` reads `args_os`: a path is not required to be valid Unicode,
    /// and a non-Unicode path is the operator's business rather than ours to
    /// reject.
    #[test]
    fn the_seam_does_not_mangle_a_real_path() {
        for raw in [
            "C:\\Program Files\\a drawing.pdf",
            "D:\\Zeichnungen\\Übersicht.pdf",
        ] {
            assert_eq!(
                from_env(Some(OsString::from(raw))),
                Some(Picked::Path(PathBuf::from(raw)))
            );
        }
    }

    /// ★ **The hex channel round-trips a non-ASCII path.**
    ///
    /// The reason the interim picker does not simply print the file name: a
    /// redirected PowerShell stdout carries the console code page, so an
    /// umlaut arrives as something else and the resulting path names a
    /// different file — or none — while looking exactly like a real answer.
    #[cfg(windows)]
    #[test]
    fn the_hex_channel_round_trips_a_path_with_an_umlaut() {
        let path = "D:\\Zeichnungen\\Übersicht.pdf";
        let hex: String = path
            .as_bytes()
            .iter()
            .map(|b| format!("{b:02X}"))
            .collect();
        let decoded = decode_hex(&hex).expect("valid hex");
        assert_eq!(String::from_utf8(decoded).expect("utf-8"), path);
    }

    /// …and anything that is not hex is refused rather than guessed at.
    #[cfg(windows)]
    #[test]
    fn output_that_did_not_come_from_the_script_is_refused() {
        assert_eq!(decode_hex("4"), None, "an odd length is not a byte string");
        assert_eq!(decode_hex("zz"), None, "not hex digits");
        assert_eq!(
            decode_hex("C:\\x.pdf"),
            None,
            "a plain path must not decode as though it were hex"
        );
        assert_eq!(decode_hex(""), Some(Vec::new()));
    }
}
