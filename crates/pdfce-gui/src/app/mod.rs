//! # app — the one owner of state, and the shape of a frame
//!
//! [`PdfceApp`] is the single owner of everything the shell knows. There is
//! no global, no `thread_local`, no second source of truth: if it is state,
//! it is reachable from here, and if it is reachable from here it has one
//! owner. That is what makes the action funnel in [`actions`] enforceable —
//! a widget cannot mutate a document it has no path to.
//!
//! ## The order of a frame, and why the order is load-bearing
//!
//! [`PdfceApp::update`] runs four steps, in this order, every frame:
//!
//! 1. **Collect keyboard actions** ([`keyboard::collect`]) and dispatch the
//!    frame's **manifest chords** ([`keyboard::commands`] →
//!    [`PdfceApp::dispatch_command`]) — before any widget is built, so the
//!    map sees the frame's raw key presses rather than whatever survived a
//!    widget consuming them. The split between the two is the subject of
//!    [`keyboard`]'s ★ section: chords the manifest keymap binds arrive as
//!    command ids and go through the same dispatcher a ribbon click does,
//!    and chords the viewer owns outright arrive as actions.
//! 2. **Compose the panels** — draw, and let each surface push more
//!    actions, then the Find overlay over the top of them. Nothing mutates.
//! 3. **Apply the actions** ([`PdfceApp::apply_actions`]) — after the frame
//!    is drawn, in one place, in the order raised.
//! 4. **Settle and rasterize** ([`PdfceApp::settle_and_rasterize`]) — decide
//!    whether the cached texture still matches the (now updated) view
//!    state, and start a render if not.
//!
//! Step 4 must come after step 3 or every zoom would be rasterized one
//! frame late — visible as a page that always lags the operator's last
//! action by a frame. Step 3 must come after step 2 because that is the
//! actions-not-mutations invariant itself.
//!
//! ## Panel composition order is load-bearing (layout *and* focus order)
//!
//! egui resolves panels in the order they are added, and that order
//! determines both the rectangles they get and the order the Tab key visits
//! their widgets. **S0 adds exactly one panel** — the `CentralPanel` — so
//! there is nothing to order yet. The rule is written down here anyway,
//! where the composition lives, because it is the thing the old shell got
//! bitten by and it constrains every panel S2 and S3 add:
//!
//! > A full-width bar (toolbar, status bar) must be added **before** any
//! > side panel, or it starts at the side panel's edge instead of spanning
//! > the window. A status bar that does not span the window is not a status
//! > bar.
//!
//! That constraint conflicts with the Tab order a reviewer would prefer
//! (toolbar → canvas → footer), and in the old shell the layout property
//! won, deliberately. The same trade will be made at S2, and it should be
//! made with this paragraph in front of whoever makes it.
//!
//! ## What this build does not have, stated so it is not mistaken for an
//! oversight
//!
//! The list S0 opened with — *"no ribbon, no QAT, no dock, no status bar, no
//! find bar, no dialogs, no Open command"* — is down to **the save**. The find
//! bar is [`crate::find`], reached by Ctrl+F and by the status bar's Find
//! toggle. **Open, Close and Recent are wired**: the picker and its
//! diagnostics seam are [`files`], the list is [`recent`], and both arrive
//! through [`actions::Action::Open`] like everything else. What is left of
//! that sentence is scheduled in `PROJECT_PLAN.md` §4, and the
//! "no placeholders" invariant still applies to all of it: unavailable
//! renders **nothing** rather than a greyed-out control that explains itself
//! badly.

pub mod actions;
/// Where a document made by `file.new` comes from: the 443-byte blank-A4
/// template that ships as an asset, and the argument for why New parses a file
/// rather than the engine growing a way to create one.
pub mod blank;
pub mod cache;
pub mod conditions;
pub mod dispatch;
/// How a path gets from an operator — or from a scripted harness — to
/// [`actions::Action::Open`]. The picker, the diagnostics seam that answers
/// it without a human, and the dirty-document rule.
/// ★ **Files dragged onto the window.** Nothing in this shell read
/// `dropped_files` until 2026-08-19, so a dropped file did nothing, silently —
/// and a drop that is ignored teaches an operator that the program does not
/// accept drops, which is a conclusion they will not revisit.
pub mod dropped;
pub mod files;

/// ★ The per-frame update — `eframe`'s entry point, and the one order the
/// frame's eleven steps may happen in.
///
/// Split out of this file on 2026-08-17 at rule R2's ceiling. The seam is the
/// question each half answers: this file is *what the application is*, that one
/// is *what happens sixty times a second*. Almost every comment in it is about
/// sequence, which is the class of bug that needs a file of its own to be
/// visible.
pub mod frame;
pub mod gating;
pub mod keyboard;
// Opening a document, closing it, and the three ways an open can fail. Split
// from `state.rs` under R2 along the seam those two subjects already drew:
// that file is *what an open document is*, this one is *the document's lifetime
// on the application*.
pub mod lifecycle;
pub mod modes;
pub mod panels;
pub mod persistence;
/// ★ The **selection filter**, on disk — where it lives, and why it is written
/// immediately where the dock layout is debounced.
///
/// The difference is the whole of the module: a splitter drag reports a change
/// on every frame of the gesture, while a filter can only change on a discrete
/// click, so one decision already equals one write. Its header also carries the
/// three on-disk states and why *an empty file* must never be collapsed into
/// *no file* — that would silently overrule a deliberate choice every restart.
pub mod pickstore;

/// ★ The shell's OWN preferences — how pdfce draws, as distinct from how it
/// reads and writes PDFs.
///
/// A separate store from `pdfce_core::settings` on purpose: that one exists
/// because a **standard declines to have an opinion**, and every entry cites
/// the clause that is silent. How sharply a page is rasterised cites nothing.
/// Its header also records which five of the seven commissioned View ▸ Render
/// settings turned out to have nothing behind them, and why.
pub mod prefs;
/// The documents this operator had open: the capped, persisted list and the
/// ribbon control that draws it.
pub mod recent;
/// Writing a copy of the open document to a file the operator names — the body
/// of `file.save_copy`. Why the save mode is **incremental**, why nothing on
/// `OpenDoc` moves when one succeeds, and why the picker runs in the apply
/// phase rather than in the dispatcher.
pub mod save;

/// ★ The operator's configuration, and the **funnel** that makes it reach the
/// engine.
///
/// Not a struct-holder. Of the thirteen settings the old shell persisted,
/// **nine were never read by anything** — they were saved, loaded, shown,
/// edited, and then discarded at every call site that wrote
/// `ExtractOptions::default()` or `RenderOptions::default()` or
/// `SaveOptions::identity()`. This module owns the three replacements and a
/// `syn` check that no other file may bypass them.
pub mod settings;

/// **More than one document open at once** — the tab arithmetic behind the
/// document strip, and what switching between documents forgets.
///
/// The operator's request of 2026-08-19. See that module's header for why the
/// active document stayed on its own field rather than becoming
/// `documents[active]`, and for the invariant that makes `Status::Empty` mean
/// "no documents" rather than "one empty document".
pub mod documents;

/// The **document tab strip** — the surface `documents` is drawn on.
pub mod doctabs;

/// The host half of the Settings window — what pressing Save, Cancel or
/// Restore defaults actually does, and the order the four consequences of a
/// Save happen in.
pub mod settings_window;

pub mod state;
pub mod status;

