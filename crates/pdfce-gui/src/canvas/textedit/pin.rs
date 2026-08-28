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

/// Which content buffer a glyph's span indexes — the `EditTarget` half of a pin.
///
/// Its own function since 2026-08-27, when a second caller appeared
/// ([`operators_in_run`]). The sixty lines of argument below are the reason it
/// is a function rather than two copies: a paraphrase of them beside the
/// restyle verb would compile, would look correct, and would drift.
fn target_of(p: &pdfce_core::text_extract::GlyphProvenance) -> pdfce_core::text_edit::EditTarget {
    match p.content_stream {
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
    }
}

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
    let target = target_of(p);
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
    ///
    /// ★★★ **It is NOT the show operator's decoded text**, and assuming it was
    /// cost this project a driven run. See [`Self::find`].
    pub text: String,
    /// ★★★ **The longest stretch of [`Self::text`] that is actually IN the
    /// file** — the `find` a pinned `format_text` request must carry.
    ///
    /// # Why these are two fields and not one
    ///
    /// A `TextRun`'s `text` can differ from the operator's decoded buffer, and
    /// handing it to `format_text` as its `find` then fails with *"text to
    /// format … was not found in an editable run on the page"* — which is
    /// exactly what a driven Bold press produced on the operator's own drawing:
    /// eleven runs restyled and the twelfth refused, on a page where nothing
    /// was wrong. **The symptom is real and reproducible. The cause written
    /// here was not.**
    ///
    /// # ★★★ RETRACTED 2026-08-27 evening — the mechanism below is refuted
    ///
    /// This paragraph read:
    ///
    /// > ~~The extraction synthesises a space wherever a `TJ` offset inside one
    /// > show operator exceeds the word-gap threshold, so a title-block cell
    /// > reads `"FINISH         "` in `text` while the operator's decoded
    /// > buffer holds `"FINISH"` and a run of kerning numbers.~~
    ///
    /// `pdfce-core` **measured it** across 256 fixture PDFs, at this project's
    /// prompting, and answered:
    ///
    /// | | |
    /// |---|---|
    /// | `derived_word_space` runs (always a **separate** run) | 5 |
    /// | glyph runs containing a synthesised space | **0** ← the stated cause |
    /// | glyph runs where `len(text) != len(glyphs)` | 1 |
    ///
    /// `layout`'s `Break::Word` arm calls `close_run()` **and then** emits the
    /// derived space as its own one-character `TextRun` with no glyphs, so a
    /// `TextOrigin::Glyphs` run's `text` holds only real glyph characters. The
    /// one offender is `/ToUnicode` mapping one glyph to several characters
    /// (§9.10.3) — an `ffl` ligature is one glyph and three chars — which is a
    /// different mechanism with a different consequence.
    ///
    /// ★★ **So this field works and its reason is void**, and that is written
    /// here rather than repaired with a second guess. Three things could be
    /// true and this project cannot yet tell which: the walk is a no-op on
    /// every run it has ever seen and something else in the same commit fixed
    /// the refusal; it is doing real work for a reason nobody has stated; or
    /// the concatenation `pdfce-core` suspects is real and is ours, upstream of
    /// here. Filed as
    /// `reply_my_mechanism_was_wrong_and_here_is_the_measurement_for_your_open_question.md`,
    /// together with the request that settles it — whether the glyphs sharing
    /// one `operator_span` always slice a contiguous range out of the run's
    /// text. If they do, this walk is deleted and the span is sliced directly.
    ///
    /// ★ The lesson, and it is the mirror of the one this channel spent the
    /// week on: an **absence** claim is *"I looked and did not see it, so it is
    /// not there"*; this was a **cause** claim — *"I saw an effect and named a
    /// mechanism"* — written into a doc comment, a handover and a resume file
    /// without one measurement behind it. Same discipline, opposite sign.
    ///
    /// # How it is computed, and what it costs
    ///
    /// Walk the glyphs; keep the longest stretch whose byte ranges are
    /// **contiguous** — each glyph starting exactly where the last one ended.
    /// A character no glyph covers ends a stretch by construction, and every
    /// byte of the answer is a byte a glyph put there.
    ///
    /// ★ Note what this does **not** catch, now that the mechanism above is
    /// retracted: a ligature's glyph byte-range *is* contiguous, so a run whose
    /// `/ToUnicode` maps one glyph to three characters passes this walk whole
    /// and would still hand `format_text` three characters against a buffer
    /// holding one code. If that case is reachable, this field is not the fix
    /// for it. Unmeasured on our side and stated rather than assumed either
    /// way.
    ///
    /// ★ The cost is honest and is disclosed by the caller: on a run with an
    /// internal derived gap this is **shorter than the run**, so a restyle acts
    /// on the longest real piece rather than on all of it. `format_text`
    /// requires a non-empty `find` even when the operator is already pinned —
    /// the pin names the operator and `find` names a sub-range within it — so
    /// there is no way today to say *"the whole pinned operator"*. That is
    /// filed as an engine request; it is a small ask (`find: ""` meaning all of
    /// it, when `pinned_span` is set) and it would delete this field.
    pub find: String,
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

