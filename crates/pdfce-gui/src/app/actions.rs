//! # `app::actions` — the one channel through which anything changes
//!
//! ## The invariant this module exists to enforce
//!
//! **No code path runs from a widget to a document.** A widget that is
//! clicked, a key that is pressed, a wheel that is spun — none of them
//! change anything directly. Each produces an [`Action`], the actions are
//! collected while the frame is drawn, and they are applied *after* it, in
//! one place, by [`PdfceApp::apply`].
//!
//! `SALVAGE.md` calls this *"the single best structural decision in the old
//! GUI"*, and `PROJECT_PLAN.md` §3 lists it first among the invariants that
//! are "not up for renegotiation". It is established here at stage S0, with
//! a handful of actions and one widget, **because retrofitting it is
//! expensive**: every widget written under the other discipline has to be
//! found and rewritten, and the ones that are missed are exactly the ones
//! that produce an incoherent undo log later.
//!
//! ## Why it is worth the indirection
//!
//! Four things fall out of it, none of which can be had cheaply otherwise:
//!
//! 1. **A coherent undo log.** One operator gesture becomes one action
//!    becomes one command-log entry. A widget that mutated in place would
//!    have to remember to log, and the ones that forgot would be invisible
//!    holes in the history. (S0 has no undo — but S4's undo is only
//!    possible because the funnel already exists.)
//! 2. **The borrow checker stops fighting.** egui is immediate-mode: the
//!    document is being *read* to draw the very widget that wants to change
//!    it. Deferring the change to after the frame turns an aliasing problem
//!    into a queue.
//! 3. **Order becomes explicit.** Two actions raised in one frame are
//!    applied in a defined order, in one readable function, rather than in
//!    whatever order the layout code happened to run.
//! 4. **Every state change is greppable.** "What can change the zoom?" has
//!    a complete answer: the [`Action`] variants that touch it.
//!
//! ## Scope
//!
//! Zoom, page navigation, and — from stage S4 — **the actions that change the
//! document**: [`Action::DeleteSelection`] and the three move verbs
//! ([`Action::MoveSelection`], [`Action::MoveSubpath`], [`Action::MoveNode`]).
//! Those are why this module has a mutation path, and the path is short and
//! deliberately in one place; see [`vector_edit`], which every one of them goes
//! through so the cancel-mutate-bump-invalidate protocol is written once rather
//! than four times.
//!
//! **There is no resize action, and its absence is deliberate.**
//! `EditSession` has the whole `move_*` family and no scale verb of any kind,
//! so a `ResizeSelection` here would be an enum variant nothing could honour —
//! which is the same "no placeholders" rule this enum's own doc comment states
//! two paragraphs down. The canvas still *consumes* a grip drag, so it cannot
//! fall through to a marquee, and commits nothing; see
//! [`crate::canvas::handles`].
//!
//! ## ★ Opening and closing are actions now, and this header said they would be
//!
//! Until `file.open` existed, this paragraph read: *"Opening a document is
//! deliberately not an action at S0: it happens once, from `argv`, before the
//! event loop starts. It becomes one the moment there is an Open command, and
//! the `apply` gate that blocks an open while a save is pending lands with
//! it."*
//!
//! Both halves have now happened. [`Action::Open`] and [`Action::Close`] are
//! the first two variants that are about **which document is open** rather
//! than about the one that already is, and they are the reason [`PdfceApp::apply`]
//! no longer begins by refusing everything when nothing is open — see the
//! ★ comment at the top of its body, which is the whole of why the two are
//! matched *before* that guard rather than inside it.
//!
//! The gate is [`PdfceApp::save_pending`], consulted by both, and its own doc
//! comment carries the rule. It answers `false` in this build because there
//! is no save path at all — so no confirmation dialog is built for a
//! condition that cannot occur, and the rule has one home rather than being
//! rediscovered by whoever adds the save.
//!
//! **The dialog is not the action.** `file.open` opens a native file picker,
//! which is a UI act that happens during dispatch; what goes through the
//! funnel is its *result*, a path. See [`crate::app::files`] for the picker,
//! the diagnostics seam that lets a scripted harness answer it without a
//! human, and why the two are separated at exactly that line.
//!
//! ## ★ `Action` is no longer `Copy`, and that is a decision rather than an accident
//!
//! [`Action::DeleteSelection`] carries a `Vec<usize>` — the paint-order
//! indices to remove — and it has to, for a reason that is not about
//! convenience:
//!
//! `EditSession::delete_objects` takes a **slice** and resolves every index
//! before planning anything, *"so an out-of-range one refuses the call rather
//! than deleting the prefix that happened to resolve"*. Deleting a
//! multi-selection therefore has to be **one** command. Emitting one
//! `DeleteObject` action per selected object would renumber the page between
//! them — deleting object 5 and then object 3 deletes 5 and then whatever
//! moved into slot 3 — so the batch cannot be decomposed.
//!
//! The alternative to carrying the list would be for `apply` to read the
//! selection itself. It cannot: the selection lives in the canvas, `apply`
//! has no `egui::Context`, and giving it one would make the action funnel
//! depend on the UI framework. Carrying the operands is also simply what an
//! action *is* — a complete statement of an operator's intent, resolvable
//! after the frame that raised it.

