//! # `text::unshare` — every sentence "give this page its own copy" can say
//!
//! Seven refusals and one disclosure, for
//! [`crate::app::actions::xobject`] and [`crate::app::dispatch::format`]. The
//! sibling of [`crate::text::rotating`] and [`crate::text::resizing`], written
//! on 2026-08-28 when `EDITABLE_SURFACES.md` found `EditSession::unshare_form`
//! implemented in the engine and called by nothing in this shell, and
//! `pdfce-core` asked for it by name.
//!
//! ## ★★★ Why this feature needs the biggest refusal catalog on the canvas
//!
//! Because **the refusals are the feature's whole shape**, and because the
//! commonest one is not an error at all.
//!
//! `EditSession::unshare_form` takes `(page_index, form: ObjId)` and clones one
//! form XObject's stream so that this page — and only this page — names the
//! copy. Every other invocation site keeps naming the original and is left
//! byte-identical. That is a **structural** edit: it allocates an object, it
//! rewrites the page's `/Resources`, and it therefore runs the same guard
//! ladder every structural verb in the engine runs (encryption, certification,
//! `/Size` suppression) before it does anything at all.
//!
//! ⇒ So a control that says *"give this page its own copy"* can decline for
//! **seven distinct reasons**, of which exactly one — [`UnshareRefusal::Nested`]
//! — is a considered design position rather than a limit, and none of them is
//! visible on the page. The operator presses a button, the drawing looks
//! identical (it *must*: the copy is byte-identical to the original, which is
//! the point), and without a sentence the only difference between success and
//! every failure is a status row that says nothing either way.
//!
//! ★★ This is the project's founding defect shape with the volume turned up:
//! *a gesture that is made, is refused, and reports nothing.* `DEFECTS.md` D4a.
//! And it is worse here than for a drag, because a successful unshare also
//! looks like nothing happened — see [`unshared`], which is why the success
//! path owes a sentence too.
//!
//! ## ★★ The vocabulary, decided once
//!
//! | the file's word | the operator's word here | why |
//! |---|---|---|
//! | form XObject | **drawing** / **the shared drawing** | §8.10.1's own illustration is a CAD system's standard component; the operator calls their title block a drawing, not an XObject |
//! | invocation | **place it is drawn** | an invocation is a `Do` operator; a place is something they can point at |
//! | page | **sheet** *(only where the fan-out is the subject)* | a 36-sheet drawing set is "sheets" in every room this software is used in. Elsewhere "page", because that is what the page box in the status bar says |
//! | `ObjId` | **not named at all** | see [`unshared`]: an object number is evidence, and evidence goes to the trace |
//!
//! [`crate::text::rotating`]'s rule is inherited unchanged: **name the thing
//! the operator can see, never the thing pdfce models.** A refusal phrased in
//! the file format's vocabulary reads as an internal error, and an internal
//! error is a thing an operator reports rather than acts on.
//!
//! ## ★ What is deliberately NOT worded here
//!
//! **Nothing.** That is unusual in this directory and it is the point: every
//! `EditError` this verb can return has a variant below, including the three
//! that are unreachable on a well-formed file. [`crate::text::rotating`]'s
//! argument for keeping its two unreachable variants is the argument for all of
//! these — *"a routing bug with a sentence is a bug report; a routing bug
//! without one is a handle that does nothing"* — and this verb has more ways to
//! be routed wrongly than a rotation does, because its operand is derived
//! through two hops (a selected leaf, then that leaf's outermost enclosing
//! form) rather than being the thing that was clicked.

