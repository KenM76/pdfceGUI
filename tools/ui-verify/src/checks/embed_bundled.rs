//! `embedding_works_with_no_font_folder_at_all` — **pdfce's own fourteen faces
//! answer when nothing of the operator's can.**
//!
//! # What this is for
//!
//! `OPERATOR_REQUESTS.md` **O47** asked the operator whether pdfce should embed
//! the standard-14 faces it ships when none of their folders holds the font a
//! document names. He answered *"yes"* on 2026-08-28. This is the check that
//! keeps that answer working.
//!
//! ## ★★★ Why it is a SEPARATE check and not a parameter of the other one
//!
//! Because it asserts the opposite premise. `embedding_fonts_puts_a_program_in_
//! the_document` supplies a real font folder and would pass identically with the
//! bundled rung ripped out — the folder answers first, every time, by design.
//! Only a run with **no folder at all** can distinguish *"pdfce ships faces and
//! will use them"* from *"pdfce ships faces and never reaches them"*.
//!
//! ★★ That is also why it is worth the extra process launch. Two checks over one
//! feature, differing in one environment variable, is the shape that catches a
//! rung being unreachable — which is the same failure the whole resolver had on
//! the day it was written, when only the exact rung worked and every test
//! registered a name and then asked for it.
//!
//! ## ★★ The oracle is `substituted=true`, and it is the point of the row
//!
//! The operator's *"yes"* came with a condition: **disclosed loudly**. A build
//! that embedded a bundled face and reported it as an ordinary match would
//! satisfy the letter of the request and lose the half he can act on — the
//! document goes out with pdfce's stand-in in it, and nothing on the canvas says
//! which face went in.
//!
//! `substituted=` is `EmbedPlan::substitutes_any`, computed by the engine from
//! `FontMatch::is_substitute` on every target. It is `true` here **only if** the
//! shell reported the rung honestly on the way in — a shell that claimed `Exact`
//! for a bundled face would produce a green `missing_after=0` and a false
//! `substituted=false`, so this one field checks the disclosure and the
//! correctness together.
//!
//! ⇒ Reporting a bundled donor as `Exact` would also walk it past the engine's
//! **symbolic-font guard**, which turns on exactly that predicate. The
//! disclosure and the guard are the same flag, which is why understating it is a
//! correctness defect rather than a cosmetic one.
//!
//! ## What this does NOT establish
//!
//! **Which face was substituted, or that it looks right.** pdfce's standard-14
//! substitutes are the engine's to choose and its tests cover the choice. This
//! establishes that the shell reaches them, at the right moment, and says so.

use crate::checks::driving::{
    SHELL_DIAG_ENV, declared, declared_names, frame_of, list, stable_rect,
};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The command, invoked through the harness seam.
const INVOKE: &str = "mode.edit,tools.embed_fonts";
/// The variable this check deliberately does **not** set.
///
/// ★ Named as a constant it never uses, so a reader grepping for the font-dir
/// seam finds this file and its reason rather than concluding it was forgotten.
/// The whole point of this run is that no folder is configured.
#[allow(dead_code)]
const DELIBERATELY_UNSET: &str = "PDFCE_DIAG_FONT_DIR";
/// The window body's region.
const BODY: &str = "embed.body";
/// The Embed button's region.
const BUTTON: &str = "embed.commit";
/// The line the window writes when it opens, carrying its plan's counts.
const OPENED: &str = "embed-fonts-opened";
/// The line the apply arm writes when the engine has embedded.
const APPLIED: &str = "embed-fonts-applied";
/// The line the dispatcher writes when there is nothing to open.
const DECLINED: &str = "embed-fonts-declined";

/// See the module documentation.
pub struct EmbeddingWorksWithNoFontFolderAtAll;

