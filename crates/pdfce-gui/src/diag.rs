//! # diag — an opt-in trace of what the shell actually received
//!
//! Salvaged from `D:\Dev\pdfce\crates\pdfce-gui\src\diag.rs` (Class A,
//! `SALVAGE.md`). **The header below is carried across verbatim**, because
//! it records *why* the channel exists — an argument that took a real
//! investigation to earn and that a paraphrase would lose.
//!
//! What is salvaged at S0 is the trace channel itself ([`enabled`],
//! [`trace`]). The other 800 lines of the original — the `PDFCE_DIAG_SCRIPT`
//! scripted-input harness, its `Step` grammar, `ScriptTool`, the font-folder
//! preload — land with `tools/ui-verify` at stage S1, which is the thing
//! that consumes them. Salvaging a script grammar before there is a harness
//! to run it would be shipping a language with no speakers.
//!
//! ---
//!
//! ## Why this exists
//!
//! A GUI defect in this project has exactly one honest oracle: the running
//! application (standing rule R86). Everything else — reading the dispatch
//! chain, unit-testing the pure decision functions, checking the CLI's answer
//! to the same query — can be entirely green while the operator still cannot
//! select an object, because the thing that failed sits between the window
//! manager and our first line of code.
//!
//! That happened. On 2026-08-04 the operator reported that clicking a drawing
//! object selected nothing. The hit-test was verified correct through
//! `pdfce-cli` (the same `pdfce-core` query, same fixture, right answer), every
//! selection decision function passed headless, and the dispatch from toolbar
//! toggle to `run_vector_edit_tool` read correctly line by line. Reading harder
//! was not going to close the gap: the remaining candidates were all of the
//! form "does `Response::clicked()` fire at all", which is unobservable from
//! the source.
//!
//! ## Why it does not just take a screenshot
//!
//! The operator was using the machine for real work and explicitly asked that
//! the screen not be commandeered. So the diagnostic has to come out of the
//! process as *text*, from a window that need never be looked at — which also
//! makes it usable from a script, a CI run, or a machine with no display at
//! all.
//!
//! ## Contract
//!
//! - **Off unless asked.** Enabled only when the `PDFCE_DIAG` environment
//!   variable is set to a non-empty value, read once per process. With it
//!   unset, [`enabled`] is a relaxed atomic load and [`trace`]'s argument
//!   closure is never called — so a call site costs nothing and may be left in
//!   place permanently rather than added and deleted around each investigation
//!   (which is how the *next* defect ends up needing this file written again).
//! - **Writes to stderr, one line per event, `key=value` fields.** stderr
//!   because it needs no path, no handle to keep open, no failure mode of its
//!   own, and redirects with `2>`. `key=value` because the consumer is a grep
//!   or an LLM, not a person reading a log.
//! - **Never a user-facing string.** Nothing here is shown in the interface, so
//!   none of it belongs in [`crate::text`] (the ui-string catalog governs
//!   operator-visible copy).
//! - **Never load-bearing.** No behaviour may depend on the trace. If deleting
//!   this module changed what the application does, the trace would have become
//!   a feature with no tests.
//!
//! ## Usage
//!
//! ```text
//! PDFCE_DIAG=1 pdfce-gui file.pdf 2> trace.txt
//! ```
//!
//! ---
//!
//! ## What stage S2 added, and why: three things the harness needs
//!
//! `PROJECT_PLAN.md` §4.3 tabulates *"what the application owes the
//! harness"* — three requirements discovered by **building** `tools/ui-verify`
//! at S1 rather than by reading code. Each removes a harness workaround.
//! Two of the three are implemented in terms of machinery added here.
//!
//! ### The de-duplicating gate ([`trace_changed`])
//!
//! The trace is written for a *machine* consumer, and the machine's question
//! is almost always *"what is the current value of X?"* — answered by the
//! **last** line carrying X. A call site in the frame loop that re-emits an
//! unchanged value 60 times a second answers that question no better and
//! buries every other event while doing it. Measured on the S1 binary: the
//! `canvas-pointer` line produced **50 identical lines in 9 seconds** with
//! the pointer stationary, because it fired once per frame rather than once
//! per movement.
//!
//! That is not merely untidy. `ui-verify` reads the trace file repeatedly
//! while it drives (`Session::trace` re-parses the whole capture after every
//! settle), so per-frame noise is re-parsed on every read and grows the
//! capture without adding information. Worse, it makes a human reading the
//! trace scroll past thousands of lines to find the one event that mattered
//! — which is exactly how pdfce's own investigation missed a `UNPARSEABLE`
//! rejection that was traced on every single run.
//!
//! So: [`trace_changed`] remembers the last line emitted under a **slot**
//! and emits only when the newly built line differs. "Changed" is defined as
//! *the formatted line differs*, which is deliberately the same definition
//! the consumer uses — a difference too small to change the printed text is,
//! by construction, a difference the consumer could not have read anyway.
//!
//! ### The named-region sink ([`ui_rect`])
//!
//! §4.3 requirement 2. A pixel check needs to know **where** to look, and
//! there are only two honest sources: the application measures the rect on
//! the frame it reports (correct under every layout change), or the harness
//! hard-codes a fraction of the window (stale the first time a panel is
//! resized — the hazard §4.2 prerequisite 1 names). [`ui_rect`] is the first
//! source.
//!
//! It is a **process-global sink on purpose**, and that is the seam: the
//! ribbon is being built in `egui-shell`, which cannot depend on this crate
//! (`tools/gates/check-shell-purity.sh` enforces the one-directional
//! dependency), so it will expose a *callback* that the application supplies.
//! [`ui_rect`] already has the exact `fn(&str, egui::Rect)` shape such a
//! callback takes, captures nothing, and needs no `&mut` threaded through
//! every widget signature. Wiring the ribbon to it is therefore a single
//! registration line at start-up and **no change to this file** — which is
//! the property that lets the two agents' work land independently.
//!
//! ### Zero-cost when off, in both
//!
//! Both check [`enabled`] before touching their registries, so with
//! `PDFCE_DIAG` unset a call site costs one relaxed atomic load and no lock,
//! no hash, no allocation and no formatting. That is what makes it correct
//! to leave these calls in permanently — see the contract above.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex, MutexGuard, OnceLock};

