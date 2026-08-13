//! # `panels::objects::provider` — front-to-back page object decomposition
//!
//! The thin `pdfce-gui` adapter that plugs `pdfce-core`'s read-only vector
//! object model (`pdfce_core::vector`) into the shell. Salvaged from the old
//! shell's `object_provider.rs` (694 code lines, 313 test lines) per
//! `SALVAGE.md`'s Class A row. Decision 011 §2.1 set its shape:
//!
//! > Pass 9a's real provider is a thin `pdfce-gui` adapter that CALLS INTO
//! > `pdfce-core`'s read-only object model (which stays GUI-free); the
//! > adapter owns the trait impl, the object model owns none of it.
//!
//! ## What lives here vs in core (GUI–core separation)
//!
//! ALL geometry — decomposition, hit-testing, marquee enclosure — is
//! `pdfce_core::vector`, in PDF user space. This module owns exactly two
//! things core cannot: (1) the **coordinate-space translation** between the
//! canvas's device convention and PDF user space, and (2) the [`TargetId`] ↔
//! object-index encoding. The translation reuses the SAME transform
//! [`pdfce_render::page_device_geometry`] computes to rasterize the page (at
//! scale 1.0, which *is* canvas space), inverted — so selection geometry and
//! the render agree by construction, exactly as
//! [`crate::viewer::canvas_to_pdf_space`] does (this provider is the
//! batched, object-model-backed sibling of that per-point bridge).
//!
//! ## Single-page by design
//!
//! The canvas shows one page at a time and only ever queries
//! `view.page_index`, so a provider is built for the **current page** and
//! rebuilt on page change / edit. A query for any other `page_index` returns
//! nothing — cheap, and it keeps the decomposition off the hot path of a
//! large document (only the visible page is decomposed, not all N).
//!
//! `RIBBON_IA.md` §5.2 names this as the reason continuous page display is
//! "a larger build than it looks": this file returns nothing for any page
//! but the current one, and continuous mode needs a page *range*. That is
//! `GUI_ROADMAP.md` Phase 4 work, and it is a real change here rather than a
//! wiring change above.
//!
//! ## [`TargetId`] encoding
//!
//! A [`TargetId`] is the object's index into
//! [`pdfce_core::vector::PageObjects::objects`] (paint order), cast to
//! `u64`. Consumers treat it opaquely; only this module mints and decodes
//! it.
//!
//! ---
//!
//! # What is live at S3, and what is waiting for S4
//!
//! This whole file came across, because `SALVAGE.md`'s procedure forbids
//! salvaging by snippet — *"the old GUI's value is disproportionately in its
//! doc comments; a snippet leaves those behind and the next engineer
//! re-derives a decision that was already made and already paid for."* But
//! only some of it has a consumer today, and pretending otherwise would be
//! its own dishonesty:
//!
//! | Method group | S3 consumer | Waiting on |
//! |---|---|---|
//! | [`ObjectModelProvider::build`], [`page_objects`](ObjectModelProvider::page_objects) | the Objects panel's row list | — |
//! | [`part_kind`](ObjectModelProvider::part_kind), [`part_count`](ObjectModelProvider::part_count), [`subpath_count`](ObjectModelProvider::subpath_count), [`text_run_count`](ObjectModelProvider::text_run_count) | the Objects panel's **object → part → point** nesting | — |
//! | [`subpath_node_points`](ObjectModelProvider::subpath_node_points), [`object_node_points`](ObjectModelProvider::object_node_points), [`subpath_handle_points`](ObjectModelProvider::subpath_handle_points) | the Objects panel's point rows and the Properties panel's node readout | — |
//! | [`hit_test_all`](ObjectModelProvider::hit_test_all), [`hit_test`](ObjectModelProvider::hit_test), [`hit_test_rect`](ObjectModelProvider::hit_test_rect), [`bounds`](ObjectModelProvider::bounds) | none | **S4** — the canvas selection layer and the `CanvasTargetProvider` trait |
//! | [`part_hits`](ObjectModelProvider::part_hits), [`subpath_hits`](ObjectModelProvider::subpath_hits), [`text_run_hits`](ObjectModelProvider::text_run_hits), [`nearest_node`](ObjectModelProvider::nearest_node), [`nearest_handle`](ObjectModelProvider::nearest_handle) | none | **S4** — click-to-select and the level ladder |
//! | [`part_bounds_canvas`](ObjectModelProvider::part_bounds_canvas) and friends | none | **S4** — selection outlines |
//! | [`object_sample_points`](ObjectModelProvider::object_sample_points) | none | **S5** — the measure tools' snap query and Taubin best-fit circle |
//!
//! **Every one of them is under test below.** That is the difference between
//! carrying a method forward and leaving a stub: the S4 canvas will attach a
//! trait to a working, proven implementation rather than to code nobody has
//! run since it was pasted.
//!
//! ## What changed at salvage
//!
//! 1. **`use eframe::egui` → `use egui`**, the crate-wide S0 convention.
//! 2. **The `CanvasTargetProvider` trait impl became inherent methods.**
//!    The trait lives in `canvas/` and does not exist yet. The three
//!    methods keep their names and their exact semantics, and
//!    [`ObjectModelProvider::hit_test`] — which was the *trait's provided
//!    method* over `hit_test_all` — is written out here as an inherent
//!    method with its derivation intact, so the two still cannot disagree.
//!    Re-attaching the trait at S4 is a one-line `impl` block over methods
//!    that already have the right signatures.
//! 3. **[`TargetId`] moved here from `canvas`.** It is the *encoding*, and
//!    the encoding belongs with the thing that mints it. When `canvas`
//!    grows its substrate it re-exports this rather than defining a second
//!    one — two id types over one index space is precisely the divergence
//!    this file's own docs warn about.
//! 4. **One test did not come across:
//!    `screen_tolerance_keeps_the_on_screen_catch_radius_constant`.** It
//!    asserts a law about `canvas::screen_tolerance_to_page` and
//!    `canvas::SELECT_SCREEN_TOLERANCE_PX`, neither of which exists in this
//!    crate yet, and re-declaring those constants here to keep a test green
//!    would put the tolerance in two places — which is the *cause* of the
//!    defect the test guards, not a way to guard it. It lands in `canvas/`
//!    at S4 with the functions it is about. **The substantive regression
//!    test came across intact**:
//!    [`tests::selection_tolerance_is_honoured_per_query_not_baked_in`]
//!    proves the tolerance is a per-query parameter rather than a baked
//!    constant, which is the half that lives here.
//! 5. **Two doc cross-references were repointed** at things that exist:
//!    `crate::canvas::EmptyTargetProvider` (the shippable no-op provider)
//!    and `crate::vector_edit_tool::nearest_anchor` are both S4/S5 modules,
//!    so the claims they anchored are stated directly instead of by link.
//!
//! No arithmetic, no tolerance rule, no hit ordering and no index
//! convention changed.

use egui::{Pos2, Rect};
use pdfce_core::page_tree::Page;
use pdfce_core::vector::{
    Bounds, Handle, MarqueeMode, Matrix, PageObjects, Point, Segment, VectorObject, decompose_page,
    hit_test_point_all, hit_test_rect,
};
use pdfce_core::view::DocumentView;
use pdfce_render::page_device_geometry;
use pdfce_render::tiny_skia::{Point as SkPoint, Transform};

