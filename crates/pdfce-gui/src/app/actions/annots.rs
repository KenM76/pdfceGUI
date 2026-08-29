//! # `app::actions::annots` — the verbs that change an annotation
//!
//! Split out of [`super::apply`] under **R2** on 2026-08-18, when annotation
//! selection landed and took that file past 1,500 lines. The seam is the one
//! [`super::pages`] already draws next door: *what class of thing does this
//! verb act on?* — pages there, annotations here, page **content** in `apply`.
//!
//! ## Why it is worth its own file today rather than when it is bigger
//!
//! Because it is about to get bigger, and for a reason that is already
//! scheduled. `EditSession::set_markup_style` shipped on 2026-08-18 —
//! colour, interior, width, opacity and arrowheads on an existing annotation,
//! keeping its object id — and the Format contextual tab is the surface for
//! it. Every one of those becomes a verb in here.
//!
//! ★ And each of them will carry the same routing obligation `delete` does
//! not: a **ce dimension** is a `/Line` with `/IT /LineDimension`, it passes
//! every "markup pdfce can author" test, and restyling one through
//! `set_markup_style` regenerates it as a bare line with its label and witness
//! lines gone. `pdfce-core` refuses it by name and points at
//! `set_dimension_style`. `canvas::selection::annot::AnnotKind` carries the
//! distinction on the selected target precisely so that routing is a `match`
//! the compiler checks — see its header.
//!
//! ## What is NOT here
//!
//! **Placing** an annotation. `Action::CommitMarkup`, `CommitTextAnnot` and
//! the measure commits stay in `apply`, because their subject is the *gesture*
//! that authored them rather than the annotation afterwards. The line is the
//! same one `pages` draws: this file is what happens to a thing that already
//! exists.

use pdfce_core::object::ObjId;

use crate::app::state::OpenDoc;

/// **Remove one annotation from the document.**
///
/// Reached from `format.delete` and from the canvas's Delete key, both only
/// while an annotation is selected.
///
/// # Why it goes through `vector_edit` like everything else
///
/// So the undo entry, the epoch bump, the cache invalidation and the
/// disclosure happen the one way they happen for every other document change.
/// The closure returns the disclosure list, which is where the **collateral**
/// goes: the operator named one annotation and the engine may legitimately
/// have removed or altered more — a `/Popup` companion (§12.5.6.14 is a
/// `shall`), replies orphaned, group members promoted.
///
/// # `page` is for the message, not for the verb
///
/// `delete_annotation` finds the annotation by id wherever it lives, and it
/// has to: a reply may sit on a different page from the comment it replies to,
/// so a page-scoped delete would miss it.
///
/// # ★ This is not redaction
///
/// It removes an entry from `/Annots`. It does not touch page content, and an
/// incremental save leaves the previous revision in the file.
/// `docs/core-api/03-capabilities.md` §3.4 states that rule, and
/// [`crate::text::markup::deleted_collateral`] observes it in the wording it
/// chooses — never "removed".
pub(super) fn delete(doc: &mut OpenDoc, page: usize, id: ObjId) {
    super::apply::vector_edit(doc, "delete-annotation", page, 1, |session| {
        session.delete_annotation(id).map(|report| {
            crate::text::markup::deleted_collateral(
                report.popup_removed,
                report.parent_popup_cleared,
                report.replies_orphaned,
                report.group_members_promoted,
            )
            .into_iter()
            .collect()
        })
    });
    // The selection named an object that no longer exists. Cleared here rather
    // than left for the next frame to notice: an outline around a deleted
    // annotation promises that a second Delete would do something, and the
    // second Delete would refuse.
    doc.selection.clear_annot();
}

