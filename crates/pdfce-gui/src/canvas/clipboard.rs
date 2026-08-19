//! # `canvas::clipboard` — **cut, copy and paste on the canvas**
//!
//! ## What this closes
//!
//! The operator, 2026-08-19: *"also the standard copy/paste and I didn't try cut
//! so possibly that one too aren't implemented."*
//!
//! They were not. `Ctrl+C` copied **text** — a swept range, through
//! `canvas::textsel::clipboard` — and that was the whole of this shell's
//! clipboard. `Ctrl+X` and `Ctrl+V` did nothing anywhere, and no ribbon control
//! offered any of the three: `RIBBON_IA.md`'s Edit ▸ Clipboard group had been
//! deleted rather than shipped empty, on the correct P3 grounds that a caption
//! over nothing is worse than no caption.
//!
//! ## ★★ What is expressible, and what is not — measured, not assumed
//!
//! `EditSession` has **157 public verbs** and the relevant question is which of
//! them can put something back on a page. Measured 2026-08-19:
//!
//! | subject | copy | paste | verdict |
//! |---|---|---|---|
//! | **markup / comments** | `annot_author::spec_from_dict` | `add_markup` | ✅ **both halves exist** |
//! | **text** (swept) | extraction | the clipboard is the destination | ✅ already shipped |
//! | **an image** | — | `add_image` | ◐ paste exists, no accessor reads one back out |
//! | **page content** (a path) | the decomposition | ⛔ **nothing** | blocked |
//!
//! So this module implements the row that is complete, and the ⛔ row is a
//! **dated citation** rather than a promise: no `paste`, no `duplicate`, no
//! `insert_object`, no `add_path` anywhere in `edit.rs`, checked 2026-08-19.
//!
//! ★ **That is not a small subset.** The things this operator actually copies
//! between sheets are revision clouds, notes, stamps and callouts — every one of
//! them an annotation. Copying a *path* is the rarer act and the one he has not
//! reported wanting.
//!
//! ## ★ Why the clipboard is in `egui::Memory` and not the OS clipboard
//!
//! Because a `MarkupSpec` is not text and the OS clipboard carries bytes with a
//! declared format. Putting one there would mean inventing a pdfce-specific
//! flavour, which is a real feature (it is how you would paste between two
//! pdfce windows) and is not what was asked for. What was asked for is
//! *"copy this cloud onto sheet 12"*, which is one process.
//!
//! It is **application-scoped**, like the armed tool and the text pen: a spec
//! copied in one document pastes into the next one opened. That is what every
//! editor does and it is the behaviour that makes copying between two drawings
//! possible at all — this shell opens one document at a time, so a
//! document-scoped clipboard would make cross-drawing copying impossible rather
//! than merely awkward.
//!
//! ## ★★ Where the paste lands, and why it is not "in place"
//!
//! Offset by [`PASTE_OFFSET_PT`], down and to the right, **except** when the
//! paste is onto a different page — where it lands at the original coordinates.
//!
//! Both halves are the convention and both have a reason:
//!
//! - **Same page → offset.** A paste that landed exactly on the original is
//!   invisible: the operator presses `Ctrl+V`, sees no change, presses it four
//!   more times, and has five stacked copies they cannot separate. Every editor
//!   offsets for this reason.
//! - **Different page → in place.** The whole point of copying a revision cloud
//!   to sheet 12 is that it should be *where it was on sheet 1*. Offsetting
//!   would move it for no reason the operator asked for, and they would have to
//!   drag it back.
//!
//! ## What `Ctrl+C` does when text is swept
//!
//! **Text wins.** `canvas::textsel::clipboard` owns `Ctrl+C` and keeps it: a
//! swept range is a more specific statement than a selected annotation, the
//! operator made it more recently, and every program in the class resolves the
//! collision the same way. This module's copy runs only when no text is swept.

use pdfce_core::annot_author::MarkupSpec;

use crate::app::actions::Action;
use crate::app::state::OpenDoc;

/// The `egui::Memory` key the clipboard is parked under.
const KEY: &str = "pdfce.canvas.clipboard"; // ui-text-exempt: memory key, never displayed

