//! `form_field` — **place a form field on the page, then click an existing one
//! and get its properties.**
//!
//! The driven assertion for the operator's request of 2026-08-26, in both its
//! halves:
//!
//! > *"when I click one I should be able to click on the canvas to place the
//! > position or drag a box for size then a pop up lets me set the details for
//! > the feature"* … *"when I click on an existing form field on the page it's
//! > properties should come up in our side pane for editing it's properties."*
//!
//! # ★★★ Why this check is the only oracle for most of the feature
//!
//! Everything between a click and an authored field crosses boundaries a unit
//! test cannot: an armed tool in `egui::Memory`, a gesture resolved from a real
//! pointer, a canvas→page transform against a real page, a **second OS window**,
//! and five `pdfce-core` verbs. The unit tests cover each rule; not one of them
//! covers the sequence, and the sequence is where this project's defects live.
//!
//! The precedent is the shell's own founding defect: the Delete key's guard was
//! *"analysis-confirmed, NOT empirically verified"*, its unit test built a bare
//! context with no widgets, and the condition that broke the real application
//! could not occur in the harness.
//!
//! # ★★ The dialog is answered by a seam, and that is not a shortcut
//!
//! `PDFCE_DIAG_FORM_ACCEPT=1` makes the placement dialog press its own Add on
//! the first frame it is authorable. This harness drives **one** window — the
//! one `Session::launch` found — and the dialog is a deferred viewport with a
//! window of its own, so without the seam everything downstream of placing is
//! unreachable: the five engine verbs, the narrowing in
//! `app::actions::forms::author`, and all four rule-4 disclosures.
//!
//! Two seams already exist for exactly this shape — `PDFCE_DIAG_OPEN_PATH` and
//! `PDFCE_DIAG_INSERT_PATH`, both substituting the answer to a native picker.
//! What this one substitutes is the **operator's press**, not the authoring:
//! it sets the same flag the Add button sets, so the readiness guard, the
//! action, the remembering and the engine call are all the path an operator
//! takes.
//!
//! # ★ The two clicks aim at deliberately different places
//!
//! The first must land on **empty page** — a click on an existing widget would
//! place a field on top of one, which is legal and would make the second phase
//! ambiguous. The second must land on a widget whose canvas rect the
//! application itself published in a `form-box` line, so the check aims at
//! where the program says the box is rather than at where the fixture author
//! thought it would be. That is the `HANDOFF.md` §2 defect-8 rule: a click that
//! hits the field next to the one it aimed at is the same screenshot as a click
//! that worked.
//!
//! # Phases
//!
//! | Phase | Does | Expected |
//! |---|---|---|
//! | A | launch with `mode.edit,edit.form_text_field` | `form-tool-armed kind=Text` |
//! | B | click empty page | `form-field-open kind=Text`, then `add-form-field` succeeded |
//! | C | Escape, then click a published `form-box` | `form-field-selected field=…` |
//! | D | read the properties region | `properties.form_field` declared |

use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

/// The commands rung on startup, in order, one per frame.
///
/// ★ The list form of `PDFCE_DIAG_INVOKE`, which exists because arming a form
/// tool takes two commands: the arm declines without `edit_content`, so Edit
/// mode has to be entered first. Using it here rather than clicking a mode
/// segment also removes a whole class of flake from this check — a mode segment
/// click that misses is a failure about the ribbon, not about forms.
const INVOKE: &str = "mode.edit,edit.form_text_field";

/// The seam that answers the placement dialog. See the module header.
const ACCEPT_ENV: (&str, &str) = ("PDFCE_DIAG_FORM_ACCEPT", "1");

/// Traced when a form tool is armed.
const ARMED: &str = "form-tool-armed";
/// Traced when the placement dialog opens.
const OPENED: &str = "form-field-open";
/// Traced by the `vector_edit` funnel for the authoring verb.
const AUTHORED: &str = "add-form-field";
/// Traced when a click selects an existing field.
const SELECTED: &str = "form-field-selected";
/// The census line naming every **selectable** widget, in canvas space.
///
/// ★ `form-target`, not `form-box`. The two censuses describe different sets
/// and the difference is exactly what form authoring added: `form-box` lists
/// what a click can FILL, which excludes a drop-down, a push button and any
/// widget with no appearance. Aiming at that list would make this check unable
/// to reach three of the five kinds it exists to verify — and on a fixture whose
/// only text field is undrawn, unable to reach anything at all.
const BOX_LINE: &str = "form-target";
/// The properties section's published region.
const PROPERTIES_REGION: &str = "properties.form_field";

