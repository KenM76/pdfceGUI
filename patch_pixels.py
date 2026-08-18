import io

p = 'tools/ui-verify/src/checks/text_annot_focus.rs'
s = io.open(p, encoding='utf-8').read()

# --- imports ---------------------------------------------------------------
old = "use crate::coords::{CanvasMapping, DocPoint, PageGeometry};"
assert s.count(old) == 1
s = s.replace(old, "use crate::coords::{CanvasMapping, DocPoint, LRect, PageGeometry};", 1)

# --- capture BEFORE the drag ----------------------------------------------
old = """    // --- B: drag the box ---------------------------------------------------
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, target.page)?;
    let frame = session.frame()?;
    let from = frame.to_screen(mapping.doc_to_window(target)?);
    let to = frame.to_screen(mapping.doc_to_window(DocPoint {
        page: target.page,
        x: target.x + BOX_PT,
        y: target.y + BOX_PT,
    })?);
    driver.drag(from, to)?;
    session.settle(20);"""
new = """    // --- B: capture the paper, then drag the box ---------------------------
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, target.page)?;
    let frame = session.frame()?;
    let corner = mapping.doc_to_window(target)?;
    let far = mapping.doc_to_window(DocPoint {
        page: target.page,
        x: target.x + BOX_PT,
        y: target.y + BOX_PT,
    })?;
    // The box in the window's own logical coordinates, inset so the assertion
    // is about the annotation's INTERIOR. Its border would answer the pixel
    // question on its own, and a border is the one part `spec` draws
    // unconditionally — an empty box with a frame round it would pass a test
    // that sampled the edge, which is the failure being investigated.
    let box_rect = inset(LRect::from_corners(corner, far), BORDER_INSET_LOGICAL);

    let before_path = ctx.out("text_annot_focus.before.png");
    let before_shot = crate::capture::window_to_png(&session, &before_path)?;
    report.artifact(before_path);

    let from = frame.to_screen(corner);
    let to = frame.to_screen(far);
    driver.drag(from, to)?;
    session.settle(20);"""
assert s.count(old) == 1
s = s.replace(old, new, 1)

# --- the pixel assertion after Accept --------------------------------------
old = """    report.note(format!(
        "the unclicked dialog took the keystrokes: Accept authored, {before} -> {after} \\
         annotation(s)"
    ));
    Ok(None)
}"""
new = """    report.note(format!(
        "the unclicked dialog took the keystrokes: Accept authored, {before} -> {after} \\
         annotation(s)"
    ));

    // --- E: ★★ AND IT IS ON THE PAGE ---------------------------------------
    //
    // `add-text-annot` says the funnel ran. It does not say anything is
    // VISIBLE, and the operator's report — *"nothing gets added"* — is about
    // what they can see. An annotation authored without a usable appearance
    // renders as nothing at all, and every trace assertion above passes on
    // exactly that build. `D:/dev/rag/egui` states the rule this obeys:
    // **a rendering defect has one oracle, and it is a rendered pixel.**
    //
    // The pointer is parked away from the box first, for `markup_rectangle`'s
    // reason: a capture taken with the cursor over the region would measure a
    // hover as well as an annotation.
    driver.move_to(frame.to_screen(mapping.doc_to_window(DocPoint {
        page: target.page,
        x: target.x + BOX_PT * 2.0,
        y: target.y,
    })?))?;
    session.settle(10);
    let after_path = ctx.out("text_annot_focus.after.png");
    let after_shot = crate::capture::window_to_png(&session, &after_path)?;
    report.artifact(after_path);

    let px = frame.logical_to_capture_pixels(box_rect);
    if px.area() == 0 {
        return Err(Error::new(
            "the placed box resolves to no pixels of the captured client area, so there is \\
             nothing to look at. The --doc-point is probably off the visible page.",
        ));
    }
    let changed = changed_pixels(&before_shot, &after_shot, px);
    let total = px.area();
    let ratio = f64::from(changed) / f64::from(total);
    if ratio < MIN_CHANGED_RATIO {
        return Ok(Some(format!(
            "the annotation was AUTHORED and is not VISIBLE. `add-text-annot` traced and the \\
             count went {before} -> {after}, but only {changed} of {total} pixels \\
             ({:.2}%) inside the placed box differ from the same region before the drag — \\
             below the {:.0}% floor. The operator's words were *\\"nothing gets added\\"*. \\
             Every trace assertion in this check and in \\
             `text_annot_places_and_authors` passes on this build, because they measure \\
             whether the engine was CALLED. Look for a missing or empty appearance stream, \\
             an ink colour equal to the paper, or a page texture that was not invalidated.",
            ratio * 100.0,
            MIN_CHANGED_RATIO * 100.0
        )));
    }
    report.note(format!(
        "and it is on the page: {changed} of {total} pixels ({:.1}%) inside the box changed",
        ratio * 100.0
    ));
    Ok(None)
}

/// Shrink a rect on every side, in logical units.
fn inset(r: LRect, by: f64) -> LRect {
    LRect::from_corners(
        crate::coords::LPoint::new(r.min_x() + by, r.min_y() + by),
        crate::coords::LPoint::new(r.max_x() - by, r.max_y() - by),
    )
}

/// How far inside the placed box to sample, in logical units.
///
/// `canvas::textannot::spec` gives a text box a 1 pt border, deliberately —
/// *"on a drawing sheet a borderless caption is indistinguishable from the
/// sheet"*. That border is drawn whether or not the words are, so a sample
/// that included the edge would report "something appeared" for an empty box,
/// which is the exact state under investigation. Six logical units clears a
/// 1 pt border at any zoom this harness runs at.
const BORDER_INSET_LOGICAL: f64 = 6.0;

/// The share of sampled pixels that must differ for the annotation to count as
/// drawn.
///
/// Two characters of 12 pt text in a 220 pt box cover well under a percent of
/// it, so this floor is deliberately tiny — it separates *nothing at all* from
/// *something*, and is not a measurement of how much was drawn. Anti-aliasing
/// and the canvas's own re-render jitter are what it has to clear.
const MIN_CHANGED_RATIO: f64 = 0.001;

/// Count pixels of `region` that differ between two captures.
///
/// A plain per-channel threshold rather than a perceptual difference: the
/// question is *"did anything appear here"*, and text on paper is a large
/// contrast wherever it lands. The threshold exists only to ignore the
/// one-or-two-level noise a re-render produces on identical content.
fn changed_pixels(before: &crate::image::Image, after: &crate::image::Image, region: crate::coords::PixRect) -> u32 {
    let mut n = 0;
    for y in region.y..region.y.saturating_add(region.h) {
        for x in region.x..region.x.saturating_add(region.w) {
            let (Some(a), Some(b)) = (before.pixel(x, y), after.pixel(x, y)) else {
                continue;
            };
            let d = u16::from(a.r.abs_diff(b.r))
                .max(u16::from(a.g.abs_diff(b.g)))
                .max(u16::from(a.b.abs_diff(b.b)));
            if d > 12 {
                n += 1;
            }
        }
    }
    n
}"""
assert s.count(old) == 1
s = s.replace(old, new, 1)

io.open(p, 'w', encoding='utf-8').write(s)
print('ok')
