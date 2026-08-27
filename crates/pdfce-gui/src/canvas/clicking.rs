//! # `canvas::clicking` — what a click MEANS
//!
//! ## The seam
//!
//! Split out of [`crate::canvas::interact`] on 2026-08-20 under R2, when the
//! Shift constraints took that file past the 1,500-line ceiling. It is the seam
//! the file was always going to split along rather than the cheapest one to
//! reach: `interact`'s subject is *one frame of canvas interaction* — read the
//! pointer, advance the gesture machine, decompose if a hit test needs it, route
//! the outcome, re-resolve, draw — and this is one of eleven outcomes it routes,
//! carrying a third of its lines.
//!
//! [`crate::canvas::pressing`] already owns the companion question, *what would
//! a press land on*. This owns *what does a completed click do about it*, and
//! the pair is easier to reason about than either was inside `interact`.
//!
//! ## ★★ The whole subject is a LADDER, and its order is the design
//!
//! A click is exactly one thing. Never two. The arms below are tried in order
//! and the first that answers consumes the click:
//!
//! | # | arm | why it is here and not one rung later |
//! |---|---|---|
//! | 1 | **the Node tool** | the most specific: the operator armed a tool whose entire subject is anchors |
//! | 2 | **the text caret** | an armed tool owns the press — this codebase's rule everywhere |
//! | 3 | **an annotation under the pointer** | below every armed tool, above the text fall-through |
//! | 4 | **a text sweep** | the fall-through for Read and Review |
//! | 5 | **a vertex markup** (PolyLine, Polygon) | a click-built shape; mutually exclusive with 4 by armed tool |
//! | 6 | **a sticky note** | the one text annotation placed by a click rather than a drag |
//! | 7 | **a measure pick** | the dimension tools |
//! | 8 | **content selection** | what a click meant before any of the above existed |
//!
//! Rung 3 is the one whose position was got wrong once and cost the operator
//! four reports — see its own comment, which is preserved verbatim below along
//! with the record of a diagnosis that was wrong and the test that caught it.
//!
//! ## conventions: click-selects
//!
//! Corpus: `ui-conventions/click-selects.md`.
//!
//! ★ **Most of that corpus is about a HIT TEST and this module is about
//! ROUTING**, so most rows below name where the rule actually lives rather than
//! claiming it. That is the honest answer and it is not a dodge: C8 — *click
//! priority is stated, not emergent* — is **this module's whole subject**, and
//! it is the row that had no single home until the ladder was extracted into a
//! file whose header could state it.
//!
//! - C1 ink-not-bounding-box: NOT THIS FILE — `ObjectModelProvider::subpath_hits`
//!   and `selection::annot::under_pointer` decide it, and the outstanding
//!   `/Square` case is `OPERATOR_REQUESTS.md` O14 row 8.
//! - C2 unfilled-interior-belongs-behind: NOT THIS FILE — same two places, same
//!   open row.
//! - C3 topmost-wins: `super::input::probe` returns the front-most target and
//!   rung 8 hands it to `SelectionState::click` unchanged; rung 3's
//!   `under_pointer` walks `/Annots` in paint order for the same reason.
//! - C4 tolerance-in-screen-units: `map.tolerance()` — the frame's one mapping,
//!   so it is constant at every zoom. Passed, never re-derived.
//! - C5 segments-clamp: NOT THIS FILE — the provider's distance tests.
//! - C6 empty-space-deselects: **answered here, twice.** Rung 8's
//!   `SelectionState::click` clears on a miss, and the `annot_hit.is_none()`
//!   guard above the ladder clears the annotation selection on a miss in the
//!   modes that have one — which had to be explicit, because an annotation
//!   selection is a second store that rung 8 cannot see.
//! - C7 drawn-outline-is-the-live-target: NOT THIS FILE — the painters and
//!   `dimdrag::vertex_at`/`handles::grip_at` own the drawn-vs-target pair, and
//!   that corpus row records the 2026-08-20 defect where they disagreed.
//! - C8 priority-is-stated: **this module.** The eight rungs above, in one
//!   place, in order, each with the reason it sits where it does — and each
//!   reachable only through this one function, so a press cannot mean different
//!   things depending on which path ran first.

use crate::app::actions::forms::FieldAction;
use egui::Pos2;