/// One page object, addressed opaquely.
///
/// The wrapped `u64` is the object's index into
/// [`pdfce_core::vector::PageObjects::objects`] — i.e. **paint order**, and
/// the same index `pdfce-cli object-list` prints as `index=` and
/// `object-move` / `object-delete` / `node-move` take as an operand. One
/// index space across every surface is what makes "object 412" mean the same
/// thing in a panel row, in a trace line and on a command line.
///
/// A newtype rather than a bare `usize` so it cannot be confused with a row
/// position, a page index or a subpath index — all of which are also
/// `usize`, all of which appear within a few lines of each other in the
/// Objects panel, and one of which (row position) counts in the opposite
/// direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TargetId(pub u64);

/// The fallback canvas-space slack a click may miss an object's edge by,
/// used ONLY when the caller cannot supply a live zoom (a non-finite or
/// non-positive zoom makes a screen-to-page tolerance conversion return
/// `0.0`, which would make selection impossible rather than merely fussy).
///
/// Canvas space is the page's device space at zoom 1.0, where one unit is
/// one PDF point (the `page_device_geometry` scale-1.0 map is
/// distance-preserving — a pure rotation + Y-flip + translation), so this is
/// also, in effect, a ~3 pt page-space tolerance.
///
/// **This used to be the only tolerance**, applied at every zoom level, and
/// that was a bug: the pointer is divided by `zoom` before it reaches
/// [`ObjectModelProvider::hit_test`], so a constant canvas-space tolerance
/// is a *shrinking* on-screen catch radius — 1.5 px at 50% zoom, 0.75 px at
/// 25%. Objects were effectively unclickable whenever the operator zoomed
/// out to see a whole drawing. The live tolerance arrives as a parameter,
/// derived at the call site from a screen-pixel constant divided by the
/// zoom.
pub const FALLBACK_SELECT_TOLERANCE: f64 = 3.0;

/// The object-model-backed provider for one page (module docs).
pub struct ObjectModelProvider {
    /// The page this provider answers for; queries for any other index miss.
    page_index: usize,
    /// The decomposed objects, in PDF user space (paint order).
    objects: PageObjects,
    /// PDF user space → canvas space (the render device map at scale 1.0).
    to_canvas: Transform,
    /// Canvas space → PDF user space (the inverse), or `None` for a
    /// degenerate (non-invertible) page — then the provider declines every
    /// query rather than fabricate geometry.
    to_pdf: Option<Transform>,
}

/// Which KIND of part the "Part" rung is standing on for a given object.
///
/// The rung is shared between path SUBPATHS and text RUNS, and almost
/// everything about it is identical — nearest-first hit order, an outline to
/// draw, Escape to ascend, Delete to remove. What differs is the **verb
/// set**, and that is exactly what this tells a caller:
///
/// | | `Subpath` | `Run` |
/// |---|---|---|
/// | Delete | `delete_subpath` | `delete_text_run` |
/// | Drag to move | `move_subpath` | **nothing — no core verb exists** |
/// | Descend to Point | yes | no (a run has no anchors) |
///
/// The Point-rung row needs no guard anywhere:
/// [`ObjectModelProvider::nearest_node`] reaches
/// [`ObjectModelProvider::subpath_node_points`], which matches
/// `VectorObject::Path` only, so a text entry can never produce a node hit.
/// The ladder caps itself at two rungs for text by construction rather than
/// by a check — which is also why the Objects panel's tree can nest a text
/// object one level and a path object two, with no special case in the row
/// builder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartKind {
    /// A subpath of a path object.
    Subpath,
    /// A show operator ("run") of a text object.
    Run,
}

impl ObjectModelProvider {
    /// Build a provider for `page` (at `page_index`) from `view`.
    ///
    /// Returns `None` only if the page's content cannot be decoded/tokenized
    /// (the same failure the renderer would hit). A caller then says so in
    /// words rather than showing an empty list — a failure state must never
    /// be visually indistinguishable from a success state that happens to
    /// have no content.
    ///
    /// # Pass a SESSION view, not the base document (decision 018)
    ///
    /// Callers pass `&session.view()`. Passing `&session.document().view()`
    /// decomposes the *base revision*, so hit-testing, marquee selection and
    /// the measure tools' snapping all address geometry the operator can no
    /// longer see and miss geometry they can. The raster and this provider
    /// must be built from the *same* view, or the canvas shows one document
    /// and responds as another.
    ///
    /// At S3 the second half of that hazard is what bites: the Objects panel
    /// would list the pre-edit object set while the canvas draws the
    /// post-edit page, and the panel exists precisely to answer "what am I
    /// looking at".
    #[must_use]
    pub fn build(view: &DocumentView<'_>, page: &Page, page_index: usize) -> Option<Self> {
        Self::build_or_reason(view, page, page_index).ok()
    }

    /// [`Self::build`], keeping the reason the page would not decompose.
    ///
    /// # Why the reason is worth a second constructor
    ///
    /// [`Self::build`] throws the `ContentError` away, which is right for a
    /// *panel*: an operator is told the page's content could not be read, in
    /// the catalog's words, and a tokenizer's error text is not a sentence
    /// anybody outside this project can act on.
    ///
    /// It is wrong for the **diagnostic channel**.
    /// `crate::app::state::OpenDoc::trace_object_count` emits
    /// `objects-unavailable page=… reason=decompose-failed detail=…`, and the
    /// `detail=` is the whole value of the line: without it a harness learns
    /// that a page did not decompose and nothing about why, which is a
    /// question it then has to answer by hand.
    ///
    /// Before the decomposition cache moved onto `OpenDoc`, the trace kept
    /// that detail by running **its own** `decompose_page` — a second
    /// decomposition of the same page, which is precisely the *"two
    /// decompositions quietly diverge"* pattern decision 011 warns about and
    /// [`Self::page_objects`]' own docs exist to prevent. This constructor is
    /// what let that second call be deleted: one decomposition, and the
    /// failure reason survives it.
    ///
    /// The error is stringified here rather than propagated as a
    /// `ContentError` so the cache that stores it does not have to name a
    /// `pdfce-core` error type in its own signature — the only consumer wants
    /// a line of trace text, and `ContentError` is `#[non_exhaustive]`.
    ///
    /// # Errors
    ///
    /// The page's `/Contents` could not be resolved, inflated or tokenized —
    /// the same failure the renderer would hit on the same page.
    pub fn build_or_reason(
        view: &DocumentView<'_>,
        page: &Page,
        page_index: usize,
    ) -> Result<Self, String> {
        let objects =
            decompose_page(view, page, Matrix::IDENTITY).map_err(|err| err.to_string())?;
        let (_, _, to_canvas) = page_device_geometry(page, 1.0);
        Ok(Self {
            page_index,
            objects,
            to_canvas,
            to_pdf: to_canvas.invert(),
        })
    }

    /// Construct directly from parts — the seam the headless unit tests use
    /// (a [`PageObjects`] plus an explicit canvas↔PDF transform), so the
    /// adapter logic is proven without a live `Document` or an egui frame.
    #[cfg(test)]
    pub(crate) fn from_parts(
        page_index: usize,
        objects: PageObjects,
        to_canvas: Transform,
    ) -> Self {
        Self {
            page_index,
            objects,
            to_canvas,
            to_pdf: to_canvas.invert(),
        }
    }

