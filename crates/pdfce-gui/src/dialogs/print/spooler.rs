//! # `dialogs::print::spooler` — the one file that knows `pdfce-print` exists
//!
//! ## ★ Read this first: `pdfce-print` is NOT a dependency of this crate
//!
//! `crates/pdfce-gui/Cargo.toml` names `pdfce-core` and `pdfce-render` and
//! **not** `pdfce-print`. That is a fact about the manifest, checked rather
//! than assumed, and it is why this module exists at all.
//!
//! Everything else in [`crate::dialogs::print`] — the three tabs, the range
//! parser, the zoom anchor, the preview raster cache, the clip disclosure,
//! the commit button's label — is written against the types *in this file*.
//! Nothing else in the dialog names a printing type, so linking the real
//! crate is a change to **one file** rather than a change to four.
//!
//! ## What this build can actually do, said plainly
//!
//! Nothing spools. [`list_printers`] returns [`Unavailable::NotLinked`], the
//! dialog renders [`crate::text::print::spooler_unavailable`], and the commit
//! button **is not drawn at all** — absent rather than greyed, because
//! greying is for *temporarily* unavailable and no setting in this dialog
//! would ever make this build reach a spooler (`PROJECT_PLAN.md` §3, the
//! no-placeholders invariant).
//!
//! That is a truthful report, not a stub pretending to be a feature. It is
//! also deliberately **not** the same sentence as "no printers were found":
//! `pdfce-print` itself refuses to collapse those two facts — non-Windows
//! `list_printers` returns `Err(Unsupported)` rather than an empty `Vec`,
//! because *"reporting the same value for 'this platform cannot enumerate
//! printers at all' would collapse two different facts into one and send a
//! caller looking for hardware"* (`lib.rs:1859-1866`). The distinction
//! survives the port; see [`crate::text::print`]'s header for the three
//! sentences it feeds.
//!
//! ## ★ The exact change that makes this build print
//!
//! Two edits, in this order. They are written out in full because the next
//! hand should not have to re-derive them from the type definitions below.
//!
//! **1. `crates/pdfce-gui/Cargo.toml`, in `[dependencies]`, beside the two
//! existing path dependencies:**
//!
//! ```toml
//! pdfce-print = { path = "../../../pdfce/crates/pdfce-print" }
//! ```
//!
//! No `default-features = false` and no `[features]` forwarding: unlike
//! `pdfce-core` and `pdfce-render`, `pdfce-print` declares no features, so
//! there is no strippable capability to forward and nothing for the JPX
//! lesson in that manifest's header to bite on. It is a **path** dependency
//! for the same reason the other two are — this crate builds against the
//! live engine, so divergence surfaces at compile time rather than at
//! fold-in.
//!
//! **2. This file, and only this file.** Each of the four functions below
//! loses its refusal and gains the call named in its own doc comment. The
//! type definitions stay: they are the dialog's *view* of a planned job, and
//! keeping them means the widgets bind to types this crate owns rather than
//! to a dependency's, which is what confines the change here.
//!
//! ## Why mirror the types rather than re-export them
//!
//! Two reasons, and the second is the one that matters.
//!
//! 1. The dialog needs a handful of values (a placement, a sheet size, a
//!    resolution verdict), not the whole crate. Mirroring the values it
//!    reads makes the seam small enough to hold in the head.
//! 2. **No arithmetic is mirrored.** There is no `place_page` here, no
//!    `sequence()`, no `job_resolution`, no `plan_job` — those are
//!    `pdfce-print`'s, they are tested there, and a second copy would be the
//!    failure this project already names about range parsers: *"two range
//!    parsers would eventually disagree about something like `5,1-2` … and an
//!    operator moving between the GUI and a script would have no way to know
//!    which one they were talking to."* The same is true, with paper at
//!    stake, of two placement calculations. So [`plan`] is a **hole**, not an
//!    implementation: it either calls the engine or it refuses.
//!
//! ## What is deliberately NOT here
//!
//! **Imposition — n-up, booklet, poster.** `FEATURES.md` records it as
//! `core — · cli [x] · gui [ ]`, and the roadmap names the prerequisite:
//! *"needs the sheet composition extracted into `pdfce-print` so both shells
//! share one implementation."* Until that lands, an imposition control in
//! this dialog would be an affordance for something that cannot happen, which
//! is precisely what the no-placeholders rule forbids. When it does land it
//! is **one new tab**, not a change to the three that exist: n-up, booklet
//! and poster remap the *job* rather than scale a page, and
//! `docs/core-api/03` §6.4 records that the mutual-exclusion guard between
//! them is **CLI-local** — *"`pdfce-print` will not stop you. A new GUI shell
//! must re-implement this guard."* That guard is the first thing that tab
//! owes.

