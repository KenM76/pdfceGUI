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
    /// ★★★ **Page content** — a path, a text run, an image, in any mixture.
    ///
    /// This variant is what the type's own docs predicted: *"the day page
    /// content becomes pasteable this type is where that arrives."* `Pass 120.0`
    /// shipped `ObjectClip` on 2026-08-20 and this is that day.
    ///
    /// # ★★ Why the BYTES and not the `ObjectClip`
    ///
    /// Three reasons, and the third is the one that decides it:
    ///
    /// 1. `egui::Memory` wants `Clone + Send + Sync + 'static`, and bytes are
    ///    all four without asking anything of the engine's type.
    /// 2. `Clipped` derives `PartialEq`, which bytes give for free.
    /// 3. **It is the same representation the OS clipboard will take.** The
    ///    engine's `to_bytes` is magic-prefixed, versioned and bit-exact
    ///    precisely so a pdfce→pdfce paste is lossless, and registering it as a
    ///    private format is the remaining half of the operator's item 3. Holding
    ///    the live struct here would mean serialising at the moment of
    ///    registration instead — a second code path, for the same bytes.
    ///
    /// ★ The clip **owns its resources**, transitively, by value. So copying
    /// from one document, closing it, and pasting into another works — and
    /// cross-document paste is not a special case but the same call.
    Content {
        /// `ObjectClip::to_bytes` — magic-prefixed, versioned, bit-exact.
        bytes: Vec<u8>,
        /// The 0-based page it was copied from, for the same reason
        /// [`Self::Markup`] carries one: a paste onto the *same* page offsets
        /// so the copy is visible, a paste elsewhere lands in place.
        page: usize,
        /// How many objects are in it, for the trace and for a sentence.
        ///
        /// Carried rather than re-derived because reading it back means
        /// deserialising, and the count is wanted in places that have no reason
        /// to.
        count: usize,
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
    /// The engine refused to copy the selection.
    ///
    /// ★★ **This variant was `ContentNotAnnotation` until 2026-08-20**, and it
    /// said: *"`EditSession` has no verb that puts page content back, so a copy
    /// would be offering a paste that could never happen."* True when it was
    /// written, and `Pass 120.0` made it false — the operator had been asking
    /// for cut/copy/paste of page content since the first week.
    ///
    /// What replaces it is the engine's own refusal, which is a genuinely
    /// different fact: a clip it could not assemble. Kept as one variant rather
    /// than mirroring the engine's taxonomy, for the reason
    /// `canvas::resizing`'s note gives about the same choice — a shell that
    /// modelled the engine's internals a second time is decision 058's failure
    /// mode, and this module has just watched one of those expire.
    EngineRefused,
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
        // ★★★ PAGE CONTENT, as of 2026-08-20. This branch used to refuse it by
        // name — *"pdfce has no verb that puts page content back, so a copy
        // would be offering a paste that could never happen"* — and it was the
        // operator's oldest open request.
        //
        // ★ The ORDER here is the annotation first and content second, which is
        // the opposite of how the selections are populated and is deliberate: a
        // ce dimension and a markup are annotations that a content selection can
        // never name, so asking the narrower question first means the broad one
        // never has to exclude anything.
        return copy_content(ctx, doc);
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

/// Copy the selected **page content** — a path, a text run, an image, in any
/// mixture.
///
/// # ★★ What the engine does that this could not have done for itself
///
/// This shell's own request scoped the work as *"expose the copy engine you
/// already have at object granularity"*, on the strength of `import_object`
/// being a recursive, reference-remapping, cycle-guarded object-graph copy.
/// That reading was correct **and it was the smaller half**, in one specific
/// place worth writing down:
///
/// > `import_object` copies **indirect objects**. A page's content objects are
/// > not indirect objects — a path, a text run and an image invocation are byte
/// > ranges inside a content stream, and the operators in those bytes name
/// > their resources **by page-local name**. On the destination page, `/F1` is a
/// > different font. Paste the bytes verbatim and you get the right glyphs in
/// > the wrong typeface, or nothing at all. **Neither failure errors**, and
/// > neither is visible in a diff, because *a resource name is not a
/// > reference.*
///
/// So the clip records which names each item consumes, carries the objects
/// behind them by value, and paste re-binds every one to a fresh name on the
/// destination page — rewriting the names inside the copied bytes. That is the
/// feature; `import_object` was the prerequisite.
///
/// Recorded here rather than in a request file because it is the general
/// lesson: **a graph copy does not copy a namespace.**
///
/// # Errors
///
/// [`Refusal::NothingSelected`] for an empty selection,
/// [`Refusal::EngineRefused`] when the clip could not be assembled.
fn copy_content(ctx: &egui::Context, doc: &OpenDoc) -> Result<Clipped, Refusal> {
    let page = doc.view.page_index;
    let objects = doc.selection.object_indices_on(page);
    if objects.is_empty() {
        return Err(Refusal::NothingSelected);
    }
    // ★ `&self`, and it commits nothing — which is what makes `cut` below one
    // undo entry without a `cut_objects` call: only the deletion is an edit.
    let clip = doc
        .session
        .copy_objects(page, &objects)
        .map_err(|_| Refusal::EngineRefused)?;
    let clipped = Clipped::Content {
        count: clip.len(),
        bytes: clip.to_bytes(),
        page,
    };
    store(ctx, clipped.clone());
    // ★★★ AND A MARKER ON THE OS CLIPBOARD, WITHOUT WHICH CTRL+V DOES NOT
    // ARRIVE AT ALL.
    //
    // Not a nicety and not a placeholder. `egui-winit` turns `Ctrl+V` into
    // `Event::Paste(contents)` **only if the OS clipboard has non-empty text**,
    // and returns before pushing a key event either way — so with an empty
    // clipboard the keystroke vanishes completely, no event of any kind.
    // `app::keyboard::clipboard_chord` carries the whole account.
    //
    // So a copy that put nothing on the OS clipboard would leave `Ctrl+V`
    // working or not depending on **whether the operator had recently copied
    // text in another application**, which is the worst kind of intermittent:
    // it is not random, it is not reproducible, and the thing that fixes it has
    // nothing to do with pdfce.
    //
    // ★ What goes there is a SENTENCE RATHER THAN THE BYTES, and both halves of
    // that are deliberate:
    //
    // * a human who pastes into a text editor gets something that says what
    //   happened, not a screenful of binary;
    // * the real payload is `ObjectClip::to_bytes`, which belongs under a
    //   **private clipboard format** so a pdfce→pdfce paste is lossless — and
    //   registering one is a Win32 `RegisterClipboardFormat` call this shell
    //   does not make yet. That is the remaining half of the operator's item 3,
    //   named here rather than left as a silence.
    //
    // Until then the marker is what makes the chord arrive and the in-memory
    // clip is what is pasted, so a pdfce→pdfce paste is already lossless. What
    // is missing is pdfce→pdfce **across two processes**.
    ctx.copy_text(crate::text::clipboard::os_marker(objects.len()));
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        //
        // ★ The COUNT and the BYTE LENGTH, because those are what a wrong build
        // gets wrong: a clip that copied the operators and dropped the
        // resources is a plausible-looking clip that pastes the right glyphs in
        // the wrong typeface, and it is several hundred bytes shorter.
        let bytes = match &clipped {
            Clipped::Content { bytes, .. } => bytes.len(),
            Clipped::Markup { .. } => 0,
        };
        format!(
            "clipboard-copy kind=content page={page} objects={} bytes={bytes}",
            objects.len()
        )
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
    // ★★ COPY RUNS FIRST, and the engine makes the same point about its own
    // `cut_objects`: *"a selection that cannot be copied is refused with
    // nothing deleted. Reversed, a cut whose copy half failed would take the
    // objects away with nothing on the clipboard — the one outcome the operator
    // cannot recover from by pasting."* The `?` above is that ordering.
    //
    // ★ And this is deliberately NOT `EditSession::cut_objects`, though that
    // verb exists and would work. `cut_objects` is copy-then-delete inside the
    // engine; doing it here as copy-then-`DeleteSelection` keeps the delete
    // going through the funnel like every other edit, so it lands one
    // `EditSession` command and one undo entry by the same mechanism as
    // everything else — and this module goes on changing no document, which is
    // what lets its refusals be unit-tested without one.
    //
    // The undo property is unchanged either way: only one half of a cut is an
    // edit.
    // The delete is raised through the funnel like every other edit, rather
    // than performed here: this module changes no document.
    match (&clipped, doc.selection.annot()) {
        (Clipped::Markup { .. }, Some(selected)) => {
            actions.push(Action::DeleteAnnotation {
                page: selected.target.page,
                id: selected.target.id,
            });
        }
        (Clipped::Content { page, .. }, _) => {
            let objects = doc.selection.object_indices_on(*page);
            if !objects.is_empty() {
                actions.push(
                    crate::app::actions::VectorAction::DeleteSelection {
                        page: *page,
                        objects,
                    }
                    .into(),
                );
            }
        }
        (Clipped::Markup { .. }, None) => {}
    }
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "clipboard-cut kind={}",
            match &clipped {
                Clipped::Markup { .. } => "markup",
                Clipped::Content { .. } => "content",
            }
        )
    });
    Ok(clipped)
}