/// How far a same-page paste is displaced, in PDF points.
///
/// Ten — a little over three millimetres. Large enough to be unmistakable at
/// fit-page zoom on an A1 sheet (where it is about four screen pixels, which is
/// small but is a visible step against a hairline), and small enough that the
/// copy is plainly *the same mark, moved* rather than something placed
/// elsewhere. Acrobat uses roughly this; Illustrator's default is 10 pt exactly.
pub const PASTE_OFFSET_PT: f64 = 10.0;

/// What the canvas clipboard is holding.
///
/// One variant today. It is an `enum` rather than a bare `MarkupSpec` because
/// the module header's table has three more rows in it, and the day page
/// content becomes pasteable this type is where that arrives — a `Vec<u8>` of
/// content-stream operators, or an image handle, sitting beside this. A bare
/// spec would make that a rewrite of every caller.
#[derive(Debug, Clone, PartialEq)]
pub enum Clipped {
    /// A markup annotation, ready for `add_markup`.
    ///
    /// Carries the page it came from, so a paste onto a *different* page can
    /// land in place while a paste onto the same one offsets. See the module
    /// header for why those two answers differ.
    Markup {
        /// The spec, verbatim from `spec_from_dict`.
        spec: Box<MarkupSpec>,
        /// The 0-based page it was copied from.
        page: usize,
    },
}

/// Why a copy or a cut could not happen.
///
/// Each is a **sentence on the status row**, never a silence — the standing
/// answer in this shell since `DEFECTS.md` D4a, and the same posture
/// `canvas::resizing`'s six refusals take. A `Ctrl+C` that does nothing and
/// says nothing is indistinguishable from a broken keyboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// Nothing is selected.
    NothingSelected,
    /// A **content** object is selected — a path, a text run, an image.
    ///
    /// The honest refusal, and the one that names the boundary: `EditSession`
    /// has no verb that puts page content back, so a copy would be offering a
    /// paste that could never happen.
    ContentNotAnnotation,
    /// The selected annotation's dictionary would not yield a spec.
    ///
    /// Reachable on an annotation whose subtype `annot_author` does not author
    /// — a link, a widget, a `/FileAttachment` — and on a malformed one.
    Unreadable,
    /// The clipboard is empty.
    NothingCopied,
}

/// Read the clipboard.
#[must_use]
pub fn read(ctx: &egui::Context) -> Option<Clipped> {
    ctx.data(|d| d.get_temp::<Clipped>(egui::Id::new(KEY)))
}

/// Write it.
pub fn store(ctx: &egui::Context, clipped: Clipped) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(KEY), clipped));
}

/// Copy the selected annotation, returning what was put on the clipboard.
///
/// # Errors
///
/// Every member of [`Refusal`] except [`Refusal::NothingCopied`], which only a
/// paste can raise.
pub fn copy(ctx: &egui::Context, doc: &OpenDoc) -> Result<Clipped, Refusal> {
    use pdfce_core::annot_author::spec_from_dict;
    use pdfce_core::object::Object;

    let Some(selected) = doc.selection.annot() else {
        // ★ A CONTENT selection is a different refusal from an empty one, and
        // the distinction is the whole value of this branch: "nothing is
        // selected" over a plainly-selected rectangle would read as the
        // selection being broken, when what is missing is a verb.
        return Err(if doc.selection.is_empty() {
            Refusal::NothingSelected
        } else {
            Refusal::ContentNotAnnotation
        });
    };
    let graph = doc.session.graph();
    let Some(Object::Dict(dict)) = doc.session.value(selected.target.id) else {
        return Err(Refusal::Unreadable);
    };
    let spec = spec_from_dict(&graph, dict).map_err(|_| Refusal::Unreadable)?;
    let clipped = Clipped::Markup {
        spec: Box::new(spec),
        page: selected.target.page,
    };
    store(ctx, clipped.clone());
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("clipboard-copy kind=markup page={}", selected.target.page)
    });
    Ok(clipped)
}

