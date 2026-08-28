//! `the_format_tab_offers_font_controls_for_swept_text` — **the ribbon route
//! to a restyle, and the sentence that tells an operator how to reach it.**
//!
//! # What this is for, and how it differs from `restyle_text`
//!
//! `restyle_text` drives the **panel** route: sweep, find the *This text*
//! section in the Properties dock, press its Bold. This drives the **ribbon**
//! route, which shipped on 2026-08-27 as `RIBBON_IA.md` §5.8's Font group, and
//! it also drives the half of O37 that is not a capability at all.
//!
//! O37 shipped with an admission written into its own row:
//!
//! > You are in Edit mode, so a drag with the Select tool draws a marquee round
//! > objects. Press **T** first — that arms the text tool — then sweep across
//! > the words. ★ **That is a discoverability gap and it is ours, not a
//! > limitation. Nothing on screen tells you to press T.**
//!
//! Three surfaces now do. This check asserts two of them in the state an
//! operator is actually in when they need them, which is the state **before**
//! anything is swept — and that ordering is the whole design of the check.
//!
//! ## ★★★ The two phases, and why the first one has to come first
//!
//! | phase | the operator's state | what must be true |
//! |---|---|---|
//! | 1 | clicked a piece of text with the Select tool; **nothing swept** | the Format tab appears, it carries a **Font** group, and the Properties panel says how to get an operand |
//! | 2 | pressed `T`, swept the words | the ribbon's **Bold** commits a restyle to the document |
//!
//! Phase 1 cannot be reached after phase 2, because a sweep is not undone by
//! clicking again — so a check that swept first would have destroyed the state
//! it is meant to observe. That is not a harness convenience; it is the
//! operator's own sequence. They click the thing they want to change *before*
//! they know a sweep is needed, which is precisely why the gap existed.
//!
//! ## ★★ What phase 1 can and cannot see, said plainly
//!
//! The ribbon publishes `ribbon.item.<id>` for **every** command control,
//! enabled or greyed — deliberately, and `egui_shell::ribbon::control`'s own
//! note says why: *"the question a consumer asks is where is this control, and
//! a control that is greyed is still a control that was drawn somewhere."*
//!
//! So a region tells this check the control is **on screen**. It does not tell
//! it the control is greyed. That is a real limit and it is not papered over:
//! the greying is asserted by
//! `app::conditions::tests::the_font_groups_visibility_follows_the_mode_and_its_enablement_the_sweep`,
//! which reads the registered command's own predicate against the published
//! conditions — the join, not either half. What *this* check adds, and what no
//! unit test can, is that the controls are **drawn at all**, on a real ribbon,
//! in a window, at a real width, on the tab that really appeared.
//!
//! ★ And the appearing is itself under test. The Format tab is contextual: its
//! `visible_when` moved from `selection.any` to `selection.formattable` when
//! the Font group landed, and a build that missed that change shows **no tab**
//! after a sweep and therefore no Font group — a whole feature with no surface,
//! which is exactly the shape of defect this project exists to catch.
//!
//! # The oracle
//!
//! Phase 1: the regions `ribbon.tab.format`, `ribbon.group.format.font`, the
//! five `ribbon.item.format.*`, and `properties.text.route`.
//!
//! Phase 2: `text-style-applied … applied=N` **and** the `format-text` label
//! `vector_edit` writes when the edit reached the engine — the same two-line
//! oracle `restyle_text` uses, for its reason: the first without the second is
//! a module that decided to act and whose action never landed.

use crate::checks::driving::{
    self, SHELL_DIAG_ENV, click_mode_segment, declared, declared_names, list,
};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The mode whose canvas may select page content, and the only mode the Font
/// group is drawn in — `mode.edit_content` is its `visible_when`.
const MODE: &str = "edit";
/// The contextual Format tab's strip region.
const FORMAT_TAB: &str = "ribbon.tab.format";
/// The Font group's captioned band.
const FONT_GROUP: &str = "ribbon.group.format.font";
/// The Bold control, which is the one this check presses.
const BOLD_ITEM: &str = "ribbon.item.format.bold";
/// Every control the Font group must draw, in manifest order.
///
/// ★ Asserted as a **list**, not as "Bold is there". Three of the five are
/// `Item::Custom`s drawn by `app::fontband`, and a custom item that the
/// manifest names and no renderer matches draws **nothing** while the shell
/// reserves its space — which is the defect `COLOUR_SWATCH` shipped with for
/// the whole of v0.1.0, invisible because a gap in a band looks like a gap in a
/// band. Only naming all five catches it.
const FONT_ITEMS: [&str; 5] = [
    "ribbon.item.format.font",
    "ribbon.item.format.font_size",
    "ribbon.item.format.bold",
    "ribbon.item.format.italic",
    "ribbon.item.format.font_colour",
];
/// The Properties panel's sentence for a text object with nothing swept.
const ROUTE_REGION: &str = "properties.text.route";
/// The `text-style-applied` summary line.
const STYLE_EVENT: &str = "text-style-applied";
/// The `text-style-declined` line.
const DECLINED_EVENT: &str = "text-style-declined";
/// The label `vector_edit` writes when the restyle reached the engine.
const APPLIED: &str = "format-text";
/// The sweep's own oracle.
const SELECTION_EVENT: &str = "canvas-text-selection";
/// How far to sweep along the baseline, in PDF points.
const SWEEP_PT: f64 = 60.0;
/// `T` as a Windows virtual key — the text-sweep tool.
///
/// ★ Pressed only in **phase 2**, and the fact that phase 1 works without it is
/// the point of the check: the whole complaint is that an operator does not
/// know to press it, so the surfaces that tell them must be observed in the
/// state where they have not.
const VK_T: u16 = 0x54;

