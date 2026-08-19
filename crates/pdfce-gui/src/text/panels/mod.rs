//! # `text::panels` — every string the dock's panels show
//!
//! One area of the catalog described in [`crate::text`]'s header. It covers
//! the panel bodies in [`crate::panels`]. The three document-structure
//! panels are in this file; Comments, Fonts, Objects and Properties each have
//! their own module, and Forms and Pages have their own areas one level up.
//!
//! | Module | Panel |
//! |---|---|
//! | `mod.rs` (this file) | the three **document-structure** panels — Bookmarks, Layers, Signatures — plus [`byte_size`], which two areas share |
//! | [`comments`] | the Comments panel — every annotation on the document, what each one is, and the five disclosures a row can carry |
//! | [`fonts`] | the Fonts panel's inventory report |
//! | [`objects`] | the Objects panel, and the wording of every [`crate::panels::objects::summary::ObjectSummary`] fact |
//! | [`properties`] | the Properties panel |
//!
//! ## Almost every sentence here is salvaged verbatim, and that is the point
//!
//! These strings came across from the old shell's `ui_text.rs` (7,912 lines,
//! 1,193 entries) **with their doc comments**, because the doc comment is
//! usually the record of a defect the wording was changed to fix. Three
//! examples, all of which are below:
//!
//! - [`signature_leaves_tail`] is worded as *under-protection* rather than
//!   as damage, because ISO 32000-1 §12.8.1 makes whole-file coverage a
//!   `should` — the document is conforming, and an operator told "invalid"
//!   about a legal file has been misled just as surely as one told nothing.
//! - [`layers_session_only_note`] exists because a panel of tickboxes over a
//!   document is, by every other application's convention, an editor — and
//!   this one is not. Its doc comment carries the full wording history,
//!   including the two occasions the sentence was wrong.
//! - [`fonts::font_verdict_removable`] and its four siblings are **two
//!   words each**, and were full sentences until a screenshot of the running
//!   panel showed the row clipped at the dock's edge with the byte size cut
//!   to `59`.
//!
//! Rewriting any of those from scratch would re-derive a decision already
//! paid for, which is exactly what `SALVAGE.md`'s procedure forbids.
//!
//! ## The three panels in this file share a posture
//!
//! **Each says what it cannot tell you, first.** The Signatures panel opens
//! with the sentence that pdfce performs no cryptographic verification,
//! because a panel headed "Signatures" listing byte counts is the single
//! likeliest place in this application for an operator to take away more
//! than was said. The Layers panel opens by saying that a toggle changes what
//! you see and not the document, and that nothing it does is saved — which at
//! S3 also had to say the toggle was absent, and at S4 does not, because it
//! is back. The Bookmarks panel says when its own reader gave up.
//!
//! That ordering is not stylistic. A caveat below a list arrives after the
//! operator has already drawn a conclusion.
//!
//! ## Conventions, restated from [`crate::text`] because they bind here
//!
//! - **Sentence case, no trailing period on labels; full sentences with
//!   punctuation for prose.**
//! - **Name the thing and what the operator can do about it.**
//! - **Never state a capability the build does not have.** Several strings
//!   below were amended at salvage for exactly this reason, and each says so
//!   in its own doc comment rather than being quietly reworded.

/// The Comments panel — every annotation on the document, listed.
pub mod comments;
/// The ce-dimension properties section — the bottom tier of the style cascade
/// made reachable, with the tier each value came from named beside it.
///
/// Its own header carries the one rule this catalog must not break: **it never
/// builds a label**. A limit tolerance suppresses the nominal rather than
/// printing beside it, and a panel previewing the two by concatenation
/// disagrees with the bytes in the page.
pub mod dimension;
/// The Fonts panel's inventory report.
pub mod fonts;
/// The Objects panel, and the wording of every object fact.
pub mod objects;
/// The Properties panel.
pub mod properties;

// ---------------------------------------------------------------------------
// Shared
// ---------------------------------------------------------------------------

