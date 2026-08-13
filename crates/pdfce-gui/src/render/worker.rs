//! # `render::worker` — rasterization on a background thread
//!
//! **Salvaged from `D:\Dev\pdfce\crates\pdfce-gui\src\render_worker.rs`**
//! (Class A, `SALVAGE.md`: *"Generation counter + between-operator
//! cancellation. **Measured**: six rapid zoom steps start six generations
//! and complete one. Do not touch the design."*). The header and every
//! explanatory comment below are carried across; the measured numbers in
//! them are the evidence that justifies the whole module and must not be
//! lost to a paraphrase.
//!
//! ---
//!
//! One job: keep a slow page from freezing the application. This module
//! owns the worker thread, the channel, the cancellation token and the
//! generation counter that `raster.rs` named when it documented itself
//! as the seam where off-thread rendering would happen.
//!
//! ## Why this exists, and what it is NOT
//!
//! **It does not make anything faster.** A page that took 10 s still
//! takes 10 s. What changes is that the 10 s is spent on a thread the
//! operator is not waiting on, so the window keeps repainting, the
//! zoom keeps responding, and the render can be abandoned.
//!
//! The evidence that justified building it: a real CAD sheet measured
//! **~10 s at 1× and ~58 s at 2×**, rasterized inline on the UI thread.
//! At those numbers the application does not render slowly — it stops
//! answering. `raster.rs` predicted exactly this and deferred the work
//! until "a real corpus produces pages slow enough to drop frames".
//!
//! ## The three things that make it correct
//!
//! **A generation counter.** A worker that finishes after its request
//! was superseded must have its result *discarded*, not painted. Every
//! spawn takes the next generation; a reply whose generation is not the
//! current one is dropped. Without this, releasing a zoom gesture would
//! paint whichever render happened to finish last rather than the one
//! that matches the screen.
//!
//! **Cancellation that stops work.** [`RenderCancel`] is polled between
//! content-stream operators, so a superseded render abandons the page
//! rather than running to completion and having its output thrown away.
//! At 58 s a discarded result still occupies a core and still delays
//! whatever the operator asked for next. Measured: **28.9 ms** from
//! `cancel()` to thread exit mid-render, against **10,367 ms** to let
//! one finish.
//!
//! **A bounded in-frame wait.** See [`RenderWorker::spawn`] — this is
//! what keeps a fast page indistinguishable from the synchronous
//! behaviour it replaces.
//!
//! ## What this module does not decide
//!
//! Whether, and how, the canvas discloses that it is showing a stale
//! picture. That is a shell question and it lives in the shell, not here.
//! This module only reports, via [`RenderWorker::in_flight_since`], how
//! long the current render has been outstanding, so the shell can decide.
//!
//! ---
//!
//! ## Salvage note: the staleness keys, and which one is still deferred
//!
//! The original [`RenderKey`] compared **five** inputs. Three were absent at
//! S0, and their absence was a decision rather than an oversight:
//!
//! | key | what it invalidates | state |
//! |---|---|---|
//! | `annotations` | the annotation-visibility toggle (§12.5 `/AP` `/N`) | **landed, S4** |
//! | `layers_generation` | the optional-content layer overrides (§8.11.4.3) | **landed, S4** |
//! | `font_env_generation` | operator-supplied font folders | still deferred |
//!
//! Each was added to the original because **without it the cached texture
//! does not invalidate and the control silently does nothing** — a real,
//! separately-diagnosed defect in all three cases. That is the failure
//! mode to expect: not a crash, a control that appears inert. So the rule
//! for every one of them is: *the key lands in the same commit as the
//! surface that varies it, never later.* Carrying a key with no surface able
//! to change it would put a constant in the request and an untriggerable
//! branch in the comparison — which is the "no state a surface can reach"
//! invariant broken from the other side.
//!
//! ### Why two landed at S4 and the third did not
//!
//! Both of the two are **inputs an operator-facing control now varies**, and
//! both of those controls are `RIBBON_IA.md`'s rather than this module's
//! invention:
//!
//! - `view.show_annotations` is a View ▸ Display control that is already
//!   drawn, already enabled whenever a document has pages, and — until this
//!   key existed — could not have changed a pixel if it had been wired up.
//! - The Layers panel was built **without its visibility checkbox
//!   specifically because this key did not carry `layers_generation`**;
//!   `crate::panels::layers`' own header names that as the false one of its
//!   three preconditions.
//!
//! `font_env_generation` has no such control: nothing in this build lets an
//! operator name a font folder, so the bundled [`pdfce_render::FontEnvironment`]
//! is the only environment any render can use and a generation counter over
//! it would count to one and stop. It lands with the font-folder surface,
//! under the same rule.
//!
//! ### The other half of the invalidation, which is NOT in this module
//!
//! A key on the *request* only stops the worker from de-duplicating two
//! genuinely different renders. It does not, on its own, make the shell ask
//! for the second one: the shell decides "the cached texture is stale" by
//! comparing the texture's own key against the one it wants. That comparison
//! lives in [`crate::app::state::PdfceApp::settle_and_rasterize`], and it
//! reads **this same [`RenderKey`]**, recorded on
//! [`crate::render::raster::PageTexture`] when the pixels were uploaded.
//!
//! That is deliberate and it is the structural half of the fix. Before S4 the
//! shell kept its own two-field comparison (page index, raster scale) beside
//! this type's two-field one, and a third key added to one and not the other
//! would compile, run, and produce exactly the inert control this table
//! warns about. There is now one key type, constructed by one function
//! ([`RenderKey::new`]), and adding a field to it changes both sides at once.
//!
//! The original also carried `cmyk_intent`, `fonts` and
//! `view_magnification` on the request. All three have correct defaults in
//! [`pdfce_render::RenderOptions`] (the operator-ruled `NeutralBlack`
//! intent, the bundled font environment, and `None` = the print-correct
//! `/D`-initial optional-content state), and this build has no surface that
//! varies any of them, so they are left to that default and travel on the
//! request when a settings surface exists to move them.
//!
//! **`view_magnification` deserves one extra sentence**, because it looks
//! adjacent to `layers_generation` and is not. §8.11.4.4's usage
//! applications recompute a layer's state from the zoom, and §8.11.4.5
//! forbids a print or aggregate path from applying them at all (core API
//! trap T-12.8). Leaving it `None` is therefore the *print-correct* answer
//! rather than a gap — and if a viewer ever opts in, it needs no new key of
//! its own, because it is a pure function of `raster_scale`, which is
//! already compared.

