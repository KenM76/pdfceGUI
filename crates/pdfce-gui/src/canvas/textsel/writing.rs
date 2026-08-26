//! # `canvas::textsel::writing` — recovering the direction the extraction drops
//!
//! ## The operator's report, 2026-08-26
//!
//! > *"I have text placed vertically on the bottom left corner of the
//! > SW41177.pdf. In Adobe when I hover over it the I cursor re-orients itself
//! > to match the text orientation, and when I select the text it shades each
//! > letter as part of the same block. when I copy and paste into notepad, I
//! > get the text on one line as expected. […] as it is now the I cursor
//! > doesn't reorient and it pastes each letter onto its own line."*
//!
//! Three symptoms, one cause, and the cause is upstream of everything this
//! shell does with text.
//!
//! ---
//!
//! ## 1. ★★★ The cause: `ExtractedGlyph` publishes a length where a vector
//! belongs
//!
//! `pdfce-core` places every glyph by the §9.4.4 text rendering matrix and then
//! publishes four scalars out of it:
//!
//! | field | what it is | measured how |
//! |---|---|---|
//! | `x`, `y` | the glyph **origin** in default user space | `trm.e`, `trm.f` — exact, and orientation-independent |
//! | `advance` | how far the next glyph sits **along the writing direction** | `tx * tm_ctm.x_scale() * sign(tx)` — a **magnitude** |
//! | `size` | the effective font size | `trm.y_scale()` — also a magnitude |
//!
//! The two basis *vectors* of the rendering matrix are reduced to their
//! *lengths*, so **which way the text runs is not published at all**. Verified
//! against the source at `pdfce-core/src/text_extract/page.rs` (the
//! `let advance = tx * tm_ctm.x_scale() * …` line), not inferred from the
//! rustdoc — which still calls the field a *"horizontal advance"*, a name that
//! is true of the overwhelming majority of PDFs and false of this one.
//!
//! Everything downstream then assumes the missing vector is `(1, 0)`:
//!
//! * **Segmentation.** `text_extract::layout::classify` breaks a line whenever
//!   `|Δy| > line_gap_ratio × size`. Text that advances **in y** therefore
//!   changes baseline at every single glyph, so the extraction emits a
//!   `DerivedLineBreak` run between every pair of letters. That is the operator's
//!   *"pastes each letter onto its own line"*, exactly and completely.
//!   Measured on `SW41177.pdf` page 36: the vertical file-path stamp is **72
//!   runs and 71 derived breaks** for what is one line of text.
//! * **Geometry.** A glyph box is taken as `x … x + advance` across and
//!   `y − 0.22·size … y + 0.85·size` up. For 90° text that box is the right
//!   *size* rotated the wrong way and hung off the wrong corner, so the
//!   selection wash sits beside the letters instead of over them. That is
//!   *"shades each letter as part of the same block"* not happening.
//! * **The cursor.** `canvas::cursor` draws pdfce's own I-beam and has no
//!   reason to know which way to draw it, because nothing it can ask knows.
//!
//! ★ **A request is filed with the engine** — see
//! `D:\Dev\FeatureRequests\pdfce_FeatureRequests\`. Direction-aware
//! segmentation belongs in the extraction, where one fix would serve every
//! caller, and publishing the writing direction on `ExtractedGlyph` belongs
//! there too. **This module is the shell-side workaround, and it is reported as
//! one** (pdfce decision 058: a workaround the GUI does not report is a crate
//! boundary defect that stays).
//!
//! ---
//!
//! ## 2. ★★ How the direction is recovered without guessing at it
//!
//! Two passes. The first decides **which directions this page uses**, on
//! evidence horizontal text cannot manufacture; the second groups glyphs into
//! lines, and will only ever group along a direction the first pass admitted.
//! That ordering is the whole safety argument: pass two is permissive, and it
//! is permissive inside a set that pass one had to be convinced to put anything
//! into at all.
//!
//! ### 2.1 ★★★ Pass one — chains, and the draft that was correct and useless
//!
//! The first version of this file measured directions from **runs**, on an
//! argument that is still true: `layout::Builder::push_glyph` calls
//! `close_run()` before it emits any derived whitespace, so a run holding two
//! or more glyphs is a sequence *the engine itself certified as adjacent*, and
//! the vector between its first and last glyph origins is parallel to the
//! writing direction by construction — including through `TJ` kerning, since
//! §9.4.3's adjustment translates the text matrix along its own x axis.
//!
//! **It was measured against the file and it barely worked.** On `SW41177.pdf`
//! page 36 the vertical stamp is 72 runs and only **10** of them hold two
//! glyphs — and those ten exist only because `:`, `i`, `n`, `c` and the space
//! are narrow enough that one of their advances stays inside `line_gap_ratio ×
//! size`. A vertical label set in wide letters, which is what `PART NO` in a
//! title block is, would have produced no two-glyph run at all, the census
//! would have been empty, and the feature would have done **nothing, silently,
//! on exactly the page it was written for**.
//!
//! That is a lesson worth more than the code: an argument that a measurement is
//! *sound* says nothing about whether the measurement is *available*.
//!
//! So [`census`] now looks for **chains**. A *step* is the displacement from
//! one glyph's origin to the next glyph's, in content order and straight
//! **through** the derived break runs — those breaks are the verdict being
//! re-judged, so honouring them would be assuming the answer. A step is sound
//! when its length falls in the window the engine's own `classify` would have
//! accepted along that direction. A chain is a maximal sequence of consecutive
//! sound steps pointing the same way, and a chain of at least three steps in a
//! non-horizontal direction is what admits that direction.
//!
//! **Horizontal text cannot clear that bar, and not by luck:**
//!
//! * *within* a line every step is horizontal and is dropped before it counts;
//! * the only non-horizontal steps an ordinary page has are the jumps *between*
//!   lines, and each of those is separated from the next by a whole line of
//!   horizontal steps — so they are never consecutive, and can never form a
//!   chain of two.
//!
//! ### 2.2 Pass two, and the failure direction
//!
//! [`lines`] then walks the same glyph sequence and groups it, adopting a
//! direction for a stretch **only from the census**. A page whose census is
//! empty — every ordinary document — returns an empty [`Rotated`] and the shell
//! behaves exactly as it did before this file existed.
//!
//! That is the deliberate failure direction. When this module is wrong it is
//! wrong by doing nothing, and *"nothing"* is the behaviour that shipped and
//! that the operator has been using. There is no input for which it can invent
//! a rotated line out of horizontal text.
//!
//! ### 2.3 ★ What an ordinary page pays
//!
//! One pass over the page's glyphs, in which the overwhelmingly common step
//! exits after two multiplications and a comparison — the horizontal test is
//! first for that reason. If the census comes back empty the second pass never
//! runs at all.
//!
//! This is not a micro-optimisation, it is the regression guard. The benchmark
//! sheet carries 129,758 objects, and this project has three times broken the
//! canvas with a change meant to affect one narrow case. A page with no rotated
//! text on it cannot reach the code that regroups glyphs, so it cannot be
//! changed by it. The cost is measured in this module's tests, not asserted
//! here.
//!
//! ---
//!
//! ## 3. The second pass, which IS `classify` with the axis put back
//!
//! [`lines`] walks the page's glyphs in content order — **through** the derived
//! break runs, which is the whole point, since those breaks are what is being
//! re-judged — and applies the engine's own three thresholds in the line's own
//! frame instead of in the page's:
//!
//! | engine, in page axes | here, in the line's frame |
//! |---|---|
//! | `\|Δy\| > line_gap_ratio × size` → line | `\|perp\| > line_gap_ratio × size` → line |
//! | `Δx − advance < −backward_jump_ratio × size` → line | `along − advance < −backward_jump_ratio × size` → line |
//! | `Δx − advance > word_gap_ratio × size` → word | `along − advance > word_gap_ratio × size` → word |
//!
//! where `along = d · dir` and `perp = d × dir`. Substituting `dir = (1, 0)`
//! gives back the engine's version line for line, which is the argument that
//! this is a generalisation rather than a second opinion.
//!
//! ★ **The ratios come from the caller's `ExtractOptions`, never from
//! `::default()`.** `app::settings::SettingsExt::extract_options` is the funnel
//! every extraction in this application goes through, and the operator can move
//! `word_gap_ratio`. A regrouping that used the default while the extraction
//! used the setting would put a word space in one and not the other — which is
//! the class of defect that funnel was built to end.
//!
//! ---
//!
//! ## 4. What this module produces, and the one thing it does not decide
//!
//! [`Rotated`] carries, per page:
//!
//! * **[`Rotated::line_of`]** — which rotated line each glyph belongs to, so
//!   `textsel::resolve` can band a selection's boxes by it instead of by the
//!   engine's one-glyph-per-line;
//! * **[`Rotated::lines`]** — each line's unit direction, so a box can be built
//!   in the right frame and the cursor can be turned to match;
//! * **[`Rotated::is_artefact`]** — whether a given `DerivedLineBreak` run is
//!   *internal to* a rotated line, and therefore a separator the extraction
//!   should never have emitted.
//!
//! ★ **The third one only ever removes, and only what it can prove.** A break
//! is claimed exactly when [`adjacent`] holds across it — the extraction's own
//! `Break::None` window, measured in the line's frame — so a claimed break is
//! one between two letters of a word and copies as nothing. Every break the
//! predicate declines to judge survives untouched, which is what makes
//! `W:\Engineering\Products…` come back as a path without any page anywhere
//! losing a newline that was real.
//!
//! **It does not decide what a selection looks like.** No box is built here, no
//! string is assembled, nothing is projected into canvas space. That stays in
//! `textsel::resolve`, whose §5 promise is that the highlight and the copy come
//! from one walk — this module hands that walk a better grouping and takes no
//! part in either half.
//!
//! ---
//!
//! ## 5. The stated limits
//!
//! * **A quadrant rotation is exact; an arbitrary angle is bounded.** For text
//!   at 90°, 180° or 270° the band this enables is axis-aligned in page space,
//!   so the canvas wash — which is a `Rect` — covers it exactly. For text at,
//!   say, 30°, the band is a genuine parallelogram and the wash is its bounding
//!   box, which over-covers at the corners. That is still strictly better than
//!   the pre-existing behaviour (a scatter of boxes beside the letters), and it
//!   is named here rather than discovered.
//! * **Vertical *writing mode* is not this.** A Type 0 font with `/WMode 1`
//!   (§9.7.4.3) advances downward with a different glyph metric set. What is
//!   handled here is ordinary horizontal-mode text placed by a rotated matrix,
//!   which is what CAD exporters emit and what `SW41177.pdf` contains. True
//!   `/WMode 1` would need `pdfce-core` to publish `w1`/position vectors, and
//!   it does not.
//! * **The engine's hit test is still axis-aligned.** `EditableTextModel`'s
//!   line boxes are built the same wrong way, so a click on rotated text lands
//!   by nearest-line fallback. See [`Rotated::direction_at`] for what this
//!   module does about the part that matters — the cursor — and
//!   `textsel`'s own tests for the accuracy that was measured rather than
//!   assumed.

