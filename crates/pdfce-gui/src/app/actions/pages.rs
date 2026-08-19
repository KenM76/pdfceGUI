//! # `app::actions::pages` — the four page verbs, and the resync a structural
//! edit owes
//!
//! [`super::apply`] is the interpreter for every [`Action`]; this file is the
//! body of the four that act on **pages** rather than on the marks drawn on
//! them — [`Action::RotatePages`], [`Action::DeletePages`],
//! [`Action::ReorderPages`] and [`Action::ExtractPages`] — plus the one
//! function every *other* edit in the application now calls as well,
//! [`resync`].
//!
//! ## ★ Why these four are not four more arms in `apply.rs`
//!
//! Rule R2's own justification decides it, exactly as it decided the
//! `apply.rs` split from `actions.rs`: *"the value of the limit is that the
//! file has to have a single subject."* `apply.rs`'s subject is **the
//! cancel–mutate–bump–invalidate protocol** — what happens when a request to
//! change the document is granted, and the ordering that makes it safe. That
//! protocol is the same for every verb and changes only when the protocol
//! changes.
//!
//! This file's subject is a different one, and it is the reason page verbs
//! could not simply be four more `vector_edit` calls:
//!
//! > **A page index is a position, not an identity**, and the application
//! > holds four things that are stated in page indices.
//!
//! Those four are the flattened page vector, every cached raster, the canvas's
//! object selection, and the Pages panel's own picks. `HANDOFF.md` §10 states
//! the general rule — *"Selection is an identity — page, object, subpath, node
//! — not a position"* — and `crate::canvas::interact`'s header states the
//! measured half of it: *"`move_*` renumbers nothing … the `delete_*` family is
//! the one that renumbers."* A page delete is that sentence one structure up,
//! and a page **reorder** is a third case neither of them names.
//!
//! ## ★★ The table this whole file exists to implement
//!
//! | | page vector | rasters | canvas selection | panel picks | `view.page_index` |
//! |---|---|---|---|---|---|
//! | markup, move, fill *(existing verbs)* | unchanged | current page's dropped | survives — resolved against the new epoch | untouched | valid |
//! | **rotate** | `/Rotate` differs | **all stale** — every turned page's picture is sideways | survives; a rotation adds and removes no operator | **survives** — the same sheets are still the same sheets | valid |
//! | **reorder** | order differs | **all stale** — page *N* is a different sheet | **cleared** | **remapped** — the permutation says where each went | valid, may now show a different sheet |
//! | **delete** | shorter | **all stale** | **cleared** | **cleared** — the picked sheets are gone | **may be past the end**; clamped |
//! | **extract** | unchanged | unchanged | untouched | untouched | valid |
//!
//! Every row of that table is asserted below or in
//! [`crate::panels::pages::select`], because every cell is a way for the
//! application to end up drawing the wrong sheet or aiming a destructive verb
//! at one nobody chose — and none of them fails loudly.
//!
//! ## ★ [`resync`] is called from `vector_edit`, not from these four arms
//!
//! That placement is the one design decision in this file worth arguing, and
//! it is `HANDOFF.md` §6's one-choke-point rule applied to a *consequence*
//! rather than to a dispatch.
//!
//! The naive arrangement is for each page arm to do its own tidying after its
//! own `vector_edit` call. It is wrong for a reason that is invisible until
//! undo exists — and undo exists: `Action::Undo` runs the *same* engine
//! commands backwards, through the *same* `vector_edit`, and an undone
//! `DeletePages` puts four sheets back. Tidying in the four arms would leave
//! the undo of a page delete showing a page vector that is four sheets short,
//! with a page count in the status bar to match, and nothing anywhere would
//! error.
//!
//! So the resync sits at the one place every document change already passes
//! through, and it is **self-describing rather than told**: it compares the
//! page vector it has against the one the session now reports and acts on the
//! difference. A verb it has never heard of gets the right treatment, which is
//! what makes it a choke point rather than a fifth copy of a rule.
//!
//! Its cost on an edit that changed no page is one page-tree walk and one
//! `Vec` comparison, paid **per operator gesture** rather than per frame. That
//! is the same order as the `Arc::get_mut` and the epoch bump beside it.
//!
//! ## What [`resync`] cannot do, and who does it instead
//!
//! The **Pages panel's picks** live on `PdfceApp::panels`, not on `OpenDoc`,
//! and `vector_edit` takes an `&mut OpenDoc`. They are therefore handled in
//! `apply.rs`'s two arms that know which edit ran — which is sound because
//! those are the only two verbs that can move a pick, and it is *stated* here
//! rather than left to be discovered, because the consequence of forgetting is
//! a Delete aimed at sheets the operator did not choose.
//!
//! The **thumbnail cache** needs nothing at all: it is keyed on
//! `(edit_epoch, pixels_per_point)` and `ThumbnailCache::sync` empties itself
//! the moment the epoch moves, which `vector_edit` has already done by the time
//! the next frame draws the panel. That is the key-carries-the-staleness design
//! `apply.rs` wishes the page texture had.

use std::path::{Path, PathBuf};

use pdfce_core::edit::EditSession;
use pdfce_core::object::ObjId;

use crate::app::files::{self, Picked};
use crate::app::state::OpenDoc;
use crate::text::pages as t;

