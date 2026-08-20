//! # `canvas::textedit` — **editing the page's own words**, and placing new ones
//!
//! `DEFECTS.md` **D4** is the defect that began this project, reported as:
//!
//! > *"text editing is weird and doesn't just edit the existing box and move the
//! > text correctly as you type plus flow to the next line doesn't work."*
//!
//! This module is the shell half of the answer. It arms a tool, puts a caret in
//! a run, collects keystrokes into a draft, and commits the draft as **one**
//! `EditSession` command. What it fixes that the old shell did not is in
//! [`disposition`] — the two cases D4b names as *wrong on commit* — and what it
//! deliberately does not fix is listed under **What is out of scope** below,
//! in words, because a silently-disabled typing loop is how the old shell told
//! an operator it could not do something.
//!
//! ---
//!
//! ## §1. The admission argument `canvas::tool`'s header asked for
//!
//! That header names one exclusion and one only:
//!
//! > **Text *editing*** — Phase 5, the defect that began this project — remains
//! > outside, and for exactly the original reason: it is a caret in a
//! > re-laid-out box, it would drag a whole subsystem's state through this type
//! > […] **Whoever brings the second should have to make this argument again,
//! > in this file.**
//!
//! It is made there, at [`CanvasTool::TextEdit`](crate::canvas::tool::CanvasTool::TextEdit),
//! and the short form is this: the objection was *"it would drag a whole
//! subsystem's state through this type"*, and the state does not go through the
//! type. What crosses the boundary is one [`TextEditKind`] — the same single
//! value `Markup` and `Measure` each carry — and the draft, the caret and the
//! anchor live in `egui::Memory` exactly where a half-finished measure pick
//! lives, for the reason `canvas::measure`'s header gives: *"a half-finished
//! pick is not part of the document and a document saved mid-gesture must not
//! carry one."* A half-typed word is the same category.
//!
//! ## §2. Two kinds, one tool — and why it is not two tools
//!
//! `edit.text` edits a run that is already on the page; `edit.add_text` places
//! new page content. They are the same *gesture* — click somewhere, type, press
//! Enter — differing only in what the click resolves to and which engine verb
//! the commit calls. So they are one [`CanvasTool`] variant carrying a
//! [`TextEditKind`], which is `MarkupKind`'s argument restated: the operator is
//! doing exactly one of the two, and a type that could say both would need
//! discipline to keep honest.
//!
//! ## §3. It is a CLICK tool, like measure and unlike markup
//!
//! [`press_kind`](crate::canvas::gesture::press_kind) answers
//! `drag: None, click: caps.edit_content` for this tool. There is no drag in
//! "put the caret here", so there is no `DragKind` for one and inventing a
//! `DragKind::TextEdit` that every arm ignored would be the placeholder this
//! project's no-placeholders invariant forbids — the identical argument the
//! measure and vertex-markup rungs make immediately above it.
//!
//! ## §4. It does not disturb the text-selection gate
//!
//! `canvas::textsel::gate`'s §3 warns that exclusivity between the text sweep
//! and the content marquee is now *by precedence*, not by construction. This
//! variant does not touch that: `takes_the_press` asks `tool.is_text()`, which
//! is `matches!(tool, CanvasTool::Text)` and is therefore **false** for
//! `TextEdit(_)` by construction rather than by an added condition. A press with
//! this tool armed is claimed by this module's own rung in `press_kind`, which
//! sits *above* the text-selection question for the same reason the measure rung
//! does — an armed tool takes the press — so the two can never both claim one
//! press. `gate.rs`'s tests carry a case asserting exactly that.
//!
//! ## §5. Capability: `edit_content`, and nothing wider
//!
//! Unlike text *selection*, this authors. Every entry point is gated on
//! `Capabilities::edit_content`, which is Edit alone in the shipped manifest —
//! the dispatch arms decline by name and trace, and
//! [`crate::canvas::tool::retire_forbidden`] disarms the tool on the way into a
//! mode that cannot author, so a draft cannot survive into Read.
//!
//! ---
//!
//! ## §6. What is out of scope, said in words rather than by a dead key
//!
//! `DEFECTS.md` D4a's **cross-run editing** is not built, and cannot be from
//! here: it needs a multi-run edit request in `pdfce-core` that does not exist —
//! `EditRequest` pins to one show operator, and *"a `TJ` array is one
//! operator"*. The old shell handled this by setting a `cross_run` flag that
//! **silently disabled the whole typing loop**, which is the failure this
//! module's [`Refusal::SpansRuns`] exists to avoid: a caret that lands where two
//! runs meet refuses **in a sentence**, on the status bar, naming what to do
//! instead. The sentence is `crate::text::textedit::spans_runs`.
//!
//! D4c's **reflow gates** are out of scope and untouched.
//!
//! ---
//!
//! ## §7. The two costs an operator pays, and where they are disclosed
//!
//! Rule 4 — *disclosure lives off-canvas* — so neither of these is drawn on the
//! page. Both reach the status bar through the disclosure list `vector_edit`
//! records, and one of them is a disclosure this shell adds because the engine
//! does not:
//!
//! * **`Reflow`** — the engine already discloses that the line may now overrun
//!   its margin.
//! * **`Pin`** — the engine discloses nothing, because from its side pinning is
//!   what was asked for. But a pinned tail does not make room, so a longer
//!   replacement grows *into* it. [`plan`] adds
//!   `crate::text::textedit::pinned_tail_disclosure` for exactly this, which is
//!   why the pin is never silent.

/// The caret's own arithmetic - insert, delete, and the four movements -
/// split out under R2 on 2026-08-20. Pure functions of a `&str` and an index,
/// with no window in them; its header says why that is a seam and not a cut.
pub mod caret;
pub use caret::{backspace, delete_forward, insert, word_left, word_right};
pub mod disposition;
// The byte-level proof that the untouched tail did not move, with the old
// shell's own `EditOptions::default()` run beside it as the falsifier.
// `#[cfg(test)]` inside; it compiles to nothing in a release build.
mod proof;
// The per-keystroke re-measure measurement `DEFECTS.md` D4b's fix would need,
// and the reason it is not wired. `#[ignore]`d; run it and read the numbers.
mod cost;
/// ★★ The face, size and colour NEW page text is written in — the Phase 5 row
/// that read *"choosing what those three controls are is a decision, not an
/// omission"*. The decision is in that module's header, along with why it lives
/// in `egui::Memory` where the markup pen does not.
pub mod pen;

use egui::{Pos2, Ui};
use pdfce_core::text_edit::{
    BlockRecognitionOptions, EditOptions, EditRequest, EditableTextModel, GlyphRef, ReflowEngine,
    TextPosition, reflow_recognition_options,
};

use crate::app::state::OpenDoc;
use crate::canvas::mapping::PageMapping;
use disposition::Reason;

/// `egui::Memory` key for the in-flight draft.
///
/// One key for both kinds, because one draft can be in flight: arming the other
/// kind clears it, exactly as arming a different `MarkupKind` cannot reach a
/// drag already in flight.
const DRAFT_MEMORY_KEY: &str = "pdfce-textedit-draft"; // ui-text-exempt: internal memory id, never displayed

