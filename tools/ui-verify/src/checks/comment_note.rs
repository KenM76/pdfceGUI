//! `a_note_can_be_written_onto_a_shape_that_exists` — **the Comments panel
//! stopped being a viewer, and this is what says it stayed that way.**
//!
//! # The gap this closes, in the engine's own words
//!
//! `pdfce-core`'s reply to this shell's blocker (`Pass 154.0`) lists what a
//! read-only comment list costs a reviewer, and none of the four is an edge
//! case:
//!
//! > comment a shape you just drew, comment a highlight you just swept, fix a
//! > typo in your own comment, answer someone else's.
//!
//! All four were impossible here until 2026-08-28, for a reason that is
//! structural rather than lazy: `MarkupOptions` is an **author-time** type, and
//! a cloud, a rectangle and an arrow are authored on mouse-release from
//! geometry alone. There is no text-entry moment in that gesture and there must
//! not be one — a dialog on every shape a reviewer draws is the interaction
//! nobody ships. So the conventional model needs a verb acting on an annotation
//! that **already exists**, and until `set_markup_note` there was none.
//!
//! # ★★★ Why this cannot be a unit test, in the specific
//!
//! The chain is five links and each has its own passing test:
//!
//! | # | Link | Its own test |
//! |---|---|---|
//! | 1 | the row decides the annotation is note-editable | yes (`note_controls` is a `match` over `CommentRow`) |
//! | 2 | *Add note* opens the draft | yes (`NoteDraft`'s suite) |
//! | 3 | Save raises `AnnotAction::SetNote` | no — that is a widget, and a widget's effect is observable only in a window |
//! | 4 | the apply arm resolves the author and calls the engine | partially |
//! | 5 | the engine writes `/Contents` and the panel reads it back | yes, on both sides separately |
//!
//! Link 3 is the one that has burned this project repeatedly, most recently
//! **on 2026-08-28 itself**: the O51 scale switches were written into an arm
//! that never runs, compiled, read correctly, and drew nothing, with every unit
//! test green. *"Nothing tested that the control is on screen."* This check is
//! that test for this control.
//!
//! # What it does
//!
//! `PDFCE_DIAG_INVOKE` supplies the two commands at launch rather than clicking
//! for them — Review mode, the Comments panel, and the rectangle tool — because
//! the subject here is the note, not the ribbon, and three extra clicks are
//! three extra ways for the check to fail at something it is not testing.
//! `markup_rectangle` is the check that proves those controls are reachable by
//! mouse.
//!
//! | Phase | Does | Expected |
//! |---|---|---|
//! | A | drag a rectangle on the page | `add-markup` — there is now one annotation |
//! | B | read the panel's census | `comments-panel listed=1 with_note=0` |
//! | C | click *Add note* | `comments.note_box` and `comments.note_save` appear |
//! | D | type four letters into the box | — |
//! | E | click *Save note* | `set-markup-note-applied … keys=…Contents…` |
//! | F | read the census again | `with_note=1` |
//!
//! # ★★ Phase F is the assertion that matters, and B is what makes it mean
//! anything
//!
//! `set-markup-note-applied` says the engine accepted the call. `with_note=1`
//! says **the panel read the words back out of the document**, which is the
//! only evidence that a reviewer would see anything. Both are needed and
//! neither is sufficient: a build that wrote `/Contents` into a session the
//! panel does not read from would pass the first and fail the second, and that
//! is not a hypothetical — the Comments panel reads `doc.session.view()`
//! precisely because reading the file on disk showed nothing until a save.
//!
//! Phase B pins `with_note=0` first, so F cannot be satisfied by a fixture that
//! arrived with a commented annotation already on it.
//!
//! # ★ The word typed is TAIL
//!
//! Four letters, all of them already in the closed `vk` list (`T`, `A`, `I`,
//! `L`), and a word a drafter would recognise in a failure message. The list is
//! deliberately closed and grown one key at a time with a reason; spelling a
//! word out of what is already there is cheaper than adding two constants to
//! type something prettier.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::text_selection::aim;
use crate::checks::{Check, CheckContext};
use crate::coords::{DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

/// The commands supplied at launch: Review mode, the Comments panel, the
/// rectangle tool.
///
/// ★ The panel opens **before** the shape is drawn, deliberately. A dock
/// appearing between the frame a check takes its coordinate mapping from and
/// the frame it clicks in changes the canvas width and puts the click somewhere
/// else — a fault this project has already recorded twice, and one that reads
/// as a broken feature rather than as a stale coordinate.
const INVOKE: &str = "mode.review,markup.comments,markup.rectangle";
/// The panel's per-frame census.
const CENSUS: &str = "comments-panel";
/// The line the apply arm writes once the engine has written the note.
///
/// `-applied`, per the convention this project adopted after making the
/// same-name mistake twice: `vector_edit` writes its own bare `set-markup-note`
/// line for the identical edit, and `.last()` on the bare name reads that one
/// and finds no keys.
const APPLIED: &str = "set-markup-note-applied";
/// The apply arm's line for the shape drawn in phase A.
const MARKUP_APPLIED: &str = "add-markup";
/// The region the first row's *Add note* control publishes.
const EDIT_REGION: &str = "comments.note_edit";
/// The region the open editor's text box publishes.
const BOX_REGION: &str = "comments.note_box";
/// The region the open editor's Save publishes.
const SAVE_REGION: &str = "comments.note_save";
/// The page's own region, so a failure can say whether a sheet was drawn.
const PAGE_REGION: &str = "page";
/// Where the rectangle is drawn, as fractions of the page — well inside the
/// sheet, away from a title block, and away from the edges.
const SHAPE: ((f64, f64), (f64, f64)) = ((0.35, 0.35), (0.55, 0.50));
/// `TAIL`, four keystrokes. See the module header.
const WORD: [u16; 4] = [vk::T, vk::A, vk::I, vk::L];

/// See the module documentation.
pub struct ANoteCanBeWrittenOntoAShape;

impl Check for ANoteCanBeWrittenOntoAShape {
    fn name(&self) -> &'static str {
        "a_note_can_be_written_onto_a_shape_that_exists"
    }

    fn defect(&self) -> &'static str {
        "the Comments panel lists annotations and cannot write one word onto any of them, so a \
         reviewer can draw a cloud round a mistake and has nowhere to say what is wrong with it \
         — a reviewer's main surface reduced to a viewer"
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

/// Run the sequence. `Err` is SKIP, `Ok(Some(_))` is FAIL, `Ok(None)` is a pass.
#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check draws a shape with a drag, clicks two \
             panel controls and types four letters. Reported as SKIPPED rather than passed: a \
             check that did not run has learned nothing.",
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

    let mut spec = LaunchSpec::new(&exe, ctx.out("comment-note.trace.txt"));
    spec.pdf = Some(pdf.clone());
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
        "launched {} as pid {} with PDFCE_DIAG_INVOKE={INVOKE}",
        exe.display(),
        session.pid()
    ));
    session.settle(40);
    let driver = Driver::new(session.window());

    let trace = session.trace()?;
    if declared(&trace, ui_rect, PAGE_REGION).is_none() {
        return Err(Error::new(format!(
            "the application declared no `{PAGE_REGION}` region, so no sheet is on screen and \
             there is nothing to draw on. Regions beginning `page`: {}.",
            list(&declared_names(&trace, ui_rect, "page"))
        )));
    }
    if trace.last(CENSUS).is_none() {
        return Err(Error::new(format!(
            "the Comments panel drew no `{CENSUS}` line after `{INVOKE}`, so the panel is not \
             open and every control this check aims at is absent for that reason rather than \
             for the one under test. SKIPPED, not failed."
        )));
    }

    // --- A: draw a rectangle ------------------------------------------------
    let corner = |f: (f64, f64)| DocPoint::new(0, f.0 * page.width_pt, f.1 * page.height_pt);
    let from = aim(ctx, &session, page, corner(SHAPE.0))?;
    let to = aim(ctx, &session, page, corner(SHAPE.1))?;
    driver.drag(from, to)?;
    session.settle(30);

    let trace = session.trace()?;
    if trace.events(MARKUP_APPLIED).count() == 0 {
        return Err(Error::new(format!(
            "the drag authored no annotation — no `{MARKUP_APPLIED}` line — so there is nothing \
             on the page to comment on. That is the step BEFORE the one under test; \
             `markup_rectangle_arms_from_the_ribbon` and `dragging_a_markup_moves_it` are the \
             checks that own it. SKIPPED. Trace: {}.",
            session.trace_path().display()
        )));
    }

    // --- B: the census before, which is what makes F mean anything ----------
    let Some(before) = trace.last(CENSUS) else {
        return Ok(Some(format!(
            "the panel stopped tracing `{CENSUS}` after a shape was drawn, so it is no longer \
             reading the document."
        )));
    };
    let listed = before.get_usize("listed").unwrap_or(0);
    let with_note_before = before.get_usize("with_note").unwrap_or(0);
    if listed == 0 {
        return Ok(Some(format!(
            "the engine authored the shape and the panel lists NOTHING: `{}`. The panel reads \
             `doc.session.view()` — the base revision with every unsaved edit applied — so a \
             zero here means it is reading the file on disk instead, and a reviewer would see \
             their own markup only after saving.",
            before.raw
        )));
    }
    if with_note_before != 0 {
        return Err(Error::new(format!(
            "the document already carries {with_note_before} commented annotation(s) before \
             this check writes one, so `with_note` cannot be used as the oracle. Use a fixture \
             with no commented markup, or the pass would be satisfied by the fixture. Line: \
             `{}`.",
            before.raw
        )));
    }
    report.note(format!(
        "the panel lists {listed} row(s), none carrying a note: `{}`",
        before.raw
    ));

    // --- C: open the editor -------------------------------------------------
    let edit = declared(&trace, ui_rect, EDIT_REGION).ok_or_else(|| {
        Error::new(format!(
            "no `{EDIT_REGION}` region. The panel publishes it for the FIRST row that offers a \
             note editor, so its absence means either no row offered one — a ce dimension and a \
             direct-dictionary annotation both decline, with a caption — or the control is not \
             drawn at all. Regions beginning `comments`: {}.",
            list(&declared_names(&trace, ui_rect, "comments"))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(edit))?;
    session.settle(16);

    let trace = session.trace()?;
    let Some(text_box) = declared(&trace, ui_rect, BOX_REGION) else {
        return Ok(Some(format!(
            "clicking *Add note* opened no editor: no `{BOX_REGION}` region on the next frame. \
             The button is drawn — its rect is what was clicked — so the failure is between the \
             press and the draft: `NoteDraft::begin`, or the `draft.editing(id, epoch)` test \
             that decides whether the editor draws. ★ Suspect the EPOCH first. The draft is \
             stamped `(annotation, edit epoch)` and the rectangle drawn in phase A bumped the \
             epoch; if anything re-seeds or re-syncs after the press, the editor closes on the \
             frame it opens. Regions beginning `comments`: {}.",
            list(&declared_names(&trace, ui_rect, "comments"))
        )));
    };

    // --- D: type into it ----------------------------------------------------
    //
    // Clicked first: a `TextEdit` takes keystrokes only when it has focus, and
    // egui gives focus on a click rather than on being drawn.
    driver.click_at(session.frame()?.declared_center(text_box))?;
    session.settle(8);
    for key in WORD {
        driver.press(key)?;
    }
    session.settle(10);

    // --- E: save ------------------------------------------------------------
    let trace = session.trace()?;
    let save = declared(&trace, ui_rect, SAVE_REGION).ok_or_else(|| {
        Error::new(format!(
            "the editor is open and publishes no `{SAVE_REGION}`, so there is no way to commit \
             what was typed. Regions beginning `comments`: {}.",
            list(&declared_names(&trace, ui_rect, "comments"))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(save))?;
    session.settle(24);

    let trace = session.trace()?;
    let Some(applied) = trace.events(APPLIED).last() else {
        return Ok(Some(format!(
            "SAVE WROTE NOTHING: no `{APPLIED}` line after clicking Save. Look, in order, at \
             (1) whether the click reached the button — a `set-markup-note-refused` line means \
             it did and the engine declined; (2) the panel's own `verb` slot, which is drained \
             after the scroll area closes and pushes `Action::Annot(SetNote)`; (3) the apply \
             arm. ★ A refusal by name is the likely one: `set_markup_note` refuses a WIDGET and \
             a ce dimension, and this check aims at whatever row the panel published first. \
             Trace: {}.",
            session.trace_path().display()
        )));
    };
    let keys = applied.get("keys").unwrap_or("");
    if !keys.contains("Contents") {
        return Ok(Some(format!(
            "the engine accepted the call and did NOT write the words: `{}`. `keys_written` \
             names what actually moved, and `/Contents` is not among them — which is the one \
             outcome that looks like success from every other angle.",
            applied.raw
        )));
    }
    report.note(format!("★ the engine wrote the note: `{}`", applied.raw));

    // --- F: the panel reads it back ----------------------------------------
    let Some(after) = trace.last(CENSUS) else {
        return Ok(Some(format!(
            "the panel stopped tracing `{CENSUS}` after the note was written."
        )));
    };
    let with_note_after = after.get_usize("with_note").unwrap_or(0);
    if with_note_after == 0 {
        return Ok(Some(format!(
            "THE WORDS WENT INTO THE DOCUMENT AND THE PANEL CANNOT SEE THEM. The engine \
             reported `{}` and the panel still says `{}`. The panel reads \
             `doc.session.view()`, so this is the case where the write went somewhere the read \
             does not look — or the epoch did not bump and the row is a cached listing.",
            applied.raw, after.raw
        )));
    }
    report.note(format!(
        "★★ the panel read the note back out of the session: `{}`",
        after.raw
    ));
    Ok(None)
}
