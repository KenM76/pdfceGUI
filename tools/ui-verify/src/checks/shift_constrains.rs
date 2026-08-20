//! `shift_constrains_a_resize` — **Shift preserves aspect**, driven end to end,
//! and proved by the *difference* between two drags in one process.
//!
//! # What this is for
//!
//! `ui-conventions/drag-moves.md` D5 — *"modifiers constrain, and the
//! constraint is announced"* — was found **absent from every drag in this
//! shell** by the conventions sweep of 2026-08-20, and Shift-preserves-aspect
//! is the sharpest instance: it is *the* resize convention, present in every
//! program in the class for thirty years. An operator who holds Shift and gets
//! a free-form resize does not conclude that pdfce chose differently; they
//! conclude it is broken.
//!
//! # ★★ Why this is a PAIR of drags and not one
//!
//! Because the only trustworthy oracle for a constraint is a **comparison**.
//!
//! A single constrained drag reporting `sx == sy` proves nothing on its own:
//! the south-east grip dragged along a rough diagonal produces near-equal
//! factors anyway, so a build that ignored Shift entirely could satisfy that
//! assertion by luck and would then satisfy it in CI for ever. What cannot
//! happen by luck is *the same travel producing unequal factors without the key
//! and equal ones with it*.
//!
//! So the check drags **deliberately lopsided** — far on x, barely on y — twice
//! from the same grip, with an undo between them, and asserts:
//!
//! | | assertion | what a wrong build does |
//! |---|---|---|
//! | 1 | unmodified: `sx` and `sy` differ materially | if they do not, the fixture or the travel is wrong and the check SKIPS rather than passing — a check that cannot distinguish must not claim to |
//! | 2 | with Shift: `sx == sy` | a build that ignores the modifier reproduces run 1's unequal pair |
//! | 3 | with Shift: the kept factor is the **dominant** one | a build that averaged, or that always took `sx`, passes 2 and fails here on the run where y dominates |
//! | 4 | a `constrain lock=Aspect` line was traced | the arithmetic happened and the operator was told; D5's second clause |
//!
//! Assertion 3 is the one that earns the lopsided travel. `aspect` keeps the
//! factor further from unity, and the cheapest wrong implementation — *"take
//! sx"* — is indistinguishable from the right one on any drag where x happens
//! to dominate. The check therefore drags **x-dominant** and asserts the kept
//! factor equals the x factor of the *unconstrained* run, which is a number the
//! wrong build has no way to produce for a y-dominant travel.
//!
//! # ★ Why the trace and not the pixels
//!
//! This project's standing rule is that *a trace can say the verb ran and
//! cannot say the screen changed*, and that every layout, repaint or clipping
//! defect has exactly one oracle: a rendered screenshot.
//!
//! This is not a layout defect. What is being asserted is **arithmetic that
//! reaches the engine** — `resize-commit` carries the two factors and
//! `move-nodes` proves they became an edit — and a screenshot could not
//! distinguish `sx=1.42, sy=1.42` from `sx=1.42, sy=1.39` on any shape an
//! operator would draw. A capture is nonetheless attached on the failure
//! branch, because the one thing this check cannot see is whether the *caption*
//! rendered, and a human reading a failure will want to look.
//!
//! # What it does NOT cover, said rather than implied
//!
//! The three axis-locked drags — a move, a perimeter vertex and a Bézier
//! handle — are not driven here. They share `constrain::axis` with this one and
//! it is unit-tested, but *sharing a function is not the same as reaching it*,
//! and this project has shipped three features whose parts were all correct and
//! whose join was unobserved. Named as a gap in `OPERATOR_REQUESTS.md` O14
//! rather than left to be discovered.

use crate::checks::driving::{self, SHELL_DIAG_ENV, click_mode_segment};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::{Driver, Key};
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The mode whose canvas may select page content.
const MODE: &str = "edit";
/// `resize-commit grip=… sx=… sy=… ax=… ay=…` — the shell's own report.
const COMMIT_EVENT: &str = "resize-commit";
/// `resize-declined reason=…` — the six worded refusals.
const DECLINED_EVENT: &str = "resize-declined";
/// `constrain lock=…` — traced once per transition by `canvas::constrain`.
const CONSTRAIN_EVENT: &str = "constrain";
/// The region the selection outline publishes.
const OUTLINE_REGION: &str = "canvas.selection-outline";