    /// Which page this provider answers for.
    ///
    /// Read by the caller that decides whether to rebuild after a page step.
    /// Exposed rather than re-derived because "is my provider still about
    /// the page I am looking at?" must have exactly one answer.
    #[must_use]
    pub fn page_index(&self) -> usize {
        self.page_index
    }

    /// The current page's decomposed vector objects.
    ///
    /// The **shared escape hatch** every consumer reads the already-
    /// decomposed objects through — the Objects panel's row list today, the
    /// snap engine and the Taubin best-fit circle later — so each reuses the
    /// ONE decomposition this provider built rather than running a second
    /// `decompose_page` per frame. That avoids the exact "two decompositions
    /// quietly diverge" pattern decision 011 warns against.
    ///
    /// Everything in [`PageObjects`] is in **PDF user / page space** — the
    /// frame the model stores — so a caller with a canvas-space point
    /// converts it first, either with [`Self::canvas_to_pdf`]'s public
    /// sibling [`crate::viewer::canvas_to_pdf_space`] or through this
    /// provider's own queries.
    #[must_use]
    pub fn page_objects(&self) -> &PageObjects {
        &self.objects
    }

    /// Which subpath of `object` a canvas-space click lands on — the second
    /// selection level, for objects that hold a whole drawing.
    ///
    /// A thin adapter over [`pdfce_core::vector::hit_test_subpaths`], exactly
    /// like [`Self::hit_test_all`] is over the per-object query: convert
    /// canvas space to PDF user space, apply the same degenerate-tolerance
    /// fallback, and let the core own the geometry. Sharing that fallback
    /// matters — without it a click could select an object and then find none
    /// of its subpaths, which reads as "the second level is broken" rather
    /// than "the tolerance was zero".
    ///
    /// Nearest first. Empty for a non-path object or an out-of-range index.
    #[must_use]
    pub fn subpath_hits(&self, object: usize, point: Pos2, tolerance: f64) -> Vec<usize> {
        let Some(pdf) = self.canvas_to_pdf(point) else {
            return Vec::new();
        };
        pdfce_core::vector::hit_test_subpaths(&self.objects, object, pdf, resolve(tolerance))
    }

    /// What kind of part the object at `index` is decomposed into, or `None`
    /// for an object with no Part rung at all (an image).
    #[must_use]
    pub fn part_kind(&self, index: usize) -> Option<PartKind> {
        match self.objects.objects.get(index) {
            Some(VectorObject::Path(_)) => Some(PartKind::Subpath),
            Some(VectorObject::Text(_)) => Some(PartKind::Run),
            _ => None,
        }
    }

    /// Which part of `object` a canvas-space click lands on — **whichever
    /// kind of part that object has**.
    ///
    /// ONE dispatcher rather than a kind match at each call site. The
    /// alternative is duplicated-predicate drift: two places deciding "which
    /// part is under the pointer" go out of step invisibly, and the operator
    /// finds that descending works for a drawing and not for a label.
    #[must_use]
    pub fn part_hits(&self, object: usize, point: Pos2, tolerance: f64) -> Vec<usize> {
        match self.part_kind(object) {
            Some(PartKind::Subpath) => self.subpath_hits(object, point, tolerance),
            Some(PartKind::Run) => self.text_run_hits(object, point, tolerance),
            None => Vec::new(),
        }
    }

    /// A part's bounds in **canvas** space, for drawing its outline —
    /// whichever kind of part it is. The dispatcher for
    /// [`Self::subpath_bounds_canvas`] / [`Self::text_run_bounds_canvas`],
    /// for the same anti-drift reason as [`Self::part_hits`].
    #[must_use]
    pub fn part_bounds_canvas(&self, object: usize, part: usize) -> Option<Rect> {
        match self.part_kind(object) {
            Some(PartKind::Subpath) => self.subpath_bounds_canvas(object, part),
            Some(PartKind::Run) => self.text_run_bounds_canvas(object, part),
            None => None,
        }
    }

    /// How many parts the object at `index` has, whichever kind they are.
    ///
    /// This is what the Objects panel's tree counts to decide whether a row
    /// gets an expander, and how many child rows it contributes when open.
    #[must_use]
    pub fn part_count(&self, index: usize) -> usize {
        match self.part_kind(index) {
            Some(PartKind::Subpath) => self.subpath_count(index),
            Some(PartKind::Run) => self.text_run_count(index),
            None => 0,
        }
    }

    /// Which **run** (show operator) of the text object at `object` a
    /// canvas-space click lands on — the text-side twin of
    /// [`Self::subpath_hits`].
    ///
    /// A thin adapter over [`pdfce_core::vector::hit_test_text_runs`], with
    /// the same canvas→PDF conversion and the same degenerate-tolerance
    /// fallback its sibling uses. Sharing that fallback matters for the same
    /// reason: without it a click could select a text object and then find
    /// none of its runs, which reads as "the second level is broken" rather
    /// than "the tolerance was zero".
    ///
    /// Nearest first. **Empty for a non-text object, an out-of-range index,
    /// or a text object whose runs could not be laid out** — the core query
    /// deliberately does not fall back to the object's enclosing box there,
    /// because naming run 0 for an object whose runs were never measured
    /// would hand a caller a deletable target that is the wrong one.
    #[must_use]
    pub fn text_run_hits(&self, object: usize, point: Pos2, tolerance: f64) -> Vec<usize> {
        let Some(pdf) = self.canvas_to_pdf(point) else {
            return Vec::new();
        };
        pdfce_core::vector::hit_test_text_runs(&self.objects, object, pdf, resolve(tolerance))
    }

    /// How many runs the text object at `object` has, or `0` for anything
    /// else — the text twin of [`Self::subpath_count`], and `0` for the same
    /// reason it is: a path has no runs, and a loop over none of them is
    /// exactly the right amount of work.
    #[must_use]
    pub fn text_run_count(&self, object: usize) -> usize {
        match self.objects.objects.get(object) {
            Some(VectorObject::Text(t)) => t.runs.len(),
            _ => 0,
        }
    }

    /// A text run's bounds in **canvas** space, for drawing its outline.
    ///
    /// Same argument as [`Self::subpath_bounds_canvas`]: the object's own
    /// bounds would draw a rectangle around every label on the sheet and
    /// tell the operator they had selected the whole thing again — which is
    /// the misunderstanding entering the object exists to resolve. On the
    /// measured CAD export that rectangle spans the entire drawing.
    #[must_use]
    pub fn text_run_bounds_canvas(&self, object: usize, run: usize) -> Option<Rect> {
        let Some(VectorObject::Text(t)) = self.objects.objects.get(object) else {
            return None;
        };
        self.pdf_bounds_to_canvas(t.runs.get(run)?.bounds)
    }

