//! # `engine_overlay_skew` — the shell's model of a page and the engine's
//! disagree the moment content is added, and this test says by how much
//!
//! ## Why this test exists, and why it lives in the SHELL's test tree
//!
//! `OPERATOR_REQUESTS.md` row **O64**, in the operator's words:
//!
//! > *"When I add a new image to a pdf I can't edit it unless I save the
//! > document first … I assume this probably affects more than just images."*
//!
//! He is right, and the cause is not in this crate. `EditSession` keeps two
//! page-tree readers side by side:
//!
//! | reader | what it sees | who uses it |
//! |---|---|---|
//! | `EditSession::pages()` = `page_tree::pages_in(&self.graph())` | the **overlay** — the document as this session has edited it | every *authoring* verb, and this shell |
//! | `page_tree::pages(&self.base)` | the **base** — the document as it was on disk | every *content-editing* verb, and `EditSession::page_objects` |
//!
//! `add_image` appends a **new content stream** and a new `/XObject`
//! resource, and both writes land in the session overlay. So after an insert
//! the shell's decomposition (taken from `session.view()`, overlay-aware) has
//! N+1 objects while the engine's own model — the one every geometry verb
//! resolves an index against — still has N. The shell selects the new image
//! at index N and asks the engine to transform it; the engine answers
//! `ObjectOutOfRange`, the funnel traces `-refused`, and the operator sees a
//! gesture that does nothing. Saving flattens the overlay into a new base,
//! which is exactly why his workaround works.
//!
//! ## Why it is a test rather than a paragraph in a request
//!
//! Because `D:\Dev\pdfce` is READ-ONLY to this project, the fix is not mine
//! to make — it is a feature request. A request that asserts a defect in
//! somebody else's crate had better carry a reproduction, and this project's
//! standing rule is that **a backlog row is a record, not evidence**. Three
//! documents in this repository have previously stated an absence that was
//! false. So the claim in the request is this file, and this file is run by
//! `cargo test --workspace` on every commit.
//!
//! ## What each test asserts, and what it will do when the engine is fixed
//!
//! Both tests are written to **pass on the broken engine and fail on the
//! fixed one**, and each says so in its own assertion message. That is
//! deliberate and it is the only honest shape available: a test that asserted
//! the *correct* behaviour would be a red test in a green repository for as
//! long as the request is open, and would be muted within the week. When the
//! engine lands the fix these two go red, and the message tells the reader to
//! invert them and close row O64.
//!
//! ## Scope — the operator's generalisation, stated as code
//!
//! `add_image` is the sample, not the specification. The defect belongs to
//! **every path that adds page content as a new content stream**, which the
//! engine's own source says is exactly four: `add_image`, `add_text`,
//! `paste_objects` and `flatten_fields`. Annotations are **not** affected:
//! they are addressed by `ObjId` through the overlay-aware `self.value()`,
//! which is why a markup, a ce dimension, a redaction mark and a form field
//! are all editable the instant they are authored. Only the first test here
//! is cheap enough to write without a fixture factory; the rest are named in
//! the request.

use pdfce_core::edit::EditSession;

/// A one-page document with a little content, built from a fixture that
/// already ships with this repository.
///
/// `a1-titleblock.pdf` is used rather than a synthetic blank because a page
/// with **zero** existing objects would make the skew assertion trivially
/// true (0 vs 1 proves nothing about indexing) and because a real page is
/// what the operator has.
fn fixture() -> pdfce_core::document::Document {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/a1-titleblock.pdf"
    );
    pdfce_core::document::Document::load(std::path::Path::new(path))
        .expect("fixture a1-titleblock.pdf must load")
}