/// Format a byte count for a listing.
///
/// Base-1024 arithmetic with the colloquial "KB"/"MB" labels rather than the
/// pedantically correct "KiB"/"MiB": this is read by operators comparing the
/// figure against what their file manager tells them, and matching that is
/// worth more here than matching IEC. The exact count is always shown beside
/// it (see [`fonts::font_size_line`]), so nothing is lost to the rounding.
///
/// Deliberately different from the byte counts in the Signatures panel, which
/// are printed raw. Those exist to be compared against a file's own length —
/// an exactness task. These exist to be ranked across up to a couple of
/// hundred rows — a magnitude task. Different purpose, different format.
#[must_use]
pub fn byte_size(bytes: usize) -> String {
    #[allow(
        clippy::cast_precision_loss,
        reason = "a display rounding to one or two decimals; the exact count is printed alongside" // ui-text-exempt: clippy lint justification, never displayed
    )]
    let n = bytes as f64;
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", n / 1024.0)
    } else {
        format!("{:.2} MB", n / (1024.0 * 1024.0))
    }
}

/// Shown in any panel when no document is open.
///
/// **One sentence for every panel**, deliberately. A panel is never
/// blanked — a blank region is indistinguishable from a broken one — and a
/// bespoke "open a document to…" sentence per panel would be nine chances for one of
/// them to drift into a different voice, for no gain: at the moment nothing
/// is open, which panel the operator is looking at does not change the
/// answer.
#[must_use]
pub fn panel_no_document() -> &'static str {
    "Open a document to see this panel."
}

/// Shown when the dock holds a panel this build does not have.
///
/// Reachable exactly one way: a **saved layout** — or a named workspace —
/// naming a panel whose capability is not compiled into this binary
/// (`SHELL_FRAMEWORK.md` §5b). The dock's loader drops such entries and
/// reports them, so in practice this is the belt to that loader's braces.
///
/// It says what happened rather than apologising, because the operator's
/// next question is whether their layout is broken. It is not: the rest of
/// it loaded, and re-saving will forget this entry.
///
/// **Not a placeholder.** The no-placeholders rule forbids a control that
/// looks available and is not; it does not forbid explaining a pane the
/// operator's own saved layout asked for. A blank pane here would be
/// indistinguishable from a panel that had nothing to say.
#[must_use]
pub fn panel_unknown() -> &'static str {
    "This panel is not part of this build. Your saved layout asked for it; \
     everything else in the layout loaded normally."
}

// ---------------------------------------------------------------------------
// Signatures
// ---------------------------------------------------------------------------

/// The caveat, shown ABOVE the list on every visit.
///
/// A panel headed "Signatures" listing byte counts is the likeliest place in
/// this application for an operator to take away more than was said. The
/// sentence is first, not a tooltip, because the person who most needs it is
/// the one who will not hover.
#[must_use]
pub fn signatures_not_a_validity_check() -> &'static str {
    "pdfce does not check whether these signatures are valid — it cannot yet. What follows is what each signature COVERS: which parts of the file it would protect if it is valid."
}

/// No signature carries a byte range.
#[must_use]
pub fn signatures_none() -> &'static str {
    "This document has no signatures. (An empty signature field, waiting to be signed, is not one.)"
}

/// The file could not be measured.
///
/// Distinct from "no signatures": pdfce could not read the file's length
/// from disk, so it has nothing to compare a byte range against. Saying
/// "no signatures" here would be a claim about the document made from an
/// inability to look.
#[must_use]
pub fn signatures_file_unreadable() -> &'static str {
    "pdfce could not read this file's size from disk, so it cannot say what the signatures cover. Nothing here is a statement about the document."
}

/// A signature field with no name of its own.
#[must_use]
pub fn signature_unnamed() -> &'static str {
    "(unnamed signature)"
}

/// The good case.
#[must_use]
pub fn signature_covers_whole_file(covered: u64) -> String {
    format!("Covers the whole file — {covered} bytes, up to the last one.")
}

