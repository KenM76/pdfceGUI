//! `the_line_weight_switch_reaches_the_resize` — **tick the switch, drag a
//! grip, and the border thickens with the shape.**
//!
//! # What this is for
//!
//! `OPERATOR_REQUESTS.md` **O51**:
//!
//! > *"if that was the resize question about scaling line weight, etc with
//! > resize it got the answer wrong. default should be what it said, but there
//! > should be an option that they do scale with resize. Inkscape has options
//! > for this and I want the same."*
//!
//! ## ★★★ What this check exists to catch, which is not "the switch is missing"
//!
//! The switch is a checkbox writing a `bool` into `egui::Memory`. Nothing about
//! that can plausibly fail. **The chain in front of the engine is what fails**,
//! and it has five links, three of which are pure wiring:
//!
//! | # | link | a unit test can see it? |
//! |---|---|---|
//! | 1 | the Select tool's option row is drawn at all | partly — `armed::options` returns early for every other tool |
//! | 2 | the checkbox writes the store | yes — `canvas::scaling`'s round trip |
//! | 3 | the value reaches `resizing::Frame` on the commit frame | **no** |
//! | 4 | it travels on the action rather than being re-read at apply time | **no** |
//! | 5 | it reaches `ResizeOptions` and the engine acts on it | yes — `to_options` |
//!
//! ★★ Link 3 is the one that was wrong for the life of the feature and in the
//! opposite direction: `annots::resize` **derived** `scale_stroke_width` from
//! whether the drag was proportional, so an operator's answer could not reach
//! the engine at all. A build that regressed to that would pass every unit test
//! in the chain, because each end of it is correct in isolation.
//!
//! ## ★★ The oracle is `stroke=` on the applied line, not a pixel
//!
//! `resize-annotation-applied … stroke=true|false` reports whether the engine
//! wrote a new `/BS /W`. A screenshot cannot separate *"the border thickened
//! because `/BS /W` changed"* from *"the border thickened because §12.5.5's
//! matrix scaled the drawn stroke"* — those are different outcomes with the
//! same picture, and only the first is the switch doing its job.
//!
//! ⇒ **The picture is the same in the case this check is about.** That is why
//! it reads the trace, and it is a fact about the format rather than a
//! limitation of the harness — the identical argument `markup_move` makes for
//! its `keys=` field.
//!
//! ## The sequence
//!
//! | # | step | oracle |
//! |---|---|---|
//! | A | Review mode, rectangle tool, drag a shape | `markup-commit` |
//! | B | Escape, click it | `annot-select` |
//! | C | open the Tool panel, click the *Scale line weight* switch | `resize-modifiers stroke=true` |
//! | D | drag a corner grip **proportionally** | `resize-annotation-applied … stroke=true` |
//!
//! ★ Step D drags **diagonally by equal amounts** on purpose. A non-uniform
//! resize of a pdfce-authored appearance is fine — it is rebuilt — but making
//! the drag uniform keeps this check about the switch rather than about the
//! distortion refusal, which is a different feature with a different sentence.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::text_selection::aim;
use crate::checks::{Check, CheckContext};
use crate::coords::{DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Review mode, the rectangle tool, and the Tool panel — all three rung.
///
/// ★ `view.panel_tool` last, because the panel changes the dock's width and
/// therefore the canvas rect. Opening it **before** the shape is drawn would
/// mean every coordinate this check computes was taken against a layout that
/// then moved — the harness-coordinate hazard `D:/dev/rag/egui/` records.
///
/// ⇒ It is rung after the drawing steps for that reason and the mapping is
/// re-read afterwards.
const INVOKE: &str = "mode.review,markup.rectangle";
/// Opens the Tool panel, rung separately after the shape exists.
const PANEL_COMMAND: &str = "view.panel_tool";
/// The line the canvas writes when a shape is authored.
const COMMIT_EVENT: &str = "markup-commit";
/// The line the canvas writes when a click selects an annotation.
const SELECT_EVENT: &str = "annot-select";
/// The switch's own published rect.
const SWITCH_REGION: &str = "tool.scale.stroke";
/// The line the panel writes when a switch changes.
const MODIFIERS_EVENT: &str = "resize-modifiers";
/// The line the apply arm writes when the engine has resized it.
const APPLIED_EVENT: &str = "resize-annotation-applied";
/// The page's own region, so a failure can say whether a sheet was drawn.
const PAGE_REGION: &str = "page";

/// Where the shape is drawn, as fractions of the page.
const SHAPE: ((f64, f64), (f64, f64)) = ((0.30, 0.30), (0.50, 0.45));
/// How far the bottom-right corner is dragged, as fractions of the page.
///
/// ★ **Equal in both axes**, so the scale is uniform. See the module header.
const GRIP_TRAVEL: (f64, f64) = (0.10, 0.10);

/// See the module documentation.
pub struct TheLineWeightSwitchReachesTheResize;

impl Check for TheLineWeightSwitchReachesTheResize {
    fn name(&self) -> &'static str {
        "the_line_weight_switch_reaches_the_resize"
    }

    fn defect(&self) -> &'static str {
        "the operator's Scale line weight switch cannot reach the engine — `annots::resize` \
         DERIVES `scale_stroke_width` from whether the drag was proportional, so the switch is \
         drawn, stores its value, and is overridden on exactly the resizes where somebody was \
         most likely to have an opinion"
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
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check draws a shape, selects it, ticks a \
             checkbox in a dock panel and drags a grip. Every one is a real pointer gesture.",
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
        .ok_or_else(|| Error::new("no --pdf. This check needs a page to draw a shape on."))?;
    let page: PageGeometry = match ctx.page_size {
        Some((w, h)) => PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&pdf).ok_or_else(|| {
            Error::new(format!(
                "could not read a page size from {}, and this check places its shape in page \
                 fractions. Pass --page-size.",
                pdf.display()
            ))
        })?,
    };
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("scale-switch.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env.push((
        "PDFCE_DIAG_INVOKE".to_owned(),
        format!("{INVOKE},{PANEL_COMMAND}"),
    ));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {} with the Tool panel open",
        exe.display(),
        session.pid()
    ));
    session.settle(45);
    let driver = Driver::new(session.window());

    if declared(&session.trace()?, ui_rect, PAGE_REGION).is_none() {
        return Err(Error::new(format!(
            "the application declared no `{PAGE_REGION}` region, so no sheet is on screen and \
             there is nothing to draw on. Regions beginning `page`: {}.",
            list(&declared_names(&session.trace()?, ui_rect, "page"))
        )));
    }

    // --- A: draw a rectangle ------------------------------------------------
    //
    // ★ The mapping is taken NOW, after the Tool panel has opened and settled.
    // A rect computed before the dock's width changed would aim at the page as
    // it used to sit — the harness-coordinate hazard, which this project has
    // already met once and written up.
    let corner = |f: (f64, f64)| DocPoint::new(0, f.0 * page.width_pt, f.1 * page.height_pt);
    let from = aim(ctx, &session, page, corner(SHAPE.0))?;
    let to = aim(ctx, &session, page, corner(SHAPE.1))?;
    driver.drag(from, to)?;
    session.settle(30);

    let trace = session.trace()?;
    if trace.events(COMMIT_EVENT).count() == 0 {
        return Ok(Some(format!(
            "THE RECTANGLE TOOL AUTHORED NOTHING: no `{COMMIT_EVENT}` line, so \
             `markup.rectangle` did not arm or the drag was not seen as one. Two steps BEFORE \
             the one under test. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("★ a rectangle was authored");

    // --- B: put the pen down, then select it --------------------------------
    //
    // ★ Escape first. The markup pen stays armed after a commit, so a click
    // without this draws a second rectangle instead of selecting the first —
    // `markup_move` records the same step and the same reason.
    driver.press(crate::sys::vk::V)?;
    session.settle(12);
    let centre = corner((
        f64::midpoint(SHAPE.0.0, SHAPE.1.0),
        f64::midpoint(SHAPE.0.1, SHAPE.1.1),
    ));
    driver.click_at(aim(ctx, &session, page, centre)?)?;
    session.settle(24);

    if session.trace()?.events(SELECT_EVENT).count() == 0 {
        return Ok(Some(format!(
            "THE SHAPE COULD NOT BE SELECTED: no `{SELECT_EVENT}` line after a click at its \
             centre, so no grips are drawn and there is nothing to drag. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("★ the shape was selected, so its grips are drawn");

    // --- C: tick the switch -------------------------------------------------
    let trace = session.trace()?;
    let Some(switch) = declared(&trace, ui_rect, SWITCH_REGION) else {
        return Ok(Some(format!(
            "★★★ THE SWITCH IS NOT DRAWN: the application declared no `{SWITCH_REGION}` region \
             with the Select tool armed and the Tool panel open.\n\
             `panels::tool::armed::options` returns early for every tool but Select, and this \
             check disarms the markup pen at step B — so if the pen did not go down, the panel \
             is showing the markup options instead. Regions beginning `tool.`: {}. Trace: {}.",
            list(&declared_names(&trace, ui_rect, "tool.")),
            session.trace_path().display()
        )));
    };
    if !switch.is_substantial() {
        return Err(Error::new(format!(
            "`{SWITCH_REGION}` was declared at {switch:?}, which has no usable area to click — \
             the dock is probably too narrow to lay the row out."
        )));
    }
    driver.click_at(session.frame()?.declared_center(switch))?;
    session.settle(20);

    let trace = session.trace()?;
    let Some(modifiers) = trace
        .events(MODIFIERS_EVENT)
        .filter(|l| l.get("stroke") == Some("true"))
        .last()
    else {
        return Ok(Some(format!(
            "★★ THE SWITCH DID NOT TAKE: a click at the centre of `{SWITCH_REGION}` produced no \
             `{MODIFIERS_EVENT} stroke=true` line.\n\
             That line is written only when the value CHANGES, so either the click missed the \
             checkbox — its rect is published from the response's own rect, so a miss means the \
             dock moved between the trace and the click — or the store did not take it. Trace: \
             {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("★★ the switch is on: `{}`", modifiers.raw));

    // --- D: drag the bottom-right grip, proportionally ----------------------
    let grip = aim(ctx, &session, page, corner(SHAPE.1))?;
    let landing = aim(
        ctx,
        &session,
        page,
        corner((SHAPE.1.0 + GRIP_TRAVEL.0, SHAPE.1.1 + GRIP_TRAVEL.1)),
    )?;
    driver.drag(grip, landing)?;
    session.settle(40);

    let trace = session.trace()?;
    let Some(applied) = trace.events(APPLIED_EVENT).last() else {
        return Ok(Some(format!(
            "★★ THE GRIP DRAG REACHED NO RESIZE: no `{APPLIED_EVENT}` line.\n\
             Either the press missed the grip — it aimed at the shape's own bottom-right \
             corner, which is where the grip is centred — or `resize_annotation` refused. A \
             refusal traces `resize-annotation-refused`; look for that first, and note that a \
             pdfce-authored appearance is REBUILT and should never be refused. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("★★★ the engine resized it: `{}`", applied.raw));

    // --- the oracle ---------------------------------------------------------
    if applied.get("stroke") != Some("true") {
        return Ok(Some(format!(
            "★★★ THE SWITCH DID NOT REACH THE ENGINE: `{}` reports stroke=false, and \
             `{MODIFIERS_EVENT} stroke=true` was recorded before the drag.\n\
             **This is the state the feature was in until 2026-08-28**, in the opposite \
             direction: `annots::resize` DERIVED `scale_stroke_width` from whether the drag was \
             proportional rather than from the operator's answer. Check that \
             `resizing::Frame::modifiers` is read on the commit frame, that it travels on \
             `Action::Annot(Resize)` rather than being re-read at apply time, and that \
             `Modifiers::to_options` is what builds the request. Trace: {}.",
            applied.raw,
            session.trace_path().display()
        )));
    }
    if applied.get("uniform") != Some("true") {
        return Err(Error::new(format!(
            "the drag came out NON-uniform (`{}`), and this check drags equal amounts in both \
             axes on purpose so that it is about the switch rather than about the distortion \
             refusal. Reported as SKIPPED rather than failed: the assertion above passed, and a \
             non-uniform drag means the harness's two axes disagree — probably a page whose \
             aspect ratio makes equal page-fraction travel unequal in points.",
            applied.raw
        )));
    }
    report.note("★ the resize was uniform and the engine wrote a new border width");
    Ok(None)
}
