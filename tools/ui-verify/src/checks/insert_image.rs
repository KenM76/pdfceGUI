//! `insert_image_places_a_picture` — a picture reaches the page, and the
//! resolution the window promised is the one the document reports.
//!
//! # The gap this closes
//!
//! `edit.insert_image` was `★ P3` scaffolded with the recorded reason **"No
//! recorded reason for the missing arm"** — one of three such entries — while
//! `EditSession::add_image` had shipped the whole time.
//!
//! # ★ The assertion this check exists for, and it is the LAST one
//!
//! **The resolution the window previewed and the resolution the document
//! reported are the same number.**
//!
//! That equality is not decoration. `pdfce-core` deleted its own copy of
//! `pixels / (points / 72)` on 2026-08-19 specifically so there would be one
//! derivation left, after this shell asked for the pure preview and said it
//! would rather have nothing than two implementations. The engine holds up its
//! half with a test; **the shell can only hold up its half by making the
//! equality observable**, which is what the `dpi=` field on
//! `insert-image-requested` is for.
//!
//! The failure it catches is specific and quiet. A re-derivation in the window
//! — the obvious four-liner — measures the *requested* rectangle rather than
//! the *placed* one, so under `ImageFit::Contain`, which is the default, it is
//! low by exactly the letterbox ratio. Both numbers look perfectly reasonable.
//! An operator sizing a logo would be told it plots at 300 dpi and get 200.
//!
//! # Why the fixture is written by the check
//!
//! A committed binary would be an asset to keep in step with a decoder, and
//! `ui_verify::png` already encodes one for the screenshot path — so the check
//! writes its own two-colour PNG into the scratch directory and points
//! `PDFCE_DIAG_IMAGE_PATH` at it. Hermetic, deterministic, and it exercises the
//! importer on bytes nothing else in this repository produced.
//!
//! The picture is deliberately **wide and short**. A square would let a
//! letterbox bug pass: `Contain` on a square picture in a square box is the
//! identity, and the whole point of the last assertion is the case where the
//! placed rectangle differs from the requested one.

use crate::checks::driving::{
    SHELL_DIAG_ENV, declared, declared_names, declared_or_in_overflow, list,
};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The mode the command is offered in. `edit.insert_image` is gated on
/// `edit_content`, which Read and Review both refuse.
const MODE: &str = "edit";
/// The window's own region.
const WINDOW: &str = "dialog:insert-image";
/// The Insert button.
const INSERT: &str = "insert-image.insert";
/// The width spinner — the one numeric control a harness can move, named
/// because the fit radios and the picture facts publish nothing aimable.
const WIDTH: &str = "insert-image.width";
/// The trace the importer emits when the file was read.
const IMPORTED: &str = "image-imported";
/// The trace the window emits on commit, carrying the PREVIEWED resolution.
const REQUESTED: &str = "insert-image-requested";
/// The label `vector_edit` traces when `add_image` succeeded.
const APPLIED: &str = "add-image";
/// The environment seam that answers the image picker.
const IMAGE_PATH_ENV: &str = "PDFCE_DIAG_IMAGE_PATH"; // ui-text-exempt: an environment variable name

/// The fixture's pixel size.
///
/// ★ **Wide and short, deliberately.** `Contain` on a square picture in a
/// square box is the identity, so a square fixture would let a letterbox defect
/// pass the last assertion — which is the assertion this check exists for.
const FIXTURE_W: u32 = 64;
/// See [`FIXTURE_W`].
const FIXTURE_H: u32 = 16;

/// See the module documentation.
pub struct InsertImagePlacesAPicture;