use std::sync::Arc;
use std::sync::mpsc::{Receiver, RecvTimeoutError, SyncSender, sync_channel};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use pdfce_core::edit::EditSession;
use pdfce_core::page_tree::Page;
use pdfce_render::cancel::RenderCancel;
use pdfce_render::{Diagnostics, tiny_skia::Pixmap};

/// How long [`RenderWorker::spawn`] will wait, on the UI thread, for a
/// render it just started.
///
/// # Why a blocking wait is the right answer here
///
/// The requirement is that a page rasterizing in milliseconds behaves
/// exactly as it did when rendering was synchronous — no flash, no
/// spinner, no frame of stale content. Handing every render to a worker
/// and collecting it next frame would cost such a page one frame of
/// staleness for no benefit.
///
/// So the spawn waits briefly and collects the result inline when it
/// arrives. One frame at 60 Hz is ~16.7 ms; this is deliberately under
/// that, so even in the worst case the wait cannot itself drop a frame.
/// A page that beats the deadline never touches the asynchronous path
/// at all, and a page that misses it hands control back to the event
/// loop after a delay the operator cannot perceive.
///
/// This is the one place the UI thread blocks on rendering, it is
/// bounded by a constant, and the bound is the whole point.
const IN_FRAME_BUDGET: Duration = Duration::from_millis(12);

/// A finished rasterization, ready for the shell to upload as a texture.
///
/// The worker produces pixels; it does not touch egui. Texture upload
/// needs an `egui::Context` and belongs on the UI thread, which is also
/// what keeps this module free of any GUI type beyond the ones the
/// shell hands back.
pub struct RenderedPixels {
    /// The rasterized page.
    pub pixmap: Pixmap,
    /// Render-time findings for the diagnostics surface.
    ///
    /// Carried even though S0 has no status bar to show them in: they are
    /// the renderer's honesty report (which glyphs were substituted, which
    /// features were skipped), and a render that produced them and threw
    /// them away would have to be re-run to get them back. The surface
    /// that displays them lands at stage S2.
    pub diagnostics: Diagnostics,
    /// Everything this render was *of*, so the shell can key its texture.
    ///
    /// One field rather than a copy of each input: the texture's staleness is
    /// decided by comparing this against the key the shell currently wants,
    /// and a hand-copied subset is how an input silently stops being
    /// compared. See [`RenderKey`].
    pub key: RenderKey,
}

