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
//!    actions. Nothing mutates.
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
//! ## What S0 does not have, stated so it is not mistaken for an oversight
//!
//! No ribbon, no QAT, no dock, no status bar, no find bar, no dialogs, no
//! Open command. Every one of them is scheduled in `PROJECT_PLAN.md` §4.
//! The "no placeholders" invariant applies: unavailable renders **nothing**
//! rather than a greyed-out control that explains itself badly.

pub mod actions;
pub mod cache;
/// How a path gets from an operator — or from a scripted harness — to
/// [`actions::Action::Open`]. The picker, the diagnostics seam that answers
/// it without a human, and the dirty-document rule.
pub mod files;
pub mod keyboard;
pub mod modes;
pub mod persistence;
/// The documents this operator had open: the capped, persisted list and the
/// ribbon control that draws it.
pub mod recent;
pub mod state;
pub mod status;

use actions::Action;
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
#[derive(Default)]
pub struct PdfceApp {
    /// What, if anything, is open.
    pub status: Status,

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

        Self {
            status: Status::default(),
            shell,
            commands,
            ribbon,
            panel_registry,
            dock,
            modes,
            layout,
            recent,
            recent_choice: None,
            panels: crate::panels::PanelsState::default(),
        }
    }

    /// The conditions the ribbon evaluates its predicates against.
    ///
    /// Rebuilt every frame because that is what it describes — the state
    /// *this* frame is drawn from. Five conditions, and the set is closed:
    /// `crate::shell::commands` has a test asserting no predicate names
    /// anything outside it, so a typo in a manifest cannot silently
    /// produce a control that is disabled forever.
    ///
    /// # ★ `selection.any` is published from here, and only now
    ///
    /// It was deliberately absent while the selection lived in
    /// `egui::Memory`: this function has no `egui::Context`, so it could not
    /// have read the selection even if it wanted to, and publishing a
    /// condition it could not evaluate would have armed a **destructive**
    /// control that could not work — the inverse of the no-placeholders rule
    /// and the exact shape of defect D1.
    ///
    /// The selection now lives on [`state::OpenDoc`], so the answer is one
    /// field read. Two surfaces come alive with it, both of which the manifest
    /// has been carrying unpowered: the contextual **Format** tab
    /// (`visible_when: "selection.any"`, which is the appear-on-selection
    /// affordance `RIBBON_IA.md` §5.8 calls the single largest usability
    /// change) and the **Delete** inside it (`enabled_when` the same). One
    /// spelling, one source — see `shell::manifest::format::VISIBLE_WHEN`.
    ///
    /// **The Objects panel's focus is not a selection and must never satisfy
    /// this**, which is what `panels::PanelsState::focus`'s own test asserts
    /// through the enable machinery: a panel row being focused must not arm a
    /// destructive command, because the operator would have no way to tell
    /// which of two "selections" it was about to act on. This reads
    /// `doc.selection` and nothing else.
    fn conditions(&self) -> egui_shell::commands::ConditionSet {
        let mut set = egui_shell::commands::ConditionSet::new();
        if let Status::Open(doc) = &self.status {
            set.set("doc.open");
            if !doc.pages.is_empty() {
                set.set("doc.pages");
            }
            if !doc.selection.is_empty() {
                set.set("selection.any");
            }
        }
        // `undo.available` and `redo.available` are still deliberately absent:
        // there is no undo stack to report on yet. Setting them would arm
        // controls that cannot work. They arrive with their subsystem.
        set
    }

    /// Draw the ribbon and translate what the operator invoked.
    ///
    /// # ★ The one custom item, and why it is not a command
    ///
    /// `Item::Custom` is `egui-shell`'s extension point for a control that is
    /// not a button, and its own doc names the case: *"a colour swatch, a zoom
    /// slider, a scale picker, a split button with a gallery."* The Recent
    /// control is the last of those. A `Command` item could only ever render
    /// as a button, and a button cannot ask *which* of ten documents — so
    /// modelling the list as a command would have meant either ten commands
    /// or a command that opened whichever file it felt like.
    ///
    /// The renderer therefore does two things and no more: draw, and report.
    /// The chosen path is parked in [`Self::recent_choice`] and the
    /// `file.recent` token is returned, so the *command* goes through
    /// [`Self::dispatch_command`] exactly as a ribbon click does. Parking
    /// before dispatching is load-bearing and the order below says so.
    ///
    /// An item whose `kind` this application does not know draws **nothing**
    /// and returns `None` — the same posture the shell takes towards an
    /// unknown command id, and the reason the manifest's `colour_swatch`
    /// (whose control is not built) leaves a gap rather than a mystery
    /// widget.
    fn ribbon_band(&mut self, ui: &mut egui::Ui, actions: &mut Vec<Action>) {
        let Some(shell) = self.shell.as_ref() else {
            return;
        };
        let conditions = self.conditions();

        // The rect sink. Every group caption and mode segment publishes its
        // rect under a stable name, which is what lets `ui-verify` assert
        // legibility on the frame the rect was measured on instead of
        // hard-coding fractions that go stale the first time a panel moves
        // (`PROJECT_PLAN.md` §4.2 prerequisite 1).
        let mut report_rect = |name: &str, rect: egui::Rect| {
            crate::diag::ui_rect(name, rect);
        };

        // Resolved before the closure, because inside it `self` is not
        // reachable: the closure borrows `self.recent` mutably while
        // `&self.commands` is handed to the ribbon. `Option` rather than an
        // expectation — a build whose registry has no `file.recent` draws no
        // menu instead of panicking in the paint loop.
        let recent_token = self
            .commands
            .get(crate::shell::commands::FILE_RECENT)
            .map(|c| c.handler);
        let recent = &mut self.recent;
        let mut chosen: Option<std::path::PathBuf> = None;
        let mut custom = |ui: &mut egui::Ui, item: &egui_shell::ribbon::CustomItem<'_>| {
            if item.kind != crate::shell::manifest::RECENT_FILES {
                return None;
            }
            let picked = crate::app::recent::menu(ui, recent, std::time::Instant::now())?;
            chosen = Some(picked);
            recent_token
        };

        let tokens = egui::Panel::top("ribbon")
            .show(ui, |ui| {
                egui_shell::ribbon::Ribbon::new()
                    .with_conditions(&conditions)
                    .reporting_rects_to(&mut report_rect)
                    .with_custom_items(&mut custom)
                    .render(ui, shell, &self.commands, &mut self.ribbon)
            })
            .inner;

        // Park the operand BEFORE dispatching the token that consumes it.
        // Reversing these two lines would make the `file.recent` arm find an
        // empty slot and fall back to the newest entry — which is a defined
        // behaviour, and therefore a defect with no symptom except that the
        // operator's third choice opened their first.
        if chosen.is_some() {
            self.recent_choice = chosen;
        }

        for token in tokens {
            self.dispatch_token(token, actions);
        }
    }

    /// Draw the left and right docks and their panel bodies.
    ///
    /// The dock knows nothing about PDFs — it is handed opaque
    /// [`egui_shell::dock::PanelId`]s and hands them back, and this closure
    /// is the single place a `PanelId` becomes a `crate::panels::Panel`.
    /// One dispatcher, exactly as the ribbon has one: an id that does not
    /// resolve draws its own explanation rather than an empty pane, because
    /// an empty pane is indistinguishable from a panel that had nothing to
    /// say.
    fn docks(&mut self, ui: &mut egui::Ui, actions: &mut Vec<Action>) {
        // Borrows split before the closure: the body needs `status` and
        // `panels` while `show` holds `dock` mutably, and the closure
        // cannot reach through `self` for them.
        // Conditions are computed before the destructure, because building
        // them needs `&self` and the destructure takes it apart.
        let conditions = self.conditions();

        let Self {
            status,
            shell,
            commands,
            panel_registry,
            dock,
            panels,
            ..
        } = self;
        let doc = match status {
            Status::Open(doc) => Some(&**doc),
            _ => None,
        };

        // The right-click host. `None` if the manifest failed to validate —
        // in which case there is no ribbon either, and a context menu
        // offering commands the ribbon cannot show would be the only route
        // to them, which is worse than none.
        let host = shell
            .as_ref()
            .map(|s| crate::shell::menus::MenuHost::new(s, commands, &conditions));

        let mut report_rect = |name: &str, rect: egui::Rect| {
            crate::diag::ui_rect(name, rect);
        };

        // Tokens collected inside the dock body and dispatched after it, for
        // the same reason actions are applied after the frame: the closure
        // holds borrows the dispatcher needs back.
        let mut tokens = Vec::new();
        // A second `Vec`, and it has to be: `tokens` is already captured
        // mutably by the body closure below, so the tab-menu handler cannot
        // also borrow it.
        let mut tab_tokens = Vec::new();
        let mut tab_menu = |tab: &mut egui_shell::dock::TabMenu<'_>| {
            // The dock hands out the tab's `Response`; what a right-click on
            // it offers is the application's business, which is the whole
            // point of the seam. Supplying a handler also takes the dock's
            // built-in Close off that tab — deliberately, because two menus
            // on one `Response` are two writers of one popup id.
            tab_tokens.extend(
                host.iter()
                    .flat_map(|h| h.attach(tab.response(), crate::shell::menus::DOCK_TAB)),
            );
        };
        let report = egui_shell::dock::Dock::new()
            .with_registry(panel_registry)
            .with_tab_menu(&mut tab_menu)
            .reporting_rects_to(&mut report_rect)
            .show(
                ui,
                dock,
                |panel_id, ui| match crate::panels::Panel::from_command_id(panel_id.as_str()) {
                    Some(panel) => {
                        tokens.extend(panel.show(ui, doc, panels, host.as_ref(), actions));
                    }
                    None => {
                        ui.label(crate::text::panels::panel_unknown());
                    }
                },
            );

        for token in tokens.into_iter().chain(tab_tokens) {
            self.dispatch_token(token, actions);
        }

        // ★ Bind the mode selector to the dock, and the dock to disk.
        //
        // Three separate questions, deliberately in this order.
        //
        // 1. **Did the operator change mode?** Then the arrangement they are
        //    leaving is recorded and the one they are entering is applied.
        //    Compared against `modes`' own idea of the active mode rather
        //    than a flag, because the ribbon owns the selector and there is
        //    no event: it is drawn, the operator clicks, and the two differ
        //    on the next frame.
        // 2. **Did they rearrange it?** `layout_changed` is the dock's own
        //    report of a splitter drag, a tab move or a close — not a
        //    comparison this function has to re-derive.
        // 3. **Is a write due?** `tick` runs the debounce. It returns when to
        //    look again, and **egui only wakes on input**, so without the
        //    repaint request an arrangement changed and then left alone would
        //    sit unwritten until some unrelated frame happened along.
        //
        // The order of 1 and 2 is load-bearing: recording after a mode change
        // would file the outgoing arrangement under the incoming mode.
        if let Some(mode) = self.ribbon.mode().map(str::to_owned)
            && self.modes.active() != Some(mode.as_str())
        {
            self.modes.on_mode_changed(
                &mode,
                &mut self.dock,
                &mut self.layout,
                &self.panel_registry,
            );
        }
        if report.layout_changed {
            let arrangement = self.dock.layout().clone();
            self.modes.record_layout(&arrangement, &mut self.layout);
        }
        if let Some(after) = self.layout.tick(std::time::Instant::now()) {
            ui.ctx().request_repaint_after(after);
        }

        // What the dock actually drew, not what the layout asked for. The
        // two differ whenever a saved layout names a panel this build does
        // not have, and the difference is the thing worth tracing.
        crate::diag::trace_changed(DOCK_SLOT, || {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed.
                "dock panels={} overflowed={} changed={}",
                report.panels_drawn.len(),
                report.panels_overflowed,
                report.layout_changed,
            )
        });
    }

    /// Turn one invoked command into whatever the application does about it.
    ///
    /// Resolved token → id → [`Self::dispatch_command`], rather than matching
    /// on the raw token number. The numbers are assigned in per-tab blocks in
    /// `crate::shell::commands` and are meaningful only there; duplicating
    /// them here would create a second place to keep in step, and a silent
    /// mis-dispatch is the failure that would result.
    ///
    /// **The id is cloned rather than borrowed**, and it is not an
    /// oversight: the arms below need `&mut self` — a panel to activate, a
    /// dock to reset, a mode to select — and a `&str` borrowed out of
    /// `self.commands` would hold `self` shared for the whole match. One
    /// short allocation per *invoked command* (an operator click, not a
    /// frame) is the right price for arms that can act on the application.
    fn dispatch_token(
        &mut self,
        token: egui_shell::commands::HandlerToken,
        actions: &mut Vec<Action>,
    ) {
        let Some(id) = self
            .commands
            .iter()
            .find(|c| c.handler == token)
            .map(|c| c.id.clone())
        else {
            return;
        };
        self.dispatch_command(&id, actions);
    }

    /// Do whatever this build does about the command named `id`.
    ///
    /// **The one dispatcher.** A ribbon click, a QAT click, a context-menu
    /// click and a keyboard chord all arrive here, which is what makes it
    /// impossible for a chord and a button that share a command to do
    /// different things — the defect `crate::app::keyboard`'s header is
    /// about, closed structurally rather than by agreement.
    ///
    /// **A command with no arm is not an error.** At S2 most of the ribbon is
    /// scaffolding for behaviour that lands at S3 and later, and the
    /// honest thing is to say so once per invocation in the trace rather
    /// than to pretend the click did something. Where a command is *known*
    /// not to be implementable yet, its arm says why in the trace rather
    /// than falling through to the generic line — a reader of a trace from a
    /// machine they cannot see should not have to guess which kind of
    /// nothing happened.
    fn dispatch_command(&mut self, id: &str, actions: &mut Vec<Action>) {
        match id {
            // ★ Open. The command that makes pdfce a reader rather than a
            // viewer of one file.
            //
            // It was registered, drawn on the File tab, drawn on the QAT,
            // bound to Ctrl+O in the keymap — and had no arm, so the only way
            // to open a document was `argv`. That is defect D1's shape with
            // the most consequential verb in the application behind it.
            //
            // The dialog runs HERE, during dispatch, and only its *result*
            // becomes an action. See `crate::app::files` for why that line is
            // where it is, and for the `PDFCE_DIAG_OPEN_PATH` seam that lets
            // a scripted harness answer the dialog without a human — a native
            // dialog is a hard wall for synthetic input, and substituting the
            // answer is the only thing that gets past it.
            "file.open" => {
                Self::open_picked(crate::app::files::pick_document(), actions);
            }
            // ★ Close. `doc.open` gates the control, so the no-document case
            // is unreachable from the ribbon — and the action handles it
            // anyway, because a customized keymap can reach any command from
            // any state.
            "file.close" => actions.push(Action::Close),
            // ★ Recent. The operand comes from the `recent_files` custom item
            // (see `Self::ribbon_band`), which parked it before returning this
            // command's token.
            //
            // Reaching this arm with nothing parked is not an error and not
            // unreachable: an operator may bind a chord to `file.recent` or
            // put it on their quick-access toolbar, neither of which draws a
            // menu. The defined answer is **the newest document that can be
            // seen right now** — which is what "recent" means with no further
            // qualification, and which skips an entry on a drive that is not
            // connected rather than reporting a failure the operator did not
            // cause.
            crate::shell::commands::FILE_RECENT => {
                let path = self
                    .recent_choice
                    .take()
                    .or_else(|| self.recent.newest_present(std::time::Instant::now()));
                match path {
                    Some(path) => actions.push(Action::Open(path)),
                    None => crate::diag::trace(|| {
                        // ui-text-exempt: diagnostic trace, never displayed.
                        "recent-declined reason=nothing-reachable".to_owned()
                    }),
                }
            }
            "view.zoom_in" => actions.push(Action::ZoomIn),
            "view.zoom_out" => actions.push(Action::ZoomOut),
            // `ZoomTo(1.0)`, not `Fit(FitMode::None)`. The latter only stops the
            // per-frame re-fit and leaves the zoom where it was, so this
            // control used to pin whatever magnification happened to be
            // showing while promising one PDF point per screen point.
            "view.zoom_actual" => actions.push(Action::ZoomTo(1.0)),
            "view.zoom_fit_page" => actions.push(Action::Fit(crate::viewer::FitMode::Page)),
            "view.zoom_fit_width" => actions.push(Action::Fit(crate::viewer::FitMode::Width)),
            "view.next_page" => actions.push(Action::NextPage),
            "view.prev_page" => actions.push(Action::PrevPage),
            // This control was drawn and enabled from the moment the ribbon
            // landed, and did nothing — a live instance of D1's shape: an
            // affordance that looks available and is inert. It became
            // wirable when `RenderKey` gained `annotations`.
            "view.show_annotations" => actions.push(Action::ToggleAnnotations),
            // ★ The ribbon's Delete — the contextual Format tab's one command.
            //
            // The id is `format.delete`, not `edit.delete`: `RIBBON_IA.md`
            // §5.8 puts Delete on the **Format** tab, which is contextual and
            // appears only while something is selected, and
            // `shell::commands` registers exactly that id gated on
            // `selection.any`. There is no `edit.delete` in this build, and
            // adding an arm for one would be an arm no token can ever reach —
            // dead code wearing a design pattern, which is what the
            // no-placeholders invariant forbids.
            //
            // It became wirable when the selection moved onto `OpenDoc`: this
            // function has no `egui::Context`, so while the selection lived in
            // `egui::Memory` there was no route from a ribbon click to the
            // thing it was about to delete. That is the whole of why the
            // control has been drawn-but-unwired until now.
            //
            // **The rule is not restated here.**
            // `SelectionState::deletable_objects_on` decides what a Delete may
            // act on — Object rung only, ascending, de-duplicated, this page
            // only — and the canvas's Delete key reads the same method. Two
            // statements of a destructive rule is one too many.
            //
            // An empty list raises nothing rather than an empty action the
            // engine would have to refuse. That is reachable in practice: the
            // Format tab is visible whenever *anything* is selected, including
            // at a rung whose delete verb does not exist yet.
            "format.delete" => {
                if let Status::Open(doc) = &self.status {
                    let page = doc.view.page_index;
                    let objects = doc.selection.deletable_objects_on(page);
                    if !objects.is_empty() {
                        actions.push(Action::DeleteSelection { page, objects });
                    }
                }
            }
            // ★ The Properties panel. See [`Self::show_panel`] for the
            // mount-versus-nothing decision, which is the only interesting
            // part of this.
            //
            // `file.properties` rather than a `view.panel_*` id because
            // `RIBBON_IA.md` §5.1 puts Properties in File ▸ Document — it
            // describes the document, not the screen — and
            // `crate::panels::Panel::command_id` is the one place that
            // binding is written down.
            "file.properties" => self.show_panel(crate::panels::Panel::Properties),
            // ★ Reset layout. `ResetScope::All`, and the scope is a decision.
            //
            // `RIBBON_IA.md`'s rule is why a scope exists at all: *"an
            // operator who only wanted the right dock back must not lose
            // their left one."* Honouring that properly needs a **chooser**,
            // and this build has no modal, no popup and no split-button
            // affordance to put one in — see the note on
            // `crate::text::commands::view_reset_layout`, which used to
            // promise the choice in its tooltip and no longer does.
            //
            // Given one button and no chooser, `All` is the only scope whose
            // behaviour matches the words on it: a control named "Reset
            // layout" that reset half the layout would be the more surprising
            // of the two failures. It is also the least destructive it looks:
            // `Modes::reset` restores *this mode's* default and leaves every
            // other mode's saved workspace alone.
            //
            // What a chooser needs, so the next hand does not have to
            // re-derive it: three commands (`view.reset_layout_left`,
            // `_right`, `_all`) with their own `CommandText`, an
            // `egui_shell::manifest::Item` kind that renders a split button
            // or a submenu, and this arm becoming three that pass the
            // matching `ResetScope`. `ResetScope::ALL` already lists them
            // narrowest-first, in the order such a menu should offer them.
            "view.reset_layout" => {
                let scope = egui_shell::layout::ResetScope::All;
                let changed = self.modes.reset(
                    scope,
                    &mut self.dock,
                    &mut self.layout,
                    &self.panel_registry,
                );
                crate::diag::trace(|| {
                    format!(
                        // ui-text-exempt: diagnostic trace, never displayed.
                        "layout-reset scope={scope} changed={changed}"
                    )
                });
            }
            // ★ The mode selector's three keyboard positions.
            //
            // `RibbonState::set_mode`'s own doc commissions exactly this:
            // *"This is what an application calls when the operator presses
            // the Ctrl+1 its manifest bound to `mode.read` — the shell
            // reports the command's token, the application dispatches it,
            // and dispatching it means calling this."*
            //
            // Wired here because the keymap route now reaches it. The mode
            // ids are the command ids without their `mode.` prefix, which is
            // the manifest's own convention — see
            // `crate::shell::manifest::built_in`'s mode list beside its
            // keymap — and an id the manifest does not declare is declined
            // rather than adopted, so a customized keymap naming a fourth
            // mode cannot put the ribbon into a state it has no tab list for.
            //
            // Nothing else happens here: the dock follows on the same frame,
            // in `Self::docks`, which compares `ribbon.mode()` against
            // `modes.active()` and moves the workspace across. One place does
            // that, and it must stay one place — see its ★ comment on why the
            // order of *record* and *restore* is load-bearing.
            "mode.read" | "mode.review" | "mode.edit" => {
                if let Some(mode) = id.strip_prefix("mode.") {
                    if self.modes.is_known(mode) {
                        self.ribbon.set_mode(mode.to_owned());
                    } else {
                        crate::diag::trace(|| {
                            format!(
                                // ui-text-exempt: diagnostic trace, never displayed.
                                "command-declined id={id} reason=mode-not-declared"
                            )
                        });
                    }
                }
            }
            other => {
                crate::diag::trace(|| {
                    format!(
                        // ui-text-exempt: diagnostic trace, never displayed.
                        "command-unimplemented id={other}"
                    )
                });
            }
        }
    }

    /// **Turn what the picker said into what the application does about it.**
    ///
    /// Split out of the `file.open` arm for one reason: it is the only part
    /// of that arm a test may run. Dispatching `file.open` itself opens a
    /// **real modal dialog** and blocks until a human dismisses it, so a test
    /// that did it would hang `cargo test` behind an invisible window — see
    /// [`crate::app::files`]' third rule. Everything downstream of the answer
    /// is therefore reachable from a test with the answer supplied directly,
    /// and only the `env::var_os` read itself is not (and cannot be:
    /// `std::env::set_var` is `unsafe` in edition 2024 and this crate forbids
    /// unsafe code).
    ///
    /// Associated rather than a method because it touches nothing on `self`
    /// — which is itself the point. A picker's answer becomes an action and
    /// nothing else; the deciding, the loading and the failure classification
    /// all happen after the frame, in [`Self::apply_actions`].
    ///
    /// The three answers get three different treatments, and the differences
    /// are the whole reason [`crate::app::files::Picked`] is not an
    /// `Option<PathBuf>`:
    ///
    /// | answer | what happens | why |
    /// |---|---|---|
    /// | a path | [`Action::Open`] | the ordinary case |
    /// | cancelled | nothing at all, not even a trace line | the operator changed their mind; that is a complete and correct outcome and reporting it would be noise on every dismissed dialog |
    /// | unavailable | a trace naming the gap | this is a **build** limitation, not an operator choice, and it is the one a reader of a trace from a machine they cannot see most needs told apart from "the click never arrived" |
    fn open_picked(picked: crate::app::files::Picked, actions: &mut Vec<Action>) {
        use crate::app::files::Picked;
        match picked {
            Picked::Path(path) => actions.push(Action::Open(path)),
            Picked::Cancelled => {}
            Picked::Unavailable => crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                "open-unavailable reason=no-picker-in-this-build".to_owned()
            }),
        }
    }

    /// **Put `panel` on screen, mounting it first if the operator's
    /// arrangement no longer holds it.**
    ///
    /// # The decision: mount, rather than do nothing
    ///
    /// `DockState::activate` returns `false` for a panel that is not mounted,
    /// and its own docs say what that means: *"the caller's fallback is to
    /// mount it or to restore a default arrangement, not to refuse."* Both
    /// are defensible. This mounts, for three reasons:
    ///
    /// 1. **Read mode has no Properties panel by design** — `app::modes`'
    ///    `spec("read")` gives it none, because "an inspector in a mode with
    ///    no edit verbs is a panel whose every row is a fact you cannot act
    ///    on". But File ▸ Properties is on the File tab, and the File tab is
    ///    in *every* mode. Doing nothing would make a visible, enabled
    ///    control inert in the mode the application **opens in** — defect
    ///    D1's exact shape, and the thing `PROJECT_PLAN.md`'s no-placeholders
    ///    invariant forbids.
    /// 2. **There is no other route.** Properties is not on View ▸ Panels
    ///    (it is `file.properties`, not `view.panel_properties`), so an
    ///    operator who closes its tab has no second way back. A command that
    ///    is the only route to a surface must be able to produce it.
    /// 3. **A mode default is a starting arrangement, not a prohibition.**
    ///    The dock belongs to the operator; asking for a panel is a
    ///    rearrangement, and rearrangements are what the layout store exists
    ///    to remember.
    ///
    /// # Where it lands
    ///
    /// Wherever the **Edit mode default** puts it, because that arrangement
    /// names every panel this build has and agrees with the other modes about
    /// which side each one belongs on (Properties is on the right in Review
    /// and in Edit). Reading the placement out of `app::modes` rather than
    /// writing a side into this function is what stops "where does Properties
    /// live" from acquiring a second answer. The right dock is the fallback
    /// if that lookup ever fails, which it cannot today.
    ///
    /// `DockLayout::mount` is deliberately permissive about out-of-range
    /// column and stack indices — it clamps — so a live arrangement with
    /// fewer columns than the default needs no special case here.
    ///
    /// # Why the arrangement is recorded
    ///
    /// A mount is a layout change, and `Dock::show`'s `layout_changed` report
    /// describes what the *operator* did to the dock during the frame — it
    /// does not see a programmatic edit. Without this call the panel would
    /// appear and then vanish on the next restart, which reads as a bug in
    /// persistence rather than as a design choice about mounting.
    fn show_panel(&mut self, panel: crate::panels::Panel) {
        let id = egui_shell::dock::PanelId::new(panel.command_id());
        if self.dock.activate(&id) {
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed.
                    "panel-shown id={} mounted=already",
                    id.as_str()
                )
            });
            return;
        }

        let home = crate::app::modes::layout_for("edit").find(&id);
        let (side, column, stack) = home.map_or((egui_shell::dock::DockSide::Right, 0, 0), |a| {
            (a.side, a.column, a.stack)
        });
        self.dock
            .layout_mut()
            .mount(side, column, stack, id.clone());
        self.dock.normalize();
        let shown = self.dock.activate(&id);
        self.modes
            .record_layout(self.dock.layout(), &mut self.layout);
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed.
                "panel-shown id={} mounted=now side={} shown={shown}",
                id.as_str(),
                side.key(),
            )
        });
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