use std::fmt;

// ---------------------------------------------------------------------------
// Failure
// ---------------------------------------------------------------------------

/// Why the print system could not be reached.
///
/// # One variant today, and that is honest rather than lazy
///
/// A build that cannot link `pdfce-print` has exactly one thing to say, and
/// inventing variants for conditions this build cannot detect would be an
/// enum full of states nothing can produce — the same "no placeholders" rule
/// [`crate::app::actions::Action`]'s own docs state for *its* variants.
///
/// When the manifest line lands this becomes three, mapping one-to-one onto
/// the three sentences in [`crate::text::print`]'s header:
///
/// | variant | from | sentence |
/// |---|---|---|
/// | `Spooler(PrintError)` | `list_printers` / `device_features` returning `Err` | [`crate::text::print::spooler_unavailable`] |
/// | `Device(PrintError)` | `printer_caps` returning `Err` for one device | [`crate::text::print::device_unavailable`] |
/// | `NotLinked` | *deleted* | — |
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Unavailable {
    /// `pdfce-print` is not a dependency of this crate. See the module docs
    /// for the one manifest line that removes this variant.
    NotLinked,
}

impl fmt::Display for Unavailable {
    /// **Diagnostic text, not operator copy.**
    ///
    /// It reaches a `PDFCE_DIAG` trace line and, once the spool path is
    /// reachable, [`crate::text::print::failed`]'s `detail` argument — which
    /// is the same passing-through of a structured engine error that
    /// [`crate::text::canvas_render_failed`] does, and for the same reason:
    /// the engine's own sentence is the specific half.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotLinked => f.write_str("pdfce-print is not linked into this build"),
        }
    }
}

// ---------------------------------------------------------------------------
// Job description — what the operator's answers become
// ---------------------------------------------------------------------------

/// How a page is sized onto the sheet.
///
/// **Four modes, not three**, and the fourth is not a rounding error:
/// `pdfce-print` keeps `Fit` and `ShrinkOversized` apart because collapsing
/// them — *"the natural simplification"* — *"silently blows a business card
/// up to A4"* (`lib.rs:490-494`). Fit scales in both directions; Shrink only
/// ever reduces.
///
/// Maps to `pdfce_print::ScaleMode`, variant for variant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ScaleMode {
    /// Scale to fill the printable area, up or down, preserving aspect.
    Fit,
    /// One PDF point to 1/72 inch of paper, whatever that costs.
    ActualSize,
    /// Like [`Self::ActualSize`], except an oversized page is reduced.
    ShrinkOversized,
    /// An explicit multiplier, where `1.0` is actual size.
    Custom(f64),
}

/// Odd/even filtering, applied **over** a page range rather than instead of
/// one — "pages 1-10, even only" is a thing operators ask for.
///
/// Maps to `pdfce_print::PageSubset`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum PageSubset {
    /// No filtering.
    #[default]
    All,
    /// Document pages 1, 3, 5 … — **the numbers printed on the paper**.
    Odd,
    /// Document pages 2, 4, 6 ….
    Even,
}

/// Copy ordering. Maps to `pdfce_print::Collate`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Collate {
    /// The whole set, then the whole set again.
    #[default]
    Collated,
    /// Every copy of page 1, then every copy of page 2.
    Uncollated,
}

/// Which way up the sheet is fed. Maps to `pdfce_print::Orientation`.
///
/// `Auto` is resolved **per page** from the page's own aspect, which is what
/// keeps a document mixing portrait text with a landscape drawing upright
/// throughout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Orientation {
    /// Choose from each page's own shape.
    #[default]
    Auto,
    /// Force portrait.
    Portrait,
    /// Force landscape.
    Landscape,
}