/// Every show operator of `run` on `page`, in ONE extraction.
///
/// The form `crate::app::actions::textstyle` wants: a restyle acts on operators
/// and a selection names runs, and this is the hop between them. See
/// [`operators_in_run`] for why the two are not the same thing.
#[must_use]
pub fn operators(doc: &OpenDoc, page: usize, run: usize) -> Vec<Operator> {
    let Some(page_ref) = doc.pages.get(page) else {
        return Vec::new();
    };
    use crate::app::settings::SettingsExt;
    let opts = doc.settings.extract_options().with_provenance(true);
    let Ok(text) =
        pdfce_core::text_extract::extract_page_view(&doc.session.view(), page_ref, page, &opts)
    else {
        return Vec::new();
    };
    let model = EditableTextModel::recognize(&text, &BlockRecognitionOptions::default());
    operators_in_run(&model, &text, run)
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
            find: text.runs.get(run).map(longest_sourced).unwrap_or_default(),
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

/// The longest run of characters in `run.text` that every byte of came from a
/// glyph — see [`RunStyle::find`] for why this is not simply `run.text`.
///
/// # The rule, in one line
///
/// A glyph publishes `text_start` and `text_len` into its run's `text`. A
/// stretch is *sourced* while each glyph begins exactly where the previous one
/// ended; the first gap is a derived character, and derived characters are not
/// in the file.
///
/// # ★ Longest rather than first
///
/// A run reading `"A       LONGER PHRASE"` has its real content after the gap,
/// not before it. Taking the first stretch would restyle the `A` and leave the
/// phrase, which is both wrong and hard to notice. Ties keep the earlier one,
/// which is arbitrary and stated so nobody has to wonder.
fn longest_sourced(run: &pdfce_core::text_extract::TextRun) -> String {
    let mut best: (usize, usize) = (0, 0);
    let mut start: Option<usize> = None;
    let mut end = 0_usize;
    for glyph in &run.glyphs {
        let (gs, ge) = (
            glyph.text_start as usize,
            glyph.text_start as usize + glyph.text_len as usize,
        );
        match start {
            Some(_) if gs == end => end = ge,
            _ => {
                if let Some(s) = start
                    && end - s > best.1 - best.0
                {
                    best = (s, end);
                }
                start = Some(gs);
                end = ge;
            }
        }
    }
    if let Some(s) = start
        && end - s > best.1 - best.0
    {
        best = (s, end);
    }
    run.text.get(best.0..best.1).unwrap_or_default().to_owned()
}

/// ★★★ **Every show operator a run is made of**, in content order, each with
/// the `find` text that names all of it.
///
/// # Why a run is not an operator, which is the thing this function exists to say
///
/// It is tempting — and this shell did it for one afternoon — to treat a
/// `TextRun` as a show operator: pin the first glyph's operator, pass the run's
/// text as `find`, and restyle. It works on most runs and fails on real
/// drawings, because `layout` closes a run on *geometry* and a producer closes a
/// show operator on *whatever its writer felt like*. A title-block cell reading
/// `FINISH` came back as one run spanning several `Tj`s, so the pin named the
/// first and the `find` named all of them, and `format_text` refused with *"text
/// to format ("FINISH ") was not found in an editable run on the page"* — on a
/// page where the very same string is found instantly by an UNpinned search.
///
/// That refusal is correct and is not a bug: `find` selects a contiguous code
/// range **within one string element**, and the shell was asking for a range
/// that spans several.
///
/// ⇒ **The operator is the unit of a restyle**, so the operator is what this
/// answers with. A run of three `Tj`s is three entries, three `format_text`
/// calls and three undo entries, and every one of them restyles exactly what it
/// names.
///
/// # The `find` per entry
///
/// The glyphs that share that operator's span, sliced out of the run's text by
/// their own `text_start`/`text_len`. Every byte comes from a glyph, so no
/// **derived** character — a space the extraction synthesised from a `TJ` offset
/// — can get in, which is the second way the naive version failed.
///
/// # Order
///
/// Content order, ascending. A caller wanting the descending order that keeps
/// byte offsets stable across edits reverses it, and
/// `crate::app::actions::textstyle` does, with the argument.
#[must_use]
pub fn operators_in_run(
    model: &EditableTextModel<'_>,
    page_text: &pdfce_core::text_extract::PageText,
    run: usize,
) -> Vec<Operator> {
    let mut out: Vec<Operator> = Vec::new();
    let Some(text) = page_text.runs.get(run) else {
        return out;
    };
    for (index, glyph) in text.glyphs.iter().enumerate() {
        let Some(p) = model.provenance(GlyphRef::new(run, index)) else {
            continue;
        };
        let (gs, ge) = (
            glyph.text_start as usize,
            glyph.text_start as usize + glyph.text_len as usize,
        );
        match out.last_mut() {
            // Same operator as the glyph before: extend its find text, but only
            // over bytes a glyph actually covers. A gap here is a derived
            // character and must not join the two halves.
            Some(last) if last.pin.span == p.operator_span => {
                if last.end == gs {
                    last.end = ge;
                    last.find
                        .push_str(text.text.get(gs..ge).unwrap_or_default());
                }
            }
            _ => out.push(Operator {
                pin: Pinned {
                    span: p.operator_span,
                    target: target_of(p),
                    text_matrix: p.text_matrix,
                    ctm: p.ctm,
                },
                find: text.text.get(gs..ge).unwrap_or_default().to_owned(),
                end: ge,
            }),
        }
    }
    out.retain(|o| !o.find.is_empty());
    out
}

/// One show operator inside a run: how to name it, and what to say it is.
#[derive(Debug, Clone, PartialEq)]
pub struct Operator {
    /// The locator.
    pub pin: Pinned,
    /// The text of the glyphs that share this operator — the `find` a pinned
    /// `format_text` must carry.
    pub find: String,
    /// Where the find text ends in the run's `text`, for extending it.
    end: usize,
}
