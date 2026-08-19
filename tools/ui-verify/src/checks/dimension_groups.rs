//! `dimension_groups_window_makes_a_group` — the Manage-groups window opens,
//! and a group made in it reaches the document and comes back joinable.
//!
//! # The gap this closes
//!
//! `measure.manage_groups` was registered, drawn on Measure ▸ Scale and
//! **inert** for the whole life of this build. The operator hit it by name on
//! 2026-08-18: *"I still can't get to edit dimension groups when I click on
//! it."*
//!
//! # Why this needs driving, and not a unit test
//!
//! Because the chain has five links and **four of them are frame-level or
//! cross-process**, and every individual link already had tests while the
//! feature did not exist:
//!
//! 1. a ribbon press reaches `app::dispatch`'s arm;
//! 2. the arm resolves the measure tool's active authoring group out of
//!    `egui::Memory` — which may hold no state at all — and builds the dialog;
//! 3. the dialog's Add button raises an `Action`, which is applied **after**
//!    the frame it was raised in;
//! 4. the apply calls `EditSession::add_dimension_group`, which writes the
//!    `/PieceInfo` sidecar;
//! 5. the **next** frame re-reads `dimension_model()` and draws a row for it.
//!
//! Link 5 is the one worth the whole check. A group that is created and does
//! not come back in the model is a group nothing can ever draw a dimension
//! into — and it looks *exactly* like success at links 1 through 4, because
//! the undo entry is there, the epoch moved, and the trace line says the verb
//! ran.
//!
//! # The assertion it would be easy to leave out
//!
//! The last one: **a second `draw_into` radio appears**. Asserting only the
//! `add-dimension-group` trace line would pass on a build where the window
//! writes to the document and lists nothing — which is the shape of every
//! panel in this project's history that shipped with a body, a rail entry and
//! no control anyone could click.
//!
//! The radio is also the *point of the feature*. `MeasureState::group` had
//! existed since the Phase 7 salvage, documented as the group picker, and
//! **nothing in the build ever wrote to it**: a second group could be created
//! and joined by nothing. A row with a radio on it is the only evidence that
//! the group is reachable rather than merely recorded.

use crate::checks::driving::{
    SHELL_DIAG_ENV, TAB_EVENT, declared, declared_names, declared_or_in_overflow, list, shell_trace,
};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

/// The mode whose tab list carries Measure.
const MODE: &str = "review";
/// The window's own region.
const WINDOW: &str = "dialog:dimension-groups";
/// The new-group name field.
const NAME_FIELD: &str = "dimension-groups.new_name";
/// The Add button.
const ADD: &str = "dimension-groups.add";
/// The prefix of the per-group authoring radios.
const DRAW_INTO: &str = "dimension-groups.draw_into.";
/// The appearance-defaults block, which proves the lower half drew at all.
const APPEARANCE: &str = "dimension-groups.appearance";
/// The trace event `vector_edit` emits when the engine verb succeeded.
///
/// `apply::vector_edit` traces the label it was given on the success path, so
/// this string is `DimensionAction::AddGroup`'s label verbatim. A *refusal*
/// traces `add-dimension-group-refused`, which is a different event and is
/// reported separately below — the difference between "the arm never ran" and
/// "the engine declined" is the whole diagnosis.
const APPLIED: &str = "add-dimension-group";
/// The keystrokes that spell the new group's name.
const NAME_KEYS: [u16; 6] = [vk::D, vk::E, vk::T, vk::A, vk::I, vk::L];

/// See the module documentation.
pub struct DimensionGroupsWindowMakesAGroup;