/// The environment variable that supplies a draft when no keyboard can.
///
/// **A diagnostic seam in the shape `app::files`' two already have** — see
/// `DIAG_OPEN_PATH` and `DIAG_SAVE_PATH`, which exist because a native modal
/// cannot be driven from a harness. This one exists because **this machine
/// cannot inject text**: `tools/ui-verify`'s `sys::vk` is a deliberately closed
/// list of eight non-character virtual keys, and its own comment refuses to
/// grow into `pub const A..Z` on the ground that *"a harness that can press any
/// key is a harness whose scripts stop being readable"*.
///
/// Typing is this feature's entire input, so without a seam the only honest
/// verification would be *"the tool armed"* — which is `HANDOFF.md` §2's
/// grid lesson exactly: an assertion in the right direction that measures the
/// wrong thing.
///
/// **It is not load-bearing and it is not a second input path.** It is read at
/// exactly one place, [`typing`], on the frame a caret is set, and what it does
/// is push characters through **the same** [`insert`] every `egui::Event::Text`
/// goes through. A build with the variable unset cannot tell it exists; a build
/// with it set still has to route the click, resolve the anchor, plan the
/// disposition and reach the engine, which is every link the check is about.
pub const DIAG_TYPE: &str = "PDFCE_DIAG_TYPE"; // ui-text-exempt: an environment variable name, never displayed

/// **Which of the two text verbs is armed.**
///
/// One value carried on the tool, for [`MarkupKind`](crate::canvas::markup::MarkupKind)'s
/// argument: the operator is doing exactly one of these, so a type that could
/// express both would have illegal states to prevent by discipline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextEditKind {
    /// `edit.text` — replace the words in a run that is already on the page.
    Edit,
    /// `edit.add_text` — place new page content where the operator clicks.
    Add,
}

impl TextEditKind {
    /// The command id that arms this kind.
    ///
    /// The single binding between an id and a kind, in the shape
    /// `shell::commands::markup_for_command` has — read from both directions by
    /// `crate::shell::commands::text_edit_for_command` and by the label the
    /// status bar shows, so the two cannot drift.
    #[must_use]
    pub const fn command_id(self) -> &'static str {
        match self {
            // ui-text-exempt: command ids, never displayed.
            Self::Edit => "edit.text",
            Self::Add => "edit.add_text",
        }
    }
}

/// **What the caret is attached to** — the half of the draft that the click
/// resolves and typing never changes.
#[derive(Debug, Clone, PartialEq)]
pub enum Anchor {
    /// A run already on the page, by index into `PageText::runs`, with the text
    /// it held when the caret landed.
    ///
    /// The *original* is carried and the geometry is not, deliberately. The
    /// original is what `EditRequest::find` needs and is a fact about the moment
    /// the operator clicked; everything else — the pinned span, the matrices,
    /// the block alignment — is a pure function of `(page_text, run)` that
    /// [`plan`] re-derives at commit. That is the old shell's own ruling, and
    /// its reason is worth keeping: *"storing a copy on `PendingEdit` would be a
    /// second source of truth that can go stale when the page is rebuilt."*
    Run { run: usize, original: String },
    /// A point in **PDF user space** where new text will be placed.
    Origin { x: f64, y: f64 },
}

/// An in-progress, operator-composed edit. Never written anywhere until commit.
#[derive(Debug, Clone, PartialEq)]
pub struct Draft {
    /// Which page it belongs to. A page change abandons it — see [`load`].
    pub page: usize,
    /// Which verb will commit it.
    pub kind: TextEditKind,
    /// What the caret is on.
    pub anchor: Anchor,
    /// The operator's in-progress text.
    pub text: String,
    /// ★★ **Where the caret sits inside [`Self::text`], as a CHARACTER index.**
    ///
    /// `0` is before the first character; `text.chars().count()` is after the
    /// last. Every edit and every movement clamps into that range, so the
    /// invariant `caret <= text.chars().count()` holds by construction and no
    /// caller has to check it.
    ///
    /// # The defect this field is
    ///
    /// It did not exist until 2026-08-20. `insert` extended the end of the
    /// string and `backspace` popped the last character, so the caret was not
    /// merely fixed at the end - **there was no caret**, and the painter drew
    /// its line at the right edge of the run's glyph box because that is the
    /// only position an append-only draft has. The operator:
    ///
    /// > *"the cursor just sits at the end of a text line. It can't be moved to
    /// > the center of an existing text block."*
    ///
    /// Exactly right, and it made editing existing page text almost useless: a
    /// title-block cell reading `SHEET 1 OF 4` could only be changed by deleting
    /// it back to `SHEET ` and retyping.
    ///
    /// # Why characters and not bytes
    ///
    /// Because every operation here is expressed in keystrokes, and one
    /// keystroke is one `char`. A byte index would make Left-arrow over `e` -
    /// two bytes - either move half a character or need a decode at every use.
    /// `backspace` already worked in `char`s for the same reason (a byte
    /// truncation of a multi-byte character is a panic in Rust, not mojibake),
    /// so this is that decision applied consistently rather than a new one.
    ///
    /// The cost is that every operation is O(n) in the draft's length. A draft
    /// is one show operator - a cell, a label, a line of a note - so n is tens
    /// of characters, and the alternative is a byte index plus a boundary check
    /// at every call site.
    pub caret: usize,
    /// Whether the diagnostic seam has already been consumed for this draft, so
    /// a seam-supplied string is typed **once** rather than on every frame.
    ///
    /// `pub` only so a test in another module can build a draft that is already
    /// past the seam. Nothing outside this module should ever set it to `false`.
    pub seeded: bool,
}

/// Why a click could not start a draft, in a form the status bar can render.
///
/// Every variant is a *sentence to show*, not a state to be silent in. That is
/// the whole difference from the old shell, which set a boolean and stopped
/// responding to the keyboard.
///
/// ★★ **`SpansRuns` was the third variant and is gone as of 2026-08-19.** It
/// refused every click whose visual line was made of more than one show
/// operator, which on a CAD sheet is nearly every click — see [`resolve_run`]
/// for the measurement and for why the refusal was answering a question about
/// the *line* when the operator was editing a *run*. The two that are left are
/// both genuine absences of a thing to edit: no text under the pointer, and no
/// readable text on the page at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// The click landed on no text at all.
    NoRun,
    /// The page's text could not be extracted (an image-only page, a damaged
    /// content stream).
    NoText,
    /// ★★ **The run is real, readable, and inside a form XObject — which
    /// `pdfce-core`'s text-edit surgery does not enter.** 2026-08-20.
    ///
    /// # Why this variant had to exist, and what it replaced
    ///
    /// Nothing. The click was accepted, a caret was placed, keystrokes were
    /// taken, a plan was built, and the engine then refused the commit with
    /// *"text to edit (\"p\") was not found in an editable run on the page"* —
    /// **to the trace only**. The operator saw a caret that took their typing
    /// and threw it away in silence. Their words:
    ///
    /// > *"Still no editing text on top of the canvas."*
    ///
    /// # The mechanism, because it is not obvious from either side
    ///
    /// `GlyphProvenance` carries two fields and the shell was reading one:
    /// `operator_span` is a **byte span into a decoded content buffer**, and
    /// `content_stream` names *which* buffer. For text drawn by the page's own
    /// stream the two agree with what the edit surgery walks. For text inside a
    /// `Do`-invoked form XObject the span indexes the FORM's bytes, and
    /// `pdfce-core`'s `find_anchor` compares it against page-stream offsets —
    /// where it can never match. Worse, a pinned request skips the text search
    /// entirely, so the loop exhausts and reports `NoMatch(find)`, which blames
    /// the operator's text for a failure of the pin.
    ///
    /// # Why refusing is better than trying
    ///
    /// Because the attempt **cannot** succeed — this is a named non-goal of
    /// that cut of the engine (`pdfce-core/src/text_edit/edit.rs:79`) — and a
    /// control that accepts input it will discard is this project's defining
    /// defect class. An honest refusal at the click costs the operator one
    /// click; the silent version cost them a sentence they had already typed
    /// and the belief that the feature works.
    ///
    /// ★ **On a CAD sheet this is the common case, not an edge case.** Measured
    /// on the benchmark drawing: 1,696 show operators of real drawing text
    /// inside the form, against 3,007 metadata glyphs in the page stream. Filed
    /// as an engine request the same day.
    InsideForm,
}