impl eframe::App for PdfceApp {
    /// eframe 0.35's entry point is `ui`, **not** `update`.
    ///
    /// The trait hands a root [`egui::Ui`] rather than a [`egui::Context`]
    /// (`eframe-0.35.0/src/epi.rs:176`), and panels are added *inside* that
    /// `Ui` — `CentralPanel::show(ui, …)`, not `show(ctx, …)`. Anyone
    /// arriving from an older eframe, or from a code sample, will write
    /// `update` and get a "not a member of trait" error whose message does
    /// not say what to write instead; hence this note.
    ///
    /// The context is cloned out at the top because the raster bookkeeping
    /// needs it after the panel closure has ended, and `Ui::ctx()` borrows.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // Step 1 — keyboard, before any widget can consume a key.
        let page_count = match &self.status {
            Status::Open(doc) => Some(doc.pages.len()),
            _ => None,
        };
        let mut actions = keyboard::collect(&ctx, page_count);

        // Step 1a — the chords the MANIFEST binds.
        //
        // ★ This is the second half of the one-owner-per-chord fix. The
        // keymap is data — `egui-shell` deliberately does not dispatch it,
        // because "the application owns the question of what has focus and
        // what a chord means" — so until now every binding in it was a
        // documented promise with nothing behind it, and `keyboard::collect`
        // quietly bound two of the same chords to something else.
        //
        // `keyboard::commands` returns command *ids*, which go through the
        // same dispatcher a ribbon click does. That is what makes a chord and
        // its button incapable of disagreeing.
        //
        // Owned rather than borrowed (`Vec<String>`) because dispatching
        // needs `&mut self` and the keymap lives in `self.shell`. It is empty
        // on all but the handful of frames where a chord was actually
        // pressed.
        let chord_commands =
            keyboard::commands(&ctx, self.shell.as_ref().and_then(|s| s.keymap.as_ref()));
        for id in chord_commands {
            self.dispatch_command(&id, &mut actions);
        }

