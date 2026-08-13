//! FIXTURE — the catalog file of the CLEAN ui-strings fixture.
//!
//! This file is deliberately full of operator-visible prose. It exists to
//! prove the catalog exclusion works: if `check-ui-strings.sh` ever stopped
//! excluding `ui_text.rs`, the clean fixture would start failing and the
//! self-test's assertion A would say so.
//!
//! Not compiled by anything. It is input to a shell gate, not to rustc.

/// Label for the destructive command in the object context menu.
pub const DELETE_SELECTED: &str = "Delete selected object";

/// Title of the settings dialog's first section.
pub const SETTINGS_APPEARANCE: &str = "Appearance and theme";

/// Status-bar text shown while a page raster is being rebuilt.
pub const RENDERING_PAGE: &str = "Rendering page…";
