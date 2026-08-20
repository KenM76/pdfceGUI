//! # `canvas::textsel` — selecting text on the page, and copying what was selected
//!
//! The gesture Acrobat Reader has and this shell did not. `FEATURES.md`
//! recorded the gap in the operator's own terms:
//!
//! > **Read selects no text.** Acrobat Reader lets you select and copy text,
//! > and Read mode should; this shell has no canvas text-selection gesture at
//! > all […] so *"only what Reader would allow"* is currently a strict subset
//! > of what was asked for.
//!
//! The ask it closes was given on 2026-08-14: *"in read mode the document
//! shouldn't allow editing and should allow only selecting of objects that
//! acrobat reader would allow."* `app::modes::capability` answered the first
//! half — Read refuses every content gesture — and in doing so made the second
//! half visible: **Reader allows text selection**, so a Read mode that refuses
//! it is not "only what Reader allows", it is less.
//!
//! It also unblocks three Phase 6 markup kinds. `FEATURES.md` lists underline,
//! strikeout and squiggly as *"engine-ready, but they mark text and there is no
//! text-selection gesture yet"*; `pdfce_core::annot_author::MarkupSpec::
//! TextMarkup` takes a `Vec<Quad>`, and [`TextSelection::quads`] is that vector
//! one projection away. Nothing here authors anything — see §6.
//!
//! ★ **Those three landed on 2026-08-14**, in [`crate::canvas::markup::text`],
//! and the sentence above turned out to be one word wrong: the vector is not a
//! projection *away*, it is a projection *back*. See §5.1 —
//! [`TextSelection::page_quads`] now travels beside the canvas boxes, produced
//! by the same pass over the same glyphs, because inverting the canvas
//! projection at the authoring site would have been the second derivation this
//! module exists to make unavailable.
//!
//! ---
//!
//! ## 1. ★ The interaction decisions, and which application each came from
//!
//! Standing instruction (`HANDOFF.md` §3.4, sharpened 2026-08-14): *"make your
//! best educated guesses to match what inkscape, acrobat, and SolidWorks do"*,
//! recording which one was followed and why, and — where they disagree —
//! saying which won. **Acrobat wins ties about *reading*, because Acrobat is
//! what pdfce replaces.**
//!
//! | Question | Acrobat | Inkscape | SolidWorks | Shipped | Why |
//! |---|---|---|---|---|---|
//! | drag selects a **range** or a **rectangle** | both (range default, `Alt` for rectangle) | range | range | **range** | unanimous on the default; the rectangle is deferred, §2 |
//! | **double**-click | word | word | word | **word** | unanimous |
//! | **triple**-click | paragraph | line | line | **line** | §1.1 — the disagreement is the interesting part |
//! | crosses **columns** | yes | n/a (one text object) | n/a (one note) | **yes** | falls out of content order, §4 |
//! | crosses **pages** | yes | n/a | n/a | **no** | §4 — and it is a cost decision, stated as one |
//! | visible **caret** | no (Select tool) | yes (text tool) | yes (note editing) | **no** | §1.2 |
//! | **Escape** | clears | deselects | leaves the field | **clears** | unanimous |
//! | **Ctrl+A** | all text | all objects | all in field | **all text on the page** | §1.3 |
//! | **Ctrl+C** | copies | copies | copies | **copies** | unanimous |
//! | Shift+click | extends from the anchor | extends | extends | **extends** | unanimous |
//!
//! ### 1.1 Triple-click: Acrobat says paragraph, and this ships a line
//!
//! The one row where the reference applications disagree and Acrobat **did not
//! win**, so it needs the argument.
//!
//! Acrobat selects a paragraph. Inkscape and SolidWorks select a line. A PDF
//! content stream contains neither: `pdfce-core`'s own extraction documentation
//! is blunt that lines are **derived** (its S5 sourcing note — *"no line or
//! paragraph markers exist in a content stream, in tagged or untagged files
//! alike"*), and paragraphs are derived a second time, from the lines, by
//! `EditableTextModel`'s block recognition using a leading-gap ratio and an
//! indent ratio.
//!
//! So the choice is between a unit this engine derives once and a unit it
//! derives twice. A triple-click that selected a *block* would be the
//! operator's most emphatic gesture resolved through the shakiest inference in
//! the stack, and when it got the paragraph wrong — on a drawing sheet's title
//! block, where "paragraph" means very little — there would be no smaller unit
//! to fall back to, because the double-click below it is a word. The line is
//! the honest middle rung, it is what two of the three reference applications
//! do, and [`EditableTextModel::line_range_at`] is a published verb for it
//! where a block range is not.
//!
//! ### 1.2 No caret, and that is a statement rather than an omission
//!
//! Acrobat Reader's Select tool draws a highlight and **no blinking caret**; a
//! caret appears only where something can be typed (a form field). Inkscape and
//! SolidWorks both draw one, and both are *editing* text when they do.
//!
//! Reading is the subject here, so Acrobat wins — and there is a second,
//! stronger reason that is about this shell rather than about convention: a
//! caret promises an **insertion point**, and there is nothing to insert.
//! Phase 5 (in-place text editing) is explicitly last in the operator's order
//! and `HANDOFF.md` says *"do not start it early"*. A caret drawn now would be
//! an affordance for a feature that does not exist, which is the
//! no-placeholders invariant read straight (`PROJECT_PLAN.md` §3).
//!
//! `pdfce-core` already publishes everything a caret needs
//! ([`EditableTextModel::caret_x`], `caret_left`, `caret_right`, `caret_up`,
//! `caret_down`) — written for Phase 5. None of it is called here. That is
//! where a caret comes from when there is something to type into.
//!
//! ### 1.3 Ctrl+A means "everything this gesture can select"
//!
//! Acrobat selects all the text; Inkscape selects all the objects in the layer;
//! SolidWorks selects everything in the field being edited. All three are the
//! same rule — *select everything the thing you are currently selecting in* —
//! and this shell applies it: where a press selects text, Ctrl+A selects the
//! page's text ([`select_all`]).
//!
//! **The other half is deliberately absent and is named rather than implied:**
//! in a mode that selects page content there is no select-all, because
//! `canvas::selection` has no "every object on the page" verb and inventing one
//! inside a keyboard handler would put a selection rule somewhere other than
//! the module that owns selection rules. So Ctrl+A does nothing in Edit today,
//! exactly as it did before this change — no regression, one honest gap, and
//! the shape of the fix recorded here.
//!
//! ---
//!
//! ## 2. What a drag does, and the rectangle that is not built
//!
//! A drag selects the **range** between where the button went down and where
//! the pointer is — in the engine's content order, which is what makes it flow
//! round line ends and across a column break rather than sweeping a box.
//!
//! Acrobat's second mode — `Alt`+drag for a **rectangular** text selection — is
//! genuinely useful on the drawing sheets this application exists for, where a
//! parts table's column is a rectangle and emphatically not a range. It is
//! **not built**, and the reason is the one the brief for this work put first:
//! *"one derivation, so what is shown and what is copied cannot diverge"*. A
//! rectangular selection is a second selection model — its copy is column-wise,
//! its reading order is its own, and it cannot be expressed as a
//! `(TextPosition, TextPosition)` pair at all — so it would be a second
//! [`resolve`] with a second quad derivation and a second copy path beside it.
//! That is the divergence this module is built to make impossible, bought for a
//! modifier.
//!
//! What it would take, so the next hand does not re-derive it: a
//! `Selection::Rect(egui::Rect)` variant beside the `(anchor, focus)` pair
//! [`TextSelection`] carries today, resolved by filtering
//! `PageText`'s glyphs on their own geometry rather than by
//! [`EditableTextModel::resolve_range`], with the copy assembled per line from
//! the surviving glyphs. Both variants would then have to flow through one
//! `resolve` returning one [`TextSelection`], which is what keeps the promise
//! above.
//!
//! ---
//!
//! ## 3. ★ THE MODE GATE — moved out, and where it went
//!
//! **[`gate`], and its header is the whole argument.** It used to be this
//! section — ~190 lines on why text selection needs no capability, why it still
//! has to be told apart from the content marquee, what the rule yields mode by
//! mode, and what changed when [`crate::canvas::tool::CanvasTool::Text`] gave it
//! a second disjunct. It moved with [`takes_the_press`] and with the three tests
//! that are about it, when this file crossed R2's 1,500-line limit for the
//! second time.
//!
//! The one-line version, so a reader here is not sent away for nothing:
//!
//! > **A press means text when the text tool is armed, *or* when the select tool
//! > is active and the mode cannot select content** — and selecting text needs
//! > no capability at all, because it authors nothing.
//!
//! ---
//!
//! ## 4. One page, and content order
//!
//! **A selection is a range on one page.** Acrobat's crosses pages; this one
//! does not, and it is a cost decision rather than a taste one.
//!
//! `crate::find`'s header carries the measurement: a whole-document extraction
//! is 331–449 ms on this project's fixtures, which is why Find never searches
//! on a keystroke. A cross-page selection needs a document-wide index — every
//! page walked, tokenised and font-resolved — and it needs it live, because a
//! drag samples the pointer sixty times a second. Per **page** it is one
//! extraction cached on `(page, edit epoch)` and free thereafter
//! (`app::cache::PageTextCache`); per **document** it is Find's number
//! paid again on every page turn, to make a gesture work that ends at the
//! window edge anyway. The anchor and the focus would also live on pages with
//! different [`crate::canvas::mapping::PageMapping`]s, which `canvas::interact`
//! is single-page by construction and says so.
//!
//! **Columns are a different matter and need nothing.** `PageText::runs` is in
//! page content order, and the engine inserts a derived line break at a
//! backward horizontal jump — its `backward_jump_ratio`, which exists because
//! *"a two-column page whose columns share baselines runs the two columns
//! together into one line with no separator at all"*. So a drag from the first
//! column into the second selects everything between them in content order,
//! with the columns separated, which is what Acrobat does. Neither this module
//! nor the operator has to know a column existed.
//!
//! Content order is **not** appearance order, and the engine says so
//! (§14.8.2.3.1: the two orderings *"may or may not coincide"*). A file whose
//! producer emitted its text out of visual order will select out of visual
//! order. That is the file's ordering, faithfully reported; inventing a
//! geometric reading order here would be a third derivation on top of two.
//!
//! ---
//!
//! ## 5. ★ One derivation: what is highlighted IS what is copied
//!
//! The brief's own requirement, and the defect it names: *"Highlight the
//! selected text, drawn from the same quads the copy will use — one derivation,
//! so what is shown and what is copied cannot diverge."*
//!
//! [`resolve`] is that one derivation. It takes the ordered pair of
//! [`TextPosition`]s **once**, walks the covered runs **once**, and in that
//! single pass produces both halves of [`TextSelection`]: the string is sliced
//! from the runs' own text as the walk passes through them, and the boxes are
//! accumulated from the glyphs inside the same byte windows. There is no second
//! entry point, no "recompute the quads for drawing", and no way to ask for one
//! without the other — [`TextSelection`]'s fields are populated together or the
//! value does not exist.
//!
//! That also makes the highlight free to draw. The quads are stored in **canvas
//! space**, which is zoom-independent, so a frame that merely paints an
//! existing selection runs no extraction, builds no model and does no
//! geometry — the same property `canvas::selection` relies on for its outlines
//! and `crate::find::Hit::canvas` for its wash.
//!
//! ### 5.1 ★ The same pass produces a THIRD output, and that is why
//!
//! [`TextSelection`] carries its boxes twice: [`TextSelection::quads`] in
//! **canvas space**, which is what the overlay paints, and
//! [`TextSelection::page_quads`] in **PDF user space**, which is what a
//! `/QuadPoints` text markup is authored from ([`crate::canvas::markup::text`]).
//! Both are `boxes` — the one `Vec` accumulated in [`resolve`]'s single walk —
//! and neither can exist without the other.
//!
//! It would have been one field fewer to store the canvas boxes alone and let
//! the authoring site invert [`crate::viewer::canvas_to_pdf_space`] over their
//! corners. That is refused, for two reasons and the second is the one that
//! decides it:
//!
//! * **It is a second derivation of the geometry**, arriving through the door
//!   §5 exists to lock. The rule is not *"do not extract twice"* — it is that
//!   what is shown and what is committed must be the same value, and two
//!   spellings of the same projection are exactly how they come to differ.
//! * **The inverse is not the identity on a rotated page.** The forward hop is
//!   `find::reveal::quad_to_canvas`, which maps all four corners and takes their
//!   bounds precisely because `/Rotate 90` sends `ul`/`lr` to two corners that
//!   are no longer the extremes. Inverting a *bounded* rect corner by corner
//!   gives back two opposite corners in an order `Rect::from_corners` is not
//!   promised to normalise — a mark that lands mirrored about the page's centre
//!   line, in the file, discovered after saving. That is the failure
//!   [`crate::canvas::markup`]'s own §1 is built around, reintroduced by an
//!   optimisation worth eight bytes a line.
//!
//! ### Why the highlight does not repeat Find's defect
//!
//! `HANDOFF.md` §2's defect 3 is *"Find's current-hit highlight completely
//! covered the word it highlighted"*, found by driving the binary and fixed by
//! taking the wash from alpha 168 down to 96. The lesson recorded on
//! `overlay::CURRENT_ALPHA` is general: *the operator's next act after finding a
//! hit is to READ it*.
//!
//! It applies here with more force, not less — a selection is what you are
//! about to copy, and an operator who cannot read it cannot tell whether they
//! swept the right words. So the selection wash reuses the same themed colour at
//! the same low end (`overlay::TEXT_SELECTION_ALPHA`), with the compile-time
//! bound that made Find's fix stick, and it is drawn **unstroked**: Find strokes
//! its current hit to distinguish it from its neighbours, and a text selection
//! has no neighbours to be distinguished from. A stroke per line box would also
//! draw a visible seam between two lines of one selection, which is a boundary
//! the operator did not make.
//!
//! ### The glyph box, and the constant that had two candidates
//!
//! A glyph carries an origin, an advance and a size — not a box. `pdfce-core`
//! approximates one in two places and **they do not agree**:
//!
//! | site | ascent | descent |
//! |---|---|---|
//! | `EditSession`'s search quad (what `TextMatch::quad` is) | `+0.85 × size` | `−0.22 × size` |
//! | `TextRun::bbox` and `Line::bbox` | `+0.75 × size` | `−0.25 × size` |
//!
//! There is no shared constant to inherit, so it is a choice, and it is made
//! **for the search quad's numbers**: `crate::find` draws its highlights from
//! `TextMatch::quad`, and Find is the surface an operator will see next to this
//! one — searching for a word and then selecting the same word must not produce
//! two boxes of visibly different heights over the same glyphs. Matching the
//! *bbox* numbers would instead match a box nothing paints.
//!
//! ---
//!
//! ## 6. This module authors nothing, and neither does copying
//!
//! No function here takes `&mut EditSession`, raises an
//! [`crate::app::actions::Action`], or bumps `edit_epoch`. The selection lives
//! on [`crate::app::state::OpenDoc`] beside the object selection, which
//! `canvas`'s header already argues is not a document mutation: *"a selection
//! *names* parts of a document and changes nothing a save would write."*
//!
//! Copying is the same class one step further out: it reads the extraction and
//! calls `egui::Context::copy_text`. It is not routed through the action funnel
//! for the reason `file.print` is not — the funnel exists for work that touches
//! a document or must not happen mid-frame, and this is neither.
//!
//! Rule 4 (disclosure lives off-canvas) is satisfied the way `canvas::overlay`'s
//! header states it: with nothing selected this paints nothing at all, so *would
//! a screenshot of the canvas differ from a screenshot of the same document
//! saved and reopened?* answers no by construction. A selection wash is a
//! pre-commit affordance — the cursor, describing what a copy would take — in
//! exactly the category rule 4 admits alongside the rubber band and the snap
//! indicator.
//!
//! ## 7. Staleness
//!
//! A [`TextPosition`] is `(run index, byte offset)` **into a particular
//! extraction**. An edit re-writes content streams, so run indices renumber and
//! byte offsets move: a position recorded before an edit can name different
//! glyphs, no glyphs, or the right glyphs in the wrong place. That is Find's
//! staleness problem exactly, and `crate::find`'s header rejects the same two
//! wrong answers — re-resolving automatically (an extraction per edit) and
//! drawing the old geometry anyway (*"a highlight that may be over the wrong
//! text, which is the one thing rule 4 forbids outright"*).
//!
//! Find keeps the query and drops the geometry, because a query is something the
//! operator typed. A selection has no such half: it **is** geometry. So the
//! whole thing is dropped — [`TextSelection::epoch`] records the revision it was
//! resolved against, [`TextSelection::live`] answers `false` the instant that
//! moves, and the overlay is handed nothing.
//!
//! ★ **Authoring a text markup is itself an edit**, so marking a selection
//! makes that selection stale on the very next frame: `add_markup` goes through
//! `vector_edit`, which bumps `edit_epoch`, and the wash disappears. Acrobat
//! keeps its selection across a markup and this does not, which is a real
//! difference and is recorded rather than smoothed over. The alternative is a
//! second staleness rule — *"an edit that adds an annotation does not move the
//! text"* — living outside this module and free to disagree with the one here;
//! the epoch is the only signal there is, and refining it into kinds of edit is
//! a mechanism, not a line. What the operator loses is one re-sweep to underline
//! *and* strike out the same words.
//!
//! ## 8. Where the rest of this module is
//!
//! The two chords (`Ctrl+A`, `Ctrl+C`), the guard in front of them and the one
//! function that writes the clipboard live in [`clipboard`], re-exported flat so
//! every call site still writes `textsel::copy` and `textsel::pending_key`.
//! Split there rather than anywhere else because [`clipboard::copy`] is reached
//! by two **ribbon commands** that have no selection at all — see that module's
//! header for the seam and for the R2 measurement that forced it.