        // Step 1b — the ribbon, above the canvas.
        //
        // Added before the `CentralPanel` because panel composition order is
        // load-bearing for both geometry and Tab focus: a panel added later
        // carves its space from what is left, so the canvas must be last or
        // it takes the whole window and the ribbon draws over it.
        //
        // The shell executes nothing. `show` returns the handler tokens the
        // operator invoked this frame, and they are translated into
        // `Action`s here — the same one-choke-point discipline every other
        // surface follows. A token with no arm yet is not an error; at S2
        // most of the ribbon is scaffolding for behaviour that lands later,
        // and `dispatch_token` says so per token rather than silently.
        self.ribbon_band(ui, &mut actions);

        // Step 1b² — the status bar, before the docks.
        //
        // **Order, not preference.** This module's own header states the rule
        // the old shell was bitten by: *"a full-width bar must be added
        // before any side panel, or it starts at the side panel's edge
        // instead of spanning the window."* A status bar that stops at the
        // dock is not a status bar.
        //
        // `exact_size`, never `default_height`: a content-driven status
        // height and a per-frame fit-to-viewport zoom form a measured
        // feedback loop — 230 % → 224 % → 215 % drift from a status line
        // that grew (R128, `D:\dev\rag\egui\bottom_panel_height_...md`).
        egui::Panel::bottom("status")
            .exact_size(crate::app::status::HEIGHT_PTS)
            .show(ui, |ui| {
                crate::app::status::show(ui, &self.status, &mut actions);
            });