/// Read the draft without creating one. `None` when nothing is being composed.
#[must_use]
pub fn read(ctx: &egui::Context) -> Option<Draft> {
    ctx.data(|d| d.get_temp::<Draft>(egui::Id::new(DRAFT_MEMORY_KEY)))
}

/// Store a draft.
pub(crate) fn store(ctx: &egui::Context, draft: Draft) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(DRAFT_MEMORY_KEY), draft));
}

/// Forget the draft, returning whether there was one.
///
/// The abandon half of the Escape ladder, and the retirement path
/// `crate::canvas::tool::retire_forbidden` calls when the mode loses
/// `edit_content`. Returns `bool` for the reason `measure::abandon` does: the
/// ladder rung above it needs to know whether this rung consumed the key.
pub fn abandon(ctx: &egui::Context) -> bool {
    let had = read(ctx).is_some();
    ctx.data_mut(|d| d.remove::<Draft>(egui::Id::new(DRAFT_MEMORY_KEY)));
    if had {
        // ui-text-exempt: diagnostic trace, never displayed.
        crate::diag::trace(|| "text-edit-abandon".to_owned());
    }
    had
}

/// The draft for `page`, or `None` — dropping any draft that belongs to another
/// page or another kind.
///
/// The two synchronisations `measure::load` performs, for the same two reasons.
/// A draft composed on page 3 must not commit against page 4's run indices, and
/// a draft begun under `Edit` must not be committed by `Add` because the
/// operator pressed the other ribbon button mid-word.
#[must_use]
pub fn load(ctx: &egui::Context, page: usize, kind: TextEditKind) -> Option<Draft> {
    let draft = read(ctx)?;
    if draft.page == page && draft.kind == kind {
        return Some(draft);
    }
    abandon(ctx);
    None
}

// ===========================================================================
// Starting a draft
// ===========================================================================

/// Everything resolving a click needs, gathered by the caller so this module
/// reads no globals.
pub struct Click<'a> {
    /// The document, for its page-text extraction.
    pub doc: &'a OpenDoc,
    /// Which page was clicked.
    pub page_index: usize,
    /// Which verb is armed.
    pub kind: TextEditKind,
    /// Where, in canvas space.
    pub canvas_point: Pos2,
}

/// **Start (or move) a draft from a click.**
///
/// Returns the refusal to show, if the click could not begin one. `Ok(())` means
/// a draft is now in flight and the next keystroke will reach it.
///
/// # Why an existing draft is committed rather than discarded
///
/// Clicking elsewhere while composing is the operator saying *"that word is
/// finished"*, not *"throw it away"* — every editor behaves this way, and the
/// old shell settled it under the name `commit_on_click`. So the caller is
/// handed the commit as an [`crate::app::actions::Action`] before the new draft
/// starts. Escape, and only Escape, discards.
pub fn click(
    ctx: &egui::Context,
    click: &Click<'_>,
    actions: &mut Vec<crate::app::actions::Action>,
) -> Result<(), Refusal> {
    if let Some(existing) = read(ctx) {
        commit_into(ctx, &existing, actions);
        abandon(ctx);
    }
    let anchor = match click.kind {
        TextEditKind::Add => {
            let page = click
                .doc
                .pages
                .get(click.page_index)
                .ok_or(Refusal::NoText)?;
            let pdf = crate::viewer::canvas_to_pdf_space(click.canvas_point, page)
                .ok_or(Refusal::NoText)?;
            Anchor::Origin {
                x: f64::from(pdf.x),
                y: f64::from(pdf.y),
            }
        }
        // ★★ **A click that names no run starts a new one**, as of 2026-08-19.
        //
        // This used to be `resolve_run(click)?` — a bare `?`, so a click on
        // blank paper with the caret armed refused, wrote a sentence to the
        // status row, and did nothing. Two separate tools were needed to type a
        // character in an empty spot versus in an existing word, and which one
        // you had was invisible.
        //
        // The operator, 2026-08-19:
        //
        // > *"How do I make new text when I click on the canvas and expect to
        // > edit there? Same problem as the previous."*
        //
        // Every editor he has used does this with **one** text tool: click in
        // text to edit it, click in space to start some. So a `NoRun` refusal
        // becomes an origin at the click point, and the two ribbon commands
        // (`edit.text`, `edit.add_text`) survive as two doors into one room.
        //
        // ★ Only `NoRun` falls through. Every other refusal — an encrypted
        // document, a page that will not decompose, a run the engine cannot
        // address — is still reported, because those say *this cannot be done
        // here* rather than *there is nothing here*. Swallowing them would put
        // a caret on a page that cannot take the edit, which is D4a's defect
        // with a nicer opening move.
        TextEditKind::Edit => match resolve_run(click) {
            Ok(anchor) => anchor,
            Err(Refusal::NoRun) => {
                let page = click
                    .doc
                    .pages
                    .get(click.page_index)
                    .ok_or(Refusal::NoText)?;
                let pdf = crate::viewer::canvas_to_pdf_space(click.canvas_point, page)
                    .ok_or(Refusal::NoText)?;
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    "text-edit-became-add reason=no-run-under-the-click".to_owned()
                });
                Anchor::Origin {
                    x: f64::from(pdf.x),
                    y: f64::from(pdf.y),
                }
            }
            Err(other) => return Err(other),
        },
    };
    let text = match &anchor {
        Anchor::Run { original, .. } => original.clone(),
        Anchor::Origin { .. } => String::new(),
    };
    // ★★ The caret lands WHERE THE CLICK LANDED, not at the end.
    //
    // `caret_index_at` measures the click's x against the run's own glyph
    // advances - the same glyph boxes the caret is drawn from - so clicking
    // between the `1` and the ` ` of `SHEET 1 OF 4` puts the caret between
    // them. Before 2026-08-20 the draft had no caret index at all, so a click
    // anywhere in a run behaved as a click at its end.
    //
    // Falls back to the end of the text, which is the old behaviour, when the
    // run's glyphs cannot be read. That is the right fallback rather than the
    // start: appending is the less destructive of the two if the operator
    // types without looking.
    let caret = match &anchor {
        Anchor::Run { run, .. } => {
            caret_index_at(click, *run).unwrap_or_else(|| text.chars().count())
        }
        Anchor::Origin { .. } => 0,
    };
    store(
        ctx,
        Draft {
            page: click.page_index,
            kind: click.kind,
            anchor,
            text,
            caret,
            seeded: false,
        },
    );
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        //
        // The whole reason this line exists: an armed text tool with a caret in
        // it and an armed text tool without one are the same screenshot, and the
        // caret blinks so even a captured frame cannot settle it. `DEFECTS.md`
        // D14's lesson — the freehand trail that authored two points — is that a
        // trace line must carry the number a wrong build would get wrong, so this
        // names the run and the length rather than only saying a caret exists.
        let d = read(ctx);
        let (anchor, len) = d.as_ref().map_or((String::new(), 0), |d| {
            (
                match &d.anchor {
                    Anchor::Run { run, .. } => format!("run={run}"),
                    Anchor::Origin { x, y } => format!("origin={x:.1},{y:.1}"),
                },
                d.text.chars().count(),
            )
        });
        format!(
            "text-edit-caret kind={:?} page={} {anchor} len={len}",
            click.kind, click.page_index
        )
    });
    Ok(())
}