/// The case that matters: content exists beyond the signed range.
///
/// Worded as under-protection rather than as damage. ISO 32000-1 §12.8.1
/// makes whole-file coverage a `should`, so this document is CONFORMING —
/// and an operator told "invalid" about a legal file has been misled just as
/// surely as one told nothing.
#[must_use]
pub fn signature_leaves_tail(covered: u64, tail: u64) -> String {
    format!(
        "Covers {covered} bytes, but {tail} bytes come after the signed range — content this signature does not protect. That is allowed by the standard, and it means the signature guarantees less than its presence suggests."
    )
}

/// Overlapping or backwards ranges.
#[must_use]
pub fn signature_range_malformed() -> &'static str {
    "This signature's byte range is malformed — its parts overlap or run backwards, which the standard does not permit. The numbers below are what the file claims; another reader may compute something different, or refuse it."
}

/// A single range, which cannot verify.
#[must_use]
pub fn signature_single_range() -> &'static str {
    "This signature declares one continuous range, so it includes its own signature value in what it signs. A signature in that shape cannot verify anywhere."
}

/// The line naming which state of the file the coverage numbers describe.
///
/// **New at salvage, and not decoration.** The old panel's doc comment
/// carried this fact and the panel never said it out loud:
///
/// > Unsaved edits are not counted, and cannot be: they are not in the file
/// > yet. The panel says which state it is describing rather than leaving an
/// > operator to assume.
///
/// The second sentence was a promise the code did not keep — the panel
/// stated the numbers and left the reader to work out that they are about
/// bytes on disk. `/ByteRange` is a claim about bytes, so it can only be
/// checked against bytes, and the length used is the file **on disk right
/// now**. Once this shell can edit, "does the signature cover the file as it
/// currently exists" and "does it cover what I am looking at" are different
/// questions with different answers, and only one of them is being answered.
#[must_use]
pub fn signatures_measured_on_disk() -> &'static str {
    "Measured against the file as it is on disk right now. Any edits you have not saved are not part of these numbers."
}

// ---------------------------------------------------------------------------
// Layers
// ---------------------------------------------------------------------------

/// Shown when the document declares no optional content at all.
///
/// Distinct from "no layers I could read": most PDFs simply have none, and
/// saying so plainly stops an operator hunting for a panel fault.
#[must_use]
pub fn layers_none() -> &'static str {
    "This document has no layers."
}

/// Count above the list.
#[must_use]
pub fn layers_count(total: usize) -> String {
    if total == 1 {
        "1 layer.".to_owned()
    } else {
        format!("{total} layers.")
    }
}

/// The disclosure above the list, always shown.
///
/// ## ★ Wording history, because this string has been wrong twice
///
/// It has now had three lives, and the record matters more than any one of
/// them: **nothing compiles a doc comment against the behaviour it
/// describes**, so the only defence is that the sentence and the control
/// change in the same commit, and that the reason is written down here when
/// it does.
///
/// | When | What it said | Was it true? |
/// |---|---|---|
/// | old shell, pre-checkbox | *"Switching a layer changes what you see, not the document. Nothing here is saved."* | **No** — for three commits it described a checkbox the panel did not yet have, and the error was found by a person reading the file. |
/// | old shell, post-checkbox | the same sentence | yes |
/// | this build, S3 | *"…Switching a layer on or off is not available in this build…"* | yes — the panel genuinely had no control |
/// | this build, S4 (**now**) | the sentence below | yes — the control is back |
///
/// ## ★ What changed at S4, and when
///
/// S4 completed the third and last of the preconditions
/// `crate::panels::layers`' header tracks. `crate::app::actions::Action`
/// gained `SetLayerVisible`, `ResetLayers` and `ToggleAnnotations`, each
/// implemented in `PdfceApp::apply`; the render worker's `RenderKey` had
/// already gained `layers_generation`, and `crate::app::state::OpenDoc` had
/// already gained the override and its mutators. So the panel has a control
/// again, and the S3 clause — *"is not available in this build"* — became
/// false the moment it did. It is removed **in the same commit that added
/// the control**, which is the whole of the discipline this table exists to
/// record.
///
/// ## Why the first sentence is the salvaged one, verbatim
///
/// *"Switching a layer changes what you see, not the document"* is the thing
/// an operator most needs to know and the hardest for them to discover: a
/// panel of tickboxes over a document is, by every other application's
/// convention, an editor. Restating it in fresh words would re-derive a
/// decision already paid for (`SALVAGE.md`), so it comes back exactly as it
/// was.
///
/// ## Why a clause was ADDED, and what it is for
///
/// The old sentence's second half — *"Nothing here is saved"* — is true and
/// incomplete in a way that reads as a pdfce limitation. It is not one:
/// §8.11.2.1 puts a group's live ON/OFF state **outside the document
/// entirely**, so there is nowhere in the file for a save to put it. An
/// operator who reads "not saved" as "pdfce cannot save this yet" will wait
/// for a version that can, and no version can. So the sentence now names the
/// consequence they will actually meet — the states come back on reopen —
/// and attributes it to the format rather than to the build.
///
/// That is also why it does **not** promise the states survive anywhere
/// else: they do not survive a reopen, a second window, or an export.
#[must_use]
pub fn layers_session_only_note() -> &'static str {
    "Switching a layer changes what you see, not the document. Nothing here is saved — a layer's on or off state lives outside the file, so this document opens with its own settings again next time."
}

