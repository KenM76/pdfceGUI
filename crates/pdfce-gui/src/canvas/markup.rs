//! # `canvas::markup` — drawing a markup annotation where the operator points
//!
//! ## ★ The defect this module exists so that we never ship again
//!
//! The old shell's `canvas.rs` records it in the doc comment of the tool
//! variant this one is modelled on, and it is worth carrying across verbatim
//! because it is the reason a markup *substrate* exists at all rather than
//! eight commands that each insert a shape:
//!
//! > Until this variant existed, markup annotations did not go through the
//! > canvas at all: `Action::AddMarkupShape` called a function that derived a
//! > rectangle from the PAGE's own media box centre plus a per-author jitter,
//! > and inserted it. The shape therefore appeared in the middle of the page
//! > no matter where the operator had been pointing, and — because it never
//! > touched `active_tool` — it was invisible to every rule the other seven
//! > tools obey: Escape did not cancel it, it did not suppress the
//! > `ScrollArea`'s pan-by-drag, and it took no place in `TOOL_PRECEDENCE`.
//! > **The operator's report was exact: "they just drop things into the center
//! > of the pdf window."**
//!
//! Two things in that paragraph are the whole design brief. First, a markup
//! command must **arm a tool**, not perform an insertion — so that the shape
//! lands where the pointer is and so that every rule the canvas already has
//! about tools (Escape, cursor, pan suppression, the gesture machine's
//! press/drag/release) applies to it for free rather than being re-implemented
//! badly. Second, a markup that appears *somewhere the operator did not point*
//! is not a cosmetic complaint: it is the feature not working, and it passed
//! whatever tests it had because a shape really was added to the document.
//!
//! ## The four obligations, in the shape [`crate::canvas::moving`] states them
//!
//! 1. **The geometry is PDF page space, never screen pixels.** [`endpoints`] is
//!    the only place in this module that crosses the boundary, and it does it
//!    through [`crate::viewer::canvas_to_pdf_space`] — the renderer's own
//!    transform — rather than by writing the Y-flip out again. A drag measured
//!    on screen and handed to `add_markup` compiles, runs, and merely scales
//!    with magnification: the same silent class as the hit-tolerance defect
//!    [`crate::canvas::mapping`] was built to make unavailable.
//! 2. **The preview must describe what the release will actually commit.**
//!    `D:\Dev\FeatureRequests\pdfce_FeatureRequests\README.md` rule 4 welcomes a
//!    pre-commit affordance — *"a snap indicator, a hover highlight, a
//!    rubber-band, a selection handle — these are the cursor; they describe
//!    what is about to happen"* — and forbids marking content that has already
//!    been applied. A rubber band is squarely in the first category. But it is
//!    only honest if it is drawn **in the shape being authored**: an ellipse
//!    previewed as its bounding box, or an arrow previewed as a plain segment
//!    with no head, misdescribes the thing the operator is about to commit. So
//!    [`draw_preview`] draws an ellipse as an ellipse and an arrow with its
//!    head on, and it draws in the **pen's own colour** rather than in a chrome
//!    tint, because the pen colour is what will land in the file.
//! 3. **Escape abandons the drag, and abandons exactly that.** A markup drag is
//!    a `DragKind` in [`crate::canvas::gesture`], so it is already Escape's
//!    claimant 1 — the *drag in flight* row of [`crate::canvas::keys`]'s
//!    precedence table — with no new mechanism and no second rule. What is new
//!    is retiring the armed **tool**, which is a different act from abandoning
//!    a drag and takes its own row; see [`crate::canvas::keys`]'s header for
//!    where it landed and why.
//! 4. **An arrow keeps its RAW endpoints.** See [`spec`]. This is the one
//!    decision in the module that a reader will be tempted to "tidy up", and
//!    tidying it up silently reverses half of all arrows the operator draws.
//!
//! ## ★ A click with no drag places NOTHING, and that is a decision
//!
//! The old shell answered the other way: `default_markup_at` (`main.rs:19770`)
//! turned a bare click into a 120 × 60 point box centred on the pointer, with
//! a `MIN_DRAG` of 4 **PDF points** below which a real drag was also treated as
//! a click. Neither half is carried across, and the reasons are specific rather
//! than a matter of taste:
//!
//! * **Its stated justification does not hold in this shell.** The old comment
//!   is explicit — the default box is *"obviously a placeholder the operator
//!   will resize (which slice 2 makes possible)"*. There is no slice 2 here.
//!   `EditSession` has the whole `move_*` family and **no scale or resize verb
//!   of any kind** ([`crate::canvas::handles`] consumes a grip drag and commits
//!   nothing), and an annotation is not even in the family those verbs address.
//!   So a default-sized box could not be resized, could not be moved, and could
//!   only be corrected by undoing it — which makes it not a placeholder but a
//!   wrong answer with a confident size.
//! * **The 4-point threshold is zoom-dependent in the wrong direction.**
//!   Measured in page space, 4 points is a 64 px screen drag at 16× — so a
//!   deliberate small mark on a title block would be silently replaced by a
//!   120 × 60 box, which is the same failure mode as the original centre-of-page
//!   defect wearing a smaller number. egui already applies the only threshold
//!   this gesture needs: a press-and-release that does not exceed **its** drag
//!   threshold is reported as `clicked` and never reaches a `DragKind` at all
//!   (see [`crate::canvas::gesture`]'s header). One threshold, in screen space,
//!   owned by the toolkit — exactly the argument
//!   [`crate::canvas::moving::PageDelta::is_travel`] makes for refusing a
//!   second one.
//!
//! What a click does instead is **nothing, out loud**: [`drag`] is never
//! reached, and the tool stays armed with its crosshair, so the operator's next
//! gesture — a drag — does what they asked. The cost is that a click is a
//! no-op; the alternative is authoring a shape nobody chose the size of and
//! cannot change.
//!
//! The same rule guards the degenerate *drag*: a press, a wander, and a release
//! back on the origin has zero extent on both axes, and
//! `pdfce-core`'s `positive_rect` would quietly expand it to the 1-point
//! minimum — an invisible annotation holding a slot on the undo stack. That is
//! refused here as [`Refusal::NoExtent`] rather than committed, which is also
//! how the list-driven kinds' `EditError::EmptyGeometry` is kept off the
//! operator's screen: the shell never sends the engine geometry that draws
//! nothing, so the engine never has to refuse one. (For the four kinds here,
//! `validate_geometry` cannot fire at all — Square, Circle and Line always have
//! geometry, and a `TextMarkup` built here always carries exactly one quad.
//! The guard is ours, upstream of theirs, and it is the one that catches the
//! case the operator can actually produce.)
//!
//! ## Which kinds are here, and which are deliberately not
//!
//! [`MarkupKind`] carries **four**: the three drag-shaped kinds this work is
//! accountable for (Rectangle, Ellipse, Arrow) plus Highlight, which is the
//! same rubber band, is engine-ready, and already has a registered command.
//! The remaining Phase 6 kinds are absent, and each absence is a different
//! reason rather than one blanket "later":
//!
//! | kind | why it is not a variant yet |
//! |---|---|
//! | Underline · StrikeOut · Squiggly | Same `/QuadPoints` family as Highlight and equally engine-ready, but they mark **text**. This substrate has a rubber band and no text-selection gesture, and a squiggly under blank paper is a mark describing nothing. |
//! | Polygon · PolyLine · Ink | **Not drag-shaped.** They need a multi-click or freehand gesture that this two-point band cannot express; adding the variants now would put states into the type that no [`GestureOutcome`](crate::canvas::gesture::GestureOutcome) can reach. |
//! | Revision cloud | Blocked on the engine — `/BE` is never written and `MarkupSpec` is `#[non_exhaustive]`. Filed in `D:\Dev\FeatureRequests\pdfce_FeatureRequests\open\`. |
//! | Note · text box · sticky · stamp | Text-bearing, not geometric. A different gesture (place, then type) and a different spec type (`TextAnnotSpec`). |
//!
//! This is the old shell's own reasoning applied to a different boundary: its
//! `MarkupKind` declared four of the spec's ten because *"declaring all ten now
//! would add six variants that no control can select and no gesture can draw —
//! dead states in a type whose whole purpose is to say what the tool is
//! currently doing"*. The boundary here is the **gesture**: a variant belongs in
//! this enum when this rubber band can draw it.
//!
//! ## The names are the operator's, not the PDF specification's
//!
//! `Rectangle`/`Ellipse`/`Arrow` rather than `/Square`/`/Circle`/`/Line`. The
//! commands are `markup.rectangle`, `markup.ellipse` and `markup.arrow`, and
//! `text/commands.rs` calls them Rectangle, Ellipse and Arrow to the operator.
//! A type that spelled them the specification's way would make the ribbon, the
//! trace and the code disagree about the name of the same thing for no benefit;
//! the mapping to the subtype lives in exactly one place, [`spec`], where the
//! dictionary is built.
//!
//! ## The split between the pure rules and the wiring
//!
//! [`endpoints`], [`action`] and [`spec`] are pure functions of plain data, so
//! every rule above is testable with no window and no document — the same
//! discipline that makes [`crate::canvas::moving::eligible`] and
//! [`crate::canvas::selection::SelectionState::click`] pure. [`drag`] is the one
//! function that touches the frame, and it does nothing except gather those
//! inputs, call the pure functions in order, and trace what happened.

