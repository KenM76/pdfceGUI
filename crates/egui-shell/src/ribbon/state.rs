//! [`RibbonState`] — the two facts the ribbon itself owns, and the report
//! from the last frame it drew.
//!
//! # What is deliberately *not* in here
//!
//! Nothing the application owns. The condition set, the registry and the
//! manifest are all supplied per frame by [`super::Ribbon::render`]. This
//! struct holds only what the ribbon itself decided — which tab the
//! operator clicked, which mode they slid to — because those are the only
//! two facts that would be lost if it did not.
//!
//! That is what makes the ribbon re-renderable from nothing: an
//! application can drop this value, build a fresh one, and get a coherent
//! ribbon on the next frame (the first visible tab, the first mode). It
//! also means there is exactly one place a customization layer or a
//! session restore has to write to.
//!
//! # Why it is not `serde`-derived
//!
//! Layout persistence arrives with `layout` at S3 and will want to persist
//! the ribbon's state alongside the dock's, in **one** document rather
//! than two. Deriving it here would create a second serialization format
//! that then has to be migrated when the first one arrives — and a format
//! that shipped is a format somebody has a file in.
//!
//! # ★ Why the base `egui::Id` lives here
//!
//! `egui` keeps focus, hover and popup state per widget id. Two ribbons in
//! one context — two document windows in one viewport — that shared ids
//! would share all three, and the symptom is that hovering a control in
//! one window highlights the corresponding control in the other: baffling
//! until it is understood and trivial afterwards.
//!
//! The salt therefore belongs to the *instance*, which is this struct, and
//! every id the ribbon derives comes from it through
//! [`super::ctx::Ctx::id`]. Deriving rather than letting `egui`
//! auto-generate is a second, separate requirement: an auto-generated id
//! shifts when a group moves into the overflow menu, so a control would
//! lose keyboard focus when the window is resized.

use super::FrameReport;

/// The ribbon's own presentation state: which tab is active, which mode is
/// selected, and what the last frame contained.
///
/// See this module's header for what is deliberately absent.
#[derive(Debug, Clone)]
pub struct RibbonState {
    pub(super) base_id: egui::Id,
    pub(super) active_tab: Option<String>,
    pub(super) mode: Option<String>,
    pub(super) last_frame: FrameReport,
}

impl Default for RibbonState {
    fn default() -> Self {
        Self::new()
    }
}

impl RibbonState {
    /// Fresh state: no tab chosen, no mode chosen.
    ///
    /// Both resolve on the first frame — the first visible tab and the
    /// first mode in the manifest — so an application does not have to
    /// know the manifest's contents to start.
    #[must_use]
    pub fn new() -> Self {
        Self {
            base_id: egui::Id::new("egui-shell-ribbon"),
            active_tab: None,
            mode: None,
            last_frame: FrameReport::default(),
        }
    }

    /// Use a different base `egui::Id`.
    ///
    /// Needed only when two ribbons are drawn in one `egui` context — two
    /// document windows in one viewport, say. See this module's header for
    /// the symptom without one.
    #[must_use]
    pub fn with_id_salt(mut self, salt: impl std::hash::Hash + std::fmt::Debug) -> Self {
        self.base_id = egui::Id::new("egui-shell-ribbon").with(salt);
        self
    }

    /// The tab whose band is drawn.
    #[must_use]
    pub fn active_tab(&self) -> Option<&str> {
        self.active_tab.as_deref()
    }

    /// Ask for a tab to be active. Takes effect on the next frame; an id
    /// that is not visible falls back to the first visible tab.
    ///
    /// A tab set here is also **pinned into the strip**: whatever the
    /// window width, [`super::plan::plan_tab_strip`] keeps the active tab
    /// out of the overflow menu. So this is a complete way to drive the
    /// ribbon from a keyboard binding or a restored session — the tab is
    /// guaranteed to be visible, not merely selected.
    pub fn set_active_tab(&mut self, tab_id: impl Into<String>) {
        self.active_tab = Some(tab_id.into());
    }

    /// The selected mode, or `None` before the first frame.
    #[must_use]
    pub fn mode(&self) -> Option<&str> {
        self.mode.as_deref()
    }

    /// Select a mode.
    ///
    /// This is what an application calls when the operator presses the
    /// `Ctrl+1` its manifest bound to `mode.read` — the shell reports the
    /// command's token, the application dispatches it, and dispatching it
    /// means calling this. The mode selector is a *second* way to reach
    /// the same state, not a separate one.
    pub fn set_mode(&mut self, mode_id: impl Into<String>) {
        self.mode = Some(mode_id.into());
    }

    /// What the last rendered frame contained.
    #[must_use]
    pub fn last_frame(&self) -> &FrameReport {
        &self.last_frame
    }

    /// The `egui::Id` of one mode-selector segment.
    ///
    /// Published so a harness can drive the selector by keyboard — focus a
    /// segment, send an arrow — rather than by synthesising a click at a
    /// guessed coordinate.
    #[must_use]
    pub fn mode_segment_id(&self, mode_id: &str) -> egui::Id {
        self.base_id.with("mode").with(mode_id)
    }
}