use crate::app::actions::Action;
use crate::app::modes::Capabilities;
use crate::app::state::OpenDoc;
use crate::canvas::input::probe;

/// ★★★ **How deep into the stack under one point the operator has asked to
/// go**, and where they asked it.
///
/// # What this closes
///
/// The operator, 2026-08-26: *"when I click on one of the objects all I get is
/// the page selected."* The engine already computed the whole front-to-back
/// list of what a click is over; this shell took the first entry and discarded
/// the rest, so anything underneath anything was unreachable at every point,
/// for ever.
///
/// `Alt`+click at the same place now steps one deeper each time and wraps —
/// which is Illustrator's *Select Behind* (`Ctrl`+click there) and Figma's
/// deep-select, the two conventions for exactly this.
///
/// # ★★ Why it resets on pointer travel, and why the threshold is generous
///
/// A depth is only meaningful *at a point*: three clicks in three different
/// places are three first clicks, not a walk into a stack. So the cursor
/// remembers where it was established and resets when the pointer has moved
/// away from there.
///
/// [`CYCLE_RESET_PTS`] is the radius. It is deliberately larger than a pixel:
/// an operator holding `Alt` and clicking repeatedly does not hold the mouse
/// perfectly still, and a one-pixel threshold would silently restart the cycle
/// on the second click and make the feature look broken in the most confusing
/// possible way — it would work, sometimes, depending on how steady their hand
/// was.
#[derive(Clone, Copy, Debug, Default)]
struct CycleCursor {
    /// Where the cycle was established, in canvas space.
    at: egui::Pos2,
    /// How many candidates to skip. `0` is a plain click.
    depth: usize,
}

/// How far the pointer may drift and still be "the same point" for cycling, in
/// canvas points. See [`CycleCursor`].
const CYCLE_RESET_PTS: f32 = 4.0;

/// The `egui::Memory` slot [`CycleCursor`] lives in.
const CYCLE_MEMORY_KEY: &str = "pdfce-canvas-cycle"; // ui-text-exempt: internal memory id, never displayed

/// **What depth this click means**, advancing or resetting the cursor.
///
/// `alt` is the operator asking to go deeper. Without it the cursor is reset,
/// so an ordinary click always lands on the front-most candidate — which is
/// what makes this feature invisible to anyone not using it.
fn cycle_depth(ctx: &egui::Context, point: egui::Pos2, alt: bool) -> usize {
    let id = egui::Id::new(CYCLE_MEMORY_KEY);
    let previous = ctx
        .data_mut(|d| d.get_temp::<CycleCursor>(id))
        .unwrap_or_default();
    let same_place = previous.at.distance(point) <= CYCLE_RESET_PTS;
    let next = if alt && same_place {
        CycleCursor {
            at: previous.at,
            depth: previous.depth.saturating_add(1),
        }
    } else if alt {
        // First `Alt`+click at a new point: step past the front-most candidate,
        // because a plain click already offers that one and repeating it would
        // make the modifier look inert.
        CycleCursor {
            at: point,
            depth: 1,
        }
    } else {
        CycleCursor {
            at: point,
            depth: 0,
        }
    };
    ctx.data_mut(|d| d.insert_temp(id, next));
    next.depth
}
use crate::canvas::mapping::PageMapping;
use crate::canvas::pick::PickFilter;
use crate::canvas::selection::SelectionState;
use crate::canvas::textsel::TextSelection;
use crate::canvas::tool::CanvasTool;
use crate::panels::objects::provider::ObjectModelProvider;

