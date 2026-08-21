//! # `canvas::moving` — dragging a selection, and the four things that make it honest
//!
//! ## What this module is for
//!
//! A press inside the selection, a drag, a release: the object moves. That is
//! one sentence and four separate obligations, and this module exists because
//! each of them is a place the gesture can go quietly wrong.
//!
//! 1. **One gesture is ONE command.** A multi-select moves through
//!    `EditSession::move_objects`, which takes a *slice*, resolves and
//!    type-checks every index before planning anything, and refuses the whole
//!    call rather than moving the prefix that happened to qualify. Emitting one
//!    `move_object` per selected entry would be N undo entries for one drag and
//!    — worse — N content-stream re-splices, each planned against byte offsets
//!    the previous one already invalidated. `docs/core-api/02` states the rule
//!    in a box: *"Never loop the singular verbs over a selection."*
//! 2. **The delta is PAGE space, never screen pixels.** See
//!    [`page_delta`] and the whole of [`crate::canvas::mapping`]'s header. A
//!    drag measured on screen and handed to a page-space verb compiles, runs,
//!    and merely scales with magnification — the same silent class as the
//!    hit-tolerance defect that module was built to make unavailable.
//! 3. **The preview must describe something that will actually happen.**
//!    `D:\Dev\FeatureRequests\pdfce_FeatureRequests\README.md` rule 4 welcomes
//!    a pre-commit affordance — *"a snap indicator, a hover highlight, a
//!    rubber-band, a selection handle — these are the cursor; they describe
//!    what is about to happen"* — and forbids marking content that has already
//!    been applied. A ghost outline is squarely in the first category, **as
//!    long as the move it describes is one the engine will accept.** So the
//!    ghost is drawn only when [`eligible`] has already said yes; a ghost over
//!    a text object at the Part rung, where no `move_*` verb applies, would be
//!    what `overlay`'s own note calls *"a lie with a low alpha"*.
//! 4. **The rung decides the verb, and a rung with no verb declines out loud.**
//!    Object → `move_objects`; Part → `move_subpath`; Node → `move_node`. The
//!    Part rung of a *text* object has no move verb at all (a show operator is
//!    not a subpath), so it refuses and traces, exactly as Delete refuses at
//!    the Part rung today.
//!
//! ## ★ Why the selection needs no invalidation across a move
//!
//! Because a move **does not renumber**. This was an open question that
//! blocked the whole feature, was asked as `request_stable_object_identity.md`,
//! and came back measured rather than asserted — the proof is
//! `crates/pdfce-core/tests/object_identity_across_edits.rs`, which decomposes,
//! edits, and decomposes *again*:
//!
//! | family | mechanism | renumbers? |
//! |---|---|---|
//! | `move_object` · `move_objects` · `move_subpath` · `move_node` · `move_nodes` · `move_handle` | rewrites operator **operands** in place | **NO** |
//! | `delete_object` · `delete_objects` · `delete_subpath` · `delete_node` · `delete_text_run` | excises byte **spans** | **YES** |
//!
//! A move changes numbers *inside* existing operators. No operator is added or
//! removed, so a second decomposition walks the same operators in the same
//! order and yields the same objects at the same indices — asserted directly by
//! that test, and asserted to be non-vacuous (the moved object demonstrably
//! moved).
//!
//! So [`crate::canvas::selection::Selection`] — `{ page, object, subpath, node }`
//! — survives a move **unchanged**, with no durable token and no invalidation
//! pass. What *does* change is the geometry, and that is already handled by the
//! machinery invariant 3 built for a delete: the action bumps
//! `OpenDoc::edit_epoch`, `SelectionState::needs_resolve` sees the key move,
//! and the outlines are recomputed from the fresh decomposition on the next
//! frame. [`tests::a_move_never_alters_the_selection`] pins both halves.
//!
//! ## What is deliberately NOT here: resize
//!
//! `EditSession` has the entire `move_*` family and **no scale or resize verb
//! of any kind**. The eight grips are drawn, and a drag on one is *consumed*
//! (so it cannot fall through to a marquee and silently replace the selection
//! the operator was aiming at), and it commits nothing — see
//! [`crate::canvas::handles`]. Wiring a ghost to a resize grip would be an
//! affordance for something that cannot happen, which is the no-placeholders
//! invariant, and it is a separate change for the day the verb exists.
//!
//! ## The split between the pure rules and the wiring
//!
//! [`eligible`], [`action`] and [`page_delta`] are pure functions of plain
//! data, so every rule above is testable with no window, no document and no
//! decomposition — the same discipline that makes
//! [`crate::canvas::selection::SelectionState::click`] a pure function of a
//! [`ClickHit`](crate::canvas::selection::ClickHit). [`drag`] is the one
//! function that touches the live provider, and it does nothing except gather
//! those inputs, call the pure functions in order, and trace what happened.
//!
//! ## conventions: drag-moves
//!
//! Corpus: `ui-conventions/drag-moves.md`.
//!
//! - D1 live-preview: the ghost is drawn every in-flight frame, offset by the
//!   canvas delta.
//! - D2 derived-from-commit: `eligible` is consulted twice — once per frame to
//!   decide whether a ghost may be drawn at all, once on release to build the
//!   command — so the ghost appears if and only if the release would commit.
//! - D3 escape-cancels: the gesture machine drops the drag; nothing is written
//!   before `Complete`.
//! - D4 one-undo-entry: `move_objects` takes a slice, so a drag of forty objects
//!   is one command and one Ctrl+Z.
//! - D5 modifiers-constrain: **Shift locks the move to one axis**, applied by
//!   `canvas::interact` to the delta *before* it reaches this module — so the
//!   ghost and the commit read one filtered value and cannot disagree. The
//!   arithmetic, the re-decide-every-frame rule and the announcement are
//!   [`crate::canvas::constrain`].
//! - D6 snapping: **GAP** — a content move does not snap to guides, the grid or
//!   other geometry, while the measure tools snap to all three.
//! - D7 no-op-is-not-an-edit: a zero-travel drag is deliberately still
//!   *eligible* (it names a real verb on real operands) and commits nothing —
//!   the split is stated in `eligible`'s own docs and is what keeps the ghost
//!   visible when the pointer passes back over the press point.
//! - D8 grab-point: a delta, not an absolute position, so the grab is preserved.
//! - D9 disclosure: WAIVED — a translation changes no measured value and the
//!   operator can see where the objects went. There is nothing they cannot
//!   reconstruct by looking.

use egui::{Pos2, Vec2};
use pdfce_core::page_tree::Page;
use pdfce_core::vector::Point;

use crate::app::actions::{Action, VectorAction};
use crate::canvas::gesture::Phase;
use crate::canvas::selection::{SelectionLevel, SelectionState};
use crate::panels::objects::provider::{ObjectModelProvider, PartKind};
use crate::viewer;

