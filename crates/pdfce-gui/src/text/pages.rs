//! # `text::pages` — every string the Pages panel shows
//!
//! One area of the catalog described in [`crate::text`]'s header, consumed by
//! [`crate::panels::pages`] — the grid, its captions and its tile states — and
//! by [`crate::app::actions::pages`], which words what a page **delete** broke.
//! Those are the only two readers.
//!
//! The second joined the first rather than getting a module of its own because
//! both are sentences about *pages* in the same vocabulary — sheets, page
//! numbers, this document — and a reader who came here for one half would not
//! find the other. See the disclosure section at the foot of this file for the
//! rule-4 obligation those strings discharge.
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

// ---------------------------------------------------------------------------
// ★ THE PAGE VERBS' DISCLOSURES — what a delete broke, in words
//
// A second audience for this module, and the header's *"consumed by
// `crate::panels::pages` and by nothing else"* is now *"and by
// `crate::app::actions::pages`, which words what a page delete broke"*. The
// two belong together rather than in `crate::text::status`: they are sentences
// about **pages**, they use the same vocabulary as the panel above them
// (sheets, page numbers, this document), and splitting them would put half the
// page copy where a reader looking for the other half would not find it.
//
// # Why these exist at all — rule 4, and the engine asking for them
//
// `EditSession::delete_pages` returns a `DanglingReport` and its own
// documentation says what it is for:
//
//   > pdfce **exceeds** Acrobat here on purpose. … surface (don't silently
//   > leave) dangling bookmarks/links/destinations as a reviewable post-delete
//   > report … rather than silently leaving them broken the way Acrobat does.
//
// The engine reports and deliberately does **not** repair, because repointing
// a bookmark at "whatever page now occupies that index" would be pdfce
// deciding what the author meant. That leaves exactly one obligation on this
// side: say so. A delete that quietly broke 300 bookmarks and drew nothing is
// the shape of failure rule 4 exists to forbid — the drawing is unchanged, the
// file is not, and the operator would find out from a diff.
//
// # Why they are counted and not listed
//
// The engine's own choice, and this follows it: *"a delete that orphans 300
// bookmarks should say '300', not list them."* The status bar has **one row**
// that may not grow (R128), so a list could not be drawn there even if the
// report carried one.
//
// # The wording rule these follow
//
// Each names **what is now wrong** rather than what pdfce did, because that is
// the sentence an operator can act on. "3 bookmarks now point at pages that
// are no longer here" is actionable; "the dangling reference census reported
// 3" is a status line about pdfce.
// ---------------------------------------------------------------------------

/// Bookmarks (outline items, §12.3.3) whose destination page was removed.
///
/// Singular and plural spelled out rather than assembled with `(s)`, exactly
/// as [`pages_count`] does and for the same reason: one broken bookmark is a
/// perfectly ordinary outcome of deleting one page, and `1 bookmark(s)` reads
/// as a form field rather than as a sentence.
#[must_use]
pub fn deleted_dangling_bookmarks(count: usize) -> String {
    if count == 1 {
        "1 bookmark now points at a page that is no longer in this document.".to_owned()
    } else {
        format!("{count} bookmarks now point at pages that are no longer in this document.")
    }
}

/// Links on **surviving** pages whose destination page was removed (§12.5.6.5).
///
/// "on the pages that remain" is load-bearing: links that left with their own
/// page are deliberately not counted by the engine, because reporting them
/// would inflate the number with references that no longer exist to be broken.
/// The sentence says which set it is talking about so the number can be
/// trusted.
#[must_use]
pub fn deleted_dangling_links(count: usize) -> String {
    if count == 1 {
        "1 link on the pages that remain points at a page that was removed.".to_owned()
    } else {
        format!("{count} links on the pages that remain point at pages that were removed.")
    }
}

/// Named destinations (§12.3.2.3) that resolved to a removed page.
///
/// Named destinations are reached from *outside* this document as well as from
/// within it — another PDF's link, a URL fragment, a script — which is why they
/// are disclosed separately from bookmarks rather than added to that count.
#[must_use]
pub fn deleted_dangling_destinations(count: usize) -> String {
    if count == 1 {
        "1 named destination now points at a page that is no longer in this document.".to_owned()
    } else {
        format!(
            "{count} named destinations now point at pages that are no longer in this document."
        )
    }
}