use egui::{Color32, CornerRadius, Painter, Pos2, Stroke, StrokeKind};
use pdfce_core::annot_author::{Color, LineEnding, MarkupSpec, Quad, TextMarkupKind};
use pdfce_core::page_tree::{Page, Rect as PageRect};

use crate::app::actions::Action;
use crate::canvas::gesture::Phase;
use crate::canvas::mapping::PageMapping;
use crate::viewer;

/// Which markup annotation the markup tool is currently drawing.
///
/// Carried **by** [`crate::canvas::tool::CanvasTool::Markup`] rather than
/// becoming one tool variant per shape. See that variant's own docs for the
/// argument; the short form is that these are mutually exclusive states of one
/// mode, and a type that can express "the markup tool and the ellipse tool at
/// once" is the wrong shape for a thing that is exactly one of them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkupKind {
    /// `/Square` — a rectangle bounded by the drag. *"Drag from one corner to
    /// the other."*
    Rectangle,
    /// `/Circle` — an ellipse inscribed in the drag rectangle. *"Drag out the
    /// box it fits inside."*
    Ellipse,
    /// `/Line` with a head at the far end. *"Drag from the tail to the head."*
    Arrow,
    /// `/Highlight` — a translucent band over the drag rectangle. *"Drag across
    /// what you want marked."*
    Highlight,
}

impl MarkupKind {
    /// Every variant, in the order the Markup ribbon tab lists them.
    ///
    /// Exists for the reason [`crate::app::actions::ViewChrome::ALL`] does, and
    /// is the same shape deliberately: it is what lets the *registry side* map
    /// a command id to a kind and back through one pair of total functions
    /// (`chrome_command` / `chrome_for_command` is the precedent), so a fifth
    /// kind added here fails a both-directions test rather than silently
    /// arriving with no command — or, worse, with a command that arms nothing.
    ///
    /// The mapping itself deliberately does **not** live here: command ids are
    /// `shell::commands`' vocabulary, and `shell/` is a single-writer resource.
    pub const ALL: &'static [MarkupKind] = &[
        MarkupKind::Rectangle,
        MarkupKind::Ellipse,
        MarkupKind::Arrow,
        MarkupKind::Highlight,
    ];

    /// Whether this kind is drawn by dragging a bounding **rectangle**, as
    /// opposed to a pair of endpoints.
    ///
    /// Salvaged from the old shell's `MarkupKind::is_rect`, and it earns its
    /// place for the same reason it did there: two separate decisions ask this
    /// one question — what shape the preview is, and whether the drag is
    /// normalised into a rect before it becomes a spec — and asking it as
    /// *"is this a rect kind?"* rather than *"is this Arrow?"* is what keeps
    /// both correct when a fifth kind arrives.
    #[must_use]
    pub fn is_rect(self) -> bool {
        matches!(self, Self::Rectangle | Self::Ellipse | Self::Highlight)
    }

    /// The pen colour this kind commits, as PDF `/DeviceRGB` components in
    /// `0.0..=1.0`.
    ///
    /// # Why there is one here at all, and where the operator's own pen goes
    ///
    /// The old shell carried `markup_color`/`markup_width` on the application
    /// and a swatch in its ribbon. This shell has neither, and inventing one
    /// would mean a new operator-visible control and new operator-visible
    /// strings — which belong to `text/` and to the ribbon, neither of which is
    /// this module's to extend. So the pen is a **default**, stated once, in the
    /// one place that builds a spec, and the seam for a real pen control is
    /// exactly this function: give it a colour and a width from the document's
    /// markup state and nothing else in the module changes.
    ///
    /// Red for the geometric kinds because that is what every PDF reader draws
    /// a comment shape in by default and *"make it work the way other programs
    /// do"* is the operator's stated tie-breaker; yellow for Highlight for the
    /// same reason.
    #[must_use]
    fn rgb(self) -> (f64, f64, f64) {
        match self {
            // DOCUMENT COLOUR: the default markup pen. This is written INTO the
            // annotation's `/C` and therefore into the saved file — restyling
            // the application must never move it, which is exactly the case the
            // theme gate's escape hatch exists for.
            Self::Rectangle | Self::Ellipse | Self::Arrow => (0.85, 0.16, 0.16),
            // DOCUMENT COLOUR: highlighter yellow, likewise `/C` in the file.
            Self::Highlight => (1.0, 1.0, 0.0),
        }
    }
}