    /// Whether deleting run `run` of text object `object` would be refused
    /// because the run AFTER it has no position of its own (§9.4.2).
    ///
    /// A pure query the shell asks **before** offering the control (R83),
    /// answered from the same `positioned_by` flag
    /// [`pdfce_core::edit::EditSession::delete_text_run`] refuses on — so a
    /// disabled affordance and the verb cannot disagree about which runs are
    /// deletable.
    ///
    /// `false` for a non-text object or an out-of-range index: there is no
    /// deletion to refuse.
    #[must_use]
    pub fn text_run_delete_would_move_next(&self, object: usize, run: usize) -> bool {
        let Some(VectorObject::Text(t)) = self.objects.objects.get(object) else {
            return false;
        };
        // The LAST run is never refused — nothing follows it to be moved.
        // And a single-run object deletes the whole text object, which the
        // core verb allows unconditionally.
        t.runs.len() > 1
            && t.runs.get(run + 1).is_some_and(|next| {
                next.positioned_by == pdfce_core::vector::RunPositioning::Inherited
            })
    }

    /// A subpath's bounds in **canvas** space, for drawing its outline.
    ///
    /// The object's own bounds would draw a rectangle around the entire
    /// drawing and tell the operator they had selected the whole thing again
    /// — which is the misunderstanding entering the object exists to resolve.
    #[must_use]
    pub fn subpath_bounds_canvas(&self, object: usize, subpath: usize) -> Option<Rect> {
        let b = pdfce_core::vector::subpath_bounds(&self.objects, object, subpath)?;
        self.pdf_bounds_to_canvas(b)
    }

    /// The page-space anchor sample points of the object at paint-order
    /// `index` — the circular best-fit tool's fit input.
    ///
    /// A path object contributes every anchor of every subpath, in **PDF
    /// user / page space** (the frame [`Self::page_objects`] stores and
    /// [`fit_circle_taubin`](pdfce_core::dimension::fit_circle_taubin)
    /// consumes); a text/image/form object (or an out-of-range index)
    /// contributes nothing — they carry no snap/fit node geometry, the same
    /// exclusion the snap engine applies. Reuses the ONE decomposition this
    /// provider already built, never a second `decompose_page`.
    #[must_use]
    pub fn object_sample_points(&self, index: usize) -> Vec<Point> {
        match self.objects.objects.get(index) {
            Some(VectorObject::Path(path)) => path
                .page_subpaths()
                .iter()
                .flat_map(|sp| sp.anchors().collect::<Vec<_>>())
                .collect(),
            _ => Vec::new(),
        }
    }

    /// How many parts (subpaths) the path object at paint-order `index` has,
    /// or `0` for a non-path object.
    ///
    /// Exists so a caller can iterate an object's parts without reaching
    /// into `objects.objects` and re-doing the `VectorObject::Path` match at
    /// a call site whose job is drawing rows. `0` for a non-path is the
    /// honest answer rather than an `Option`: a text run has no subpaths, and
    /// a loop over none of them is exactly the right amount of work.
    #[must_use]
    pub fn subpath_count(&self, index: usize) -> usize {
        match self.objects.objects.get(index) {
            Some(VectorObject::Path(path)) => path.page_subpaths().len(),
            _ => 0,
        }
    }

    /// The anchors of ONE subpath, each paired with its **object-scoped**
    /// index — the Point rung's pick set (decision 028 §Q1).
    ///
    /// # Why not [`Self::object_sample_points`], which already returns anchors
    ///
    /// That one returns the whole object's flat list, and using it as a node
    /// pick set is a hazard decision 028 found already shipped: on a measured
    /// CAD export one path object holds **6,681 anchors**, so "the nearest
    /// anchor to the press" can easily belong to a subpath the operator is
    /// not pointing at, and nothing is drawn beforehand to say which. Scoping
    /// the pick set to the ENTERED subpath is what makes the grab predictable
    /// — the operator can only hit points they descended into and can see.
    ///
    /// The same number is why the Objects panel nests points under a *part*
    /// rather than listing an object's anchors directly: 6,681 sibling rows
    /// under one object is not a tree, it is a wall.
    ///
    /// # Why the index is object-scoped even though the set is subpath-scoped
    ///
    /// Decision 025 §1.3(b): the number pdfce shows and the number
    /// `pdfce-cli node-move --node N` addresses must be the same number.
    /// `vector::anchor_count` counts across the whole object, so the running
    /// offset is added here rather than letting the GUI invent a second
    /// numbering that would disagree with every other consumer.
    ///
    /// Returns empty for a non-path object or an out-of-range index — the
    /// same exclusion [`Self::object_sample_points`] applies, for the same
    /// reason (text and image objects are not node-editable, decision 011
    /// §2.1).
    #[must_use]
    pub fn subpath_node_points(&self, index: usize, subpath: usize) -> Vec<(usize, Point)> {
        let Some(VectorObject::Path(path)) = self.objects.objects.get(index) else {
            return Vec::new();
        };
        let subpaths = path.page_subpaths();
        // The running offset IS the object-scoped index of the target
        // subpath's first anchor, because `anchor_count` flattens the same
        // walk in the same order.
        let mut offset = 0usize;
        for (i, sp) in subpaths.iter().enumerate() {
            let anchors: Vec<Point> = sp.anchors().collect();
            if i == subpath {
                return anchors
                    .into_iter()
                    .enumerate()
                    .map(|(k, p)| (offset + k, p))
                    .collect();
            }
            offset += anchors.len();
        }
        Vec::new()
    }

    /// **Every** anchor of the path object at paint-order `index`, each with
    /// its object-scoped index — [`Self::subpath_node_points`] flattened
    /// across all subpaths.
    ///
    /// # Why the whole object and not one subpath
    ///
    /// A multi-node **selection** is object-scoped: nothing stops an operator
    /// Ctrl-clicking one anchor on a shape's outer subpath and another on a
    /// hole inside it, and a selection set holds both by their object-scoped
    /// index. A multi-node **drag** therefore has to look up positions across
    /// the whole object — asking per-subpath would mean the caller
    /// re-deriving which subpath each selected index falls in, which is
    /// exactly the offset arithmetic [`Self::subpath_node_points`] exists to
    /// keep in one place.
    ///
    /// Empty for a non-path object, for the same reason
    /// [`Self::subpath_count`] returns `0`.
    #[must_use]
    pub fn object_node_points(&self, index: usize) -> Vec<(usize, Point)> {
        let Some(VectorObject::Path(path)) = self.objects.objects.get(index) else {
            return Vec::new();
        };
        path.page_subpaths()
            .iter()
            .flat_map(|sp| sp.anchors())
            .enumerate()
            .collect()
    }