use std::collections::HashMap;

use egui::{Pos2, Rect};
use pdfce_core::annot_author::Quad;
use pdfce_core::page_tree::{Page, Rect as PdfRect};
use pdfce_core::text_edit::{BlockRecognitionOptions, EditableTextModel, TextPosition};
use pdfce_core::text_extract::PageText;

/// The two keyboard verbs and the clipboard write. See §8.
pub mod clipboard;

/// **Who owns the primary button** — the mode gate and the whole argument for
/// it. See §3.
pub mod gate;

pub use clipboard::{TextKey, apply_key, copy, pending_key};

/// Re-exported flat, so every call site still writes `textsel::takes_the_press`
/// and nothing outside `canvas/` learns that the module was split. The same
/// contract [`clipboard`]'s re-export above honours, for the same reason.
pub use gate::takes_the_press;

/// How far above the baseline a glyph's box reaches, as a fraction of its
/// effective font size.
///
/// `pdfce-core`'s **search-quad** number, deliberately, where its run and line
/// boxes use `0.75`. See the module header §5: `crate::find` paints
/// `TextMatch::quad`, and a selected word must not be a visibly different height
/// from the same word found. There is no shared constant in core to inherit, so
/// this is a decision rather than a reference.
const GLYPH_ASCENT: f32 = 0.85;