/// Everything the operator can ask of the document's **set of pages**.
///
/// ## Why this is a sub-enum rather than five more variants on `Action`
///
/// The same three reasons [`super::dimensions::DimensionAction`] gives, and the
/// first is again the one that decided it:
///
/// 1. **They share a rule the flat enum could not express.** Every verb here
///    can **renumber** the document, and what each owes the shell's derived
///    state afterwards is different: a rotation preserves both selections
///    (nothing renumbers), a reorder remaps the Pages panel's picks and clears
///    the canvas selection, a delete clears both, an insert navigates to what
///    arrived. As five flat variants that rule is re-derived in five arms; as a
///    family it lives here, where a sixth verb has to answer it.
///
///    The failure that guards against is specific and silent: a page verb with
///    the wrong invalidation produces a **correct document** and a wrong
///    screen, so nothing fails and the operator sees a selection pointing at a
///    sheet that has moved.
/// 2. **R2.** `super`'s enum crossed 1,500 lines when image placement landed,
///    and the alternative to a seam is thinner prose — which the file-size
///    gate's own header names as the incentive it refuses to create.
/// 3. **The destination already existed.** This module has held the five
///    verbs' *bodies* since page operations shipped, and `apply` already routed
///    every one of them here. The enum was the only half still living
///    elsewhere.
#[derive(Debug, Clone, PartialEq)]
pub enum PageAction {
    /// **Insert another document's pages into this one, after the current page.**
    ///
    /// Raised by `pages.insert_from_file` once the picker has answered.
    ///
    /// # ★ Why this is an editing verb and not an open
    ///
    /// `pdfce_core::pageops::insert` also inserts pages, and returns the bytes
    /// of a **new document**. Wiring that would have meant replacing
    /// `OpenDoc::session` wholesale, which discards the undo stack — invisible
    /// in any test that checks page counts, and visible the first time an
    /// operator presses Ctrl+Z twice.
    ///
    /// So it was filed rather than shipped, and `pdfce-core` answered the same
    /// day with `EditSession::insert_pages`: the missing member of the
    /// `delete_pages` / `reorder_pages` / `rotate_pages` family. It records
    /// **one** undoable command however many pages arrive, exactly as a reorder
    /// does however many pages move.
    ///
    /// # What it does not carry, and why the operator is told
    ///
    /// The session verb copies each page and everything reachable from it —
    /// content, resources, fonts, XObjects — at fresh object numbers. It does
    /// **not** merge the source's document-level structures: outlines, the
    /// AcroForm field tree, named destinations, page labels. That is the honest
    /// cost of staying incremental, because a document-level merge rewrites
    /// objects an incremental save exists in order not to touch.
    ///
    /// `crate::text::pages::inserted` says so, because an operator whose
    /// bookmarks did not come across is entitled to know that before they go
    /// looking for a bug.
    InsertPagesFromFile {
        /// The document to take pages from.
        path: std::path::PathBuf,
        /// Which of ITS pages, 0-based, **in the order the operator asked
        /// for**.
        ///
        /// Order is carried rather than sorted, and duplicates are kept,
        /// because the range grammar treats the text as a sequence: `3,1-2`
        /// inserts source page 3 first, and `1,1` inserts a page twice. Both
        /// are things an operator can only ask for in one gesture if this
        /// field preserves them.
        pages: Vec<usize>,
        /// Where they land, in the engine's own vocabulary.
        ///
        /// ★ `pdfce_core::pageops::InsertPosition` directly rather than a
        /// local enum mapped at the boundary. Four choices — `Start`, `End`,
        /// `Before(n)`, `After(n)` — and a second spelling of them would be a
        /// second place for "before" and "after" to drift, where the drift is
        /// silent because both compile and both insert *somewhere*.
        position: pdfce_core::pageops::InsertPosition,
    },
    /// **Turn the operand pages by `delta` degrees**, as one undoable command.
    ///
    /// Raised by `pages.rotate_left` (−90) and `pages.rotate_right` (+90).
    ///
    /// # Why a delta rather than an absolute angle
    ///
    /// Because that is what the button means and what `EditSession::rotate_pages`
    /// implements: a selection of pages at 0°, 90° and 180° turned right lands
    /// at 90°, 180° and 270°, **not** all at 90°. The engine's own doc comment
    /// confirms Acrobat persists the absolute result of exactly that
    /// arithmetic. An absolute variant would be a different verb (*set the
    /// rotation of these pages to N*), which no control in this build offers.
    ///
    /// # It changes no page's identity
    ///
    /// A rotation rewrites one `/Rotate` entry per page. Nothing is added,
    /// removed or renumbered, so both selections survive it untouched — which
    /// is why the apply arm's resync is about *pictures* (every cached raster
    /// of a turned page is now wrong) and not about *indices*.
    RotatePages {
        /// 0-based page indices, ascending and unique.
        pages: Vec<usize>,
        /// A relative turn in degrees, a multiple of 90.
        delta: i32,
    },
    /// **Remove the operand pages from the document**, as one undoable
    /// command.
    ///
    /// Raised by `pages.delete` from the ribbon's Pages tab and from the page
    /// tile's context menu.
    ///
    /// # ★ This is the one action in the enum that renumbers pages
    ///
    /// `HANDOFF.md` §10 states the rule for objects — *"Selection is an
    /// identity — page, object, subpath, node — not a position"* — and this is
    /// its page-level instance. After the removal, every index above the lowest
    /// deleted page names a **different sheet**. Both selections in the
    /// application are therefore invalid, in different ways, and the apply arm
    /// deals with both:
    ///
    /// * the **page** selection named exactly the sheets that no longer exist,
    ///   so it is cleared;
    /// * the **canvas** selection names objects on a page *index*, and that
    ///   index now resolves to another sheet's content, so it is cleared too.
    ///
    /// # It is destructive and, until undo lands, irreversible
    ///
    /// No confirmation dialog, deliberately, and the reasoning is at the apply
    /// arm: `crate::app::save`'s `save_pending` is the one predicate this
    /// application consults before a destructive path, the engine records the
    /// removal as an undoable command already, and **nothing is written to
    /// disk** — the operator's file on disk is untouched until they save a
    /// copy, which is a separate deliberate act with its own dialog.
    DeletePages {
        /// 0-based page indices, ascending and unique.
        pages: Vec<usize>,
    },
    /// **Put the document's pages in a new order**, as one undoable command.
    ///
    /// Raised by `pages.move_up` and `pages.move_down`, which differ only in
    /// the permutation `crate::panels::pages::ops::move_order` computes.
    ///
    /// `order[i]` is the **current** 0-based index of the page that should end
    /// up at position `i` — `EditSession::reorder_pages`' contract verbatim,
    /// carried through unaltered so there is no second spelling of it to drift.
    /// The engine refuses anything that is not a permutation of
    /// `0..page_count`, and `move_order` builds one by construction.
    ///
    /// # ★ A reorder renumbers positions without destroying anything
    ///
    /// Which makes it the *middle* case between a move (nothing changes
    /// identity) and a delete (identities cease to exist), and the two
    /// selections get two different answers:
    ///
    /// * the **page** selection follows its sheets, through
    ///   [`crate::panels::pages::select::PageSelection::remap`] — the
    ///   permutation states exactly where each one went, so clearing would
    ///   throw away information the edit had in hand and make the reorder
    ///   arrows unusable twice in a row;
    /// * the **canvas** selection is cleared, because its entries carry a page
    ///   *index* and this crate cannot rewrite them —
    ///   `crate::canvas::selection::SelectionState` exposes no mutator for the
    ///   page of an entry, and inventing one would put a second page-remapping
    ///   rule in the module that owns object identity. Clearing is the honest
    ///   answer and it is stated rather than silent.
    ReorderPages {
        /// The new order, as `order[new_position] = current_index`.
        order: Vec<usize>,
    },
    /// **Write the operand pages out as a new standalone document.**
    ///
    /// Raised by `pages.extract`. The one page verb that changes **no**
    /// document: `pdfce_core::pageops::extract` returns the complete bytes of a
    /// freestanding PDF and the open session is not touched, which is exactly
    /// what the Review mode's stance requires — `crate::panels::pages`' header
    /// quotes the operator: *"an extraction writes a different file."*
    ///
    /// # ★ Why it is an action at all, when it mutates nothing
    ///
    /// For [`Self::SaveCopy`]'s reason and only that one: it opens a **native
    /// save dialog**, and `crate::app::files::pick_save_path` carries a
    /// frame-timing requirement dispatch cannot honour — `PdfceApp::central`
    /// dispatches the canvas's context-menu tokens from inside
    /// `egui::CentralPanel::show`, and a modal opened mid-layout blocks the
    /// frame it is being drawn in. The apply phase is always outside every
    /// closure. The page tile's context menu is dispatched from a panel body
    /// rather than the canvas, but the rule is the surface's, not the caller's.
    ExtractPages {
        /// 0-based page indices, ascending and unique. **Order is honoured** by
        /// the engine, so this is simultaneously "extract these pages" and
        /// "extract them in this order"; the panel produces them ascending.
        pages: Vec<usize>,
    },
}

