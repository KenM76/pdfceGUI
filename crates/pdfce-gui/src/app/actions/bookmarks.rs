//! # `app::actions::bookmarks` — the three verbs whose subject is one entry in
//! the document's outline
//!
//! Split out of [`super::action`] under **R2** on 2026-08-28, the day the
//! family grew from one verb to three. `super`'s own declaration of `action`
//! wrote the rule down in advance — *"the next family of variants to **grow**
//! is the one that will have to become a sub-enum beside `PageAction` and
//! `DimensionAction`"* — and named markup as the measured candidate on the
//! grounds that markup was the largest. Markup did not grow that week;
//! bookmarks did, from `AddBookmark` alone to add, rename and delete, when
//! `pdfce-core` `Pass 156.0` shipped `set_outline_title` and
//! `delete_outline_item`. The measurement stands and is still the answer the
//! day markup grows; the rule was about growth, and this is what grew.
//!
//! ## What makes these a family rather than a size-driven cut
//!
//! Every variant here **addresses its operand by `ObjId`** and by nothing
//! else, and that is a property no other family in the enum has for the same
//! reason. The reason is the one the engine reported from its own CLI:
//!
//! > *"the indices shift after every add … I got this wrong myself while
//! > driving the command and nested something two levels deeper than intended,
//! > and the output looked entirely plausible."*
//!
//! An outline is a tree that every edit to it renumbers. A position in the
//! walk — "the fourth row", "the second child of the first" — names a
//! different bookmark after any add, any delete, and any undo of either.
//! `OutlineItem::id` exists precisely so a GUI does not have to hold one, and
//! its own doc comment says so: *"identity is what a GUI needs and the tree
//! cannot otherwise supply."*
//!
//! ⇒ So the shared property is not *"they are all about bookmarks"*, which
//! would be a subject label. It is that **all three are resolvable after the
//! frame that raised them**, which is the one thing the action funnel requires
//! of an operand and the one thing a tree position cannot promise.
//!
//! ## ★★ `/Count` is two different quantities, and the sign carries open/closed
//!
//! §12.3.3 is where implementations of this feature go wrong, and the engine
//! sent the table to this shell unprompted because it expected us to build a
//! panel:
//!
//! | | root `/Outlines` (Table 152) | an item (Table 153) |
//! |---|---|---|
//! | counts | all visible items **including** the top level | visible **descendants**, excluding itself |
//! | sign | **cannot** be negative | **positive = open, negative = closed** |
//!
//! A **closed** item contributes exactly **1** to its ancestors' counts,
//! however large its subtree is. Three consequences, and this module is built
//! around all three:
//!
//! 1. **Nothing here diffs a count to describe an edit.** Adding a bookmark
//!    under a collapsed ancestor leaves the document's total unchanged, so a
//!    surface reporting *"added N"* from a root-count diff reports **zero for
//!    a correct save**. [`BookmarkAction::Add`] adds one bookmark and the panel
//!    says one bookmark; there is no number to get wrong.
//! 2. **A delete's count comes from the engine, not from the tree we drew.**
//!    See [`BookmarkAction::Delete`].
//! 3. **`open` is the only reason a disclosure about visibility can be
//!    written at all.** `pdfce_core::outline::OutlineItem::open` is the shell's
//!    read of that sign, and §12.3.3 defines no `/Open` key, so the sign is the
//!    only carrier there is.
//!
//! ## What this module does NOT do, and why the absence is deliberate
//!
//! **Reorder and re-parent.** The engine's note of 2026-08-28 lists them as
//! not shipped — *"Reorder and re-parent do not [ship], and neither has a CLI
//! subcommand yet"* — so there is no variant for either and no drag handle in
//! the panel. R9: a capability that does not exist renders nothing. A greyed
//! "Move up" button would be a promise the engine cannot keep, and greying is
//! reserved for something that is *temporarily* unavailable and can say when
//! it will not be.

use pdfce_core::object::ObjId;

use crate::app::state::OpenDoc;