/// **The three regions the application draws** — the ribbon band, the docks
/// and the central area.
///
/// Split out on 2026-08-20 along the seam this file's own header had already
/// drawn in prose. See that module's header; the one-line version is that
/// `mod.rs` answers *what is the application and how is it built* and
/// `surfaces.rs` answers *what does it draw*.
pub mod surfaces;
/// Read mode and full screen — the two View ▸ Window verbs that change the
/// shape of the application rather than anything about the document.
pub mod window;

use state::Status;

/// Named region: the whole central panel, in window logical points.
///
/// The outermost region this crate owns today. When the ribbon, the docks and
/// the status bar land, each declares its own — see [`crate::diag::ui_rect`]
/// for the seam and for the naming rule.
const REGION_CENTRAL_PANEL: &str = "central-panel"; // ui-text-exempt: trace region name, never displayed

/// Named region: the one-sentence explanation shown when nothing is open, or
/// when the document could not be opened.
///
/// One name for all four non-open arms. A check asking "is the shell's
/// explanatory text legible?" is asking about the region, not about which of
/// the four sentences happens to be in it — and the four are laid out
/// identically, so they are genuinely the same region.
const REGION_STATUS_MESSAGE: &str = "status-message"; // ui-text-exempt: trace region name, never displayed

/// Trace slot: what the dock drew this frame.
///
/// De-duplicated on the rendered line, so a dock that is not changing costs
/// one line rather than one per frame — the lesson `canvas-pointer` taught
/// when a stationary pointer emitted fifty identical lines in nine seconds.
const DOCK_SLOT: &str = "dock"; // ui-text-exempt: trace slot name, never displayed

/// The whole application state.
///
/// One field at S0. It will grow — settings, the command log, the dock
/// layout, the parked documents — and the discipline that keeps it
/// comprehensible is that every field carries a doc comment saying what it
/// is for and, where it is not obvious, why it is *here* rather than inside
/// [`state::OpenDoc`]. The rule of thumb: state that dies with the document
/// lives on `OpenDoc`; state that outlives it lives here.
/// ## ★ No `Default`, as of 2026-08-17
///
/// It derived one until the settings store arrived, and nothing in the
/// workspace ever called it — checked, not assumed. Removing it is the right
/// answer rather than a workaround for `StoreLocation` having no `Default`:
///
/// A defaulted `PdfceApp` would have **no shell manifest, no command registry,
/// no panel registry and no settings store** — a state [`PdfceApp::new`] can
/// never produce and every method here assumes away. Deriving a constructor
/// for an unreachable state is how a test ends up asserting something about a
/// program that cannot exist, and this crate has a standing preference for
/// making such states unrepresentable rather than merely unused.
///
/// `new()` is the constructor. It is the only one.
pub struct PdfceApp {
    /// What, if anything, is open — **the document the operator is looking
    /// at**, when several are open.
    ///
    /// Unchanged in meaning by the multi-document work of 2026-08-19, and
    /// deliberately so: every panel, the canvas, the status bar and the
    /// condition set read this one field and none of them had to learn that
    /// other documents exist. [`documents`]' header carries the argument for
    /// keeping it a field rather than folding it into a vector.
    pub status: Status,

    /// **The other open documents**, in tab order with the active one removed.
    ///
    /// Empty for the whole life of a single-document session, which is what
    /// makes the encoding free when it is not being used.
    ///
    /// The invariant, stated once and asserted by
    /// [`PdfceApp::document_count`]: *if this is non-empty, [`Self::status`] is
    /// not [`Status::Empty`]*. Nothing outside [`documents`] may write it —
    /// the two functions there that flatten and rebuild the pair are the only
    /// code that knows the encoding.
    pub parked: Vec<Status>,

    /// **Which tab position [`Self::status`] occupies.**
    ///
    /// `0` whenever nothing is open, and always `< document_count()`
    /// otherwise. It is a position in the operator's strip, not an index into
    /// [`Self::parked`] — see [`documents`] §1 for the picture.
    pub active_slot: usize,

    /// The shell definition — tabs, groups, modes, QAT, keymap — as data.
    ///
    /// Built once at start-up by merging the compiled-in layer with any
    /// operator customization, then never mutated by a frame. It is the
    /// single source of truth for *where a command appears*; the registry
    /// below is the single source of truth for *what exists at all*.
    ///
    /// `None` only if the built-in manifest failed to validate, which is a
    /// programming error rather than an operator-reachable state: the
    /// ribbon then does not render and the status surface says why. It is
    /// deliberately not a `panic!` — a validation bug in one tab should not
    /// cost the operator the ability to open and read a document.
    pub shell: Option<egui_shell::manifest::Shell>,

    /// Every command this build actually has.
    ///
    /// **This is how a removable capability disappears** (`R8`,
    /// `SHELL_FRAMEWORK.md` §5b): a component that is not compiled in
    /// registers nothing, the manifest item naming it resolves to no
    /// command, and the item is dropped. No `#[cfg]` reaches the ribbon,
    /// and a future DLL-loaded module registers through this same call.
    pub commands: egui_shell::commands::CommandRegistry,

    /// Which tab and which mode are showing. Survives frames; does not
    /// survive a restart until layout persistence lands at S3.
    pub ribbon: egui_shell::ribbon::RibbonState,

    /// Which panels exist, and what they are called.
    ///
    /// The dock is generic over an opaque `PanelId`; this is where those
    /// ids acquire a label and a tooltip. It is the panel-side twin of
    /// [`Self::commands`], and it disappears the same way: a panel whose
    /// capability is not compiled in is never registered, so the dock
    /// drops the tab that named it and reports why.
    pub panel_registry: egui_shell::dock::PanelRegistry,

    /// Where the panels are, and which of them is on top of each stack.
    ///
    /// Owned by the operator, not by the application — it is the thing
    /// layout persistence saves and a named workspace restores.
    pub dock: egui_shell::dock::DockState,

    /// Which mode is active, and each mode's remembered arrangement.
    ///
    /// **A mode is a named workspace** (`MODES_AND_PANELS.md` Part 2): Read,
    /// Review and Edit are three defaults the operator then adjusts, and
    /// leaving Edit and coming back restores the arrangement rather than a
    /// default. That is why the mode selector and the dock are not two
    /// independent features — the selector is how you reach a workspace.
    pub modes: crate::app::modes::Modes,

    /// Where the arrangement is written, and when.
    ///
    /// Beside `settings.txt` in the same directory `pdfce-core` resolves,
    /// never a location computed here — so it inherits the portable-first
    /// ordering and the writability probe. Debounced: one splitter drag is
    /// one write, and a continuous drag cannot starve the write out.
    pub layout: crate::app::persistence::LayoutStore,

    /// **The documents this operator had open**, newest first, persisted
    /// beside the layout in the directory `pdfce-core` resolves.
    ///
    /// On [`Self`] rather than on [`state::OpenDoc`] by the rule this struct's
    /// own header states: state that dies with the document lives on the
    /// document, state that outlives it lives here. A recent list that died
    /// with the document would be a list of one.
    ///
    /// Written by exactly one call site — [`PdfceApp::open_path`], on a
    /// successful open — and read by the `recent_files` ribbon item and by
    /// the `file.recent` dispatch arm.
    pub recent: crate::app::recent::RecentFiles,

    /// **The document tab a context menu was opened on**, for the length of
    /// one dispatch.
    ///
    /// `window.close_document` and `window.close_other_documents` act on the
    /// tab that was **right-clicked**, not on the one on screen — which is what
    /// every tab context menu on this desktop does, and what makes them
    /// different commands from `file.close`.
    ///
    /// ★ Parked rather than carried, in the same shape and for the same reason
    /// as [`Self::recent_choice`]: `egui_shell`'s menu reports the operator's
    /// intent as a `HandlerToken` and nothing else, so it has no channel for an
    /// operand. Set immediately before the dispatch and cleared immediately
    /// after, so it can never be read by a command that did not come from a
    /// tab.
    pub tab_menu_target: Option<usize>,