/// **Bring everything stated in page indices back into agreement with the
/// session.**
///
/// Called from `super::apply::vector_edit`'s success path — every document
/// change in the application, including an undo or a redo of one. See the
/// module header for why it lives there rather than in the four page arms, and
/// for the table of what each kind of edit invalidates.
///
/// # The comparison, and why it is `(id, rotate)` rather than the whole page
///
/// `pdfce_core::page_tree::Page` is not `PartialEq` and comparing it fully
/// would compare two resolved `/Resources` dictionaries, which is expensive and
/// answers a question nobody asked. The pair below is the **complete** set of
/// page facts anything in this application caches:
///
/// * **`id`** — the page object's identity. A change in the *sequence* of ids
///   is a reorder or a delete, and it is the only signal that separates "page 3
///   is a different sheet now" from "page 3 looks different now". Nothing else
///   in `Page` can tell those apart, which is why identity rather than geometry
///   is the key.
/// * **`rotate`** — the one page attribute an edit in this build changes that
///   does not change the id. A rotation leaves every index meaning the same
///   sheet and makes every cached picture of it wrong.
///
/// A page whose *media box* changed would be missed. No verb in this build
/// changes one, and the honest note is here rather than in a comment claiming
/// completeness: the day a crop verb lands, its extent belongs in this pair.
///
/// # What a failed page walk does
///
/// Traces and returns, leaving the previous vector in place. The alternative —
/// emptying it — would turn a transient page-tree read failure into a document
/// that appears to have no pages, and an operator cannot save what the shell
/// has decided is empty. `page_tree::pages` fails only on structural damage,
/// which an edit through `EditSession` cannot introduce; this is the honest
/// answer for a case that should not arise rather than a case that is expected.
pub(super) fn resync(doc: &mut OpenDoc) {
    let before: Vec<(ObjId, u16)> = doc.pages.iter().map(|p| (p.id, p.rotate)).collect();
    let after = match doc.session.pages() {
        Ok(pages) => pages,
        Err(error) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("pages-resync-failed detail={error} kept={}", before.len())
            });
            return;
        }
    };
    let now: Vec<(ObjId, u16)> = after.iter().map(|p| (p.id, p.rotate)).collect();
    if now == before {
        // The overwhelmingly common case: a markup, a move, a form fill. The
        // page vector already describes the document, every raster is still a
        // picture of the right sheet, and nothing below has anything to do.
        return;
    }

    // ★ The identity sequence, which is the fact that decides whether an INDEX
    // changed meaning. A rotation leaves it alone; a delete and a reorder do
    // not. Compared before `doc.pages` is overwritten, because afterwards
    // there is nothing left to compare against.
    let renumbered = before
        .iter()
        .map(|(id, _)| *id)
        .ne(now.iter().map(|(id, _)| *id));

    doc.pages = after;
    let page_count = doc.pages.len();

    // Every cached raster in the strip is keyed on a page **index**, and an
    // index that has changed meaning — or a sheet that has been turned — makes
    // every one of them a picture of something else. Cleared wholesale rather
    // than selectively: working out which strip entries survive a permutation
    // is a second statement of the permutation, and the cache refills from the
    // visible set on the next frame anyway.
    doc.strip_rasters.clear();

    // ★ …and the CURRENT page's raster, for the same reason and only for that
    // reason — 2026-08-18.
    //
    // This line used to be absent, with a comment saying `page_texture` was
    // *"dropped by `vector_edit` itself, which is why it is not touched
    // here"*. That was true and it was also what made every ordinary edit
    // blank the page: `vector_edit` dropped the texture on **every** edit in
    // order to trigger a re-render, so the one case that genuinely needed it
    // never had to ask.
    //
    // Now `vector_edit` keeps the raster and signals staleness through
    // `page_texture_epoch`, so the drop belongs where its REASON is. The
    // distinction is the whole of it: after a content edit the old raster is
    // an older picture of the same sheet, and showing it for two frames is
    // right. After a delete or a reorder it is a picture of a **different
    // sheet** — the index resolves elsewhere — and showing it would be wrong
    // rather than merely late.
    doc.page_texture = None;

    if renumbered {
        // The canvas selection names objects by paint-order index **on a page
        // index**, and that index now resolves to a different sheet. Cleared
        // rather than remapped: `SelectionState` exposes no way to rewrite an
        // entry's page, and adding one would put a page-remapping rule inside
        // the module that owns object identity. See the module header's table.
        doc.selection.clear();
        // …and the text selection, for the same reason — except that this one
        // is already keyed on the epoch `vector_edit` has bumped, so dropping
        // it is belt to that braces. It is dropped anyway because "already
        // stale by its key" and "gone" read differently in a debugger, and the
        // wash it paints is the most visible piece of state in the canvas.
        doc.text_selection = None;
        // The operator may have been on a page that no longer exists.
        // `go_to_page` clamps, which is the only defined answer — there is no
        // page to return to, and refusing to move would leave the canvas
        // pointed past the end of the document.
        doc.view.go_to_page(doc.view.page_index, page_count);
        // `tracked_page` follows, exactly as it does for a page-display change:
        // leaving it behind makes `canvas::strip` read the clamp as a
        // *navigation* and scroll to it on the next frame, so a delete would
        // jump the view for a reason the operator did not cause.
        doc.tracked_page = doc.view.page_index;
    }

    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "pages-resync was={} now={page_count} renumbered={} page={} epoch={}",
            before.len(),
            u8::from(renumbered),
            doc.view.page_index + 1,
            doc.edit_epoch,
        )
    });
}

// ---------------------------------------------------------------------------
// The disclosures a delete owes
// ---------------------------------------------------------------------------