        // Step 1c — the docks, between the ribbon and the canvas.
        //
        // Order is load-bearing twice over. The ribbon is a full-width bar
        // and must be added *before* any side panel, or it would start at
        // the dock's edge instead of spanning the window. The canvas must
        // be added *after* both, because a `CentralPanel` takes whatever is
        // left and there must be something left for it to take.
        self.docks(ui, &mut actions);

        // Step 2 — compose. Nothing here mutates a document; surfaces push
        // onto `actions`.
        egui::CentralPanel::default().show(ui, |ui| {
            // Declare the panel's own rect before drawing into it. This is
            // the outermost named region the application owns, and it is the
            // one a screenshot oracle uses to tell "the control is drawn but
            // clipped out of its pane" from "the control is not drawn" —
            // `PROJECT_PLAN.md` §4.2 prerequisite 2 records two cases where a
            // traced rect was correct and the control was still clipped.
            crate::diag::ui_rect(REGION_CENTRAL_PANEL, ui.max_rect());
            self.central(ui, &mut actions);
        });

        // Step 3 — apply, after the frame is drawn.
        let pixels_per_point = ctx.pixels_per_point();
        self.apply_actions(actions, pixels_per_point);

        // Step 4 — decide whether the picture on screen still matches the
        // state that was just updated, and start a render if not.
        self.settle_and_rasterize(&ctx, pixels_per_point);
    }
}