/// How many layers currently differ from the document's own configuration.
///
/// Salvaged verbatim from the old shell. Shown beside the Reset control, and
/// only when the number is non-zero, so the pair reads as one statement:
/// *this many things differ, and here is the way back*.
///
/// **"Differ from the document", not "you changed"**, because the panel
/// computes it by comparing the effective hidden set against
/// `pdfce_core::annot::optional_content_default_off` rather than by counting
/// clicks. A layer switched off and on again is back to agreeing with the
/// document and is not counted — which is the answer the operator would give
/// if asked, and the one a click-counter gets wrong.
#[must_use]
pub fn layers_overridden(n: usize) -> String {
    if n == 1 {
        "1 layer differs from the document.".to_owned()
    } else {
        format!("{n} layers differ from the document.")
    }
}

/// Label on the control that drops the operator's layer changes.
#[must_use]
pub fn layers_reset_label() -> &'static str {
    "Reset"
}

/// Tooltip on that control.
///
/// Says what it returns **to**, because "reset" in a layers panel could
/// equally be read as "turn everything on" — and those are different acts on
/// a document that declares a "Confidential" watermark off by default.
/// Revealing such a layer is a disclosure event; returning to the document's
/// own configuration is the opposite of one.
///
/// The second sentence exists to make the difference concrete rather than
/// leaving it in the word "specifies".
#[must_use]
pub fn layers_reset_tooltip() -> &'static str {
    "Go back to the layer states the document itself specifies. This shows hidden layers as hidden again."
}

/// Tooltip on a layer's visibility control.
///
/// Salvaged verbatim. Repeats the boundary that
/// [`layers_session_only_note`] states above the list, deliberately: the note
/// is read once when the panel opens and the tooltip is read at the moment of
/// the click, which is when the question *"am I editing this file?"* is
/// actually being asked.
#[must_use]
pub fn layer_toggle_tooltip() -> &'static str {
    "Show or hide this layer on screen. The document is not changed."
}

/// Tooltip on a layer whose state the operator has changed.
///
/// Names the document's own state, so the operator can always see what they
/// are diverging FROM without resetting to find out. Salvaged verbatim.
///
/// Both arms are needed and they are not symmetric in consequence: *"You have
/// shown this layer. The document hides it."* is the one that matters, since
/// a layer the document hides may be hidden for a reason.
#[must_use]
pub fn layer_overridden_tooltip(document_wanted_visible: bool) -> &'static str {
    if document_wanted_visible {
        "You have hidden this layer. The document shows it."
    } else {
        "You have shown this layer. The document hides it."
    }
}

/// Some layers' states are managed automatically (§8.11.4.4).
///
/// Says what the list IS rather than apologising for what it is not: the
/// states shown are the document's opening states, and for these layers the
/// page may legitimately disagree at the current zoom.
#[must_use]
pub fn layers_auto_managed(n: usize) -> String {
    if n == 1 {
        "1 layer switches itself on or off as you zoom. The state shown here is the one the document opens in.".to_owned()
    } else {
        format!(
            "{n} layers switch themselves on or off as you zoom. The states shown here are the ones the document opens in."
        )
    }
}

