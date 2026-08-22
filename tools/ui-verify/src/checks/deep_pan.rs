//! `panning_at_deep_zoom_stays_where_it_was_put` — the operator's "it jumps
//! back" report, made falsifiable.
//!
//! # The report
//!
//! `OPERATOR_REQUESTS.md` O24, 2026-08-22:
//!
//! > *"is that the challenge I was running into trying to pan over a little bit
//! > at high zoom, but it would jump back to it's original location I panned
//! > from because I couldn't pan to the next point?"*
//!
//! Two claims in one sentence and they need separating, because they have
//! different causes and only one of them is a bug:
//!
//! | claim | would be |
//! |---|---|
//! | *"I couldn't pan to the next point"* | a **quantised** pan — the view refuses small movements and only moves in steps |
//! | *"it would jump back to its original location"* | a **reverting** pan — the view moves and is then put back |
//!
//! ★★ A quantised pan is what an `f32` scroll offset does when its
//! representable spacing exceeds the drag: `last - pan` rounds straight back to
//! `last`, so the view does not move at all. It looks like the drag was
//! ignored. A reverting pan is something actively re-setting the offset after
//! the drag — a different fault with a different fix.
//!
//! This check tells them apart by measuring the offset at three moments: before
//! the drag, immediately after, and several frames later.
//!
//! # ★★ Why it rolls the wheel rather than dragging
//!
//! The first version drag-panned with the primary button and reported the view
//! as stuck. That was a **harness** defect: `canvas::input::pan_delta` pans on
//! the middle button always and on the primary button only under the hand
//! tool, so a primary drag with the default Select tool correctly rubber-band
//! selected and correctly moved nothing. The check had measured a gesture the
//! application never offered, and blamed the application for not honouring it.
//!
//! It is recorded here rather than quietly fixed because it is the same shape
//! as the false layout report in `checks::driving::declared`'s header: a
//! measurement of the wrong thing is indistinguishable, from the verdict line,
//! from a real defect. **Ask what the check sampled before asking what is
//! broken.**
//!
//! The wheel is unconditional — no tool, no modifier, no button — so a view
//! that does not move under it is unambiguously the application's fault.

use crate::checks::driving;
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};

/// `VK_CONTROL`, held while the wheel rolls to make it a zoom.
const VK_CONTROL: u16 = 0x11;

/// ★★ **Where on the page to zoom INTO**, in PDF user-space points.
///
/// `banana.pdf` draws a banana at life size and, beside it, two banana cells at
/// **the same scale** — 0.85 pt and 0.17 pt across, with labels 0.085 pt tall.
/// At a zoom where the whole page fits a screen the pair covers roughly one
/// pixel. They are the reason the fixture exists and the only thing on it worth
/// magnifying.
///
/// The operator, 2026-08-22: *"You should try zooming into the two cells, then
/// try slightly panning to test. Right now you are just zooming into a blank
/// area on the canvas."* He was right — the check zoomed about the viewport
/// centre, which on this page is white paper, so every raster it exercised was
/// empty. The geometry assertions were still valid (a placement is a placement
/// whatever the pixels show), but nothing about the run resembled what he does
/// with it, and a screenshot oracle added later would have had nothing to look
/// at.
///
/// Located by rendering the region at 30x and measuring: the pair sits at
/// (540, 560), about 600 micrometres past the tip of the arrow that points at
/// them.
const CELLS_PT: (f64, f64) = (540.0, 560.0);

/// The zoom group on the status bar — its right end is `+`.
const ZOOM_REGION: &str = "status-group:zoom";

/// The canvas viewport, which the drag happens inside.
const CANVAS_REGION: &str = "canvas-viewport";

/// The canvas's own state line, read for the zoom it reports.
const CANVAS_EVENT: &str = "canvas";

