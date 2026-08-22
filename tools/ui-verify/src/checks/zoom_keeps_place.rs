//! `zooming_does_not_throw_away_where_the_operator_panned` — two reports, one
//! question: *does a zoom keep the view where I put it?*
//!
//! # The reports
//!
//! `OPERATOR_REQUESTS.md` O24e and O24f, 2026-08-22:
//!
//! > *"if I am zoomed out to about page size, pan the cells to the center of
//! > the screen, then start to zoom, the page snaps back to near the center
//! > position."*
//!
//! > *"I do lose the view at 2000000% magnification."*
//!
//! The same failure at two scales, and the same shape at both: a zoom
//! **discards the position** instead of magnifying about it.
//!
//! | where | cause |
//! |---|---|
//! | at fit-page zoom | `geometry::zoom_anchor_offset` clamped to `display - viewport`, which is **zero or negative** when the page is no larger than the viewport — so the offset was forced to 0, the centred position |
//! | at ~2,100,000 % | the `f64` tier's anchor was seeded from the **previous frame's** scroll offset, and then never moved on a zoom at all |
//!
//! # ★★ Why this is not the same check as O24c's
//!
//! `panning_at_deep_zoom_stays_where_it_was_put` asks whether a **pan** moves
//! the view and whether the pixels land in the right place. This asks whether a
//! **zoom** preserves it. They failed independently and were fixed
//! independently; one check covering both would have gone red for one reason
//! while the other was still broken, and the second would have been found later
//! and blamed on the first fix.
//!
//! The measurement is the page point under the **viewport centre**, before and
//! after each zoom. Zoom-to-cursor holds the point under the *pointer*, and
//! this check puts the pointer at the centre, so a correct build keeps that
//! page point fixed however deep it goes.
//!
//! ★ Measured in page units rather than screen pixels, deliberately. Screen
//! pixels are what the defect happens in, but page units are what "the same
//! place on the drawing" means — and the tolerance has to shrink with the zoom,
//! or a check at a million percent would quietly accept a metre of drift.

use crate::checks::driving;
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};

/// `VK_CONTROL`, held while the wheel rolls to make it a zoom.
const VK_CONTROL: u16 = 0x11;

/// The canvas viewport, whose centre the pointer sits at.
const CANVAS_REGION: &str = "canvas-viewport";

/// The canvas's own state line.
const CANVAS_EVENT: &str = "canvas";

/// The `f64` position line, which carries the tier.
const POS_EVENT: &str = "canvas-pos";

/// Where to roll the wheel to knock the view off its centred position.
///
/// ★ The pan is what makes this check able to fail. The centred position is
/// exactly where the O24e defect snapped **to**, so a check that zoomed without
/// panning first would have watched the view "stay" where the bug was about to
/// put it anyway — green, and measuring nothing.
const PAN_AT: (f32, f32) = (0.30, 0.30);

/// How many Ctrl+wheel notches per stage.
const STAGE: usize = 8;

/// How many stages to climb.
///
/// ★ Sized to cross **both** boundaries this check cares about: the region
/// raster tier at about 2,070 % and the `f64` position tier at about
/// 2,118,000 % on a Letter sheet. A run that stopped between them would test
/// one hand-over and silently skip the other.
///
/// Measured, not guessed. Eight notches multiply the zoom by roughly five, so
/// the stages walk 76 % → 377 → 1,867 → 9,247 → 45,798 → ~227,000 →
/// ~1,130,000 → ~5,600,000. Four stages stopped at 45,798 % and the tier guard
/// below correctly refused to call that a pass; seven clear the second
/// boundary with a stage in hand.
const STAGES: usize = 7;

/// How far the anchored page point may drift **per wheel notch**, as a
/// fraction of the page width currently visible.
///
/// # ★★ Per notch, not per stage, and the difference is not pedantry
///
/// The first version read the position once per stage of eight notches and
/// judged it against a one-notch tolerance. It failed by 3.17 pt against
/// 2.57 pt — a real number and a meaningless one, because eight anchored zoom
/// steps each carry their own `f32` rounding and the accumulated slop was being
/// measured against the budget for one.
///
/// The tempting fix is to multiply the tolerance by the notch count. That is
/// loosening a threshold to fit an observation, which is exactly how a check
/// stops being able to see the defect it was written for. Reading after
/// **every** notch keeps the tolerance tight and localises a failure to the
/// notch that caused it.
///
/// ★ A fraction rather than an absolute, because the tolerance must shrink with
/// the zoom. Two percent of what is on screen is generous against a defect that
/// discards the position outright — O24e moved the view by the whole pan, and
/// O24f by the whole zoom ratio.
const DRIFT_FRACTION: f64 = 0.02;

