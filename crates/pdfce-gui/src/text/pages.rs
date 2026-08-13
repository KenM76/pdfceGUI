//! # `text::pages` — every string the Pages panel shows
//!
//! One area of the catalog described in [`crate::text`]'s header, consumed
//! by [`crate::panels::pages`] and by nothing else.
//!
//! It is a sibling of [`crate::text::panels`] rather than a module inside it
//! for the same reason [`crate::text::forms`] is: that directory's own header
//! declares it covers *"the three document-structure panels"* and their two
//! inspector siblings, and the Pages panel is neither. It is a **navigator**
//! whose copy is about pictures, page geometry and the cost of drawing —
//! vocabulary that has nothing in common with a font inventory or a signature
//! byte range, and that would be read past by anyone maintaining either.
//!
//! ## ★ The posture: an undrawn thumbnail must SAY it is undrawn
//!
//! This is the whole reason half the strings below exist, and it is the
//! project's no-placeholders rule (`RIBBON_IA.md` P3) applied to a picture
//! rather than to a control.
//!
//! A page thumbnail that has not been rasterized yet is, on screen, a
//! rectangle. A rectangle the colour of paper **is a picture of an empty
//! page** — and an empty page is a thing a real PDF can contain. So a
//! thumbnail grid that draws blank rectangles while it works is not
//! "loading"; it is *asserting something false about the document*, and the
//! operator has no way to tell the two apart. The old shell drew exactly that
//! (`main.rs`'s `thumbnail_rail`: a bordered rect in `extreme_bg_color` with
//! the page number), and it is the one part of that rail this panel did not
//! carry across.
//!
//! Every state a tile can be in therefore has **words**:
//!
//! | State | String | Says |
//! |---|---|---|
//! | queued, previews on | [`thumbnail_not_drawn_yet`] | *this is not a picture of the page yet* |
//! | previews off | [`thumbnail_previews_off`] | *and it will not become one until you say so* |
//! | the render hit the time ceiling | [`thumbnail_abandoned`] | *pdfce started and stopped* |
//! | the page would not draw | [`thumbnail_failed`] | *this page is the problem, not the panel* |
//!
//! No spinner, and that is deliberate rather than an omission: a dozen
//! spinning icons is motion, not information, and only one page is ever
//! being drawn at a time anyway.
//!
//! ## Conventions, restated from [`crate::text`] because they bind here
//!
//! - **Sentence case, no trailing period on labels; full sentences with
//!   punctuation for prose.**
//! - **Name the thing and what the operator can do about it.**
//!   [`previews_paused_note`] is the worked example: it names the page, the
//!   measured cost, and the control that resumes.
//! - **Never state a capability the build does not have.**

/// The document's page count, as the panel's first line.
///
/// Singular and plural are spelled out rather than assembled with a `(s)`,
/// which reads as a form field rather than as a sentence. One page is a
/// reachable and perfectly ordinary case — most drawings are one sheet.
#[must_use]
pub fn pages_count(total: usize) -> String {
    if total == 1 {
        "1 page".to_owned()
    } else {
        format!("{total} pages")
    }
}

/// How many pages the operator has picked, shown only when that is not zero.
///
/// ★ **This number is the operand list the ribbon's Pages tab already
/// promises.** Every one of those commands' tooltips says *"the selected
/// pages"* — `pages.delete` is *"Remove the selected pages from this
/// document"* — so the count here is not decoration, it is the answer to
/// *"selected where?"* that the tooltips leave open. Wording it as a plain
/// count rather than as an instruction keeps it a statement of fact.
#[must_use]
pub fn pages_selected(selected: usize) -> String {
    if selected == 1 {
        "1 page selected".to_owned()
    } else {
        format!("{selected} pages selected")
    }
}

/// A document with a page tree that resolved to nothing.
///
/// Rare and not impossible: a damaged `/Pages` node can flatten to an empty
/// vector while the file still opens. Saying so beats an empty grid, which
/// reads as a panel that failed rather than as a document that is empty.
#[must_use]
pub fn pages_none() -> &'static str {
    "This document has no pages."
}