/// The canvas's `f64` pan position — the only field that can measure this.
///
/// ★ Not `canvas`'s `off=` and not its `rect=`. Both are `f32`, and at the
/// zoom this check works at their representable spacing is larger than the
/// drag, so both would read "unchanged" against an application that panned
/// perfectly. See `canvas::trace::position`.
const POS_EVENT: &str = "canvas-pos";

/// How far to zoom in before the FIRST probe.
///
/// Lands around 100,000 %, which is past the whole-page raster ceiling and on
/// the `scroll` position tier — the tier the operator was on when he reported
/// this.
const PRESSES: usize = 16;

/// How many MORE presses before the second probe.
///
/// # ★★ Why the check probes twice
///
/// The position is owned by two different mechanisms at two different depths —
/// an `f32` scroll offset below the deep threshold and an `f64`
/// `viewer::deep::DeepAnchor` above it — and they are a hard branch, not a
/// re-parameterisation. A pan that works on one says nothing about the other.
///
/// ★ Probing only the deepest would have been the tempting choice and it would
/// have missed the operator's actual case, which was on the shallow tier. One
/// probe per mechanism is the minimum that can honestly claim panning works
/// "at high zoom".
const MORE_PRESSES: usize = 44;

/// How many wheel notches to roll.
///
/// # ★★ Why the wheel and not a drag
///
/// The first version of this check drag-panned with the primary button and
/// reported a stuck view. It was wrong: `canvas::input::pan_delta` pans on the
/// **middle** button always, and on the primary button only while the hand
/// tool is active. The default tool is Select, so a primary drag correctly
/// rubber-band selected and correctly did not move the view. The harness had
/// measured a gesture the application never claimed to honour, and blamed the
/// application.
///
/// The wheel is unconditional — no tool, no modifier, no button — so a view
/// that does not move under it is unambiguously the application's. It is also
/// what an operator reaches for first.
///
/// Three notches: small enough to be the "little bit" he described, large
/// enough that a working build moves visibly.
const NOTCHES: i32 = -3;

/// The page's own rect, on the `ui-rect` channel.
const PAGE_REGION: &str = "page";

/// How far a raster may sit from where its own region says it belongs, in
/// window logical points, before this is a defect.
///
/// ★ Not zero. The application computes the placement in `f32` from a page rect
/// that is itself `f32`, and the trace prints three decimals; the harness
/// recomputes in `f64` from those printed values. A fraction of a point of
/// disagreement is the arithmetic, not the defect. The defect it is looking for
/// is `render::strategy::region_for`'s grid step — half a viewport, several
/// hundred points — so there is four orders of magnitude between the noise and
/// the signal and no need to tune this finely.
const PLACEMENT_TOLERANCE_PT: f64 = 2.0;

/// See the module documentation.
pub struct PanningAtDeepZoomStaysWhereItWasPut;

impl Check for PanningAtDeepZoomStaysWhereItWasPut {
    fn name(&self) -> &'static str {
        "panning_at_deep_zoom_stays_where_it_was_put"
    }

    fn defect(&self) -> &'static str {
        "panning a little at high zoom does nothing, or moves and then jumps back to where it \
         started — so a point just off screen cannot be reached at all"
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

/// One reading of the canvas's `f64` position line.
#[derive(Debug, Clone)]
struct Pos {
    /// How far the view has been panned from the acting page's corner, in
    /// screen pixels.
    at: (f64, f64),
    /// `scroll` or `deep` — which mechanism owns the position.
    tier: String,
    /// Where the raster was actually painted, window logical points.
    paint: (f64, f64),
    /// The page-space rect that raster is a picture of, or `None` for a
    /// whole-page raster.
    region: Option<(f64, f64, f64, f64)>,
    /// The page's extent, same units as `region`.
    ext: (f64, f64),
}