/// See the module documentation.
pub struct TheFormatTabOffersFontControlsForSweptText;

impl Check for TheFormatTabOffersFontControlsForSweptText {
    fn name(&self) -> &'static str {
        "the_format_tab_offers_font_controls_for_swept_text"
    }

    fn defect(&self) -> &'static str {
        "an operator who wants to change how text looks has to already know to press T and \
         sweep — the ribbon offers no font controls, or offers them on a tab that never appears \
         for a text selection, so a capability the engine has and the panel exposes is \
         unreachable from the surface an operator looks at first"
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

/// Poll until the restyle reports one way or the other, and answer how long it
/// took.
///
/// A bounded poll rather than a fixed sleep, for `restyle_text`'s reason: a
/// restyle re-resolves its pin from a fresh provenance extraction per run, so a
/// sweep across a title-block label is a dozen extractions, and a fixed sleep
/// long enough for the worst case makes every run slow while a pleasant one
/// reads the trace mid-gesture and reports "nothing happened" about a gesture
/// that is still running.
fn wait_for_verdict(session: &Session) -> Result<u128> {
    const CEILING_MS: u128 = 20_000;
    let started = std::time::Instant::now();
    loop {
        session.settle(4);
        let trace = session.trace()?;
        if trace.last(STYLE_EVENT).is_some() || trace.last(DECLINED_EVENT).is_some() {
            return Ok(started.elapsed().as_millis());
        }
        if started.elapsed().as_millis() > CEILING_MS {
            return Ok(started.elapsed().as_millis());
        }
    }
}