/// Whether tracing was requested for this process.
///
/// Resolved once and cached: the check sits in a per-frame path, and re-reading
/// the environment there would put a lock and an allocation in the frame loop
/// to answer a question that cannot change after start-up.
pub fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| {
        // ui-text-exempt: environment variable name, never displayed
        std::env::var_os("PDFCE_DIAG").is_some_and(|v| !v.is_empty())
    })
}

/// Emit one trace line, building the message only if tracing is on.
///
/// Takes a closure rather than a `String` so a disabled build path performs no
/// formatting — the call sites interpolate rects, pointer positions and hit
/// counts, and doing that work every frame to throw it away would be a real
/// cost in the one loop that must not get slower.
pub fn trace(f: impl FnOnce() -> String) {
    if enabled() {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        eprintln!("pdfce-diag {}", f());
    }
}

// ---------------------------------------------------------------------------
// The de-duplicating gate
// ---------------------------------------------------------------------------

/// The last line emitted under each [`trace_changed`] slot.
///
/// A `Mutex` rather than a `thread_local!` or a `RefCell` because
/// [`ui_rect`] is designed to be handed to `egui-shell` as a plain
/// `fn(&str, Rect)` callback (see the module docs), and a callback whose
/// correctness depends on which thread invokes it is a trap for whoever wires
/// it up. The lock is uncontended in practice — everything that traces layout
/// runs on the UI thread — and it is only ever taken when tracing is on.
///
/// Keys are `&'static str`, which is not an accident: a slot names a *call
/// site*, and call sites are known at compile time. It also means the
/// steady-state (nothing changed) path performs **no allocation at all** —
/// only a hash of a string that already exists.
static LAST_LINE: LazyLock<Mutex<HashMap<&'static str, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// The last rect emitted for each named region by [`ui_rect`].
///
/// Separate from [`LAST_LINE`] and typed as a [`egui::Rect`] rather than as a
/// rendered string for two reasons: region names are runtime values (a ribbon
/// group's caption id is data, not a literal), so they cannot key
/// [`LAST_LINE`]; and comparing the rect itself rather than its rendering
/// keeps the comparison independent of the format the line happens to be
/// printed in.
static LAST_UI_RECT: LazyLock<Mutex<HashMap<String, egui::Rect>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// The region names [`ui_rect`] has been called with **so far this frame**.
///
/// ## ★ Why this exists: the trace is a CHANGE LOG, and a change log cannot
/// say that something stopped
///
/// [`ui_rect`] emits only when a region's rect *differs* from the last one
/// emitted for that name, which is what keeps the channel usable — a per-frame
/// dump of ~60 regions at 60 fps is a torrent nobody can read. The cost is
/// that a region which stops being drawn **emits nothing**, so its last known
/// rect stands in the trace forever and a reader has no way to tell "still
/// there, unmoved" from "gone forty frames ago".
///
/// That is not academic. It made `ui-verify`'s UI-scale check report **18
/// ribbon controls as lying outside the window** at a large scale. They did
/// not: the ribbon's overflow had correctly swallowed them, and every one of
/// those rects was its position from an earlier frame at a smaller scale. The
/// screenshot showed a perfectly laid-out ribbon with a *5 more* button. The
/// harness was reading a fossil and reporting it as a live layout defect —
/// the exact false-defect outcome `crate::diag`'s own contract is written to
/// avoid.
///
/// So [`end_ui_frame`] diffs this set against the previous frame's and emits
/// `ui-rect-gone name=…` for anything that disappeared. The log stays a change
/// log and becomes an *honest* one, reporting both directions of change.
static UI_RECTS_THIS_FRAME: LazyLock<Mutex<std::collections::HashSet<String>>> =
    LazyLock::new(|| Mutex::new(std::collections::HashSet::new()));

