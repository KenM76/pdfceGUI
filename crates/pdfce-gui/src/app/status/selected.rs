//! # `status::selected` — what is selected, said in words
//!
//! One line at the left of the status bar, naming the thing the operator has
//! selected and — when it matters — how many other things were under the same
//! click.
//!
//! ## ★★★ The defect this exists for
//!
//! The operator, 2026-08-26:
//!
//! > *"when I click on one of the objects all I get is the page selected."*
//!
//! He was reporting the truth, precisely. His file wraps the whole visible body
//! of the sheet in a page-sized form XObject, the engine does not enter one, and
//! its bounding box therefore wins every click at every point. He was selecting
//! a page-sized object.
//!
//! **Nothing on screen said so.** The selection outline is drawn round the
//! page edge, which looks exactly like *"the page is selected"* — a state this
//! program does not have. There was no surface anywhere that would have told
//! him *"you have selected a Form containing 214 objects"*, which is a
//! diagnosis, from which the next question follows on its own.
//!
//! ★ This line does not fix the selecting. It makes the selecting **legible**,
//! which is what turns an unexplainable interface into a solvable one — and it
//! is the surface every future refusal sentence will be printed on.
//!
//! ## Why the left, and why it is the thing that yields
//!
//! The right-hand cluster is fixed controls the operator reaches for — page,
//! zoom, fit, Find, the pick filter — and `status::fitting` may shed only two
//! of those, because the rest have no other home. This is a **readout**: it
//! costs nothing to lose, because everything it says is also visible in the
//! Objects panel and in the selection outline.
//!
//! So it goes on the left with the other narration, where `egui`'s left-to-right
//! run gives up its space first, and it elides rather than pushing. That is
//! `status`'s own rule about what yields, applied to the newest thing on the
//! bar rather than exempting it.
//!
//! ## What it says, and what it refuses to say
//!
//! | state | line |
//! |---|---|
//! | nothing selected | nothing at all |
//! | one object | its kind, and its size in points |
//! | one object, more underneath | `… · 1 of 5 here` |
//! | several objects | `3 objects selected` |
//!
//! ★★ **Nothing when nothing is selected**, rather than *"Nothing selected"*.
//! A status bar that narrates the absence of a thing spends a permanent line on
//! the most common state in the program. `HOW_IT_SHOULD_WORK.md` §8.2 argues for
//! a tutorial string there and it may well be right — but that is a decision
//! about teaching, and this line is a decision about reporting. They should not
//! be made at once and they should not be made by the same code.

use egui::Ui;

use crate::app::state::OpenDoc;
use crate::text::status as t;

/// The region this line publishes, so a driven check can find it.
pub const REGION: &str = "status-group:selected"; // ui-text-exempt: trace region name, never displayed

/// Draw the selection readout, or nothing.
///
/// Takes `&OpenDoc` and the context: the selection is on the document, and the
/// **depth** of the click that made it is in `egui::Memory` — see
/// [`crate::canvas::depth`] for why those two live apart.
pub(super) fn show(ui: &mut Ui, doc: &OpenDoc) {
    let page = doc.view.page_index;
    let objects = doc.selection.object_indices_on(page);
    if objects.is_empty() {
        return;
    }

    let text = if objects.len() > 1 {
        t::selection_many(objects.len())
    } else {
        // ★ The kind and the size come from the DECOMPOSITION, not from the
        // selection: a selection is four integers and knows nothing about what
        // it names. `page_objects` is the cache the canvas and the Objects
        // panel already read, so this adds no work on a frame that has drawn
        // either of them — and on a frame that has not, it is the same
        // extraction they would have paid for anyway.
        let described = doc.page_objects().and_then(|provider| {
            let model = provider.page_objects();
            let object = model.objects.get(objects[0])?;
            let kind = crate::text::panels::objects::object_kind_label(
                crate::panels::objects::summary::object_kind(object),
            );
            // Canvas space, from the provider's own projection rather than a
            // second one built here — `bounds` is what the overlay draws the
            // selection outline from, so the number in this line and the box on
            // screen cannot describe different rectangles.
            let size = provider
                .bounds(page, crate::canvas::target::TargetId(objects[0] as u64))
                .map(|r| (r.width(), r.height()));
            Some(match size {
                Some((w, h)) => t::selection_one(kind, w, h),
                None => t::selection_one_unsized(kind),
            })
        });
        // A selection naming an object the decomposition does not have is a
        // stale index — which `SelectionState::resolve` exists to prevent and
        // which is therefore not expected. Saying the plain thing is better
        // than saying nothing: the operator still learns that something is
        // selected.
        described.unwrap_or_else(|| t::selection_many(1))
    };

    // ★★ The stack count, appended rather than separate, because it is a fact
    // about the SAME selection: *"this one, and there were four others."* A
    // second label would read as a second subject.
    //
    // `canvas::depth` returns `None` unless there were at least two candidates,
    // so the common case adds nothing — and it returns `None` for a selection
    // that did not come from a click at all, which is what stops this claiming
    // a stack the operator is not pointing at.
    let text = match crate::canvas::depth::taken(ui.ctx(), page, objects[0]) {
        Some(depth) => t::selection_with_depth(&text, depth.taken + 1, depth.of),
        None => text,
    };

    let response = ui.label(text);
    crate::diag::ui_rect(REGION, response.rect);
}
