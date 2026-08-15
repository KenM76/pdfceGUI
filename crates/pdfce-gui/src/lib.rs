//! # pdfce-gui — native desktop shell (rebuild), library root
//!
//! **This crate is a library with a thin binary in front of it.** The
//! binary (`src/main.rs`) does one thing: read `argv`, call [`run`]. Every
//! module, every type and every test lives here.
//!
//! ## Why a library at all, when the product is an executable
//!
//! Three reasons, and the third is the one that bites daily.
//!
//! 1. **`tools/ui-verify` and any integration test can `use pdfce_gui::…`.**
//!    Without a library target every assertion has to cross the process
//!    boundary, even when the question is really a unit-level one — "does
//!    this manifest validate?" does not need a window.
//! 2. **`cargo doc` documents something.** A binary crate's rustdoc is
//!    empty, and this project's standing rule is that the documentation is
//!    the logic. Docs that cannot be browsed are docs that rot.
//! 3. **`main.rs` stops being a contention point.** With the module tree in
//!    a binary, *every* new module must edit the same file — which is the
//!    one guaranteed merge conflict when work runs in parallel, and this
//!    project runs work in parallel by design. Moving the tree here does not
//!    remove the shared file, but it removes it from the path of the
//!    `argv`-and-viewport code that has nothing to do with it.
//!
//! Converted at the S2 → S3 boundary, deliberately: it changes visibility
//! across every module, so it wants a moment when nothing else is in
//! flight, and it wants to happen *before* the panel modules multiply.
//! `PROJECT_PLAN.md` §4.2b records the decision.
//!
//! ## Where everything lives
//!
//! | module | responsibility | headlessly testable |
//! |---|---|---|
//! | [`app`] | the one owner of state; frame composition; actions | partly |
//! | [`shell`] | the ribbon/mode/keymap definition, **as data** | **yes** |
//! | [`viewer`] | page index, zoom ladder, fit math, raster ceiling | **yes** |
//! | [`render`] | off-thread rasterization; pixmap → texture | worker keys only |
//! | [`canvas`] | drawing the page, wheel/ctrl-wheel/middle-drag input | geometry only |
//! | [`find`] | the search query, its options, stepping, staleness and the bar | mostly |
//! | [`panels`] | the dock's panel bodies, and the page object model behind them | mostly |
//! | [`text`] | every operator-visible string (the ui-text catalog) | n/a |
//! | [`diag`] | the opt-in `PDFCE_DIAG` trace channel | n/a |
//!
//! The split is driven by testability: a windowed UI cannot run on a CI
//! runner, so every piece of *logic* that could be wrong in a way a human
//! would notice — an off-by-one page step, a fit scale that overflows an
//! axis, a zoom that blows the rasterizer's allocation guard — is pushed
//! into a pure function with a unit test. What is left is wiring. Wiring
//! can be reviewed; arithmetic needs tests.
//!
//! ## Privacy posture, carried across unchanged
//!
//! This crate makes no network calls of any kind. The only file it opens is
//! the one it is asked to open.

#![forbid(unsafe_code)]

