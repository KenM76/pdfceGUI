//! `a_pan_leaves_the_fit` — **fit, pan away, resize, and the view stays where
//! the operator put it.**
//!
//! # The request
//!
//! `OPERATOR_REQUESTS.md` **O55**, 2026-08-28:
//!
//! > *"if the canvas window is resized the pdf should resize to match **unless
//! > the person has changed the zoom or panned around**."*
//!
//! Its sibling `a_fit_command_puts_the_page_on_screen` proves the first half —
//! a resize re-fits and re-places. **This one proves the exception**, and the
//! two are deliberately separate checks because they assert opposite outcomes
//! from the same gesture: there, the resize must move the page; here, it must
//! not.
//!
//! ## ★★★ Why this half is the one with the bite
//!
//! `ViewState::set_zoom` has always dropped out of a fit, so *"changed the
//! zoom"* held before today. **Panning did not.** `doc.last_scroll_offset` is
//! written every frame from the scroll area's own state and nothing consulted
//! the fit — so an operator who fitted, then panned to look at a corner, was
//! still *in* fit mode.
//!
//! ⇒ On its own that was harmless, because a fit only ever placed the page
//! once. The moment a resize re-places — which is the other half of O55 — it
//! becomes **the resize throwing the operator's position away**, and the two
//! changes have to land together or the pair is worse than neither.
//!
//! ## ★★ Why the HAND tool, and why the primary button
//!
//! `canvas::input::pan_delta` treats two gestures as one pan: the middle
//! button always, and the primary button while the hand tool is active. This
//! harness has no middle-button driver — a third gesture-class hole, found the
//! same day as the secondary click and the window resize — so the check arms
//! the hand tool and uses the primary drag it already has.
//!
//! ★ That is not a workaround around the subject: `pan_delta` is one function
//! and both buttons reach it, so a build that dropped the fit for one and not
//! the other is not reachable. The check drives the door that exists.
//!
//! ## ★★★ The wheel is deliberately NOT this gesture, and the sibling proves it
//!
//! Scrolling a fit-width document is how every reader in the class is read,
//! and a wheel notch that dropped the fit would stop the page re-fitting the
//! moment anybody looked at the second half of it.
//!
//! ⇒ `a_fit_command_puts_the_page_on_screen` **wheel-scrolls into the
//! pasteboard and then asserts the fit still places the page**. So the pair
//! pins both directions: the wheel keeps the fit, a pan leaves it. Neither
//! check can be satisfied by a build that treats all view movement alike.
//!
//! ## ★★★ Falsified, 2026-08-28, and the result names its own limit
//!
//! With `doc.view.set_fit(FitMode::None)` removed from `canvas::offset`'s pan
//! arm and nothing else changed, this check fails and reports:
//!
//! > `margins l=8.0 r=8.0 t=108.4 b=108.3`
//!
//! — dead centre on both axes, against `l=-39.0 r=-105.0 t=120.4 b=-27.3` for
//! the correct build. So the assertion is live and the tolerance is nowhere
//! near either result.
//!
//! ★★ **It would NOT have caught the state this shipped in before today**, and
//! that is worth saying rather than leaving somebody to assume otherwise. Then,
//! a resize re-placed nothing at all, so a panned page was not re-centred and
//! this check would have passed for the wrong reason. It has teeth only against
//! a build that re-places on a resize — which is the other half of O55, and the
//! state that existed for the ten minutes between the two edits.
//!
//! ⇒ **A guard written with a fix guards the fix, not the original defect.**
//! The original is covered by its sibling; this covers the regression the fix
//! made possible.
//!
//! # The sequence
//!
//! | # | step | oracle |
//! |---|---|---|
//! | A | press **Fit page**, note where the page is | `canvas … rect=` |
//! | B | **pan** with the hand tool, assert the page actually moved | the rect moved |
//! | C | **resize** the window | `canvas-viewport` changed |
//! | D | the page is **not** re-centred | its margins are still lopsided |
//!
//! ★ Step B asserts its own precondition, for the reason the sibling states: a
//! pan that did nothing would leave the page centred, and step D would then
//! pass against a build that re-centres on every resize — measuring nothing.