/// **What removing these pages broke, as operator-facing sentences.**
///
/// `vector_edit`'s disclosure channel carries exactly this shape of thing —
/// *"the drawing is unchanged but the file is not, and rule 4 forbids letting
/// the operator find that out from a diff"* — and a page delete is the verb
/// with the most to disclose in the whole application.
///
/// `EditSession::delete_pages` computes the census and deliberately does **not**
/// repair; its own documentation names surfacing the result as the front end's
/// job and calls Acrobat's silence *"a low bar, not a target to literally
/// copy."* This is that half.
///
/// # Returns
///
/// One sentence per fact that is true, in the order an operator would care
/// about them: the references they navigate by first, then the numbering, then
/// the prepress structure. **Empty** when nothing was broken, which is the
/// ordinary case for a drawing set and which makes `vector_edit` record no
/// sentence at all rather than an empty one — see its own docs on why that
/// distinction matters.
///
/// The wording is [`crate::text::pages`]', under rule R1. This function decides
/// *which* sentences and in what order, and no words at all.
///
/// # Why the two halves arrive separately
///
/// `DeleteOutcome` is `#[non_exhaustive]`, so no test outside `pdfce-core` can
/// build one — and a rule-4 disclosure whose *selection* logic cannot be
/// asserted is a rule-4 disclosure nobody has checked. `DanglingReport` is
/// `Default` and constructible field by field, so taking it plus the one
/// separation count keeps the decision testable. The caller does the
/// destructuring, which is one line and is the line that would not compile if
/// the engine's shape changed.
fn delete_disclosures(
    dangling: &pdfce_core::pageops::DanglingReport,
    sets_split: usize,
) -> Vec<String> {
    let mut notes = Vec::new();
    if dangling.outline_items > 0 {
        notes.push(t::deleted_dangling_bookmarks(dangling.outline_items));
    }
    if dangling.links > 0 {
        notes.push(t::deleted_dangling_links(dangling.links));
    }
    if dangling.named_destinations > 0 {
        notes.push(t::deleted_dangling_destinations(
            dangling.named_destinations,
        ));
    }
    if dangling.page_labels_stale {
        notes.push(t::deleted_page_labels_stale().to_owned());
    }
    if sets_split > 0 {
        notes.push(t::deleted_separations_repaired(sets_split));
    }
    notes
}

// ---------------------------------------------------------------------------
// The four verbs
// ---------------------------------------------------------------------------

/// **Insert another document's pages after `after_page`.**
///
/// # ★ Why this uses the SESSION verb and not `pageops::insert`
///
/// `pdfce_core::pageops::insert` also inserts pages and returns the bytes of a
/// **new document**. Wiring that would have meant replacing `OpenDoc::session`
/// wholesale — which discards the undo stack, invisibly to any test that
/// checks page counts, and visibly the first time an operator presses Ctrl+Z
/// twice.
///
/// So it was filed rather than shipped, and `pdfce-core` answered the same day
/// with `EditSession::insert_pages`: the missing member of the `delete_pages` /
/// `reorder_pages` / `rotate_pages` family. It records **one** undoable command
/// however many pages arrive, exactly as a reorder does however many move.
///
/// # What it does not carry
///
/// Page content, resources, fonts and XObjects come across at fresh object
/// numbers. The source's **document-level** structures do not — outlines, the
/// AcroForm field tree, named destinations, page labels. That is the honest
/// cost of staying incremental, because a document-level merge rewrites objects
/// an incremental save exists in order not to touch.
///
/// [`crate::text::pages::inserted`] says so in the disclosure, because an
/// operator whose bookmarks did not come across is entitled to know at the
/// moment it happened rather than by going looking for a bug.
///
/// # The three ways it can decline, and why they read differently
///
/// | condition | sentence |
/// |---|---|
/// | the file would not open | [`crate::text::pages::insert_failed`], carrying the engine's own reason — encrypted, truncated, not a PDF |
/// | it opened and has no pages | [`crate::text::pages::insert_empty`] — **not a failure**, and collapsing it into one would send the operator looking for corruption that is not there |
/// | the insert itself refused | `vector_edit`'s own decline path, as every other edit |
pub(super) fn insert_from_file(
    doc: &mut OpenDoc,
    path: &Path,
    pages: &[usize],
    position: pdfce_core::pageops::InsertPosition,
) {
    // Loaded OUTSIDE the edit closure and borrowed inside it: `insert_pages`
    // takes a `DocumentView` over it, so it has to outlive the call — and
    // loading it inside would mean deciding what to do about a *load* failure
    // from a context that can only report an *edit* failure.
    let source = match pdfce_core::document::Document::load(path) {
        Ok(source) => source,
        Err(error) => {
            let detail = error.to_string();
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "insert-pages-refused path={path:?} reason={detail}"
                )
            });
            super::record_note(doc.edit_epoch, crate::text::pages::insert_failed(&detail));
            return;
        }
    };
    if pages.is_empty() {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("insert-pages-refused path={path:?} reason=no-pages")
        });
        super::record_note(
            doc.edit_epoch,
            crate::text::pages::insert_empty().to_owned(),
        );
        return;
    }
    let view = source.view();
    let count = pages.len();

    // ★ Where the first inserted sheet will land, computed BEFORE the edit.
    //
    // Afterwards the document has more pages and `position` no longer names a
    // slot in it — `End` in particular means something different once the
    // pages have arrived. Working it out here is also what lets this be a
    // plain number rather than a second interpretation of `InsertPosition`.
    let landing = match position {
        pdfce_core::pageops::InsertPosition::Start => 0,
        pdfce_core::pageops::InsertPosition::End => doc.pages.len(),
        pdfce_core::pageops::InsertPosition::Before(n) => n,
        pdfce_core::pageops::InsertPosition::After(n) => n.saturating_add(1),
        // `InsertPosition` is `#[non_exhaustive]`: a variant added upstream
        // lands the operator on the page they were already on, which is wrong
        // but harmless, where a panic would lose the insert they just made.
        _ => doc.view.page_index,
    };
    let before = doc.pages.len();

    super::apply::vector_edit(doc, "insert-pages", landing, count, |session| {
        session
            .insert_pages(&view, pages, position)
            // ★ `InsertOutcome`, not a `usize`, since 2026-08-19 — and the
            // second field is the one this shell asked for. `orphaned_widgets`
            // is EXACT rather than an upper bound (the engine's reply: no field
            // in the target can be claiming a widget that just arrived, because
            // `/AcroForm` is not merged and every object number is remapped), so
            // the number goes in front of the operator unhedged and a zero drops
            // the clause entirely.
            //
            // ★ `orphaned_widgets_unrecoverable` joined it later the same day,
            // and the two numbers are two different pieces of news. The engine
            // measured its own output and found that of 13 orphans, 11 could be
            // registered and 2 had lost their identity permanently — and said
            // plainly that the old undifferentiated sentence *"is true of both
            // and useful for only one"*, because it describes a chore for the
            // 11 and a permanent loss for the 2, in the milder wording.
            .map(|outcome| {
                vec![crate::text::pages::inserted(
                    outcome.pages_inserted,
                    outcome.orphaned_widgets,
                    outcome.orphaned_widgets_unrecoverable,
                    crate::text::pages::Structures {
                        outline_dropped: outcome.source_outline_dropped,
                        labels_dropped: outcome.source_page_labels_dropped,
                        labels_stale: outcome.page_labels_stale,
                    },
                    landing,
                )]
            })
    });

    // ★★ GO TO WHAT WAS INSERTED — the half that makes this a feature rather
    // than a verb.
    //
    // An operator who inserts four sheets wants to see them; leaving the view
    // on the page they were reading means the only evidence anything happened
    // is a sentence in the status bar. `HANDOFF.md` §3 instruction 0 is exactly
    // this: *"what would a competent user reach for next, within this same
    // gesture?"* — and the answer is "look at them".
    //
    // Guarded on the page count actually having grown, so a refused insert
    // does not navigate: `vector_edit` reports a refusal to the trace and the
    // disclosure, and moving the view on a failure would be a second, wordless
    // claim that something landed.
    if doc.pages.len() > before {
        let target = landing.min(doc.pages.len().saturating_sub(1));
        doc.view.go_to_page(target, doc.pages.len());
        doc.tracked_page = doc.view.page_index;
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "insert-pages-landed at={target} pages={} was={before}",
                doc.pages.len()
            )
        });
    }
}

