//! # `canvas::pressing` — **what a press would land on, and what it would mean**
//!
//! ## Why this is its own file
//!
//! R2, on 2026-08-19, when the Node tool and the Bézier-handle hit test pushed
//! `canvas::interact` past 1,500 lines. It is a real seam rather than a
//! convenient cut: everything here answers one question — *if the primary
//! button went down at this point, right now, what would happen?* — and nothing
//! here changes anything. `interact`'s remaining sections advance a gesture,
//! route a click and paint; this one only looks.
//!
//! ## ★★ The precedence, in one place, because it was learned three times
//!
//! Four different things can be under the pointer at once, and the order they
//! are asked in is the whole behaviour:
//!
//! | # | claimant | why it outranks the next |
//! |---|---|---|
//! | 1 | a **Bézier handle** of a selected anchor | it sits *inside* the selection box, so anything asked before it would swallow every press on one |
//! | 2 | an **anchor**, reached through the inflated move box | an anchor sitting on the bounding box's edge is half outside it |
//! | 3 | a **resize grip** — Object rung only | it scales the whole object, which is the wrong subject at an inner rung |
//! | 4 | the **selection body** → move | the least specific claim, so it answers last |
//!
//! **The most specific thing under the pointer wins, and specificity is depth
//! down the selection ladder.** That sentence is the rule, and each clause of
//! it was paid for separately on 2026-08-19:
//!
//! 1. the corner resize grips covered the corner **anchors**, so the end points
//!    of every path were undraggable while the middle ones worked — fixed by
//!    confining the eight grips to the Object rung;
//! 2. the move box still did not *reach* an anchor sitting on its own edge,
//!    which is the same defect from the other side and needed an **inflation**
//!    rather than another suppression;
//! 3. `grip_at` answered `Move` for every press on a **handle**, so every
//!    attempt to shape a curve moved the whole object — and that one is entirely
//!    plausible from a chair, because the object *did* move.
//!
//! ## ★ Everything here reads `press_origin`, not the current pointer
//!
//! `egui` does not call an interaction a drag until the pointer has travelled a
//! threshold, so by the frame it says so the pointer is **already that far from
//! where it went down** — measured at 94 PDF points of error on an A1 sheet at
//! 0.21× zoom. A grip is an 8 pt square and a handle mark is 7 pt across.
//! Reading the current position misses both, and the miss is silent: the
//! gesture becomes a marquee, which *clears the selection the operator was
//! trying to resize*.

use egui::Pos2;

use crate::app::modes::Capabilities;
use crate::app::state::OpenDoc;
use crate::canvas::gesture::{self, PressMeaning};
use crate::canvas::handles::Grip;
use crate::canvas::mapping::PageMapping;
use crate::canvas::selection::{SelectionLevel, SelectionState};
use crate::canvas::tool::CanvasTool;
use crate::canvas::{annotdrag, dimdrag, handledrag, handles, overlay, widgetdrag, zoom};

/// Everything the frame learned by looking at where a press would land.
///
/// Returned as one struct rather than a tuple because four of the five members
/// are `Option`s of similar-looking types, and a caller that transposed two of
/// them would compile. Named fields make that a spelling error instead.
pub struct Press {
    /// The grip under the press origin, if any. `Grip::Move` for a press
    /// anywhere inside the (possibly inflated) selection box.
    pub grip: Option<Grip>,
    /// The Bézier handle under the press origin, as `(anchor, side)`.
    pub handle: Option<(usize, pdfce_core::vector::Handle)>,
    /// Every drawn handle of the selected anchors, in canvas space.
    ///
    /// Carried out as well as used here, because the paint pass wants the same
    /// list and re-deriving it would mean a second `page_objects()` borrow and
    /// a second answer that could differ from this one.
    pub visible_handles: Vec<(usize, pdfce_core::vector::Handle, Pos2)>,
    /// What a press means — the drag it would start, and whether a click has a
    /// meaning at all.
    pub meaning: PressMeaning,
}

