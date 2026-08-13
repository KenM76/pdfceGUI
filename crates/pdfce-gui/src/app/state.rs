//! # `app::state` — what is open, and how its picture stays current
//!
//! Two things live here:
//!
//! 1. [`Status`] and [`OpenDoc`] — the shape of "what, if anything, is
//!    open", and everything one open document owns.
//! 2. The **raster bookkeeping** — [`PdfceApp::settle_and_rasterize`], the
//!    per-frame decision about whether the cached page texture is still a
//!    picture of what the operator is looking at, and if not whether to
//!    re-rasterize now or wait for a zoom gesture to settle.
//!
//! ## What is NOT here, and where it went
//!
//! The two **derived caches** an open document owns — the page decomposition
//! and the font inventory — live in [`crate::app::cache`], along with the
//! four methods that read them. Their fields are still declared on [`OpenDoc`]
//! below, because that is what bounds their lifetime and is the whole point of
//! their move onto the document; only the types, the accessors and their
//! argument moved out.
//!
//! The seam is a real one rather than a cut made to satisfy rule R2's
//! 1,500-line gate: everything left in this file answers *"what is open, and
//! what is the operator looking at?"*, while everything in `cache.rs` answers
//! *"what expensive derived value do several surfaces need, and how is it
//! computed once?"* — a different question, with its own shared hazard
//! (staleness against [`OpenDoc::edit_epoch`]), its own shared device (a `Cell`
//! key beside a `RefCell` payload) and its own documented exemption from the
//! `&mut`-only mutation rule. `cache.rs`'s header carries that argument in
//! full. Splitting anywhere else would have left half an explanation on each
//! side of the cut.
//!
//! ## Three ways to fail, three ways to say so
//!
//! [`Status`] distinguishes what most viewers conflate, and the
//! distinction is carried across from the old shell because it is one of
//! the things pdfce does that its competitors do not:
//!
//! - [`Status::Failed`] — the *file* is wrong: damaged, truncated, not a
//!   PDF. "Something is wrong with your document."
//! - [`Status::Unsupported`] — the file is fine and **pdfce** is not
//!   finished. `pdfce-core` detects such a file and refuses it cleanly
//!   rather than misparsing it into plausible-looking garbage. Presenting
//!   that as "failed to open" would tell the operator a lie about their own
//!   file.
//! - [`Status::NeedsPassword`] — a third thing again: pdfce *can* decrypt
//!   this document and has not been told how.
//!
//! The branch between them is made on **structured error data** from
//! `pdfce-core` ([`DocError`], [`XrefErrorKind`]), never by matching on a
//! message string. That is exactly what "core errors are stable, structured
//! diagnostics" is *for*, and it is what makes the distinction reliable
//! rather than a heuristic that decays as error prose is edited.
//!
//! ## Rendering happens on state change, never per frame
//!
//! egui redraws continuously; rasterizing a PDF page at 60 Hz would be
//! absurd. The canvas holds one cached [`PageTexture`], and the texture
//! carries the [`RenderKey`] it was rendered from — page, raster scale,
//! annotation visibility, layer-override generation. Staleness is that key
//! compared against the one the current view wants, and there is deliberately
//! no second field list to keep in step with it (see [`RenderKey`]'s own
//! docs; a key compared on one side and not the other is a control that
//! ticks and redraws nothing).
//!
//! **Two staleness policies apply**, split by the key's own
//! `discrete_inputs` / `scale_bits` categories, and the difference is the
//! whole of why zoom feels smooth:
//!
//! - **Discrete change — commit immediately.** A page step, an annotation
//!   toggle, a layer toggle. None has a gesture in flight and none has an
//!   intermediate value on the way to it, so any delay is pure latency; for
//!   a page change there is not even a stale texture worth showing, because
//!   it is a picture of a different page.
//! - **Zoom change — debounce by [`ZOOM_SETTLE`]**, drawing the existing
//!   texture scaled to the new size in the meantime. A Ctrl+wheel gesture
//!   emits dozens of zoom values on the way to the one the operator wants;
//!   rasterizing each would burn CPU producing images nobody sees. The
//!   interim scaled texture is soft, not blank or blocky — which is exactly
//!   what every other document viewer does, so it reads as normal rather
//!   than as a glitch. A **discrete** command (Ctrl+0, Ctrl+Plus) bypasses
//!   the debounce through [`OpenDoc::zoom_commanded`]: there is no gesture in
//!   flight, so waiting would just feel unresponsive.
//!
//! [`OpenDoc`] also carries the page decomposition and the font inventory,
//! moved off `crate::panels::PanelsState` at S4 so the document's own
//! lifetime bounds them and no identity key is needed — see
//! [`OpenDoc::page_objects`] — and, from the same stage, the **canvas
//! selection**, moved off `egui::Memory` for the same reason. See
//! [`OpenDoc::selection`].

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use pdfce_core::document::{DocError, Document};
use pdfce_core::edit::EditSession;
use pdfce_core::object::ObjId;
use pdfce_core::page_tree::Page;
use pdfce_core::xref::XrefErrorKind;
use pdfce_render::LayerVisibility;

use crate::app::PdfceApp;
use crate::app::cache::{FontCache, PageObjectCache};
use crate::canvas::selection::SelectionState;
use crate::render::raster::{self, PageTexture};
use crate::render::worker::{RenderKey, RenderRequest, RenderWorker, RenderedPixels};
use crate::viewer::{self, ViewState};

/// How long a zoom must stop changing before it is committed to a real
/// rasterization.
///
/// Long enough to swallow a whole wheel gesture, short enough that a
/// deliberate single step does not feel laggy. 150 ms is the value the old
/// shell settled on against real CAD sheets; it is a constant rather than a
/// literal so the next person to tune it does so once, with a paper trail.
const ZOOM_SETTLE: Duration = Duration::from_millis(150);

/// What, if anything, is open.
///
/// `Box`ed in the `Open` arm because [`OpenDoc`] is much larger than the
/// error arms and an un-boxed enum would make every `Status` move copy it.
#[derive(Default)]
pub enum Status {
    /// Nothing open. The start-up state when no path was given on the
    /// command line.
    #[default]
    Empty,
    /// A document is open.
    Open(Box<OpenDoc>),
    /// The file is damaged, truncated, or not a PDF.
    Failed { path: PathBuf, message: String },
    /// The file is well-formed and uses something pdfce does not implement.
    Unsupported { path: PathBuf, message: String },
    /// The file is encrypted and pdfce has not been given the password.
    NeedsPassword { path: PathBuf },
}

/// Where the pointer was over the page when a Ctrl+wheel arrived.
///
/// Recorded on the frame the wheel is seen, consumed on the next one, so
/// the scroll offset can be moved to keep that point still. See
/// [`crate::canvas::geometry::zoom_anchor_offset`].
///
/// **It has to span two frames**, and that is not an implementation
/// detail: the new zoom is not known when the wheel is seen. The zoom is an
/// [`crate::app::actions::Action`] applied after the UI is built, and it
/// *clamps* — so the only honest source of "how big is the page now" is the
/// next frame's own display size. Recording the *inputs* and solving later
/// avoids predicting a clamp we do not control.
#[derive(Debug, Clone, Copy)]
pub struct ZoomAnchor {
    /// The pointer's position as a fraction of the page's drawn size.
    pub frac: (f32, f32),
    /// The scroll offset before the zoom step.
    pub offset_before: (f32, f32),
    /// The page's drawn size before the zoom step.
    pub display_before: (f32, f32),
    /// The scroll viewport, needed for the centring-margin term.
    pub viewport: (f32, f32),
}