    /// The Bézier control points ("handles") of one subpath, each tagged with
    /// the **object-scoped index of the node it belongs to** and which side
    /// of that node it shapes.
    ///
    /// # Which handle belongs to which node
    ///
    /// A cubic segment carries two control points, and they belong to
    /// *different* nodes — this is the part that is easy to get backwards.
    /// Segment `k` runs from anchor `k` to anchor `k+1`, so its `c1` shapes
    /// the curve LEAVING anchor `k` and its `c2` shapes the curve ARRIVING
    /// at anchor `k+1`. That is exactly the split
    /// [`pdfce_core::vector::Handle`] names, and it is why the enum is worded
    /// by direction of travel rather than "first/second": first-and-second
    /// are properties of a *segment*, and a segment says nothing about which
    /// node the operator selected.
    ///
    /// Straight segments contribute nothing. pdfce refuses to invent a handle
    /// for a line — turning a line into a curve is a different operation with
    /// a different name — so a node with no curve on a side simply has no
    /// mark there, and the absence is stated in the readout rather than drawn
    /// as a ghost (decision 028 §Q2).
    ///
    /// `v`/`y` implicit control points need no special handling here: the
    /// decomposition already resolves them into explicit `c1`/`c2`
    /// (`Segment::Cubic`'s own doc comment), so this sees one uniform shape
    /// and the promotion-to-`c` happens far downstream in the planner.
    #[must_use]
    pub fn subpath_handle_points(
        &self,
        object: usize,
        subpath: usize,
    ) -> Vec<(usize, Handle, Point)> {
        let Some(VectorObject::Path(path)) = self.objects.objects.get(object) else {
            return Vec::new();
        };
        let subpaths = path.page_subpaths();
        let mut offset = 0usize;
        for (i, sp) in subpaths.iter().enumerate() {
            let anchors = sp.anchors().count();
            if i != subpath {
                offset += anchors;
                continue;
            }
            let mut out = Vec::new();
            for (k, seg) in sp.segments.iter().enumerate() {
                if let Segment::Cubic { c1, c2, .. } = *seg {
                    // `c1` shapes the curve leaving anchor k …
                    out.push((offset + k, Handle::Outgoing, c1));
                    // … and `c2` shapes the curve arriving at anchor k+1.
                    out.push((offset + k + 1, Handle::Incoming, c2));
                }
            }
            return out;
        }
        Vec::new()
    }

    /// The handle of `subpath` nearest `point` within `tolerance`, as
    /// `(node index, side)` — the Point rung's handle pick.
    ///
    /// # Why handles are hit-tested BEFORE nodes
    ///
    /// A handle sits close to its own node exactly when the curve is nearly
    /// flat there. If the node won ties, the handle would be unreachable
    /// precisely in the case where the operator most wants it — to pull a
    /// flat segment into a curve. Checking the smaller target first is the
    /// standard resolution and the one decision 028 §Q3 specifies.
    ///
    /// `point` is in **PDF page space**, unlike [`Self::nearest_node`]'s
    /// canvas-space input: the only caller is the drag classifier, which has
    /// already converted the press origin to page space to compute the drag's
    /// reference point. Converting back to canvas just to convert forward
    /// again would be two chances to disagree with itself for no benefit.
    #[must_use]
    pub fn nearest_handle(
        &self,
        object: usize,
        subpath: usize,
        pdf: Point,
        tolerance: f64,
    ) -> Option<(usize, Handle)> {
        let mut best: Option<((usize, Handle), f64)> = None;
        for (index, side, p) in self.subpath_handle_points(object, subpath) {
            if !p.is_finite() {
                continue;
            }
            let d = p.distance(pdf);
            if d <= tolerance && best.is_none_or(|(_, bd)| d < bd) {
                best = Some(((index, side), d));
            }
        }
        best.map(|(hit, _)| hit)
    }

    /// The object-scoped index of the anchor of `subpath` nearest `point`
    /// within `tolerance`, or `None` — the Point rung's pick.
    ///
    /// Takes canvas space and converts internally, exactly as
    /// [`Self::subpath_hits`] does, so the canvas→PDF frame conversion stays
    /// in the one place that owns it rather than being re-derived by each
    /// caller. `tolerance` is in PDF units, already converted from screen
    /// pixels by the caller.
    ///
    /// **Ties resolve to the lower index**, which is the same rule the vector
    /// edit tool's own nearest-anchor search uses — so a point equidistant
    /// from two anchors picks the same one whether it was reached by clicking
    /// or by dragging. (That tool lands at S5; the rule is stated here rather
    /// than cross-referenced so it survives being read alone.)
    #[must_use]
    pub fn nearest_node(
        &self,
        object: usize,
        subpath: usize,
        point: Pos2,
        tolerance: f64,
    ) -> Option<usize> {
        let pdf = self.canvas_to_pdf(point)?;
        let mut best: Option<(usize, f64)> = None;
        for (index, p) in self.subpath_node_points(object, subpath) {
            if !p.is_finite() {
                continue;
            }
            let d = p.distance(pdf);
            if d <= tolerance && best.is_none_or(|(_, bd)| d < bd) {
                best = Some((index, d));
            }
        }
        best.map(|(index, _)| index)
    }

    // -----------------------------------------------------------------
    // The canvas target-provider surface.
    //
    // These four were `impl CanvasTargetProvider for ObjectModelProvider`
    // in the old shell. The trait lives in `canvas/` and lands at S4; the
    // methods are inherent here in the meantime, with their signatures and
    // semantics unchanged, so re-attaching the trait is an `impl` block
    // and nothing else.
    // -----------------------------------------------------------------

    /// Every object under the pointer, **front-most first**.
    ///
    /// A thin adapter, as the module docs promise: convert canvas space to
    /// PDF user space, resolve the tolerance, and hand both to
    /// [`pdfce_core::vector::hit_test_point_all`], which owns the geometry.
    ///
    /// The list is what click-through cycling steps through. Without it, an
    /// object completely covered by another is unselectable by any click.
    #[must_use]
    pub fn hit_test_all(&self, page_index: usize, point: Pos2, tolerance: f64) -> Vec<TargetId> {
        if page_index != self.page_index {
            return Vec::new();
        }
        let Some(pdf) = self.canvas_to_pdf(point) else {
            return Vec::new();
        };
        hit_test_point_all(&self.objects, pdf, resolve(tolerance))
            .into_iter()
            .map(|i| TargetId(i as u64))
            .collect()
    }

    /// The **topmost** object under the pointer, or `None`.
    ///
    /// Defined as the head of [`Self::hit_test_all`] rather than as a second
    /// query, so "what does a plain click select?" and "what does cycling
    /// start from?" cannot come to different answers. This was the trait's
    /// *provided* method in the old shell — i.e. the same derivation,
    /// enforced by the trait rather than by this comment; the comment is what
    /// carries the guarantee until the trait comes back.
    #[must_use]
    pub fn hit_test(&self, page_index: usize, point: Pos2, tolerance: f64) -> Option<TargetId> {
        self.hit_test_all(page_index, point, tolerance)
            .into_iter()
            .next()
    }

    /// Every object **fully enclosed** by a canvas-space marquee rect.
    ///
    /// Fully-enclosed rather than touched is the deliberate default
    /// (decision 011, matching Inkscape): a marquee that grabs everything it
    /// grazes is unusable on a dense drawing, which is the document class
    /// pdfce is for. This is the one place that convention is decided.
    #[must_use]
    pub fn hit_test_rect(&self, page_index: usize, rect: Rect) -> Vec<TargetId> {
        if page_index != self.page_index {
            return Vec::new();
        }
        let Some(bounds) = self.canvas_rect_to_pdf_bounds(rect) else {
            return Vec::new();
        };
        hit_test_rect(&self.objects, bounds, MarqueeMode::Enclosed)
            .into_iter()
            .map(|i| TargetId(i as u64))
            .collect()
    }

    /// One object's canvas-space bounding rect, or `None` for a stale id.
    ///
    /// A stale id resolving to `None` rather than panicking is the contract:
    /// a selection set can outlive an edit that removed what it named, and
    /// the correct response is to drop it silently, not to crash the frame
    /// that is trying to draw.
    #[must_use]
    pub fn bounds(&self, page_index: usize, target: TargetId) -> Option<Rect> {
        if page_index != self.page_index {
            return None;
        }
        let obj = self.objects.objects.get(usize::try_from(target.0).ok()?)?;
        self.pdf_bounds_to_canvas(obj.page_bbox())
    }