use std::collections::{HashMap, HashSet};

use pdfce_core::text_edit::{GlyphRef, TextPosition};
use pdfce_core::text_extract::{ExtractOptions, ExtractedGlyph, PageText, TextOrigin};

/// How far off the page's x axis a direction must be before this module claims
/// it: **two degrees**, stored as its cosine because the test is a dot product
/// in a hot loop and `f32::cos` is not `const`.
///
/// `|dir.x| >= this` means "within two degrees of the page's x axis", in either
/// sense — 180° text is as horizontal as 0° text for the purpose of the
/// *census*, because its advance still moves along x.
///
/// Two degrees rather than a hair: a producer that lays out a "horizontal" line
/// by accumulating a rotation matrix can leave a few hundredths of a degree of
/// residue, and italic or slanted *faces* do not tilt the baseline at all. Two
/// degrees over a 300-point line is ten points of rise, which no producer emits
/// by accident.
///
/// ★ 180° text is nonetheless **not** left alone; it is caught by [`BACKWARDS`]
/// in the second pass, for the reason given there. The literal is checked
/// against `cos(2°)` by `tests::the_angle_constants_match_their_cosines`,
/// because a hand-typed cosine that drifts from the degrees in this comment
/// would mis-group silently rather than fail.
const HORIZONTAL_COS: f32 = 0.999_390_8; // cos(2°)

/// Two directions are the same direction when their unit vectors agree to
/// within this cosine — **one degree**, stored the same way and for the same
/// reason as [`HORIZONTAL_COS`], and checked by the same test.
///
/// It governs *recognition* only, never geometry: the vector actually used to
/// build a box is always the one measured from that line's own glyphs.
const SAME_DIR_COS: f32 = 0.999_847_7; // cos(1°)

/// A displacement shorter than this many user-space units carries no reliable
/// direction, so it is not measured.
///
/// Not a fraction of the font size: the quantity being guarded is `f32`
/// cancellation in `b.x − a.x` at page coordinates, which is an absolute
/// property of the numbers and not of the type size.
const MIN_SPAN: f32 = 1e-3;

