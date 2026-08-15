//! # `app::actions::apply` — the other end of the funnel: what an [`Action`]
//! actually does
//!
//! [`super`] declares the **vocabulary** — one variant per thing an operator
//! can ask for, each carrying a complete statement of that intent. This file is
//! the **interpreter**: [`PdfceApp::apply_actions`] drains the frame's queue,
//! [`PdfceApp::apply`] routes one intent to one state transition, and
//! [`vector_edit`] is the four-step protocol every arm that changes a document
//! goes through.
//!
//! ## ★ Why this is its own file
//!
//! `app/actions.rs` crossed the 1,500-line gate (standing rule **R2**) when
//! `file.save_copy` was wired, and the rule's own justification decides where
//! the cut goes rather than the line count: *"the value of the limit is that
//! the file has to have a single subject."*
//!
//! Two subjects were sharing one file, and they change for entirely different
//! reasons:
//!
//! | | subject | what changes it |
//! |---|---|---|
//! | [`super`] | **what an operator can ask for**, and what each request must carry to remain resolvable after the frame that raised it | a new command with a new operand |
//! | here | **what happens when one is granted**, and the ordering that makes a mutation safe | a new engine verb, or a change to the cancel-mutate-bump-invalidate protocol |
//!
//! It is the same seam `app/mod.rs` has been split along four times —
//! `dispatch.rs` (*what does this verb do*), `conditions.rs` (*what is true
//! right now*), `gating.rs` (*what may this mode do*), `panels.rs` — and the
//! same one `app/state.rs` was split along to produce `lifecycle.rs`. The test
//! for whether a split was along a seam is whether the tests came with it, and
//! they did: both tests below drive [`vector_edit`], and neither reads the
//! [`Action`] enum at all.
//!
//! ## What did NOT move, and why
//!
//! The **edit-disclosure store** stayed in [`super`], beside the type it holds,
//! even though [`vector_edit`] is its only writer. Two reasons: it is read by
//! `crate::app::status` through [`super::last_edit_disclosure`], which makes it
//! part of this module tree's published surface rather than an implementation
//! detail of applying; and Rust's own visibility rule means a child module can
//! reach an ancestor's private items, so `record_edit_disclosure` is callable
//! here without being made `pub` to anybody else. Splitting a store from its
//! type to follow its writer would have been the tidier-looking edit and the
//! one that widened a private thing's visibility for no gain.

use std::sync::Arc;

use pdfce_core::edit::{EditError, EditSession};