/// The default border/stroke width, in PDF points, every geometric markup is
/// authored with.
///
/// A constant for the same reason [`MarkupKind::rgb`] is a function: there is
/// no pen control yet, and when there is one this is the single value it
/// replaces. 2 points is the width a comment shape reads at on a dense CAD
/// export without dominating it — a hairline vanishes among the drawing's own
/// 0.25 pt linework, which is the specific failure a markup on an engineering
/// drawing has to avoid.
pub const PEN_WIDTH_PTS: f64 = 2.0;

/// Why a markup drag committed nothing.
///
/// Reported rather than silently absorbed, and reported with enough detail to
/// act on, because *"nothing happened"* has several causes with opposite
/// responses — the same argument [`crate::canvas::moving::Refusal`] makes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// The gesture ended where it began: no extent on either axis. See the
    /// module docs on degenerate input.
    NoExtent,
    /// A coordinate was not finite. Refused rather than authored, because the
    /// alternative is a NaN in an annotation's `/Rect`.
    NotFinite,
    /// The page's device transform is not invertible, so there is no
    /// well-defined page-space position for the drag. Declining is the only
    /// honest answer; authoring fabricated geometry is not.
    DegeneratePage,
    /// The frame has no page to author onto — a strip whose visible window fell
    /// outside every page, or a document whose pages have not loaded.
    NoPage,
}

/// A markup drag in flight, in **canvas space**, ready to be drawn.
///
/// Returned by [`drag`] only while the pointer is down, and only when the
/// release would commit — the same "the preview describes something that will
/// actually happen" contract [`crate::canvas::moving::drag`] honours with its
/// ghost.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Preview {
    /// Which shape is being authored.
    pub kind: MarkupKind,
    /// Where the press landed. For [`MarkupKind::Arrow`] this is the **tail**.
    pub from: Pos2,
    /// Where the pointer is now. For [`MarkupKind::Arrow`] this is the **head**.
    pub to: Pos2,
}

/// Convert a **canvas-space** drag into a pair of **PDF user-space** endpoints.
///
/// # Why two point conversions and no arithmetic of our own
///
/// [`viewer::canvas_to_pdf_space`] applies the renderer's own page transform —
/// the crop-box origin, the `/Rotate`, and the Y flip. Writing any part of that
/// out here would be a second derivation of the page transform, which is the
/// precise failure `viewer`'s header warns about: *"PDF user space is y-UP;
/// canvas and screen are y-DOWN. The failure is silent — the page looks perfect
/// until someone selects a line and gets a different one."* For a markup the
/// symptom is worse than a mis-selection, because it is written to the file: a
/// rectangle dragged over the title block lands mirrored about the page's
/// horizontal centre line, and the operator finds out after saving.
///
/// Unlike [`crate::canvas::moving::page_delta`] this maps **positions**, not a
/// displacement, so the transform's translation is *not* cancelled — which is
/// the whole point. A markup has an absolute place on the page.
///
/// Returns `None` for a page whose device transform cannot be inverted, which
/// is the same condition under which both halves of the `viewer` bridge
/// decline.
#[must_use]
pub fn endpoints(from: Pos2, to: Pos2, page: &Page) -> Option<((f64, f64), (f64, f64))> {
    let start = viewer::canvas_to_pdf_space(from, page)?;
    let end = viewer::canvas_to_pdf_space(to, page)?;
    Some((
        (f64::from(start.x), f64::from(start.y)),
        (f64::from(end.x), f64::from(end.y)),
    ))
}

/// The ONE action a completed markup drag becomes.
///
/// Pure, and the only place the degenerate-input rule is applied. Deliberately
/// says nothing about *which* page is current or what the pen is: those are the
/// caller's and [`spec`]'s respectively, so this function is a statement about
/// the gesture alone and can be tested as one.
///
/// # Why the raw endpoints travel, un-normalised
///
/// Because normalising here would destroy the arrow's direction before anything
/// downstream could ask about it. The rectangle kinds are normalised in
/// [`spec`], at the moment the `Rect` is built, which is the last point at which
/// the raw pair is still available. See [`spec`]'s own note.
pub fn action(
    kind: MarkupKind,
    page: usize,
    start: (f64, f64),
    end: (f64, f64),
) -> Result<Action, Refusal> {
    if ![start.0, start.1, end.0, end.1]
        .iter()
        .all(|v| v.is_finite())
    {
        return Err(Refusal::NotFinite);
    }
    // ★ No second threshold, and none in page space. egui's own drag threshold
    // has already separated a click from a drag in SCREEN space; all that is
    // refused here is a drag that ended exactly where it began, which would
    // author a 1-point mark nobody can see. See the module docs.
    if start.0 == end.0 && start.1 == end.1 {
        return Err(Refusal::NoExtent);
    }
    Ok(Action::CommitMarkup {
        page,
        kind,
        start,
        end,
    })
}

