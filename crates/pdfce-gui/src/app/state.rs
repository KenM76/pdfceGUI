//! # `app::state` — what is open
//!
//! One thing lives here: [`Status`] and [`OpenDoc`] — the shape of "what, if
//! anything, is open", and everything one open document owns.
//!
//! ## What is NOT here, and where it went
//!
//! The **raster bookkeeping** — the per-frame decision about whether the
//! cached page texture is still a picture of what the operator is looking at,
//! and if not whether to re-rasterize now or wait for a zoom gesture to settle
//! — was the second half of this file until Phase 4 and now lives in
//! [`crate::render::settle`]. Phase 4 made it considerably larger (one texture
//! became a texture plus a bounded strip cache, and one staleness question
//! became two), and the seam is the one this header had already named:
//! everything here answers *"what is open, and what is the operator looking
//! at?"*, and everything there answers *"what does the picture need to be, and
//! what should be done about it this frame?"*. Only the second belongs in
//! `render/`, beside the worker it schedules.
//!
//! The **fields** it reads are still declared below, because that is what
//! bounds their lifetime and is the whole point of their living on the
//! document; only the methods moved.
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
//! absurd. The canvas holds one cached [`PageTexture`] for the **current
//! page**, plus — under a continuous page-display mode only — a bounded cache
//! of the *other* visible pages ([`OpenDoc::strip_rasters`]). Each texture
//! carries the [`RenderKey`] it was rendered from — page, raster scale,
//! annotation visibility, layer-override generation — and staleness is that
//! key compared against the one the view wants, with deliberately no second
//! field list to keep in step with it (see [`RenderKey`]'s own docs; a key
//! compared on one side and not the other is a control that ticks and redraws
//! nothing).
//!
//! **The comparison, the zoom debounce and the strip's scheduling all live in
//! [`crate::render::settle`]**, which carries the full argument for both
//! staleness policies. What lives here is the state they read.
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
use std::time::Instant;

use pdfce_core::document::{DocError, Document};
use pdfce_core::edit::EditSession;
use pdfce_core::object::ObjId;
use pdfce_core::page_tree::Page;
use pdfce_core::xref::XrefErrorKind;
use pdfce_render::LayerVisibility;