/// **Why this page did not get its own copy of the shared drawing.**
///
/// # ★★★ A `Copy` enum rather than the engine's own `Display`
///
/// [`crate::text::status::TextStyleRefusal`]'s reason, adopted unchanged and
/// for the third time: a `format!` of an `EditError` would route **diagnostic
/// prose into the UI**, which `tools/gates/check-ui-strings.sh`'s exclusion 3
/// names in as many words — *"this exclusion is not permission to route UI text
/// through an error type."* An enum keeps
/// [`crate::app::status::decline::Declined`] `Copy`, keeps its `line()`
/// returning `&'static str`, and keeps every operator-visible word in this
/// file under **R1**.
///
/// # ★★ The one variant that carries no number, and why that is deliberate
///
/// [`Self::WouldExposeHiddenObjects`] is raised by the engine with a `count` —
/// how many cross-reference entries the file's `/Size` is currently hiding —
/// and this enum drops it. Two reasons, either sufficient:
///
/// 1. Carrying it would make this type non-`Copy`-friendly in the sense that
///    matters: `Declined::line()` returns `&'static str`, and a counted
///    sentence needs a `String` and an allocation on a path that runs while a
///    status bar is being laid out.
/// 2. **The number is not actionable and is barely meaningful to the reader.**
///    "17 hidden cross-reference entries" tells an operator nothing they can
///    do. What they can act on is *this file is damaged in a way that makes it
///    unsafe to add anything to*, and that is what the sentence says. The count
///    goes to the trace, where evidence belongs — the same split
///    `canvas::textedit::report` makes for `followers_repositioned`.
///
/// # ★ Ordering
///
/// The variants are in **the order the engine checks them**, which is also the
/// order of decreasing "this is about the whole document" and increasing "this
/// is about what you just clicked". A reader comparing this enum against
/// `EditSession::unshare_form`'s body should be able to walk both top to bottom
/// together.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnshareRefusal {
    /// The document carries `/Encrypt` (§7.6).
    ///
    /// `EditError::DocumentEncrypted`, the engine's first guard. Reachable on
    /// an ordinary file — plenty of drawing sets ship with an owner password
    /// set for printing — and completely invisible on the canvas, which is why
    /// it is worded rather than traced.
    Encrypted,
    /// The document carries an enforced certification signature (§12.8.4,
    /// `/Perms /DocMDP`).
    ///
    /// `EditError::CertificationForbidsChange`. ★ The variant an operator has
    /// no way whatsoever to guess at: a signed drawing looks exactly like an
    /// unsigned one on the canvas, and the sentence is the only surface that
    /// says otherwise.
    Certified,
    /// The file's trailer `/Size` is suppressing cross-reference entries, and
    /// creating the copy would raise `/Size` and expose them (§7.5.5).
    ///
    /// `EditError::ObjectCreationWouldExposeHiddenObjects`. The engine's own
    /// account of why this is refused rather than performed: *"the exposed
    /// objects are ones the operator did not touch and may not even parse; the
    /// document is frequently loadable **only** because the filter is hiding
    /// them."*
    ///
    /// ★ The count is deliberately not carried — see the enum's docs.
    WouldExposeHiddenObjects,
    /// The drawing is reached on this page only from **inside another
    /// drawing**.
    ///
    /// `EditError::FormNestedInAnotherForm`. ★★★ **The one refusal here that is
    /// a decision rather than a limit**, and the only one with a real remedy
    /// the operator can reach from where they are standing.
    ///
    /// Re-binding a nested invocation means editing the **parent** form, which
    /// may itself be shared — so the act's blast radius would depend on the
    /// document's nesting structure. `pdfce-core`'s decision 076 states the
    /// principle and this shell agrees with it: *"a default whose semantics
    /// silently depend on the document's nesting structure is worse than one
    /// that always means the same thing."*
    ///
    /// ⇒ This shell should hit it **rarely**, because it always hands the verb
    /// `FormLeaf::containment[0]` — the outermost enclosing form, which is by
    /// construction invoked by the page rather than by another form. See
    /// `panels::objects::provider::ObjectModelProvider::containing_form_object`,
    /// whose whole doc comment is about that choice. If this sentence appears,
    /// either the decomposition disagrees with the page's own `/Resources`, or
    /// something has started passing `parent()`.
    Nested,
    /// No `/XObject` name in the page's resources resolves to that drawing.
    ///
    /// `EditError::FormNotOnPage`. Reachable through a stale operand: the
    /// selection is resolved when the command is dispatched, and an edit
    /// between that and the apply phase can have re-pointed the page. The
    /// sentence therefore sends the operator to select it again rather than
    /// implying the document is wrong.
    NotOnPage,
    /// The document has no unused object number left.
    ///
    /// `EditError::ObjectNumbersExhausted`. Effectively unreachable — it means
    /// a document at the 32-bit object-number ceiling — and kept for
    /// [`crate::text::rotating`]'s stated reason: the sentence's *existence* is
    /// a tripwire, and its cost is four lines.
    NumbersExhausted,
    /// Nothing that is selected on this page is drawn inside a shared drawing,
    /// so the verb has no operand.
    ///
    /// ★★ **Shell-side, raised before the engine is called**, and it is the
    /// only variant here that is not an `EditError`. It is the state the
    /// command's `enabled_when("selection.in_form")` greys the ribbon item for
    /// — and greying enforces nothing, because the context menu, a chord and a
    /// future script all reach the dispatcher without consulting it. This is
    /// what those routes get.
    ///
    /// ★ It is deliberately **not**
    /// [`crate::app::status::decline::Declined::InsideForm`], although that
    /// variant is one line away and is about the same fact. That sentence reads
    /// *"That object is inside a form — pdfce cannot edit inside one yet"*,
    /// which is the report for a verb that refused **because** the selection is
    /// in a form. This verb refuses because it is **not**. Reusing the sentence
    /// would state the exact inverse of what happened.
    NothingInAForm,
    /// Anything else the engine declined.
    ///
    /// ★ A catch-all with a **hand-written** sentence, not a rendered error.
    /// `TextStyleRefusal::Other` and `RotateRefusal::Other` set the precedent
    /// and the reasoning is unchanged: wording a decline is catalog work per
    /// refusal, and the honest fallback says *nothing changed* rather than
    /// guessing at a cause.
    ///
    /// It covers `EditError::PageOutOfRange` and `EditError::PageTree`, both of
    /// which mean the page vector moved under a queued command — a state whose
    /// only honest operator-facing content is "nothing happened, try again".
    Other,
}