#[allow(clippy::too_many_lines)]
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
        .ok_or_else(|| Error::new("no --pdf. This check needs a page carrying real text."))?;
    let target = ctx.target.ok_or_else(|| {
        Error::new(
            "no --doc-point. Pass PAGE,X,Y in PDF user space naming the LEFT END of a piece of \
             text's baseline. `pdfce-cli extract-text --json` gives the first glyph's x and y \
             of every run; use those. A point on blank paper selects no object in phase 1 and \
             sweeps nothing in phase 2, and the check would report both surfaces as broken.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a page object, clicks a ribbon \
             tab, sweeps the pointer and presses a button, and none of that can be simulated \
             from the trace.",
        ));
    }
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;
    let page: PageGeometry = match ctx.page_size {
        Some((w, h)) => PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&pdf).ok_or_else(|| {
            Error::new(format!(
                "cannot read a page size from {}. Pass --page-size WxH.",
                pdf.display()
            ))
        })?,
    };

    let mut spec = LaunchSpec::new(&exe, ctx.out("font-group.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    session.settle(40);
    let driver = Driver::new(session.window());

    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(20);

    // =======================================================================
    // PHASE 1 — click the text as an OBJECT, and nothing else.
    //
    // This is the state O37 admitted to: the operator has clicked the thing
    // they want to change, and the program has to tell them what to do next.
    // =======================================================================
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, vocab, page, target.page)?;
    let frame = session.frame()?;
    // ★ A little way ALONG the baseline and a little above it, not at the
    // baseline's left end. `--doc-point` names the first glyph's origin, which
    // is the bottom-left corner of the first character — a point on the very
    // edge of the ink and, on a six-point label, a click that can land in the
    // paper beside it. Two points in and two points up is inside the glyph box
    // of any text this fixture carries.
    let on_text = frame.to_screen(mapping.doc_to_window(DocPoint::new(
        target.page,
        target.x + 2.0,
        target.y + 2.0,
    ))?);
    driver.click_at(on_text)?;
    session.settle(24);

    let trace = session.trace()?;
    let Some(_tab) = declared(&trace, ui_rect, FORMAT_TAB) else {
        let shot = ctx.out("font_group.no-format-tab.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        return Ok(Some(format!(
            "★ CLICKING A PIECE OF TEXT RAISED NO FORMAT TAB: no `{FORMAT_TAB}` region.\n\
             Two candidates. (1) **The click selected nothing**, in which case the tab is right \
             to stay away and the `--doc-point` is not on ink — the screenshot beside this \
             report settles it, and this would be an aim problem rather than a defect. (2) \
             **`selection.formattable` is not published**, which is the condition the tab's \
             `visible_when` names; it is the union of the object selection and a live text \
             selection, and a build that spelled it either way round loses one of the tab's two \
             subjects. Tabs declared: {}. Trace: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab.")),
            session.trace_path().display()
        )));
    };
    report.note("★ clicking a piece of text raised the contextual Format tab");

    // ★★ The Properties panel's sentence, read BEFORE the ribbon tab is
    // clicked, because clicking a tab does not disturb the panel and reading it
    // first keeps the two surfaces independent. This is the sentence the
    // module `panels::properties::text`'s header claimed existed for weeks and
    // did not: `section` returned before drawing anything whenever there was no
    // sweep, so the panel said nothing at all in exactly this state.
    if driving::declared(&trace, ui_rect, ROUTE_REGION).is_none() {
        let shot = ctx.out("font_group.no-route-sentence.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        return Ok(Some(format!(
            "★ A PIECE OF TEXT IS SELECTED AND THE PROPERTIES PANEL DOES NOT SAY HOW TO CHANGE \
             IT: no `{ROUTE_REGION}` region.\n\
             This is O37's own complaint. `panels::properties::text::route` draws the heading \
             and one sentence naming the Text tool and its chord whenever the selected object \
             is text and nothing is swept. Three candidates: the guard decided this object is \
             not text (`summary::object_kind`), more than one object is selected (the sentence \
             is deliberately single-selection only), or the section returned before drawing — \
             which is what it did for the whole of the feature's first week. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("★★ the Properties panel named the route: the Text tool, and the key that arms it");

    // The Format tab is contextual and is not the active tab merely by
    // appearing — the band draws whichever tab is active, so its contents are
    // unobservable until it is clicked. That is correct behaviour and not a
    // defect: a tab that stole focus on every selection would move the ribbon
    // under the operator's hand.
    let tab_rect = declared(&trace, ui_rect, FORMAT_TAB).expect("checked above");
    driver.click_at(session.frame()?.declared_center(tab_rect))?;
    session.settle(20);

    let trace = session.trace()?;
    if declared(&trace, ui_rect, FONT_GROUP).is_none() {
        let shot = ctx.out("font_group.no-group.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        return Ok(Some(format!(
            "★ THE FORMAT TAB CARRIES NO FONT GROUP: no `{FONT_GROUP}` region.\n\
             The most likely cause is `mode.edit_content`, which every one of the group's five \
             items names as its `visible_when` — a group all of whose items are hidden is not \
             drawn at all, by design, so an unpublished condition removes the whole band \
             silently. Groups declared: {}. Trace: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.group.")),
            session.trace_path().display()
        )));
    }
    let missing: Vec<&str> = FONT_ITEMS
        .into_iter()
        .filter(|name| {
            driving::declared_or_in_overflow(&session, &driver, ui_rect, name)
                .ok()
                .flatten()
                .is_none()
        })
        .collect();
    if !missing.is_empty() {
        let shot = ctx.out("font_group.missing-items.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        return Ok(Some(format!(
            "★ THE FONT GROUP IS DRAWN AND {} OF ITS FIVE CONTROLS ARE MISSING: {}.\n\
             Three of the five — the face chooser, the size field and the colour swatch — are \
             `Item::Custom`s drawn by `app::fontband`, and a custom kind the manifest names and \
             no renderer matches draws NOTHING while the shell reserves its space. That is a \
             gap in a band, which looks like a gap in a band; `manifest::COLOUR_SWATCH`'s own \
             note records it shipping that way for the whole of v0.1.0. The other two are \
             ordinary command items and their absence would mean the manifest lost them. Items \
             declared: {}. Trace: {}.",
            missing.len(),
            list_of(&missing),
            list(&declared_names(&trace, ui_rect, "ribbon.item.format.")),
            session.trace_path().display()
        )));
    }
    report.note("★★ the Font group drew all five controls with nothing swept — greyed, and there");

    // =======================================================================
    // PHASE 2 — arm the text tool, sweep, and press the ribbon's Bold.
    // =======================================================================
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, vocab, page, target.page)?;
    let frame = session.frame()?;
    let start =
        frame.to_screen(mapping.doc_to_window(DocPoint::new(target.page, target.x, target.y))?);
    let end = frame.to_screen(mapping.doc_to_window(DocPoint::new(
        target.page,
        target.x + SWEEP_PT,
        target.y,
    ))?);
    driver.press(VK_T)?;
    session.settle(16);
    driver.drag(start, end)?;
    session.settle(24);

    let trace = session.trace()?;
    let swept = trace
        .events(SELECTION_EVENT)
        .last()
        .and_then(|l| l.get("chars"))
        .and_then(|n| n.parse::<usize>().ok())
        .unwrap_or(0);
    if swept == 0 {
        return Err(Error::new(format!(
            "the drag from (page {}, {:.1}, {:.1}) rightwards {SWEEP_PT} pt selected no text, \
             so there was nothing for the Font group to act on. SKIPPED rather than failed: \
             this says the --doc-point is not on text, which is the harness's aim and not the \
             program's behaviour. Trace: {}.",
            target.page,
            target.x,
            target.y,
            session.trace_path().display()
        )));
    }
    report.note(format!("the sweep selected {swept} character(s)"));

    // ★ Re-find Bold rather than reusing the phase-1 rect. The sweep can change
    // the band: `selection.any` may have gone (a text press clears the object
    // selection on some routes), which changes nothing about the Font group but
    // could reflow the Selection group beside it — and a stale rect is how a
    // check clicks whatever happens to be at those coordinates now. `D:/dev/rag/egui/`
    // carries the general form of this: harness coordinates go stale when a
    // layout changes, and the symptom is a click that lands on the wrong thing
    // and a failure that blames the feature.
    let bold = driving::declared_or_in_overflow(&session, &driver, ui_rect, BOLD_ITEM)?
        .ok_or_else(|| {
            Error::new(format!(
                "no `{BOLD_ITEM}` region after the sweep, though it was there before it. \
                 SKIPPED rather than failed: a button that was never pressed proves nothing \
                 about pressing it. Trace: {}.",
                session.trace_path().display()
            ))
        })?;
    driver.click_at(session.frame()?.declared_center(bold))?;

    let waited = wait_for_verdict(&session)?;
    report.note(format!(
        "the restyle took {waited} ms of wall clock — one provenance extraction per run"
    ));

    let trace = session.trace()?;
    if let Some(declined) = trace.events(DECLINED_EVENT).last() {
        return Ok(Some(format!(
            "the ribbon's Bold was pressed and the restyle DECLINED: `{}`.\n\
             That is the program answering rather than staying silent, so the whole chain — \
             tab, group, custom renderer, token, dispatch arm, action — works, and the answer \
             is what is worth reading. A refusal means neither `set_synthetic` nor the \
             `set_font` retry took, which is either an unpinnable run or the engine naming a \
             real face that then failed glyph coverage (filed and confirmed, 2026-08-27). \
             Trace: {}.",
            declined.raw,
            session.trace_path().display()
        )));
    }
    let Some(applied) = trace.events(STYLE_EVENT).last() else {
        let shot = ctx.out("font_group.no-effect.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        return Ok(Some(format!(
            "★ THE RIBBON'S BOLD WAS PRESSED AND NOTHING HAPPENED AND NOTHING WAS DECLINED: no \
             `{STYLE_EVENT}` line and no `{DECLINED_EVENT}` line.\n\
             The panel's Bold and this one raise the same action through different routes, so \
             the candidates are the parts they do NOT share. (1) **The control was greyed** — \
             `selection.text` is its enable predicate and the sweep above set it, so this would \
             mean the condition and the sweep disagree. (2) **`dispatch::format` does not claim \
             the id**, in which case the token reached the dispatcher and fell through; \
             `handles` is the list. (3) **The click missed** — the region was declared, so the \
             screenshot beside this report settles it. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!(
        "★ the ribbon's Bold committed a restyle: `{}`",
        applied.raw
    ));

    let n: usize = applied
        .get("applied")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    if n == 0 {
        return Ok(Some(format!(
            "the restyle reported `applied=0`, so the arm decided to act and every run it \
             tried refused without saying so. That is worse than a decline: `{}`. Trace: {}.",
            applied.raw,
            session.trace_path().display()
        )));
    }
    if trace.last(APPLIED).is_none() {
        return Ok(Some(format!(
            "the ribbon's Bold computed `{}` and no `{APPLIED}` line followed, so the action \
             was raised and its apply arm never ran. Nothing reached the document, which from a \
             chair is indistinguishable from the button doing nothing. Trace: {}.",
            applied.raw,
            session.trace_path().display()
        )));
    }
    report.note(
        "★★★ the whole ribbon route works: a click on text raised the tab, the tab carried the \
         Font group, and its Bold restyled the swept text in the open document",
    );
    Ok(None)
}

/// Join borrowed names for a failure message.
///
/// `driving::list` takes owned `String`s and `driving::list_str` takes a slice
/// of `&str` — this is the latter, spelled locally only because the filter
/// above produces a `Vec<&str>` and handing it straight over reads better than
/// a collect-into-owned at the call site.
fn list_of(names: &[&str]) -> String {
    driving::list_str(names)
}
