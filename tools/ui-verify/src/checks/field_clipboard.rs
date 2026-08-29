//! `a_form_field_can_be_copied_and_pasted_both_ways` — the driven proof of
//! `OPERATOR_REQUESTS.md` **O58**.
//!
//! # What was wrong
//!
//! **Ken, 2026-08-29:** *"wire the request. ctrl v for paste as new. ctrl shift
//! v for paste as duplicate."*
//!
//! Before this, `Ctrl+C` over a selected form field did **nothing** — and not
//! by refusal. There was no path at all: `canvas::clipboard::copy` reads
//! `doc.selection`, a selected form field lives on `doc.selected_field`, and
//! `/Widget` is deliberately excluded from annotation selection. So the chord
//! fell through to the *content* copy, found an empty content selection, and
//! refused with *"nothing is selected"* over an object with visible grips
//! around it.
//!
//! # ★★★ Why a unit test cannot discharge this, and it is R1's argument again
//!
//! `canvas::fieldclip::tests` proves the offset rule and the loss list, and
//! `text::fieldclip::tests` proves the sentences. Neither can see the six
//! things standing between those functions and the operator's keyboard, and
//! **every one of them has been a real defect on this project**:
//!
//! | | the failure it would hide |
//! |---|---|
//! | the **three-rung fork** in `dispatch::clipboard` | a text draft or an empty content selection taking the chord first |
//! | the **binding** | fourteen declared shortcuts turned out never to have been dispatched, 2026-08-18 |
//! | `Ctrl+Shift+V` **reaching egui at all** | winit derives modifier state from key events; a synthesised `VK_SHIFT` is not always recognised (`sys::vk::LSHIFT`'s note) |
//! | the **mode gate** | a form field is content, so Read refuses both chords — a check that stayed in Read would report the gate as a clipboard defect |
//! | the **action queue** | `FieldAction::Paste` drains at end of frame; a variant with no arm traces nothing and does nothing |
//! | the **engine's merge** | the duplicate paste's entire claim is that `add_text_field` merges on a matching `/T`. Nothing in this shell can prove that; only the running engine can |
//!
//! ★ The last row is the one that matters most. This shell **asserts to the
//! operator** that a duplicate paste keeps the original's font, colour and
//! calculation script, and the whole basis of that claim is one branch inside
//! `pdfce-core`. A green unit suite would restate the claim; only a driven run
//! against the real engine can test it.
//!
//! # The oracle
//!
//! Three trace lines, and the check reads all three because each answers a
//! question the others cannot:
//!
//! | line | question |
//! |---|---|
//! | `fieldclip-copy` | did the chord reach the FIELD path, rather than the content path? |
//! | `fieldclip-paste` | was a paste raised, and in which of the two senses? |
//! | `form-target` | did a **second box** actually appear on the page? |
//!
//! ★★ The third is the one that makes this check worth running. `fieldclip-paste`
//! proves an *intent* was raised; it does not prove the engine accepted it. A
//! build whose `FieldAction::Paste` arm was never written would emit the paste
//! line and add no field, and a check reading only the first two would pass
//! against it. Counting `form-target` boxes before and after is what makes the
//! difference between "the shell asked" and "the document changed".
//!
//! # The sequence
//!
//! 1. launch in Edit with the text-field tool armed and the placement dialog
//!    auto-accepting, and place a field with one click;
//! 2. Escape to disarm, click blank paper to clear the selection, click the
//!    field to select it;
//! 3. `Ctrl+C` — assert a `fieldclip-copy` line;
//! 4. `Ctrl+V` — assert `fieldclip-paste mode=NewField` **and one more box**;
//! 5. `Ctrl+Shift+V` — assert `fieldclip-paste mode=Duplicate` **and one more
//!    box again**.
//!
//! ★ Step 5 runs against the clipboard written in step 3, not against anything
//! step 4 left behind, which is the property that makes the two chords
//! independent: an operator copies once and pastes several times.
//!
//! # What this check does NOT prove, said out loud
//!
//! That the duplicate **shares a value** with its source. That is the whole
//! point of the chord and it is invisible from the outside: proving it needs
//! typing into one box and reading the other, which needs a fill gesture this
//! harness does not yet have. The box count proves a second widget arrived; it
//! does not prove the two are one field.
//!
//! ⇒ Recorded here rather than left implied, because a check whose name says
//! "both ways" and whose body proves half of one of them is exactly the
//! green-result-reporting-nothing this harness exists to avoid. The engine
//! request in the channel asks for the verb that would make this assertable.