/// A tile's caption — the page number an operator would say out loud.
///
/// **1-based.** Everything inside pdfce indexes from 0; a human counts from
/// 1, and the conversion happens here and at no other point in the panel, so
/// there is exactly one place the off-by-one could be.
#[must_use]
pub fn page_number(page_index: usize) -> String {
    format!("{}", page_index + 1)
}

/// A tile's tooltip: which page, how big it is, and what a click does.
///
/// The page size is in **millimetres**, from the page's own extent in points
/// — because the operator this panel is for is looking at a drawing sheet
/// set, and "A1" or "841 × 594" is how they identify a sheet. It is also the
/// one useful fact a tile can carry before its picture exists, which is why
/// the tooltip is worth having on an undrawn tile at all.
///
/// The gestures are named because none of them is discoverable: nothing on
/// screen says that Ctrl adds to the selection.
#[must_use]
pub fn page_tile_tooltip(page_index: usize, width_mm: f32, height_mm: f32) -> String {
    format!(
        "Page {} — {width_mm:.0} × {height_mm:.0} mm. Click to go there, \
         Ctrl+click to add it to the selection, Shift+click to extend.",
        page_index + 1
    )
}

/// A tile whose page has not been rasterized yet.
///
/// See this module's header: the alternative is a blank rectangle, which is
/// a picture of an empty page and therefore a lie about the document.
#[must_use]
pub fn thumbnail_not_drawn_yet() -> &'static str {
    "Not drawn yet"
}

/// A tile whose page will not be rasterized, because previews are off.
///
/// Distinct from [`thumbnail_not_drawn_yet`] on purpose. "Not drawn yet"
/// promises a picture is coming; with previews off, none is, and an operator
/// waiting for one that will never arrive has been misled by a word.
#[must_use]
pub fn thumbnail_previews_off() -> &'static str {
    "Preview off"
}

/// A tile whose render pdfce started and abandoned.
///
/// Reachable when the document is edited, or the panel closed, while a page
/// is being drawn. Not a failure — nothing is wrong with the page — so it
/// must not read like one.
#[must_use]
pub fn thumbnail_abandoned() -> &'static str {
    "Not finished"
}

/// A tile whose page the renderer refused.
///
/// Names the *page* as the subject, because that is what is true: the panel
/// works, and this one page did not draw. The canvas will say the same thing
/// at more length if the operator navigates to it, which is the right place
/// for the detail.
#[must_use]
pub fn thumbnail_failed() -> &'static str {
    "Would not draw"
}

/// The label of the control that turns page previews on and off.
#[must_use]
pub fn previews_label() -> &'static str {
    "Draw page previews"
}

/// …and its tooltip, which states the cost rather than hiding it.
///
/// ★ **The number in this sentence is measured, not estimated.**
/// `BENCHMARK.md` records a real CAD drawing whose content stream costs
/// ~0.74 s to interpret *at any scale* — a one-by-one-**point** region of it
/// costs 691 ms — so a thumbnail of such a page is not cheap merely because
/// it is small. That is the single most surprising fact about this panel and
/// it belongs where the operator meets it.
#[must_use]
pub fn previews_tooltip() -> &'static str {
    "Draw a picture of each page. A dense drawing can take most of a second \
     per page whatever size it is drawn at, because the cost is in reading \
     the page rather than in filling the pixels — so pdfce stops on its own \
     when it meets one."
}

/// Why previews stopped, and what resumes them.
///
/// Named parts: the page that was slow, what it cost, and the control. A
/// message that said only "previews paused" would leave the operator hunting
/// for a cause pdfce already knows.
///
/// The cost is printed in **seconds to one decimal** rather than in
/// milliseconds, because the number's job here is to justify a decision, and
/// "0.8 s" justifies it in a way "812 ms" does not.
#[must_use]
pub fn previews_paused_note(page_index: usize, millis: u128) -> String {
    let seconds = millis as f32 / 1000.0;
    format!(
        "Page previews stopped: page {} took {seconds:.1} s to draw. \
         Turn “{}” back on to carry on drawing them.",
        page_index + 1,
        previews_label()
    )
}
