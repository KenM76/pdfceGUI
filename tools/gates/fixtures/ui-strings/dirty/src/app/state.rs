//! FIXTURE — the NESTED module carrying the planted violation.
//!
//! `src/app/state.rs` is two levels down. The gate's predecessor scanned
//! `"$SRC_DIR"/*.rs`, which does not match this path, so it read `lib.rs` and
//! `ui_text.rs`, found nothing, and printed `ui-strings: clean` — the exact
//! fail-open documented in PROJECT_PLAN.md §4.1.
//!
//! The ported gate uses `find`, sees this file, and fails. That difference is
//! the whole content of `--self-test` assertion B.
//!
//! DO NOT "FIX" THE VIOLATION BELOW. It is the fixture.
//!
//! Not compiled by anything.

/// Returns the label drawn on the object context menu's destructive item.
pub fn delete_label() -> &'static str {
    // THE PLANTED VIOLATION: a bare, whitespace-bearing, operator-visible
    // literal, outside the catalog, with no exemption marker. Everything the
    // gate looks for, in a file a flat glob cannot see.
    "Delete selected object"
}

/// A widget id, exempted — present so the fixture proves the gate reports the
/// violation and NOT this line.
pub fn panel_id() -> &'static str {
    // ui-text-exempt: an egui id_salt, never rendered.
    "objects panel"
}