/// ★ **The one predicate this whole module is built on**: would the extraction
/// have called these two glyphs adjacent, had it been measuring along their own
/// direction rather than along the page's x axis?
///
/// `d` is the displacement from the previous glyph's origin to this one's,
/// `dir` the unit direction to measure it in, `advance` the previous glyph's
/// advance and `size` the larger of the two effective font sizes.
///
/// # It is `layout::classify`, with the axis restored
///
/// The engine's three clauses, in the engine's order, in the line's frame:
///
/// | `pdfce-core`, in page axes | here |
/// |---|---|
/// | `\|Δy\| > line_gap_ratio × size` → new line | `\|perp\| > line_gap_ratio × size` |
/// | `Δx − advance < −backward_jump_ratio × size` → new line | `along − advance < −backward_jump_ratio × size` |
/// | `Δx − advance > word_gap_ratio × size` → word space | `along − advance > word_gap_ratio × size` |
///
/// Substituting `dir = (1, 0)` gives `along = Δx` and `perp = Δy` and this
/// becomes `classify` line for line, which is the argument that it is a
/// generalisation and not a second opinion.
///
/// # ★★ Why the WORD clause is treated as a break here, when the engine keeps
/// the run open through it
///
/// This is the deliberate difference, and it is the guard against the only way
/// this module could damage an ordinary page.
///
/// The engine tests the **baseline** first, so a gap it calls a word gap has
/// already been proved to be on the same baseline — in page axes. Rotate the
/// frame and that proof goes with it: two short lines *stacked* along the very
/// direction the census admitted (a centred one-word-per-line block on a page
/// that also carries a vertical stamp) have `perp = 0` between them, because
/// their stacking direction and the measured writing direction are the same
/// vector. The baseline clause cannot separate them; only the size of the gap
/// can.
///
/// So a rotated line here is the strictest thing it can be — **a maximal chain
/// of glyphs the extraction would have called adjacent**, `Break::None` and
/// nothing weaker. What that costs is a rotated line whose words are separated
/// by a *gap* rather than by a real space glyph: it bands as two, and its
/// derived break survives. What it buys is that no page can have two genuinely
/// separate lines merged into one band or one clipboard line.
///
/// The cost is small and was measured rather than assumed: on `SW41177.pdf`
/// page 36 the vertical stamp is 82 glyphs and **every one of its 81 steps
/// passes**, worst gap 0.010 pt against a 1.600 pt threshold, because its words
/// are separated by real space glyphs as almost all real text is.
pub fn adjacent(
    d: (f32, f32),
    dir: (f32, f32),
    advance: f32,
    size: f32,
    opts: &ExtractOptions,
) -> bool {
    let along = d.0 * dir.0 + d.1 * dir.1;
    let perp = d.0.mul_add(-dir.1, d.1 * dir.0);
    let gap = along - advance;
    perp.abs() <= opts.line_gap_ratio * size
        && gap >= -opts.backward_jump_ratio * size
        && gap <= opts.word_gap_ratio * size
}

/// One line of text that does not run along the page's x axis.
#[derive(Debug, Clone)]
pub struct Line {
    /// The unit writing direction in **PDF user space**, measured from this
    /// line's own glyph origins.
    ///
    /// Not snapped to an axis even when it is within a whisker of one: the
    /// measurement is what the file says, and rounding it would move the box a
    /// caller draws from it. Snapping belongs to presentation — see
    /// `canvas::cursor`, which quantises the *cursor* angle and nothing else.
    pub dir: (f32, f32),
    /// The line's glyphs, in page content order, never fewer than two.
    ///
    /// A single glyph is not a line: it has no measurable direction of its own
    /// and nothing to be banded with, so it is left to the engine's grouping
    /// exactly as before.
    pub glyphs: Vec<GlyphRef>,
}

/// Everything this module knows about one page's rotated text.
///
/// Empty — and cheap — for every page whose text runs along x, which is nearly
/// all of them. See §2.3.
#[derive(Debug, Clone, Default)]
pub struct Rotated {
    /// The rotated lines, in the order their first glyph appears.
    pub lines: Vec<Line>,
    /// Which line each rotated glyph belongs to, as an index into
    /// [`Self::lines`].
    line_of: HashMap<(usize, usize), usize>,
    /// Which `DerivedLineBreak` runs fall *inside* a rotated line.
    ///
    /// A set rather than a map from run to replacement text, because
    /// [`adjacent`] admits only the `Break::None` case: a break this module
    /// claims is one the extraction should never have emitted, so it copies as
    /// nothing at all. A break it does not claim is left entirely alone.
    breaks: HashSet<usize>,
}

impl Rotated {
    /// Whether this page has any rotated text at all.
    ///
    /// The one question `textsel::resolve` asks before doing anything
    /// differently, so that the horizontal path is not merely equivalent but
    /// *unreached*.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Which rotated line a glyph belongs to, if any.
    #[must_use]
    pub fn line_of(&self, gref: GlyphRef) -> Option<usize> {
        self.line_of.get(&(gref.run, gref.glyph)).copied()
    }

    /// **Whether the derived line break at `run` is an artefact** — i.e. falls
    /// between two glyphs this module found to be adjacent along their own
    /// writing direction.
    ///
    /// `false` means "not ours": a real line break, a break between two
    /// horizontal lines, or a gap large enough that [`adjacent`] declined to
    /// judge it. In every one of those the caller copies the run unchanged,
    /// which is what makes today's behaviour the floor.
    #[must_use]
    pub fn is_artefact(&self, run: usize) -> bool {
        self.breaks.contains(&run)
    }

    /// **The writing direction at a page-space point**, for the cursor.
    ///
    /// Returns the unit direction of the rotated line whose band contains the
    /// point, or `None` when the point is not over rotated text — including
    /// when it is over ordinary horizontal text, which needs no answer because
    /// the default I-beam is already right for it.
    ///
    /// # Why containment and not the engine's hit test
    ///
    /// `EditableTextModel::hit_test` deliberately falls back to the nearest
    /// line when no box contains the point, which is correct for a *drag* —
    /// Acrobat starts a sweep from the nearest text — and wrong for a *cursor*,
    /// which must say "text here" only where there is text. Reusing it would
    /// turn the I-beam sideways over blank paper anywhere below the stamp.
    ///
    /// The band is the union of the line's glyph cells in the line's own frame,
    /// so this is an exact test for text at any angle, not a bounding-box
    /// approximation.
    #[must_use]
    pub fn direction_at(&self, page: &PageText, x: f32, y: f32) -> Option<(f32, f32)> {
        self.lines
            .iter()
            .find(|line| line.contains(page, x, y))
            .map(|line| line.dir)
    }