/// The canvas's latest `f64` position line, parsed, **with the page rect from
/// the same reading**.
///
/// # ★★ Why the two must be read together
///
/// The first version of this check read the position here and the page rect at
/// the end of the probe. They came from different frames, so the placement
/// cross-check compared a paint rectangle recorded before the wheel against a
/// page rectangle recorded after it — and reported a 120-point placement error
/// that was exactly the wheel movement, against a correct build.
///
/// That is the third time in two days this harness has produced a confident,
/// specific, entirely wrong verdict by sampling two quantities at two moments.
/// The pairing is now structural: one function, one return value, and no way
/// for a caller to hold one without the other.
fn position(session: &Session, ui_rect: &str) -> Result<Option<(Pos, crate::geom::LRect)>> {
    let trace = session.trace()?;
    let Some(page_rect) = driving::declared(&trace, ui_rect, PAGE_REGION) else {
        return Ok(None);
    };
    Ok(trace.events(POS_EVENT).last().and_then(|l| {
        let pair = |k: &str| -> Option<(f64, f64)> {
            let (x, y) = l.get(k)?.split_once(',')?;
            Some((x.parse().ok()?, y.parse().ok()?))
        };
        let region = match l.get("region") {
            Some("none") | None => None,
            Some(v) => {
                let n: Vec<f64> = v.split(',').filter_map(|p| p.parse().ok()).collect();
                if n.len() == 4 {
                    Some((n[0], n[1], n[2], n[3]))
                } else {
                    return None;
                }
            }
        };
        Some((
            Pos {
                at: pair("at")?,
                tier: l.get("tier").unwrap_or("?").to_owned(),
                paint: pair("paint")?,
                region,
                ext: pair("ext")?,
            },
            page_rect,
        ))
    }))
}