/// **Is `run` drawn from inside a form XObject?** `Some(true)` / `Some(false)`,
/// or `None` when the question could not be asked.
///
/// A thin forward to [`crate::app::state::OpenDoc::run_is_inside_a_form`],
/// which owns the extraction and the cache. It is worth a named function here
/// anyway: this is the one place in the shell that encodes *"what pdfce-core
/// can and cannot edit"*, and the day the engine gains form editing this is the
/// function to delete rather than a condition to find.
///
/// # Why the answer is cached one level down and not here
///
/// Because the only way to ask is a **second extraction of the whole page with
/// provenance on** - `PageTextCache` deliberately leaves provenance off, since
/// the canvas, the find bar and the text sweep do not need it and it costs.
/// Measured on the benchmark CAD sheet: **336 ms**. Doing it inline froze the
/// UI for a third of a second on every click that landed on text, and made a
/// driven check flake because the trace had not been written by the time the
/// settle window closed - a performance defect that presented as harness
/// flakiness, which this project has been caught by before.
///
/// `None` means **not measured**, never "yes". See `FormRunCache::flags`.
fn inside_a_form(c: &Click<'_>, run: usize) -> Option<bool> {
    c.doc.run_is_inside_a_form(run)
}

/// **Which character boundary a click landed on, inside `run`.**
///
/// Returns a character index in `0..=glyphs.len()`, or `None` when the run or
/// the page cannot be read.
///
/// # How it decides
///
/// Every glyph `pdfce-core` publishes carries its origin `x` and its `advance`,
/// so a run's character boundaries are `x[0]`, `x[0]+adv[0]`, `x[1]+adv[1]`, …
/// The click's x is compared against each glyph's MIDPOINT: past the midpoint
/// means the caret belongs after that glyph. That is the rule every text field
/// uses and it is what makes clicking "on" a character feel like clicking
/// *near* the boundary the operator was aiming at, rather than requiring them
/// to hit a one-pixel gap.
///
/// # Why the x axis alone
///
/// Because a run is one show operator, which is one baseline. The vertical
/// question - *which line?* - was already answered by `resolve_run`'s hit test
/// before this is called, and asking it again here with different arithmetic is
/// how a caret comes to land on a different line from the one that was clicked.
///
/// ★ Rotated runs. The comparison is done in **PDF user space** against the
/// glyph origins as published, which is the same space `resolve_run` works in.
/// For a run rotated off the horizontal this compares the wrong axis and the
/// caret will land at a boundary the operator did not aim at - it is still
/// inside the run, and still better than always landing at the end, but it is
/// not right. Fixing it properly means projecting the click onto the run's own
/// baseline direction, which needs the text matrix rather than the glyph
/// boxes. Recorded here rather than silently approximated.
fn caret_index_at(c: &Click<'_>, run: usize) -> Option<usize> {
    let page = c.doc.pages.get(c.page_index)?;
    let pdf = crate::viewer::canvas_to_pdf_space(c.canvas_point, page)?;
    let text = c.doc.page_text()?;
    let glyphs = &text.runs.get(run)?.glyphs;
    if glyphs.is_empty() {
        return None;
    }
    let x = pdf.x;
    let mut index = 0;
    for g in glyphs {
        if x < g.x + g.advance / 2.0 {
            return Some(index);
        }
        index += 1;
    }
    Some(index)
}