/// Which optional-content groups the operator has hidden, if any.
///
/// # ★ `None` is not "hide nothing"
///
/// `pdfce_render::LayerVisibility` **replaces** the document's own default
/// configuration rather than merging with it (core API trap T-12.9). So:
///
/// | state | meaning |
/// |---|---|
/// | `hidden: None` | obey the document's `/D` configuration (§8.11.4.3) |
/// | `hidden: Some({})` | show **every** layer, including ones the document turns off |
/// | `hidden: Some({…})` | exactly these are hidden |
///
/// Collapsing the first two would silently reveal every layer a document had
/// turned off, which on a drawing with a "Confidential" watermark layer is a
/// disclosure defect rather than a cosmetic one.
///
/// That is also why a set is stored rather than operator *deltas*: the
/// renderer wants the complete answer, so the complete answer is what is
/// held. A delta would have to be resolved against the document's defaults at
/// render time, in a second place, with the merge rules the engine
/// deliberately refused to define.
///
/// # ★ The operator's toggle is session-only, and nothing here can save it
///
/// §8.11.2.1 puts the live state outside the document entirely: the toggle is
/// *"session-only state, held nowhere the save path can see it"*, lost on
/// reopen. That is a property of the format rather than a gap in this build,
/// it is what `crate::text::panels::layers_session_only_note` discloses, and
/// it is why changing it must not bump [`OpenDoc::edit_epoch`].
#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct LayerOverride {
    /// The complete hidden set, or `None` to obey the document.
    hidden: Option<BTreeSet<ObjId>>,
    /// How many times the above has changed.
    ///
    /// The render staleness key — see
    /// [`crate::render::worker::RenderKey`], whose own docs explain why a
    /// counter beats comparing the set on every frame. `0` is the
    /// never-touched state, which is exactly `hidden: None`.
    generation: u64,
}

/// One open document and everything the shell knows about looking at it.
pub struct OpenDoc {
    /// Where it came from.
    ///
    /// Read by [`PdfceApp::open_path`], which hands it to
    /// [`crate::app::recent::RecentFiles::remember`] — so the recent list is
    /// built from the path the document was *actually* opened from rather
    /// than from whatever the caller happened to have, which is the same
    /// distinction that makes the recording conditional on the open having
    /// succeeded. Still also what the window title becomes (`<file> — pdfce`)
    /// and what a document switcher will need.
    pub path: PathBuf,
    /// The edit session — the single owner of the document, through which
    /// every future mutation will pass.
    ///
    /// **An `Arc` for one specific reason**: a render worker holds a clone
    /// for as long as it rasterizes, so the borrow can cross the thread
    /// boundary and `session.view()` can be called on the far side. That is
    /// also why every future mutation must go through a path that first
    /// calls [`RenderWorker::cancel_and_wait`] — `Arc::get_mut` fails while
    /// a render is running, and the alternatives were rejected with numbers
    /// (see that method).
    pub session: Arc<EditSession>,
    /// The flattened page vector, resolved once at open.
    pub pages: Vec<Page>,
    /// Which page, at what zoom, chosen how.
    pub view: ViewState,
    /// The cached raster, or `None` before the first one arrives.
    pub page_texture: Option<PageTexture>,
    /// Why the current page would not draw, if it would not.
    ///
    /// Held rather than propagated: the document is still open and the
    /// operator can still navigate away from a page that will not draw.
    pub render_error: Option<String>,
    /// The single-slot background rasterizer.
    pub render_worker: RenderWorker,
    /// The zoom seen at the end of the previous frame, used to detect that
    /// the zoom changed at all.
    pub observed_zoom: f32,
    /// The earliest instant at which the current zoom may be committed to a
    /// real rasterization — the [`ZOOM_SETTLE`] debounce deadline.
    pub zoom_commit_at: Instant,
    /// Set by any *discrete* zoom command during this frame's action
    /// dispatch, and consumed at the end of the frame. It is what
    /// distinguishes "the operator pressed Ctrl+0" (commit at once) from
    /// "the operator is mid-wheel-gesture" (wait for the gesture to settle).
    pub zoom_commanded: bool,
    /// See [`ZoomAnchor`]. Written by the canvas, consumed by the canvas on
    /// the following frame.
    pub zoom_anchor: Option<ZoomAnchor>,
    /// Bumped by **every action that changes the document's content**.
    ///
    /// Not a document version in any meaningful sense — it is a cheap "has
    /// anything been edited since I last looked?" token, and the only thing
    /// that reads it is [`OpenDoc::trace_object_count`]. It exists because
    /// the object count that `PROJECT_PLAN.md` §4.3 requirement 3 asks for is
    /// **expensive to compute** (`decompose_page` fully decodes, tokenizes
    /// and walks every content stream on the page, with no cache anywhere in
    /// `pdfce-core`), so it must not be recomputed per frame, and the only
    /// honest cheap test for "is my last count still true?" is "has an edit
    /// happened".
    ///
    /// **Stage S2 never bumps it**, because stage S2 has no editing action —
    /// `Action` is zoom and page navigation, and both leave the content
    /// untouched. This field is therefore the documented *seam*: the first
    /// mutating arm added to [`crate::app::actions::PdfceApp::apply`] must
    /// bump it, and the count then re-traces on the following frame with no
    /// further work. A mutating action that forgets is a stale `objects=`
    /// line, which is why the bump belongs in the funnel every mutation is
    /// already required to pass through rather than at each verb.
    pub edit_epoch: u64,
    /// The `(page index, edit epoch)` the last `objects` line was traced for,
    /// or `None` before the first one.
    ///
    /// Both halves are needed: the count is a property of *this page* in
    /// *this revision*, so paging away and back must re-trace (the count is
    /// different) and an edit must re-trace (the count may be different).
    pub objects_traced_for: Option<(usize, u64)>,
    /// The scroll offset the canvas settled on at the end of the last frame.
    ///
    /// Kept because middle-drag panning has to compute "where the view
    /// should be now" BEFORE the scroll area is built, and the area's own
    /// state is only readable after. Storing last frame's settled value
    /// lets the pan be applied in the same frame as the movement rather
    /// than a frame late — which is the difference between panning that
    /// tracks the hand and panning that lags it.
    pub last_scroll_offset: egui::Vec2,
    /// ★ **What the operator has selected on the canvas.**
    ///
    /// # Why it is a field of the document rather than a value in `egui::Memory`
    ///
    /// It was the latter until this stage, and the move is the same argument
    /// the decomposition cache's move made, applied to state rather than to a
    /// cache. A selection is **document-scoped**: closing a document must
    /// forget it, and a selection retained across an open would name paint-order
    /// indices in a file that no longer exists.
    ///
    /// `egui::Memory` outlives documents, so the canvas had to *detect* the
    /// change — with a `DocumentToken` built from the `Arc<EditSession>`'s
    /// allocation address mixed with the page count, compared once per frame,
    /// forgetting everything when it moved. That is the identical shape, and
    /// the identical hazard, as the `panels::DocKey` this stage deleted: an
    /// address is not an identity, a freed allocation can be reused, and
    /// holding an `Arc` or a `Weak` to make it one would disable editing
    /// outright (`Arc::get_mut` fails while any other strong **or weak**
    /// reference exists — see [`Self::session`]).
    ///
    /// Here the question does not arise. [`Self::new`]'s own doc comment is the
    /// whole proof: *"opening a document constructs a whole new `OpenDoc`, so a
    /// cached texture or a page index can never refer to a page from a previous
    /// file."* A selection **inside** that structure inherits the guarantee for
    /// free, by construction rather than by comparison, on every frame, at zero
    /// cost. So `DocumentToken` and `SelectionState::sync_document` were
    /// deleted rather than repaired.
    ///
    /// # ★ Public, and why that does not breach actions-not-mutations
    ///
    /// `crate::app::actions`' invariant is that **no code path runs from a
    /// widget to a *document***. A selection is not the document: it names
    /// parts of it and changes nothing that a save would write. It sits with
    /// [`Self::last_scroll_offset`] and [`Self::zoom_anchor`] as per-document
    /// *view* state the canvas is permitted to write directly, and for the same
    /// reason they are — it is settled during the frame, from input that only
    /// exists during the frame, and deferring it would make a click land one
    /// frame after the operator made it.
    ///
    /// The **edit** that a selection leads to is still an action, and that is
    /// the line: Delete does not remove anything here. It raises
    /// [`crate::app::actions::Action::DeleteSelection`] carrying the operand
    /// list, applied after the frame through the one funnel, exactly as before
    /// the move. Nothing that touches `EditSession` moved.
    ///
    /// # Who writes it
    ///
    /// `crate::canvas` — the click, marquee, Escape and Delete gestures — and
    /// nothing else. Everything outside the canvas **reads**: the ribbon's
    /// `selection.any` condition, the Format tab's Delete, and (from the next
    /// stage) the Properties panel. A second writer would be a second
    /// selection model, which is precisely what
    /// `panels::PanelsState::focus`'s docs refuse to become.
    pub selection: SelectionState,
    /// The current page's decomposition. Read through
    /// [`Self::page_objects`]; see [`crate::app::cache`] for why it is a cache,
    /// why it is behind a `RefCell`, and why the accessors live there while the
    /// field lives here.
    pub(super) page_objects: PageObjectCache,
    /// The document's font inventory. Read through
    /// [`Self::font_inventory`]; see [`crate::app::cache`].
    pub(super) fonts: FontCache,
    /// Whether annotation appearances (`/AP` `/N`, §12.5) are painted over
    /// the page content.
    ///
    /// `true` at open, because that is what a reader does with a file it was
    /// handed and what [`pdfce_render::RenderOptions`] defaults to. Read
    /// through [`Self::annotations_visible`], changed through
    /// [`Self::set_annotations_visible`].
    ///
    /// **This is view state, not document state.** It changes what is drawn
    /// and nothing that is saved, which is why it lives beside `view` and
    /// dies with the document rather than bumping [`Self::edit_epoch`].
    annotations: bool,
    /// The operator's optional-content override. See [`LayerOverride`].
    layers: LayerOverride,
}