/// Everything a completed click needs, gathered by the caller.
///
/// The `Frame` shape this codebase already uses for `resizing`, `handledrag`
/// and `dimdrag`, and for the reason those give: the members are read-only
/// facts about one frame, so grouping them says what they are and removes the
/// failure a long parameter list invites — three of the four `bool`s below
/// would compile in each other's places.
///
/// The two things that are **mutated** stay outside it, deliberately: a
/// `Frame` is what the frame knows, and a selection is what the document is.
pub struct Frame<'a> {
    /// The frame's context, for the caret and the pick, both of which store
    /// per-frame state in `egui::Memory`.
    pub ctx: &'a egui::Context,
    /// The open document. Read-only: everything that changes it goes through
    /// `actions`.
    pub doc: &'a OpenDoc,
    /// Which page the click is on.
    pub page_index: usize,
    /// The frame's one screen ⟷ canvas mapping.
    pub map: &'a PageMapping,
    /// The decomposition, if this frame asked for one. `None` is not an error:
    /// `interact` builds it only when the frame's outcome needs a hit test, and
    /// every rung below that consumes it degrades to "nothing was hit".
    pub targets: Option<&'a ObjectModelProvider>,
    /// Which tool is armed. The primary discriminator of the ladder.
    pub active_tool: CanvasTool,
    /// What this mode is allowed to do.
    pub caps: Capabilities,
    /// ★ What the OPERATOR is allowing clicks to land on.
    ///
    /// Beside `caps` because the two compose, and the composition is an
    /// `AND` in one direction only: a mode decides what may be authored,
    /// this decides what is worth pointing at, and switching a class on
    /// here can never grant a capability the mode withholds. See
    /// [`crate::canvas::pick`]'s header.
    ///
    /// Sampled once per frame for the same reason `caps` and `active_tool`
    /// are: a gesture means what it meant when it started.
    pub pick: PickFilter,
    /// The markup pen's current settings, for a vertex markup's click.
    pub pen: crate::canvas::markup::pen::Pen,
    /// Where the click landed, in canvas space.
    pub point: Pos2,
    /// Whether Shift was held **at the press** — extend rather than replace.
    pub shift: bool,
    /// The second click of a double.
    pub double: bool,
    /// The third click of a triple.
    pub triple: bool,
}