/// **Move one markup annotation by a page-space delta.**
///
/// Reached from `canvas::annotdrag` on the release of a drag, and from nothing
/// else.
///
/// # ★★★ The disclosure is about the half the canvas cannot show
///
/// A move writes `/Rect` *and* the absolute-coordinate geometry keys, and the
/// canvas renders from the appearance stream, so the operator sees the same
/// picture whether one half was written or both. There is therefore nothing to
/// disclose about the move having worked -- they can see that.
///
/// What they cannot see is the **pop-up left behind**. §12.5.6.14 makes a
/// pop-up a separate annotation with its own placement and leaves whether it
/// follows to the reader; `pdfce-core` reports the object number and says the
/// decision is the shell's. This shell does not draw pop-ups at all, so one
/// stranded across the sheet is invisible here and visible in Acrobat.
///
/// ⇒ ★★ **That is Rule 4's surviving half exactly**: an inference or a
/// consequence the operator cannot see still owes an off-canvas report. Render
/// normally; report separately. Both.
///
/// # ★ What is deliberately NOT disclosed
///
/// **`geometry_keys_moved` being empty**, which the engine warns about by name:
/// a Text note, a Stamp or a Link has no geometry key because its `/Rect` *is*
/// its geometry, so empty is a correct answer and reporting it would manufacture
/// an anomaly out of the commonest case.
///
/// **`rect_differences_untouched`**, for a different reason: `/RD` holds inset
/// distances rather than coordinates, translating them would deform the
/// annotation, and not translating them is therefore not a limitation to
/// confess but the only correct behaviour. A sentence about it would teach an
/// operator to worry about something that is right.
pub(super) fn move_annot(doc: &mut OpenDoc, id: ObjId, dx: f64, dy: f64) {
    super::apply::vector_edit(doc, "move-annotation", 0, 1, |session| {
        session.move_annotation(id, dx, dy).map(|outcome| {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    // `-applied`, per the convention `forms::import_data`
                    // records: the funnel writes its own bare-named line for
                    // the same edit and `.last()` would read that one.
                    "move-annotation-applied id={} dx={dx:.3} dy={dy:.3} keys={} popup={}",
                    id.num,
                    outcome.geometry_keys_moved.len(),
                    outcome.popup_left_behind.is_some()
                )
            });
            outcome
                .popup_left_behind
                .map(|_| vec![crate::text::markup::popup_left_behind()])
                .unwrap_or_default()
        })
    });
}