use super::{Action, EditDisclosure, record_edit_disclosure};
use crate::app::PdfceApp;
use crate::app::state::{OpenDoc, Status};
use crate::viewer;

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
        // ★ The three actions that are about WHICH document is open, matched
        // BEFORE the guard below.
        //
        // Every other arm acts on the open document, so the guard's "no
        // document: silently drop" is the right answer for all of them. It is
        // the wrong answer for these three, and in opposite directions: an Open
        // or a New with nothing open is the *ordinary* case — it is how the
        // operator gets their first document after launching with no argument —
        // and a Close with nothing open is a no-op that must still not be
        // reached through a path that assumes a document.
        //
        // All three consult `save_pending` first. See its own docs for the
        // rule; the short version is that an Open must not run out from under a
        // save, and that the one save this build has — `file.save_copy`, applied
        // two arms down — is **synchronous**, so it is never in flight across a
        // frame and there is nothing for these three to run out from under. It
        // is emphatically not "are there unsaved edits?"; see
        // `crate::app::save` section 3.
        //
        // Two more arms follow them for different reasons — `SaveCopy` because
        // the guard's silence is wrong for a chord, `Find` because of a borrow —
        // and each says so at its own site.
        //
        // The `_ => {}` arm moves nothing, and every arm that moves `action`
        // returns, so the value is still whole below. That is a property of the
        // control flow rather than a coincidence: adding an arm here that fell
        // through would be a use-after-move and the compiler would say so.
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
            // ★ New, beside Open for both of the reasons Open is here: with
            // nothing open it is the *ordinary* case, and it consults the same
            // one predicate rather than a second rule of its own.
            Action::New => {
                if self.save_pending() {
                    crate::diag::trace(|| {
                        // ui-text-exempt: diagnostic trace, never displayed in the UI
                        "new-declined reason=save-pending".to_owned()
                    });
                    return;
                }
                self.new_document();
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
            // ★ Save a copy — matched here for the guard's own reason rather
            // than for a borrow one, and it is the third kind of case this
            // pre-guard block holds.
            //
            // It needs `&self.status` and nothing else, so it could have sat
            // below with the rest. It is here because the guard's answer for
            // "no document" is a **silent** drop, and that is the wrong answer
            // for a command bound to `Ctrl+S`: a keymap reaches any command
            // from any state, and an operator who presses the save chord over
            // an empty shell must not be indistinguishable, in the trace, from
            // one whose keystroke never arrived. `Action::Find` is here for the
            // same half of the same argument.
            //
            // The arm routes and does not compute. Everything about what a save
            // IS — the suggested name, the picker, the incremental mode, the
            // `SaveOptions`, the write, what happens to the epoch (nothing) —
            // lives in `crate::app::save`, which is also where the reason this
            // runs in the apply phase rather than in the dispatcher is written
            // down. Note what it does NOT do: `vector_edit`. A save reads the
            // session (`to_incremental_bytes(&self)`), so there is no worker to
            // cancel, no `Arc::get_mut` to fail, no epoch to bump and no texture
            // to drop — see `save`'s section 2.
            Action::SaveCopy => {
                match &self.status {
                    Status::Open(doc) => crate::app::save::save_copy(doc),
                    _ => crate::diag::trace(|| {
                        // ui-text-exempt: diagnostic trace, never displayed in the UI
                        "save-copy-declined reason=no-document".to_owned()
                    }),
                }
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
            Action::Open(_) | Action::New | Action::Close | Action::SaveCopy | Action::Find(_) => {
                // ui-text-exempt: a panic message, read from a stack trace by
                // whoever moved one of these five arms. Never rendered.
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
            // ★ One markup annotation, through the same four-step protocol
            // every other document change uses.
            //
            // The arm routes; it does not compute. `markup::spec` is a pure,
            // unit-tested function of the kind and the two endpoints — which is
            // where the per-kind normalisation rule lives, and where it must
            // live, because "an arrow keeps its raw endpoints" is a rule with a
            // test rather than a line of wiring.
            //
            // `.map(|_| Vec::new())` adapts `add_markup`'s `ObjId` to the
            // disclosure list `vector_edit` traces, and the empty vec is a
            // statement rather than a placeholder: authoring an annotation
            // rewrites no existing operator, so there is nothing whose *form*
            // changed and therefore nothing rule 4 obliges us to disclose. The
            // new object's id is discarded because nothing here addresses it —
            // the Comments panel that will is a separate surface with its own
            // way of finding annotations on a page.
            // One `add_dimension`, one undo entry — the same contract
            // `CommitMarkup` below holds, through the same `vector_edit` funnel
            // so the cancel-mutate-bump-invalidate protocol is not written a
            // second time.
            Action::CommitDimension { page, group, kind } => {
                vector_edit(doc, "add-dimension", page, 1, |session| {
                    session.add_dimension(page, group, kind).map(|_| Vec::new())
                });
            }
            // ★ The `Option` is `markup::Refusal::Mismatched` arriving at the
            // only place it can: a kind holding another family's geometry.
            //
            // Unconstructible from a gesture — `markup::action` refuses the pair
            // before an `Action` exists — and handled rather than unwrapped,
            // because an `Action` is plain data that a test can build and a
            // future undo/redo surface could replay. Declining by name beats a
            // panic in the frame that is trying to draw, and it emphatically
            // beats authoring a shape nobody asked for.
            Action::CommitMarkup {
                page,
                kind,
                geometry,
            } => {
                if let Some(spec) = crate::canvas::markup::spec(kind, &geometry) {
                    vector_edit(doc, "add-markup", page, 1, |session| {
                        session.add_markup(page, &spec).map(|_| Vec::new())
                    });
                } else {
                    crate::canvas::markup::decline(
                        kind,
                        page,
                        crate::canvas::markup::Refusal::Mismatched,
                    );
                }
            }
            // ★ One text markup, through the SAME funnel and the same engine
            // verb as the drag-shaped kinds above.
            //
            // The label differs (`add-text-markup`) and nothing else does, which
            // is the point: a `/Underline` and a `/Square` are one
            // `add_markup(page, &spec)` apart, so the four-step protocol, the
            // undo entry, the epoch bump and the texture drop are not written a
            // second time for a second family of annotation.
            //
            // The arm routes; it does not compute. `markup::text::spec` is a
            // pure, unit-tested function of the kind and the quads, and the
            // quads themselves were derived once, by `textsel::resolve`, from
            // the same pass that painted the wash the operator was looking at
            // when they pressed the button.
            //
            // ★ Note the second-order consequence, which is deliberate and is
            // documented at `canvas::textsel` §7: `vector_edit` bumps
            // `edit_epoch`, so the selection that authored this annotation is
            // **stale on the next frame** and its wash disappears. Acrobat keeps
            // its selection across a markup; this does not, because the epoch is
            // the only staleness signal there is and refining it into kinds of
            // edit would be a second rule living outside the module that owns
            // the first.
            Action::CommitTextMarkup { page, kind, quads } => {
                let spec = crate::canvas::markup::text::spec(kind, quads);
                vector_edit(doc, "add-text-markup", page, 1, |session| {
                    session.add_markup(page, &spec).map(|_| Vec::new())
                });
            }
            // ★★ **The commit `DEFECTS.md` D4b is about**, and the two lines
            // that make it different from the old shell's are `plan`'s.
            //
            // The old shell wrote, at its ONLY call site:
            //
            // ```text
            // doc.session_mut().edit_text(&req, &EditOptions::default())
            // ```
            //
            // `EditOptions::default()` is `FollowerDisposition::Reflow`, for
            // every run on every page of every document — so a right-aligned
            // tail was pushed off its margin and a rotated line's tail was slid
            // sideways along an axis its baseline does not run down. Neither is
            // a missing feature; both are the wrong arithmetic, chosen by
            // default because the type was constructible at the point of use.
            //
            // The arm still routes. `canvas::textedit::plan` is where the rule
            // lives, and it is one call: it re-derives the provenance pin, reads
            // the run's matrices, asks the engine for the block's alignment, and
            // returns the request and the options together so a caller cannot
            // take one without the other.
            //
            // ★ The disclosure list is the engine's own, PLUS one this shell
            // adds. `EditReport::disclosures` already says a reflowed line may
            // overrun its margin; it says nothing at all about a pinned one,
            // because from the engine's side pinning is what was asked for. But
            // a pinned tail does not make room, so a longer replacement grows
            // into it — and rule 4 forbids letting the operator discover that
            // from a diff. `plan`'s reason is what decides whether the sentence
            // is appended, so the disclosure and the disposition cannot disagree.
            Action::CommitTextEdit {
                page,
                run,
                original,
                replacement,
            } => {
                let plan = crate::canvas::textedit::plan(doc, page, run, &original, &replacement);
                let reason = plan.reason;
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    //
                    // ★ It names the DISPOSITION and the REASON, and it has to
                    // name both. `HANDOFF.md` §2's grid lesson is that a check
                    // asserting a relation is satisfied by any absurdity in the
                    // right direction — and "an edit happened" is exactly such a
                    // relation here. A build that had reverted to
                    // `EditOptions::default()` would produce an identical
                    // `edit-text` line one line below this one; only this line
                    // carries the number that build would get wrong.
                    format!(
                        "text-edit-plan page={page} run={run} disposition={:?} reason={reason:?}                          pinned={}",
                        plan.options.disposition,
                        plan.request.pinned_span.is_some()
                    )
                });
                vector_edit(doc, "edit-text", page, 1, |session| {
                    session
                        .edit_text(&plan.request, &plan.options)
                        .map(|report| {
                            let mut notes = report.disclosures;
                            if reason.pins_the_tail() {
                                notes.push(crate::text::textedit::pinned_tail_disclosure(reason));
                            }
                            notes
                        })
                });
            }
            // ★ New page text, through the same funnel and the same four steps.
            //
            // `AddTextRequest::new` supplies the engine's own documented default
            // — a bundled 12-pt black Helvetica run — and this arm does not
            // override it. That is a decision and not an omission: a font, size
            // and colour picker is a *surface*, and `RIBBON_IA.md` P3's rule is
            // that a capability which is absent renders nothing rather than a
            // control that explains itself badly. Picking a face here from
            // nothing would be this arm computing rather than routing, and the
            // engine's default is the one answer that is already argued
            // somewhere. What the operator gets today is legible, real page
            // content they can undo; what they do not get yet is a choice about
            // its appearance, and `Format` is where that lands.
            Action::CommitAddText { page, origin, text } => {
                let req = pdfce_core::text_edit::AddTextRequest::new(page, origin, text);
                vector_edit(doc, "add-text", page, 1, |session| {
                    session.add_text(&req).map(|report| report.disclosures)
                });
            }
            // ★ The three things a mode change does. See the variant's docs
            // for why each one is here and not somewhere more convenient.
            //
            // The no-op guard is not an optimisation: the ribbon raises this
            // on every click, including a click on the position that is
            // already active, and without the guard each of those would drop
            // every strip raster and re-render the visible pages.
            Action::SetPageDisplay(display) => {
                if doc.view.display != display {
                    doc.view.display = display;
                    doc.strip_rasters.clear();
                    // `tracked_page` follows, so the new arrangement does not
                    // read the current page as "navigated to" and scroll to it
                    // on its first frame.
                    doc.tracked_page = doc.view.page_index;
                }
                // Recorded unconditionally, because the operator has stated a
                // choice and a document that was showing the mode by *default*
                // has nothing on disk saying so. `remember` is itself a no-op
                // when the file already says this, so a repeated click still
                // costs no write.
                //
                // ★ …and only for a document that HAS a file. `stored_under`
                // is the one predicate that says so. A created document's path
                // is a name, so writing here would store an arrangement
                // against `Untitled 1.pdf` — which the next session's
                // `Untitled 1.pdf` would then inherit, having chosen nothing.
                if let Some(path) = doc.stored_under() {
                    crate::viewer::remembered::remember(path, display);
                }
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    format!(
                        "page-display-set mode={} page={}",
                        display.id(),
                        doc.view.page_index
                    )
                });
            }
            Action::SetLayerVisible { group, visible } => doc.set_layer_visible(group, visible),
            Action::ResetLayers => doc.reset_layers(),
            Action::ToggleAnnotations => {
                let showing = doc.annotations_visible();
                doc.set_annotations_visible(!showing);
            }
            Action::ToggleViewChrome(chrome) => {
                let on = !chrome.read(&doc.view);
                chrome.write(&mut doc.view, on);
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    format!("view-chrome {chrome:?} on={on}")
                });
            }
            // The guides are stored and written to disk in one step, because
            // the file IS the store's authority: `remember` is a
            // read-modify-write of the whole line, so there is no half-applied
            // state to guard against and nothing to reconcile on the next
            // open. Unconditional, exactly as `SetPageDisplay`'s `remember` is
            // — a `Guides` that equals what is already there is a gesture that
            // raised no action at all (`canvas::guides::release` compares
            // before it pushes), so a redundant write is unreachable rather
            // than merely cheap.
            // ★ …with the same `stored_under` guard `SetPageDisplay` carries,
            // for the same reason and against the same failure: a guide
            // position stored under a name rather than a location is a guide
            // the next `Untitled 1.pdf` inherits. The guides still work in the
            // session — they live on `doc` — they simply are not persisted for
            // a document that has nowhere to persist them to.
            Action::SetGuides(guides) => {
                doc.guides = guides;
                if let Some(path) = doc.stored_under() {
                    crate::canvas::guides::remember(path, &doc.guides);
                }
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    format!(
                        "guides-set n={} page={}",
                        doc.guides.len(),
                        doc.view.page_index
                    )
                });
            }
            Action::Form(edit) => crate::panels::forms::edit::apply(doc, &edit),
            // ===============================================================
            // ★ THE REDACTION MARKING VERBS
            //
            // Three arms, each one call, through the same `vector_edit` funnel
            // every other document change uses — which is the whole reason they
            // are one line each. Marking is an ordinary edit: it authors an
            // annotation, the engine records it as an undoable command, and the
            // page has to re-raster because a `/Redact` mark draws a red
            // outline the operator needs to see.
            //
            // ★ **Nothing here removes anything.** The irreversible half is
            // `crate::dialogs::redact`, which reaches no arm in this file at
            // all: it changes no document, so it has nothing to order against
            // and no epoch to bump, and routing it through here would put the
            // one operation that cannot be undone into a queue that replays.
            //
            // `.map(|_| Vec::new())` on the first two adapts the engine's
            // `Vec<ObjId>`/`ObjId` to the disclosure list `vector_edit` traces,
            // and the empty vec is a statement rather than a placeholder —
            // authoring an annotation rewrites no existing operator, so nothing
            // changed form and rule 4 owes the operator nothing. It is the same
            // adaptation `CommitMarkup` makes one screen up.
            // ===============================================================
            Action::MarkRedactionsBySearch { query, pattern } => {
                if !query.is_empty() {
                    let page = doc.view.page_index;
                    // The label distinguishes the two marking modes on the
                    // trace, because a pattern that marked nothing and a
                    // literal that marked nothing are different diagnoses:
                    // one is a query the document does not contain, the other
                    // is very often a `#` the operator meant literally.
                    let label = if pattern {
                        // ui-text-exempt: diagnostic trace, never displayed in the UI
                        "redact-mark-pattern"
                    } else {
                        // ui-text-exempt: diagnostic trace, never displayed in the UI
                        "redact-mark-search"
                    };
                    let before = crate::panels::redact::mark_ids(&doc.session).len();
                    vector_edit(doc, label, page, 1, |session| {
                        // ★ Case-INSENSITIVE, always, and it is not a missing
                        // control. Over-marking is the safe direction of error
                        // on this verb and under-marking is not: a mark the
                        // operator did not want is one row and one click in the
                        // review list, and a mark they did want and did not get
                        // is a name shipped in a document they believe is
                        // redacted. The old shell made the same ruling in the
                        // same words.
                        if pattern {
                            session.mark_redactions_by_pattern(&query, true)
                        } else {
                            session.mark_redactions_by_search(&query, true)
                        }
                        .map(|_| Vec::new())
                    });
                    // ★ Reported AFTER the edit, from the same census the panel
                    // lists from, so the number on the trace and the number of
                    // rows on screen cannot disagree. `created=0` is the
                    // interesting value: it is a search that found nothing,
                    // which on a scanned page is the named real-world failure
                    // — `crate::text::redact::search_hint` is the sentence that
                    // warns about it, and this is how a reader of a trace sees
                    // it happen.
                    let after = crate::panels::redact::mark_ids(&doc.session).len();
                    crate::diag::trace(|| {
                        format!(
                            // ui-text-exempt: diagnostic trace, never displayed in the UI
                            "redact-marked mode={} created={} total={}",
                            if pattern { "pattern" } else { "literal" },
                            after.saturating_sub(before),
                            after
                        )
                    });
                }
            }
            Action::MarkPageForRedaction { page } => {
                // Resolved here rather than carried on the action because the
                // rectangle is the page's, not the operator's — see the
                // variant's docs. A page index past the end is unreachable from
                // the panel and is answered rather than indexed, because an
                // action is plain data a test can build.
                if let Some(spec) = doc
                    .pages
                    .get(page)
                    .map(crate::panels::redact::whole_page_spec)
                {
                    vector_edit(doc, "redact-mark-page", page, 1, |session| {
                        session.add_redaction(page, &spec).map(|_| Vec::new())
                    });
                } else {
                    crate::diag::trace(|| {
                        // ui-text-exempt: diagnostic trace, never displayed in the UI
                        format!("redact-mark-page-declined page={page} reason=no-such-page")
                    });
                }
            }
            Action::RemoveRedactionMark { annot_id } => {
                let page = doc.view.page_index;
                vector_edit(doc, "redact-unmark", page, 1, |session| {
                    session.delete_redaction_mark(annot_id).map(|()| Vec::new())
                });
            }
            // ===============================================================
            // ★ THE PAGE VERBS
            //
            // Four arms, each one call, because everything that could be a
            // rule lives elsewhere: the operand list and the permutation in
            // `crate::panels::pages::ops` (pure, unit-tested), the engine call
            // and the disclosures in `super::pages`, and the four-step
            // protocol in `vector_edit` — which now carries a fifth step that
            // brings the page vector, the strip rasters, the canvas selection
            // and the view back into agreement with the session.
            //
            // `page=` on the trace line is the FIRST operand rather than "the
            // page", and `n=` is how many were named. There is no single page
            // a multi-page verb is about; the first one is the honest answer
            // and `n=` is the field that actually says what happened, exactly
            // as `history_step`'s own docs argue for the undo case.
            // ===============================================================
            Action::RotatePages { pages, delta } => {
                if !pages.is_empty() {
                    let first = pages.first().copied().unwrap_or(0);
                    vector_edit(doc, "rotate-pages", first, pages.len(), |session| {
                        super::pages::rotate(session, &pages, delta)
                    });
                }
            }
            // ★ **The destructive one**, and the one that renumbers.
            //
            // Two things happen here that no other arm needs, and both are
            // about a *position* ceasing to mean what it meant:
            //
            // 1. `vector_edit`'s resync clears the **canvas** selection and
            //    clamps the view — see `super::pages::resync`;
            // 2. the **Pages panel's** picks are cleared here, because they
            //    live on `self.panels` rather than on the document and
            //    `vector_edit` cannot reach them.
            //
            // The panel's own `retain_below` would drop the picks that fell
            // off the end on the next frame, and that is NOT sufficient: the
            // pages that were deleted are exactly the ones that were picked,
            // so the survivors of a clamp would be picks pointing at sheets
            // that have shuffled down into their indices. Clearing is both
            // correct and provable — every picked sheet is gone.
            //
            // Guarded on the epoch rather than on a return value, so the
            // clear happens only for an edit that actually applied: a refused
            // delete (the engine refuses removing every page, §7.7.3.3) must
            // leave the operator's selection exactly as they built it.
            //
            // **No confirmation dialog.** `crate::app::save::save_pending` is
            // the one predicate this application consults before a destructive
            // path and it is about a save being in flight, not about unsaved
            // work; the engine records this as an undoable command; and
            // nothing reaches disk — the operator's file is untouched until
            // they choose to save a copy. A modal here would be the only one
            // in the application and would be asking about the one destructive
            // act that is already reversible in the session.
            Action::DeletePages { pages } => {
                if !pages.is_empty() {
                    let first = pages.first().copied().unwrap_or(0);
                    let before = doc.edit_epoch;
                    vector_edit(doc, "delete-pages", first, pages.len(), |session| {
                        super::pages::delete(session, &pages)
                    });
                    if doc.edit_epoch != before {
                        self.panels.pages_mut().selection.clear();
                    }
                }
            }
            // ★ **The middle case**: every page survives, and every index
            // means a different sheet.
            //
            // The canvas selection is cleared by the resync; the panel's picks
            // are **remapped** rather than cleared, because the permutation
            // states exactly where each picked sheet went. See
            // `crate::panels::pages::select::PageSelection::remap` for why the
            // two selections get different answers to the same edit — and for
            // why clearing here would make the reorder arrows unusable twice
            // in a row, which is the one gesture they exist for.
            Action::ReorderPages { order } => {
                if !order.is_empty() {
                    let before = doc.edit_epoch;
                    vector_edit(doc, "reorder-pages", 0, order.len(), |session| {
                        super::pages::reorder(session, &order)
                    });
                    if doc.edit_epoch != before {
                        let landed = crate::panels::pages::ops::inverse(&order);
                        self.panels.pages_mut().selection.remap(&landed);
                    }
                }
            }
            // ★ The one page verb that goes nowhere near `vector_edit`: it
            // changes no document, it opens a native save dialog, and it is an
            // `Action` for `Action::SaveCopy`'s frame-timing reason and only
            // that one. See `super::pages::extract`.
            Action::ExtractPages { pages } => super::pages::extract(doc, &pages),
            // ★ **Undo and redo**, through the same [`vector_edit`] funnel every
            // other document change goes through — which is the whole of why
            // these two arms are one line each. See [`history_step`].
            Action::Undo => history_step(doc, Direction::Undo),
            Action::Redo => history_step(doc, Direction::Redo),
            // ★ Reaching here means the frame's drain was removed or moved.
            // Traced rather than ignored: the symptom otherwise is a control
            // that does nothing on every press, with no evidence why.
            Action::Command(id) => crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed.
                    "command-action-undrained id={id}"
                )
            }),
        }
    }
}

