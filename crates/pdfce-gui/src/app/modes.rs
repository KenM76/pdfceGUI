//! # `app::modes` — Read / Review / Edit as named workspaces
//!
//! `MODES_AND_PANELS.md` closes its analysis by identifying the two
//! requests that arrived together — a three-position mode selector, and
//! flexible panel areas — as one system, in one sentence:
//!
//! > **A mode is capability (g).** Read, Review and Edit are three built-in
//! > named workspaces, shipped as defaults, each remembering the operator's
//! > arrangement of it.
//!
//! This module is that sentence, implemented. It binds each mode the
//! **manifest** declares to a named workspace in the layout document that
//! [`crate::app::persistence`] keeps on disk, so that:
//!
//! - each mode starts from a default arrangement suited to what that mode
//!   is *for*;
//! - the operator's own rearrangement of a mode is remembered, per mode;
//! - Read → Edit → Read restores **your Edit**, not a default.
//!
//! ## ★ Three modes are configuration, not a built-in — on both sides
//!
//! `egui-shell`'s workspace store ships **no names at all**, and says why:
//! *"an application that wants three modes registers three workspaces; one
//! that wants eleven registers eleven; one that wants none never calls this
//! module."* `SHELL_FRAMEWORK.md` §4 states the same rule from the
//! manifest's side — *Read/Review/Edit is a configuration, not a built-in*.
//!
//! That rule binds **here** too, and this module honours it in the one way
//! that matters: [`Modes`] holds whatever mode ids
//! `crate::shell::manifest::built_in` declares, in that order, and has no
//! opinion about how many there are or what they are called. Adding a
//! fourth mode to the manifest is one line in the manifest; nothing here
//! changes, and nothing here needs to.
//!
//! What *is* pdfce's business, and therefore is here, is the **default
//! arrangement per mode** — see [`layout_for`]. `egui-shell` cannot supply
//! that: it does not know what a panel is for, so a default arrangement
//! invented there would be the framework inventing an application's
//! information architecture.
//!
//! ## The three defaults, and where they come from
//!
//! `MODES_AND_PANELS.md` Part 1's table, reduced to what the *dock* does:
//!
//! | Mode | Left | Right |
//! |---|---|---|
//! | **Read** | Pages, Bookmarks | — |
//! | **Review** | Pages, Bookmarks | Comments, Properties |
//! | **Edit** | Pages, Bookmarks / Layers, Signatures, Fonts | Objects / Properties, Comments |
//!
//! Read is the point of the whole feature — *"a PDF viewer, with pdfce's
//! inspection panels available but nothing that authors anything"* — so its
//! default mounts the two surfaces that answer *where am I* and nothing
//! that describes an object you are not allowed to edit. Review adds the
//! two surfaces markup work needs. Edit is everything, with **Objects on
//! the right**, opposite the navigators, because an inspector and a
//! navigator are consulted in different directions.
//!
//! A mode this module has never heard of gets the **full** arrangement: a
//! mode with no opinion recorded about it should not have panels taken
//! away, because removing is the opinionated act.
//!
//! ## ★ Panels this build does not have
//!
//! The defaults above name `view.panel_pages` and `view.panel_comments`,
//! and **this build registers neither** — see [`ABSENT_PANELS`]. That is
//! deliberate, and it is the `SHELL_FRAMEWORK.md` §5b mechanism rather than
//! an oversight: [`layout_for_build`] filters every default through the
//! live [`PanelCatalog`], so an id nothing registers is simply not mounted,
//! and the day a Pages panel is registered under that id it appears in the
//! defaults with no edit here at all.
//!
//! Writing the *intended* arrangement and filtering it is strictly better
//! than writing only what exists today, because the alternative is that the
//! intent lives in a document nobody re-reads when the panel lands.
//! `every_default_panel_is_registered_or_declared_absent` is what keeps
//! [`ABSENT_PANELS`] honest in both directions.
//!
//! ## What a mode change must **not** do
//!
//! `MODES_AND_PANELS.md` Part 1's behavioural rules, and the first two are
//! the ones this module is accountable for:
//!
//! > 1. **Switching modes never destroys work.** Read ⇄ Edit is a view
//! >    stance, not a save boundary. Unsaved edits survive a trip through
//! >    Read mode untouched.
//! > 2. **The undo stack is not cleared, ever.**
//!
//! That is enforced **structurally** rather than by care:
//! [`Modes::on_mode_changed`] takes a [`DockState`] and a
//! [`LayoutStore`], and neither of them can reach a document, a selection,
//! an edit session or an undo stack. There is no path from this module to
//! any of them, and `switching_modes_touches_neither_the_document_nor_the_selection`
//! asserts the consequence against a real open document anyway — because a
//! later edit that *adds* such a path should fail a test rather than pass a
//! review.
//!
//! ## What this module deliberately does not do
//!
//! - **It does not change which tabs the ribbon shows.** That is the
//!   manifest's `Mode::tabs` and `egui-shell`'s renderer; a mode's tab set
//!   and a mode's panel arrangement are two different things that happen to
//!   share a name.
//! - **It does not set the page-display default.** `MODES_AND_PANELS.md`
//!   Part 1: *"Read defaults to continuous scroll; Review and Edit default
//!   to single page."* That is a `crate::viewer` concern — there is no
//!   continuous-scroll display mode in this build yet — and it is recorded
//!   here only so the next reader knows it is a known, deliberate omission
//!   rather than a missed row of the table.
//! - **It does not decide the default mode.** Part 1 rule 5 makes that a
//!   setting; today `crate::app::PdfceApp::new` starts in the manifest's
//!   first mode and [`start`] adopts whatever it is handed.