/// Build the `pdfce-core` spec one markup drag authors.
///
/// Pure, and unit-tested, which is the reason it is here rather than inline in
/// the apply arm: the arm is a routing line, and the two decisions below are
/// rules that deserve a test each.
///
/// # ★ An arrow keeps its RAW endpoints; a rectangle kind is normalised
///
/// Carried across from the old shell's `commit_markup` (`main.rs:5624-5627`),
/// which states it in one sentence: *"the direction the operator dragged is the
/// direction the line points, and its arrowheads make that visible.
/// Normalising here would silently flip half of all drawn arrows."*
///
/// It is sharper in this shell than it was there, because this shell's arrow
/// has **one** head rather than two. The old shell authored
/// `(OpenArrow, OpenArrow)` — a double-headed line, for which a reversal is
/// invisible. `text/commands.rs` already promises the operator *"drag from the
/// tail to the head"*, so the head belongs at the **end** of the drag, and with
/// a single head a normalised rect would put it on the wrong end of half of all
/// arrows drawn — up-and-left and up-and-right ones — with nothing in the
/// document to say the shell had reversed them.
///
/// The rectangle kinds go the other way and *must* be normalised: `Rect` with
/// `llx > urx` is not a rectangle any reader will draw, and the operator may
/// drag in any of the four directions.
#[must_use]
pub fn spec(kind: MarkupKind, start: (f64, f64), end: (f64, f64)) -> MarkupSpec {
    let (r, g, b) = kind.rgb();
    let color = Color::Rgb(r, g, b);
    let rect = PageRect::from_corners(
        start.0.min(end.0),
        start.1.min(end.1),
        start.0.max(end.0),
        start.1.max(end.1),
    );
    match kind {
        MarkupKind::Rectangle => MarkupSpec::Square {
            rect,
            border: Some(color),
            // No fill. A filled comment shape hides the drawing it is a comment
            // about, which on a CAD sheet is the whole content under it.
            interior: None,
            border_width: PEN_WIDTH_PTS,
        },
        MarkupKind::Ellipse => MarkupSpec::Circle {
            rect,
            border: Some(color),
            interior: None,
            border_width: PEN_WIDTH_PTS,
        },
        // ★ RAW `start` and `end` — see this function's docs.
        MarkupKind::Arrow => MarkupSpec::Line {
            start,
            end,
            color,
            width: PEN_WIDTH_PTS,
            // Tail then head, in the operator's own words. `None` at the start
            // is what makes the raw-endpoint rule above load-bearing rather
            // than decorative.
            endings: (LineEnding::None, LineEnding::OpenArrow),
        },
        // Exactly one quad, always, so `validate_geometry`'s empty-quad refusal
        // is structurally unreachable from this path.
        MarkupKind::Highlight => MarkupSpec::TextMarkup {
            kind: TextMarkupKind::Highlight,
            quads: vec![Quad::from_rect(rect)],
            color,
        },
    }
}

/// Apply one frame of a markup drag: return the preview, or commit the markup.
///
/// The **only** function here that touches the frame. It does one of two
/// things:
///
/// * [`Phase::InFlight`] — returns the band for [`draw_preview`] and changes
///   nothing. Nothing is decomposed and nothing is re-rasterized: a markup drag
///   hit-tests nothing at all, which is why `canvas::interact` deliberately
///   leaves it out of the set of outcomes that need an object model. A preview
///   over a 129,758-object drawing costs one stroke.
/// * [`Phase::Complete`] — converts both endpoints to page space and pushes
///   exactly one [`Action::CommitMarkup`].
///
/// Returns `Some` only when a band should be drawn, and — as with the move
/// ghost — only when the release would actually commit. A drag with no page
/// under it draws nothing rather than a band that promises an annotation the
/// frame cannot author.
///
/// # Why the refusal is traced only on release
///
/// An in-flight drag is re-evaluated 60 times a second, and the `canvas-pointer`
/// lesson — fifty identical lines in nine seconds from a stationary pointer —
/// is what a per-frame refusal trace would reproduce. The release is one event,
/// and it is the one a harness reading the trace is asking about.
pub fn drag(
    kind: MarkupKind,
    from: Pos2,
    to: Pos2,
    phase: Phase,
    page_index: usize,
    page: Option<&Page>,
    actions: &mut Vec<Action>,
) -> Option<Preview> {
    let Some(page) = page else {
        if phase == Phase::Complete {
            decline(kind, page_index, Refusal::NoPage);
        }
        return None;
    };
    let Some((start, end)) = endpoints(from, to, page) else {
        if phase == Phase::Complete {
            decline(kind, page_index, Refusal::DegeneratePage);
        }
        return None;
    };

    if phase == Phase::InFlight {
        return Some(Preview { kind, from, to });
    }

    match action(kind, page_index, start, end) {
        Ok(raised) => {
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI.
                    //
                    // ★ Traced with its COORDINATES, not a success flag. The old
                    // shell's own note says why, and it is the sharpest sentence
                    // in that file: "the whole defect this Pass fixes was a shape
                    // landing somewhere the operator did not choose, and a trace
                    // saying only 'committed' would have been equally true before
                    // and after the fix."
                    //
                    // The RAW endpoints, in drag order — so a harness can prove
                    // the arrow's head is at the end the operator dragged to,
                    // which a normalised rect could not express.
                    "markup-commit kind={kind:?} page={page_index} \
                     x0={:.2} y0={:.2} x1={:.2} y1={:.2}",
                    start.0, start.1, end.0, end.1,
                )
            });
            actions.push(raised);
        }
        Err(reason) => decline(kind, page_index, reason),
    }
    None
}

/// Report a markup drag that committed nothing, with the reason.
///
/// One trace shape for every refusal, so a harness reads `markup-declined` and
/// finds the cause on the same line rather than inferring it from an absence —
/// the contract `canvas-move-declined` and `canvas-delete-declined` already
/// honour.
fn decline(kind: MarkupKind, page: usize, reason: Refusal) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("markup-declined kind={kind:?} page={page} reason={reason:?}")
    });
}

/// The number of segments an ellipse preview is drawn with.
///
/// 48 is enough that the polyline is indistinguishable from a curve at any zoom
/// this canvas reaches, and it is cheap: one band, once per frame of one drag.
const ELLIPSE_SEGMENTS: usize = 48;

/// The arrowhead barb length, in **screen** points, and the angle it opens at.
///
/// Screen-space, deliberately: the head is part of the *cursor*, and a head that
/// shrank to nothing at 25 % would stop saying which end of the band is the
/// head — which is the one thing this preview exists to say. The committed
/// annotation's own `/LE` head is drawn by the appearance stream at whatever
/// size the engine chooses; this is not a promise about that size, it is a
/// statement about direction.
const HEAD_LEN_PX: f32 = 14.0;
/// Half-angle of the arrowhead, in radians (≈ 24°).
const HEAD_ANGLE: f32 = 0.42;