    /// ★★ **Where a page-space point lands in the text**, when it lands on a
    /// rotated line.
    ///
    /// `None` when the point is not inside any rotated band — including when it
    /// is over ordinary horizontal text — and the caller then asks
    /// `EditableTextModel::hit_test`, whose answer is correct for everything
    /// this module does not claim.
    ///
    /// # ★★★ Why this exists, and why finding out cost two failing tests
    ///
    /// The first version of §8 fixed the *grouping* and left hit-testing to the
    /// engine, on the reasoning that `hit_test` falls back to the nearest line
    /// and would therefore land somewhere sensible. It does land somewhere
    /// sensible. It lands on **the wrong side of the glyph**.
    ///
    /// `EditableTextModel`'s line boxes are built the same way every other box
    /// in the extraction is: `x … x + advance` across, `y − ¼size … y + ¾size`
    /// up. For a 90° glyph at `(100, 300)` that box occupies `x ∈ 100..108.7`
    /// while the ink occupies roughly `x ∈ 91..103` — the two overlap by a
    /// third and the box is hung off the wrong corner. A press on the middle of
    /// the letter is therefore *outside* every line box, the nearest-line
    /// fallback fires, and the caret snaps to whichever edge the fallback likes.
    ///
    /// Measured, not reasoned: a sweep down the fixture's `UPWARD` produced
    /// `UPWAR`, one letter short, and a sweep along `INVERTED` produced nothing
    /// at all because both ends collapsed onto the same caret slot. Both are
    /// exactly what the operator would have hit.
    ///
    /// # The caret rule
    ///
    /// The point is projected into the line's frame and matched against the
    /// glyph cells' `along` spans. Inside a cell, the caret goes to whichever
    /// **half** the point is in — before the glyph or after it — which is the
    /// convention every text application shares and is what makes a sweep that
    /// ends on the last letter include it. Before the first cell or past the
    /// last, it clamps to that end.
    ///
    /// Containment is tested against the band, **not** an infinite strip. A
    /// strip would extend the vertical stamp's 8-point column the full height
    /// of the sheet and steal every press in it, which on a title block is a
    /// lot of presses on other things.
    #[must_use]
    pub fn position_at(&self, page: &PageText, x: f32, y: f32) -> Option<TextPosition> {
        let line = self.lines.iter().find(|l| l.contains(page, x, y))?;
        let origin = line.origin(page)?;
        let d = (x - origin.0, y - origin.1);
        let along = d.0 * line.dir.0 + d.1 * line.dir.1;

        let mut last: Option<(GlyphRef, &ExtractedGlyph)> = None;
        for gref in &line.glyphs {
            let Some(g) = glyph(page, *gref) else {
                continue;
            };
            // Each cell is measured from the line's origin along its own
            // direction, not from the previous cell's end: a `TJ` adjustment
            // moves a glyph without changing the previous glyph's advance, and
            // accumulating advances would drift by the whole of it.
            let start = (g.x - origin.0).mul_add(line.dir.0, (g.y - origin.1) * line.dir.1);
            let end = start + g.advance;
            if along < start {
                // Before this cell: the caret belongs at its start, which is
                // also the answer for a point before the whole line.
                return Some(TextPosition::new(gref.run, g.text_start as usize));
            }
            if along <= end {
                let after = along > start + g.advance * 0.5;
                return Some(caret(*gref, g, after));
            }
            last = Some((*gref, g));
        }
        // Past the last cell.
        let (gref, g) = last?;
        Some(caret(gref, g, true))
    }
}

/// The caret slot before or after one glyph, as a [`TextPosition`].
///
/// `TextPosition` is `(run index, byte offset into that run's text)`, and a
/// glyph knows its own byte window — so "after" is the start plus the length,
/// which is the next glyph's start whenever there is one and the run's end
/// otherwise. Written once here rather than at the three call sites above,
/// because getting `text_len` wrong is silent: one code may decode to several
/// characters (§9.10.3's `ffl`), so a caret built by adding one would land
/// inside a character and slice nothing.
fn caret(gref: GlyphRef, g: &ExtractedGlyph, after: bool) -> TextPosition {
    let at = g.text_start as usize + if after { g.text_len as usize } else { 0 };
    TextPosition::new(gref.run, at)
}

impl Line {
    /// Whether `(x, y)` in PDF user space falls inside this line's band.
    ///
    /// Projected into the line's own frame and tested against the extremes of
    /// its glyph cells — `along` from the first glyph's origin to the last
    /// glyph's origin plus its advance, `perp` across the ascender/descender
    /// span. Exact at any angle, because the frame rotates with the text.
    fn contains(&self, page: &PageText, x: f32, y: f32) -> bool {
        let Some(extent) = self.extent(page) else {
            return false;
        };
        let Some(origin) = self.origin(page) else {
            return false;
        };
        let d = (x - origin.0, y - origin.1);
        let along = d.0 * self.dir.0 + d.1 * self.dir.1;
        let perp = d.0.mul_add(-self.dir.1, d.1 * self.dir.0);
        along >= extent.along.0
            && along <= extent.along.1
            && perp >= extent.perp.0
            && perp <= extent.perp.1
    }

    /// The first glyph's origin — the line frame's origin.
    fn origin(&self, page: &PageText) -> Option<(f32, f32)> {
        let g = glyph(page, *self.glyphs.first()?)?;
        Some((g.x, g.y))
    }

    /// The line's extremes in its own frame: how far its cells reach along the
    /// writing direction and across it.
    ///
    /// Public geometry lives in `textsel::resolve`, which builds the quad a
    /// selection paints and marks with. This is the same accumulation done for
    /// the *whole* line rather than for the selected part of it, and it exists
    /// for one caller — [`Line::contains`] — because a cursor asks about the
    /// line, not about the selection.
    fn extent(&self, page: &PageText) -> Option<Extent> {
        let origin = self.origin(page)?;
        let mut extent: Option<Extent> = None;
        for gref in &self.glyphs {
            let Some(g) = glyph(page, *gref) else {
                continue;
            };
            let d = (g.x - origin.0, g.y - origin.1);
            let along = d.0 * self.dir.0 + d.1 * self.dir.1;
            let perp = d.0.mul_add(-self.dir.1, d.1 * self.dir.0);
            let cell = Extent {
                along: (along, along + g.advance),
                perp: (
                    perp - g.size * super::GLYPH_DESCENT,
                    perp + g.size * super::GLYPH_ASCENT,
                ),
            };
            extent = Some(match extent {
                None => cell,
                Some(e) => Extent {
                    along: (e.along.0.min(cell.along.0), e.along.1.max(cell.along.1)),
                    perp: (e.perp.0.min(cell.perp.0), e.perp.1.max(cell.perp.1)),
                },
            });
        }
        extent
    }
}

/// A span in a line's own frame: along the writing direction, and across it.
#[derive(Debug, Clone, Copy)]
struct Extent {
    /// Minimum and maximum displacement along the writing direction.
    along: (f32, f32),
    /// Minimum and maximum displacement across it — descender to ascender.
    perp: (f32, f32),
}

/// Fetch a glyph by reference, or nothing if the reference is stale.
fn glyph(page: &PageText, gref: GlyphRef) -> Option<&ExtractedGlyph> {
    page.runs.get(gref.run)?.glyphs.get(gref.glyph)
}