use egui_shell::dock::{Column, DockLayout, DockState, PanelCatalog, PanelId, SideLayout, Stack};
use egui_shell::layout::ResetScope;
use egui_shell::manifest::Shell;

use crate::app::persistence::LayoutStore;
use crate::panels::Panel;

/// The prefix every mode-owned workspace name carries.
///
/// A workspace name is free text an operator chooses, so a mode's own
/// workspace has to be distinguishable from one the operator made and
/// happened to call "Read". The prefix does that, and it does two more
/// things worth having:
///
/// - it is the **mode id**, not the label, so renaming or translating
///   "Review" does not orphan the arrangement behind it;
/// - it makes the machine-owned entries filterable, so a future "load
///   workspace" menu can list the operator's own and leave these out — see
///   [`mode_of_workspace`].
pub const MODE_WORKSPACE_PREFIX: &str = "mode:"; // ui-text-exempt: a key prefix inside a file, never displayed

/// The workspace name that holds `mode_id`'s remembered arrangement.
#[must_use]
pub fn workspace_name(mode_id: &str) -> String {
    format!("{MODE_WORKSPACE_PREFIX}{mode_id}")
}

/// The mode a workspace belongs to, if it is a mode's rather than an
/// operator's.
#[must_use]
pub fn mode_of_workspace(name: &str) -> Option<&str> {
    name.strip_prefix(MODE_WORKSPACE_PREFIX)
}

/// **Panel ids the defaults name that this build does not register, and
/// why.**
///
/// `(id, reason)`, in the shape and for the reasons
/// `crate::shell::manifest::PLANNED` uses for absent *commands*: an
/// omission that is data can be tested, enumerated and grepped, whereas an
/// omission that is a comment becomes stale the day it stops being true.
///
/// Tested in both directions by
/// `every_default_panel_is_registered_or_declared_absent`: nothing in a
/// default layout may be missing from both `Panel::ALL` and this list, and
/// nothing in this list may already exist as a panel. So the day either
/// panel lands, the suite fails until this entry is removed — which is the
/// same commit in which the default starts mounting it.
pub const ABSENT_PANELS: &[(&str, &str)] = &[
    (
        "view.panel_pages",
        // ui-text-exempt: developer note about an ABSENT panel; never rendered.
        "N — page thumbnails have no panel in this build; `shell::manifest::PLANNED` \
         carries the matching command. Named here because Read and Review are \
         thumbnail-first arrangements and the default should acquire it the day it \
         is registered, not the day someone re-reads MODES_AND_PANELS.md.",
    ),
    (
        "view.panel_comments",
        // ui-text-exempt: developer note about an ABSENT panel; never rendered.
        "N — annotation authoring does not exist yet, so neither does the panel that \
         lists comments. It is what Review's right dock is FOR, per Part 1's table, \
         so the arrangement names it and mounts nothing until it is real.",
    ),
];

/// One side's default arrangement: a list of stacks, each a list of tabs.
///
/// A single column per side, deliberately. Multiple columns are what the
/// dock is *for* — a narrow navigator beside a wide inspector — but they
/// are an arrangement the operator reaches by widening and splitting, not
/// one to hand somebody on their first launch. The model expresses them;
/// the defaults do not use them.
///
/// **Owned rather than a `&'static` table**, for one reason worth stating
/// because the static form is the obvious first attempt and does not
/// compile: [`Panel::command_id`] is an ordinary function, so its result
/// cannot be promoted into a `'static` slice literal. The alternative is a
/// table of string literals plus a test asserting each one still matches
/// its panel — a second spelling of every id, kept in step by a test rather
/// than by construction. Two `Vec`s built on a mode change are cheaper than
/// that, in every sense.
type SideSpec = Vec<Vec<&'static str>>;

/// Both sides of one mode's default arrangement.
struct ModeSpec {
    /// The leading-edge dock's stacks, top to bottom.
    left: SideSpec,
    /// The trailing-edge dock's stacks, top to bottom.
    right: SideSpec,
    /// The left dock's width in points.
    left_width: f32,
    /// The right dock's width in points.
    right_width: f32,
}

/// The Pages panel's id. Absent from this build — see [`ABSENT_PANELS`].
const PAGES: &str = "view.panel_pages";
/// The Comments panel's id. Absent from this build — see [`ABSENT_PANELS`].
const COMMENTS: &str = "view.panel_comments";

/// The default width of a navigator dock, in points.
///
/// Wide enough for two columns of page thumbnails, which is the measurement
/// that decides this number: a thumbnail rail one column wide wastes the
/// dock, and three columns makes each too small to recognise a drawing by.
const NAVIGATOR_WIDTH: f32 = 280.0;

/// The default width of an inspector dock, in points.
///
/// Wider than a navigator because its rows are `label: value` pairs whose
/// values are paths, font names and coordinate triples — content that wraps
/// badly and reads terribly when it does.
const INSPECTOR_WIDTH: f32 = 320.0;