    /// **A `Close others` sequence waiting on an unsaved-edits answer**, and
    /// the tab it is keeping.
    ///
    /// `None` for all but the handful of frames between one document's
    /// question and the operator's answer. See
    /// [`PdfceApp::apply_close_other_documents`], which parks it, and
    /// [`PdfceApp::resume_after_unsaved`], which picks it up.
    ///
    /// ★ A **slot**, kept in step by the loop that parks it, rather than a
    /// path or an identity. The alternative was tried on paper and does not
    /// work: a created document's path is a *name*, so an identity keyed on it
    /// cannot find an `Untitled 2.pdf`, and *"close the others and keep my
    /// unsaved scratch document"* is precisely the case that must not close the
    /// wrong thing.
    ///
    /// ★ Cleared at the top of every close arm, so a cancelled sequence cannot
    /// be picked up later by an unrelated question. A cancel produces no
    /// answer, so it never resumes on its own; the clear is what stops it
    /// waiting around for one that belongs to somebody else.
    pub closing_others: Option<usize>,

    /// **The window title as it was last set**, so it is set again only when
    /// it changes.
    ///
    /// A viewport command is a message to the windowing system; sending one
    /// sixty times a second to assert a string that has not moved is waste
    /// that shows up as work on the platform's own thread rather than in this
    /// process's profile, which makes it the kind of waste nobody finds. One
    /// `String` comparison per frame is the price of not doing that.
    ///
    /// Empty at start-up, which is not a title anything sets — so the first
    /// frame always sends one, and the static title from the viewport builder
    /// is replaced by the derived one immediately.
    pub last_window_title: String,

    /// **How many documents `file.new` has made this session.**
    ///
    /// The ordinal in `Untitled 1.pdf`, `Untitled 2.pdf`, … — see
    /// [`crate::text::files::untitled`] for why they are numbered at all
    /// (Inkscape and SolidWorks number theirs; Acrobat does not) and
    /// [`PdfceApp::new_document`] for where it is incremented.
    ///
    /// On [`Self`] rather than on [`state::OpenDoc`] by this struct's own
    /// rule: state that dies with the document lives on the document, state
    /// that outlives it lives here. A counter that died with the document
    /// would count to one forever.
    ///
    /// Deliberately **not persisted**, unlike [`Self::recent`] and
    /// [`Self::layout`]. It exists so that two documents created in one run
    /// are distinguishable — on screen and, more usefully, in the trace of a
    /// driven run — and an operator returning tomorrow has no use for
    /// yesterday's numbering. Restarting starts again at one, which is what
    /// Inkscape and SolidWorks both do.
    pub created_documents: u32,

    /// The recent document the operator picked **this frame**, waiting for
    /// the command that acts on it.
    ///
    /// # Why a field and not a direct action
    ///
    /// The Recent menu is drawn inside the ribbon's custom-item renderer,
    /// which the shell requires to report intent as an
    /// `egui_shell::HandlerToken` and nothing else — it has no channel for an
    /// operand. So the operand is parked here for the length of one frame and
    /// the token is returned, which sends the choice through
    /// [`Self::dispatch_command`] — the same choke point a ribbon click, a
    /// chord and a context-menu row reach.
    ///
    /// That is the same shape as `file.open`, deliberately: the dialog picks
    /// the operand, the command is the verb. It is emphatically *not* a
    /// half-finished intent living across frames — [`Self::ribbon_band`] sets
    /// it and dispatches in the same statement pair, and the `file.recent`
    /// arm `take`s it, so it is `None` again before the frame ends.
    pub recent_choice: Option<std::path::PathBuf>,

    /// The panels' own working state: the page decomposition and the font
    /// inventory, both of which are caches with no equivalent in
    /// `pdfce-core`.
    ///
    /// **The caches moved onto `OpenDoc` at S4, and `DocKey` was deleted
    /// rather than repaired** — an identity key exists only because a cache
    /// outlives the thing it describes, and inside `OpenDoc` the document's
    /// own lifetime bounds it. What is left here is genuinely panel-view
    /// state: scroll positions, expansion, focus. `open_path` forgets it,
    /// which is why it needs no key of any kind.
    pub panels: crate::panels::PanelsState,

    /// **The Find bar's state** — the query, the search options, whether the
    /// bar is on screen, and the last search's hits.
    ///
    /// Here rather than on [`state::OpenDoc`] by this struct's own rule, and
    /// the rule cuts *through* the subject rather than around it: the query
    /// and the options outlive a document (closing one file and opening
    /// another is the likeliest moment to search for the same term again),
    /// while the hits do not. So the state lives here beside
    /// [`Self::panels`], with the same `forget_document` seam for the half
    /// that dies — and the one piece that is genuinely per-document, the
    /// pending scroll-to-a-hit, lives on `OpenDoc` as `find_reveal`.
    ///
    /// See `crate::find`'s header for the wildcard trap this module exists to
    /// avoid, for why the bar is docked rather than floating over the canvas,
    /// and for what an edit does to a hit list.
    pub find: crate::find::FindState,

    /// ★ **What a click on the page is allowed to land on** — the operator's
    /// selection filter (`OPERATOR_REQUESTS.md` O17).
    ///
    /// Here rather than on [`state::OpenDoc`], by this struct's own rule and
    /// for the same reason [`Self::find`] is: it is a statement about **how
    /// the operator works**, not about a document. Somebody who has switched
    /// the drawing's line work off to reach a buried label wants it still off
    /// when they open the next sheet — closing a file is not a reason to
    /// undo a decision about how you point at things. It is persisted across
    /// sessions for the same reason, and by the same argument
    /// `crate::app::persistence` makes about the dock layout: *a
    /// rearrangeable thing that forgets itself each restart is worse than a
    /// fixed one*, because it charges the rearrangement every session and
    /// teaches the operator not to bother.
    ///
    /// There is therefore **no `forget_document` seam** for this field, and
    /// its absence is deliberate rather than an omission.
    ///
    /// See [`crate::canvas::pick`] for why the filter is subtractive, why
    /// that makes its default a guarantee rather than a choice, and why it
    /// composes with the mode's capabilities as an `AND`.
    pub pick_filter: crate::canvas::pick::PickFilter,

    /// The modal dialogs — currently Print, and the place any other lands.
    ///
    /// Held on the application rather than inside the command arm that
    /// opens it because a dialog outlives the click: it is shown once per
    /// frame from [`Self::update`], **after** the canvas and the docks, so
    /// that it draws over them rather than under.
    ///
    /// That ordering is the one thing about this field worth remembering.
    /// It is also the single exception to the "nothing floats over the
    /// canvas" stance, and an intentional one: a *modal* is not a floating
    /// panel — it takes the frame, does one job, and leaves. The stance is
    /// about surfaces that hover over a document you are still working in.
    pub dialogs: crate::dialogs::DialogsState,

    /// The operator's answers to the thirteen questions the PDF standard
    /// declines to answer, live.
    ///
    /// # ★ This is the LIVE configuration, not what is on disk
    ///
    /// The distinction is load-bearing in one direction: when a save to disk
    /// fails, the session still adopts the choice and says it will not survive
    /// a restart. So this field can be ahead of the file, never behind it, and
    /// the Settings window opens a draft from **this** rather than from a
    /// re-read — an operator whose last save failed must be shown what pdfce is
    /// actually doing, not what it wished it had written.
    ///
    /// Nothing reads these fields directly. Every consumer goes through
    /// [`crate::app::settings::SettingsExt`], which is enforced by a `syn`
    /// check rather than by convention — see that module's header for the nine
    /// settings the old shell persisted and never honoured.
    pub settings: pdfce_core::settings::Settings,