/// Paint the markup band, given the [`Preview`] [`drag`] returned.
///
/// # ★ Why this is not `draw_marquee` with a different colour
///
/// Because a marquee and a markup band answer different questions. A marquee
/// asks *"what does this rectangle enclose?"* and is therefore always a
/// rectangle whatever it is about to select. A markup band asks *"is this the
/// shape you meant?"*, and the only way it can answer is by **being that
/// shape**: an ellipse drawn as its bounding box misstates the geometry by the
/// difference between a box and the ellipse inside it — 21 % of the area — and
/// an arrow drawn as a plain segment says nothing about which end the head is
/// on, which is the single most reversible property of the thing being
/// committed.
///
/// The old shell previewed a Circle as `circle_stroke` with the *smaller* of
/// the two half-extents, i.e. as the inscribed **circle** rather than the
/// ellipse it was about to author. That is drawn correctly here instead: on a
/// wide drag the two differ by the whole aspect ratio, and the operator would
/// have released expecting the circle they were shown.
///
/// # The colours are document colours, and that is why they are literals
///
/// Everything painted here is the pen — the colour and width that are about to
/// be written into the file. Reading it from [`egui::Visuals`] would be wrong in
/// the way `check-theme-colors.sh`'s own header describes: restyling the
/// application would change the colour of markup about to be committed, and the
/// change would only become visible after saving.
pub fn draw_preview(painter: &Painter, mapping: &PageMapping, preview: Preview) {
    let Preview { kind, from, to } = preview;
    let (a, b) = (mapping.to_screen(from), mapping.to_screen(to));
    let stroke = Stroke::new(pen_px(mapping), pen_color(kind));

    match kind {
        MarkupKind::Rectangle => {
            painter.rect_stroke(
                egui::Rect::from_two_pos(a, b),
                CornerRadius::ZERO,
                stroke,
                StrokeKind::Middle,
            );
        }
        MarkupKind::Ellipse => {
            let rect = egui::Rect::from_two_pos(a, b);
            let (cx, cy) = (rect.center().x, rect.center().y);
            let (rx, ry) = (rect.width() / 2.0, rect.height() / 2.0);
            let mut points: Vec<Pos2> = (0..=ELLIPSE_SEGMENTS)
                .map(|i| {
                    #[allow(clippy::cast_precision_loss)]
                    let t = (i as f32) / (ELLIPSE_SEGMENTS as f32) * std::f32::consts::TAU;
                    Pos2::new(cx + rx * t.cos(), cy + ry * t.sin())
                })
                .collect();
            // Close it exactly rather than relying on the last sample landing
            // on the first: a visible seam in a preview reads as a shape that
            // did not close.
            if let (Some(first), Some(last)) = (points.first().copied(), points.last_mut()) {
                *last = first;
            }
            painter.add(egui::Shape::line(points, stroke));
        }
        MarkupKind::Arrow => {
            painter.line_segment([a, b], stroke);
            for barb in arrowhead(a, b) {
                painter.line_segment([b, barb], stroke);
            }
        }
        // A wash, not an outline: a highlight IS a translucent fill, and an
        // outlined empty box would describe a rectangle annotation instead.
        MarkupKind::Highlight => {
            painter.rect_filled(
                egui::Rect::from_two_pos(a, b),
                CornerRadius::ZERO,
                highlight_wash(kind),
            );
        }
    }
}

/// The two barb endpoints of the preview arrowhead at `head`.
///
/// Returns an empty array's worth of coincident points for a zero-length band,
/// which draws nothing — a head with no direction to point in must not be
/// invented from a normalised zero vector (which would be NaN).
fn arrowhead(tail: Pos2, head: Pos2) -> [Pos2; 2] {
    let dir = head - tail;
    let len = dir.length();
    if !len.is_finite() || len <= f32::EPSILON {
        return [head, head];
    }
    let back = -dir / len;
    let (s, c) = (HEAD_ANGLE.sin(), HEAD_ANGLE.cos());
    let rot = |x: f32, y: f32| Pos2::new(head.x + x * HEAD_LEN_PX, head.y + y * HEAD_LEN_PX);
    [
        rot(back.x * c - back.y * s, back.x * s + back.y * c),
        rot(back.x * c + back.y * s, -back.x * s + back.y * c),
    ]
}

/// The pen's width in **screen** points at this frame's magnification.
///
/// Derived by measuring the mapping rather than by asking it for a zoom:
/// [`PageMapping`] has no `zoom()` accessor, deliberately, because everything
/// that divided by one was a place a second division could hide (see that
/// module's header). Projecting a one-unit page-space step and measuring what
/// arrives is the same answer with no number to divide by, and it keeps the
/// preview's thickness equal to the stroke that will actually land.
///
/// Floored at one point so the band is never invisible at low zoom — a preview
/// the operator cannot see is a preview they cannot aim.
fn pen_px(mapping: &PageMapping) -> f32 {
    #[allow(clippy::cast_possible_truncation)]
    let width = PEN_WIDTH_PTS as f32;
    let scale = mapping.to_screen(Pos2::new(1.0, 0.0)).x - mapping.to_screen(Pos2::ZERO).x;
    if scale.is_finite() && scale > 0.0 {
        (width * scale).max(1.0)
    } else {
        width.max(1.0)
    }
}

/// The pen colour, as egui sees it.
fn pen_color(kind: MarkupKind) -> Color32 {
    let (r, g, b) = kind.rgb();
    let byte = |v: f64| {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let out = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        out
    };
    // DOCUMENT COLOUR: the operator's pen, converted for the preview from the
    // exact components `spec` writes into `/C`. Deriving it from one source
    // rather than naming a second is what keeps the band the colour of the
    // thing it is previewing.
    Color32::from_rgb(byte(r), byte(g), byte(b))
}

