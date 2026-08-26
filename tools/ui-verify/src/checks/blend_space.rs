//! `blend_space` — **a page whose colours change with zoom says so.**
//!
//! The driven assertion for the operator's report of 2026-08-26:
//!
//! > *"seems I get different results depending on Zoom level. The [shading]
//! > boxes … on zoom out the colors between our rendering and the
//! > references don't match, but they do when I am zoomed in. up to 474% they
//! > are mismatched, but at 579% they match."*
//!
//! # ★★★ What is actually happening, measured before this check was written
//!
//! `pdfce-render` composites a page containing transparency in a **subtractive
//! CMYK buffer**, which is the correct space for it. That buffer has a
//! documented ceiling — `MAX_CMYK_BUFFER_BYTES`, 256 MiB at 20 B/px, i.e.
//! **13,421,772 pixels**. Past it the renderer falls back to compositing in
//! sRGB and counts that it did (`cmyk_buffer_refused`).
//!
//! On an A4 page that ceiling is crossed at **zoom 534 %** — dead centre of the
//! band the operator bracketed. Bisected with the CLI to the pixel: buffer used
//! at scale 5.33 (13,394,232 px), refused at 5.34 (13,444,992 px).
//!
//! Crossing it changes the rendered colour. Measured on the conformance suite's
//! composite page by
//! box-averaging **every pixel** of both renders into a common grid — so that
//! resampling could not masquerade as the effect — the transparency patches
//! move by up to **16 levels out of 255**.
//!
//! ★★ Two earlier measurements got this wrong in opposite directions and are
//! recorded because both are tempting. Sampling a sparse lattice reported
//! 358 of 576 cells differing, all of it text sampled at two pixel sizes.
//! Excluding every cell that was not flat removed that noise correctly — and
//! removed **the gradients**, which are the thing the operator is looking at.
//! Only a full box average is stable for flat patches, gradients and text
//! alike.
//!
//! # What this check asserts, and what it deliberately does not
//!
//! It asserts that when the fallback engages, **the operator is told** — the
//! `status-group:blend-space` disclosure appears — and that when it has not
//! engaged, the line is **absent**. Both halves, because a disclosure that is
//! always on says nothing.
//!
//! It does **not** assert that the colours are right on either side of the
//! ceiling. That is the engine's question and it is filed as one. What this
//! shell owes today is rule 4's surviving half: an inference the operator
//! cannot see — a screenshot says nothing about which space a page was
//! composited in — owes an off-canvas report.
//!
//! # Phases
//!
//! | Phase | Does | Expected |
//! |---|---|---|
//! | A | open a page with transparency at fit zoom | **no** blend-space line |
//! | B | Ctrl+wheel up until the raster passes 13.4 Mpx | the line appears |

use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk::CONTROL as VK_CONTROL;
use crate::trace::Trace;

/// The canvas viewport's published region name.
const CANVAS_REGION: &str = "canvas-viewport";

/// The published region of the status-bar disclosure.
const REGION: &str = "status-group:blend-space";
/// The canvas trace line, which carries the live zoom.
const ZOOM_KEY: &str = "zoom";
/// Wheel notches per batch, and how many batches before giving up.
const BATCH: usize = 4;
/// Enough batches to climb from fit (~85 %) past 534 % and well beyond.
const MAX_BATCHES: usize = 40;
/// How many settle rounds to give the render worker to reach the climbed zoom.
///
/// Generous: a whole-page raster past the ceiling is 30 M pixels and took
/// 678 ms in the run that discovered this wait was needed at all.
const RASTER_WAIT_TICKS: usize = 60;
/// The engine's ceiling, in pixels: `MAX_CMYK_BUFFER_BYTES` / 20 B per px.
///
/// ★ Duplicated from `pdfce-render`, where it is `pub(crate)` and therefore
/// unreadable from here. **That is the finding, not an accident**: this shell
/// cannot choose a raster that respects a ceiling it cannot see, which is why
/// `render::strategy` keeps asking for whole pages four times past it. Filed as
/// an engine request. The number is used only to decide how far this check
/// should zoom, never to make an assertion, so a change in the engine makes the
/// check zoom the wrong distance rather than report a false verdict.
const CEILING_PX: f64 = (256 * 1024 * 1024 / 20) as f64;