/// Apply **one** vector-geometry edit to `doc`, as one undoable command.
///
/// The shared body of every arm above that changes the document — Delete, the
/// three move verbs, and [`Action::CommitMarkup`].
///
/// **Markup joins it rather than getting its own copy**, and the name is now a
/// little narrow for what it does. That is the better trade: `add_markup` needs
/// the identical four steps in the identical order — the worker cancelled
/// first, the mutation through `Arc::get_mut`, the epoch bumped so the canvas
/// re-resolves and the raster is rebuilt, the texture dropped — and the only
/// thing it does differently is return an `ObjId` where the vector verbs return
/// a disclosure list. That is one `.map` at the call site, against a whole
/// second hand-written copy of a protocol whose entire reason for existing is
/// that four hand-written copies would be four chances to omit a step.
///
/// It exists as one function rather than five copies for
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
/// status line is `app::status`'s to own, not this module's to invent.
///
/// ## ★ The outstanding half, discharged 2026-08-14
///
/// The paragraph above used to end *"That is the outstanding half"*, and it is
/// no longer outstanding. The list is now **recorded as well as traced** — as
/// an [`EditDisclosure`] stamped with the epoch this edit produced — and
/// [`crate::app::status`] draws it in the bar beside the fill disclosure it
/// copies, on the row that may not grow (R128).
///
/// Three things about that, each a decision:
///
/// - **The trace is unchanged, not replaced.** It is the record of what
///   happened; the bar is what an operator reads *now* and loses at the next
///   edit. A disclosure that survives only on screen is one the next reader of
///   `PDFCE_DIAG` cannot audit, and one that survives only in the trace is the
///   defect this section used to name.
/// - **This module still does not draw anything**, which is why the split is
///   at a recorded value rather than at a formatted line. Everything an
///   operator sees — the framing, the mark, the eliding, the hover — is
///   decided in `app::status` and `text::status`, exactly as the original
///   sentence said it must be.
/// - **The stamp is the epoch bumped one line above, not the one the edit ran
///   against.** The revision on screen from now until the next edit is the new
///   one, so an undo silences the sentence by moving the epoch past it, with
///   nothing anywhere remembering to clear it. Stamping the old epoch would
///   produce a disclosure that was invisible from the moment it was written —
///   which is the failure mode `crate::panels::forms::edit::apply`'s ★ comment
///   records for the fill precedent.
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
///
/// # ★ Why the error type is generic, and it is one word of generality
///
/// It took `pdfce_core::edit::EditError` for the whole of its life, because
/// every verb that came through it — delete, the three moves, `add_markup`,
/// `add_dimension` — reports that type. The two text verbs do not:
/// `EditSession::edit_text` reports `text_edit::EditError` and
/// `EditSession::add_text` reports `text_edit::AddTextError`, and neither
/// converts into the first.
///
/// The three ways out were: a second copy of the four-step protocol for each new
/// error type, which is the exact thing this function exists to prevent; a
/// `map_err` to a string at every call site, which would put the *formatting* of
/// a refusal in five places; or one bound. The bound is `Display`, which is the
/// only capability the error branch below actually uses — it puts the message on
/// the trace and declines. Nothing here inspects a variant, so nothing here
/// needed to know the type.
fn vector_edit<E: std::fmt::Display>(
    doc: &mut OpenDoc,
    label: &str,
    page: usize,
    operands: usize,
    edit: impl FnOnce(&mut EditSession) -> Result<Vec<String>, E>,
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
            // ★ **Step 5, added when the page verbs landed** — see
            // `super::pages`' header, which carries the whole argument and the
            // table of what each kind of edit invalidates.
            //
            // Here rather than in the four page arms, because `Action::Undo`
            // and `Action::Redo` come through this same function and run those
            // same engine commands **backwards**: an undone page delete puts
            // sheets back, and an arm-side resync could not see it. This is the
            // one place every document change already passes through, which is
            // `HANDOFF.md` §6's rule applied to a consequence rather than to a
            // dispatch.
            //
            // It is self-describing rather than told — it compares the page
            // vector it has against the one the session now reports — so an
            // edit that touched no page costs one page-tree walk and one `Vec`
            // comparison, per operator gesture, and does nothing else.
            super::pages::resync(doc);
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
            // ★ Surfaced as well as traced — see this function's "The
            // disclosures" section. Stamped with the epoch bumped above: the
            // revision on screen from now until the next edit, so an undo
            // retires the sentence by moving past it.
            //
            // AFTER the trace, which is what lets the list travel by MOVE
            // rather than by clone: `crate::diag::trace` runs its closure only
            // when `PDFCE_DIAG` is set, and that closure only *borrows*
            // `disclosures` to join it. Recording first would have meant
            // cloning a vec on every edit to keep both readers fed.
            record_edit_disclosure(if disclosures.is_empty() {
                // The overwhelmingly common case: the surgery expressed the
                // operator's request without changing anyone's form, so there
                // is nothing to disclose and the previous edit's sentence —
                // already stale by its epoch — is dropped outright.
                None
            } else {
                Some(EditDisclosure {
                    epoch: doc.edit_epoch,
                    notes: disclosures,
                })
            });
        }
        // A refusal is the engine's, and it is structured. Reporting it and
        // leaving the document alone is still the whole response here — and
        // as of 2026-08-14 that is a *scope* statement rather than the "there
        // is nowhere to say it" this comment used to make. There is now
        // somewhere: `app::status` draws the `Ok` arm's disclosure list.
        //
        // A refusal is deliberately not routed to it, because the two are
        // different acts. A disclosure is **after the fact** — the edit
        // happened, and the operator is owed the part they cannot see. A
        // refusal is a **decline**: nothing happened, and the sentence has to
        // arrive while the operator still believes it did. Sharing one slot
        // would mean an undone gesture and a completed one wearing the same
        // wording in the same place, which is worse than the trace-only state
        // it replaced. That is `FEATURES.md`'s "Worded decline" row, which
        // wants its own decision about wording and placement; this arm is
        // where it lands when it is taken.
        //
        // Note also that `EditError` is `Display` output — diagnostic prose an
        // error writes about itself — and `check-ui-strings.sh`'s exclusion 3
        // says in as many words that this exclusion "is not permission to
        // route UI text through an error type". So wording a decline is
        // catalog work in `text/`, not a `format!` of this value.
        Err(error) => crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("{label}-refused page={page} n={operands} detail={error}")
        }),
    }
}