use crate::checks::driving::SHELL_DIAG_ENV;
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;
use crate::trace::Trace;

/// Edit mode, and the text-field tool armed. Same pair `field_menu` uses.
const INVOKE: &str = "mode.edit,edit.form_text_field";

/// The placement dialog accepts itself, so no dialog has to be driven.
const ACCEPT_ENV: (&str, &str) = ("PDFCE_DIAG_FORM_ACCEPT", "1");

/// The per-widget census the canvas publishes — one line per drawn box.
const BOX_LINE: &str = "form-target";

/// `form-field-selected`, with `field=` on a selection and bare on a clear.
const SELECTED: &str = "form-field-selected";

/// The copy's own line.
const COPY_LINE: &str = "fieldclip-copy";

/// The paste's own line, carrying `mode=NewField` or `mode=Duplicate`.
const PASTE_LINE: &str = "fieldclip-paste";

/// Where the first field is placed, as page fractions.
///
/// Well inside the sheet on both axes, because two pastes each displace the
/// copy ten points down and to the right and all three boxes must stay on
/// paper — a box pasted off the sheet would produce no `form-target` line and
/// the check would report a clipboard defect for a geometry problem.
const PLACE_AT: (f64, f64) = (0.30, 0.45);

/// See the module documentation.
pub struct AFormFieldCanBeCopiedAndPastedBothWays;

impl Check for AFormFieldCanBeCopiedAndPastedBothWays {
    fn name(&self) -> &'static str {
        "a_form_field_can_be_copied_and_pasted_both_ways"
    }

    fn defect(&self) -> &'static str {
        "Ctrl+C over a selected form field does nothing at all — not a refusal, no path: the \
         clipboard reads the object selection, a selected field lives elsewhere, and /Widget is \
         excluded from annotation selection, so the chord falls through to the content copy and \
         says 'nothing is selected' over an object with grips around it"
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

/// Every widget box the canvas named, as `(page, field, canvas centre)`.
///
/// The same reader `field_menu` uses, kept in step with it deliberately: two
/// parsers of one trace line is how two checks come to disagree about what the
/// program said.
fn boxes(trace: &Trace) -> Vec<(usize, String, (f64, f64))> {
    trace
        .events(BOX_LINE)
        .filter_map(|l| {
            let page: usize = l.get("page")?.parse().ok()?;
            let field = l.get("field")?.to_owned();
            let (min, size) = l.get("rect")?.split_once(")+(")?;
            let (x, y) = min.trim_start_matches('(').split_once(',')?;
            let (w, h) = size.trim_end_matches(')').split_once(',')?;
            let (x, y): (f64, f64) = (x.trim().parse().ok()?, y.trim().parse().ok()?);
            let (w, h): (f64, f64) = (w.trim().parse().ok()?, h.trim().parse().ok()?);
            Some((page, field, (x + w / 2.0, y + h / 2.0)))
        })
        .collect()
}

/// How many DISTINCT field names have a box on page 0.
///
/// ★ Distinct **names**, not lines. The census is re-emitted every frame it
/// changes, so counting lines counts repaints. And it is names rather than
/// boxes because a paste-as-new must raise the count and this is the number
/// that says so unambiguously.
fn field_names(trace: &Trace) -> std::collections::BTreeSet<String> {
    boxes(trace)
        .into_iter()
        .filter(|(p, _, _)| *p == 0)
        .map(|(_, f, _)| f)
        .collect()
}