/// Resolve a click on existing page text to the run it landed in.
///
/// Two hops, and the first is the one `canvas::mapping`'s header calls *the
/// classic silent defect*: the canvas is Y-down from the page's top-left with
/// `/Rotate` applied, and every glyph position `pdfce-core` publishes is in PDF
/// user space — Y-up from the un-rotated CropBox. `viewer::canvas_to_pdf_space`
/// is the single bridge, and it works by inverting the **renderer's own**
/// transform, so the geometry and the picture agree by construction. This is
/// deliberately the identical route `canvas::textsel::hit` takes, because a
/// second conversion here is how a caret comes to land on a different line from
/// the highlight.
fn resolve_run(c: &Click<'_>) -> Result<Anchor, Refusal> {
    let page = c.doc.pages.get(c.page_index).ok_or(Refusal::NoText)?;
    let pdf = crate::viewer::canvas_to_pdf_space(c.canvas_point, page).ok_or(Refusal::NoText)?;
    let text = c.doc.page_text().ok_or(Refusal::NoText)?;
    let model = EditableTextModel::recognize(&text, &BlockRecognitionOptions::default());
    let pos = model
        .hit_test(f64::from(pdf.x), f64::from(pdf.y))
        .ok_or(Refusal::NoRun)?;
    // ★★★ **D4a's boundary used to REFUSE here, and refusing was the defect.**
    //
    // The old code read: *if the caret's visual line begins and ends in
    // different runs, `return Err(Refusal::SpansRuns)`* — on the argument that
    // "the thing the operator is looking at is not a thing `pdfce-core` can be
    // asked to replace".
    //
    // **That argument was about the LINE. The operator is editing a RUN.**
    // `EditSession::edit_text` replaces one show operator, which is exactly what
    // a click resolves; whether the *neighbours* on that line are separate
    // operators is a fact about what happens to **them**, not about whether this
    // edit is possible. The engine has answered the neighbour question since
    // before this shell existed — `FollowerDisposition::Pin` leaves every
    // follower `Tm` untouched — and this module has been *choosing* between the
    // two dispositions on every commit for days.
    //
    // ★ How expensive the refusal was, measured rather than guessed. A
    // SolidWorks-exported drawing sheet writes text as one show operator per
    // *cell*, so nearly every visual line on a title block or a parts table is
    // multi-run. `tools/ui-verify`'s `text_edit_on_a_real_drawing` armed the
    // caret on `SW41177.pdf` and clicked a point `pdfce-cli find-text` reported
    // as carrying the word `PART`: **`text-edit-declined reason=SpansRuns`**.
    //
    // So on this operator's own documents the feature refused essentially every
    // click, and his report — *"text editing on canvas still doesn't work"*,
    // twice, weeks apart — was **exactly accurate**. Two passing driven checks
    // said otherwise, and both drove fixtures this repository generated to
    // verify itself: a 924-byte three-line file and blank paper.
    //
    // What the refusal was right about is kept, and it is the *disclosure*: the
    // operator is editing one piece of something that looks like one line, and
    // the other pieces will not move. That is now said — before the commit and
    // after it — rather than used as a reason to do nothing. See
    // [`disposition::Reason::SharesTheLine`], and `crate::text::textedit`.
    //
    // The line is re-derived at commit rather than carried on the `Anchor`, for
    // the reason the `Anchor` docs already give: everything but the original
    // text is a pure function of `(page_text, run)` and a copy would go stale
    // when the page is rebuilt.
    let original = text
        .runs
        .get(pos.run)
        .map(|r| r.text.clone())
        .ok_or(Refusal::NoRun)?;
    if original.is_empty() {
        return Err(Refusal::NoRun);
    }
    // ★★ **Announced BEFORE the edit, not after it.**
    //
    // `MeasureState`'s derived-point rule in the operator's own vocabulary:
    // *a derived point is pdfce's inference, so rule 4 requires it to be
    // announced before it is picked, not after.* The same is true of a layout
    // consequence — the operator is about to type into what looks like one line,
    // and the pieces beside it will not move. Telling them at commit time is
    // telling them after they have already chosen.
    //
    // It is a **note**, not a refusal: the caret is placed either way, and this
    // returns `Ok`. `crate::text::textedit::pinned_tail_disclosure` says the
    // same fact in the past tense when the edit lands, and both are wanted —
    // one is a warning and one is a receipt.
    if model
        .line_range_at(pos)
        .is_some_and(|(from, to)| from.run != to.run)
    {
        crate::app::actions::record_note(
            c.doc.edit_epoch,
            crate::text::textedit::shares_the_line_note().to_owned(),
        );
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            //
            // ★ Named so a driven check can tell the two shapes apart. Until
            // 2026-08-19 this case emitted `text-edit-declined reason=SpansRuns`
            // and placed no caret; it now emits this and a caret, and a harness
            // that could not distinguish "refused" from "allowed and disclosed"
            // would pass against either.
            format!("text-edit-shares-line run={}", pos.run)
        });
    }
    // ★★★ **THE EDITABILITY CHECK, and it is the last thing before the caret.**
    //
    // Added 2026-08-20 on the operator's *"Still no editing text on top of the
    // canvas."* Every stage of this module worked; the commit reached
    // `pdfce-core` and was refused, **to the trace only**, so a caret took his
    // keystrokes and discarded them in silence.
    //
    // The cause is one field this shell was not reading. `GlyphProvenance`
    // carries a byte span AND the name of the buffer that span indexes:
    //
    // ```text
    // pub content_stream: ContentStreamRef,   // Page, or Form { object }
    // pub operator_span:  ByteSpan,           // …within THAT buffer
    // ```
    //
    // `commit` pins the request with `operator_span` alone. For page-stream text
    // that is right. For text drawn inside a `Do`-invoked form XObject the span
    // indexes the FORM's decoded bytes, while the engine's `find_anchor` walks
    // the PAGE's — so the pin matches nothing, and because a pinned request
    // skips the text search entirely, the loop exhausts and returns
    // `NoMatch(find)`. That error names the operator's text, which is why the
    // sentence reads *"text to edit ("p") was not found in an editable run"*
    // about text that is plainly there.
    //
    // ★ Refusing rather than attempting, because the attempt CANNOT succeed:
    // form-XObject content is a named non-goal of that cut of the engine
    // (`pdfce-core/src/text_edit/edit.rs:79`). Placing a caret that will eat a
    // sentence is this project's defining defect class, and one honest click of
    // refusal is cheaper than a paragraph typed into nothing.
    //
    // ★★ On a CAD sheet this is the COMMON case. Measured on the benchmark
    // drawing: 1,696 show operators of real drawing text inside the form,
    // against 3,007 metadata glyphs in the page's own stream. So this refusal
    // will fire often, and that is the honest picture of what the engine can do
    // today rather than a pessimistic guard.
    //
    // The provenance is only populated when the extraction asked for it, which
    // `app::cache`'s read-only pass deliberately does not — so `None` here means
    // *"not measured"*, not *"page stream"*, and the caret is allowed. Refusing
    // on an unmeasured answer would block editing everywhere on a guess.
    if inside_a_form(c, pos.run) == Some(true) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("text-edit-declined reason=InsideForm run={}", pos.run)
        });
        return Err(Refusal::InsideForm);
    }

    Ok(Anchor::Run {
        run: pos.run,
        original,
    })
}

// ===========================================================================
// Typing
// ===========================================================================

/// **Consume this frame's keystrokes into the draft.**
///
/// Returns `true` when the draft was committed by Enter, so the caller knows the
/// caret is gone.
///
/// # Why the events are read raw rather than through a `TextEdit` widget
///
/// Because the caret is painted in PDF space, on the page, at the glyphs' own
/// scale — which is what *"just edit the existing box"* means. An `egui`
/// `TextEdit` would be a second box floating over the first, and the old shell's
/// one virtue here is worth keeping: it had a real caret in the page, and no
/// widget in the typing path.
pub fn typing(
    ui: &Ui,
    ctx: &egui::Context,
    focused: bool,
    actions: &mut Vec<crate::app::actions::Action>,
) -> bool {
    let Some(mut draft) = read(ctx) else {
        return false;
    };
    let mut changed = false;
    // The diagnostic seam, consumed exactly once per draft. See [`DIAG_TYPE`].
    if !draft.seeded {
        draft.seeded = true;
        changed = true;
        if let Ok(seed) = std::env::var(DIAG_TYPE)
            && !seed.is_empty()
        {
            draft.text.clear();
            draft.caret = insert(&mut draft.text, 0, &seed);
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!("text-edit-seeded len={}", draft.text.chars().count())
            });
        }
    }
    if focused {
        for ev in ui.input(|i| i.events.clone()) {
            match ev {
                egui::Event::Text(t) if !t.is_empty() => {
                    draft.caret = insert(&mut draft.text, draft.caret, &t);
                    changed = true;
                }
                egui::Event::Key {
                    key: egui::Key::Backspace,
                    pressed: true,
                    ..
                } => {
                    draft.caret = backspace(&mut draft.text, draft.caret);
                    changed = true;
                }
                egui::Event::Key {
                    key: egui::Key::Delete,
                    pressed: true,
                    ..
                } => {
                    draft.caret = delete_forward(&mut draft.text, draft.caret);
                    changed = true;
                }
                // ★★ **Caret movement**, 2026-08-20, on the operator's report
                // that *"the cursor just sits at the end of a text line. It
                // can't be moved to the center of an existing text block."*
                //
                // These five arms are what makes the caret a caret. Before
                // them the draft had no position at all: text was appended and
                // Backspace popped, so changing `SHEET 1 OF 4` to `SHEET 2 OF
                // 4` meant deleting back to `SHEET ` and retyping the rest.
                //
                // ★ `changed` is set for a pure movement, and that is
                // deliberate rather than sloppy. It is the flag that decides
                // whether the draft is written back to `egui::Memory`, and a
                // moved caret IS a changed draft - without this the arrow keys
                // would appear to work for one frame and then snap back on the
                // next load. It does NOT put anything on the undo stack:
                // `commit_into` compares the TEXT with the original, so a draft
                // whose caret moved and whose characters did not still pushes
                // no action.
                egui::Event::Key {
                    key: egui::Key::ArrowLeft,
                    pressed: true,
                    modifiers,
                    ..
                } => {
                    draft.caret = if modifiers.command {
                        word_left(&draft.text, draft.caret)
                    } else {
                        draft.caret.saturating_sub(1)
                    };
                    changed = true;
                }
                egui::Event::Key {
                    key: egui::Key::ArrowRight,
                    pressed: true,
                    modifiers,
                    ..
                } => {
                    let end = draft.text.chars().count();
                    draft.caret = if modifiers.command {
                        word_right(&draft.text, draft.caret)
                    } else {
                        (draft.caret + 1).min(end)
                    };
                    changed = true;
                }
                egui::Event::Key {
                    key: egui::Key::Home,
                    pressed: true,
                    ..
                } => {
                    draft.caret = 0;
                    changed = true;
                }
                egui::Event::Key {
                    key: egui::Key::End,
                    pressed: true,
                    ..
                } => {
                    draft.caret = draft.text.chars().count();
                    changed = true;
                }
                // ★ Enter commits, and the condition is the Accept control's
                // own rather than a second one — the same R92 objection to
                // deriving one fact twice that the old shell recorded when it
                // wired this key.
                egui::Event::Key {
                    key: egui::Key::Enter,
                    pressed: true,
                    ..
                } => {
                    commit_into(ctx, &draft, actions);
                    abandon(ctx);
                    return true;
                }
                _ => {}
            }
        }
    }
    if changed {
        store(ctx, draft);
    }
    false
}