// ---------------------------------------------------------------------------
// Undo and redo — one function, one direction parameter
// ---------------------------------------------------------------------------

/// Which end of the command log a step moves.
///
/// An enum rather than a `bool`: `history_step(doc, true)` at a call site says
/// nothing, and the two call sites are one line apart in the same `match`,
/// which is exactly the distance at which a transposition survives review.
///
/// **Named `Direction` rather than `History`** because
/// `crate::app::status::decline::History` already means something else one
/// module away — *what the two stacks currently hold* — and two types with one
/// name in one crate is a grep that answers the wrong question. This one is a
/// direction of travel; that one is a state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Direction {
    /// Take back the most recent command — `edit.undo`, `Ctrl+Z`.
    Undo,
    /// Re-apply the most recently undone one — `edit.redo`, `Ctrl+Y` /
    /// `Ctrl+Shift+Z`.
    Redo,
}

impl Direction {
    /// What a step **would** move, without moving it.
    ///
    /// `EditSession::undo_kind`/`redo_kind`, which take `&self` — so this is
    /// askable before the render worker is stopped and before `Arc::get_mut` is
    /// attempted, which is what lets an empty stack be declined without paying
    /// for a cancelled raster. `None` is the empty stack, and it is the same
    /// answer `can_undo`/`can_redo` give, from the same field.
    fn peek(self, session: &EditSession) -> Option<pdfce_core::edit::CommandKind> {
        match self {
            Self::Undo => session.undo_kind(),
            Self::Redo => session.redo_kind(),
        }
    }

