//! `dimension_groups_panel_makes_a_group` — the Dimension-groups **panel**
//! opens, a group made in it reaches the document and comes back joinable, and
//! the same group can then be renamed and removed.
//!
//! # ★ It drove a WINDOW until 2026-08-19, and the rewrite is the point
//!
//! The surface moved into the dock that day, because a window whose content
//! outgrew the screen could push its own title bar — and its only ✕ — off the
//! desktop, and the operator could not close it. `crate::checks` has no way to
//! notice that from a passing check: every assertion below passed against the
//! window on the run that shipped it.
//!
//! Two things about this check changed with the surface and both are worth
//! naming, because they are what a driven check is *for*:
//!
//! 1. **It clicks fold headings now.** Five of the panel's six sections start
//!    shut, so the regions this check aims at do not exist until a heading is
//!    pressed. That is not a cost — it means the check proves the folds work,
//!    which is half of what the operator asked for and the half a unit test
//!    cannot see.
//! 2. **It no longer asserts a window opened.** It asserts a panel *body*
//!    region appeared, which is a weaker claim about layout and a stronger one
//!    about reach: the panel is the last tab of a stack in Review's right dock,
//!    so its region appearing proves the ribbon control raised a tab that was
//!    behind another one.
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
//! `add-dimension-group` trace line would pass on a build where the panel
//! writes to the document and lists nothing — which is the shape of every
//! panel in this project's history that shipped with a body, a rail entry and
//! no control anyone could click.
//!
//! The radio is also the *point of the feature*. `MeasureState::group` had
//! existed since the Phase 7 salvage, documented as the group picker, and
//! **nothing in the build ever wrote to it**: a second group could be created
//! and joined by nothing. A row with a radio on it is the only evidence that
//! the group is reachable rather than merely recorded.
//!
//! # ★ And the round trip, added 2026-08-19 with the verbs it exercises
//!
//! Create, **rename**, **delete** — ending with the list exactly as long as it
//! started. Each verb alone would show only that its arm exists; together they
//! show that the panel's three controls act on **the same group**.
//!
//! That is the failure a per-row surface actually has and no single-verb
//! assertion can see: a rename field bound to the *selected* row while Delete
//! acts on the *authoring* row would let every individual step report success
//! while the operator renamed one group and deleted another. The two are
//! deliberately different things in this panel — the radio chooses where the
//! next dimension goes, the name chooses whose settings are on screen — which
//! is exactly what makes confusing them possible.
//!
//! **The populated-group branch is not covered**, and is named rather than left
//! unstated: deleting a group that still has members asks a destination
//! question, and reaching it needs a fixture whose dimensions already live in a
//! named group.

use crate::checks::driving::{
    SHELL_DIAG_ENV, TAB_EVENT, declared, declared_names, declared_or_in_overflow, list, live_names,
    shell_trace,
};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::geom::LRect;
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

/// The mode whose tab list carries Measure.
const MODE: &str = "review";
/// The panel body's own region.
///
/// ★ Renamed from `dialog:dimension-groups` when the surface moved into the
/// dock. The prefix is load-bearing in the trace — a reader scanning for what
/// drew has to be able to tell a floating window from a docked body — so it
/// moved with the surface rather than being kept for the harness's convenience.
const PANEL: &str = "panel:dimension-groups";
/// The prefix of the foldable sections' heading regions.
const HEADING: &str = "dimension-groups.heading.";
/// The new-group name field.
const NAME_FIELD: &str = "dimension-groups.new_name";
/// The Add button.
const ADD: &str = "dimension-groups.add";
/// The prefix of the per-group authoring radios.
const DRAW_INTO: &str = "dimension-groups.draw_into.";
/// The appearance-defaults block, which proves the lower half drew at all.
const APPEARANCE: &str = "dimension-groups.appearance";
/// The prefix of the per-row name regions — what selects a group for the lower
/// half of the panel.
const ROW: &str = "dimension-groups.row.";
/// The rename field.
const RENAME: &str = "dimension-groups.rename";
/// The Delete button.
const DELETE: &str = "dimension-groups.delete";
/// The label `vector_edit` traces when `rename_dimension_group` succeeded.
const RENAMED: &str = "rename-dimension-group";
/// The label it traces for `delete_dimension_group_with`.
const DELETED: &str = "delete-dimension-group";
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
/// One more letter, appended when renaming.
///
/// ★ **Appended rather than retyped**, and that is what makes the rename
/// observable without reading the text back. The field opens seeded with the
/// group's current name, and the Rename button is drawn **only while the draft
/// differs from it** — so a single extra letter producing a commit is itself
/// evidence the field was seeded from the document rather than opening empty.
///
/// A rename box that opened empty would let an operator wipe a name by pressing
/// a button they took for a no-op, and nothing else in this check would notice.
const RENAME_KEY: u16 = vk::L;

