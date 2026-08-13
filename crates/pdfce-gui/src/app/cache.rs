//! # `app::cache` — the two derived values a document is worth keeping, and why they live on it
//!
//! ## What is in here
//!
//! Two caches, and the four [`OpenDoc`] methods that read them:
//!
//! | cache | what it holds | keyed on | read by |
//! |---|---|---|---|
//! | [`PageObjectCache`] | the current page's decomposition | `(page index, edit epoch)` | the Objects panel, the Properties panel, the canvas hit test, the `objects n=` trace |
//! | [`FontCache`] | the document's font inventory | `edit epoch` | the Fonts panel, the Properties panel |
//!
//! ## ★ Why this is a module of its own — the seam, stated
//!
//! `app/state.rs` reached 1,468 lines against the 1,500-line gate (rule R2),
//! and the S4 selection move was about to add to it. A size gate is only
//! useful if the split it forces is a **real** seam rather than an arbitrary
//! cut at line 750, so the question was which of `state.rs`'s subjects is
//! separable without leaving a dangling half-explanation behind.
//!
//! These two are. Everything else in `state.rs` answers *"what is open, and
//! what is the operator looking at?"* — [`crate::app::state::Status`]'s
//! three-way failure distinction, [`OpenDoc`]'s view fields, the raster
//! bookkeeping that keeps the page texture honest. These two answer a
//! different question: *"what expensive thing derived from the document do
//! several surfaces need, and how do we compute it once?"* They share one
//! argument (the cost of `pdfce-core` recomputation), one hazard (staleness
//! against `edit_epoch`), and one structural device (a `Cell` key beside a
//! `RefCell` payload, for the borrow reason below). None of that is shared
//! with anything left behind.
//!
//! The seam is also the one the caches themselves already implied: they were
//! moved off `crate::panels::PanelsState` onto `OpenDoc` earlier in this same
//! stage, and the whole argument for that move — *a cache should be bounded by
//! the lifetime of what it describes* — is a statement about caches as a
//! class, not about any one of them.
//!
//! ## ★ Why interior mutability, and why that is not a smell here
//!
//! A panel body is handed `&OpenDoc`, never `&mut` — that is the
//! actions-not-mutations invariant, and it is not negotiable
//! (`PROJECT_PLAN.md` §3). A lazily-built cache behind a shared reference is
//! precisely what [`RefCell`] is for: the cache is *derived*, so filling it
//! changes nothing an observer could see, and the alternative — building it
//! eagerly on every page change whether or not any surface asked — would cost
//! a decomposition per page step with the Objects panel closed.
//!
//! It applies to **caches only**, which is the other reason they are worth
//! collecting in one file: the exemption has a visible boundary. State that
//! decides what appears on the page (the layer override, the annotation flag,
//! the selection) stays behind `&mut self` over in `state.rs` and reaches it
//! through an [`crate::app::actions::Action`], or *"what can change what is
//! drawn?"* stops having a complete answer.
//!
//! ## ★ Why neither cache can panic on a double borrow
//!
//! The `RefCell` hazard is a `borrow_mut` taken while a `Ref` is still alive.
//! It is unreachable here by a borrow-checker argument rather than by care,
//! and the argument is the same for both caches:
//!
//! 1. The validity key is a [`Cell`], **outside** the `RefCell`, so the
//!    already-built path reads it and takes only a shared `borrow()`.
//! 2. `borrow_mut` is reached only when the key has *moved*.
//! 3. A live `Ref<'a, …>` borrows `&'a OpenDoc`, so while one exists nothing
//!    can take `&mut OpenDoc` — and `view.page_index` and `edit_epoch` change
//!    only through `&mut self`. The key therefore **cannot** move while a
//!    `Ref` is outstanding.
//!
//! Keeping the key in a `Cell` rather than inside the `RefCell` is what makes
//! step 1 true, and is the entire reason each cache is two fields rather than
//! one. [`tests::a_second_reader_shares_the_decomposition_rather_than_rebuilding_it`]
//! holds two `Ref`s at once, so the property is exercised and not merely
//! argued.

