//! `panning_at_deep_zoom_stays_where_it_was_put` — the operator's "it jumps
//! back" report, made falsifiable.
//!
//! # The report
//!
//! `OPERATOR_REQUESTS.md` O24, 2026-08-22:
//!
//! > *"is that the challenge I was running into trying to pan over a little bit
//! > at high zoom, but it would jump back to it's original location I panned
//! > from because I couldn't pan to the next point?"*
//!
//! Two claims in one sentence and they need separating, because they have
//! different causes and only one of them is a bug:
//!
//! | claim | would be |
//! |---|---|
//! | *"I couldn't pan to the next point"* | a **quantised** pan — the view refuses small movements and only moves in steps |
//! | *"it would jump back to its original location"* | a **reverting** pan — the view moves and is then put back |
//!
//! ★★ A quantised pan is what an `f32` scroll offset does when its
//! representable spacing exceeds the drag: `last - pan` rounds straight back to
//! `last`, so the view does not move at all. It looks like the drag was
//! ignored. A reverting pan is something actively re-setting the offset after
//! the drag — a different fault with a different fix.
//!
//! This check tells them apart by measuring the offset at three moments: before
//! the drag, immediately after, and several frames later.
//!
//! # ★★ Why it rolls the wheel rather than dragging
//!
//! The first version drag-panned with the primary button and reported the view
//! as stuck. That was a **harness** defect: `canvas::input::pan_delta` pans on
//! the middle button always and on the primary button only under the hand
//! tool, so a primary drag with the default Select tool correctly rubber-band
//! selected and correctly moved nothing. The check had measured a gesture the
//! application never offered, and blamed the application for not honouring it.
//!
//! It is recorded here rather than quietly fixed because it is the same shape
//! as the false layout report in `checks::driving::declared`'s header: a
//! measurement of the wrong thing is indistinguishable, from the verdict line,
//! from a real defect. **Ask what the check sampled before asking what is
//! broken.**
//!
//! The wheel is unconditional — no tool, no modifier, no button — so a view
//! that does not move under it is unambiguously the application's fault.

use crate::checks::driving;
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};

/// The zoom group on the status bar — its right end is `+`.
const ZOOM_REGION: &str = "status-group:zoom";

/// The canvas viewport, which the drag happens inside.
const CANVAS_REGION: &str = "canvas-viewport";

/// The canvas's own state line, read for the zoom it reports.
const CANVAS_EVENT: &str = "canvas";

/// The canvas's `f64` pan position — the only field that can measure this.
///
/// ★ Not `canvas`'s `off=` and not its `rect=`. Both are `f32`, and at the
/// zoom this check works at their representable spacing is larger than the
/// drag, so both would read "unchanged" against an application that panned
/// perfectly. See `canvas::trace::position`.
const POS_EVENT: &str = "canvas-pos";

/// How far to zoom in before the FIRST probe.
///
/// Lands around 100,000 %, which is past the whole-page raster ceiling and on
/// the `scroll` position tier — the tier the operator was on when he reported
/// this.
const PRESSES: usize = 16;

/// How many MORE presses before the second probe.
///
/// # ★★ Why the check probes twice
///
/// The position is owned by two different mechanisms at two different depths —
/// an `f32` scroll offset below the deep threshold and an `f64`
/// `viewer::deep::DeepAnchor` above it — and they are a hard branch, not a
/// re-parameterisation. A pan that works on one says nothing about the other.
///
/// ★ Probing only the deepest would have been the tempting choice and it would
/// have missed the operator's actual case, which was on the shallow tier. One
/// probe per mechanism is the minimum that can honestly claim panning works
/// "at high zoom".
const MORE_PRESSES: usize = 44;

/// How many wheel notches to roll.
///
/// # ★★ Why the wheel and not a drag
///
/// The first version of this check drag-panned with the primary button and
/// reported a stuck view. It was wrong: `canvas::input::pan_delta` pans on the
/// **middle** button always, and on the primary button only while the hand
/// tool is active. The default tool is Select, so a primary drag correctly
/// rubber-band selected and correctly did not move the view. The harness had
/// measured a gesture the application never claimed to honour, and blamed the
/// application.
///
/// The wheel is unconditional — no tool, no modifier, no button — so a view
/// that does not move under it is unambiguously the application's. It is also
/// what an operator reaches for first.
///
/// Three notches: small enough to be the "little bit" he described, large
/// enough that a working build moves visibly.
const NOTCHES: i32 = -3;

/// See the module documentation.
pub struct PanningAtDeepZoomStaysWhereItWasPut;

impl Check for PanningAtDeepZoomStaysWhereItWasPut {
    fn name(&self) -> &'static str {
        "panning_at_deep_zoom_stays_where_it_was_put"
    }

    fn defect(&self) -> &'static str {
        "panning a little at high zoom does nothing, or moves and then jumps back to where it \
         started — so a point just off screen cannot be reached at all"
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