/// How far below the baseline a glyph's box reaches, as a fraction of its
/// effective font size. The descender half of [`GLYPH_ASCENT`]'s pairing.
const GLYPH_DESCENT: f32 = 0.22;

/// **A range of characters on one page, and everything derived from it.**
///
/// The two halves of §5's promise travel together: [`Self::quads`] is what the
/// overlay paints and [`Self::text`] is what a copy writes, and both are
/// produced by one pass of [`resolve`] over one ordered pair of positions.
/// There is no constructor that fills one without the other.
///
/// Empty selections do not exist as values: [`resolve`] returns `None` when the
/// range covers no glyphs, so a plain click — which collapses the range —
/// clears the field rather than storing a selection with nothing in it. That is
/// what makes `Option<TextSelection>` on the document a two-state question
/// instead of a three-state one.
#[derive(Debug, Clone, PartialEq)]
pub struct TextSelection {
    /// Which page the range is on. A selection is single-page — module header
    /// §4 — so this is a fact about the whole value rather than about one end.
    pub page: usize,
    /// Where the gesture started. Held so a drag or a Shift+click can extend
    /// **from** it: the anchor is the end the operator is not moving, and
    /// re-deriving it from the quads would be impossible once the focus has
    /// crossed it.
    anchor: TextPosition,
    /// Where the pointer is now. The end a drag moves.
    focus: TextPosition,
    /// The [`crate::app::state::OpenDoc::edit_epoch`] the positions above were
    /// resolved against. See the module header §7 — this is the whole of the
    /// staleness mechanism.
    epoch: u64,
    /// The selected glyphs' boxes, **in canvas space**, one per line of the
    /// selection.
    ///
    /// Canvas space (Y-down, page top-left, `/Rotate` applied) rather than PDF
    /// user space, and projected once here rather than per frame, for the
    /// reason `crate::find::Hit::canvas` gives for doing the same: page
    /// geometry cannot change while a document is open, so the answer is
    /// constant for the life of the selection, and the paint path becomes a
    /// projection with no PDF concepts in it at all.
    ///
    /// One box per line rather than one per glyph — a hundred adjacent
    /// rectangles paint as one band anyway, and merging them is what lets a
    /// selection over a paragraph cost four boxes instead of four hundred.
    pub quads: Vec<Rect>,
    /// ★ **The same boxes, in PDF user space** — ready to become a text
    /// markup's `/QuadPoints`.
    ///
    /// One entry per entry of [`Self::quads`], in the same order, from the same
    /// accumulation in [`resolve`]. Not a conversion *of* that field and not a
    /// second walk: the walk produces one `Vec` of PDF-space rectangles and both
    /// of these are built from it, which is what makes *"what is highlighted is
    /// what is marked"* true by construction rather than by two functions
    /// agreeing. Module header §5.1 carries the argument, including why
    /// inverting the canvas projection at the authoring site is the wrong answer
    /// on a rotated page.
    ///
    /// `Quad` rather than `Rect` because that is the type
    /// [`pdfce_core::annot_author::MarkupSpec::TextMarkup`] takes, and building
    /// it here — once, from the rectangle the glyphs actually produced — leaves
    /// the authoring site with nothing geometric to decide.
    pub page_quads: Vec<Quad>,
    /// **Exactly the characters those boxes cover**, ready for the clipboard.
    ///
    /// Includes the engine's derived word spaces and line breaks, because they
    /// are runs in their own right and the walk passes straight through them —
    /// which is what makes a copied paragraph read as a paragraph rather than
    /// as one unbroken word.
    pub text: String,
}