/// A 2x2 opaque PNG, encoded inline so this test needs no image fixture.
///
/// The bytes are a minimal valid PNG: signature, IHDR, a single IDAT holding
/// two zlib-stored scanlines, and IEND. Written out rather than generated so
/// the test has no dependency on an encoder crate.
fn tiny_png() -> Vec<u8> {
    // Built at test time with the `image` crate if available would be
    // simpler, but this crate does not depend on it for tests. A 1x1 grey
    // PNG, byte for byte.
    vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // signature
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR length + type
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1 x 1
        0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53, 0xDE, // bitdepth 8, RGB
        0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, // IDAT length + type
        0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00, 0x00, 0x03, 0x01, 0x01, 0x00, 0x18, 0xDD, 0x8D,
        0xB0, // IDAT crc
        0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82, // IEND
    ]
}

/// ★★★ **The skew, stated with no gesture at all.**
///
/// After `add_image` the two models of page 0 must describe the same page.
/// They do not: the shell's decomposition (which is what the canvas hit-tests
/// and what the Objects panel lists) gains the image, and
/// `EditSession::page_objects` — the engine's own model, and the one every
/// geometry verb resolves an index against — does not.
///
/// No pointer, no raster, no race. The inequality **is** the defect.
#[test]
fn the_engine_cannot_see_content_this_session_added() {
    let doc = fixture();
    let mut session = EditSession::new(doc);

    let before = session
        .page_objects(0)
        .expect("page 0 decomposes before the edit")
        .objects
        .len();

    let png = tiny_png();
    let image = pdfce_core::image_import::import(&png).expect("a 1x1 PNG must import");
    let rect = pdfce_core::page_tree::Rect {
        llx: 100.0,
        lly: 100.0,
        urx: 200.0,
        ury: 200.0,
    };
    let spec = pdfce_core::edit::NewImage::new(0, rect, &image);
    session.add_image(&spec).expect("add_image must succeed");

    // The shell's view: overlay-aware, and this is exactly the call
    // `crate::app::cache::ensure_page_objects` makes.
    let shell_count = {
        let overlay_pages = session.pages().expect("the overlay page tree walks");
        let page = overlay_pages.first().expect("page 0 exists in the overlay");
        let view = session.view();
        pdfce_core::vector::decompose_page(
            &view,
            page,
            pdfce_core::vector::Matrix::new(1.0, 0.0, 0.0, 1.0, 0.0, 0.0),
        )
        .expect("the overlay page decomposes")
        .objects
        .len()
    };

    // The engine's view: base-derived, and this is what every geometry verb
    // resolves `object_index` against.
    let engine_count = session
        .page_objects(0)
        .expect("page 0 still decomposes after the edit")
        .objects
        .len();

    assert_eq!(
        shell_count,
        before + 1,
        "the SHELL must see the image it just inserted — if this fails the \
         defect has moved and O64's diagnosis is wrong"
    );

    assert_ne!(
        shell_count, engine_count,
        "★ THE ENGINE WAS FIXED. `EditSession::page_objects` now agrees with \
         the overlay decomposition, which means the content-editing verbs can \
         address content added this session. INVERT THIS ASSERTION to \
         `assert_eq!`, delete the one below it, and close OPERATOR_REQUESTS.md \
         row O64."
    );

    assert_eq!(
        engine_count, before,
        "the engine's model is the page as it was on disk: {engine_count} \
         objects where the shell has {shell_count}. Every geometry verb \
         resolves an index against this model, so index {before} — the image \
         the shell just selected — is out of range."
    );
}

