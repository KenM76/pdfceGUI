//! # `app::actions::redactimg` — say at MARK time that a region covers an image
//!
//! ## What this closes
//!
//! **Ken, 2026-09-03:** *"every time I've tried the redact feature it tells me
//! it can't because there is objects that weren't redacted."*
//!
//! Reproduced with `pdfce-cli` alone, so the refusal itself is the engine's and
//! is not ours to remove:
//!
//! ```text
//! redaction refused: redaction region on page 1 intersects an image; pdfce
//! cannot yet destroy image pixels (clipping or masking would leave them
//! recoverable, ISO 32000-1 §12.5.6.23) — apply refused rather than producing a
//! false redaction
//! ```
//!
//! ★ **The refusal is right and we are not trying to get around it.** Clipping
//! or masking would leave the pixels recoverable, and a redaction pdfce cannot
//! stand behind is the one thing this feature must never produce.
//!
//! ## ★★★ What is wrong is WHEN he finds out
//!
//! The gate is in `redact-apply`, and apply is **all-or-nothing for the
//! document**. So the sequence an operator actually lives through is:
//!
//! 1. mark twelve regions across a drawing, carefully;
//! 2. press Apply;
//! 3. be told the whole thing is refused, because **one** of them grazed the
//!    bounding box of a logo.
//!
//! Nothing says *which* one. On a title block, a value worth redacting is often
//! inches from a company logo, and the mark that offends may be the one drawn
//! ten minutes ago. `OPERATOR_REQUESTS.md` O103 asks the engine for a
//! per-region refusal so the other eleven can apply; until that lands, the
//! least this shell can do is say so **at the moment the rectangle is drawn**,
//! when it is one gesture to redraw it.
//!
//! ## This is DISCLOSURE, not a gate — and the distinction is load bearing
//!
//! It refuses nothing and blocks nothing. The mark is authored exactly as
//! before, because a mark is reversible and costs nothing, and because pdfce
//! must not decide on the operator's behalf that a region is not worth marking:
//! the engine may gain the capability, and the same mark would then apply
//! cleanly. What changes is only that he is told, in the same breath as the
//! success, and can act while it is cheap.
//!
//! ★★ Rule 4: nothing is drawn on the canvas. The mark renders exactly as any
//! other mark renders, because it IS any other mark — a warning tint would be
//! pdfce styling its own uncertainty into content, which is the thing the rule
//! forbids by name. The sentence goes where every other disclosure goes.

use crate::app::state::OpenDoc;
use crate::canvas::pick::PickClass;
use crate::canvas::target::CanvasTargetProvider;
use pdfce_core::vector::MarqueeMode;

/// How many of `targets` on `page_index` are raster images.
///
/// Split out as its own function because it is the whole factual claim this
/// module makes, and because it is the part that could be wrong in a way an
/// operator would notice: over-counting invents a warning about a page that
/// would have redacted cleanly, and under-counting is the silence this module
/// exists to end.
fn image_count(
    doc: &OpenDoc,
    page_index: usize,
    targets: &[crate::canvas::target::TargetId],
) -> usize {
    let Some(provider) = doc.page_objects() else {
        return 0;
    };
    targets
        .iter()
        .filter(|t| {
            matches!(
                provider.object_class(page_index, **t),
                Some(PickClass::Image)
            )
        })
        .count()
}

/// **Does anything the operator just selected sit on a raster image?**
///
/// Used by the selection route, where the answer needs no geometry at all: the
/// operator picked the objects, so their classes are already known and asking
/// the decomposition a second question could only produce a second answer.
#[must_use]
pub fn images_in_selection(doc: &OpenDoc) -> usize {
    let page_index = doc.view.page_index;
    let targets = doc.selection.targets_on(page_index);
    image_count(doc, page_index, &targets)
}

/// **How many raster images are anywhere on `page_index`.**
///
/// The whole-page route's question, and it is the simple one: a mark that
/// covers the page covers every image on it, so any image at all means the
/// apply will be refused.
#[must_use]
pub fn images_on_page(doc: &OpenDoc, page_index: usize) -> usize {
    let Some(provider) = doc.page_objects() else {
        return 0;
    };
    let Some(page) = doc.pages.get(page_index) else {
        return 0;
    };
    // ★ The sheet in CANVAS space, taken from the render geometry rather than
    // from the media box directly: canvas space is the page's device space at
    // scale 1.0, and `page_device_geometry` is the one place that mapping
    // lives. Building the rectangle from `/MediaBox` by hand would be a second
    // statement of it, and the two would disagree the first time a page carried
    // a `/Rotate` or a crop.
    let (w, h, _) = pdfce_render::page_device_geometry(page, 1.0);
    let whole = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(w as f32, h as f32));
    let hits = provider.hit_test_rect(page_index, whole, MarqueeMode::Touched);
    image_count(doc, page_index, &hits)
}