impl OpenDoc {
    /// Build the state for a freshly opened document.
    ///
    /// Everything starts fresh, deliberately: opening a document
    /// constructs a whole new `OpenDoc`, so a cached texture or a page
    /// index can never refer to a page from a previous file. A `reset()`
    /// method would be a second, weaker way to achieve the same thing and
    /// an invitation to reuse an `OpenDoc` across documents — which is
    /// exactly the stale-state bug that constructing fresh state prevents
    /// by design.
    ///
    /// `pub(crate)` rather than private so a panel's own test can build the
    /// document state its body reads, through the **same** constructor
    /// [`PdfceApp::open_path`] uses. A test-only alternative constructor
    /// would be a second way to assemble an `OpenDoc`, which is precisely
    /// what this function's own argument says not to have.
    pub(crate) fn new(path: PathBuf, session: EditSession, pages: Vec<Page>) -> Self {
        let view = ViewState::default();
        Self {
            path,
            session: Arc::new(session),
            pages,
            observed_zoom: view.zoom,
            view,
            page_texture: None,
            render_error: None,
            render_worker: RenderWorker::default(),
            // In the past, so the first zoom change commits at once rather
            // than waiting out a debounce nobody started.
            zoom_commit_at: Instant::now(),
            zoom_commanded: false,
            zoom_anchor: None,
            edit_epoch: 0,
            objects_traced_for: None,
            last_scroll_offset: egui::Vec2::ZERO,
            // Empty, like everything else here — and that is the entire
            // mechanism by which a selection can never refer to a previous
            // file. See the field's own docs.
            selection: SelectionState::default(),
            page_objects: PageObjectCache::default(),
            fonts: FontCache::default(),
            // What a reader shows. `pdfce_render::RenderOptions` defaults the
            // same way, and agreeing with it means a document opened here and
            // a page rendered by `pdfce-cli` start from the same picture.
            annotations: true,
            // `None`, meaning "obey the document's own default configuration"
            // — which is a distinct state from "hide nothing". See
            // `LayerOverride`.
            layers: LayerOverride::default(),
        }
    }

    /// The page currently being viewed, if the index is in range.
    #[must_use]
    pub fn current_page(&self) -> Option<&Page> {
        self.pages.get(self.view.page_index)
    }

    /// The current page's on-screen extent in PDF user-space units, with
    /// `/Rotate` applied.
    ///
    /// Falls back to a US Letter shape for a document with no pages, so the
    /// fit arithmetic has something finite to divide by. Nothing is drawn
    /// in that state — the canvas shows [`crate::text::canvas_no_pages`] —
    /// so the value is never seen; it exists so the arithmetic upstream of
    /// the check does not have to special-case an empty document as well.
    #[must_use]
    pub fn current_extent(&self) -> (f32, f32) {
        self.current_page()
            .map_or((612.0, 792.0), viewer::page_extent_pts)
    }

    /// Whether annotation appearances are painted over the page content.
    #[must_use]
    pub fn annotations_visible(&self) -> bool {
        self.annotations
    }

    /// Show or hide annotation appearances (§12.5).
    ///
    /// A staleness key, so changing it makes the cached texture stale and the
    /// page re-rasterizes on the next frame — see [`RenderKey`]. That is the
    /// whole difference between this being a control and being a bool nobody
    /// can see.
    ///
    /// **Deliberately does NOT bump [`Self::edit_epoch`]**: nothing about the
    /// document has changed, only what is drawn of it. Bumping would throw
    /// away the decomposition and the font inventory to no purpose, and would
    /// make an `objects n=` line re-trace as though an edit had happened.
    pub fn set_annotations_visible(&mut self, visible: bool) {
        self.annotations = visible;
    }

    /// The complete set of optional-content groups currently hidden.
    ///
    /// The operator's override if there is one, and otherwise the
    /// **document's own** answer from
    /// `pdfce_core::annot::optional_content_default_off` — which is the
    /// print/export-correct `/D`-initial OFF set (§8.11.4.3), and the same
    /// resolution `pdfce_core::layers::read_layers` reports per layer as
    /// `visible_by_default`.
    ///
    /// This is what a visibility control reads to compute the *next* set:
    /// the override replaces the document's configuration rather than merging
    /// with it (T-12.9), so a caller starts from the complete current answer
    /// and hands back a complete new one. Handing in only the groups the
    /// operator touched would show every layer the document had turned off.
    ///
    /// Computed rather than cached: it is read when a control is clicked, not
    /// per frame, and a cached copy would be one more thing to invalidate on
    /// an edit that added a layer.
    #[must_use]
    pub fn hidden_layers(&self) -> BTreeSet<ObjId> {
        self.layers.hidden.clone().unwrap_or_else(|| {
            pdfce_core::annot::optional_content_default_off(&self.session.view())
        })
    }