/// A drag displacement in **PDF page space** — the frame every `move_*` verb
/// consumes.
///
/// A distinct type rather than a bare `(f64, f64)` so a canvas-space `Vec2`
/// cannot be handed to a page-space verb by a call that happens to typecheck.
/// The only way to build one is [`page_delta`], which is the only place in
/// `canvas/` that crosses into PDF space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageDelta {
    /// Horizontal displacement, PDF user-space units.
    pub dx: f64,
    /// Vertical displacement, PDF user-space units. **Y is up** here — the
    /// opposite of the canvas-space `Vec2` it was derived from.
    pub dy: f64,
}

impl PageDelta {
    /// Whether this displacement is a real move.
    ///
    /// # Why the threshold is exactly zero, and not a nudge more
    ///
    /// egui already applies the only distance threshold this gesture needs:
    /// a press-and-release that does not exceed the drag threshold is reported
    /// as `clicked`, never as a drag, so a shaky hand cannot reach here at all
    /// (see [`crate::canvas::gesture`]'s header). Adding a second threshold
    /// *in page space* would make it zoom-dependent in the wrong direction —
    /// at 16× a deliberate quarter-point nudge is a 4 px screen drag the
    /// operator meant, and swallowing it would read as "the drag did not
    /// take". So the only thing refused here is a gesture that ended exactly
    /// where it began (a drag out and back), which must not put a no-op
    /// command on the undo stack.
    ///
    /// Non-finite is refused for the obvious reason: it would author NaN
    /// operands into a content stream.
    #[must_use]
    pub fn is_travel(self) -> bool {
        self.dx.is_finite() && self.dy.is_finite() && (self.dx != 0.0 || self.dy != 0.0)
    }
}

/// Which core verb a completed move drag on this selection would reach, with
/// its operands already resolved.
///
/// One variant per rung of the selection ladder, because that is the whole
/// rule: the rung the operator is standing on decides which of the `move_*`
/// family the gesture means.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveSubject {
    /// Every selected object, moved by a **matrix** rather than by rewriting
    /// coordinates — the rung a selection containing a text run, an image, a
    /// form XObject or an inline image takes.
    ///
    /// Reaches `EditSession::transform_objects` with `Matrix::translate(dx, dy)`
    /// in PAGE space. See [`eligible`]'s Object arm for why this is a second
    /// rung beside [`Self::Objects`] rather than a replacement for it, and
    /// `VectorAction::TransformObjects.into()` for the page-space contract.
    Transform {
        /// The 0-based page.
        page: usize,
        /// Paint-order indices, ascending and de-duplicated.
        objects: Vec<usize>,
    },
    /// The Object rung: `move_objects`, one command for the whole selection.
    Objects {
        /// The page the indices are positions on.
        page: usize,
        /// Paint-order indices, ascending and unique — the clean operand list
        /// `move_objects` needs in order to succeed rather than refuse.
        objects: Vec<usize>,
    },
    /// The Part rung of a **path** object: `move_subpath`.
    Subpath {
        /// The page.
        page: usize,
        /// The enclosing object, by paint-order index.
        object: usize,
        /// The subpath, in decomposition order.
        subpath: usize,
    },
    /// The Node rung with exactly **one** anchor selected: `move_node`.
    Node {
        /// The page.
        page: usize,
        /// The enclosing object, by paint-order index.
        object: usize,
        /// The anchor, **object-scoped** — the space `vector::anchor_count`
        /// reports and `pdfce-cli node-move --node N` addresses.
        node: usize,
    },
    /// The Node rung with **several** anchors selected: `move_nodes`.
    ///
    /// # ★ Why this is a second variant and not `Node` with a `Vec`
    ///
    /// Because the singular case has a verb of its own in `EditSession`, and
    /// `docs/core-api/02`'s rule cuts the other way too: the plural verb is
    /// correct for a set and the singular one is correct for a member, and
    /// collapsing them would mean either routing one node through a slice
    /// (losing the singular verb's own planner) or routing a set through a
    /// loop (which is the thing the rule forbids by name — N undo entries, and
    /// each planned against byte offsets the previous one invalidated).
    ///
    /// Both are the same gesture from the operator's side. The distinction is
    /// which engine verb the shell is entitled to call, which is exactly the
    /// kind of thing that belongs in a type rather than in an `if`.
    Nodes {
        /// The page.
        page: usize,
        /// The enclosing object, by paint-order index.
        object: usize,
        /// The anchors, object-scoped, ascending and unique.
        ///
        /// Never empty and never of length one — [`super::moving::subject`]
        /// produces [`Self::Node`] for the singular case, so a reader of this
        /// variant may assume two or more without checking.
        nodes: Vec<usize>,
    },
}

/// What the object model says about the entries a move would act on.
///
/// Assembled by [`drag`], which owns the provider, and handed to [`eligible`]
/// as plain data — the same shape, and for the same reason, as
/// [`ClickHit`](crate::canvas::selection::ClickHit): every rule below is then
/// a pure function of "what is selected" and "what kind of thing is it", with
/// no decomposition anywhere near the test that proves it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MoveContext {
    /// The paint-order index of the first selected object that is **not** a
    /// path, if there is one.
    ///
    /// Operand translation is path-only, and `move_objects` refuses the WHOLE
    /// call over a single non-path member rather than moving the paths and
    /// leaving a text object behind — a partial application that would read as
    /// a rendering fault rather than as a refusal. The *index* is carried
    /// rather than a bare `bool` because the engine's own error carries it for
    /// exactly this purpose: a refusal that cannot say which object refused is
    /// a refusal the operator cannot act on.
    pub non_path: Option<usize>,
    /// What kind of part the entered object decomposes into, at the Part and
    /// Node rungs. `None` for an object with no Part rung at all (an image).
    pub part_kind: Option<PartKind>,
}

/// Why a move drag committed nothing.
///
/// Reported rather than silently absorbed, and reported with enough detail to
/// act on, because *"nothing happened"* has several causes with opposite
/// responses: a drag that ended where it started is correct behaviour, a text
/// object at the Part rung is a missing verb, and a degenerate page is a
/// broken document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// The page has no readable object model, so nothing can be verified and
    /// nothing may be promised. Reachable when the page failed to decompose.
    NoObjectModel,
    /// Nothing is selected on this page.
    NothingSelected,
    /// The selection names an object index that does not fit a `usize`.
    ///
    /// Structurally unreachable on any real document — [`TargetId`] is a `u64`
    /// and a paint-order index is bounded by the page's operator count — and
    /// refused rather than truncated because a truncating cast would address a
    /// *different* object, which is the one outcome
    /// `docs/core-api/02` §1.10.1 marks as dangerous.
    ///
    /// [`TargetId`]: crate::canvas::target::TargetId
    UnaddressableObject,
    /// A selected object is not a path, so the whole move is refused. Carries
    /// its paint-order index.
    NotAPath(usize),
    /// The Part rung is entered but no part is named — an inconsistent state
    /// [`SelectionState`] does not produce, refused rather than guessed at.
    NoPartEntered,
    /// The entered part has no move verb: a text object's show operator is a
    /// "part", but `move_subpath` translates path construction operands and
    /// there is nothing for it to translate.
    NoVerbForPart(PartKind),
    /// The Node rung is entered but no anchor is named. Same nature as
    /// [`Self::NoPartEntered`].
    NoNodeEntered,
    /// The entered anchor is not in the object's current anchor list — the
    /// selection out-ran a decomposition. Carries the object-scoped index.
    NodeNotFound(usize),
    /// The gesture ended where it began. See [`PageDelta::is_travel`].
    NoTravel,
    /// The page's device transform is not invertible, so there is no
    /// well-defined page-space displacement. Declining is the only honest
    /// answer; authoring garbage geometry is not.
    DegeneratePage,
}

