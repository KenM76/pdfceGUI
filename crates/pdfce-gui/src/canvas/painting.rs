//! # `canvas::painting` — everything the canvas draws, once everything is
//! decided
//!
//! One function, [`draw`], lifted out of [`super::interact`] on 2026-08-19 when
//! that file crossed R2's 1,500-line ceiling for the first time.
//!
//! ## ★ Why this is a seam and not a size
//!
//! `tools/gates/check-file-size.sh`'s own header refuses a split made to fit a
//! number: *"Split the module along its seams — one subject per file."* The
//! seam was already written into `interact`, as the numbered sections of its
//! own body:
//!
//! > 1 the pointer · 2 what a press would land on · 3 advance the gesture ·
//! > 4 the decomposition · 5 apply the gesture · 5b the right-click · 6 keys ·
//! > 7 re-resolve · **8 draw**
//!
//! Sections 1 to 7 answer *what happened and what does it mean*. This answers
//! *what does that look like*: it reads values the first seven produced, writes
//! nothing but pixels, raises no `Action`, and makes no decision.
//!
//! What stayed behind in section 8 is everything that is **not** painting — the
//! typing loop, the keyboard-ownership check and the cursor icon — which had
//! ended up under the same heading because they run at the same moment, not
//! because they are the same subject.
//!
//! ## ★★ The layer order IS this module's content
//!
//! Every position in the sequence is an argument, and each one travelled here
//! with the code rather than being summarised:
//!
//! | layer | why it is where it is |
//! |---|---|
//! | **grid** | under everything: the only thing here about the *paper* rather than about something the operator has selected, searched for or is dragging |
//! | **find highlights** | a wash answering *where is the text I asked about*, under the outline, which is a statement about what a verb would act on |
//! | **selection outlines**, **grips** | |
//! | **marquee** | |
//! | **guides** | on TOP of the selection — a guide is a line the operator aligns to, and an outline a few points across does not hide a hairline crossing it. The reverse order would hide the guide behind the very object being aligned to it |
//! | **move ghost**, **resize ghost** | over the real outline, and both stay visible: the pair is what states the change |
//! | **markup band**, **freehand trail**, **vertex run**, **measure preview** | last, over everything: while a gesture is in flight the shape IS the cursor, and anything drawn over it obscures the one thing being aimed |
//!
//! **Re-ordering any two of these is a behaviour change.**

use egui::{Rect, Ui};

use super::{grid, guides, handles, markup, measure, overlay};
use crate::app::state::OpenDoc;
use crate::canvas::mapping::PageMapping;
use crate::canvas::markup::pen::Pen;
use crate::canvas::measure::Resolved;
use crate::canvas::selection::SelectionState;
use crate::canvas::tool::CanvasTool;

/// Everything the painting pass needs, and nothing else.
///
/// A struct rather than sixteen parameters, and the grouping is a statement:
/// **every member is a product of the decision half.** It also removes the
/// failure a long parameter list invites — five of them are `Option`s and
/// several are adjacent, so a swap would compile.
pub(super) struct Frame<'a> {
    /// The page on screen.
    pub page_index: usize,
    /// The clip rectangle every painter in this pass is bounded by.
    pub clip: Rect,
    /// The frame's screen ⟷ canvas mapping.
    pub map: &'a PageMapping,
    /// The selection as it stands **after** the gesture was applied.
    pub selection: &'a SelectionState,
    /// The rubber-band, if one is in flight.
    pub marquee: Option<Rect>,
    /// The move ghost's canvas-space displacement, if a move would commit.
    pub ghost: Option<egui::Vec2>,
    /// The resize ghost's grip and factors, if a resize would commit.
    pub resize_ghost: Option<(handles::Grip, (f32, f32))>,
    /// The markup band, if one would commit.
    pub band: Option<markup::band::Preview>,
    /// The freehand trail, already simplified, in canvas space.
    pub ink_trail: Option<Vec<egui::Pos2>>,
    /// The armed tool — read by the previews that draw whenever a tool is
    /// armed rather than only during a gesture.
    pub active_tool: CanvasTool,
    /// The markup pen, so a preview is drawn in the colour it will author.
    pub pen: Pen,
    /// The pointer, in screen space, if it is over the window at all.
    pub screen_pos: Option<egui::Pos2>,
    /// The find results, for the highlight wash.
    pub find: &'a crate::find::FindState,
    /// The text sweep's own selection, if there is one.
    pub text_selection: Option<&'a crate::canvas::textsel::TextSelection>,
    /// What the measure tool would snap to under the pointer.
    pub measure_hover: Option<Resolved>,
    /// The measure picks placed so far.
    pub measure_picked: &'a [Rect],
}