impl TextSelection {
    /// Whether this selection still describes the revision it was made
    /// against.
    ///
    /// The gate the overlay and every copy path ask before spending it. See the
    /// module header §7: after an edit the positions inside name runs that have
    /// moved, and painting the stored quads anyway is the one thing rule 4
    /// forbids outright.
    #[must_use]
    pub fn live(&self, epoch: u64) -> bool {
        self.epoch == epoch
    }

    /// The quads to paint on `page`, or nothing at all.
    ///
    /// Nothing when the page is not this selection's, and nothing when the
    /// revision has moved — so the overlay stops drawing by being handed an
    /// empty slice rather than by a check of its own, exactly as
    /// `find::FindState::page_highlights` arranges.
    #[must_use]
    pub fn highlights(&self, page: usize, epoch: u64) -> &[Rect] {
        if self.page == page && self.live(epoch) {
            &self.quads
        } else {
            &[]
        }
    }

    /// A selection built from nothing but a page, a revision and a list of
    /// page-space boxes — for the tests of the modules that **consume** one.
    ///
    /// # Why this exists rather than a fixture
    ///
    /// [`resolve`] is the only constructor, deliberately (see the type's docs),
    /// and it needs a `PageText` — which `pdfce-core` makes
    /// `#[non_exhaustive]`, so this crate cannot build one and every test here
    /// drives a real extraction of a real file. That is right for *this* module
    /// and wrong for [`crate::canvas::markup::text`], whose rules are about a
    /// selection's **page, revision and boxes** and nothing else: forcing it to
    /// open a fixture and hunt for a page whose glyphs happen to sit where the
    /// assertion needs them would make its tests slower, flakier and about the
    /// fixture instead of about the rule.
    ///
    /// `#[cfg(test)]`, so it cannot become a second production constructor —
    /// which is the property that keeps "an empty selection is `None`" true of
    /// every value the application can actually hold.
    ///
    /// The canvas boxes are filled with the page boxes' own numbers rather than
    /// a projection, because there is no page here to project through. They are
    /// therefore **not** what a real selection would paint; what is faithful is
    /// the one property a consumer depends on — that the two vectors have the
    /// same length and the same order.
    #[cfg(test)]
    #[must_use]
    pub fn for_test(page: usize, epoch: u64, page_quads: Vec<Quad>) -> Self {
        let quads = page_quads
            .iter()
            .map(|q| {
                Rect::from_min_max(
                    Pos2::new(q.ll.0 as f32, q.ll.1 as f32),
                    Pos2::new(q.ur.0 as f32, q.ur.1 as f32),
                )
            })
            .collect();
        Self {
            page,
            anchor: TextPosition::new(0, 0),
            focus: TextPosition::new(0, 0),
            epoch,
            quads,
            page_quads,
            // Not read by anything this constructor exists for; a copy is what
            // `resolve` produces from real runs, and inventing plausible prose
            // here would make a test look like it was about the text when it is
            // about the geometry.
            text: String::new(),
        }
    }

    /// ★ **The quads a text markup would be authored from**, or nothing at all.
    ///
    /// [`Self::highlights`]'s twin, and deliberately the same shape: the caller
    /// is handed an empty slice rather than being asked to check a revision for
    /// itself, so a stale selection cannot be marked by a caller who forgot —
    /// which is the *"a highlight that may be over the wrong text"* failure
    /// (module header §7) with an annotation written into the file instead of a
    /// wash drawn over it.
    ///
    /// There is no page argument, where [`Self::highlights`] takes one: the
    /// overlay draws a *particular* page and has to be told which, while an
    /// authoring caller is asking *"where would this go"* and the answer
    /// includes [`Self::page`]. Handing back the quads without the page would be
    /// the invitation to pair them with `doc.view.page_index`, which is the
    /// current page and not necessarily this selection's.
    #[must_use]
    pub fn marks(&self, epoch: u64) -> &[Quad] {
        if self.live(epoch) {
            &self.page_quads
        } else {
            &[]
        }
    }

    /// How many characters are selected. For the trace line and for tests.
    ///
    /// Byte length rather than a `char` count, deliberately and to match the
    /// `chars=` trace field: it is the length of the string a copy puts on the
    /// clipboard, so a trace and a clipboard cannot disagree, and it is the unit
    /// `TextPosition` already speaks (`pdfce-core` keys glyphs by byte offset
    /// because one code may decode to many code points).
    #[must_use]
    pub fn len(&self) -> usize {
        self.text.len()
    }

    /// Whether the selection covers nothing.
    ///
    /// Always `false` for a value that exists — [`resolve`] returns `None`
    /// rather than an empty selection, which is the invariant everything else
    /// here rests on. It is written anyway because clippy asks for it beside a
    /// `len`, and asking for it is right: a reader meeting `len()` is entitled
    /// to the companion, and the honest implementation *states* the invariant
    /// instead of leaving it to be inferred from four call sites.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

/// The facts about one page that every entry point below needs.
///
/// A struct rather than four parameters for the reason
/// `canvas::interact::Frame` is one: they are settled together, once, by the
/// caller that has the document, and passing them separately invites a call
/// site to fetch one of them for itself — which for `text` would mean a second
/// extraction and for `epoch` would mean a selection that outlives the
/// revision it describes.
#[derive(Clone, Copy)]
pub struct PageContext<'a> {
    /// The page's extracted text, from
    /// [`crate::app::state::OpenDoc::page_text`]. **The** extraction — see
    /// that method's docs on why there is only one.
    pub text: &'a PageText,
    /// The page itself, for the PDF-user-space → canvas projection.
    pub page: &'a Page,
    /// Which page this is, in the session's page space.
    pub index: usize,
    /// The revision the extraction describes, stamped onto any selection this
    /// produces. Module header §7.
    pub epoch: u64,
}

