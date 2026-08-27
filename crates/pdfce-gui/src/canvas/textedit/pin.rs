//! # `canvas::textedit::pin` — naming the exact show operator, and the exact
//! buffer it lives in
//!
//! ## What a pin is, and why nothing that edits text may go without one
//!
//! `pdfce-core`'s text verbs — [`EditSession::edit_text`] and
//! [`EditSession::format_text`] — locate their operand two ways. Given only a
//! search string they find the **first** show operator on the page whose
//! decoded text matches. Given a `pinned_span` they find the one whose byte
//! span in the decoded content buffer is exactly that.
//!
//! The difference is not an optimisation. On a title-block sheet with two runs
//! reading `REV A`, the unpinned form edits the wrong one, silently, with no
//! error anywhere. This operator's own benchmark drawing carries **3,007
//! single-character show operators** in one page stream, so "the first match is
//! the one the operator meant" is false on the documents this program exists
//! for.
//!
//! ## Why this is its own module rather than a private detail of the caret
//!
//! It was a private detail of the caret until 2026-08-27, inline in
//! [`super::plan`], and that was correct while exactly one thing edited text.
//! `format_text` is the second: restyling an existing run takes **the same
//! `pinned_span` and the same `EditTarget`** as replacing its text — the engine
//! shaped the two verbs that way deliberately, *"so a shell that has decided
//! which stream a caret is in does not have to translate that decision between
//! two verbs."*
//!
//! A second copy of that decision is the thing to avoid. The `EditTarget` arm
//! below is nine lines of code and sixty of argument, and the argument is what
//! makes it right; a paraphrase of it beside the restyle verb would compile,
//! would look correct, and would drift.
//!
//! ## ★★ The extraction here is NOT the shared page-text cache
//!
//! `crate::app::cache`'s extraction runs with `ExtractOptions::default()`, and
//! `capture_provenance` **defaults to off** — the engine's own words:
//! *"`None` unless the extraction set `ExtractOptions::capture_provenance`;
//! this keeps the default Pass 4 output byte-for-byte unchanged."*
//!
//! With it off, `provenance()` answers `None` for every glyph, and a caller
//! built on the shared cache would get **no pin at all** while every line of it
//! kept compiling. That is this project's canonical failure shape: a correct
//! decision function wired to a value that is always the same.
//!
//! Widening the shared cache is the other option and is worse. Every consumer
//! of `page_text()` — Find, both copy verbs, the text sweep — would then pay
//! for provenance on every page, and extraction is the expensive thing this
//! shell does (392 ms on the benchmark sheet). Paying it **once per edit**, in
//! [`resolve`], is the whole cost, and an edit is already an operation that
//! saves and re-rasters.
//!
//! ★ The run index is shared between the two extractions, which is safe and is
//! worth stating: `capture_provenance` populates a field and changes no
//! segmentation, so `runs[i]` names the same run under both options.

use pdfce_core::span::ByteSpan;
use pdfce_core::text_edit::{BlockRecognitionOptions, EditTarget, EditableTextModel, GlyphRef};
use pdfce_core::text_extract::TextColor;

use crate::app::state::OpenDoc;

/// Everything a pinned text verb needs in order to name one show operator.
///
/// The three fields travel together because they are **one measurement**. The
/// span alone is the defect this shell shipped first: it pinned the offset and
/// discarded the stream, and the engine correctly reported *"text not found"*
/// about text that was plainly there.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pinned {
    /// The show operator's byte span within its own decoded content buffer.
    pub span: ByteSpan,
    /// **Which buffer that span indexes.** Never `Auto` when the provenance
    /// was read; see the argument on [`resolve`].
    pub target: EditTarget,
    /// `Tm` in force at the run's first glyph — read by
    /// [`super::disposition::is_upright`].
    pub text_matrix: [f32; 6],
    /// The CTM in force at the run's first glyph.
    pub ctm: [f32; 6],
}

/// The pin for `run`, from a model **already recognised over a
/// provenance-carrying extraction**.
///
/// `None` when the extraction did not capture provenance, or when the run has
/// no glyphs. Both mean the same thing to a caller — *this run cannot be
/// pinned* — and both must be treated as a refusal rather than as permission to
/// fall back to an unpinned request, for the reason the module header gives.
#[must_use]
pub fn of_run(model: &EditableTextModel<'_>, run: usize) -> Option<Pinned> {
    let p = model.provenance(GlyphRef::new(run, 0))?;
    // ★★★ NAME THE BUFFER THE PIN INDEXES. `Pass 119.0`, and this
    // is the line that makes form editing SAFE rather than merely
    // possible.
    //
    // `EditTarget::Auto` is the engine's default and is right for a
    // caller that has only a search string: it tries the page's own
    // `/Contents` first, then each form in `Do` order, and edits the
    // first stream that matches.
    //
    // **It is the wrong default for a PINNED request.** A pin is a
    // byte span into ONE decoded buffer, and `GlyphProvenance`
    // carries the name of that buffer beside it — the two fields are
    // one fact, and reading half of it is the defect this shell
    // shipped in the first place (the span was pinned, the stream
    // was discarded, and the engine reported "text not found" about
    // text that was plainly there).
    //
    // Under `Auto`, a span that indexes a form's bytes is offered to
    // the page's stream first. On this operator's own benchmark
    // sheet that stream holds **3,007 single-character show
    // operators**, so "an arbitrary offset happens to name a
    // matching operator in the wrong buffer" is not a theoretical
    // collision — it is a dense field of near-misses, and the result
    // would be an edit that succeeded on the wrong glyph with no
    // error anywhere.
    //
    // So: the shell knows exactly which stream it measured, and it
    // says so. `Form { object }` is an error if the page does not
    // paint that form, which is the answer we want — a loud refusal
    // beats a widened search when the caller had a measurement.
    let target = match p.content_stream {
        pdfce_core::text_extract::ContentStreamRef::Page => {
            pdfce_core::text_edit::EditTarget::PageContents
        }
        pdfce_core::text_extract::ContentStreamRef::Form { object } => {
            pdfce_core::text_edit::EditTarget::Form { object }
        }
        // ★ `ContentStreamRef` is `#[non_exhaustive]`, so a buffer
        // kind added later lands here. `Auto` is the right fallback
        // and not merely the compiling one: it is the engine's own
        // default, it searches everywhere including whatever the new
        // kind is, and it degrades to the pre-`119.0` behaviour
        // rather than to a refusal. A `PageContents` fallback would
        // silently narrow the search for a stream nobody here has
        // heard of, which is the worse direction.
        _ => pdfce_core::text_edit::EditTarget::Auto,
    };
    Some(Pinned {
        span: p.operator_span,
        target,
        text_matrix: p.text_matrix,
        ctm: p.ctm,
    })
}