/// Two-sided printing. Maps to `pdfce_print::Duplex`.
///
/// **Driver-gated, never simulated.** pdfce will not fake duplex by
/// reordering pages and asking the operator to reinsert the stack: *"that is
/// a workflow with a documented mis-assembly failure mode, and offering it as
/// though it were duplex would be claiming a capability the hardware does not
/// have."* [`DeviceFeatures::supports_duplex`] is what the dialog consults
/// before drawing the control at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Duplex {
    /// One side only — also what a device that cannot duplex does.
    #[default]
    Simplex,
    /// Two-sided, flipped on the long edge: the usual book binding.
    LongEdge,
    /// Two-sided, flipped on the short edge: notepad binding.
    ShortEdge,
}

/// The arithmetic half of a job: which pages, at what size, in what order.
///
/// Maps to `pdfce_print::JobSpec`, field for field. **Kept separate from
/// [`DeviceSettings`]** for the engine's own reason: everything here is
/// arithmetic pdfce performs and can be exact about, and everything there is
/// a *request to the driver* which the driver may quietly decline. Presenting
/// both as though pdfce controlled them is what makes a job silently come out
/// single-sided with nothing to say so.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct JobSpec {
    /// Zero-based page indices, in document order.
    pub(crate) pages: Vec<usize>,
    /// How each page is sized onto the sheet.
    pub(crate) mode: ScaleMode,
    /// Upper bound on rendering resolution, in DPI. **A memory bound, not a
    /// quality preference** — and one pdfce chose rather than the operator,
    /// which is why [`JobResolution::capped`] must be disclosed.
    pub(crate) max_dpi: u32,
    /// Odd/even filtering, applied over [`Self::pages`].
    pub(crate) subset: PageSubset,
    /// Send the sequence back to front.
    pub(crate) reverse: bool,
    /// How many copies.
    pub(crate) copies: u16,
    /// Copy ordering.
    pub(crate) collate: Collate,
}

/// The driver half of a job: what pdfce asks the device to do.
///
/// Maps to `pdfce_print::DeviceSettings`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct DeviceSettings {
    /// Sheet orientation.
    pub(crate) orientation: Orientation,
    /// Two-sided printing, if the device supports it.
    pub(crate) duplex: Duplex,
    /// Ask the driver to pick the input tray from each page's size.
    pub(crate) pick_tray_by_page_size: bool,
}

// ---------------------------------------------------------------------------
// Device description — what comes back
// ---------------------------------------------------------------------------

/// One printer the system knows about. Maps to `pdfce_print::Printer`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Printer {
    /// The name the spooler reports, and the one a job is addressed to.
    pub(crate) name: String,
    /// The driver's name.
    ///
    /// Carried because two printers can share a human-readable name closely
    /// enough that an operator cannot tell them apart, and the driver usually
    /// distinguishes them. Traced rather than shown today: the selector is a
    /// combo of names, and a two-line row is a change to make on evidence
    /// that the ambiguity actually bites.
    pub(crate) driver: String,
    /// The port, for the same reason as [`Self::driver`].
    pub(crate) port: String,
    /// Whether this is the system default — the dialog's initial selection.
    pub(crate) is_default: bool,
}

/// What a device says it can do, beyond geometry.
///
/// Maps to `pdfce_print::DeviceFeatures`. Read **once**, when the dialog
/// opens: asking a driver this question sixty times a second while a dialog
/// sits open would be rude to a service other applications share.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct DeviceFeatures {
    /// The driver reports duplex support. The dialog draws no duplex control
    /// without it (R83).
    pub(crate) supports_duplex: bool,
    /// How many copies the driver can produce itself.
    ///
    /// **Reported, not used.** pdfce sends its own sequence today, so this is
    /// carried to the trace so a later decision about hardware collation can
    /// be made on evidence rather than on assumption.
    pub(crate) max_copies: u16,
}