    /// Replace the operator's optional-content override with `hidden`.
    ///
    /// The **complete** hidden set, for the reason above. Bumps the
    /// generation, which is what makes the cached page texture stale.
    ///
    /// Bumps it even when the set is unchanged, deliberately: comparing two
    /// `BTreeSet<ObjId>`s to save a re-render costs more than the re-render
    /// is likely to, and a control that calls this has by definition just
    /// been clicked. A spurious re-render is a wasted rasterization; a missed
    /// one is a control that appears inert, and those are not equally bad.
    pub fn set_hidden_layers(&mut self, hidden: BTreeSet<ObjId>) {
        self.layers.hidden = Some(hidden);
        self.layers.generation = self.layers.generation.wrapping_add(1);
    }

    /// Show or hide one optional-content group.
    ///
    /// The single-checkbox convenience over [`Self::hidden_layers`] and
    /// [`Self::set_hidden_layers`], seeding from the document's own defaults
    /// on the first toggle so the override starts out agreeing with what the
    /// operator is looking at.
    ///
    /// **It does not apply `/RBGroups` radio semantics.** A group in a radio
    /// group may have at most one member visible at a time (Table 101), so
    /// turning one on has to turn its siblings off — and the sibling list
    /// comes from `pdfce_core::layers::read_layers`, which is the *control's*
    /// reading, not this type's. A control that needs it composes the whole
    /// set and calls [`Self::set_hidden_layers`]; a half-implementation here
    /// would be a second visibility algebra beside the engine's, which is
    /// what the replace-not-merge contract exists to prevent.
    pub fn set_layer_visible(&mut self, group: ObjId, visible: bool) {
        let mut hidden = self.hidden_layers();
        if visible {
            hidden.remove(&group);
        } else {
            hidden.insert(group);
        }
        self.set_hidden_layers(hidden);
    }

    /// Drop the operator's override and go back to obeying the document.
    ///
    /// Distinct from hiding nothing, and the distinction is the whole of
    /// T-12.9: this restores the document's own `/D` configuration, whereas
    /// `set_hidden_layers(BTreeSet::new())` reveals every layer the document
    /// turns off.
    pub fn reset_layers(&mut self) {
        self.layers.hidden = None;
        self.layers.generation = self.layers.generation.wrapping_add(1);
    }

    /// The override to hand a render, or `None` to obey the document.
    fn layer_visibility(&self) -> Option<LayerVisibility> {
        self.layers
            .hidden
            .as_ref()
            .map(|hidden| LayerVisibility::hiding(hidden.iter().copied()))
    }

    /// What a render of the current view would be *of*.
    ///
    /// The staleness key the shell wants, built from the same constructor the
    /// worker labels its output with — see
    /// [`crate::render::worker::RenderKey::new`]. One arithmetic path, so
    /// "what I want" and "what I have" cannot disagree about how a key is
    /// spelled.
    ///
    /// `pub(crate)` for one reason: a panel whose control is blocked on
    /// something else needs to be able to assert that *its* input reaches the
    /// key. `crate::panels::layers` does exactly that — see
    /// `the_render_key_no_longer_blocks_a_layer_toggle` — which is how the
    /// next person to restore that checkbox learns which of its three
    /// preconditions is still open without re-deriving the answer.
    pub(crate) fn render_key(&self, raster_scale: f32) -> RenderKey {
        RenderKey::new(
            self.view.page_index,
            raster_scale,
            self.annotations,
            self.layers.generation,
        )
    }

    /// Hand the current page to a worker and, if it beats the in-frame
    /// budget, absorb the result immediately.
    ///
    /// A failure is recorded in `render_error` rather than propagated: the
    /// document is still open and the operator can still navigate away from
    /// a page that will not draw.
    fn rasterize_current(&mut self, ctx: &egui::Context, raster_scale: f32) {
        let Some(page) = self.pages.get(self.view.page_index) else {
            self.page_texture = None;
            return;
        };
        // `spawn` waits a bounded number of milliseconds inline, so a page
        // that rasterizes quickly returns its pixels here and never touches
        // the asynchronous path — behaviour identical to a synchronous
        // render. A page that misses that budget returns `None` and is
        // collected by `poll_render` on a later frame, with the previous
        // texture staying on screen meanwhile.
        let outcome = self.render_worker.spawn(RenderRequest {
            // The `Arc` is handed over rather than a `DocumentView`, which
            // is what lets the borrow stay local to the worker thread.
            session: Arc::clone(&self.session),
            page: page.clone(),
            page_index: self.view.page_index,
            raster_scale,
            annotations: self.annotations,
            layers: self.layer_visibility(),
            layers_generation: self.layers.generation,
        });
        if let Some(result) = outcome {
            self.absorb_render(ctx, result);
        }
        // Rasterization happens *after* the canvas has already been laid
        // out this frame, so the new texture cannot be drawn until the next
        // one. Without this the display would wait for whatever unrelated
        // input happened to arrive next, which on an idle window is "until
        // the operator wiggles the mouse" — the page would appear to take
        // an arbitrarily long time to show up.
        ctx.request_repaint();
    }

    /// Turn a finished rasterization into the cached texture.
    ///
    /// Shared by the in-frame fast path and the per-frame poll so the two
    /// cannot drift: a render that beat the budget and one that took a
    /// minute must produce exactly the same canvas state.
    fn absorb_render(&mut self, ctx: &egui::Context, result: Result<RenderedPixels, String>) {
        match result {
            Ok(pixels) => {
                self.page_texture = Some(raster::texture_from_pixels(ctx, &pixels));
                self.render_error = None;
            }
            Err(message) => {
                self.page_texture = None;
                self.render_error = Some(message);
            }
        }
    }