/// The engine call behind [`Action::RotatePages`].
///
/// Handed to `vector_edit` as a closure rather than run here, so the whole
/// four-step protocol — cancel the worker, mutate through `Arc::get_mut`, bump
/// the epoch, drop the texture — is the one in `apply.rs` and not a fifth copy.
///
/// The `usize` `rotate_pages` returns is how many pages **actually changed**,
/// and it is discarded: a page already at the requested rotation contributes
/// nothing and records no command, which is the engine's business and not a
/// disclosure. The empty list is the statement that a rotation rewrites no
/// operator's form and therefore owes rule 4 nothing.
///
/// [`Action::RotatePages`]: super::Action::RotatePages
pub(super) fn rotate(
    session: &mut EditSession,
    pages: &[usize],
    delta: i32,
) -> Result<Vec<String>, pdfce_core::edit::EditError> {
    session.rotate_pages(pages, delta).map(|_| Vec::new())
}

/// The engine call behind [`Action::DeletePages`], with its disclosures.
///
/// The one verb in this file whose return value carries sentences — see
/// [`delete_disclosures`].
///
/// [`Action::DeletePages`]: super::Action::DeletePages
pub(super) fn delete(
    session: &mut EditSession,
    pages: &[usize],
) -> Result<Vec<String>, pdfce_core::edit::EditError> {
    session.delete_pages(pages).map(|outcome| {
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "pages-deleted removed={} freed={} bookmarks={} links={} \
                 destinations={} labels_stale={}",
                outcome.pages_removed,
                outcome.objects_freed,
                outcome.dangling.outline_items,
                outcome.dangling.links,
                outcome.dangling.named_destinations,
                u8::from(outcome.dangling.page_labels_stale),
            )
        });
        delete_disclosures(&outcome.dangling, outcome.separations.sets_split)
    })
}

/// The engine call behind [`Action::ReorderPages`].
///
/// `order` is passed through untouched: `crate::panels::pages::ops::move_order`
/// builds it to `reorder_pages`' contract and the engine re-checks that it is a
/// permutation, so there is nothing for this layer to add and one more place
/// for a rule to be restated if it did.
///
/// [`Action::ReorderPages`]: super::Action::ReorderPages
pub(super) fn reorder(
    session: &mut EditSession,
    order: &[usize],
) -> Result<Vec<String>, pdfce_core::edit::EditError> {
    session.reorder_pages(order).map(|()| Vec::new())
}

// ---------------------------------------------------------------------------
// Extract — the verb that changes no document
// ---------------------------------------------------------------------------

/// **Write the operand pages out as a new standalone document.**
///
/// The whole of [`Action::ExtractPages`], and the one page verb that does not
/// go anywhere near `vector_edit`: `pdfce_core::pageops::extract` reads a
/// `DocumentView` and returns bytes. Nothing is mutated, so there is no worker
/// to cancel, no `Arc::get_mut` to fail, no epoch to bump and no texture to
/// drop — which is `crate::app::save`'s §2 argument for `file.save_copy`,
/// reaching the same conclusion for the same reason.
///
/// # ★ The view is the SESSION's, not the file's
///
/// `doc.session.view()` rather than the loaded `Document`, so an extraction
/// carries the operator's **unsaved edits** — decision 018, and the same choice
/// `file.copy_document_text` makes one dispatch arm over. An operator who
/// rotates three sheets and then extracts them must get the rotated sheets;
/// getting the file as it was opened would be a silent, plausible-looking
/// wrong answer.
///
/// # Why the destination is asked for rather than derived
///
/// The operator's standing rule — *Read may produce a new document; it may not
/// modify this one* — is enforced by **asking**, exactly as
/// `crate::app::files::pick_save_path`'s own docs describe: a path the operator
/// names cannot silently be the one they opened. [`suggested_path`] guarantees
/// the *suggestion* is never that file, so accepting the default without
/// reading it is safe too.
///
/// This is the third caller of that picker and it shares the
/// `PDFCE_DIAG_SAVE_PATH` seam with the other two, which is why
/// `tools/ui-verify`'s page-ops check can answer a native modal no synthetic
/// input can reach.
///
/// [`Action::ExtractPages`]: super::Action::ExtractPages
pub(super) fn extract(doc: &OpenDoc, pages: &[usize]) {
    if pages.is_empty() {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "extract-declined reason=no-pages".to_owned()
        });
        return;
    }
    let suggested = suggested_path(doc);
    let target =
        match files::pick_save_path(&suggested, crate::text::files::extract_pages_dialog_title()) {
            Picked::Path(path) => path,
            // A cancelled extraction is a complete, correct, uninteresting
            // outcome — `save_copy`'s wording, and its reasoning.
            Picked::Cancelled => return,
            Picked::Unavailable => {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "extract-unavailable reason=no-picker-in-this-build".to_owned()
                });
                return;
            }
        };
    write_extract(doc, pages, &target);
}