    // ----------------------------- geometry -----------------------------

    /// Map a canvas-space point into PDF user space (the object model's
    /// frame), or `None` on a degenerate page.
    fn canvas_to_pdf(&self, p: Pos2) -> Option<Point> {
        let inv = self.to_pdf?;
        let mut pts = [SkPoint::from_xy(p.x, p.y)];
        inv.map_points(&mut pts);
        let out = pts[0];
        Some(Point::new(f64::from(out.x), f64::from(out.y)))
    }

    /// Map a PDF-space point into canvas space (for a selection outline).
    fn pdf_to_canvas(&self, p: Point) -> Pos2 {
        // Narrowing to f32 for egui; the object bounds are page geometry,
        // well within f32 range.
        #[allow(clippy::cast_possible_truncation)]
        let mut pts = [SkPoint::from_xy(p.x as f32, p.y as f32)];
        self.to_canvas.map_points(&mut pts);
        Pos2::new(pts[0].x, pts[0].y)
    }

    /// The canvas-space rect enclosing a PDF-space [`Bounds`] under the page
    /// transform (its four corners mapped, then bounded — the transform may
    /// rotate, so the axis-aligned canvas rect is the bound of the mapped
    /// quad).
    fn pdf_bounds_to_canvas(&self, b: Bounds) -> Option<Rect> {
        if b.is_empty() {
            return None;
        }
        let corners = [
            Point::new(b.min.x, b.min.y),
            Point::new(b.max.x, b.min.y),
            Point::new(b.max.x, b.max.y),
            Point::new(b.min.x, b.max.y),
        ];
        let mut rect: Option<Rect> = None;
        for c in corners {
            let p = self.pdf_to_canvas(c);
            rect = Some(match rect {
                None => Rect::from_min_max(p, p),
                Some(r) => r.union(Rect::from_min_max(p, p)),
            });
        }
        rect
    }

    /// The PDF-space bounding box of a canvas-space marquee rect (its four
    /// corners mapped back, then bounded).
    fn canvas_rect_to_pdf_bounds(&self, rect: Rect) -> Option<Bounds> {
        let corners = [
            rect.left_top(),
            rect.right_top(),
            rect.right_bottom(),
            rect.left_bottom(),
        ];
        let mut b = Bounds::EMPTY;
        for c in corners {
            b = b.union_point(self.canvas_to_pdf(c)?);
        }
        if b.is_empty() { None } else { Some(b) }
    }
}