    /// Where [`Self::settings`] is written, resolved once at start-up.
    ///
    /// Cloned from `pdfce_core::settings::resolve_store()`, which is memoised
    /// per process — and that memoisation is a **correctness** property rather
    /// than a performance one. This project's own 2026-08-13 report found the
    /// symptom: two callers in one process disagreeing, the layout store
    /// resolving `Portable` while the recent list resolved `PlatformFallback`,
    /// so two files meant to sit beside each other did not. Holding the answer
    /// here as well means the settings window's store-location line and the
    /// save path cannot diverge even within one frame.
    pub settings_store: pdfce_core::settings::StoreLocation,

    /// ★ **The colour and width the next markup is authored with.**
    ///
    /// `RIBBON_IA.md` §5.5's Style group, which this shell shipped without: the
    /// pen was two hard-coded constants and the manifest's `colour_swatch` item
    /// was declared and never built, so the group rendered an empty caption.
    /// §5.5 named the consequence in advance — *"which is why a placed markup
    /// feels final"* — and it is half of what the operator reported.
    ///
    /// **On the application, not on `OpenDoc`.** A pen is a tool setting, and
    /// an operator who picks green expects the next rectangle to be green in
    /// whatever file they draw it in, exactly as a pencil does not change
    /// colour when you turn the page. See `canvas::markup::pen`'s header for
    /// why it is also not in the settings file.
    pub pen: crate::canvas::markup::pen::Pen,

    /// ★★ **The last form-field settings the operator accepted**, for the next
    /// placement — their *"remember last settings"* of 2026-08-26.
    ///
    /// Beside the pen and for the same reason: this is a **tool setting**, so
    /// it belongs to the operator and their session, not to a document. Someone
    /// who turns the border off expects the next field to have no border
    /// whatever file they place it in, exactly as a pen does not change colour
    /// when you turn the page.
    ///
    /// ★ …and, like the pen, deliberately **not** written to `userdata`. See
    /// `canvas::formfield::draft::Remembered`: a remembered setting that
    /// survived a restart would silently govern a different document days
    /// later, which is the shape of a setting nobody can find the source of.
    pub form_defaults: crate::canvas::formfield::Remembered,

    /// ★ **The shell's own preferences** — render quality and the zoom settle
    /// delay. See [`prefs`] for why these are not in the engine's settings
    /// store, and for the five commissioned neighbours that turned out to have
    /// nothing behind them.
    pub prefs: prefs::Prefs,

    /// The draft the Settings window is editing, if it is open.
    ///
    /// `None` is the normal state and the window renders nothing. `Some` is
    /// both "the window is open" and "here is the working copy" — one field,
    /// because two would admit the state where a window is open with no draft
    /// behind it.
    pub settings_draft: Option<crate::dialogs::settings::Draft>,

    /// This application's own top-level window, as a raw `HWND` cast to
    /// `isize`. `None` when the platform did not report one.
    ///
    /// # ★ Why the shell holds a platform handle at all
    ///
    /// For exactly one purpose: to **own** the driver's printer-properties
    /// dialog, which is a modal window Windows creates on our behalf when
    /// `pdfce-print` calls `DocumentProperties` with `DM_IN_PROMPT`.
    ///
    /// An unowned modal is not a cosmetic problem. It can fall behind the
    /// application's own window, at which point the operator sees a frozen
    /// pdfce with no visible dialog and no way to dismiss the thing blocking
    /// it. The handle is what tells Windows to keep it in front.
    ///
    /// # Why it is captured once, at start-up
    ///
    /// `eframe::Frame` also carries it and is available every frame, which
    /// would mean threading a `&Frame` down through the dialog stack to the
    /// one button that needs it. Captured here it is a plain `isize` — `Copy`,
    /// `'static`, nothing borrowed — and the window it names is created before
    /// the first frame and lives as long as the process.
    ///
    /// # Why `isize` and not a handle type
    ///
    /// Because `pdfce-print` takes an `isize`, deliberately: *"no windowing
    /// dependency is added — `parent` is a raw window handle the caller
    /// already owns, passed as an integer, and this crate never creates a
    /// window."* A crate that must stay free of a windowing dependency cannot
    /// name a windowing type in its signature, and this is the shell holding
    /// up its end of that.
    pub window: Option<isize>,
}

/// The raw `HWND` behind an eframe window, as an `isize`.
///
/// Called once, from `crate::run`, with the `eframe::CreationContext`. See
/// [`PdfceApp::window`] for what it is for and why it is captured there.
///
/// `None` for every non-Win32 handle and for a platform that reports none.
/// That is not an error and is not disclosed: the only caller passes it
/// straight to `pdfce-print`, whose contract already says a null owner is
/// legal.
#[must_use]
pub fn window_handle(source: &impl raw_window_handle::HasWindowHandle) -> Option<isize> {
    match source.window_handle().ok()?.as_raw() {
        raw_window_handle::RawWindowHandle::Win32(win32) => Some(win32.hwnd.get()),
        _ => None,
    }
}

