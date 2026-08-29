//! # `app::actions::xobject` — the verbs whose subject is a form XObject
//!
//! One verb today: **give this page its own private copy of a shared drawing**,
//! so that a later edit to it changes this page and no other.
//!
//! Written 2026-08-28, when `EDITABLE_SURFACES.md` — an audit keyed on the
//! *engine's* verb list rather than on this shell's feature list — found
//! `EditSession::unshare_form` implemented in `pdfce-core` and named nowhere in
//! `crates/pdfce-gui/src`.
//!
//! ## ★★★ Why this is a sub-enum file on day one, holding one variant
//!
//! [`super::action`]'s rule is *"the next family of variants to **grow**"*, and
//! a family of one has not grown. Three answers, and the third is the one that
//! decides it:
//!
//! 1. **`action.rs` is at 1,441 of R2's 1,500 lines.** A variant with the doc
//!    comment this one needs — an operand derived through two hops, a
//!    granularity decision that is the engine's rather than ours, and a
//!    refusal ladder of seven — does not fit, and trimming the prose to make it
//!    fit is the response R2's own message forbids: *"split the module along
//!    its seams rather than raising the limit"*, and the seam is not "wherever
//!    the count ran out".
//! 2. **`super::attachments` is the precedent for arriving as a sub-enum**, and
//!    its header states the rule this follows: *"a family that arrives with
//!    three verbs at once has grown before anybody had to measure it."* This
//!    family arrives with one — see the next point for why it is nonetheless a
//!    family rather than a stray.
//! 3. ★★ **The family is defined by its OPERAND, and the operand is unique in
//!    this crate.** Every other authoring verb here addresses either a
//!    paint-order index into one content stream (`super::vector`), a stable
//!    annotation `ObjId` (`super::annot`), an outline item `ObjId`
//!    (`super::bookmarks`), a page index (`super::pages`) or a byte span into a
//!    decoded buffer (`super::textstyle`). This one addresses **a form XObject
//!    stream object, paired with the page that invokes it** — a `(usize,
//!    ObjId)` whose two halves are not independent, because the verb's whole
//!    subject is the relationship between them. `EDITABLE_SURFACES.md` lists
//!    three further form-XObject verbs as open gaps; they land here, and they
//!    land here because of the operand, not because of the noun.
//!
//! ## ★★★ The one fact a reader of this file must not get wrong
//!
//! **The granularity is one PAGE, not one invocation**, and it is the engine's
//! decision rather than a simplification made here. From `unshare_form`'s own
//! documentation:
//!
//! > If this page invokes the form under several names, **all of them** are
//! > re-pointed at the one copy. The unit decision 076 speaks of is the page;
//! > splitting two invocations *on the same page* would need a per-invocation
//! > identity the object model does not carry, and would make "unshare" mean
//! > something different depending on how many times the page happened to draw
//! > it.
//!
//! ⇒ So there is deliberately **no** variant here that takes a `TargetId`, a
//! paint-order index, or anything else naming *which* of a page's several
//! invocations was clicked. Adding one would be offering a granularity the
//! engine does not implement — a placeholder wearing an enum variant, which is
//! what `super`'s `OVERVIEW.md` forbids in as many words when it explains why
//! there is no `ResizeSelection`.
//!
//! ## ★★ What this file does NOT do, and where it happens instead
//!
//! **It does not derive the operand.** The `ObjId` arrives already resolved,
//! from `crate::app::dispatch::format`, which reads the first leaf of the
//! selection and asks
//! `panels::objects::provider::ObjectModelProvider::containing_form_object` for
//! the **outermost** enclosing form. That method's doc comment carries the
//! whole argument for why position 0 of `FormLeaf::containment` is the only
//! element the verb accepts — the last element is the *innermost* form, which
//! is exactly the operand `EditError::FormNestedInAnotherForm` exists to
//! refuse.
//!
//! Deriving it here instead was considered and is wrong for the funnel's own
//! reason, stated in `OVERVIEW.md`: an `Action` is *"a complete statement of an
//! operator's intent, resolvable after the frame that raised it"*. A `TargetId`
//! is not resolvable after the frame — an edit between the gesture and the
//! queue draining renumbers the page — and an `ObjId` is. The resolution
//! belongs on the near side of the funnel; the operand travels.

use pdfce_core::object::ObjId;

use crate::app::state::OpenDoc;
use crate::text::unshare::UnshareRefusal;