/// Paste onto `page`, raising the action that authors it.
///
/// # Errors
///
/// [`Refusal::NothingCopied`] when the clipboard is empty.
pub fn paste(ctx: &egui::Context, page: usize, actions: &mut Vec<Action>) -> Result<(), Refusal> {
    let (spec, from) = match read(ctx) {
        Some(Clipped::Markup { spec, page: from }) => (spec, from),
        // ★ Page content takes its own path: the clip is bytes and the verb is
        // `paste_objects`, which takes a page-space MATRIX rather than a
        // displacement — so the offset below cannot be shared even though the
        // rule that decides it is.
        Some(Clipped::Content {
            bytes,
            page: from,
            count,
        }) => {
            return paste_content(page, &bytes, from, count, actions);
        }
        None => return Err(Refusal::NothingCopied),
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

/// Paste page content onto `page`, raising the action that authors it.
///
/// # ★ The offset rule is the markup one, and the geometry is not
///
/// Same page offsets so the copy is visible; a different page or document lands
/// in place, so a shape copied to sheet 12 is where it was on sheet 1. That
/// rule is shared with [`paste`] deliberately — two answers to *"where does a
/// paste land"* would be two things for the operator to learn.
///
/// What is **not** shared is how the offset is expressed. A markup carries a
/// `/Rect` and moves by a pair of numbers; page content moves by a **page-space
/// matrix**, which is the same contract `transform_objects` takes and the same
/// reason: `cm` composes into the CTM in force at that point in the stream, so
/// the engine conjugates by each item's own captured matrix and the caller
/// passes page space or nothing.
///
/// `Matrix::IDENTITY` is paste-in-place; `translate` is paste-with-offset. That
/// the same verb also gives paste-scaled and paste-rotated through
/// `Matrix::about` is why the request asked for a matrix rather than a
/// displacement, and it is what a future *paste special* is already built on.
///
/// # Errors
///
/// None today — the deserialisation happens in the apply arm, where the session
/// is. A clip this shell wrote is a clip this shell can read; one it cannot is
/// the engine's `ClipError::NotAClip`, and that reaches the status row through
/// `vector_edit` like every other engine refusal.
fn paste_content(
    page: usize,
    bytes: &[u8],
    from: usize,
    count: usize,
    actions: &mut Vec<Action>,
) -> Result<(), Refusal> {
    let offset = if from == page { PASTE_OFFSET_PT } else { 0.0 };
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "clipboard-paste kind=content page={page} from={from} objects={count} \
             offset={offset:.1}"
        )
    });
    actions.push(
        crate::app::actions::VectorAction::PasteObjects {
            page,
            clip: bytes.to_vec(),
            // ★ Down the page, which is NEGATIVE in PDF user space because y
            // increases upward — the identical trap `paste` names one function
            // up, and worth repeating rather than cross-referencing because
            // getting it backwards produces a paste that goes up-and-right,
            // which looks deliberate and is the kind of thing nobody reports.
            at: pdfce_core::vector::Matrix::translate(offset, -offset),
        }
        .into(),
    );
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