/// What a worker sends back: pixels, a failure, or nothing at all.
enum Outcome {
    Done(Box<RenderedPixels>),
    Failed(String),
    /// The render observed its cancellation token and stopped early.
    /// Distinguished from a failure so the shell does not report a
    /// deliberate abandonment as a render error.
    Cancelled,
}

/// What a render is *of* — the staleness keys, as one comparable value.
///
/// # Why this is load-bearing rather than bookkeeping
///
/// The shell decides "the texture is stale" by comparing these keys
/// against the cached texture, and re-runs that decision every frame.
/// While a background render is in flight the texture has NOT been
/// replaced yet, so the decision keeps coming out the same way. Without
/// a way to recognise that the render already running is *for the very
/// request being asked for again*, each frame would cancel the previous
/// render and start an identical one — and a page slower than one frame
/// would never finish. Not a slow render: a render that can never
/// complete, on a page that used to merely be slow.
///
/// `raster_scale` is compared by bit pattern rather than by `==`
/// because it comes from the same arithmetic each frame; an exact float
/// comparison is right here and a tolerance would be wrong, since any
/// difference at all means the shell wants a different picture.
///
/// # ★ It is also the SHELL's staleness key, and that is the point
///
/// This type is public and is recorded on
/// [`crate::render::raster::PageTexture`] because the same comparison has to
/// be made in two places for a control to work:
///
/// 1. **"Is the render already running the one I want?"** — here, in
///    [`RenderWorker::spawn`], or a slow page never finishes.
/// 2. **"Is the picture on screen still a picture of what I am looking
///    at?"** — in [`crate::app::state::PdfceApp::settle_and_rasterize`], or
///    nothing ever *asks* for the second render.
///
/// Those were two independent field lists until S4, and the failure mode of
/// letting them drift is the one the module docs describe: a control that
/// ticks and changes nothing. One type, one constructor
/// ([`Self::new`]), and a field added to it is compared on both sides
/// or on neither.
///
/// # The two categories of input, and why the split is here
///
/// [`Self::discrete_inputs`] and [`Self::scale_bits`] between them cover
/// every field, and the division is a **policy**, not a convenience:
///
/// - A **discrete** input (page, annotation visibility, layer override) is
///   changed by a command or a click. There is no gesture in flight, no
///   intermediate value on the way to it, and no stale picture worth
///   showing, so it re-rasterizes at once.
/// - The **scale** is changed by a wheel gesture that emits dozens of values
///   on the way to the one that was wanted, so it is debounced
///   (`crate::app::state::ZOOM_SETTLE`) and the existing texture is drawn
///   scaled in the meantime.
///
/// Stating it as two methods rather than as a comment means the shell reads
/// the categories off the key instead of re-deriving them, and a new key
/// added to neither accessor fails
/// [`tests::every_render_input_is_either_discrete_or_the_scale`].
///
/// See the module docs for the one further key this will grow
/// (`font_env_generation`) and the rule that it lands with the surface that
/// varies it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RenderKey {
    /// Which page (0-based).
    page_index: usize,
    /// The raster scale, by bit pattern — see the type docs.
    raster_scale_bits: u32,
    /// Whether annotation appearances (`/AP` `/N`) are painted over the page
    /// content (§12.5). The `bool` **is** the key: there is no generation to
    /// count, because there is exactly one bit of state.
    annotations: bool,
    /// How many times the operator's optional-content override has changed.
    ///
    /// A **counter, not the set**. The set is a `BTreeSet<ObjId>` that the
    /// staleness check would otherwise compare element-by-element on every
    /// frame, for a value that changes only when a control is clicked. A
    /// monotonic counter answers the same question — *is this a different
    /// override from the one that texture was drawn with?* — in one `u64`
    /// comparison, and it answers it correctly for the case a set comparison
    /// would get wrong nowhere and slower everywhere.
    ///
    /// It counts *changes*, so `0` means "no override at all — obey the
    /// document's own default configuration", which
    /// [`pdfce_render::LayerVisibility`]'s replace-not-merge contract makes a
    /// genuinely distinct state from "an override that hides nothing" (core
    /// API trap T-12.9).
    layers_generation: u64,
}

