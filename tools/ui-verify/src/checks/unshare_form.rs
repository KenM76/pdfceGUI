//! `the_context_menu_gives_this_page_its_own_copy_of_a_shared_form` — the
//! driven proof that `EditSession::unshare_form` has an operator route, and
//! that the route is not the ribbon.
//!
//! # What this is for
//!
//! `EDITABLE_SURFACES.md`, written 2026-08-28, is an audit keyed on the
//! **engine's** verb list rather than on this shell's feature list. It found
//! `unshare_form` implemented in `pdfce-core` and named nowhere in
//! `crates/pdfce-gui/src`, and `pdfce-core` asked for it by name:
//!
//! > *"So you can offer the button. Please un-suppress it rather than leaving
//! > the suppression in place — a control withheld on the strength of a note
//! > that has since been withdrawn is exactly the kind of thing that stays
//! > withheld for months."*
//!
//! ## ★★★ Why the audit's own instrument cannot answer this, which is the
//! ## whole reason this file exists
//!
//! `tools/verb-coverage.py` greps this crate for each engine verb's name.
//! `EDITABLE_SURFACES.md` states the limit of that measurement in as many
//! words:
//!
//! > **A hit means the NAME appears**, not that a reachable operator route
//! > calls it. A call site behind a condition nothing sets is a hit here and
//! > dead in the running program. Only `tools/ui-verify` answers that question.
//!
//! ⇒ So the audit that found this gap **would report it fixed the moment the
//! identifier appeared in a source file**, whether or not a single click could
//! reach it. This check is the other half: the verb is called because a person
//! pressed something.
//!
//! ## ★★★ The route under test is the CONTEXT MENU, deliberately
//!
//! `OPERATOR_REQUESTS.md` **O53** rules that a command must not exist only on
//! the ribbon. That rule is doing more work for this command than for most, and
//! the reason is about *when* the operator needs it rather than about
//! consistency:
//!
//! An operator who needs to unshare is, by construction, **mid-gesture**. They
//! have clicked inside a title block, they are about to type into it, and the
//! moment the choice is worth anything is *before* that keystroke — because
//! afterwards the edit is already in the one shared stream and every sheet has
//! it. The Format contextual tab is the correct second home. The pointer is the
//! first.
//!
//! ★★ And the ribbon route is the one a check could pass on while the useful
//! one was broken: a Format-tab click proves a band item dispatches, which
//! `font_group` and its neighbours already prove for that tab. Nothing before
//! this file had ever **pressed a context-menu row** — see the harness gap
//! below, which is the finding this check turned up.
//!
//! ## ★★★ The harness gap this check found, and had to close
//!
//! `right_clicking_a_form_field_opens_its_menu` was the first driven context
//! menu in this project's history, on 2026-08-28. It asserts that the right
//! menu **resolved** and that it **offered something**, and it stops there.
//!
//! It stopped there because it had to. `shell::menus::MenuHost::attach_with`
//! called `egui_shell::menu::Menu::attach` — the convenience constructor that
//! takes *no optional capabilities at all* — so pdfce's context menus drew rows
//! and published **no `ui_rect` for any of them**. There was no coordinate to
//! aim at, so no check could press a row, so the entire "does the menu row
//! actually do the thing" question was unaskable.
//!
//! ⇒ That is the same shape `field_menu`'s own header records one layer up:
//! *"a gesture with no driver is a gesture R1 cannot reach, and the gap left no
//! failing test behind to advertise itself."* There the driver was missing;
//! here the **target** was. Both are invisible to a green suite.
//!
//! `MenuHost::attach_with` now supplies a rect sink, so every row of every
//! pdfce context menu publishes `menu.item.<context>.<command id>` through the
//! same `crate::diag::ui_rect` channel the ribbon and the status bar use. This
//! check is the first consumer; every future menu check inherits it.
//!
//! ★ Publishing is the only possible answer for a popup, and
//! `egui_shell::menu::report`'s header says why: a context menu is drawn **at
//! the pointer**, and `egui` may flip it to any of several alignments to keep
//! it on screen. There is no fraction of the window it can be hard-coded to and
//! no layout a harness could re-derive.
//!
//! ## The oracle: `unshare-form-applied`, and what a wrong build gets wrong
//!
//! `app::actions::xobject` traces
//! `unshare-form-applied page=… original=… copy=… moved=…` on the success path.
//! Three of those four fields are load-bearing here:
//!
//! | field | what a wrong build reports |
//! |---|---|
//! | `original=` | the **innermost** enclosing form instead of the outermost — the operand `EditError::FormNestedInAnotherForm` exists to refuse. On this flat fixture the two coincide, which is why the check also asserts the number against the page's own object list rather than merely against itself |
//! | `copy=` | the same number as `original`, i.e. nothing was allocated |
//! | `moved=` | `0`, i.e. the page's `/XObject` names were not re-pointed and the copy is an orphan |
//!
//! ★★ **The absence of the line is the interesting failure**, not its content.
//! A build where the menu row is greyed, where the dispatcher has no arm, where
//! `containing_form_object` returns the innermost form, or where the engine
//! refuses, all produce **no line at all** — and each of those is a state in
//! which the operator presses a row and the page looks exactly as it did. That
//! is why this check exists and a unit test would not do: on this command,
//! *"nothing visibly happened"* is what **success** looks like too.
//!
//! ## Why this check pins its own fixture and ignores `--pdf`
//!
//! `form_selection`'s reason, unchanged and for the same subject: a check whose
//! subject is *"what happens to a form XObject"* cannot take an arbitrary
//! document. On a drawing with no forms — the operator's own SolidWorks export
//! has **zero** — the honest answer is *"there was nothing to unshare"*, which
//! is neither a pass nor a defect.
//!
//! The fixture is the engine's `forms-xobject/page-sized-form.pdf`, read from
//! the read-only corpus at `D:\Dev\pdfce`: one 200 × 200 pt page whose only
//! page object is a page-sized form holding three 40 × 40 squares.
//!
//! ★ It is invoked **once**, not thirty-six times, and that is fine —
//! `unshare_form` does not require a form to be shared, and refusing to
//! privatise a singly-invoked form would be a rule nobody wrote. What the
//! fixture has to supply is a **leaf**, so that a click produces the operand
//! this command derives from, and it does.
//!
//! ## The sequence
//!
//! | # | step | oracle |
//! |---|---|---|
//! | 1 | open the fixture, click the Edit mode segment | `ribbon-mode` |
//! | 2 | click the centre of the middle square | `canvas-selection first=leaf:` |
//! | 3 | right-click the same point | `canvas-menu context=canvas.object` |
//! | 4 | the unshare row is on screen and clickable | `menu.item.canvas.object.format.unshare_form` |
//! | 5 | click it | `unshare-form-applied` |
//!
//! Step 4 is O53's assertion and step 5 is the audit's. Neither substitutes for
//! the other: a row that is drawn and does nothing passes 4, and a command
//! reachable only from the ribbon would pass 5 if this check pressed a band
//! item instead.

