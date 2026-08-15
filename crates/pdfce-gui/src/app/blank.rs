//! # `app::blank` — where a new document comes from, and why it is a file
//!
//! `file.new` (`RIBBON_IA.md` §5.1, the File ▸ File band) makes a blank
//! document. This module holds the 443 bytes it makes it *out of*, the
//! decisions behind them, and nothing else — the lifetime transition itself is
//! [`crate::app::PdfceApp::new_document`]'s, beside `open_path` and
//! `close_document`, because that is one subject and this is another.
//!
//! ## ★ 1. The engine cannot create a document, and that is deliberate
//!
//! `pdfce_core::document::Document` has exactly four constructors —
//! `load`, `load_with_password`, `from_bytes`, `from_bytes_with_password`
//! (`D:\Dev\pdfce\crates\pdfce-core\src\document.rs:360-404`) — and **every
//! one of them parses existing PDF bytes**. `EditSession` can rotate, delete
//! and reorder pages (`edit.rs:3848`, `:14739`, `:15039`) and has no verb that
//! *creates* one. `pageops::insert` and `pageops::merge` take pages only from
//! an already-loaded `DocumentView`. There is no path anywhere in the engine
//! that conjures a page from nothing.
//!
//! The obvious response is to ask pdfce for a `Document::blank(…)`. **Do not
//! file that request.** The engine's own module header states the reason as a
//! named, permanent invariant (`document.rs:10-19`):
//!
//! > `Document` is simultaneously the parse result AND […] the write source
//! > […]. **No separate builder/generation model may ever be introduced** —
//! > the audited prior art shows exactly how that bifurcation forecloses
//! > round-trip editing.
//!
//! A blank-document constructor is a generation model. Asking for one is
//! asking the engine to break the invariant that decides its architecture, and
//! it would have been refused — which is the fifth instance of `HANDOFF.md`
//! §11's rule that a claim gets verified against their source *before* it is
//! filed, and the first where the verification stopped a filing rather than
//! corrected one.
//!
//! The engine's own tests reach a blank document by writing minimal PDF bytes
//! and parsing them back (`edit.rs:18066`, `fn blank_page_doc`). That helper is
//! `#[cfg(test)]` and `pub(crate)`, with its own note saying why: *"a builder
//! that produces deliberately-minimal PDFs is a testing tool, not part of the
//! engine's API, and exposing it would invite production code to construct
//! documents outside the one object model."*
//!
//! ## ★ 2. So New opens a file, which is the thing this shell already does
//!
//! [`TEMPLATE`] is a real PDF, authored once, checked in, and compiled into the
//! binary. `file.new` hands it to `Document::from_bytes` and marks the result
//! as having no path. **The shell authors no PDF bytes in code**, the engine
//! gains no verb, and the whole of New's implementation is the open path with
//! the source swapped from a disk read to a slice.
//!
//! That matters beyond convenience. A hand-built byte string in a `const` is a
//! second PDF writer living inside the GUI, which is exactly what the engine's
//! invariant refuses on the other side of the boundary. A file is inspectable,
//! diffable, openable in Acrobat, and covered by
//! `tools/gates/check-shipped-assets.py`; a `const [u8]` is none of those.
//!
//! The asset ships under `assets/PROVENANCE.md` as **own work under MIT**,
//! which exempts it from that gate's notice surfaces (checks 4 and 5) and from
//! nothing else. Read that note before touching the bytes; the cross-reference
//! table stores absolute offsets and the file is not hand-editable.
//!
//! ## ★ 3. The page is A4, and Letter was rejected on the evidence
//!
//! Standing instruction 4 (`HANDOFF.md` §3): *match what Inkscape, Acrobat and
//! SolidWorks do — but first ask which of them actually has the surface.*
//!
//! **All three have this surface**, which is unusual, so the head-count is
//! worth having:
//!
//! | application | what its New does about size |
//! |---|---|
//! | **Acrobat** | creates the blank page immediately at a **locale default** — A4 under ISO locales, Letter under US ones. It does not ask. |
//! | **Inkscape** | `Ctrl+N` creates from the **default template**, which ships as **A4**. It does not ask. A size chooser exists and is a *different command*, `Ctrl+Alt+N`. |
//! | **SolidWorks** | `Ctrl+N` **does** ask — but what it asks is *which kind of document* (part / assembly / drawing). The sheet-size question comes second and only for drawings. |
//!
//! Two decisions fall out of that table and they are separate:
//!
//! **New does not ask.** Two of the three create immediately from a default;
//! the third asks a question — *what kind of document is this* — that pdfce has
//! no analogue for, because every pdfce document is the same kind. A dialog
//! offering one control would be SolidWorks' shape with SolidWorks' content
//! removed. Inkscape's split is the model followed: the plain verb makes a
//! document, and choosing a size is a **separate command** for later. See
//! `crate::shell::manifest::PLANNED`'s `file.new_from_template` row.
//!
//! **The default is A4.** Inkscape ships A4. Acrobat produces A4 on any metric
//! locale and Letter only on the US branch. And the operator's own documents —
//! the test set this project is measured against — are **A3 and A1 SolidWorks
//! drawing sheets**, which is A-series evidence, not Letter evidence: an
//! operator whose sheets are A3 and A1 has an A-series drawer, an A-series
//! plotter and A-series habits. Letter is reachable only through the US-locale
//! branch of one of the three, and this shell has no locale question to ask.
//!
//! So A4 wins on two of three plus the operator's own corpus, and it is
//! recorded here rather than assumed. What is *not* claimed is that A4 is the
//! right size for this operator's next new sheet — it very plausibly is not.
//! That is what the size picker is for, and it is a follow-up row rather than a
//! silent guess dressed up as a default.
//!
//! ## ★ 4. What a document with no file is, and what must not happen to it
//!
//! `crate::app::state::OpenDoc::path` means *where this came from*. A created
//! document came from nowhere, so its path is a **name** —
//! `crate::text::files::untitled` — and
//! [`crate::app::state::OpenDoc::stored_under`] is the one predicate that tells
//! the two apart. Three things consult it, and each would be a real defect
//! without it:
//!
//! - the **recent list** must not gain a row for a file that does not exist;
//! - the **remembered page display** must be neither read nor written under a
//!   fabricated path;
//! - the **guides store** likewise.
//!
//! Everything else in the shell treats `path` as an identity or a label — the
//! forms cache key, the Pages panel caption, the trace — and all of those are
//! correct for a name. See the field's own documentation.
//!
//! ## ★ 5. A new document CAN be saved — corrected 2026-08-14
//!
//! This section used to read *"A new document cannot be saved, because no
//! document can"*, and it is kept as a correction rather than deleted because
//! the analysis under it turned out to be exactly right and is what the fix was
//! built from. It said: both engine verbs take `&self` so an `Arc<EditSession>`
//! can call either, `crate::app::files::pick_save_path` already exists with its
//! diagnostic seam, **"save a copy is a shell task, not an engine gap"**, and
//! what remained was one decision — incremental preserves superseded content
//! and any existing signature, a full rewrite destroys the signature.
//!
//! `file.save_copy` was wired on 2026-08-14 and that decision was already made:
//! **incremental**, because the command's own shipped tooltip had promised it in
//! words on an operator-visible surface. `crate::app::save` §1 carries the
//! argument.
//!
//! What that means for New specifically: a created document saves like any
//! other, and the copy it writes is the 443-byte template plus an appended
//! revision carrying whatever the operator authored on it. Two things about it
//! are New's own and are argued at `crate::app::save::suggested_path`:
//!
//! * the suggested name is the document's own — `Untitled 1.pdf`, with **no**
//!   `-copy` suffix, because there is no original to avoid overwriting;
//! * saving does **not** give the document a file. `path` stays `Untitled
//!   1.pdf`, `origin` stays [`crate::app::state::Origin::Created`], no Recent
//!   row appears, and no per-document preference is stored — because this is
//!   Save a *copy*, not Save As. `OpenDoc::origin`'s own note that *"a created
//!   document that gains a file gains it through a save"* still stands, and
//!   still refers to a `file.save_as` this build does not have.
//!
//! What is still absent is in-place `file.save`, blocked on autosave and crash
//! recovery in `crate::shell::manifest::PLANNED`.