impl RenderKey {
    /// The key for a render of `page_index` at `raster_scale`, with these
    /// annotation and layer settings.
    ///
    /// **The one place a key is computed from parts.** The shell calls it to
    /// ask what it wants; [`Self::of`] calls it to say what a request is.
    /// Two constructors doing the same arithmetic is how the two sides of the
    /// staleness comparison drift.
    #[must_use]
    pub fn new(
        page_index: usize,
        raster_scale: f32,
        annotations: bool,
        layers_generation: u64,
    ) -> Self {
        Self {
            page_index,
            raster_scale_bits: raster_scale.to_bits(),
            annotations,
            layers_generation,
        }
    }

    /// The inputs whose change must re-rasterize **immediately**.
    ///
    /// See the type docs: none of these has a gesture behind it, so waiting
    /// out the zoom debounce would make a click feel unresponsive for no
    /// benefit — and for a page change there is not even a stale picture
    /// worth showing, because it is a picture of a different page.
    #[must_use]
    pub fn discrete_inputs(&self) -> (usize, bool, u64) {
        (self.page_index, self.annotations, self.layers_generation)
    }

    /// The one input that is **debounced** rather than committed at once.
    #[must_use]
    pub fn scale_bits(&self) -> u32 {
        self.raster_scale_bits
    }

    /// The key `request` describes.
    fn of(request: &RenderRequest) -> Self {
        Self::new(
            request.page_index,
            request.raster_scale,
            request.annotations,
            request.layers_generation,
        )
    }
}

/// A render currently running on a worker thread.
struct InFlight {
    rx: Receiver<Outcome>,
    cancel: RenderCancel,
    handle: Option<JoinHandle<()>>,
    key: RenderKey,
    generation: u64,
    started: Instant,
}

/// Owns at most one in-flight rasterization.
///
/// Deliberately single-slot: the canvas shows one page at one scale, so
/// a second concurrent render is always a superseded first one. Keeping
/// a queue would mean deciding which of several stale results to paint,
/// which is a question with no good answer.
#[derive(Default)]
pub struct RenderWorker {
    in_flight: Option<InFlight>,
    next_generation: u64,
}

impl std::fmt::Debug for RenderWorker {
    // Hand-written: `Receiver` and `JoinHandle` are not `Debug`, and the
    // useful state is whether something is running and which request it
    // belongs to — not the channel internals.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderWorker")
            .field(
                "in_flight_generation",
                &self.in_flight.as_ref().map(|f| f.generation),
            )
            .field("next_generation", &self.next_generation)
            .finish()
    }
}

/// Everything a render needs, owned, so it can cross a thread boundary.
///
/// `DocumentView<'a>` borrows its graph, so the worker cannot be handed
/// one — it is handed the `Arc<EditSession>` and calls `view()` on the
/// far side, where the borrow stays local to the closure. That is the
/// whole reason the open document's session is an `Arc`, and the reason
/// `ObjectGraph` had to gain `Send + Sync` in the engine.
pub struct RenderRequest {
    /// The edit session to render. Rendered through `session.view()` **on
    /// the worker**, never `session.document()` — the view composes the
    /// overlay and the staging buffer, so unsaved edits are what gets
    /// drawn. S0 makes no edits, but the rule is structural: the canvas
    /// renders the *edited* state, and a base read here is how every
    /// editing feature becomes invisible at once.
    pub session: Arc<EditSession>,
    /// The page to draw. Cloned out of the page vector by the caller so
    /// the worker owns it.
    pub page: Page,
    /// Which page (0-based) — a staleness key.
    pub page_index: usize,
    /// Device pixels per PDF user-space unit — the operator's zoom already
    /// multiplied by `pixels_per_point` ([`crate::viewer::raster_scale`]).
    /// The second staleness key.
    pub raster_scale: f32,
    /// Whether annotation appearances are painted over the page content
    /// (§12.5, [`pdfce_render::RenderOptions::annotations`]).
    ///
    /// The third staleness key, and the first one this build actually varies.
    /// `true` is what a reader does with a file it was handed; `false`
    /// reproduces the content-only raster, which is what View ▸ Display's
    /// `view.show_annotations` exists to ask for.
    pub annotations: bool,
    /// The operator's optional-content override, or `None` to obey the
    /// document's own default configuration (§8.11.4.3).
    ///
    /// **A complete answer, never a patch.** `pdfce-render` uses this
    /// *instead of* the document's `/D` configuration rather than merging
    /// with it (core API trap T-12.9), so the caller computes the whole
    /// hidden set — starting from
    /// `pdfce_core::annot::optional_content_default_off` — and hands it in.
    /// `None` and `Some(empty)` are therefore different renders: the first
    /// obeys the document, the second shows every layer.
    ///
    /// Not itself a staleness key — [`Self::layers_generation`] is, and its
    /// own docs say why a counter beats comparing the set.
    pub layers: Option<pdfce_render::LayerVisibility>,
    /// How many times the override above has changed — the fourth staleness
    /// key. See [`RenderKey::layers_generation`].
    pub layers_generation: u64,
}

