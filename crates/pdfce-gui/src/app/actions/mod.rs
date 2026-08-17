//! # `app::actions` — the one channel through which anything changes
//!
//! ## The invariant this module exists to enforce
//!
//! **No code path runs from a widget to a document.** A widget that is
//! clicked, a key that is pressed, a wheel that is spun — none of them
//! change anything directly. Each produces an [`Action`], the actions are
//! collected while the frame is drawn, and they are applied *after* it, in
//! one place, by [`PdfceApp::apply`].
//!
//! `SALVAGE.md` calls this *"the single best structural decision in the old
//! GUI"*, and `PROJECT_PLAN.md` §3 lists it first among the invariants that
//! are "not up for renegotiation". It is established here at stage S0, with
//! a handful of actions and one widget, **because retrofitting it is
//! expensive**: every widget written under the other discipline has to be
//! found and rewritten, and the ones that are missed are exactly the ones
//! that produce an incoherent undo log later.
//!
//! ## Why it is worth the indirection
//!
//! Four things fall out of it, none of which can be had cheaply otherwise:
//!
//! 1. **A coherent undo log.** One operator gesture becomes one action
//!    becomes one command-log entry. A widget that mutated in place would
//!    have to remember to log, and the ones that forgot would be invisible
//!    holes in the history. (S0 has no undo — but S4's undo is only
//!    possible because the funnel already exists.)
//! 2. **The borrow checker stops fighting.** egui is immediate-mode: the
//!    document is being *read* to draw the very widget that wants to change
//!    it. Deferring the change to after the frame turns an aliasing problem
//!    into a queue.
//! 3. **Order becomes explicit.** Two actions raised in one frame are
//!    applied in a defined order, in one readable function, rather than in
//!    whatever order the layout code happened to run.
//! 4. **Every state change is greppable.** "What can change the zoom?" has
//!    a complete answer: the [`Action`] variants that touch it.
//!
//! ## Scope
//!
//! Zoom, page navigation, and — from stage S4 — **the actions that change the
//! document**: [`Action::DeleteSelection`], the three move verbs
//! ([`Action::MoveSelection`], [`Action::MoveSubpath`], [`Action::MoveNode`])
//! and, from Phase 6, [`Action::CommitMarkup`].
//! Those are why this module has a mutation path, and the path is short and
//! deliberately in one place; see [`vector_edit`], which every one of them goes
//! through so the cancel-mutate-bump-invalidate protocol is written once rather
//! than four times.
//!
//! **There is no resize action, and its absence is deliberate.**
//! `EditSession` has the whole `move_*` family and no scale verb of any kind,
//! so a `ResizeSelection` here would be an enum variant nothing could honour —
//! which is the same "no placeholders" rule this enum's own doc comment states
//! two paragraphs down. The canvas still *consumes* a grip drag, so it cannot
//! fall through to a marquee, and commits nothing; see
//! [`crate::canvas::handles`].
//!
//! ## ★ Opening and closing are actions now, and this header said they would be
//!
//! Until `file.open` existed, this paragraph read: *"Opening a document is
//! deliberately not an action at S0: it happens once, from `argv`, before the
//! event loop starts. It becomes one the moment there is an Open command, and
//! the `apply` gate that blocks an open while a save is pending lands with
//! it."*
//!
//! Both halves have now happened. [`Action::Open`] and [`Action::Close`] are
//! the first two variants that are about **which document is open** rather
//! than about the one that already is, and they are the reason [`PdfceApp::apply`]
//! no longer begins by refusing everything when nothing is open — see the
//! ★ comment at the top of its body, which is the whole of why the two are
//! matched *before* that guard rather than inside it.
//!
//! The gate is [`PdfceApp::save_pending`], consulted by both, and its own doc
//! comment carries the rule. It answers `false` in this build because there
//! is no save path at all — so no confirmation dialog is built for a
//! condition that cannot occur, and the rule has one home rather than being
//! rediscovered by whoever adds the save.
//!
//! **The dialog is not the action.** `file.open` opens a native file picker,
//! which is a UI act that happens during dispatch; what goes through the
//! funnel is its *result*, a path. See [`crate::app::files`] for the picker,
//! the diagnostics seam that lets a scripted harness answer it without a
//! human, and why the two are separated at exactly that line.
//!
//! ## ★ `Action` is no longer `Copy`, and that is a decision rather than an accident
//!
//! [`Action::DeleteSelection`] carries a `Vec<usize>` — the paint-order
//! indices to remove — and it has to, for a reason that is not about
//! convenience:
//!
//! `EditSession::delete_objects` takes a **slice** and resolves every index
//! before planning anything, *"so an out-of-range one refuses the call rather
//! than deleting the prefix that happened to resolve"*. Deleting a
//! multi-selection therefore has to be **one** command. Emitting one
//! `DeleteObject` action per selected object would renumber the page between
//! them — deleting object 5 and then object 3 deletes 5 and then whatever
//! moved into slot 3 — so the batch cannot be decomposed.
//!
//! The alternative to carrying the list would be for `apply` to read the
//! selection itself. It cannot: the selection lives in the canvas, `apply`
//! has no `egui::Context`, and giving it one would make the action funnel
//! depend on the UI framework. Carrying the operands is also simply what an
//! action *is* — a complete statement of an operator's intent, resolvable
//! after the frame that raised it.

use std::cell::RefCell;

use crate::viewer::FitMode;

/// What applying an [`Action`] does — the interpreter half of this module.
///
/// Split out under **R2**; see its own header for the seam. Not `pub`: nothing
/// outside `app` applies an action, and the two entry points it adds are
/// inherent methods on [`crate::app::PdfceApp`] rather than free functions, so
/// they are reachable exactly where they were before the split.
mod apply;
/// The four page verbs' bodies, and the structural resync every edit owes.
///
/// A sibling of [`apply`] rather than part of it, on rule R2's own reasoning:
/// that file's subject is the cancel–mutate–bump–invalidate protocol, and this
/// one's is *a page index is a position, not an identity*. See its header for
/// the table of what each kind of page edit invalidates.
mod pages;

// ---------------------------------------------------------------------------
// The edit disclosure — what [`vector_edit`] carries out to `app::status`
//
// See [`vector_edit`]'s "The disclosures" section for what a disclosure IS.
// This block is the answer to the question that section used to leave open:
// *where does an operator read one?*
// ---------------------------------------------------------------------------

/// The rule-4 sentences one vector edit owed, and the revision they describe.
///
/// `pdfce-core`'s vector verbs return `Result<Vec<String>, EditError>`, and the
/// `Vec<String>` is the **disclosure list**: prose the surgery owes because it
/// had to change an operator's *form* to express their request. Rule 4 says a
/// disclosure belongs on an operator-visible surface, and until 2026-08-14 this
/// list reached `PDFCE_DIAG` and nothing else — recorded, not disclosed.
///
/// # Why this is shaped like [`crate::panels::forms::edit::FillDisclosure`]
///
/// Because it is the same fact in a different verb, and the precedent had
/// already settled every question this one raises: a note about an edit,
/// stamped with the epoch the edit produced, read by a surface that draws it
/// only while it still describes the document on screen. Building a second
/// mechanism beside that one would give the status bar two ways to learn the
/// same kind of thing, and the second would be the one that forgot to retire
/// itself.
///
/// # ★ What it deliberately does NOT carry: the verb's name
///
/// A `FillDisclosure` carries the **field name**, because a fill raised from
/// the Forms panel happens in a list of forty rows and the sentence is read
/// somewhere other than where the value was typed. The vector verbs have no
/// such gap: the gesture that raises one is a drag on the object the sentence
/// is about, the sentence appears on the next frame, and core's own wording
/// (*"This shape…"*, *"This point…"*) is written for exactly that reading.
///
/// The only name available here is [`vector_edit`]'s `label` — `move-node`,
/// `delete-objects` — which is a **trace token**, not operator copy. Putting it
/// on screen would either ship a hyphenated internal identifier to an operator
/// or require a second catalog translating trace tokens into English, which is
/// a second vocabulary for the verbs the ribbon already names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditDisclosure {
    /// The revision this describes — [`OpenDoc::edit_epoch`] **after** the
    /// edit. A disclosure whose epoch is not the document's current one
    /// describes an edit that has since been undone or superseded, and must
    /// not be shown.
    pub epoch: u64,
    /// The sentences, in the order the planner pushed them, **verbatim** from
    /// `pdfce-core`. They are finished English prose written where the fact is
    /// known; this shell frames them (see
    /// [`crate::text::status::edit_disclosure_line`]) and rewrites nothing.
    pub notes: Vec<String>,
}

