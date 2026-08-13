//! # `canvas::selection` — selection as IDENTITY, and the invariant it exists to hold
//!
//! ## ★ The invariant, stated first because everything here is shaped by it
//!
//! `GUI_ROADMAP.md` Phase 1, from the operator's own words on 2026-08-13:
//!
//! > *"if I select a node or something for a tool, I should be able to pan
//! > and zoom out without losing my first selection."*
//! >
//! > **Navigation is not an edit. Panning, zooming, changing fit mode,
//! > rotating the view, switching page-display mode and changing ribbon tab
//! > must never alter the selection.**
//!
//! The roadmap names **three** ways the natural implementation loses it, each
//! of which looks reasonable in isolation. This module closes all three, and
//! each closure is a structural property rather than a promise:
//!
//! | # | The way it is lost | What closes it here |
//! |---|---|---|
//! | 1 | **Selection stored in screen coordinates.** Zoom changes the mapping, so the stored point stops naming the thing it named. | [`Selection`] holds **no coordinate of any kind**. It is `page + object + subpath + node`, four integers, none of which a zoom can touch. There is no constructor that takes a `Pos2`. |
//! | 2 | **Selection cleared by a click that was really a drag.** A gesture begins with a press; if press-on-empty clears, every drag that starts on blank paper destroys the selection. | Nothing in this module is called on a press. The clear is driven by [`SelectionState::click`], which [`crate::canvas::gesture`] raises only for a **completed click with no drag**. |
//! | 3 | **Selection invalidated by re-decomposition.** The provider rebuilds on page change and on edit; a rebuild triggered by zoom, or by a page change that is not a page change in the operator's sense, must not drop it. | [`SelectionState::resolve`] **re-resolves against the new decomposition** instead of discarding, and — the part that is easy to get wrong — it only validates entries **on the page the provider serves**. An entry for another page is left completely alone. |
//!
//! Row 3's second half is the one that makes the acceptance criterion pass:
//! *"select a node, zoom out three rungs, pan across the sheet, switch to
//! Continuous, come back, switch ribbon tab — the node is still selected and
//! still the entered level."* Going to another page builds a provider for
//! that page, and a `resolve` that pruned everything it could not find would
//! wipe the selection on the way past. Coming back would find nothing.
//!
//! ## Why paint-order index is the identity, and what it does not survive
//!
//! `Selection::object` is a [`TargetId`], which is the object's index into
//! `PageObjects::objects` — **paint order**. It is the same number
//! `pdfce-cli object-list` prints and `object-delete` takes, so "object 412"
//! means one thing across every surface. That is what makes it usable as an
//! identity here.
//!
//! It is an identity **within one revision of one page**, and no further.
//! Deleting an object renumbers every object painted after it. So an edit
//! moves the meaning of a retained index, and this module's honest position
//! is to say so rather than to pretend otherwise:
//!
//! - A rebuild with the **same** revision (the zoom / page-return / panel
//!   case, which is the invariant's whole subject) re-resolves exactly.
//! - A rebuild after an **edit** re-resolves against the new decomposition
//!   and drops what no longer exists; indices that *shifted* silently name
//!   their new neighbour. Closing that needs a stable per-object token from
//!   `pdfce-core`, which does not exist — `decompose_page` mints indices, not
//!   identities. It is recorded here rather than in a comment nobody reads,
//!   and it is a boundary finding for the engine, not a shortcut taken here.
//!
//! ## Why the level is state and not derived
//!
//! [`SelectionLevel`] could be inferred from whether `subpath`/`node` are
//! `Some`. It is stored instead, because *"inside this object, nothing picked
//! yet"* is a real state — reached by entering an object at a point where no
//! subpath was close enough — and an inferred level would collapse it into
//! "not inside anything at all". The operator would then find Escape taking
//! two presses on one path and one on another, for no reason they could see.
//!
//! ## What this module deliberately does NOT do
//!
//! It never draws, never touches egui, never reads a pointer, and never
//! reaches a document. It is a state machine over four integers and a
//! provider trait, which is precisely why every invariant above can be
//! asserted in a unit test rather than hoped for in a running window.

use std::collections::BTreeSet;

use egui::Rect;

use crate::canvas::target::{CanvasTargetProvider, TargetId};

/// One selected thing, addressed by **identity** and never by position.
///
/// Four integers, and the shape is `GUI_ROADMAP.md`'s — *"page, object index,
/// sub-path, node"*. Enough to re-resolve against a fresh decomposition, and —
/// the point — containing nothing a zoom, a pan or a fit mode could
/// invalidate.
///
/// `Ord` so a selection set has a stable, reviewable order and so a
/// [`BTreeSet`] can de-duplicate it. The ordering is `(page, object, subpath,
/// node)`, i.e. document order first, which is also the order the outlines
/// are painted in — a multi-select that painted in click order would
/// re-stack its outlines whenever the operator shift-clicked, which reads as
/// flicker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Selection {
    /// The 0-based page index the object lives on.
    ///
    /// Carried even though the canvas draws one page today, because it is
    /// what lets a selection survive navigating away and back — the case the
    /// acceptance criterion turns on — and because `GUI_ROADMAP.md` Phase 4
    /// puts several pages on screen at once.
    pub page: usize,
    /// The object, by paint-order index (see the module docs on identity).
    pub object: TargetId,
    /// The entered part — a path's subpath or a text object's show-operator
    /// run — if the operator has descended one rung.
    ///
    /// `None` means "the whole object", which is a different statement from
    /// "part 0".
    pub subpath: Option<usize>,
    /// The entered anchor, **object-scoped**, if the operator has descended
    /// two rungs.
    ///
    /// Object-scoped rather than part-scoped because that is the space
    /// `vector::anchor_count` reports and `pdfce-cli node-move --node N`
    /// addresses; a second numbering would make the number pdfce shows
    /// disagree with the number the operator can act on.
    ///
    /// `Some` implies `subpath.is_some()`: there is no way to pick a point
    /// without being inside the part that holds it. [`SelectionState`] is the
    /// only thing that constructs these and it maintains that.
    pub node: Option<usize>,
}

impl Selection {
    /// A whole-object selection.
    #[must_use]
    pub fn object(page: usize, object: TargetId) -> Self {
        Self {
            page,
            object,
            subpath: None,
            node: None,
        }
    }
}