/// **Scale a markup annotation about an anchor.** `OPERATOR_REQUESTS.md` O51.
///
/// ★★★ The disclosure is the operator's own ruling, carried through. He asked
/// for Inkscape's toggles — *"default should be what it said, but there should
/// be an option that they do scale with resize"* — and the sentence that
/// belongs beside a default is the one that says the default fired.
///
/// ★★ **`stroke_width: None` is the case that owes a sentence**, which is the
/// engine's own instruction: *"an operator who scaled a square 3× and expected
/// a heavier border needs telling it stayed."* That is Rule 4's surviving half
/// — a line weight left alone is invisible on the canvas, because the shape
/// grew around it and nothing says the border did not.
///
/// ★ **`CarriedDistorted` is the other one**, and it is not a defect: neither
/// PDF nor SVG has a per-axis stroke width, so a non-uniform scale of an
/// appearance pdfce did not author produces an anisotropic border by
/// arithmetic. The engine refuses that case unless it is allowed; where it
/// proceeds, the operator is told.
pub(super) fn resize(
    doc: &mut OpenDoc,
    id: ObjId,
    anchor: (f64, f64),
    (sx, sy): (f64, f64),
    uniform: bool,
    modifiers: crate::canvas::scaling::Modifiers,
) {
    // ★★★ **THE OPERATOR'S SWITCHES, and they replaced a derivation.**
    //
    // Until 2026-08-28 this read `with_scale_stroke_width(uniform)` — the flag
    // taken from whether the drag was proportional rather than from anything
    // anybody asked for. That was a **workaround for a refusal**: with a
    // foreign appearance and a uniform scale the engine refuses unless either
    // the stroke scales or distortion is allowed, and forcing the first made
    // the common case work when no control existed.
    //
    // ⇒ It also made the operator's answer unreachable, on exactly the resizes
    // where they were most likely to have one. `OPERATOR_REQUESTS.md` **O51**
    // is a correction about precisely this shape of reasoning, so deriving the
    // flag from geometry after building the control would be making the same
    // mistake twice in one file.
    //
    // ★ What replaced the workaround is the worded decline below, not a
    // different guess.
    //
    // The discriminator behind the DEFAULTS is unchanged and is the engine's,
    // promoted from this shell's own CAD argument: *is the property a length in
    // the space being transformed?* An inset is; a line weight is a drafting
    // convention. `canvas::scaling` carries the whole account.
    let opts = modifiers.to_options();
    super::apply::vector_edit(doc, "resize-annotation", 0, 1, |session| {
        session
            .resize_annotation(id, anchor, sx, sy, &opts)
            .inspect_err(|error| {
                // ★★★ **The refusal is caught here and worded**, rather than
                // being left to `vector_edit`'s generic arm, which traces and
                // says nothing to the operator.
                //
                // A resize that silently did nothing is this project's founding
                // failure: the operator drags a grip, lets go, the shape snaps
                // back, and no surface anywhere says why. It is the same shape
                // as the annotation drag that was consumed and discarded, and
                // the same shape as the markup move that had no branch.
                //
                // ★★ Recorded from INSIDE the closure because the condition is
                // not knowable before the call — whether an appearance is
                // pdfce's own is a property of the file. `record_save_failure`
                // is called from the apply phase for the identical reason;
                // `record_flatten_certified` is not, because its refusal is a
                // query.
                //
                // ★ Only this one variant. Every other `EditError` keeps
                // today's trace-only behaviour, which is honest: wording a
                // decline is catalog work per refusal, and a `format!` of an
                // `EditError`'s `Display` would route diagnostic prose into the
                // UI — the thing `check-ui-strings`' exclusion 3 names in as
                // many words.
                if let pdfce_core::edit::EditError::ResizeAppearanceNotRebuildable {
                    uniform: was_uniform,
                    ..
                } = error
                {
                    crate::app::status::decline::record_resize_not_rebuildable(*was_uniform);
                }
            })
            .map(|outcome| {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed
                    format!(
                        "resize-annotation-applied id={} sx={sx:.4} sy={sy:.4} uniform={uniform} \
                         keys={} appearance={:?} stroke={}",
                        id.num,
                        outcome.geometry_keys_scaled.len(),
                        outcome.appearance,
                        outcome.stroke_width.is_some()
                    )
                });
                let mut notes = Vec::new();
                if outcome.stroke_width.is_none() {
                    notes.push(crate::text::markup::stroke_width_unchanged());
                }
                if matches!(
                    outcome.appearance,
                    pdfce_core::edit::ResizedAppearance::CarriedDistorted
                ) {
                    notes.push(crate::text::markup::appearance_distorted());
                }
                notes
            })
    });
}