thread_local! {
    /// The most recent vector edit's disclosures, waiting to be read by the
    /// status bar.
    ///
    /// # ★ Why a thread-local, and why that is sound rather than smuggled
    ///
    /// The same answer `crate::panels::forms::edit`'s `LAST_FILL` gives, and
    /// for the same reason it is worth restating rather than cross-referencing
    /// away: it *should* be a field on [`OpenDoc`], beside `edit_epoch`,
    /// dropped with the document. `OpenDoc` is declared in
    /// `crate::app::state`, which this work may not extend, so the constraint
    /// is a **territory boundary rather than a design judgement** — stated
    /// here so whoever lifts it knows what the preferred shape is.
    ///
    /// Why it is nonetheless sound: this is not document state. It is a note
    /// about an edit that has already gone through the funnel, it cannot
    /// change a pixel of the page, and nothing reads it except a bar deciding
    /// whether to draw a sentence. It is correctly scoped too — `eframe`'s
    /// update loop is one thread, so the writer and the reader are the same
    /// thread, and a test on another thread gets its own empty slot rather
    /// than another test's leftovers (which a `static Mutex` would hand it).
    ///
    /// Staleness is handled by the `epoch` rather than by clearing: the
    /// sentence is shown only while it describes the revision on screen, so an
    /// undo silences it without anything having to remember to.
    static LAST_EDIT: RefCell<Option<EditDisclosure>> = const { RefCell::new(None) };
}

/// What the last vector edit disclosed, if it still describes the open
/// document.
///
/// **The status bar's read** — see [`crate::app::status`]. Returns `None` when
/// the last edit disclosed nothing, was on another document, or has since been
/// undone or superseded.
///
/// # ★ It cannot be live at the same time as a fill disclosure
///
/// Both are keyed on [`OpenDoc::edit_epoch`], and one edit bumps the epoch
/// once. So the epoch on screen was produced by exactly one edit, which was
/// either a form edit (recording a `FillDisclosure` and no `EditDisclosure`)
/// or a vector edit (the reverse). The bar therefore never has to arbitrate
/// between two disclosure lines competing for one row: the mutual exclusion is
/// a property of the epoch, not a rule anybody has to enforce.
#[must_use]
pub fn last_edit_disclosure(epoch: u64) -> Option<EditDisclosure> {
    LAST_EDIT.with_borrow(|slot| {
        slot.as_ref()
            .filter(|d| d.epoch == epoch && !d.notes.is_empty())
            .cloned()
    })
}

/// Record what an edit disclosed — or, with `None`, that it disclosed nothing.
///
/// Written unconditionally by [`vector_edit`], including the overwhelmingly
/// common empty case. Overwriting with `None` is not required for correctness
/// (the epoch filter above already retires a stale sentence) and is done
/// anyway, so the slot never holds a note whose only defence against being
/// shown is an integer comparison that a future undo implementation could get
/// wrong.
fn record_edit_disclosure(disclosure: Option<EditDisclosure>) {
    LAST_EDIT.with_borrow_mut(|slot| *slot = disclosure);
}

/// **Put one sentence on the status bar's disclosure row**, stamped with the
/// revision currently on screen.
///
/// The narrow public door onto the same slot [`record_edit_disclosure`] writes,
/// and it exists for exactly one caller: `canvas::interact`, when a click with
/// the caret tool armed **cannot** place a caret. That is not an edit — nothing
/// was written, no epoch moved — so it has no disclosure list to ride in on, and
/// without this it would have nowhere to be said.
///
/// It has to be said somewhere. `DEFECTS.md` D4a records the old shell's
/// handling of the same case: a `cross_run` flag that *"silently disables the
/// whole typing loop"*, so the operator pressed keys and nothing happened. A
/// limit stated in a sentence is a limit; the same limit stated by a keyboard
/// that stops responding is a bug report.
///
/// **`epoch` is the CURRENT one, not a new one**, and that is what makes the
/// lifetime right without anything remembering to clear it: the sentence is
/// visible from now until the next real edit moves the epoch past it, which is
/// the same rule `vector_edit`'s own stamp follows and for the same reason its
/// ★ comment gives.
pub(crate) fn record_note(epoch: u64, note: String) {
    record_edit_disclosure(Some(EditDisclosure {
        epoch,
        notes: vec![note],
    }));
}

/// Which piece of View ▸ Display chrome a [`Action::ToggleViewChrome`] is
/// about.
///
/// An enum rather than three action variants — see that variant's own docs —
/// and it lives here rather than in `canvas` because it is the *operand of an
/// action*, and `shell::commands` (which maps ids to it) must not have to
/// reach into the canvas to name one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewChrome {
    /// `view.rulers` — the gutters along the canvas edges.
    Rulers,
    /// `view.grid` — the drawing grid over each page.
    Grid,
    /// `view.guides` — whether the operator's guides are shown and draggable.
    Guides,
}

impl ViewChrome {
    /// Every variant, in the order View ▸ Display lists them.
    ///
    /// Iterated by the tests that assert each has a command and each command
    /// has a `selected:` condition — the same both-directions check
    /// `PageDisplay::ALL` exists for, and for the same reason: a fourth toggle
    /// added to the enum with no registration would draw nothing and nothing
    /// else in the suite would notice.
    pub const ALL: &'static [ViewChrome] =
        &[ViewChrome::Rulers, ViewChrome::Grid, ViewChrome::Guides];

    /// Read this toggle out of a view state.
    #[must_use]
    pub fn read(self, view: &crate::viewer::ViewState) -> bool {
        match self {
            ViewChrome::Rulers => view.rulers,
            ViewChrome::Grid => view.grid,
            ViewChrome::Guides => view.guides,
        }
    }

    /// Write this toggle into a view state.
    ///
    /// The pair with [`Self::read`], so the enum's mapping onto
    /// [`crate::viewer::ViewState`]'s three fields is stated exactly twice, in
    /// adjacent functions, instead of once per consumer.
    pub fn write(self, view: &mut crate::viewer::ViewState, on: bool) {
        match self {
            ViewChrome::Rulers => view.rulers = on,
            ViewChrome::Grid => view.grid = on,
            ViewChrome::Guides => view.guides = on,
        }
    }
}