/// Which rung of the selection ladder the operator has entered.
///
/// Three rungs, and the ladder is the vector-editor convention the operator
/// asked for: double-click descends, Escape ascends **one rung per press**.
///
/// The cap is structural rather than checked. A text object decomposes into
/// runs, and a run has no anchors, so
/// [`CanvasTargetProvider::nearest_node`] can never return a node for one —
/// the ladder stops at two rungs for text without a special case anywhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum SelectionLevel {
    /// Whole objects. The rung a click starts on and Escape returns to.
    #[default]
    Object,
    /// Inside one object, selecting its parts — a path's subpaths or a text
    /// object's runs.
    ///
    /// This rung exists because a PDF path object can hold an entire drawing:
    /// one measured CAD export has **1,194 subpaths in a single object**, so
    /// "the object under the pointer" is usually not the thing the operator
    /// means.
    Part,
    /// Inside one part, selecting its anchors.
    ///
    /// Scoped to the entered part deliberately: the same measured export has
    /// one object holding **6,681 anchors**, and offering all of them as a
    /// grab target is what made the old ungated gesture unpredictable — the
    /// nearest anchor to a press could easily belong to a subpath the
    /// operator was not pointing at, with nothing drawn to say which.
    Node,
}

impl SelectionLevel {
    /// The rung one step up, or `None` at the top.
    #[must_use]
    pub fn ascend(self) -> Option<Self> {
        match self {
            Self::Object => None,
            Self::Part => Some(Self::Object),
            Self::Node => Some(Self::Part),
        }
    }
}

/// What one press of Escape did — reported rather than silently absorbed.
///
/// The caller traces it and, in the `Nothing` case, is free to let Escape
/// fall through to whatever else owns the key. Returning a value rather than
/// a `bool` is what keeps *"Escape ascends exactly one rung"* assertable:
/// a test can press Escape three times and check the three outcomes in
/// order, which a `bool` could not distinguish from one press that collapsed
/// the whole ladder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EscapeOutcome {
    /// Left the entered rung, returning to the one above. The selection is
    /// **not** cleared — that is the next press's job.
    LeftLevel(SelectionLevel),
    /// Was at the Object rung with something selected: cleared it.
    ClearedSelection,
    /// Nothing was selected and no rung was entered. The canvas did not
    /// consume the key.
    Nothing,
}

/// What the provider found under a completed click, at every rung at once.
///
/// Assembled by the canvas — which owns the provider and the coordinate
/// conversion — and handed here as plain integers, so [`SelectionState::click`]
/// is a pure function of "what is there" and "where am I" with no geometry in
/// it. Every branch of the ladder is then testable without a document, a
/// decomposition or an egui frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ClickHit {
    /// The front-most object under the pointer, if any.
    pub object: Option<TargetId>,
    /// The nearest part of the **entered** object, if the click was inside
    /// one and a part was within tolerance.
    pub part: Option<usize>,
    /// The nearest anchor of the **entered** part, object-scoped.
    pub node: Option<usize>,
}

/// The whole of the canvas's selection state.
///
/// # ★ Where this lives, and why that is the whole of its document scoping
///
/// It is a field of `crate::app::state::OpenDoc` — the open document itself.
/// That is not filing: it is the mechanism, and it replaced one.
///
/// A selection is document-scoped state, so closing a document must forget it.
/// Until this stage the value lived in `egui::Memory`, which outlives
/// documents, so the canvas had to *detect* the change: a `DocumentToken`
/// built from the `Arc<EditSession>`'s allocation address mixed with the page
/// count, compared on every frame by a `sync_document` method that reset
/// everything when it moved. Both are now **deleted**, along with the
/// residual hazard they carried — an address is not an identity, and a reused
/// allocation with a matching page count would have carried a stale selection
/// into a new file, while holding an `Arc` or a `Weak` to make it a real
/// identity would have disabled editing outright (`Arc::get_mut` fails while
/// any other strong **or weak** reference exists).
///
/// What replaced them is `OpenDoc::new`'s own doc comment: *"opening a
/// document constructs a whole new `OpenDoc`, so a cached texture or a page
/// index can never refer to a page from a previous file."* A selection held
/// inside that structure inherits the guarantee by construction, on every
/// frame, at no cost, with nothing to compare. `panels::DocKey` and the
/// decomposition cache went the same way in the same stage, for the same
/// reason.
///
/// **A page change is still not a document change**, and never was — that is
/// invariant 3, and it is [`Self::resolve`]'s business, not this note's.
///
/// # Why it caches canvas-space outlines
///
/// Drawing the selection every frame needs each entry's bounds, and bounds
/// come from a decomposition. `decompose_page` resolves every `/Contents`
/// stream, inflates it, concatenates, tokenizes and walks the whole token
/// stream resolving fonts as it goes, with **no cache anywhere in
/// `pdfce-core`** — so asking for it per frame is not an option.
///
/// Canvas-space bounds are the right thing to cache because they are
/// **zoom-independent**: canvas space is the page's device space at scale
/// 1.0, so a zoom or a pan changes where the outline is *drawn* and not what
/// it *is*. The cache is therefore keyed on `(page, edit epoch)` and survives
/// every navigation — which is the invariant again, from the drawing side.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SelectionState {
    /// The selected entries, in `(page, object, subpath, node)` order.
    entries: Vec<Selection>,
    /// The rung the operator has entered.
    level: SelectionLevel,
    /// Canvas-space outline rects for the entries on the resolved page, in
    /// the order they should be painted.
    outlines: Vec<(Selection, Rect)>,
    /// The `(page, edit epoch)` [`Self::outlines`] describes, or `None`
    /// before the first resolve.
    resolved_for: Option<(usize, u64)>,
}

impl SelectionState {
    /// The selected entries, in document order.
    #[must_use]
    pub fn entries(&self) -> &[Selection] {
        &self.entries
    }

    /// Whether anything is selected.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// How many entries are selected — the `sel=` the diagnostic trace
    /// reports, and the number `ui-verify` reads to tell a click that landed
    /// from one that did not.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// The rung the operator has entered.
    #[must_use]
    pub fn level(&self) -> SelectionLevel {
        self.level
    }

    /// The object the operator is inside, if any.
    ///
    /// Derived rather than stored, and safe to derive because every path that
    /// sets a level above [`SelectionLevel::Object`] also collapses the
    /// entries onto one object — a rung is a place *inside one thing*.
    #[must_use]
    pub fn entered_object(&self) -> Option<Selection> {
        (self.level != SelectionLevel::Object)
            .then(|| self.entries.first().copied())
            .flatten()
    }

    /// The canvas-space outlines to draw, with the entry each came from.
    ///
    /// Paired rather than a bare `Vec<Rect>` because the overlay needs to
    /// know which entry each box belongs to, and because [`Self::resolve`]
    /// drops entries the provider no longer knows — which breaks positional
    /// correspondence with [`Self::entries`].
    #[must_use]
    pub fn outlines(&self) -> &[(Selection, Rect)] {
        &self.outlines
    }