use std::sync::Arc;

use pdfce_core::edit::{EditError, EditSession};

use crate::app::PdfceApp;
use crate::app::state::{OpenDoc, Status};
use crate::viewer::{self, FitMode};

/// One operator intent, applied after the frame that raised it.
///
/// Every variant is reachable from a real control today. A variant nothing
/// can raise is dead code wearing a design pattern, and the "no
/// placeholders" invariant (`PROJECT_PLAN.md` §3) applies to enums as much
/// as to labels.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// **Open this document, replacing whatever is open.**
    ///
    /// Raised by `file.open` once the picker has answered, by `file.recent`
    /// once the operator has chosen a row, and by nothing else — the two
    /// surfaces that can name a file. It is deliberately *not* raised by
    /// `crate::run`'s `argv` path, which calls
    /// [`PdfceApp::open_path`] directly: there is no frame yet, so there is
    /// no frame to defer to, and routing it through an empty action queue
    /// would be ceremony rather than discipline.
    ///
    /// # Why the path travels, and why it is already absolute by the time
    /// anyone stores it
    ///
    /// Same reason [`Self::DeleteSelection`] carries its operand list: an
    /// action is a complete statement of intent, resolvable after the frame
    /// that raised it. The picker's answer cannot be re-derived later —
    /// the dialog is gone — so the only alternative would be a field on the
    /// application holding a half-finished intent between frames, which is
    /// precisely the state the funnel exists to avoid.
    ///
    /// Absolutizing happens in [`crate::app::recent::RecentFiles::remember`]
    /// rather than here, because it is a property of *storing* a path, not of
    /// opening one: `Document::load` is perfectly happy with a relative path
    /// and the operator's shell already resolved it.
    Open(std::path::PathBuf),
    /// **Close the open document and return to [`Status::Empty`].**
    ///
    /// Raised by `file.close`, which is gated on `doc.open`, so the no-document
    /// case is unreachable from the ribbon — and handled anyway, because a
    /// customized keymap can reach any command from any state and an action
    /// that assumed otherwise would be a panic waiting for an operator to
    /// find it.
    ///
    /// Carries nothing. There is exactly one document in this build; when
    /// there are several (the document switcher `OpenDoc::path` is held for),
    /// this grows the identity of the one being closed, which is the same
    /// change [`Self::DeleteSelection`] made when it grew its page number.
    Close,
    /// Multiply the current zoom by a factor — the Ctrl+wheel path.
    ///
    /// Carries the factor rather than a target zoom because that is what
    /// egui's `zoom_delta` reports, and because the *clamp* must be applied
    /// by the state machine that owns the ceiling, not by the widget.
    ZoomBy(f32),
    /// Step to the next zoom-ladder rung above the current zoom.
    ZoomIn,
    /// Step to the next zoom-ladder rung below the current zoom.
    ZoomOut,
    /// Enter a fit mode — a *mode*, not a one-shot: the zoom is re-derived
    /// from the viewport every frame until an explicit zoom pins it.
    Fit(FitMode),
    /// Pin the zoom to an exact factor.
    ///
    /// **`Fit(FitMode::None)` is not this**, and the difference was a live
    /// defect the status bar surfaced. `FitMode::None` only stops the
    /// per-frame re-fit; it leaves `zoom` wherever it happened to be. So an
    /// "Actual size" control raising it pinned 73 % *at* 73 % while its
    /// tooltip promised one PDF point per screen point — a control whose
    /// label and behaviour disagreed, on two surfaces at once, because the
    /// ribbon's `view.zoom_actual` raised the same thing.
    ///
    /// `ZoomBy(1.0 / zoom)` would land on the right number and is still
    /// wrong: it routes a discrete command through the wheel path, which
    /// carries the 150 ms settle debounce that exists for continuous
    /// gestures. A command that arrives in one piece should commit in one
    /// piece.
    ZoomTo(f32),
    /// Step one page toward the end of the document.
    NextPage,
    /// Step one page toward the start of the document.
    PrevPage,
    /// Jump to a 0-based page index, clamped into the document.
    GoToPage(usize),
    /// Remove the canvas selection's objects from `page`, as **one**
    /// undoable command.
    ///
    /// Raised by the canvas when Delete or Backspace is pressed with a
    /// non-empty selection and no text field focused — the defect `DEFECTS.md`
    /// D1 is about, from the other end. D1's fix (`ctx.text_edit_focused()`
    /// rather than `ctx.egui_wants_keyboard_input()`) made the key *reachable*
    /// after a canvas click; this is the verb it reaches.
    ///
    /// # The operand list is already clean, and must be
    ///
    /// `objects` arrives ascending and de-duplicated from
    /// [`crate::canvas::selection::SelectionState::object_indices_on`],
    /// because `EditSession::delete_objects` resolves **every** index before
    /// planning anything: one stale or duplicated entry refuses the whole
    /// call. That refusal is the correct engine behaviour — the alternative
    /// is deleting the prefix that happened to resolve — so the shell's job
    /// is to hand it a list that can succeed.
    ///
    /// # Why the page travels with the list
    ///
    /// A paint-order index is a position on **one page**. Re-deriving the
    /// page here from `doc.view.page_index` would be a second source of truth
    /// that is right until the moment it matters: an action is applied after
    /// the frame that raised it, and a page step raised in the same frame is
    /// applied first if it was pushed first. Carrying the page makes the
    /// statement complete.
    DeleteSelection {
        /// The 0-based page the indices are positions on.
        page: usize,
        /// Paint-order indices, ascending and unique.
        objects: Vec<usize>,
    },
    /// Displace the canvas selection's objects on `page` by a **page-space**
    /// delta, as **one** undoable command.
    ///
    /// Raised by [`crate::canvas::moving::drag`] when a move drag that began
    /// inside the selection is released. The Object-rung member of the move
    /// family; its siblings are [`Self::MoveSubpath`] and [`Self::MoveNode`].
    ///
    /// # Why the whole list travels, exactly as it does for Delete
    ///
    /// `EditSession::move_objects` takes a **slice**, and resolves *and
    /// type-checks* every index before planning anything, so one non-path or
    /// one stale entry refuses the whole call rather than moving the prefix
    /// that happened to qualify. Emitting one `move_object` per selected
    /// object would be wrong twice over: N undo entries for one drag, and — the
    /// correctness half — each call re-splices the content stream, so the
    /// second index would be planned against byte offsets the first already
    /// invalidated. `docs/core-api/02` states it in a box: *"Never loop the
    /// singular verbs over a selection."*
    ///
    /// # ★ Why this does NOT invalidate the selection, and Delete does
    ///
    /// Because `move_*` **does not renumber**, and that is measured rather
    /// than assumed — `crates/pdfce-core/tests/object_identity_across_edits.rs`
    /// decomposes, edits, and decomposes again. A move rewrites operands
    /// *inside* existing operators, so no operator is added or removed and the
    /// second decomposition yields the same objects at the same indices. The
    /// `delete_*` family excises byte **spans** and therefore does renumber,
    /// which is why `pdfce_core::vector::remap_index_after_delete` exists and
    /// why nothing like it is needed here. See
    /// [`crate::canvas::moving`]'s header for the full table.
    ///
    /// # Units
    ///
    /// `dx`/`dy` are **PDF user-space** points, Y-**up** — produced by
    /// [`crate::canvas::moving::page_delta`], which is the one place a
    /// canvas-space drag crosses into page space. A screen-pixel delta here
    /// would compile, run, and scale the move with the magnification.
    MoveSelection {
        /// The 0-based page the indices are positions on.
        page: usize,
        /// Paint-order indices, ascending and unique.
        objects: Vec<usize>,
        /// Horizontal displacement, PDF user-space points.
        dx: f64,
        /// Vertical displacement, PDF user-space points (Y is up).
        dy: f64,
    },
    /// Displace **one subpath** of one path object by a page-space delta, as
    /// one undoable command — the Part rung's move verb.
    ///
    /// Raised only when the entered object decomposes into *subpaths*. A text
    /// object's Part rung is a show-operator run, which `move_subpath` has
    /// nothing to translate, so the canvas declines and traces rather than
    /// borrowing the Object rung's verb — the same rule, and the same reason,
    /// as `SelectionState::deletable_objects_on`'s rung guard.
    MoveSubpath {
        /// The 0-based page.
        page: usize,
        /// The enclosing object, by paint-order index.
        object: usize,
        /// The subpath, in decomposition order.
        subpath: usize,
        /// Horizontal displacement, PDF user-space points.
        dx: f64,
        /// Vertical displacement, PDF user-space points (Y is up).
        dy: f64,
    },
    /// Drag **one anchor** of one path object to an absolute page-space point
    /// — the Node rung's move verb.
    ///
    /// # Why a destination and not a displacement
    ///
    /// Because that is `EditSession::move_node`'s signature, and the signature
    /// is right: the operand being rewritten *is* a coordinate pair, and the
    /// planner maps the destination through the object's CTM affine inverse in
    /// one step. Expressing it as a delta would make the planner reconstruct
    /// the point it was given, in a space the caller would then have had to
    /// name. The canvas computes it as *"where the anchor is now, plus the
    /// drag"*, and refuses the move outright if the decomposition can no
    /// longer say where the anchor is — see
    /// [`crate::canvas::moving::Refusal::NodeNotFound`].
    ///
    /// `node` is **object-scoped**: the space `vector::anchor_count` reports
    /// and `pdfce-cli node-move --node N` addresses. A second numbering would
    /// make the number pdfce shows disagree with the number the operator can
    /// act on.
    MoveNode {
        /// The 0-based page.
        page: usize,
        /// The enclosing object, by paint-order index.
        object: usize,
        /// The anchor, object-scoped.
        node: usize,
        /// Where the anchor ends up, in PDF user space.
        to: pdfce_core::vector::Point,
    },
    /// Show or hide one optional-content group.
    ///
    /// **View state, not document state.** It changes what this session
    /// draws and never what a save would write — which is why it does not
    /// bump `edit_epoch` and why the Layers panel's own note says a toggle
    /// changes what you see and not the document.
    ///
    /// It does invalidate the page raster, and that is now expressible:
    /// `RenderKey` gained `layers_generation` in the same stage as this
    /// variant, honouring `render/worker.rs`'s rule that *"the key ships in
    /// the same commit as its control"*. Before that, a checkbox here would
    /// have redrawn nothing — which is why the panel shipped without one.
    SetLayerVisible {
        /// The optional-content group.
        group: pdfce_core::object::ObjId,
        /// Whether it should be drawn.
        visible: bool,
    },
    /// Restore every optional-content group to the document's own default.
    ///
    /// Not "show everything": a document may declare groups that are off by
    /// default, and revealing those would be a different act from undoing
    /// the operator's own hiding.
    ResetLayers,
    /// Show or hide annotations as a class.
    ///
    /// Same nature as [`Self::SetLayerVisible`] — a view stance, tracked by
    /// `RenderKey`, invisible to a save.
    ToggleAnnotations,
    /// **One thing the operator asked Find to do** — run the search, or step
    /// to the adjacent hit.
    ///
    /// # Why a search goes through the funnel at all, when it changes nothing
    ///
    /// Because it needs `&mut EditSession`.
    /// [`pdfce_core::edit::EditSession::find_text_with`] takes a mutable
    /// borrow — it is a read that mutates the session's own working state —
    /// and `OpenDoc::session` is an `Arc` precisely so the render worker can
    /// hold a clone while it rasterizes. `Arc::get_mut` fails while any other
    /// strong reference exists, so the worker has to be stopped first, and
    /// stopping a render **in the middle of laying out a frame** is exactly
    /// what this funnel exists to prevent. Applied after the frame, it is one
    /// short pause in a rasterization that was going to restart anyway.
    ///
    /// So the rule this variant honours is not the letter of
    /// "actions-not-mutations" (a search mutates no document) but its
    /// *reason*: do no expensive or ordering-sensitive work in the middle of
    /// a frame.
    ///
    /// # Why stepping is an action too, when it moves no bytes
    ///
    /// Because it **navigates**: moving to the next hit changes the page and
    /// the scroll offset, which is `Action::GoToPage`'s territory. Doing it
    /// in the widget would put a page change inside the frame that is already
    /// drawing the old page — the one-frame-late class of defect
    /// `crate::app`'s header describes for the whole apply phase.
    ///
    /// The operand is carried, exactly as [`Self::DeleteSelection`] carries
    /// its index list, because an action is a complete statement of intent:
    /// *which* way to step cannot be re-derived after the frame that asked.
    /// See `crate::find` for what happens on the other end.
    Find(crate::find::FindRequest),
    /// **One form-field edit**, as one undoable command.
    ///
    /// The variant `crate::panels::forms` raises for every one of its verbs —
    /// fill, toggle, choose, reset, regenerate appearances, flatten — carrying
    /// the whole intent so it is resolvable after the frame that raised it, in
    /// the same way [`Self::DeleteSelection`] carries its operand list.
    ///
    /// # Why the arm below is one line and not four
    ///
    /// It does not go through [`vector_edit`], and `crate::panels::forms::edit`'s
    /// own header carries the reason: the six form outcome types do not unify
    /// into `Result<Vec<String>, EditError>`, so that module performs the
    /// cancel-mutate-bump-invalidate protocol itself, once, for all of them.
    /// A second copy of the protocol here would be the fifth hand-written
    /// instance of a four-step sequence `vector_edit` exists to have exactly
    /// one of.
    Form(crate::panels::forms::edit::FormEdit),
}