/// The region names that were drawn during the **previous** frame.
static UI_RECTS_LAST_FRAME: LazyLock<Mutex<std::collections::HashSet<String>>> =
    LazyLock::new(|| Mutex::new(std::collections::HashSet::new()));

/// Lock a registry, ignoring poisoning.
///
/// A panic while one of these locks was held would otherwise disable the
/// trace for the rest of the process — and the trace is the thing you reach
/// for *because* something went wrong. `into_inner` keeps the channel alive
/// on a possibly-stale map, which can at worst cost one duplicate or one
/// suppressed line. The contract says the trace is never load-bearing; this
/// is that contract applied to its own failure mode.
fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Emit one trace line **only when it differs from the last line emitted
/// under the same slot**.
///
/// # What this is for
///
/// Frame-loop call sites. A value that is re-reported unchanged 60 times a
/// second tells a consumer nothing it did not already know from the previous
/// line, and buries the events that *are* news. See the module docs for the
/// measured case (50 identical `canvas-pointer` lines in 9 seconds) and for
/// why noise costs the harness real work rather than merely looking untidy.
///
/// # The definition of "changed", and why it is the formatted line
///
/// Not the underlying value: the **rendered text**. Two consequences, both
/// wanted:
///
/// * A difference too small to change the printed text is a difference the
///   consumer could not have read anyway, so suppressing it loses nothing.
///   The pointer trace prints `{:.2}`; sub-hundredth jitter is invisible to
///   the parser by construction.
/// * A call site does not have to invent an epsilon, or keep a parallel copy
///   of its own state to compare against. There is one rule, in one place.
///
/// # Slots
///
/// A slot is the event name, plus a discriminator when one event has several
/// independent subjects. Two call sites sharing a slot will each suppress the
/// other's lines, which is a real bug and the reason the parameter is
/// `&'static str` — it is meant to be a literal you can grep for.
///
/// Costs nothing when tracing is off: the closure is not called and neither
/// registry is touched.
pub fn trace_changed(slot: &'static str, f: impl FnOnce() -> String) {
    if !enabled() {
        return;
    }
    let line = f();
    // The lock is released before the write: `eprintln!` takes stderr's own
    // lock, and holding two locks in a fixed order across a call that can
    // block is a deadlock waiting for a second tracer to be added.
    let changed = record_if_changed(&mut lock(&LAST_LINE), slot, &line);
    if changed {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        eprintln!("pdfce-diag {line}");
    }
}