/// Turn a draft into the action that commits it, if it says anything.
///
/// A draft byte-identical to what it replaces is **not a write**. Without this,
/// an operator who typed a character and deleted it again would put a no-op
/// entry on the undo stack every time they clicked away — the old shell's own
/// finding, and it matters more here because clicking out commits.
fn commit_into(ctx: &egui::Context, draft: &Draft, actions: &mut Vec<crate::app::actions::Action>) {
    use crate::app::actions::Action;
    match &draft.anchor {
        Anchor::Run { run, original } if draft.text != *original && !draft.text.is_empty() => {
            actions.push(Action::CommitTextEdit {
                page: draft.page,
                run: *run,
                original: original.clone(),
                replacement: draft.text.clone(),
            });
        }
        Anchor::Origin { x, y } if !draft.text.is_empty() => {
            actions.push(Action::CommitAddText {
                page: draft.page,
                origin: (*x, *y),
                text: draft.text.clone(),
                // ★ Sampled HERE, at the commit, not read in `apply`. See the
                // variant's own docs: an action is what the operator asked for,
                // and it is applied on a later frame.
                pen: pen::read(ctx),
            });
        }
        _ => {}
    }
}

// ===========================================================================
// Planning the commit — where D4b's two fixes actually take effect
// ===========================================================================

/// A planned in-place edit: the request, the options, and the disclosure the
/// engine will not write for us.
pub struct Plan {
    /// The request, with its provenance pin.
    pub request: EditRequest,
    /// ★ The options, with the [`disposition`] this module exists to choose.
    pub options: EditOptions,
    /// Why that disposition, for the trace and the disclosure.
    pub reason: Reason,
}

/// **Plan a commit against the page as it is now.**
///
/// Called from the apply arm rather than from the canvas, because it needs the
/// document and an `Action` is plain data. It is still one function in one place
/// — the arm routes to it and computes nothing itself.
///
/// The three things it derives, all from `(page_text, run)`:
///
/// 1. **the provenance pin** — `operator_span`, which is how the surgery finds
///    *this* show operator rather than the first one whose text matches. Without
///    it, editing the second `TITLE` on a title-block sheet edits the first.
/// 2. **the matrices** — `Tm` and the CTM in force at the run's first glyph,
///    which is what [`disposition::is_upright`] reads.
/// 3. **the block alignment** — through `ReflowEngine::detect_alignment` on a
///    model recognised with [`reflow_recognition_options`], i.e. the **relaxed**
///    recogniser. That is the old shell's own choice for its reflow target and
///    the reason carries here unchanged: the default recogniser splits on
///    indentation, so a right-aligned block whose lines start at different x —
///    which is what right alignment *is* — is exactly the shape it fragments,
///    and a fragmented block is a one-line block, and a one-line block reports
///    `SingleLineDefault`. Using the default model would make the alignment
///    fix unreachable on precisely the documents it is for.
#[must_use]
pub fn plan(doc: &OpenDoc, page: usize, run: usize, original: &str, replacement: &str) -> Plan {
    let mut request = EditRequest::find_replace(page, original, replacement);
    let mut matrices = (
        [1.0_f32, 0.0, 0.0, 1.0, 0.0, 0.0],
        [1.0_f32, 0.0, 0.0, 1.0, 0.0, 0.0],
    );
    let mut finding = None;
    // ★★ Whether the caret's visual line is made of more than one show
    // operator, re-derived here rather than carried on the `Anchor`.
    //
    // The `Anchor` docs give the rule and it applies unchanged: everything but
    // the original text is a pure function of `(page_text, run)`, and a copy
    // taken when the operator clicked would go stale when the page is rebuilt.
    //
    // Defaults to `false`, which is the *permissive* direction — `Reflow` — and
    // that is the honest default for the same reason the identity matrices below
    // are: it is what a page whose provenance could not be read gets, and a
    // shell that pinned on no evidence would be claiming to have measured
    // something it never saw. The single-run case is also the overwhelmingly
    // commoner one in ordinary prose documents.
    let mut shares_the_line = false;

    // ★★ **This extraction is its own, and it is NOT `doc.page_text()`.**
    //
    // That was the first shape of this function and it was silently broken.
    // `app::cache`'s extraction runs with `ExtractOptions::default()`, and
    // `capture_provenance` **defaults to off** — the engine says so in terms:
    // *"`None` unless the extraction set `ExtractOptions::capture_provenance`;
    // this keeps the default Pass 4 output byte-for-byte unchanged."* With it
    // off, `model.provenance(..)` answers `None` for every glyph, and this
    // function would have:
    //
    // * left `pinned_span` at `None`, so the surgery would locate the **first**
    //   operator whose text matches rather than the one the caret is in — which
    //   on a title-block sheet with two runs reading `REV A` edits the wrong
    //   one; and
    // * fallen back to the identity matrices below, so **the rotation guard
    //   would never fire** and D4b case 2 would be unfixed while every unit test
    //   in `disposition` stayed green. That is precisely `HANDOFF.md` §2's
    //   shape: a correct decision function, wired to a value that is always the
    //   same.
    //
    // Widening the shared cache was the other option and is the worse one: every
    // caller of `page_text()` — Find, both copy verbs, the text sweep — would
    // then pay for provenance on every page, and `app::cache`'s own header
    // records that extraction is the expensive thing this shell does (392 ms on
    // the benchmark sheet). Paying it **once per commit**, here, is the whole
    // cost, and a commit is already an operation that saves and re-rasters.
    //
    // The run index is shared between the two extractions, which is safe and is
    // worth stating: `capture_provenance` populates a field and changes no
    // segmentation, so `runs[i]` names the same run under both options.
    if let Some(page_ref) = doc.pages.get(page) {
        // ★ The funnel's output, MODIFIED — not a second construction.
        //
        // `with_provenance(true)` is the one thing no setting governs: it is the
        // substrate for editing text, and `app::cache`'s read-only extraction
        // deliberately leaves it off because it costs and it is not needed
        // there. Everything else — the word gap, the unmappable sentinel, the
        // replacement-text precedence — comes from the operator, so the runs
        // this editor addresses are segmented exactly as the runs the canvas
        // paints and the find bar searches. Two extractions of one page under
        // two configurations would put the glyph the operator clicked and the
        // glyph this code edits one step out of step.
        use crate::app::settings::SettingsExt;
        let opts = doc.settings.extract_options().with_provenance(true);
        if let Ok(text) =
            pdfce_core::text_extract::extract_page_view(&doc.session.view(), page_ref, page, &opts)
        {
            let model = EditableTextModel::recognize(&text, &BlockRecognitionOptions::default());
            let gref = GlyphRef::new(run, 0);
            if let Some(p) = model.provenance(gref) {
                request.pinned_span = Some(p.operator_span);
                matrices = (p.text_matrix, p.ctm);
            }
            // ★ The SAME model the caret's hit test used, with the same
            // options — `BlockRecognitionOptions::default()` — because the
            // question is *how did the thing the operator clicked get
            // segmented*, and asking it of a differently-recognised model would
            // answer about a different segmentation. The relaxed model below is
            // for alignment detection, which is a different question about the
            // same page.
            if let Some((from, to)) = model.line_range_at(TextPosition::new(run, 0)) {
                shares_the_line = from.run != to.run;
            }
            let relaxed = EditableTextModel::recognize(&text, &reflow_recognition_options());
            finding = relaxed
                .block_at(TextPosition::new(run, 0))
                .and_then(|b| ReflowEngine::new(&relaxed).detect_alignment(b).ok())
                .map(disposition::from_detection);
        }
    }

    let reason = disposition::choose(matrices.0, matrices.1, shares_the_line, finding);
    Plan {
        request,
        options: disposition::options(reason),
        reason,
    }
}