    /// Report how many objects the current page holds, on the `PDFCE_DIAG`
    /// channel — on document open, on a page change, and after any edit.
    ///
    /// # Why a count and not a deletion event
    ///
    /// `PROJECT_PLAN.md` §4.3 requirement 3, stated in one sentence:
    ///
    /// > Strictly better evidence than a `delete-objects` event: it measures
    /// > the property the check is about rather than the verb that was meant
    /// > to change it.
    ///
    /// The regression test for D1 asks "did pressing Delete remove the
    /// object?". Against a binary with only a deletion event, the available
    /// evidence is the **absence** of that event, and absence is weak: it
    /// cannot distinguish "the deletion path never ran" from "the deletion
    /// path ran and did nothing" from "the event is traced somewhere the code
    /// did not reach". `ui-verify`'s `delete_key` module admits that evidence
    /// exactly once, under three stated conditions, and says that removing
    /// any one of them turns the check into a SKIP.
    ///
    /// A count needs none of that. Read it before, read it after, compare.
    /// It is also robust to a deletion implemented by some future verb that
    /// nobody remembered to add a trace to — the count measures the page, not
    /// the code path.
    ///
    /// # The line
    ///
    /// ```text
    /// pdfce-diag objects n=412 page=0 paths=380 text=30 images=2 forms=0
    /// ```
    ///
    /// `n=` is the total — the field name `ui-verify`'s vocabulary already
    /// uses for a count — and it is `PageObjects::objects.len()`, the same
    /// index space `EditSession::delete_objects` takes, so "n dropped by one"
    /// and "one object was deleted" are the same statement rather than two
    /// that have to be reconciled. The per-kind breakdown comes free from the
    /// decomposition's own diagnostics and is worth having: a count that
    /// changed by one is more convincing when you can see *which kind* left.
    ///
    /// # Failure is a different event, deliberately
    ///
    /// A page whose content streams will not decode traces
    /// `objects-unavailable page=… reason=…`, **never** `objects` with a
    /// missing or zero `n`. The distinction matters to a consumer: `objects`
    /// is a claim that the count was measured, and a check comparing before
    /// and after must be able to trust that. An `objects` line with no `n`
    /// would read as "the binary stopped reporting", and one with `n=0` would
    /// read as "the page is empty" — a false statement about the document.
    ///
    /// # Cost, and the gate that makes it affordable
    ///
    /// `decompose_page` resolves every `/Contents` stream, inflates it,
    /// concatenates, tokenizes and walks the whole token stream, resolving
    /// fonts as it goes. There is no cache inside `pdfce-core`; the old GUI
    /// keeps exactly one decomposition per page for precisely this reason.
    /// So this is gated twice: it does nothing at all unless tracing is on,
    /// and even then it runs only when `(page, edit_epoch)` has moved. On a
    /// CAD sheet under `PDFCE_DIAG`, that is one decomposition per page
    /// visited, not one per frame.
    ///
    /// The gate is a stored key rather than [`crate::diag::trace_changed`]
    /// on purpose: `trace_changed` de-duplicates the *rendered line*, which
    /// would still require computing the count in order to render it. The
    /// expensive part is the count itself, so the gate has to sit in front of
    /// it.
    ///
    /// # ★ It counts the SHARED decomposition, not one of its own
    ///
    /// Until S4 this ran its own `decompose_page`, because the only cache was
    /// on the panels and this is not a panel — a second decomposition of the
    /// same page, i.e. the *"two decompositions quietly diverge"* pattern
    /// decision 011 warns about, sitting in the code whose entire job is to
    /// report a trustworthy number about that page. It now reads
    /// [`Self::page_objects`], so `n=` is by construction the count of the
    /// objects the Objects panel lists and the canvas hit-tests. The cost
    /// gate is unchanged: nothing is built with tracing off, and with it on
    /// the page decomposes once per `(page, epoch)` — what the private
    /// decomposition already cost, minus the duplicate.
    fn trace_object_count(&mut self) {
        if !crate::diag::enabled() {
            return;
        }
        let key = (self.view.page_index, self.edit_epoch);
        if self.objects_traced_for == Some(key) {
            return;
        }
        // Recorded BEFORE the work, so a decomposition that fails is not
        // retried on every subsequent frame — the failure is deterministic
        // (same bytes, same code), exactly as `settle_and_rasterize` argues
        // for the render error it holds.
        self.objects_traced_for = Some(key);

        if self.current_page().is_none() {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!(
                    "objects-unavailable page={} pages={} reason=no-such-page",
                    self.view.page_index,
                    self.pages.len()
                )
            });
            return;
        }

        // The shared decomposition — see this method's docs. It is built from
        // `session.view()` and not `document.view()`: the edit session's view
        // is the *edited* state (base plus staged changes), which is the
        // state the operator is looking at and the state a delete is meant to
        // have changed. Counting the base revision would report a number that
        // never moves however many objects are removed.
        if let Some(provider) = self.page_objects() {
            let model = provider.page_objects();
            let d = &model.diagnostics;
            // Read out before the closure so the `Ref` is not held across it.
            let (n, paths, text, images, forms) =
                (model.objects.len(), d.paths, d.text, d.images, d.forms);
            let page_index = self.view.page_index;
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!(
                    "objects n={n} page={page_index} paths={paths} text={text} images={images} forms={forms}"
                )
            });
            return;
        }

        // A distinct event, deliberately: `objects` is a claim that the count
        // was measured, and a check comparing before against after must be
        // able to trust that. An `objects` line with no `n` would read as
        // "the binary stopped reporting"; one with `n=0` would read as "the
        // page is empty", which is a false statement about the document.
        let detail = self
            .page_objects_failure()
            .map_or_else(|| "unknown".to_owned(), |err| err.clone());
        let page_index = self.view.page_index;
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("objects-unavailable page={page_index} reason=decompose-failed detail={detail}")
        });
    }

    /// Collect a background render, if one has finished.
    ///
    /// Called once per frame. Returns whether anything was absorbed, so the
    /// caller can request the repaint that draws it.
    fn poll_render(&mut self, ctx: &egui::Context) -> bool {
        let Some(result) = self.render_worker.poll() else {
            return false;
        };
        self.absorb_render(ctx, result);
        true
    }
}

impl PdfceApp {
    /// Open `path`, replacing whatever was open.
    ///
    /// The document is loaded **read-only**: `Document::load` maps the
    /// bytes, `page_tree::pages` flattens the page tree, and nothing here
    /// writes. S0 is a viewer.
    ///
    /// Note the deliberate structure of the match: each `Err` arm is chosen
    /// by *structured* error data, never by inspecting a message. See the
    /// module docs on the three-way failure distinction.
    pub fn open_path(&mut self, path: PathBuf) {
        self.status = match Document::load(&path) {
            Ok(doc) => match pdfce_core::page_tree::pages(&doc) {
                Ok(pages) => {
                    Status::Open(Box::new(OpenDoc::new(path, EditSession::new(doc), pages)))
                }
                // The header and cross-reference table were fine and the
                // page tree is not. That is a damaged file, not an
                // unimplemented feature.
                Err(err) => Status::Failed {
                    path,
                    message: err.to_string(),
                },
            },
            // §7.6: pdfce CAN decrypt this one and has not been told how.
            // Neither damaged nor unsupported — a third thing.
            Err(DocError::PasswordRequired | DocError::PasswordRequiresNormalisation) => {
                Status::NeedsPassword { path }
            }
            Err(err) if is_unsupported_structure(&err) => Status::Unsupported {
                path,
                message: err.to_string(),
            },
            Err(err) => Status::Failed {
                path,
                message: err.to_string(),
            },
        };
        // ★ Forget the panels' own view state, because a NEW DOCUMENT is
        // open and none of it describes anything any more.
        //
        // This is the second half of deleting `panels::DocKey`. The caches it
        // used to guard now live on `OpenDoc` and die with it, but what is
        // left on `PanelsState` — which object rows are expanded, which row
        // the Properties panel is describing — hangs off the *application*
        // and therefore does outlive a document. Those are paint-order
        // indices: positions on one page of one revision, not identities.
        //
        // The old answer was to give the cache a document identity and
        // compare it every frame, which is what needed an `Arc` address and
        // carried the ABA hazard. The answer here is that documents are
        // opened in exactly one place — this function — so forgetting is a
        // single statement at the one moment it is true, and there is no
        // identity to key on at all.
        //
        // Unconditional, including on a failed open: whatever was showing is
        // gone either way, and stale expansion state over a document that
        // could not be read is the worse of the two states to leave behind.
        self.panels.forget_document();

        // ★ Remember the file — but only if it actually opened.
        //
        // The recent list is a list of documents the operator has *read*, and
        // offering one that cannot be opened invites the same failure again
        // from a surface whose whole promise is "this worked before". A file
        // that failed is not lost: it is still wherever the operator got it
        // from, and `Open…` reaches it.
        //
        // Placed here rather than in the `Action::Open` arm on purpose: this
        // is the one function that opens documents, and `argv` reaches it
        // without an action, so a caller-side call would miss the first
        // document of every session — the one an operator is most likely to
        // want back.
        //
        // `remember` absolutizes, de-duplicates, caps and writes; re-opening
        // what is already at the front of the list writes nothing at all.
        if let Status::Open(doc) = &self.status {
            let path = doc.path.clone();
            self.recent.remember(&path);
        }

        // Forget every de-duplicated trace slot, so this document gets its
        // own canvas line and its own region declarations rather than
        // inheriting the previous document's because the numbers happened to
        // match. §4.3 requirement 1 is "at least once per document open", and
        // a consumer is entitled to read that as a line about *this*
        // document. (S2 has no Open command — the path comes from argv — so
        // this fires once today. It is written now because the second open is
        // the one that would silently break it.)
        crate::diag::reset_change_gates();
        crate::diag::trace(|| {
            let kind = match &self.status {
                Status::Empty => "empty",
                Status::Open(d) => {
                    return format!("open ok pages={} path={:?}", d.pages.len(), d.path);
                }
                Status::Failed { .. } => "failed",
                Status::Unsupported { .. } => "unsupported",
                Status::NeedsPassword { .. } => "needs-password",
            };
            format!("open {kind}")
        });
    }