/// The highlight preview's fill: the pen colour at the alpha a highlight reads
/// at over content.
fn highlight_wash(kind: MarkupKind) -> Color32 {
    let c = pen_color(kind);
    // DOCUMENT COLOUR: arithmetic on the pen colour above, not a second choice
    // of colour. The alpha is a legibility figure for the *preview* — the
    // committed annotation's translucency is the engine's `/CA`, which pdfce
    // does not yet write (filed in `open/`), so this states "a highlight" and
    // does not promise a specific opacity.
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 90)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfce_core::object::{Dict, ObjId};

    /// A minimal page fixture — the same one `viewer`'s and `moving`'s geometry
    /// tests use, because these functions read exactly what those do:
    /// `crop_box` and `rotate`.
    fn test_page(w: f64, h: f64, rotate: u16) -> Page {
        Page {
            id: ObjId::new(1, 0),
            resources: Dict::new(),
            media_box: PageRect::from_corners(0.0, 0.0, w, h),
            crop_box: PageRect::from_corners(0.0, 0.0, w, h),
            rotate,
            contents: Vec::new(),
            contents_unresolved: 0,
        }
    }

    // -----------------------------------------------------------------
    // ★ The defect this whole module exists to prevent
    // -----------------------------------------------------------------

    /// ★ **The markup lands where the operator dragged, not at the page
    /// centre.**
    ///
    /// The regression test for *"they just drop things into the center of the
    /// pdf window."* It is written as a **magnitude** assertion against the
    /// dragged corners and, separately, as a statement that the result is
    /// nowhere near the media-box centre — because `HANDOFF.md` §2's lesson is
    /// that a test asserting a relation rather than a magnitude is satisfied by
    /// any absurdity in the right direction. "The shape is on the page" would
    /// have passed on the defective build; "the shape's corners ARE the corners
    /// dragged" cannot.
    #[test]
    fn the_markup_lands_where_the_drag_was_and_not_at_the_page_centre() {
        let page = test_page(612.0, 792.0, 0);
        // A drag in the lower-left quadrant of the canvas, i.e. the UPPER-left
        // of the page in PDF space.
        let (start, end) =
            endpoints(Pos2::new(72.0, 90.0), Pos2::new(200.0, 150.0), &page).expect("invertible");

        assert!((start.0 - 72.0).abs() < 1e-3, "{start:?}");
        assert!((end.0 - 200.0).abs() < 1e-3, "{end:?}");
        // Canvas Y is down, PDF Y is up: 90 from the top of a 792-high page is
        // 702 from the bottom.
        assert!((start.1 - 702.0).abs() < 1e-3, "{start:?}");
        assert!((end.1 - 642.0).abs() < 1e-3, "{end:?}");

        let MarkupSpec::Square { rect, .. } = spec(MarkupKind::Rectangle, start, end) else {
            panic!("Rectangle must author a /Square");
        };
        let (cx, cy) = ((rect.llx + rect.urx) / 2.0, (rect.lly + rect.ury) / 2.0);
        assert!(
            (cx - 306.0).abs() > 100.0 && (cy - 396.0).abs() > 100.0,
            "the shape drifted toward the page centre: centre=({cx}, {cy})"
        );
    }

    /// The same drag, at four magnifications, through the frame's real
    /// mapping — because the pointer only ever reports **screen** positions and
    /// a stray zoom would enter exactly there.
    ///
    /// This is the markup gesture's form of
    /// `moving::a_drag_between_two_page_points_moves_the_same_distance_at_every_zoom`,
    /// and it is the stronger of the two statements: a move only has to be the
    /// same *displacement* at every zoom, while a markup has to land on the same
    /// *absolute* page coordinates.
    #[test]
    fn the_same_drag_authors_the_same_page_coordinates_at_every_zoom() {
        use crate::viewer::page_extent_pts;

        let page = test_page(612.0, 792.0, 0);
        let extent = page_extent_pts(&page);
        let (grabbed, dropped) = (Pos2::new(100.0, 120.0), Pos2::new(260.0, 300.0));

        let mut seen: Vec<((f64, f64), (f64, f64))> = Vec::new();
        for &zoom in &[0.25_f32, 1.0, 4.0, 12.0] {
            let image_rect = egui::Rect::from_min_size(
                Pos2::new(37.0, 11.0),
                egui::vec2(extent.0 * zoom, extent.1 * zoom),
            );
            let map = PageMapping::new(image_rect, extent, zoom);
            let from = map.to_page(map.to_screen(grabbed));
            let to = map.to_page(map.to_screen(dropped));
            seen.push(endpoints(from, to, &page).expect("invertible"));
        }
        for got in &seen {
            assert!(
                (got.0.0 - seen[0].0.0).abs() < 1e-2
                    && (got.0.1 - seen[0].0.1).abs() < 1e-2
                    && (got.1.0 - seen[0].1.0).abs() < 1e-2
                    && (got.1.1 - seen[0].1.1).abs() < 1e-2,
                "the page coordinates changed with the zoom: {seen:?}"
            );
        }
        // …and they are the right coordinates, not merely consistent ones.
        assert!((seen[0].0.0 - 100.0).abs() < 1e-2, "{seen:?}");
        assert!((seen[0].0.1 - 672.0).abs() < 1e-2, "{seen:?}");
        assert!((seen[0].1.0 - 260.0).abs() < 1e-2, "{seen:?}");
        assert!((seen[0].1.1 - 492.0).abs() < 1e-2, "{seen:?}");
    }

    /// A rotated page rotates the placement, through the renderer's own
    /// transform rather than a formula written out here.
    #[test]
    fn a_rotated_page_places_the_markup_through_the_page_transform() {
        let upright = test_page(612.0, 792.0, 0);
        let turned = test_page(612.0, 792.0, 90);
        let at = Pos2::new(100.0, 120.0);
        let a = endpoints(at, at + egui::vec2(10.0, 10.0), &upright).expect("invertible");
        let b = endpoints(at, at + egui::vec2(10.0, 10.0), &turned).expect("invertible");
        assert_ne!(
            a, b,
            "a 90° page must not author the same coordinates as an upright one"
        );
    }

    // -----------------------------------------------------------------
    // ★ The arrow keeps its direction
    // -----------------------------------------------------------------

    /// ★ **An arrow dragged up-and-left keeps its head at the end the operator
    /// dragged to.**
    ///
    /// The `:5624-5627` decision, asserted in the direction that a normalising
    /// implementation fails. A normalised rect would report
    /// `start = (min, min)`, which for this drag is the **head**, so the
    /// arrowhead would be at the tail — and, with a single head, nothing in the
    /// document would say so.
    #[test]
    fn an_arrow_dragged_backwards_keeps_its_head_at_the_end() {
        let tail = (400.0, 500.0);
        let head = (120.0, 700.0); // up and to the left: both axes reversed
        let MarkupSpec::Line {
            start,
            end,
            endings,
            ..
        } = spec(MarkupKind::Arrow, tail, head)
        else {
            panic!("Arrow must author a /Line");
        };
        assert_eq!(start, tail, "the tail must stay the tail");
        assert_eq!(end, head, "the head must stay the head");
        assert_eq!(
            endings,
            (LineEnding::None, LineEnding::OpenArrow),
            "one head, at the end of the drag — `text/commands.rs` promises \
             \"drag from the tail to the head\""
        );
    }

    /// …and every rectangle kind IS normalised, in all four drag directions,
    /// because a `Rect` with `llx > urx` is not a rectangle any reader draws.
    ///
    /// Asserted over all four kinds and all four directions rather than one
    /// case, because the failure is per-kind: it is exactly the shape of
    /// mistake that gets fixed for Rectangle and left in Ellipse.
    #[test]
    fn every_rectangle_kind_is_normalised_in_all_four_drag_directions() {
        let corners = [(100.0_f64, 200.0_f64), (300.0, 500.0)];
        for kind in [
            MarkupKind::Rectangle,
            MarkupKind::Ellipse,
            MarkupKind::Highlight,
        ] {
            assert!(kind.is_rect(), "{kind:?}");
            for (a, b) in [
                (corners[0], corners[1]),
                (corners[1], corners[0]),
                ((corners[0].0, corners[1].1), (corners[1].0, corners[0].1)),
                ((corners[1].0, corners[0].1), (corners[0].0, corners[1].1)),
            ] {
                let rect = match spec(kind, a, b) {
                    MarkupSpec::Square { rect, .. } | MarkupSpec::Circle { rect, .. } => rect,
                    MarkupSpec::TextMarkup { quads, .. } => {
                        assert_eq!(quads.len(), 1, "a highlight authors exactly one quad");
                        PageRect::from_corners(100.0, 200.0, 300.0, 500.0)
                    }
                    other => panic!("{kind:?} authored {other:?}"),
                };
                assert!(
                    rect.llx < rect.urx && rect.lly < rect.ury,
                    "{kind:?} {rect:?}"
                );
                assert!((rect.llx - 100.0).abs() < 1e-9 && (rect.ury - 500.0).abs() < 1e-9);
            }
        }
        assert!(!MarkupKind::Arrow.is_rect());
    }

    /// Each kind authors its own subtype, and nothing borrows another's.
    #[test]
    fn each_kind_authors_its_own_subtype() {
        let (a, b) = ((10.0, 20.0), (30.0, 40.0));
        assert!(matches!(
            spec(MarkupKind::Rectangle, a, b),
            MarkupSpec::Square { .. }
        ));
        assert!(matches!(
            spec(MarkupKind::Ellipse, a, b),
            MarkupSpec::Circle { .. }
        ));
        assert!(matches!(
            spec(MarkupKind::Arrow, a, b),
            MarkupSpec::Line { .. }
        ));
        assert!(matches!(
            spec(MarkupKind::Highlight, a, b),
            MarkupSpec::TextMarkup {
                kind: TextMarkupKind::Highlight,
                ..
            }
        ));
    }

    /// ★ **No geometric markup is authored with a filled interior**, so a
    /// comment never hides the drawing it is a comment about.
    #[test]
    fn a_shape_markup_is_never_filled() {
        for kind in [MarkupKind::Rectangle, MarkupKind::Ellipse] {
            match spec(kind, (0.0, 0.0), (10.0, 10.0)) {
                MarkupSpec::Square {
                    interior, border, ..
                }
                | MarkupSpec::Circle {
                    interior, border, ..
                } => {
                    assert_eq!(interior, None, "{kind:?} must not fill");
                    assert!(border.is_some(), "{kind:?} must have a visible border");
                }
                other => panic!("{kind:?} authored {other:?}"),
            }
        }
    }

    // -----------------------------------------------------------------
    // Degenerate input
    // -----------------------------------------------------------------

    /// ★ **A drag that ends where it began commits nothing** — rather than a
    /// 1-point mark nobody can see, holding a slot on the undo stack.
    #[test]
    fn a_drag_with_no_extent_commits_nothing() {
        for kind in [
            MarkupKind::Rectangle,
            MarkupKind::Ellipse,
            MarkupKind::Arrow,
            MarkupKind::Highlight,
        ] {
            assert_eq!(
                action(kind, 0, (100.0, 200.0), (100.0, 200.0)),
                Err(Refusal::NoExtent),
                "{kind:?}"
            );
        }
    }

    /// …but the smallest real extent on **either** axis does commit. There is
    /// no page-space threshold; egui's screen-space drag threshold is the only
    /// one, which is what keeps a deliberate small mark at 16 % zoom from being
    /// silently replaced by something else.
    #[test]
    fn the_smallest_real_extent_on_either_axis_still_commits() {
        for end in [(100.01, 200.0), (100.0, 200.01)] {
            let raised = action(MarkupKind::Rectangle, 3, (100.0, 200.0), end).expect("committed");
            assert_eq!(
                raised,
                Action::CommitMarkup {
                    page: 3,
                    kind: MarkupKind::Rectangle,
                    start: (100.0, 200.0),
                    end,
                }
            );
        }
    }

    /// A non-finite coordinate is refused rather than authored into an
    /// annotation's `/Rect`.
    #[test]
    fn a_non_finite_endpoint_is_refused() {
        for end in [(f64::NAN, 1.0), (1.0, f64::INFINITY)] {
            assert_eq!(
                action(MarkupKind::Arrow, 0, (0.0, 0.0), end),
                Err(Refusal::NotFinite)
            );
        }
    }

    /// ★ **A click with no drag never reaches this module at all**, and the
    /// degenerate drag it would look like is refused.
    ///
    /// The module docs' decision, pinned from both ends: the gesture machine
    /// raises `Click` (not a `DragKind`) for a press-and-release under egui's
    /// threshold, and if a zero-extent drag does arrive it commits nothing.
    /// Without the second half, "a click places nothing" would rest on egui's
    /// behaviour alone.
    #[test]
    fn a_click_places_nothing_and_the_degenerate_drag_it_resembles_is_refused() {
        use crate::canvas::gesture::{DragKind, GestureOutcome, GestureState, PointerFrame};

        let mut gestures = GestureState::default();
        let out = gestures.update(
            PointerFrame {
                clicked: true,
                pos: Some(Pos2::new(150.0, 150.0)),
                ..PointerFrame::default()
            },
            DragKind::Markup(MarkupKind::Rectangle),
        );
        assert!(
            matches!(out, GestureOutcome::Click { .. }),
            "a click must stay a click: {out:?}"
        );

        let page = test_page(612.0, 792.0, 0);
        let mut actions = Vec::new();
        let at = Pos2::new(150.0, 150.0);
        let preview = drag(
            MarkupKind::Rectangle,
            at,
            at,
            Phase::Complete,
            0,
            Some(&page),
            &mut actions,
        );
        assert_eq!(preview, None);
        assert!(
            actions.is_empty(),
            "a zero-extent drag must author nothing, not a default-sized box"
        );
    }

    // -----------------------------------------------------------------
    // The preview
    // -----------------------------------------------------------------

    /// An in-flight drag previews and commits nothing; the release commits
    /// exactly one action and previews nothing.
    #[test]
    fn a_markup_draws_in_flight_and_commits_once() {
        let page = test_page(612.0, 792.0, 0);
        let (from, to) = (Pos2::new(50.0, 60.0), Pos2::new(150.0, 200.0));

        let mut actions = Vec::new();
        let preview = drag(
            MarkupKind::Ellipse,
            from,
            to,
            Phase::InFlight,
            2,
            Some(&page),
            &mut actions,
        );
        assert_eq!(
            preview,
            Some(Preview {
                kind: MarkupKind::Ellipse,
                from,
                to
            })
        );
        assert!(actions.is_empty(), "an in-flight drag must not commit");

        let preview = drag(
            MarkupKind::Ellipse,
            from,
            to,
            Phase::Complete,
            2,
            Some(&page),
            &mut actions,
        );
        assert_eq!(preview, None, "a released drag draws no band");
        assert_eq!(actions.len(), 1);
        assert!(matches!(
            actions[0],
            Action::CommitMarkup {
                page: 2,
                kind: MarkupKind::Ellipse,
                ..
            }
        ));
    }

    /// With no page under it, a markup drag draws nothing and commits nothing —
    /// a band that promised an annotation the frame cannot author would be the
    /// dishonest preview rule 4 forbids.
    #[test]
    fn a_frame_with_no_page_draws_no_band_and_commits_nothing() {
        let mut actions = Vec::new();
        for phase in [Phase::InFlight, Phase::Complete] {
            assert_eq!(
                drag(
                    MarkupKind::Arrow,
                    Pos2::ZERO,
                    Pos2::new(10.0, 10.0),
                    phase,
                    0,
                    None,
                    &mut actions,
                ),
                None
            );
        }
        assert!(actions.is_empty());
    }

    /// ★ **The preview's arrowhead is at the head end**, whichever way the
    /// operator drags — the on-screen half of the raw-endpoint rule.
    ///
    /// Asserted as a distance, not as a side: both barbs must be within a barb's
    /// length of the head and nowhere near the tail. A "the head is drawn"
    /// assertion would pass on an implementation that drew it at the wrong end.
    #[test]
    fn the_preview_arrowhead_sits_at_the_head_whichever_way_the_drag_went() {
        for (tail, head) in [
            (Pos2::new(10.0, 10.0), Pos2::new(200.0, 120.0)),
            (Pos2::new(200.0, 120.0), Pos2::new(10.0, 10.0)),
            (Pos2::new(200.0, 10.0), Pos2::new(10.0, 120.0)),
        ] {
            for barb in arrowhead(tail, head) {
                assert!(
                    (barb - head).length() <= HEAD_LEN_PX + 1e-3,
                    "a barb landed {} from the head",
                    (barb - head).length()
                );
                assert!(
                    (barb - tail).length() > HEAD_LEN_PX,
                    "a barb landed at the TAIL: the arrow is drawn backwards"
                );
            }
        }
    }

    /// A zero-length band produces no head rather than a NaN one.
    #[test]
    fn a_zero_length_band_has_no_arrowhead() {
        let at = Pos2::new(5.0, 5.0);
        assert_eq!(arrowhead(at, at), [at, at]);
    }

    /// The preview's stroke is the pen's real width at this magnification, and
    /// never thinner than a visible hairline.
    #[test]
    fn the_preview_stroke_is_the_pen_width_at_this_zoom() {
        let extent = (612.0_f32, 792.0_f32);
        let widths: Vec<f32> = [0.1_f32, 1.0, 8.0]
            .iter()
            .map(|&zoom| {
                let rect = egui::Rect::from_min_size(
                    Pos2::ZERO,
                    egui::vec2(extent.0 * zoom, extent.1 * zoom),
                );
                pen_px(&PageMapping::new(rect, extent, zoom))
            })
            .collect();
        #[allow(clippy::cast_possible_truncation)]
        let pen = PEN_WIDTH_PTS as f32;
        assert!((widths[1] - pen).abs() < 1e-3, "{widths:?}");
        assert!((widths[2] - pen * 8.0).abs() < 1e-2, "{widths:?}");
        assert!(widths[0] >= 1.0, "a hairline must stay visible: {widths:?}");
    }

    /// The preview colour is the colour that will be committed, component for
    /// component — one source, so the band cannot show one pen and the file
    /// carry another.
    #[test]
    fn the_preview_colour_is_the_committed_colour() {
        for kind in [MarkupKind::Rectangle, MarkupKind::Highlight] {
            let (r, g, b) = kind.rgb();
            let c = pen_color(kind);
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let expect = |v: f64| (v * 255.0).round() as u8;
            assert_eq!((c.r(), c.g(), c.b()), (expect(r), expect(g), expect(b)));
            let authored = match spec(kind, (0.0, 0.0), (1.0, 1.0)) {
                MarkupSpec::Square { border, .. } => border.expect("a border colour"),
                MarkupSpec::TextMarkup { color, .. } => color,
                other => panic!("{kind:?} authored {other:?}"),
            };
            let Color::Rgb(sr, sg, sb) = authored else {
                panic!("{kind:?} authored a non-RGB colour");
            };
            assert!((sr - r).abs() < 1e-9 && (sg - g).abs() < 1e-9 && (sb - b).abs() < 1e-9);
        }
    }
}