/// **Update the selection from a drag** — press at `from`, pointer now at `to`,
/// both in canvas space.
///
/// The anchor is re-derived from `from` on every frame rather than kept from
/// the press, and that is not laziness: `PointerFrame::press_origin` already
/// guarantees `from` is *where the button actually went down* (its own header
/// records the 94-point error that guarantee exists to fix), so re-deriving is
/// exact, and it removes the one state a drag could otherwise carry across
/// frames and get wrong.
///
/// Returns `None` when the drag covers no glyphs — a sweep across blank paper
/// selects nothing rather than the nearest word, which is what Acrobat does and
/// what stops a stray drag on a drawing sheet's margin producing a selection the
/// operator did not make.
#[must_use]
pub fn drag(ctx: &PageContext<'_>, from: Pos2, to: Pos2) -> Option<TextSelection> {
    let model = model(ctx);
    let anchor = hit(&model, ctx, from)?;
    let focus = hit(&model, ctx, to)?;
    resolve(&model, ctx, anchor, focus)
}

/// **Update the selection from a click.**
///
/// The four cases, in the order they are tested, which is also the order of
/// increasing emphasis:
///
/// | gesture | result | from |
/// |---|---|---|
/// | triple-click | the **line** under the pointer | Inkscape / SolidWorks — §1.1 |
/// | double-click | the **word** under the pointer | all three |
/// | Shift+click | extend the existing selection, keeping its anchor | all three |
/// | plain click | collapse — i.e. **clear** | all three |
///
/// `current` is the selection as it stands; it is read only by the Shift case,
/// which needs the anchor to extend *from*. A Shift+click with nothing selected
/// falls through to a plain click, because there is no anchor to extend and
/// inventing one at the top of the page would select a paragraph the operator
/// never pointed at.
#[must_use]
pub fn click(
    ctx: &PageContext<'_>,
    current: Option<&TextSelection>,
    point: Pos2,
    shift: bool,
    double: bool,
    triple: bool,
) -> Option<TextSelection> {
    let model = model(ctx);
    let at = hit(&model, ctx, point)?;
    if triple {
        // `line_range_at` answers `None` for a position on a run carrying no
        // clustered glyph, which `hit_test` does not produce — but a `None`
        // here must clear rather than fall through to the word case, or an
        // emphatic gesture would quietly become a weaker one.
        let (start, end) = model.line_range_at(at)?;
        return resolve(&model, ctx, start, end);
    }
    if double {
        let (start, end) = model.word_range_at(at);
        return resolve(&model, ctx, start, end);
    }
    if shift && let Some(current) = current.filter(|c| c.page == ctx.index && c.live(ctx.epoch)) {
        return resolve(&model, ctx, current.anchor, at);
    }
    // A plain click collapses the range onto one caret slot, which covers no
    // glyphs, so `resolve` answers `None` and the caller clears. Expressed as a
    // degenerate range rather than as an early `return None` on purpose: there
    // is one rule for what a range means, and "a click is an empty range" is a
    // statement in that rule rather than an exception beside it.
    resolve(&model, ctx, at, at)
}

/// **Select every character on the page** — Ctrl+A. Module header §1.3.
///
/// The range runs from the first byte of the first run to the last byte of the
/// last: [`EditableTextModel::resolve_range`] orders and clamps the pair itself,
/// and walks whole intervening runs, so this needs no knowledge of where the
/// glyphs actually are. A page whose extraction produced no runs at all answers
/// `None`, which clears — the honest result for a page with no text on it.
#[must_use]
pub fn select_all(ctx: &PageContext<'_>) -> Option<TextSelection> {
    let model = model(ctx);
    let last = ctx.text.runs.len().checked_sub(1)?;
    let end = ctx.text.runs.get(last)?.text.len();
    resolve(
        &model,
        ctx,
        TextPosition::new(0, 0),
        TextPosition::new(last, end),
    )
}

/// Build the derived line/column/block structure over `ctx.text`.
///
/// Rebuilt per gesture event rather than cached, which is the same judgement
/// the old shell recorded (*"cheap, index-only"*) and is affordable for a
/// structural reason: the model **borrows** the `PageText` and owns no glyph
/// data, so recognition is a clustering pass over indices rather than a copy of
/// the page. The expensive half — the content-stream walk — is the thing that
/// *is* cached, on `(page, edit epoch)`, in [`crate::app::cache::PageTextCache`].
///
/// Caching the model instead would mean storing a value that borrows a
/// `RefCell`'s contents, which is the self-referential shape neither `Ref` nor
/// this crate's cache pattern can express.
///
/// `BlockRecognitionOptions::default()` and not a customized one: its ratios and
/// `ExtractOptions`' segmentation ratios are two halves of one derivation, and
/// tuning either alone would make the lines this shell paints and the lines the
/// engine derived describe different text.
fn model<'a>(ctx: &PageContext<'a>) -> EditableTextModel<'a> {
    EditableTextModel::recognize(ctx.text, &BlockRecognitionOptions::default())
}

/// Where a canvas-space point lands in the page's text.
///
/// Two hops, and the first is the one that is easy to get backwards: the canvas
/// speaks **Y-down from the page's top-left with `/Rotate` applied**, and every
/// glyph position `pdfce-core` reports is in **PDF user space — Y-up, from the
/// un-rotated CropBox's lower-left**. `canvas::mapping`'s header names conflating
/// those two as *the classic silent defect*, and it is silent here in the worst
/// way: the page looks perfect, and a drag selects a mirrored line.
///
/// So the conversion goes through [`crate::viewer::canvas_to_pdf_space`], which
/// is the single bridge for that hop and works by inverting the **renderer's
/// own** device transform — so the geometry and the picture agree by
/// construction rather than by two implementations happening to match.
///
/// `None` when the page's transform will not invert, or when the page has no
/// clustered glyph at all. Note that [`EditableTextModel::hit_test`] otherwise
/// **always answers**, falling back to the nearest line when no line's box
/// contains the point — which is deliberate and is Acrobat's behaviour: a drag
/// begun in the margin selects from the nearest text rather than from nothing.
fn hit(model: &EditableTextModel<'_>, ctx: &PageContext<'_>, canvas: Pos2) -> Option<TextPosition> {
    let pdf = crate::viewer::canvas_to_pdf_space(canvas, ctx.page)?;
    model.hit_test(f64::from(pdf.x), f64::from(pdf.y))
}