    /// **Close whatever is open and go back to [`Status::Empty`].**
    ///
    /// The other half of [`Self::open_path`], and it forgets exactly what
    /// that function forgets — which is the whole of why it exists as a
    /// sibling rather than as `self.status = Status::Empty` at the call site.
    ///
    /// # What closing has to forget, and why each thing is here
    ///
    /// - **The document itself.** Dropping the [`Status::Open`] box drops the
    ///   `Arc<EditSession>`, the page vector, the cached texture, the
    ///   decomposition and the font inventory, the selection, and the render
    ///   worker — every one of which lives *inside* `OpenDoc` precisely so
    ///   that this is a single move rather than a checklist. `OpenDoc::new`'s
    ///   own docs make the argument from the other direction: state that dies
    ///   with the document belongs on the document.
    /// - **The panels' view state**, through [`crate::panels::PanelsState::forget_document`].
    ///   Expansion sets and the Properties focus are paint-order indices —
    ///   positions on one page of one revision — and they hang off the
    ///   *application*, so they genuinely do outlive a document. Leaving them
    ///   behind means the Objects panel keeps rows expanded for a file that is
    ///   no longer open, which is the same staleness `open_path` forgets for
    ///   the same reason.
    /// - **The de-duplicated trace slots**, so the next document opened in
    ///   this session gets its own canvas line and its own region
    ///   declarations rather than inheriting these because the numbers
    ///   happened to match.
    ///
    /// # What it deliberately does NOT forget
    ///
    /// The **recent list**. Closing a document is not disowning it; it is the
    /// single most likely moment for an operator to reach for the one they
    /// had before it.
    ///
    /// The **dock arrangement** and the **mode**. Those belong to the
    /// operator and outlive every document, which is what
    /// [`crate::app::persistence`] exists to make true across restarts, let
    /// alone across a close.
    pub fn close_document(&mut self) {
        // Traced before the drop, because after it there is nothing left to
        // say which document this was — and "closed" with no name is a line
        // that cannot be matched against the `open` line that preceded it.
        crate::diag::trace(|| match &self.status {
            Status::Open(doc) => format!("close path={:?} pages={}", doc.path, doc.pages.len()),
            Status::Empty => "close nothing-open".to_owned(),
            Status::Failed { path, .. }
            | Status::Unsupported { path, .. }
            | Status::NeedsPassword { path } => format!("close unopened path={path:?}"),
        });
        self.status = Status::Empty;
        self.panels.forget_document();
        crate::diag::reset_change_gates();
    }

    /// **Whether a save is in flight, so an Open or a Close must wait.**
    ///
    /// # The rule, stated where it will be needed
    ///
    /// A document is written by appending an incremental update to a file the
    /// operator names. While that is happening, the bytes on disk are a
    /// partial revision and the `EditSession` the writer is reading from must
    /// not be dropped or replaced. So:
    ///
    /// > **An Open or a Close must not proceed while a save is pending.** The
    /// > operator is asked what to do about it — wait, or discard — and the
    /// > action is applied afterwards or not at all. It is never applied
    /// > underneath the save.
    ///
    /// # Why it answers `false`, and why that is not a stub
    ///
    /// **There is no save in this build.** `file.save` is in
    /// `crate::shell::manifest::PLANNED`, blocked on autosave and crash
    /// recovery; `file.save_copy` is registered and has no dispatch arm. There
    /// is therefore no state in which this could be true, and
    /// `PROJECT_PLAN.md`'s no-placeholders invariant is explicit that the
    /// answer to that is **nothing** — not a confirmation dialog wired to a
    /// condition that cannot occur, and not an `unimplemented!()` waiting for
    /// an operator to find it.
    ///
    /// What this is instead is the **seam**: one predicate, consulted by both
    /// [`crate::app::actions::Action::Open`] and
    /// [`crate::app::actions::Action::Close`], carrying the rule in its own
    /// docs. When the save lands, it reads that subsystem's state and the two
    /// arms grow their confirmation — in one place, already wired, rather
    /// than in two arms somebody has to remember to find. `file.close`'s own
    /// tooltip already promises the operator this behaviour
    /// ("You are asked what to do about unsaved edits first"), which is the
    /// other reason the rule is written down here rather than left implicit:
    /// the promise exists on an operator-visible surface today.
    #[must_use]
    pub fn save_pending(&self) -> bool {
        false
    }