/// How far to drag the grip on x, in screen pixels.
///
/// ★ Deliberately far more than [`DRAG_Y_PX`]. See the module header: a
/// lopsided travel is what makes assertions 1 and 3 able to fail.
const DRAG_X_PX: f32 = 90.0;
/// How far to drag the grip on y, in screen pixels.
const DRAG_Y_PX: f32 = 12.0;

/// How different the two unconstrained factors must be before this check will
/// claim to have measured anything.
///
/// Below this the drags are effectively square, assertion 2 could pass by luck,
/// and the honest outcome is SKIP. Chosen as five per cent because the intended
/// travel is 7.5:1 and anything close to square means the selection's aspect
/// ratio swallowed the lopsidedness — a fact about the fixture, not the build.
const MIN_DISCRIMINATION: f64 = 0.05;

/// See the module documentation.
pub struct ShiftConstrainsAResize;

impl Check for ShiftConstrainsAResize {
    fn name(&self) -> &'static str {
        "shift_constrains_a_resize"
    }

    fn defect(&self) -> &'static str {
        "Shift does not preserve aspect on a resize — *the* resize convention, present in every \
         program in the class, and absent from every drag in this shell until 2026-08-20. An \
         operator holding Shift gets a free-form resize and cannot tell whether the key did \
         anything"
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