// ===========================================================================
// Painting
// ===========================================================================

/// What [`preview`] needs.
pub struct Preview<'a> {
    /// The document, for the page geometry.
    pub doc: &'a OpenDoc,
    /// Which page is on screen.
    pub page_index: usize,
    /// The frame's screen ⟷ canvas mapping.
    pub map: &'a PageMapping,
}

/// **Draw the caret and the draft's extent.**
///
/// ## ★ D4a's ghost text, and why this build draws no glyphs at all
///
/// The old shell drew the draft *as text*, in an `egui` proportional font, over
/// a translucent mask — which `DEFECTS.md` D4a names as the second contributor
/// to "weird": *"you type in the wrong typeface at the wrong widths, then it
/// snaps to reality on Accept."*
///
/// The tempting fix is a better ghost. This module draws **no ghost**, and that
/// is a decision rather than an omission:
///
/// * A ghost in the wrong face is a **lie about the document**, and it is the
///   precise lie `HANDOFF.md`'s Rule 4 one-line test catches — *would a
///   screenshot of the editing canvas differ from a screenshot of the same
///   document saved and reopened?* Under the old shell it differed in the one
///   respect the operator was looking at.
/// * A ghost in the *right* face would need this shell to rasterize the run's
///   embedded font at the draft's own advances, which is `pdfce-render`'s work
///   and is a re-raster of a region per keystroke — and `BENCHMARK.md`'s
///   measured fact is that ~99 % of render cost on dense CAD is
///   resolution-independent, so *"just re-raster the run's box"* is not the
///   cheap operation it sounds like.
///
/// So the canvas shows a **caret and a bracket**: where the text is, and how
/// wide the engine's real metrics say the draft will be. The characters
/// themselves are shown off-canvas, in the status bar, where `text::textedit`
/// owns the sentence — which is Rule 4's own instruction about where a derived,
/// not-yet-applied thing belongs.
pub fn preview(ui: &Ui, ctx: &egui::Context, p: &Preview<'_>) {
    let Some(draft) = read(ctx) else {
        return;
    };
    if draft.page != p.page_index {
        return;
    }
    let Some(page) = p.doc.pages.get(p.page_index) else {
        return;
    };
    let theme = egui_shell::theme::Theme::of(ui.ctx());
    let painter = ui.painter();
    // A 1 s blink, and a repaint request so it actually blinks on a canvas with
    // no other reason to redraw.
    let on = (ui.input(|i| i.time) * 1.6) as i64 % 2 == 0;
    ui.ctx()
        .request_repaint_after(std::time::Duration::from_millis(400));
    let Some(rect) = caret_box(p.doc, &draft, page) else {
        return;
    };
    let screen = egui::Rect::from_two_pos(p.map.to_screen(rect.min), p.map.to_screen(rect.max));
    // The extent bracket: always drawn, so the operator can see WHERE the edit
    // is even between blinks.
    painter.rect_stroke(
        screen,
        0.0,
        egui::Stroke::new(1.0, theme.palette.accent),
        egui::StrokeKind::Outside,
    );
    if on {
        // ★★ The caret is drawn AT THE CARET, 2026-08-20.
        //
        // This used to be `screen.right_top()..right_bottom()` - the right edge
        // of the whole run's box - and that was not a drawing choice. An
        // append-only draft has no other position to draw, and the operator
        // read the result exactly as it was: *"the cursor just sits at the end
        // of a text line."*
        //
        // `caret_x` answers `None` when the position cannot be derived, and the
        // right edge is then the honest fallback: it is where the next typed
        // character will appear if the caret is at the end, which is the
        // overwhelmingly common case for a draft the glyphs cannot describe.
        let x = caret_x(p.doc, &draft, page, p.map).unwrap_or(screen.right());
        painter.line_segment(
            [
                egui::Pos2::new(x, screen.top()),
                egui::Pos2::new(x, screen.bottom()),
            ],
            egui::Stroke::new(1.5, theme.palette.accent),
        );
    }
}

/// **The caret's x in SCREEN space**, or `None` when it cannot be derived.
///
/// # The approximation, stated rather than hidden
///
/// The glyph metrics on the page describe the run **as it was**, and the draft
/// is not laid out until it is committed - nothing measures the operator's
/// in-progress string against the font until `EditSession::edit_text` runs. So:
///
/// * while `caret <= glyphs.len()`, the position is **exact** - it is a real
///   glyph boundary read from the page.
/// * beyond that, the operator has typed more characters than the run had, and
///   the position is **extrapolated** by repeating the last glyph's advance.
///
/// The second case is an estimate and can drift on a proportional font. It is
/// a pre-commit affordance - the cursor, in rule 4's own words - and it
/// disappears the moment the edit is applied, so nothing the operator keeps is
/// derived from it. Drawing nothing there would be worse: a caret that vanishes
/// as soon as you type past the end of the old text reads as the editor giving
/// up.
fn caret_x(
    doc: &OpenDoc,
    draft: &Draft,
    page: &pdfce_core::page_tree::Page,
    map: &crate::canvas::mapping::PageMapping,
) -> Option<f32> {
    let Anchor::Run { run, .. } = &draft.anchor else {
        return None;
    };
    let text = doc.page_text()?;
    let glyphs = &text.runs.get(*run)?.glyphs;
    let last = glyphs.last()?;
    let (x, y) = if draft.caret == 0 {
        let first = glyphs.first()?;
        (first.x, first.y)
    } else if let Some(g) = glyphs.get(draft.caret - 1) {
        (g.x + g.advance, g.y)
    } else {
        // Past the end of the original run - extrapolate, and say so above.
        #[allow(clippy::cast_precision_loss)]
        let extra = (draft.caret - glyphs.len()) as f32;
        (last.x + last.advance * (1.0 + extra), last.y)
    };
    let canvas = crate::viewer::pdf_space_to_canvas(Pos2::new(x, y), page)?;
    Some(map.to_screen(canvas).x)
}