/// The document carries a `/PageLabels` tree (§12.4.2) the deletion left
/// numerically stale.
///
/// A sentence rather than a count, because the underlying fact is a boolean:
/// the tree is one object and the operator's question is *"are my page numbers
/// wrong now?"*.
///
/// It says pdfce left them **deliberately**, because the alternative reading —
/// that pdfce failed to update them — invites the operator to report a bug
/// against behaviour that matches Acrobat's and was chosen. Acrobat leaves them
/// stale and silent; this is the "and says so" half.
#[must_use]
pub fn deleted_page_labels_stale() -> &'static str {
    "This document numbers its own pages, and those numbers were left as they were — the \
     sheets that remain still carry the labels they had before the deletion."
}

/// Preseparated page sets (§14.11.4) that lost at least one plate.
///
/// The one class of broken reference the engine **repairs** rather than
/// reporting, and it is still disclosed for exactly that reason: something in
/// the file changed that the operator did not ask for. `DeleteOutcome`'s own
/// docs draw the line — a bookmark's target is a question about *authorial
/// intent* that pdfce must not guess at, while a separation dictionary's
/// `/Pages` array is a *structural* fact pdfce knows the answer to.
#[must_use]
pub fn deleted_separations_repaired(sets: usize) -> String {
    if sets == 1 {
        "1 set of printing plates lost a member, and the plates that remain were updated to \
         list only each other."
            .to_owned()
    } else {
        format!(
            "{sets} sets of printing plates lost members, and the plates that remain were \
             updated to list only each other."
        )
    }
}

// ---------------------------------------------------------------------------
// Insert from file
// ---------------------------------------------------------------------------

/// The title on the picker `pages.insert_from_file` opens.
///
/// Not the Open dialog's title. The two pick a PDF and mean opposite things —
/// one replaces what is on screen, the other adds to it — and a picker headed
/// *"Open a PDF"* over a document the operator is editing is a sentence that
/// says the wrong thing at the moment they are most likely to read it.
#[must_use]
pub const fn insert_dialog_title() -> &'static str {
    "Insert pages from a PDF"
}

/// ★ **What arrived, where it went, and the one thing that did not come with
/// it.**
///
/// # Why the sentence names all three
///
/// `EditSession::insert_pages` copies each page and everything reachable from
/// it — content streams, resources, fonts, XObjects — at fresh object numbers.
/// It does **not** merge the source document's *document-level* structures:
/// **outlines (bookmarks), the AcroForm field tree, named destinations and
/// page labels**.
///
/// That is a deliberate, documented consequence of staying incremental — a
/// document-level merge rewrites objects an incremental save exists in order
/// not to touch — and `pdfce-core` states it as a choice between two correct
/// answers rather than a limitation.
///
/// It is still something pdfce did that the operator did not ask for, so
/// rule 4 applies: an operator whose bookmarks did not come across is entitled
/// to know that **here**, at the moment it happened, rather than by going
/// looking for a bug in a document they have already saved.
///
/// # Why the page number is 1-based and the count is not a range
///
/// *"after page 7"* is the sheet the operator was looking at, in the numbering
/// the page box and the thumbnails use. A 0-based index here would be the only
/// place in the application that counted differently.
#[must_use]
pub fn inserted(count: usize, after_page_index: usize) -> String {
    let after = after_page_index.saturating_add(1);
    let pages = if count == 1 { "page" } else { "pages" };
    format!(
        "Inserted {count} {pages} after page {after}. Bookmarks, form fields and page labels from that file did not come across — its pages did."
    )
}

/// The chosen file could not be opened, and why.
///
/// `detail` is `pdfce-core`'s own error `Display`, passed through for the same
/// reason [`crate::text::canvas_render_failed`] passes one through: those
/// errors are specific, and replacing one with *"could not open the file"*
/// discards the half that says whether it was encrypted, truncated or not a
/// PDF at all.
///
/// **Says nothing was inserted.** A failure part-way through a multi-page
/// insert would otherwise leave the operator wondering whether some of it
/// landed; the verb is one command and either records it or does not.
#[must_use]
pub fn insert_failed(detail: &str) -> String {
    format!("Nothing was inserted. {detail}")
}

/// The chosen file has no pages to insert.
///
/// A separate sentence from [`insert_failed`] because it is not a failure: the
/// file opened, it is a valid PDF, and it is empty. Collapsing the two would
/// send an operator looking for corruption in a file that has none.
#[must_use]
pub const fn insert_empty() -> &'static str {
    "That PDF has no pages, so nothing was inserted."
}