/// ★ **The page's writing directions, corroborated by chains of glyphs.**
///
/// A *step* is the displacement from one glyph's origin to the next glyph's, in
/// page content order and **through** the derived break runs the extraction
/// inserted. A step is *sound* when its length sits in the window the engine's
/// own `classify` would have accepted along that direction — that is,
/// `len − previous advance` inside `[−backward_jump_ratio × size,
/// word_gap_ratio × size]`. A *chain* is a maximal sequence of consecutive
/// sound steps that all point the same way.
///
/// **A chain of at least [`MIN_CHAIN_STEPS`] steps in a non-horizontal
/// direction is what admits that direction**, and its overall displacement —
/// first origin to last, the longest baseline available — is the vector
/// published.
///
/// # ★★ Why a chain, and why that bar cannot be cleared by accident
///
/// The first draft measured a direction from *runs*, on the sound argument that
/// the engine closes a run before it emits any derived whitespace, so a
/// two-glyph run is a pair it certified as adjacent. That argument is correct
/// and the method was **useless**: on `SW41177.pdf` page 36 only **10 of the 72
/// runs** in the vertical stamp hold two glyphs, and they do so only because
/// `:`, `i`, `n`, `c` and the space are narrow enough that one advance stays
/// inside `line_gap_ratio × size`. A vertical label made of wide letters —
/// `PART NO`, the exact thing a title block carries — would have produced no
/// two-glyph run at all and the feature would have silently done nothing.
///
/// The chain replaces that certainty with corroboration, and the bar is set
/// where horizontal text cannot reach it:
///
/// * **Within** a horizontal line every step is horizontal, so every step is
///   excluded before it is counted.
/// * The only non-horizontal steps an ordinary page produces are the jumps
///   *between* lines — and those are, by construction, separated from each
///   other by a whole line of horizontal steps. **They can never be
///   consecutive**, so they can never form a chain of two, let alone
///   [`MIN_CHAIN_STEPS`].
///
/// So the false positive this could theoretically suffer requires several
/// glyphs in a row, each exactly one advance from the last, along a common
/// direction off the page's x axis. That is not a coincidence available to
/// horizontal text — it is a line of rotated text, which is the thing being
/// looked for.
///
/// # Cost
///
/// One pass over the page's glyphs, and the common case exits each step after
/// two multiplications and a comparison (the horizontal test). Measured on the
/// benchmark drawing rather than assumed — see this module's tests.
///
/// Returns a deduplicated list, strongest evidence first, so a page with one
/// dominant rotated direction and a stray outlier matches the dominant one
/// first.
#[must_use]
pub fn census(page: &PageText, opts: &ExtractOptions) -> Vec<(f32, f32)> {
    // (direction, best chained span) — the span is the evidence weight, and a
    // longer chain is better evidence for the same reason a longer baseline is:
    // `f32` cancellation in the subtraction is a fixed absolute error, so
    // dividing it by a longer span makes the direction more exact.
    let mut found: Vec<((f32, f32), f32)> = Vec::new();
    let mut chain: Option<Chain> = None;
    let mut previous: Option<(f32, f32, f32, f32)> = None;

    for (_, g) in glyphs(page) {
        let Some(last) = previous.replace((g.x, g.y, g.advance, g.size)) else {
            continue;
        };
        let d = (g.x - last.0, g.y - last.1);
        let len = d.0.hypot(d.1);
        if !len.is_finite() || len < MIN_SPAN {
            chain = None;
            continue;
        }
        let unit = (d.0 / len, d.1 / len);
        // The horizontal exit, first and cheapest: nearly every step on nearly
        // every page lands here.
        if unit.0.abs() >= HORIZONTAL_COS {
            chain = None;
            continue;
        }
        // The same predicate pass two applies, in the step's own direction —
        // where `perp` is zero by construction, since the direction IS the
        // step.
        let size = last.3.max(g.size).max(1e-6);
        if !adjacent(d, unit, last.2, size, opts) {
            chain = None;
            continue;
        }
        match chain.as_mut() {
            Some(open) if open.dir.0 * unit.0 + open.dir.1 * unit.1 >= SAME_DIR_COS => {
                open.steps += 1;
                open.end = (g.x, g.y);
            }
            _ => {
                chain = Some(Chain {
                    dir: unit,
                    steps: 1,
                    start: (last.0, last.1),
                    end: (g.x, g.y),
                });
            }
        }
        // Published as soon as the bar is cleared and re-published as the chain
        // grows, so a chain still being extended when the page ends is not
        // lost. Re-publication is idempotent on the direction and takes the
        // MAXIMUM span rather than a sum, which is what keeps one long chain
        // worth exactly one long chain.
        if let Some(open) = chain.as_ref()
            && open.steps >= MIN_CHAIN_STEPS
        {
            let span = (open.end.0 - open.start.0, open.end.1 - open.start.1);
            let total = span.0.hypot(span.1);
            if total >= MIN_SPAN {
                let refined = (span.0 / total, span.1 / total);
                match found
                    .iter_mut()
                    .find(|(k, _)| k.0 * refined.0 + k.1 * refined.1 >= SAME_DIR_COS)
                {
                    Some((dir, weight)) => {
                        if total > *weight {
                            *dir = refined;
                            *weight = total;
                        }
                    }
                    None => found.push((refined, total)),
                }
            }
        }
    }

    found.sort_by(|a, b| b.1.total_cmp(&a.1));
    found.into_iter().map(|(dir, _)| dir).collect()
}

/// How many consecutive sound steps must agree before their direction is
/// admitted.
///
/// Three steps means four glyphs. Two would already be unreachable by
/// horizontal text (see [`census`]), and three is taken instead because the
/// cost of the extra glyph is nil — a rotated label worth banding has more than
/// four letters — while the margin against a pathological producer is a whole
/// order of coincidence.
const MIN_CHAIN_STEPS: u32 = 3;

/// A run of consecutive same-direction steps, while it is being extended.
struct Chain {
    /// The direction of its first step, which every later step must match.
    /// Kept rather than re-averaged so the chain cannot drift round a curve one
    /// tolerance at a time.
    dir: (f32, f32),
    /// How many steps it holds.
    steps: u32,
    /// The origin of its first glyph — one end of the measuring baseline.
    start: (f32, f32),
    /// The origin of its latest glyph — the other end.
    end: (f32, f32),
}

/// Every glyph on the page, in content order, paired with its reference.
///
/// **Through** the derived whitespace runs rather than around them: those runs
/// are the extraction's verdict about where lines end, and this module exists
/// to re-judge that verdict, so treating them as boundaries would be assuming
/// the answer.
fn glyphs(page: &PageText) -> impl Iterator<Item = (GlyphRef, &ExtractedGlyph)> {
    page.runs.iter().enumerate().flat_map(|(run, r)| {
        r.glyphs
            .iter()
            .enumerate()
            .map(move |(glyph, g)| (GlyphRef::new(run, glyph), g))
    })
}