    /// Move the log by one command.
    fn step(self, session: &mut EditSession) -> Option<pdfce_core::edit::CommandKind> {
        match self {
            Self::Undo => session.undo(),
            Self::Redo => session.redo(),
        }
    }

    /// The trace event naming the request: `undo` / `redo`.
    fn event(self) -> &'static str {
        match self {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            Self::Undo => "undo",
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            Self::Redo => "redo",
        }
    }

    /// [`vector_edit`]'s label — the event naming what the engine did.
    ///
    /// Distinct from [`Self::event`] on purpose, and it is the same two-line
    /// vocabulary `markup-commit` / `add-markup` already uses: the first line is
    /// **the shell decided**, and carries the `CommandKind`; the second is
    /// **the engine did it**, and carries the epoch. A harness that wants to
    /// know whether the caches were invalidated reads the second, and a harness
    /// that wants to know what the operator took back reads the first.
    fn applied(self) -> &'static str {
        match self {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            Self::Undo => "undo-applied",
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            Self::Redo => "redo-applied",
        }
    }

    /// The worded decline for an empty stack.
    fn declined(self) -> crate::app::status::decline::Declined {
        use crate::app::status::decline::Declined;
        match self {
            Self::Undo => Declined::NothingToUndo,
            Self::Redo => Declined::NothingToRedo,
        }
    }
}