use super::fit_places_the_view::{invoke, page_rect};
use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Arms the hand tool, so a primary drag is a pan. See the module header.
const INVOKE: &str = "view.tool_hand";
/// The canvas viewport, which every measurement below is made against.
const CANVAS_REGION: &str = "canvas-viewport";
/// The Fit page control.
const FIT_ITEM: &str = "ribbon.item.view.zoom_fit_page";
/// How far the pan drags, as a fraction of the canvas on each axis.
///
/// ★★ A fraction, not a distance. `the_line_weight_switch_reaches_the_resize`
/// got the same class of constant wrong three times in one evening — page
/// fractions, then points, then fractions of the operand — and the lesson
/// generalises: **the space a travel is expressed in has to be the space the
/// thing being measured lives in.** Here the subject is *"did the page move
/// within the canvas"*, so the canvas is the space.
///
/// 0.25 of the canvas is far past [`CENTRED_TOLERANCE`] at any window size
/// this harness will produce.
const PAN_BY: f32 = 0.25;
/// How much smaller the window is made, in physical pixels.
const RESIZE_BY_PX: i32 = 160;
/// The window chrome, added back so the restore returns to the start size.
const BORDER_PX: i32 = 16;
/// The title bar and border, vertically.
const TITLEBAR_PX: i32 = 39;
/// How far a margin may differ and still count as "centred", in points.
///
/// ★ The same role as the sibling's `EDGE_TOLERANCE`: it absorbs the `f32` fit
/// division and pixel-grid rounding, and nothing else. The pan moves the page
/// by [`PAN_BY`], which is thirty times this.
const CENTRED_TOLERANCE: f32 = 4.0;

/// See the module documentation.
pub struct APanLeavesTheFit;

