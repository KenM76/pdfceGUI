//! `a_marquee_over_a_table_takes_its_text_as_well_as_its_lines` — **the
//! operator drew a box round a table and could not move it.**
//!
//! # The report
//!
//! Ken, 2026-09-01, on `TR-0461-1500-copy.pdf`:
//!
//! > *"I can't box select the tables in the left or right top corners using the
//! > mouse — it only picks up the lines of each table, so I can't drag the
//! > entire thing and move it somewhere else, or cut/copy and paste it
//! > elsewhere."*
//!
//! ## ★★★ What "only the lines" would mean, and why it needs measuring
//!
//! A CAD-exported table is two kinds of object drawn in one place: **paths**
//! (the rules and the border) and **text** (every cell's contents). They are
//! separate objects in the content stream and nothing in the file says they
//! belong together.
//!
//! So *"it only picks up the lines"* has three candidate causes and they want
//! opposite fixes:
//!
//! | | what would be wrong | how this check tells |
//! |---|---|---|
//! | the marquee excludes **text objects** | the hit test, or a filter above it | the selection has paths and no text |
//! | the marquee is **`Enclosed`** and the table touches the page edge, so it cannot be surrounded | the gesture, not the hit test | a marquee that fits INSIDE the page selects both |
//! | the selection is right and the **drag** refuses a mixed set | `canvas::moving`, not selection at all | both kinds selected, and no move line |
//!
//! ⇒ This check settles the first two by drawing a band that fits comfortably
//! inside the page around a table that does **not** touch the edge, and asking
//! what came back. A green result here moves the investigation to the third,
//! which is a different module and a different report.
//!
//! ★★ It is deliberately NOT a screenshot. Two objects selected and one object
//! selected draw the same blue outline round the same table; the distinguishing
//! fact is the census, and `canvas-selection` carries it.
//!
//! ## The fixture is the operator's own drawing
//!
//! Copied to scratch first. The check writes nothing — a marquee is a read —
//! but the application persists layout and recent-file state beside whatever it
//! opens, and this project's standing rule is that the suite's side effects do
//! not land in the operator's own folder.