/// The default arrangement for `mode_id`, **before** this build's panels
/// are taken into account.
///
/// The intended arrangement, naming every panel the mode is specified to
/// offer whether or not this build has it. Almost every caller wants
/// [`layout_for_build`] instead; this exists so the intent is expressible,
/// testable and readable on its own.
///
/// An unrecognised `mode_id` gets the full arrangement — see the module
/// header on why removing is the opinionated act.
#[must_use]
pub fn layout_for(mode_id: &str) -> DockLayout {
    build(&spec(mode_id), None)
}

/// The default arrangement for `mode_id`, with panels this build does not
/// register dropped and whatever they emptied pruned.
///
/// This is the one an application calls. `SHELL_FRAMEWORK.md` §5b: a
/// capability's presence is expressed by registering it and by nothing
/// else, so a default that mounts a panel nothing registers must mount
/// nothing rather than produce a tab whose body cannot be drawn.
///
/// The filter runs over the same [`PanelCatalog`] the dock and the layout
/// loader use, so "what a fresh profile starts with" and "what a saved
/// layout is allowed to contain" cannot disagree.
#[must_use]
pub fn layout_for_build(mode_id: &str, catalog: &dyn PanelCatalog) -> DockLayout {
    build(&spec(mode_id), Some(catalog))
}

/// The specification for one mode.
///
/// The `match` is the one place in this crate that knows what "read" means
/// as an *arrangement*. Note what it is not: it is not a list of the modes
/// that exist. [`Modes`] takes that from the manifest, so a mode with no
/// arm here still works — it simply starts from the full arrangement.
fn spec(mode_id: &str) -> ModeSpec {
    match mode_id {
        // Read — a reader. The two surfaces that answer "where am I", and
        // nothing that describes an object the mode does not let you touch.
        // No Objects, no Properties: Part 1's table gives Read neither, and
        // an inspector in a mode with no edit verbs is a panel whose every
        // row is a fact you cannot act on.
        "read" => ModeSpec {
            left: vec![vec![PAGES, Panel::Bookmarks.command_id()]],
            right: Vec::new(),
            left_width: NAVIGATOR_WIDTH,
            right_width: INSPECTOR_WIDTH,
        },
        // Review — the markup stance. Read's navigators, plus the two
        // surfaces markup work needs: the comment list you are working
        // through, and the properties of the markup you are placing.
        // Properties is scoped to markup in this mode by the *mode*, not by
        // the dock; the dock mounts one panel either way.
        "review" => ModeSpec {
            left: vec![vec![PAGES, Panel::Bookmarks.command_id()]],
            right: vec![vec![COMMENTS, Panel::Properties.command_id()]],
            left_width: NAVIGATOR_WIDTH,
            right_width: INSPECTOR_WIDTH,
        },
        // Edit — everything. Two stacks per side rather than one long tab
        // bar, because the previous implementation's reasoning still holds:
        // *"reaching one surface must not hide another you are using AT THE
        // SAME TIME"*. Navigating pages while reading the layer list is one
        // such pair; picking an object while reading its properties is the
        // other, and it is why Objects and Properties are separate stacks
        // rather than two tabs of one.
        "edit" => ModeSpec {
            left: vec![
                vec![PAGES, Panel::Bookmarks.command_id()],
                vec![
                    Panel::Layers.command_id(),
                    Panel::Signatures.command_id(),
                    Panel::Fonts.command_id(),
                ],
            ],
            right: vec![
                vec![Panel::Objects.command_id()],
                vec![Panel::Properties.command_id(), COMMENTS],
            ],
            left_width: NAVIGATOR_WIDTH,
            right_width: INSPECTOR_WIDTH,
        },
        // A mode this module has no opinion about. The full arrangement,
        // for the reason in the module header — and it is reachable: an
        // operator's customized manifest may declare a fourth mode, and a
        // fourth mode with an empty dock would look like a broken build.
        _ => spec("edit"),
    }
}

/// Turn a specification into a layout, optionally filtered by a catalog.
fn build(spec: &ModeSpec, catalog: Option<&dyn PanelCatalog>) -> DockLayout {
    let mut layout = DockLayout::new(
        side(&spec.left, spec.left_width, catalog),
        side(&spec.right, spec.right_width, catalog),
    );
    // Cheap, and it is what lets `layout_for` be asserted `is_normalized`
    // rather than relying on `DockState::new` to repair a compiled-in
    // constant — the posture the dock's own docs ask an application to
    // take towards its defaults.
    layout.normalize();
    layout
}