impl Check for APanLeavesTheFit {
    fn name(&self) -> &'static str {
        "a_pan_leaves_the_fit"
    }

    fn defect(&self) -> &'static str {
        "a fit stays active after the operator pans away from it, so the next canvas resize \
         re-places the page and throws away the position they chose — the half of O55 that \
         only bites once a resize re-places at all"
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

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a ribbon control, drags the page \
             and resizes the window.",
        ));
    }
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no --pdf. This check needs a page to fit."))?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("fit-pan.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCE_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    session.settle(45);
    let driver = Driver::new(session.window());

    let trace = session.trace()?;
    let canvas = declared(&trace, ui_rect, CANVAS_REGION).ok_or_else(|| {
        Error::new(format!(
            "no `{CANVAS_REGION}`; is a document open? Regions: {}.",
            list(&declared_names(&trace, ui_rect, "canvas"))
        ))
    })?;

    // --- A: fit the page ----------------------------------------------------
    invoke(&session, &driver, ui_rect, FIT_ITEM)?;
    session.settle(24);
    let Some(fitted) = page_rect(&session)? else {
        return Err(Error::new(
            "the canvas published no page rect after Fit page. SKIPPED.".to_owned(),
        ));
    };
    report.note(format!(
        "★ fitted: page {:.1},{:.1}..{:.1},{:.1}",
        fitted.min.x, fitted.min.y, fitted.max.x, fitted.max.y
    ));

    // --- B: pan away, and assert it moved -----------------------------------
    let frame = session.frame()?;
    // ★ Both ends expressed as FRACTIONS of the canvas rather than as a point
    // plus a pixel offset, so the drag scales with whatever viewport the window
    // happens to have — the same reason `PAN_BY` below is a fraction and not a
    // distance. A fixed pixel travel is a distance in the screen's space, and
    // the canvas is not the screen.
    let from = frame.declared_at(canvas, 0.35, 0.35);
    let to = frame.declared_at(canvas, 0.35 + PAN_BY, 0.35 + PAN_BY);
    driver.drag(from, to)?;
    session.settle(24);

    let Some(panned) = page_rect(&session)? else {
        return Err(Error::new(
            "the canvas stopped publishing a page rect after the pan. SKIPPED.".to_owned(),
        ));
    };
    if (panned.min.x - fitted.min.x).abs() < CENTRED_TOLERANCE
        && (panned.min.y - fitted.min.y).abs() < CENTRED_TOLERANCE
    {
        return Err(Error::new(format!(
            "the pan did not move the page (still at {:.1},{:.1}), so this run cannot tell a fit \
             that was LEFT from one that was never disturbed. The hand tool may not have armed — \
             `view.tool_hand` is rung through PDFCE_DIAG_INVOKE at launch. SKIPPED rather than \
             passed.",
            panned.min.x, panned.min.y
        )));
    }
    report.note(format!(
        "★★ panned: page {:.1},{:.1}..{:.1},{:.1}",
        panned.min.x, panned.min.y, panned.max.x, panned.max.y
    ));

    // --- C: resize ----------------------------------------------------------
    let Some(handle) = session.window() else {
        return Err(Error::new(
            "no window handle to resize. SKIPPED.".to_owned(),
        ));
    };
    let start = session.frame()?;
    let (cw, ch) = start.client_size;
    let (w, h) = (
        i32::try_from(cw).unwrap_or(1100) + BORDER_PX,
        i32::try_from(ch).unwrap_or(800) + TITLEBAR_PX,
    );
    crate::sys::resize_window(handle, w - RESIZE_BY_PX, h - RESIZE_BY_PX);
    session.settle(30);

    let after_trace = session.trace()?;
    let canvas_after = declared(&after_trace, ui_rect, CANVAS_REGION);
    let after = page_rect(&session)?;
    crate::sys::resize_window(handle, w, h);
    session.settle(25);

    let (Some(page_after), Some(canvas_after)) = (after, canvas_after) else {
        return Err(Error::new(
            "the canvas stopped publishing its rect across the resize. SKIPPED.".to_owned(),
        ));
    };
    if (canvas_after.width() - canvas.width()).abs() < CENTRED_TOLERANCE {
        return Err(Error::new(format!(
            "the resize did not change the canvas (still {:.1} pt wide), so there is no resize \
             for the fit to have survived. SKIPPED rather than passed.",
            canvas_after.width()
        )));
    }

    // --- D: the page must NOT have been re-centred --------------------------
    let left = page_after.min.x - canvas_after.min.x;
    let right = canvas_after.max.x - page_after.max.x;
    let top = page_after.min.y - canvas_after.min.y;
    let bottom = canvas_after.max.y - page_after.max.y;
    report.note(format!(
        "★★★ after the resize: margins l={left:.1} r={right:.1} t={top:.1} b={bottom:.1}"
    ));
    if (left - right).abs() < CENTRED_TOLERANCE && (top - bottom).abs() < CENTRED_TOLERANCE {
        return Ok(Some(format!(
            "★★★ THE RESIZE RE-CENTRED A PAGE THE OPERATOR HAD PANNED AWAY FROM: margins \
             l={left:.1} r={right:.1} t={top:.1} b={bottom:.1} are equal on both axes.\n\
             `OPERATOR_REQUESTS.md` O55: *\"unless the person has changed the zoom or panned \
             around\"*. The fit is still active after the pan, so `canvas::fit::placement` \
             re-placed on the viewport change and discarded the position they chose.\n\
             `canvas::offset`'s pan arm must call `doc.view.set_fit(FitMode::None)`. Note that \
             the WHEEL deliberately does not — `a_fit_command_puts_the_page_on_screen` asserts \
             the opposite for it, and a build that treats all view movement alike fails one of \
             this pair whichever way it goes. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("★ the pan survived the resize, so the fit was left as the operator expected");
    Ok(None)
}