use crate::app::PdfceApp;
use crate::app::cache::{FontCache, PageObjectCache};
use crate::canvas::selection::SelectionState;
use crate::render::raster::PageTexture;
use crate::render::worker::{RenderKey, RenderRequest, RenderWorker};
use crate::viewer::{self, ViewState};

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
/// ★ **Re-exported, not declared here.** The type moved to [`crate::viewer`]
/// when the rulers landed — R2's ceiling forced a split out of this file, and
/// a zoom fact belongs beside `ViewState::zoom` and `ZOOM_LADDER`; that
/// module's own docs carry the argument. The re-export keeps it a *move*
/// rather than a rename, so `canvas::zoom` still names it by this path.
pub use crate::viewer::ZoomAnchor;

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
    /// The **current page's** cached raster, or `None` before the first one
    /// arrives.
    ///
    /// Its meaning is unchanged by Phase 4 and deliberately so: three surfaces
    /// outside this module depend on this field and on this spelling —
    /// `crate::app::status` reads its `diagnostics`, and both
    /// `crate::app::actions`' `vector_edit` and `crate::panels::forms::edit`
    /// invalidate it by assigning `None`. None is about a strip. The rule that
    /// split buys, enforced rather than remembered: **the current page is never
    /// in [`Self::strip_rasters`]** — see
    /// [`crate::render::strip::StripRasters`], which carries the argument.
    pub page_texture: Option<PageTexture>,
    /// **The other visible pages' rasters**, under a continuous mode.
    ///
    /// Empty for the whole of a single-page session — which is the mechanical
    /// form of "continuous is an option, not a replacement": the default path
    /// allocates nothing here and runs the code it ran before Phase 4. Bounded
    /// by [`crate::render::strip::MAX_CACHED_TEXELS`] and pruned to the
    /// visible set every frame by [`crate::render::settle`].
    pub strip_rasters: crate::render::strip::StripRasters,
    /// **Which pages the canvas drew this frame**, nearest the viewport
    /// centre first.
    ///
    /// Published by [`crate::canvas::show`] during layout and read by
    /// [`crate::render::settle`] after the frame, because "which pages are on
    /// screen" is only knowable once the scroll area has settled — the same
    /// reason [`Self::last_scroll_offset`] is stored rather than derived. It is
    /// the **complete** input to the strip's scheduling: what to keep, what to
    /// evict and what to render next all come from this list and the current
    /// page index. Empty means single page, and the strip pass returns at once.
    pub strip_visible: Vec<usize>,
    /// **The page index the canvas last derived from the scroll position.**
    ///
    /// The one piece of state that tells a *navigation* apart from a *scroll*
    /// under a continuous mode, which matters because both write
    /// [`crate::viewer::ViewState::page_index`]: the canvas writes it every
    /// frame from where the operator has scrolled to and records the same value
    /// here, while a page **command** writes it and not this. So
    /// `page_index != tracked_page` means exactly one thing — something other
    /// than the scroll asked for a different page. See
    /// [`crate::canvas::strip::page_scroll_offset`], which is its only reader
    /// and carries the full argument.
    pub tracked_page: usize,
    /// What the render worker was rendering when this frame's poll took its
    /// slot.
    ///
    /// Read *before* [`crate::render::worker::RenderWorker::poll`] and
    /// consumed by the absorb, for one specific reason: a render **failure**
    /// arrives as a bare message with no [`RenderKey`], so the page it is
    /// about is only knowable from the slot it came out of. Without this a
    /// strip page that would not draw would be attributed to the current page
    /// and blank the whole canvas.
    pub render_in_flight: Option<RenderKey>,
    /// Why the current page would not draw, if it would not.
    ///
    /// Held rather than propagated: the document is still open and the
    /// operator can still navigate away from a page that will not draw.
    ///
    /// **The current page's**, and only its. A strip page that will not draw
    /// records its refusal in [`Self::strip_rasters`] and says so in its own
    /// rect, because one bad sheet in a forty-page set must not replace the
    /// other thirty-nine with a message.
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
    /// **A search hit that has been navigated to and is waiting to be
    /// scrolled into view.** See [`crate::find::Reveal`].
    ///
    /// Sits here beside [`Self::zoom_anchor`] because it is the same *kind* of
    /// thing for the same reason: per-document **view** bookkeeping that has
    /// to span two frames, because the page it targets has not been navigated
    /// to yet on the frame the request is made. Written by
    /// `crate::find::apply` during the apply phase, consumed by
    /// `crate::canvas::show` on the first frame that is actually showing the
    /// hit's page, and abandoned if that frame never comes — the same
    /// hold/solve/drop shape `zoom::anchor_step` documents, minus the zoom.
    ///
    /// On `OpenDoc` rather than on `FindState` — which owns everything else
    /// about Find — by this struct's own rule: state that dies with the
    /// document lives here. A pending scroll position is a fact about *this*
    /// document's pages and means nothing in another one, and putting it here
    /// makes forgetting it free rather than another `forget_document` line
    /// somebody has to remember.
    pub find_reveal: Option<crate::find::Reveal>,
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
    /// ★ **The operator's guide lines**, per page, in canvas space.
    ///
    /// View state, not document content — a guide changes nothing a save would
    /// write — so it sits here beside [`Self::selection`] rather than anywhere
    /// near `EditSession`. It is nevertheless **remembered per document**, in
    /// `guides.txt`, because a guide is *work the operator did* rather than a
    /// switch they flicked; [`crate::canvas::guides`]' header §2 carries the
    /// argument, including why that is a fourth store beside `layout.ron`,
    /// `recent.txt` and `page-display.txt`. Changed through
    /// [`crate::app::actions::Action::SetGuides`], which is what makes the
    /// file write happen once per gesture rather than once per frame of a drag.
    pub guides: crate::canvas::guides::Guides,
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
        // ★ The one field read from disk here rather than started empty, and
        // the one `ViewState` default it overrides. `canvas::guides::opening`
        // owns both halves and the rule joining them.
        let (guides, view) = crate::canvas::guides::opening(&path);
        Self {
            path,
            session: Arc::new(session),
            pages,
            observed_zoom: view.zoom,
            view,
            page_texture: None,
            // Empty by construction, like everything else here. A strip cache
            // carried across an open would hold textures of another file's
            // pages under this file's indices.
            strip_rasters: crate::render::strip::StripRasters::default(),
            strip_visible: Vec::new(),
            // Equal to `view.page_index`, so a freshly opened document is not
            // mistaken for one whose page was navigated to before the first
            // frame — which would scroll a continuous strip on open for no
            // reason the operator asked for.
            tracked_page: 0,
            render_in_flight: None,
            render_error: None,
            render_worker: RenderWorker::default(),
            // In the past, so the first zoom change commits at once rather
            // than waiting out a debounce nobody started.
            zoom_commit_at: Instant::now(),
            zoom_commanded: false,
            zoom_anchor: None,
            // Nothing to reveal on a document nobody has searched yet — and,
            // like every other field here, fresh by construction rather than
            // by a reset somebody has to call.
            find_reveal: None,
            edit_epoch: 0,
            objects_traced_for: None,
            last_scroll_offset: egui::Vec2::ZERO,
            // Empty, like everything else here — and that is the entire
            // mechanism by which a selection can never refer to a previous
            // file. See the field's own docs.
            selection: SelectionState::default(),
            // Read above, before `path` was moved into the struct.
            guides,
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
        self.render_key_for(self.view.page_index, raster_scale)
    }

    /// What a render of **any** page in this view's current settings would be
    /// *of*.
    ///
    /// [`Self::render_key`]'s general form, and the one a continuous strip
    /// needs: every visible page is rendered with the same scale, annotation
    /// stance and layer override, so the only thing that varies between them
    /// is the page index. Written as one function with the current page as a
    /// special case, rather than two, because two would be two places for the
    /// annotation stance to be forgotten — and a key that omitted it would
    /// leave the strip's pages showing annotations after the operator turned
    /// them off, while the current page obeyed.
    pub(crate) fn render_key_for(&self, page_index: usize, raster_scale: f32) -> RenderKey {
        RenderKey::new(
            page_index,
            raster_scale,
            self.annotations,
            self.layers.generation,
        )
    }

    /// Everything a worker needs to rasterize `page_index`, or `None` if there
    /// is no such page.
    ///
    /// The one constructor for a [`RenderRequest`], so the current page and a
    /// strip page cannot be rendered with different options. It exists here,
    /// on the document, rather than in [`crate::render::settle`] because the
    /// annotation stance and the layer override are **private** fields of this
    /// type — and they should stay private: they are changed through
    /// [`Self::set_annotations_visible`] and [`Self::set_hidden_layers`],
    /// which are the methods that keep the staleness keys moving.
    pub(crate) fn render_request_for(
        &self,
        page_index: usize,
        raster_scale: f32,
    ) -> Option<RenderRequest> {
        let page = self.pages.get(page_index)?;
        Some(RenderRequest {
            // The `Arc` is handed over rather than a `DocumentView`, which is
            // what lets the borrow stay local to the worker thread.
            session: Arc::clone(&self.session),
            page: page.clone(),
            page_index,
            raster_scale,
            annotations: self.annotations,
            layers: self.layer_visibility(),
            layers_generation: self.layers.generation,
        })
    }

    /// **Where every page this view is showing sits**, in one coordinate
    /// space.
    ///
    /// Built from the page vector and the view state, so it cannot disagree
    /// with either. The convenience over calling
    /// [`crate::viewer::strip::Strip::new`] at each site is not brevity: it is
    /// that the three arguments after `pages` are all view state, and a call
    /// site that passed its own idea of the display mode or the zoom would be
    /// laying out a strip the rest of the frame does not agree with.
    #[must_use]
    pub fn strip(&self) -> crate::viewer::strip::Strip {
        crate::viewer::strip::Strip::new(
            &self.pages,
            self.view.display,
            self.view.page_index,
            self.view.zoom,
        )
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
    pub(crate) fn trace_object_count(&mut self) {
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
        // ★ …and the search results, for a stronger version of the same
        // reason.
        //
        // A hit carries a page index and a page-space rectangle, both of which
        // are positions in ONE file. Carrying them into another is not
        // staleness — a freshly opened document's `edit_epoch` is 0, so the
        // epoch test that catches an edit would happily declare them current —
        // it is nonsense, and it would put highlights on whatever happens to
        // be at those coordinates in the new file. The query and the operator's
        // options survive; see `crate::find::FindState::forget_document`.
        self.find.forget_document();

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

        // ★ **The page-display mode this document opens in.**
        //
        // Two sources, in this precedence, and the order is the operator's
        // requirement of 2026-08-12 rather than a convenience:
        //
        // 1. **what this document was last shown in**, from
        //    `viewer::remembered` — *"so a sheet set does not inherit a
        //    report's setting"*;
        // 2. failing that, **the ribbon mode's default**, from
        //    `PageDisplay::default_for_mode` — which is where
        //    `MODES_AND_PANELS.md`'s "Read defaults to continuous scroll;
        //    Review and Edit default to single page" lives.
        //
        // The two are genuinely different questions and the `Option` between
        // them carries the difference: `None` from the store means "nobody has
        // chosen for this document", which in Read mode must become
        // continuous. A store that returned `Single` for an unknown document
        // would silently invert the operator decision of 2026-08-13, and it is
        // exactly the collapse `remembered::recall`'s own docs refuse.
        //
        // Placed here, in the one function that opens documents, for the same
        // reason the recent-list call is: `argv` reaches this without an
        // action, so a caller-side version would miss the first document of
        // every session.
        //
        // The ribbon mode is read out first, as an owned `String`, so the
        // `&mut self.status` borrow below does not have to be interleaved with
        // a read of a sibling field inside a trace closure.
        let ribbon_mode = self.ribbon.mode().unwrap_or_default().to_owned();
        if let Status::Open(doc) = &mut self.status {
            let remembered = viewer::remembered::recall(&doc.path);
            let display =
                remembered.unwrap_or_else(|| viewer::PageDisplay::default_for_mode(&ribbon_mode));
            doc.view.display = display;
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "page-display mode={} source={} ribbon-mode={ribbon_mode}",
                    display.id(),
                    if remembered.is_some() {
                        "document"
                    } else {
                        "mode-default"
                    },
                )
            });
        }

        // Forget every de-duplicated trace slot, so this document gets its
        // own canvas line and its own region declarations rather than
        // inheriting the previous document's because the numbers happened to
        // match. §4.3 requirement 1 is "at least once per document open", and
        // a consumer is entitled to read that as a line about *this*
        // document. (Written before there was an Open command, when this
        // fired once per process, precisely because the SECOND open is the one
        // that would silently break it. There is an Open command now — this
        // function is reached from `argv`, from `file.open`'s picker and from
        // the Recent menu — so the second open happens routinely and the gate
        // reset is load-bearing rather than anticipatory.)
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
    /// - **The search results**, through
    ///   [`crate::find::FindState::forget_document`], for a stronger version
    ///   of that argument: a hit's page index and its page-space rectangle are
    ///   positions in one file, and the epoch test that catches an *edit*
    ///   cannot catch a *different document* — a freshly opened one's
    ///   `edit_epoch` is 0, so stale hits would read as current. The query and
    ///   the search options survive, because those describe the operator
    ///   rather than the document.
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
        // The hit list describes a document that is no longer open. See
        // `open_path` for the argument; the two sites are deliberately
        // symmetric.
        self.find.forget_document();
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
/// At module level rather than inside `mod tests`, and `pub(crate)`, because
/// three other modules' tests need the identical starting point:
/// [`crate::app::cache`]'s assert against caches whose fields are declared on
/// [`OpenDoc`], `crate::app::status`'s drive the bar over a real document, and
/// `crate::find`'s run a real search and a real reveal against real page
/// geometry. A second fixture opener would be a second way to assemble an
/// `OpenDoc` — exactly what [`OpenDoc::new`]'s own docs argue against — so the
/// visibility widens rather than the function being copied.
#[cfg(test)]
pub(crate) fn open_fixture(rel: &str) -> OpenDoc {
    let path = crate::panels::objects::test_support::engine_fixture(rel);
    let doc = Document::load(&path).expect("the fixture loads");
    let pages = pdfce_core::page_tree::pages(&doc).expect("a page tree");
    OpenDoc::new(path, EditSession::new(doc), pages)
}