use crate::checks::driving::{SHELL_DIAG_ENV, click_mode_segment, declared, declared_names, list};
use crate::checks::text_selection::aim;
use crate::checks::{Check, CheckContext};
use crate::coords::{DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Content selection needs Edit.
const MODE: &str = "edit";
/// ★★ Single page, fitted, BEFORE anything is aimed at.
///
/// The first run of this check drove a band to screen y=2 — above the canvas
/// entirely — because the file is ten pages shown continuously and the view had
/// been scrolled by the layout it inherited. `aim` faithfully computed where the
/// table WOULD be and the drag went there, off the canvas, selecting nothing.
///
/// A region off the top of the canvas looks exactly like a hit test that
/// excluded everything, which is the fourth instance of that shape in this
/// harness. Fitting the page first makes the aim a statement about the document
/// rather than about the scroll position the run happened to start from.
const INVOKE: &str = "view.page_single,view.zoom_fit_page";
/// The selection census the marquee writes.
const SELECTION: &str = "canvas-selection"; // ui-text-exempt: a trace event name, never displayed
/// The page region, so a failure can say whether a sheet was drawn at all.
const PAGE_REGION: &str = "page"; // ui-text-exempt: a trace region name, never displayed

/// The operator's drawing.
const FIXTURE: &str = r"C:\Users\Ken\OneDrive\pdfTests\TR-0461-1500-copy.pdf";
/// Its page, in points — 1224 × 792, measured off a 1.2× raster.
const FIXTURE_PAGE: PageGeometry = PageGeometry {
    width_pt: 1224.0,
    height_pt: 792.0,
};

/// The band, in page points, around the **INSPECTION STATUS** table.
///
/// ★★ Both corners are comfortably inside the sheet. That is the whole design
/// of this check: if a band that never approaches the page edge still returns
/// lines only, the fault is in the hit test. If it returns lines AND text, the
/// fault is that the operator's table touches the edge and an `Enclosed`
/// marquee cannot surround it — which is a gesture problem with a different
/// remedy, and this check will say so by passing.
const BAND_FROM: (f64, f64) = (14.0, 618.0);
const BAND_TO: (f64, f64) = (258.0, 784.0);

pub struct AMarqueeOverATableTakesItsTextAsWellAsItsLines;

impl Check for AMarqueeOverATableTakesItsTextAsWellAsItsLines {
    fn name(&self) -> &'static str {
        "a_marquee_over_a_table_takes_its_text_as_well_as_its_lines"
    }

    fn defect(&self) -> &'static str {
        "a box drawn round a table selects its rules and not its words, so dragging the \
         selection moves the grid and leaves the contents behind — and a cut takes half a table"
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
            "input is disabled (--no-input). The subject is a rubber-band drag.",
        ));
    }
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let source = std::path::PathBuf::from(FIXTURE);
    if !source.is_file() {
        return Err(Error::new(format!(
            "the operator's drawing is not at {FIXTURE}. This check is about a table in THAT \
             file; a substitute would be measuring a different document."
        )));
    }
    // A scratch copy. A marquee writes nothing, but the application persists
    // layout and recent-file state beside what it opens.
    let pdf = ctx.out("marquee-table.pdf");
    if let Some(dir) = pdf.parent() {
        std::fs::create_dir_all(dir).map_err(|e| Error::new(e.to_string()))?;
    }
    std::fs::copy(&source, &pdf).map_err(|e| Error::new(e.to_string()))?;

    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let page = crate::fixture::page_geometry(&pdf).unwrap_or(FIXTURE_PAGE);

    let mut spec = LaunchSpec::new(&exe, ctx.out("marquee-table.trace.txt"));
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
    report.note(format!(
        "launched {} as pid {} on a scratch copy of the operator's drawing",
        exe.display(),
        session.pid()
    ));
    session.settle(50);
    let driver = Driver::new(session.window());

    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    if declared(&session.trace()?, ui_rect, PAGE_REGION).is_none() {
        return Err(Error::new(format!(
            "no `{PAGE_REGION}` region, so no sheet is on screen. Regions beginning `page`: {}.",
            list(&declared_names(&session.trace()?, ui_rect, "page"))
        )));
    }

    // ★★★ **ARM THE SELECT TOOL FIRST**, and this line is the repair that made
    // this check able to run at all — 2026-09-02.
    //
    // Its first three runs reported "THE BAND SELECTED NOTHING AT ALL: no
    // `canvas-selection … via=pv.marquee` line", which reads as a hit test
    // that excluded everything. It was not: the trace carried **no**
    // `canvas-selection` line of any kind and only sixteen `canvas-pointer`
    // ones, so no rubber band had ever begun. The press belongs to whichever
    // tool is armed, and nothing here had armed one.
    //
    // ★★ Two of the three failures were previously written off as the harness
    // "driving the band above the canvas" — true of the first run, and it
    // masked this. A check that fails for two different reasons in two runs is
    // one whose SECOND diagnosis nobody looked for.
    //
    // A `false` return means the pointer route to the tool is unavailable, which
    // is a SKIP rather than a failure: the band could not be started, so nothing
    // about what a band selects was measured.
    if !crate::checks::driving::arm_select_from_ribbon(&session, &driver, ui_rect, report)? {
        return Err(Error::new(
            "the select tool could not be armed from the ribbon, so no rubber band could be \
             started. Nothing about what a band selects was measured.",
        ));
    }

    // --- the band ------------------------------------------------------------
    let from = aim(
        ctx,
        &session,
        page,
        DocPoint::new(0, BAND_FROM.0, BAND_FROM.1),
    )?;
    let to = aim(ctx, &session, page, DocPoint::new(0, BAND_TO.0, BAND_TO.1))?;
    report.note(format!(
        "band ({:.0}, {:.0}) → ({:.0}, {:.0}) in page points, both corners inside the sheet",
        BAND_FROM.0, BAND_FROM.1, BAND_TO.0, BAND_TO.1
    ));
    driver.drag(from, to)?;
    session.settle(60);

    let trace = session.trace()?;
    let Some(line) = trace
        .events(SELECTION)
        .filter(|l| l.get("via") == Some("pv.marquee"))
        .last()
    else {
        return Ok(Some(format!(
            "★★★ THE BAND SELECTED NOTHING AT ALL: no `{SELECTION} … via=pv.marquee` line \
             followed the drag. Either the rubber band did not start — a press on the page in \
             Edit mode with the Select tool armed should begin one — or it completed with an \
             empty hit set over a table that plainly has content in it. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let count: usize = line
        .get("sel")
        .and_then(|s| s.parse().ok())
        .unwrap_or_default();
    report.note(format!(
        "★ the band selected {count} object(s): `{}`",
        line.raw
    ));

    if count == 0 {
        return Ok(Some(format!(
            "★★★ THE BAND ENCLOSED THE TABLE AND SELECTED NOTHING: `{}`.\n\
             Both corners of the band are well inside the sheet, so this is not the operator's \
             page-edge case. Look at `hit_test_rect` and at whether `MarqueeMode::Enclosed` is \
             being asked about objects whose bounds are in a different space. Trace: {}.",
            line.raw,
            session.trace_path().display()
        )));
    }

    // ★★ The census is the oracle. A table's rules are ONE path object per line
    // in most CAD exports and its words are one text object per cell, so a band
    // over this table should return well into double figures. Two or three is
    // the signature of a hit test that found the border and nothing else.
    if count < 4 {
        return Ok(Some(format!(
            "★★ THE BAND TOOK ALMOST NOTHING: {count} object(s) from a table with six labelled \
             rows and a dozen rules — `{}`.\n\
             That is the operator's report reproduced: *\"it only picks up the lines of each \
             table\"*. A CAD table is paths AND text as separate objects; if only a handful \
             came back, one of those kinds is being excluded. Look first at whether a pick \
             filter is being applied to the marquee that is not applied to a click. Trace: {}.",
            line.raw,
            session.trace_path().display()
        )));
    }

    report.note(
        "★★★ …which is a whole table's worth of objects, so the band is not excluding a kind. \
         If the operator still cannot move it, the fault is downstream of selection — the drag, \
         or the fact that HIS tables touch the page edge and an enclosing band cannot surround \
         them.",
    );
    Ok(None)
}