/// See the module documentation.
pub struct ZoomingDoesNotThrowAwayWhereTheOperatorPanned;

impl Check for ZoomingDoesNotThrowAwayWhereTheOperatorPanned {
    fn name(&self) -> &'static str {
        "zooming_does_not_throw_away_where_the_operator_panned"
    }

    fn defect(&self) -> &'static str {
        "zooming after panning snaps the view back to the centre of the page, or loses it \
         entirely past about two million percent — the zoom discards the position instead of \
         magnifying about it"
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

/// One reading: the page point under the viewport centre, the zoom, the span.
#[derive(Debug, Clone, Copy)]
struct Held {
    /// The page point under the viewport centre, in PDF user-space points.
    page: (f64, f64),
    /// Logical points per user-space unit.
    zoom: f64,
    /// How wide the viewport is in page units — the scale the drift tolerance
    /// is taken against.
    span: f64,
}

/// Read the page point currently under the centre of the canvas.
///
/// # ★★ Derived from the page's own rect, not from the scroll offset
///
/// The scroll offset is meaningless above the deep threshold — that is the
/// entire reason the deep tier exists — so a check reading it would be
/// measuring nothing on exactly the runs O24f is about. The page's drawn rect
/// and the zoom are published on every frame at every tier, and
/// `(centre - rect.min) / zoom` is the page point under the centre in both.
///
/// ★ It is `f32`-derived and therefore imprecise at the top of the range. That
/// is acceptable *here* and would not be elsewhere: this compares a point
/// against itself across one zoom step, with a tolerance proportional to what
/// is on screen, so an error of a few representable steps sits far below the
/// threshold. A check measuring absolute position would have to read the `f64`
/// line instead.
fn held(session: &Session, canvas: crate::geom::LRect) -> Result<Option<Held>> {
    let trace = session.trace()?;
    let Some(line) = trace.events(CANVAS_EVENT).last() else {
        return Ok(None);
    };
    let (Some(rect), Some(zoom)) = (line.get_rect("rect"), line.get_f32("zoom")) else {
        return Ok(None);
    };
    if zoom <= 0.0 {
        return Ok(None);
    }
    let cx = f64::from(canvas.min.x + canvas.max.x) / 2.0;
    let cy = f64::from(canvas.min.y + canvas.max.y) / 2.0;
    let z = f64::from(zoom);
    Ok(Some(Held {
        page: (
            (cx - f64::from(rect.min.x)) / z,
            (cy - f64::from(rect.min.y)) / z,
        ),
        zoom: z,
        span: f64::from(canvas.max.x - canvas.min.x) / z,
    }))
}