/// One operator intent, applied after the frame that raised it.
///
/// Every variant is reachable from a real control today. A variant nothing
/// can raise is dead code wearing a design pattern, and the "no
/// placeholders" invariant (`PROJECT_PLAN.md` §3) applies to enums as much
/// as to labels.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// **Open this document, replacing whatever is open.**
    ///
    /// Raised by `file.open` once the picker has answered, by `file.recent`
    /// once the operator has chosen a row, and by nothing else — the two
    /// surfaces that can name a file. It is deliberately *not* raised by
    /// `crate::run`'s `argv` path, which calls
    /// [`PdfceApp::open_path`] directly: there is no frame yet, so there is
    /// no frame to defer to, and routing it through an empty action queue
    /// would be ceremony rather than discipline.
    ///
    /// # Why the path travels, and why it is already absolute by the time
    /// anyone stores it
    ///
    /// Same reason [`Self::DeleteSelection`] carries its operand list: an
    /// action is a complete statement of intent, resolvable after the frame
    /// that raised it. The picker's answer cannot be re-derived later —
    /// the dialog is gone — so the only alternative would be a field on the
    /// application holding a half-finished intent between frames, which is
    /// precisely the state the funnel exists to avoid.
    ///
    /// Absolutizing happens in [`crate::app::recent::RecentFiles::remember`]
    /// rather than here, because it is a property of *storing* a path, not of
    /// opening one: `Document::load` is perfectly happy with a relative path
    /// and the operator's shell already resolved it.
    Open(std::path::PathBuf),
    /// **Make a blank document, replacing whatever is open.**
    ///
    /// Raised by `file.new` and by nothing else. Carries no operand: unlike
    /// [`Self::Open`] there is nothing to name — the bytes are compiled in
    /// (`crate::app::blank::TEMPLATE`) and the document's own name is derived
    /// after the frame from a counter on the application, which is where a
    /// number that outlives one document belongs.
    ///
    /// Applied by [`PdfceApp::new_document`], which carries the reasoning:
    /// where the template comes from, why the engine has no part in it, and
    /// why New leaves the operator's mode alone.
    New,
    /// **Write the open document, edits and all, to a file the operator
    /// names.**
    ///
    /// Raised by `file.save_copy` and by nothing else. Applied by
    /// [`crate::app::save::save_copy`], whose header carries every decision in
    /// this feature: why the save mode is **incremental** (a promise already
    /// shipped in the command's tooltip), why nothing on
    /// [`crate::app::state::OpenDoc`] moves, and which `SaveOptions` fields
    /// were chosen and why.
    ///
    /// # ★ Why it carries no path, when [`Self::Open`] carries one
    ///
    /// Because there is no operand to carry. `Open`'s path is the **answer to a
    /// dialog that is gone by the time the action is applied** — it cannot be
    /// re-derived, so it travels, and its picker therefore runs during dispatch.
    /// A save has the opposite shape: what to *suggest* is a pure function of
    /// the open document (`save::suggested_path`), so nothing is lost by asking
    /// later, and asking later is required rather than merely allowed.
    ///
    /// `crate::app::files::pick_save_path` documents a **frame-timing
    /// requirement** — a native modal opened inside an `egui` layout closure
    /// blocks the frame it is being drawn in, leaving a half-painted window
    /// behind a dialog. Dispatch does not always satisfy it:
    /// `crate::app::PdfceApp::central` dispatches the canvas's context-menu
    /// tokens from *inside* `egui::CentralPanel::show`. The apply phase always
    /// does — it is step 3, after every surface has closed. So raising an action
    /// is not ceremony here; it is the only placement that honours the
    /// requirement from every route the command can be invoked by.
    ///
    /// # It is matched before the document guard
    ///
    /// With nothing open there is nothing to write, and a keymap can reach
    /// `Ctrl+S` from any state. The guard's silent drop would make that
    /// indistinguishable from a chord that never arrived, so — like
    /// [`Self::Find`] — this is answered above it, by name, on the trace.
    SaveCopy,
    /// **Close the open document and return to [`Status::Empty`].**
    ///
    /// Raised by `file.close`, which is gated on `doc.open`, so the no-document
    /// case is unreachable from the ribbon — and handled anyway, because a
    /// customized keymap can reach any command from any state and an action
    /// that assumed otherwise would be a panic waiting for an operator to
    /// find it.
    ///
    /// Carries nothing. There is exactly one document in this build; when
    /// there are several (the document switcher `OpenDoc::path` is held for),
    /// this grows the identity of the one being closed, which is the same
    /// change [`Self::DeleteSelection`] made when it grew its page number.
    Close,
    /// Multiply the current zoom by a factor — the Ctrl+wheel path.
    ///
    /// Carries the factor rather than a target zoom because that is what
    /// egui's `zoom_delta` reports, and because the *clamp* must be applied
    /// by the state machine that owns the ceiling, not by the widget.
    ZoomBy(f32),
    /// Step to the next zoom-ladder rung above the current zoom.
    ZoomIn,
    /// Step to the next zoom-ladder rung below the current zoom.
    ZoomOut,
    /// Enter a fit mode — a *mode*, not a one-shot: the zoom is re-derived
    /// from the viewport every frame until an explicit zoom pins it.
    Fit(FitMode),
    /// Pin the zoom to an exact factor.
    ///
    /// **`Fit(FitMode::None)` is not this**, and the difference was a live
    /// defect the status bar surfaced. `FitMode::None` only stops the
    /// per-frame re-fit; it leaves `zoom` wherever it happened to be. So an
    /// "Actual size" control raising it pinned 73 % *at* 73 % while its
    /// tooltip promised one PDF point per screen point — a control whose
    /// label and behaviour disagreed, on two surfaces at once, because the
    /// ribbon's `view.zoom_actual` raised the same thing.
    ///
    /// `ZoomBy(1.0 / zoom)` would land on the right number and is still
    /// wrong: it routes a discrete command through the wheel path, which
    /// carries the 150 ms settle debounce that exists for continuous
    /// gestures. A command that arrives in one piece should commit in one
    /// piece.
    ZoomTo(f32),
    /// Step one page toward the end of the document.
    NextPage,
    /// Step one page toward the start of the document.
    PrevPage,
    /// Jump to a 0-based page index, clamped into the document.
    GoToPage(usize),
    /// Remove the canvas selection's objects from `page`, as **one**
    /// undoable command.
    ///
    /// Raised by the canvas when Delete or Backspace is pressed with a
    /// non-empty selection and no text field focused — the defect `DEFECTS.md`
    /// D1 is about, from the other end. D1's fix (`ctx.text_edit_focused()`
    /// rather than `ctx.egui_wants_keyboard_input()`) made the key *reachable*
    /// after a canvas click; this is the verb it reaches.
    ///
    /// # The operand list is already clean, and must be
    ///
    /// `objects` arrives ascending and de-duplicated from
    /// [`crate::canvas::selection::SelectionState::object_indices_on`],
    /// because `EditSession::delete_objects` resolves **every** index before
    /// planning anything: one stale or duplicated entry refuses the whole
    /// call. That refusal is the correct engine behaviour — the alternative
    /// is deleting the prefix that happened to resolve — so the shell's job
    /// is to hand it a list that can succeed.
    ///
    /// # Why the page travels with the list
    ///
    /// A paint-order index is a position on **one page**. Re-deriving the
    /// page here from `doc.view.page_index` would be a second source of truth
    /// that is right until the moment it matters: an action is applied after
    /// the frame that raised it, and a page step raised in the same frame is
    /// applied first if it was pushed first. Carrying the page makes the
    /// statement complete.
    DeleteSelection {
        /// The 0-based page the indices are positions on.
        page: usize,
        /// Paint-order indices, ascending and unique.
        objects: Vec<usize>,
    },
    /// Displace the canvas selection's objects on `page` by a **page-space**
    /// delta, as **one** undoable command.
    ///
    /// Raised by [`crate::canvas::moving::drag`] when a move drag that began
    /// inside the selection is released. The Object-rung member of the move
    /// family; its siblings are [`Self::MoveSubpath`] and [`Self::MoveNode`].
    ///
    /// # Why the whole list travels, exactly as it does for Delete
    ///
    /// `EditSession::move_objects` takes a **slice**, and resolves *and
    /// type-checks* every index before planning anything, so one non-path or
    /// one stale entry refuses the whole call rather than moving the prefix
    /// that happened to qualify. Emitting one `move_object` per selected
    /// object would be wrong twice over: N undo entries for one drag, and — the
    /// correctness half — each call re-splices the content stream, so the
    /// second index would be planned against byte offsets the first already
    /// invalidated. `docs/core-api/02` states it in a box: *"Never loop the
    /// singular verbs over a selection."*
    ///
    /// # ★ Why this does NOT invalidate the selection, and Delete does
    ///
    /// Because `move_*` **does not renumber**, and that is measured rather
    /// than assumed — `crates/pdfce-core/tests/object_identity_across_edits.rs`
    /// decomposes, edits, and decomposes again. A move rewrites operands
    /// *inside* existing operators, so no operator is added or removed and the
    /// second decomposition yields the same objects at the same indices. The
    /// `delete_*` family excises byte **spans** and therefore does renumber,
    /// which is why `pdfce_core::vector::remap_index_after_delete` exists and
    /// why nothing like it is needed here. See
    /// [`crate::canvas::moving`]'s header for the full table.
    ///
    /// # Units
    ///
    /// `dx`/`dy` are **PDF user-space** points, Y-**up** — produced by
    /// [`crate::canvas::moving::page_delta`], which is the one place a
    /// canvas-space drag crosses into page space. A screen-pixel delta here
    /// would compile, run, and scale the move with the magnification.
    MoveSelection {
        /// The 0-based page the indices are positions on.
        page: usize,
        /// Paint-order indices, ascending and unique.
        objects: Vec<usize>,
        /// Horizontal displacement, PDF user-space points.
        dx: f64,
        /// Vertical displacement, PDF user-space points (Y is up).
        dy: f64,
    },
    /// Displace **one subpath** of one path object by a page-space delta, as
    /// one undoable command — the Part rung's move verb.
    ///
    /// Raised only when the entered object decomposes into *subpaths*. A text
    /// object's Part rung is a show-operator run, which `move_subpath` has
    /// nothing to translate, so the canvas declines and traces rather than
    /// borrowing the Object rung's verb — the same rule, and the same reason,
    /// as `SelectionState::deletable_objects_on`'s rung guard.
    MoveSubpath {
        /// The 0-based page.
        page: usize,
        /// The enclosing object, by paint-order index.
        object: usize,
        /// The subpath, in decomposition order.
        subpath: usize,
        /// Horizontal displacement, PDF user-space points.
        dx: f64,
        /// Vertical displacement, PDF user-space points (Y is up).
        dy: f64,
    },
    /// Drag **one anchor** of one path object to an absolute page-space point
    /// — the Node rung's move verb.
    ///
    /// # Why a destination and not a displacement
    ///
    /// Because that is `EditSession::move_node`'s signature, and the signature
    /// is right: the operand being rewritten *is* a coordinate pair, and the
    /// planner maps the destination through the object's CTM affine inverse in
    /// one step. Expressing it as a delta would make the planner reconstruct
    /// the point it was given, in a space the caller would then have had to
    /// name. The canvas computes it as *"where the anchor is now, plus the
    /// drag"*, and refuses the move outright if the decomposition can no
    /// longer say where the anchor is — see
    /// [`crate::canvas::moving::Refusal::NodeNotFound`].
    ///
    /// `node` is **object-scoped**: the space `vector::anchor_count` reports
    /// and `pdfce-cli node-move --node N` addresses. A second numbering would
    /// make the number pdfce shows disagree with the number the operator can
    /// act on.
    MoveNode {
        /// The 0-based page.
        page: usize,
        /// The enclosing object, by paint-order index.
        object: usize,
        /// The anchor, object-scoped.
        node: usize,
        /// Where the anchor ends up, in PDF user space.
        to: pdfce_core::vector::Point,
    },
    // =======================================================================
    // ★ THE PAGE VERBS — structural edits, and the family that renumbers
    //
    // Four variants for five commands (`pages.rotate_left` and
    // `pages.rotate_right` share one), plus `pages.extract`, which is not here
    // at all — see `ExtractPages` below for why it is and `pages.split` /
    // `pages.merge_into` / `pages.insert_from_file` for why they are not.
    //
    // # Why the operand list travels, and is not re-derived at apply time
    //
    // The same argument `DeleteSelection` makes, one structure up: the operand
    // is the **Pages panel's multi-select**, resolved once by
    // `crate::panels::pages::ops::operands`. Re-reading `PanelsState` during
    // the apply would be a second reading of a set the operator may have
    // changed between the frame that raised the action and the frame that
    // applies it — and for `DeletePages` the consequence of reading it twice
    // is destroying sheets nobody chose.
    //
    // # ★ What separates these from every action above them
    //
    // Everything else in this enum either leaves the page *count* and the page
    // *order* alone, or is not a document edit at all. These three do neither,
    // and that is why `crate::app::actions::apply::page_edit` exists beside
    // `vector_edit` rather than each arm doing its own bookkeeping: a page
    // delete or reorder invalidates the flattened page vector, every cached
    // raster keyed on a page index, the canvas selection's page identity and
    // the panel's own picks, all at once, and a missed one is a stale picture
    // or a verb aimed at the wrong sheet.
    /// **Turn the operand pages by `delta` degrees**, as one undoable command.
    ///
    /// Raised by `pages.rotate_left` (−90) and `pages.rotate_right` (+90).
    ///
    /// # Why a delta rather than an absolute angle
    ///
    /// Because that is what the button means and what `EditSession::rotate_pages`
    /// implements: a selection of pages at 0°, 90° and 180° turned right lands
    /// at 90°, 180° and 270°, **not** all at 90°. The engine's own doc comment
    /// confirms Acrobat persists the absolute result of exactly that
    /// arithmetic. An absolute variant would be a different verb (*set the
    /// rotation of these pages to N*), which no control in this build offers.
    ///
    /// # It changes no page's identity
    ///
    /// A rotation rewrites one `/Rotate` entry per page. Nothing is added,
    /// removed or renumbered, so both selections survive it untouched — which
    /// is why the apply arm's resync is about *pictures* (every cached raster
    /// of a turned page is now wrong) and not about *indices*.
    RotatePages {
        /// 0-based page indices, ascending and unique.
        pages: Vec<usize>,
        /// A relative turn in degrees, a multiple of 90.
        delta: i32,
    },
    /// **Remove the operand pages from the document**, as one undoable
    /// command.
    ///
    /// Raised by `pages.delete` from the ribbon's Pages tab and from the page
    /// tile's context menu.
    ///
    /// # ★ This is the one action in the enum that renumbers pages
    ///
    /// `HANDOFF.md` §10 states the rule for objects — *"Selection is an
    /// identity — page, object, subpath, node — not a position"* — and this is
    /// its page-level instance. After the removal, every index above the lowest
    /// deleted page names a **different sheet**. Both selections in the
    /// application are therefore invalid, in different ways, and the apply arm
    /// deals with both:
    ///
    /// * the **page** selection named exactly the sheets that no longer exist,
    ///   so it is cleared;
    /// * the **canvas** selection names objects on a page *index*, and that
    ///   index now resolves to another sheet's content, so it is cleared too.
    ///
    /// # It is destructive and, until undo lands, irreversible
    ///
    /// No confirmation dialog, deliberately, and the reasoning is at the apply
    /// arm: `crate::app::save`'s `save_pending` is the one predicate this
    /// application consults before a destructive path, the engine records the
    /// removal as an undoable command already, and **nothing is written to
    /// disk** — the operator's file on disk is untouched until they save a
    /// copy, which is a separate deliberate act with its own dialog.
    DeletePages {
        /// 0-based page indices, ascending and unique.
        pages: Vec<usize>,
    },
    /// **Put the document's pages in a new order**, as one undoable command.
    ///
    /// Raised by `pages.move_up` and `pages.move_down`, which differ only in
    /// the permutation `crate::panels::pages::ops::move_order` computes.
    ///
    /// `order[i]` is the **current** 0-based index of the page that should end
    /// up at position `i` — `EditSession::reorder_pages`' contract verbatim,
    /// carried through unaltered so there is no second spelling of it to drift.
    /// The engine refuses anything that is not a permutation of
    /// `0..page_count`, and `move_order` builds one by construction.
    ///
    /// # ★ A reorder renumbers positions without destroying anything
    ///
    /// Which makes it the *middle* case between a move (nothing changes
    /// identity) and a delete (identities cease to exist), and the two
    /// selections get two different answers:
    ///
    /// * the **page** selection follows its sheets, through
    ///   [`crate::panels::pages::select::PageSelection::remap`] — the
    ///   permutation states exactly where each one went, so clearing would
    ///   throw away information the edit had in hand and make the reorder
    ///   arrows unusable twice in a row;
    /// * the **canvas** selection is cleared, because its entries carry a page
    ///   *index* and this crate cannot rewrite them —
    ///   `crate::canvas::selection::SelectionState` exposes no mutator for the
    ///   page of an entry, and inventing one would put a second page-remapping
    ///   rule in the module that owns object identity. Clearing is the honest
    ///   answer and it is stated rather than silent.
    ReorderPages {
        /// The new order, as `order[new_position] = current_index`.
        order: Vec<usize>,
    },
    /// **Write the operand pages out as a new standalone document.**
    ///
    /// Raised by `pages.extract`. The one page verb that changes **no**
    /// document: `pdfce_core::pageops::extract` returns the complete bytes of a
    /// freestanding PDF and the open session is not touched, which is exactly
    /// what the Review mode's stance requires — `crate::panels::pages`' header
    /// quotes the operator: *"an extraction writes a different file."*
    ///
    /// # ★ Why it is an action at all, when it mutates nothing
    ///
    /// For [`Self::SaveCopy`]'s reason and only that one: it opens a **native
    /// save dialog**, and `crate::app::files::pick_save_path` carries a
    /// frame-timing requirement dispatch cannot honour — `PdfceApp::central`
    /// dispatches the canvas's context-menu tokens from inside
    /// `egui::CentralPanel::show`, and a modal opened mid-layout blocks the
    /// frame it is being drawn in. The apply phase is always outside every
    /// closure. The page tile's context menu is dispatched from a panel body
    /// rather than the canvas, but the rule is the surface's, not the caller's.
    ExtractPages {
        /// 0-based page indices, ascending and unique. **Order is honoured** by
        /// the engine, so this is simultaneously "extract these pages" and
        /// "extract them in this order"; the panel produces them ascending.
        pages: Vec<usize>,
    },
    /// **Author one markup annotation on `page`**, from the drag that drew it.
    ///
    /// Raised by [`crate::canvas::markup::drag`] when a markup band is
    /// released, and by nothing else — there is no path from a ribbon button to
    /// an annotation, which is the whole point of the substrate. A `markup.*`
    /// command *arms a tool*; the tool draws; the drag raises this.
    ///
    /// # ★ Why that matters more here than for any other action
    ///
    /// The old shell had the other arrangement and it produced the defect the
    /// markup work exists to fix: its `Action::AddMarkupShape` derived a
    /// rectangle from the page's own media-box centre and inserted it, so the
    /// shape appeared in the middle of the page *"no matter where the operator
    /// had been pointing"*. The operator's report was exact — **"they just drop
    /// things into the center of the pdf window."** An action that carries
    /// geometry the operator never supplied is not a shortcut; it is the
    /// feature not working, and it passes any test that asks whether an
    /// annotation was added.
    ///
    /// # Units, and why the endpoints are RAW
    ///
    /// `start` and `end` are **PDF user-space** points, Y-**up**, produced by
    /// [`crate::canvas::markup::endpoints`] — the one place a markup drag
    /// crosses out of canvas space. They are the drag's endpoints **in drag
    /// order**, deliberately un-normalised: for an arrow, `start` is the tail
    /// and `end` is the head, and normalising them into a rectangle here would
    /// silently reverse every arrow drawn up-and-left or up-and-right.
    /// [`crate::canvas::markup::spec`] normalises per kind, at the one moment a
    /// rectangle is actually needed, and carries the full argument.
    ///
    /// # Why the page travels
    ///
    /// The same reason it does on [`Self::DeleteSelection`]: an action is a
    /// complete statement of intent, resolvable after the frame that raised it.
    /// Re-deriving the page from `doc.view.page_index` in the apply would be a
    /// second source of truth that is right until a page step raised in the
    /// same frame is applied first.
    /// **Author a ce dimension on the page** — the release of a completed
    /// measure pick.
    ///
    /// Raised by [`crate::canvas::measure`] when a pick machine returns a
    /// `DimensionKind`, which for the linear tool is the **third** click (what,
    /// to what, and where it sits) and for the others is the pick that first
    /// makes the geometry knowable.
    ///
    /// # ★ Why the geometry arrives whole rather than as points
    ///
    /// `DimensionKind` is `pdfce-core`'s own type and it is carried across
    /// unchanged, which is the property the salvage's two equivalence tests
    /// exist to protect: the value built here is **byte-for-byte the one
    /// `pdfce-cli dimension-add` builds** from the same picks, so a dimension
    /// authored on the canvas and one authored from the command line are the
    /// same bytes in the file. Decomposing it into coordinates here and
    /// rebuilding it in the apply arm would put a second constructor in the
    /// path and quietly end that guarantee.
    ///
    /// This is also why the variant carries no colour, width or standard: those
    /// live on the **group**, which is why `group` is the other field.
    CommitDimension {
        /// Page index the dimension is placed on.
        page: usize,
        /// The authoring group it joins, which is what carries its scale,
        /// number format and drafting standard.
        group: pdfce_core::dimension::GroupId,
        /// The immutable geometry, straight from the pick machine.
        kind: pdfce_core::dimension::DimensionKind,
    },
    /// **Author one markup annotation on the page** — the release of a band
    /// drag, the release of a freehand stroke, or the ending of a vertex run.
    ///
    /// # ★ Why THREE gestures share one variant, where text markup got its own
    ///
    /// Because [`crate::canvas::markup::spec`] is *"the single place a gesture
    /// becomes a `MarkupSpec`"*, and that claim is what the equivalence with
    /// `pdfce-cli markup-add` rests on: a canvas-authored annotation has to be
    /// byte-identical to a CLI-authored one, and the cheapest way to keep two
    /// things identical is for there to be one of them. Three variants would be
    /// three apply arms, each free to build its own spec, and the day one of them
    /// acquired a normalisation the others did not is the day the guarantee
    /// quietly stopped holding — with nothing to notice it, because every arm
    /// would still author a perfectly valid annotation.
    ///
    /// So the geometry became an enum
    /// ([`crate::canvas::markup::Geometry`]) rather than the variant becoming
    /// three. [`Self::CommitTextMarkup`] stays separate for the reason its own
    /// docs give and the reason is *different*: its operand is not a gesture at
    /// all — it is a text selection that already exists on the document — so it
    /// shares no rule with anything here.
    CommitMarkup {
        /// The 0-based page the annotation is authored onto.
        page: usize,
        /// Which shape — and therefore which `/Subtype`, pen and normalisation
        /// rule. See [`crate::canvas::markup::spec`].
        kind: crate::canvas::markup::MarkupKind,
        /// The geometry the gesture produced, **in PDF user space**: two raw
        /// drag endpoints, a run of clicked vertices, or one or more freehand
        /// strokes. Which of the three is a property of the kind, and the pairing
        /// is checked by [`crate::canvas::markup::action`] before this is ever
        /// built.
        geometry: crate::canvas::markup::Geometry,
        /// ★ **The pen the operator had when the gesture completed**, carried
        /// in the action rather than read at apply time.
        ///
        /// The funnel's whole premise is that an `Action` is *plain data
        /// describing an edit*, and the colour and width are part of what the
        /// edit is — not context to be looked up later. Reading the live pen in
        /// the apply arm would author a mark in whatever colour the operator
        /// happened to have selected by the time the queue drained, which for a
        /// queue is a real gap and not a theoretical one: the dispatcher raises
        /// actions during the frame and `apply` runs at the end of it.
        ///
        /// It also makes the action **replayable**, which the variant's own
        /// docs already claim of the rest of its fields: an `Action` a test
        /// builds, or a future undo/redo surface re-runs, authors the same
        /// annotation it did the first time rather than the same shape in a
        /// different colour.
        pen: crate::canvas::markup::pen::Pen,
    },

    /// ★ **Calibrate a dimension group** — say what its numbers mean.
    ///
    /// Raised by `crate::dialogs::scale` and by nothing else.
    ///
    /// # Why this is an `Action` and not a call
    ///
    /// `EditSession::set_group_scale` **re-propagates every member's baked
    /// appearance stream**. A dimension's label is drawn into its `/AP`, so
    /// changing the scale rewrites every dimension in the group — which may be
    /// dozens of annotations across several pages.
    ///
    /// That makes it a document edit with an undo step, and the funnel's whole
    /// purpose is that such an edit is ordered against every other and appears
    /// **once** in the command log. One `Ctrl+Z` undoes a recalibration,
    /// whatever it touched. That is the group model's own promise — *a group
    /// exists so its members agree* — and a dialog issuing one call per member
    /// would break it in the most annoying way available: an undo stack the
    /// operator has to press forty times.
    ///
    /// # Why it carries no page
    ///
    /// Every other document-editing variant here names one. A group is
    /// **document-scoped by construction**: its members may be on any page, and
    /// the sidecar that records it is not a page property. Adding a page here
    /// would be a field the apply arm had to ignore, which is how a reader
    /// comes to believe a recalibration is page-local.
    SetGroupScale {
        /// The group to recalibrate.
        group: pdfce_core::dimension::GroupId,
        /// The tri-state scale to store — always `Calibrated` from this
        /// dialog, because a back-calculated scale is by definition neither
        /// "1:1" nor "never set".
        scale: pdfce_core::dimension::ScaleState,
        /// The number format: the display unit, and how its fractional part is
        /// written.
        format: pdfce_core::dimension::NumberFormat,
    },
    /// ★ **Mark the text the operator has selected** — underline, strikeout or
    /// squiggly.
    ///
    /// Raised by `crate::app::dispatch` when one of the three Text markup
    /// commands is invoked, through
    /// [`crate::canvas::markup::text::mark`], which is where every rule about
    /// *which* selection is eligible lives.
    ///
    /// # Why this is not [`Self::CommitMarkup`] with a different kind
    ///
    /// Because the operand is a different shape and no amount of naming hides
    /// it. `CommitMarkup` carries **two points** — a drag — and normalises or
    /// preserves them per kind; this carries **a list of quads**, one per line
    /// of a text selection, already in PDF user space and already grouped. A
    /// single variant would have to carry both and leave half of itself empty
    /// for every value, which is the shape that makes an apply arm ask *which
    /// kind is this again* before it can read its own operands.
    ///
    /// # Why the quads travel, and why the page travels with them
    ///
    /// An action is a complete statement of intent, resolvable after the frame
    /// that raised it — the property [`Self::DeleteSelection`] is built on. The
    /// selection it came from may be cleared by the same frame's Escape or
    /// replaced by a click before this is applied, so the quads are copied out
    /// at the moment the operator asked. The `page` is the **selection's**, not
    /// `doc.view.page_index`: a selection made on one sheet and marked after
    /// paging away must mark the sheet it was made on.
    CommitTextMarkup {
        /// The 0-based page the annotation is authored onto — the page the
        /// selection was made on.
        page: usize,
        /// Which subtype, and therefore which appearance the engine draws.
        kind: crate::canvas::markup::text::TextMarkKind,
        /// The selected lines' boxes, PDF user space, in content order —
        /// `crate::canvas::textsel::TextSelection::page_quads`, which is the
        /// same list the wash was painted from.
        quads: Vec<pdfce_core::annot_author::Quad>,
    },
    /// ★ **Replace the words in ONE show operator** — `DEFECTS.md` D4's verb.
    ///
    /// One operator, one `EditSession::edit_text`, one undo entry. The scope
    /// limit is `pdfce-core`'s and is stated on `EditRequest`: a request pins to
    /// one show operator, and a `TJ` array is one operator. A caret that landed
    /// where two runs meet never becomes this variant —
    /// `canvas::textedit::Refusal::SpansRuns` refuses it in a sentence first.
    ///
    /// # Why it carries the ORIGINAL as well as the replacement
    ///
    /// Because the engine's request is a find/replace, not an index-and-splice:
    /// `EditRequest::find` is *"the text to locate within one show operator's
    /// decoded run"*, and the surgery re-tokenises the content buffer to find
    /// it. The `run` index alone would not survive the round trip.
    ///
    /// # Why it does NOT carry the disposition
    ///
    /// That is the whole of D4b, so it is worth stating where it is not.
    /// `FollowerDisposition` is derived at apply time by
    /// `canvas::textedit::plan`, from the page **as it is when the action
    /// lands** — not from what the canvas believed when the key was pressed.
    /// Carrying it would make the choice a fact about a frame; deriving it makes
    /// it a fact about the document. The old shell's failure was of exactly the
    /// second kind read the first way: it wrote `EditOptions::default()` at its
    /// single call site and never asked the page anything.
    CommitTextEdit {
        /// The 0-based page holding the run.
        page: usize,
        /// Which run, by index into `PageText::runs` — the anchor for the
        /// provenance pin `plan` re-derives.
        run: usize,
        /// The run's text when the caret landed on it. `EditRequest::find`.
        original: String,
        /// What the operator typed. `EditRequest::replace`.
        replacement: String,
    },
    /// **Place NEW page text** — `edit.add_text`'s verb.
    ///
    /// Additive, and that is the difference from [`Self::CommitTextEdit`] rather
    /// than a detail: it rewrites no existing operator, so there is nothing whose
    /// form changed and the engine's own R46 additivity applies. It is
    /// deliberately **not** `Action::CommitMarkup` with a text kind — a markup
    /// text box is an annotation layered over the page and is removable by
    /// deleting it; this becomes the page's own content, exactly like the text
    /// already there, which is what the command's shipped tooltip promises.
    CommitAddText {
        /// The 0-based page.
        page: usize,
        /// Where the baseline starts, in **PDF user space** — the space
        /// `AddTextRequest::origin` is specified in, converted once at the click
        /// through `viewer::canvas_to_pdf_space` so no second conversion can
        /// disagree with the caret the operator saw.
        origin: (f64, f64),
        /// What the operator typed.
        text: String,
    },
    /// Show or hide one optional-content group.
    ///
    /// **View state, not document state.** It changes what this session
    /// draws and never what a save would write — which is why it does not
    /// bump `edit_epoch` and why the Layers panel's own note says a toggle
    /// changes what you see and not the document.
    ///
    /// It does invalidate the page raster, and that is now expressible:
    /// `RenderKey` gained `layers_generation` in the same stage as this
    /// variant, honouring `render/worker.rs`'s rule that *"the key ships in
    /// the same commit as its control"*. Before that, a checkbox here would
    /// have redrawn nothing — which is why the panel shipped without one.
    SetLayerVisible {
        /// The optional-content group.
        group: pdfce_core::object::ObjId,
        /// Whether it should be drawn.
        visible: bool,
    },
    /// Restore every optional-content group to the document's own default.
    ///
    /// Not "show everything": a document may declare groups that are off by
    /// default, and revealing those would be a different act from undoing
    /// the operator's own hiding.
    ResetLayers,
    /// **Choose how many pages are on screen, and in what arrangement.**
    ///
    /// The four positions of View ▸ Page display, as one action carrying which
    /// — because they are a radio and an action per position would be four
    /// arms doing the same thing to four different constants.
    ///
    /// # It is a view stance, and it still goes through the funnel
    ///
    /// Same nature as [`Self::ToggleAnnotations`] and [`Self::SetLayerVisible`]:
    /// it changes what is drawn and nothing a save would write, so it does not
    /// bump `edit_epoch`. It goes through the funnel anyway for the reason the
    /// funnel exists — the mode is changed from a ribbon button *while the
    /// canvas is drawing the old arrangement*, and applying it mid-frame would
    /// leave the frame's layout, its scroll offset and its texture lookups
    /// describing two different modes at once.
    ///
    /// # ★ Applying it does three things, and the third is why this is not a
    /// one-line arm
    ///
    /// 1. **sets `view.display`** — the arrangement itself;
    /// 2. **remembers it against this document**, through
    ///    [`crate::viewer::remembered`], which is the operator's requirement of
    ///    2026-08-12: *"so a sheet set does not inherit a report's setting."*
    ///    Recording it here rather than in the dispatcher is deliberate — a
    ///    customized keymap can reach the command too, and a choice made by a
    ///    chord must persist exactly as one made by a click;
    /// 3. **drops the strip's cached rasters**, because a mode change is the
    ///    one event that makes a *visible* page stop being visible. Leaving
    ///    them would hold GPU memory for pages that cannot be reached until the
    ///    operator switches back.
    SetPageDisplay(crate::viewer::PageDisplay),
    /// Show or hide annotations as a class.
    ///
    /// Same nature as [`Self::SetLayerVisible`] — a view stance, tracked by
    /// `RenderKey`, invisible to a save.
    ToggleAnnotations,
    /// **The three View ▸ Display chrome toggles** — rulers, grid, guides.
    ///
    /// One variant carrying which, rather than three variants, for the reason
    /// [`Self::SetPageDisplay`] gives about the page-display radio: the
    /// operand *is* the command, `crate::shell::commands::chrome_for_command`
    /// is the single binding between an id and a [`ViewChrome`], and its
    /// inverse is what publishes the `selected:` condition that renders each
    /// one pressed. Three arms would be three places for that mapping to be
    /// spelled and a fourth toggle would be added to two of them.
    ///
    /// # Why it goes through the funnel when it changes nothing a save writes
    ///
    /// The same reason `SetPageDisplay` does, and it is sharper here: the
    /// rulers change how much room the canvas has. Applying that in the middle
    /// of the frame that is *already laying the strip out into the old
    /// viewport* would leave the frame's fit scale, its scroll offset and its
    /// page rects describing two different canvases at once. Deferred, the
    /// next frame reserves the gutters once and everything downstream is
    /// consistent with them.
    ///
    /// Deliberately does **not** bump `edit_epoch`: nothing about the document
    /// has changed, only what is drawn beside and over it. Bumping would throw
    /// away the decomposition and the font inventory to no purpose.
    ToggleViewChrome(ViewChrome),
    /// **The operator's guides for this document, after a gesture changed
    /// them.**
    ///
    /// Carries the whole next collection rather than an add / move / remove
    /// verb. Three reasons, and the first is the one that decided it:
    ///
    /// 1. **The gesture already computes it.** `canvas::guides::release`
    ///    resolves create, move and delete through one table, and handing over
    ///    the result is the same "compute the next value from the previous one
    ///    and store it" shape the canvas already uses for the selection.
    /// 2. **The apply has exactly one thing to persist.** `guides.txt` is
    ///    rewritten from the whole set either way — it is a
    ///    read-modify-write of one line — so a verb would be decomposed here
    ///    and recomposed there.
    /// 3. **The operand is small.** Bounded by
    ///    `canvas::guides::MAX_PER_DOCUMENT`, twelve bytes each, and raised
    ///    once per *release* rather than once per frame of a drag.
    SetGuides(crate::canvas::guides::Guides),
    /// **One thing the operator asked Find to do** — run the search, or step
    /// to the adjacent hit.
    ///
    /// # Why a search goes through the funnel at all, when it changes nothing
    ///
    /// Because it needs `&mut EditSession`.
    /// [`pdfce_core::edit::EditSession::find_text_with`] takes a mutable
    /// borrow — it is a read that mutates the session's own working state —
    /// and `OpenDoc::session` is an `Arc` precisely so the render worker can
    /// hold a clone while it rasterizes. `Arc::get_mut` fails while any other
    /// strong reference exists, so the worker has to be stopped first, and
    /// stopping a render **in the middle of laying out a frame** is exactly
    /// what this funnel exists to prevent. Applied after the frame, it is one
    /// short pause in a rasterization that was going to restart anyway.
    ///
    /// So the rule this variant honours is not the letter of
    /// "actions-not-mutations" (a search mutates no document) but its
    /// *reason*: do no expensive or ordering-sensitive work in the middle of
    /// a frame.
    ///
    /// # Why stepping is an action too, when it moves no bytes
    ///
    /// Because it **navigates**: moving to the next hit changes the page and
    /// the scroll offset, which is `Action::GoToPage`'s territory. Doing it
    /// in the widget would put a page change inside the frame that is already
    /// drawing the old page — the one-frame-late class of defect
    /// `crate::app`'s header describes for the whole apply phase.
    ///
    /// The operand is carried, exactly as [`Self::DeleteSelection`] carries
    /// its index list, because an action is a complete statement of intent:
    /// *which* way to step cannot be re-derived after the frame that asked.
    /// See `crate::find` for what happens on the other end.
    Find(crate::find::FindRequest),
    /// **One form-field edit**, as one undoable command.
    ///
    /// The variant `crate::panels::forms` raises for every one of its verbs —
    /// fill, toggle, choose, reset, regenerate appearances, flatten — carrying
    /// the whole intent so it is resolvable after the frame that raised it, in
    /// the same way [`Self::DeleteSelection`] carries its operand list.
    ///
    /// # Why the arm below is one line and not four
    ///
    /// It does not go through [`vector_edit`], and `crate::panels::forms::edit`'s
    /// own header carries the reason: the six form outcome types do not unify
    /// into `Result<Vec<String>, EditError>`, so that module performs the
    /// cancel-mutate-bump-invalidate protocol itself, once, for all of them.
    /// A second copy of the protocol here would be the fifth hand-written
    /// instance of a four-step sequence `vector_edit` exists to have exactly
    /// one of.
    Form(crate::panels::forms::edit::FormEdit),
    // =======================================================================
    // ★ THE REDACTION MARKING VERBS
    //
    // Three variants, all **reversible**, and that is the property that puts
    // them in this enum at all. Marking authors a `/Redact` annotation and
    // removes nothing; the engine records each one as an undoable command, so
    // every one of these goes through `vector_edit` exactly as a markup does
    // and `Ctrl+Z` takes it back.
    //
    // The **irreversible** half is deliberately not here and must never be.
    // Applying a redaction writes a new file and changes no document, so it
    // contributes nothing to the undo log, has nothing to order against, and
    // has no epoch to bump — `crate::dialogs`' header's test for what belongs
    // in the funnel, and `crate::dialogs::redact` fails all three parts of it.
    // An `Action::ApplyRedactions` would also mean the one operation in this
    // program that cannot be undone travelling as plain data through a queue
    // that a future replay, a macro or a test could re-run.
    // =======================================================================
    /// **Mark every occurrence of some text for redaction.**
    ///
    /// Raised by [`crate::panels::redact`]'s Find & mark control. Applied
    /// through `vector_edit`, so it is one undoable command however many marks
    /// it creates — which is the right granularity: the operator asked one
    /// question, and taking back "mark every occurrence of this name" one
    /// annotation at a time would be unusable.
    ///
    /// # ★ The query is carried, not a hit list
    ///
    /// The panel could resolve the matches itself and push the quads, the way
    /// [`Self::CommitTextMarkup`] carries the selection's boxes. It must not,
    /// for a reason specific to this verb: `pdfce-core`'s own
    /// `mark_redactions_by_search_with` documents the trap — a front end whose
    /// search and whose marking disagree about *which hits exist* produces
    /// "three highlights and eleven redaction marks", and *"on the one
    /// operation whose whole purpose is removing content irreversibly, 'the
    /// mark set is a superset of the highlight set' is not a cosmetic
    /// difference."* Handing the engine the query lets the engine answer both
    /// halves with one scan.
    MarkRedactionsBySearch {
        /// The text, already trimmed by the panel.
        query: String,
        /// Whether to read the query as a pattern (`#` any digit, `?` any
        /// character) rather than as literal text.
        ///
        /// A `bool` here rather than an enum, unlike
        /// `crate::redact::ResidualAcknowledgement` — because this one is
        /// *named at its field* and reads as a sentence at the one call site
        /// that builds it, while that one is a positional argument at a call
        /// site where a transposition would write a file.
        pattern: bool,
    },
    /// **Mark the whole of one page for redaction.**
    ///
    /// Raised by [`crate::panels::redact`]'s Mark whole page control. The page
    /// is carried rather than read from `doc.view` at apply time, on
    /// [`Self::CommitTextMarkup`]'s rule: the operator marked the sheet they
    /// were looking at, and an action applied after a frame in which they also
    /// paged away must mark the sheet they meant.
    ///
    /// The rectangle is not carried, because it is not the operator's choice —
    /// it is the page's crop box, and `crate::panels::redact::whole_page_spec`
    /// is the one place that decision is made and tested.
    MarkPageForRedaction {
        /// The 0-based page to cover.
        page: usize,
    },
    /// **Take one redaction mark off.**
    ///
    /// Raised by a row's Remove control. The engine's
    /// `EditSession::delete_redaction_mark` rather than its general annotation
    /// delete, deliberately and on core's own instruction: the two record
    /// different `CommandKind`s so that an undo tooltip can say *"remove a
    /// redaction mark"* rather than *"delete annotation"*, and — as that
    /// method's docs put it — *"I decided not to redact that"* is a different
    /// claim from *"delete annotation"*.
    ///
    /// The **annotation id**, not a row index: a list position is a position in
    /// a census rebuilt every frame, and by the time the apply phase runs the
    /// same index may name a different mark. `crate::app` §10's rule —
    /// *selection is an identity, not a position* — applied to a list.
    RemoveRedactionMark {
        /// The `/Redact` annotation to delete.
        annot_id: pdfce_core::object::ObjId,
    },
    /// **Take back the most recent change** — the other end of the command log
    /// every mutating action in this enum writes to.
    ///
    /// Raised by `edit.undo`, which is on the quick-access toolbar in every
    /// mode and bound to `Ctrl+Z`. Applied by `apply::history_step`.
    ///
    /// # ★ Why this is an action at all, when it is "just" a method call
    ///
    /// Because `EditSession::undo` takes `&mut self` and
    /// [`crate::app::state::OpenDoc::session`] is an `Arc` — held by the render
    /// worker while it rasterizes — so `Arc::get_mut` fails unless the worker is
    /// stopped first. Stopping a render **in the middle of laying out a frame**
    /// is exactly what this funnel exists to prevent, and it is the same
    /// argument [`Self::Find`] makes for a search that mutates no document.
    ///
    /// It is also the ordinary one: an undo changes the document, so it changes
    /// the decomposition, the page-text cache, the font inventory and the
    /// canvas selection, and every one of those is invalidated by the epoch
    /// bump that only the apply phase may perform.
    ///
    /// # Why it carries nothing
    ///
    /// Because the operand is the log's own top, and the log is on the session.
    /// Carrying a depth or a `CommandKind` would be carrying a *copy* of state
    /// the apply is about to read anyway — and a stale copy at that, since a
    /// frame's earlier action can push a command between the raise and the
    /// apply. `crate::app::actions`' rule is that an action is a complete
    /// statement of **intent**; "take back the last thing" is complete.
    ///
    /// # An empty log is not an error
    ///
    /// `undo.available` greys the control, so a *click* cannot reach an empty
    /// log — but `Ctrl+Z` can, from any mode, because
    /// `crate::app::modes::capability::offers_command` lets a command on no tab
    /// through everywhere. The apply arm declines it in words rather than in
    /// silence; see `crate::app::status::decline::Declined::NothingToUndo` for
    /// why that is worth a sentence when the greyed control is already there.
    Undo,
    /// **Re-apply the most recently undone change.**
    ///
    /// Raised by `edit.redo`, bound to both `Ctrl+Y` and `Ctrl+Shift+Z`, and
    /// applied by the same `apply::history_step` — one function, one direction
    /// parameter, because the two differ in exactly which engine verb they call
    /// and in nothing else. Two arms would be two copies of the guard, the
    /// trace and the decline, and one of them would eventually acquire a step
    /// the other did not.
    ///
    /// Everything in [`Self::Undo`]'s docs applies unchanged. The one fact
    /// worth stating separately is the redo stack's own lifetime: the engine
    /// clears it whenever a new command is recorded (`EditSession::commit` —
    /// *"the redone future no longer exists once history diverges"*), so a
    /// redo that was available before an edit is not available after it, and
    /// the condition follows on the next frame with nothing here to remember.
    Redo,
    /// **Invoke a registered command by id**, from a surface that is not the
    /// ribbon.
    ///
    /// ★ The one variant that is not a statement about the document. It exists
    /// so a *second route to an existing command* cannot become a second
    /// implementation of it: the Find bar's OCR offer means exactly what
    /// `file.ocr` on the ribbon means, and wiring it straight to
    /// `DialogsState::open_ocr` would have put that command's guards in two
    /// places — the failure `crate::app`'s one-choke-point invariant names.
    ///
    /// **Drained during the frame, never in [`PdfceApp::apply_actions`]**, for
    /// two reasons that are hard rather than stylistic: `dispatch_command`
    /// needs an `&egui::Context` and the apply phase is deliberately given
    /// none, and a dialog it opens must be drawn by `DialogsState::show` on the
    /// same frame. The drain and its full argument are at the call site in
    /// `crate::app`; the arm below exists only to notice if it is ever removed.
    Command(String),
}