/// The sheet, the printable area within it, and the resolution — **already
/// turned for this job's orientation**.
///
/// Maps to `pdfce_print::DeviceGeometry`.
///
/// # ★ Turned, and that word is the whole defect this type prevents
///
/// `printer_caps` reports the device's *default* `DEVMODE`. On a
/// portrait-default printer that is a portrait printable area — so a
/// landscape job planned against it under-scales every page to about 77 % of
/// correct size, leaves a wide empty margin, and **reports no clip**, so
/// nothing says it happened. The engine removed the `From` impl that made
/// that mistake reachable, *"because a wrong answer that is one `.into()`
/// away will be reached again"*, leaving `DeviceGeometry::from_caps` as the
/// only route — and it cannot be called without stating the orientation and
/// the first page.
///
/// The port honours that by not exposing raw capabilities at all: [`plan`]
/// takes the orientation and the page sizes and hands back a geometry that
/// has already been turned, so the picture the preview draws and the paper
/// the job lands on are the same claim.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct DeviceGeometry {
    /// Resolution in dots per inch, horizontal and vertical.
    ///
    /// A pair rather than one number because asymmetric devices are real
    /// (600×300 on some plotters), and the engine renders at the **smaller**
    /// axis so the driver is not left to resample.
    pub(crate) dpi: (u32, u32),
    /// The printable area in points — smaller than the sheet by the
    /// unprintable margins the driver reports.
    pub(crate) printable_pt: (f64, f64),
    /// The full sheet in points.
    pub(crate) physical_pt: (f64, f64),
    /// Where the printable area starts relative to the sheet corner, in
    /// points: the top-left unprintable margin.
    pub(crate) offset_pt: (f64, f64),
}

/// Where and how big one page lands on the sheet.
///
/// Maps to `pdfce_print::Placement`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Placement {
    /// Multiplier from PDF points to paper points.
    pub(crate) scale: f64,
    /// Offset within the *printable area*, in paper points.
    pub(crate) offset_x_pt: f64,
    /// Vertical offset, same units.
    pub(crate) offset_y_pt: f64,
    /// **The scaled page does not fit and will lose content off the edges.**
    ///
    /// Acrobat clips silently here. pdfce reports it — the operator's
    /// standing ruling that parity is a floor — and this flag is the whole
    /// reason the preview hatches, the caption counts, and the commit
    /// button's own label carries the number.
    pub(crate) clipped: bool,
}

/// Where one page lands, and how big to render it.
///
/// Maps to `pdfce_print::PagePlan`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PagePlan {
    /// The **document** page this describes, zero-based.
    ///
    /// ★ Not a position in the plan list. The plan list is the job's
    /// *sequence* — subset-filtered, possibly reversed, possibly repeated for
    /// copies — so the two coincide only for a whole-document forward job.
    /// Indexing page sizes by a plan's position rather than by this field is
    /// a live defect the salvaged preview carries a comment about; see
    /// [`super::preview`].
    pub(crate) index: usize,
    /// Placement on the sheet.
    pub(crate) placement: Placement,
    /// The scale to rasterise at, in device pixels per PDF point.
    ///
    /// **Already carries the print scale** (`dpi / 72 × placement.scale`), so
    /// the pixels handed to the spooler are the size they will occupy on
    /// paper and the blit is a 1:1 copy. Rendering at device resolution and
    /// letting the driver stretch resamples twice, and on a CAD drawing —
    /// whose value is thin lines — that is the difference an operator notices
    /// first.
    pub(crate) render_scale: f64,
}

/// The resolution a job will render at, and whether pdfce's cap bound.
///
/// Maps to `pdfce_print::JobResolution`, plus one value flattened: the engine
/// exposes `uncapped_page_mb()` as a method, and it is carried here as a
/// field so no formula of the engine's is restated in this crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct JobResolution {
    /// The DPI actually used.
    pub(crate) dpi: u32,
    /// The device's own resolution, before the cap.
    pub(crate) device_dpi: u32,
    /// Whether [`JobSpec::max_dpi`] reduced it.
    ///
    /// **The case that must be disclosed**: pdfce chose a number the operator
    /// did not, by pdfce's own memory judgement.
    pub(crate) capped: bool,
    /// Roughly what one page at the *device's* resolution would cost, in
    /// megabytes — the number that justifies the cap, from
    /// `JobResolution::uncapped_page_mb`.
    pub(crate) uncapped_page_mb: u64,
}