/// ★ **The one derivation** — module header §5.
///
/// One ordered pair in, one [`TextSelection`] out, and both of its halves
/// produced by the same walk over the same byte windows:
///
/// * the **string** is sliced out of each covered run's own `text`, so derived
///   word spaces and line breaks — which are runs carrying no glyphs — are
///   copied along with the characters they separate;
/// * the **boxes** are accumulated from the glyphs whose byte ranges intersect
///   those same windows, grouped by the line the engine put each glyph on.
///
/// The glyph list comes from [`EditableTextModel::resolve_range`] rather than
/// being re-derived from the byte windows here, because that function already
/// owns the intersection rule (including its correct treatment of a zero-width
/// caret window, which selects nothing) and a second implementation of it is
/// precisely how a highlight comes to cover one glyph more than the copy does.
///
/// Returns `None` for a range covering no glyphs. That is the *only* way a
/// caller clears a selection through this module, which is what makes "an empty
/// selection is `None`" true everywhere rather than in most places.
fn resolve(
    model: &EditableTextModel<'_>,
    ctx: &PageContext<'_>,
    anchor: TextPosition,
    focus: TextPosition,
) -> Option<TextSelection> {
    let covered = model.resolve_range(anchor, focus);
    if covered.is_empty() {
        return None;
    }

    // Which line the engine clustered each glyph onto. Built from
    // `model.lines()` rather than by re-clustering on baseline y: the engine's
    // lines already account for the backward-jump split that separates two
    // columns sharing a baseline (module header §4), and a box drawn from a
    // second clustering would span a column gap the copy does not.
    let mut line_of: HashMap<(usize, usize), usize> = HashMap::new();
    for (index, line) in model.lines().iter().enumerate() {
        for gref in &line.glyphs {
            line_of.insert((gref.run, gref.glyph), index);
        }
    }

    // The boxes, in the order their lines are first met, so a selection's quads
    // are in the same content order as its text. `Vec` rather than a map keyed
    // on the line index: the count is one per line of the selection, so a linear
    // scan is cheaper than hashing, and the order is the point.
    let mut boxes: Vec<(usize, PdfRect)> = Vec::new();
    for gref in &covered {
        let Some(glyph) = model.glyph(*gref) else {
            continue;
        };
        // The engine's own approximation of a glyph box, with the ascent and
        // descent fractions chosen in the module header §5.
        let (x0, x1) = (glyph.x, glyph.x + glyph.advance);
        let cell = PdfRect::from_corners(
            f64::from(x0.min(x1)),
            f64::from(glyph.y - glyph.size * GLYPH_DESCENT),
            f64::from(x0.max(x1)),
            f64::from(glyph.y + glyph.size * GLYPH_ASCENT),
        );
        // A glyph the line clustering did not claim still has to be drawn, or a
        // selection would silently highlight less than it copies. `usize::MAX`
        // keyed per glyph would merge them all into one band, so unclaimed
        // glyphs get a box each — visibly correct, and rare enough that the
        // cost is not worth a second clustering rule.
        let key = line_of
            .get(&(gref.run, gref.glyph))
            .copied()
            .unwrap_or(usize::MAX);
        match boxes
            .iter_mut()
            .find(|(k, _)| *k == key && key != usize::MAX)
        {
            Some((_, r)) => {
                *r = PdfRect::from_corners(
                    r.llx.min(cell.llx),
                    r.lly.min(cell.lly),
                    r.urx.max(cell.urx),
                    r.ury.max(cell.ury),
                );
            }
            None => boxes.push((key, cell)),
        }
    }

    // The characters, from the runs themselves. `get(..)` rather than indexing:
    // core guarantees a `TextPosition`'s offset is on a glyph boundary and
    // therefore on a UTF-8 boundary, and a stale position that is not must
    // contribute nothing rather than panic in the middle of a drag.
    let (start, end) = ordered(anchor, focus);
    let mut text = String::new();
    for index in start.run..=end.run.min(ctx.text.runs.len().saturating_sub(1)) {
        let Some(run) = ctx.text.runs.get(index) else {
            break;
        };
        let lo = if index == start.run {
            start.byte_offset
        } else {
            0
        };
        let hi = if index == end.run {
            end.byte_offset
        } else {
            run.text.len()
        };
        if let Some(slice) = run.text.get(lo..hi) {
            text.push_str(slice);
        }
    }

    // ★ The projection into canvas space, through `find::reveal::quad_to_canvas`
    // — the SAME function Find projects its hits with. Reusing it rather than
    // mapping two corners here is what makes a selection box and a find box over
    // the same word land in the same place on a rotated page: it maps all four
    // corners and bounds them, because `/Rotate 90` sends the `ul`/`lr` pair to
    // two corners that are no longer the extremes.
    //
    // ★ **Both spaces are kept, and they are pushed in the same iteration** —
    // module header §5.1. A box whose projection declines contributes to
    // *neither*: the two vectors are index-aligned by construction, and a
    // `filter_map` on one with a plain `map` on the other would let the wash and
    // the authored mark describe different sets of glyphs, in the direction
    // nobody would notice (the mark is in the file; the wash is gone by the next
    // frame).
    let mut quads: Vec<Rect> = Vec::with_capacity(boxes.len());
    let mut page_quads: Vec<Quad> = Vec::with_capacity(boxes.len());
    for (_, rect) in boxes {
        let quad = Quad::from_rect(rect);
        if let Some(canvas) = crate::find::reveal::quad_to_canvas(&quad, ctx.page) {
            quads.push(canvas);
            page_quads.push(quad);
        }
    }
    if quads.is_empty() {
        return None;
    }

    Some(TextSelection {
        page: ctx.index,
        anchor,
        focus,
        epoch: ctx.epoch,
        quads,
        page_quads,
        text,
    })
}

/// The two positions in content order.
///
/// `TextPosition`'s own ordering key is private to `pdfce-core`, so the tuple is
/// spelled here — once, in the one function that needs it, rather than at each
/// of [`resolve`]'s two uses of "the earlier one".
fn ordered(a: TextPosition, b: TextPosition) -> (TextPosition, TextPosition) {
    if (a.run, a.byte_offset) <= (b.run, b.byte_offset) {
        (a, b)
    } else {
        (b, a)
    }
}

