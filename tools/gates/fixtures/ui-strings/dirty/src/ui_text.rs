//! FIXTURE — the catalog file of the DIRTY ui-strings fixture.
//!
//! Excluded from the scan, exactly as in the clean fixture. Present so the two
//! fixtures differ in ONE file only, which is what makes the self-test's
//! diagnosis unambiguous when it fires.
//!
//! Not compiled by anything.

/// Label for the destructive command in the object context menu.
pub const DELETE_SELECTED: &str = "Delete selected object";

/// Title of the settings dialog's first section.
pub const SETTINGS_APPEARANCE: &str = "Appearance and theme";