/// The verbs whose subject is a form XObject — a drawing invoked by a page,
/// possibly by many pages, possibly several times by one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XObjectAction {
    /// ★★★ **Give this page its own private copy of a shared drawing.**
    ///
    /// `EditSession::unshare_form`. Raised by `crate::app::dispatch::format`'s
    /// `format.unshare_form` arm — from the Format contextual tab and from the
    /// canvas context menu — and by nothing else.
    ///
    /// # Why the operator needs this, concretely
    ///
    /// ISO 32000-1 §8.10.1 names a CAD system's standard component as the
    /// *purpose* of form XObjects, and this operator's drawing sets are exactly
    /// that: one title block, one stream object, invoked from thirty-six
    /// sheets. Since `pdfce-core` `Pass 119.0` this shell can **edit text
    /// inside a form**, which means an operator fixing a typo on sheet 12
    /// changes all thirty-six — and pdfce cannot prevent that structurally,
    /// because there is exactly one stream object to write.
    ///
    /// `pdfce-core`'s decision 076 ruled that edit-in-place-and-disclose is the
    /// **default**, and `R206` requires that two defensible behaviours ship as
    /// two options. This variant is the second option. Until it existed the
    /// operator had the default and no choice at all, which is the state `R206`
    /// exists to prevent.
    ///
    /// # ★★ Both fields are load-bearing and neither is redundant
    ///
    /// `page` is **not** merely for the trace, unlike
    /// `super::annot::AnnotAction::Delete`'s. The verb's signature is
    /// `(page_index, form)` and the page is half the operand: unsharing is
    /// defined as *"re-point **this page's** references"*, and the same form on
    /// a different page is a different, equally valid call that this one must
    /// not perform.
    ///
    /// `form` is the **outermost** enclosing form's `ObjId`, resolved before the
    /// action was raised. See the module header for why that resolution is not
    /// done here and why the innermost form would be refused.
    ///
    /// # `Copy`, which its neighbours are not
    ///
    /// Both fields are `Copy` — a `usize` and an `ObjId` — so the whole enum is,
    /// and `Action` is not made heavier by carrying it. `super::annot` and
    /// `super::bookmarks` are not `Copy` because they carry `String`s and
    /// `Vec`s; nothing here needs one, and nothing here should grow one: a
    /// second copy of a name the document already holds is how a stale operand
    /// gets written back.
    Unshare {
        /// The 0-based page whose references move. Half the operand, not a
        /// trace field.
        page: usize,
        /// The **outermost** enclosing form's stream object.
        form: ObjId,
    },
}

/// Apply one form-XObject verb.
///
/// One arm today, matching its neighbours' shape: `super::bookmarks::apply` and
/// `super::attachments::apply` are both reached from `super::apply` by a single
/// line, so the family's rules live with the family rather than in the
/// interpreter's match.
pub(super) fn apply(doc: &mut OpenDoc, action: XObjectAction) {
    match action {
        XObjectAction::Unshare { page, form } => unshare(doc, page, form),
    }
}

/// **Clone a shared form XObject for one page, as one undoable command.**
///
/// # ★★★ Every refusal is caught INSIDE the closure and worded
///
/// `super::apply::vector_edit`'s `Err` arm traces and says nothing to the
/// operator, and its own comment explains why that is a scope statement rather
/// than an oversight: wording a decline is catalog work per refusal, and a
/// `format!` of an `EditError`'s `Display` would route diagnostic prose into
/// the UI. So a verb that owes sentences words them here, from inside the
/// closure, the way `super::annots::resize` and `super::annots::rotate` do.
///
/// **This verb owes a sentence for every one of its refusals**, which is
/// unusual — `resize` words exactly one of six — and the reason is a property
/// of what a refusal *looks like* here rather than of how many there are:
///
/// | after a refusal, the operator sees | and infers |
/// |---|---|
/// | the page, unchanged | "it worked — the copy is identical, after all" |
/// | no outline moving, no colour changing | "…so I can safely type in the title block now" |
///
/// A silent decline on this command does not read as "nothing happened". It
/// reads as **success**, because success looks like nothing happening too. The
/// operator then edits a title block they still share with thirty-five other
/// sheets, believing they have privatised it. That is the most expensive
/// failure this shell can produce from one unworded branch, and it is why
/// `crate::text::unshare`'s sentences all end by restating that the sharing is
/// untouched.
///
/// ★★ Recorded from **inside** the closure rather than before the call, for
/// `record_resize_not_rebuildable`'s stated reason: whether the engine will
/// refuse is a property of the FILE — is it encrypted, is it certified, is its
/// `/Size` suppressing entries, is this form reached only from inside another —
/// and none of those is knowable from the selection the dispatcher holds. The
/// one refusal that *is* a query the shell can answer itself
/// ([`UnshareRefusal::NothingInAForm`]) is recorded in the dispatcher, which is
/// the same placement `record_inside_form` uses and for the same reason.
///
/// # The disclosure
///
/// `UnshareFormReport` names the copy, the original and how many references
/// moved. The two object ids go to the **trace** and the count goes to the
/// **status row**, which is `canvas::textedit::report`'s rule applied
/// unchanged: a number about a content stream is evidence, and evidence belongs
/// where a driven check can read it; a count of places on the sheet in front of
/// the operator is a disclosure.
fn unshare(doc: &mut OpenDoc, page: usize, form: ObjId) {
    super::apply::vector_edit(doc, "unshare-form", page, 1, |session| {
        session
            .unshare_form(page, form)
            .inspect_err(|error| {
                crate::app::status::decline::record_unshare(refusal_for(error));
            })
            .map(|report| {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    //
                    // ★ It names BOTH object numbers and the count. The count
                    // is also on the status row, and that duplication is
                    // deliberate: a driven check must be able to assert the
                    // number without reading prose, and the two coming from one
                    // `report` value means they cannot disagree.
                    //
                    // `original` is on the line because it is the number a
                    // wrong build gets wrong in the quietest way — a verb handed
                    // the INNERMOST form instead of the outermost would refuse
                    // on a nested drawing and, on a singly-nested one, would
                    // succeed against the wrong object.
                    format!(
                        "unshare-form-applied page={page} original={} copy={} moved={}",
                        report.original.num, report.copy.num, report.references_moved
                    )
                });
                vec![crate::text::unshare::unshared(report.references_moved)]
            })
    });
}