/// The verbs whose subject is one entry in the document's outline.
///
/// See the module header for what makes them a family: every one of them names
/// its operand by `ObjId`, because an outline is a tree that every edit to it
/// renumbers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BookmarkAction {
    /// ★ **Add a bookmark to the document's outline.**
    ///
    /// Raised by `crate::panels::bookmarks::add` and by nothing else.
    ///
    /// # ★ Why nothing here counts anything
    ///
    /// `EditSession::add_outline_item` maintains `/Count`, and `/Count` is two
    /// different quantities — see the module header's table. The consequence
    /// the engine flagged as *"the entire difficulty of the feature"*: **adding
    /// a bookmark under a collapsed ancestor does not change the document's
    /// total**, because the new item is not visible. A surface reporting
    /// *"added N"* by diffing the root count therefore reports **zero for a
    /// correct save**.
    ///
    /// So this variant carries one bookmark, the apply arm adds one bookmark,
    /// and the panel says one bookmark. There is no number to get wrong.
    ///
    /// # Why the parent is an `ObjId` and not a position
    ///
    /// Because a position is invalidated by the very edit this performs. The
    /// engine hit that in its own CLI — *"the indices shift after every add …
    /// I got this wrong myself while driving the command and nested something
    /// two levels deeper than intended, and the output looked entirely
    /// plausible."* `OutlineItem::id` exists for this.
    ///
    /// `None` is the top level, which is `add_outline_item`'s own spelling.
    Add {
        /// The item it goes under, or `None` for the top level.
        parent: Option<ObjId>,
        /// The title. Trimmed and non-empty by the time it gets here.
        title: String,
        /// The 0-based page it points at — the one the operator is looking at.
        page: usize,
    },
    /// ★ **Rename a bookmark** — write a new `/Title` onto one outline item.
    ///
    /// Raised by `crate::panels::bookmarks::edit` and by nothing else.
    /// `pdfce-core` `Pass 156.0`; the engine's covering note calls it *"the
    /// commonest bookmark edit there is"*, which is why it is the verb the
    /// panel puts first once a row is selected.
    ///
    /// # ★ The verb with no structural risk, and saying so is load-bearing
    ///
    /// `set_outline_title`'s own doc comment is unusually reassuring, and the
    /// reassurance is a fact a reader of *this* file needs:
    ///
    /// > *"a title is a text string (§7.9.2) on one dictionary, and nothing in
    /// > the `/First`/`/Last`/`/Next`/`/Prev`/`/Count` machinery depends on
    /// > it."*
    ///
    /// ⇒ **A rename cannot move, orphan, hide or renumber anything.** That is
    /// why this arm reports no disclosure at all: there is no consequence the
    /// operator cannot see. The new title appears in the row they are looking
    /// at, on the next frame, and that is the whole of what happened. Every
    /// other verb in this enum owes a sentence to `app::status`; this one owes
    /// none, and inventing one — *"Bookmark renamed."* under a row that now
    /// visibly reads the new name — would be noise standing where a real
    /// disclosure belongs.
    ///
    /// # Why the title travels by value
    ///
    /// The panel holds a **draft** that the operator is still typing into, and
    /// the queue drains after the frame. Borrowing it would tie the action's
    /// lifetime to the panel state, which `PdfceApp::apply` cannot reach — it
    /// has no `egui::Context` and deliberately does not — so the operand comes
    /// with it, which is what an action *is*: a complete statement of intent,
    /// resolvable after the frame that raised it.
    ///
    /// Encoding is the engine's problem and is documented as deliberately not
    /// ours: `set_outline_title` routes through *"the same `crate::textstring`
    /// path every other text string uses"*, because `Pass 150.0` shipped a
    /// defect from two paths disagreeing about PDFDocEncoding. So an em dash or
    /// an accented name in this `String` needs nothing from this crate.
    Rename {
        /// The outline item whose `/Title` is being replaced.
        item: ObjId,
        /// The new title. Trimmed and non-empty by the time it gets here — a
        /// bookmark with a blank title is legal and is an invisible row, which
        /// is the same defect as no row.
        title: String,
    },
    /// ★★★ **Delete a bookmark AND everything under it.**
    ///
    /// Raised by `crate::panels::bookmarks::edit` and by nothing else.
    /// `pdfce-core` `Pass 156.0`.
    ///
    /// # ★★ The subtree goes too, and that is a decision with a reason
    ///
    /// The engine takes Acrobat's behaviour and states the alternative it
    /// rejected, which is the part worth carrying here because it is the part
    /// an operator would otherwise discover:
    ///
    /// > *"promoting orphaned children to the deleted item's parent silently
    /// > **reorganises** a document's navigation, and an operator who deleted
    /// > one chapter heading would find its ten sections spliced into the top
    /// > level. Deleting what was asked for is the predictable act."*
    ///
    /// ⇒ This is therefore a verb whose blast radius is **larger than the thing
    /// the operator clicked**, and the whole of the UI obligation follows from
    /// that one sentence. It is stated before the press by
    /// `crate::panels::bookmarks::edit`, from the tree the panel already drew,
    /// and it is stated again after the press from the engine's own count. See
    /// [`delete`] for why the answer is given twice and why the two numbers are
    /// allowed to differ.
    ///
    /// # Why there is no confirmation dialog, and it IS a choice
    ///
    /// `HANDOFF.md`'s rule is *confirmed or clearly undoable*, and this is the
    /// second. One press produces **one** `EditSession` command, so one
    /// `Ctrl+Z` puts the entire subtree back — the engine plans every relink
    /// (`/Prev`, `/Next`, the parent's `/First`/`/Last`, every open ancestor's
    /// `/Count`) inside that one command, so there is no half-undone state to
    /// reach. A modal would buy nothing that the undo does not already buy, and
    /// it would cost the thing modals always cost: an operator who has answered
    /// *"are you sure?"* four times stops reading it, and the fifth one is the
    /// one that mattered.
    ///
    /// The consequence the operator actually needs is **not** *"are you
    /// sure?"* — it is *"this takes the eleven bookmarks underneath as well"*,
    /// which a confirmation dialog is a bad place to put because it arrives
    /// after the decision. It is on the panel, beside the button, before the
    /// press.
    ///
    /// # No page index
    ///
    /// An outline is a document-level structure (§12.3.3) reached from the
    /// catalogue's `/Outlines`, not from any page. The item's own destination
    /// may name a page, and it is irrelevant here: this deletes the bookmark,
    /// never the page it points at, and nothing on any page changes.
    Delete {
        /// The outline item to remove, together with its whole subtree.
        item: ObjId,
    },
}