/// Placing a form field, and selecting one that already exists.
pub struct FormFieldPlaceAndSelect;

impl Check for FormFieldPlaceAndSelect {
    fn name(&self) -> &'static str {
        "form_field"
    }

    fn defect(&self) -> &'static str {
        "the five form-field commands arm a tool and a click on the page places nothing — or a \
         field is placed and clicking it again offers no way to rename or delete it, leaving \
         every form pdfce authors editable only by authoring it again"
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

/// One `form-box` census line, parsed back into a canvas rectangle.
struct PlacedBox {
    page: usize,
    field: String,
    /// Canvas-space centre — what the click aims at.
    centre: (f64, f64),
}

/// Read the application's own census of where the form's boxes are.
///
/// ★★ The application's numbers, not the fixture's. `canvas/forms.rs` publishes
/// one line per widget precisely so a harness can aim at where the program says
/// the box is; a check that computed the rect from the PDF would be asserting
/// that two independent derivations agree, and would report a disagreement as a
/// hit-test failure.
fn placed_boxes(trace: &Trace) -> Vec<PlacedBox> {
    trace
        .events(BOX_LINE)
        .filter_map(|l| {
            let page = l.get("page")?.parse().ok()?;
            let field = l.get("field")?.to_owned();
            // `rect=(x,y)+(w,h)` — the canvas rect, as the census writes it.
            let raw = l.get("rect")?;
            let (min, size) = raw.split_once(")+(")?;
            let (x, y) = min.trim_start_matches('(').split_once(',')?;
            let (w, h) = size.trim_end_matches(')').split_once(',')?;
            let (x, y): (f64, f64) = (x.trim().parse().ok()?, y.trim().parse().ok()?);
            let (w, h): (f64, f64) = (w.trim().parse().ok()?, h.trim().parse().ok()?);
            Some(PlacedBox {
                page,
                field,
                centre: (x + w / 2.0, y + h / 2.0),
            })
        })
        .collect()
}