/// A page whose colours change with zoom must say so.
pub struct BlendSpaceFallbackIsDisclosed;

impl Check for BlendSpaceFallbackIsDisclosed {
    fn name(&self) -> &'static str {
        "blend_space"
    }

    fn defect(&self) -> &'static str {
        "the same page renders different colours at different zooms, with nothing on screen \
         saying so — the operator compares a patch against a reference, sees it disagree, and \
         has no way to learn that zooming out would make it agree"
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

/// The live zoom, as the canvas published it.
fn zoom_now(trace: &Trace, canvas_event: &str) -> Option<f64> {
    trace
        .last(canvas_event)
        .and_then(|l| l.get(ZOOM_KEY))
        .and_then(|v| v.parse().ok())
}

/// The scale of the most recent **completed** raster.
///
/// ★★ `raster-blend-space`, which the worker emits when a render FINISHES —
/// not `render-spawn`, which it emits when one starts. Both were tried and the
/// difference cost a run: a whole-page raster past the ceiling is 30 M pixels
/// and takes seconds, so the check saw `render-spawn scale=8.01`, believed the
/// question had been asked, and asserted against a texture still showing the
/// scale-3.6 render. The zoom, the spawn and the finished raster are three
/// different clocks and only the third one drives the status bar.
fn last_raster_scale(trace: &Trace) -> f64 {
    trace
        .last("raster-blend-space")
        .and_then(|l| l.get("scale"))
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.0)
}

