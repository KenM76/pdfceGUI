//! Tests for [`super`] — restyling existing text.
//!
//! ## What these are for, and what they cannot be
//!
//! R1: *"the tests pass" is not a report of working software.* These prove the
//! **engine chain** — that a selection's run ordinals reach `format_text` and
//! that the document afterwards says what the operator asked for. They cannot
//! prove the panel, the combo box or the disclosure line, and the driven check
//! in `tools/ui-verify` is what does that.
//!
//! ★ Every assertion below is on the **document after the edit**, read back
//! through a fresh extraction — never on the return value of the thing under
//! test. A test that asserts "the function returned Ok" is a test of the
//! function's own opinion of itself.

#![cfg(test)]
// ★ The INNER attribute, load-bearing rather than redundant beside the
// `#[cfg(test)] mod tests;` that declares this file.
//
// `tools/gates/check-ui-strings.sh` and `check-theme-colors.sh` both recognise
// this exact line as "nothing in this file reaches the shipped binary". The
// property that earns the exemption is *not in the release build*, and a
// filename is a restatement of that which goes stale; the attribute is the
// fact itself. Without it, every assertion message below is reported as
// operator-facing copy — which is how a 28-hit report once trained people to
// ignore this gate.

use super::{StyleChange, apply};
use crate::app::state::{OpenDoc, ROTATED_TEXT, open_local_fixture};

/// The `Tf` size and `/BaseFont` in force on `run` of page 0, read fresh.
///
/// Goes through `pin::inspect` — the same road the panel reads by — rather than
/// through a private path, so a break in that road fails these too.
fn style_of(doc: &OpenDoc, run: usize) -> (f32, Option<String>) {
    let read = crate::canvas::textedit::pin::inspect(doc, 0, run)
        .expect("the fixture's first run carries provenance");
    (read.style.size, read.style.font_resource)
}

/// How many runs page 0 has, from the shared extraction.
fn run_count(doc: &OpenDoc) -> usize {
    doc.page_text().map_or(0, |t| t.runs.len())
}

/// ★★★ **The headline: a size change reaches the file.**
///
/// The operator's ask, reduced to its smallest true statement — press a number,
/// and the text on the page is that size afterwards.
///
/// Falsified by removing `StyleChange::stamp`'s `Size` arm: with nothing
/// stamped the request is empty, the engine returns `NoOp`, and the size read
/// back is unchanged, which fails here by name.
#[test]
fn a_size_change_reaches_the_document() {
    let mut doc = open_local_fixture(ROTATED_TEXT);
    let (before, _) = style_of(&doc, 0);
    assert!(before > 0.0, "the fixture's first run has a size to change");

    let target = f64::from(before) * 2.0;
    apply(&mut doc, 0, &[0], &StyleChange::Size(target));

    let (after, _) = style_of(&doc, 0);
    assert!(
        (f64::from(after) - target).abs() < 0.01,
        "the run's size should now be {target}, and it is {after}"
    );
}

/// **An edit through this module is undoable**, because it went through the
/// funnel rather than around it.
///
/// The property that is easiest to lose and hardest to notice: a verb called
/// directly on the session still edits the document and still saves, and only
/// `Ctrl+Z` tells you it was wrong.
#[test]
fn a_restyle_is_one_undoable_command() {
    let mut doc = open_local_fixture(ROTATED_TEXT);
    let (before, _) = style_of(&doc, 0);
    assert!(
        !doc.session.can_undo(),
        "a freshly opened document has no history"
    );

    apply(
        &mut doc,
        0,
        &[0],
        &StyleChange::Size(f64::from(before) * 2.0),
    );
    assert!(doc.session.can_undo(), "the restyle is in the undo log");

    let session = std::sync::Arc::get_mut(&mut doc.session).expect("sole owner in a test");
    session.undo().expect("undo the restyle");
    doc.edit_epoch = doc.edit_epoch.wrapping_add(1);

    let (after, _) = style_of(&doc, 0);
    assert!(
        (after - before).abs() < 0.01,
        "undo should put the size back to {before}, and it is {after}"
    );
}

/// **The epoch moves**, which is what makes every cached read — the panel's own
/// stamp among them — notice.
///
/// ★ Its own test rather than an assertion inside the one above, because it is
/// a different failure: an edit that lands in the file and does not bump the
/// epoch shows the operator the *old* size in the panel for ever after, and the
/// page they are looking at is right while the numbers beside it are wrong.
#[test]
fn a_restyle_bumps_the_edit_epoch() {
    let mut doc = open_local_fixture(ROTATED_TEXT);
    let before = doc.edit_epoch;
    let (size, _) = style_of(&doc, 0);
    apply(&mut doc, 0, &[0], &StyleChange::Size(f64::from(size) + 3.0));
    assert_ne!(doc.edit_epoch, before, "the edit epoch must move");
}