    /// The union of the current outlines, in canvas space — the box the
    /// resize grips are placed around.
    ///
    /// The union rather than the first entry's box, because a multi-select
    /// is one thing to act on: eight grips around one member of a set of
    /// five would say the gesture applies to that member alone.
    #[must_use]
    pub fn outline_union(&self) -> Option<Rect> {
        self.outlines
            .iter()
            .map(|(_, r)| *r)
            .reduce(|acc, r| acc.union(r))
    }

    /// The paint-order indices selected on `page`, ascending — the operand
    /// list for a batched edit.
    ///
    /// Ascending and de-duplicated because `EditSession::delete_objects`
    /// resolves **every** index before planning anything, so a duplicate or a
    /// stale entry refuses the whole call rather than deleting the prefix
    /// that happened to resolve. Handing it a clean list is the difference
    /// between "delete refused" and "delete did half of what I asked".
    #[must_use]
    pub fn object_indices_on(&self, page: usize) -> Vec<usize> {
        self.entries
            .iter()
            .filter(|e| e.page == page)
            .filter_map(|e| usize::try_from(e.object.0).ok())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    /// ★ The indices a **Delete** may act on for `page` — empty unless the
    /// operator is at the Object rung.
    ///
    /// # Why the rung guard lives here rather than at each call site
    ///
    /// Because there are now two call sites and they must not be able to
    /// disagree: the canvas's Delete/Backspace keys, and the ribbon's
    /// `format.delete` on the contextual Format tab (reached through
    /// `crate::app::PdfceApp::dispatch_token`). A rule stated twice is a rule
    /// that drifts, and the drift here is destructive rather than cosmetic.
    ///
    /// # ★ And why the guard is not caution
    ///
    /// At the Part or Node rung the selection names a subpath or an anchor
    /// *inside* one object, while the only verb wired to it is
    /// `EditSession::delete_objects`, which removes **whole objects**. Deleting
    /// the enclosing object because the operator asked to delete one line of it
    /// is exactly the class of error that cannot be excused by "they can undo
    /// it": one measured CAD export holds an entire drawing view as a single
    /// path object with 1,194 subpaths, so the difference between the two
    /// readings is one line and the whole view.
    ///
    /// `pdfce-core` has the verbs for the deeper rungs — `delete_subpath`,
    /// `delete_node` and `delete_text_run`, the last of which also needs the
    /// `ObjectModelProvider::text_run_delete_would_move_next` guard asked
    /// BEFORE the control is offered (R83). They are their own actions and
    /// their own change; refusing here is the honest interim.
    ///
    /// # Returns
    ///
    /// Ascending and de-duplicated — [`Self::object_indices_on`]'s contract,
    /// which is what `EditSession::delete_objects` needs in order to succeed
    /// rather than refuse the whole batch. Empty means *"nothing this verb may
    /// remove"*, which deliberately does **not** distinguish "nothing selected"
    /// from "selected at a rung with no delete verb": a caller that must tell
    /// those apart asks [`Self::level`], and the canvas does exactly that in
    /// order to trace the difference.
    #[must_use]
    pub fn deletable_objects_on(&self, page: usize) -> Vec<usize> {
        if self.level != SelectionLevel::Object {
            return Vec::new();
        }
        self.object_indices_on(page)
    }

    /// Apply a **completed click** — never a press.
    ///
    /// See invariant 2 in the module docs: a press that turns out to be a
    /// drag must leave the selection completely alone, so this is only ever
    /// reached from [`crate::canvas::gesture::GestureOutcome::Click`], which
    /// is raised on release and only when no drag happened.
    ///
    /// # The rules, and why each one
    ///
    /// - **Plain click, hit, at the Object rung** — replace the selection.
    /// - **Plain click, miss** — clear. Clicking empty paper deselects; that
    ///   is what every editor does, and the alternative strands an operator
    ///   with no way back to "nothing selected" except a key they have not
    ///   discovered.
    /// - **Shift+click, hit** — toggle that entry's membership, leaving the
    ///   rest alone. Toggle rather than add, so shift-clicking a selected
    ///   object is its own undo.
    /// - **Shift+click, miss** — unchanged. There is nothing to toggle, and
    ///   clearing here would make an over-shoot destroy a set the operator
    ///   spent five clicks building.
    /// - **Double-click, hit** — descend one rung into the object under the
    ///   pointer (the operator's stated model: *"double-click to get to the
    ///   next level down"*). A double-click at the Node rung changes nothing:
    ///   there is nothing below a point.
    /// - **Plain click while inside an object, hitting nothing in it** —
    ///   leave the rung and apply the ordinary Object-rung rule. Clicking
    ///   away is how every editor exits a group; staying inside until Escape
    ///   strands an operator who has forgotten they descended, which is the
    ///   failure a depth model must avoid above all.
    pub fn click(&mut self, page: usize, hit: ClickHit, shift: bool, double: bool) {
        if double {
            self.descend(page, hit);
            return;
        }
        match self.level {
            SelectionLevel::Object => self.click_at_object_rung(page, hit, shift),
            SelectionLevel::Part | SelectionLevel::Node => self.click_inside(page, hit, shift),
        }
    }

    /// Replace or extend the selection with a marquee's enclosed set.
    ///
    /// Always resolves to the **Object** rung, and ascends if the operator
    /// was inside one. A rubber-band names a region of the page, and a region
    /// contains objects; there is no sensible reading of "every subpath of
    /// some other object that this box happens to cover".
    ///
    /// Plain replaces, `Shift` adds. An empty plain marquee therefore clears,
    /// which is the Inkscape convention and is **not** the failure invariant
    /// 2 is about: that one is about a *press*, and this runs on release,
    /// after a real enclosure test. Panning is the middle button and never
    /// reaches here at all.
    pub fn marquee(&mut self, page: usize, hits: &[TargetId], shift: bool) {
        let found: Vec<Selection> = hits
            .iter()
            .map(|&object| Selection::object(page, object))
            .collect();
        if shift {
            self.entries.extend(found);
        } else {
            self.entries = found;
        }
        self.level = SelectionLevel::Object;
        self.normalise();
    }

    /// Ascend one rung, or clear, or decline the key. See [`EscapeOutcome`].
    ///
    /// **One press, one rung** — decision 025's L1. The old shell shipped
    /// Escape as "clear everything", so an operator who descended two rungs
    /// to reach a line and pressed Escape once found themselves back at the
    /// page. Collapsing the ladder in one press is exactly as wrong as
    /// requiring three presses to clear a single selection.
    pub fn escape(&mut self) -> EscapeOutcome {
        match self.level.ascend() {
            Some(SelectionLevel::Part) => {
                for entry in &mut self.entries {
                    entry.node = None;
                }
                self.level = SelectionLevel::Part;
                self.normalise();
                EscapeOutcome::LeftLevel(SelectionLevel::Part)
            }
            Some(SelectionLevel::Object) => {
                for entry in &mut self.entries {
                    entry.subpath = None;
                    entry.node = None;
                }
                self.level = SelectionLevel::Object;
                self.normalise();
                EscapeOutcome::LeftLevel(SelectionLevel::Object)
            }
            // `ascend()` never returns `Node`; the arm exists so adding a
            // fourth rung is a compile error here rather than a silent
            // fall-through to "clear the selection".
            Some(SelectionLevel::Node) => EscapeOutcome::Nothing,
            None if !self.entries.is_empty() => {
                self.entries.clear();
                self.outlines.clear();
                EscapeOutcome::ClearedSelection
            }
            None => EscapeOutcome::Nothing,
        }
    }

    /// ★ **Re-resolve against a fresh decomposition** — invariant 3.
    ///
    /// Called every frame; does real work only when `(page, epoch)` has
    /// moved, because that is the only time the answer can have changed. A
    /// zoom, a pan, a fit-mode change, a ribbon-tab change and a window
    /// resize all leave both halves of that key untouched, so they cost one
    /// comparison and change nothing — which is the invariant, enforced by
    /// the shape of the code rather than by a rule somebody has to remember.
    ///
    /// # Only the resolved page is validated, and that is the load-bearing part
    ///
    /// An entry on **another** page is left exactly as it was. The provider
    /// serves one page (`panels::objects::provider`, "Single-page by
    /// design"), so it has nothing to say about the others, and a `resolve`
    /// that pruned everything it could not find would wipe the selection the
    /// moment the operator paged away — and find nothing on the way back.
    /// That is the acceptance criterion, and it is this `if`.
    ///
    /// # What is dropped, and silently
    ///
    /// An entry whose object the provider no longer knows, i.e. one an edit
    /// removed. Dropping it is not a fact the operator needs disclosed: they
    /// deleted it. Keeping it would leave a selection naming a hole, and the
    /// next Delete would refuse the whole batch because
    /// `EditSession::delete_objects` resolves every index before planning.
    /// # `None` means "this page has no object model", not "nothing is selected"
    ///
    /// A page whose content streams will not decode has no decomposition, and
    /// the honest response is to draw no outlines while keeping the
    /// selection — the two states are different, and conflating them would
    /// make an undecodable page silently deselect. The `(page, epoch)` key is
    /// still recorded, so the failed decomposition is **not retried every
    /// frame**: the failure is deterministic (same bytes, same code), which is
    /// the same argument `PanelsState::provider_built` and
    /// `settle_and_rasterize`'s render-error hold both make.
    pub fn resolve(&mut self, targets: Option<&dyn CanvasTargetProvider>, page: usize, epoch: u64) {
        if self.resolved_for == Some((page, epoch)) {
            return;
        }
        self.resolved_for = Some((page, epoch));
        let Some(targets) = targets else {
            self.outlines.clear();
            return;
        };

        // Drop only entries ON THIS PAGE that no longer resolve.
        self.entries
            .retain(|e| e.page != page || targets.bounds(page, e.object).is_some());
        if self.entries.is_empty() {
            self.level = SelectionLevel::Object;
        }

        self.outlines = self
            .entries
            .iter()
            .filter(|e| e.page == page)
            .filter_map(|e| Some((*e, self.outline_rect(targets, e)?)))
            .collect();
    }

    /// Whether [`Self::resolve`] would do any work for `(page, epoch)`.
    ///
    /// The canvas asks **before** building a decomposition, because building
    /// one is the expensive half: `decompose_page` inflates, concatenates,
    /// tokenizes and walks every content stream on the page with no cache
    /// anywhere in `pdfce-core`. A zoom, a pan, a fit change and a ribbon-tab
    /// change all leave both halves of the key untouched, so on the
    /// overwhelming majority of frames this is `false` and nothing is built
    /// at all.
    #[must_use]
    pub fn needs_resolve(&self, page: usize, epoch: u64) -> bool {
        self.resolved_for != Some((page, epoch))
    }

    /// The canvas-space rect to outline for one entry: the **part's** box
    /// once the operator is inside one, the object's box otherwise.
    ///
    /// Falling back to the object's box when a part has no bounds is
    /// deliberate — the alternative is drawing nothing for a selection that
    /// exists, and a correct action with no feedback is indistinguishable
    /// from a broken one.
    fn outline_rect(&self, targets: &dyn CanvasTargetProvider, entry: &Selection) -> Option<Rect> {
        let object = usize::try_from(entry.object.0).ok()?;
        entry
            .subpath
            .and_then(|part| targets.part_bounds(entry.page, object, part))
            .or_else(|| targets.bounds(entry.page, entry.object))
    }

    /// A plain or shift click while at the Object rung.
    fn click_at_object_rung(&mut self, page: usize, hit: ClickHit, shift: bool) {
        match (shift, hit.object) {
            (false, Some(object)) => self.entries = vec![Selection::object(page, object)],
            (false, None) => self.entries.clear(),
            (true, Some(object)) => {
                let entry = Selection::object(page, object);
                if let Some(at) = self.entries.iter().position(|e| *e == entry) {
                    self.entries.remove(at);
                } else {
                    self.entries.push(entry);
                }
            }
            (true, None) => {}
        }
        self.normalise();
    }

    /// A plain or shift click while inside an object.
    ///
    /// Three outcomes, in precedence order: re-pick at the current rung; fall
    /// back one rung and re-pick there; or leave the object entirely and
    /// behave like an ordinary Object-rung click. The middle case is what
    /// stops an operator being stranded at a rung whose targets they keep
    /// missing — at the Node rung, a click that misses every anchor but lands
    /// on a part ascends to that part rather than doing nothing.
    fn click_inside(&mut self, page: usize, hit: ClickHit, shift: bool) {
        let Some(entered) = self.entered_object() else {
            // No entry to be inside of: the level and the entries disagreed,
            // which `normalise` prevents. Recover rather than panic.
            self.level = SelectionLevel::Object;
            self.click_at_object_rung(page, hit, shift);
            return;
        };
        let same_object = hit.object == Some(entered.object) && page == entered.page;

        if same_object && self.level == SelectionLevel::Node && hit.node.is_some() {
            self.pick_within(entered, hit.part.or(entered.subpath), hit.node, shift);
            return;
        }
        if same_object && let Some(part) = hit.part {
            // Either a re-pick at the Part rung, or the Node rung falling
            // back one rung onto a part.
            self.level = SelectionLevel::Part;
            self.pick_within(entered, Some(part), None, shift);
            return;
        }
        // The click left the object. Ascend and treat it as an ordinary
        // Object-rung click, which also covers "clicked a different object"
        // and "clicked empty paper".
        self.level = SelectionLevel::Object;
        self.click_at_object_rung(page, hit, shift);
    }

    /// Select a part or a node inside the entered object.
    fn pick_within(
        &mut self,
        entered: Selection,
        part: Option<usize>,
        node: Option<usize>,
        shift: bool,
    ) {
        let entry = Selection {
            page: entered.page,
            object: entered.object,
            subpath: part,
            node,
        };
        if shift {
            if let Some(at) = self.entries.iter().position(|e| *e == entry) {
                self.entries.remove(at);
            } else {
                self.entries.push(entry);
            }
        } else {
            self.entries = vec![entry];
        }
        self.normalise();
    }

    /// Descend one rung into whatever is under a double-click.
    ///
    /// A double-click on a **different** object enters that object rather
    /// than descending inside the current one: PDF path objects do not nest,
    /// so carrying a part or node index across would address an index in a
    /// different object's space.
    fn descend(&mut self, page: usize, hit: ClickHit) {
        let Some(object) = hit.object else {
            // A double-click is also a click, and a click on empty paper
            // leaves. Doing anything else here strands the operator.
            self.level = SelectionLevel::Object;
            self.entries.clear();
            self.outlines.clear();
            return;
        };
        let entered = self.entered_object();
        let same_object = entered.is_some_and(|e| e.object == object && e.page == page);

        let (level, entry) = match (same_object, self.level) {
            // Already inside this object at the Part rung: descend to Node.
            // With no anchor within tolerance the rung is still entered —
            // "inside this part, nothing picked yet" is a real state, and
            // refusing to descend would make the gesture feel unreliable on
            // exactly the curves whose anchors are hard to hit.
            (true, SelectionLevel::Part) => (
                SelectionLevel::Node,
                Selection {
                    page,
                    object,
                    subpath: hit.part.or_else(|| entered.and_then(|e| e.subpath)),
                    node: hit.node,
                },
            ),
            // Nothing is below a point.
            (true, SelectionLevel::Node) => return,
            // Entering an object (possibly a different one) from the top.
            _ => (
                SelectionLevel::Part,
                Selection {
                    page,
                    object,
                    subpath: hit.part,
                    node: None,
                },
            ),
        };
        self.level = level;
        self.entries = vec![entry];
        self.normalise();
    }

    /// Restore the two structural rules the rest of the module relies on.
    ///
    /// 1. **Entries are ordered and unique.** Document order, so the outlines
    ///    paint in a stable sequence rather than re-stacking on every
    ///    shift-click; unique, so a batched edit is handed a clean operand
    ///    list.
    /// 2. **A rung above `Object` means exactly one object is entered.** A
    ///    rung is a place *inside one thing*, and [`Self::entered_object`]
    ///    derives that from the first entry. Anything that would leave the
    ///    two disagreeing collapses to the Object rung instead — recovering
    ///    is better than asserting, because the state is reachable from a
    ///    marquee arriving while inside an object and the honest response is
    ///    to step out.
    fn normalise(&mut self) {
        self.entries.sort_unstable();
        self.entries.dedup();
        if self.entries.is_empty() {
            self.level = SelectionLevel::Object;
            self.outlines.clear();
            return;
        }
        if self.level != SelectionLevel::Object {
            let first = self.entries[0];
            if self
                .entries
                .iter()
                .any(|e| e.object != first.object || e.page != first.page)
            {
                self.level = SelectionLevel::Object;
                for entry in &mut self.entries {
                    entry.subpath = None;
                    entry.node = None;
                }
                self.entries.sort_unstable();
                self.entries.dedup();
            }
        }
        // The outlines describe the entries; any change to the entries makes
        // them stale, and a stale outline is a box drawn around something the
        // operator no longer has selected.
        self.resolved_for = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::target::StubTargets;
    use egui::{Pos2, Rect};

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::from_min_size(Pos2::new(x, y), egui::vec2(w, h))
    }

    /// A page with two objects, the first of which has two parts.
    fn stub(page: usize) -> StubTargets {
        StubTargets::new(
            page,
            [rect(0.0, 0.0, 100.0, 100.0), rect(200.0, 200.0, 50.0, 50.0)],
        )
        .with_parts(
            0,
            [rect(0.0, 0.0, 40.0, 40.0), rect(60.0, 60.0, 40.0, 40.0)],
        )
    }

    fn hit_object(index: u64) -> ClickHit {
        ClickHit {
            object: Some(TargetId(index)),
            ..ClickHit::default()
        }
    }

    // -----------------------------------------------------------------
    // ★ Invariant 1 — selection is identity, never position
    // -----------------------------------------------------------------

    /// ★ **Navigation never alters the selection.**
    ///
    /// The acceptance criterion, as close to literally as a headless test can
    /// state it: select a node, then perform every navigation the roadmap
    /// names — zoom out three rungs, pan, change fit mode, rotate the view,
    /// change page-display mode, switch ribbon tab — and assert the selection
    /// is byte-identical afterwards.
    ///
    /// # ★ Phase 3's gestures were added to THIS sweep, not to a parallel test
    ///
    /// The hand-tool pan, the anchored discrete zoom, the marquee zoom and
    /// zoom-to-selection are navigation, so they belong to the invariant that
    /// already governs navigation. A second test asserting the same property
    /// about four more operations would be a second place for the property to
    /// be stated — and the first one to be forgotten when a fifth arrives.
    ///
    /// **Zoom-to-selection is the interesting addition**, because it is the
    /// only navigation that *reads* the selection: it resolves the selection's
    /// bounds and frames them. Reading is exactly where a "helpful" edit —
    /// normalise the entries, collapse to the outlined ones, drop what has no
    /// bounds — would creep in, and it would be invisible until the operator
    /// zoomed to a node and found they had selected the object instead.
    ///
    /// What this cannot reach is the *wiring*: that a released
    /// `MarqueeIntent::Zoom` never calls [`SelectionState::marquee`] at all.
    /// That is structural in `canvas::interact` — the two intents are separate
    /// match arms over an exhaustive enum, and only one of them names the
    /// selection — and it is asserted from the gesture side by
    /// `canvas::gesture`'s `a_zoom_marquee_is_the_same_band_with_the_other_intent`.
    ///
    /// It is expressed as *"drive the view state and then compare"* because
    /// that is the honest model of what navigation is: those operations act
    /// on [`crate::viewer::ViewState`], and the property being asserted is
    /// that no route exists from there to here. The test would fail the
    /// moment somebody gave `SelectionState` a screen coordinate to keep in
    /// step, which is the defect it guards.
    #[test]
    fn navigating_the_view_never_alters_the_selection() {
        use crate::viewer::{FitMode, MAX_ZOOM, ViewState};

        let targets = stub(0);
        let mut sel = SelectionState::default();
        // Select a node: click, double-click into the part, double-click
        // again to reach the anchor.
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
        sel.resolve(Some(&targets), 0, 0);
        assert_eq!(sel.level(), SelectionLevel::Node);
        assert_eq!(sel.entries()[0].node, Some(4));
        let before = sel.clone();

        // Every navigation the invariant names, in one sweep.
        let mut view = ViewState::default();
        for _ in 0..3 {
            view.zoom_out(MAX_ZOOM);
        }
        view.set_fit(FitMode::Width);
        view.apply_fit((612.0, 792.0), (300.0, 900.0), MAX_ZOOM);
        view.set_fit(FitMode::Page);
        view.apply_fit((612.0, 792.0), (1_600.0, 400.0), MAX_ZOOM);
        view.zoom_by(1.37, MAX_ZOOM);
        view.next_page(4);
        view.prev_page(4);

        // ---- Phase 3's navigation gestures, in the same sweep -------------
        use crate::canvas::geometry;
        use crate::canvas::mapping::PageMapping;
        use crate::canvas::zoom::{self, ZoomOutcome};

        let extent = (200.0_f32, 300.0_f32);
        let frame = zoom::CanvasFrame {
            map: PageMapping::new(
                Rect::from_min_size(Pos2::new(12.0, 7.0), egui::vec2(extent.0, extent.1)),
                extent,
                1.0,
            ),
            extent,
            display: (extent.0, extent.1),
            viewport: (400.0, 400.0),
            viewport_rect: Rect::from_min_size(Pos2::new(10.0, 5.0), egui::vec2(400.0, 400.0)),
            offset: (0.0, 0.0),
        };

        // A hand-tool / space-bar pan. The same arithmetic the middle drag
        // uses, and it must move the view — a pan that clamped to a no-op
        // would make the assertion below vacuous.
        let panned = geometry::pan_offset(
            (120.0, 80.0),
            (30.0, -20.0),
            (1_600.0, 1_600.0),
            (800.0, 800.0),
        );
        assert_ne!(panned, (120.0, 80.0), "the pan must actually move the view");

        // An anchored discrete zoom: arm on a page point, step the ladder,
        // solve. This is Ctrl+Plus, end to end, minus the `egui::Context`.
        let anchor = zoom::hold(zoom::frac_of(Pos2::new(50.0, 50.0), extent), &frame);
        view.zoom_in(MAX_ZOOM);
        let _ = geometry::zoom_anchor_offset(
            anchor.offset_before,
            anchor.display_before,
            (extent.0 * view.zoom, extent.1 * view.zoom),
            anchor.viewport,
            anchor.frac,
        );

        // A marquee zoom to a region of the page.
        let region = Rect::from_min_max(Pos2::new(10.0, 10.0), Pos2::new(90.0, 120.0));
        if let ZoomOutcome::Zoomed { applied, .. } =
            zoom::plan_framing(&frame, region, 16.0, 1.0).outcome
        {
            view.set_zoom(applied, MAX_ZOOM);
        }

        // ★ Zoom to the selection — the one navigation that reads it.
        let bounds = sel
            .outline_union()
            .expect("a resolved selection has bounds to frame");
        if let ZoomOutcome::Zoomed { applied, .. } =
            zoom::plan_framing(&frame, bounds, 16.0, 1.0).outcome
        {
            view.set_zoom(applied, MAX_ZOOM);
        }

        // ---- Phase 4's page-display modes, in the same sweep ---------------
        //
        // ★ Added HERE rather than in a parallel test, for the reason this
        // test's header already gives about Phase 3's gestures: a page-display
        // change is navigation, and navigation is governed by this invariant.
        // A second test asserting the same property about a fifth operation
        // would be a second place for the property to live and the first one
        // to be forgotten.
        //
        // `FEATURES.md` names "page-display mode" in the list of things the
        // selection is asserted byte-identical across, and until Phase 4 there
        // was only one mode — so the clause was true and untested. It is now
        // exercised: every arrangement, including the two that put several
        // pages on screen at once, and a full strip laid out for each so the
        // geometry the mode produces is real rather than nominal.
        use crate::viewer::{PageDisplay, strip::Strip};
        use pdfce_core::object::{Dict, ObjId};
        use pdfce_core::page_tree::{Page, Rect as PageRect};

        let pages: Vec<Page> = (0..4)
            .map(|_| Page {
                id: ObjId::new(1, 0),
                resources: Dict::new(),
                media_box: PageRect::from_corners(0.0, 0.0, 612.0, 792.0),
                crop_box: PageRect::from_corners(0.0, 0.0, 612.0, 792.0),
                rotate: 0,
                contents: Vec::new(),
                contents_unresolved: 0,
            })
            .collect();
        for &display in PageDisplay::ALL {
            view.display = display;
            let strip = Strip::new(&pages, display, view.page_index, view.zoom);
            // The mode really does change the layout, or the loop asserts
            // nothing about the modes it iterates.
            assert!(!strip.is_empty());
            let metrics = crate::viewer::strip::row_metrics(&pages, display, view.page_index, 1.0);
            view.apply_fit(metrics.extent, (900.0, 700.0), metrics.max_zoom);
        }
        assert!(
            Strip::new(&pages, PageDisplay::Continuous, 0, 1.0).size().y
                > Strip::new(&pages, PageDisplay::Single, 0, 1.0).size().y,
            "the continuous strip must be taller than one page, or the sweep \
             above passed through four modes that all laid out the same thing"
        );

        // The provider is rebuilt on the way — that is what a page step, a
        // page-display change and a ribbon-tab change do — and the selection
        // must come through it.
        sel.resolve(Some(&stub(0)), 0, 0);

        assert_eq!(
            sel, before,
            "a view change reached the selection; it must not be able to"
        );
    }

    /// ★ **A selection on another page survives a provider for a different
    /// page** — the half of invariant 3 the acceptance criterion turns on.
    ///
    /// Paging away rebuilds the provider for the new page. A `resolve` that
    /// pruned everything it could not find would wipe the selection on the
    /// way past, and coming back would find nothing.
    #[test]
    fn paging_away_and_back_keeps_the_selection() {
        let mut sel = SelectionState::default();
        sel.click(0, hit_object(1), false, false);
        sel.resolve(Some(&stub(0)), 0, 0);
        assert_eq!(sel.len(), 1);
        assert_eq!(sel.outlines().len(), 1);

        // Page 1: a provider that knows nothing about page 0's objects.
        sel.resolve(Some(&stub(1)), 1, 0);
        assert_eq!(sel.len(), 1, "the entry for page 0 must survive");
        assert!(
            sel.outlines().is_empty(),
            "nothing on page 1 is selected, so nothing on page 1 is outlined"
        );

        // …and back.
        sel.resolve(Some(&stub(0)), 0, 0);
        assert_eq!(sel.entries(), [Selection::object(0, TargetId(1))]);
        assert_eq!(sel.outlines().len(), 1, "the outline comes back with it");
    }

    /// A rebuild at the same revision is a no-op that costs one comparison —
    /// which is what makes "resolve every frame" affordable and therefore
    /// what makes the invariant cheap enough to actually hold.
    #[test]
    fn re_resolving_at_the_same_revision_changes_nothing() {
        let targets = stub(0);
        let mut sel = SelectionState::default();
        sel.click(0, hit_object(0), false, false);
        sel.resolve(Some(&targets), 0, 0);
        let after_first = sel.clone();
        for _ in 0..50 {
            sel.resolve(Some(&targets), 0, 0);
        }
        assert_eq!(sel, after_first);
    }

    /// An edit that removed a selected object drops **that** entry and keeps
    /// the rest, rather than clearing the selection or leaving a hole a
    /// batched delete would refuse.
    #[test]
    fn an_edit_that_removed_an_object_drops_only_that_entry() {
        let mut sel = SelectionState::default();
        sel.click(0, hit_object(0), false, false);
        sel.click(0, hit_object(1), true, false);
        sel.resolve(Some(&stub(0)), 0, 0);
        assert_eq!(sel.len(), 2);

        // The page now holds one object: index 1 is gone.
        let after_edit = StubTargets::new(0, [rect(0.0, 0.0, 100.0, 100.0)]);
        sel.resolve(Some(&after_edit), 0, 1);
        assert_eq!(sel.entries(), [Selection::object(0, TargetId(0))]);
    }

    /// An undecodable page loses its outlines and keeps its selection. The
    /// two states are different and must not be conflated — and the failure
    /// is recorded, so the decomposition that would not decode is not retried
    /// on every frame.
    #[test]
    fn losing_the_provider_does_not_lose_the_selection() {
        let mut sel = SelectionState::default();
        sel.click(0, hit_object(0), false, false);
        sel.resolve(Some(&stub(0)), 0, 0);
        assert_eq!(sel.outlines().len(), 1);

        // The operator pages to a sheet whose content will not decode.
        assert!(sel.needs_resolve(1, 0));
        sel.resolve(None, 1, 0);
        assert!(sel.outlines().is_empty());
        assert_eq!(sel.len(), 1, "the selection is not the outline");
        assert!(
            !sel.needs_resolve(1, 0),
            "a failed decomposition must not be retried every frame"
        );
    }

    /// ★ **Delete's operand list is empty at every rung below Object** — the
    /// one statement of that rule, asserted where it lives.
    ///
    /// The canvas keys and the ribbon's `format.delete` both read
    /// [`SelectionState::deletable_objects_on`], so this test covers both. It
    /// is the destructive case: the only wired verb removes whole objects, and
    /// one measured CAD export holds an entire drawing view as a single path
    /// object with 1,194 subpaths.
    ///
    /// The page filter is asserted too, because a paint-order index is a
    /// position on **one** page and handing `delete_objects` an index from
    /// another one would remove whatever happens to sit at that slot.
    #[test]
    fn only_the_object_rung_offers_anything_to_delete() {
        let mut sel = SelectionState::default();
        assert!(sel.deletable_objects_on(0).is_empty(), "nothing selected");

        sel.click(0, hit_object(1), false, false);
        sel.click(0, hit_object(0), true, false);
        assert_eq!(
            sel.deletable_objects_on(0),
            vec![0, 1],
            "ascending and de-duplicated, or `delete_objects` refuses the batch"
        );
        assert!(
            sel.deletable_objects_on(1).is_empty(),
            "an index is a position on ONE page"
        );

        // Descend into the object: the rung has no delete verb.
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
        assert!(
            sel.deletable_objects_on(0).is_empty(),
            "deleting the enclosing object because one subpath was selected is \
             a destructive wrong action, not a convenience"
        );
        assert_eq!(sel.len(), 1, "and asking does not change the selection");

        // …and back out again, which restores the operand list.
        assert_eq!(
            sel.escape(),
            EscapeOutcome::LeftLevel(SelectionLevel::Object)
        );
        assert_eq!(sel.deletable_objects_on(0), vec![0]);
    }

    // -----------------------------------------------------------------
    // Click semantics
    // -----------------------------------------------------------------

    #[test]
    fn a_plain_click_replaces_and_a_shift_click_toggles() {
        let mut sel = SelectionState::default();
        sel.click(0, hit_object(0), false, false);
        assert_eq!(sel.entries(), [Selection::object(0, TargetId(0))]);

        sel.click(0, hit_object(1), true, false);
        assert_eq!(sel.len(), 2, "shift adds");

        sel.click(0, hit_object(1), true, false);
        assert_eq!(sel.len(), 1, "shift on a selected entry removes it");

        sel.click(0, hit_object(1), false, false);
        assert_eq!(
            sel.entries(),
            [Selection::object(0, TargetId(1))],
            "a plain click replaces rather than adding"
        );
    }

    /// A plain click on empty paper clears; a shift click on empty paper does
    /// not. The asymmetry is deliberate — an over-shot shift-click must not
    /// destroy a set that took five clicks to build.
    #[test]
    fn a_miss_clears_only_without_shift() {
        let mut sel = SelectionState::default();
        sel.click(0, hit_object(0), false, false);
        sel.click(0, ClickHit::default(), true, false);
        assert_eq!(sel.len(), 1, "shift+miss leaves the selection alone");
        sel.click(0, ClickHit::default(), false, false);
        assert!(sel.is_empty(), "a plain click on empty paper clears");
    }

    /// Entries are held in document order however they were clicked, so the
    /// outlines paint in a stable sequence.
    #[test]
    fn entries_are_ordered_and_unique_however_they_were_clicked() {
        let mut sel = SelectionState::default();
        sel.click(0, hit_object(1), false, false);
        sel.click(0, hit_object(0), true, false);
        assert_eq!(
            sel.entries(),
            [
                Selection::object(0, TargetId(0)),
                Selection::object(0, TargetId(1))
            ]
        );
        assert_eq!(sel.object_indices_on(0), vec![0, 1]);
        assert!(sel.object_indices_on(1).is_empty());
    }

    // -----------------------------------------------------------------
    // The ladder
    // -----------------------------------------------------------------

    #[test]
    fn a_double_click_descends_one_rung_at_a_time() {
        let mut sel = SelectionState::default();
        sel.click(0, hit_object(0), false, false);
        assert_eq!(sel.level(), SelectionLevel::Object);

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
        assert_eq!(sel.entries()[0].subpath, Some(1));

        sel.click(
            0,
            ClickHit {
                object: Some(TargetId(0)),
                part: Some(1),
                node: Some(6),
            },
            false,
            true,
        );
        assert_eq!(sel.level(), SelectionLevel::Node);
        assert_eq!(sel.entries()[0].node, Some(6));

        // Nothing is below a point.
        let at_the_bottom = sel.clone();
        sel.click(
            0,
            ClickHit {
                object: Some(TargetId(0)),
                part: Some(1),
                node: Some(6),
            },
            false,
            true,
        );
        assert_eq!(sel, at_the_bottom);
    }

    /// ★ **Escape ascends exactly one rung per press.**
    ///
    /// The old shell shipped Escape as "clear everything", so an operator two
    /// rungs inside a drawing found one press putting them back at the page.
    /// Asserted as a sequence of outcomes rather than a boolean, so a
    /// regression that collapsed the ladder cannot pass by clearing on the
    /// first press and reporting `true` three times.
    #[test]
    fn escape_ascends_one_rung_per_press() {
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
        sel.click(
            0,
            ClickHit {
                object: Some(TargetId(0)),
                part: Some(0),
                node: Some(2),
            },
            false,
            true,
        );
        assert_eq!(sel.level(), SelectionLevel::Node);

        assert_eq!(
            sel.escape(),
            EscapeOutcome::LeftLevel(SelectionLevel::Part),
            "the first press leaves the Node rung and nothing else"
        );
        assert_eq!(sel.len(), 1, "leaving a rung does not clear the selection");
        assert_eq!(sel.entries()[0].node, None);
        assert_eq!(sel.entries()[0].subpath, Some(0));

        assert_eq!(
            sel.escape(),
            EscapeOutcome::LeftLevel(SelectionLevel::Object)
        );
        assert_eq!(sel.entries()[0].subpath, None);
        assert_eq!(sel.len(), 1);

        assert_eq!(sel.escape(), EscapeOutcome::ClearedSelection);
        assert!(sel.is_empty());

        assert_eq!(
            sel.escape(),
            EscapeOutcome::Nothing,
            "with nothing selected the canvas must not consume Escape"
        );
    }

    /// A click that misses everything inside the entered object leaves the
    /// object rather than stranding the operator at a rung.
    #[test]
    fn clicking_away_leaves_the_entered_object() {
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
        assert_eq!(sel.level(), SelectionLevel::Part);

        sel.click(0, ClickHit::default(), false, false);
        assert_eq!(sel.level(), SelectionLevel::Object);
        assert!(sel.is_empty());
    }

    /// A click on a *different* object while inside one leaves and selects
    /// that object — PDF path objects do not nest, so there is nothing to
    /// nest into.
    #[test]
    fn clicking_a_different_object_leaves_rather_than_nesting() {
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
        sel.click(0, hit_object(1), false, false);
        assert_eq!(sel.level(), SelectionLevel::Object);
        assert_eq!(sel.entries(), [Selection::object(0, TargetId(1))]);
    }

    /// At the Node rung, a click that misses every anchor but lands on a part
    /// ascends one rung and re-picks — rather than doing nothing, which is
    /// how an operator gets stuck at a rung whose targets they keep missing.
    #[test]
    fn missing_every_anchor_falls_back_to_the_part_rung() {
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
        sel.click(
            0,
            ClickHit {
                object: Some(TargetId(0)),
                part: Some(0),
                node: Some(1),
            },
            false,
            true,
        );
        assert_eq!(sel.level(), SelectionLevel::Node);

        sel.click(
            0,
            ClickHit {
                object: Some(TargetId(0)),
                part: Some(1),
                node: None,
            },
            false,
            false,
        );
        assert_eq!(sel.level(), SelectionLevel::Part);
        assert_eq!(sel.entries()[0].subpath, Some(1));
        assert_eq!(sel.entries()[0].node, None);
    }

    // -----------------------------------------------------------------
    // Marquee
    // -----------------------------------------------------------------

    #[test]
    fn a_marquee_replaces_and_a_shift_marquee_extends() {
        let mut sel = SelectionState::default();
        sel.marquee(0, &[TargetId(0)], false);
        assert_eq!(sel.entries(), [Selection::object(0, TargetId(0))]);

        sel.marquee(0, &[TargetId(1)], true);
        assert_eq!(sel.len(), 2);

        sel.marquee(0, &[TargetId(1)], false);
        assert_eq!(sel.entries(), [Selection::object(0, TargetId(1))]);
    }

    /// A marquee always lands at the Object rung and takes the operator out
    /// of any object they were inside.
    #[test]
    fn a_marquee_ascends_to_the_object_rung() {
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
        assert_eq!(sel.level(), SelectionLevel::Part);
        sel.marquee(0, &[TargetId(0), TargetId(1)], false);
        assert_eq!(sel.level(), SelectionLevel::Object);
        assert_eq!(sel.entries()[0].subpath, None);
    }

    // -----------------------------------------------------------------
    // Outlines
    // -----------------------------------------------------------------

    /// The outline of an entered part is the **part's** box, not the
    /// object's. An object-sized rectangle around a part tells the operator
    /// they selected the whole thing again, which is the misunderstanding
    /// entering the object exists to resolve.
    #[test]
    fn an_entered_parts_outline_is_the_parts_own_box() {
        let targets = stub(0);
        let mut sel = SelectionState::default();
        sel.click(0, hit_object(0), false, false);
        sel.resolve(Some(&targets), 0, 0);
        let whole = sel.outlines()[0].1;

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
        sel.resolve(Some(&targets), 0, 0);
        let part = sel.outlines()[0].1;
        assert!(part.width() < whole.width());
        assert!(whole.contains_rect(part));
    }

    /// The grips sit around the union of the selection, not around one
    /// member of it.
    #[test]
    fn the_grip_box_is_the_union_of_the_selection() {
        let targets = stub(0);
        let mut sel = SelectionState::default();
        assert_eq!(sel.outline_union(), None);
        sel.click(0, hit_object(0), false, false);
        sel.click(0, hit_object(1), true, false);
        sel.resolve(Some(&targets), 0, 0);
        let union = sel
            .outline_union()
            .expect("two selected objects have a union");
        assert!(union.contains_rect(rect(0.0, 0.0, 100.0, 100.0)));
        assert!(union.contains_rect(rect(200.0, 200.0, 50.0, 50.0)));
    }
}