/// Paint the canvas.
///
/// Takes `ui` and `doc` alongside [`Frame`] because both are borrows the caller
/// still owns and neither is a *product* of the decision half — putting them in
/// the struct would make it a bag rather than a grouping.
pub(super) fn draw(
    ui: &Ui,
    ctx: &egui::Context,
    doc: &OpenDoc,
    pages: &[crate::canvas::strip::PageView],
    f: &Frame<'_>,
) {
    let Frame {
        page_index,
        clip,
        map,
        selection,
        marquee,
        ghost,
        resize_ghost,
        active_tool,
        pen,
        screen_pos,
        ..
    } = f;
    let (page_index, clip, map, selection) = (*page_index, *clip, *map, *selection);
    let (marquee, ghost, resize_ghost) = (*marquee, *ghost, *resize_ghost);
    let (active_tool, pen, screen_pos) = (*active_tool, *pen, *screen_pos);
    let find = f.find;
    let text_selection = f.text_selection;
    let measure_hover = f.measure_hover;
    let measure_picked = f.measure_picked;
    let ctx = ctx.clone();

    // ---- 8. draw --------------------------------------------------------
    let painter = ui.painter().with_clip_rect(clip);
    // ★ The grid goes UNDER everything, including the find wash. It is the
    // only thing painted here that is about the *paper* rather than about
    // something the operator has selected, searched for or is dragging, so
    // anything drawn over it is a statement about the drawing and must win.
    // Draws nothing at all with the toggle off. See `rulers`' header §2 for
    // why it is per page rather than across the viewport.
    if doc.view.grid {
        grid::draw(ui, doc, pages, clip);
    }
    // ★ The find highlights go on FIRST, under everything else.
    //
    // They are a wash over page content — an answer to "where is the text I
    // asked about" — while the selection outline is a statement about what a
    // verb would act on. Painting the wash over the outline would dim the
    // control feedback with a hint; painting it under leaves both readable.
    //
    // `page_highlights` yields nothing at all when the results are not current
    // — a stale epoch, a query the operator has edited, a closed bar — so an
    // edit stops the highlights by supplying an empty iterator rather than by
    // a check here. That is what keeps rule 4: this file cannot paint a mark
    // over content the search no longer describes, because it is never handed
    // one. See `crate::find`'s staleness section.
    //
    // ★ **Once per drawn page, each through its own map** — the one place the
    // canvas is legitimately about pages other than the one being acted on. A
    // search describes the whole document, so under a continuous mode its hits
    // are on several of the pages on screen at once, and painting them all
    // through the acting page's map would stack every page's highlights onto
    // one page. That is the failure this feature was most likely to ship
    // silently: the hits are found, the wash is drawn, and it is drawn in the
    // wrong place — which looks like a highlight bug rather than a mapping one.
    //
    // The loop reduces to exactly the previous call under `Single`, where
    // `pages` holds one entry and it is the acting page.
    for view in pages {
        overlay::draw_find_hits(
            &painter,
            ui.visuals(),
            &view.map,
            find.page_highlights(view.page, doc.edit_epoch),
        );
    }
    // ★ The text selection's wash, in the same layer as the find wash and for
    // the same reason: both are statements about *characters on the page*
    // rather than about a control, so they belong under anything that describes
    // a verb's operand. They cannot in fact both be on screen over the same
    // glyphs and matter — Find is a query and this is a sweep — but the
    // ordering is stated rather than left to chance, because the day they
    // overlap the reader has to be able to see both.
    //
    // Per drawn page, through that page's own map, exactly as the find wash is:
    // the selection is single-page, so all but one iteration is handed an empty
    // slice — but painting through the *acting* page's map instead would put a
    // continuous-strip selection on the wrong sheet, which is the failure the
    // find wash's own comment records as the one most likely to ship silently.
    for view in pages {
        overlay::draw_text_selection(
            &painter,
            ui.visuals(),
            &view.map,
            text_selection
                .as_ref()
                .map_or(&[][..], |s| s.highlights(view.page, doc.edit_epoch)),
        );
    }
    overlay::draw_selection(&painter, ui.visuals(), map, selection);
    draw_anchors(&painter, ui, doc, map, selection, page_index);
    if let Some(rect) = marquee {
        overlay::draw_marquee(&painter, ui.visuals(), map, rect);
    }
    // The ghost sits ON TOP of the real outline, and both stay visible: the
    // pair is what states the displacement. `ghost` is `Some` only when
    // `moving::drag` has already established that the release will commit — a
    // preview of a move that will be refused is the thing rule 4 and the
    // no-placeholders invariant both forbid.
    // The guides sit on TOP of the selection, and the order is the point: a
    // guide is a line the operator has to see while they align something to
    // it, and a selection outline is a box a few points across that a hairline
    // crossing it does not hide. The reverse order would hide a guide behind
    // exactly the object the operator is aligning to it.
    guides::draw(ui, doc, pages, clip);
    if let Some(delta) = ghost {
        overlay::draw_move_ghost(&painter, ui.visuals(), map, selection, delta);
    }
    // ★ The resize ghost, on the same layer and under the same contract: it is
    // `Some` only when `resizing::drag` has established that a release would
    // commit, so a preview of a refused gesture is never drawn. The anchor is
    // re-read from the same `grip_box` the drag measured against rather than
    // carried on the value, because it is a pure function of the selection and
    // carrying it would be a second copy that could go stale between the frame
    // that computed it and the frame that paints.
    if let Some((grip, factors)) = resize_ghost
        && let Some(bounds) = overlay::grip_box(map, selection)
    {
        overlay::draw_resize_ghost(
            &painter,
            ui.visuals(),
            map,
            selection,
            // ★ `pivot`, not `anchor` — the SAME point `canvas::resizing`
            // commits about. `anchor` is where the grip is; the pivot is the
            // opposite corner, which is what stays still. Using the wrong one
            // here would preview a shape growing away from the operator's hand
            // and then commit one growing towards it, so the object would jump
            // by its own size on release.
            grip.pivot(bounds),
            factors,
        );
    }
    // Last, and over everything: the band IS the cursor for as long as it
    // exists, and a guide or an outline drawn over the shape being authored
    // would obscure the one thing the operator is aiming.
    if let Some(band) = f.band {
        markup::band::draw_preview(&painter, map, band, pen);
    }
    // …and the freehand trail, on the same argument and in the same layer: while
    // the button is down the stroke IS the cursor, and it is drawn from the
    // simplified point list the release will author rather than from the raw
    // input, so the mark does not visibly change shape at the moment it commits.
    if let Some(trail) = &f.ink_trail {
        markup::ink::draw_preview(&painter, map, trail, pen);
    }
    // ★ …and the vertex run, which is drawn on EVERY frame the tool is armed
    // rather than only while a gesture is in flight — because for this family
    // there is no "in flight" the frame can see. A run between clicks is a
    // pointer that is not down, so a preview gated on a gesture would appear only
    // during the instant of a click and the operator would be placing vertices
    // into a canvas that never showed them.
    //
    // It takes the frame's `map` and the pointer, and it draws three things: the
    // committed run, the rubber segment to the pointer, and — for a Polygon
    // alone — the closing segment back to the first vertex, which is the single
    // visible difference between the two tools before the commit. See
    // `markup::vertex::preview`.
    if let Some(kind) = active_tool.markup_kind().filter(|k| k.is_vertex()) {
        markup::vertex::preview(
            ui,
            doc.current_page(),
            page_index,
            kind,
            map,
            screen_pos.map(|p| map.to_page(p)),
            pen,
        );
    }
    // …and the measure preview, on the same argument: while a pick is in
    // progress the preview IS the cursor, and it describes what the next click
    // will commit.
    //
    // ★ It takes the frame's `map`, and the comment here used to say it did not
    // need one *"because it converts through the renderer's own page
    // transform"*. That was the defect: the renderer's transform at scale 1.0
    // lands in **canvas** space — page top-left origin, no zoom — and the
    // painter speaks screen, so every mark the measure preview drew was offset
    // by wherever the page sat in the window and drawn at 100 % whatever the
    // magnification. See `measure::page_to_screen`, which is now the one place
    // both hops happen.
    if let Some(kind) = active_tool.measure_kind() {
        measure::preview(
            ui,
            measure::Preview {
                doc,
                page_index,
                kind,
                map,
                hover: measure_hover,
                picked: measure_picked,
            },
        );
    }
    // ★ …and the caret, which is the same argument once more: while a draft is
    // in flight the caret IS the cursor, and it describes where the next
    // keystroke lands.
    //
    // It draws a caret and an extent bracket and **no glyphs** — see
    // `textedit::preview`, which carries the argument for why a better ghost is
    // the wrong fix for `DEFECTS.md` D4a rather than a deferred one.
    if active_tool.text_edit_kind().is_some() {
        crate::canvas::textedit::preview(
            ui,
            &ctx,
            &crate::canvas::textedit::Preview {
                doc,
                page_index,
                map,
            },
        );
    }

    // ★ **The keystrokes**, read raw and consumed here.
    //
    // After the gesture machine and before the cursor, which is the only place
    // it can be: it needs `actions` (Enter commits) and it must not run on a
    // frame the canvas does not own the keyboard for.
    //
    // `!ctx.text_edit_focused()` is the guard, and it is `DEFECTS.md` **D1**'s
    // predicate rather than `egui_wants_keyboard_input()` — for the identical
    // reason `app::keyboard` and `canvas::tool::space_held` use it. The wrong
    // one is true whenever *any* widget has focus, and the canvas takes focus on
    // click, so a build using it would stop accepting characters the moment the
    // operator clicked the page they are trying to type on. The right one asks
    // whether a **text field** has it — the page-number box, a Properties value
    // — which is the only case where a character is not ours.
}

