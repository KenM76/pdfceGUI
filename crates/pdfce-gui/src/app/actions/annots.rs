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
) {
    // ★★ The default options: nothing rides with the geometry except `/RD`.
    //
    // The discriminator the engine promoted from this shell's own CAD argument
    // is *"is the property a length in the space being transformed?"* An inset
    // is; a line weight is a drafting convention. That is why `/RD` scales by
    // default and `/BS /W` does not, and why the two opposite defaults are
    // consistent rather than arbitrary.
    //
    // ★ Built with the BUILDERS. `ResizeOptions` is `#[non_exhaustive]`, so the
    // struct-expression form — including `..Default::default()` — is a compile
    // error outside `pdfce-core`, and the fields being `pub` makes that look
    // like a mistake at this end. The engine flagged it after an integration
    // test in a third crate refused to compile.
    let opts = pdfce_core::edit::ResizeOptions::new()
        // ★★★ `uniform` decides this, and it is not the operator's toggle.
        //
        // For a UNIFORM scale, scaling the stroke is not a workaround — the
        // placement matrix scales it by exactly the factor asked for, so
        // carrying a foreign appearance IS the correct result and the engine
        // proceeds with no flag. For a non-uniform one it stays off, which is
        // the shipped default and the case the engine may refuse.
        //
        // The operator's Inkscape-style toggle is a separate control and is not
        // built yet; when it is, it ORs with this.
        .with_scale_stroke_width(uniform);
    super::apply::vector_edit(doc, "resize-annotation", 0, 1, |session| {
        session
            .resize_annotation(id, anchor, sx, sy, &opts)
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