/// Run the four phases.
#[allow(
    clippy::too_many_lines,
    reason = "one driven sequence; splitting it would hide the ORDER, which is the subject" // ui-text-exempt: lint justification
)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new(
            "no --pdf. Pass a document that already carries an /AcroForm — phase C clicks an \
             existing field, and on a drawing with no form there is nothing to click and this \
             check would silently measure only half of itself.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check is three real clicks, a keystroke and \
             the foreground. Reported as SKIPPED rather than passed: a check that did not run \
             has learned nothing.",
        ));
    }
    let vocab = &ctx.profile.vocab;
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot state \
             where its controls are.",
            ctx.profile.name
        ))
    })?;
    let target = ctx.target.ok_or_else(|| {
        Error::new(
            "no --doc-point. Pass PAGE,X,Y in PDF user space naming EMPTY page — somewhere the \
             fixture has no form widget. There is deliberately no default: a placement click \
             that landed on an existing field would still open the dialog, so the check would \
             pass while aiming at the wrong thing.",
        )
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

    // --- A: launch with the tool already armed -----------------------------
    let mut spec = LaunchSpec::new(&exe, ctx.out("form_field.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push(("PDFCE_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.env
        .push((ACCEPT_ENV.0.to_owned(), ACCEPT_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} as pid {} with PDFCE_DIAG_INVOKE={INVOKE} and {}={}",
        exe.display(),
        session.pid(),
        ACCEPT_ENV.0,
        ACCEPT_ENV.1
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);

    let trace = session.trace()?;
    if !trace.started(vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so {}={} did not reach the process. Trace: {}.",
            vocab.start_event,
            ctx.profile.diag_env.0,
            ctx.profile.diag_env.1,
            session.trace_path().display()
        )));
    }
    let Some(armed) = trace.last(ARMED) else {
        return Ok(Some(format!(
            "no `{ARMED}` line. The two commands `{INVOKE}` were rung and the text-field tool \
             was not armed, so nothing below could work. Either the command has no arm, or Edit \
             mode was not entered and the arm declined on `edit_content`. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if armed.get("kind") != Some("Text") {
        return Ok(Some(format!(
            "`{ARMED}` reports kind={:?}, not Text — `edit.form_text_field` armed the wrong \
             tool, which would place the wrong control for every one of the five commands.",
            armed.get("kind")
        )));
    }
    report.note("the text-field tool is armed");

    // --- B: click empty page, and the field is authored --------------------
    let mapping = CanvasMapping::from_trace(&trace, vocab, page, target.page)?;
    let window_point = mapping.doc_to_window(DocPoint::new(target.page, target.x, target.y))?;
    let frame = session.frame()?;
    let driver = Driver::new(session.window());
    report.note(format!(
        "clicking empty page at PDF ({}, {}) on page {}",
        target.x, target.y, target.page
    ));
    driver.click_at(frame.to_screen(window_point))?;
    session.settle(30);

    let trace = session.trace()?;
    let Some(opened) = trace.last(OPENED) else {
        return Ok(Some(format!(
            "the click placed nothing: no `{OPENED}` line. The tool was armed (phase A proved \
             it), so the failure is between the pointer and the action — the gesture did not \
             resolve to a form placement, or the click never reached the canvas. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!(
        "the dialog opened for kind={:?} named {:?}",
        opened.get("kind"),
        opened.get("name")
    ));
    // ★ The generated name is asserted non-empty because it is what makes the
    // dialog acceptable: `is_authorable` refuses a blank one, so a naming bug
    // would make the seam below do nothing and the failure would read as "the
    // engine refused" rather than "the name was never generated".
    if opened.get("name").is_none_or(str::is_empty) {
        return Ok(Some(
            "the dialog opened with no generated field name. Nothing can be authored without \
             one — Add stays greyed — so a placement would silently do nothing."
                .to_owned(),
        ));
    }
    if !trace
        .events(AUTHORED)
        .any(|l| !l.raw.contains("refused") && !l.raw.contains("failed"))
    {
        return Ok(Some(format!(
            "the dialog opened and accepted and no field was authored: no clean `{AUTHORED}` \
             line. This is the half no unit test reaches — five engine verbs behind one \
             narrowing. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("a field was authored");

    // --- C: disarm, then click a field that already exists -----------------
    //
    // ★ Escape first. The tool stays armed after a placement, exactly as a
    // markup pen does, so a second click without this would place a SECOND
    // field rather than select one — and the check would fail with a message
    // about selection when the cause was arming.
    driver.press(crate::sys::vk::ESCAPE)?;
    session.settle(12);

    let trace = session.trace()?;
    let boxes = placed_boxes(&trace);
    let Some(existing) = boxes.iter().find(|b| b.page == target.page) else {
        return Err(Error::new(format!(
            "the application published no `{BOX_LINE}` line for page {}, so this check has no \
             widget to aim at. Either the fixture's form has no widget on that page, or the \
             census stopped being written. Reported as a SKIP rather than a failure because a \
             fixture with no field on the clicked page is a harness problem, not a defect.",
            target.page
        )));
    };
    report.note(format!(
        "aiming at the field {:?} the application placed at canvas ({:.1}, {:.1})",
        existing.field, existing.centre.0, existing.centre.1
    ));
    // The census is canvas space; `doc_to_window` takes PDF space. The flip is
    // the one arithmetic this check does, and it is the mapping's own formula
    // read backwards: `canvas_y = page_height - doc_y`.
    let doc_y = page.height_pt - existing.centre.1;
    let point = mapping.doc_to_window(DocPoint::new(existing.page, existing.centre.0, doc_y))?;
    driver.click_at(frame.to_screen(point))?;
    session.settle(25);

    let trace = session.trace()?;
    let Some(selected) = trace
        .events(SELECTED)
        .filter(|l| l.get("field").is_some())
        .last()
    else {
        return Ok(Some(format!(
            "clicking an existing form field selected nothing: no `{SELECTED}` line with a \
             field name. In Edit mode a click on a widget must select it for its properties; \
             what an operator meets if this regresses is a field they can see, can place, and \
             cannot rename or delete. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!(
        "selected {:?} on page {:?}",
        selected.get("field"),
        selected.get("page")
    ));

    // --- D: and the properties section is actually on screen ---------------
    //
    // ★ The region, not just the trace line. A selection that no panel drew is
    // the whole defect restated: the operator clicked, something was recorded,
    // and nothing appeared. This is the difference between "the model changed"
    // and "the operator can see it", and only the second is the feature.
    let drawn = trace
        .events(ui_rect)
        .any(|l| l.get("name") == Some(PROPERTIES_REGION));
    if !drawn {
        return Ok(Some(format!(
            "the field was selected and the `{PROPERTIES_REGION}` region was never declared, so \
             the properties section did not draw. The most likely causes are that the \
             Properties panel is not open in this layout — which this check cannot open and \
             should learn to — or that the section returned early. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("the form-field properties section drew");
    Ok(None)
}