impl Check for DimensionGroupsWindowMakesAGroup {
    fn name(&self) -> &'static str {
        "dimension_groups_window_makes_a_group"
    }

    fn defect(&self) -> &'static str {
        "Measure > Manage dimension groups is drawn and does nothing — or it opens a window \
         that cannot create a group, or creates one the model never gives back, so a second \
         scale on one drawing is unreachable and every dimension lands in the default group \
         for ever"
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
             a ribbon control, a text field and a button, and types six letters. Reported as \
             SKIPPED rather than passed: a check that did not run has learned nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot say \
             where its controls are.",
            ctx.profile.name
        ))
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("dimension_groups.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    let driver = Driver::new(session.window());

    // --- 1: Review, so the Measure tab is offered --------------------------
    crate::checks::driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;

    // --- 2: the Measure tab ------------------------------------------------
    let trace = session.trace()?;
    let tab = declared(&trace, ui_rect, "ribbon.tab.measure").ok_or_else(|| {
        Error::new(format!(
            "no `ribbon.tab.measure` region after switching to {MODE}. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(14);
    if !shell_trace(&session)?
        .events(TAB_EVENT)
        .any(|l| l.get("tab") == Some("measure"))
    {
        return Err(Error::new(
            "the click on the Measure tab produced no tab-selected line, so nothing below \
             would mean anything.",
        ));
    }

    // --- 3: open the window ------------------------------------------------
    //
    // ★ Through `declared_or_in_overflow`, not `declared`. At the harness's
    // window width a band can legitimately fold controls into the overflow —
    // which on 2026-08-18 produced two FALSE failures that were believed and
    // written down as harness limitations. Looking in both places is the fix
    // that stopped that recurring.
    let Some(item) = declared_or_in_overflow(
        &session,
        &driver,
        ui_rect,
        "ribbon.item.measure.manage_groups",
    )?
    else {
        return Ok(Some(format!(
            "the Measure tab declares no `ribbon.item.measure.manage_groups`, on the band or \
             in the overflow. Items declared: {}.",
            list(&declared_names(
                &session.trace()?,
                ui_rect,
                "ribbon.item.measure."
            ))
        )));
    };
    driver.click_at(session.frame()?.declared_center(item))?;
    session.settle(18);

    let trace = session.trace()?;
    if declared(&trace, ui_rect, WINDOW).is_none() {
        // Distinguish "no arm" from "the arm declined", which is the same
        // diagnosis `page_ops::no_effect` draws and the one worth the lines: a
        // scaffolded command traces `command-unimplemented`, a gated one traces
        // `command-declined`, and a broken one traces neither.
        let unimplemented = trace
            .events("command-unimplemented")
            .any(|l| l.get("id") == Some("measure.manage_groups"));
        let declined = trace
            .events("command-declined")
            .filter(|l| l.get("id") == Some("measure.manage_groups"))
            .filter_map(|l| l.get("reason").map(str::to_owned))
            .last();
        return Ok(Some(if unimplemented {
            "`measure.manage_groups` was clicked and traced `command-unimplemented` — it is \
             still scaffolded. The control is drawn, it is on the ribbon, and there is no \
             dispatch arm behind it."
                .to_owned()
        } else if let Some(reason) = declined {
            format!(
                "`measure.manage_groups` was clicked and DECLINED with reason={reason}. The \
                 arm exists and refused; in {MODE} it should not."
            )
        } else {
            format!(
                "`measure.manage_groups` was clicked, traced neither a decline nor an \
                 unimplemented line, and no `{WINDOW}` region appeared. The arm ran and built \
                 no window."
            )
        }));
    }
    report.note("the Manage-groups window opened from Measure > Scale");

    // --- 4: it drew a list, and the list has the default group in it -------
    let before = declared_names(&trace, ui_rect, DRAW_INTO);
    if before.is_empty() {
        return Ok(Some(format!(
            "the window opened and declared no `{DRAW_INTO}*` region, so it drew no group \
             rows at all. Every document has a default group, so an empty list is the window \
             failing to read `dimension_model()` rather than a document with no groups."
        )));
    }
    if declared(&trace, ui_rect, APPEARANCE).is_none() {
        return Ok(Some(format!(
            "the window listed {} group row(s) and declared no `{APPEARANCE}` region, so the \
             lower half — the drafting standard, the layer switch and the five appearance \
             defaults — did not draw. A list with no settings under it is a picker, not a \
             manager.",
            before.len()
        )));
    }
    report.note(format!(
        "{} group row(s) listed, and the settings block drew: {}",
        before.len(),
        list(&before)
    ));

    // --- 5: type a name ----------------------------------------------------
    let field = declared(&trace, ui_rect, NAME_FIELD).ok_or_else(|| {
        Error::new(format!(
            "the window declared no `{NAME_FIELD}` region, so there is nothing to type a \
             group name into and the Add button can never leave its greyed state."
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(field))?;
    session.settle(10);
    for key in NAME_KEYS {
        driver.press(key)?;
        session.settle(3);
    }
    session.settle(10);

    // --- 6: add it ---------------------------------------------------------
    let trace = session.trace()?;
    let add = declared(&trace, ui_rect, ADD)
        .ok_or_else(|| Error::new(format!("the window declared no `{ADD}` region.")))?;
    let requested_before = trace.events("dimension-group-add").count();
    driver.click_at(session.frame()?.declared_center(add))?;
    session.settle(20);

    let trace = session.trace()?;
    if trace.events("dimension-group-add").count() <= requested_before {
        return Ok(Some(
            "the Add button took no click — no new `dimension-group-add` line was traced. \
             Either the six keystrokes never reached the name field (so the button is still \
             greyed, which is correct behaviour and means the TYPING failed) or the button is \
             drawn and inert."
                .to_owned(),
        ));
    }
    if let Some(refusal) = trace
        .events(&format!("{APPLIED}-refused"))
        .filter_map(|l| l.get("detail").map(str::to_owned))
        .last()
    {
        return Ok(Some(format!(
            "the Add button raised its action and the engine REFUSED it: {refusal}. The shell \
             half works; this is a `pdfce-core` verdict and belongs in a request."
        )));
    }
    if trace.last(APPLIED).is_none() {
        return Ok(Some(format!(
            "the Add button was pressed and traced its request, and no `{APPLIED}` line \
             followed — so the `Action` was raised and its apply arm never ran, or ran and \
             could not borrow the session. Nothing reached the document."
        )));
    }
    report.note("the group reached the document through the action funnel");

    // --- 7: ★ and it comes BACK, with a radio on it ------------------------
    let after = declared_names(&session.trace()?, ui_rect, DRAW_INTO);
    if after.len() <= before.len() {
        return Ok(Some(format!(
            "★ the group was WRITTEN and did not come back. `{APPLIED}` was traced, so \
             `EditSession::add_dimension_group` ran and the undo log has an entry — and the \
             window still lists {} row(s), the same as before. A group the model does not \
             give back is a group nothing can ever draw a dimension into, and it is \
             indistinguishable from success at every earlier step. Rows: {}.",
            after.len(),
            list(&after)
        )));
    }
    report.note(format!(
        "the new group is listed and carries an authoring radio: {}",
        list(&after)
    ));
    Ok(None)
}