impl UnshareRefusal {
    /// The sentence.
    ///
    /// # ★★★ Every one of them ends by saying the sharing is unchanged
    ///
    /// That clause is not padding, and it is the clause that took the longest
    /// to get right. The operator pressed this button **because they are about
    /// to edit something**, and the thing they need to know after a refusal is
    /// not "it failed" — it is ***do not now go and type into that title
    /// block***, because doing so will change thirty-six sheets.
    ///
    /// A refusal that says only "pdfce could not do that" leaves them believing
    /// the safe state might have been reached. Every sentence below therefore
    /// closes the loop explicitly: the page still shares the drawing.
    ///
    /// # ★★ Remedy first where there is one
    ///
    /// [`crate::text::resizing`]'s rule, inherited: the operator is looking at
    /// something that did not happen, and the useful half is *what to do now*.
    /// [`Self::Nested`] and [`Self::NothingInAForm`] both name a next act;
    /// the rest have none, and none is invented for them.
    #[must_use]
    pub const fn line(self) -> &'static str {
        match self {
            // ★ "Encrypted" is a word the operator will have met — it is what
            // the password dialog in every other reader calls it — and the
            // limit is placed on pdfce, not on the document, because the file
            // is not malformed and there is nothing in it to fix.
            Self::Encrypted => {
                "This document is encrypted, and pdfce cannot add anything to an encrypted file \
                 yet. This page still shares that drawing with every other page that uses it."
            }
            // ★★ "Signed", not "certified" — `RotateRefusal::Certified` made
            // the same call and the argument is the same: the operator's word
            // for what happened to the file is that somebody signed it. And it
            // says the limit is the DOCUMENT's, because an operator told only
            // "cannot" goes looking for a setting to change.
            Self::Certified => {
                "This document has been signed, and the signature does not allow a change of this \
                 kind. pdfce copied nothing, so this page still shares that drawing."
            }
            // ★★ It says the file is DAMAGED, in those words, because that is
            // the actionable fact and because the alternative reading — "pdfce
            // is being fussy" — invites somebody to go looking for an override.
            // There is none, and there should be none: the hidden objects are
            // ones nothing in this document points at and some of them may not
            // parse at all.
            Self::WouldExposeHiddenObjects => {
                "This file's index is holding back entries that are damaged or unreadable, and \
                 adding anything to the file would expose them. pdfce copied nothing, so this \
                 page still shares that drawing."
            }
            // ★★★ The remedy is the whole sentence. "Select the form" is the
            // command one row above this one in the same menu, so the operator
            // is told to do a thing they can see.
            //
            // ★ It names the CONSEQUENCE of the alternative rather than
            // forbidding it: an operator who genuinely wants every sheet to
            // change is doing nothing wrong, and this feature exists to make
            // that a choice instead of an accident.
            Self::Nested => {
                "That drawing is drawn from inside another one, so giving this page its own copy \
                 would mean copying the outer drawing too — and that one may be shared as well. \
                 Use Select the form first to pick the outer one, or edit in place and accept \
                 that every page using it changes."
            }
            // ★ It sends them to re-select rather than reporting a fault,
            // because the reachable cause is a stale operand — the page changed
            // between the click and the command draining — and "select it again
            // and press this again" is a complete instruction.
            Self::NotOnPage => {
                "That drawing is not on this page any more, so there was nothing to copy. Select \
                 something inside it again and try once more."
            }
            // ★ No remedy, because there is none short of rebuilding the file
            // in another tool. What it does say is the one thing that is true
            // and useful: nothing was changed.
            Self::NumbersExhausted => {
                "This file has no room left for another object, so pdfce could not make the copy. \
                 Nothing was changed, and this page still shares that drawing."
            }
            // ★★ It explains what the command is FOR in the same breath as
            // refusing, because the reachable route to this state is a chord or
            // a menu row on a selection that has nothing to do with forms — an
            // operator who has not yet learned what the command does. The
            // second clause is the instruction.
            Self::NothingInAForm => {
                "Nothing you have selected is drawn inside a shared drawing, so there is nothing \
                 to give this page a copy of. Click something inside the title block or border \
                 first, then use this."
            }
            // ★ No cause named, because none is known. It says the page is
            // exactly as it was, and — the clause every sentence here carries —
            // that the sharing is untouched.
            Self::Other => {
                "pdfce could not give this page its own copy, and it changed nothing. This page \
                 still shares that drawing with every other page that uses it."
            }
        }
    }
}