/// **Route one completed click.**
///
/// See the module header for the ladder and its order. Raises actions and
/// mutates the two selections; changes no document directly.
pub fn click(
    frame: Frame<'_>,
    selection: &mut SelectionState,
    text_selection: &mut Option<TextSelection>,
    actions: &mut Vec<Action>,
) {
    let Frame {
        ctx,
        doc,
        page_index,
        map,
        targets,
        active_tool,
        caps,
        pick,
        pen,
        point,
        shift,
        double,
        triple,
    } = frame;

    // ★★ The annotation under the pointer, resolved BEFORE the ladder.
    //
    // Ahead of it rather than inside it because the arm that consumes
    // this has to be an `if let` — a click that hits nothing must fall
    // through and mean exactly what it meant before this feature
    // existed. See that arm for the full reasoning.
    //
    // # The guard, and why each half of it
    //
    // **`CanvasTool::Select`** — nothing armed. With a pen, a caret or
    // a measure tool armed the press belongs to that tool, which is
    // this codebase's rule everywhere else, so an annotation
    // underneath must not steal it.
    //
    // **`caps.author_markup`** — Review and Edit, not Read. Read is a
    // reader: it may fill a form and sweep text and may not change the
    // document, and selecting a stamp exists in order to act on it.
    //
    // # Cost
    //
    // One `/Annots` walk and one `/PieceInfo` read, **per click** —
    // not per frame. Both are bounded by the number of annotations
    // rather than by document size, and neither decomposes anything:
    // an annotation's geometry is its `/Rect`, four numbers in a
    // dictionary. A click on the 129,758-object benchmark sheet costs
    // the same as a click on a blank page.
    let annot_hit =
        if matches!(active_tool, crate::canvas::tool::CanvasTool::Select) && caps.author_markup {
            crate::canvas::selection::annot::under_pointer(doc, page_index, point, map)
        } else {
            None
        };
    // A click that missed every annotation, in a mode that could have
    // hit one, **deselects**. Clicking away is the gesture every
    // operator tries first, and without this the outline would survive
    // a click on blank paper — which reads as the selection being
    // stuck rather than as the click having missed.
    if annot_hit.is_none() && caps.author_markup {
        selection.clear_annot();
    }
    // ★ A click is a measure pick, a **text** gesture, or a content
    // selection — never two of them.
    //
    // The text branch asks `super::textsel::takes_the_press` again rather than
    // inferring "the click must be text because the mode cannot select
    // content": `press_kind` reports `click: true` for *two* different
    // reasons, and inferring from the flag would be a second, quieter
    // statement of the rule, free to disagree with the one that decided
    // the drag. One function, two readers — the same shape
    // `active_tool.measure_kind()` already has one line above.
    // ★ …and the **caret** tool takes it before either, which is
    // `press_kind`'s own rung order restated where the click is routed.
    //
    // It has to be restated rather than inferred, for the reason the
    // paragraph below gives about the text branch: `press_kind` reports
    // `click: true` for three different reasons now, and reading the flag
    // would be a second, quieter statement of a rule that is already
    // written. Asking `text_edit_kind()` again is asking the same
    // question of the same value.
    //
    // A refusal is shown rather than swallowed. That is D4a's whole
    // lesson: the old shell's answer to a caret it could not place was a
    // boolean and a keyboard that stopped responding.
    // ★★ **The Node tool takes the click before anything else**, and
    // it is first because it is the most specific: the operator armed a
    // tool whose entire subject is anchors, so a click means an anchor
    // if one is there and "show me this shape's anchors" if not. See
    // `SelectionState::click_direct`.
    //
    // `hit` here already carries the object, the nearest part AND the
    // nearest node, because the probe that produced it is the one a
    // double-click descent uses. That is why this needed no new query:
    // the information the ladder made you perform two gestures to reach
    // was in the very first click all along.
    //
    // ★ **The text tool's kind is decided by the CLICK, not by which
    // ribbon command was pressed**, as of 2026-08-19. `CanvasTool::Text`
    // in a mode that can author now places a caret; `textedit::click`
    // falls back to a fresh origin when the point names no run. The
    // operator's report is why:
    //
    // > *"How do I edit text when on the canvas? I get a box and the I
    // > cursor, but I can't type anything. How do I make new text when I
    // > click on the canvas and expect to edit there? Same problem."*
    //
    // He was getting the I-beam because the text tool SWEPT text, and
    // the tool that types was a different tool reachable only through
    // `Edit ▸ Content ▸ Edit text` — four steps of ritual before a
    // character could be typed, and no surface anywhere saying so. One
    // tool now, click decides, which is Illustrator, Word, Inkscape and
    // every other program he has used.
    let text_kind = active_tool.text_edit_kind().or_else(|| {
        (active_tool.is_text() && caps.edit_content)
            .then_some(crate::canvas::textedit::TextEditKind::Edit)
    });
    if active_tool.is_node() && caps.edit_content {
        let hit = targets
            // Depth 0: the Node tool addresses anchors within an object it
            // has already entered, so "the object underneath" is not a
            // question it asks.
            .map(|t| probe(t, selection, page_index, point, map, pick, 0))
            .unwrap_or_default();
        selection.click_direct(page_index, hit, shift);
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "canvas-selection via=node-tool mod={shift} sel={} level={:?} node={:?}",
                selection.len(),
                selection.level(),
                hit.node
            )
        });
    } else if let Some(kind) = text_kind {
        match crate::canvas::textedit::click(
            ctx,
            &crate::canvas::textedit::Click {
                doc,
                page_index,
                kind,
                canvas_point: point,
            },
            actions,
        ) {
            Ok(()) => {}
            Err(refusal) => {
                crate::app::actions::record_note(
                    doc.edit_epoch,
                    crate::text::textedit::refusal(refusal).to_owned(),
                );
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    format!("text-edit-declined reason={refusal:?}")
                });
            }
        }
    // ★★ **AN ANNOTATION UNDER THE POINTER TAKES THE CLICK.**
    //
    // The arm that closes `FEATURES.md`'s *"the canvas selection cannot
    // address an annotation"*, reported by the operator four ways:
    // *"How do I edit a stamp I've applied?"*, *"I still can't get to
    // edit dimension groups when I click on it."*
    //
    // # Why it sits HERE and not one arm earlier or later
    //
    // **Below every armed tool**, because this codebase's stated rule is
    // *"the press belongs to whichever tool is armed"* — an operator who
    // armed the caret, a pen or a measure tool asked for that gesture,
    // and a stamp underneath must not steal it.
    //
    // **Above the text-selection fall-through**, which is the arm that
    // was silently swallowing these clicks. `super::textsel::takes_the_press`
    // is true for the plain Select tool whenever `edit_content` is
    // false — i.e. in **Read and Review** — so in Review, the mode an
    // operator is in *because* they are working on markup, every click
    // on a stamp was being consumed as a text-selection click. Nothing
    // was broken downstream; the click never got there.
    //
    // ★ **My first diagnosis of this was wrong and a test caught it.**
    // I read `press_kind`'s `click: caps.edit_content || text` and
    // concluded Review produced no click event at all. It produces one
    // — `text` is true there for exactly the reason above — and
    // `review_mode_places_markup_but_refuses_content` failed against
    // the "fix", which is the second time this week a test has been the
    // thing that noticed. The predicate was not too coarse; the
    // ROUTING had no arm for annotations.
    //
    // # Why a miss falls through rather than swallowing the click
    //
    // Because this arm must be **additive**. A click that hits no
    // annotation has to mean exactly what it meant before — text in
    // Review, content in Edit — or adding annotation selection would
    // have taken away text selection in the same stroke. `annot_hit`
    // is therefore computed ahead of the ladder and this arm is an
    // `if let`, so a miss is not a branch at all.
    } else if let Some(hit) = annot_hit {
        selection.select_annot(hit.clone());
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "annot-select page={} id={:?} kind={:?} subtype={} locked={} rect={:?}",
                hit.target.page,
                hit.target.id,
                hit.target.kind,
                hit.target.subtype,
                hit.target.locked,
                hit.outline,
            )
        });
    } else if super::textsel::takes_the_press(active_tool, caps) {
        if let (Some(page_text), Some(page)) = (doc.page_text(), doc.pages.get(page_index)) {
            // ★ The SAME options the extraction ran with, through the funnel —
            // `textsel::PageContext::opts` documents why a bare
            // `ExtractOptions::default()` here would be a defect rather than a
            // shortcut.
            let opts = crate::app::settings::SettingsExt::extract_options(&doc.settings);
            let text_ctx = super::textsel::PageContext {
                text: &page_text,
                page,
                index: page_index,
                epoch: doc.edit_epoch,
                opts: &opts,
            };
            *text_selection = super::textsel::click(
                &text_ctx,
                text_selection.as_ref(),
                point,
                shift,
                double,
                triple,
            );
            // `via=` names the gesture rather than the result, so a
            // harness can tell a double-click that happened to cover
            // one word from a sweep that did.
            let via = if triple {
                "line"
            } else if double {
                "word"
            } else if shift {
                "extend"
            } else {
                "clear"
            };
            super::trace::text_selection(page_index, text_selection.as_ref(), via);
        }
    // ★ A **vertex markup** click — PolyLine and Polygon, whose whole
    // gesture is clicks.
    //
    // It sits beside the measure branch rather than inside the markup
    // wiring further down, because the thing being routed is a *click*
    // and this is the arm that routes clicks. The two are mutually
    // exclusive by construction — one armed tool per frame — so the
    // order between them is a statement rather than a tie-break, and
    // `gesture::press_kind` has already given this press a live click
    // and no drag, so there is no state in which one press both places a
    // vertex and replaces the selection.
    //
    // Note what it does NOT need: a decomposition. A vertex lands where
    // the operator clicked and hit-tests nothing, which is why
    // `needs_targets` above grew no term for it — a polygon drawn over
    // the 129,758-object benchmark sheet decomposes nothing.
    } else if let Some(kind) = active_tool.markup_kind().filter(|k| k.is_vertex()) {
        super::markup::vertex::click(
            pen,
            ctx,
            kind,
            page_index,
            point,
            double,
            doc.current_page(),
            actions,
        );
    } else if let crate::canvas::tool::CanvasTool::Form(kind) = active_tool {
        // ★★ A CLICK places a form control at its conventional size.
        //
        // The operator, 2026-08-26: *"I should be able to click on the canvas
        // to place the position or drag a box for size"*. Both, and this is the
        // first half.
        //
        // Unlike the sticky note one arm below, the size here is a REAL promise
        // about what is drawn: a `/Widget`'s `/Rect` is its extent, not a
        // discarded hint, so a 14 pt square really is a 14 pt check box. That is
        // why the numbers live on the kind with their reasoning
        // (`FormFieldKind::default_size_pt`) rather than being one shared
        // constant — a check box and a text field are not the same shape, and
        // sizing them alike would make every click need a resize afterwards.
        //
        // The click point is the LOWER-LEFT corner rather than the centre. Both
        // are defensible; lower-left is chosen because it matches what the drag
        // does — the press is one corner and the control grows from it — so the
        // two gestures agree about what the pointer meant.
        if let Some(page) = doc.current_page()
            && let Some((at, _)) = super::markup::band::endpoints(point, point, page)
        {
            let (w, h) = kind.default_size_pt();
            actions.push(
                FieldAction::Begin {
                    page: page_index,
                    kind,
                    rect: pdfce_core::page_tree::Rect {
                        llx: at.0,
                        lly: at.1,
                        urx: at.0 + w,
                        ury: at.1 + h,
                    },
                }
                .into(),
            );
        }
    } else if let crate::canvas::tool::CanvasTool::TextAnnot(kind) = active_tool {
        // ★ The STICKY's whole placing gesture: one click, one point.
        //
        // The dragged kinds reach the dialog through
        // `GestureOutcome::TextAnnot` instead, so this arm is the
        // sticky's alone — but it is written for the family rather than
        // for the variant, and guarded by `is_dragged`, so a second
        // click-placed kind added later takes this path without an
        // edit and a kind that stops being click-placed leaves it.
        //
        // The rect is a small square around the point. A `/Text`
        // marker is fixed-size and `NoZoom` — the format discards the
        // rect's extent — so the size here is not a promise about what
        // is drawn; what matters is the LOWER-LEFT corner, which is
        // where the marker lands. `STICKY_PT` is documented at its
        // definition for exactly that reason.
        if !kind.is_dragged()
            && let Some(page) = doc.current_page()
            && let Some((at, _)) = super::markup::band::endpoints(point, point, page)
        {
            actions.push(Action::BeginTextAnnot {
                page: page_index,
                kind,
                rect: pdfce_core::page_tree::Rect {
                    llx: at.0,
                    lly: at.1,
                    urx: at.0 + crate::canvas::textannot::STICKY_PT,
                    ury: at.1 + crate::canvas::textannot::STICKY_PT,
                },
            });
        }
    } else if let Some(kind) = active_tool.measure_kind() {
        super::measure::click(
            super::measure::Pick {
                ctx,
                doc,
                page_index,
                kind,
                canvas_point: point,
                // ★ The double-click travels to the pick rather than
                // being re-read there. It is the radius/diameter tool's
                // **ending** — the gesture has no natural one, so the
                // operator supplies it — and it is carried on the same
                // value as the click it belongs to for the reason every
                // other field is: one click, one complete statement.
                double,
                targets: targets.map(|t| t as &dyn super::target::CanvasTargetProvider),
                map,
            },
            actions,
        );
    } else {
        // ★★★ `Alt`+click reaches PAST whatever is on top. See [`CycleCursor`].
        //
        // The depth is computed here rather than inside `probe`, because it is
        // a fact about this gesture — how many times the operator has asked, at
        // this point — and `probe` is a pure question about a point. Keeping
        // the cursor at the gesture end means the node-tool branch above, which
        // asks the same question for a different purpose, is unaffected: it
        // passes `0` and behaves exactly as it always has.
        let alt = ctx.input(|i| i.modifiers.alt);
        let depth = cycle_depth(ctx, point, alt);
        let hit = targets
            .map(|t| probe(t, selection, page_index, point, map, pick, depth))
            .unwrap_or_default();
        // ★ How many there were, so the status line can say *"2 of 5 here"*
        // rather than leaving the operator to discover a stack by cycling into
        // it. Computed only when something is under the pointer — the count is
        // a second walk of the same list and there is no reason to pay for it
        // on a click that hit nothing.
        let under = targets
            .filter(|_| hit.object.is_some())
            .map(|t| {
                crate::canvas::input::candidate_count(t, page_index, point, map.tolerance(), pick)
            })
            .unwrap_or(0);
        selection.click(page_index, hit, shift, double);
        // ★ Recorded WITH the object it is about, so it cannot be claimed for
        // a selection that arrived some other way — see `canvas::depth::taken`.
        if let Some(object) = hit.object {
            crate::canvas::depth::remember(ctx, depth, under, page_index, object);
        }
        super::trace::selection_event(selection, "click", double);
        if under > 1 {
            crate::diag::trace(move || {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("canvas-pick-depth depth={depth} of={under} alt={alt}")
            });
        }
    }
}