/// Build one side, dropping unregistered panels and pruning what they
/// empty.
fn side(
    stacks: &[Vec<&'static str>],
    width: f32,
    catalog: Option<&dyn PanelCatalog>,
) -> SideLayout {
    let kept: Vec<Stack> = stacks
        .iter()
        .filter_map(|tabs| {
            let tabs: Vec<PanelId> = tabs
                .iter()
                .copied()
                .filter(|id| catalog.is_none_or(|c| c.contains(id)))
                .map(PanelId::new)
                .collect();
            (!tabs.is_empty()).then(|| Stack::tabbed(tabs))
        })
        .collect();

    if kept.is_empty() {
        // Not an empty visible side: a side with no columns that is still
        // marked visible is how an application ends up with a permanent
        // grey stripe nobody can remove.
        return SideLayout::none();
    }
    SideLayout::new([Column::new(kept)]).with_width(width)
}

/// The modes this application has, and which one is in force.
///
/// The ids come from the **manifest**, in the order it declares them, and
/// this type has no opinion about how many there are or what they are
/// called — see the module header. What it owns is the binding between a
/// mode and its remembered arrangement.
#[derive(Debug, Default, Clone)]
pub struct Modes {
    /// Every mode id the manifest declares, in declaration order.
    ids: Vec<String>,
    /// The mode in force, or `None` before the first adoption.
    ///
    /// `None` is meaningful rather than a placeholder: it is what makes the
    /// first [`Self::on_mode_changed`] an *adoption* — there is no outgoing
    /// arrangement to remember, because nothing has been arranged yet.
    active: Option<String>,
}

impl Modes {
    /// Take the mode list from a shell manifest.
    ///
    /// `None` — a manifest that failed to validate — yields no modes, and
    /// every method below then declines rather than inventing one. A build
    /// whose ribbon could not be assembled must not silently acquire a
    /// three-position mode model from somewhere else.
    #[must_use]
    pub fn from_shell(shell: Option<&Shell>) -> Self {
        Self {
            ids: shell
                .map(|s| s.modes().iter().map(|m| m.id.clone()).collect())
                .unwrap_or_default(),
            active: None,
        }
    }

    /// Every mode id, in the manifest's order.
    #[must_use]
    pub fn ids(&self) -> &[String] {
        &self.ids
    }

    /// The first mode the manifest declares — what the application opens
    /// in until the default-mode setting exists (Part 1 rule 5).
    #[must_use]
    pub fn first(&self) -> Option<&str> {
        self.ids.first().map(String::as_str)
    }

    /// The mode in force, if one has been adopted.
    #[must_use]
    pub fn active(&self) -> Option<&str> {
        self.active.as_deref()
    }

    /// Whether the manifest declares this mode.
    #[must_use]
    pub fn is_known(&self, mode_id: &str) -> bool {
        self.ids.iter().any(|id| id == mode_id)
    }

    /// **Adopt `mode_id`: remember the outgoing mode's arrangement, and
    /// restore the incoming one's.**
    ///
    /// The whole feature, in five steps:
    ///
    /// 1. A mode the manifest does not declare is declined — an unknown id
    ///    must not acquire a workspace, or a typo in a customized manifest
    ///    would quietly accumulate arrangements nothing can ever restore.
    /// 2. Re-adopting the mode already in force does nothing, so a caller
    ///    may drive this straight from `RibbonState::mode()` every frame
    ///    without checking first.
    /// 3. The **outgoing** mode's workspace is written from what is on
    ///    screen right now. This is what makes "each mode remembers your
    ///    arrangement of it" true without an explicit save.
    /// 4. The **incoming** mode's workspace is restored if it has one, and
    ///    otherwise its built-in default is used — filtered through
    ///    `catalog`, so a saved-but-stale panel and a compiled-out one are
    ///    handled identically.
    /// 5. The result is recorded, which arms the debounced write. A crash
    ///    after a mode change therefore costs nothing.
    ///
    /// Returns whether the arrangement was changed.
    ///
    /// **It cannot touch a document.** See the module header: the argument
    /// list is the proof, and a test asserts the consequence anyway.
    pub fn on_mode_changed(
        &mut self,
        mode_id: &str,
        dock: &mut DockState,
        store: &mut LayoutStore,
        catalog: &dyn PanelCatalog,
    ) -> bool {
        if !self.is_known(mode_id) {
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed.
                    "mode-unknown id={mode_id}"
                )
            });
            return false;
        }
        if self.active.as_deref() == Some(mode_id) {
            return false;
        }

        if let Some(from) = self.active.clone() {
            store
                .document_mut()
                .save_workspace(workspace_name(&from), dock.layout().clone());
        }

        let restored = store
            .document()
            .workspace(&workspace_name(mode_id))
            .cloned();
        let remembered = restored.is_some();
        dock.set_layout(restored.unwrap_or_else(|| layout_for_build(mode_id, catalog)));

        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed.
                "mode-changed from={:?} to={mode_id} remembered={remembered} panels={}",
                self.active,
                dock.layout().panels().count(),
            )
        });

        self.active = Some(mode_id.to_owned());
        self.record_layout(dock.layout(), store);
        true
    }

    /// Record the live arrangement as both the document's `active` and the
    /// current mode's workspace.
    ///
    /// Called when the dock reports
    /// [`egui_shell::dock::DockFrameReport::layout_changed`].
    ///
    /// **Both, and that is the point.** The document's `active` is the
    /// arrangement in force; the mode's workspace is the arrangement to
    /// come back to. Writing only the first would mean a crash mid-session
    /// cost the operator every rearrangement they had made since the last
    /// mode change, which is the "only saved at exit" failure wearing a
    /// different hat. Writing only the second would leave `active` stale in
    /// a file an operator may read.
    ///
    /// Idempotent: recording an arrangement that is already recorded arms
    /// no write, so a caller that calls it unconditionally costs nothing.
    pub fn record_layout(&self, layout: &DockLayout, store: &mut LayoutStore) {
        store.record_active(layout);
        let Some(active) = self.active.as_deref() else {
            return;
        };
        let name = workspace_name(active);
        if store.document().workspace(&name) == Some(layout) {
            return;
        }
        store.document_mut().save_workspace(name, layout.clone());
    }

    /// Restore part of the current mode's arrangement to its default.
    ///
    /// `RIBBON_IA.md`'s rule is why this has a scope at all: *"an operator
    /// who only wanted the right dock back must not lose their left one."*
    /// The scoping itself is `egui-shell`'s; what this adds is the one
    /// thing the shell cannot know — **which** default, given that the
    /// right default depends on the mode in force.
    ///
    /// Saved workspaces are untouched, including the current mode's, which
    /// is then immediately overwritten by [`Self::record_layout`] with the
    /// reset arrangement. That is the intended reading of "reset this
    /// mode": the mode goes back to its default and remembers that it did.
    ///
    /// Returns whether anything changed.
    pub fn reset(
        &self,
        scope: ResetScope,
        dock: &mut DockState,
        store: &mut LayoutStore,
        catalog: &dyn PanelCatalog,
    ) -> bool {
        let default = layout_for_build(self.active.as_deref().unwrap_or_default(), catalog);
        let mut layout = dock.layout().clone();
        if !egui_shell::layout::reset::reset(&mut layout, scope, &default) {
            return false;
        }
        dock.set_layout(layout);
        self.record_layout(dock.layout(), store);
        true
    }
}

