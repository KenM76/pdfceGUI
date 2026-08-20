//! # `canvas::textedit::paint` — **what a draft looks like on the page**
//!
//! ## Why this is its own file
//!
//! R2, on 2026-08-20, when the in-place editor pushed `canvas::textedit` past
//! 1,500 lines. A real seam: everything here answers one question — *what does
//! an operator see while they are composing?* — and nothing here takes a
//! keystroke, resolves a click or builds an action.
//!
//! ## The one thing to know before changing anything in here
//!
//! **The draft is drawn ONCE and measured ONCE.** The editor box's text and the
//! caret's position come from the same string, the same `FontId` and the same
//! size, in that order, a few lines apart. That is not tidiness — it is the
//! defect this file was rewritten to remove.
//!
//! The caret used to be derived from the page's own glyph advances, which was
//! right while the page's glyphs were the only thing on screen and became wrong
//! the moment a preview was drawn in a different font: the caret would sit
//! somewhere other than between the characters the operator could see, and
//! drift further with every keystroke.
//!
//! Two derivations of one position, agreeing at first and separating under use,
//! is the same class of defect as the vertex drag that tracked at `1/zoom` and
//! the snap marker that sat off by the scroll origin. This project has now met
//! it three times. **If you add anything to this file that needs to know where
//! a character is, measure it from the layout — never from the document.**
//!
//! ## Rule 4 lives here too
//!
//! An in-place editor covers applied content while it is open. It does not
//! restyle it, mark it, tint it or flag it — and the moment it closes, what
//! replaces it is `pdfce-render`'s output with no marking of any kind. See
//! [`preview`]'s own header for the argument against D4a's ghost text and why
//! an opaque editor is a different thing from a translucent one.

use egui::{Pos2, Ui};

use super::{Anchor, Draft, Preview, read};
use crate::app::state::OpenDoc;

/// The smallest the in-place editor's text may be drawn, in points.
///
/// A 4 pt note at 25 % zoom is a box two pixels high, and an operator cannot
/// type into a line. The box grows past the run it covers rather than becoming
/// illegible — the alternative is a preview that technically exists.
const MIN_PREVIEW_PT: f32 = 11.0;

/// The largest, in points. A title at 400 % zoom would otherwise fill the canvas
/// with a single word.
const MAX_PREVIEW_PT: f32 = 40.0;

/// How much of the editor box's height the glyphs take, leaving the rest as
/// leading. A cap height is roughly 70 % of a line box, and text set at the full
/// box height sits on the edges and reads as cramped.
const PREVIEW_FILL: f32 = 0.72;

/// How far in from the editor box's left edge the text starts, in points.
///
/// Shared by the text and the caret, so a caret at index 0 sits exactly where
/// the first character does rather than a hair to one side of it.
const PREVIEW_INSET_PT: f32 = 2.0;