/// The DISTINCT boxes on page 0 — a set of `(field, centre)`, not a line count.
///
/// ★★★ **The first version of this function counted trace lines and it was
/// wrong, and it was wrong in the direction that still passes.** The census is
/// re-emitted on every frame it changes, so the cumulative line count went
/// 1 → 3 → 6 across the two pastes: 1, then 1+2, then 3+3. Both assertions held
/// — the number did rise each time — and the check reported PASS while measuring
/// *repaints* rather than *widgets*.
///
/// That is this project's standing failure: **ask what the check SAMPLED before
/// believing what it says.** A build that pasted nothing but repainted twice
/// would have satisfied the old version exactly as well.
///
/// A set of `(field, centre)` is immune, because a re-emitted census re-states
/// the same pairs. Two widgets of one field differ by centre — the paste offset
/// guarantees it — so a duplicate raises this count without raising
/// [`field_names`], which is the distinction the whole feature is about.
///
/// The centre is rounded to whole canvas pixels before it enters the set: the
/// census prints one decimal, and a scroll of a fraction of a pixel between two
/// frames would otherwise make one box look like two.
fn distinct_boxes(trace: &Trace) -> std::collections::BTreeSet<(String, i64, i64)> {
    boxes(trace)
        .into_iter()
        .filter(|(p, _, _)| *p == 0)
        .map(|(_, field, (x, y))| (field, x.round() as i64, y.round() as i64))
        .collect()
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check places a field with a click, selects it \
             with two more, and then presses Ctrl+C, Ctrl+V and Ctrl+Shift+V — the three chords \
             being the subject. Reported as SKIPPED rather than passed: a check that did not run \
             has learned nothing.",
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
        .ok_or_else(|| Error::new("no --pdf. This check needs a page to place a field on."))?;
    let page: PageGeometry = match ctx.page_size {
        Some((w, h)) => PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&pdf).ok_or_else(|| {
            Error::new(format!(
                "cannot read a page size from {}. This check places its field in page fractions \
                 and needs the box to turn them into points. Pass --page-size WxH.",
                pdf.display()
            ))
        })?,
    };

    let mut spec = LaunchSpec::new(&exe, ctx.out("field-clipboard.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCE_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.env
        .push((ACCEPT_ENV.0.to_owned(), ACCEPT_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {} with PDFCE_DIAG_INVOKE={INVOKE}",
        exe.display(),
        session.pid()
    ));
    session.settle(45);
    let driver = Driver::new(session.window());

    // --- A: place one field -------------------------------------------------
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, 0)?;
    let at = DocPoint::new(0, PLACE_AT.0 * page.width_pt, PLACE_AT.1 * page.height_pt);
    driver.click_at(session.frame()?.to_screen(mapping.doc_to_window(at)?))?;
    session.settle(35);

    let trace = session.trace()?;
    let Some((_, field, centre)) = boxes(&trace).into_iter().find(|(p, _, _)| *p == 0) else {
        return Err(Error::new(format!(
            "the text-field tool placed nothing: a click on the page produced no `{BOX_LINE}` \
             line for page 1, so `edit.form_text_field` did not arm or the placement dialog did \
             not accept. That is two steps BEFORE the subject of this check, so it is a SKIP \
             rather than a failure of the clipboard. Trace: {}",
            session.trace_path().display()
        )));
    };
    report.note(format!(
        "★ placed the field {field:?} at canvas ({:.1}, {:.1})",
        centre.0, centre.1
    ));

    // --- B: disarm, clear, select ------------------------------------------
    //
    // ★ Escape first, or the next click places a SECOND field rather than
    // selecting one — the tool stays armed after a placement, as a markup pen
    // does. `field_menu`'s phase B carries the full argument for all three
    // steps and this is the same sequence, deliberately.
    driver.press(vk::ESCAPE)?;
    session.settle(12);

    let targets = crate::checks::formaim::targets(&trace);
    let blank =
        crate::checks::formaim::blank_canvas_point(&targets, page, 0, centre).ok_or_else(|| {
            Error::new(format!(
                "no blank paper could be found on page 1 near the field this check placed. \
                 Without a clearing click the field stays selected from authoring, and \
                 `canvas::forms::select_click` traces only on a CHANGE — so the selecting click \
                 below would land dead centre and announce nothing. A property of the document, \
                 not of the clipboard: SKIP. Candidates were around ({:.1}, {:.1}).",
                centre.0, centre.1
            ))
        })?;
    let blank_window =
        mapping.doc_to_window(DocPoint::new(0, blank.0, page.height_pt - blank.1))?;
    driver.click_at(session.frame()?.to_screen(blank_window))?;
    session.settle(20);
    if !session
        .trace()?
        .events(SELECTED)
        .any(|l| l.get("field").is_none())
    {
        return Ok(Some(format!(
            "THE SELECTION COULD NOT BE CLEARED: a click on blank paper at canvas ({:.1}, {:.1}) \
             produced no bare `{SELECTED}` line. A primary click on paper is an unambiguous \
             deselect, so this says the click never reached the form surface — which would make \
             every assertion below unreadable either way. Trace: {}",
            blank.0,
            blank.1,
            session.trace_path().display()
        )));
    }

    // The census is canvas space and `doc_to_window` takes PDF space, so the
    // flip is the one arithmetic this check does.
    let field_window =
        mapping.doc_to_window(DocPoint::new(0, centre.0, page.height_pt - centre.1))?;
    let field_screen = session.frame()?.to_screen(field_window);
    driver.click_at(field_screen)?;
    session.settle(25);
    if !session
        .trace()?
        .events(SELECTED)
        .any(|l| l.get("field").is_some_and(|f| f == field))
    {
        return Ok(Some(format!(
            "THE FIELD COULD NOT BE SELECTED: a click at its centre produced no `{SELECTED}` \
             line naming {field:?}, on a frame where the clearing click above proved the \
             selection channel is live. Trace: {}",
            session.trace_path().display()
        )));
    }
    report.note(format!("★ selected {field:?}"));

    let before = session.trace()?;
    let names_before = field_names(&before);
    let boxes_before = distinct_boxes(&before).len();
    report.note(format!(
        "before any paste: {} field name(s), {boxes_before} box(es) on page 1",
        names_before.len()
    ));

    // --- C: Ctrl+C ----------------------------------------------------------
    driver.press_chord(&[vk::CONTROL], vk::C)?;
    session.settle(20);
    let after_copy = session.trace()?;
    if after_copy.events(COPY_LINE).next().is_none() {
        return Ok(Some(format!(
            "THE DEFECT. Ctrl+C over a selected form field produced no `{COPY_LINE}` line. The \
             field WAS selected — the assertion above proved it — so the chord did not reach the \
             field path. Two readings the trace can tell apart: a `command-declined … \
             reason=text-owns-the-clipboard` line means rung 1 of `dispatch::clipboard`'s fork \
             took it; a `clipboard-copy kind=content` line means it fell through to the OBJECT \
             clipboard, which is the pre-2026-08-29 behaviour exactly. Trace: {}",
            session.trace_path().display()
        )));
    }
    report.note("★ Ctrl+C produced a fieldclip-copy line");

    // --- D: Ctrl+V — paste as a NEW field -----------------------------------
    driver.press_chord(&[vk::CONTROL], vk::V)?;
    session.settle(35);
    let after_new = session.trace()?;
    let new_paste = after_new
        .events(PASTE_LINE)
        .filter(|l| l.get("mode").is_some_and(|m| m == "NewField"))
        .count();
    if new_paste == 0 {
        return Ok(Some(format!(
            "Ctrl+V raised no `{PASTE_LINE} mode=NewField` line after a successful copy. The \
             clipboard held a field, so either the binding does not reach `edit.paste` or the \
             dispatcher's clipboard-kind branch did not take the field arm. Trace: {}",
            session.trace_path().display()
        )));
    }
    let names_after_new = field_names(&after_new);
    if names_after_new.len() <= names_before.len() {
        return Ok(Some(format!(
            "★★ Ctrl+V RAISED THE PASTE AND NOTHING ARRIVED. `{PASTE_LINE} mode=NewField` is in \
             the trace and page 1 still has {} distinct field name(s), the same as before. So \
             the shell asked and the document did not change — which is what a missing \
             `FieldAction::Paste` arm looks like, and is precisely the state a check reading \
             only the paste line would have passed. Names now: {:?}. Trace: {}",
            names_after_new.len(),
            names_after_new,
            session.trace_path().display()
        )));
    }
    // ★★★ WHAT it was called, not just that there is one more.
    //
    // The naming rule was wrong until 2026-08-29 — it produced `Text1 2` from
    // `Text1`, a space separator plus no awareness that the base was already
    // numbered — and a check counting NAMES would have passed against it
    // unchanged. Acrobat's sourced convention is a plain numeric suffix
    // continuing an existing number, so `Text1` -> `Text2`.
    //
    // ★ The assertion is on the SHAPE rather than the literal: it forbids the
    // two spellings that are wrong for a stated reason — a space (which breaks
    // the scripting rationale the convention exists for) and a dot (which is
    // the fully-qualified-name separator, so `Text.2` is a CHILD field, a
    // hierarchy nobody asked for). It does not pin `Text2` exactly, because the
    // base name comes from whatever the placement dialog chose and pinning it
    // would make this check fail the day that default changes.
    let fresh: Vec<&String> = names_after_new.difference(&names_before).collect();
    let Some(pasted) = fresh.first() else {
        return Ok(Some(format!(
            "the field count rose but no NEW name appeared, which should be impossible. Trace: {}",
            session.trace_path().display()
        )));
    };
    if pasted.contains('.') || pasted.contains(' ') {
        return Ok(Some(format!(
            "THE PASTED FIELD IS CALLED {pasted:?}, and neither spelling is allowed. A DOT is the fully-qualified-name separator (ISO 32000-1 12.7.3.2), so a dotted name makes the paste a CHILD field of the original rather than an independent one — a third shape, and not the one Ctrl+V promises. A SPACE breaks the reason the convention exists: Acrobat numbers duplicates `Date1`, `Date2` so a script can loop over fields sharing the non-number part of the name. Trace: {}",
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★ Ctrl+V added a new field named {pasted:?} (from {field:?}): {} name(s) -> {}",
        names_before.len(),
        names_after_new.len()
    ));

    // --- E: Ctrl+Shift+V — paste as a DUPLICATE -----------------------------
    //
    // ★ Against the clipboard written in step C, not against anything step D
    // left behind. An operator copies once and pastes several times, and a
    // clipboard that emptied itself on paste would break that.
    let boxes_before_dup = distinct_boxes(&after_new).len();
    driver.press_chord(&[vk::CONTROL, vk::SHIFT], vk::V)?;
    session.settle(35);
    let after_dup = session.trace()?;
    let dup_paste = after_dup
        .events(PASTE_LINE)
        .filter(|l| l.get("mode").is_some_and(|m| m == "Duplicate"))
        .count();
    if dup_paste == 0 {
        return Ok(Some(format!(
            "Ctrl+Shift+V raised no `{PASTE_LINE} mode=Duplicate` line, on a build where Ctrl+V \
             worked one step earlier. So the chord itself is the failure rather than the paste \
             path. ★ The most likely cause is the modifier, not the binding: winit derives its \
             modifier state from key EVENTS, and a synthesised `VK_SHIFT` — the 'either shift' \
             virtual key a real keyboard never sends — is not always recognised. `sys::vk::LSHIFT` \
             exists for exactly this and is the first thing to try. Trace: {}",
            session.trace_path().display()
        )));
    }
    let boxes_after_dup = distinct_boxes(&after_dup).len();
    if boxes_after_dup <= boxes_before_dup {
        return Ok(Some(format!(
            "★★★ Ctrl+Shift+V raised `{PASTE_LINE} mode=Duplicate` and page 1 still has \
             {boxes_after_dup} box(es), unchanged from {boxes_before_dup}. The duplicate paste \
             authors a field with the SOURCE'S OWN NAME and relies on `pdfce-core` merging it \
             into the existing field as a second widget (`edit.rs:13523`, `merged: true`). If \
             nothing arrived, that merge did not happen — which would mean the engine now \
             REFUSES a duplicate name rather than merging, and the central claim in \
             `canvas::fieldclip`'s header is false. Check the trace for a refusal naming \
             FieldNameTaken before assuming this harness is at fault. Trace: {}",
            session.trace_path().display()
        )));
    }
    let names_after_dup = field_names(&after_dup);
    if names_after_dup.len() != names_after_new.len() {
        return Ok(Some(format!(
            "★★★ THE DUPLICATE MADE A NEW FIELD. Boxes went {boxes_before_dup} -> \
             {boxes_after_dup}, which is right, but the number of distinct field NAMES went {} \
             -> {} — and a duplicate must add a box WITHOUT adding a name. That is the whole \
             difference between the two chords: Ctrl+Shift+V's promise to the operator is that \
             typing in either box fills both, and two names means two independent fields. Names \
             now: {:?}. Trace: {}",
            names_after_new.len(),
            names_after_dup.len(),
            names_after_dup,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★ Ctrl+Shift+V added a box without adding a name: {boxes_before_dup} -> \
         {boxes_after_dup} boxes, {} name(s) unchanged — which is the merge, observed from \
         outside the engine",
        names_after_dup.len()
    ));

    Ok(None)
}