/// Everything the application needs to start with a persisted, mode-aware
/// dock.
///
/// Returned as a struct rather than a tuple because three values whose
/// types are `Modes`, `LayoutStore` and `DockState` are easy to bind in the
/// wrong order and hard to notice having done so.
pub struct Startup {
    /// The modes, with the opening one already adopted.
    pub modes: Modes,
    /// The layout file, already loaded. Its
    /// [`LayoutStore::report`] is what a status surface should disclose.
    pub layout: LayoutStore,
    /// The dock, already holding the opening mode's arrangement.
    pub dock: DockState,
}

/// Load the layout and adopt the manifest's first mode.
///
/// The whole start-up sequence, in one call, because its order is
/// load-bearing and getting it wrong is silent:
///
/// 1. The mode list comes from the manifest.
/// 2. The **fallback** handed to the loader is the opening mode's default,
///    so a first run — or a file that could not be parsed at all — starts
///    from an arrangement that suits the mode the application opens in
///    rather than from some other mode's.
/// 3. The document is loaded, fail-soft, with `catalog` deciding which
///    saved mounts this build can honour.
/// 4. The opening mode is adopted, which restores its remembered
///    arrangement if it has one.
///
/// ## Why step 4 may discard the file's `active` arrangement
///
/// The document's `active` is *"the arrangement in force"* — in force in
/// whichever mode was showing when the application last closed, which is
/// not necessarily the one it now opens in. The mode's own workspace is the
/// better answer to "what should Read look like", so it wins. `active` is
/// still kept current by [`Modes::record_layout`], because it is what a
/// person reading the file expects to find and what the loader falls back
/// to if a workspace has to be dropped.
#[must_use]
pub fn start(shell: Option<&Shell>, catalog: &dyn PanelCatalog) -> Startup {
    let modes = Modes::from_shell(shell);
    let fallback = layout_for_build(modes.first().unwrap_or_default(), catalog);
    assemble(modes, LayoutStore::load(&fallback, catalog), catalog)
}

/// [`start`], reading from an explicit directory. For tests and a future
/// `--user-data-dir` override.
#[must_use]
pub fn start_in(
    dir: &std::path::Path,
    shell: Option<&Shell>,
    catalog: &dyn PanelCatalog,
) -> Startup {
    let modes = Modes::from_shell(shell);
    let fallback = layout_for_build(modes.first().unwrap_or_default(), catalog);
    assemble(
        modes,
        LayoutStore::load_in(dir, &fallback, catalog),
        catalog,
    )
}