/// **Draw the in-place editor: what you are typing, where you are typing it.**
///
/// ## ★★★ D4a's ghost text, the decision that followed it, and why that
/// ## decision was half-right
///
/// The old shell drew the draft *as text*, in an `egui` proportional font, over
/// a **translucent mask** — which `DEFECTS.md` D4a names as the second
/// contributor to "weird": *"you type in the wrong typeface at the wrong
/// widths, then it snaps to reality on Accept."*
///
/// This module's answer, until 2026-08-20, was to draw **no glyphs at all** —
/// a caret and a bracket, with this promise attached:
///
/// > *"The characters themselves are shown off-canvas, in the status bar, where
/// > `text::textedit` owns the sentence."*
///
/// **That promise was never kept.** `text::tool::text_edit_live` says *"Enter
/// commits what you have typed. Esc abandons it."* and nothing anywhere renders
/// the draft. So the operator typed into a bracket and their characters
/// appeared nowhere at all:
///
/// > *"I can edit text now, but there is no live preview of that either."*
/// > — 2026-08-20
///
/// ## Why an in-place editor BOX is not the ghost D4a condemns
///
/// The distinction is not cosmetic and it is the whole justification for
/// reversing the decision:
///
/// | | old ghost | this |
/// |---|---|---|
/// | drawn | translucent, **over** the original glyphs | **opaque**, covering them |
/// | reads as | the document, in the wrong typeface | an editor, obviously |
/// | on commit | "it snapped to reality" | the editor closed |
///
/// D4a's defect is that the ghost **imitated applied content**. Rule 4's
/// one-line test — *would a screenshot of the editing canvas differ from a
/// screenshot of the same document saved and reopened?* — caught it because the
/// old shell's canvas differed **in the one respect the operator was looking
/// at**, while claiming to be the document.
///
/// An opaque editor box differs too, and does not claim otherwise. Rule 4
/// permits exactly this by name: *"a snap indicator, a hover highlight, a
/// rubber-band … these are the cursor; they describe what is about to
/// happen."* An in-place editor **is** the cursor. What the rule forbids is
/// styling content **already applied** as though it were pending — and this
/// covers the applied content rather than restyling it.
///
/// Every program does it this way, which is the second half of the argument: a
/// spreadsheet cell, a Word table cell, a CAD attribute editor, a file-name
/// rename in Explorer. All of them cover the original with a filled box while
/// you type and reveal the result on commit. Nobody is surprised when the box
/// closes.
///
/// ## The two objections that stood, and what happened to them
///
/// **"A ghost in the wrong face is a lie about the document."** True of a
/// translucent one. An opaque box makes no claim about the document's typeface
/// because it is visibly not the document — it has an edge, a fill and UI text.
///
/// **"A ghost in the right face would need re-rasterizing the run's embedded
/// font per keystroke, and `BENCHMARK.md` says ~99 % of render cost on dense CAD
/// is resolution-independent."** Still true, and this is why the box does **not**
/// attempt the document's typeface. It costs one filled rectangle and one text
/// layout per frame.
///
/// ## The caret is measured against the text AS DRAWN
///
/// Not against the page's glyph advances, which is where it used to come from.
/// The draft is now drawn in the shell's font, so a caret placed by the
/// document's metrics would sit somewhere other than between the characters the
/// operator can see — and would drift further with every keystroke. One string,
/// one font, one size, measured once: the preview and the caret cannot disagree.
pub fn preview(ui: &Ui, ctx: &egui::Context, p: &Preview<'_>) {
    let Some(draft) = read(ctx) else {
        return;
    };
    if draft.page != p.page_index {
        return;
    }
    let Some(page) = p.doc.pages.get(p.page_index) else {
        return;
    };
    let theme = egui_shell::theme::Theme::of(ui.ctx());
    let painter = ui.painter();
    // A 1 s blink, and a repaint request so it actually blinks on a canvas with
    // no other reason to redraw.
    let on = (ui.input(|i| i.time) * 1.6) as i64 % 2 == 0;
    ui.ctx()
        .request_repaint_after(std::time::Duration::from_millis(400));
    let Some(rect) = caret_box(p.doc, &draft, page) else {
        return;
    };
    let screen = egui::Rect::from_two_pos(p.map.to_screen(rect.min), p.map.to_screen(rect.max));
    // ★★★ **WHAT YOU ARE TYPING, WHERE YOU ARE TYPING IT.** 2026-08-20.
    //
    // The operator: *"I can edit text now, but there is no live preview of that
    // either."* He is right and it made the feature nearly unusable — the page
    // renders **committed** glyphs, the draft lived beside them, and nothing
    // drew it. So an operator saw the old text, a blinking caret, and no
    // evidence that their keystrokes had landed anywhere.
    //
    // # ★★ Why an in-place EDITOR BOX and not text overlaid on the page
    //
    // The tempting shape is "draw the draft where the glyphs are, in the
    // document's own font, so it looks like the finished result". Two problems,
    // and the second is fatal:
    //
    // 1. **This shell does not have the document's font.** The page is
    //    rasterised by `pdfce-render` from embedded programs; egui's text stack
    //    has its own faces. The draft would render in a different typeface at a
    //    different width whatever we did.
    // 2. **★ The original glyphs are still underneath.** They are baked into
    //    the page raster and this shell cannot un-draw them. Text drawn on top
    //    of text is illegible, and the shorter the edit the worse it gets —
    //    changing `SHEET 1 OF 4` to `SHEET 2 OF 4` would show both `1` and `2`
    //    superimposed, which is the one character the operator is looking at.
    //
    // Masking the original needs the page's local background colour, which this
    // shell would have to *guess* — it is whatever the drawing has there, not
    // necessarily white.
    //
    // So: an **opaque editor box**, which is what every in-place editor in every
    // program already is. A spreadsheet cell, a Word table cell, a CAD attribute
    // editor, a file-name rename in Explorer — all of them cover the original
    // with a filled box while you type and reveal the result on commit. It is
    // the convention *and* the honest picture: the box says "this is a draft in
    // an editor", which is exactly what it is, and it makes no promise about
    // typeface or metrics that the commit would then break.
    //
    // # The size is the RUN's, not the UI's
    //
    // The glyph box gives the run's height, so the editor sits at the size of
    // the text it is replacing and a long draft is visibly long. Clamped to
    // something legible, because a 4 pt note at 25 % zoom is a box two pixels
    // high and the operator would be typing into a line.
    let height = screen.height().clamp(MIN_PREVIEW_PT, MAX_PREVIEW_PT);
    // ★ ONE font binding and ONE layout, shared by the fill, the text and the
    // caret below. Two `FontId`s built separately would be two derivations of
    // one fact, and the caret would sit where a *slightly different* string
    // would have ended. See this module's header.
    let font = egui::FontId::proportional(height * PREVIEW_FILL);
    let laid = painter.layout_no_wrap(draft.text.clone(), font.clone(), theme.palette.text);

    // ★★ THE BOX GROWS WITH WHAT IS IN IT, and it has to.
    //
    // It was `screen.shrink(1.0)` — the glyph box, exactly — until this was
    // driven and looked at. Two ways that is wrong, and the second is the
    // serious one:
    //
    // 1. **A draft longer than the run it replaces** overflows a box sized to
    //    the original, so the tail of what you are typing sits on bare page.
    // 2. **★ An `Anchor::Origin` draft has no glyph box at all.** `caret_box`
    //    returns a nominal 6 × 14 pt for new text, so Add-text drew its
    //    characters almost entirely OUTSIDE the fill — text on the page
    //    background in the shell's font, which is exactly the translucent ghost
    //    D4a condemns, arrived at by accident.
    //
    // So the width is the greater of the run's extent and the laid-out text.
    // Every in-place editor in every program does this — a spreadsheet cell
    // editor grows as you type past the column, and it grows because the
    // alternative is text that has escaped its own control.
    let width = screen
        .width()
        .max(laid.rect.width() + PREVIEW_INSET_PT * 2.0);
    let body = egui::Rect::from_min_size(
        egui::pos2(screen.left(), screen.center().y - height / 2.0),
        egui::vec2(width, height),
    );
    painter.rect_filled(body, 0.0, theme.palette.surface);
    painter.galley(
        egui::pos2(
            body.left() + PREVIEW_INSET_PT,
            body.center().y - laid.rect.height() / 2.0,
        ),
        laid.clone(),
        theme.palette.text,
    );

    // The bracket, drawn round the EDITOR rather than round the run: it is the
    // extent of what the operator is composing, which after the first keystroke
    // is no longer the extent of what they are replacing.
    painter.rect_stroke(
        body,
        0.0,
        egui::Stroke::new(1.0, theme.palette.accent),
        egui::StrokeKind::Outside,
    );
    if on {
        // ★★★ The caret is measured against the text AS DRAWN — not against
        // the page's glyph metrics. 2026-08-20, with the live preview.
        //
        // It used to come from `caret_x`, which walks the RUN's glyph advances.
        // That was right when the page's own glyphs were the only thing on
        // screen. It is wrong now: the draft is drawn in the shell's font
        // inside an editor box, so a caret placed by the document's metrics
        // would sit somewhere other than between the characters the operator
        // can actually see — and the further they typed, the further out it
        // would drift.
        //
        // This is the same class of defect as the vertex drag that tracked at
        // `1/zoom`: two derivations of one position, agreeing at first and
        // separating under use. So there is one derivation. The preview draws
        // the text; the caret measures **the same string, in the same font, at
        // the same size**, and the two cannot disagree.
        let prefix: String = draft.text.chars().take(draft.caret).collect();
        let advance = painter
            .layout_no_wrap(prefix, font.clone(), theme.palette.text)
            .rect
            .width();
        let x = body.left() + PREVIEW_INSET_PT + advance;
        painter.line_segment(
            [
                egui::Pos2::new(x, body.top()),
                egui::Pos2::new(x, body.bottom()),
            ],
            egui::Stroke::new(1.5, theme.palette.accent),
        );
    }
}