use crate::checks::driving::{SHELL_DIAG_ENV, click_mode_segment, declared, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry, ScreenPoint, WindowFrame};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

/// The fixture, under the engine's read-only synthetic corpus.
const FIXTURE: &str = "forms-xobject/page-sized-form.pdf";

/// The fixture's page, in PDF points. Stated rather than read, for
/// `form_selection`'s reason: the whole file is fourteen objects of
/// hand-written syntax, and a page size that changed would change what every
/// constant below means.
const PAGE: PageGeometry = PageGeometry {
    width_pt: 200.0,
    height_pt: 200.0,
};

/// The centre of the middle square (PDF user space, 80,80 → 120,120).
///
/// The **middle** one, exactly as `form_selection` aims: it is furthest from
/// every page edge, so a small error in the coordinate hop lands on paper
/// rather than off-window — and a failure then reads "selected nothing" rather
/// than "the click went outside the client area", which are different
/// diagnoses.
///
/// ★ It matters twice here rather than once, because the same point is
/// right-clicked. A popup opened near an edge is repositioned by `egui`, which
/// is exactly the case the published rect exists to survive — but a check
/// should not be *testing* that incidentally while trying to test something
/// else.
const ON_A_SQUARE: (f64, f64) = (100.0, 100.0);

/// `canvas-selection … first=object:N|leaf:N|none` — what a click selected.
const SELECTION_EVENT: &str = "canvas-selection";
/// The field naming which index space the selection landed in.
const FIRST_FIELD: &str = "first";
/// `canvas-menu context=…` — which menu a right-click resolved.
const MENU_EVENT: &str = "canvas-menu";
/// The context a selected page object must resolve to.
const OBJECT_CONTEXT: &str = "canvas.object";
/// The published rect of the row this check presses.
const ROW_REGION: &str = "menu.item.canvas.object.format.unshare_form";
/// The prefix every context-menu row publishes under.
const ROW_PREFIX: &str = "menu.item.canvas.object.";
/// `unshare-form-applied page=… original=… copy=… moved=…` — the success line.
const APPLIED: &str = "unshare-form-applied";