/// Assemble the new document and put it on disk, reporting on the trace.
///
/// Split from [`extract`] so the picker and the write are separable in the
/// reading as well as in the testing — `crate::app::save::write_and_report`'s
/// reason, and this half is the one a unit test can reach, because it never
/// opens a dialog.
fn write_extract(doc: &OpenDoc, pages: &[usize], target: &Path) {
    let assembled = pdfce_core::pageops::extract(&doc.session.view(), pages);
    let (bytes, report) = match assembled {
        Ok(pair) => pair,
        Err(error) => {
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "extract-failed path={target:?} n={} detail={error}",
                    pages.len()
                )
            });
            return;
        }
    };
    match std::fs::write(target, &bytes) {
        Ok(()) => crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                //
                // `pages=` beside `bytes=` for `HANDOFF.md` §2's reason about
                // the ink trail: a build that extracted the wrong count — the
                // whole document, say, or one page where three were picked —
                // writes a perfectly good PDF, and this field is the only
                // thing in the line that would differ. `path` is Debug-quoted
                // exactly as `save-copy`'s is, so a Windows path with a space
                // in it cannot make every field after it unreadable.
                "extract path={target:?} pages={} bytes={} asked={}",
                report.pages,
                bytes.len(),
                pages.len(),
            )
        }),
        Err(error) => crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "extract-failed path={target:?} bytes={} detail={error}",
                bytes.len()
            )
        }),
    }
}

/// The name and folder [`extract`] offers the picker.
///
/// `<stem>-pages.pdf` beside the document, which is
/// `crate::app::save::suggested_path`'s shape with
/// [`crate::text::files::extract_pages_suffix`] in place of `-copy`. It is a
/// separate function rather than a shared one taking a suffix because that one
/// is private to a module this work may not edit, and because the two are
/// allowed to diverge: an extraction of pages 3–7 could one day suggest a name
/// that says so, and a save-a-copy never could.
///
/// # ★ It is never the file that was opened
///
/// The promise [`crate::text::files::extract_pages_suffix`] makes, as a
/// mechanism. The extension is forced to `.pdf` for `save_copy`'s reason: the
/// bytes are a PDF whatever the source was called, and `SHEET.PDF` extracting
/// to `SHEET-pages.PDF` would be one more way for a downstream tool to disagree
/// about case.
fn suggested_path(doc: &OpenDoc) -> PathBuf {
    let Some(source) = doc.stored_under() else {
        // A created document has a name, not a location. Offer the name and
        // let the picker choose the folder — `save::suggested_path`'s answer
        // for the same state, and the only honest one.
        return doc.path.clone();
    };
    let stem = source.file_stem().map_or_else(
        // ui-text-exempt: a filename fallback for a path with no stem, not
        // operator copy.
        || String::from("document"),
        |s| s.to_string_lossy().into_owned(),
    );
    let name = format!("{stem}{}.pdf", crate::text::files::extract_pages_suffix());
    source
        .parent()
        .map_or_else(|| PathBuf::from(&name), |dir| dir.join(&name))
}