/// Whether the disclosure is on screen **now**.
///
/// ★★ `ui_rect` is a **change log**, not a per-frame census — this project's
/// own RAG records that a widget which stops being drawn publishes nothing, so
/// "the region has ever appeared" and "the region is on screen" are different
/// questions. Phase A therefore checks that the line has **never** appeared,
/// which a change log can answer honestly, rather than that it is absent now,
/// which it cannot.
fn ever_declared(trace: &Trace, ui_rect: &str) -> bool {
    trace.events(ui_rect).any(|l| l.get("name") == Some(REGION))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new(
            "no --pdf. Pass a document that CONTAINS TRANSPARENCY — the CMYK compositing buffer \
             is only allocated for a page that needs it, so on an ordinary drawing this check \
             would zoom for a minute and correctly observe nothing.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check is a long Ctrl+wheel climb against the \
             real window. Reported as SKIPPED rather than passed.",
        ));
    }
    let vocab = &ctx.profile.vocab;
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;
    let page = match ctx.page_size {
        Some((w, h)) => (w, h),
        None => {
            let g = crate::fixture::page_geometry(&pdf).ok_or_else(|| {
                Error::new(format!(
                    "cannot read a page size from {}. Pass --page-size WxH — this check needs it \
                     to know how far to zoom.",
                    pdf.display()
                ))
            })?;
            (g.width_pt, g.height_pt)
        }
    };
    // The zoom at which the whole-page raster passes the ceiling. Reported so a
    // failure can be read without re-deriving it.
    let crossing = (CEILING_PX / (page.0 * page.1)).sqrt();
    report.note(format!(
        "page {:.0}x{:.0} pt; the CMYK buffer ceiling is crossed at zoom {:.0}%",
        page.0,
        page.1,
        crossing * 100.0
    ));

    let mut spec = LaunchSpec::new(&exe, ctx.out("blend_space.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);

    let trace = session.trace()?;
    if !trace.started(vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so the diagnostic switch did not reach the process.",
            vocab.start_event
        )));
    }

    // --- A: at fit zoom the page is small and the line must be absent ------
    let fit = zoom_now(&trace, vocab.canvas_event).unwrap_or(0.0);
    report.note(format!("opened at zoom {:.0}%", fit * 100.0));
    if fit >= crossing {
        return Err(Error::new(format!(
            "the document opened at zoom {:.0}%, already past the {:.0}% crossing, so phase A \
             has nothing to observe. Pass a smaller page or a larger window.",
            fit * 100.0,
            crossing * 100.0
        )));
    }
    if ever_declared(&trace, ui_rect) {
        return Ok(Some(format!(
            "the `{REGION}` disclosure is showing at zoom {:.0}%, below the {:.0}% at which the \
             compositing buffer is refused. A disclosure that is always on says nothing, and \
             this one would tell the operator to zoom out when they already are.",
            fit * 100.0,
            crossing * 100.0
        )));
    }
    report.note("no blend-space disclosure at fit zoom, which is correct");

    // --- B: climb past the ceiling ----------------------------------------
    let frame = session.frame()?;
    let driver = Driver::new(session.window());
    // ★ Aimed at the canvas centre ONCE and left there: Ctrl+wheel is
    // zoom-about-the-pointer, so the point under the cursor stays under it for
    // the whole climb and no re-aiming is needed.
    let canvas = crate::checks::driving::declared(&trace, ui_rect, CANVAS_REGION)
        .ok_or_else(|| Error::new(format!("no `{CANVAS_REGION}`; is a document open?")))?;
    let at = frame.declared_center(canvas);

    let target = crossing * 1.15;
    let mut batches = 0;
    loop {
        let now = zoom_now(&session.trace()?, vocab.canvas_event).unwrap_or(0.0);
        if now >= target || batches >= MAX_BATCHES {
            break;
        }
        driver.scroll_at_held(at, &[VK_CONTROL], 1, BATCH)?;
        session.settle(5);
        batches += 1;
    }
    // ★★★ **Wait for the RASTER to catch up with the zoom.**
    //
    // The first run of this check failed and the product was innocent: it
    // climbed to 801 %, asserted, and the trace showed the last raster had been
    // spawned at scale 3.60 — below the 5.34 crossing, so the fallback had
    // never engaged and the absent disclosure was correct.
    //
    // Zoom and raster are not the same clock. A wheel notch changes the zoom on
    // the frame it arrives; the raster is produced by a worker, is debounced so
    // that a climb does not spawn a render per notch, and at these sizes takes
    // most of a second. Asserting on the disclosure before the raster exists is
    // measuring the wrong surface, and a failure there looks exactly like a
    // broken feature.
    //
    // So the check waits for the application to say it has rasterised at a
    // scale past the ceiling, and only then reads the disclosure. Bounded, so a
    // build that never gets there SKIPS with the reason rather than hanging.
    let mut waited = 0;
    while waited < RASTER_WAIT_TICKS {
        let t = session.trace()?;
        if last_raster_scale(&t) >= crossing {
            break;
        }
        session.settle(10);
        waited += 1;
    }
    let trace = session.trace()?;
    let rastered = last_raster_scale(&trace);
    report.note(format!(
        "the last raster was spawned at scale {rastered:.2}; the ceiling is at {crossing:.2}"
    ));
    if rastered < crossing {
        return Err(Error::new(format!(
            "the zoom reached the crossing but the RASTER did not: the last render was \
             spawned at scale {rastered:.2}, below the {crossing:.2} at which the \
             compositing buffer is refused. Nothing past this point would mean \
             anything, so it is a SKIP rather than a failure."
        )));
    }
    // One more settle so the finished raster reaches the texture the status bar
    // reads, rather than only having been spawned.
    session.settle(20);
    let trace = session.trace()?;
    let reached = zoom_now(&trace, vocab.canvas_event).unwrap_or(0.0);
    report.note(format!(
        "climbed to zoom {:.0}% in {batches} batch(es); the crossing is at {:.0}%",
        reached * 100.0,
        crossing * 100.0
    ));
    if reached < crossing {
        return Err(Error::new(format!(
            "only reached {:.0}%, short of the {:.0}% crossing, so the fallback was never \
             engaged and there is nothing to assert. The wheel may not be reaching the canvas.",
            reached * 100.0,
            crossing * 100.0
        )));
    }

    // ★★★ THE PRECONDITION THAT IS ABOUT THE FIXTURE, NOT THE BUILD — and it has
    // to be asked BEFORE the verdict below, because its absence is
    // symptom-identical to the defect.
    //
    // Everything above this line is arithmetic on the page's dimensions: at
    // scale `crossing` a whole-page raster exceeds the CMYK buffer's pixel
    // ceiling. That says the buffer *would* be refused — **if the page asked
    // for one at all.** A page with no transparency on it never engages the
    // buffer, so it is never refused, so `blends_in_wrong_space` is zero, so
    // there is correctly nothing to disclose and the application is right to
    // say nothing.
    //
    // ★ This is not hypothetical. On 2026-08-26 the full suite was run against
    // `SW41177.pdf` — a SOLIDWORKS drawing set, which is the operator's own
    // document and the harness's usual `--pdf` — and this check reported FAIL
    // with a report that read as *"the page's colours have changed and nothing
    // on screen says so"*. Every `raster-blend-space` line in that run said
    // `cmyk_buffer=false refused=0 wrong_space=0`, at every scale up to 3.26.
    // Nothing had changed and nothing was owed. Re-run against
    // the industry print-conformance suite's composite page, the file this check was written
    // for, it passed with the disclosure appearing exactly at the crossing.
    //
    // **A CAD drawing is line work.** The fixture the operator's suite runs on
    // is the one least likely to use transparency, so left unguarded this check
    // is a standing false red on every routine run — and a standing false red
    // is worse than no check, because it trains a reader to skip the section.
    // `crate::report`'s three-state model exists for exactly this: PRECONDITION
    // ABSENT is a SKIP, and it names what was missing.
    if !buffer_was_refused(&trace) {
        return Err(Error::new(format!(
            "the zoom passed the {:.0}% crossing and the renderer never engaged the CMYK \
             compositing buffer at all — every `{BLEND_EVENT}` line reports `refused=0`. That \
             is a fact about the FIXTURE, not the build: a page with no transparency on it \
             composites nothing, so nothing falls back to sRGB and there is correctly nothing \
             to disclose. Line-work drawings are the common case. SKIPPED rather than failed, \
             because a page that cannot change colour cannot demonstrate that the shell would \
             say so. Point --pdf at a document that uses transparency — this check was written \
             against the industry print-conformance suite's composite page, which needs `--page-size 596x791` \
             since its page box cannot be read from the file. Trace: {}.",
            crossing * 100.0,
            session.trace_path().display()
        )));
    }

    if !ever_declared(&trace, ui_rect) {
        return Ok(Some(format!(
            "at zoom {:.0}%, past the {:.0}% at which the renderer refuses the CMYK compositing \
             buffer, no `{REGION}` line was declared — and the renderer DID refuse it, so the \
             page's colours have changed and nothing on screen says so, which is the whole of \
             what was reported. Trace: {}.",
            reached * 100.0,
            crossing * 100.0,
            session.trace_path().display()
        )));
    }
    report.note("the disclosure appeared once the page's colours became approximate");
    Ok(None)
}

/// `raster-blend-space cmyk_buffer=… refused=… wrong_space=… scale=…` — the
/// renderer's own report of which space it composited in.
const BLEND_EVENT: &str = "raster-blend-space";

/// **Whether the renderer ever actually refused the CMYK buffer.**
///
/// The precondition the verdict rests on — see its call site for the run that
/// made it necessary. `refused` counts the times a composite fell back to sRGB
/// because the buffer would have exceeded its ceiling; a page that never asks
/// for the buffer reports `refused=0` at every scale, and so does a page whose
/// zoom never crossed the ceiling.
///
/// The second of those is already excluded above by the `rastered < crossing`
/// guard, so reaching here with `refused=0` everywhere means the first: **the
/// page has no transparency on it.**
fn buffer_was_refused(trace: &crate::trace::Trace) -> bool {
    trace
        .events(BLEND_EVENT)
        .any(|l| l.get("refused").is_some_and(|n| n != "0"))
}