/// Mark the entered object's anchors when the operator is inside one.
///
/// # ★ Why this is a function here rather than three lines at the call site
///
/// Because it is the **only** place in the paint pass that needs the object
/// model, and reaching for it costs a `Ref` into the document's decomposition
/// cache. Keeping that borrow inside one short function is what guarantees it is
/// released before the rest of the frame — the same discipline
/// `app::cache::page_objects`' own docs set out, and the reason
/// `canvas::interact` has a comment about dropping its `Ref` explicitly.
///
/// It draws nothing at the Object rung. An object's anchors are not the
/// operator's subject there — the object is — and painting thousands of hollow
/// squares over a selection they are about to *move as a whole* would be noise
/// with a rendering cost.
fn draw_anchors(
    painter: &egui::Painter,
    ui: &Ui,
    doc: &OpenDoc,
    map: &PageMapping,
    selection: &SelectionState,
    page_index: usize,
) {
    use crate::canvas::selection::SelectionLevel;

    if !matches!(
        selection.level(),
        SelectionLevel::Part | SelectionLevel::Node
    ) {
        return;
    }
    let Some(entered) = selection.entered_object() else {
        return;
    };
    if entered.page != page_index {
        return;
    }
    let Ok(object) = usize::try_from(entered.object.0) else {
        return;
    };
    let Some(page) = doc.pages.get(page_index) else {
        return;
    };
    let Some(provider) = doc.page_objects() else {
        return;
    };
    // ★★ **The entered SUBPATH's anchors, not the object's** — and this is the
    // difference between a usable feature and a decoration.
    //
    // The first version drew the whole object's, with a 400-anchor cap above
    // which the unselected ones were suppressed. Driving it against
    // `SW41177.pdf` produced `canvas-anchors total=4972`: one object on this
    // operator's own drawing carries five thousand anchors, so the cap fired,
    // nothing unselected drew, and the operator had **no way to see where any
    // anchor was** — on precisely the documents the feature exists for.
    //
    // A cap that suppresses the answer on the documents that need it is not a
    // performance guard, it is the feature not working. The right scope was
    // there all along and is also the semantically correct one: the operator
    // descended into a *subpath*, and its anchors are what they may pick. A
    // subpath is tens of anchors where an object is thousands, so the cap
    // becomes a backstop rather than the normal case.
    //
    // `subpath_node_points` returns OBJECT-scoped indices, which is what the
    // selection and `move_nodes` both speak — the offset arithmetic lives in
    // the provider, in one place, exactly so callers like this one cannot get
    // it subtly wrong.
    //
    // Converted to canvas space HERE, by the one function entitled to do it,
    // and the `Ref` is dropped before anything is painted.
    let anchors = match entered.subpath {
        Some(subpath) => provider.subpath_node_points(object, subpath),
        // No part entered yet — the Node rung is unreachable from here, and the
        // object's whole anchor list is the honest answer to "what could you
        // descend into". The cap still applies and still fires on a CAD object,
        // which is correct: at the Part rung the operator's subject is the
        // subpath, and five thousand dots would be noise rather than an answer.
        None => provider.object_node_points(object),
    };
    let points: Vec<(usize, egui::Pos2)> = anchors
        .into_iter()
        .filter_map(|(index, p)| {
            crate::viewer::pdf_space_to_canvas(egui::pos2(p.x as f32, p.y as f32), page)
                .map(|c| (index, c))
        })
        .collect();
    drop(provider);

    let selected = selection
        .selected_nodes_on(page_index, entered.object)
        .into_iter()
        .collect();
    overlay::draw_anchors(painter, ui.visuals(), map, &points, &selected);
}