/// Tooltip on a layer whose `/Intent` excludes viewing (§8.11.2.3).
///
/// Explains why a layer the document lists as off is shown anyway —
/// otherwise the only available reading is "pdfce got it wrong".
///
/// ## ★ A second sentence was added at S4, and it is not a stylistic one
///
/// The first sentence alone became **actively misleading** the moment the
/// visibility control returned, because it invites the reading *"so there is
/// no point switching this row"*. The opposite is true, and it is a property
/// of the engine rather than of this panel:
///
/// `pdfce_render`'s interpreter resolves a group's state from
/// `oc_off_set()`, and when an operator override is in force that function
/// returns **the override verbatim** — no `/Intent` filtering, no `/AS`
/// usage application (`interpret.rs`, and `annot.rs` for annotation `/OC`).
/// Intent filtering happens only inside
/// `pdfce_core::annot::optional_content_default_off`, which is what builds
/// the *document's* answer. So:
///
/// | state | is a design-intent group's `/OFF` membership honoured? |
/// |---|---|
/// | no override (the document's own configuration) | **no** — §8.11.2.3 filters it out, and the group draws |
/// | any override in force | **yes, for every group in the set**, this one included |
///
/// That asymmetry is the engine's documented replace-not-merge contract
/// (core API trap T-12.9) doing exactly what it says. It is disclosed rather
/// than papered over, per rule 4: pdfce inferred something and the inference
/// changes the page.
#[must_use]
pub fn layer_design_intent_tooltip() -> &'static str {
    "This layer is marked for design use, not viewing, so the document's own on or off setting for it does not affect what is drawn. Switching it here does: your choice replaces the document's whole layer configuration for as long as this document is open."
}

/// Placeholder for a layer whose `/Name` is absent.
///
/// `/Name` is Required (Table 98), so its absence is a real malformation.
/// The placeholder says so rather than inventing "Layer 3", which would
/// disguise a defect as data from the file.
#[must_use]
pub fn layer_unnamed() -> &'static str {
    "(no name in the file)"
}

/// Text marker for a layer drawn by default. TEXT, never colour alone.
#[must_use]
pub fn layer_visible_marker() -> &'static str {
    "shown"
}

/// Text marker for a layer hidden by default.
#[must_use]
pub fn layer_hidden_marker() -> &'static str {
    "hidden"
}

/// Tooltip on a locked layer.
///
/// States what the lock actually is: the specification's own table blesses
/// JavaScript and `/AS` bypass, so calling it "cannot be changed" would
/// overstate it.
///
/// At S4 this became the tooltip on a **disabled** control rather than on a
/// bare row, which is why `crate::panels::layers` attaches it with
/// `on_disabled_hover_text` as well as `on_hover_text` — egui does not show
/// the ordinary hover text of a disabled widget, and this is the one row
/// whose explanation is the whole reason it looks broken.
#[must_use]
pub fn layer_locked_tooltip() -> &'static str {
    "The document marks this layer locked, so a viewer should not offer to switch it. It is an interface lock, not a guarantee — the document's own scripts can still change it."
}

/// Tooltip on a layer that content references but the default configuration
/// never registered.
#[must_use]
pub fn layer_unregistered_tooltip() -> &'static str {
    "Page content uses this layer, but the document never listed it in its layer configuration. Some readers will not show it in their own layer panel at all."
}

/// Tooltip on a layer in a radio-button group.
///
/// Table 101's `/RBGroups` are "radio button" groups: at most one member
/// visible at a time.
///
/// **The wording did not change at S4, and that is worth recording**: it was
/// already written as a plain statement of what a switch does, so restoring
/// the control turned a description of the document into a description of the
/// panel without a word moving. The fact was worth knowing either way — a CAD
/// drawing with two mutually exclusive title blocks is a different document
/// from one with two independent ones.
///
/// Note what it does **not** say: that switching this layer *off* switches a
/// sibling on. "At most one" permits none, and choosing a replacement would
/// be pdfce deciding which alternate the operator meant.
#[must_use]
pub fn layer_radio_tooltip() -> &'static str {
    "One of a group where switching this layer on switches the others off."
}