/// **Disclosure: this page now has its own copy, and here is what moved.**
///
/// # ★★★ Why a SUCCESS owes a sentence at all, which is unusual
///
/// Most disclosures in this crate exist because a consequence is invisible.
/// This one exists because **the whole act is invisible, by design**.
///
/// `EditSession::unshare_form` clones the form stream verbatim — the engine's
/// own comment says the copy *"carries the ORIGINAL's value verbatim, span and
/// all"*, so unsharing costs no duplicated bytes until the copy is actually
/// edited. The page therefore renders **pixel-for-pixel identically** before and
/// after. Nothing moves, nothing changes colour, nothing appears or disappears.
///
/// ⇒ Without a sentence, the operator's evidence that the command worked is
/// indistinguishable from their evidence that it did nothing — which is the
/// same state as an unworded refusal, arriving through the success path. R8b
/// rule 4 as narrowed by pdfce's decision 059 (*render normally, report
/// separately*) applies with unusual force: there is nothing to render.
///
/// # ★★ What the engine asked a shell to say, verbatim
///
/// [`pdfce_core::edit::UnshareFormReport`] is documented as naming the copy and
/// how many references moved *"so a shell can say what happened rather than
/// only that it worked"*. This is that sentence.
///
/// # ★★ What it does NOT say: the object numbers
///
/// `UnshareFormReport::original` and `::copy` are `ObjId`s, and neither reaches
/// the status row. That is the split `canvas::textedit::report` states as a
/// rule and this file follows: *a number about a content stream is evidence, not
/// a disclosure.* An operator cannot act on "object 47"; a driven check can, and
/// a regression then names itself. Both numbers go to the trace from
/// `app::actions::xobject`, where the object-clipboard and text-edit arms
/// already send theirs.
///
/// # The plural, and why it is a branch rather than a format string
///
/// `references_moved` is *"usually 1. Greater than 1 when the page invoked the
/// same form under several names"* — a real case on CAD output, where one title
/// block is drawn once per view. The two sentences are genuinely different
/// statements, not one sentence with a number in it:
///
/// - at 1, the operator needs to know the change is now local to this page;
/// - above 1, they additionally need to know that **all** the places this page
///   draws it moved together, because the alternative reading — "one of the
///   three title blocks on this sheet is now private and two are not" — would be
///   a genuinely alarming and genuinely wrong thing to infer, and it is exactly
///   what an operator who knows the page draws it three times will infer from
///   silence.
///
/// That plurality is the engine's decision, stated in the verb's own docs: *"the
/// unit of this operation is the PAGE"*. The sentence says so.
#[must_use]
pub fn unshared(references_moved: usize) -> String {
    if references_moved > 1 {
        // ★ The count is named because it is the whole point of this branch —
        // it is the number the operator would otherwise have to trust — and
        // because it is a count of things they can see on the sheet in front of
        // them, which is what separates a disclosure from evidence.
        format!(
            "This page now has its own copy of that drawing, and all {references_moved} places \
             this page draws it use the copy. Editing it from here changes this page only; every \
             other page still shares the original."
        )
    } else {
        "This page now has its own copy of that drawing. Editing it from here changes this page \
         only; every other page still shares the original."
            .to_owned()
    }
}