use pdfce_core::document::Document;
use pdfce_core::page_tree::Page;

/// **The blank document, as bytes.**
///
/// 443 bytes: one A4 page with an empty content stream, a classic
/// cross-reference table, and nothing else. `assets/PROVENANCE.md` documents
/// every object in it and why each is shaped the way it is.
///
/// `include_bytes!` rather than a read at start-up, for two reasons that both
/// bite in the field: a portable folder whose template file was deleted would
/// produce a New that fails on a machine nobody can see, and a template that
/// can be replaced on disk is a template whose bytes are not the bytes the
/// tests pinned.
pub const TEMPLATE: &[u8] = include_bytes!("assets/blank-a4.pdf");

/// The template page's width in PDF units. ISO 216 A4: 210 mm at 72/inch.
///
/// Public so the test below can assert the *asset* matches the *decision*
/// rather than merely matching itself. A constant compared against nothing is
/// documentation; compared against the parsed `MediaBox` it is a check.
pub const WIDTH_PT: f64 = 595.276;

/// The template page's height in PDF units. ISO 216 A4: 297 mm at 72/inch.
pub const HEIGHT_PT: f64 = 841.89;

/// **Parse [`TEMPLATE`] into a document and its page vector.**
///
/// The same two steps `PdfceApp::open_path` performs on a file, in the same
/// order, so a created document reaches `OpenDoc` through the identical
/// pipeline an opened one does. Nothing here is a shortcut around the engine.
///
/// # Errors
///
/// The engine's own message, ready to be shown by
/// `crate::text::open_failed`. **Unreachable in a correct build** — the bytes
/// are compiled in and [`tests::the_template_parses_and_holds_exactly_one_page`]
/// pins that they parse — but returned rather than unwrapped, because the
/// state it would describe is "this binary was built with a corrupt asset",
/// and an operator meeting that deserves a sentence rather than a stack trace.
pub fn document() -> Result<(Document, Vec<Page>), String> {
    let doc = Document::from_bytes(TEMPLATE.to_vec()).map_err(|err| err.to_string())?;
    let pages = pdfce_core::page_tree::pages(&doc).map_err(|err| err.to_string())?;
    Ok((doc, pages))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★ **The compiled-in template really is a document.**
    ///
    /// The one assertion that makes [`document`]'s error arm unreachable, and
    /// therefore the one that lets `file.new` be described as a command that
    /// cannot fail. Without it the claim would rest on the asset having been
    /// correct on the day it was written.
    #[test]
    fn the_template_parses_and_holds_exactly_one_page() {
        let (_doc, pages) = document().expect("the compiled-in template must parse");
        assert_eq!(pages.len(), 1, "New makes a one-page document");
    }

    /// ★ **The page is A4, to the tenth of a point.**
    ///
    /// This is the decision in §3 of the module header being *checked* rather
    /// than merely written down. A future edit that regenerated the asset at
    /// Letter — 612 × 792, which is what most minimal-PDF recipes on the
    /// internet carry, including the engine's own `blank_page_doc` fixture —
    /// fails here, naming both numbers, rather than shipping a silently
    /// different default.
    #[test]
    fn the_template_page_is_a4() {
        let (_doc, pages) = document().expect("the template parses");
        let media = pages[0].media_box;
        let width = media.urx - media.llx;
        let height = media.ury - media.lly;
        assert!(
            (width - WIDTH_PT).abs() < 0.1 && (height - HEIGHT_PT).abs() < 0.1,
            "the template is {width} x {height} pt; A4 is {WIDTH_PT} x {HEIGHT_PT}. \
             If this default was changed deliberately, change `app::blank`'s header \
             argument with it — the reasoning is what makes the number defensible."
        );
    }

    /// ★ **The page has a content stream, empty though it is.**
    ///
    /// A page with no `/Contents` is legal (§7.7.3.3) and would render
    /// identically — which is exactly why this needs an assertion rather than
    /// an eyeball. Every real producer emits a content stream, so a template
    /// without one would exercise a renderer path no other document in this
    /// project takes, and would prove less than it appears to on the day
    /// somebody uses New to reproduce a rendering defect.
    #[test]
    fn the_template_page_carries_a_content_stream() {
        let (_doc, pages) = document().expect("the template parses");
        assert_eq!(
            pages[0].contents.len(),
            1,
            "the blank page must carry exactly one (empty) content stream"
        );
    }

    /// The asset stays a template rather than becoming a document.
    ///
    /// Not a change-detector: the failure it guards against is somebody
    /// "improving" the template by embedding a font, a logo or a title block,
    /// which would make every new document carry bytes the operator did not
    /// ask for and would quietly move this directory out of the own-work
    /// provenance it is declared under. Two kilobytes is roughly four times
    /// the honest size and nowhere near a single embedded face.
    #[test]
    fn the_template_is_still_a_few_hundred_bytes() {
        assert!(
            TEMPLATE.len() < 2048,
            "the blank template is {} bytes; it was 443. Anything that big is \
             carrying content, and `assets/PROVENANCE.md` describes a file that \
             carries none.",
            TEMPLATE.len()
        );
    }
}