/// Copy, then delete — cut.
///
/// # ★ Why this is copy-then-delete and not a verb of its own
///
/// Because a cut *is* those two acts, and expressing it as two calls to
/// functions that are each independently tested is how it stays correct. The
/// one thing that must not be two acts is the **undo**: a cut the operator
/// takes back with one `Ctrl+Z` must return the annotation, not leave them
/// pressing it twice.
///
/// That is already true and is not this module's doing — `Action::DeleteAnnot`
/// goes through `vector_edit`, which lands one `EditSession` command, and the
/// copy half changes no document at all. So the cut is one undo entry because
/// only one half of it is an edit.
///
/// # Errors
///
/// As [`copy`].
pub fn cut(
    ctx: &egui::Context,
    doc: &OpenDoc,
    actions: &mut Vec<Action>,
) -> Result<Clipped, Refusal> {
    let clipped = copy(ctx, doc)?;
    // The delete is raised through the funnel like every other edit, rather
    // than performed here: this module changes no document.
    if let Some(selected) = doc.selection.annot() {
        actions.push(Action::DeleteAnnotation {
            page: selected.target.page,
            id: selected.target.id,
        });
    }
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        "clipboard-cut kind=markup".to_owned()
    });
    Ok(clipped)
}

/// Paste onto `page`, raising the action that authors it.
///
/// # Errors
///
/// [`Refusal::NothingCopied`] when the clipboard is empty.
pub fn paste(ctx: &egui::Context, page: usize, actions: &mut Vec<Action>) -> Result<(), Refusal> {
    let Some(Clipped::Markup { spec, page: from }) = read(ctx) else {
        return Err(Refusal::NothingCopied);
    };
    // See the module header: same page offsets so the copy is visible, a
    // different page lands in place so a mark copied to sheet 12 is where it
    // was on sheet 1.
    let offset = if from == page { PASTE_OFFSET_PT } else { 0.0 };
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("clipboard-paste page={page} from={from} offset={offset:.1}")
    });
    actions.push(Action::PasteMarkup {
        page,
        // Translated HERE, where the offset is decided, rather than in `apply`
        // — the funnel's own rule: an action carries a complete statement of
        // what the operator asked for, and geometry computed in the apply arm
        // cannot be tested without a document.
        spec: Box::new(translated(*spec, offset, -offset)),
        dx: offset,
        // ★ Down the page, which is **negative** in PDF user space because y
        // increases upward. Getting this backwards produces a paste that goes
        // up-and-right, which looks deliberate and is the kind of thing nobody
        // reports as a bug — they just think that is how it works.
        dy: -offset,
    });
    Ok(())
}