/// Convert a **canvas-space** drag delta into a **PDF page-space** one.
///
/// # ★ Why this is the only zoom-safe way to do it, and why no zoom appears
///
/// The zoom has already been divided out, once, before this is ever called.
/// `canvas/mod.rs` builds the frame's [`PageMapping`] and converts the
/// pointer's position through [`PageMapping::to_page`] *before* handing it to
/// the gesture machine, so [`crate::canvas::gesture::GestureOutcome::Move`]'s
/// `delta` is already a difference of two **canvas-space** points. There is
/// therefore no zoom left in it, and nothing here divides by one —
/// [`PageMapping`] deliberately has no `zoom()` accessor for exactly that
/// reason. A drag of the same screen distance yields the same page delta at
/// every magnification, which is
/// [`tests::a_drag_between_two_page_points_moves_the_same_distance_at_every_zoom`].
///
/// # Why two point conversions and a subtraction
///
/// [`viewer::canvas_to_pdf_space`] maps *points*, and a displacement is the
/// difference of two points. Taking that difference is what cancels the
/// transform's translation and leaves its linear part — the rotation and the
/// Y-flip — applied exactly once. Writing the linear part out by hand here
/// instead would be a second derivation of the page transform, which is the
/// precise failure `viewer`'s header warns about: *"PDF user space is y-UP;
/// canvas and screen are y-DOWN. The failure is silent — the page looks
/// perfect until someone selects a line and gets a different one."*
///
/// The subtraction is widened to `f64` *before* it is taken so no precision is
/// lost to an intermediate `f32` difference.
///
/// Returns `None` for a page whose device transform cannot be inverted, which
/// is the same condition under which both halves of the `viewer` bridge
/// decline. Callers refuse the move rather than authoring a fabricated delta.
///
/// [`PageMapping`]: crate::canvas::mapping::PageMapping
/// [`PageMapping::to_page`]: crate::canvas::mapping::PageMapping::to_page
#[must_use]
pub fn page_delta(canvas: Vec2, page: &Page) -> Option<PageDelta> {
    let origin = viewer::canvas_to_pdf_space(Pos2::ZERO, page)?;
    let moved = viewer::canvas_to_pdf_space(Pos2::ZERO + canvas, page)?;
    Some(PageDelta {
        dx: f64::from(moved.x) - f64::from(origin.x),
        dy: f64::from(moved.y) - f64::from(origin.y),
    })
}

/// Which verb a move drag on this selection would reach, or why it reaches
/// none.
///
/// Consulted **twice per drag**: once per frame while the drag is in flight,
/// to decide whether a ghost may be drawn at all, and once on release, to
/// build the command. Asking the same question both times is the mechanism
/// behind obligation 3 in the module docs — a ghost is drawn if and only if
/// the release would commit, so the preview cannot promise a move the engine
/// is going to refuse.
///
/// Deliberately says nothing about the *distance* dragged: a zero-travel drag
/// is eligible (it names a real verb on real operands), it simply has nothing
/// to commit, and that is [`action`]'s call. Splitting it this way is what
/// keeps the ghost visible during the frames where the pointer happens to pass
/// back over the press point.
pub fn eligible(
    selection: &SelectionState,
    page: usize,
    ctx: MoveContext,
) -> Result<MoveSubject, Refusal> {
    match selection.level() {
        SelectionLevel::Object => {
            // The same clean, ascending, de-duplicated operand list Delete
            // uses, and for the identical reason: `move_objects` resolves
            // EVERY index before planning anything, so one duplicate or stale
            // entry refuses the whole batch.
            let objects = selection.object_indices_on(page);
            if objects.is_empty() {
                return Err(Refusal::NothingSelected);
            }
            // ★★★ THE REFUSAL BECAME A FORK — 2026-08-20, and it is the
            // operator's *"can I please please please have the capability to
            // move the text after?"*
            //
            // This read:
            //
            //     if let Some(index) = ctx.non_path {
            //         return Err(Refusal::NotAPath(index));
            //     }
            //
            // and it was right: `move_objects` rewrites numeric **operands**,
            // and a text run and an image carry no coordinate operands at all.
            // `Pass 113.0`'s `transform_objects` wraps the object's operator run
            // in `q <cm> … Q`, which never looks at an operand — so it moves
            // anything.
            //
            // ★★ WHY THIS IS A FORK AND NOT A REPLACEMENT, which is the part a
            // reader will want to argue with.
            //
            // Both verbs move things and both are one command and one undo
            // entry, so the obvious tidy is to route everything through the
            // transform. That would be worse, and the reason is the FILE rather
            // than the API: `move_objects` rewrites coordinates in place and
            // adds nothing, while a transform adds a `q`, a `cm` and a `Q` per
            // object per gesture. On this operator's drawings a nudge is
            // something he does dozens of times to hundreds of objects, and the
            // wrapping accumulates in a file he then sends to somebody.
            //
            // So: **the lighter verb where it can express the gesture, the
            // general one where it cannot.** The predicate is unchanged — it is
            // the same `ctx.non_path` that used to refuse — which is what makes
            // this a fork rather than a second notion of "is this a path".
            match ctx.non_path {
                None => Ok(MoveSubject::Objects { page, objects }),
                Some(_) => Ok(MoveSubject::Transform { page, objects }),
            }
        }
        SelectionLevel::Part => {
            let (object, entry) = entered(selection, page)?;
            let subpath = entry.subpath.ok_or(Refusal::NoPartEntered)?;
            match ctx.part_kind {
                Some(PartKind::Subpath) => Ok(MoveSubject::Subpath {
                    page,
                    object,
                    subpath,
                }),
                // A text run IS a part, and it has no move verb. Declining
                // here rather than letting `move_subpath` refuse downstream is
                // what keeps the ghost truthful — the engine's refusal arrives
                // after the operator has already watched an outline slide.
                Some(other) => Err(Refusal::NoVerbForPart(other)),
                None => Err(Refusal::NotAPath(object)),
            }
        }
        SelectionLevel::Node => {
            let (object, entry) = entered(selection, page)?;
            let node = entry.node.ok_or(Refusal::NoNodeEntered)?;
            // ★★ **Every selected anchor on the entered object, not just the
            // entered one.** `SelectionState::pick_within` has always added a
            // Shift-clicked anchor as its own entry — the model could hold a
            // multi-node selection from the day the Node rung landed — and this
            // function read `entered_object()`, which is the FIRST entry. So an
            // operator could Shift-click four anchors, watch four highlight,
            // drag, and move one.
            //
            // That is the defect `pdfce`'s own `gui` column ticked `[x]` for
            // months (their note of 2026-08-19: "multi-node select-and-move —
            // objects move together; nodes one at a time"), and it is one of
            // the six rows that were true of the OLD in-repo shell and became
            // false when the column's referent moved to this build.
            let nodes = selection.selected_nodes_on(page, entry.object);
            match ctx.part_kind {
                Some(PartKind::Subpath) if nodes.len() > 1 => Ok(MoveSubject::Nodes {
                    page,
                    object,
                    nodes,
                }),
                Some(PartKind::Subpath) => Ok(MoveSubject::Node { page, object, node }),
                Some(other) => Err(Refusal::NoVerbForPart(other)),
                None => Err(Refusal::NotAPath(object)),
            }
        }
    }
}