use std::cell::{Cell, Ref, RefCell};

use pdfce_core::fontinfo::FontInventory;

use crate::app::state::OpenDoc;
use crate::panels::objects::provider::ObjectModelProvider;

/// The page decomposition, held for as long as the document is open.
///
/// # Why this is a cache at all
///
/// `pdfce_core::vector::decompose_page` resolves every `/Contents` stream,
/// inflates it, concatenates, tokenizes and walks the whole token stream
/// resolving fonts as it goes, and there is **no cache anywhere in
/// `pdfce-core`**. On a CAD sheet that is a frame's worth of work; doing it
/// per frame at 60 Hz is not an option, and doing it *twice* per frame — once
/// for the Objects panel and once for a canvas hit test — is the *"two
/// decompositions quietly diverge"* failure decision 011 names.
///
/// So there is exactly one, and it lives here: on the document, whose
/// lifetime bounds it exactly. See the module docs for the borrow argument
/// the two-field shape exists to satisfy.
///
/// `pub(in crate::app)` rather than private because [`OpenDoc`] declares the
/// field and lives in the sibling module `crate::app::state`. The type is not
/// part of the crate's surface: nothing outside `crate::app` can name it, and
/// every read goes through [`OpenDoc::page_objects`].
#[derive(Default)]
pub(in crate::app) struct PageObjectCache {
    /// The `(page index, edit epoch)` the decomposition below describes, or
    /// `None` before the first attempt.
    ///
    /// Both halves are needed, and for the same reason
    /// `OpenDoc::objects_traced_for` needs both: a decomposition is a property
    /// of **this page** in **this revision**. Paging away and back must
    /// rebuild (different content), and an edit must rebuild (the objects
    /// moved).
    ///
    /// Notice what is *not* in it: any document identity. There is none to
    /// carry, because opening a document constructs a whole new [`OpenDoc`]
    /// and this cache dies with the old one. That is the point of the move —
    /// see [`OpenDoc::page_objects`].
    pub(in crate::app) built_for: Cell<Option<(usize, u64)>>,
    /// The decomposition, or the reason the page would not decode.
    ///
    /// `None` means "not attempted". `Some(Err(_))` means "attempted and
    /// failed", which is a **different state**: the failure is deterministic
    /// (same bytes, same code), so a page whose content will not decode must
    /// not be re-decomposed on every frame. That is the same reasoning the
    /// render-error hold in `PdfceApp::settle_and_rasterize` uses.
    provider: RefCell<Option<Result<ObjectModelProvider, String>>>,
}

/// The document's font inventory, held for as long as the document is open.
///
/// Cached for the same reason [`PageObjectCache`] is, and the sweep is more
/// expensive: `pdfce_core::fontinfo::inventory` **decodes every embedded font
/// program**, because that is where the `OS/2` table lives. On a document
/// carrying a megabyte of CJK outlines that is not a per-frame cost.
///
/// Document-scoped rather than page-scoped — paging does not drop it — but
/// **not** revision-scoped-by-accident: an edit can add or remove a font, so
/// the epoch is the key.
#[derive(Default)]
pub(in crate::app) struct FontCache {
    /// The `edit_epoch` the inventory below describes, or `None` before the
    /// first build. See [`PageObjectCache::built_for`] for the borrow
    /// argument this `Cell` is half of.
    pub(in crate::app) built_for: Cell<Option<u64>>,
    /// The inventory. `pdfce_core::fontinfo::inventory` is **infallible** —
    /// it reports problems in its `diagnostics` rather than in a `Result`
    /// (core API trap T-9.8) — so there is no error arm here, and an empty
    /// inventory does not mean a clean document.
    inventory: RefCell<Option<FontInventory>>,
}