impl RenderWorker {
    /// Start rendering `request`, abandoning whatever was running.
    ///
    /// Returns `Some` when the render finished inside
    /// [`IN_FRAME_BUDGET`] — the fast path, which behaves exactly as
    /// the previous synchronous code did. Returns `None` when it is
    /// still running, in which case the shell should keep drawing the
    /// previous texture and call [`Self::poll`] on later frames.
    ///
    /// Cancels the previous render *before* spawning rather than after:
    /// two rasterizations of a CAD page competing for cores make both
    /// slower, and the old one's output is already known to be unwanted.
    pub fn spawn(&mut self, request: RenderRequest) -> Option<Result<RenderedPixels, String>> {
        let key = RenderKey::of(&request);

        // Already rendering exactly this? Leave it alone. See `RenderKey`
        // — without this the per-frame staleness check would cancel and
        // restart the same render forever, and any page slower than one
        // frame would never appear at all.
        if self.in_flight.as_ref().is_some_and(|f| f.key == key) {
            return None;
        }

        self.cancel_in_flight();

        self.next_generation = self.next_generation.wrapping_add(1);
        let generation = self.next_generation;
        let cancel = RenderCancel::new();

        // Capacity 1: the worker sends exactly one message and exits.
        // A bounded channel makes that a compile-time-ish guarantee
        // rather than an unbounded buffer nobody drains.
        let (tx, rx): (SyncSender<Outcome>, Receiver<Outcome>) = sync_channel(1);
        let worker_cancel = cancel.clone();

        // Traced BEFORE the move, because `request` is consumed by the
        // closure. Every generation that starts gets a line, which is what
        // makes the "six rapid zoom steps start six generations and complete
        // one" observation checkable from outside the process rather than
        // being a claim about code that has to be believed.
        //
        // The page and scale ride along because they are the whole
        // `RenderKey`: a trace that says a render started without saying
        // what OF cannot distinguish a legitimate new request from the
        // restart-the-same-render livelock the key exists to prevent.
        let (traced_page, traced_scale) = (request.page_index, request.raster_scale);
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        crate::diag::trace(|| {
            format!("render-spawn gen={generation} page={traced_page} scale={traced_scale}")
        });

        let handle = std::thread::spawn(move || {
            let outcome = render_on_worker(&request, &worker_cancel);
            // A send failure means the shell dropped the receiver — the
            // document was closed, or a later render superseded this
            // one and the slot was replaced. Both are ordinary; there is
            // nobody left to tell.
            let _ = tx.send(outcome);
        });

        let started = Instant::now();

        // The bounded in-frame wait. See IN_FRAME_BUDGET.
        match rx.recv_timeout(IN_FRAME_BUDGET) {
            Ok(outcome) => {
                // Finished inside the budget: join immediately so no
                // thread outlives the call, and return inline.
                let _ = handle.join();
                let elapsed_ms = started.elapsed().as_millis();
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                crate::diag::trace(|| {
                    format!("render-inline gen={generation} ms={elapsed_ms} async=0")
                });
                Self::outcome_to_result(outcome)
            }
            Err(RecvTimeoutError::Timeout) => {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                crate::diag::trace(|| {
                    format!(
                        "render-async-started gen={generation} budget_ms={}",
                        IN_FRAME_BUDGET.as_millis()
                    )
                });
                self.in_flight = Some(InFlight {
                    rx,
                    cancel,
                    handle: Some(handle),
                    key,
                    generation,
                    started,
                });
                None
            }
            Err(RecvTimeoutError::Disconnected) => {
                // The worker panicked without sending. Surface it as a
                // render failure rather than hanging forever waiting for
                // a message that will never arrive.
                let _ = handle.join();
                Some(Err(crate::text::canvas_render_worker_stopped().to_owned()))
            }
        }
    }