/// A job, planned: the turned geometry, the resolution verdict, and one entry
/// per sheet in the order it will be sent.
///
/// # Why one struct rather than three calls
///
/// The three come from the same three engine calls, in a fixed order, against
/// the same inputs — and getting the order wrong is exactly the orientation
/// defect described on [`DeviceGeometry`]. Returning them together means the
/// dialog cannot plan against one geometry and preview against another.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Job {
    /// The sheet this job was planned against, already turned.
    pub(crate) device: DeviceGeometry,
    /// What resolution it will render at, and whether pdfce capped it.
    pub(crate) resolution: JobResolution,
    /// One entry per sheet, in send order.
    pub(crate) plans: Vec<PagePlan>,
}

impl Job {
    /// How many sheets of this job will lose content off an edge.
    ///
    /// Counted over the **whole job**, not the sheet on screen, because a
    /// multi-page job's clip is usually on a sheet the operator is not
    /// looking at. This one number reaches three surfaces — the preview
    /// caption, the commit button's label, and the trace — and it is computed
    /// in one place so they cannot disagree.
    pub(crate) fn clipped(&self) -> usize {
        self.plans.iter().filter(|p| p.placement.clipped).count()
    }
}

/// One rendered page, ready to blit.
///
/// Maps to `pdfce_print::PageBitmap`. **RGBA8, row-major, top row first** —
/// i.e. `pixmap.data().to_vec()` handed over unchanged, premultiplied, with
/// no conversion in between. The engine is explicit that this is the
/// contract; re-encoding it here would be a second colour convention of
/// exactly the kind [`crate::render::raster`]'s header exists to prevent.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PageBitmap {
    /// Width in device pixels.
    pub(crate) width: u32,
    /// Height in device pixels.
    pub(crate) height: u32,
    /// The pixels, premultiplied RGBA8.
    pub(crate) rgba: Vec<u8>,
    /// Where this page lands on the sheet.
    pub(crate) placement: Placement,
    /// The page's own size in PDF points — what the driver picks paper from.
    pub(crate) page_pt: (f64, f64),
}

/// What a spool attempt did. Maps to `pdfce_print::SpoolReport`.
///
/// **Never constructed in this build**, because [`spool`] cannot succeed
/// here. The `allow` is scoped to this one type and names the condition that
/// removes it, following the precedent `crate::viewer` sets for salvaged
/// items whose first consumer arrives in a later stage. Deleting the type
/// instead would mean the footer had no shape to render a success into, and
/// the day the manifest line lands the success path would be written from
/// scratch rather than reviewed.
#[allow(
    dead_code,
    reason = "constructed by the adapter once pdfce-print is linked; see the module header" // ui-text-exempt: lint justification, never displayed
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpoolReport {
    /// Pages sent.
    pub(crate) pages: usize,
    /// Whether a job was actually started.
    pub(crate) printed: bool,
    /// The device's reported resolution.
    pub(crate) dpi: (i32, i32),
    /// Pages whose placement reported [`Placement::clipped`].
    pub(crate) clipped_pages: usize,
    /// The spooler's job ID, when one was started.
    pub(crate) job_id: Option<u32>,
}

// ---------------------------------------------------------------------------
// The four holes
// ---------------------------------------------------------------------------
//
// Each function below is a hole with a refusal in it. The doc comment on each
// names the exact engine call that fills it. Nothing here computes a
// placement, a sequence or a resolution: see the module docs on why a second
// implementation of any of those is worse than no implementation at all.

/// Enumerate the system's printers.
///
/// Fill with `pdfce_print::list_printers()`, mapping each `Printer` field for
/// field. Called **once**, when the dialog opens — enumerating printers
/// touches the spooler, and doing it per frame while a dialog sits open would
/// be rude to a service other applications share.
///
/// # Errors
///
/// [`Unavailable::NotLinked`] in this build, always. Once linked: whatever
/// `pdfce-print` reports, which on a non-Windows target is `Unsupported` —
/// **not** an empty `Vec`, and the dialog says something different for each.
pub(crate) fn list_printers() -> Result<Vec<Printer>, Unavailable> {
    Err(Unavailable::NotLinked)
}