/// A direction that a *single* glyph may adopt from its neighbour: one the page
/// census already vouches for, or the reverse of the page's x axis.
///
/// # Why 180° text needs a name of its own
///
/// `classify` breaks a line on a backward jump, and `advance` is published as a
/// positive magnitude, so text advancing in `−x` produces `Δx − advance ≈
/// −2·advance` at every glyph — a backward jump, every time, and therefore a
/// derived line break between every letter. Its symptom is identical to the
/// vertical case and its cause is a different clause.
///
/// But its direction is *horizontal*, so [`census`] refuses it by design (a
/// census that admitted `(−1, 0)` would also admit every ordinary line, since
/// the two are one dot product apart). It is therefore offered separately, and
/// only ever as a candidate to be confirmed by the same gap test every other
/// direction passes.
const BACKWARDS: (f32, f32) = (-1.0, 0.0);

/// ★ **Regroup a page's glyphs into lines along their own writing direction.**
///
/// The second pass. See §3 for the thresholds and why they are the engine's own
/// with the axis restored.
///
/// `opts` must be the **same** `ExtractOptions` that produced `page` — in this
/// application, `app::settings::SettingsExt::extract_options`, never
/// `ExtractOptions::default()`. Passing a different one is not a crash, it is
/// the quiet class of defect where the engine derived a word space and this
/// derived a word break, or the reverse.
#[must_use]
pub fn lines(page: &PageText, opts: &ExtractOptions) -> Rotated {
    let directions = census(page, opts);
    if directions.is_empty() {
        // ★ The horizontal fast path, and the regression guard. See §2.3: a
        // page with no rotated text cannot reach the walk below, so it cannot
        // be changed by it.
        return Rotated::default();
    }

    let mut state = Walk {
        opts,
        directions,
        lines: Vec::new(),
        line_of: HashMap::new(),
        breaks: HashSet::new(),
        open: None,
        pending_break: None,
    };
    for (run_index, run) in page.runs.iter().enumerate() {
        // A derived break run carries no glyphs; it is *evidence to be
        // re-judged*, so its index is remembered and the walk continues through
        // it rather than treating it as a boundary. An `/ActualText` run has no
        // glyph geometry at all (§14.9.4 makes the mapping impossible, not
        // merely unimplemented), so it closes the open line: there is nothing
        // to measure a direction across it with.
        match run.origin {
            TextOrigin::DerivedLineBreak => state.pending_break = Some(run_index),
            TextOrigin::DerivedWordSpace => state.pending_break = None,
            TextOrigin::ActualText => {
                state.close();
                state.pending_break = None;
            }
            TextOrigin::Glyphs => {
                for (glyph_index, g) in run.glyphs.iter().enumerate() {
                    state.push(GlyphRef::new(run_index, glyph_index), g);
                }
            }
            // `TextOrigin` is `#[non_exhaustive]`, so a future origin lands
            // here. It is treated as `ActualText` — the conservative reading —
            // because the two things this walk needs from a run are its glyph
            // geometry and the knowledge that a break is derived, and an origin
            // this build has never heard of supplies neither. Closing the line
            // costs at worst a band that could have been one; assuming it was a
            // derived break would DELETE a separator that may be real.
            _ => {
                state.close();
                state.pending_break = None;
            }
        }
    }
    state.finish()
}

/// The in-progress regrouping. A struct rather than a pile of `let mut`s
/// because [`Walk::push`] is the whole of §3 and wants to be readable on its
/// own.
struct Walk<'a> {
    /// The caller's extraction options — the three ratios, and nothing else.
    opts: &'a ExtractOptions,
    /// The page's certified directions, best evidence first.
    directions: Vec<(f32, f32)>,
    /// Lines closed so far.
    lines: Vec<Line>,
    /// Glyph → line index, for the lines closed so far and the open one.
    line_of: HashMap<(usize, usize), usize>,
    /// Internal derived breaks and what they copy as.
    breaks: HashSet<usize>,
    /// The line being built.
    open: Option<Open>,
    /// The index of the most recent `DerivedLineBreak` run not yet resolved —
    /// set when the walk passes one, consumed by the next glyph that decides
    /// whether it was real.
    pending_break: Option<usize>,
}

/// The line [`Walk`] is currently extending.
struct Open {
    /// Its direction, once known. `None` while the line is a single glyph whose
    /// census could not vouch for a direction yet — see §2.2.
    dir: Option<(f32, f32)>,
    /// Its glyphs so far.
    glyphs: Vec<GlyphRef>,
    /// The previous glyph's origin, advance and size — everything the gap test
    /// needs, kept rather than re-fetched so the walk never indexes back into
    /// the page.
    last: (f32, f32, f32, f32),
    /// Derived break runs seen inside this line. Held on the line rather than
    /// published immediately because a line of fewer than two glyphs is
    /// discarded, and its breaks must be discarded with it or a copy would lose
    /// a newline that was real.
    breaks: Vec<usize>,
}