impl PdfceApp {
    /// The central area: the canvas when a document is open, and an
    /// explanation when one is not.
    ///
    /// Each non-open state renders **one sentence and nothing else**. There
    /// is deliberately no "Open…" button, no retry, and no password field:
    /// S0 opens the file named on the command line and has no other way to
    /// open anything, so a control here would either not exist or not work.
    /// Saying plainly what happened, and what the operator can do about it
    /// outside the application, is the honest version of that.
    fn central(&mut self, ui: &mut egui::Ui, actions: &mut Vec<actions::Action>) {
        // The status message's own rect, for whichever non-open arm draws
        // one. Declared through `ui_rect` so a legibility check measures the
        // text the application actually laid out rather than a fraction of
        // the window written into the harness — see `crate::diag::ui_rect`.
        //
        // `.inner.rect` and not `.response.rect`: `centered_and_justified`
        // returns the JUSTIFIED CONTAINER as its response, which is the whole
        // available area, while the label is drawn centred inside it.
        // Reporting the container would name a region that is mostly empty
        // background, and a contrast measurement over it would be dominated
        // by pixels the sentence never touched — a real measurement of the
        // wrong thing, which is the failure mode this whole mechanism exists
        // to prevent.
        // Built before the `&mut self.status` borrow, because the host reads
        // the shell and the registry that live beside it.
        let conditions = self.conditions();
        let host = self
            .shell
            .as_ref()
            .map(|s| crate::shell::menus::MenuHost::new(s, &self.commands, &conditions));

        // The canvas is drawn first and dispatched second, in two statements
        // rather than one match arm, because `dispatch_token` needs `&self` —
        // `format.delete` reads the selection off the open document — and the
        // arm holds `&mut self.status`. Letting the borrow end at the `if let`
        // is the whole reason this is not simply the first arm below.
        if let Status::Open(doc) = &mut self.status {
            let tokens = crate::canvas::show(ui, doc, host.as_ref(), actions);
            // Dispatched here rather than inside `show`: the canvas reports
            // intent and the application decides, which is the same seam the
            // ribbon and the dock already use.
            for token in tokens {
                self.dispatch_token(token, actions);
            }
            return;
        }

        let message = match &self.status {
            // ui-text-exempt: a panic message, read from a stack trace by
            // whoever broke the two-statement structure above. Never rendered.
            Status::Open(_) => unreachable!("handled above, before the borrow ended"),
            Status::Empty => {
                ui.centered_and_justified(|ui| ui.label(crate::text::canvas_no_document()))
            }
            Status::Failed { path, message } => {
                let text = crate::text::open_failed(path, message);
                ui.centered_and_justified(|ui| ui.colored_label(ui.visuals().error_fg_color, text))
            }
            Status::Unsupported { path, message } => {
                // Deliberately NOT `error_fg_color`. "pdfce cannot do this
                // yet" is not an error in the operator's document, and
                // painting it red would say it was — the same conflation the
                // three-way status split exists to avoid.
                let text = crate::text::open_unsupported(path, message);
                ui.centered_and_justified(|ui| ui.label(text))
            }
            Status::NeedsPassword { path } => {
                let text = crate::text::open_needs_password(path);
                ui.centered_and_justified(|ui| ui.label(text))
            }
        };
        crate::diag::ui_rect(REGION_STATUS_MESSAGE, message.inner.rect);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::selection::{ClickHit, SelectionLevel};
    use crate::canvas::target::TargetId;
    use crate::panels::objects::test_support::engine_fixture;

    /// An application with a four-page fixture open, and nothing selected.
    fn opened() -> PdfceApp {
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
    fn select_object(app: &mut PdfceApp, index: u64, shift: bool) {
        let Status::Open(doc) = &mut app.status else {
            panic!("no document open") // ui-text-exempt: test panic, never displayed
        };
        let page = doc.view.page_index;
        doc.selection.click(
            page,
            ClickHit {
                object: Some(TargetId(index)),
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
    #[test]
    fn the_selection_condition_follows_the_selection() {
        let mut app = PdfceApp::new();
        assert!(
            !app.conditions().is_set("selection.any"),
            "nothing is open, so nothing can be selected"
        );

        app = opened();
        assert!(app.conditions().is_set("doc.pages"));
        assert!(
            !app.conditions().is_set("selection.any"),
            "a freshly opened document has nothing selected"
        );

        select_object(&mut app, 1, false);
        assert!(app.conditions().is_set("selection.any"));

        // Escape at the Object rung clears, and the condition follows it back
        // down — a tab that stayed visible over an empty selection would offer
        // a Delete with nothing to delete.
        let Status::Open(doc) = &mut app.status else {
            unreachable!()
        };
        doc.selection.escape();
        assert!(!app.conditions().is_set("selection.any"));
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
        let mut app = opened();
        let delete = token_for(&app, "format.delete");

        // Nothing selected: nothing raised. An empty batch would be an action
        // the engine has to refuse, reported as a failure the operator caused.
        let mut actions = Vec::new();
        app.dispatch_token(delete, &mut actions);
        assert!(actions.is_empty());

        select_object(&mut app, 2, false);
        select_object(&mut app, 0, true);
        let mut actions = Vec::new();
        app.dispatch_token(delete, &mut actions);
        assert_eq!(
            actions,
            vec![Action::DeleteSelection {
                page: 0,
                objects: vec![0, 2],
            }],
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
        let mut app = opened();
        select_object(&mut app, 1, false);

        let Status::Open(doc) = &mut app.status else {
            unreachable!()
        };
        // Double-click into part 1 of the selected object.
        doc.selection.click(
            0,
            ClickHit {
                object: Some(TargetId(1)),
                part: Some(1),
                node: None,
            },
            false,
            true,
        );
        assert_eq!(doc.selection.level(), SelectionLevel::Part);

        let delete = token_for(&app, "format.delete");
        let mut actions = Vec::new();
        app.dispatch_token(delete, &mut actions);
        assert!(
            actions.is_empty(),
            "the Part rung has no delete verb wired, and the ribbon must not \
             borrow the Object rung's any more than the key may"
        );

        // …and the tab is still visible, which is why the decline has to be
        // handled rather than made unreachable: something IS selected.
        assert!(app.conditions().is_set("selection.any"));
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
        let mut app = opened();
        let properties =
            egui_shell::dock::PanelId::new(crate::panels::Panel::Properties.command_id());
        let token = token_for(&app, "file.properties");

        for mode in ["read", "review", "edit"] {
            app.modes
                .on_mode_changed(mode, &mut app.dock, &mut app.layout, &app.panel_registry);

            let mut actions = Vec::new();
            app.dispatch_token(token, &mut actions);
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
            app.dispatch_token(token, &mut Vec::new());
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
        let mut app = opened();
        app.modes
            .on_mode_changed("edit", &mut app.dock, &mut app.layout, &app.panel_registry);
        let default = crate::app::modes::layout_for_build("edit", &app.panel_registry);
        let reset = token_for(&app, "view.reset_layout");

        let mut actions = Vec::new();
        app.dispatch_token(reset, &mut actions);
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
        app.dispatch_token(reset, &mut Vec::new());
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
        let mut app = opened();
        let keymap = app
            .shell
            .as_ref()
            .and_then(|s| s.keymap.as_ref())
            .expect("the built-in manifest binds chords");
        let bound = keymap.get("Ctrl+0").expect("Ctrl+0 is bound").to_owned();

        let mut from_chord = Vec::new();
        app.dispatch_command(&bound, &mut from_chord);

        let mut from_button = Vec::new();
        app.dispatch_token(token_for(&app, &bound), &mut from_button);

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
        let mut app = opened();
        for (command, mode) in [
            ("mode.review", "review"),
            ("mode.edit", "edit"),
            ("mode.read", "read"),
        ] {
            app.dispatch_command(command, &mut Vec::new());
            assert_eq!(app.ribbon.mode(), Some(mode));
        }
    }
}