/// Which sentence an `EditError` from `unshare_form` earns.
///
/// # ★★ A total match with a named catch-all, not a `_ =>` with a guess
///
/// Every variant the verb's own documentation names has an arm, in the order
/// the engine checks them, so this function and `EditSession::unshare_form`'s
/// guard ladder can be read side by side. `PageOutOfRange` and `PageTree` fall
/// to [`UnshareRefusal::Other`] deliberately rather than getting sentences of
/// their own: both mean the page vector moved under a queued command, and the
/// only honest operator-facing content for that is *nothing happened*.
///
/// ★ It is a free function rather than a `From` impl because a `From` would
/// invite this mapping to be reused for another verb's errors, and it is not
/// reusable: `FormNotOnPage` earns a sentence about re-selecting *because this
/// command's operand is derived from a selection*, which is not true of every
/// caller the engine has.
fn refusal_for(error: &pdfce_core::edit::EditError) -> UnshareRefusal {
    use pdfce_core::edit::EditError;
    match error {
        EditError::DocumentEncrypted => UnshareRefusal::Encrypted,
        EditError::CertificationForbidsChange { .. } => UnshareRefusal::Certified,
        EditError::ObjectCreationWouldExposeHiddenObjects { .. } => {
            UnshareRefusal::WouldExposeHiddenObjects
        }
        EditError::FormNestedInAnotherForm { .. } => UnshareRefusal::Nested,
        EditError::FormNotOnPage { .. } => UnshareRefusal::NotOnPage,
        EditError::ObjectNumbersExhausted => UnshareRefusal::NumbersExhausted,
        _ => UnshareRefusal::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfce_core::edit::EditError;

    /// ★★★ **Every refusal the verb documents maps to its own sentence**, and
    /// none of them falls through to the catch-all.
    ///
    /// The failure this pins is the one that is invisible in a diff: somebody
    /// adds an arm above, mistypes a variant name, and the compiler is happy
    /// because `_ =>` catches it. The operator then meets *"pdfce could not do
    /// that"* where they should have met *"that drawing is drawn from inside
    /// another one — use Select the form first"*, which is the difference
    /// between a dead end and an instruction.
    #[test]
    fn each_documented_refusal_earns_its_own_sentence() {
        for (error, expected) in [
            (EditError::DocumentEncrypted, UnshareRefusal::Encrypted),
            (
                EditError::CertificationForbidsChange { permission: 2 },
                UnshareRefusal::Certified,
            ),
            (
                EditError::ObjectCreationWouldExposeHiddenObjects { count: 17 },
                UnshareRefusal::WouldExposeHiddenObjects,
            ),
            (
                EditError::FormNestedInAnotherForm { form: 7 },
                UnshareRefusal::Nested,
            ),
            (
                EditError::FormNotOnPage {
                    form: 7,
                    page_index: 3,
                },
                UnshareRefusal::NotOnPage,
            ),
            (
                EditError::ObjectNumbersExhausted,
                UnshareRefusal::NumbersExhausted,
            ),
        ] {
            assert_eq!(
                refusal_for(&error),
                expected,
                "{error} fell through to the wrong sentence"
            );
        }
    }

    /// ★ **An error the verb does not document still gets a sentence**, and it
    /// is the one that promises nothing changed.
    ///
    /// The catch-all is not a hole; it is the honest fallback. This pins that
    /// it resolves to `Other` rather than to whichever arm happens to be first,
    /// which is what a reordering accident would produce.
    #[test]
    fn an_undocumented_error_falls_to_the_honest_fallback() {
        assert_eq!(
            refusal_for(&EditError::PageOutOfRange { index: 9, count: 2 }),
            UnshareRefusal::Other
        );
    }
}