/// Apply one bookmark verb.
///
/// The dispatch half of this module, reached from `PdfceApp::apply`'s single
/// [`super::action::Action::Bookmark`] arm. It is a free function taking
/// `&mut OpenDoc` rather than a method, exactly like [`super::dimensions::apply`]
/// and [`super::pages::apply`], because the caller is the one place that owns
/// the borrow and the arm should be one line.
///
/// **Every arm goes through [`super::apply::vector_edit`]** — the
/// cancel–mutate–bump–invalidate protocol — and none of them may hand-roll it.
/// Its doc comment carries the argument: four hand-written copies of a
/// four-step protocol are four chances to omit a step, and the two steps most
/// easily omitted (the epoch bump and the structural resync) fail *silently*,
/// leaving an edit that happened in the document and did not happen on screen.
///
/// ★ The `page` argument passed to `vector_edit` is **`0` for all three**, and
/// that is honest rather than lazy: an outline is document-level, no page is
/// being edited, and the parameter exists only so the diagnostic trace can say
/// which sheet a geometry edit touched. [`super::dimensions::apply`] passes `0`
/// for its group verbs for the identical reason. The one exception is
/// [`BookmarkAction::Add`], which passes the destination page — not because a
/// page is being changed, but because the page is the operand that decides what
/// the bookmark points at, and a trace that could not say which one would be
/// unable to check the commonest thing to get wrong.
pub(super) fn apply(doc: &mut OpenDoc, action: BookmarkAction) {
    match action {
        // ★ One bookmark, one undo entry, and NO count reported.
        //
        // See the variant: `/Count` is two quantities and its sign is the
        // open/closed flag, so a bookmark added under a collapsed ancestor
        // leaves the document's total unchanged. A disclosure built by diffing
        // it would say "0" for a correct save.
        //
        // The destination is an explicit page at `Fit`, which is the only form
        // `add_outline_item` authors without refusing — named and remote
        // destinations are refused by name, and `DestView::Unknown` is refused
        // because the reader keeps an extension's fit NAME and discards its
        // parameters, so re-emitting it would write a view that is not the one
        // the source had.
        BookmarkAction::Add {
            parent,
            title,
            page,
        } => {
            super::apply::vector_edit(doc, "add-bookmark", page, 1, |session| {
                session
                    .add_outline_item(
                        parent,
                        &title,
                        Some(pdfce_core::outline::Destination::Page {
                            page_index: page,
                            view: pdfce_core::outline::DestView::Fit,
                        }),
                    )
                    .map(|_| Vec::new())
            });
        }
        BookmarkAction::Rename { item, title } => rename(doc, item, &title),
        BookmarkAction::Delete { item } => delete(doc, item),
    }
}