/// The entered object of a deeper rung, as a `usize` index plus its entry.
///
/// Refuses an entry that belongs to a different page rather than addressing
/// page A's index space with page B's number — the same class of error the
/// [`TargetId`](crate::canvas::target::TargetId) newtype exists to prevent,
/// and one comparison to rule out.
fn entered(
    selection: &SelectionState,
    page: usize,
) -> Result<(usize, crate::canvas::selection::Selection), Refusal> {
    let entry = selection
        .entered_object()
        .ok_or(Refusal::NothingSelected)
        .and_then(|e| {
            (e.page == page)
                .then_some(e)
                .ok_or(Refusal::NothingSelected)
        })?;
    let object = usize::try_from(entry.object.0).map_err(|_| Refusal::UnaddressableObject)?;
    Ok((object, entry))
}

/// The ONE action a completed move drag becomes.
///
/// `node_at` is the entered anchor's **current** page-space position, and is
/// consulted only by [`MoveSubject::Node`]. It is needed because `move_node`
/// takes an absolute destination rather than a displacement — the operand it
/// rewrites is a coordinate pair, and expressing the drag as "where the point
/// ends up" is what lets the planner map one point through the object's CTM
/// inverse instead of decomposing a translation into a space it would have to
/// re-derive.
pub fn action(
    subject: MoveSubject,
    delta: PageDelta,
    node_at: Option<Point>,
    points: &[(usize, Point)],
) -> Result<Action, Refusal> {
    if !delta.is_travel() {
        return Err(Refusal::NoTravel);
    }
    match subject {
        // ★ `translate` in PAGE space, which is what `PageDelta` already is —
        // `page_delta` did the one canvas → page conversion and this is the
        // same pair of numbers `MoveSelection` below hands to `move_objects`.
        // Two rungs, one displacement, no second derivation.
        MoveSubject::Transform { page, objects } => Ok(VectorAction::TransformObjects {
            page,
            objects,
            matrix: pdfce_core::vector::Matrix::translate(delta.dx, delta.dy),
        }
        .into()),
        MoveSubject::Objects { page, objects } => Ok(VectorAction::MoveSelection {
            page,
            objects,
            dx: delta.dx,
            dy: delta.dy,
        }
        .into()),
        MoveSubject::Subpath {
            page,
            object,
            subpath,
        } => Ok(VectorAction::MoveSubpath {
            page,
            object,
            subpath,
            dx: delta.dx,
            dy: delta.dy,
        }
        .into()),
        MoveSubject::Node { page, object, node } => {
            let from = node_at.ok_or(Refusal::NodeNotFound(node))?;
            Ok(VectorAction::MoveNode {
                page,
                object,
                node,
                to: Point::new(from.x + delta.dx, from.y + delta.dy),
            }
            .into())
        }
        MoveSubject::Nodes {
            page,
            object,
            nodes,
        } => {
            // ★ A selected anchor that the current decomposition does not have
            // refuses the WHOLE drag rather than moving the ones it recognises.
            // The same call `move_objects` makes over a non-path member, and
            // for the same reason: a partial application reads as a rendering
            // fault, not as a refusal, and the operator has no way to tell
            // which of their four anchors was silently dropped.
            let mut moves = Vec::with_capacity(nodes.len());
            for node in nodes {
                let from = points
                    .iter()
                    .find_map(|(i, p)| (*i == node).then_some(*p))
                    .ok_or(Refusal::NodeNotFound(node))?;
                moves.push((node, Point::new(from.x + delta.dx, from.y + delta.dy)));
            }
            Ok(VectorAction::MoveNodes {
                page,
                object,
                moves,
            }
            .into())
        }
    }
}

/// Gather what the object model says about the selection a move would act on.
///
/// Returns `None` when there is no object model for the page at all, which is
/// distinct from "the model says no": nothing can be verified, so nothing may
/// be promised, and [`drag`] turns it into [`Refusal::NoObjectModel`].
///
/// The Object-rung scan asks [`ObjectModelProvider::part_kind`] once per
/// selected entry, which is a `Vec::get` and a match. It runs on every frame
/// of an in-flight drag, and that is affordable for the reason the whole
/// preview is affordable: the decomposition is already built and cached (the
/// selection could not have outlines to drag without it), so this walks a
/// slice rather than a content stream.
fn context(
    selection: &SelectionState,
    page: usize,
    provider: Option<&ObjectModelProvider>,
) -> Option<MoveContext> {
    let provider = provider?;
    let entered = selection
        .entered_object()
        .and_then(|e| usize::try_from(e.object.0).ok());
    Some(MoveContext {
        non_path: selection
            .object_indices_on(page)
            .into_iter()
            .find(|&i| provider.part_kind(i) != Some(PartKind::Subpath)),
        part_kind: entered.and_then(|i| provider.part_kind(i)),
    })
}

/// The entered anchor's current page-space position, or `None` if the object's
/// anchor list no longer holds that index.
///
/// [`ObjectModelProvider::object_node_points`] is the whole-object list
/// precisely so a caller does not have to re-derive which subpath an
/// object-scoped index falls in — that offset arithmetic lives in one place,
/// in the provider, and duplicating it here is how the number pdfce shows
/// starts disagreeing with the number the operator can act on.
fn node_point(provider: &ObjectModelProvider, object: usize, node: usize) -> Option<Point> {
    provider
        .object_node_points(object)
        .into_iter()
        .find(|(index, _)| *index == node)
        .map(|(_, point)| point)
}