/// ★★ **Every run of a multi-run selection is restyled, not just the first.**
///
/// The case the descending-order argument exists for. It is asserted on the
/// *last* run as well as the first, because an implementation that restyled
/// only the head of the list would pass a test that looked at the head.
#[test]
fn a_multi_run_selection_restyles_every_run() {
    let mut doc = open_local_fixture(ROTATED_TEXT);
    let count = run_count(&doc);
    assert!(
        count >= 2,
        "the fixture must have at least two runs; it has {count}"
    );

    // Only the runs that actually carry provenance — a derived-whitespace run
    // has no show operator to pin and is correctly skipped by the engine.
    let pinnable: Vec<usize> = (0..count)
        .filter(|r| crate::canvas::textedit::pin::inspect(&doc, 0, *r).is_some())
        .take(3)
        .collect();
    assert!(
        pinnable.len() >= 2,
        "at least two runs must be pinnable; {} were",
        pinnable.len()
    );

    let before: Vec<f32> = pinnable.iter().map(|r| style_of(&doc, *r).0).collect();
    apply(&mut doc, 0, &pinnable, &StyleChange::Size(31.0));

    for (run, was) in pinnable.iter().zip(before) {
        let (now, _) = style_of(&doc, *run);
        assert!(
            (now - 31.0).abs() < 0.01,
            "run {run} was {was} and should now be 31, but it is {now}"
        );
    }
}

/// **A selection covering no runs changes nothing and says so.**
///
/// The guard that stops an empty gesture reaching the engine at all. Asserted
/// on the epoch rather than on the decline, because "nothing happened to the
/// document" is the claim that matters and the sentence is `text::status`'s
/// business.
#[test]
fn an_empty_selection_edits_nothing() {
    let mut doc = open_local_fixture(ROTATED_TEXT);
    let before = doc.edit_epoch;
    apply(&mut doc, 0, &[], &StyleChange::Size(31.0));
    assert_eq!(doc.edit_epoch, before, "an empty run list must not edit");
}

/// ★★★ **A run ordinal that does not exist is declined, not guessed at.**
///
/// The failure this guards is the expensive one: an out-of-range ordinal that
/// fell back to "the first run whose text matches" would restyle a piece of
/// text the operator never selected, in a file they then send to somebody.
#[test]
fn an_out_of_range_run_edits_nothing() {
    let mut doc = open_local_fixture(ROTATED_TEXT);
    let before = doc.edit_epoch;
    apply(&mut doc, 0, &[9_999], &StyleChange::Size(31.0));
    assert_eq!(
        doc.edit_epoch, before,
        "an unpinnable run must decline rather than edit something else"
    );
}

/// **Bold reaches the file on a page with no bold face**, which is the case the
/// engine's two-verb complement exists for.
///
/// ★ Asserted by the *absence of a refusal* and the presence of an edit rather
/// than by reading a "synthetic" flag out of the file: `R90`'s synthesis is
/// deliberately not recorded in the PDF — it is re-detectable from the bytes,
/// which is a different question from the one this test asks.
#[test]
fn bold_applies_on_a_page_with_no_bold_face() {
    let mut doc = open_local_fixture(ROTATED_TEXT);
    let before = doc.edit_epoch;
    apply(
        &mut doc,
        0,
        &[0],
        &StyleChange::Weight {
            bold: true,
            italic: false,
        },
    );
    assert_ne!(
        doc.edit_epoch, before,
        "bold must apply on every page; if this fails the two-verb complement is broken or the page grew a real Bold"
    );
}

/// ★★★ **The two-verb retry — and the engine defect it found.**
///
/// `textedit/format_family.pdf` is a `/Times-Roman` run `hello world` on a page
/// that also carries `/F2` (`Calibri-Bold`, fully covering) and `/F3`
/// (`Times-Bold`, whose `/Differences` remaps `o` to `/bullet`, so it does NOT
/// cover the run).
///
/// Asking for synthetic bold is refused by `gate_synthesis` — *"a REAL bold
/// face is available"* — **naming `Times-Bold`**, presumably because it matches
/// the run's family. This module takes that offer, and the offer is refused for
/// coverage. So **bold is unreachable on that page** through either verb, which
/// contradicts the engine's own "between the two verbs every page is covered",
/// while `format-text --set-font F2` succeeds from the command line throughout.
///
/// Filed with the engine. What is asserted here is the half that is this
/// shell's to get right:
///
/// 1. **nothing is half-applied** — the run is left exactly as it was;
/// 2. **the operator is told the actionable thing.** The refusal reported is
///    the RETRY's (*"that face has no shape for one of these characters"*), not
///    the synthesis refusal that sent us there — *"there is a real bold face,
///    use it"* is useless advice when using it is what just failed.
///
/// ★ Written as a **characterisation** test with the engine revision named,
/// because half of what it pins is a defect and not an intention. When the
/// engine picks a covering face, this test starts failing on assertion 1 — and
/// that failure is the good news, not a regression. `pdfce-core` at `914389c`,
/// 2026-08-27.
#[test]
fn bold_when_the_named_real_face_cannot_cover_the_run() {
    use crate::app::state::open_fixture;
    let mut doc = open_fixture("textedit/format_family.pdf");
    let (size_before, face_before) = style_of(&doc, 0);
    let epoch_before = doc.edit_epoch;

    apply(
        &mut doc,
        0,
        &[0],
        &StyleChange::Weight {
            bold: true,
            italic: false,
        },
    );

    let (size_after, face_after) = style_of(&doc, 0);
    assert_eq!(
        (size_before, &face_before),
        (size_after, &face_after),
        "nothing may be half-applied when both routes to bold refuse"
    );
    assert_eq!(
        doc.edit_epoch, epoch_before,
        "a refusal must not bump the epoch — the page did not change"
    );
    assert_eq!(
        crate::app::status::decline::recorded_for_test(),
        Some(crate::app::status::decline::Declined::TextStyle(
            crate::text::status::TextStyleRefusal::FaceLacksCharacters
        )),
        "the operator must be told the ACTIONABLE refusal — the retry's coverage failure, not the synthesis refusal that named a face which then would not take"
    );
}