    /// Decide whether the cached page texture is still valid and, if not,
    /// whether to re-rasterize now or wait for a zoom gesture to settle.
    ///
    /// See the module docs, "Rendering happens on state change".
    pub fn settle_and_rasterize(&mut self, ctx: &egui::Context, pixels_per_point: f32) {
        let Status::Open(doc) = &mut self.status else {
            return;
        };

        // The page object count, if it can have changed since it was last
        // reported. Here rather than in the open path because "on open" is
        // only one of the three occasions §4.3 requirement 3 asks for — the
        // others are a page change and an edit, both of which have already
        // been applied by the time this runs (see the frame order in the
        // module docs of `crate::app`). One call site that cannot be
        // forgotten beats three that can.
        doc.trace_object_count();

        // Collect a background render FIRST, before deciding staleness.
        // Order matters: a render that finished since the last frame has
        // already updated `page_texture`'s keys, so polling first is what
        // stops the staleness test below from seeing the pre-render state
        // and spawning a second render for a page that just arrived.
        if doc.poll_render(ctx) {
            ctx.request_repaint();
        }
        // While one is in flight, keep the frames coming. Nothing else
        // wakes egui when a worker finishes — without this the finished
        // page would sit in the channel until the operator moved the mouse,
        // which is the same "arbitrarily long wait" the request_repaint in
        // `rasterize_current` exists to prevent.
        if doc.render_worker.is_rendering() {
            ctx.request_repaint();
        }

        // Did the zoom change since last frame, and by what route?
        let now = Instant::now();
        if (doc.observed_zoom - doc.view.zoom).abs() > f32::EPSILON {
            doc.observed_zoom = doc.view.zoom;
            doc.zoom_commit_at = if doc.zoom_commanded {
                now // discrete command: no gesture in flight, do not wait
            } else {
                now + ZOOM_SETTLE
            };
        }
        doc.zoom_commanded = false;

        // ★ The staleness comparison, and why it is ONE key.
        //
        // "Is the picture on screen still a picture of what the operator is
        // looking at?" is asked of the same `RenderKey` the worker labelled
        // the texture with. Until S4 this compared two hand-picked fields
        // while the worker compared its own two, and the lists had to be
        // kept in step by review — and a key added to one and not the other
        // compiles, runs, and produces a control that ticks and changes
        // nothing. The categories below are the key's own, so the policy
        // lives with the type rather than being re-derived here.
        let wanted_scale = viewer::raster_scale(doc.view.zoom, pixels_per_point);
        let wanted = doc.render_key(wanted_scale);
        let current = doc.page_texture.as_ref().map(|t| t.key);
        // No texture at all is "stale" in the discrete sense: there is
        // nothing on screen worth waiting to replace.
        let stale_discrete =
            current.is_none_or(|k| k.discrete_inputs() != wanted.discrete_inputs());
        let stale_scale = current.is_some_and(|k| k.scale_bits() != wanted.scale_bits());

        // A page whose previous render failed must not be retried every
        // frame: the failure is deterministic (same bytes, same code), so
        // retrying would peg a core producing the same error. Any discrete
        // change is a genuinely different request and clears the hold —
        // hiding annotations can be exactly what makes a page that would not
        // draw draw.
        if doc.render_error.is_some() && !stale_discrete {
            return;
        }

        if stale_discrete {
            doc.rasterize_current(ctx, wanted_scale);
        } else if stale_scale {
            if now >= doc.zoom_commit_at {
                doc.rasterize_current(ctx, wanted_scale);
            } else {
                // Nothing else will wake egui up when the debounce expires,
                // so schedule it.
                ctx.request_repaint_after(doc.zoom_commit_at - now);
            }
        }
    }
}

/// Whether a load failure is "pdfce is not finished" rather than "your file
/// is broken".
///
/// Matched on the structured error, never on its message. Today the live
/// case is an encryption configuration pdfce will not decrypt (§7.6) —
/// reached either as the cross-reference layer's capability-gap refusal or
/// as a `crypto::EncryptionUnsupported` in its own right.
fn is_unsupported_structure(err: &DocError) -> bool {
    matches!(
        err,
        DocError::Xref(x) if matches!(x.kind, XrefErrorKind::EncryptionUnsupported)
    ) || matches!(err, DocError::Encryption(_))
}

/// Open a fixture the way [`PdfceApp::open_path`] does, without a frame —
/// the same three calls in the same order, so what is under test is the state
/// machine rather than an approximation of it.
///
/// At module level rather than inside `mod tests`, and `pub(super)`, because
/// [`crate::app::cache`]'s tests need the identical starting point: they assert
/// against caches whose fields are declared on [`OpenDoc`], and a second
/// fixture opener would be a second way to assemble one — exactly what
/// [`OpenDoc::new`]'s own docs argue against.
#[cfg(test)]
pub(super) fn open_fixture(rel: &str) -> OpenDoc {
    let path = crate::panels::objects::test_support::engine_fixture(rel);
    let doc = Document::load(&path).expect("the fixture loads");
    let pages = pdfce_core::page_tree::pages(&doc).expect("a page tree");
    OpenDoc::new(path, EditSession::new(doc), pages)
}