/// Press one of the panel's fold headings, and wait for the frame that answers.
///
/// # ★ Why this exists, and why it returns `Option` rather than failing
///
/// Five of the panel's six sections start shut, so the regions this check aims
/// at — the name field, the Add button, Rename, Delete, the appearance block —
/// **do not exist** until their heading is pressed. That is the surface the
/// operator asked for (*"each section should be able to fold up like the
/// settings one"*) and it makes this check the only thing in the project that
/// proves the folds open at all.
///
/// It returns the heading's own rect rather than a bare `bool` so a caller can
/// press it a second time to shut the section again. Shutting matters more than
/// it looks: the panel lives in a dock column, `declared` retires a region that
/// stops being published, and a section left open pushes everything below it
/// down the scroll region — where the next `declared_center` would aim at a
/// point the operator cannot see. Closing what has been inspected is how this
/// check stays inside the visible column without a single hard-coded height.
///
/// `None` means the heading is not on screen at all, which is a real failure
/// and is reported by the caller in its own words — a fold whose heading cannot
/// be found is a section that is unreachable, not a section that is shut.
fn fold(session: &Session, driver: &Driver, ui_rect: &str, key: &str) -> Result<Option<LRect>> {
    let trace = session.trace()?;
    let Some(heading) = declared(&trace, ui_rect, &format!("{HEADING}{key}")) else {
        return Ok(None);
    };
    driver.click_at(session.frame()?.declared_center(heading))?;
    session.settle(16);
    Ok(Some(heading))
}

/// See the module documentation.
pub struct DimensionGroupsPanelMakesAGroup;