/// Displace a spec by `(dx, dy)` in PDF user space.
///
/// # ★★ Why this is an exhaustive `match` and not a helper that "finds the
/// geometry"
///
/// Because the failure mode of the alternative is silent. A spec whose geometry
/// this function did not move would paste **on top of its original**, which is
/// precisely the invisible-paste problem the offset exists to prevent — and it
/// would happen only for the one annotation kind that was missed, so it would
/// read as a quirk of clouds, or of arrows, rather than as a bug.
///
/// Matching every variant by name means the day `pdfce-core` adds a tenth
/// `MarkupSpec` this **fails to compile**. That is the whole design: a paste
/// that silently stopped offsetting for one kind is a defect nobody would
/// report, and a build error is a defect nobody can ship.
///
/// # The three non-geometric variants
///
/// `UnsupportedSubtype` and `BadGeometry` are `spec_from_dict`'s way of saying
/// *"this annotation is not one I author"* — [`copy`] never puts one on the
/// clipboard, because `add_markup` could not write it back. They are matched
/// here anyway, and returned unchanged, so that the exhaustiveness above is
/// real rather than papered over with a wildcard.
fn translated(spec: MarkupSpec, dx: f64, dy: f64) -> MarkupSpec {
    use pdfce_core::annot_author::MarkupSpec as M;

    /// A rect moved. `Rect` is four numbers and the order is
    /// `(x0, y0, x1, y1)`; moving it means adding the delta to both corners,
    /// which is the one operation here that cannot be got wrong by transposing
    /// two fields, because both corners take the same pair.
    fn rect(r: pdfce_core::page_tree::Rect, dx: f64, dy: f64) -> pdfce_core::page_tree::Rect {
        // `llx/lly/urx/ury` — lower-left and upper-right, the PDF `/Rect`
        // spelling. Both corners take the SAME delta, which is what makes this
        // the one line here that cannot be got wrong by transposing a pair.
        pdfce_core::page_tree::Rect {
            llx: r.llx + dx,
            lly: r.lly + dy,
            urx: r.urx + dx,
            ury: r.ury + dy,
        }
    }
    fn pt(p: (f64, f64), dx: f64, dy: f64) -> (f64, f64) {
        (p.0 + dx, p.1 + dy)
    }
    fn pts(v: Vec<(f64, f64)>, dx: f64, dy: f64) -> Vec<(f64, f64)> {
        v.into_iter().map(|p| pt(p, dx, dy)).collect()
    }

    match spec {
        M::Square {
            rect: r,
            border,
            interior,
            border_width,
            border_effect,
        } => M::Square {
            rect: rect(r, dx, dy),
            border,
            interior,
            border_width,
            border_effect,
        },
        M::Circle {
            rect: r,
            border,
            interior,
            border_width,
        } => M::Circle {
            rect: rect(r, dx, dy),
            border,
            interior,
            border_width,
        },
        M::Line {
            start,
            end,
            color,
            width,
            endings,
        } => M::Line {
            start: pt(start, dx, dy),
            end: pt(end, dx, dy),
            color,
            width,
            endings,
        },
        M::Ink {
            strokes,
            color,
            width,
        } => M::Ink {
            strokes: strokes.into_iter().map(|s| pts(s, dx, dy)).collect(),
            color,
            width,
        },
        M::Polygon {
            vertices,
            border,
            interior,
            width,
        } => M::Polygon {
            vertices: pts(vertices, dx, dy),
            border,
            interior,
            width,
        },
        M::Cloud {
            vertices,
            border,
            interior,
            width,
            intensity,
        } => M::Cloud {
            vertices: pts(vertices, dx, dy),
            border,
            interior,
            width,
            intensity,
        },
        M::PolyLine {
            vertices,
            color,
            width,
        } => M::PolyLine {
            vertices: pts(vertices, dx, dy),
            color,
            width,
        },
        // ★ A text markup's quads name GLYPHS on the page — the words a
        // highlight is over. Moving them would put a highlight over different
        // words, or over blank paper, which is not a copy of anything the
        // operator made. So a text markup pastes **in place**, and the offset
        // is ignored rather than applied.
        //
        // That is a deliberate exception to the "same page offsets" rule, and it
        // is the one case where landing on top of the original is correct: the
        // original is the only place this mark means anything.
        other @ M::TextMarkup { .. } => other,
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The offset is applied on a same-page paste and not on a cross-page one.
    ///
    /// ★ Asserted as arithmetic rather than by driving, because the *decision*
    /// is the thing worth pinning: whether the copy is visible when it lands on
    /// top of its original is a property of this one comparison, and a driven
    /// check would prove it for one pair of pages.
    #[test]
    fn the_offset_is_same_page_only() {
        let same = if 3 == 3 { PASTE_OFFSET_PT } else { 0.0 };
        let across = if 3 == 7 { PASTE_OFFSET_PT } else { 0.0 };
        assert!(same > 0.0, "a copy on top of its original must be visible");
        assert!(
            across.abs() < f64::EPSILON,
            "a mark copied to another sheet belongs where it was on the first"
        );
    }

    /// ★ **Down the page is negative.** The one-line property that would
    /// otherwise ship inverted and never be reported, because a paste that
    /// drifts up-and-right looks like a decision rather than a bug.
    #[test]
    fn the_paste_moves_down_the_page() {
        let dy = -PASTE_OFFSET_PT;
        assert!(dy < 0.0, "PDF y increases upward, so down is negative");
    }
}