// ★★ `caret_x` was DELETED on 2026-08-20, with the live preview, and the reason
// is worth keeping.
//
// It derived the caret's position from the RUN's own glyph advances — exact
// while `caret <= glyphs.len()` and extrapolated beyond it, with a doc comment
// explaining the approximation honestly.
//
// The live preview made it wrong rather than approximate. The draft is now drawn
// in the shell's font inside an in-place editor box, so a caret placed by the
// DOCUMENT's metrics would sit somewhere other than between the characters the
// operator can see — and would drift further with every keystroke. Two
// derivations of one position, agreeing at first and separating under use, which
// is the same class of defect as the vertex drag that tracked at `1/zoom`.
//
// So there is one derivation now: the preview draws the string, and the caret
// measures the same string in the same font at the same size, inline above.
// Removed rather than left unused, because a plausible-looking helper is
// something a later hand reaches for.

/// The draft's box in **canvas** space, or `None` when it cannot be derived.
///
/// For an existing run this is the union of its glyph boxes; for a new-text
/// origin it is a nominal one-line box at the click. Both are converted through
/// [`crate::viewer::pdf_space_to_canvas`], the inverse of the bridge
/// [`resolve_run`] uses, so the caret lands on the glyphs it was resolved from.
fn caret_box(
    doc: &OpenDoc,
    draft: &Draft,
    page: &pdfce_core::page_tree::Page,
) -> Option<egui::Rect> {
    match &draft.anchor {
        Anchor::Run { run, .. } => {
            let text = doc.page_text()?;
            let r = text.runs.get(*run)?;
            let mut acc: Option<egui::Rect> = None;
            for g in &r.glyphs {
                let lo =
                    crate::viewer::pdf_space_to_canvas(Pos2::new(g.x, g.y + g.size * -0.25), page)?;
                let hi = crate::viewer::pdf_space_to_canvas(
                    Pos2::new(g.x + g.advance, g.y + g.size * 0.9),
                    page,
                )?;
                let b = egui::Rect::from_two_pos(lo, hi);
                acc = Some(acc.map_or(b, |a| a.union(b)));
            }
            acc
        }
        Anchor::Origin { x, y } => {
            #[allow(clippy::cast_possible_truncation)]
            let (x, y) = (*x as f32, *y as f32);
            let lo = crate::viewer::pdf_space_to_canvas(Pos2::new(x, y - 3.0), page)?;
            let hi = crate::viewer::pdf_space_to_canvas(Pos2::new(x + 6.0, y + 11.0), page)?;
            Some(egui::Rect::from_two_pos(lo, hi))
        }
    }
}