/// ★★★ **The placement invariant, recomputed independently.**
///
/// The pixels on screen must be a picture of the page area they cover. The
/// application places a region raster with `render::region::region_on_screen`,
/// which is
///
/// ```text
/// screen.x = page_rect.min.x + region.llx / extent.x * page_rect.width()
/// screen.y = page_rect.min.y + (extent.y - region.ury) / extent.y * page_rect.height()
/// ```
///
/// — the y term flipped because page space is y-up and the canvas is y-down.
/// This recomputes it here, from the traced region and the page's own rect,
/// and compares against the traced paint rect.
///
/// It is an **independent** check rather than a restatement because the region
/// it reads is the one the *held texture* is a picture of. `OPERATOR_REQUESTS`
/// O24c was the placement being computed from the region the shell wanted
/// *next* instead, which differs exactly while a new raster is in flight —
/// most of the time during a pan. Under that defect the traced region still
/// describes the pixels, this formula still says where they belong, and the
/// two disagree by a whole grid step.
///
/// Returns the disagreement in window logical points.
fn placement_error(pos: &Pos, page_rect: crate::geom::LRect) -> Option<(f64, f64)> {
    let (llx, _lly, _urx, ury) = pos.region?;
    if pos.ext.0 <= 0.0 || pos.ext.1 <= 0.0 {
        return None;
    }
    let w = f64::from(page_rect.max.x - page_rect.min.x);
    let h = f64::from(page_rect.max.y - page_rect.min.y);
    let want_x = f64::from(page_rect.min.x) + llx / pos.ext.0 * w;
    let want_y = f64::from(page_rect.min.y) + (pos.ext.1 - ury) / pos.ext.1 * h;
    Some((pos.paint.0 - want_x, pos.paint.1 - want_y))
}

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
        .ok_or_else(|| Error::new("no --pdf. There is nothing to pan."))?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check zooms in and drags the canvas. Reported \
             as SKIPPED rather than passed.",
        ));
    }
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("deep-pan.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    let driver = Driver::new(session.window());

    let trace = session.trace()?;
    // The fixture's page box, for the document→window conversion that aims the
    // zoom at the cells. `--page-size` overrides it for a fixture whose
    // `/MediaBox` this harness cannot read.
    let page: crate::coords::PageGeometry = match ctx.page_size {
        Some((w, h)) => crate::coords::PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&pdf).ok_or_else(|| {
            Error::new(format!(
                "cannot read a page size from {}, so the cells cannot be aimed at. Pass                  --page-size WxH.",
                pdf.display()
            ))
        })?,
    };
    // ★ Asserted present but not clicked. The zoom is driven by Ctrl+wheel at
    // the cells (see `CELLS_PT`); this is the check that a document is open at
    // all, and its absence is a far clearer failure than a mapping built from a
    // canvas that was never laid out.
    driving::declared(&trace, ui_rect, ZOOM_REGION)
        .ok_or_else(|| Error::new(format!("no `{ZOOM_REGION}`; is a document open?")))?;
    let canvas = driving::declared(&trace, ui_rect, CANVAS_REGION)
        .ok_or_else(|| Error::new(format!("no `{CANVAS_REGION}`; is a document open?")))?;
    let frame = session.frame()?;

    // ★★★ ZOOM AT THE CELLS, not at the middle of the sheet.
    //
    // Ctrl+wheel is zoom-about-the-pointer, so the point under the cursor stays
    // put: aim once and every subsequent notch magnifies the same content. The
    // `+` button zooms about the viewport centre, which on this page is blank
    // paper. See `CELLS_PT`.
    let mapping = crate::coords::CanvasMapping::from_trace(&trace, vocab, page, 0)?;
    let at_cells = frame
        .to_screen(mapping.doc_to_window(crate::coords::DocPoint::new(0, CELLS_PT.0, CELLS_PT.1))?);
    report.note(format!(
        "zooming at the cells — page ({}, {})",
        CELLS_PT.0, CELLS_PT.1
    ));
    driver.scroll_at_held(at_cells, &[VK_CONTROL], 1, PRESSES)?;
    session.settle(60);

    let zoom = session
        .trace()?
        .events(CANVAS_EVENT)
        .last()
        .and_then(|l| l.get_f32("zoom"))
        .unwrap_or(0.0);
    report.note(format!("zoomed to {:.0}%", zoom * 100.0));

    if let Some(bad) = probe(&session, &driver, &frame, canvas, ui_rect, report)? {
        return Ok(Some(bad));
    }

    // …and again on the other side of the deep threshold, where a different
    // mechanism owns the position entirely. See `MORE_PRESSES`.
    driver.scroll_at_held(at_cells, &[VK_CONTROL], 1, MORE_PRESSES)?;
    session.settle(120);
    if let Some(bad) = probe(&session, &driver, &frame, canvas, ui_rect, report)? {
        return Ok(Some(bad));
    }
    Ok(None)
}