/// The canvas's reported `f64` pan position, and which tier produced it.
fn position(session: &Session) -> Result<Option<((f64, f64), String)>> {
    let trace = session.trace()?;
    Ok(trace.events(POS_EVENT).last().and_then(|l| {
        let at = l.get("at")?;
        let (x, y) = at.split_once(',')?;
        let tier = l.get("tier").unwrap_or("?").to_owned();
        Some(((x.parse().ok()?, y.parse().ok()?), tier))
    }))
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
        .ok_or_else(|| Error::new("no --pdf. There is nothing to pan."))?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check zooms in and drags the canvas. Reported \
             as SKIPPED rather than passed.",
        ));
    }
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("deep-pan.trace.txt"));
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
    let zoom_group = driving::declared(&trace, ui_rect, ZOOM_REGION)
        .ok_or_else(|| Error::new(format!("no `{ZOOM_REGION}`; is a document open?")))?;
    let canvas = driving::declared(&trace, ui_rect, CANVAS_REGION)
        .ok_or_else(|| Error::new(format!("no `{CANVAS_REGION}`; is a document open?")))?;
    let frame = session.frame()?;

    for _ in 0..PRESSES {
        driver.click_at(frame.declared_at(zoom_group, 0.93, 0.5))?;
        session.settle(8);
    }
    session.settle(60);

    let zoom = session
        .trace()?
        .events(CANVAS_EVENT)
        .last()
        .and_then(|l| l.get_f32("zoom"))
        .unwrap_or(0.0);
    report.note(format!("zoomed to {:.0}%", zoom * 100.0));

    if let Some(bad) = probe(&session, &driver, &frame, canvas, report)? {
        return Ok(Some(bad));
    }

    // …and again on the other side of the deep threshold, where a different
    // mechanism owns the position entirely. See `MORE_PRESSES`.
    for _ in 0..MORE_PRESSES {
        driver.click_at(frame.declared_at(zoom_group, 0.93, 0.5))?;
        session.settle(8);
    }
    session.settle(120);
    if let Some(bad) = probe(&session, &driver, &frame, canvas, report)? {
        return Ok(Some(bad));
    }
    Ok(None)
}

/// Roll the wheel once over the canvas and report what the view did.
///
/// Returns `Ok(None)` when the view moved and stayed moved, and
/// `Ok(Some(verdict))` when it did not — the two failure shapes described in
/// this module's header.
///
/// ★ Takes the report so its notes carry BOTH probes' numbers. A verdict that
/// says "the view did not move" without saying which tier it was on sends the
/// next reader to whichever of the two files they guess first.
fn probe(
    session: &Session,
    driver: &Driver,
    frame: &crate::coords::WindowFrame,
    canvas: crate::geom::LRect,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    let zoom = session
        .trace()?
        .events(CANVAS_EVENT)
        .last()
        .and_then(|l| l.get_f32("zoom"))
        .unwrap_or(0.0);

    let Some((before, tier)) = position(session)? else {
        return Err(Error::new(
            "the canvas never reported a `canvas-pos` line, so there is nothing to compare. \
             Either the build predates the f64 position trace or no page was drawn. SKIPPED.",
        ));
    };

    driver.scroll_at(frame.declared_at(canvas, 0.5, 0.5), NOTCHES)?;
    session.settle(20);
    let Some((after, _)) = position(session)? else {
        return Err(Error::new(
            "the canvas stopped reporting a position. SKIPPED.",
        ));
    };

    // ★ And again several frames later. A view that moves and is then put back
    // is a different defect from one that never moved, and only a second
    // reading can tell them apart — the operator described both in one
    // sentence, so the check must be able to say which he saw.
    session.settle(90);
    let Some((settled, _)) = position(session)? else {
        return Err(Error::new(
            "the canvas stopped reporting a position. SKIPPED.",
        ));
    };

    report.note(format!(
        "at {:.0}% on tier `{tier}`: {before:?} → {after:?} → {settled:?}",
        zoom * 100.0
    ));

    let moved = (after.0 - before.0).abs() + (after.1 - before.1).abs();
    // One notch is tens of pixels on every platform this runs on; a build that
    // moved by less than a single pixel did not move.
    if moved < 1.0 {
        return Ok(Some(format!(
            "★★ THE VIEW DID NOT MOVE. {NOTCHES} wheel notches at {:.0}% zoom left the position \
             at {before:?} (tier `{tier}`, moved {moved:.3} px). This is the operator's \"I \
             couldn't pan to the next point\". On the `scroll` tier it means the `f32` offset's \
             representable spacing exceeds the movement, so `last - pan` rounds back to `last`; \
             on the `deep` tier it means the wheel is not reaching `DeepAnchor::panned`.",
            zoom * 100.0
        )));
    }

    let reverted = (settled.0 - before.0).abs() + (settled.1 - before.1).abs();
    if reverted < moved / 2.0 {
        return Ok(Some(format!(
            "★★ THE VIEW REVERTED. The wheel moved the position from {before:?} to {after:?} \
             at {:.0}% on tier `{tier}`, and ninety frames later it is back at {settled:?}. This \
             is the operator's \"it would jump back to its original location\" — something is \
             re-setting the position after the gesture. Read what forces one every frame: \
             `zoom::consume_anchor`, `find::take_reveal_offset` and `strip::page_scroll_offset`, \
             and the last of those fires whenever the current-page reading disagrees with the \
             tracked one.",
            zoom * 100.0
        )));
    }

    Ok(None)
}