/// **Rename one bookmark**, as one undoable command, disclosing nothing.
///
/// # Why the disclosure list is empty, deliberately
///
/// `vector_edit` surfaces whatever this returns to `app::status`, and the rule
/// that module's header states is that a disclosure is *"the part they cannot
/// see"*. A rename has no such part. `set_outline_title` writes `/Title` on one
/// dictionary and touches nothing else — its own doc comment says *"nothing in
/// the `/First`/`/Last`/`/Next`/`/Prev`/`/Count` machinery depends on it"* — so
/// the entire effect of this call is a row in the panel beside the operator's
/// pointer reading the words they just typed.
///
/// Emitting *"Bookmark renamed."* would put a sentence in the one slot
/// `app::status` has for consequences, describing something with no
/// consequences, and it would evict the previous edit's real disclosure to do
/// it. `super::forms::rename` reports one because a form field's rename
/// **does** have an invisible part: renaming a parent renames its descendants,
/// and the count of them is not on screen. Nothing analogous exists here.
///
/// # What happens on a refusal
///
/// `vector_edit` traces it and leaves the document alone, which is the whole
/// response this shell gives to any engine refusal today. Three are reachable:
/// `DocumentEncrypted`, the certification gate, and `NotADictionary` if the id
/// no longer resolves to an outline item — which is what an id from a stale
/// draft becomes after an undo. The panel does not pre-empt any of them; see
/// `crate::panels::bookmarks::edit`'s header for why the encryption case is
/// not gated at the widget.
fn rename(doc: &mut OpenDoc, item: ObjId, title: &str) {
    super::apply::vector_edit(doc, "rename-bookmark", 0, 1, |session| {
        session.set_outline_title(item, title).map(|()| Vec::new())
    });
}