/// Tooltip on a radio-group member whose group also contains a locked layer.
///
/// ## ★ This is pdfce's answer to a question the standard leaves open
///
/// `pdfce_core::layers`' own module docs name it `DA-A8` and hand it here
/// verbatim: *"a locked group's state 'cannot be changed through the user
/// interface', while a sibling being turned ON means 'all others **shall** be
/// turned OFF'. Reported, not resolved — resolving it is the toggling
/// surface's decision to make and to disclose."*
///
/// **pdfce lets the lock win.** Turning on a radio member leaves a locked
/// sibling exactly as it was, so the panel can end up showing two members of
/// a mutually exclusive group at once.
///
/// The reasoning, since the choice is not obvious and the losing option is
/// respectable: both rules are addressed to the *user interface*, so the
/// tie-break has to be about which failure an operator can see and act on.
/// Turning off a locked layer as a side effect of clicking a **different**
/// row is a lock bypass through a side door — invisible at the moment it
/// happens, and it is exactly the "Confidential watermark quietly switched
/// off" shape that `/Locked` exists to prevent. Two title blocks painted over
/// each other is wrong too, but it is wrong *on the screen*, where the
/// operator is already looking. Between an invisible violation and a visible
/// one, take the visible one — and then say so, which is what this string is.
#[must_use]
pub fn layer_radio_locked_sibling_tooltip() -> &'static str {
    "Another layer in this group is locked, so pdfce will not switch it off for you. Switching this layer on can leave two of the group showing at once, which the document says should not happen."
}

// ---------------------------------------------------------------------------
// Bookmarks
// ---------------------------------------------------------------------------

/// Summary line above the tree.
#[must_use]
pub fn bookmarks_count(total: usize) -> String {
    if total == 1 {
        "1 bookmark.".to_owned()
    } else {
        format!("{total} bookmarks.")
    }
}

/// Shown when the document has an outline but no items pdfce could read.
#[must_use]
pub fn bookmarks_empty() -> &'static str {
    "This document has no bookmarks."
}

/// Disclosure when pdfce's own reader had to give up part-way.
///
/// A truncated tree looks exactly like a short one from the outside, so
/// silence here would let an operator conclude the document simply has few
/// bookmarks. Stated as what it is: pdfce stopped, the document did not end.
#[must_use]
pub fn bookmarks_truncated() -> &'static str {
    "pdfce stopped reading this outline early — it loops back on itself or is deeper than pdfce follows. Some bookmarks are missing from this list."
}

/// An outline item with no title of its own.
///
/// Its row still has to exist: a bookmark's children hang off it, and
/// omitting an untitled parent would show them at the wrong depth, silently
/// misrepresenting the document's structure.
#[must_use]
pub fn bookmark_untitled() -> &'static str {
    "(untitled)"
}

/// Tooltip on a bookmark row, naming where it goes.
///
/// The destination page is stated rather than left to be discovered by
/// clicking: an operator scanning a long outline for "where is the parts
/// list" should not have to jump to find out.
#[must_use]
pub fn bookmark_row_tooltip(page_number: usize) -> String {
    format!("Go to page {page_number}.")
}

/// A heading bookmark: no destination, by design.
#[must_use]
pub fn bookmark_row_heading_tooltip() -> &'static str {
    "A heading. It groups the bookmarks beneath it and does not point at a page of its own."
}