/// Resolve a caller-supplied tolerance, falling back on a degenerate one.
///
/// A degenerate tolerance (`0.0` from a non-finite or zero zoom, or a
/// negative value) would silently make every query a miss. Falling back to
/// the fixed canvas-space value instead is the right trade: *fussy at low
/// zoom* is a far better failure than *selection is broken*.
///
/// Extracted into one function rather than repeated at each of the four call
/// sites that need it, because the four must agree — a click that selects an
/// object and then finds none of its subpaths reads as "the second level is
/// broken" when the real answer is that one site forgot the fallback.
fn resolve(tolerance: f64) -> f64 {
    if tolerance.is_finite() && tolerance > 0.0 {
        tolerance
    } else {
        FALLBACK_SELECT_TOLERANCE
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfce_core::content::ContentStream;
    use pdfce_core::vector::{NoXObjects, decompose};

    /// A provider over a content stream, with an identity canvas transform
    /// (so canvas space == PDF space and the assertions read directly).
    fn provider(src: &[u8]) -> ObjectModelProvider {
        let cs = ContentStream::parse(src.to_vec()).expect("parse");
        let objects = decompose(&cs, Matrix::IDENTITY, &NoXObjects);
        ObjectModelProvider::from_parts(0, objects, Transform::identity())
    }

    #[test]
    fn click_inside_a_filled_rectangle_returns_its_target() {
        // One filled rectangle 10..90 square; a click at its centre hits it.
        let p = provider(b"10 10 80 80 re f");
        let hit = p.hit_test(0, Pos2::new(50.0, 50.0), 3.0);
        assert_eq!(hit, Some(TargetId(0)));
        // A click on empty canvas misses.
        assert_eq!(p.hit_test(0, Pos2::new(200.0, 200.0), 3.0), None);
        // A query for a different page misses regardless.
        assert_eq!(p.hit_test(1, Pos2::new(50.0, 50.0), 3.0), None);
    }

    /// The regression test for the zoom-inverted-tolerance bug: a click that
    /// misses a hairline stroke by 4 canvas units must MISS at a tight
    /// tolerance and HIT at a forgiving one.
    ///
    /// This is what makes the fix meaningful rather than cosmetic. Before it,
    /// the tolerance was hard-coded at 3.0 canvas units at every zoom, so at
    /// "Fit page" (~0.5x on a letter page in a typical window) the operator's
    /// real on-screen catch radius was ~1.5 px and thin geometry could not be
    /// clicked at all. The tolerance now arrives from the caller, scaled by
    /// `1 / zoom`, which keeps the on-screen radius constant.
    ///
    /// The *other half* of that law — that the caller's conversion really is
    /// `1 / zoom` — is asserted in `canvas/` at S4, where the conversion
    /// lives. See this module's header, "What changed at salvage" §4.
    #[test]
    fn selection_tolerance_is_honoured_per_query_not_baked_in() {
        // A zero-width horizontal line at y=20; click 4 units above it.
        let p = provider(b"10 20 m 100 20 l S");
        let near_miss = Pos2::new(50.0, 24.0);

        // Tight tolerance (the old zoomed-out effective radius): a miss.
        assert_eq!(p.hit_test(0, near_miss, 1.5), None);
        // Forgiving tolerance (what a zoomed-out click now supplies): a hit.
        assert_eq!(p.hit_test(0, near_miss, 6.0), Some(TargetId(0)));

        // A degenerate tolerance must NOT silently disable selection — it
        // falls back to the fixed canvas-space value, so a click within
        // 3.0 units still lands.
        assert_eq!(p.hit_test(0, Pos2::new(50.0, 22.0), 0.0), Some(TargetId(0)));
        assert_eq!(
            p.hit_test(0, Pos2::new(50.0, 22.0), f64::NAN),
            Some(TargetId(0))
        );
        assert_eq!(
            p.hit_test(0, Pos2::new(50.0, 22.0), -1.0),
            Some(TargetId(0)),
            "a negative tolerance is degenerate too, and must fall back"
        );
    }

    #[test]
    fn bounds_round_trips_the_object_bbox_into_canvas_space() {
        let p = provider(b"10 10 80 80 re f");
        let r = p.bounds(0, TargetId(0)).expect("bounds");
        // Under the identity transform the canvas rect is the PDF bbox.
        assert!((r.min.x - 10.0).abs() < 1e-3 && (r.min.y - 10.0).abs() < 1e-3);
        assert!((r.max.x - 90.0).abs() < 1e-3 && (r.max.y - 90.0).abs() < 1e-3);
        // A stale target id resolves to nothing rather than panicking.
        assert_eq!(p.bounds(0, TargetId(99)), None);
    }

    #[test]
    fn marquee_encloses_only_fully_contained_objects() {
        // Two rectangles; a marquee over the first only encloses it.
        let p = provider(b"10 10 20 20 re f 200 200 20 20 re f");
        let hits = p.hit_test_rect(
            0,
            Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(100.0, 100.0)),
        );
        assert_eq!(hits, vec![TargetId(0)]);
        // A marquee spanning both encloses both.
        let both = p.hit_test_rect(
            0,
            Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(300.0, 300.0)),
        );
        assert_eq!(both, vec![TargetId(0), TargetId(1)]);
        // Wrong page: nothing.
        assert!(p.hit_test_rect(1, Rect::EVERYTHING).is_empty());
    }

    #[test]
    fn a_text_object_is_selectable_by_its_bbox() {
        // A text object is bbox-only but still a valid target.
        let p = provider(b"BT /F1 12 Tf 40 40 Td (Hi) Tj ET");
        // The show origin (40,40) is inside the inflated text bbox.
        assert!(p.hit_test(0, Pos2::new(40.0, 40.0), 3.0).is_some());
    }

    /// Overlapping objects are all reported, front-most first, in CANVAS
    /// space — the input click-through cycling steps through. Without this
    /// the covered rectangle here is unselectable by any click.
    #[test]
    fn overlapping_objects_are_all_reported_front_most_first() {
        // A small filled rectangle painted first, then a big one over it.
        let p = provider(b"40 40 20 20 re f 0 0 100 100 re f");
        let hits = p.hit_test_all(0, Pos2::new(50.0, 50.0), 3.0);
        assert_eq!(hits, vec![TargetId(1), TargetId(0)]);
        // The topmost query is exactly that list's head.
        assert_eq!(p.hit_test(0, Pos2::new(50.0, 50.0), 3.0), Some(TargetId(1)));
        // Only the cover is under a point outside the covered object.
        assert_eq!(
            p.hit_test_all(0, Pos2::new(5.0, 5.0), 3.0),
            vec![TargetId(1)]
        );
        // A miss is an empty list, and a wrong page is too.
        assert!(p.hit_test_all(0, Pos2::new(500.0, 500.0), 3.0).is_empty());
        assert!(p.hit_test_all(1, Pos2::new(50.0, 50.0), 3.0).is_empty());
    }

    /// The tolerance fallback applies to the all-hits query as well: a
    /// degenerate tolerance must not silently make cycling find nothing when
    /// plain selection would still have found something.
    #[test]
    fn a_degenerate_tolerance_falls_back_for_the_all_hits_query_too() {
        let p = provider(b"10 20 m 100 20 l S");
        let near = Pos2::new(50.0, 22.0);
        assert_eq!(p.hit_test_all(0, near, 0.0), vec![TargetId(0)]);
        assert_eq!(p.hit_test_all(0, near, f64::NAN), vec![TargetId(0)]);
    }

    #[test]
    fn page_objects_feeds_the_snap_engine_from_the_one_decomposition() {
        // The shared accessor: a consumer reads the provider's
        // already-decomposed objects (no second `decompose_page`) and
        // resolves a query in the same PDF/page space `PageObjects` stores.
        use pdfce_core::vector::{Point, SnapConfig, SnapKind, snap_candidates};
        let p = provider(b"10 20 m 100 20 l S");
        let model = p.page_objects();
        let cands = snap_candidates(Point::new(11.0, 21.0), &SnapConfig::new(5.0), model);
        assert_eq!(cands[0].kind, SnapKind::Endpoint);
        assert_eq!(cands[0].point, Point::new(10.0, 20.0));
    }

    /// **The part rung dispatches by object kind, and images have none.**
    ///
    /// This is what the Objects panel's tree builder relies on to decide
    /// whether a row gets an expander. A path with subpaths expands, a text
    /// object with runs expands, an image is a leaf — and the panel asks one
    /// question rather than matching on `VectorObject` itself, which is the
    /// duplicated-predicate drift [`ObjectModelProvider::part_hits`]'s own
    /// docs warn about.
    #[test]
    fn part_kind_and_part_count_answer_for_every_object_kind() {
        let p = provider(b"0 0 m 10 0 l 100 5 m 110 5 l S");
        assert_eq!(p.part_kind(0), Some(PartKind::Subpath));
        assert_eq!(p.part_count(0), 2);

        let t = provider(b"BT /F1 12 Tf 40 40 Td (Hi) Tj ET");
        assert_eq!(t.part_kind(0), Some(PartKind::Run));
        assert_eq!(t.part_count(0), t.text_run_count(0));
        assert_eq!(t.subpath_count(0), 0, "a text object has no subpaths");

        let i = provider(b"q 100 0 0 50 10 10 cm BI /W 1 /H 1 /CS /G /BPC 8 ID \x00 EI Q");
        assert_eq!(i.part_kind(0), None, "an image has no part rung");
        assert_eq!(i.part_count(0), 0);

        // Out of range is a leaf, not a panic.
        assert_eq!(p.part_kind(99), None);
        assert_eq!(p.part_count(99), 0);
    }

    /// A part hit dispatches to the right query for the object's kind.
    #[test]
    fn part_hits_dispatches_to_the_kind_specific_query() {
        let p = provider(b"0 0 m 10 0 l 100 5 m 110 5 l S");
        // A press on the second part must name the second part.
        assert_eq!(p.part_hits(0, Pos2::new(105.0, 5.0), 3.0).first(), Some(&1));
        // An image has no parts, so a press anywhere over it names none.
        let i = provider(b"q 100 0 0 50 10 10 cm BI /W 1 /H 1 /CS /G /BPC 8 ID \x00 EI Q");
        assert!(i.part_hits(0, Pos2::new(50.0, 30.0), 3.0).is_empty());
    }

    /// A part's outline is the PART's box, not the object's.
    ///
    /// The whole reason the rung exists: an object-sized rectangle around a
    /// part tells the operator they selected the whole thing again.
    #[test]
    fn a_part_outline_is_smaller_than_its_objects() {
        let p = provider(b"0 0 m 10 0 l 100 5 m 110 5 l S");
        let part = p.part_bounds_canvas(0, 1).expect("part 1 has bounds");
        let whole = p.bounds(0, TargetId(0)).expect("the object has bounds");
        assert!(
            part.width() < whole.width(),
            "part {part:?} is not narrower than object {whole:?}"
        );
        assert!(whole.contains_rect(part));
        // An out-of-range part is `None`, never the object's own box —
        // returning the object there is how "the second level does nothing"
        // becomes "the second level lies".
        assert_eq!(p.part_bounds_canvas(0, 9), None);
    }
}

/// The Point rung's pick sets: which points belong to which part, and which
/// handle belongs to which node.
///
/// Separate from the module's main test block because these answer a
/// different question — not "does a click find the object" but "does the
/// index the operator sees mean what `node-move --node N` means".
#[cfg(test)]
mod node_rung_tests {
    use super::*;
    use pdfce_core::content::ContentStream;
    use pdfce_core::vector::{NoXObjects, decompose};

    fn provider(src: &[u8]) -> ObjectModelProvider {
        let cs = ContentStream::parse(src.to_vec()).expect("parse");
        let objects = decompose(&cs, Matrix::IDENTITY, &NoXObjects);
        ObjectModelProvider::from_parts(0, objects, Transform::identity())
    }