/// **Answer the text selection's own two chords, Ctrl+A and Ctrl+C.**
///
/// Moved here from `canvas::interact` on 2026-08-20 under R2, and it belongs
/// here: every rule it enforces is a rule about *this* module, and the caller
/// that used to hold them could not have got them right without knowing all of
/// them.
///
/// ★ These two live apart from [`crate::canvas::keys::canvas_keys`] because
/// both need the page's **extraction** — one to build a range over it, one to
/// read a string out of a selection made against it — and `canvas_keys` is
/// deliberately a document-free function that a headless `egui::Context` can
/// drive end to end. Escape stays there, where its precedence question is
/// answered.
///
/// ★ Gated on [`takes_the_press`], the same predicate the press is gated on, so
/// a mode whose primary button does not select content does not answer Ctrl+A
/// with a text selection the operator has no gesture to clear. §1.3 of this
/// module's header records that the *other* half of Ctrl+A — select every
/// object — is a known gap rather than an oversight.
///
/// ★★ **[`pending_key`] FIRST, and the ordering is the fix for a defect that
/// shipped and that driving the binary caught.** The chord is read off
/// `egui::InputState` — one map lookup — and the page's extraction is fetched
/// **only** when one fired. The first version asked for the extraction in order
/// to discover that no chord had been pressed, which built it on the first
/// frame of every reading canvas: measured at **392 ms at open** on
/// `ncored-benchmark-cad-drawing.pdf`, paid by an operator who had touched
/// nothing. It is the same gate `canvas::interact` step 4 puts in front of
/// `page_objects()`, for the same reason.
pub fn keys(
    ctx: &egui::Context,
    doc: &crate::app::state::OpenDoc,
    page_index: usize,
    active_tool: crate::canvas::tool::CanvasTool,
    caps: crate::app::modes::Capabilities,
    selection: &mut Option<TextSelection>,
) {
    if let Some(key) = pending_key(ctx)
        && takes_the_press(active_tool, caps)
        && let (Some(page_text), Some(page)) = (doc.page_text(), doc.pages.get(page_index))
    {
        let text_ctx = PageContext {
            text: &page_text,
            page,
            index: page_index,
            epoch: doc.edit_epoch,
        };
        apply_key(ctx, &text_ctx, key, selection);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::{FOUR_PAGES, OpenDoc, open_fixture};

    /// Run `body` against page 0 of a fixture, with the real extraction.
    ///
    /// Everything below drives a **real** `PageText`: `PageText`, `TextRun` and
    /// `ExtractedGlyph` are all `#[non_exhaustive]`, so this crate cannot build
    /// one — which is a constraint worth naming rather than working around,
    /// because it means every assertion here is about the engine's actual output
    /// on an actual file.
    fn on_page<R>(body: impl FnOnce(&PageContext<'_>) -> R) -> R {
        let doc: OpenDoc = open_fixture(FOUR_PAGES);
        let text = doc.page_text().expect("the fixture's first page extracts");
        let page = doc.pages.first().expect("the fixture has pages");
        body(&PageContext {
            text: &text,
            page,
            index: 0,
            epoch: doc.edit_epoch,
        })
    }

    /// The canvas point at the centre of the first glyph the page draws.
    ///
    /// Derived from the extraction rather than guessed, for the reason
    /// `ui-verify`'s `coords` module gives about guessed points: a coordinate
    /// that misses is symptom-identical to a hit test that is broken, and this
    /// project has already filed one retracted defect on exactly that.
    fn first_glyph_centre(ctx: &PageContext<'_>) -> Pos2 {
        let run = ctx
            .text
            .runs
            .iter()
            .find(|r| !r.glyphs.is_empty())
            .expect("the fixture's page draws glyphs");
        let g = run.glyphs.first().expect("checked non-empty");
        let pdf = egui::pos2(g.x + g.advance / 2.0, g.y + g.size * 0.25);
        crate::viewer::pdf_space_to_canvas(pdf, ctx.page).expect("a real page projects")
    }

    // =======================================================================
    // ★ One derivation — module header §5
    // =======================================================================

    /// ★ **What is highlighted is what is copied.**
    ///
    /// The brief's own requirement, asserted the only way it can be asserted
    /// from outside: select every character on the page, and check that the
    /// value carries *both* halves and that they describe the same thing —
    /// non-empty text, at least one box, and a box count that cannot exceed the
    /// number of lines the engine derived.
    ///
    /// The last clause is what makes this more than "something was produced": a
    /// build whose grouping key was wrong would emit one box per **glyph**, and
    /// a page of text has far more glyphs than lines.
    #[test]
    fn a_selection_carries_its_text_and_its_boxes_from_one_pass() {
        on_page(|ctx| {
            let all = select_all(ctx).expect("the fixture's page has text");
            assert!(!all.text.is_empty(), "select-all copied nothing");
            assert!(!all.quads.is_empty(), "select-all highlighted nothing");
            let lines = model(ctx).lines().len();
            assert!(
                all.quads.len() <= lines.max(1),
                "{} boxes for {lines} derived lines — the per-line grouping is not grouping",
                all.quads.len()
            );
            assert_eq!(all.page, 0);
            assert!(all.live(ctx.epoch));
        });
    }

    /// ★ **The boxes exist in both spaces, index for index** — module header
    /// §5.1.
    ///
    /// The property the text-markup kinds rest on: the wash the operator sees
    /// and the `/QuadPoints` written into the file are the same boxes, so a
    /// build where one vector was filtered and the other was not would mark
    /// glyphs it never highlighted. Asserted as an equal length **and** as a
    /// per-entry correspondence of *width* — a length check alone would pass on
    /// a build that pushed the right number of wrong quads.
    ///
    /// The width comparison is deliberately loose about units: canvas space is
    /// scaled by nothing here (it is page points, Y-down) so on an upright page
    /// the two widths are equal, and the assertion is written as "both
    /// non-degenerate and within a point" rather than as equality, because a
    /// rotated fixture would legitimately swap the axes.
    #[test]
    fn every_painted_box_has_the_page_space_quad_a_markup_would_use() {
        on_page(|ctx| {
            let all = select_all(ctx).expect("the fixture's page has text");
            assert!(!all.page_quads.is_empty(), "no quads to author from");
            assert_eq!(
                all.quads.len(),
                all.page_quads.len(),
                "the wash and the mark must describe the same boxes"
            );
            for (canvas, quad) in all.quads.iter().zip(&all.page_quads) {
                let quad_width = (quad.ur.0 - quad.ul.0).abs();
                let quad_height = (quad.ul.1 - quad.ll.1).abs();
                assert!(
                    quad_width > 0.0 && quad_height > 0.0,
                    "a degenerate quad marks nothing: {quad:?}"
                );
                assert!(
                    (f64::from(canvas.width()) - quad_width).abs() < 1.0
                        || (f64::from(canvas.height()) - quad_width).abs() < 1.0,
                    "the painted box {canvas:?} and the authored quad {quad:?} are not the \
                     same box"
                );
            }
            // …and `marks` is the accessor that enforces the revision, exactly
            // as `highlights` does for the painted half.
            assert_eq!(all.marks(ctx.epoch).len(), all.page_quads.len());
            assert!(
                all.marks(ctx.epoch + 1).is_empty(),
                "a stale selection must not author an annotation over glyphs that may have moved"
            );
        });
    }

    /// ★ **An edit makes a selection stale, and a stale selection paints
    /// nothing** — module header §7.
    ///
    /// Both halves, because the second is the one rule 4 turns on: a stored quad
    /// after an edit may be over different glyphs, and drawing it anyway is the
    /// thing `crate::find`'s staleness section calls out as forbidden outright.
    #[test]
    fn an_edit_makes_a_selection_stale_and_stops_the_highlight() {
        on_page(|ctx| {
            let all = select_all(ctx).expect("the fixture's page has text");
            assert!(!all.highlights(0, ctx.epoch).is_empty());
            assert!(!all.live(ctx.epoch + 1), "one edit later");
            assert!(
                all.highlights(0, ctx.epoch + 1).is_empty(),
                "a quad recorded before an edit may cover different glyphs after it"
            );
            assert!(
                all.highlights(1, ctx.epoch).is_empty(),
                "…and a selection describes one page, so another page's overlay gets nothing"
            );
        });
    }

    // =======================================================================
    // The gestures
    // =======================================================================

    /// ★ **A double-click selects a word, and a triple-click selects at least as
    /// much.**
    ///
    /// The two emphatic gestures, asserted *against each other* rather than
    /// separately: a build where triple-click fell through to the word case
    /// would pass two independent "selects something" tests and fail this one.
    /// A word is also asserted to contain no whitespace, which is what
    /// distinguishes it from a line on any page whose lines have more than one
    /// word — and the test says so rather than assuming it.
    #[test]
    fn a_double_click_takes_a_word_and_a_triple_click_takes_at_least_the_line() {
        on_page(|ctx| {
            let at = first_glyph_centre(ctx);
            let word = click(ctx, None, at, false, true, false)
                .expect("a double-click on a glyph selects its word");
            let line = click(ctx, None, at, false, false, true)
                .expect("a triple-click on a glyph selects its line");
            assert!(!word.text.is_empty());
            assert!(
                !word.text.trim().contains(char::is_whitespace),
                "a word must not span a space: {:?}",
                word.text
            );
            assert!(
                line.text.len() >= word.text.len(),
                "a line ({:?}) cannot be shorter than a word inside it ({:?})",
                line.text,
                word.text
            );
        });
    }

    /// ★ **A plain click clears** — Acrobat, Inkscape and SolidWorks alike.
    ///
    /// Expressed as `None` rather than as an empty selection, which is the
    /// invariant `TextSelection`'s own docs rest on: the field on the document
    /// is a two-state question.
    #[test]
    fn a_plain_click_clears_the_selection() {
        on_page(|ctx| {
            let at = first_glyph_centre(ctx);
            assert!(
                click(ctx, None, at, false, false, false).is_none(),
                "a click collapses the range, and an empty range is no selection"
            );
        });
    }

    /// ★ **A drag selects the range between its ends, and it is
    /// direction-blind.**
    ///
    /// Dragging right-to-left must select exactly what dragging left-to-right
    /// selected — the case a naive implementation gets wrong by assuming the
    /// press is the earlier position, and the same class of error
    /// `GestureOutcome::Markup`'s docs record for a normalised rect.
    #[test]
    fn a_drag_selects_the_same_range_in_both_directions() {
        on_page(|ctx| {
            let all = select_all(ctx).expect("the fixture's page has text");
            // Two points well inside the selection's own first box, so the drag
            // is known to be over glyphs rather than guessed to be.
            let box_ = all.quads[0];
            let left = egui::pos2(box_.min.x + 1.0, box_.center().y);
            let right = egui::pos2(box_.max.x - 1.0, box_.center().y);

            let forward = drag(ctx, left, right).expect("a drag across a line selects it");
            let backward = drag(ctx, right, left).expect("…and so does the same drag reversed");
            assert_eq!(
                forward.text, backward.text,
                "a gesture must mean the same thing in both directions"
            );
            assert_eq!(forward.quads, backward.quads);
            assert!(!forward.text.is_empty());
        });
    }

    /// Shift+click extends from the anchor rather than starting again — and with
    /// nothing selected it behaves as a plain click, because there is no anchor
    /// to extend from.
    #[test]
    fn shift_click_extends_from_the_anchor_and_needs_one() {
        on_page(|ctx| {
            let all = select_all(ctx).expect("the fixture's page has text");
            let box_ = all.quads[0];
            let start = egui::pos2(box_.min.x + 1.0, box_.center().y);
            let end = egui::pos2(box_.max.x - 1.0, box_.center().y);

            // A quarter of the way across the line, not one canvas unit: a
            // one-unit sweep can begin and end inside the same glyph, which
            // resolves both ends onto the *same* caret boundary and therefore
            // covers nothing. That is correct behaviour and a useless fixture —
            // and it is what the first draft of this test did.
            let quarter = egui::pos2(box_.min.x + box_.width() / 4.0, box_.center().y);
            let seed = drag(ctx, start, quarter).expect("a quarter-line sweep selects glyphs");
            let extended = click(ctx, Some(&seed), end, true, false, false)
                .expect("shift+click extends to the pointer");
            assert!(
                extended.text.len() > seed.text.len(),
                "extending must grow the range: {:?} then {:?}",
                seed.text,
                extended.text
            );

            assert!(
                click(ctx, None, end, true, false, false).is_none(),
                "shift+click with nothing selected has no anchor, so it clears like a plain click"
            );
        });
    }

    /// Ctrl+A takes the whole page and nothing beyond it — the range is clamped
    /// by `resolve_range`, so the last run's end is a real boundary rather than
    /// a byte past one.
    ///
    /// ★ Compared against **`plain_text()`**, not `sourced_text()`, and the
    /// difference is a lesson worth keeping: the first draft of this test split
    /// `sourced_text()` on whitespace and looked for the words in the copy, and
    /// it failed with `select-all dropped "OneChapter"`. `sourced_text()`
    /// deliberately omits every derived space and line break — it is the honest
    /// lower bound on *what the file provides* — so on this fixture it runs
    /// `Page One` and `Chapter 1` together into a token that exists in no
    /// selection anyone could make.
    ///
    /// That is exactly the distinction a copy has to get right in the other
    /// direction: [`resolve`] walks the **runs**, derived-whitespace runs
    /// included, which is what makes a copied paragraph paste as a paragraph.
    /// Asserting against `plain_text()` is asserting against the same
    /// segmentation the operator can see on the page.
    #[test]
    fn select_all_takes_every_character_on_the_page() {
        on_page(|ctx| {
            let all = select_all(ctx).expect("the fixture's page has text");
            let plain = ctx.text.plain_text();
            assert!(
                plain.split_whitespace().count() >= 4,
                "vacuous unless the fixture really has several words: {plain:?}"
            );
            for word in plain.split_whitespace() {
                assert!(
                    all.text.contains(word),
                    "select-all dropped {word:?} from {:?}",
                    all.text
                );
            }
            // …and the separators came too, or the copy would paste as one word.
            assert!(
                all.text.contains(char::is_whitespace),
                "a copy that drops the derived spaces pastes as one word: {:?}",
                all.text
            );
        });
    }

    /// A drag that touches no glyph selects nothing.
    ///
    /// Asserted at a point far outside the page box, because
    /// `EditableTextModel::hit_test` deliberately falls back to the *nearest*
    /// line rather than answering `None` — so the clearing has to come from the
    /// range covering no glyphs, and a build that had "nearest line" leak into a
    /// selection would fail here rather than in front of an operator.
    #[test]
    fn a_degenerate_drag_selects_nothing() {
        on_page(|ctx| {
            let far = egui::pos2(-10_000.0, -10_000.0);
            assert!(
                drag(ctx, far, far).is_none(),
                "a zero-length drag covers no glyphs, wherever it is"
            );
        });
    }

    // =======================================================================
    // Ordering
    //
    // The two keyboard verbs and the cost gate in front of them moved to
    // `clipboard.rs` with their tests — see this module's §8.
    // =======================================================================

    /// The ordering helper puts the earlier position first, on both axes of the
    /// key — the run before the offset, which is the order content is in.
    #[test]
    fn positions_order_by_run_then_offset() {
        let a = TextPosition::new(1, 5);
        let b = TextPosition::new(1, 9);
        let c = TextPosition::new(2, 0);
        assert_eq!(ordered(a, b), (a, b));
        assert_eq!(
            ordered(b, a),
            (a, b),
            "the same pair reversed must order the same way"
        );
        // Across runs, the run index decides regardless of the offsets — a
        // position at byte 0 of run 2 is after byte 5 of run 1.
        assert_eq!(ordered(c, a), (a, c));
        assert_eq!(ordered(a, c), (a, c));
    }
}