/// ★★ **And the consequence, driven through the verb the operator's drag
/// actually calls.**
///
/// Moving a placed image goes through `MoveSubject::Transform` →
/// `VectorAction::TransformObjects` → `EditSession::transform_objects`. This
/// asserts that call refuses, and names the error, so the request can quote
/// it rather than describe it.
#[test]
fn transforming_a_just_inserted_image_is_refused() {
    let doc = fixture();
    let mut session = EditSession::new(doc);

    let before = session
        .page_objects(0)
        .expect("page 0 decomposes before the edit")
        .objects
        .len();

    let png = tiny_png();
    let image = pdfce_core::image_import::import(&png).expect("a 1x1 PNG must import");
    let rect = pdfce_core::page_tree::Rect {
        llx: 100.0,
        lly: 100.0,
        urx: 200.0,
        ury: 200.0,
    };
    let spec = pdfce_core::edit::NewImage::new(0, rect, &image);
    session.add_image(&spec).expect("add_image must succeed");

    // `before` is the paint-order index the shell computes for the new image:
    // it selects `objects.len() - 1` of its own (N+1)-object model.
    let outcome = session.transform_objects(
        0,
        &[before],
        pdfce_core::vector::Matrix::translate(10.0, 0.0),
        pdfce_core::vector::TransformOptions::default(),
    );

    assert!(
        outcome.is_err(),
        "★ THE ENGINE WAS FIXED — a just-inserted image can now be \
         transformed. Invert this assertion and close OPERATOR_REQUESTS.md \
         row O64. Outcome was: {outcome:?}"
    );

    // Name the refusal, so the feature request can quote it rather than
    // describe it, and so a DIFFERENT refusal arriving later is a test
    // failure rather than a silent change of subject.
    let err = outcome.unwrap_err();
    let text = format!("{err:?}");
    assert!(
        text.contains("ObjectOutOfRange"),
        "the refusal must still be an out-of-range INDEX — if the engine now \
         refuses for another reason the diagnosis in O64 needs re-reading. \
         Got: {text}"
    );
}

/// ★★★ **The half nobody had reported, and the one with teeth: after a page
/// is deleted, the engine's content verbs address a DIFFERENT SHEET.**
///
/// `delete_pages` commits into the overlay, so `EditSession::pages()` returns
/// three pages while `page_tree::pages(&self.base)` — which every geometry
/// verb and `EditSession::page_objects` read — still returns four. The shell
/// computes a page index against the overlay and hands it to a verb that
/// resolves it against the base.
///
/// Two consequences, and the second is why this is filed as urgent rather
/// than as a nuisance:
///
/// 1. **An index the document no longer has still resolves.** Asking for
///    page 3 of a three-page document must be `PageOutOfRange`. It is not.
/// 2. **Therefore an index the document DOES have resolves to the wrong
///    sheet.** Delete page 0, then move or delete an object on what the
///    operator sees as page 0, and the verb edits the page that used to be
///    page 0 — a different sheet — and returns `Ok`. Nothing refuses, nothing
///    discloses, and the wrong drawing is changed.
///
/// This test asserts (1), because it is the crisp one: a count mismatch needs
/// no fixture with distinguishable content and cannot be argued with. (2)
/// follows from it arithmetically and is stated in the request.
#[test]
fn after_a_page_is_deleted_the_engine_still_indexes_the_old_page_set() {
    let doc = fixture_four_pages();
    let mut session = EditSession::new(doc);

    let before = session.pages().expect("the page tree walks").len();
    assert_eq!(before, 4, "the fixture is a four-page document");

    session
        .delete_pages(&[0])
        .expect("deleting the first page must succeed");

    let overlay = session.pages().expect("the page tree walks").len();
    assert_eq!(overlay, 3, "the OVERLAY correctly has three pages left");

    // The engine's own content model, asked for a page the document no
    // longer has. It must refuse. On the current engine it does not, because
    // it is looking at the base document, which still has four.
    let out_of_range = session.page_objects(3);

    assert!(
        out_of_range.is_ok(),
        "★ THE ENGINE WAS FIXED — page 3 of a three-page document is now \
         correctly out of range, which means the content verbs resolve their \
         page against the overlay. INVERT this assertion (assert it is an \
         Err naming PageOutOfRange) and close the page-index half of \
         OPERATOR_REQUESTS.md row O64. Outcome was: {:?}",
        out_of_range.map(|o| o.objects.len())
    );
}

/// The four-page fixture, used only by the page-index test above.
fn fixture_four_pages() -> pdfce_core::document::Document {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/four-pages.pdf");
    pdfce_core::document::Document::load(std::path::Path::new(path))
        .expect("fixture four-pages.pdf must load")
}