/// The whole decision [`trace_changed`] makes, over an explicit map.
///
/// Split out from [`trace_changed`] — and taking the map as an argument
/// rather than reaching for the global — so the de-duplication rule is
/// testable without an environment variable, without stderr capture, and
/// without two parallel tests fighting over one process-global registry. The
/// rule is the interesting part; `eprintln!` is not.
///
/// Returns whether the caller should print, and records the line if so.
fn record_if_changed(
    map: &mut HashMap<&'static str, String>,
    slot: &'static str,
    line: &str,
) -> bool {
    if map.get(slot).is_some_and(|prev| prev == line) {
        return false;
    }
    map.insert(slot, line.to_owned());
    true
}

/// Declare where a **named UI region** is, in window logical points.
///
/// `PROJECT_PLAN.md` §4.3 requirement 2. Emits
///
/// ```text
/// pdfce-diag ui-rect name=<name> rect=[[x0 y0] - [x1 y1]]
/// ```
///
/// the first time a region is seen and again whenever it moves or resizes,
/// and nothing at all on the frames in between.
///
/// # Why the application declares this rather than the harness computing it
///
/// A pixel check — "is this caption legible?" — has to know which pixels to
/// measure. The alternative source is a fraction of the window written into
/// the harness, and such a fraction is correct exactly until the first panel
/// is resized, the ribbon collapses to an icon rail, or a workspace is
/// switched. `MODES_AND_PANELS.md` puts all three on the roadmap. A rect
/// measured on the frame it is reported for cannot go stale, because there is
/// no interval between the measurement and the claim.
///
/// # The `rect=` format is not a choice
///
/// It is `egui::Rect`'s own `Debug`, `[[x0 y0] - [x1 y1]]`, because
/// `tools/ui-verify`'s parser already reads that shape (`trace.rs`'s
/// `parse_egui_rect`) for the canvas rect. Emitting a second, tidier spelling
/// would mean two parsers for one concept, which is how the two ends of a
/// bridge drift apart. Pass the `Rect` and let `Debug` write it.
///
/// # The seam for the ribbon
///
/// This function takes `&str` and `egui::Rect`, captures nothing, and returns
/// nothing — it *is* an `fn(&str, egui::Rect)`. `egui-shell` cannot call into
/// this crate (the dependency is one-directional and gated), so it exposes a
/// callback of that shape and the application registers this function.
/// Nothing here needs to change when it does.
///
/// # Naming regions
///
/// Names are matched literally by checks, so they are part of the contract:
/// pick a stable, lowercase, hyphenated noun for the thing an operator would
/// point at (`page`, `canvas-viewport`, `ribbon-group-caption:view/zoom`).
/// Renaming one silently un-aims whatever check was measuring it.
/// [`ui_rect`], but **only if the region is actually visible** inside `clip`.
///
/// # ★ Why a scroll area needs this, and why the plain call is a trap there
///
/// `egui` lays out every child of a `ScrollArea` and then *clips* the ones
/// outside the viewport. So a collapsible header scrolled below the fold still
/// runs its layout, still has a perfectly good `Rect`, and calling [`ui_rect`]
/// with it publishes coordinates for something **nobody can see**.
///
/// A harness reading that declaration measures the pixels at those coordinates
/// — which belong to whatever is genuinely on screen there: another panel, the
/// document, the desktop. It then reports a contrast figure that is a fact
/// about the wrong widget.
///
/// That is not hypothetical. The first live run of `settings_headings_legible`
/// — the regression check for `DEFECTS.md` **D2**, which had SKIPPED for the
/// whole life of the project — reported three of eight headings as illegible
/// or blank. The dialog was fine: the two headings actually on screen measured
/// **13.91:1** against a 3:1 floor. All three "failures" were headings
/// scrolled out of view, and the check was reading the Pages panel and the
/// drawing behind the dialog.
///
/// A check that fires when nothing is wrong is one that gets switched off, and
/// this one guards the defect that justified building the harness.
///
/// # Why the fix is here rather than in the harness
///
/// The harness *could* intersect every rect with the dialog's body. Doing it
/// here is better for a reason that outlives this dialog: it makes the
/// declaration **mean** something — *this region is on screen at this rect* —
/// so every consumer gets the guarantee rather than each one re-deriving it.
/// It is the same repair as `ui-rect-gone`: the channel should describe what
/// is visible, not what was laid out.
///
/// # The test is intersection, not containment
///
/// A heading half-scrolled off the bottom is still partly on screen and still
/// worth measuring — a contrast check samples what it can reach. Requiring
/// full containment would silently drop the boundary case, which on a scroll
/// area is a case that exists on almost every frame.
pub fn ui_rect_visible(name: &str, rect: egui::Rect, clip: egui::Rect) {
    if !enabled() {
        return;
    }
    if clip.intersects(rect) {
        ui_rect(name, rect);
    }
    // Deliberately silent when it does not intersect. This is not a retirement
    // — `end_ui_frame` handles that, and a region that scrolls out of view and
    // back is exactly the case it was built for: it emits `ui-rect-gone` on the
    // frame the region stops being declared, and the rect is re-emitted when it
    // returns.
}