/// **Disclosure appended to a text edit that changed shared content: how to
/// avoid it next time.**
///
/// # ★★★ Why the shell adds a sentence to the engine's own list
///
/// `pdfce-core` already puts a `"SHARED CONTENT: …"` sentence into
/// `text_edit::EditReport::disclosures`, worded for direct display, and
/// `canvas::textedit::report`'s header rules — correctly — that **re-wording it
/// here would be a second account of one fact, free to drift**. Nothing below
/// re-words it. This is a second, *different* fact, and it is one the engine
/// cannot state: **pdfce-core does not know what this shell's commands are
/// called.**
///
/// The engine's sentence says *what happened* — the edit changed every place
/// this form is drawn, because the standard binds a form to no page and there is
/// exactly one stream holding those glyphs. Complete, and true. What it cannot
/// say is *what to do about it*, because the answer is the name of a control in
/// a program it has never seen.
///
/// ⇒ The precedent for appending is already in the same apply arm:
/// `crate::text::textedit::pinned_tail_disclosure` is a shell-authored sentence
/// pushed onto the engine's list, for the same shape of reason — the engine says
/// nothing about a pinned tail because from its side pinning is what was asked
/// for, and the operator still owes it.
///
/// # ★★★ The sequence is UNDO first, and getting that wrong would be a lie
///
/// This is the sentence's load-bearing clause and it is worth the paragraph.
///
/// The naive remedy — *"press Unshare now"* — **does not work, and would make
/// things worse.** The edit has already been written into the one shared stream
/// object; every page that draws it already shows the change. Unsharing at that
/// point copies the **already-edited** stream to this page and re-points this
/// page at it. The other thirty-five sheets keep the original object, which is
/// the one that was edited. The operator would end up with the change on every
/// sheet **and** a redundant private copy, and a sentence that told them to do
/// that would have caused the damage it was warning about.
///
/// The order that works is **undo, unshare, edit again**, and it is stated in
/// that order with no room to read it otherwise.
///
/// # ★★ Why it is worded as a future-tense offer, not a warning
///
/// Because at the moment it is read, the fan-out has already happened and may
/// well have been wanted — §8.10.1's whole purpose for the feature is that one
/// component appears on many sheets, and a drawing-office correction to a title
/// block is *supposed* to reach all of them. This shell must not imply the
/// operator has made a mistake. It names the option they did not know they had.
///
/// # ★ Why it is appended rather than replacing the engine's sentence
///
/// The engine's sentence carries `InvocationSet::describe()` — the actual
/// counts, "3 pages, 5 places" or whatever this document is — and that is the
/// fact that makes the disclosure *startling*, which is the property
/// `canvas::textedit::report` says it is meant to have. Dropping it to make room
/// for a remedy would trade the alarming half for the useful half. Both, in the
/// engine's order: what happened, then what to do.
#[must_use]
pub fn shared_content_remedy() -> String {
    "To change this page on its own instead, undo, then use Give this page its own copy, then \
     make the change again."
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★★ **Every refusal is a sentence, and none of them is empty.**
    ///
    /// The check that a variant added later cannot ship silent — the founding
    /// rule applied to the enum that exists to serve it. Copied deliberately
    /// from `crate::text::rotating::tests::every_refusal_is_a_sentence` rather
    /// than generalised: a shared helper would be one more thing to keep in
    /// step, and the assertion is three lines.
    #[test]
    fn every_refusal_is_a_sentence() {
        for why in [
            UnshareRefusal::Encrypted,
            UnshareRefusal::Certified,
            UnshareRefusal::WouldExposeHiddenObjects,
            UnshareRefusal::Nested,
            UnshareRefusal::NotOnPage,
            UnshareRefusal::NumbersExhausted,
            UnshareRefusal::NothingInAForm,
            UnshareRefusal::Other,
        ] {
            let line = why.line();
            assert!(!line.is_empty(), "{why:?} has no sentence");
            assert!(
                line.ends_with('.'),
                "{why:?} is not a sentence — the founding rule is that a refusal IS one"
            );
        }
    }

    /// ★★★ **Every refusal says the sharing is unchanged**, which is the one
    /// clause the operator acts on.
    ///
    /// The reachable failure this pins: somebody adds a variant, writes a
    /// perfectly good sentence explaining *why* pdfce declined, and leaves out
    /// the half that stops the operator going straight on to type into a title
    /// block that is still shared by thirty-six sheets. The refusal would read
    /// as complete and would omit the only part that prevents damage.
    ///
    /// ★ Asserted on a **word**, not on a phrase, because the wording of each
    /// sentence is deliberately different and pinning a phrase would either
    /// force seven identical endings or fail on the first rewrite. Every
    /// sentence must mention what is *shared* or that nothing *changed*; that
    /// is the property, and it is the weakest assertion that still catches the
    /// omission.
    #[test]
    fn every_refusal_says_the_state_is_unchanged() {
        for why in [
            UnshareRefusal::Encrypted,
            UnshareRefusal::Certified,
            UnshareRefusal::WouldExposeHiddenObjects,
            UnshareRefusal::Nested,
            UnshareRefusal::NotOnPage,
            UnshareRefusal::NumbersExhausted,
            UnshareRefusal::NothingInAForm,
            UnshareRefusal::Other,
        ] {
            let line = why.line().to_lowercase();
            assert!(
                line.contains("shares")
                    || line.contains("shared")
                    || line.contains("changed")
                    || line.contains("nothing to copy"),
                "{why:?} does not tell the operator that the sharing is untouched, which is the \
                 clause that stops them editing a title block they still share"
            );
        }
    }

    /// ★★ **The plural branch names the count and the singular does not.**
    ///
    /// Both halves matter. A build that dropped the count on the multi-name
    /// case would leave an operator who knows the sheet draws its title block
    /// three times to guess whether one of the three moved or all of them; a
    /// build that printed "1 place" on the ordinary case would put a number in
    /// front of every operator for no reason at all.
    #[test]
    fn the_disclosure_names_a_count_only_when_there_is_one_to_name() {
        let one = unshared(1);
        assert!(!one.contains('1'), "the ordinary case must carry no count");
        assert!(one.contains("this page only"));

        let three = unshared(3);
        assert!(three.contains('3'), "the multi-name case must say how many");
        assert!(three.contains("this page only"));
    }

    /// ★★★ **The shared-content remedy states undo BEFORE unshare.**
    ///
    /// The one assertion in this file that pins a *sequence* rather than a
    /// property, and it is pinned because getting it backwards produces a
    /// sentence that reads perfectly and causes the damage it warns about: at
    /// the moment this is read the edit is already in the shared stream, so
    /// unsharing first copies the edited version and leaves every other page
    /// changed as well. See [`shared_content_remedy`]'s docs for the full
    /// account.
    #[test]
    fn the_remedy_puts_undo_first() {
        let line = shared_content_remedy();
        let undo = line.find("undo").expect("the remedy names undo");
        let copy = line
            .find("own copy")
            .expect("the remedy names the unshare command");
        assert!(
            undo < copy,
            "the remedy must say undo FIRST — unsharing after the edit copies the edited stream \
             and leaves every other page changed too"
        );
    }
}