/// One completed resize's two factors, read off `resize-commit`.
#[derive(Debug, Clone, Copy)]
struct Factors {
    sx: f64,
    sy: f64,
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
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new("no --pdf. This check needs a drawing with a selectable shape on page 1.")
    })?;
    let target = ctx.target.ok_or_else(|| {
        Error::new(
            "no --doc-point. Pass PAGE,X,Y in PDF user space naming a point where the fixture \
             has a selectable SHAPE — not text and not a picture, both of which the resize \
             refuses by name. There is deliberately no default: a click on empty page is \
             symptom-identical to a broken hit test.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a mode segment, clicks page \
             content and performs two grip drags, one of them with Shift held. Reported as \
             SKIPPED rather than passed.",
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

    let mut spec = LaunchSpec::new(&exe, ctx.out("shift-constrains.trace.txt"));
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

    // --- 1: Edit, the one mode whose canvas selects content ----------------
    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(20);

    // --- 2: select the shape -----------------------------------------------
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, vocab, page, target.page)?;
    let window_point = mapping.doc_to_window(DocPoint::new(target.page, target.x, target.y))?;
    let frame = session.frame()?;
    let at = frame.to_screen(window_point);
    driver.click_at(at)?;
    session.settle(16);

    let trace = session.trace()?;
    let selected = trace
        .last(vocab.click_event)
        .and_then(|l| l.get_usize(vocab.click_selection_field))
        .or_else(|| {
            trace
                .last(vocab.canvas_event)
                .and_then(|l| l.get_usize(vocab.canvas_selection_field))
        });
    if selected == Some(0) {
        return Err(Error::new(format!(
            "the click at (page {}, {:.1}, {:.1}) selected nothing, so there are no grips to \
             drag. A fact about the fixture and the point, not about the constraint — aim at a \
             shape. SKIPPED rather than FAILED for exactly that reason.",
            target.page + 1,
            target.x,
            target.y
        )));
    }
    report.note("the click selected a shape, so the outline and its grips are drawn");

    // --- 3: the unconstrained drag, which is the CONTROL --------------------
    let free = match one_drag(&session, &driver, ui_rect, None)? {
        Ok(f) => f,
        Err(why) => return Ok(Some(why)),
    };
    report.note(format!(
        "unmodified: sx={:.4} sy={:.4} — the control",
        free.sx, free.sy
    ));
    let spread = (free.sx - free.sy).abs();
    if spread < MIN_DISCRIMINATION {
        return Err(Error::new(format!(
            "the unconstrained drag produced sx={:.4} and sy={:.4}, which differ by only \
             {spread:.4}. This check proves a constraint by the DIFFERENCE between a free drag \
             and a locked one, so a control run that is already square cannot discriminate: a \
             build that ignored Shift entirely would pass. That is a fact about this shape's \
             aspect ratio against a {DRAG_X_PX:.0}×{DRAG_Y_PX:.0} px travel, not about the \
             build, so it is SKIPPED rather than passed. Aim at a shape whose selection box is \
             not far taller than it is wide.",
            free.sx, free.sy
        )));
    }

    // ★ Put the shape back before measuring again. Without this the second drag
    // starts from the *resized* box, so its factors are relative to different
    // extents and the two runs are not comparable — the check would then be
    // asserting something true about two different objects.
    driver.press_chord(&[crate::sys::vk::CONTROL], crate::sys::vk::Z)?;
    session.settle(20);

    // --- 4: the same travel, with Shift held throughout ---------------------
    let locked = match one_drag(&session, &driver, ui_rect, Some(Key::Shift))? {
        Ok(f) => f,
        Err(why) => return Ok(Some(why)),
    };
    report.note(format!(
        "★ with Shift: sx={:.4} sy={:.4}",
        locked.sx, locked.sy
    ));

    // --- 5: assertion 2 — the two factors are the same ----------------------
    //
    // Compared against a tolerance rather than for exact equality: both numbers
    // arrive through a `{:.4}` format in the trace, so two values that ARE the
    // same `f32` can print a unit apart in the last place.
    if (locked.sx - locked.sy).abs() > 1e-3 {
        let shot = ctx.out("shift-constrains-aspect.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        return Ok(Some(format!(
            "★ SHIFT DID NOT PRESERVE ASPECT. The same {DRAG_X_PX:.0}×{DRAG_Y_PX:.0} px drag \
             gave sx={:.4} sy={:.4} unmodified and sx={:.4} sy={:.4} with Shift held — the two \
             are the same shape of answer, so the modifier changed nothing.\n\
             Look at `canvas::interact`'s `GestureOutcome::Resize` arm: the flag reaches \
             `resizing::Frame::constrain`, and `resizing::drag` applies \
             `constrain::aspect` between `factors` and the in-flight return. If the lock were \
             applied at the CALL SITE instead, the ghost would be constrained and the commit \
             would not — which is this failure exactly. Trace: {}.",
            free.sx,
            free.sy,
            locked.sx,
            locked.sy,
            session.trace_path().display()
        )));
    }

    // --- 6: assertion 3 — it kept the DOMINANT factor -----------------------
    //
    // The travel is x-dominant, so the kept factor must be the control run's
    // `sx`. A build that averaged the pair, or that took `sy`, or that took the
    // factor closer to unity, satisfies assertion 2 and fails here.
    if (locked.sx - free.sx).abs() > 0.02 {
        return Ok(Some(format!(
            "★ SHIFT LOCKED THE PROPORTION TO THE WRONG FACTOR: it kept {:.4}, and the drag's \
             dominant axis produced {:.4} unmodified.\n\
             `constrain::aspect` keeps the factor FURTHER FROM UNITY, which is the same thing \
             as the axis the pointer travelled furthest along relative to the box's own \
             extent. Keeping the other one shrinks a shape the operator was enlarging. \
             Averaging the pair — the other plausible wrong answer — lands between the two and \
             fails this same assertion. Trace: {}.",
            locked.sx,
            free.sx,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★ the locked resize kept the dominant factor: {:.4}, the same the free drag produced \
         on x",
        locked.sx
    ));

    // --- 7: assertion 4 — and the operator was TOLD -------------------------
    let trace = session.trace()?;
    let announced = trace
        .events(CONSTRAIN_EVENT)
        .filter_map(|l| l.get("lock").map(str::to_owned))
        .any(|l| l == "Aspect");
    if !announced {
        let shot = ctx.out("shift-constrains-caption.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        return Ok(Some(format!(
            "the aspect lock WORKED and was never announced: no `{CONSTRAIN_EVENT} lock=Aspect` \
             line was traced.\n\
             `drag-moves` D5 has two clauses and this is the second — *the affordance shows the \
             constraint while it is active* — whose stated failure mode is an operator who \
             holds Shift, gets a result they did not expect, and cannot tell whether the \
             modifier did anything. `constrain::resize` is what announces; a caller that passed \
             the raw modifier straight to `Frame::constrain` would behave correctly and say \
             nothing. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("★ and it was announced — `constrain lock=Aspect` reached the status row");
    Ok(None)
}

/// Perform one south-east grip drag and return the factors it committed.
///
/// The grip is re-located from the selection outline **on every call**, not
/// cached: the first drag changes the object's extent, so a cached corner would
/// aim the second drag at the interior — which is a MOVE, and would produce no
/// `resize-commit` at all.
///
/// `Ok(Err(String))` is a check failure with its sentence already written;
/// `Err` is a skip.
fn one_drag(
    session: &Session,
    driver: &Driver,
    ui_rect: &'static str,
    modifier: Option<Key>,
) -> Result<std::result::Result<Factors, String>> {
    let trace = session.trace()?;
    let outline = driving::declared(&trace, ui_rect, OUTLINE_REGION).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{OUTLINE_REGION}` region, so the harness does not \
             know where the grips are. It refuses to guess: a guessed grip position lands \
             inside the object, which is a MOVE drag, and this check would then measure the \
             wrong gesture."
        ))
    })?;
    let frame = session.frame()?;
    // `declared_at(1.0, 1.0)` — the bottom-right corner, where `handles` centres
    // the south-east grip. Not the centre: that is `Grip::Move`.
    let from = frame.declared_at(outline, 1.0, 1.0);
    let w = (outline.max.x - outline.min.x).max(1.0);
    let h = (outline.max.y - outline.min.y).max(1.0);
    let to = frame.declared_at(outline, 1.0 + DRAG_X_PX / w, 1.0 + DRAG_Y_PX / h);
    // A mid-point so the drag passes through frames where the constraint is
    // live rather than teleporting from press to release. `drag_via`'s own
    // header makes the same argument for holding the modifier throughout.
    let via = frame.declared_at(
        outline,
        1.0 + DRAG_X_PX / (2.0 * w),
        1.0 + DRAG_Y_PX / (2.0 * h),
    );

    let before = session.trace()?.events(COMMIT_EVENT).count();
    driver.drag_via(
        from,
        via,
        std::time::Duration::from_millis(60),
        to,
        modifier,
    )?;
    session.settle(30);

    let trace = session.trace()?;
    let Some(commit) = trace.events(COMMIT_EVENT).nth(before) else {
        let declined = trace
            .events(DECLINED_EVENT)
            .filter_map(|l| l.get("reason").map(str::to_owned))
            .last();
        return Ok(Err(match declined {
            Some(reason) => format!(
                "the grip drag was DECLINED: reason={reason}. `NotAPath` means the point aimed \
                 at text or a picture; `ManyObjects` means the click selected more than one. \
                 Both are honest refusals and neither is what this check is for. Trace: {}.",
                session.trace_path().display()
            ),
            None => format!(
                "the grip drag committed nothing and declined nothing — the state the whole \
                 resize feature is a fix for. Before asking about the constraint, look at \
                 `resize_scales_a_shape`, which measures that alone. Trace: {}.",
                session.trace_path().display()
            ),
        }));
    };
    // A missing or unparsable field answers 0.0, which fails every assertion
    // downstream — the safe direction: a check that could not read a number must
    // not report that the number was right.
    Ok(Ok(Factors {
        sx: commit.get("sx").and_then(|v| v.parse().ok()).unwrap_or(0.0),
        sy: commit.get("sy").and_then(|v| v.parse().ok()).unwrap_or(0.0),
    }))
}