/// See the module documentation.
pub struct TheContextMenuGivesThisPageItsOwnCopyOfASharedForm;

impl Check for TheContextMenuGivesThisPageItsOwnCopyOfASharedForm {
    fn name(&self) -> &'static str {
        "the_context_menu_gives_this_page_its_own_copy_of_a_shared_form"
    }

    fn defect(&self) -> &'static str {
        "`EditSession::unshare_form` has no operator route, so a title block invoked from \
         thirty-six sheets can be edited in place and cannot be privatised first — the operator \
         has decision 076's default and no option at all, which is the state R206 exists to \
         prevent"
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

/// Resolve a fixture under the engine repository's synthetic corpus.
///
/// ★ The path is derived, not configured, and `None` rather than a panic —
/// `form_selection`'s helper verbatim in shape, for the reason its own docs
/// give: `D:\Dev\pdfce` is READ-ONLY to this project, and a missing corpus is a
/// SKIP with a reason rather than a crash mid-suite.
fn engine_fixture(rel: &str) -> Option<std::path::PathBuf> {
    let path = std::path::Path::new("D:/Dev/pdfce/fixtures/synthetic").join(rel);
    path.is_file().then_some(path)
}

/// Run the sequence.
///
/// The three-way return is the SKIP/FAIL/PASS rule made structural, as
/// everywhere in this suite: `Err` is a precondition that was absent (SKIP),
/// `Ok(Some(_))` is an assertion that did not hold (FAIL), `Ok(None)` is a
/// pass. An author who reaches for `?` gets a SKIP, which is the safe default —
/// the unsafe default would be a pass.
#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let vocab = &ctx.profile.vocab;

    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input), and this check cannot be performed without \
             clicking and right-clicking. Reported as SKIPPED rather than passed: a check that \
             did not run has learned nothing.",
        ));
    }
    let fixture = engine_fixture(FIXTURE).ok_or_else(|| {
        Error::new(format!(
            "the engine's form fixture is not at D:/Dev/pdfce/fixtures/synthetic/{FIXTURE}. This \
             check pins it and ignores --pdf: its subject is what happens to a form XObject, and \
             on a document with no forms the honest answer is 'there was nothing to unshare', \
             which is neither a pass nor a defect."
        ))
    })?;
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot state \
             where its mode segments or its menu rows are. Both are load-bearing here: this \
             check has to leave Read mode, and it has to press a row in a popup whose position \
             depends on where the pointer was.",
            ctx.profile.name
        ))
    })?;

    // --- launch ------------------------------------------------------------
    let mut spec = LaunchSpec::new(&exe, ctx.out("unshare_form.trace.txt"));
    spec.pdf = Some(fixture.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    // The shell's channel too: `click_mode_segment` reads `egui-shell`'s own
    // trace, and without this the mode click looks like a miss.
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} as pid {} on {}",
        exe.display(),
        session.pid(),
        fixture.display()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);

    let trace = session.trace()?;
    if !trace.started(vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so the diagnostic switch did not reach the process and \
             this check has no oracle. Captured stderr is at {}.",
            vocab.start_event,
            session.trace_path().display()
        )));
    }

    // --- 1: leave Read, where a content click is refused BY DESIGN ---------
    let driver = Driver::new(session.window());
    click_mode_segment(&session, &driver, ui_rect, "edit")?;
    report.note(
        "clicked the Edit mode segment first — the shell's default mode is Read, where a canvas \
         click on content is refused by design (DEFECTS.md D6)",
    );

    // --- 2: select something INSIDE the form -------------------------------
    //
    // ★ The operand this command derives from is a LEAF, and nothing else will
    // do: `format.unshare_form`'s dispatch arm reads the selection's first leaf
    // and asks for that leaf's outermost enclosing form. Selecting the form
    // itself would leave `selection.in_form` false and grey the row — which is
    // correct behaviour and would look exactly like the defect.
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, vocab, PAGE, 0)?;
    report.note(format!(
        "canvas rect {:?} at zoom {:.3}",
        mapping.image_rect, mapping.zoom
    ));
    let frame = session.frame()?;
    let target = aim(&mapping, &frame, ON_A_SQUARE)?;
    report.note(format!(
        "the middle square's centre (page 0, {:.1}, {:.1}) -> screen ({}, {})",
        ON_A_SQUARE.0,
        ON_A_SQUARE.1,
        target.x(),
        target.y()
    ));
    driver.click_at(target)?;
    session.settle(15);

    let after = session.trace()?;
    let Some(first) = last_first(&after) else {
        return Err(Error::new(format!(
            "the click produced no `{SELECTION_EVENT} … {FIRST_FIELD}=` line, so the harness has \
             no oracle for what is selected and everything after it would be guesswork. \
             `a_click_inside_a_form_selects_what_is_drawn_there` is the check that owns this \
             step; if it is also failing, fix that one first. Trace: {}",
            session.trace_path().display()
        )));
    };
    if !first.starts_with("leaf:") {
        return Err(Error::new(format!(
            "the click on the middle square selected `{FIRST_FIELD}={first}`, and this check \
             needs a LEAF — an object painted from inside the form — because that is the only \
             operand `format.unshare_form` can derive its form from. Reported as SKIPPED rather \
             than failed: the failure is in the deep hit test, which \
             `a_click_inside_a_form_selects_what_is_drawn_there` owns, and blaming the unshare \
             for it would send the next reader to the wrong file. Trace: {}",
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★ selected {FIRST_FIELD}={first} — inside the form"
    ));

    // --- 3: right-click the same point -------------------------------------
    driver.right_click_at(target)?;
    session.settle(35);

    let trace = session.trace()?;
    let Some(menu) = trace.events(MENU_EVENT).last() else {
        return Ok(Some(format!(
            "THE RIGHT-CLICK RESOLVED NO MENU AT ALL: no `{MENU_EVENT}` line after a secondary \
             click on a selected leaf. `canvas::menus::attach` writes that line on every frame \
             carrying a secondary click, so its absence means the click never reached the canvas \
             response. Trace: {}",
            session.trace_path().display()
        )));
    };
    let context = menu.get("context").unwrap_or_default();
    if context != OBJECT_CONTEXT {
        return Ok(Some(format!(
            "THE RIGHT-CLICK ON A FORM-INTERIOR OBJECT RESOLVED `{context}`, NOT \
             `{OBJECT_CONTEXT}`: `{}`. A leaf IS an object selection, so the object menu is the \
             one that must appear; resolving the view menu here would mean the right-click hit \
             test cannot see inside a form, which is the same defect as \
             `a_click_inside_a_form_selects_what_is_drawn_there` reaching through a second door. \
             Trace: {}",
            menu.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!("★★ the right-click resolved `{}`", menu.raw));

    // --- 4: the row is on screen, and O53 is satisfied ---------------------
    //
    // ★★★ This assertion is about the ROUTE, not about the verb, and it is the
    // one that fails if `format.unshare_form` is registered on the ribbon only.
    // A greyed row publishes no rect either — `plan::resolve` drops a disabled
    // command before it is drawn — so this also catches a build where
    // `selection.in_form` is not published for a leaf selection.
    let Some(row) = declared(&trace, ui_rect, ROW_REGION) else {
        return Ok(Some(format!(
            "★★★ THE UNSHARE ROW IS NOT IN THE CANVAS OBJECT MENU: no `{ROW_REGION}` region \
             after the menu opened. Rows it DID publish: {}.\n\
             Three readings, and all three are the defect O53 names: the command is registered \
             on the Format ribbon tab only; the row is drawn but disabled, because \
             `selection.in_form` is not set for a leaf selection and a disabled command is \
             dropped before it is drawn; or `MenuHost::attach_with` has stopped supplying a rect \
             sink, in which case no context-menu row anywhere in this application can be pressed \
             by a check. Trace: {}",
            list(&declared_names(&trace, ui_rect, ROW_PREFIX)),
            session.trace_path().display()
        )));
    };
    if !row.is_substantial() {
        return Ok(Some(format!(
            "`{ROW_REGION}` was published at {row:?}, which has no usable area — so the row \
             exists in the plan and was laid out to nothing. A click aimed at a degenerate \
             rectangle proves nothing, and this is itself the finding."
        )));
    }
    report.note(format!(
        "★★★ the unshare row is in the CONTEXT MENU at {row:?} — O53's requirement that a \
         command must not exist only on the ribbon"
    ));

    // --- 5: press it, and read what the verb did ---------------------------
    let before = session.trace()?.events(APPLIED).count();
    driver.click_at(session.frame()?.declared_center(row))?;
    session.settle(30);

    let trace = session.trace()?;
    let applied: Vec<_> = trace.events(APPLIED).collect();
    let Some(line) = applied.get(before) else {
        return Ok(Some(format!(
            "★★★ THE ROW WAS PRESSED AND NOTHING WAS UNSHARED: no new `{APPLIED}` line.\n\
             **This is what the defect looks like from the operator's chair, and it looks like \
             success**: the copy `unshare_form` makes is byte-identical to the original, so a \
             page that WAS unshared renders pixel-for-pixel as one that was not. Nothing on \
             screen distinguishes them.\n\
             Candidate causes, in the order they are worth checking: `app::dispatch::format` has \
             no arm for `format.unshare_form` (its `handles` and its `match` must agree — \
             `shell::commands::reach` fails closed on that and would be red); \
             `containing_form_object` returned the INNERMOST enclosing form, which \
             `EditError::FormNestedInAnotherForm` refuses by name; or the engine declined for a \
             document-wide reason, in which case `app::status::decline` is carrying a worded \
             sentence and the status bar is showing it. Trace: {}",
            session.trace_path().display()
        )));
    };
    report.note(format!("★★★ `{}`", line.raw));

    let original = line.get("original").unwrap_or_default();
    let copy = line.get("copy").unwrap_or_default();
    let moved = line.get("moved").unwrap_or_default();

    if copy == original || copy.is_empty() {
        return Ok(Some(format!(
            "THE COPY IS NOT A COPY: `{}` reports copy={copy:?} against original={original:?}. \
             `unshare_form` allocates a new object number and clones the stream into it; two \
             equal numbers mean the page was re-pointed at the object it already named, which is \
             a no-op wearing a success line.",
            line.raw
        )));
    }
    if moved != "1" {
        return Ok(Some(format!(
            "THE PAGE'S REFERENCES DID NOT MOVE AS EXPECTED: `{}` reports moved={moved:?}, and \
             this fixture's page invokes its one form under exactly one name.\n\
             `0` means the copy was allocated and nothing was re-pointed at it — an orphan \
             object in the file and no change to what the page draws, which is strictly worse \
             than refusing. A number above 1 means the page's /XObject dictionary named the form \
             more than once, which this fourteen-object hand-written fixture does not do, so it \
             says the count is being derived from something other than this page's own names.",
            line.raw
        )));
    }
    report.note(format!(
        "★★★ page 0 now names its own copy of form {original}: object {copy}, {moved} reference \
         moved. Every other invocation site keeps naming {original} and is byte-identical."
    ));

    Ok(None)
}

/// A page-space point, through the mapping and the window frame, to a desktop
/// point.
///
/// Its own function so the click and the right-click cannot hop differently —
/// the class of error `crate::coords` exists to prevent. Both gestures in this
/// check aim at the *same* screen point, and that is load-bearing: the menu
/// must open over the thing that was selected, not over a second guess at where
/// it is.
fn aim(mapping: &CanvasMapping, frame: &WindowFrame, point: (f64, f64)) -> Result<ScreenPoint> {
    let window = mapping.doc_to_window(DocPoint {
        page: 0,
        x: point.0,
        y: point.1,
    })?;
    Ok(frame.to_screen(window))
}

/// The `first=` value of the most recent `canvas-selection` line, if any.
///
/// ★ The **last** line rather than a count of new ones, for
/// `form_selection::last_first`'s reason: `canvas-selection` is emitted through
/// `diag::trace_changed`, so a click producing the same selection as the
/// previous one emits nothing, and a consumer that counted lines would read a
/// legitimate no-change as a dropped event.
fn last_first(trace: &Trace) -> Option<String> {
    trace
        .last(SELECTION_EVENT)
        .and_then(|l| l.get(FIRST_FIELD))
        .map(str::to_owned)
}