/// Which position tier the canvas last reported.
fn tier(session: &Session) -> Result<String> {
    let trace = session.trace()?;
    Ok(trace
        .events(POS_EVENT)
        .last()
        .and_then(|l| l.get("tier"))
        .unwrap_or("?")
        .to_owned())
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let vocab = &ctx.profile.vocab;
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no --pdf. There is nothing to zoom."))?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check pans and zooms the canvas. Reported as \
             SKIPPED rather than passed.",
        ));
    }
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("zoom-keeps-place.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    let driver = Driver::new(session.window());

    let trace = session.trace()?;
    let canvas = driving::declared(&trace, ui_rect, CANVAS_REGION)
        .ok_or_else(|| Error::new(format!("no `{CANVAS_REGION}`; is a document open?")))?;
    let frame = session.frame()?;
    let centre = frame.declared_at(canvas, 0.5, 0.5);

    // --- knock the view off centre, at the starting (page-fit) zoom ----------
    //
    // ★ The middle button is the operator's own gesture; this harness has no
    // middle drag, and a primary drag pans only under the hand tool. The wheel
    // is unconditional and moves the view off the centred position, which is
    // all this check needs in order to be capable of failing.
    driver.scroll_at(frame.declared_at(canvas, PAN_AT.0, PAN_AT.1), -4)?;
    session.settle(20);

    let Some(mut prev) = held(&session, canvas)? else {
        return Err(Error::new(
            "the canvas never published a rect and a zoom, so there is no page point to follow. \
             SKIPPED.",
        ));
    };
    report.note(format!(
        "panned off-centre; the page point under the centre is ({:.2}, {:.2}) at {:.0}%",
        prev.page.0,
        prev.page.1,
        prev.zoom * 100.0
    ));

    let mut tiers: Vec<String> = Vec::new();
    let mut worst = 0.0_f64;
    let mut climbed = 0usize;
    for stage in 0..STAGES {
        let stage_from = prev.zoom;
        // ★★ ONE NOTCH AT A TIME. See `DRIFT_FRACTION` — reading once per
        // stage compared eight steps of accumulated rounding against the
        // budget for one, and failed a correct build by 3.17 pt against 2.57.
        for notch in 0..STAGE {
            driver.scroll_at_held(centre, &[VK_CONTROL], 1, 1)?;
            session.settle(6);

            let now = tier(&session)?;
            if !tiers.contains(&now) {
                tiers.push(now.clone());
            }

            let Some(after) = held(&session, canvas)? else {
                return Err(Error::new(
                    "the canvas stopped publishing a rect and a zoom. SKIPPED.",
                ));
            };
            let drift = (after.page.0 - prev.page.0)
                .abs()
                .max((after.page.1 - prev.page.1).abs());
            let allowed = prev.span.min(after.span) * DRIFT_FRACTION;
            worst = worst.max(if allowed > 0.0 { drift / allowed } else { 0.0 });
            if after.zoom > prev.zoom {
                climbed += 1;
            }

            if drift > allowed {
                return Ok(Some(format!(
                    "★★ THE ZOOM THREW THE VIEW AWAY. Notch {notch} of stage {stage}, between \
                     {:.0}% and {:.0}% (tier `{now}`): the page point under the viewport centre \
                     moved from ({:.4}, {:.4}) to ({:.4}, {:.4}) — {drift:.4} pt, where \
                     {allowed:.4} is the tolerance. The pointer was ON the centre, so \
                     zoom-to-cursor should have held that point. On tier `scroll` this is O24e: \
                     `geometry::zoom_anchor_offset` clamping to `display - viewport`, which is \
                     zero or negative whenever the page is no larger than the viewport, forcing \
                     the offset to the centred position. On tier `deep` it is O24f: the anchor \
                     seeded from the PREVIOUS frame's scroll offset, or never moved on a zoom \
                     at all because nothing called `DeepAnchor::zoomed_about`.",
                    prev.zoom * 100.0,
                    after.zoom * 100.0,
                    prev.page.0,
                    prev.page.1,
                    after.page.0,
                    after.page.1
                )));
            }
            prev = after;
        }
        report.note(format!(
            "stage {stage}: {:.0}% to {:.0}% on tier `{}`, worst per-notch drift so far is \
             {:.0}% of the tolerance",
            stage_from * 100.0,
            prev.zoom * 100.0,
            tiers.last().map_or("?", String::as_str),
            worst * 100.0
        ));
    }

    // ★ A run that never climbed has said nothing. Checked once at the end
    // rather than per notch: a single notch that lands on the rung the zoom is
    // already at is not a fault, a whole run that never moves is.
    if climbed < STAGES {
        return Err(Error::new(format!(
            "only {climbed} of {} wheel notches zoomed in, ending at {:.0}%. Either the \
             maximum-zoom setting is capping this run or the Ctrl was lost and the wheel \
             panned instead. SKIPPED rather than passed: a run that did not zoom has said \
             nothing about whether zooming keeps its place.",
            STAGES * STAGE,
            prev.zoom * 100.0
        )));
    }

    // ★★★ REFUSE TO PASS A RUN THAT NEVER LEFT ONE TIER.
    //
    // Half of what this check is for is the HAND-OVER between the `f32` scroll
    // offset and the `f64` anchor, and a run that stayed on one side of it has
    // not tested that at all. The same guard as `deep_pan`'s
    // `REGION_TIER_REQUIRED`, for the same reason: on 2026-08-22 a check
    // reported PASS twice against a binary with the defect deliberately put
    // back in, because it never reached the tier it was named after.
    if tiers.len() < 2 {
        return Err(Error::new(format!(
            "every reading was on tier `{}` — the run never crossed the boundary between the f32 \
             scroll offset and the f64 anchor, which is half of what this check is for. That \
             boundary is at SUB_PIXEL_CONTENT_EXTENT / page_height, about 2,118,000 % on a US \
             Letter sheet. Raise STAGES or STAGE until it is crossed. SKIPPED rather than passed.",
            tiers.first().map_or("?", String::as_str)
        )));
    }
    report.note(format!("crossed tiers: {}", tiers.join(" to ")));

    Ok(None)
}