    /// **Node indices stay OBJECT-scoped across a subpath boundary.**
    ///
    /// This is decision 025 §1.3(b) made testable. The pick set is scoped to
    /// one part, but the numbering is not — because the number pdfce shows
    /// and the number `pdfce-cli node-move --node N` addresses have to be the
    /// same number. A subpath-scoped index would restart at 0 on the second
    /// part and quietly address a point in the first.
    ///
    /// The Objects panel's point rows print these numbers, which is what
    /// makes this a live invariant at S3 rather than an S4 one.
    #[test]
    fn the_second_parts_points_keep_counting_from_the_first() {
        // Two parts of two anchors each: indices 0,1 then 2,3.
        let p = provider(b"0 0 m 10 0 l 100 5 m 110 5 l S");
        let first: Vec<usize> = p
            .subpath_node_points(0, 0)
            .into_iter()
            .map(|(i, _)| i)
            .collect();
        let second: Vec<usize> = p
            .subpath_node_points(0, 1)
            .into_iter()
            .map(|(i, _)| i)
            .collect();
        assert_eq!(first, vec![0, 1]);
        assert_eq!(
            second,
            vec![2, 3],
            "the second part must continue the object's numbering, not restart"
        );
    }

    /// The whole object's flat list agrees with the per-part lists
    /// concatenated.
    ///
    /// Two functions walk the same anchors in the same order and both hand
    /// out object-scoped indices. If they ever disagreed, a multi-node drag
    /// would move a different point from the one the panel row named — and
    /// nothing about that looks wrong at the moment it happens.
    #[test]
    fn the_object_wide_point_list_matches_the_parts_concatenated() {
        let p = provider(b"0 0 m 10 0 l 100 5 m 110 5 l S");
        let mut per_part = Vec::new();
        for part in 0..p.subpath_count(0) {
            per_part.extend(p.subpath_node_points(0, part));
        }
        assert_eq!(p.object_node_points(0), per_part);
    }

    /// The pick set contains ONLY the named part's points.
    ///
    /// The whole reason the rung exists: a measured CAD object holds 6,681
    /// anchors, and offering all of them as a grab target is what made the
    /// old ungated gesture unpredictable.
    #[test]
    fn a_parts_pick_set_excludes_every_other_part() {
        let p = provider(b"0 0 m 10 0 l 100 5 m 110 5 l S");
        let pts: Vec<Point> = p
            .subpath_node_points(0, 1)
            .into_iter()
            .map(|(_, q)| q)
            .collect();
        assert_eq!(pts.len(), 2);
        assert!(
            pts.iter().all(|q| q.x >= 100.0),
            "part 1's pick set must not contain part 0's points: {pts:?}"
        );
    }

    /// **A cubic's two control points belong to DIFFERENT nodes** — the thing
    /// most likely to be implemented backwards.
    ///
    /// Segment k runs from anchor k to anchor k+1, so `c1` shapes the curve
    /// LEAVING anchor k and `c2` shapes the curve ARRIVING at anchor k+1.
    /// Assigning both to one node would look plausible, draw two handles in
    /// roughly the right place, and make every handle drag move the wrong end
    /// of the curve.
    #[test]
    fn a_cubics_two_handles_belong_to_the_nodes_at_its_two_ends() {
        // m(0,0) then c with c1=(10,40) c2=(60,40) to=(70,0).
        // Anchors: 0 -> (0,0), 1 -> (70,0).
        let p = provider(b"0 0 m 10 40 60 40 70 0 c S");
        let hs = p.subpath_handle_points(0, 0);
        assert_eq!(hs.len(), 2, "one cubic contributes exactly two handles");

        let outgoing = hs
            .iter()
            .find(|(_, s, _)| *s == Handle::Outgoing)
            .expect("c1");
        assert_eq!(outgoing.0, 0, "c1 shapes the curve LEAVING anchor 0");
        assert_eq!(outgoing.2, Point::new(10.0, 40.0));

        let incoming = hs
            .iter()
            .find(|(_, s, _)| *s == Handle::Incoming)
            .expect("c2");
        assert_eq!(incoming.0, 1, "c2 shapes the curve ARRIVING at anchor 1");
        assert_eq!(incoming.2, Point::new(60.0, 40.0));
    }

    /// **A straight segment contributes no handle, and none is invented.**
    ///
    /// pdfce refuses to turn a line into a curve without being asked, so the
    /// absence must show up as nothing drawn — not as a placeholder sitting
    /// on the node, which would advertise an edit that will be refused.
    #[test]
    fn a_straight_part_has_no_handles_at_all() {
        let p = provider(b"0 0 m 10 0 l 20 0 l S");
        assert!(p.subpath_handle_points(0, 0).is_empty());
    }

    /// `v` and `y` resolve to explicit control points before they get here.
    ///
    /// Worth pinning because the GUI would otherwise need to know about the
    /// short spellings, and getting `v` (first control = current point) and
    /// `y` (second control = endpoint) confused is the classic error in this
    /// operator family.
    #[test]
    fn the_short_curve_spellings_still_yield_two_handles() {
        // `v`: c1 is implicitly the current point (0,0), c2 = (60,40).
        let p = provider(b"0 0 m 60 40 70 0 v S");
        let hs = p.subpath_handle_points(0, 0);
        assert_eq!(hs.len(), 2, "`v` is a cubic and has both handles resolved");
        let outgoing = hs
            .iter()
            .find(|(_, s, _)| *s == Handle::Outgoing)
            .expect("c1");
        assert_eq!(
            outgoing.2,
            Point::new(0.0, 0.0),
            "`v`'s first control point IS the current point"
        );
    }

    /// A handle grab resolves to the node it belongs to, not to the nearest
    /// node in space.
    #[test]
    fn grabbing_a_handle_names_its_own_node() {
        let p = provider(b"0 0 m 10 40 60 40 70 0 c S");
        // Press right on c2 = (60,40), which is far nearer anchor 1 (70,0)
        // than anchor 0 — and is c2, so it must report node 1 / Incoming.
        let hit = p.nearest_handle(0, 0, Point::new(60.0, 40.0), 2.0);
        assert_eq!(hit, Some((1, Handle::Incoming)));
    }

    /// A node pick resolves to the nearest anchor within tolerance, and
    /// ties go to the lower index.
    #[test]
    fn a_node_pick_takes_the_nearest_anchor_and_ties_go_low() {
        let p = provider(b"0 0 m 100 0 l S");
        assert_eq!(p.nearest_node(0, 0, Pos2::new(2.0, 0.0), 5.0), Some(0));
        assert_eq!(p.nearest_node(0, 0, Pos2::new(98.0, 0.0), 5.0), Some(1));
        // Exactly halfway: the lower index wins.
        assert_eq!(p.nearest_node(0, 0, Pos2::new(50.0, 0.0), 60.0), Some(0));
        // Out of tolerance: nothing, rather than the nearest regardless.
        assert_eq!(p.nearest_node(0, 0, Pos2::new(50.0, 0.0), 5.0), None);
    }

    /// An out-of-range part yields nothing rather than panicking or wrapping.
    #[test]
    fn an_out_of_range_part_yields_no_points_or_handles() {
        let p = provider(b"0 0 m 10 0 l S");
        assert!(p.subpath_node_points(0, 9).is_empty());
        assert!(p.subpath_handle_points(0, 9).is_empty());
        assert!(p.object_node_points(9).is_empty());
        assert!(p.object_sample_points(9).is_empty());
    }
}