    /// Collect a finished render, if one is ready. Never blocks.
    ///
    /// Returns `None` both when nothing is running and when the render
    /// is still going — the shell's action is the same either way.
    pub fn poll(&mut self) -> Option<Result<RenderedPixels, String>> {
        let flight = self.in_flight.as_mut()?;
        match flight.rx.try_recv() {
            Ok(outcome) => {
                let mut flight = self.in_flight.take()?;
                if let Some(handle) = flight.handle.take() {
                    let _ = handle.join();
                }
                let elapsed_ms = flight.started.elapsed().as_millis();
                let generation = flight.generation;
                let kind = match &outcome {
                    Outcome::Done(_) => "done",
                    Outcome::Cancelled => "cancelled",
                    Outcome::Failed(_) => "failed",
                };
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                crate::diag::trace(|| {
                    format!("render-async-done gen={generation} ms={elapsed_ms} outcome={kind}")
                });
                Self::outcome_to_result(outcome)
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => None,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                let mut flight = self.in_flight.take()?;
                if let Some(handle) = flight.handle.take() {
                    let _ = handle.join();
                }
                Some(Err(crate::text::canvas_render_worker_stopped().to_owned()))
            }
        }
    }

    /// How long the current render has been outstanding, if any.
    ///
    /// The shell uses this to decide whether the canvas has been stale
    /// long enough to say so. Returning the duration rather than a
    /// boolean keeps the threshold — a presentation decision — out of
    /// this module.
    #[allow(
        dead_code,
        reason = "the stale-canvas disclosure is a status-bar sentence and lands at stage S2; kept with the clock it reads, because the threshold is the shell's decision and the measurement is this module's" // ui-text-exempt: clippy lint justification, never displayed
    )]
    pub fn in_flight_since(&self) -> Option<Duration> {
        self.in_flight.as_ref().map(|f| f.started.elapsed())
    }

    /// Whether a render is currently running.
    pub fn is_rendering(&self) -> bool {
        self.in_flight.is_some()
    }

    /// Stop any in-flight render and wait for the thread to exit.
    ///
    /// **This is the choke point that makes `Arc<EditSession>`
    /// workable.** A worker holds a clone of the session for as long as
    /// it renders, so `Arc::get_mut` fails while one is running. Every
    /// mutation must go through a path that calls this first — so by the
    /// time any edit touches the session, the render holding the other
    /// reference has exited.
    ///
    /// The alternative rulings were rejected with numbers: blocking the
    /// edit until the render finishes costs up to 58 s, which is the
    /// freeze this whole module exists to remove; snapshotting the
    /// session would need a public deep-copy impl on `EditSession`
    /// (which is not `Clone`) and would copy the document per edit.
    /// Cancel-then-mutate costs the measured **28.9 ms** of teardown.
    ///
    /// S0 makes no edits, so nothing calls this yet outside [`Drop`]. It
    /// is salvaged now, with its argument intact, because the first edit
    /// to arrive without it would reintroduce the 58-second freeze
    /// through a door that had already been closed once.
    #[allow(
        dead_code,
        reason = "the mutation choke point; S0 has no mutations, and the first stage that does (S4) must route through this rather than re-derive it" // ui-text-exempt: clippy lint justification, never displayed
    )]
    pub fn cancel_and_wait(&mut self) {
        self.cancel_in_flight();
    }

    /// Cancel, drain and join. Idempotent.
    fn cancel_in_flight(&mut self) {
        let Some(mut flight) = self.in_flight.take() else {
            return;
        };
        flight.cancel.cancel();
        if let Some(handle) = flight.handle.take() {
            // Join rather than detach: the whole point is that the
            // session's other reference is gone when this returns. A
            // detached thread might still be holding it.
            let _ = handle.join();
        }
    }

    fn outcome_to_result(outcome: Outcome) -> Option<Result<RenderedPixels, String>> {
        match outcome {
            Outcome::Done(pixels) => Some(Ok(*pixels)),
            Outcome::Failed(message) => Some(Err(message)),
            // A cancelled render has no result and is not a failure.
            // The shell keeps whatever it was already showing.
            Outcome::Cancelled => None,
        }
    }
}

impl Drop for RenderWorker {
    /// Closing a document must not leave a 58-second render running
    /// against a session nobody can see.
    fn drop(&mut self) {
        self.cancel_in_flight();
    }
}