/// Roll the wheel once over the canvas and report what the view did.
///
/// Returns `Ok(None)` when the view moved and stayed moved, and
/// `Ok(Some(verdict))` when it did not — the two failure shapes described in
/// this module's header.
///
/// ★ Takes the report so its notes carry BOTH probes' numbers. A verdict that
/// says "the view did not move" without saying which tier it was on sends the
/// next reader to whichever of the two files they guess first.
fn probe(
    session: &Session,
    driver: &Driver,
    frame: &crate::coords::WindowFrame,
    canvas: crate::geom::LRect,
    ui_rect: &str,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    let zoom = session
        .trace()?
        .events(CANVAS_EVENT)
        .last()
        .and_then(|l| l.get_f32("zoom"))
        .unwrap_or(0.0);

    let Some((first, rect_first)) = position(session, ui_rect)? else {
        return Err(Error::new(
            "the canvas never reported a `canvas-pos` line, so there is nothing to compare. \
             Either the build predates the f64 position trace or no page was drawn. SKIPPED.",
        ));
    };

    let (before, tier) = (first.at, first.tier.clone());

    driver.scroll_at(frame.declared_at(canvas, 0.5, 0.5), NOTCHES)?;
    session.settle(20);
    let Some((mid, rect_mid)) = position(session, ui_rect)? else {
        return Err(Error::new(
            "the canvas stopped reporting a position. SKIPPED.",
        ));
    };

    // ★ And again several frames later. A view that moves and is then put back
    // is a different defect from one that never moved, and only a second
    // reading can tell them apart — the operator described both in one
    // sentence, so the check must be able to say which he saw.
    session.settle(90);
    let Some((last, rect_last)) = position(session, ui_rect)? else {
        return Err(Error::new(
            "the canvas stopped reporting a position. SKIPPED.",
        ));
    };

    let (after, settled) = (mid.at, last.at);
    report.note(format!(
        "at {:.0}% on tier `{tier}`: {before:?} → {after:?} → {settled:?}",
        zoom * 100.0
    ));

    let moved = (after.0 - before.0).abs() + (after.1 - before.1).abs();
    // One notch is tens of pixels on every platform this runs on; a build that
    // moved by less than a single pixel did not move.
    if moved < 1.0 {
        return Ok(Some(format!(
            "★★ THE VIEW DID NOT MOVE. {NOTCHES} wheel notches at {:.0}% zoom left the position \
             at {before:?} (tier `{tier}`, moved {moved:.3} px). This is the operator's \"I \
             couldn't pan to the next point\". On the `scroll` tier it means the `f32` offset's \
             representable spacing exceeds the movement, so `last - pan` rounds back to `last`; \
             on the `deep` tier it means the wheel is not reaching `DeepAnchor::panned`.",
            zoom * 100.0
        )));
    }

    let reverted = (settled.0 - before.0).abs() + (settled.1 - before.1).abs();
    if reverted < moved / 2.0 {
        return Ok(Some(format!(
            "★★ THE VIEW REVERTED. The wheel moved the position from {before:?} to {after:?} \
             at {:.0}% on tier `{tier}`, and ninety frames later it is back at {settled:?}. This \
             is the operator's \"it would jump back to its original location\" — something is \
             re-setting the position after the gesture. Read what forces one every frame: \
             `zoom::consume_anchor`, `find::take_reveal_offset` and `strip::page_scroll_offset`, \
             and the last of those fires whenever the current-page reading disagrees with the \
             tracked one.",
            zoom * 100.0
        )));
    }

    // ★★★ O24c — THE PIXELS MUST BE A PICTURE OF WHERE THEY ARE DRAWN.
    //
    // Checked at all three readings, because the window in which it fails is
    // exactly the window in which a new region's raster is in flight: check
    // only the settled one and the defect is invisible, which is how it
    // shipped. See `placement_error`.
    //
    // ★ `scroll` tier only. Above the deep threshold the placement comes from
    // the `f64` anchor rather than from the page's rect, and this formula does
    // not describe it — comparing anyway would report a defect that is only a
    // wrong model.
    for (when, pos, page_rect) in [
        ("before", &first, rect_first),
        ("after", &mid, rect_mid),
        ("settled", &last, rect_last),
    ] {
        if pos.tier != "scroll" {
            continue;
        }
        if let Some((ex, ey)) = placement_error(pos, page_rect)
            && (ex.abs() > PLACEMENT_TOLERANCE_PT || ey.abs() > PLACEMENT_TOLERANCE_PT)
        {
            return Ok(Some(format!(
                "★★ THE PIXELS ARE IN THE WRONG PLACE ({when} the wheel, at {:.0}%). The raster \
                 on screen is a picture of page region {:?}, which belongs at the rect this \
                 harness recomputed, but it was painted {ex:.1},{ey:.1} points away from there. \
                 This is the operator's \"if I pan a little too far it jumps back in the \
                 opposite direction\": `canvas::show` is placing the held texture by the region \
                 the shell wants NEXT rather than by the one those pixels are OF, and the two \
                 differ by `render::strategy::region_for`'s half-viewport grid step whenever a \
                 raster is in flight. See `render::worker::RenderKey::region`.",
                zoom * 100.0,
                pos.region
            )));
        }
    }

    Ok(None)
}