impl Check for InsertImagePlacesAPicture {
    fn name(&self) -> &'static str {
        "insert_image_places_a_picture"
    }

    fn defect(&self) -> &'static str {
        "Edit > Insert image is drawn and does nothing — or it opens a window whose Insert \
         button reaches no document, or promises a resolution the placement does not deliver, \
         which under the default fit is wrong by exactly the letterbox ratio and looks \
         perfectly reasonable in both places"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(skip) => report.skip(skip.to_string()),
        }
    }
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no fixture document. Pass --pdf."))?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a mode segment, a ribbon tab, \
             a ribbon control and a button. Reported as SKIPPED rather than passed: a check \
             that did not run has learned nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot say \
             where its controls are.",
            ctx.profile.name
        ))
    })?;

    // --- 0: the fixture ------------------------------------------------------
    //
    // ★★ **A PNG this harness encodes, or a file named by `PDFCE_UIV_IMAGE`.**
    //
    // The env seam was added 2026-08-19, on the operator's report that *"the
    // insert image button doesn't insert it either"* — for a **jpg** — while
    // this check, which drives exactly that button, was passing.
    //
    // It was passing on a PNG the harness authors itself, and that is the same
    // fixture trap that hid the text-editing defect for three weeks: *a check
    // that drives a file the test tool authored tests the shape the test tool
    // imagines.* `image_import` accepts PNG, JPEG, BMP and TIFF down four
    // different decoders, and exactly one of them was ever exercised here.
    //
    // An environment variable rather than a CLI flag, because every other input
    // this suite takes (`--pdf`, `--doc-point`) names the thing under test and
    // this one names what the harness feeds it.
    let supplied = std::env::var_os("PDFCE_UIV_IMAGE").map(std::path::PathBuf::from);
    if let Some(path) = supplied.as_ref() {
        if !path.exists() {
            return Err(Error::new(format!(
                "PDFCE_UIV_IMAGE names {} and there is no file there.",
                path.display()
            )));
        }
        report.note(format!(
            "using the image at {} ({} bytes) instead of the built-in PNG",
            path.display(),
            std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
        ));
    }
    let fixture = supplied
        .clone()
        .unwrap_or_else(|| ctx.out("insert_image_fixture.png"));
    let pixels = fixture_pixels();
    let png = crate::png::encode_rgb(FIXTURE_W, FIXTURE_H, &pixels).ok_or_else(|| {
        Error::new(
            "the harness's own PNG encoder refused a fixture it was handed the right number \
             of bytes for. Nothing about the application has been tested; this is the check's \
             own precondition.",
        )
    })?;
    if supplied.is_none() {
        std::fs::write(&fixture, &png).map_err(|e| {
            Error::new(format!(
                "cannot write the fixture image to {}: {e}",
                fixture.display()
            ))
        })?;
        report.note(format!(
            "wrote a {FIXTURE_W}×{FIXTURE_H} fixture PNG, {} bytes",
            png.len()
        ));
    }

    let mut spec = LaunchSpec::new(&exe, ctx.out("insert_image.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((IMAGE_PATH_ENV.to_owned(), fixture.display().to_string()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(fixture.clone());
    session.settle(40);
    let driver = Driver::new(session.window());

    crate::checks::driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;

    // ★★ **The operator's own sequence, when asked for**: a NEW document from
    // the template first, then the insert.
    //
    // 2026-08-20, verbatim: *"Make a new document from a template … and insert
    // the image."* He reports the insert doing nothing and has done since it
    // shipped, while this check — which inserts into an **opened** PDF — has
    // passed throughout, including against his own file.
    //
    // A created document is not an opened one: `Origin::Created`, no file
    // behind it, `stored_under()` empty. If the difference lives there, this is
    // the flag that finds it, and it costs one chord.
    if std::env::var_os("PDFCE_UIV_NEW_DOCUMENT").is_some() {
        driver.press_chord(&[crate::input::Key::Ctrl.vk()], 0x4E)?;
        session.settle(20);
        let made = session.trace()?;
        let Some(line) = made.last("new-document") else {
            return Err(Error::new(
                "Ctrl+N traced no `new-document` line, so no blank document was made and the\
                 rest of this run would be about the wrong document.",
            ));
        };
        report.note(format!("made a blank document first: `{}`", line.raw));
    }

    // --- 1: the Edit tab ---------------------------------------------------
    let trace = session.trace()?;
    let tab = declared(&trace, ui_rect, "ribbon.tab.edit").ok_or_else(|| {
        Error::new(format!(
            "no `ribbon.tab.edit` region in {MODE}. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(14);

    // --- 2: pick, import, and open -----------------------------------------
    let Some(item) =
        declared_or_in_overflow(&session, &driver, ui_rect, "ribbon.item.edit.insert_image")?
    else {
        // ★ A screenshot at the moment of failure, because this message has
        // been wrong before. `D:/dev/rag/egui/` records the rule: a layout or
        // reachability defect has exactly one oracle and it is a rendered
        // screenshot. A trace can say a region was declared and say nothing at
        // all about whether a click could reach it.
        let shot = ctx.out("insert_image.no-edit-items.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        return Ok(Some(format!(
            "the Edit tab declares no `ribbon.item.edit.insert_image`, on the band or in the \
             overflow. Items declared: {}. A screenshot of the moment is attached.",
            list(&declared_names(
                &session.trace()?,
                ui_rect,
                "ribbon.item.edit."
            ))
        )));
    };
    driver.click_at(session.frame()?.declared_center(item))?;
    session.settle(24);

    let trace = session.trace()?;
    if trace.last(IMPORTED).is_none() {
        let failed = trace.last("image-import-failed");
        let unimplemented = trace
            .events("command-unimplemented")
            .any(|l| l.get("id") == Some("edit.insert_image"));
        return Ok(Some(if unimplemented {
            "`edit.insert_image` was clicked and traced `command-unimplemented` — it is still \
             scaffolded."
                .to_owned()
        } else if let Some(line) = failed {
            format!(
                "the fixture PNG was REFUSED by the importer: `{}`. The harness wrote it with \
                 `ui_verify::png`, so this is either a real importer defect or the encoder \
                 and the decoder disagree — and the second is worth knowing too.",
                line.raw
            )
        } else {
            format!(
                "`edit.insert_image` was clicked and traced no `{IMPORTED}` line. The picker's \
                 `{IMAGE_PATH_ENV}` seam was set, so either the arm never ran or it never \
                 reached the picker."
            )
        }));
    }
    if declared(&trace, ui_rect, WINDOW).is_none() {
        return Ok(Some(format!(
            "the picture imported and no `{WINDOW}` region appeared, so the file was read and \
             the operator was shown nothing. The import is the half that can fail; the window \
             is the half that makes it a feature."
        )));
    }
    if declared(&trace, ui_rect, WIDTH).is_none() {
        return Ok(Some(format!(
            "the window drew and declares no `{WIDTH}` region, so the box has no size control \
             — which is the whole of how a picture is placed here."
        )));
    }
    report.note("the fixture imported and the placement window opened");

    // --- 3: insert ---------------------------------------------------------
    let Some(button) = declared(&trace, ui_rect, INSERT) else {
        return Ok(Some(format!(
            "the window declares no `{INSERT}` region. It is drawn only while the box CAN be \
             placed, so its absence means the seeded default was refused — a window that \
             opens declining its own default."
        )));
    };
    driver.click_at(session.frame()?.declared_center(button))?;
    session.settle(26);

    let trace = session.trace()?;
    let Some(requested) = trace.last(REQUESTED) else {
        return Ok(Some(format!(
            "Insert was pressed and no `{REQUESTED}` line was traced, so the button is drawn \
             and inert."
        )));
    };
    if let Some(refusal) = trace
        .events(&format!("{APPLIED}-refused"))
        .filter_map(|l| l.get("detail").map(str::to_owned))
        .last()
    {
        return Ok(Some(format!(
            "the window raised its action and the engine REFUSED it: {refusal}. The shell half \
             works; this is a `pdfce-core` verdict and belongs in a request."
        )));
    }
    let Some(applied) = trace.last(APPLIED) else {
        return Ok(Some(format!(
            "the window raised `{REQUESTED}` and no `{APPLIED}` line followed, so the action \
             was raised and its apply arm never ran, or could not borrow the session. Nothing \
             reached the document."
        )));
    };
    report.note(format!("placed: `{}`", applied.raw));

    // --- 4: ★ the disclosures reached the operator -------------------------
    //
    // `add_image`'s outcome always carries at least the resolution sentence, so
    // an EMPTY disclosure list means the apply arm called the engine and threw
    // its report away — which leaves every fact an operator cannot see at
    // editing zoom unsaid, and which nothing else here would notice.
    let disclosures = applied.get("disclosures").unwrap_or("none");
    if disclosures == "none" || disclosures.is_empty() {
        return Ok(Some(format!(
            "the image was placed and reported NO disclosures. `add_image` always returns at \
             least the resolution, so this is the apply arm discarding the report — and every \
             fact the operator cannot see on screen goes with it. Line: `{}`.",
            applied.raw
        )));
    }

    // --- 5: ★★ the promise and the result are the same number --------------
    let Some(previewed) = requested.get("dpi").and_then(|v| v.parse::<f64>().ok()) else {
        return Err(Error::new(format!(
            "the `{REQUESTED}` line carries no readable `dpi=`, so the promise cannot be \
             compared with the result: `{}`",
            requested.raw
        )));
    };
    // The disclosure states the figure as "… is N dpi", which is the operator's
    // own sentence rather than a field — so it is read out of the words. That
    // is deliberate: asserting on the SENTENCE proves the number the operator
    // actually sees, not one carried alongside it for the harness.
    let Some(reported) = dpi_in(disclosures) else {
        return Ok(Some(format!(
            "the placement disclosed `{disclosures}`, which states no resolution. The number \
             is the one fact `pdfce-core` calls *\"not a warning — a number\"*, and an \
             operator who cannot see it at editing zoom has no other source for it."
        )));
    };
    if (previewed - reported).abs() > 1.0 {
        return Ok(Some(format!(
            "★ the window promised {previewed:.0} dpi and the document reports {reported:.0}. \
             They come from one producer by design — `pdfce-core` deleted its own copy of the \
             formula so there would be one derivation left — so a difference means this shell \
             has re-derived it. Under `Contain`, which is the default, a re-derivation \
             measures the REQUESTED rectangle instead of the placed one and is low by exactly \
             the letterbox ratio. Both numbers look reasonable."
        )));
    }
    report.note(format!(
        "the window promised {previewed:.0} dpi and the document reported {reported:.0} — one \
         producer, two readings, same answer"
    ));
    Ok(None)
}

/// The fixture's pixels: two vertical bands, so a placement that lands
/// transposed or mirrored is visible in an artifact a human opens.
///
/// The colours are irrelevant to every assertion — nothing here reads a pixel
/// back — and they are two rather than one so the saved artifact is legible as
/// a picture rather than as a swatch.
fn fixture_pixels() -> Vec<u8> {
    let mut out = Vec::with_capacity((FIXTURE_W * FIXTURE_H * 3) as usize);
    for _ in 0..FIXTURE_H {
        for x in 0..FIXTURE_W {
            if x < FIXTURE_W / 2 {
                out.extend_from_slice(&[0x20, 0x60, 0xC0]);
            } else {
                out.extend_from_slice(&[0xF0, 0xC0, 0x20]);
            }
        }
    }
    out
}

/// Read a resolution out of a disclosure sentence.
///
/// The sentence is *"At this size the picture is 300 dpi."* — so this looks for
/// the token before `dpi` and parses it. Reading the operator's own words
/// rather than a field carried beside them is the point: it proves the number
/// **they see**, and a build that traced one figure and displayed another would
/// pass a field comparison.
fn dpi_in(text: &str) -> Option<f64> {
    let idx = text.find("dpi")?;
    text[..idx]
        .split_whitespace()
        .next_back()
        .and_then(|token| token.parse::<f64>().ok())
}