pub fn ui_rect(name: &str, rect: egui::Rect) {
    if !enabled() {
        return;
    }
    // Recorded before the change test, so a region that is drawn at an
    // unchanged rect still counts as PRESENT this frame. Getting this the
    // other way round would make every unmoved region look retired, which is
    // the failure this set exists to prevent, inverted.
    lock(&UI_RECTS_THIS_FRAME).insert(name.to_owned());
    let changed = record_rect_if_changed(&mut lock(&LAST_UI_RECT), name, rect);
    if changed {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        eprintln!("pdfce-diag ui-rect name={name} rect={rect:?}");
    }
}

/// **Close a frame's region census and report anything that stopped being
/// drawn.**
///
/// Called once at the end of every frame, from `crate::app::frame`. See
/// [`UI_RECTS_THIS_FRAME`] for the defect this exists to remove — in one
/// sentence: a change log that only reports appearances lets a consumer read a
/// stale rect as a live one, and that produced a confident, wrong,
/// eighteen-item layout-defect report.
///
/// # What it emits
///
/// One `ui-rect-gone name=…` line per region that was drawn last frame and was
/// not drawn this frame. Nothing at all on a steady frame, which is the common
/// case and keeps the channel as quiet as it was before.
///
/// # It also forgets the region's last rect
///
/// Deliberately, and it is the half that is easy to omit. Without it, a region
/// that disappears and later comes back **at the same rect** would emit
/// nothing on its return — `record_rect_if_changed` would compare against the
/// remembered value and suppress it — leaving the trace saying the region went
/// away and never saying it returned. Forgetting on retirement makes a
/// reappearance always visible.
pub fn end_ui_frame() {
    if !enabled() {
        return;
    }
    let mut this = lock(&UI_RECTS_THIS_FRAME);
    let mut last = lock(&UI_RECTS_LAST_FRAME);
    let mut retired: Vec<String> = last.difference(&this).cloned().collect();
    if !retired.is_empty() {
        // Sorted so a diff between two runs of the same scenario is stable.
        // `HashSet` iteration order is not, and an unstable trace is one that
        // cannot be compared against a previous capture.
        retired.sort();
        let mut rects = lock(&LAST_UI_RECT);
        for name in &retired {
            rects.remove(name);
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            eprintln!("pdfce-diag ui-rect-gone name={name}");
        }
    }
    std::mem::swap(&mut *last, &mut this);
    this.clear();
}

/// The decision [`ui_rect`] makes, over an explicit map — see
/// [`record_if_changed`] for why it is split out this way.
///
/// The comparison is exact rather than epsilon-based, deliberately. An
/// unmoved region is laid out from the same inputs every frame and produces a
/// bit-identical `Rect`; a region that moved by a quarter of a point moved,
/// and a check measuring it wants to know. There is no third case in which an
/// epsilon would help.
fn record_rect_if_changed(
    map: &mut HashMap<String, egui::Rect>,
    name: &str,
    rect: egui::Rect,
) -> bool {
    if map.get(name).is_some_and(|prev| *prev == rect) {
        return false;
    }
    // Only allocates when the region is new or has actually moved.
    map.insert(name.to_owned(), rect);
    true
}