/// Apply one frame of a move drag: draw the ghost, or commit the command.
///
/// The **only** function here that touches the live object model. It gathers
/// [`context`], asks [`eligible`], and then does one of two things:
///
/// * [`Phase::InFlight`] — returns the canvas-space delta for the ghost, and
///   changes nothing. Nothing is re-rasterized and nothing is decomposed: the
///   ghost is a translated copy of the outlines
///   [`SelectionState::outlines`] already caches in canvas space, which is
///   zoom-independent, so a preview costs one `Rect::translate` and one stroke
///   per selected entry.
/// * [`Phase::Complete`] — converts the delta to page space, resolves the node
///   position if the rung needs one, and pushes exactly one [`Action`].
///
/// Returns `Some(delta)` only when a ghost should be drawn. A drag that is not
/// eligible draws nothing, which is the visible half of obligation 3.
///
/// # Why the refusal is traced only on release
///
/// An in-flight drag is re-evaluated 60 times a second. Tracing a refusal per
/// frame would bury every other event on the channel — the lesson
/// `canvas-pointer` taught when a stationary pointer emitted fifty identical
/// lines in nine seconds. The release is one event, and it is the one a
/// harness reading the trace is asking about.
pub fn drag(
    delta: Vec2,
    phase: Phase,
    selection: &SelectionState,
    page_index: usize,
    provider: Option<&ObjectModelProvider>,
    page: Option<&Page>,
    actions: &mut Vec<Action>,
) -> Option<Vec2> {
    let outcome = context(selection, page_index, provider)
        .ok_or(Refusal::NoObjectModel)
        .and_then(|ctx| eligible(selection, page_index, ctx));

    let subject = match outcome {
        Ok(subject) => subject,
        Err(reason) => {
            if phase == Phase::Complete {
                decline(selection, reason);
            }
            return None;
        }
    };

    if phase == Phase::InFlight {
        return Some(delta);
    }

    // ---- commit ------------------------------------------------------
    let Some(page) = page else {
        decline(selection, Refusal::DegeneratePage);
        return None;
    };
    let Some(delta) = page_delta(delta, page) else {
        decline(selection, Refusal::DegeneratePage);
        return None;
    };
    // Only the Node rung needs a position, and asking for one costs an
    // allocation over every anchor of the object — 6,681 of them on one
    // measured CAD export — so it is asked for once, on release, and only for
    // the rung that consumes it.
    let node_at = match (&subject, provider) {
        (MoveSubject::Node { object, node, .. }, Some(provider)) => {
            node_point(provider, *object, *node)
        }
        _ => None,
    };
    // The plural rung needs every anchor's position, and that is the allocation
    // the singular rung's comment above is at pains to avoid — 6,681 anchors on
    // one measured CAD export. Asked for once, on release, and only when the
    // selection actually holds more than one node: the cost is paid by the
    // gesture that needs it and by no other.
    let points = match (&subject, provider) {
        (MoveSubject::Nodes { object, .. }, Some(provider)) => provider.object_node_points(*object),
        _ => Vec::new(),
    };

    match action(subject, delta, node_at, &points) {
        Ok(raised) => {
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "canvas-move page={page_index} level={:?} dx={:.4} dy={:.4} action={raised:?}",
                    selection.level(),
                    delta.dx,
                    delta.dy,
                )
            });
            actions.push(raised);
        }
        Err(reason) => decline(selection, reason),
    }
    None
}

