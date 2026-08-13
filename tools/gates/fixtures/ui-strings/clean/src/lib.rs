//! FIXTURE — crate root of the CLEAN ui-strings fixture.
//!
//! Top-level file, reachable by BOTH a flat `src/*.rs` glob and a recursive
//! `find`. It is here so the fixture has a realistic shape, and so the dirty
//! fixture's top level can be clean while its nested module is not — which is
//! what makes the self-test's assertion C meaningful.
//!
//! Not compiled by anything.

pub mod app;
pub mod ui_text;

/// Every string that reaches the operator comes from the catalog.
pub fn window_title() -> &'static str {
    ui_text::SETTINGS_APPEARANCE
}