/// Tooltip on a bookmark that points nowhere pdfce can resolve.
///
/// Distinct from a bookmark with no destination at all, which is a heading
/// and perfectly normal. This one MEANT to point somewhere and pdfce could
/// not work out where — the operator should know the difference before
/// concluding the document is broken.
#[must_use]
pub fn bookmark_row_unresolved_tooltip() -> &'static str {
    "This bookmark points somewhere pdfce could not resolve — it may use a destination form pdfce does not read yet, or name a page that is not in this document."
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The three "cannot tell you" openers are genuinely different
    /// sentences.**
    ///
    /// Each of the three structure panels leads with a limitation, and the
    /// value of doing so is entirely in the limitation being specific. Three
    /// near-identical hedges would satisfy the convention and teach an
    /// operator to skip the first line of every panel, which is worse than
    /// having none.
    #[test]
    fn each_structure_panel_leads_with_its_own_limitation() {
        let openers = [
            signatures_not_a_validity_check(),
            layers_session_only_note(),
            bookmarks_truncated(),
        ];
        for (i, a) in openers.iter().enumerate() {
            for b in openers.iter().skip(i + 1) {
                assert_ne!(a, b);
            }
            assert!(a.len() > 40, "an opener too short to be specific: {a}");
        }
    }

    /// **The Layers note says a toggle changes the VIEW, and not the
    /// document.**
    ///
    /// This test replaces `the_layers_note_states_that_switching_is_unavailable`,
    /// which asserted the S3 truth — that the panel had no visibility control
    /// — by pinning the words "not available". S4 gave the panel its control
    /// back (`crate::app::actions::Action::SetLayerVisible`), so that clause
    /// became a lie and came out **in the same commit as the checkbox**.
    ///
    /// The test is rewritten rather than deleted because the *thing it
    /// guards* did not go away, it inverted. Two directions now:
    ///
    /// 1. **The claim that must be present.** A panel of tickboxes over a
    ///    document reads as an editor. "not the document" is the clause that
    ///    says it is not, and it is the single most load-bearing phrase in
    ///    this panel.
    /// 2. **The claim that must be absent.** If a later change reverts to the
    ///    S3 wording *without* removing the control, the panel is back to
    ///    describing a program that does not exist — the failure the old
    ///    shell's module header records happening twice, to two different doc
    ///    comments, in this exact file. A control that says of itself that it
    ///    is unavailable is not a copy-edit defect; it is the operator
    ///    concluding the application is broken.
    ///
    /// Asserted on clauses rather than on the whole string: a copy edit
    /// should be free, and a capability claim should not.
    #[test]
    fn the_layers_note_says_a_toggle_changes_the_view_and_not_the_document() {
        let note = layers_session_only_note();
        assert!(
            note.contains("not the document"),
            "the panel now HAS a visibility control, so the note's whole job is \
             to say that using it does not edit the file: {note}"
        );
        assert!(
            note.contains("Nothing here is saved"),
            "an operator who ticks a layer and closes the document must have \
             been told the tick does not travel with the file: {note}"
        );
        assert!(
            !note.contains("not available"),
            "the S3 clause is back and the control is here — the panel is now \
             denying a capability it ships: {note}"
        );
    }

    /// **Reset says what it returns TO, not merely that it resets.**
    ///
    /// `Action::ResetLayers` restores *the document's own default*, which on
    /// a file that declares a "Confidential" watermark off by default is
    /// emphatically not "show everything". Those are two different acts and
    /// only one of them is a disclosure event, so the control that performs
    /// the safe one must not be readable as the other.
    ///
    /// The label alone cannot carry that — "Reset" in a layers panel is
    /// genuinely ambiguous — so the tooltip is where the distinction lives
    /// and this is what keeps it there.
    #[test]
    fn the_reset_control_names_what_it_returns_to() {
        let label = layers_reset_label();
        assert!(!label.trim().is_empty());
        assert!(
            !label.ends_with('.'),
            "a label is a name and takes no trailing period: {label}"
        );

        let tip = layers_reset_tooltip();
        assert!(
            tip.contains("the document"),
            "reset must name the state it returns to, or it reads as 'turn \
             everything on': {tip}"
        );
        assert!(
            tip.contains("hidden"),
            "the tooltip has to say that hidden layers go back to hidden — that \
             is the half an operator would otherwise get wrong: {tip}"
        );
    }

    /// **The overridden count is a count of layers, and it is singular at
    /// one.**
    ///
    /// Same reasoning as [`layers_count`]: cheap to get wrong, immediately
    /// visible, and it sits directly beside the Reset control where an
    /// operator is deciding whether to click.
    #[test]
    fn the_overridden_count_agrees_with_itself_about_number() {
        assert!(
            layers_overridden(1).starts_with("1 layer "),
            "{}",
            layers_overridden(1)
        );
        assert!(
            layers_overridden(3).starts_with("3 layers "),
            "{}",
            layers_overridden(3)
        );
        for n in [1_usize, 3] {
            assert!(
                layers_overridden(n).contains("the document"),
                "the count is only meaningful against what it differs FROM: {}",
                layers_overridden(n)
            );
        }
    }

    /// **The two override tooltips name the document's own state, and say
    /// opposite things.**
    ///
    /// The point of [`layer_overridden_tooltip`] is that an operator can see
    /// what they are diverging from without resetting to find out. If both
    /// arms read alike, the row that says "you are showing content this
    /// document hides" — the one with a disclosure consequence — is
    /// indistinguishable from its harmless twin.
    #[test]
    fn an_overridden_layer_says_which_way_the_document_asked() {
        let doc_shows = layer_overridden_tooltip(true);
        let doc_hides = layer_overridden_tooltip(false);
        assert_ne!(doc_shows, doc_hides);
        assert!(doc_shows.contains("hidden"), "{doc_shows}");
        assert!(doc_hides.contains("shown"), "{doc_hides}");
    }

    /// A bookmark's three destination states read as three different things.
    ///
    /// "Points at a page", "is a heading" and "pdfce could not follow it"
    /// are one good outcome, one normal outcome and one problem. Collapsing
    /// any two would send an operator looking for a fault in a document that
    /// has none, or reassure them about one that does.
    #[test]
    fn the_three_bookmark_states_are_distinguishable() {
        let go = bookmark_row_tooltip(7);
        let heading = bookmark_row_heading_tooltip();
        let unresolved = bookmark_row_unresolved_tooltip();
        assert!(go.contains('7'), "the destination page must be named: {go}");
        assert_ne!(go, heading);
        assert_ne!(heading, unresolved);
        assert_ne!(go, unresolved);
    }

    /// Counted lines say "1 layer", not "1 layers".
    ///
    /// Cheap to get wrong, immediately visible, and the reason both
    /// functions branch rather than appending an `s`.
    #[test]
    fn counted_lines_are_singular_at_one() {
        assert!(layers_count(1).starts_with("1 layer."));
        assert!(layers_count(2).starts_with("2 layers"));
        assert!(bookmarks_count(1).starts_with("1 bookmark."));
        assert!(bookmarks_count(0).starts_with("0 bookmarks"));
        assert!(layers_auto_managed(1).starts_with("1 layer switches"));
        assert!(layers_auto_managed(3).starts_with("3 layers switch"));
    }

    /// Byte sizes cross their unit boundaries where they should.
    ///
    /// Base 1024, and the boundary cases are where an off-by-one shows up as
    /// `1024 B` sitting above `1.0 KB` in a sorted list.
    #[test]
    fn byte_sizes_use_base_1024_and_switch_units_at_the_boundary() {
        assert_eq!(byte_size(0), "0 B");
        assert_eq!(byte_size(1023), "1023 B");
        assert_eq!(byte_size(1024), "1.0 KB");
        assert_eq!(byte_size(1024 * 1024 - 1), "1024.0 KB");
        assert_eq!(byte_size(1024 * 1024), "1.00 MB");
    }

    /// The two signature-coverage sentences state opposite facts and must
    /// not read alike.
    ///
    /// One is reassurance and one is a warning, and both are about a
    /// conforming file. An operator who cannot tell them apart at a glance
    /// gets no value from the panel at all.
    #[test]
    fn full_coverage_and_a_tail_read_as_different_answers() {
        let full = signature_covers_whole_file(4096);
        let tail = signature_leaves_tail(4096, 512);
        assert_ne!(full, tail);
        assert!(full.contains("4096"));
        assert!(tail.contains("4096") && tail.contains("512"));
        // The warning has to name the consequence, not just the numbers.
        assert!(tail.contains("does not protect"));
    }
}
