#![cfg(test)]
//! # `app::state::tests` — the document record's own assertions
//!
//! Split out of [`super`] on 2026-08-26, when form-field selection pushed that
//! file past R2's 1,500-line limit. The convention is already in this tree —
//! `app::actions::tests` is the same split for the same reason — and it is the
//! right one: a test module is a distinct subject from the type it tests, and
//! moving it changes nothing about what runs.
//!
//! `use super::*;` below is what keeps that true: every name these tests reach
//! for is still the one they reached for when they lived in that file.
//!
//! ★ The inner `#![cfg(test)]` at the top is **load-bearing beyond the
//! compiler**. `check-ui-strings.sh` recognises that exact attribute as "this
//! whole file is out of the shipped binary" and stops reporting its assertion
//! messages as operator copy — matched on the attribute rather than on the
//! filename, because the property that earns the exemption is *not in the
//! binary* and a filename is a restatement of that which goes stale. Without
//! it this split reports fourteen false positives, which is how a report gets
//! trained out of being read.

use super::*;

// =======================================================================
// The staleness keys that landed at S4
// =======================================================================

/// **★ Every input that changes the picture changes the render key.**
///
/// The acceptance criterion for the `RenderKey` completion, from the
/// shell's side rather than the worker's.
/// [`PdfceApp::settle_and_rasterize`] asks "is the texture still a
/// picture of what I am looking at?" by comparing this key, so an input
/// it does not carry is a control that ticks and redraws nothing.
#[test]
fn every_view_input_that_changes_the_picture_changes_the_render_key() {
    let mut doc = open_fixture(PAINTED_LAYERS);
    let base = doc.render_key(2.0);

    assert_ne!(base, doc.render_key(2.5), "the raster scale");

    doc.view.page_index = 1;
    assert_ne!(base, doc.render_key(2.0), "the page");
    doc.view.page_index = 0;

    doc.set_annotations_visible(false);
    assert_ne!(base, doc.render_key(2.0), "annotation visibility");
    doc.set_annotations_visible(true);
    assert_eq!(base, doc.render_key(2.0), "…and back again");

    doc.set_layer_visible(ObjId::new(5, 0), true);
    assert_ne!(base, doc.render_key(2.0), "the layer override");
}

/// **A layer or annotation change is DISCRETE, not debounced.**
///
/// A click has no gesture in flight, so waiting out the 150 ms zoom
/// settle would be latency buying nothing. Asserted through the key's own
/// categories — what `settle_and_rasterize` reads — so an input that
/// lands in the wrong one fails here rather than being noticed later as
/// sluggishness.
#[test]
fn a_layer_or_annotation_change_commits_at_once_rather_than_settling() {
    let mut doc = open_fixture(PAINTED_LAYERS);
    let before = doc.render_key(2.0);
    doc.set_layer_visible(ObjId::new(5, 0), true);
    let after = doc.render_key(2.0);
    assert_ne!(after.discrete_inputs(), before.discrete_inputs());
    assert_eq!(
        after.scale_bits(),
        before.scale_bits(),
        "a layer toggle must not look like a zoom, or it inherits the debounce"
    );

    doc.set_annotations_visible(false);
    let hidden = doc.render_key(2.0);
    assert_ne!(hidden.discrete_inputs(), after.discrete_inputs());
    assert_eq!(hidden.scale_bits(), after.scale_bits());
}

/// **★ "Obey the document" and "hide nothing" are different renders.**
///
/// Core API trap T-12.9: [`LayerVisibility`] REPLACES the document's
/// default configuration rather than merging with it, so `None` and
/// `Some(empty)` are not two spellings of one state. Collapsing them
/// reveals every layer the document turned off — on a drawing whose
/// "Confidential" watermark is an off-by-default layer, that is a
/// disclosure defect, not a cosmetic one.
#[test]
fn obeying_the_document_is_not_the_same_as_hiding_nothing() {
    let mut doc = open_fixture(PAINTED_LAYERS);
    assert!(
        doc.layer_visibility().is_none(),
        "a freshly opened document obeys its own configuration"
    );

    doc.set_hidden_layers(BTreeSet::new());
    let showing_all = doc.layer_visibility().expect("an override is in force");
    assert_eq!(showing_all.hidden_count(), 0);

    doc.reset_layers();
    assert!(
        doc.layer_visibility().is_none(),
        "reset must restore `None`, not an empty override"
    );
}