pub mod app;
pub mod canvas;
pub mod diag;
// The shell's stationary, screen-anchored surfaces — Print today, Properties
// and the settings host to come. A dialog is one transaction with a start and
// an end; a panel is somewhere you dip in and out of. See DIALOGS' own header
// for that distinction and for why a print does not push an `Action`.
pub mod dialogs;
// Find: the query and its options, the one place a search is run, the rule
// that decides what the position readout says, and the bar itself. See FIND's
// own header for the `find_text` wildcard trap it exists to avoid, for why the
// bar is docked rather than floating, and for what an edit does to a hit list.
pub mod find;
// The icon set: SVG path data, a subset parser, a tiny-skia rasterizer and
// the painter `egui-shell`'s ribbon calls back into. Supplying that painter
// is what stops the ribbon falling back to text labels — see `icons::paint`.
pub mod icons;
// OCR: what image the recogniser is shown, the thread it runs on, and the
// named refusals it can come back with. It authors no PDF — `pdfce-core`'s
// `ocr::layer` writes the invisible mode-3 sandwich and this shell is that
// function's first caller anywhere. See OCR's own header for why recognition
// reads the document as it was OPENED, and for the y-flip it deliberately does
// not perform.
pub mod ocr;
// The dock's panel bodies — Bookmarks, Layers, Signatures, Fonts, Objects and
// the properties panel. See PANELS' own header for the reachability contract
// every one of them has to satisfy.
pub mod panels;
// Redaction: the apply pipeline and its absence proof, salvaged whole from the
// old shell — the ONE place that proof exists anywhere, `pdfce-core` included.
// See REDACT's own header for the two full rewrites, for why the proof is made
// unskippable rather than merely available, and for why a redaction never
// overwrites the file it came from.
pub mod redact;
pub mod render;
// The pdfce shell definition — the seven-tab ribbon, three modes, QAT and
// keymap, expressed as DATA over `egui-shell`'s manifest types rather than
// as rendering code. See SHELL_FRAMEWORK.md; this module is the sole
// consumer of `text::{ribbon, commands}`.
pub mod shell;
pub mod text;
pub mod viewer;

use std::path::PathBuf;

/// The window's opening size, in egui points.
///
/// Large enough that a fit-to-page US Letter sheet is legible without any
/// resizing, which is the first thing an operator does after launching.
const INITIAL_WINDOW_SIZE: [f32; 2] = [1100.0, 800.0];

/// The smallest window the shell will let the operator make.
///
/// Below this the canvas stops being usable rather than merely small;
/// enforcing it in the viewport builder is cheaper than defending every
/// layout against a 200×100 window.
const MIN_WINDOW_SIZE: [f32; 2] = [640.0, 480.0];

/// Start the application, optionally opening a document.
///
/// Everything from here down is the event loop. The caller has already
/// answered anything that must be decided *before* a window exists — a
/// terminal invocation must not open a window it then has to be told to
/// close, which is why argument handling belongs to the binary and not to
/// this function.
///
/// # Errors
///
/// Propagates whatever `eframe::run_native` reports: a windowing system
/// that could not be reached, a graphics backend that failed to initialise.
pub fn run(initial: Option<PathBuf>) -> eframe::Result {
    let mut viewport = egui::ViewportBuilder::default()
        .with_title(text::window_title())
        .with_inner_size(INITIAL_WINDOW_SIZE)
        .with_min_inner_size(MIN_WINDOW_SIZE);

    // Test-harness placement: put the window somewhere explicit and do NOT
    // let it take focus.
    //
    // Carried across from the old shell with its reasoning intact. A GUI
    // defect has one honest oracle — the running application — but driving
    // that on the operator's own desktop takes their focus and covers their
    // work. Given a position off the visible desktop plus `with_active`
    // off, the process runs a genuine event loop that synthesized window
    // messages can drive and [`diag`] can report on, while nothing appears
    // in front of anyone. `tools/ui-verify` is the consumer.
    //
    // Deliberately NOT `with_visible(false)`: a hidden window is not merely
    // an invisible one — it stops being laid out, so the very interactions
    // under test would be skipped and the trace would show a fault that is
    // only an artefact of the harness.
    if let Some(spec) = std::env::var_os("PDFCE_DIAG_VIEWPORT") {
        let nums: Vec<f32> = spec
            .to_string_lossy()
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();
        if let [x, y, w, h] = nums[..] {
            viewport = viewport
                .with_position([x, y])
                .with_inner_size([w, h])
                .with_active(false);
        }
    }

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    // Unconditional first line when tracing. Without it an empty trace has
    // two very different meanings — "the process never saw PDFCE_DIAG" and
    // "the process saw nothing worth reporting" — and a harness cannot tell
    // them apart. That ambiguity cost the old shell's investigation a round
    // trip on 2026-08-04.
    diag::trace(|| format!("start argv1={initial:?}"));

    eframe::run_native(
        "pdfce",
        native_options,
        Box::new(move |cc| {
            app::configure_context(&cc.egui_ctx);
            let mut app = app::PdfceApp::new();
            if let Some(path) = initial {
                app.open_path(path);
            }
            Ok(Box::new(app))
        }),
    )
}