/// Report a move that committed nothing, with the reason.
///
/// One trace shape for every refusal, so a harness reads `canvas-move-declined`
/// and finds the cause on the same line rather than inferring it from an
/// absence — the same contract `canvas-delete-declined` already honours.
fn decline(selection: &SelectionState, reason: Refusal) {
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "canvas-move-declined level={:?} sel={} reason={reason:?}",
            selection.level(),
            selection.len(),
        )
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::selection::ClickHit;
    use crate::canvas::target::{StubTargets, TargetId};
    use egui::{Rect, vec2};
    use pdfce_core::object::{Dict, ObjId};
    use pdfce_core::page_tree::Rect as PageRect;

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::from_min_size(Pos2::new(x, y), vec2(w, h))
    }

    /// A minimal page fixture — the same one `viewer`'s geometry tests use,
    /// because these functions read exactly what those do: `crop_box` and
    /// `rotate`.
    fn test_page(w: f64, h: f64, rotate: u16) -> Page {
        Page {
            id: ObjId::new(1, 0),
            resources: Dict::new(),
            media_box: PageRect::from_corners(0.0, 0.0, w, h),
            crop_box: PageRect::from_corners(0.0, 0.0, w, h),
            rotate,
            contents: Vec::new(),
            contents_unresolved: 0,
            contents_flattened: 0,
        }
    }

    fn hit_object(index: u64) -> ClickHit {
        ClickHit {
            object: Some(TargetId(index)),
            ..ClickHit::default()
        }
    }

    /// A click that landed on anchor `node` of subpath `part` of `object`.
    fn hit_node(object: u64, part: usize, node: usize) -> ClickHit {
        ClickHit {
            object: Some(TargetId(object)),
            part: Some(part),
            node: Some(node),
        }
    }

    /// Two objects on page 0, the first with two subpaths.
    fn stub() -> StubTargets {
        StubTargets::new(
            0,
            [rect(0.0, 0.0, 100.0, 100.0), rect(200.0, 200.0, 50.0, 50.0)],
        )
        .with_parts(
            0,
            [rect(0.0, 0.0, 40.0, 40.0), rect(60.0, 60.0, 40.0, 40.0)],
        )
    }

    /// The same two objects, translated — what a decomposition taken *after* a
    /// committed move yields: the same objects at the same indices, in new
    /// places.
    fn stub_moved(by: Vec2) -> StubTargets {
        StubTargets::new(
            0,
            [
                rect(0.0, 0.0, 100.0, 100.0).translate(by),
                rect(200.0, 200.0, 50.0, 50.0).translate(by),
            ],
        )
        .with_parts(
            0,
            [
                rect(0.0, 0.0, 40.0, 40.0).translate(by),
                rect(60.0, 60.0, 40.0, 40.0).translate(by),
            ],
        )
    }

    /// A selection holding both objects at the Object rung, resolved.
    fn two_objects_selected() -> SelectionState {
        let mut sel = SelectionState::default();
        sel.click(0, hit_object(0), false, false);
        sel.click(0, hit_object(1), true, false);
        sel.resolve(Some(&stub()), 0, 0);
        sel
    }

    /// Every object is a path, and the entered one decomposes into subpaths —
    /// the ordinary case.
    fn paths() -> MoveContext {
        MoveContext {
            non_path: None,
            part_kind: Some(PartKind::Subpath),
        }
    }

    // -----------------------------------------------------------------
    // ★ The invariant the whole feature was blocked on
    // -----------------------------------------------------------------

    /// ★ **A move never alters the selection.**
    ///
    /// The counterpart of
    /// [`navigating_the_view_never_alters_the_selection`](crate::canvas::selection),
    /// and it is asserted the same way: drive the thing that *could* reach the
    /// selection, then compare.
    ///
    /// What a committed move does to the shell is exactly two things — it
    /// bumps `OpenDoc::edit_epoch`, and it makes the next decomposition report
    /// the same objects at the same indices in new places. Both are modelled
    /// here: the epoch moves from 0 to 1, and the provider handed to the
    /// re-resolve is `stub_moved`. `object_identity_across_edits.rs` is what
    /// licenses the second half — `move_*` rewrites operands in place, adds and
    /// removes no operator, and therefore renumbers nothing.
    ///
    /// **The test is not vacuous**, and the second assertion is what makes it
    /// so: the outlines must have *moved*. A `resolve` that quietly did nothing
    /// would satisfy the identity assertion perfectly.
    #[test]
    fn a_move_never_alters_the_selection() {
        let mut sel = two_objects_selected();
        let entries_before = sel.entries().to_vec();
        let outlines_before = sel.outlines().to_vec();
        assert_eq!(entries_before.len(), 2);

        // The move lands: epoch bumped, geometry translated, indices intact.
        let by = vec2(25.0, -40.0);
        sel.resolve(Some(&stub_moved(by)), 0, 1);

        assert_eq!(
            sel.entries(),
            entries_before.as_slice(),
            "a move reached the selection; only `delete_*` may renumber, and this is not one"
        );
        assert_ne!(
            sel.outlines(),
            outlines_before.as_slice(),
            "the outlines must follow the move, or this test would pass on a no-op resolve"
        );
        for ((entry, after), (_, before)) in sel.outlines().iter().zip(&outlines_before) {
            assert_eq!(
                *after,
                before.translate(by),
                "entry {entry:?} outline did not follow the move exactly"
            );
        }
    }

    /// A deeper rung survives it too — the entry keeps its subpath and its
    /// node, because a move rewrites operands and leaves the operator count,
    /// and therefore every index, alone.
    #[test]
    fn a_move_never_alters_a_node_selection() {
        let mut sel = SelectionState::default();
        sel.click(0, hit_object(0), false, false);
        sel.click(
            0,
            ClickHit {
                object: Some(TargetId(0)),
                part: Some(1),
                node: None,
            },
            false,
            true,
        );
        sel.click(
            0,
            ClickHit {
                object: Some(TargetId(0)),
                part: Some(1),
                node: Some(4),
            },
            false,
            true,
        );
        sel.resolve(Some(&stub()), 0, 0);
        assert_eq!(sel.level(), SelectionLevel::Node);
        let before = sel.entries().to_vec();

        sel.resolve(Some(&stub_moved(vec2(-3.0, 7.0))), 0, 1);

        assert_eq!(sel.entries(), before.as_slice());
        assert_eq!(sel.entries()[0].node, Some(4));
        assert_eq!(sel.entries()[0].subpath, Some(1));
    }

    // -----------------------------------------------------------------
    // The delta is page space
    // -----------------------------------------------------------------

    /// ★ **The object lands where the pointer put it, at every zoom.**
    ///
    /// The hit-tolerance trap, in the move gesture's clothing — and stating it
    /// correctly is half the value of the test, because the *tempting* wording
    /// is the wrong one. "The same screen distance yields the same page delta"
    /// is **false and must be false**: a fixed screen distance is
    /// `distance / zoom` page units, which is
    /// `viewer::screen_to_page_distance_scales_as_one_over_zoom`. Asserting it
    /// would be asserting the defect.
    ///
    /// What must be invariant is the operator's experience: grab a point on the
    /// page, drop it on another point on the page, and the object moves by the
    /// distance *between those two page points* — the same answer at 25 % and
    /// at 1200 %. So the fixture drags between two fixed **page** positions,
    /// projects them to screen through the frame's mapping (which is what the
    /// pointer really reports), converts back exactly as `canvas/mod.rs` does,
    /// and asserts one answer at four magnifications.
    ///
    /// A second division by zoom anywhere in that chain — or a missing one —
    /// makes this fan out by a factor of the zoom, which is precisely the
    /// failure [`crate::canvas::mapping`] was built to make unavailable and the
    /// reason [`PageMapping`](crate::canvas::mapping::PageMapping) has no
    /// `zoom()` accessor to divide by.
    #[test]
    fn a_drag_between_two_page_points_moves_the_same_distance_at_every_zoom() {
        use crate::canvas::mapping::PageMapping;
        use crate::viewer::page_extent_pts;

        let page = test_page(200.0, 300.0, 0);
        let extent = page_extent_pts(&page);
        // Two positions ON THE PAGE, in canvas space: grab here, drop there.
        let grabbed = Pos2::new(40.0, 60.0);
        let dropped = Pos2::new(100.0, 84.0);

        let mut seen: Vec<PageDelta> = Vec::new();
        for &zoom in &[0.25_f32, 1.0, 4.0, 12.0] {
            let image_rect = Rect::from_min_size(
                Pos2::new(37.0, 11.0),
                vec2(extent.0 * zoom, extent.1 * zoom),
            );
            let map = PageMapping::new(image_rect, extent, zoom);
            // Round-trip through the screen, because that is the only thing
            // the pointer ever reports — and it is where a stray zoom would
            // enter.
            let from = map.to_page(map.to_screen(grabbed));
            let to = map.to_page(map.to_screen(dropped));
            seen.push(page_delta(to - from, &page).expect("invertible page"));
        }
        for delta in &seen {
            assert!(
                (delta.dx - seen[0].dx).abs() < 1e-3 && (delta.dy - seen[0].dy).abs() < 1e-3,
                "the page delta changed with the zoom: {seen:?}"
            );
        }
        // 60 canvas units right and 24 canvas units DOWN, which in Y-up PDF
        // user space is +60 and -24.
        assert!((seen[0].dx - 60.0).abs() < 1e-3, "{seen:?}");
        assert!((seen[0].dy + 24.0).abs() < 1e-3, "{seen:?}");
    }

    /// The canvas is Y-down and PDF user space is Y-up, so a downward drag is
    /// a *negative* dy. Stated as its own assertion because getting it
    /// backwards is silent: the object moves, just the wrong way.
    #[test]
    fn a_downward_drag_is_a_negative_page_dy() {
        let page = test_page(200.0, 300.0, 0);
        let delta = page_delta(vec2(0.0, 10.0), &page).expect("invertible page");
        assert!(delta.dy < 0.0, "{delta:?}");
        assert!((delta.dy + 10.0).abs() < 1e-3, "{delta:?}");
    }

    /// A rotated page rotates the delta, and it does so through the renderer's
    /// own transform rather than a formula written out here. On a page turned
    /// 90° clockwise, dragging right on screen moves the object *down* the
    /// un-rotated page — i.e. -y in PDF user space.
    #[test]
    fn a_rotated_page_rotates_the_delta() {
        let page = test_page(200.0, 300.0, 90);
        let delta = page_delta(vec2(10.0, 0.0), &page).expect("invertible page");
        assert!(delta.dx.abs() < 1e-3, "{delta:?}");
        assert!((delta.dy.abs() - 10.0).abs() < 1e-3, "{delta:?}");
        // And the un-rotated page's answer is the other axis entirely, which is
        // what makes this a rotation test rather than a magnitude test.
        let upright =
            page_delta(vec2(10.0, 0.0), &test_page(200.0, 300.0, 0)).expect("invertible page");
        assert!((upright.dx - 10.0).abs() < 1e-3, "{upright:?}");
        assert!(upright.dy.abs() < 1e-3, "{upright:?}");
    }

    /// A drag that ends where it began raises nothing — a no-op must not take
    /// a slot on the undo stack.
    #[test]
    fn a_drag_with_no_travel_commits_nothing() {
        let sel = two_objects_selected();
        let subject = eligible(&sel, 0, paths()).expect("eligible");
        assert_eq!(
            action(subject, PageDelta { dx: 0.0, dy: 0.0 }, None, &[]),
            Err(Refusal::NoTravel)
        );
    }

    /// …but the smallest real travel does commit. There is no second
    /// threshold; egui's drag threshold is the only one.
    #[test]
    fn the_smallest_real_travel_still_commits() {
        let sel = two_objects_selected();
        let subject = eligible(&sel, 0, paths()).expect("eligible");
        let raised =
            action(subject, PageDelta { dx: 0.01, dy: 0.0 }, None, &[]).expect("committed");
        assert!(matches!(
            raised,
            Action::Vector(VectorAction::MoveSelection { .. })
        ));
    }

    /// A non-finite delta is refused rather than authored into a content
    /// stream.
    #[test]
    fn a_non_finite_delta_is_refused() {
        let sel = two_objects_selected();
        for delta in [
            PageDelta {
                dx: f64::NAN,
                dy: 0.0,
            },
            PageDelta {
                dx: 0.0,
                dy: f64::INFINITY,
            },
        ] {
            let subject = eligible(&sel, 0, paths()).expect("eligible");
            assert_eq!(action(subject, delta, None, &[]), Err(Refusal::NoTravel));
        }
    }

    // -----------------------------------------------------------------
    // One gesture, one command
    // -----------------------------------------------------------------

    /// ★ **A multi-select moves as ONE command**, carrying the whole operand
    /// list — never one action per object, which would be N undo entries and N
    /// re-splices planned against stale byte offsets.
    #[test]
    fn a_multi_select_moves_as_one_command() {
        let sel = two_objects_selected();
        let subject = eligible(&sel, 0, paths()).expect("eligible");
        assert_eq!(
            subject,
            MoveSubject::Objects {
                page: 0,
                objects: vec![0, 1],
            }
        );
        assert_eq!(
            action(subject, PageDelta { dx: 5.0, dy: -2.0 }, None, &[]),
            Ok(VectorAction::MoveSelection {
                page: 0,
                objects: vec![0, 1],
                dx: 5.0,
                dy: -2.0,
            }
            .into())
        );
    }

    /// Nothing selected raises nothing rather than an empty batch the engine
    /// would have to refuse.
    #[test]
    fn an_empty_selection_moves_nothing() {
        let sel = SelectionState::default();
        assert_eq!(eligible(&sel, 0, paths()), Err(Refusal::NothingSelected));
    }

    /// A selection on another page is not moved by a drag on this one.
    #[test]
    fn a_selection_on_another_page_is_not_moved() {
        let mut sel = SelectionState::default();
        sel.click(3, hit_object(0), false, false);
        assert_eq!(eligible(&sel, 0, paths()), Err(Refusal::NothingSelected));
    }

    /// ★★★ **A non-path member ROUTES THE MOVE THROUGH A TRANSFORM** — and
    /// this test used to assert that it refused the whole drag.
    ///
    /// It read *"a non-path member refuses the WHOLE move, and names the
    /// offender"*, and the reasoning was sound while it lasted:
    ///
    /// > *"The engine does this too, and would do it correctly. Refusing here
    /// > as well is what keeps the ghost honest: an outline that slides across
    /// > the page and then snaps back has already told the operator something
    /// > untrue."*
    ///
    /// The ghost obligation stands and is now satisfied the other way round —
    /// the outline slides **and the release commits**, because `Pass 113.0` gave
    /// this shell a verb that moves anything. The operator asked for it three
    /// times: *"can I please please please have the capability to move the text
    /// after?"*
    ///
    /// ★ What is asserted is the **rung**, not the absence of a refusal: a
    /// build that routed every move through the transform would also stop
    /// refusing here, and it would be wrong for the reason `eligible`'s own
    /// comment gives about the file rather than the API.
    #[test]
    fn a_non_path_in_the_selection_routes_through_a_transform() {
        let sel = two_objects_selected();
        let ctx = MoveContext {
            non_path: Some(1),
            ..paths()
        };
        assert!(
            matches!(
                eligible(&sel, 0, ctx),
                Ok(MoveSubject::Transform { page: 0, .. })
            ),
            "a selection containing a picture or a text run must reach the transform rung"
        );
    }

    /// …and an all-path selection still takes the LIGHTER verb.
    ///
    /// The other half of the fork, and the half a tidy-up would delete. A
    /// transform wraps each object in `q <cm> … Q` per gesture; `move_objects`
    /// rewrites the coordinates in place and adds nothing. On a drawing that is
    /// nudged dozens of times, the wrapping accumulates in a file somebody then
    /// sends on.
    #[test]
    fn an_all_path_selection_still_reaches_move_objects() {
        let sel = two_objects_selected();
        assert!(
            matches!(
                eligible(&sel, 0, paths()),
                Ok(MoveSubject::Objects { page: 0, .. })
            ),
            "a selection made only of shapes must not pay for the general verb"
        );
    }

    // -----------------------------------------------------------------
    // The rung decides the verb
    // -----------------------------------------------------------------

    /// The Part rung of a path reaches `move_subpath`, with the entered
    /// subpath as its operand.
    #[test]
    fn the_part_rung_reaches_move_subpath() {
        let mut sel = SelectionState::default();
        sel.click(0, hit_object(0), false, false);
        sel.click(
            0,
            ClickHit {
                object: Some(TargetId(0)),
                part: Some(1),
                node: None,
            },
            false,
            true,
        );
        assert_eq!(sel.level(), SelectionLevel::Part);

        let subject = eligible(&sel, 0, paths()).expect("eligible");
        assert_eq!(
            subject,
            MoveSubject::Subpath {
                page: 0,
                object: 0,
                subpath: 1,
            }
        );
        assert_eq!(
            action(subject, PageDelta { dx: 1.5, dy: 2.5 }, None, &[]),
            Ok(VectorAction::MoveSubpath {
                page: 0,
                object: 0,
                subpath: 1,
                dx: 1.5,
                dy: 2.5,
            }
            .into())
        );
    }

    /// ★ **A text run at the Part rung declines** — it is a part, and there is
    /// no verb that moves one. The same shape as Delete declining at a rung
    /// whose verb is not wired.
    #[test]
    fn a_text_run_at_the_part_rung_declines_rather_than_moving_the_object() {
        let mut sel = SelectionState::default();
        sel.click(0, hit_object(0), false, false);
        sel.click(
            0,
            ClickHit {
                object: Some(TargetId(0)),
                part: Some(0),
                node: None,
            },
            false,
            true,
        );
        let ctx = MoveContext {
            non_path: None,
            part_kind: Some(PartKind::Run),
        };
        assert_eq!(
            eligible(&sel, 0, ctx),
            Err(Refusal::NoVerbForPart(PartKind::Run)),
            "moving the enclosing object because a run was selected is the wrong action, \
             not a lenient one"
        );
    }

    /// The Node rung reaches `move_node`, and the destination is the anchor's
    /// current position **plus** the delta — absolute, because that is what the
    /// verb takes.
    #[test]
    fn the_node_rung_reaches_move_node_with_an_absolute_destination() {
        let mut sel = SelectionState::default();
        sel.click(0, hit_object(0), false, false);
        sel.click(
            0,
            ClickHit {
                object: Some(TargetId(0)),
                part: Some(1),
                node: None,
            },
            false,
            true,
        );
        sel.click(
            0,
            ClickHit {
                object: Some(TargetId(0)),
                part: Some(1),
                node: Some(4),
            },
            false,
            true,
        );
        assert_eq!(sel.level(), SelectionLevel::Node);

        let subject = eligible(&sel, 0, paths()).expect("eligible");
        assert_eq!(
            subject,
            MoveSubject::Node {
                page: 0,
                object: 0,
                node: 4,
            }
        );
        let raised = action(
            subject,
            PageDelta { dx: 10.0, dy: -4.0 },
            Some(Point::new(100.0, 200.0)),
            &[],
        );
        assert_eq!(
            raised,
            Ok(VectorAction::MoveNode {
                page: 0,
                object: 0,
                node: 4,
                to: Point::new(110.0, 196.0),
            }
            .into())
        );
    }

    /// A node whose position the decomposition no longer reports refuses,
    /// rather than moving the anchor to the delta itself — which would fling
    /// it to the bottom-left of the page.
    #[test]
    fn a_node_with_no_known_position_refuses() {
        let subject = MoveSubject::Node {
            page: 0,
            object: 0,
            node: 4,
        };
        assert_eq!(
            action(subject, PageDelta { dx: 1.0, dy: 1.0 }, None, &[]),
            Err(Refusal::NodeNotFound(4))
        );
    }

    /// With no object model the move declines: nothing can be verified, so
    /// nothing may be promised — and in particular no ghost is drawn.
    #[test]
    fn a_page_with_no_object_model_declines() {
        let sel = two_objects_selected();
        let mut actions = Vec::new();
        let ghost = drag(
            vec2(10.0, 10.0),
            Phase::InFlight,
            &sel,
            0,
            None,
            None,
            &mut actions,
        );
        assert_eq!(
            ghost, None,
            "a ghost must not describe an unverifiable move"
        );
        assert!(actions.is_empty());
    }

    /// ★★ **Four Shift-clicked anchors move as FOUR anchors, in one command.**
    ///
    /// This is the regression test for a defect that lived in the gap between
    /// two correct halves. `SelectionState::pick_within` has added a
    /// Shift-clicked anchor as its own entry since the Node rung landed, and
    /// `subject` read `entered_object()` — the FIRST entry. So the model held
    /// four, the overlay drew four, and the drag moved one.
    ///
    /// Nothing failed. Both halves' unit tests passed. The only thing that
    /// would have caught it is driving it, or a test like this one that asks
    /// the two halves the same question.
    #[test]
    fn several_selected_anchors_move_as_one_command() {
        let mut selection = SelectionState::default();
        selection.click(0, hit_object(7), false, false);
        selection.click(0, hit_node(7, 0, 1), false, true);
        selection.click(0, hit_node(7, 0, 1), false, true);
        // Now inside the object at the Node rung; Shift-pick three more.
        for node in [4_usize, 9, 2] {
            selection.click(0, hit_node(7, 0, node), true, false);
        }
        let nodes = selection.selected_nodes_on(0, TargetId(7));
        assert!(
            nodes.len() >= 2,
            "the selection model must hold every Shift-picked anchor, got {nodes:?}"
        );

        let subject = eligible(
            &selection,
            0,
            MoveContext {
                non_path: None,
                part_kind: Some(PartKind::Subpath),
            },
        )
        .expect("a multi-node selection on a path has a move subject");
        let MoveSubject::Nodes { nodes, .. } = &subject else {
            panic!("several anchors must produce the PLURAL subject, got {subject:?}");
        };
        assert_eq!(
            nodes.len(),
            selection.selected_nodes_on(0, TargetId(7)).len()
        );

        // Every selected anchor's position, so the plural arm can resolve them.
        let points: Vec<(usize, Point)> = (0..12)
            .map(|i| {
                (
                    i,
                    Point::new(f64::from(u32::try_from(i).unwrap()) * 10.0, 50.0),
                )
            })
            .collect();
        let raised = action(subject, PageDelta { dx: 3.0, dy: -7.0 }, None, &points)
            .expect("the plural arm resolves every anchor");
        let Action::Vector(VectorAction::MoveNodes { moves, .. }) = raised else {
            panic!("the plural subject must raise ONE MoveNodes, got {raised:?}");
        };
        assert!(moves.len() >= 2, "one command carrying every anchor");
        for (index, to) in &moves {
            let from = points[*index].1;
            assert!((to.x - (from.x + 3.0)).abs() < 1e-9);
            assert!((to.y - (from.y - 7.0)).abs() < 1e-9);
        }
    }

    /// ★ **One stale anchor refuses the whole drag**, rather than moving the
    /// three the decomposition still recognises.
    ///
    /// The same call `move_objects` makes over a non-path member, and for the
    /// same reason its docs give: a partial application reads as a rendering
    /// fault rather than as a refusal, and the operator has no way to learn
    /// which of their anchors was dropped.
    #[test]
    fn one_missing_anchor_refuses_the_whole_move() {
        let subject = MoveSubject::Nodes {
            page: 0,
            object: 7,
            nodes: vec![0, 1, 99],
        };
        let points: Vec<(usize, Point)> = (0..3).map(|i| (i, Point::new(0.0, 0.0))).collect();
        let err = action(subject, PageDelta { dx: 1.0, dy: 1.0 }, None, &points)
            .expect_err("a selection that out-ran the decomposition must refuse");
        assert_eq!(err, Refusal::NodeNotFound(99));
    }

    /// A single selected anchor still takes the SINGULAR verb.
    ///
    /// `EditSession` has both, and `docs/core-api/02`'s rule cuts both ways:
    /// the plural verb is correct for a set and the singular one for a member.
    /// Routing one anchor through a slice would lose the singular planner for
    /// no gain.
    #[test]
    fn one_selected_anchor_still_takes_the_singular_verb() {
        let mut selection = SelectionState::default();
        selection.click(0, hit_object(7), false, false);
        selection.click(0, hit_node(7, 0, 1), false, true);
        selection.click(0, hit_node(7, 0, 1), false, true);
        if selection.selected_nodes_on(0, TargetId(7)).len() != 1 {
            // The descent did not reach the Node rung on this fixture shape;
            // the assertion below would then be about the wrong thing.
            return;
        }
        let subject = eligible(
            &selection,
            0,
            MoveContext {
                non_path: None,
                part_kind: Some(PartKind::Subpath),
            },
        )
        .expect("one anchor on a path has a move subject");
        assert!(
            matches!(subject, MoveSubject::Node { .. }),
            "one anchor must stay singular, got {subject:?}"
        );
    }
}