impl PdfceApp {
    /// Apply every action raised during the frame just drawn.
    ///
    /// Applied in the order raised. `pixels_per_point` is passed in rather
    /// than read from a context because the per-page zoom ceiling depends
    /// on it — see [`viewer::max_zoom_for_page`] — and threading it makes
    /// this function pure with respect to egui, which is what keeps it
    /// reviewable.
    pub fn apply_actions(&mut self, actions: Vec<Action>, pixels_per_point: f32) {
        for action in actions {
            self.apply(action, pixels_per_point);
        }
    }

    /// Apply a single action.
    ///
    /// Every arm is a state transition on [`crate::viewer::ViewState`],
    /// which is where the clamping and the ladder arithmetic live and are
    /// tested. This function decides *which* transition, never *what it
    /// means* — a zoom that saturates, a page step that stops at the last
    /// page and a NaN that falls back to actual size are all decided in
    /// `viewer`, under unit test.
    fn apply(&mut self, action: Action, pixels_per_point: f32) {
        // ★ The two actions that are about WHICH document is open, matched
        // BEFORE the guard below.
        //
        // Every other arm acts on the open document, so the guard's "no
        // document: silently drop" is the right answer for all of them. It is
        // the wrong answer for these two, and in opposite directions: an Open
        // with nothing open is the *ordinary* case — it is how the operator
        // gets their first document after launching with no argument — and a
        // Close with nothing open is a no-op that must still not be reached
        // through a path that assumes a document.
        //
        // Both consult `save_pending` first. See its own docs for the rule;
        // the short version is that an Open must not run out from under a
        // save, and that this build has no save for it to run out from under.
        //
        // The `_ => {}` arm moves nothing, and the two arms that do move
        // `action` both return, so the value is still whole below. That is a
        // property of the control flow rather than a coincidence: adding a
        // third arm here that falls through would be a use-after-move and the
        // compiler would say so.
        match action {
            Action::Open(path) => {
                if self.save_pending() {
                    crate::diag::trace(|| {
                        // ui-text-exempt: diagnostic trace, never displayed in the UI
                        format!("open-declined path={path:?} reason=save-pending")
                    });
                    return;
                }
                self.open_path(path);
                return;
            }
            Action::Close => {
                if self.save_pending() {
                    crate::diag::trace(|| {
                        // ui-text-exempt: diagnostic trace, never displayed in the UI
                        "close-declined reason=save-pending".to_owned()
                    });
                    return;
                }
                self.close_document();
                return;
            }
            // ★ The third arm matched before the document guard, and it is
            // here for a **borrow** reason rather than for the guard's.
            //
            // Applying a find request needs two of this struct's fields at
            // once — `self.find` and the open document inside `self.status` —
            // and the guard below takes `&mut self.status` for the rest of the
            // function, after which `self.find` is unreachable. Splitting the
            // borrow has to happen while `self` is still whole, which is here.
            //
            // It is *also* correct on the guard's own terms: with nothing
            // open there is nothing to search, and saying so on the trace is
            // more useful than the guard's silent drop, because a keymap can
            // reach `edit.find` from any state and "the chord did nothing" and
            // "the chord did nothing because no document is open" need
            // different responses from whoever is reading the trace.
            Action::Find(request) => {
                match &mut self.status {
                    Status::Open(doc) => crate::find::apply(&mut self.find, doc, request),
                    _ => crate::diag::trace(|| {
                        // ui-text-exempt: diagnostic trace, never displayed in the UI
                        format!("find-declined request={request:?} reason=no-document")
                    }),
                }
                return;
            }
            _ => {}
        }

        let Status::Open(doc) = &mut self.status else {
            // No document: nothing to zoom, nothing to navigate. Silently
            // dropping the action is correct rather than lax — the controls
            // that raise these do not exist without a document, so reaching
            // here at all would mean a keyboard binding was installed
            // without its guard.
            return;
        };

        // The per-page raster ceiling, recomputed here rather than cached
        // because it depends on BOTH the current page's extent and the
        // display's density, either of which can change between frames (a
        // page step, a window dragged to a different monitor). Caching it
        // is how a guard passes its tests and still lets the operator zoom
        // into an allocation failure on the one machine that matters.
        let max_zoom = viewer::max_zoom_for_page(doc.current_extent(), pixels_per_point);
        let page_count = doc.pages.len();

        // ★ Which zoom changes are DISCRETE, and why that matters.
        //
        // `settle_and_rasterize` debounces a zoom by 150 ms so a Ctrl+wheel
        // gesture — which emits dozens of intermediate values — rasterizes
        // once rather than dozens of times. A discrete *command* has no
        // gesture in flight, so waiting out that debounce would make a
        // keypress feel unresponsive for no benefit.
        //
        // So every zoom-changing arm except `ZoomBy` (the wheel path) sets
        // this flag. Getting it backwards is not a crash: it is a keyboard
        // zoom that lags by 150 ms, or a wheel gesture that re-rasterizes a
        // CAD sheet on every notch. Both were real behaviours in the old
        // shell's history, which is why the distinction is a named flag
        // rather than an inline condition.
        // `|=`, not `=`: several actions can be raised in one frame, and a
        // later non-zoom action must not clear a flag an earlier zoom
        // command set. `settle_and_rasterize` clears it once per frame,
        // which is the only place it may be cleared.
        //
        // Matched on a REFERENCE since `Action` stopped being `Copy` (see the
        // module docs): `matches!` moves its scrutinee, and the `match` below
        // needs the value.
        // `ZoomTo` belongs here and was missing when the variant landed —
        // the comment above already said "every zoom-changing arm except
        // `ZoomBy`", so the list and its own description disagreed. The
        // symptom was quiet: Actual size, from the button *and* from Ctrl+0,
        // waited out the 150 ms wheel-settle debounce before re-rastering,
        // as though it were a continuous gesture. A discrete command should
        // commit at once.
        doc.zoom_commanded |= matches!(
            &action,
            Action::ZoomIn | Action::ZoomOut | Action::Fit(_) | Action::ZoomTo(_)
        );

        match action {
            // Handled above, before the guard that needs an open document —
            // which is the point, since one of them is how a document becomes
            // open. Spelled out rather than folded into a catch-all so that a
            // new variant added to the enum still fails to compile here.
            // ui-text-exempt: a panic message, read from a stack trace by
            // whoever moved one of these two arms. Never rendered.
            Action::Open(_) | Action::Close | Action::Find(_) => {
                // ui-text-exempt: a panic message, read from a stack trace by
                // whoever moved one of these three arms. Never rendered.
                unreachable!("handled before the document guard")
            }
            Action::ZoomBy(factor) => doc.view.zoom_by(factor, max_zoom),
            Action::ZoomIn => doc.view.zoom_in(max_zoom),
            Action::ZoomOut => doc.view.zoom_out(max_zoom),
            Action::Fit(mode) => doc.view.set_fit(mode),
            Action::ZoomTo(zoom) => doc.view.set_zoom(zoom, max_zoom),
            Action::NextPage => doc.view.next_page(page_count),
            Action::PrevPage => doc.view.prev_page(page_count),
            Action::GoToPage(index) => doc.view.go_to_page(index, page_count),
            Action::DeleteSelection { page, objects } => {
                if !objects.is_empty() {
                    vector_edit(doc, "delete-objects", page, objects.len(), |session| {
                        session.delete_objects(page, &objects)
                    });
                }
            }
            Action::MoveSelection {
                page,
                objects,
                dx,
                dy,
            } => {
                if !objects.is_empty() {
                    vector_edit(doc, "move-objects", page, objects.len(), |session| {
                        session.move_objects(page, &objects, dx, dy)
                    });
                }
            }
            Action::MoveSubpath {
                page,
                object,
                subpath,
                dx,
                dy,
            } => {
                vector_edit(doc, "move-subpath", page, 1, |session| {
                    session.move_subpath(page, object, subpath, dx, dy)
                });
            }
            Action::MoveNode {
                page,
                object,
                node,
                to,
            } => {
                vector_edit(doc, "move-node", page, 1, |session| {
                    session.move_node(page, object, node, to)
                });
            }
            Action::SetLayerVisible { group, visible } => doc.set_layer_visible(group, visible),
            Action::ResetLayers => doc.reset_layers(),
            Action::ToggleAnnotations => {
                let showing = doc.annotations_visible();
                doc.set_annotations_visible(!showing);
            }
            Action::Form(edit) => crate::panels::forms::edit::apply(doc, &edit),
        }
    }
}