/// The shared tail of the two start-up paths.
fn assemble(mut modes: Modes, mut layout: LayoutStore, catalog: &dyn PanelCatalog) -> Startup {
    let mut dock = DockState::new(layout.active().clone());
    if let Some(first) = modes.first().map(str::to_owned) {
        modes.on_mode_changed(&first, &mut dock, &mut layout, catalog);
    }
    Startup {
        modes,
        layout,
        dock,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::PdfceApp;
    use crate::app::state::Status;
    use egui_shell::dock::{AnyPanel, DockSide, PanelInfo, PanelRegistry};
    use std::path::PathBuf;

    /// The panel registry a full build would have: every panel this crate
    /// actually implements.
    fn registry() -> PanelRegistry {
        let mut r = PanelRegistry::new();
        for panel in Panel::ALL {
            let id = panel.command_id();
            r.register(PanelInfo::new(id, id));
        }
        r
    }

    /// The manifest's real mode list.
    fn shell() -> Shell {
        crate::shell::manifest::built_in()
    }

    /// A fresh, empty directory nothing else is using.
    fn temp_dir(tag: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let dir = std::env::temp_dir().join(format!("pdfce-gui-modes-{tag}-{nanos}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("a temp dir");
        dir
    }

    /// Every panel id in a mode's default layout.
    fn ids(layout: &DockLayout) -> Vec<String> {
        layout.panels().map(|p| p.as_str().to_owned()).collect()
    }

    /// Every mode id, as string slices, for comparison against a literal
    /// array.
    fn names(modes: &Modes) -> Vec<&str> {
        modes.ids().iter().map(String::as_str).collect()
    }

    /// ★ **The mode list comes from the manifest, not from this module.**
    ///
    /// `SHELL_FRAMEWORK.md` §4 makes Read/Review/Edit a configuration
    /// rather than a built-in, and `egui-shell`'s workspace store refuses
    /// to ship three magic names for the same reason. This module is the
    /// third place that rule could have been broken — and the place where
    /// breaking it would look most reasonable, because it is the one that
    /// legitimately knows what "read" means as an arrangement.
    ///
    /// Asserted by driving a manifest with *different* modes: a fourth mode
    /// must be adoptable, and a mode the manifest does not declare must
    /// not be.
    #[test]
    fn the_mode_list_is_whatever_the_manifest_declares() {
        let real = Modes::from_shell(Some(&shell()));
        assert_eq!(names(&real), ["read", "review", "edit"]);
        assert_eq!(real.first(), Some("read"));

        let other = Shell::new()
            .with_mode(egui_shell::manifest::Mode::new(
                "proofing",
                "Proofing",
                ["file"],
            ))
            .with_mode(egui_shell::manifest::Mode::new(
                "drafting",
                "Drafting",
                ["file"],
            ));
        let modes = Modes::from_shell(Some(&other));
        assert_eq!(names(&modes), ["proofing", "drafting"]);
        assert_eq!(modes.first(), Some("proofing"));
        assert!(modes.is_known("drafting"));
        assert!(!modes.is_known("read"), "no mode is built in here");

        // A manifest that failed to validate leaves no modes at all rather
        // than a three-position model from nowhere.
        assert!(Modes::from_shell(None).ids().is_empty());
        assert!(Modes::from_shell(None).first().is_none());
    }

    /// ★ **Each mode's default is the arrangement `MODES_AND_PANELS.md`
    /// specifies.**
    ///
    /// Asserted on the *unfiltered* defaults, because that is where the
    /// intent lives: filtering is what this build's panel set does to it,
    /// and asserting the filtered form would make the test say less every
    /// time a panel is missing.
    #[test]
    fn the_three_defaults_are_the_specified_arrangements() {
        let read = layout_for("read");
        assert_eq!(ids(&read), [PAGES, Panel::Bookmarks.command_id()]);
        assert!(
            read.right.is_empty(),
            "Read is a reader: no inspector at all"
        );
        for absent in [Panel::Objects, Panel::Properties] {
            assert!(
                !read.contains(&PanelId::new(absent.command_id())),
                "{absent:?} must not be in Read's default"
            );
        }

        let review = layout_for("review");
        assert!(review.left.panels().any(|p| p.as_str() == PAGES));
        assert_eq!(
            review
                .right
                .panels()
                .map(PanelId::as_str)
                .collect::<Vec<_>>(),
            [COMMENTS, Panel::Properties.command_id()],
            "Review adds Comments and Properties, and nothing else"
        );
        assert!(!review.contains(&PanelId::new(Panel::Objects.command_id())));

        let edit = layout_for("edit");
        for panel in Panel::ALL {
            assert!(
                edit.contains(&PanelId::new(panel.command_id())),
                "Edit is everything, and {panel:?} is missing"
            );
        }
        let objects = edit
            .find(&PanelId::new(Panel::Objects.command_id()))
            .expect("Objects is mounted");
        assert_eq!(objects.side, DockSide::Right, "Objects is on the right");
    }

    /// A mode with no arm gets the full arrangement rather than an empty
    /// dock — a customized manifest's fourth mode must not look broken.
    #[test]
    fn an_unrecognised_mode_gets_the_full_arrangement() {
        assert_eq!(ids(&layout_for("proofing")), ids(&layout_for("edit")));
        assert_eq!(ids(&layout_for("")), ids(&layout_for("edit")));
    }

    /// Every default is already normalized, so a defect in a compiled-in
    /// constant fails here rather than being quietly patched on every
    /// machine that runs it.
    #[test]
    fn every_default_is_already_normalized() {
        for mode in ["read", "review", "edit", "something-else"] {
            let layout = layout_for(mode);
            assert!(layout.is_normalized(), "{mode} needed repair: {layout:?}");
            assert!(
                layout_for_build(mode, &registry()).is_normalized(),
                "{mode}, filtered, needed repair"
            );
        }
    }

    /// ★ **A panel this build does not have is not mounted, and takes
    /// nothing else with it.**
    ///
    /// `SHELL_FRAMEWORK.md` §5b applied to the *defaults* rather than to a
    /// saved file: the intended arrangement names Pages and Comments, this
    /// build registers neither, and what the operator gets is the rest of
    /// the arrangement — never a tab whose body cannot be drawn, and never
    /// an empty compartment where one used to be.
    #[test]
    fn a_default_drops_panels_this_build_does_not_register() {
        let registry = registry();
        let read = layout_for_build("read", &registry);
        assert_eq!(ids(&read), [Panel::Bookmarks.command_id()]);
        assert!(read.right.is_empty());

        let review = layout_for_build("review", &registry);
        assert_eq!(
            review
                .right
                .panels()
                .map(PanelId::as_str)
                .collect::<Vec<_>>(),
            [Panel::Properties.command_id()],
            "Comments went; Properties stayed"
        );

        // Read's whole left side is Pages + Bookmarks. A build with neither
        // must produce a side that draws NOTHING rather than an empty
        // bordered stripe nobody can remove.
        let empty = PanelRegistry::new();
        let bare = layout_for_build("read", &empty);
        assert!(bare.left.is_empty() && bare.right.is_empty());
        assert!(!bare.left.visible, "an empty side must not be visible");
    }

    /// ★ **Every panel a default names either exists or is declared
    /// absent.**
    ///
    /// [`ABSENT_PANELS`] is the `PLANNED` discipline applied to panels, and
    /// this is what keeps it honest in both directions: a default may not
    /// name an id that is neither implemented nor declared absent, and an
    /// id declared absent may not already exist. The second half is the one
    /// that matters over time — it makes the day a Pages panel lands a
    /// failing test rather than a stale comment.
    #[test]
    fn every_default_panel_is_registered_or_declared_absent() {
        let implemented: Vec<&str> = Panel::ALL.iter().map(|p| p.command_id()).collect();

        for mode in ["read", "review", "edit"] {
            for id in ids(&layout_for(mode)) {
                assert!(
                    implemented.contains(&id.as_str())
                        || ABSENT_PANELS.iter().any(|(absent, _)| *absent == id),
                    "`{id}` is mounted by {mode}'s default, is not a panel this build \
                     implements, and is not declared in ABSENT_PANELS. Implement it, \
                     remove it from the default, or declare it absent with a reason."
                );
            }
        }

        for (id, reason) in ABSENT_PANELS {
            assert!(
                !implemented.contains(id),
                "`{id}` is declared absent and yet `Panel::ALL` implements it. Remove \
                 the ABSENT_PANELS entry — the defaults will start mounting it."
            );
            assert!(
                !reason.is_empty(),
                "`{id}` is declared absent with no reason, which is the stale comment \
                 this list exists to replace"
            );
        }
    }

    /// A mode's workspace name is derived from the id, not the label, and
    /// is distinguishable from one the operator made.
    #[test]
    fn a_mode_workspace_is_named_by_id_and_is_recognisable() {
        assert_eq!(workspace_name("review"), "mode:review");
        assert_eq!(mode_of_workspace("mode:review"), Some("review"));
        assert_eq!(
            mode_of_workspace("Review"),
            None,
            "an operator's own workspace called Review is not a mode's"
        );
    }

    /// ★ **Read → Edit → Read restores YOUR Edit, not a default.**
    ///
    /// The behaviour the whole module exists for, and the one
    /// `MODES_AND_PANELS.md` Part 1 rule 3 states: *"Each mode remembers
    /// its own panel layout. Leaving Edit and coming back restores the
    /// arrangement, not a default."*
    #[test]
    fn a_mode_remembers_the_operators_own_arrangement_of_it() {
        let dir = temp_dir("remember");
        let registry = registry();
        let shell = shell();
        let Startup {
            mut modes,
            layout: mut store,
            mut dock,
        } = start_in(&dir, Some(&shell), &registry);

        assert_eq!(modes.active(), Some("read"));
        let read_default = dock.layout().clone();

        // Into Edit, and rearrange it the way an operator would: a wider
        // dock and a different tab selected.
        assert!(modes.on_mode_changed("edit", &mut dock, &mut store, &registry));
        assert_ne!(dock.layout(), &read_default);
        dock.layout_mut().left.width_pts = 461.0;
        assert!(dock.activate(&PanelId::new(Panel::Fonts.command_id())));
        let my_edit = dock.layout().clone();
        modes.record_layout(&my_edit, &mut store);

        // Back to Read: Read's own arrangement, untouched by what was done
        // in Edit.
        assert!(modes.on_mode_changed("read", &mut dock, &mut store, &registry));
        assert_eq!(dock.layout(), &read_default);

        // …and back to Edit: MINE, not the default.
        assert!(modes.on_mode_changed("edit", &mut dock, &mut store, &registry));
        assert_eq!(dock.layout(), &my_edit);
        assert_ne!(
            dock.layout(),
            &layout_for_build("edit", &registry),
            "the default came back instead of the operator's arrangement"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// ★ **…and it survives a restart.**
    ///
    /// The round trip that makes the previous test worth anything: the same
    /// sequence, through a real file, across two `Startup`s. A rearrangeable
    /// layout that forgets itself each restart is worse than a fixed one.
    #[test]
    fn a_modes_arrangement_survives_a_restart() {
        let dir = temp_dir("restart");
        let registry = registry();
        let shell = shell();

        let my_edit = {
            let Startup {
                mut modes,
                layout: mut store,
                mut dock,
            } = start_in(&dir, Some(&shell), &registry);
            modes.on_mode_changed("edit", &mut dock, &mut store, &registry);
            dock.layout_mut().right.width_pts = 377.0;
            let mine = dock.layout().clone();
            modes.record_layout(&mine, &mut store);
            assert!(store.flush(), "the change was outstanding");
            mine
        };

        let Startup {
            mut modes,
            layout: mut store,
            mut dock,
        } = start_in(&dir, Some(&shell), &registry);
        assert_eq!(modes.active(), Some("read"), "the session opens in Read");
        assert!(modes.on_mode_changed("edit", &mut dock, &mut store, &registry));
        assert_eq!(
            dock.layout(),
            &my_edit,
            "the arrangement did not survive the restart"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Adopting the mode already in force does nothing, so a caller may
    /// drive this from the ribbon's state every frame.
    #[test]
    fn re_adopting_the_current_mode_is_a_no_op() {
        let dir = temp_dir("idempotent");
        let registry = registry();
        let shell = shell();
        let Startup {
            mut modes,
            layout: mut store,
            mut dock,
        } = start_in(&dir, Some(&shell), &registry);

        store.flush();
        let saves = store.saves();
        let before = dock.layout().clone();
        for _ in 0..10 {
            assert!(!modes.on_mode_changed("read", &mut dock, &mut store, &registry));
        }
        assert_eq!(dock.layout(), &before);
        assert!(!store.is_dirty(), "a no-op must not arm a write");
        store.flush();
        assert_eq!(store.saves(), saves);

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A mode the manifest does not declare is declined rather than given a
    /// workspace of its own.
    #[test]
    fn an_undeclared_mode_is_declined() {
        let dir = temp_dir("undeclared");
        let registry = registry();
        let shell = shell();
        let Startup {
            mut modes,
            layout: mut store,
            mut dock,
        } = start_in(&dir, Some(&shell), &registry);

        let before = dock.layout().clone();
        assert!(!modes.on_mode_changed("proofing", &mut dock, &mut store, &registry));
        assert_eq!(modes.active(), Some("read"));
        assert_eq!(dock.layout(), &before);
        assert!(
            !store
                .document()
                .workspace_names()
                .contains(&"mode:proofing"),
            "an unknown mode must not accumulate a workspace"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// ★ **Switching modes touches neither the document nor the
    /// selection.**
    ///
    /// `MODES_AND_PANELS.md` Part 1 rule 1: *"Switching modes never
    /// destroys work. Read ⇄ Edit is a view stance, not a save boundary."*
    ///
    /// The argument list of [`Modes::on_mode_changed`] already makes this
    /// impossible — it can reach a `DockState` and a `LayoutStore`, and
    /// neither can reach an `EditSession` — so this test exists to make a
    /// *later* edit that widens that argument list fail here rather than
    /// pass a review.
    #[test]
    fn switching_modes_touches_neither_the_document_nor_the_selection() {
        use crate::canvas::selection::ClickHit;
        use crate::canvas::target::TargetId;

        let dir = temp_dir("no-loss");
        let mut app = PdfceApp::new();
        app.open_path(crate::panels::objects::test_support::engine_fixture(
            "pageops/four-pages.pdf",
        ));
        let Status::Open(doc) = &mut app.status else {
            panic!("the fixture opens")
        };
        doc.view.page_index = 2;
        doc.edit_epoch = 7;
        doc.selection.click(
            2,
            ClickHit {
                object: Some(TargetId(1)),
                ..ClickHit::default()
            },
            false,
            false,
        );

        let shell = shell();
        let mut modes = Modes::from_shell(Some(&shell));
        let mut store = LayoutStore::load_in(
            &dir,
            &layout_for_build("read", &app.panel_registry),
            &app.panel_registry,
        );
        for mode in ["read", "edit", "review", "read"] {
            modes.on_mode_changed(mode, &mut app.dock, &mut store, &app.panel_registry);
        }

        let Status::Open(doc) = &app.status else {
            panic!("the document is still open")
        };
        assert_eq!(doc.view.page_index, 2, "the view moved");
        assert_eq!(doc.edit_epoch, 7, "the document was touched");
        assert_eq!(doc.selection.len(), 1, "the selection was lost");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A scoped reset restores the mode's own default on one side and
    /// leaves the other alone — *"an operator who only wanted the right
    /// dock back must not lose their left one."*
    #[test]
    fn a_scoped_reset_restores_this_modes_default_on_one_side_only() {
        let dir = temp_dir("reset");
        let registry = registry();
        let shell = shell();
        let Startup {
            mut modes,
            layout: mut store,
            mut dock,
        } = start_in(&dir, Some(&shell), &registry);
        modes.on_mode_changed("edit", &mut dock, &mut store, &registry);

        dock.layout_mut().left.width_pts = 500.0;
        dock.layout_mut().right.width_pts = 500.0;
        let mangled_left = dock.layout().left.clone();

        assert!(modes.reset(ResetScope::Right, &mut dock, &mut store, &registry));
        assert_eq!(dock.layout().left, mangled_left, "the left dock moved");
        assert_eq!(
            dock.layout().right,
            layout_for_build("edit", &registry).right,
            "the right dock is not this mode's default"
        );
        // And the mode remembers that it was reset.
        assert_eq!(
            store.document().workspace("mode:edit"),
            Some(dock.layout()),
            "the reset arrangement was not recorded"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A build with no manifest still starts: no modes, the full
    /// arrangement, and a dock that works.
    #[test]
    fn a_build_with_no_manifest_still_gets_a_working_dock() {
        let dir = temp_dir("no-manifest");
        let Startup {
            modes,
            layout: store,
            dock,
        } = start_in(&dir, None, &AnyPanel);
        assert!(modes.ids().is_empty());
        assert_eq!(modes.active(), None);
        assert!(dock.layout().panels().count() > 0, "the dock is not empty");
        assert!(!store.is_noteworthy());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