/// A four-page document, three objects on every page.
#[cfg(test)]
pub(crate) const FOUR_PAGES: &str = "pageops/four-pages.pdf";
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

    // =======================================================================
    // Phase 4 — which arrangement a document opens in
    // =======================================================================

    /// ★ **Read mode opens a document continuous; every other mode opens it
    /// single page.**
    ///
    /// `MODES_AND_PANELS.md`'s table and the operator decision of 2026-08-13,
    /// asserted through the **open path** rather than through
    /// `PageDisplay::default_for_mode` — which is already tested in its own
    /// module. What this adds is that `open_path` actually consults it: the
    /// rule existing and the rule being applied are two different facts, and
    /// the second is the one an operator experiences.
    ///
    /// Driven with no remembered choice for the fixture (nothing has ever set
    /// one for a path under the engine fixtures directory), so what is measured
    /// is the mode default and not a leftover.
    #[test]
    fn read_mode_opens_a_document_continuous_and_the_others_paged() {
        for (mode, expected) in [
            ("read", viewer::PageDisplay::Continuous),
            ("review", viewer::PageDisplay::Single),
            ("edit", viewer::PageDisplay::Single),
        ] {
            let mut app = PdfceApp::new();
            app.ribbon.set_mode(mode.to_owned());
            app.open_path(engine_fixture(FOUR_PAGES));
            let Status::Open(doc) = &app.status else {
                panic!("the fixture opens");
            };
            assert_eq!(
                doc.view.display, expected,
                "{mode} mode opened the document in {:?}",
                doc.view.display
            );
        }
    }

    /// A freshly opened document is not mistaken for one that has been
    /// navigated to.
    ///
    /// `tracked_page` starting anywhere but at `view.page_index` would make
    /// the canvas scroll a continuous strip on the first frame after an open,
    /// which the operator did not ask for and which would fight a saved scroll
    /// position the moment there is one.
    #[test]
    fn a_freshly_opened_document_is_not_mid_navigation() {
        let doc = open_fixture(FOUR_PAGES);
        assert_eq!(doc.tracked_page, doc.view.page_index);
        assert!(doc.strip_visible.is_empty());
        assert!(doc.strip_rasters.is_empty());
        assert!(doc.render_in_flight.is_none());
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