// ---------------------------------------------------------------------------
// ★ EVERYTHING BELOW THIS LINE IS TEST-ONLY, AND THAT IS A GATE REQUIREMENT
//   RATHER THAN A HOUSE STYLE.
//
// `tools/gates/check-ui-strings.sh` truncates each file at its FIRST
// column-0 `#[cfg(test)]` and scans nothing after it — its own header states
// the limit in as many words ("any non-test code placed AFTER the test module
// is invisible to the checker") and records the day a planted violation
// failed to fire because of it.
//
// So a `#[cfg(test)]` item in the MIDDLE of a file silently disarms rule R1
// for the rest of that file. `plant_edit_disclosure_for_test` was written
// beside the store it plants into, next to `record_edit_disclosure` — which
// would have put the attribute at line 244 of 1,253 and left this module's
// entire `Action` enum, its doc comments and every `format!` in
// `PdfceApp::apply` unscanned. Measured, not assumed: a violation planted
// after such a line passes the gate.
//
// Keeping the test-only helper here, below all real code, costs one level of
// distance from the thing it plants into and buys back a thousand lines of
// coverage.
// ---------------------------------------------------------------------------

/// Plant a disclosure, for tests in other modules that must draw one.
///
/// `#[cfg(test)]` so it cannot become a second way to record one — the real
/// path is [`record_edit_disclosure`], called from [`vector_edit`] with the
/// epoch the edit produced, and a second entry point is how two callers come
/// to disagree about what "the last edit" means.
///
/// It exists because the status bar draws this and must prove it does not grow
/// the bar while doing so (R128), and that measurement has to happen in
/// `crate::app::status`, which cannot reach a `thread_local` here. Exactly the
/// reason `crate::panels::forms::edit::plant_fill_disclosure_for_test` exists,
/// which is the shape this follows.
#[cfg(test)]
pub(crate) fn plant_edit_disclosure_for_test(disclosure: EditDisclosure) {
    record_edit_disclosure(Some(disclosure));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★ **`edit.undo` and `edit.redo` raise actions rather than falling
    /// through to `command-unimplemented`.**
    ///
    /// The dispatch link, and the one this pair spent the whole project
    /// missing. It is `crate::app::files`'
    /// `the_save_copy_command_raises_the_save_action` for the other two
    /// commands that were registered, drawn on the quick-access toolbar, bound
    /// to a chord, and wired to nothing — and it is written the same way for
    /// the same reason: through `PdfceApp::dispatch_token` with the token the
    /// **ribbon** would raise, so a build that renamed the id or reassigned the
    /// token fails here rather than shipping a control whose press is traced
    /// and discarded.
    ///
    /// # What it deliberately does not assert
    ///
    /// That the actions *do* anything. Two arms that pushed the wrong variant
    /// would pass a test written as "some action was raised", which is why the
    /// comparison is against the exact vector — and what each variant does when
    /// applied is `crate::app::actions::apply`'s
    /// `an_undo_is_an_edit_and_moves_the_epoch_like_one`, on a real fixture with
    /// a real edit on the log.
    ///
    /// # Why an EMPTY log is the state under test here
    ///
    /// Because the dispatcher must not consult one. `undo.available` greys the
    /// control and the apply arm declines an empty stack in words — both of
    /// which are somebody else's job — and an arm that checked the session here
    /// would be the second place that question is asked. So the action is raised
    /// with nothing to undo, exactly as it would be for a `Ctrl+Z` fired at a
    /// freshly opened document, and the decline happens downstream.
    #[test]
    fn the_history_commands_raise_actions() {
        let ctx = egui::Context::default();
        let mut app = crate::app::tests::opened();

        for (id, expected) in [("edit.undo", Action::Undo), ("edit.redo", Action::Redo)] {
            let token = app
                .commands
                .get(id)
                .unwrap_or_else(|| panic!("`{id}` must be registered")) // ui-text-exempt: test panic
                .handler;
            let mut actions = Vec::new();
            app.dispatch_token(&ctx, token, &mut actions);
            assert_eq!(
                actions,
                vec![expected],
                "`{id}` must raise its action rather than falling through to \
                 `command-unimplemented`, which is what it did for the whole life of the project"
            );
        }
    }
}