impl PdfceApp {
    /// Build the application, including its shell definition.
    ///
    /// The shell is assembled **once**, here, and not per frame: merging
    /// and validating a manifest walks every tab, group and item, and doing
    /// that sixty times a second to produce a value that cannot have
    /// changed would be a per-frame cost with no per-frame cause.
    ///
    /// Order is load-bearing. Commands are registered **before** the
    /// manifest is merged, because the merge resolves every item against
    /// the registry — that resolution is what makes a capability that is
    /// not compiled in disappear from the ribbon rather than render as a
    /// dead control (`R8`, `SHELL_FRAMEWORK.md` §5b).
    #[must_use]
    #[allow(
        clippy::new_without_default,
        reason = "a defaulted PdfceApp would have no shell manifest, no command registry, no panel registry and no settings store — a state this constructor can never produce and every method assumes away. See the type's own docs." // ui-text-exempt: lint justification, never displayed
    )]
    pub fn new() -> Self {
        let mut commands = egui_shell::commands::CommandRegistry::new();
        crate::shell::commands::register(&mut commands);

        let built_in = crate::shell::manifest::built_in();
        let shell = match built_in.validate_against(&commands) {
            Ok(()) => Some(built_in),
            Err(error) => {
                // Not a panic. A validation bug in one tab must not cost
                // the operator the ability to open and read a document —
                // the ribbon is how you reach commands, not how you reach
                // the page. The trace names the offending item; the status
                // surface will say the ribbon is unavailable once it has
                // somewhere to say it (S2 gives it a home).
                crate::diag::trace(|| {
                    format!(
                        // ui-text-exempt: diagnostic trace, never displayed.
                        "shell-invalid {error}"
                    )
                });
                None
            }
        };

        // What this build does NOT have, stated once at start-up.
        //
        // `PLANNED` and `DIRECTED` are registries of deferred and
        // operator-directed ribbon entries. Reporting their sizes here is
        // not decoration: it is the one place a harness — or a person
        // reading a trace from a machine they cannot see — can learn how
        // much of the specified surface this binary actually carries. It
        // also keeps both lists live, so a stale entry is a compile
        // failure's neighbour rather than an unread comment.
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed.
                "shell commands={} planned={} directed={}",
                commands.ids().count(),
                crate::shell::manifest::PLANNED.len(),
                crate::shell::manifest::DIRECTED.len(),
            )
        });

        // Start in the first mode the manifest declares — Read.
        //
        // `RibbonState::new()` leaves the mode unset, and the shell reads an
        // unset mode as "this manifest has no modes, show every tab". That
        // is right for an application with no modes and wrong for one with
        // three: it renders a mode selector whose three positions govern
        // nothing, and every tab regardless. Setting it here removes the
        // inconsistent state rather than letting the first click repair it.
        //
        // Read rather than Edit because that is the operator's stated
        // intent for the feature — pdfce should open looking like a reader
        // (`MODES_AND_PANELS.md` Part 1). It becomes a setting when the
        // settings surface lands; the default-mode choice is noted there.
        let mut ribbon = egui_shell::ribbon::RibbonState::new();
        if let Some(first) = shell.as_ref().and_then(|s| s.modes().first()) {
            ribbon.set_mode(first.id.clone());
        }

        // Register every panel this build has, and open with the two that
        // answer "where am I" and "what is on this page" — the pair a
        // drafting reviewer reaches for first. Everything else is one
        // click away on View ▸ Panels rather than crowding the first
        // frame; `RIBBON_IA.md`'s progressive-disclosure argument applies
        // to the dock exactly as it does to the ribbon.
        // A panel's dock identity IS its ribbon command's id, and its
        // label and tooltip are that command's.
        //
        // One source of truth, and it buys two properties for free. The
        // tab in the dock and the toggle on View ▸ Panels can never
        // disagree about what a panel is called — they read the same
        // string. And a panel whose command is not registered is simply
        // never registered here either, so a capability that is not
        // compiled in loses its dock tab exactly as it loses its ribbon
        // control (`R8`, `SHELL_FRAMEWORK.md` §5b). The dock's fail-soft
        // loader then drops any saved layout entry naming it, and says so.
        let mut panel_registry = egui_shell::dock::PanelRegistry::new();
        for panel in crate::panels::Panel::ALL {
            let id = panel.command_id();
            if let Some(command) = commands.get(id) {
                let mut info = egui_shell::dock::PanelInfo::new(id, command.label.clone());
                if let Some(tooltip) = &command.tooltip {
                    info = info.with_tooltip(tooltip.clone());
                }
                panel_registry.register(info);
            }
        }

        // The dock's arrangement is not this function's to invent any more.
        //
        // `modes::start` loads the persisted layout, adopts the manifest's
        // first mode, and applies that mode's remembered workspace — or its
        // built-in default on a first run. The hand-built `DockState` this
        // replaces was a placeholder from S3 that could not survive a
        // restart and did not vary with the mode selector, so the selector
        // changed which tabs the ribbon showed and nothing else.
        //
        // Every default is filtered through the live `PanelRegistry`, which
        // is what makes a compiled-out capability lose its dock tab as well
        // as its ribbon control (`R8`) rather than mounting an empty pane.
        let startup = crate::app::modes::start(shell.as_ref(), &panel_registry);
        let crate::app::modes::Startup {
            modes,
            layout,
            dock,
        } = startup;

        // ★ The recent list is loaded for real, EXCEPT under `cfg(test)`.
        //
        // `RecentFiles::default()` points nowhere and can write nothing, which
        // is exactly what a unit test needs: several tests in this crate call
        // `PdfceApp::new()` and then `open_path(engine_fixture(…))`, and
        // without this every `cargo test` run would file
        // `D:\Dev\pdfce\fixtures\…` paths into the operator's own recent list
        // — scribbling on real user state from a test suite, in a folder the
        // operator owns and did not ask us to touch.
        //
        // The list's own behaviour is tested against a temporary directory
        // through `RecentFiles::load_in`, so nothing is left uncovered by
        // this; what is skipped is only the resolution of the real location,
        // which `recent::tests::the_recent_file_lives_beside_the_layout_file`
        // asserts against `pdfce-core`'s answer without loading anything.
        let recent = if cfg!(test) {
            crate::app::recent::RecentFiles::default()
        } else {
            crate::app::recent::RecentFiles::load()
        };

        // ★ Settings, loaded for real EXCEPT under `cfg(test)`, for exactly the
        // reason the recent list above is.
        //
        // `Settings::load` never fails — a missing file, an unreadable one, a
        // broken line or a newer schema all yield defaults with a reason in the
        // report — so the risk here is not a crash. It is that a unit test
        // would run against **the operator's own configuration**, so a suite
        // that passes on this machine could fail on another because somebody
        // had chosen `unmappable_code = omit`. A test asserting what the
        // application does must not depend on what its user prefers.
        //
        // `settings_report` is deliberately dropped rather than stored. Its
        // notes are a start-up disclosure — "line 4 is not a setting and was
        // skipped" — which belongs in the status bar, and wiring that surface
        // is a separate piece of work from making the settings reachable. What
        // must not happen is the notes being *shown in a dialog*: a
        // configuration problem may not stop pdfce opening a file.
        let settings_store = if cfg!(test) {
            pdfce_core::settings::StoreLocation {
                path: None,
                kind: pdfce_core::settings::StoreKind::None,
            }
        } else {
            pdfce_core::settings::resolve_store()
        };
        let (settings, _report) = pdfce_core::settings::Settings::load(settings_store.clone());

        // Same `cfg(test)` reasoning as the settings and the recent list above:
        // a suite that read the developer's own preferences would pass on this
        // machine and fail on another because somebody had chosen `faster`.
        let prefs = if cfg!(test) {
            prefs::Prefs::default()
        } else {
            prefs::Prefs::load().0
        };

        Self {
            status: Status::default(),
            // The operator's saved selection filter, or everything the shell
            // can pick if they have never set one. Never fails; see
            // `pickstore`'s header for the three on-disk states.
            pick_filter: pickstore::load(),
            // Nothing is parked because nothing is open. `document_count()`
            // reads this pair as "no documents at all" rather than "one empty
            // document", which is what keeps the tab strip off the screen
            // until there is something to put in it.
            parked: Vec::new(),
            active_slot: 0,
            shell,
            commands,
            ribbon,
            panel_registry,
            dock,
            modes,
            layout,
            recent,
            // Nothing has been created yet, so the first `file.new` of this
            // session makes `Untitled 1`. Not restored from disk — see the
            // field's own note on why yesterday's numbering is of no use to
            // anybody.
            created_documents: 0,
            // Not a title anything sets, so the first frame always sends one.
            last_window_title: String::new(),
            closing_others: None,
            tab_menu_target: None,
            recent_choice: None,
            panels: crate::panels::PanelsState::default(),
            find: crate::find::FindState::default(),
            dialogs: crate::dialogs::DialogsState::default(),
            settings,
            settings_store,
            settings_draft: None,
            // Filled by `crate::run` the moment the window exists — the
            // constructor runs before it does. `None` here is the honest
            // starting value and is what a test-built app keeps.
            window: None,
            pen: crate::canvas::markup::pen::Pen::default(),
            form_defaults: crate::canvas::formfield::Remembered::default(),
            // ★ Loaded for real EXCEPT under `cfg(test)`, for exactly the
            // reason the settings and the recent list above are: a suite that
            // read the developer's own preferences would pass on this machine
            // and fail on another because somebody had chosen `faster`.
            prefs,
        }
    }
}