/// Apply one page verb, and do the invalidation it owes the shell.
///
/// ## ★ Why this takes `panels` as well as `doc`
///
/// Because the answer to *"what does this edit do to what is on screen?"* is
/// **different for each of the five**, and three of the answers are about the
/// Pages panel's own picks, which live on [`crate::panels::PanelsState`] rather
/// than on the document:
///
/// | verb | canvas selection | panel picks |
/// |---|---|---|
/// | rotate | kept — nothing renumbers | kept |
/// | reorder | cleared by the resync | **remapped** through the permutation |
/// | delete | cleared by the resync | **cleared** |
/// | insert | — | — (the view navigates instead) |
/// | extract | — | — (no document changes at all) |
///
/// `vector_edit` cannot reach `PanelsState`, so the choice has to be made by
/// the caller of it — and making it *here*, beside the bodies, is what keeps
/// the table above in one place instead of spread across five arms in the
/// interpreter.
///
/// ## ★ Every guard is on the EPOCH, not on a return value
///
/// A refused delete — the engine refuses removing every page, §7.7.3.3 — must
/// leave the operator's selection exactly as they built it. Testing whether
/// the epoch moved is what distinguishes *"the edit applied"* from *"the verb
/// was called"*, and it is the one signal that is true for every path including
/// a session that could not be borrowed.
pub(super) fn apply(
    doc: &mut OpenDoc,
    panels: &mut crate::panels::PanelsState,
    action: PageAction,
) {
    match action {
        // ===============================================================
        // ★ THE PAGE VERBS
        //
        // Four arms, each one call, because everything that could be a
        // rule lives elsewhere: the operand list and the permutation in
        // `crate::panels::pages::ops` (pure, unit-tested), the engine call
        // and the disclosures in `super::pages`, and the four-step
        // protocol in `vector_edit` — which now carries a fifth step that
        // brings the page vector, the strip rasters, the canvas selection
        // and the view back into agreement with the session.
        //
        // `page=` on the trace line is the FIRST operand rather than "the
        // page", and `n=` is how many were named. There is no single page
        // a multi-page verb is about; the first one is the honest answer
        // and `n=` is the field that actually says what happened, exactly
        // as `history_step`'s own docs argue for the undo case.
        // ===============================================================
        PageAction::RotatePages { pages, delta } => {
            if !pages.is_empty() {
                let first = pages.first().copied().unwrap_or(0);
                super::apply::vector_edit(doc, "rotate-pages", first, pages.len(), |session| {
                    rotate(session, &pages, delta)
                });
            }
        }
        // ★ **The destructive one**, and the one that renumbers.
        //
        // Two things happen here that no other arm needs, and both are
        // about a *position* ceasing to mean what it meant:
        //
        // 1. `vector_edit`'s resync clears the **canvas** selection and
        //    clamps the view — see `super::pages::resync`;
        // 2. the **Pages panel's** picks are cleared here, because they
        //    live on `self.panels` rather than on the document and
        //    `vector_edit` cannot reach them.
        //
        // The panel's own `retain_below` would drop the picks that fell
        // off the end on the next frame, and that is NOT sufficient: the
        // pages that were deleted are exactly the ones that were picked,
        // so the survivors of a clamp would be picks pointing at sheets
        // that have shuffled down into their indices. Clearing is both
        // correct and provable — every picked sheet is gone.
        //
        // Guarded on the epoch rather than on a return value, so the
        // clear happens only for an edit that actually applied: a refused
        // delete (the engine refuses removing every page, §7.7.3.3) must
        // leave the operator's selection exactly as they built it.
        //
        // **No confirmation dialog.** `crate::app::save::save_pending` is
        // the one predicate this application consults before a destructive
        // path and it is about a save being in flight, not about unsaved
        // work; the engine records this as an undoable command; and
        // nothing reaches disk — the operator's file is untouched until
        // they choose to save a copy. A modal here would be the only one
        // in the application and would be asking about the one destructive
        // act that is already reversible in the session.
        PageAction::DeletePages { pages } => {
            if !pages.is_empty() {
                let first = pages.first().copied().unwrap_or(0);
                let before = doc.edit_epoch;
                super::apply::vector_edit(doc, "delete-pages", first, pages.len(), |session| {
                    delete(session, &pages)
                });
                if doc.edit_epoch != before {
                    panels.pages_mut().selection.clear();
                }
            }
        }
        // ★ **The middle case**: every page survives, and every index
        // means a different sheet.
        //
        // The canvas selection is cleared by the resync; the panel's picks
        // are **remapped** rather than cleared, because the permutation
        // states exactly where each picked sheet went. See
        // `crate::panels::pages::select::PageSelection::remap` for why the
        // two selections get different answers to the same edit — and for
        // why clearing here would make the reorder arrows unusable twice
        // in a row, which is the one gesture they exist for.
        PageAction::ReorderPages { order } => {
            if !order.is_empty() {
                let before = doc.edit_epoch;
                super::apply::vector_edit(doc, "reorder-pages", 0, order.len(), |session| {
                    reorder(session, &order)
                });
                if doc.edit_epoch != before {
                    let landed = crate::panels::pages::ops::inverse(&order);
                    panels.pages_mut().selection.remap(&landed);
                }
            }
        }
        // ★ The one page verb that goes nowhere near `vector_edit`: it
        // changes no document, it opens a native save dialog, and it is an
        // `Action` for `Action::SaveCopy`'s frame-timing reason and only
        // that one. See `super::pages::extract`.
        // ★ The one verb that reads a SECOND document, and the only page
        // action whose consequence is a navigation rather than an
        // invalidation: `insert_from_file` goes to what it inserted, because
        // an operator who inserts four sheets wants to see them.
        PageAction::InsertPagesFromFile {
            path,
            pages,
            position,
        } => insert_from_file(doc, &path, &pages, position),
        PageAction::ExtractPages { pages } => extract(doc, &pages),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::{FOUR_PAGES, Origin, open_fixture};

    /// A scratch path under the OS temporary directory.
    ///
    /// `std::env::temp_dir` rather than a path in the repository, for
    /// `crate::app::save`'s stated reason: a test that writes beside the
    /// fixtures leaves a file somebody eventually commits.
    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("pdfce-gui-page-ops-tests");
        std::fs::create_dir_all(&dir).expect("the temporary directory must be creatable");
        dir.join(name)
    }

    /// Apply one engine verb to a fixture, the way `vector_edit` does.
    ///
    /// The four-step protocol is not re-run here — there is no render worker in
    /// a unit test and nothing else holds the `Arc` — but the two steps the
    /// assertions depend on are: the mutation, and the epoch bump that
    /// [`resync`] traces. Anything that needs the *whole* protocol is
    /// `tools/ui-verify`'s job, which is where the join is proven.
    fn edit(doc: &mut OpenDoc, verb: impl FnOnce(&mut EditSession)) {
        let session = std::sync::Arc::get_mut(&mut doc.session)
            .expect("nothing else holds the session in a test");
        verb(session);
        doc.edit_epoch = doc.edit_epoch.wrapping_add(1);
        resync(doc);
    }

    /// Put one object on `page` into the canvas selection.
    ///
    /// Through [`crate::canvas::selection::SelectionState::marquee`], which is
    /// a real gesture entry point, rather than by reaching into the struct:
    /// there is no setter, deliberately — the canvas is the only writer — and a
    /// test that needed one would be asking for an API the application does not
    /// have. A marquee of one target lands at the Object rung, which is the
    /// state a page edit has to invalidate.
    fn select_object_on(doc: &mut OpenDoc, page: usize) {
        use crate::panels::objects::provider::TargetId;
        doc.selection.marquee(page, &[TargetId(0)], false);
    }

    /// ★★ **A delete shortens the page vector, and the view follows it.**
    ///
    /// The defect this catches is silent and total: `OpenDoc::pages` is
    /// *"resolved once at open"*, so without [`resync`] the panel would go on
    /// saying "4 pages", the status bar would go on saying `n/4`, and the
    /// canvas would go on rendering a `Page` whose object the engine has
    /// **freed**. Every test in the crate would pass, because nothing else in
    /// the application ever re-reads that vector.
    #[test]
    fn a_delete_shortens_the_page_vector_and_clamps_the_view() {
        let mut doc = open_fixture(FOUR_PAGES);
        assert_eq!(doc.pages.len(), 4, "the fixture must have four pages");
        doc.view.page_index = 3;

        edit(&mut doc, |s| {
            s.delete_pages(&[2, 3]).expect("two of four pages may go");
        });

        assert_eq!(doc.pages.len(), 2, "the page vector must follow the delete");
        assert_eq!(
            doc.view.page_index, 1,
            "the operator was on page 4, which no longer exists; the view must land on the \
             last page that does rather than pointing past the end"
        );
        assert_eq!(
            doc.tracked_page, doc.view.page_index,
            "a clamp the canvas reads as a NAVIGATION scrolls the strip for a reason the \
             operator did not cause"
        );
    }

    /// **★ A delete clears the canvas selection, because its page index now
    /// names a different sheet.**
    ///
    /// The exact defect class `HANDOFF.md` §10 warns about, at page level: an
    /// entry that survived would resolve against another sheet's decomposition
    /// on the next frame and draw an outline round an object nobody selected —
    /// with `format.delete` one keystroke away.
    #[test]
    fn a_delete_clears_the_canvas_selection() {
        let mut doc = open_fixture(FOUR_PAGES);
        select_object_on(&mut doc, 2);
        assert!(!doc.selection.is_empty(), "the fixture selection must take");

        edit(&mut doc, |s| {
            s.delete_pages(&[0]).expect("one of four pages may go");
        });

        assert!(
            doc.selection.is_empty(),
            "the selection named an object on page 3, and page 3 is now the sheet that was \
             page 4 — the entry survives as a pointer at something nobody chose"
        );
    }

    /// **★ A reorder renumbers without shortening, and is treated as such.**
    ///
    /// The middle case, and the one a length comparison alone would miss
    /// entirely: the page count is unchanged, so a resync that only watched
    /// `len()` would leave every strip raster and the canvas selection pointing
    /// at sheets that have moved.
    #[test]
    fn a_reorder_renumbers_and_clears_the_canvas_selection() {
        let mut doc = open_fixture(FOUR_PAGES);
        let before: Vec<ObjId> = doc.pages.iter().map(|p| p.id).collect();
        select_object_on(&mut doc, 0);

        edit(&mut doc, |s| {
            s.reorder_pages(&[1, 0, 2, 3]).expect("a legal permutation");
        });

        assert_eq!(doc.pages.len(), 4, "a reorder removes nothing");
        let after: Vec<ObjId> = doc.pages.iter().map(|p| p.id).collect();
        assert_ne!(
            before, after,
            "the page vector still describes the old order, so every index in the application \
             now names the wrong sheet"
        );
        assert_eq!(after[0], before[1], "page 2 must have moved to position 1");
        assert!(
            doc.selection.is_empty(),
            "the selection's page index survived a permutation of the pages"
        );
    }

    /// **★★ A rotation is NOT a renumbering, and the selection survives it.**
    ///
    /// The falsifying half of the two tests above. A resync that cleared the
    /// selection on any change at all would pass both of them and would make
    /// every rotate throw away work the operator had done — and no assertion
    /// anywhere else would notice, because a cleared selection is a valid
    /// state.
    ///
    /// `crate::canvas::interact`'s header states the rule this pins: a verb
    /// that adds and removes no operator renumbers nothing.
    #[test]
    fn a_rotation_refreshes_the_pages_without_clearing_the_selection() {
        let mut doc = open_fixture(FOUR_PAGES);
        let before = doc.pages[1].rotate;
        select_object_on(&mut doc, 1);

        edit(&mut doc, |s| {
            s.rotate_pages(&[1], 90).expect("a quarter turn is legal");
        });

        assert_eq!(
            doc.pages[1].rotate,
            (before + 90) % 360,
            "the page vector still carries the old rotation, so the canvas would keep drawing \
             the sheet the way it was"
        );
        assert!(
            !doc.selection.is_empty(),
            "a rotation adds and removes no operator, so nothing renumbered and there was \
             nothing to clear"
        );
        assert_eq!(doc.pages.len(), 4);
    }

    /// An edit that touches no page leaves the vector alone and the selection
    /// with it.
    #[test]
    fn an_edit_that_changes_no_page_resyncs_nothing() {
        let mut doc = open_fixture(FOUR_PAGES);
        select_object_on(&mut doc, 0);
        let before: Vec<ObjId> = doc.pages.iter().map(|p| p.id).collect();

        edit(&mut doc, |_| {});

        assert_eq!(doc.pages.iter().map(|p| p.id).collect::<Vec<_>>(), before);
        assert!(!doc.selection.is_empty());
    }

    /// **★★ The extracted file is a real document containing exactly the pages
    /// that were asked for.**
    ///
    /// The round trip in the smallest form a unit test can hold, and the same
    /// shape as `app::save`'s: write it, re-open it from disk through the
    /// loader the application uses, and count. A build that wrote the whole
    /// document — the plausible wrong answer, since `extract` and `save_copy`
    /// both produce "a PDF beside the original" — passes any check that only
    /// asks whether a file appeared.
    #[test]
    fn an_extraction_writes_exactly_the_pages_it_was_given() {
        use pdfce_core::document::Document;

        let doc = open_fixture(FOUR_PAGES);
        let target = scratch("extracted.pdf");
        let _ = std::fs::remove_file(&target);

        write_extract(&doc, &[1, 2], &target);

        let written = std::fs::read(&target).expect("the extraction must land on disk");
        assert!(
            written.starts_with(b"%PDF-"),
            "a freestanding PDF, not a fragment"
        );
        let reopened = Document::load(&target).expect("the extraction must open");
        let pages = pdfce_core::page_tree::pages(&reopened).expect("its page tree must walk");
        assert_eq!(
            pages.len(),
            2,
            "two pages were asked for and the file has {}; a build that wrote the whole \
             document produces a perfectly good PDF and would pass any check that only asks \
             whether a file appeared",
            pages.len()
        );

        // …and the source is untouched. An extraction that modified the
        // document it read from would breach the operator's standing rule
        // outright, and it is asserted rather than assumed because `extract`
        // and `save_copy` share a picker and a suffix convention.
        assert_eq!(doc.pages.len(), 4);
        let _ = std::fs::remove_file(&target);
    }

    /// **★ An extraction carries the operator's unsaved edits.**
    ///
    /// Decision 018, asserted rather than trusted: the view handed to
    /// `pageops::extract` is the **session's**, so a rotation made in this
    /// sitting is in the file that comes out. A build that passed the loaded
    /// `Document` instead would produce a valid file with the right page count
    /// and the edit silently missing — which the test above would not catch.
    #[test]
    fn an_extraction_carries_unsaved_edits() {
        use pdfce_core::document::Document;

        let mut doc = open_fixture(FOUR_PAGES);
        let before = doc.pages[0].rotate;
        edit(&mut doc, |s| {
            s.rotate_pages(&[0], 90).expect("a quarter turn is legal");
        });

        let target = scratch("extracted-rotated.pdf");
        let _ = std::fs::remove_file(&target);
        write_extract(&doc, &[0], &target);

        let reopened = Document::load(&target).expect("the extraction must open");
        let pages = pdfce_core::page_tree::pages(&reopened).expect("its page tree must walk");
        assert_eq!(pages.len(), 1);
        assert_eq!(
            pages[0].rotate,
            (before + 90) % 360,
            "the extraction was assembled from the file as it was OPENED rather than from the \
             session, so the operator's rotation is not in it"
        );
        let _ = std::fs::remove_file(&target);
    }

    /// ★ **The suggested name is never the file that was opened.**
    ///
    /// `save::suggested_path`'s guarantee, for the second write-destination
    /// this shell asks about. An operator who accepts the suggestion without
    /// reading it must not overwrite the drawing they are extracting from.
    #[test]
    fn the_suggested_extract_name_is_never_the_source_file() {
        let mut doc = open_fixture(FOUR_PAGES);
        doc.path = PathBuf::from("D:\\jobs\\4471\\Sheet 1.pdf");
        doc.origin = Origin::Opened;

        let suggested = suggested_path(&doc);
        assert_ne!(suggested, doc.path);
        assert_eq!(
            suggested,
            PathBuf::from("D:\\jobs\\4471\\Sheet 1-pages.pdf")
        );
        assert_eq!(
            suggested.parent(),
            doc.path.parent(),
            "the extraction should land beside the original, where the operator will look"
        );
    }

    /// A delete with nothing to disclose produces **no** sentence, and one with
    /// something to disclose produces one per fact.
    ///
    /// The empty case matters as much as the full one: `vector_edit` records
    /// `None` for an empty list, so a build that returned a placeholder
    /// sentence would put a line under every page delete and train the operator
    /// to ignore the ones that mean something.
    #[test]
    fn a_delete_discloses_one_sentence_per_broken_thing() {
        let mut dangling = pdfce_core::pageops::DanglingReport::default();
        assert!(
            delete_disclosures(&dangling, 0).is_empty(),
            "a clean delete owes rule 4 nothing"
        );

        dangling.outline_items = 3;
        dangling.page_labels_stale = true;
        let notes = delete_disclosures(&dangling, 0);
        assert_eq!(
            notes.len(),
            2,
            "one sentence per fact, and no more: {notes:?}"
        );
        assert!(
            notes[0].contains('3'),
            "the count must reach the operator: {notes:?}"
        );
        assert!(
            notes.iter().all(|n| n.ends_with('.')),
            "these are prose and take a full stop, per the catalog's conventions: {notes:?}"
        );
    }
}