/// A four-page document, three objects on every page.
#[cfg(test)]
pub(super) const FOUR_PAGES: &str = "pageops/four-pages.pdf";
/// Four optional-content groups: 4 and 7 on by default, 5 and 6 off.
#[cfg(test)]
pub(super) const PAINTED_LAYERS: &str = "layers/painted-layers.pdf";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::panels::objects::test_support::engine_fixture;

    // =======================================================================
    // The staleness keys that landed at S4
    // =======================================================================

    /// **★ Every input that changes the picture changes the render key.**
    ///
    /// The acceptance criterion for the `RenderKey` completion, from the
    /// shell's side rather than the worker's.
    /// [`PdfceApp::settle_and_rasterize`] asks "is the texture still a
    /// picture of what I am looking at?" by comparing this key, so an input
    /// it does not carry is a control that ticks and redraws nothing.
    #[test]
    fn every_view_input_that_changes_the_picture_changes_the_render_key() {
        let mut doc = open_fixture(PAINTED_LAYERS);
        let base = doc.render_key(2.0);

        assert_ne!(base, doc.render_key(2.5), "the raster scale");

        doc.view.page_index = 1;
        assert_ne!(base, doc.render_key(2.0), "the page");
        doc.view.page_index = 0;

        doc.set_annotations_visible(false);
        assert_ne!(base, doc.render_key(2.0), "annotation visibility");
        doc.set_annotations_visible(true);
        assert_eq!(base, doc.render_key(2.0), "…and back again");

        doc.set_layer_visible(ObjId::new(5, 0), true);
        assert_ne!(base, doc.render_key(2.0), "the layer override");
    }

    /// **A layer or annotation change is DISCRETE, not debounced.**
    ///
    /// A click has no gesture in flight, so waiting out the 150 ms zoom
    /// settle would be latency buying nothing. Asserted through the key's own
    /// categories — what `settle_and_rasterize` reads — so an input that
    /// lands in the wrong one fails here rather than being noticed later as
    /// sluggishness.
    #[test]
    fn a_layer_or_annotation_change_commits_at_once_rather_than_settling() {
        let mut doc = open_fixture(PAINTED_LAYERS);
        let before = doc.render_key(2.0);
        doc.set_layer_visible(ObjId::new(5, 0), true);
        let after = doc.render_key(2.0);
        assert_ne!(after.discrete_inputs(), before.discrete_inputs());
        assert_eq!(
            after.scale_bits(),
            before.scale_bits(),
            "a layer toggle must not look like a zoom, or it inherits the debounce"
        );

        doc.set_annotations_visible(false);
        let hidden = doc.render_key(2.0);
        assert_ne!(hidden.discrete_inputs(), after.discrete_inputs());
        assert_eq!(hidden.scale_bits(), after.scale_bits());
    }

    /// **★ "Obey the document" and "hide nothing" are different renders.**
    ///
    /// Core API trap T-12.9: [`LayerVisibility`] REPLACES the document's
    /// default configuration rather than merging with it, so `None` and
    /// `Some(empty)` are not two spellings of one state. Collapsing them
    /// reveals every layer the document turned off — on a drawing whose
    /// "Confidential" watermark is an off-by-default layer, that is a
    /// disclosure defect, not a cosmetic one.
    #[test]
    fn obeying_the_document_is_not_the_same_as_hiding_nothing() {
        let mut doc = open_fixture(PAINTED_LAYERS);
        assert!(
            doc.layer_visibility().is_none(),
            "a freshly opened document obeys its own configuration"
        );

        doc.set_hidden_layers(BTreeSet::new());
        let showing_all = doc.layer_visibility().expect("an override is in force");
        assert_eq!(showing_all.hidden_count(), 0);

        doc.reset_layers();
        assert!(
            doc.layer_visibility().is_none(),
            "reset must restore `None`, not an empty override"
        );
    }

    /// **The first toggle starts from the DOCUMENT's answer, not from
    /// nothing.**
    ///
    /// [`LayerVisibility`] wants the complete hidden set, so a control that
    /// handed in only the group the operator touched would reveal every
    /// other layer the document had turned off. The fixture declares four
    /// groups, two of them off by default; turning a third off must leave
    /// those two off.
    #[test]
    fn the_first_layer_toggle_seeds_from_the_documents_own_defaults() {
        let mut doc = open_fixture(PAINTED_LAYERS);
        let defaults = doc.hidden_layers();
        assert_eq!(
            defaults.len(),
            2,
            "this fixture must declare layers that are OFF by default, or the \
             seeding path is untested: {defaults:?}"
        );

        doc.set_layer_visible(ObjId::new(4, 0), false);
        let hidden = doc.hidden_layers();
        assert!(
            hidden.contains(&ObjId::new(4, 0)),
            "the operator's own change"
        );
        for id in &defaults {
            assert!(
                hidden.contains(id),
                "the document's own OFF set must survive the first toggle, or \
                 hiding one layer reveals every hidden one: {hidden:?}"
            );
        }

        doc.set_layer_visible(ObjId::new(5, 0), true);
        let hidden = doc.hidden_layers();
        assert!(!hidden.contains(&ObjId::new(5, 0)));
        assert!(hidden.contains(&ObjId::new(6, 0)), "and only that one");
    }

    /// **Every change to the override moves the generation.**
    ///
    /// The generation is the staleness key; the set is not. A mutator that
    /// changed the set and forgot the counter would leave the texture
    /// looking current — the inert-control defect with the override
    /// *correct*, which is the most confusing possible version of it.
    #[test]
    fn every_layer_mutation_moves_the_generation() {
        let mut doc = open_fixture(PAINTED_LAYERS);
        assert_eq!(doc.layers.generation, 0);
        doc.set_layer_visible(ObjId::new(5, 0), true);
        assert_eq!(doc.layers.generation, 1);
        doc.set_hidden_layers(BTreeSet::new());
        assert_eq!(doc.layers.generation, 2);
        doc.reset_layers();
        assert_eq!(doc.layers.generation, 3);
    }

    /// **A view toggle is not an edit.**
    ///
    /// Hiding annotations or a layer changes what is drawn and nothing that
    /// is saved, so it must not bump `edit_epoch` — which would throw away
    /// the decomposition and the font inventory for nothing, and would make
    /// the diagnostic `objects n=` line re-trace as though the document had
    /// changed.
    #[test]
    fn hiding_annotations_or_a_layer_is_not_an_edit() {
        let mut doc = open_fixture(PAINTED_LAYERS);
        let _ = doc.page_objects();
        let _ = doc.font_inventory();

        doc.set_annotations_visible(false);
        doc.set_layer_visible(ObjId::new(4, 0), false);

        assert_eq!(doc.edit_epoch, 0, "no content changed");
        assert_eq!(doc.page_objects.built_for.get(), Some((0, 0)));
        assert_eq!(doc.fonts.built_for.get(), Some(0));
    }

    // =======================================================================
    // The selection move — what replaced `canvas::selection::DocumentToken`
    // =======================================================================

    /// **★ A selection cannot outlive the document it was made on.**
    ///
    /// The `DocumentToken` deletion, asserted rather than argued — the same
    /// shape as `a_documents_decomposition_cannot_outlive_the_document` in
    /// [`crate::app::cache`], because it is the same deletion for the same
    /// reason.
    ///
    /// The old mechanism compared an `Arc` **address** every frame and cleared
    /// on a mismatch; an address is not an identity, and a reused allocation
    /// with a matching page count would have carried a stale selection into a
    /// new file. Here the question cannot be asked: opening a document builds a
    /// whole new `OpenDoc`, so its selection is `SelectionState::default()` by
    /// construction.
    ///
    /// Written as a replacement **in the same binding** — the sequence an
    /// address reuse would have needed — so that reintroducing any kind of
    /// document-identity key here is a test failure rather than a review
    /// finding.
    #[test]
    fn a_selection_cannot_outlive_the_document_it_was_made_on() {
        use crate::canvas::selection::{ClickHit, SelectionLevel};
        use crate::canvas::target::TargetId;

        let mut doc = open_fixture(FOUR_PAGES);
        assert!(
            doc.selection.is_empty(),
            "a freshly opened document has nothing selected"
        );

        doc.selection.click(
            0,
            ClickHit {
                object: Some(TargetId(1)),
                ..ClickHit::default()
            },
            false,
            false,
        );
        assert_eq!(doc.selection.len(), 1);

        doc = open_fixture(PAINTED_LAYERS);
        assert!(
            doc.selection.is_empty(),
            "a new document starts with an empty selection, whatever address \
             its session landed on"
        );
        assert_eq!(
            doc.selection.level(),
            SelectionLevel::Object,
            "…and at the top rung, not inside an object of the previous file"
        );
    }

    // =======================================================================
    // Opening a document is what forgets the panels' state
    // =======================================================================

    /// **★ Opening a document forgets the panels' view state.**
    ///
    /// The second half of the `DocKey` deletion. Expansion sets and the
    /// Properties focus are paint-order indices that live on `PdfceApp`, so
    /// they genuinely do outlive a document. The old answer was to compare a
    /// document identity every frame; the answer here is that documents are
    /// opened in exactly one place, so forgetting is one statement at the one
    /// moment it is true.
    ///
    /// Without it, opening a second document leaves the Objects panel with
    /// rows expanded for a page that no longer exists and the Properties
    /// panel describing whatever object lands at that index in the new
    /// file.
    #[test]
    fn opening_a_document_forgets_the_panels_focus_and_expansion() {
        let mut app = PdfceApp::new();
        app.panels.set_focus(7);
        app.panels.tree_mut().toggle_object(7);
        assert_eq!(app.panels.focus(), Some(7));

        app.open_path(engine_fixture(FOUR_PAGES));
        assert!(matches!(app.status, Status::Open(_)), "the fixture opens");
        assert_eq!(
            app.panels.focus(),
            None,
            "a new document makes every paint-order index meaningless"
        );
        assert!(app.panels.tree_mut().objects_expanded.is_empty());
    }

    /// …and a FAILED open forgets it too.
    ///
    /// Whatever was showing is gone either way, and stale expansion state
    /// over a document that could not be read is the worse of the two states
    /// to leave behind: the panel would look populated while the shell says
    /// the file is damaged.
    #[test]
    fn a_failed_open_forgets_the_panels_state_as_well() {
        let mut app = PdfceApp::new();
        app.panels.set_focus(3);
        app.open_path(engine_fixture("not-a-pdf.bin"));
        assert!(
            matches!(app.status, Status::Failed { .. }),
            "this fixture must fail to open, or the test proves nothing"
        );
        assert_eq!(app.panels.focus(), None);
    }
}