/// One-time egui configuration that must happen before the first frame.
///
/// Only one setting so far, and it is not optional: egui's
/// `zoom_with_keyboard` makes Ctrl+Plus/Minus/0 rescale the entire user
/// interface. In a document viewer those chords mean *page* zoom — that is
/// what they do in every browser, in Acrobat, and in every other PDF
/// reader — so egui's handler is switched off and [`keyboard`] handles
/// them. Without this the chords would silently do the wrong thing and any
/// tooltip advertising them would be telling a lie.
///
/// Note that Ctrl+**scroll** is unaffected: egui converts that to a
/// `zoom_delta` in the input state but does not act on it itself, so the
/// canvas is free to interpret it.
pub fn configure_context(ctx: &egui::Context) {
    ctx.options_mut(|o| o.zoom_with_keyboard = false);
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    // ★ Named here rather than at the top of the module: `Action` is used only
    // by these tests since the three drawing surfaces moved to
    // `crate::app::surfaces` on 2026-08-20, and a `use` at module scope that
    // only the test module needs is a `use` that fails the workspace's
    // `-D warnings` clippy gate in a release build.
    use crate::app::actions::{Action, VectorAction};
    use crate::canvas::selection::{ClickHit, SelectionLevel};
    use crate::canvas::target::TargetId;
    use crate::panels::objects::test_support::engine_fixture;

    /// An application with a four-page fixture open, and nothing selected.
    pub(crate) fn opened() -> PdfceApp {
        let mut app = PdfceApp::new();
        app.open_path(engine_fixture("pageops/four-pages.pdf"));
        assert!(matches!(app.status, Status::Open(_)), "the fixture opens");
        app
    }

    /// Select whole object `index` on the current page, the way a canvas click
    /// would — through [`crate::canvas::selection::SelectionState::click`], so
    /// the state under test is the one a gesture really produces.
    ///
    /// `shift` adds to the selection rather than replacing it, exactly as a
    /// Shift+click does.
    pub(crate) fn select_object(app: &mut PdfceApp, index: u64, shift: bool) {
        let Status::Open(doc) = &mut app.status else {
            panic!("no document open") // ui-text-exempt: test panic, never displayed
        };
        let page = doc.view.page_index;
        doc.selection.click(
            page,
            ClickHit {
                object: Some(TargetId::Object(index)),
                ..ClickHit::default()
            },
            shift,
            false,
        );
    }

    /// The handler token the ribbon would raise for `id`.
    fn token_for(app: &PdfceApp, id: &str) -> egui_shell::commands::HandlerToken {
        app.commands
            .get(id)
            .unwrap_or_else(|| panic!("`{id}` must be registered")) // ui-text-exempt: test panic, never displayed
            .handler
    }

    /// ★ **`selection.any` is published, and only when something is
    /// selected.**
    ///
    /// The condition powers two surfaces the manifest has been carrying
    /// unwired: the contextual Format tab's *appearance*, and the enable state
    /// of the Delete inside it. It could not be published while the selection
    /// lived in `egui::Memory` — [`PdfceApp::conditions`] has no
    /// `egui::Context` — so this asserts the consequence of the move rather
    /// than a new policy.
    ///
    /// Both directions matter. Publishing it when nothing is selected would
    /// arm a **destructive** command over an empty operand list, which is
    /// defect D1's shape with the worst possible verb behind it.
    /// ★ **The hand tool and the armed region zoom report a pressed state.**
    ///
    /// The two controls that had none. Both halves are asserted: unarmed must
    /// be *unset*, armed must be set. Asserting only the armed half would pass
    /// on a condition wired to a constant, which is precisely how a toggle
    /// comes to render pressed forever.
    #[test]
    fn the_memory_backed_toggles_report_their_pressed_state() {
        let app = PdfceApp::new();
        let ctx = egui::Context::default();

        let hand = egui_shell::ribbon::selected_condition("view.tool_hand");
        let region = egui_shell::ribbon::selected_condition("view.zoom_region");

        assert!(
            !app.conditions(&ctx).is_set(&hand),
            "the select tool is the default, so Hand must not read as pressed"
        );
        assert!(!app.conditions(&ctx).is_set(&region));

        crate::canvas::tool::select(&ctx, crate::canvas::tool::CanvasTool::Hand);
        crate::canvas::zoom::arm_region_zoom(&ctx);

        assert!(app.conditions(&ctx).is_set(&hand), "Hand is armed");
        assert!(
            app.conditions(&ctx).is_set(&region),
            "the region zoom is armed"
        );
    }

    /// …and they keep reporting it with **no document open**.
    ///
    /// Deliberate, and the opposite of the other conditions in this function:
    /// the armed tool survives closing a document, so a ribbon that forgot
    /// which tool you were in the moment you closed a file would be reporting
    /// something untrue about its own state. The commands are gated on
    /// `doc.pages` separately, so the control is greyed *and* pressed — which
    /// is exactly "this is the tool you are in, and there is nothing to use it
    /// on".
    #[test]
    fn an_armed_tool_stays_pressed_with_nothing_open() {
        let app = PdfceApp::new();
        let ctx = egui::Context::default();
        crate::canvas::tool::select(&ctx, crate::canvas::tool::CanvasTool::Hand);

        assert!(matches!(app.status, Status::Empty), "nothing is open");
        assert!(
            app.conditions(&ctx)
                .is_set(&egui_shell::ribbon::selected_condition("view.tool_hand")),
        );
    }

    /// An application with the engine's page-sized-form fixture open — one page
    /// object (the form) and three squares painted from inside it.
    fn opened_with_a_form() -> PdfceApp {
        let mut app = PdfceApp::new();
        app.open_path(engine_fixture("forms-xobject/page-sized-form.pdf"));
        assert!(matches!(app.status, Status::Open(_)), "the fixture opens");
        app
    }

    /// Select the form-interior leaf at `index`, the way a canvas click on a
    /// square inside the form now would.
    fn select_leaf(app: &mut PdfceApp, index: u64) {
        let Status::Open(doc) = &mut app.status else {
            panic!("no document open") // ui-text-exempt: test panic, never displayed
        };
        let page = doc.view.page_index;
        doc.selection.click(
            page,
            ClickHit {
                object: Some(TargetId::Leaf(index)),
                ..ClickHit::default()
            },
            false,
            false,
        );
    }

    /// ★★ `selection.in_form` is set for a form-interior selection and for
    /// nothing else — which is what greys `format.select_form` correctly.
    ///
    /// The two negatives are the load-bearing half. A condition that were
    /// merely a synonym for `selection.any` would light the control on every
    /// selection in every document, and the operator would meet a button that
    /// declines more often than it works.
    #[test]
    fn the_in_form_condition_is_set_only_for_a_form_interior_selection() {
        let mut app = opened_with_a_form();
        let ctx = egui::Context::default();
        assert!(
            !app.conditions(&ctx).is_set("selection.in_form"),
            "a freshly opened document has nothing selected"
        );

        // The form itself is an ordinary page object and is NOT "in a form".
        select_object(&mut app, 0, false);
        assert!(app.conditions(&ctx).is_set("selection.any"));
        assert!(
            !app.conditions(&ctx).is_set("selection.in_form"),
            "the container is not inside itself"
        );

        select_leaf(&mut app, 1);
        assert!(app.conditions(&ctx).is_set("selection.any"));
        assert!(app.conditions(&ctx).is_set("selection.in_form"));
    }

    /// ★★★ **`format.select_form` selects the container**, and what it lands on
    /// is an edit operand — which is the whole point of offering it.
    ///
    /// Before this command the operator could reach an object inside a form and
    /// could reach nothing else: the deep hit test excludes forms outright, so
    /// the container had no route on the canvas at all. This is that route, and
    /// the assertion that matters is the last one — after pressing it, Delete
    /// has something to delete.
    #[test]
    fn select_the_form_lands_on_the_container_and_it_is_deletable() {
        let mut app = opened_with_a_form();
        let ctx = egui::Context::default();
        select_leaf(&mut app, 1);

        {
            let Status::Open(doc) = &app.status else {
                unreachable!()
            };
            assert!(
                doc.selection.deletable_objects_on(0).is_empty(),
                "a form-interior object is not an operand for any paint-order verb"
            );
        }

        let mut actions = Vec::new();
        app.dispatch_command(&ctx, "format.select_form", &mut actions);

        let Status::Open(doc) = &app.status else {
            unreachable!()
        };
        assert_eq!(
            doc.selection.targets_on(0),
            vec![TargetId::Object(0)],
            "the outermost enclosing form, in the page's own index space"
        );
        assert_eq!(
            doc.selection.deletable_objects_on(0),
            vec![0],
            "and NOW there is something a verb can act on"
        );
        assert!(
            doc.selection.leaf_indices_on(0).is_empty(),
            "the leaf selection was replaced, not added to"
        );
    }

    /// Pressing it with nothing selected says so rather than doing nothing.
    ///
    /// ★ `enabled_when` greys the ribbon item and enforces nothing — every
    /// other route reaches the dispatcher unchecked — so the arm asks again,
    /// and the arm's answer is a sentence rather than silence.
    #[test]
    fn select_the_form_with_no_form_selected_says_why() {
        let mut app = opened_with_a_form();
        let ctx = egui::Context::default();
        select_object(&mut app, 0, false);

        let mut actions = Vec::new();
        app.dispatch_command(&ctx, "format.select_form", &mut actions);

        assert_eq!(
            crate::app::status::decline::recorded_for_test(),
            Some(crate::app::status::decline::Declined::InsideForm),
            "the operator pressed something that did nothing; it owes them a reason"
        );
    }

    /// ★★ **Delete on a form-interior selection explains itself.**
    ///
    /// The state this closes: the operator has an outline round the thing they
    /// want gone, presses Delete, and nothing at all happens. From where they
    /// sit, Delete is broken. It is not — no paint-order verb can address a
    /// leaf — but a program that cannot say so has, for practical purposes,
    /// the defect anyway.
    ///
    /// And the negative: at the Part or Node rung the operand list is empty for
    /// a completely different reason, one the operator can see and put
    /// themselves in, and that case stays silent. A bar that narrates the
    /// obvious stops being read.
    #[test]
    fn delete_on_a_form_interior_selection_explains_itself() {
        let mut app = opened_with_a_form();
        let ctx = egui::Context::default();
        select_leaf(&mut app, 1);

        let mut actions = Vec::new();
        app.dispatch_command(&ctx, "format.delete", &mut actions);
        assert!(
            actions.is_empty(),
            "nothing may be raised: there is no operand"
        );
        assert_eq!(
            crate::app::status::decline::recorded_for_test(),
            Some(crate::app::status::decline::Declined::InsideForm),
        );

        // …and an ordinary object still deletes, with no sentence.
        select_object(&mut app, 0, false);
        let mut actions = Vec::new();
        app.dispatch_command(&ctx, "format.delete", &mut actions);
        assert_eq!(actions.len(), 1, "the form itself is perfectly deletable");
        assert_eq!(
            crate::app::status::decline::recorded_for_test(),
            None,
            "a command that ran retires the sentence rather than adding to it"
        );
    }

    #[test]
    fn the_selection_condition_follows_the_selection() {
        let mut app = PdfceApp::new();
        assert!(
            !app.conditions(&egui::Context::default())
                .is_set("selection.any"),
            "nothing is open, so nothing can be selected"
        );

        app = opened();
        assert!(
            app.conditions(&egui::Context::default())
                .is_set("doc.pages")
        );
        assert!(
            !app.conditions(&egui::Context::default())
                .is_set("selection.any"),
            "a freshly opened document has nothing selected"
        );

        select_object(&mut app, 1, false);
        assert!(
            app.conditions(&egui::Context::default())
                .is_set("selection.any")
        );

        // Escape at the Object rung clears, and the condition follows it back
        // down — a tab that stayed visible over an empty selection would offer
        // a Delete with nothing to delete.
        let Status::Open(doc) = &mut app.status else {
            unreachable!()
        };
        doc.selection.escape();
        assert!(
            !app.conditions(&egui::Context::default())
                .is_set("selection.any")
        );
    }

    /// ★ **The ribbon's Delete raises the same action the Delete key does.**
    ///
    /// `format.delete` was drawn and enabled from the moment the Format tab
    /// landed, and did nothing — the live instance of D1's shape that this
    /// stage is accountable for. It became wirable when the selection moved
    /// onto `OpenDoc`, because [`PdfceApp::dispatch_token`] has no
    /// `egui::Context` and therefore had no route to a selection in
    /// `egui::Memory`.
    ///
    /// Asserted through the real token lookup rather than by calling the arm
    /// directly: the dispatch resolves a token back to an id, so a test that
    /// skipped that step would pass even if the command were never registered.
    #[test]
    fn the_ribbon_delete_raises_the_delete_action() {
        // A bare context: these tests exercise the dispatcher, not a
        // frame. `dispatch_command` needs one because three navigation arms
        // write the armed tool and the zoom anchor into egui memory, which
        // is where per-frame UI state lives.
        let ctx = egui::Context::default();
        let mut app = opened();
        let delete = token_for(&app, "format.delete");

        // Nothing selected: nothing raised. An empty batch would be an action
        // the engine has to refuse, reported as a failure the operator caused.
        let mut actions = Vec::new();
        app.dispatch_token(&ctx, delete, &mut actions);
        assert!(actions.is_empty());

        select_object(&mut app, 2, false);
        select_object(&mut app, 0, true);
        let mut actions = Vec::new();
        app.dispatch_token(&ctx, delete, &mut actions);
        assert_eq!(
            actions,
            vec![
                VectorAction::DeleteSelection {
                    page: 0,
                    objects: vec![0, 2],
                }
                .into()
            ],
            "one action carrying the whole batch, ascending — `delete_objects` \
             resolves every index before planning, so a second single-object \
             action would renumber the page between them"
        );
    }

    /// ★ **The ribbon's Delete obeys the same rung rule as the key.**
    ///
    /// Inside an object the selection names a subpath, and the only wired verb
    /// removes whole objects — one measured CAD export holds an entire drawing
    /// view as a single path object with 1,194 subpaths. The rule lives once,
    /// on `SelectionState::deletable_objects_on`; this asserts that the ribbon
    /// path really reads it rather than re-deriving an operand list of its
    /// own, which is exactly how two spellings of a destructive rule drift
    /// apart.
    #[test]
    fn the_ribbon_delete_declines_inside_an_object_just_as_the_key_does() {
        // A bare context: these tests exercise the dispatcher, not a
        // frame. `dispatch_command` needs one because three navigation arms
        // write the armed tool and the zoom anchor into egui memory, which
        // is where per-frame UI state lives.
        let ctx = egui::Context::default();
        let mut app = opened();
        select_object(&mut app, 1, false);

        let Status::Open(doc) = &mut app.status else {
            unreachable!()
        };
        // Double-click into part 1 of the selected object.
        doc.selection.click(
            0,
            ClickHit {
                object: Some(TargetId::Object(1)),
                part: Some(1),
                node: None,
            },
            false,
            true,
        );
        assert_eq!(doc.selection.level(), SelectionLevel::Part);

        let delete = token_for(&app, "format.delete");
        let mut actions = Vec::new();
        app.dispatch_token(&ctx, delete, &mut actions);
        assert!(
            actions.is_empty(),
            "the Part rung has no delete verb wired, and the ribbon must not \
             borrow the Object rung's any more than the key may"
        );

        // …and the tab is still visible, which is why the decline has to be
        // handled rather than made unreachable: something IS selected.
        assert!(
            app.conditions(&egui::Context::default())
                .is_set("selection.any")
        );
    }

    // -----------------------------------------------------------------------
    // The two commands that used to dispatch nowhere
    // -----------------------------------------------------------------------

    /// ★ **`file.properties` puts the Properties panel on screen, from any
    /// mode.**
    ///
    /// The command was named by File ▸ Document, named by the `objects.row`
    /// context menu, registered in `crate::shell::commands` — and had no arm,
    /// so invoking it traced `command-unimplemented` and did nothing. That is
    /// D1's shape: a control that looks available and is inert.
    ///
    /// The mode matters, which is why the test walks all three. The
    /// application **opens in Read**, and Read's default arrangement mounts no
    /// Properties panel at all (`app::modes`' `spec("read")`), so the
    /// interesting case — activate fails, mount, activate again — is the
    /// *first* one an operator meets rather than an edge case. Review and Edit
    /// mount it already and take the cheap path.
    ///
    /// Driven through the real token lookup, so a command that stopped being
    /// registered fails here rather than silently taking the `other` arm.
    #[test]
    fn the_properties_command_puts_the_panel_on_screen_in_every_mode() {
        // A bare context: these tests exercise the dispatcher, not a
        // frame. `dispatch_command` needs one because three navigation arms
        // write the armed tool and the zoom anchor into egui memory, which
        // is where per-frame UI state lives.
        let ctx = egui::Context::default();
        let mut app = opened();
        let properties =
            egui_shell::dock::PanelId::new(crate::panels::Panel::Properties.command_id());
        let token = token_for(&app, "file.properties");

        for mode in ["read", "review", "edit"] {
            app.modes
                .on_mode_changed(mode, &mut app.dock, &mut app.layout, &app.panel_registry);

            let mut actions = Vec::new();
            app.dispatch_token(&ctx, token, &mut actions);
            assert!(
                app.dock.is_on_screen(&properties),
                "`file.properties` must produce the panel in the `{mode}` arrangement, \
                 mounting it if the operator's layout no longer holds it"
            );
            assert!(
                actions.is_empty(),
                "showing a panel is a dock change, not a document action"
            );

            // Idempotent: asking twice is not a toggle. The `objects.row`
            // context menu offers this command to *describe the row just
            // clicked*, and a second invocation that hid the description
            // would be actively hostile.
            app.dispatch_token(&ctx, token, &mut Vec::new());
            assert!(app.dock.is_on_screen(&properties));
        }
    }

    /// ★ **`view.reset_layout` restores the active mode's default
    /// arrangement.**
    ///
    /// The other command with no arm. `Modes::reset` existed and was tested;
    /// nothing invoked it, so View ▸ Window ▸ Reset layout and the `dock.tab`
    /// context menu both traced `command-unimplemented`.
    ///
    /// The test asserts the arrangement is *exactly* the mode's default,
    /// which is a stronger claim than "the closed panel came back": a reset
    /// that produced some third arrangement, or that reset only one dock,
    /// would pass the weaker one.
    ///
    /// It resets **before** rearranging as well as after, deliberately. This
    /// application loads the operator's persisted layout at start-up, so the
    /// arrangement a test inherits is whatever is on the machine running it;
    /// the first dispatch is both the assertion that the command works from an
    /// arbitrary starting point and the thing that makes the second half
    /// deterministic.
    ///
    /// **`ResetScope::All` is the scope this build passes**, and that is a
    /// decision recorded in the dispatch arm, not an oversight — with no
    /// chooser surface, a control named "Reset layout" that reset half the
    /// layout would be the more surprising failure.
    #[test]
    fn the_reset_layout_command_restores_the_modes_default_arrangement() {
        // A bare context: these tests exercise the dispatcher, not a
        // frame. `dispatch_command` needs one because three navigation arms
        // write the armed tool and the zoom anchor into egui memory, which
        // is where per-frame UI state lives.
        let ctx = egui::Context::default();
        let mut app = opened();
        app.modes
            .on_mode_changed("edit", &mut app.dock, &mut app.layout, &app.panel_registry);
        let default = crate::app::modes::layout_for_build("edit", &app.panel_registry);
        let reset = token_for(&app, "view.reset_layout");

        let mut actions = Vec::new();
        app.dispatch_token(&ctx, reset, &mut actions);
        assert_eq!(
            app.dock.layout(),
            &default,
            "`view.reset_layout` must restore this mode's default, whole"
        );
        assert!(actions.is_empty(), "a layout reset touches no document");

        // Rearrange — closing a panel is the most likely reason an operator
        // reaches for this — and reset again.
        let objects = egui_shell::dock::PanelId::new(crate::panels::Panel::Objects.command_id());
        assert!(app.dock.layout_mut().close(&objects));
        assert_ne!(app.dock.layout(), &default);
        app.dispatch_token(&ctx, reset, &mut Vec::new());
        assert_eq!(app.dock.layout(), &default);
    }

    /// ★ **A keyboard chord and the control that shares its command do the
    /// same thing.**
    ///
    /// The structural half of the two-owner fix. `crate::app::keyboard` no
    /// longer knows what `Ctrl+0` *means*; it reads the id out of the manifest
    /// keymap and hands it here, so the chord and the ribbon button land in
    /// one arm by construction.
    ///
    /// This asserts the consequence: dispatching the id the keymap binds to
    /// `Ctrl+0` raises exactly what the ribbon's Actual size raises. It would
    /// have failed before the fix — the chord raised `Fit(FitMode::Page)` and
    /// the button raised `ZoomTo(1.0)`, which is the defect in one line.
    #[test]
    fn the_chord_and_the_button_raise_the_same_action() {
        // A bare context: these tests exercise the dispatcher, not a
        // frame. `dispatch_command` needs one because three navigation arms
        // write the armed tool and the zoom anchor into egui memory, which
        // is where per-frame UI state lives.
        let ctx = egui::Context::default();
        let mut app = opened();
        let keymap = app
            .shell
            .as_ref()
            .and_then(|s| s.keymap.as_ref())
            .expect("the built-in manifest binds chords");
        let bound = keymap.get("Ctrl+0").expect("Ctrl+0 is bound").to_owned();

        let mut from_chord = Vec::new();
        app.dispatch_command(&ctx, &bound, &mut from_chord);

        let mut from_button = Vec::new();
        app.dispatch_token(&ctx, token_for(&app, &bound), &mut from_button);

        assert_eq!(from_chord, from_button);
        assert_eq!(from_chord, vec![Action::ZoomTo(1.0)]);
    }

    /// The mode chords select the mode their tooltips name.
    ///
    /// `MODES_AND_PANELS.md` Part 1 §6 specifies `Ctrl+1`/`Ctrl+2`/`Ctrl+3`,
    /// and all three `crate::text::commands::mode_*` tooltips print the chord.
    /// Until this arm existed, all three sentences were false: the manifest
    /// bound the chords, nothing dispatched them, and `Ctrl+2` was in fact
    /// doing fit-width from `keyboard::collect`.
    #[test]
    fn the_mode_commands_move_the_ribbon_selector() {
        // A bare context: these tests exercise the dispatcher, not a
        // frame. `dispatch_command` needs one because three navigation arms
        // write the armed tool and the zoom anchor into egui memory, which
        // is where per-frame UI state lives.
        let ctx = egui::Context::default();
        let mut app = opened();
        for (command, mode) in [
            ("mode.review", "review"),
            ("mode.edit", "edit"),
            ("mode.read", "read"),
        ] {
            app.dispatch_command(&ctx, command, &mut Vec::new());
            assert_eq!(app.ribbon.mode(), Some(mode));
        }
    }
}