/// **Delete one bookmark and its whole subtree**, as one undoable command,
/// disclosing how many items went.
///
/// # ★★ The count is the disclosure, and it comes from the engine
///
/// `delete_outline_item` returns `usize` — the number of items actually
/// removed, the clicked one included. That number is the answer to the question
/// this verb raises and cannot answer any other way: **the subtree went too**,
/// and on a collapsed parent the operator could not see how large it was.
///
/// This is `HANDOFF.md`'s *"disclose off-canvas, never on the page"* in its
/// plainest form. The panel already stated the expected size before the press,
/// from the tree it had drawn; this states what the engine actually removed.
///
/// ★ **The two numbers are allowed to differ, and that is the reason both are
/// said.** `read_outline` gives up part-way on a cycle, on excessive depth, or
/// on exhausting its item budget — the panel draws a truncation notice when it
/// does — so the shell's pre-press count is a count of *what pdfce could
/// read*, and the engine's is a count of *what it removed*. On any ordinary
/// document they agree. On a damaged one the after-the-fact number is the true
/// one, and an operator who saw "3" promised and "47" reported has been told
/// something real about their file rather than being quietly lied to by the
/// only number they were shown.
///
/// # Why one is not spelled as none
///
/// Deleting a leaf removes exactly one item, and *"Bookmark deleted, including
/// its 0 bookmarks beneath it"* is the shape of sentence that makes a program
/// look like it is reading from a template. The catalog branches on the count;
/// see `crate::text::panels::bookmark_deleted`.
///
/// # What it cannot be asked to do
///
/// The **outline root** is refused by name — `EditError::OutlineRootIsNotAnItem`
/// — because the root is not an item, carries no `/Title`, and deleting it means
/// deleting the whole outline, *"a different act that gets its own verb when it
/// is wanted."* The panel cannot raise that refusal: `read_outline` reports the
/// root's *children* as its top-level items, so no `ObjId` the panel can offer
/// is the root's. The refusal is therefore unreachable from this surface rather
/// than routed around, which is the better of the two outcomes and is recorded
/// here so nobody adds a guard for a case that cannot occur.
fn delete(doc: &mut OpenDoc, item: ObjId) {
    super::apply::vector_edit(doc, "delete-bookmark", 0, 1, |session| {
        session
            .delete_outline_item(item)
            .map(|removed| vec![crate::text::panels::bookmark_deleted(removed)])
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The three verbs are three distinct values**, so a match on them
    /// cannot silently collapse.
    ///
    /// `PartialEq` is derived and `super::super::tests` compares whole
    /// `Action`s, so a variant that failed to distinguish itself would make
    /// those comparisons pass for the wrong reason — the exact failure mode
    /// the engine reported on this same Pass, where *"all three of our
    /// sabotage checks survived the first test suite"* because the fixtures
    /// could not tell the answers apart.
    #[test]
    fn the_three_verbs_are_distinguishable() {
        let id = ObjId::new(7, 0);
        let add = BookmarkAction::Add {
            parent: Some(id),
            title: "Chapter 3".to_owned(),
            page: 4,
        };
        let rename = BookmarkAction::Rename {
            item: id,
            title: "Chapter 3".to_owned(),
        };
        let delete = BookmarkAction::Delete { item: id };
        assert_ne!(add, rename);
        assert_ne!(rename, delete);
        assert_ne!(add, delete);
    }

    /// ★ **A rename of the same item to two different titles is two different
    /// actions**, and a rename of two different items to the same title is
    /// too.
    ///
    /// Both halves matter and only one of them is obvious. The queue may hold
    /// more than one action from a single frame, and `PdfceApp::apply` applies
    /// them in order; a variant that compared equal on only one of its two
    /// fields would let a de-duplicating caller — or a test asserting "the
    /// queue holds what I expected" — accept the wrong one.
    ///
    /// The fixture deliberately makes the two answers different in each
    /// direction, which is the discipline the engine's note asks for: *"when
    /// you assert that A and B differ, check your fixture can tell them
    /// apart."*
    #[test]
    fn a_rename_is_identified_by_both_its_item_and_its_title() {
        let a = ObjId::new(7, 0);
        let b = ObjId::new(8, 0);
        let same_item_new_title = (
            BookmarkAction::Rename {
                item: a,
                title: "one".to_owned(),
            },
            BookmarkAction::Rename {
                item: a,
                title: "two".to_owned(),
            },
        );
        assert_ne!(same_item_new_title.0, same_item_new_title.1);

        let same_title_new_item = (
            BookmarkAction::Rename {
                item: a,
                title: "one".to_owned(),
            },
            BookmarkAction::Rename {
                item: b,
                title: "one".to_owned(),
            },
        );
        assert_ne!(same_title_new_item.0, same_title_new_item.1);
    }

    /// ★ **The generation number is part of the identity.**
    ///
    /// `ObjId` is `(num, generation)`, and a delete addressed to `7 0 R` must
    /// not compare equal to one addressed to `7 1 R`. This is cheap to assert
    /// and it pins the thing that would make the whole "address by id, never
    /// by position" argument in the module header hollow: an id that only
    /// half-identifies is a position with extra steps.
    #[test]
    fn an_objid_generation_distinguishes_two_deletes() {
        assert_ne!(
            BookmarkAction::Delete {
                item: ObjId::new(7, 0)
            },
            BookmarkAction::Delete {
                item: ObjId::new(7, 1)
            },
        );
    }
}