/// Apply **one** vector-geometry edit to `doc`, as one undoable command.
///
/// The shared body of every arm above that changes the document — Delete and
/// the three move verbs. It exists as one function rather than four copies for
/// the reason the ordering below is load-bearing: each of the four steps is a
/// separate way to end up with an edit that is silently declined or a page that
/// silently keeps drawing what was just changed, and four hand-written copies
/// of a four-step protocol is four chances to omit a step. The `label` and the
/// operand count are carried only so the trace can say which verb ran.
///
/// # The four things that have to happen in this order
///
/// 1. **Stop the render worker.** `OpenDoc::session` is an `Arc` precisely so
///    a worker can hold a clone while it rasterizes, and
///    `RenderWorker::cancel_and_wait`'s own docs call itself *"the choke
///    point that makes `Arc<EditSession>` sound"*: `Arc::get_mut` fails while
///    any other strong reference exists, so a mutation attempted mid-render
///    would simply be refused. Cancelling first is what turns "sometimes
///    refused, depending on how fast the page rasterized" into "always
///    applied".
/// 2. **Mutate through `Arc::get_mut`.** A `None` here is not a panic: it
///    means something else still holds the session, which is a bug in the
///    caller's ordering rather than in the operator's document. It is traced
///    and the edit is declined, because declining an edit is recoverable and
///    corrupting one is not.
/// 3. **Bump `edit_epoch`.** `OpenDoc::edit_epoch`'s own doc comment names
///    this exact seam: *"the first mutating arm added to
///    `PdfceApp::apply` must bump it"*, so the object-count trace re-reads
///    and — the part that matters here — the canvas's selection re-resolves
///    against the new decomposition rather than keeping an entry that now
///    names a hole.
///
///    **A move needs the epoch bump for the geometry, not for the identity.**
///    `move_*` rewrites operator operands in place and adds or removes no
///    operator, so paint-order indices are stable across it — measured, by
///    `crates/pdfce-core/tests/object_identity_across_edits.rs`. The selection's
///    *entries* therefore survive untouched and nothing has to be remapped;
///    what has changed is where each entry's outline goes, and the epoch is
///    what makes `SelectionState::resolve` recompute it. `delete_*` excises
///    byte spans and does renumber, which is the case
///    `pdfce_core::vector::remap_index_after_delete` exists for.
/// 4. **Drop the cached texture** — see below.
///
/// # The disclosures, and the one thing this function cannot yet discharge
///
/// Every vector verb returns `Result<Vec<String>, EditError>`, and the
/// `Vec<String>` is the **disclosure list**: operator-facing strings the
/// surgery owes under rule 4, non-empty when the edit had to change an
/// operator's *form* to express the request — an `re` rectangle expanded into
/// explicit segments, an implicitly-started subpath's `m` materialised. The
/// drawing is unchanged but the bytes are no longer recoverable by reversing
/// the gesture, and rule 4 forbids letting the operator find that out from a
/// diff.
///
/// They are traced here, in full, so nothing is lost. **Tracing is not
/// surfacing**: a disclosure belongs on an operator-visible surface, and the
/// status line is `app::status`'s to own, not this module's to invent. That is
/// the outstanding half, and it is named rather than left implicit precisely
/// because a disclosure that only ever reaches `PDFCE_DIAG` has been recorded
/// and not disclosed.
///
/// # Why the cached texture is dropped
///
/// Nothing else notices an edit. `settle_and_rasterize` compares the cached
/// texture against the page index and the raster scale, and an edit changes
/// neither — so without this the page would keep showing the object where it
/// used to be until the operator zoomed or paged away. Dropping it forces a
/// re-raster on the same frame (step 4 runs after step 3), and
/// `RenderWorker::spawn` waits a bounded number of milliseconds inline, so a
/// page that rasterizes quickly never shows a gap at all.
///
/// The *right* fix is for the texture's key to carry a content generation, so
/// staleness is a property of the key rather than something each mutating arm
/// has to remember. That key lives in `render/`, which is not this module's to
/// extend; this is the honest interim, and it is one line in one shared
/// function rather than a convention spread across four verbs and counting.
fn vector_edit(
    doc: &mut OpenDoc,
    label: &str,
    page: usize,
    operands: usize,
    edit: impl FnOnce(&mut EditSession) -> Result<Vec<String>, EditError>,
) {
    doc.render_worker.cancel_and_wait();
    let Some(session) = Arc::get_mut(&mut doc.session) else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("{label}-refused page={page} n={operands} reason=session-borrowed")
        });
        return;
    };
    match edit(session) {
        Ok(disclosures) => {
            doc.edit_epoch = doc.edit_epoch.wrapping_add(1);
            doc.page_texture = None;
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "{label} page={page} n={operands} epoch={} disclosures={}",
                    doc.edit_epoch,
                    if disclosures.is_empty() {
                        // ui-text-exempt: diagnostic trace, never displayed in the UI
                        "none".to_owned()
                    } else {
                        disclosures.join(" | ")
                    }
                )
            });
        }
        // A refusal is the engine's, and it is structured. Reporting it and
        // leaving the document alone is the whole response available here:
        // the operator-facing half is a status line, which does not exist
        // yet and is not this module's to invent.
        Err(error) => crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("{label}-refused page={page} n={operands} detail={error}")
        }),
    }
}