/// Read one device's non-geometric capabilities.
///
/// Fill with `pdfce_print::device_features(printer)`. Consulted **before**
/// offering the duplex control at all (R83), never after.
///
/// # Errors
///
/// [`Unavailable::NotLinked`] in this build, always.
pub(crate) fn device_features(printer: &str) -> Result<DeviceFeatures, Unavailable> {
    let _ = printer;
    Err(Unavailable::NotLinked)
}

/// Plan the whole job: turn the geometry, resolve the resolution, place every
/// page.
///
/// # The three engine calls this stands in for, and their order
///
/// ```text
/// let caps   = pdfce_print::printer_caps(printer)?;
/// let device = pdfce_print::DeviceGeometry::from_caps(       // 1. TURN FIRST
///     &caps, settings.orientation, spec.first_page_pt(page_sizes));
/// let res    = pdfce_print::job_resolution(&device, &spec);  // 2.
/// let plans  = pdfce_print::plan_job(&device, page_sizes, &spec);   // 3.
/// ```
///
/// The order is not stylistic. Steps 2 and 3 both take `&device`, so a
/// geometry turned *after* them would leave the dialog previewing a sheet the
/// job was not planned for — the 77 %-scale defect described on
/// [`DeviceGeometry`], reintroduced by sequencing rather than by a `From`
/// impl.
///
/// `spec.first_page_pt(page_sizes)` is the **first page the job sends**, not
/// `page_sizes[0]`: the sequence may be subset-filtered or reversed, and the
/// `DEVMODE` and the geometry rotation must resolve `Auto` from the same
/// page. Taking an index of our own here is how the two come to disagree.
///
/// # Errors
///
/// [`Unavailable::NotLinked`] in this build, always. Once linked, an `Err`
/// means this *particular* device would not describe itself, which is a
/// different sentence from having no printers at all.
pub(crate) fn plan(
    printer: &str,
    settings: DeviceSettings,
    page_sizes: &[(f64, f64)],
    spec: &JobSpec,
) -> Result<Job, Unavailable> {
    // Traced in full rather than discarded, for the same two reasons as
    // [`spool`]: a refusal is what a harness needs to see, and reading every
    // operand is what keeps the shape of [`JobSpec`] honest — a field nothing
    // ever reads is a field that can quietly acquire the wrong units or stop
    // being filled at all.
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "print-plan-refused printer={printer} pages={:?} mode={:?} max_dpi={} \
             subset={:?} reverse={} copies={} collate={:?} orientation={:?} \
             duplex={:?} tray={} sizes={} reason={}",
            spec.pages,
            spec.mode,
            spec.max_dpi,
            spec.subset,
            spec.reverse,
            spec.copies,
            spec.collate,
            settings.orientation,
            settings.duplex,
            settings.pick_tray_by_page_size,
            page_sizes.len(),
            Unavailable::NotLinked,
        )
    });
    Err(Unavailable::NotLinked)
}