impl Check for DimensionGroupsPanelMakesAGroup {
    fn name(&self) -> &'static str {
        "dimension_groups_panel_makes_a_group"
    }

    fn defect(&self) -> &'static str {
        "Measure > Dimension groups is drawn and does nothing — or it opens a panel \
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

    // --- 3: show the panel -------------------------------------------------
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
    if declared(&trace, ui_rect, PANEL).is_none() {
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
                 unimplemented line, and no `{PANEL}` region appeared. The arm ran and the \
                 panel did not draw — and because the arm is a TOGGLE, the likeliest cause \
                 is that the panel was already the active tab and the click shut it."
            )
        }));
    }
    report.note("the Dimension-groups panel drew after Measure > Dimension groups");

    // --- 4: it drew a list, and the list has the default group in it -------
    // `live_names`, not `declared_names` — the three counts in this check
    // compare row populations across an edit, and the `ui-rect` channel is a
    // change log in which a deleted row's last rect stands for ever. See
    // `driving::live_names`, whose doc records the false failure this check
    // produced before the distinction existed.
    let before = live_names(&trace, ui_rect, DRAW_INTO);
    if before.is_empty() {
        return Ok(Some(format!(
            "the panel drew and declared no `{DRAW_INTO}*` region, so it listed no group \
             rows at all. Every document has a default group, so an empty list is the panel \
             failing to read `dimension_model()` rather than a document with no groups."
        )));
    }
    // ★ The appearance defaults are behind a fold that starts SHUT, so this
    // step is two clicks and an assertion between them: open it, prove the
    // longest section in the panel drew, shut it again so it cannot push the
    // controls below out of the visible column.
    //
    // Shutting it is also the second half of the operator's ask. A fold that
    // opens and will not close is a heading with a decoration on it.
    let Some(appearance_heading) = fold(&session, &driver, ui_rect, "appearance")? else {
        return Ok(Some(format!(
            "the panel listed {} group row(s) and declared no `{HEADING}appearance` region, \
             so the appearance defaults have no heading to press and are unreachable. \
             Headings declared: {}.",
            before.len(),
            list(&declared_names(&session.trace()?, ui_rect, HEADING))
        )));
    };
    if declared(&session.trace()?, ui_rect, APPEARANCE).is_none() {
        return Ok(Some(format!(
            "the Appearance heading was pressed and no `{APPEARANCE}` region followed, so the \
             fold did not open — or opened onto nothing. {} group row(s) were listed. A list \
             with no settings reachable under it is a picker, not a manager.",
            before.len()
        )));
    }
    driver.click_at(session.frame()?.declared_center(appearance_heading))?;
    session.settle(16);
    if declared(&session.trace()?, ui_rect, APPEARANCE).is_some() {
        return Ok(Some(format!(
            "the Appearance heading was pressed a second time and `{APPEARANCE}` is still \
             declared, so the fold opens and does not shut. `declared` retires a region that \
             stops being published, so this is a live region rather than a fossil — the \
             section is genuinely still drawing."
        )));
    }
    report.note(format!(
        "{} group row(s) listed, and the appearance fold opened and shut again: {}",
        before.len(),
        list(&before)
    ));

    // --- 5: open the Add fold, and type a name -----------------------------
    //
    // The Add controls sit directly under the LIST and above the selected
    // group's settings, which is where the surface's own header argues they
    // belong: adding is an action on the list, not on the selected group. That
    // placement is also what keeps this step inside the visible column — the
    // fold is three lines below a list with one row in it.
    if fold(&session, &driver, ui_rect, "add")?.is_none() {
        return Ok(Some(format!(
            "the panel declares no `{HEADING}add` region, so there is no way to open the \
             new-group controls and a document is stuck with the groups it already has. \
             Headings declared: {}.",
            list(&declared_names(&session.trace()?, ui_rect, HEADING))
        )));
    }
    let trace = session.trace()?;
    let field = declared(&trace, ui_rect, NAME_FIELD).ok_or_else(|| {
        Error::new(format!(
            "the Add-a-group fold was opened and declared no `{NAME_FIELD}` region, so there \
             is nothing to type a group name into and the Add button can never leave its \
             greyed state."
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
        .ok_or_else(|| Error::new(format!("the panel declared no `{ADD}` region.")))?;
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
    let after = live_names(&session.trace()?, ui_rect, DRAW_INTO);
    if after.len() <= before.len() {
        return Ok(Some(format!(
            "★ the group was WRITTEN and did not come back. `{APPLIED}` was traced, so \
             `EditSession::add_dimension_group` ran and the undo log has an entry — and the \
             panel still lists {} row(s), the same as before. A group the model does not \
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

    // --- 8: rename it, and delete it ---------------------------------------
    //
    // ★ A ROUND TRIP, deliberately: create, rename, delete, ending with the
    // list exactly as long as it started. Each verb alone would only show that
    // its arm exists; together they show that the panel's three controls act
    // on the SAME group — which is the failure a per-row surface actually has,
    // and which no single-verb assertion can see.
    //
    // The new group is found by set difference rather than by position. A check
    // that assumed "the last row" would be asserting something about the
    // model's ordering that nothing promises.
    let Some(added) = after.iter().find(|name| !before.contains(name)) else {
        return Ok(Some(format!(
            "the row count grew and no NEW `{DRAW_INTO}*` name appeared, which the model \
             cannot produce — before {}, after {}.",
            list(&before),
            list(&after)
        )));
    };
    let Some(id) = added.strip_prefix(DRAW_INTO) else {
        return Err(Error::new(format!(
            "`{added}` does not begin with `{DRAW_INTO}`, so the group id cannot be read out \
             of it and the rename and delete steps have nothing to aim at."
        )));
    };

    let trace = session.trace()?;
    let Some(row) = declared(&trace, ui_rect, &format!("{ROW}{id}")) else {
        return Ok(Some(format!(
            "the new group has an authoring radio and NO `{ROW}{id}` region, so its name is \
             not clickable and the lower half of the panel can never be pointed at it. \
             Every setting below the list would be unreachable for this group."
        )));
    };
    driver.click_at(session.frame()?.declared_center(row))?;
    session.settle(14);

    // --- rename ------------------------------------------------------------
    //
    // Behind the one fold that is shut for a SAFETY reason rather than a length
    // one: it carries the only two destructive verbs on the panel, and R9
    // forbids greying a control that is genuinely available to make it feel
    // safer. A fold is the honest equivalent — and it is this click.
    if fold(&session, &driver, ui_rect, "identity")?.is_none() {
        return Ok(Some(format!(
            "the panel declares no `{HEADING}identity` region, so rename and delete have no \
             heading to press and a group created by mistake is permanent. Headings \
             declared: {}.",
            list(&declared_names(&session.trace()?, ui_rect, HEADING))
        )));
    }
    let trace = session.trace()?;
    let Some(field) = declared(&trace, ui_rect, RENAME) else {
        return Ok(Some(format!(
            "the selected group's settings declare no `{RENAME}` region, so a mistyped group \
             name is permanent for the life of the document."
        )));
    };
    driver.click_at(session.frame()?.declared_center(field))?;
    session.settle(10);
    driver.press(RENAME_KEY)?;
    session.settle(8);
    // Enter rather than the button: the button is drawn beside the field only
    // while there is something to do, so it has no stable region to aim at —
    // and the row handles Enter for exactly the reason a check needs it, which
    // is that a name is a thing people type and then press Enter on.
    driver.press(vk::ENTER)?;
    session.settle(18);

    let trace = session.trace()?;
    if trace.last(RENAMED).is_none() {
        return Ok(Some(format!(
            "a letter was typed into `{RENAME}` and Enter was pressed, and no `{RENAMED}` \
             line followed. Either the field takes no keystrokes, or Enter is not a commit — \
             and because the button is drawn only while the draft differs, a field that never \
             received the letter shows no button either, so both failures look identical from \
             outside."
        )));
    }
    report.note("the group was renamed, through the document");

    // --- delete ------------------------------------------------------------
    //
    // The group is EMPTY — nothing has been drawn into it — so this exercises
    // the straight path rather than the destination question a populated group
    // asks. **That branch is not covered here**, and it is named rather than
    // left as an unstated gap: it needs a fixture whose dimensions already live
    // in a named group, which this one does not have.
    let trace = session.trace()?;
    let Some(delete) = declared(&trace, ui_rect, DELETE) else {
        return Ok(Some(format!(
            "the selected group declares no `{DELETE}` region, so a group created by mistake \
             stays in the picker for the life of the document."
        )));
    };
    driver.click_at(session.frame()?.declared_center(delete))?;
    session.settle(20);

    let trace = session.trace()?;
    if trace.last(DELETED).is_none() {
        return Ok(Some(format!(
            "Delete was pressed on an EMPTY group and no `{DELETED}` line followed. An empty \
             group is the case the engine accepts without a policy, so this is the shell half \
             and not a refusal."
        )));
    }
    let finally = live_names(&session.trace()?, ui_rect, DRAW_INTO);
    if finally.len() != before.len() {
        return Ok(Some(format!(
            "★ the round trip did not close: {} row(s) before, {} after the delete. Create, \
             rename and delete must act on the same group, and a mismatch here means one of \
             the three pointed somewhere else — which every individual step would still \
             report as a success. Rows: {}.",
            before.len(),
            finally.len(),
            list(&finally)
        )));
    }
    report.note("created, renamed and deleted the same group — the list is back where it started");
    Ok(None)
}