/// **Write the note on an annotation that already exists** — `/Contents`, and
/// conditionally `/T` and `/M` — as one undoable command.
///
/// Reached from the Comments panel's editor and from nothing else.
///
/// # ★★★ The three keys are not written as a group, and that is the contract
///
/// `pdfce-core` leaves an **omitted** key untouched rather than clearing it,
/// and its reply to this shell called getting that wrong *"the easiest way to
/// get this wrong"*:
///
/// > An implementation writing all three keys unconditionally would silently
/// > strip the author and date on every correction, leaving a review comment
/// > from nobody, dated never, looking exactly like a note somebody else had
/// > mangled.
///
/// So `author` is `None` on two quite different occasions and both must send
/// nothing: the annotation already has a byline that is not ours to move, or
/// the operator has left their name blank in Settings ▸ Comments, which is a
/// supported choice and means *comment anonymously*. `crate::app::actions::apply`
/// resolves which; this function only has to not invent one.
///
/// # ★★ `/M` is always written, and it is a modification date
///
/// §12.5.6.4 Table 170 defines `/M` as the date the annotation was **modified**,
/// and this call modifies it — so leaving it alone would leave a comment whose
/// date describes an earlier version of its own text. `crate::app::clock` is
/// the only place this shell reads a wall clock and its header carries the
/// whole argument for UTC; `None` there means the system clock is before 1970,
/// and omitting `/M` beats writing a comment dated 1969.
///
/// # ★ The disclosure is about the words that are gone
///
/// A note that replaced another one leaves **no trace on the canvas**: the
/// shape is unchanged, and a sticky's words live in a pop-up window this shell
/// does not draw. `MarkupNoteChange::replaced` carries the previous text — the
/// text, not a count — precisely so the operator can be offered it back, which
/// is what `crate::text::markup::note_replaced` does.
pub(super) fn set_note(doc: &mut OpenDoc, id: ObjId, text: &str, author: Option<&str>) {
    // Builders, not a struct literal: `MarkupNote` is `#[non_exhaustive]`,
    // which is what keeps a future field a non-breaking addition for us.
    let mut note = pdfce_core::edit::MarkupNote::new(text);
    if let Some(author) = author.map(str::trim).filter(|a| !a.is_empty()) {
        note = note.by(author);
    }
    if let Some(stamp) = crate::app::clock::pdf_date_utc() {
        note = note.at(stamp);
    }
    super::apply::vector_edit(doc, "set-markup-note", 0, 1, |session| {
        session.set_markup_note(id, &note).map(|change| {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                //
                // ★ `-applied`, per the convention `forms::import_data`
                // records: the funnel writes its own bare-named line for the
                // same edit and `.last()` would read that one instead.
                //
                // `keys` is the field worth tracing rather than the text: it is
                // the engine's own answer to "what actually moved", and the
                // whole `/T`-preservation contract above is invisible from a
                // screenshot and from the saved page alike.
                format!(
                    "set-markup-note-applied id={} chars={} keys={} replaced={}",
                    id.num,
                    text.chars().count(),
                    change.keys_written.join("+"),
                    change.replaced.is_some()
                )
            });
            change
                .replaced
                .as_deref()
                .and_then(crate::text::markup::note_replaced)
                .into_iter()
                .collect()
        })
    });
}

/// **Remove an annotation's note entirely** — `/Contents`, `/T` and `/M` — as
/// one undoable command.
///
/// Reached from the Comments panel's *Remove note* control and from nothing
/// else.
///
/// # ★★ It is not a delete, and the disclosure says so because nothing else can
///
/// The markup stays on the page with its geometry untouched. A shape with a
/// note and the same shape without one are **the same picture**, so an operator
/// who pressed the wrong button has no way to see either what they did or what
/// it cost them. `crate::text::markup::note_removed` states both — the words
/// that went, and the fact that the shape did not.
///
/// # ★ Why a separate verb from writing an empty note
///
/// `pdfce-core`'s reason, adopted rather than re-derived: *"an empty comment is
/// a comment, and a reviewer deleting their remark is not the same as leaving a
/// blank one."* An empty `/Contents` beside a `/T` and an `/M` says somebody
/// wrote nothing; no `/Contents` at all says nobody wrote anything.
pub(super) fn clear_note(doc: &mut OpenDoc, id: ObjId) {
    super::apply::vector_edit(doc, "clear-markup-note", 0, 1, |session| {
        session.clear_markup_note(id).map(|change| {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!(
                    "clear-markup-note-applied id={} keys={} had_note={} had_author={}",
                    id.num,
                    change.keys_written.join("+"),
                    change.replaced.is_some(),
                    change.replaced_author.is_some()
                )
            });
            change
                .replaced
                .as_deref()
                .and_then(crate::text::markup::note_removed)
                .into_iter()
                .collect()
        })
    });
}