impl OpenDoc {
    /// The current page's decomposition, building it on first use.
    ///
    /// # ★ This is THE decomposition — there is deliberately only one
    ///
    /// The Objects panel lists it, the Properties panel describes a row of
    /// it, the diagnostic `objects n=` line counts it, and the canvas
    /// hit-tests against it — all from *this* value. A second
    /// `decompose_page` over the same page is the *"two decompositions
    /// quietly diverge"* pattern decision 011 warns about, and
    /// [`ObjectModelProvider::page_objects`]' own docs call this the shared
    /// escape hatch that exists to prevent it.
    ///
    /// **The canvas's second decomposition is gone.** Until this stage's
    /// wiring pass, `canvas::show` built its own `ObjectModelProvider` per
    /// gesture, because the only cache was on the panels and the canvas had no
    /// route to it. That was one extra full decomposition per click and per
    /// marquee release on the same page the Objects panel had already
    /// decomposed. The canvas now calls this method, so *"what did I click?"*
    /// and *"what is in this list?"* are answered from one value by
    /// construction rather than by two code paths that happen to agree.
    ///
    /// # Why it lives on `OpenDoc` and needs no identity key
    ///
    /// It was on `crate::panels::PanelsState` until S4, guarded by a `DocKey`
    /// built partly from the `Arc<EditSession>`'s **address** — because a
    /// cache hanging off the application outlives the document it describes,
    /// so it has to say *which* document that was, and an address is the only
    /// token that was available. `crate::panels`' own header records that key,
    /// its ABA hazard, and why a `Weak` clone would have been a worse fix
    /// than the bug.
    ///
    /// Moving it here dissolves the question rather than answering it, for
    /// the reason already in [`OpenDoc::new`]'s doc comment: *"opening a
    /// document constructs a whole new `OpenDoc`, so a cached texture or a
    /// page index can never refer to a page from a previous file."* A cache
    /// held **inside** that structure inherits the guarantee for free — there
    /// is no "which document is this?" to get wrong, because the answer is
    /// "the one you are holding". So `DocKey` was deleted rather than
    /// repaired, and what remains is `(page, epoch)`: two plain values, no
    /// address, no ABA. The canvas's `DocumentToken` — the same idea, built
    /// the same way, for the selection — was deleted for the same reason in
    /// the same stage, once the selection moved onto `OpenDoc` beside this.
    ///
    /// # Returns
    ///
    /// `None` when the page's content cannot be decoded — the same failure
    /// the renderer would hit. A caller says so in words rather than showing
    /// an empty list, because a failure state indistinguishable from a
    /// success state is the same defect as no message at all. The reason is
    /// kept for the trace channel; see [`Self::page_objects_failure`].
    ///
    /// # Holding the `Ref`
    ///
    /// The return is a [`Ref`] into the cache, so it keeps a shared borrow of
    /// `*self` alive for as long as the caller holds it — which is exactly
    /// what stops a `borrow_mut` racing it (module docs, step 3). A caller
    /// that needs `&mut OpenDoc` afterwards must let it go first; `Ref`
    /// implements `Drop`, so the borrow does **not** end at its last use and
    /// an explicit `drop` is sometimes required. `canvas::interact` does that
    /// and says why.
    #[must_use]
    pub fn page_objects(&self) -> Option<Ref<'_, ObjectModelProvider>> {
        self.ensure_page_objects();
        Ref::filter_map(self.page_objects.provider.borrow(), |slot| {
            slot.as_ref().and_then(|built| built.as_ref().ok())
        })
        .ok()
    }

    /// Why the current page would not decompose, if it would not.
    ///
    /// Separate from [`Self::page_objects`] because the two audiences differ:
    /// a panel shows the operator a sentence from the text catalog, and the
    /// `PDFCE_DIAG` channel wants the engine's own error text. A harness that
    /// learns only *that* a page failed has to work out *why* by hand.
    ///
    /// `pub(in crate::app)` because the one consumer is
    /// `OpenDoc::trace_object_count`, which stayed in `state.rs` with the rest
    /// of the per-frame bookkeeping.
    pub(in crate::app) fn page_objects_failure(&self) -> Option<Ref<'_, String>> {
        self.ensure_page_objects();
        Ref::filter_map(self.page_objects.provider.borrow(), |slot| {
            slot.as_ref().and_then(|built| built.as_ref().err())
        })
        .ok()
    }

    /// Decompose the current page if the cache does not already describe it.
    ///
    /// The key is recorded **before** the work, so a page whose content will
    /// not decode is not re-decomposed on every frame: the failure is
    /// deterministic, and retrying it sixty times a second would peg a core
    /// producing the same error.
    fn ensure_page_objects(&self) {
        let key = (self.view.page_index, self.edit_epoch);
        if self.page_objects.built_for.get() == Some(key) {
            return;
        }
        self.page_objects.built_for.set(Some(key));
        let built = self.current_page().map(|page| {
            ObjectModelProvider::build_or_reason(
                // The SESSION view, never the base document's: the session
                // view is the edited state, which is the state the operator
                // is looking at and the state the canvas is drawing.
                // Decomposing the base revision would list objects the
                // operator has already removed and miss ones they have added
                // (decision 018).
                &self.session.view(),
                page,
                self.view.page_index,
            )
        });
        // A document with no such page is "no decomposition and no failure":
        // there is nothing to report a reason about. `page_objects` and
        // `page_objects_failure` both return `None`, and the caller above
        // already handles an empty document.
        *self.page_objects.provider.borrow_mut() = built;
    }

    /// The document's font inventory, building it on first use.
    ///
    /// Moved here from `crate::panels::PanelsState` at S4 for exactly the
    /// reason [`Self::page_objects`] was, and it is the cheaper half of the
    /// argument to state: the inventory decodes every embedded font program,
    /// and it is read by two panels (Fonts lists it; Properties joins one
    /// object's `/BaseFont` against it). Two inventories over one document
    /// would be two sweeps and two chances to disagree.
    ///
    /// `pdfce_core::fontinfo::inventory` is **infallible** — it reports
    /// problems in its `diagnostics` rather than in a `Result` (core API trap
    /// T-9.8) — so there is no error path here, and an empty inventory does
    /// not mean a clean document. The Fonts panel reads the diagnostics.
    #[must_use]
    pub fn font_inventory(&self) -> Ref<'_, FontInventory> {
        if self.fonts.built_for.get() != Some(self.edit_epoch) {
            self.fonts.built_for.set(Some(self.edit_epoch));
            *self.fonts.inventory.borrow_mut() =
                Some(pdfce_core::fontinfo::inventory(&self.session.view()));
        }
        Ref::map(self.fonts.inventory.borrow(), |slot| {
            // `inventory` is infallible and the block above has just filled
            // the slot for this epoch, so `None` is not a reachable state.
            slot.as_ref().expect("just built for this epoch") // ui-text-exempt: panic message, never displayed
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::app::state::{FOUR_PAGES, PAINTED_LAYERS, open_fixture};

    // =======================================================================
    // The cache move — what replaced `panels::DocKey`
    // =======================================================================

    /// **★ The decomposition cache carries NO document identity, and does
    /// not need one.**
    ///
    /// The `DocKey` deletion, asserted rather than argued. That key existed
    /// because the cache hung off the *application* and outlived the document
    /// it described, so it had to say **which** document — and the only token
    /// available was an `Arc` address, which is not an identity (see
    /// `OpenDoc::page_objects`).
    ///
    /// This replaces one document with another **in the same binding**, the
    /// sequence that would exercise an address reuse. There is nothing to get
    /// wrong: the second document's cache is a field of the second document.
    /// The remaining key is `(page, epoch)`, and it is asserted here so that
    /// putting an address, a pointer or a `Weak` back into it is a test
    /// failure rather than a review finding.
    #[test]
    fn a_documents_decomposition_cannot_outlive_the_document() {
        let mut doc = open_fixture(FOUR_PAGES);
        assert_eq!(doc.page_objects().expect("page 0").page_index(), 0);
        assert_eq!(doc.page_objects.built_for.get(), Some((0, 0)));

        doc = open_fixture(PAINTED_LAYERS);
        assert_eq!(
            doc.page_objects().expect("the layer fixture").page_index(),
            0
        );
        assert_eq!(
            doc.page_objects.built_for.get(),
            Some((0, 0)),
            "a fresh document starts un-built, whatever address it landed on"
        );
        assert_eq!(doc.pages.len(), 1, "and it is this document's page tree");
    }

    /// **A page step rebuilds the decomposition; so does an edit.**
    ///
    /// Both halves of the `(page, epoch)` key, one at a time. Serving page
    /// 0's objects while the operator is on page 1 would make every index in
    /// the Objects panel address the wrong object.
    #[test]
    fn the_decomposition_is_rebuilt_when_the_page_or_the_revision_moves() {
        let mut doc = open_fixture(FOUR_PAGES);
        assert_eq!(doc.page_objects().expect("page 0").page_index(), 0);

        doc.view.page_index = 2;
        assert_eq!(
            doc.page_objects().expect("page 2").page_index(),
            2,
            "a page step must rebuild, or the panel lists another page's objects"
        );

        // An edit renumbers objects without moving page, so the epoch is the
        // other half. `edit_epoch` is the seam every mutating action bumps.
        doc.edit_epoch = 1;
        let _ = doc.page_objects();
        assert_eq!(
            doc.page_objects.built_for.get(),
            Some((2, 1)),
            "an edit must rebuild, or the panel lists the pre-edit object set"
        );
    }

    /// **Asking twice does not decompose twice** — the point of a cache, and
    /// the case that would panic if the validity key lived *inside* the
    /// `RefCell` instead of beside it: the second call would take
    /// `borrow_mut` while the first call's `Ref` was still alive. Holding the
    /// first borrow across the second call is the assertion, not an accident
    /// of how the test is written.
    #[test]
    fn a_second_reader_shares_the_decomposition_rather_than_rebuilding_it() {
        let doc = open_fixture(FOUR_PAGES);
        let first = doc.page_objects().expect("page 0 decomposes");
        let second = doc.page_objects().expect("…and again");
        assert_eq!(first.page_objects().objects.len(), 3);
        assert_eq!(
            second.page_objects().objects.len(),
            first.page_objects().objects.len()
        );
    }

    /// **A page that is not there yields no decomposition and no invented
    /// reason.**
    ///
    /// The attempt is recorded either way, so it is not retried sixty times a
    /// second. But "there is no such page" must not be reported as a decode
    /// failure: the trace channel distinguishes `reason=no-such-page` from
    /// `reason=decompose-failed`, and a consumer is entitled to that.
    #[test]
    fn a_missing_page_yields_no_decomposition_and_no_invented_reason() {
        let mut doc = open_fixture(FOUR_PAGES);
        doc.view.page_index = 99;
        assert!(doc.page_objects().is_none());
        assert!(
            doc.page_objects_failure().is_none(),
            "there is no page, so there is no decode failure to report"
        );
        assert_eq!(
            doc.page_objects.built_for.get(),
            Some((99, 0)),
            "the attempt is still recorded, or it is retried every frame"
        );
    }

    /// **The font inventory survives a page step and is dropped by an edit.**
    ///
    /// It decodes every embedded font program, so rebuilding it per page is a
    /// large cost for a value that cannot have changed — and an edit *can*
    /// add or remove a font, so keeping it across one reports a font list the
    /// document no longer has.
    #[test]
    fn the_font_inventory_is_kept_across_pages_and_dropped_by_an_edit() {
        let mut doc = open_fixture(FOUR_PAGES);
        let _ = doc.font_inventory();
        assert_eq!(doc.fonts.built_for.get(), Some(0));

        doc.view.page_index = 3;
        let _ = doc.font_inventory();
        assert_eq!(
            doc.fonts.built_for.get(),
            Some(0),
            "a page step must NOT drop it — the inventory is document-scoped"
        );

        doc.edit_epoch = 1;
        let _ = doc.font_inventory();
        assert_eq!(doc.fonts.built_for.get(), Some(1), "an edit must drop it");
    }
}