/// **Move the command log by one, as the document change it is.**
///
/// The whole of [`Action::Undo`] and [`Action::Redo`].
///
/// # ★ Why this goes through [`vector_edit`] rather than doing the four steps
/// itself
///
/// Because an undo **is** an edit, and the only thing that distinguishes it
/// from a delete or a markup is which engine verb runs in step 2. Every reason
/// the protocol exists applies here unchanged:
///
/// | step | why an undo needs it |
/// |---|---|
/// | cancel the render worker | `EditSession::undo` takes `&mut self`, and `OpenDoc::session` is an `Arc` a rasterizing worker holds a clone of. Without the cancel, `Arc::get_mut` returns `None` **whenever the page happens to be rendering** — an undo that works or does not depending on how fast the sheet drew |
/// | mutate through `Arc::get_mut` | the same soundness argument, from the other end |
/// | bump `edit_epoch` | ★ **the step that makes the undo visible.** Every epoch-keyed cache — the page decomposition, the page-text extraction, the font inventory, the canvas selection's resolution, the Objects panel's count — believes it still describes the document until this moves. An undo that skipped it would restore the bytes and leave the operator looking at the state they just took back |
/// | drop the cached texture | `settle_and_rasterize` keys the page texture on the page index and the raster scale, and an undo changes neither, so nothing else would notice. This is what re-rasters the page |
///
/// Writing those four again here would be the fifth hand-written copy of a
/// protocol whose entire reason for existing is that hand-written copies omit
/// steps. The rule is `HANDOFF.md` §6's: one choke point.
///
/// # The disclosure list is empty, and that is a statement
///
/// The vector verbs return prose when the surgery had to change an operator's
/// *form* to express their request. An undo restores recorded `before` values —
/// it changes no form that was not already changed and disclosed when the
/// original command ran — so there is nothing new to disclose, and the empty
/// list makes [`vector_edit`] drop the **previous** edit's sentence, which is
/// exactly right: that sentence described a revision the operator has just left.
///
/// # Why the empty stack is checked HERE and not in the dispatcher
///
/// The dispatcher's arms route (`HANDOFF.md` §6). This function has to ask the
/// session what is on the log before it can act anyway, so asking in both
/// places would be two spellings of one question — and the one that drifted
/// would produce a control that is greyed while the bar says something else.
/// The decline is recorded through `crate::app::status::decline`, in the apply
/// phase, exactly as `crate::app::save`'s failure is and for the reason that
/// module's own call site documents: `decline::retire` runs at the *top* of
/// `dispatch_command`, so a sentence recorded here survives the frame that
/// raised it.
///
/// # What `page=` means on the trace line, and why it is not a lie
///
/// [`vector_edit`]'s `page` and `operands` exist so the trace can say which
/// verb ran over what. An undo is the one caller whose operands are **not**
/// page-scoped: a `CommandKind` may be a page rotation, a document-level
/// attachment or a form field, and the engine's command log does not carry a
/// page at all. What is passed is therefore the page **on screen**, and the
/// `undo` line above it carries the `CommandKind`, which is the field that
/// actually says what moved. A reader who wants the page an undo touched has to
/// read the kind; there is no honest number to put here, and inventing a
/// sentinel would be a second thing to explain.
fn history_step(doc: &mut OpenDoc, direction: Direction) {
    let event = direction.event();
    let Some(kind) = direction.peek(&doc.session) else {
        // ★ Unreachable from a control and reachable from a chord. See
        // `Declined::NothingToUndo`: the QAT button is greyed by
        // `undo.available`, and `Ctrl+Z` is offered in every mode because the
        // command is on no tab, so this is the keyboard's path and it is the
        // commonest keystroke in editing. It is both traced and worded — the
        // trace for whoever reads a run from a machine they cannot see, the
        // sentence for the operator who is looking at the page rather than at
        // an 18 pt icon.
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("{event}-declined reason=empty-stack")
        });
        crate::app::status::decline::record_history_empty(direction.declined());
        return;
    };
    // Before the mutation, so the depth is the one the operator is acting on
    // and the kind is the one they asked to move. Both come from `peek`, which
    // reads the same slot `step` is about to pop.
    let depth = doc.session.undo_depth();
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("{event} kind={kind:?} undo_depth={depth}")
    });

    let page = doc.view.page_index;
    vector_edit(doc, direction.applied(), page, 1, |session| {
        // `peek` answered `Some` against this same session and nothing has run
        // between then and here but `Arc::get_mut`, so `None` is unreachable.
        // It is dropped rather than unwrapped because a panic in the apply
        // phase loses the operator's document, and because the honest report of
        // a step that moved nothing is the one `vector_edit` already makes: an
        // epoch bump and an empty disclosure list.
        let _ = direction.step(session);
        // The turbofish is the price of `vector_edit`'s generic error type, and
        // it is paid here alone: this is the one caller whose closure never
        // fails, so it is the one place `E` is unconstrained. Named as the
        // engine's own error rather than as `Infallible`, because that is the
        // type every *other* verb reaching this function reports and a reader
        // comparing the arms should not have to notice a second one.
        Ok::<_, EditError>(Vec::new())
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::actions::last_edit_disclosure;
    use crate::app::state::{FOUR_PAGES, open_fixture};

    /// ★ **An undo is an edit, and moves the epoch like one — while an undo
    /// with nothing to undo moves nothing at all.**
    ///
    /// # The two failures this pins, and why neither is visible anywhere else
    ///
    /// 1. **The epoch.** A build whose history arm called `EditSession::undo`
    ///    directly — `Arc::get_mut(&mut doc.session).map(EditSession::undo)`,
    ///    which is the obvious three-line version — would restore the bytes and
    ///    leave `edit_epoch` where it was. Every count anybody could read from
    ///    the engine would then be correct, and the decomposition, the
    ///    page-text cache, the font inventory and the canvas selection would all
    ///    go on describing the revision the operator just left. That is the
    ///    build `tools/ui-verify`'s `undo_redo_round_trip` catches from outside
    ///    the process; this is the half that can be caught from inside it.
    /// 2. **The empty stack.** The decline must cost nothing: no epoch bump, no
    ///    dropped texture, no cancelled raster. A bump here would dissolve the
    ///    operator's selection and discard several caches to record that
    ///    *nothing happened* — which is `crate::app::save` §3.1's argument
    ///    about a save, arriving at the same answer from the other direction.
    ///
    /// # Why `is_modified` is asserted as well as the epoch
    ///
    /// Because the epoch alone cannot tell an undo from any other edit: it
    /// counts revisions, and it only ever goes up. `EditSession::is_modified`
    /// asks the **dirty set**, which is the same question a save asks, and its
    /// own doc comment says in as many words that *"an edit-then-undo reports
    /// `false`"*. So it is the one available proof that the document really
    /// went back rather than merely forward again — and the pair of them
    /// together is the whole claim: *the document is where it started, and the
    /// shell knows the revision changed.*
    #[test]
    fn an_undo_is_an_edit_and_moves_the_epoch_like_one() {
        use crate::canvas::markup::{Geometry, MarkupKind};

        let mut doc = open_fixture(FOUR_PAGES);
        let opened_at = doc.edit_epoch;

        // --- an empty log costs nothing ------------------------------------
        assert!(!doc.session.can_undo(), "the fixture opens with no history");
        history_step(&mut doc, Direction::Undo);
        history_step(&mut doc, Direction::Redo);
        assert_eq!(
            doc.edit_epoch, opened_at,
            "a history step with an empty stack must not bump the epoch — it would discard the \
             decomposition, the page-text cache and the operator's selection to record that \
             nothing happened"
        );
        assert!(!doc.session.is_modified(), "and must change no bytes");

        // --- one real edit, through the funnel every gesture uses -----------
        let spec = crate::canvas::markup::spec(
            MarkupKind::Rectangle,
            &Geometry::Band {
                start: (100.0, 100.0),
                end: (200.0, 160.0),
            },
        )
        .expect("a band is the Rectangle kind's own geometry"); // ui-text-exempt: test panic
        vector_edit(&mut doc, "add-markup", 0, 1, |session| {
            session.add_markup(0, &spec).map(|_| Vec::new())
        });
        let authored_at = doc.edit_epoch;
        assert_ne!(
            authored_at, opened_at,
            "the fixture edit did not take, so nothing below is testing what it says"
        );
        assert!(doc.session.is_modified(), "the document now differs");
        assert!(doc.session.can_undo());
        assert!(
            !doc.session.can_redo(),
            "authoring something is not a reason to offer a redo"
        );

        // --- ★ the undo ----------------------------------------------------
        history_step(&mut doc, Direction::Undo);
        assert_ne!(
            doc.edit_epoch, authored_at,
            "★ THE UNDO DID NOT BUMP THE EPOCH. The annotation is off the session and every \
             epoch-keyed cache still describes the revision that had it — so the canvas would go \
             on drawing the rectangle that was just taken back. See `vector_edit` step 3"
        );
        assert!(
            !doc.session.is_modified(),
            "★ the undo did not restore the document: the dirty set a save would write is still \
             non-empty"
        );
        assert!(!doc.session.can_undo(), "the log's only entry was consumed");
        assert!(doc.session.can_redo(), "…and is now redoable");

        // --- and back again ------------------------------------------------
        let undone_at = doc.edit_epoch;
        history_step(&mut doc, Direction::Redo);
        assert_ne!(doc.edit_epoch, undone_at, "a redo is an edit too");
        assert!(
            doc.session.is_modified(),
            "the redo did not re-apply the annotation"
        );
        assert!(doc.session.can_undo());
        assert!(!doc.session.can_redo());
    }

    /// ★ **A disclosure a verb returns is live for the revision that verb
    /// produced** — the wiring, driven rather than planted.
    ///
    /// [`plant_edit_disclosure_for_test`] proves the status bar can *draw* a
    /// disclosure. It cannot prove [`vector_edit`] ever *records* one, and it
    /// cannot prove the stamp is right — which is the failure this test
    /// exists for, because that failure is silent in both directions:
    ///
    /// - Stamp the epoch the edit ran **against** (the pre-bump value) and the
    ///   sentence is invisible from the moment it is written. Nothing errors,
    ///   no test that plants its own value notices, and the operator simply
    ///   never learns their rectangle became four lines.
    /// - Fail to record at all and the same thing happens, with the trace
    ///   still cheerfully printing `disclosures=…` — which is exactly the
    ///   "recorded, not disclosed" state this work was written to end.
    ///
    /// So the edit closure here returns a disclosure list the way a real
    /// `move_node` over an `re` rectangle does, and the assertion is made
    /// against the epoch the *document* ends up on, read back through the
    /// public accessor the bar uses.
    #[test]
    fn a_verbs_disclosure_is_live_for_the_revision_the_edit_produced() {
        record_edit_disclosure(None);
        let mut doc = open_fixture(FOUR_PAGES);
        let before = doc.edit_epoch;

        vector_edit(&mut doc, "move-node", 0, 1, |_session| {
            // The turbofish is `vector_edit`'s generic error type, named as the
            // engine's own for the reason the undo caller's is — see there.
            Ok::<_, EditError>(vec!["This shape was stored as a rectangle.".to_owned()])
        });

        assert_ne!(
            doc.edit_epoch, before,
            "the edit did not bump the epoch, so nothing below is testing what it says"
        );
        let live = last_edit_disclosure(doc.edit_epoch);
        assert!(
            live.is_some(),
            "the verb's disclosure is not live for the revision now on screen \
             (epoch {before} → {}); the bar would draw nothing and the operator \
             would learn about the rewrite from a diff",
            doc.edit_epoch
        );
        assert_eq!(
            live.expect("asserted live one line above").notes,
            vec!["This shape was stored as a rectangle.".to_owned()],
            "core's sentence must reach the store unaltered"
        );
        assert!(
            last_edit_disclosure(before).is_none(),
            "the disclosure was stamped with the revision the edit ran AGAINST rather \
             than the one it produced, which makes it invisible from the moment it is \
             written"
        );

        // A second edit that discloses nothing retires the first sentence —
        // both by the epoch and by clearing the slot outright.
        vector_edit(&mut doc, "move-node", 0, 1, |_session| {
            Ok::<_, EditError>(Vec::new())
        });
        assert!(
            last_edit_disclosure(doc.edit_epoch).is_none(),
            "an edit with nothing to disclose must leave no sentence behind"
        );
        record_edit_disclosure(None);
    }

    /// ★ **A disclosure is shown only while it describes the revision on
    /// screen.**
    ///
    /// The staleness rule, and the whole reason nothing anywhere has to
    /// remember to clear this sentence: an undo bumps the epoch, the epoch no
    /// longer matches, and the bar stops drawing it. The comparison IS the
    /// mechanism, so it is pinned rather than trusted — the same test, for the
    /// same reason, as
    /// `crate::panels::forms::edit::tests::a_disclosure_is_hidden_once_the_document_moves_past_it`.
    ///
    /// Both directions matter and both are asserted. A *later* revision must
    /// not show a note about an earlier one (the undo case, and the ordinary
    /// "they carried on editing" case). An *earlier* one must not either —
    /// that pairing is unreachable through `vector_edit`, which only ever
    /// stamps the epoch it just produced, and it is asserted anyway because
    /// the filter is what makes it unreachable.
    #[test]
    fn a_disclosure_is_hidden_once_the_document_moves_past_it() {
        record_edit_disclosure(Some(EditDisclosure {
            epoch: 7,
            notes: vec!["This shape was stored as a rectangle.".to_owned()],
        }));
        assert!(last_edit_disclosure(7).is_some());
        assert!(
            last_edit_disclosure(8).is_none(),
            "a later revision must not show a note about an earlier one"
        );
        assert!(last_edit_disclosure(6).is_none());

        // An edit that disclosed nothing draws no sentence, at any epoch.
        // `vector_edit` records `None` for this case rather than an empty
        // list, so the filter here is belt and braces — and it is exactly the
        // belt that stops an empty line appearing under every drag, which
        // would train the operator to ignore the ones that matter.
        record_edit_disclosure(Some(EditDisclosure {
            epoch: 7,
            notes: Vec::new(),
        }));
        assert!(
            last_edit_disclosure(7).is_none(),
            "an empty disclosure must draw no sentence"
        );
        record_edit_disclosure(None);
    }
}