/// Hand the rendered sheets to the spooler.
///
/// Fill with
/// `pdfce_print::spool(printer, &bitmaps, DryRun::No, None, settings, first_page_pt)`.
///
/// # ★ This is the one call in the application that consumes paper
///
/// `pdfce-print`'s own header: *"Printing consumes paper, occupies a device
/// other people may share, and cannot be undone. Nothing in this crate starts
/// a job as a side effect of anything else: `spool` is the only function that
/// reaches `StartDoc`, and it is reached only from a control an operator
/// deliberately clicked."* The shell's half of that contract is that this
/// function is reached from **one** place — the commit button — and from no
/// keyboard chord, no dispatch arm and no frame-loop condition.
///
/// `first_page_pt` must come from `bitmaps.first()`, never from the
/// document's page 0: a reversed or range-filtered job sends a different page
/// first, and the driver picks its paper from whichever one it is handed.
///
/// # Errors
///
/// [`Unavailable::NotLinked`] in this build, always. Once linked, whatever
/// the spooler reports — passed through to the operator verbatim by
/// [`crate::text::print::failed`], because a structured spooler error is the
/// specific half of that sentence.
pub(crate) fn spool(
    printer: &str,
    bitmaps: &[PageBitmap],
    settings: DeviceSettings,
    first_page_pt: (f64, f64),
) -> Result<SpoolReport, Unavailable> {
    // Traced in full rather than discarded. A refused spool is exactly the
    // event a harness needs to see, and reading every operand here is also
    // what keeps the shape of `PageBitmap` honest: a field nothing ever reads
    // is a field that can quietly acquire the wrong units.
    crate::diag::trace(|| {
        let bytes: usize = bitmaps.iter().map(|b| b.rgba.len()).sum();
        let first = bitmaps.first();
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "print-spool-refused printer={printer} sheets={} px={:?} bytes={bytes} \
             first_page_pt={first_page_pt:?} placement={:?} page_pt={:?} \
             orientation={:?} duplex={:?} tray={} reason={}",
            bitmaps.len(),
            first.map(|b| (b.width, b.height)),
            first.map(|b| b.placement),
            first.map(|b| b.page_pt),
            settings.orientation,
            settings.duplex,
            settings.pick_tray_by_page_size,
            Unavailable::NotLinked,
        )
    });
    Err(Unavailable::NotLinked)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★ Every hole refuses, and none of them invents an answer.
    ///
    /// The test that would fail if somebody "helpfully" filled [`plan`] with a
    /// local placement calculation to make the preview draw something. That
    /// would be the worse outcome by a distance: a preview showing a
    /// confidently wrong sheet is exactly what this dialog exists to prevent,
    /// and it would be indistinguishable from a correct one until the paper
    /// came out.
    #[test]
    fn every_hole_refuses_rather_than_guessing() {
        assert_eq!(list_printers(), Err(Unavailable::NotLinked));
        assert_eq!(device_features("Any"), Err(Unavailable::NotLinked));
        let spec = JobSpec {
            pages: vec![0],
            mode: ScaleMode::Fit,
            max_dpi: 300,
            subset: PageSubset::All,
            reverse: false,
            copies: 1,
            collate: Collate::Collated,
        };
        assert_eq!(
            plan("Any", DeviceSettings::default(), &[(612.0, 792.0)], &spec),
            Err(Unavailable::NotLinked)
        );
        assert_eq!(
            spool("Any", &[], DeviceSettings::default(), (612.0, 792.0)),
            Err(Unavailable::NotLinked)
        );
    }

    /// The clip count is over the whole job, and counts sheets not pages.
    ///
    /// Pinned because three surfaces read it — the preview caption, the
    /// commit button's label and the trace — and the entire point of
    /// computing it once is that the button cannot promise a different number
    /// from the caption above it.
    #[test]
    fn the_clip_count_covers_the_whole_job() {
        let placed = |clipped| Placement {
            scale: 1.0,
            offset_x_pt: 0.0,
            offset_y_pt: 0.0,
            clipped,
        };
        let job = Job {
            device: DeviceGeometry {
                dpi: (600, 600),
                printable_pt: (600.0, 780.0),
                physical_pt: (612.0, 792.0),
                offset_pt: (6.0, 6.0),
            },
            resolution: JobResolution {
                dpi: 300,
                device_dpi: 600,
                capped: true,
                uncapped_page_mb: 139,
            },
            plans: vec![
                PagePlan {
                    index: 4,
                    placement: placed(false),
                    render_scale: 4.0,
                },
                PagePlan {
                    index: 0,
                    placement: placed(true),
                    render_scale: 4.0,
                },
                PagePlan {
                    index: 2,
                    placement: placed(true),
                    render_scale: 4.0,
                },
            ],
        };
        assert_eq!(job.clipped(), 2);
        // And a job with nothing clipped reports zero rather than being
        // treated as "unknown" — the commit button's plain label depends on
        // the difference.
        let clean = Job {
            plans: vec![PagePlan {
                index: 0,
                placement: placed(false),
                render_scale: 1.0,
            }],
            ..job
        };
        assert_eq!(clean.clipped(), 0);
    }
}