/// The worker body. Runs on the spawned thread; touches no GUI type.
fn render_on_worker(request: &RenderRequest, cancel: &RenderCancel) -> Outcome {
    let mut options = pdfce_render::RenderOptions::default();
    // Everything NOT set here keeps its `RenderOptions` default
    // deliberately — see the module docs' salvage note: the bundled font
    // environment (reproducible on any machine), the operator-ruled CMYK
    // intent, and `None` view magnification (the print-correct answer,
    // T-12.8). Each becomes a request field when a surface exists to vary
    // it, not before.
    options.cancel = Some(cancel.clone());
    options.annotations = request.annotations;
    // Cloned rather than moved because the worker takes the request by
    // reference — a `BTreeSet<ObjId>` per render, against a rasterization
    // measured in seconds. `None` here is not the same as an empty set: it
    // means "obey the document" (T-12.9), and collapsing the two would
    // reveal every layer the document turned off.
    options.layers = request.layers.clone();

    // `session.view()`, NOT `session.document()` — the view composes the
    // overlay and the staging buffer, so unsaved edits are what gets drawn.
    // The borrow lives and dies inside this function, which is why the
    // request can own the `Arc` and still hand `render_page_with_view` a
    // reference.
    let view = request.session.view();
    match pdfce_render::render_page_with_view(&view, &request.page, request.raster_scale, &options)
    {
        Ok(rendered) => Outcome::Done(Box::new(RenderedPixels {
            pixmap: rendered.pixmap,
            diagnostics: rendered.diagnostics,
            // The key is derived from the request the render was actually
            // run from, so the texture cannot be labelled with anything but
            // the inputs that produced it.
            key: RenderKey::of(request),
        })),
        Err(e) if cancel.is_cancelled() => {
            // Deliberate abandonment, not a defect. Checking the token
            // rather than matching the error variant keeps this correct
            // if the render gains other early-exit paths.
            let _ = e;
            Outcome::Cancelled
        }
        Err(e) => Outcome::Failed(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A `RenderKey` for the default render of a page at a scale:
    /// annotations on (what a reader shows), no layer override.
    ///
    /// The two-argument shorthand the geometry cases use, so a test about
    /// page and scale is not obscured by two constants it does not vary.
    fn key(page_index: usize, scale: f32) -> RenderKey {
        RenderKey::new(page_index, scale, true, 0)
    }

    /// Two renders of the same thing must compare EQUAL.
    ///
    /// # Why this is the load-bearing test and not bookkeeping
    ///
    /// The shell re-runs its staleness check every frame, and while a
    /// background render is in flight the cached texture has not been
    /// replaced — so the check keeps saying "stale" and keeps asking
    /// for the same render. `spawn` recognises that request as the one
    /// already running *only* through this equality.
    ///
    /// If it fails, every frame cancels the render the previous frame
    /// started and begins an identical one. A page slower than a single
    /// frame then **never finishes at all** — which is strictly worse
    /// than the freeze this module was written to remove, and it would
    /// look like a hang rather than a bug.
    ///
    /// This was a real defect in the original's first draft: the guard did
    /// not exist, and the livelock was reasoned out before it could be
    /// observed.
    #[test]
    fn the_same_request_twice_is_recognised_as_the_same_render() {
        assert_eq!(key(3, 2.0), key(3, 2.0));
    }

    /// Every staleness key must be part of the comparison.
    ///
    /// The test above cannot distinguish a correct `RenderKey` from one
    /// that compares nothing at all and reports every pair as equal — and
    /// that failure is not hypothetical. A key that ignored a field would
    /// make the guard swallow a *genuine* new request: change the zoom,
    /// and the shell would decline to re-render because it believes the
    /// in-flight job already covers it. The page would stop responding to
    /// zoom entirely.
    ///
    /// So each field is varied one at a time. Dropping any single field
    /// from `RenderKey`'s `PartialEq` fails exactly one of these — and
    /// each key this struct grows (see the module docs) must add its own
    /// line here in the same commit.
    #[test]
    fn changing_any_single_render_input_makes_a_different_key() {
        let base = RenderKey::new(3, 2.0, true, 7);
        assert_ne!(
            base,
            RenderKey::new(4, 2.0, true, 7),
            "page index must be compared"
        );
        assert_ne!(
            base,
            RenderKey::new(3, 2.5, true, 7),
            "raster scale must be compared"
        );
        assert_ne!(
            base,
            RenderKey::new(3, 2.0, false, 7),
            "annotation visibility must be compared, or View ▸ Display's \
             `view.show_annotations` toggles a bool and redraws nothing"
        );
        assert_ne!(
            base,
            RenderKey::new(3, 2.0, true, 8),
            "the layer-override generation must be compared, or the Layers \
             panel's visibility control ticks and redraws nothing — which is \
             the exact defect that kept the checkbox out of the build"
        );
    }

    /// **★ Every field is in exactly one of the two staleness categories.**
    ///
    /// [`RenderKey::discrete_inputs`] and [`RenderKey::scale_bits`] are how
    /// the shell decides whether a change re-rasterizes **now** or waits out
    /// the zoom debounce. A field that appears in neither is a change the
    /// shell cannot see at all: the key would compare unequal, the worker
    /// would happily run the new render — and nothing would ever ask for it,
    /// because the texture would still look current.
    ///
    /// That is not the same failure as an uncompared field, and it is worse:
    /// the module's own key would be *correct* while the picture stayed
    /// wrong, so the obvious place to look would be the innocent one.
    ///
    /// Each field is varied one at a time and the pair is asserted to move.
    /// A key this struct grows must add its line here in the same commit,
    /// exactly as it must to the test above.
    #[test]
    fn every_render_input_is_either_discrete_or_the_scale() {
        let base = RenderKey::new(3, 2.0, true, 7);
        let moved = |k: RenderKey| {
            k.discrete_inputs() != base.discrete_inputs() || k.scale_bits() != base.scale_bits()
        };
        assert!(moved(RenderKey::new(4, 2.0, true, 7)), "page index");
        assert!(moved(RenderKey::new(3, 2.5, true, 7)), "raster scale");
        assert!(moved(RenderKey::new(3, 2.0, false, 7)), "annotations");
        assert!(moved(RenderKey::new(3, 2.0, true, 8)), "layers generation");
    }

    /// **The scale is the ONLY debounced input.**
    ///
    /// The other half of the split, asserted from the other side: if a
    /// discrete input leaked into the scale category it would inherit the
    /// 150 ms zoom debounce, and a click on the annotation toggle would take
    /// a fifth of a second to do anything for no reason an operator could
    /// see. If the scale leaked into the discrete category, every notch of a
    /// wheel gesture would rasterize a CAD sheet — the behaviour
    /// `ZOOM_SETTLE` exists to remove.
    #[test]
    fn only_the_raster_scale_is_debounced() {
        let base = RenderKey::new(3, 2.0, true, 7);
        // A scale change moves the scale category and NOT the discrete one.
        let rescaled = RenderKey::new(3, 2.5, true, 7);
        assert_ne!(rescaled.scale_bits(), base.scale_bits());
        assert_eq!(rescaled.discrete_inputs(), base.discrete_inputs());
        // …and each discrete change moves the discrete category and NOT the
        // scale.
        for changed in [
            RenderKey::new(4, 2.0, true, 7),
            RenderKey::new(3, 2.0, false, 7),
            RenderKey::new(3, 2.0, true, 8),
        ] {
            assert_eq!(changed.scale_bits(), base.scale_bits());
            assert_ne!(changed.discrete_inputs(), base.discrete_inputs());
        }
    }

    /// A scale difference far below any perceptible threshold is still a
    /// different render.
    ///
    /// Comparing `f32` by bit pattern rather than by a tolerance is
    /// deliberate. The shell derives `raster_scale` from the same
    /// arithmetic each frame, so an unchanged zoom yields bit-identical
    /// values and the guard holds; but any difference at all means the
    /// shell has asked for a different picture, and a tolerance would
    /// silently serve it the wrong one.
    #[test]
    fn a_one_bit_scale_difference_is_a_different_render() {
        let a = key(0, 1.0);
        let b = key(0, f32::from_bits(1.0f32.to_bits() + 1));
        assert_ne!(a, b);
    }

    /// A fresh worker is idle, and reports no in-flight age.
    ///
    /// Guards the (stage S2) status-bar disclosure against the most
    /// embarrassing failure mode: announcing that the canvas is behind
    /// when nothing is rendering.
    #[test]
    fn an_idle_worker_reports_nothing_in_flight() {
        let worker = RenderWorker::default();
        assert!(!worker.is_rendering());
        assert!(worker.in_flight_since().is_none());
    }

    /// Dropping the worker must not leave a thread running.
    ///
    /// The `Drop` impl exists because closing a document must not leave a
    /// 58-second render running against a session nobody can see. There is
    /// no page to render in a unit test, so what is checked is the weaker
    /// but still meaningful property that the teardown path is reachable
    /// and idempotent on an idle worker — a `cancel_in_flight` that
    /// panicked or blocked on an empty slot would hang every close.
    #[test]
    fn cancelling_an_idle_worker_is_a_harmless_no_op() {
        let mut worker = RenderWorker::default();
        worker.cancel_and_wait();
        worker.cancel_and_wait();
        assert!(!worker.is_rendering());
    }
}