/// The draft's box in **canvas** space, or `None` when it cannot be derived.
///
/// For an existing run this is the union of its glyph boxes; for a new-text
/// origin it is a nominal one-line box at the click. Both are converted through
/// [`crate::viewer::pdf_space_to_canvas`], the inverse of the bridge
/// [`resolve_run`] uses, so the caret lands on the glyphs it was resolved from.
fn caret_box(
    doc: &OpenDoc,
    draft: &Draft,
    page: &pdfce_core::page_tree::Page,
) -> Option<egui::Rect> {
    match &draft.anchor {
        Anchor::Run { run, .. } => {
            let text = doc.page_text()?;
            let r = text.runs.get(*run)?;
            let mut acc: Option<egui::Rect> = None;
            for g in &r.glyphs {
                let lo =
                    crate::viewer::pdf_space_to_canvas(Pos2::new(g.x, g.y + g.size * -0.25), page)?;
                let hi = crate::viewer::pdf_space_to_canvas(
                    Pos2::new(g.x + g.advance, g.y + g.size * 0.9),
                    page,
                )?;
                let b = egui::Rect::from_two_pos(lo, hi);
                acc = Some(acc.map_or(b, |a| a.union(b)));
            }
            acc
        }
        Anchor::Origin { x, y } => {
            #[allow(clippy::cast_possible_truncation)]
            let (x, y) = (*x as f32, *y as f32);
            let lo = crate::viewer::pdf_space_to_canvas(Pos2::new(x, y - 3.0), page)?;
            let hi = crate::viewer::pdf_space_to_canvas(Pos2::new(x + 6.0, y + 11.0), page)?;
            Some(egui::Rect::from_two_pos(lo, hi))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★ **A draft equal to what it replaces is not a write.**
    ///
    /// The no-op guard, and it is load-bearing because clicking away commits: an
    /// operator who typed a letter and removed it again would otherwise get an
    /// undo entry for having changed their mind.
    #[test]
    fn an_unchanged_draft_pushes_no_action() {
        let draft = Draft {
            page: 0,
            kind: TextEditKind::Edit,
            anchor: Anchor::Run {
                run: 3,
                original: "TITLE".to_owned(),
            },
            text: "TITLE".to_owned(),
            caret: 0,
            seeded: true,
        };
        let mut actions = Vec::new();
        // ★ A bare `Context`, and it is the honest one for a pure-commit test:
        // the pen it reads is whatever `TextPen::default()` is, which is the
        // engine's own default, so these assertions are about the ACTION's
        // shape and not about a pen nobody set.
        commit_into(&egui::Context::default(), &draft, &mut actions);
        assert!(actions.is_empty(), "an unchanged draft is not an edit");
    }

    /// **An emptied draft is not a deletion.**
    ///
    /// Deleting every character of a run and clicking away is ambiguous —
    /// "remove this text" and "I changed my mind" look identical — and the
    /// recoverable reading is the one that writes nothing. Removing text is
    /// redaction's job, and it is a security operation with its own surface.
    #[test]
    fn an_emptied_draft_pushes_no_action() {
        let draft = Draft {
            page: 0,
            kind: TextEditKind::Edit,
            anchor: Anchor::Run {
                run: 1,
                original: "A".to_owned(),
            },
            text: String::new(),
            caret: 0,
            seeded: true,
        };
        let mut actions = Vec::new();
        // ★ A bare `Context`, and it is the honest one for a pure-commit test:
        // the pen it reads is whatever `TextPen::default()` is, which is the
        // engine's own default, so these assertions are about the ACTION's
        // shape and not about a pen nobody set.
        commit_into(&egui::Context::default(), &draft, &mut actions);
        assert!(actions.is_empty());
    }

    /// **A changed draft pushes exactly one action, carrying both texts.**
    #[test]
    fn a_changed_draft_pushes_one_edit_carrying_both_texts() {
        let draft = Draft {
            page: 2,
            kind: TextEditKind::Edit,
            anchor: Anchor::Run {
                run: 7,
                original: "REV A".to_owned(),
            },
            text: "REV B".to_owned(),
            caret: 0,
            seeded: true,
        };
        let mut actions = Vec::new();
        // ★ A bare `Context`, and it is the honest one for a pure-commit test:
        // the pen it reads is whatever `TextPen::default()` is, which is the
        // engine's own default, so these assertions are about the ACTION's
        // shape and not about a pen nobody set.
        commit_into(&egui::Context::default(), &draft, &mut actions);
        assert_eq!(actions.len(), 1);
        assert_eq!(
            actions[0],
            crate::app::actions::Action::CommitTextEdit {
                page: 2,
                run: 7,
                original: "REV A".to_owned(),
                replacement: "REV B".to_owned(),
            }
        );
    }

    /// **An empty add-text draft places nothing.** A click with the Add tool and
    /// no typing is a caret, not a write.
    #[test]
    fn an_empty_add_text_draft_places_nothing() {
        let draft = Draft {
            page: 0,
            kind: TextEditKind::Add,
            anchor: Anchor::Origin { x: 10.0, y: 20.0 },
            text: String::new(),
            caret: 0,
            seeded: true,
        };
        let mut actions = Vec::new();
        // ★ A bare `Context`, and it is the honest one for a pure-commit test:
        // the pen it reads is whatever `TextPen::default()` is, which is the
        // engine's own default, so these assertions are about the ACTION's
        // shape and not about a pen nobody set.
        commit_into(&egui::Context::default(), &draft, &mut actions);
        assert!(actions.is_empty());
    }

    /// **The two kinds name the two registered commands, and they are
    /// different.** A copy-paste that gave both the same id would arm one tool
    /// from two buttons and nothing would notice.
    #[test]
    fn each_kind_names_its_own_registered_command() {
        assert_eq!(TextEditKind::Edit.command_id(), "edit.text");
        assert_eq!(TextEditKind::Add.command_id(), "edit.add_text");
    }

    /// **The oracle for *"it doesn't type anything in the box when I type"*.**
    ///
    /// Every existing text-edit check seeds the draft through `PDFCE_DIAG_TYPE`,
    /// which is the ONE path that bypasses the event loop — so all of them pass
    /// on a build where real typing is dead. This one drives a real
    /// `egui::Context` with a real `Event::Text` and asserts the draft grew.
    #[test]
    fn a_real_text_event_lands_in_the_draft() {
        let ctx = egui::Context::default();
        store(
            &ctx,
            Draft {
                page: 0,
                kind: TextEditKind::Add,
                anchor: Anchor::Origin { x: 10.0, y: 10.0 },
                text: String::new(),
                caret: 0,
                seeded: true,
            },
        );
        let mut input = egui::RawInput::default();
        input.events.push(egui::Event::Text("h".to_owned()));
        let mut actions = Vec::new();
        let inner = ctx.clone();
        let _ = ctx.run_ui(input, move |c| {
            egui::CentralPanel::default().show(c, |ui| {
                typing(ui, &inner, true, &mut actions);
            });
        });
        assert_eq!(read(&ctx).map(|d| d.text), Some("h".to_owned()));
        assert_ne!(
            TextEditKind::Edit.command_id(),
            TextEditKind::Add.command_id()
        );
    }
}