/// The pin for `run` on `page`, extracting the page with provenance on.
///
/// The convenience form for a caller that does not already hold a model — the
/// restyle verbs, which start from a selection rather than from a caret. A
/// caller that has just recognised a model should use [`of_run`] and not pay
/// for a second extraction.
///
/// `None` when the page is absent, when the extraction fails, or when
/// [`of_run`] answers `None`.
#[must_use]
pub fn resolve(doc: &OpenDoc, page: usize, run: usize) -> Option<Pinned> {
    inspect(doc, page, run).map(|i| i.pin)
}

/// What a run currently **looks like** — the three facts a properties panel
/// shows and a restyle changes.
///
/// # ★ Why this is separate from [`Pinned`] and returned beside it
///
/// [`Pinned`] is a *locator*: it names an operand, and every field on it is
/// consumed by the engine. This is a *reading*: every field on it is consumed
/// by a human. Merging them would mean the restyle verb carrying three fields
/// it never looks at, and — the part that matters — would make it possible to
/// pass a stale reading into an edit by passing the struct that also carries
/// the pin.
///
/// They come back from one call because they come from one `GlyphProvenance`,
/// and the extraction that produces it costs 392 ms on this operator's
/// benchmark sheet. Two calls would be two extractions for one question.
#[derive(Debug, Clone, PartialEq)]
pub struct RunStyle {
    /// The `Tf` size in points.
    pub size: f32,
    /// The `/Resources /Font` **key** in force — `F1`, not `Helvetica`.
    ///
    /// ★ Not the `/BaseFont`, and the difference is why a caller showing this
    /// to an operator has to join it against the document's font inventory
    /// first. `GlyphProvenance` records what the content stream said, which is
    /// a resource key; the human-readable name lives in the font dictionary the
    /// key resolves to.
    pub font_resource: Option<String>,
    /// ★★ **The run's own characters** — and they are not decoration.
    ///
    /// `format_text` needs a non-empty `find` **even on a pinned request**, and
    /// that surprised this shell: the pin names the show OPERATOR, and `find`
    /// then names a contiguous sub-range *within* it (`match_run`, which
    /// refuses an empty one by name). Restyling a whole run therefore means
    /// passing the whole run's text.
    ///
    /// Published here rather than re-read by the caller because the caller
    /// would have to re-extract to get it, and this function has just paid for
    /// an extraction.
    ///
    /// ★ It stays valid across a restyle: `format_text` changes how characters
    /// look and never which characters they are, so a text captured before a
    /// multi-run gesture is still the right `find` for the runs still to come.
    pub text: String,
    /// The fill colour in force, in whatever space the file set it.
    ///
    /// `TextColor::Other` is a real and important answer: the run is painted in
    /// a space this Pass does not decode, and a caller that renders it as its
    /// nearest RGB — and then writes that RGB back — has converted the
    /// operator's ink without being asked.
    pub fill: Option<TextColor>,
}

/// Everything one provenance read yields: the operand, and what it looks like.
#[derive(Debug, Clone, PartialEq)]
pub struct Inspected {
    /// The locator, for a verb.
    pub pin: Pinned,
    /// The reading, for a panel.
    pub style: RunStyle,
}

/// The pin **and** the current style for `run` on `page`, in one extraction.
///
/// The form a properties panel wants. [`resolve`] is this with the reading
/// dropped, kept as its own entry point so an edit path cannot accidentally
/// hold a stale style struct alongside a fresh pin.
#[must_use]
pub fn inspect(doc: &OpenDoc, page: usize, run: usize) -> Option<Inspected> {
    let page_ref = doc.pages.get(page)?;
    use crate::app::settings::SettingsExt;
    let opts = doc.settings.extract_options().with_provenance(true);
    let text =
        pdfce_core::text_extract::extract_page_view(&doc.session.view(), page_ref, page, &opts)
            .ok()?;
    let model = EditableTextModel::recognize(&text, &BlockRecognitionOptions::default());
    let pin = of_run(&model, run)?;
    let p = model.provenance(GlyphRef::new(run, 0))?;
    Some(Inspected {
        pin,
        style: RunStyle {
            size: p.tf_size,
            text: text
                .runs
                .get(run)
                .map(|r| r.text.clone())
                .unwrap_or_default(),
            // Lossy rather than strict: a resource key is a PDF name, which is
            // bytes, and a name that is not UTF-8 is legal. Losing a byte in a
            // label is better than showing no label at all, and nothing acts on
            // this string — the edit uses the pin.
            font_resource: p
                .font_resource
                .as_ref()
                .map(|k| String::from_utf8_lossy(k).into_owned()),
            fill: p.fill_color,
        },
    })
}