impl Walk<'_> {
    /// Add one glyph to the open line, or start a new one.
    ///
    /// The facts about the previous glyph are **copied out of the open line
    /// before anything else happens**, and the line is not borrowed again until
    /// the decision is made. That is not style: [`Walk::candidate`] reads the
    /// page census, which lives on the same `self`, so holding a mutable borrow
    /// of `self.open` across it does not compile — and the shape that does
    /// compile is also the clearer one, because the whole of §3's arithmetic
    /// then reads as plain numbers with no indirection in it.
    fn push(&mut self, gref: GlyphRef, g: &ExtractedGlyph) {
        let pending = self.pending_break.take();
        let Some((last, known)) = self.open.as_ref().map(|o| (o.last, o.dir)) else {
            self.open = Some(Open {
                dir: None,
                glyphs: vec![gref],
                last: (g.x, g.y, g.advance, g.size),
                breaks: Vec::new(),
            });
            return;
        };

        let d = (g.x - last.0, g.y - last.1);
        let size = last.3.max(g.size).max(1e-6);
        if !d.0.is_finite() || !d.1.is_finite() || !size.is_finite() {
            self.restart(gref, g);
            return;
        }

        // The direction to test in. An open line that already has one keeps it;
        // one that does not takes the best candidate this displacement matches,
        // which is what lets a single-glyph run join the line its neighbours
        // established (§2.2).
        let Some(dir) = known.or_else(|| self.candidate(d)) else {
            self.restart(gref, g);
            return;
        };

        // §3, in one predicate: [`adjacent`], which is `classify` with the axis
        // restored and the word clause tightened for the reason given there.
        if !adjacent(d, dir, last.2, size, self.opts) {
            self.restart(gref, g);
            return;
        }

        // The glyph stays on this line, so a derived break the walk passed on
        // the way here is an artefact of the axis-aligned baseline test and
        // copies as nothing.
        let Some(open) = self.open.as_mut() else {
            return;
        };
        if let Some(run) = pending {
            open.breaks.push(run);
        }
        open.dir = Some(dir);
        open.glyphs.push(gref);
        open.last = (g.x, g.y, g.advance, g.size);
    }

    /// Close the open line and begin a new one at `gref`.
    ///
    /// The three ways [`Walk::push`] can decide the glyph is not on this line
    /// all end here, and they end here rather than recursing so that the
    /// restart cannot itself decide to restart. `pending` is dropped at each call site
    /// deliberately: a derived break the walk did **not** claim is a
    /// break it leaves alone, and leaving it alone means saying nothing about
    /// it — [`Rotated::is_artefact`] answers `false` and the copy takes the run's
    /// own newline unchanged.
    fn restart(&mut self, gref: GlyphRef, g: &ExtractedGlyph) {
        self.close();
        self.open = Some(Open {
            dir: None,
            glyphs: vec![gref],
            last: (g.x, g.y, g.advance, g.size),
            breaks: Vec::new(),
        });
    }

    /// Which of the page's certified directions this displacement matches, if
    /// any.
    ///
    /// The displacement is normalised and matched against the census within
    /// [`SAME_DIR_COS`]; the direction **returned is the census's**, not the
    /// measured one, because the census was measured over a longer span and is
    /// therefore the more exact of the two (see [`census`]).
    ///
    /// [`BACKWARDS`] is offered last and unconditionally, for the 180° case the
    /// census cannot carry — see its own documentation.
    fn candidate(&self, d: (f32, f32)) -> Option<(f32, f32)> {
        let len = d.0.hypot(d.1);
        if !len.is_finite() || len < MIN_SPAN {
            return None;
        }
        let unit = (d.0 / len, d.1 / len);
        self.directions
            .iter()
            .copied()
            .chain(std::iter::once(BACKWARDS))
            .find(|k| k.0 * unit.0 + k.1 * unit.1 >= SAME_DIR_COS)
    }

    /// Close the open line, keeping it only if it is one.
    ///
    /// A line of one glyph is dropped **with its breaks**: the glyph goes back
    /// to the engine's grouping, and any derived break beside it stays a
    /// newline. Publishing the break without the line would delete a separator
    /// that no longer has a band to justify it.
    fn close(&mut self) {
        let Some(open) = self.open.take() else {
            return;
        };
        let Some(dir) = open.dir else {
            return;
        };
        if open.glyphs.len() < 2 {
            return;
        }
        let index = self.lines.len();
        for gref in &open.glyphs {
            self.line_of.insert((gref.run, gref.glyph), index);
        }
        for run in open.breaks {
            self.breaks.insert(run);
        }
        self.lines.push(Line {
            dir,
            glyphs: open.glyphs,
        });
    }

    /// Close the last line and hand back what was found.
    fn finish(mut self) -> Rotated {
        self.close();
        Rotated {
            lines: self.lines,
            line_of: self.line_of,
            breaks: self.breaks,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfce_core::document::Document;
    use pdfce_core::edit::EditSession;

    /// The fixture's page, extracted exactly as the application extracts.
    ///
    /// `ExtractOptions::default()` here rather than the settings funnel because
    /// there is no `Settings` in a unit test and the three ratios this module
    /// reads are not among the settings the funnel overrides — but the funnel
    /// is still what production passes, which is why [`super::lines`] takes the
    /// options rather than reaching for a default of its own.
    fn page(rel: &str) -> PageText {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures")
            .join(rel);
        assert!(
            path.exists(),
            "the fixture is missing at {}",
            path.display()
        );
        let doc = Document::load(&path).expect("the fixture loads");
        let session = EditSession::new(doc);
        let view = session.view();
        let pages = pdfce_core::page_tree::pages_in(&view).expect("a page tree");
        pdfce_core::text_extract::extract_page_view(&view, &pages[0], 0, &ExtractOptions::default())
            .expect("the page's text extracts")
    }

    /// The rotated-text fixture — see [`super::super::fixture`].
    fn rotated_fixture() -> PageText {
        page("rotated-text.pdf")
    }

    /// The characters of one rotated line, in order.
    fn text_of(page: &PageText, line: &Line) -> String {
        line.glyphs
            .iter()
            .filter_map(|g| {
                let run = page.runs.get(g.run)?;
                let glyph = run.glyphs.get(g.glyph)?;
                let lo = glyph.text_start as usize;
                let hi = lo + glyph.text_len as usize;
                run.text.get(lo..hi)
            })
            .collect()
    }

    /// The angle of a unit vector, in degrees, in `-180..=180`.
    fn degrees(dir: (f32, f32)) -> f32 {
        dir.1.atan2(dir.0).to_degrees()
    }

    // =======================================================================
    // The hand-computed constants
    // =======================================================================

    /// ★ The two cosines are the cosines they claim to be.
    ///
    /// They are written as decimal literals because they gate a hot loop and
    /// `f32::cos` is not `const`. A hand-typed cosine is exactly the kind of
    /// number that drifts when someone edits the degrees in the doc comment
    /// above it and not the literal below it, and every consequence of that
    /// would be a *silent* mis-grouping rather than a failure.
    #[test]
    fn the_angle_constants_match_their_cosines() {
        // The angles as the doc comments state them. Written here rather than
        // beside the cosines because a constant used only by its own test is
        // dead code in a release build, and an `allow` would have hidden the
        // next genuinely dead constant too.
        for (name, cosine, degrees) in [
            ("HORIZONTAL_COS", HORIZONTAL_COS, 2.0_f32),
            ("SAME_DIR_COS", SAME_DIR_COS, 1.0_f32),
        ] {
            assert!(
                (cosine - degrees.to_radians().cos()).abs() < 1e-6,
                "{name} is {cosine}, which is not cos({degrees}°) = {}",
                degrees.to_radians().cos()
            );
        }
    }

    // =======================================================================
    // §2.1 — the census
    // =======================================================================

    /// ★★★ **The census finds the three rotated directions and neither
    /// horizontal one.**
    ///
    /// This is the test the first draft of this module would have failed. Every
    /// rotated string in the fixture is set in wide capitals, so no run holds
    /// two glyphs and the run-based census would have come back empty — see the
    /// module header §2.1 for why that draft was correct and useless.
    ///
    /// The negative half is the load-bearing one: `HORIZONTAL` must contribute
    /// nothing, or every ordinary page would be regrouped by this module
    /// instead of left alone.
    #[test]
    fn the_census_admits_only_the_rotated_directions() {
        let page = rotated_fixture();
        let found = census(&page, &ExtractOptions::default());
        let angles: Vec<i32> = found.iter().map(|d| degrees(*d).round() as i32).collect();
        assert!(
            angles.contains(&90),
            "the 90° string was not found: {angles:?}"
        );
        assert!(
            angles.contains(&-90),
            "the 270° string was not found: {angles:?}"
        );
        assert!(
            angles.contains(&30),
            "the 30° string was not found: {angles:?}"
        );
        assert!(
            !angles.iter().any(|a| a.abs() <= 2),
            "a horizontal direction reached the census: {angles:?}"
        );
        assert!(
            !angles.iter().any(|a| a.abs() >= 178),
            "the 180° string reached the census, which only BACKWARDS may carry: {angles:?}"
        );
    }

    /// ★★ **An ordinary page produces nothing at all**, which is the whole
    /// safety argument.
    ///
    /// `a1-titleblock.pdf` is a real drawing sheet with sixteen text runs and no
    /// rotated text. If this ever returns a line, every selection on every
    /// ordinary document is going through the rotated path — and the failure
    /// would be invisible until a highlight landed in the wrong place.
    #[test]
    fn a_page_with_no_rotated_text_is_left_entirely_alone() {
        let page = page("a1-titleblock.pdf");
        assert!(
            census(&page, &ExtractOptions::default()).is_empty(),
            "the census found a rotated direction on an ordinary drawing sheet"
        );
        assert!(
            lines(&page, &ExtractOptions::default()).is_empty(),
            "an ordinary drawing sheet was regrouped"
        );
    }

    // =======================================================================
    // §2.2 — the grouping
    // =======================================================================

    /// ★★★ **Each rotated string comes back as ONE line, whole.**
    ///
    /// The operator's *"it pastes each letter onto its own line"* stated as an
    /// assertion: the extraction gives six to eight one-glyph runs per string,
    /// and this must give one line carrying all of them, in order.
    ///
    /// All four quadrant cases are here because 90° and 270° reach the
    /// extraction's *baseline* clause while 180° reaches its *backward-jump*
    /// clause — the same symptom by two different routes, and a fix that
    /// handled only the first would look complete on the operator's own file.
    #[test]
    fn every_rotated_string_is_one_line() {
        let page = rotated_fixture();
        let rotated = lines(&page, &ExtractOptions::default());
        let mut seen: Vec<(String, i32)> = rotated
            .lines
            .iter()
            .map(|l| (text_of(&page, l), degrees(l.dir).round() as i32))
            .collect();
        seen.sort();
        assert_eq!(
            seen,
            vec![
                ("DOWNWARD".to_owned(), -90),
                ("INVERTED".to_owned(), 180),
                ("SKEWED".to_owned(), 30),
                ("UPWARD".to_owned(), 90),
            ],
            "the four rotated strings did not each come back as one whole line"
        );
    }

    /// ★★ **The horizontal string is not among them**, and that is asserted
    /// separately from the list above so it cannot be lost in a diff.
    ///
    /// `HORIZONTAL` extracts as one clean run with ten glyphs and the engine's
    /// grouping of it is already right. A rotated line claiming it would take
    /// its boxes down the rotated path for no reason and would be the first
    /// step towards the regression this module is arranged to make impossible.
    #[test]
    fn the_horizontal_string_is_not_claimed() {
        let page = rotated_fixture();
        let rotated = lines(&page, &ExtractOptions::default());
        for (index, run) in page.runs.iter().enumerate() {
            if !run.text.starts_with("HORIZONTAL") {
                continue;
            }
            for glyph in 0..run.glyphs.len() {
                assert!(
                    rotated.line_of(GlyphRef::new(index, glyph)).is_none(),
                    "glyph {glyph} of the horizontal string was claimed by a rotated line"
                );
            }
            return;
        }
        panic!("the fixture no longer contains its horizontal string");
    }

    /// ★★ **Every break inside a rotated word is an artefact; the breaks
    /// between the words are not.**
    ///
    /// The copy half of the operator's report. The fixture holds 28 derived
    /// line breaks: four of them separate the five strings and are real, and
    /// the rest sit between the letters of a rotated word and are the
    /// extraction measuring along the wrong axis.
    ///
    /// Asserting the count *both ways* is the point. Claiming too few leaves
    /// letters on separate lines, which is the reported defect; claiming too
    /// many silently deletes a newline the operator can see on the page, which
    /// is worse because nothing about the page would look wrong.
    #[test]
    fn only_the_breaks_inside_a_rotated_word_are_removed() {
        let page = rotated_fixture();
        let rotated = lines(&page, &ExtractOptions::default());
        let breaks: Vec<usize> = page
            .runs
            .iter()
            .enumerate()
            .filter(|(_, r)| matches!(r.origin, TextOrigin::DerivedLineBreak))
            .map(|(i, _)| i)
            .collect();
        let claimed = breaks.iter().filter(|i| rotated.is_artefact(**i)).count();
        // Five strings of 10, 6, 8, 8 and 6 glyphs: 28 breaks in total, of
        // which the four between strings are real and the 24 inside the four
        // rotated words are not. `HORIZONTAL` contributes none, because its ten
        // glyphs are one unbroken run.
        assert_eq!(breaks.len(), 28, "the fixture's break count changed");
        assert_eq!(
            claimed, 24,
            "the wrong number of derived breaks was judged spurious"
        );
        // Run 1 is the break between `HORIZONTAL` and the 90° string — two
        // genuinely different lines, and the one boundary a reader is most
        // likely to assume is safe because the strings are so far apart.
        assert!(
            !rotated.is_artefact(1),
            "the break between two different strings was removed"
        );
    }

    // =======================================================================
    // The cursor's question
    // =======================================================================

    /// ★★ **`direction_at` answers over the letters and stays quiet elsewhere.**
    ///
    /// Both halves matter and the second is the one a careless implementation
    /// gets wrong: `EditableTextModel::hit_test` falls back to the *nearest*
    /// line, which is right for starting a drag and would turn the I-beam
    /// sideways over every blank inch below a vertical stamp. This uses band
    /// containment instead, so it must answer `None` a long way from the text.
    #[test]
    fn the_direction_is_reported_over_the_text_and_nowhere_else() {
        let page = rotated_fixture();
        let rotated = lines(&page, &ExtractOptions::default());
        // Two points inside the 90° string, taken from the generator's own
        // matrix (`0 1 -1 0 100 300`) rather than guessed: a guessed coordinate
        // that misses is symptom-identical to a broken containment test, and
        // this project has already filed one retracted defect on exactly that.
        let inside = rotated
            .direction_at(&page, 100.0, 310.0)
            .expect("the point sits on the 90° string");
        assert_eq!(degrees(inside).round() as i32, 90);
        assert!(
            rotated.direction_at(&page, 400.0, 310.0).is_none(),
            "blank paper to the right of the vertical string reported a direction"
        );
        assert!(
            rotated.direction_at(&page, 72.0, 700.0).is_none(),
            "the horizontal string reported a rotated direction"
        );
    }
}