impl Check for EmbeddingWorksWithNoFontFolderAtAll {
    fn name(&self) -> &'static str {
        "embedding_works_with_no_font_folder_at_all"
    }

    fn defect(&self) -> &'static str {
        "with no font folder configured, Embed fonts refuses everything — pdfce ships the \
         fourteen standard faces and cannot reach them, so an operator who has not set up a \
         folder is told to go and find one for a font pdfce is already carrying"
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
            "input is disabled (--no-input), and this check's subject is a click on the Embed \
             button.",
        ));
    }
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new(
            "no --pdf. This check needs a document naming a STANDARD-14 font it does not carry \
             — Helvetica, Times or Courier — which is what every CAD exporter writes.",
        )
    })?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("embed-bundled.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCE_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    // ★★★ No `PDFCE_DIAG_FONT_DIR`. That absence IS the check.
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {} with NO font folder configured — the point of this run",
        exe.display(),
        session.pid()
    ));
    session.settle(60);
    let driver = Driver::new(session.window());

    let trace = session.trace()?;
    if let Some(declined) = trace.events(DECLINED).last() {
        return Ok(Some(format!(
            "★★★ EMBED FONTS DECLINED WITH NOTHING TO DO: `{}`.\n\
             **This is the exact state O47 was answered to change.** The document names a font \
             it does not carry and pdfce ships fourteen faces, so a decline means the bundled \
             rung was not reached: `Library::scan_with(folders, true)` in `dialogs::embed`, and \
             `allow_bundled` reaching `resolve_for_embedding`. If this run's --pdf names no \
             standard-14 font, the check is aimed wrong rather than the program. Trace: {}.",
            declined.raw,
            session.trace_path().display()
        )));
    }
    if declared(&trace, ui_rect, BODY).is_none() {
        return Ok(Some(format!(
            "EMBED FONTS WAS INVOKED AND NO WINDOW APPEARED: no `{BODY}` region and no \
             `{DECLINED}` line. Regions beginning `embed`: {}. Trace: {}.",
            list(&declared_names(&trace, ui_rect, "embed")),
            session.trace_path().display()
        )));
    }
    let Some(opened) = trace.events(OPENED).last() else {
        return Ok(Some(format!(
            "the window drew and published no `{OPENED}` line, so this check cannot tell a \
             correctly greyed button from a broken one. That line is what makes the difference \
             readable; see `dialogs::embed::open`. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if opened.get("targets") == Some("0") {
        return Ok(Some(format!(
            "★★★ THE WINDOW OPENED WITH NOTHING TO EMBED: `{}`.\n\
             With no folder configured, `targets` counts exactly what pdfce can supply from its \
             OWN faces — so zero is the bundled rung not firing. Read `supplied=`: zero means \
             `Library::donor_for` answered nothing for a standard-14 name, which is \
             `allow_bundled` not reaching `resolve_for_embedding`. Trace: {}.",
            opened.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★ the window found donors with no folder set: `{}`",
        opened.raw
    ));

    let Some(button) = stable_rect(&session, ui_rect, BUTTON, 8)? else {
        return Ok(Some(format!(
            "the window drew its body and declared no `{BUTTON}` region. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let trace = session.trace()?;
    let frame = frame_of(&session, &trace, ui_rect, BUTTON)?;
    driver.click_at(frame.declared_center(button))?;
    session.settle(60);

    let trace = session.trace()?;
    let Some(applied) = trace.events(APPLIED).last() else {
        return Ok(Some(format!(
            "the Embed button was clicked and nothing reached the document: no `{APPLIED}` \
             line. The window had targets, so this is the action or its apply arm rather than \
             the resolver. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("★ the engine embedded: `{}`", applied.raw));

    // --- the oracle ---------------------------------------------------------
    let embedded: usize = applied
        .get("embedded")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    if embedded == 0 {
        return Ok(Some(format!(
            "the embed ran and embedded nothing: `{}`. Trace: {}.",
            applied.raw,
            session.trace_path().display()
        )));
    }
    if applied.get("substituted") != Some("true") {
        return Ok(Some(format!(
            "★★★ {embedded} FONT(S) WERE EMBEDDED FROM PDFCE'S OWN FACES AND REPORTED AS NOT \
             SUBSTITUTED: `{}`.\n\
             With no folder configured every donor is a bundled face, so `substituted=false` \
             means the shell told the engine `FontMatch::Exact` for one. That is not a wording \
             defect. `is_substitute` is the predicate the engine's SYMBOLIC-FONT GUARD turns \
             on, so understating the rung disables that guard from the outside — and it loses \
             the disclosure the operator's *\"yes\"* to O47 was conditional on. See `rung()` and \
             the `Match::Bundled` arm in `dialogs::embed`. Trace: {}.",
            applied.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★★ {embedded} font(s) embedded from pdfce's own faces, with no folder configured, and \
         every one disclosed as a substitute — still missing afterwards: {}",
        applied.get("missing_after").unwrap_or("?")
    ));
    Ok(None)
}