/// **The first toggle starts from the DOCUMENT's answer, not from
/// nothing.**
///
/// [`LayerVisibility`] wants the complete hidden set, so a control that
/// handed in only the group the operator touched would reveal every
/// other layer the document had turned off. The fixture declares four
/// groups, two of them off by default; turning a third off must leave
/// those two off.
#[test]
fn the_first_layer_toggle_seeds_from_the_documents_own_defaults() {
    let mut doc = open_fixture(PAINTED_LAYERS);
    let defaults = doc.hidden_layers();
    assert_eq!(
        defaults.len(),
        2,
        "this fixture must declare layers that are OFF by default, or the \
         seeding path is untested: {defaults:?}"
    );

    doc.set_layer_visible(ObjId::new(4, 0), false);
    let hidden = doc.hidden_layers();
    assert!(
        hidden.contains(&ObjId::new(4, 0)),
        "the operator's own change"
    );
    for id in &defaults {
        assert!(
            hidden.contains(id),
            "the document's own OFF set must survive the first toggle, or \
             hiding one layer reveals every hidden one: {hidden:?}"
        );
    }

    doc.set_layer_visible(ObjId::new(5, 0), true);
    let hidden = doc.hidden_layers();
    assert!(!hidden.contains(&ObjId::new(5, 0)));
    assert!(hidden.contains(&ObjId::new(6, 0)), "and only that one");
}

/// **Every change to the override moves the generation.**
///
/// The generation is the staleness key; the set is not. A mutator that
/// changed the set and forgot the counter would leave the texture
/// looking current — the inert-control defect with the override
/// *correct*, which is the most confusing possible version of it.
#[test]
fn every_layer_mutation_moves_the_generation() {
    let mut doc = open_fixture(PAINTED_LAYERS);
    assert_eq!(doc.layers.generation, 0);
    doc.set_layer_visible(ObjId::new(5, 0), true);
    assert_eq!(doc.layers.generation, 1);
    doc.set_hidden_layers(BTreeSet::new());
    assert_eq!(doc.layers.generation, 2);
    doc.reset_layers();
    assert_eq!(doc.layers.generation, 3);
}

/// **A view toggle is not an edit.**
///
/// Hiding annotations or a layer changes what is drawn and nothing that
/// is saved, so it must not bump `edit_epoch` — which would throw away
/// the decomposition and the font inventory for nothing, and would make
/// the diagnostic `objects n=` line re-trace as though the document had
/// changed.
#[test]
fn hiding_annotations_or_a_layer_is_not_an_edit() {
    let mut doc = open_fixture(PAINTED_LAYERS);
    let _ = doc.page_objects();
    let _ = doc.font_inventory();

    doc.set_annotations_visible(false);
    doc.set_layer_visible(ObjId::new(4, 0), false);

    assert_eq!(doc.edit_epoch, 0, "no content changed");
    assert_eq!(doc.page_objects.built_for.get(), Some((0, 0)));
    assert_eq!(doc.fonts.built_for.get(), Some(0));
}

// =======================================================================
// The selection move — what replaced `canvas::selection::DocumentToken`
// =======================================================================

/// **★ A selection cannot outlive the document it was made on.**
///
/// The `DocumentToken` deletion, asserted rather than argued — the same
/// shape as `a_documents_decomposition_cannot_outlive_the_document` in
/// [`crate::app::cache`], because it is the same deletion for the same
/// reason.
///
/// The old mechanism compared an `Arc` **address** every frame and cleared
/// on a mismatch; an address is not an identity, and a reused allocation
/// with a matching page count would have carried a stale selection into a
/// new file. Here the question cannot be asked: opening a document builds a
/// whole new `OpenDoc`, so its selection is `SelectionState::default()` by
/// construction.
///
/// Written as a replacement **in the same binding** — the sequence an
/// address reuse would have needed — so that reintroducing any kind of
/// document-identity key here is a test failure rather than a review
/// finding.
#[test]
fn a_selection_cannot_outlive_the_document_it_was_made_on() {
    use crate::canvas::selection::{ClickHit, SelectionLevel};
    use crate::canvas::target::TargetId;

    let mut doc = open_fixture(FOUR_PAGES);
    assert!(
        doc.selection.is_empty(),
        "a freshly opened document has nothing selected"
    );

    doc.selection.click(
        0,
        ClickHit {
            object: Some(TargetId(1)),
            ..ClickHit::default()
        },
        false,
        false,
    );
    assert_eq!(doc.selection.len(), 1);

    doc = open_fixture(PAINTED_LAYERS);
    assert!(
        doc.selection.is_empty(),
        "a new document starts with an empty selection, whatever address \
         its session landed on"
    );
    assert_eq!(
        doc.selection.level(),
        SelectionLevel::Object,
        "…and at the top rung, not inside an object of the previous file"
    );
}

// =======================================================================
// Opening a document is what forgets the panels' state
// =======================================================================