/// Forget every de-duplication slot, so the next frame re-declares
/// everything.
///
/// Called when a document is opened. Without it, opening a second document
/// whose layout happens to be identical to the first would emit **no** canvas
/// line for the new document, and §4.3 requirement 1 is specifically *"at
/// least once per document open"* — a guarantee the consumer is entitled to
/// read as "there is a line for this document", not "there is a line for some
/// document whose numbers still happen to apply".
///
/// It is cheap and it is not per-frame, so it clears both registries rather
/// than trying to decide which slots a document open could have invalidated.
pub fn reset_change_gates() {
    if !enabled() {
        return;
    }
    lock(&LAST_LINE).clear();
    lock(&LAST_UI_RECT).clear();
    // The frame census goes with them. Not clearing it would make the first
    // frame after a document open emit `ui-rect-gone` for every region of the
    // PREVIOUS document that the new one happens not to draw yet — a burst of
    // retirements that describe a document nobody has open, at the one moment
    // a reader is most likely to be looking.
    lock(&UI_RECTS_THIS_FRAME).clear();
    lock(&UI_RECTS_LAST_FRAME).clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [`trace`] must not evaluate its closure when tracing is off.
    ///
    /// This is the property that lets call sites be left in permanently:
    /// the moment a disabled trace still formats its message, every one of
    /// them becomes a per-frame allocation and the next engineer starts
    /// deleting them again.
    ///
    /// The test is written so it is meaningful in BOTH environments: if the
    /// harness itself runs under `PDFCE_DIAG`, the closure is expected to
    /// run, so the assertion follows `enabled()` rather than assuming it.
    #[test]
    fn a_disabled_trace_never_builds_its_message() {
        let mut built = false;
        trace(|| {
            built = true;
            String::new()
        });
        assert_eq!(built, enabled());
    }

    /// [`trace_changed`] must not evaluate its closure when tracing is off,
    /// for exactly the same reason as [`trace`].
    #[test]
    fn a_disabled_change_gated_trace_never_builds_its_message() {
        let mut built = false;
        trace_changed("test-disabled", || {
            built = true;
            String::new()
        });
        assert_eq!(built, enabled());
    }

    /// The property the gate exists for: a repeated identical line is
    /// emitted once.
    ///
    /// This is the fix for the measured defect — 50 identical
    /// `canvas-pointer` lines in 9 seconds with the pointer stationary.
    #[test]
    fn an_unchanged_line_is_emitted_once_and_then_suppressed() {
        let mut map = HashMap::new();
        assert!(
            record_if_changed(
                &mut map,
                "canvas-pointer",
                "canvas-pointer screen=(1.0,2.0)"
            ),
            "the first sighting of a value is news and must be emitted"
        );
        for _ in 0..50 {
            assert!(
                !record_if_changed(
                    &mut map,
                    "canvas-pointer",
                    "canvas-pointer screen=(1.0,2.0)"
                ),
                "an unchanged value tells the consumer nothing the previous line did not"
            );
        }
    }

    #[test]
    fn a_changed_line_is_emitted_again() {
        let mut map = HashMap::new();
        assert!(record_if_changed(&mut map, "canvas", "canvas zoom=1.0"));
        assert!(record_if_changed(&mut map, "canvas", "canvas zoom=1.5"));
        // …and the new value is now the one that suppresses.
        assert!(!record_if_changed(&mut map, "canvas", "canvas zoom=1.5"));
        assert!(
            record_if_changed(&mut map, "canvas", "canvas zoom=1.0"),
            "returning to a previously seen value is a change, not a repeat: the \
             consumer's last line says 1.5"
        );
    }

    /// Two slots must not suppress each other. The whole point of a slot is
    /// that one noisy call site cannot silence another.
    #[test]
    fn slots_are_independent() {
        let mut map = HashMap::new();
        assert!(record_if_changed(&mut map, "a", "same text"));
        assert!(
            record_if_changed(&mut map, "b", "same text"),
            "an identical line under a different slot is a different fact"
        );
    }

    /// A region is declared once, then again only when it actually moves.
    #[test]
    fn a_ui_rect_is_declared_once_per_layout_change() {
        use egui::{Pos2, Rect};
        let mut map = HashMap::new();
        let a = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(100.0, 20.0));
        let moved = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(100.0, 21.0));

        assert!(record_rect_if_changed(&mut map, "page", a));
        assert!(!record_rect_if_changed(&mut map, "page", a));
        assert!(
            record_rect_if_changed(&mut map, "page", moved),
            "a one-point resize moved the pixels a legibility check measures"
        );
        assert!(
            record_rect_if_changed(&mut map, "canvas-viewport", a),
            "regions are keyed by name; one must not suppress another"
        );
    }
}