/// Look at the pointer and answer what a press would do.
///
/// Changes nothing. See the module header for the precedence and for why every
/// hit test reads `press_origin`.
///
/// # Why eight arguments rather than a `Frame` struct
///
/// Because every one of them is a borrow the caller already holds and none is a
/// *product* of anything here — the same call `canvas::painting::draw` makes
/// when it takes `ui` and `doc` alongside its `Frame`. A struct assembled at the
/// one call site, passed once, and destructured immediately would be a grouping
/// that groups nothing.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn look(
    ctx: &egui::Context,
    doc: &OpenDoc,
    selection: &SelectionState,
    map: &PageMapping,
    page_index: usize,
    screen_pos: Option<Pos2>,
    active_tool: CanvasTool,
    caps: Capabilities,
) -> Press {
    let at_object_rung = selection.level() == SelectionLevel::Object;

    // ★★ At an inner rung the move-hit box is INFLATED by an anchor mark's
    // width, and without it the outermost anchors of every path are undraggable.
    //
    // An anchor mark is centred on its point, so an anchor sitting on the
    // object's bounding box is half outside it — and `grip_at` answers `Move`
    // only for a press *inside* the box. Confining the eight scale grips to the
    // Object rung stopped them CLAIMING that press; this is the other half,
    // which makes the move claim it. The operator's version of the pair is
    // *"I can drag the middle points and not the end ones"*.
    // ★★ A selected ce dimension supplies its OWN move box, and it comes
    // first.
    //
    // `overlay::grip_box` derives its answer from the selection's cached
    // content outlines, which `select_annot` clears - an annotation is not
    // content and has nothing decomposed to cache. So over a selected dimension
    // it answers `None`, the press fell through to a marquee, and pressing on
    // the dimension REPLACED the selection the operator was trying to drag.
    // That is the operator's report of 2026-08-20 in its mechanical form.
    //
    // `dimdrag::grab_box` is `Some` only for a dimension a placement drag can
    // actually finish, so no gesture is ever started that could not commit.
    let dimension_box = dimdrag::grab_box(doc, map, selection);
    let grip_box = dimension_box.or_else(|| {
        overlay::grip_box(map, selection).map(|b| {
            if at_object_rung {
                b
            } else {
                b.expand(overlay::ANCHOR_PX)
            }
        })
    });

    let origin = ctx.input(|i| i.pointer.press_origin()).or(screen_pos);

    let grip = grip_box.zip(origin).and_then(|(bounds, p)| {
        // ★ The eight scale handles belong to the Object rung. Two reasons, and
        // the second was found by driving: the subject is wrong at an inner rung
        // (the operator said *this point*, not *this whole shape*), and the
        // corner grips physically cover the corner ANCHORS.
        //
        // ★ And never over a dimension. A ce dimension has no scale verb, so
        // offering resize grips on one would be eight visible controls that
        // silently do nothing - and worse than inert, because each grip would
        // CLAIM the press and stop the corners of the box from moving the
        // dimension. The whole box is the move target.
        handles::grip_at(bounds, p, at_object_rung && dimension_box.is_none())
    });

    // The provider is asked for only at an inner rung — `handledrag::visible`
    // returns empty above it — so the ordinary case pays one `entered_object()`
    // and one `subpath` check.
    let visible_handles = doc
        .page_objects()
        .zip(doc.pages.get(page_index))
        .map(|(provider, page)| handledrag::visible(selection, &provider, page, page_index))
        .unwrap_or_default();

    let handle = origin.and_then(|p| handledrag::at(&visible_handles, map, p));

    // ★★ What a press on a selected ce dimension landed on — a corner handle,
    // its body, or nothing.
    //
    // Sampled here with the other two hit tests so that a press has one meaning
    // decided in one place (this module's header), and resolved to a VALUE
    // rather than left as two booleans, so `gesture::press_kind` stays free of
    // geometry.
    //
    // ★ A corner outranks the body, and it must: a handle sits ON the shape, so
    // every press that hits a handle also hits the body. Of the two readings,
    // the one the operator aimed at is the small square they can see.
    //
    // Cheap to ask — it answers `None` immediately unless an annotation is
    // selected and its sidecar record is a draggable kind.
    let dimension = origin.and_then(|p| {
        dimdrag::vertex_at(doc, map, selection, p)
            .map(gesture::DimensionPress::Vertex)
            .or_else(|| {
                dimension_box
                    .filter(|b| b.contains(p))
                    .map(|_| gesture::DimensionPress::Body)
            })
    });

    // Whether the press landed inside a selected MARKUP annotation's own box.
    //
    // ★ Sampled here with the other hit tests, resolved to a bool rather than
    // left for `press_kind` to compute, so that function stays free of geometry
    // — this module's stated contract. `annotdrag::grab_box` answers `None`
    // unless the selection is a markup this shell can actually move, so no
    // gesture is started that could not commit.
    let markup_body =
        origin.is_some_and(|p| annotdrag::grab_box(map, selection).is_some_and(|b| b.contains(p)));

    // Whether the press landed inside the selected FORM FIELD's box.
    //
    // ★ Same shape as `markup_body` above and the same contract:
    // `widgetdrag::grab_box` answers `None` unless a widget is selected and
    // still present in the form, so no gesture is started that could not
    // commit. The target list it consults is cached on `(path, edit_epoch)`,
    // so asking every frame costs a map lookup rather than a form walk.
    let widget_body =
        origin.is_some_and(|p| widgetdrag::grab_box(ctx, doc, map).is_some_and(|b| b.contains(p)));

    let meaning = gesture::press_kind(
        gesture::Press {
            tool: active_tool,
            grip,
            handle,
            dimension,
            markup_body,
            widget_body,
            zoom_armed: zoom::region_zoom_armed(ctx),
        },
        caps,
    );

    Press {
        grip,
        handle,
        visible_handles,
        meaning,
    }
}
