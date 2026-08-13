//! FIXTURE — crate root of the DIRTY ui-strings fixture.
//!
//! DELIBERATELY CLEAN. The whole point of this fixture is that its top level
//! is spotless and its violation is buried two directories down: a flat
//! `src/*.rs` glob reads this file, finds nothing, and reports success.
//!
//! Do not add a violation here. If one is ever added, `--self-test` assertion
//! C stops proving that assertion B required recursion, and the self-test
//! degrades into a smoke test without anybody noticing.
//!
//! Not compiled by anything.

pub mod app;
pub mod ui_text;

/// Every string that reaches the operator comes from the catalog.
pub fn window_title() -> &'static str {
    ui_text::SETTINGS_APPEARANCE
}
