//! # `canvas::gesture::meaning` — what a press MEANS, decided once and then remembered
//!
//! One pure function, [`press_kind`], and the two enums it decides between:
//! [`DragKind`], which says what a drag is going to *do*, and [`MarqueeIntent`],
//! which says what a rubber band does when it is released. Nothing in this file
//! holds state, touches egui, or knows that frames exist — a press is
//! `(tool, grip, zoom_armed, capabilities)` in and one meaning, or `None`, out.
//! That is what makes the whole precedence testable as a table.
//!
//! The state machine that *carries* a meaning across a press, a drag and a
//! release — [`PointerFrame`](super::PointerFrame),
//! [`GestureState`](super::GestureState) and
//! [`GestureOutcome`] — is the parent module, [`super`].
//! It calls this function on exactly one frame per gesture, the press frame,
//! and then never asks again.
//!
//! The precedence itself — which meaning wins when two are available, and which
//! presses a mode refuses outright — is documented on [`press_kind`], because it
//! *is* the rule rather than a note about it.
//!
//! [`press_kind`] deliberately has no case for the hand tool, and the absence is
//! load-bearing: `canvas::interact` hands the state machine a **blank** frame
//! while the hand is active, so no press ever arrives here to be classified.
//! That rule — *one state machine, one meaning per frame* — is stated in full in
//! [`super`]'s header, under "Marquee versus pan".
//!
//! ## ★ Marquee-select versus marquee-zoom: one rubber band, two releases
//!
//! Phase 3.4 adds a marquee that *zooms* to what it encloses. It is
//! deliberately **the same gesture**: same press, same in-flight rect, same
//! pixels on screen ([`crate::canvas::overlay::draw_marquee`] is not
//! duplicated), same normalisation, same Escape. What differs is one thing —
//! *what happens on release* — so what is carried is one value, [`MarqueeIntent`].
//!
//! It is sampled **at the press**, exactly as `shift` is, and for the identical
//! reason: the one-shot arming is retired when the drag completes, and an
//! intent re-read at release would be read after something else had already
//! consumed it. A gesture means what it meant when it started.

use crate::app::modes::Capabilities;
use crate::canvas::handles::Grip;
use crate::canvas::markup::MarkupKind;
use crate::canvas::tool::CanvasTool;

// Rustdoc-only: the doc comments below link to items that live in the parent
// module. The link targets have to be nameable here for the reference to
// resolve, but nothing in this file's *code* needs them — so the import is
// compiled only when rustdoc is running, and never costs an unused-import
// warning in a normal build.
#[cfg(doc)]
use super::GestureOutcome;

/// What a press landed on — decided once, on the press frame, by the caller.
///
/// # Why it is decided at press time and then remembered
///
/// A drag that began on a grip stays a resize even when the pointer wanders
/// off the grip, off the object, and off the page. Re-deciding per frame
/// would turn a resize into a marquee the instant the operator's hand moved
/// faster than the box, which is exactly when they are dragging hardest.
///
/// This is also what makes the grips *consume* their drags. See
/// [`crate::canvas::handles`]: without it, a drag aimed at a resize grip
/// would fall through to a marquee and silently replace the selection the
/// operator was trying to resize.
/// What a completed rubber-band does — **the only difference between
/// marquee-select and marquee-zoom.**
///
/// See the module docs. Carried by [`DragKind::Marquee`] and echoed back on
/// [`GestureOutcome::Marquee`] so the release arm can branch on it without
/// asking the world what mode it is in — the world may have changed since the
/// press, and the press is when the operator decided.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MarqueeIntent {
    /// Select everything the band fully encloses. The default, and what an
    /// un-armed canvas does.
    #[default]
    Select,
    /// Zoom the view to the band. Armed by
    /// [`crate::canvas::zoom::arm_region_zoom`] and retired on release.
    Zoom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragKind {
    /// The press was on empty paper, or on unselected content: rubber-band,
    /// doing whatever [`MarqueeIntent`] says on release.
    Marquee(MarqueeIntent),
    /// The press was inside the selection's body: move it.
    Move,
    /// The press was on one of the eight resize grips.
    Resize(Grip),
    /// The press was on the page with the **text tool armed**, or in a mode that
    /// cannot select its content: **sweep a range of text**.
    ///
    /// Carries nothing, unlike the three above, because a text drag's whole
    /// state is its two endpoints and those already travel on
    /// [`GestureOutcome::TextSelect`]. There is no per-drag choice to sample at
    /// the press — no kind, no intent, no grip.
    ///
    /// ★ That emptiness used to carry an extra claim: *"which is itself the
    /// reason the gate for it is a mode question rather than an armed-tool one."*
    /// The inference was wrong and is corrected rather than deleted, because it
    /// is a tempting one. Carrying no per-drag state says nothing about **who
    /// decides** the drag happens; it says only that the deciding does not have
    /// to be *remembered*. Since 2026-08-14 the gate is both — an armed
    /// [`CanvasTool::Text`], or the pre-existing mode rule — and this variant
    /// still carries nothing, because
    /// [`crate::canvas::tool::CanvasTool::Text`] itself carries nothing either.
    /// See [`crate::canvas::textsel`]'s header §3.
    TextSelect,
    /// The markup tool was armed: **draw**, in the carried shape.
    ///
    /// The kind is carried on the drag rather than re-read at the release, for
    /// the identical reason [`MarqueeIntent`] is — *a gesture means what it
    /// meant when it started*. It also gives the markup tool, for free, the
    /// property the old shell had to write code for: changing the armed kind
    /// mid-drag cannot reach a drag already in flight, so there is no
    /// in-progress gesture to discard.
    Markup(MarkupKind),
    /// Dragging out the **rectangle a text-bearing annotation will occupy** —
    /// a text box or a stamp.
    ///
    /// # ★ Its own variant rather than `Markup(kind)`, and the reason is the
    /// completion rule
    ///
    /// A `Markup` drag authors on release. This one does not: the release
    /// opens a dialog and the operator types, and nothing reaches the document
    /// until they accept. Sharing the variant would make every arm that asks
    /// *"does this release author?"* need a second predicate for the exception.
    ///
    /// The sticky note is absent because it is not dragged at all — its rect is
    /// discarded by the format, so it is placed with a click and takes the
    /// click branch beside the measure and caret tools.
    TextAnnot(crate::canvas::textannot::TextAnnotKind),
}

/// What a press means, given the tool, what it landed on and what is armed —
/// **the whole precedence, in one pure function.**
///
/// Lifted out of `canvas::interact` when the markup tool arrived, because it
/// stopped being a two-case question the moment there were three tools and it
/// is exactly the kind of rule this module exists to hold: it is a decision
/// about what the pointer means, it is drivable with no window, and leaving it
/// as a `match` in the middle of the wiring is how the ordering below becomes
/// three separate opinions.
///
/// # The order is the rule
///
/// 1. **An armed markup tool outranks everything**, including the grips. A
///    markup drag that started on a selected object's resize handle must draw a
///    shape, not resize — the operator armed a pen, and grips belong to a
///    selection they are not currently acting on. (There is no resize verb to
///    reach anyway; see [`crate::canvas::handles`].) It outranks the region
///    zoom for the same reason: only one of the two can own the primary drag,
///    and the one the operator armed *last* is not knowable here — but the one
///    that authors content is the one whose loss would be silent.
///
///    ★ This rung sees only the **band** and **freehand** kinds. The two
///    vertex kinds are answered by an early return above, beside the measure
///    tools, because their gesture is clicks and they have no drag at all —
///    [`crate::canvas::markup::MarkupKind::is_vertex`], and the comment at the
///    branch itself.
/// 2. **An armed text tool**, which sweeps a range in *every* mode — including
///    the ones whose primary button is otherwise the content marquee. It sits
///    here, above the content branch, because that branch is total: below it this
///    rung would be unreachable in exactly the mode the tool was built for. It
///    yields to an armed region zoom, and only to that; see the comment at the
///    branch itself for why the ordering is borrowed from the reading-mode text
///    row rather than decided afresh.
/// 3. **A grip** — resize on the six that resize, move on the two that do not.
/// 4. **An armed region zoom**, which turns the marquee's release into a zoom.
/// 5. **A plain marquee**, which is what an un-armed canvas does.
///
/// The hand tool is deliberately **absent** from this list, and its absence is
/// load-bearing: `canvas::interact` hands the gesture machine a *blank* frame
/// while the hand is active, so no press ever reaches this function to be
/// classified. One state machine, one meaning per frame — see the module
/// header.
///
/// # ★ The mode gate lives here, and it is two answers rather than one
///
/// The mode's [`Capabilities`] are applied **here**, at the point where a press
/// is given its meaning, rather than at the several places that act on one.
/// That ordering is the whole design: a press whose meaning is forbidden never
/// becomes a drag, so there is no band to draw, no ghost to preview, no
/// release to refuse and no half-gesture to explain.
///
/// [`PressMeaning`] carries **two** answers because the canvas has two kinds of
/// tool and they take the primary button differently — see that type's header.
/// A single `Option<DragKind>` was the first shape of this gate and it was
/// wrong in a way that would not have shown up until Review mode was used in
/// anger: it made "a drag means nothing here" and "a click means nothing here"
/// the same fact, which is exactly false for the measure tools, whose entire
/// gesture is clicks.
///
/// Refusing at the *press* is also what keeps the safety rule intact
/// (`MODES_AND_PANELS.md`: *"It never makes a visible control silently
/// inert"*). Nothing visible is refused, because in a mode that cannot select
/// there is no selection, hence no handles and no outline — see
/// `app::modes::capability` §5 and `PdfceApp::on_mode_capabilities_changed`,
/// which clears the selection on the way in precisely so that this function
/// never has to refuse a grip the operator can see.
///
/// Which capability each meaning needs:
///
/// | Meaning | Needs |
/// |---|---|
/// | [`DragKind::Markup`] | `author_markup` |
/// | a **vertex-markup** click — PolyLine, Polygon | `author_markup`, and it is the same flag on purpose: these author a comment, so a mode that draws rectangles draws polygons |
/// | a measure **click** | `author_measure` |
/// | [`DragKind::Resize`], [`DragKind::Move`] | `edit_content` |
/// | [`DragKind::Marquee`] with [`MarqueeIntent::Select`] | `edit_content` |
/// | a selecting **click** | `edit_content` |
/// | [`DragKind::Marquee`] with [`MarqueeIntent::Zoom`] | **nothing** — it is a navigation gesture that reads the document and changes none of it, so it is offered in every mode, Read included |
/// | [`DragKind::TextSelect`], and the click that goes with it | **nothing** — either because the operator armed the text tool, or *because* `edit_content` is absent, which is the one row here that reads backwards |
///
/// # ★ The text row, and why it is not an inconsistency
///
/// Every other row above asks *"does this mode permit the gesture?"*. The text
/// row asks *"is the primary button free, or has the operator claimed it?"* —
/// which is a different question that happens to read the same flag in one of
/// its two halves. That is not a capability inverted.
///
/// Selecting text authors nothing — it reads the page and writes to the
/// clipboard, which is the operator's own *copying is not authoring* ruling of
/// 2026-08-14 — so there is nothing here to permit, in either half. What there
/// is, is a collision: in a mode that selects page content the primary drag is
/// already the marquee. `crate::canvas::textsel::takes_the_press` is the one
/// place that collision is resolved, this function asks it, and
/// `canvas::interact` asks the same function again when it routes the click — so
/// the two cannot disagree about what a press meant. That module's header §3
/// carries the full argument, including why a `select_text` capability would
/// have been the wrong shape.
///
/// ★ **Since 2026-08-14 the predicate has two disjuncts**, and the second one
/// changes where the exclusivity comes from rather than whether there is any:
///
/// * **un-armed** (`CanvasTool::Select` in a mode that cannot select content) —
///   exclusive **by construction**, one flag on both sides of one branch, which
///   is how it shipped;
/// * **armed** (`CanvasTool::Text`, in any mode) — exclusive **by precedence**,
///   at rung 2 above, which is the rule `DragKind::Markup` has always used.
///
/// The property an operator can feel is untouched either way: this function
/// returns one [`DragKind`], so one press has one meaning. What a reader has to
/// know is that in Edit *both underlying facts* can now be true at once — the
/// mode can select content, and the operator has asked for text — so the order
/// of the branches below is load-bearing where it previously was not.
/// **What the primary button may do this frame** — the answer [`press_kind`]
/// returns and [`super::GestureState::update`] acts on.
///
/// # ★ Why two fields rather than one `Option<DragKind>`
///
/// Because the canvas has two kinds of authoring tool and they take the primary
/// button in genuinely different ways:
///
/// | tool | the gesture is | uses |
/// |---|---|---|
/// | markup — rectangle, ellipse, arrow, highlight | press, drag out a shape, release | the **drag** |
/// | measure — linear, radius/diameter, two-line, scale | click point A, click point B, click where the dimension sits | the **click** |
/// | select | either: click to select, drag to marquee | both |
/// | text | either: drag to sweep a range, click to take a word / a line / extend / clear | **both**, and this is the row that shows why the two fields cannot be collapsed even for a tool with no state — three of the gesture's four meanings are clicks (`canvas::textsel` §1) |
///
/// A single `Option<DragKind>` cannot express that, and the gate's first
/// version proved it: it suppressed the click whenever it suppressed the drag,
/// which is right for Read (neither means anything) and **wrong for Review**,
/// where a dimension must be placeable and page content must not be
/// selectable. The two facts have to be separable because a mode really does
/// grant one without the other.
///
/// Keeping them in one value rather than as two returns is what stops them
/// drifting apart: there is exactly one function that decides what a press
/// means, and it decides both halves in one pass over the same inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PressMeaning {
    /// What a **drag** starting now would mean, or `None` if a drag means
    /// nothing here — either because the mode forbids it, or because the armed
    /// tool simply has no drag gesture.
    ///
    /// The two reasons are deliberately not distinguished. Nothing downstream
    /// wants to tell them apart: in both cases no drag starts, no band is
    /// drawn, and nothing is committed. A reader who needs to know *why* is
    /// asking a question about the tool and the mode, which is what
    /// [`press_kind`] is for.
    pub drag: Option<DragKind>,
    /// Whether a completed **click** is reported at all.
    ///
    /// `false` makes [`super::GestureState::update`] swallow the click and the
    /// double-click, which is what stops a click in Read reaching the
    /// selection. It is asked separately from [`Self::drag`] because a measure
    /// tool needs the click while having no drag, and a mode that cannot select
    /// content must still let that click through.
    pub click: bool,
}

impl PressMeaning {
    /// A press that means nothing at all — no drag, no click.
    ///
    /// What Read grants an un-armed canvas, and the value every test that is
    /// about *refusal* names rather than spelling two fields.
    pub const NOTHING: Self = Self {
        drag: None,
        click: false,
    };

    /// A press that starts `kind` on a drag and reports a click otherwise —
    /// what an ordinary permitted press means.
    ///
    /// A constructor rather than a literal because it is what almost every test
    /// of the state machine wants, and those tests are about press/drag/release
    /// rather than about modes.
    #[must_use]
    pub const fn dragging(kind: DragKind) -> Self {
        Self {
            drag: Some(kind),
            click: true,
        }
    }

    /// A press that reports a click and starts no drag — what an armed measure
    /// tool means.
    #[must_use]
    pub const fn clicking() -> Self {
        Self {
            drag: None,
            click: true,
        }
    }
}

#[must_use]
pub fn press_kind(
    tool: CanvasTool,
    grip: Option<Grip>,
    zoom_armed: bool,
    caps: Capabilities,
) -> PressMeaning {
    // ★ A measure tool takes the click and leaves the drag alone.
    //
    // Highest precedence, above the markup tool, for the same reason the
    // markup tool sits above the grips: the operator armed it, and it is the
    // claimant whose loss would be silent. It cannot actually contend with
    // markup — one tool is armed at a time, by construction — so the ordering
    // is a statement rather than a tie-break.
    //
    // `drag: None` is the honest answer and not a stub. A ce dimension is
    // authored by clicks: point A, point B, then a third click saying how far
    // off the geometry the dimension sits. There is no drag in that gesture, so
    // there is no `DragKind` for one, and inventing a `DragKind::Measure` that
    // every arm then ignored would be a placeholder — which this project's
    // no-placeholders invariant forbids, and which would put a rubber band on
    // screen promising a gesture nothing implements.
    if tool.measure_kind().is_some() {
        return PressMeaning {
            drag: None,
            click: caps.author_measure,
        };
    }
    // ★ …and the **caret** tool does the same third time, which is what makes
    // this rung a family rather than three special cases.
    //
    // A caret is placed, not dragged: one click says *where*, and the keyboard
    // says the rest. There is no drag in that gesture, so `drag: None` is the
    // honest answer and a `DragKind::TextEdit` that every arm ignored would put a
    // rubber band on screen promising a gesture nothing implements — the
    // placeholder this project's no-placeholders invariant forbids, and the
    // argument the two rungs around it make in their own words.
    //
    // The capability is `edit_content`, where the measure rung reads
    // `author_measure` and the vertex rung `author_markup`, and the difference is
    // the whole point: those two author a dimension and a comment, which sit
    // *over* the page. This rewrites the page's own show operators. So a mode
    // that offers Markup and not content editing — which is the row
    // `MODES_AND_PANELS.md`'s gesture table calls Review — places dimensions and
    // comments and still may not put a caret in a word.
    //
    // ★ It sits **above** the text-selection question below rather than beside
    // it, and that ordering is load-bearing in the direction that is easy to get
    // backwards. `textsel::takes_the_press` is false for this tool by
    // construction (it asks `is_text`, which is `matches!(tool, Text)`), so the
    // two cannot contend today — but `caps.edit_content` is *true* in the only
    // mode this tool arms in, so if that predicate ever grew a disjunct for the
    // caret tool the content branch below would answer first and a press would
    // marquee objects under an I-beam. Returning early is what makes that
    // unreachable rather than merely unlikely.
    // ★ …and a text-annotation tool splits BOTH ways, which is why it needs its
    // own rung rather than joining either family above.
    //
    // A text box and a stamp are **dragged** — the operator is choosing how
    // wide the words are — so they want a `DragKind`. A sticky note is
    // **clicked**, because its rect is fixed-size and `NoZoom` and the format
    // discards whatever was dragged; asking for a width would be asking for a
    // number nobody reads.
    //
    // `is_dragged` is the predicate the whole family branches on, so the two
    // shapes cannot drift apart here from the way they are authored — the same
    // welding `uses_gallery` does for the stamp's text.
    if let CanvasTool::TextAnnot(kind) = tool {
        return PressMeaning {
            drag: kind.is_dragged().then_some(DragKind::TextAnnot(kind)),
            // The click is live for the sticky and, deliberately, ALSO for the
            // dragged kinds: a click with the text-box tool armed is a
            // zero-area drag, and answering `false` would make it fall through
            // to the marquee underneath — an operator who meant to place a
            // callout and twitched would select objects instead.
            click: caps.author_markup,
        };
    }
    if tool.text_edit_kind().is_some() {
        return PressMeaning {
            drag: None,
            click: caps.edit_content,
        };
    }
    // ★ …and a **vertex** markup tool does exactly the same thing, for exactly
    // the same reason — which is why it is written here, immediately beside it,
    // rather than as a special case inside the markup rung below.
    //
    // PolyLine and Polygon are picked, not dragged: click each corner, then say
    // when. There is no drag in that gesture, so there is no `DragKind` for one,
    // and `drag: None` is the honest answer rather than a stub. Inventing a
    // `DragKind::Markup` that every arm then ignored would put a rubber band on
    // screen promising a gesture nothing implements — the placeholder this
    // project's no-placeholders invariant forbids, and the same argument the
    // measure branch above makes in its own words.
    //
    // The capability is `author_markup` where the measure branch reads
    // `author_measure`: these two kinds author a **comment**, not a dimension,
    // so a mode that offers Markup and not Measure must still place them. That
    // is the row `MODES_AND_PANELS.md`'s gesture table calls Review.
    //
    // It sits ABOVE the markup rung rather than inside it because this is a
    // question about the *shape* of the gesture and that rung is a question
    // about precedence. Folding it in would mean the markup rung returned a
    // `drag` for four kinds and `None` for two while a single `click` field was
    // decided fifteen lines further down for all six — which is exactly the
    // arrangement `PressMeaning`'s own header records as the gate's first,
    // wrong, shape.
    if let Some(kind) = tool.markup_kind()
        && kind.is_vertex()
    {
        return PressMeaning {
            drag: None,
            click: caps.author_markup,
        };
    }
    // ★ Does the press mean TEXT? Asked once, here, and asked again by
    // `canvas::interact` when it routes the click — through the same function,
    // so the drag's meaning and the click's routing cannot drift apart. See
    // this function's ★ section on the text row.
    let text = crate::canvas::textsel::takes_the_press(tool, caps);
    // ★ The two worlds, split on the one flag that separates them, rather than
    // one `match` with a capability test on every arm.
    //
    // It used to be the latter, and the text row is what showed why that was the
    // wrong shape: with `edit_content` false, **three of the four arms were
    // dead** — a grip is drawn only for a content selection, so in a mode that
    // cannot make one there is no grip to hover, no resize and no move. Writing
    // them as `caps.edit_content.then_some(…)` inside one match left those dead
    // arms answering `None` and swallowing the press, so a hypothetical grip in
    // Read produced *no meaning at all* where every other press in Read means
    // text. Unreachable, and incoherent — and the incoherence is the kind that
    // becomes reachable the day something else changes.
    //
    // Split, each branch says one thing. The content branch is byte-for-byte the
    // precedence that shipped: markup, grip, armed zoom, marquee. The reading
    // branch is the armed zoom and then text, and it has no grip arm because
    // there are no grips.
    let drag = if let Some(kind) = tool.markup_kind() {
        caps.author_markup.then_some(DragKind::Markup(kind))
    // ★ **The armed TEXT tool, and it sits above the content branch on purpose.**
    //
    // This is the one rung the text-tool work added, and its placement is the
    // whole of what it does. Below `caps.edit_content` it would be dead in Edit —
    // the mode the tool exists for — because that branch is total: every value of
    // `grip` and `zoom_armed` produces a meaning there, so nothing after it is
    // reachable while the mode can select content. A tool that armed, painted an
    // I-beam and marqueed objects is the *"visible control, silently inert"*
    // failure with an extra insult.
    //
    // Above it, the rule reads the same as every other armed tool's: **the press
    // belongs to whichever tool is armed.** That is not a rule invented here —
    // `Markup` above has relied on it since it landed, for the reason its own
    // rung states — and it is what replaces the by-construction exclusivity the
    // old single-disjunct rule had. `canvas::textsel` §3 records that move from
    // construction to precedence, including the consequence that an object
    // selection and a text selection can now both be non-empty in Edit.
    //
    // ★ The region zoom is the one thing it yields to, and the ordering is
    // **borrowed rather than decided**: the reading-mode text row four branches
    // below already yields to `zoom_armed`, on the argument that the zoom is a
    // one-shot the operator armed deliberately from the ribbon and that a text
    // sweep is back on the very next press. Nothing about that argument mentions
    // *why* the press means text, so applying it to the armed tool as well keeps
    // one rule where two would otherwise appear — and the alternative would be an
    // operator whose armed zoom silently stops working for as long as the text
    // tool is down, with the zoom control still rendering pressed on another tab
    // where they cannot see it. That is exactly the "spending an Escape on
    // something inert" hazard `canvas::keys`' header argues about, in the
    // pointer's tense.
    //
    // Note this differs from `Markup` above, which outranks the zoom. The
    // distinction is the one that rung already draws: markup **authors**, so the
    // loss of its drag would be a mark that was never made, while a text sweep
    // loses nothing an operator cannot re-make with one more drag.
    } else if tool.is_text() {
        Some(if zoom_armed {
            DragKind::Marquee(MarqueeIntent::Zoom)
        } else {
            DragKind::TextSelect
        })
    } else if caps.edit_content {
        match grip {
            Some(grip) if grip.is_resize() => Some(DragKind::Resize(grip)),
            Some(_) => Some(DragKind::Move),
            None if zoom_armed => Some(DragKind::Marquee(MarqueeIntent::Zoom)),
            None => Some(DragKind::Marquee(MarqueeIntent::Select)),
        }
    // ★ An armed region zoom outranks a text sweep, and that ordering is the
    // operator's own arming decision rather than a preference: the zoom is a
    // one-shot they armed *deliberately* from the ribbon, and a reading mode is
    // exactly where they are most likely to have armed it. A text sweep is the
    // un-armed default and is back on the very next press.
    } else if zoom_armed {
        Some(DragKind::Marquee(MarqueeIntent::Zoom))
    } else if text {
        Some(DragKind::TextSelect)
    } else {
        // A reading mode with something armed that is neither — reachable only
        // through a manifest that grants markup or measure without the tab this
        // function reads. Nothing to offer, and saying so is better than
        // guessing.
        None
    };
    // Every remaining tool's click means *select what is under the pointer*,
    // which is the content capability. Including the markup tool's: a click
    // with a pen armed places nothing (a degenerate drag is refused by
    // `markup::drag`), so the click falls through to the selection exactly as
    // it did before this gate existed. That is behaviour carried across
    // deliberately rather than decided here — see `canvas::markup`.
    //
    // ★ …and the text press reports a click too, because a text gesture's
    // click carries three of its four meanings: double-click takes a word,
    // triple-click takes a line, Shift+click extends, and a plain click clears.
    // Suppressing it would leave a drag that selects and no way to unselect —
    // and it would leave the two most familiar text gestures in the product
    // class unreachable. Where that click is *routed* is `canvas::interact`'s
    // business; it asks `textsel::takes_the_press` again rather than inferring
    // it from this flag, because this flag is true for two different reasons.
    PressMeaning {
        drag,
        click: caps.edit_content || text,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::canvas::textedit::TextEditKind;

    /// ★★ **The caret tool takes the click, leaves the drag, and needs
    /// `edit_content`** — its whole rung, over every capability combination.
    ///
    /// Three claims in one loop, and each fails against a different plausible
    /// wrong implementation:
    ///
    /// * `drag.is_none()` fails a build that gave the tool a `DragKind` "for
    ///   symmetry" — which would put a rubber band on screen promising a
    ///   gesture nothing implements;
    /// * `click == edit_content` fails a build that copied the measure rung and
    ///   left `author_measure` in it — which would arm the caret in Review and
    ///   refuse it in Edit, i.e. exactly backwards;
    /// * the zoom assertion fails a build in which the rung was placed *below*
    ///   the armed-zoom branch, where a press would rubber-band a zoom region
    ///   under an I-beam.
    ///
    /// Over the whole capability lattice rather than the three shipped modes,
    /// for the reason this module's other tests are: a mode is a manifest entry
    /// and can be customized, and the rule is about the flags.
    #[test]
    fn the_caret_tool_clicks_and_never_drags_and_needs_edit_content() {
        for edit_content in [false, true] {
            for author_markup in [false, true] {
                for author_measure in [false, true] {
                    let mut caps = Capabilities::NONE;
                    caps.edit_content = edit_content;
                    caps.author_markup = author_markup;
                    caps.author_measure = author_measure;
                    for kind in [TextEditKind::Edit, TextEditKind::Add] {
                        let m = press_kind(CanvasTool::TextEdit(kind), None, false, caps);
                        assert!(m.drag.is_none(), "a caret is placed, not dragged");
                        assert_eq!(
                            m.click, edit_content,
                            "the caret needs `edit_content` and nothing else"
                        );
                    }
                    let zoomed =
                        press_kind(CanvasTool::TextEdit(TextEditKind::Edit), None, true, caps);
                    assert!(zoomed.drag.is_none());
                }
            }
        }
    }

    // -----------------------------------------------------------------
    // What a press means
    // -----------------------------------------------------------------

    /// ★ **The armed markup tool outranks the grips and the region zoom.**
    ///
    /// Both rows matter and both are failure modes with teeth: a markup drag
    /// classified as a `Resize` would be consumed and author nothing (a tool
    /// that arms and does nothing over any selected object), and one classified
    /// as a zoom marquee would zoom the page instead of drawing.
    #[test]
    fn an_armed_markup_tool_outranks_the_grips_and_the_region_zoom() {
        let armed = CanvasTool::Markup(MarkupKind::Rectangle);
        for grip in [None, Some(Grip::SouthEast), Some(Grip::Move)] {
            for zoom in [false, true] {
                assert_eq!(
                    press_kind(armed, grip, zoom, Capabilities::FULL),
                    PressMeaning::dragging(DragKind::Markup(MarkupKind::Rectangle)),
                    "grip={grip:?} zoom_armed={zoom}"
                );
            }
        }
    }

    /// …and with no markup armed, the precedence is exactly what it was before
    /// the markup tool existed. Without this, the test above would pass on a
    /// build where every press had become a markup.
    #[test]
    fn without_a_markup_tool_the_press_precedence_is_unchanged() {
        let select = CanvasTool::Select;
        let full = Capabilities::FULL;
        assert_eq!(
            press_kind(select, Some(Grip::SouthEast), false, full),
            PressMeaning::dragging(DragKind::Resize(Grip::SouthEast))
        );
        assert_eq!(
            press_kind(select, Some(Grip::Move), false, full),
            PressMeaning::dragging(DragKind::Move)
        );
        assert_eq!(
            press_kind(select, None, true, full),
            PressMeaning::dragging(DragKind::Marquee(MarqueeIntent::Zoom))
        );
        assert_eq!(
            press_kind(select, None, false, full),
            PressMeaning::dragging(DragKind::Marquee(MarqueeIntent::Select))
        );
        // A grip beats an armed zoom, as it always did.
        assert_eq!(
            press_kind(select, Some(Grip::SouthEast), true, full),
            PressMeaning::dragging(DragKind::Resize(Grip::SouthEast))
        );
    }

    // -----------------------------------------------------------------
    // The mode gate
    // -----------------------------------------------------------------

    /// ★ **A mode that cannot edit content gives every content press no
    /// meaning** — and leaves the region zoom alone.
    ///
    /// This is the operator's ask (*"in read mode the document shouldn't allow
    /// editing"*) at the point where it is decided. Every one of the four
    /// content meanings is asserted, because they are four separate arms and
    /// gating three of them would look exactly like gating all four right up
    /// until someone dragged a grip.
    ///
    /// ★ **The bare press is no longer `NOTHING`, and that is the text-selection
    /// row arriving.** It used to assert *"no marquee-select, and no selecting
    /// click either"* against `PressMeaning::NOTHING`, which was the right
    /// assertion while Read had no press meaning at all — and would be the wrong
    /// one now, because it would pass on a build that had silently taken text
    /// selection away again. What must remain true is the thing the operator
    /// actually asked for: the press means **text**, never
    /// [`DragKind::Marquee`], so nothing on the page can be selected as
    /// *content*. That is asserted by naming the variant rather than by
    /// asserting an absence.
    ///
    /// The region-zoom row is the one that would be easy to get wrong in the
    /// other direction: marquee-**zoom** is navigation, it is armed
    /// deliberately, and refusing it would take a viewer feature away from the
    /// viewing mode.
    #[test]
    fn read_mode_gives_a_content_press_no_meaning_but_keeps_the_region_zoom() {
        let select = CanvasTool::Select;
        let read = Capabilities::NONE;
        // ★ Asserted as the ABSENCE OF EVERY CONTENT MEANING rather than as
        // `PressMeaning::NOTHING`, and the change of shape is the point.
        //
        // `NOTHING` was the right assertion while Read had no press meaning at
        // all. It is the wrong one now, twice over: it would fail against the
        // feature the operator asked for, and — worse — a build that had taken
        // text selection away again would make it *pass*. What the operator
        // actually asked for is that nothing on the page can be selected,
        // moved or resized, so that is what is asserted, by naming the meanings
        // that must not appear.
        //
        // The grip rows are unreachable in practice (a grip is drawn only for a
        // content selection, which Read cannot make) and are checked anyway,
        // because "it is safe because nothing can be selected" is an argument
        // that holds only for as long as its other half does, and its other
        // half is in a different file — `HANDOFF.md` §2's lesson about a test
        // that checks a relation rather than a magnitude.
        for grip in [None, Some(Grip::SouthEast), Some(Grip::Move)] {
            let meaning = press_kind(select, grip, false, read);
            assert!(
                !matches!(
                    meaning.drag,
                    Some(
                        DragKind::Resize(_)
                            | DragKind::Move
                            | DragKind::Marquee(MarqueeIntent::Select)
                            | DragKind::Markup(_)
                    )
                ),
                "Read gave a content meaning to a press over {grip:?}: {meaning:?}"
            );
        }
        assert_eq!(
            press_kind(select, None, false, read),
            PressMeaning {
                drag: Some(DragKind::TextSelect),
                click: true,
            },
            "a bare press in a reading mode sweeps TEXT — never content"
        );
        assert_eq!(
            press_kind(CanvasTool::Markup(MarkupKind::Arrow), None, false, read),
            PressMeaning::NOTHING,
            "no markup, even with the tool somehow armed — and no text either, \
             because an armed pen keeps its own press"
        );
        assert_eq!(
            press_kind(select, None, true, read),
            PressMeaning {
                drag: Some(DragKind::Marquee(MarqueeIntent::Zoom)),
                click: true,
            },
            "a region zoom is navigation and survives every mode; it outranks the \
             text sweep because the operator armed it"
        );
    }

    /// ★ **No press ever means both a text sweep and a content marquee.**
    ///
    /// The exclusivity `canvas::textsel`'s header §3 rests on, asserted at the
    /// point where a press is given its meaning rather than only at the
    /// predicate that decides it. A build in which both were reachable would
    /// have one primary button with two meanings and no rule to choose between
    /// them — which is the ambiguity `CanvasTool::Text` exists to remove.
    ///
    /// ★ **Two tools now, where this used to walk `Select` alone**, and the
    /// difference is the point rather than extra coverage:
    ///
    /// * with **Select**, the guarantee is the original one — exclusive *by
    ///   construction*, because `takes_the_press` and `content_gesture` read the
    ///   same flag in opposite senses, so exactly one of the two is offered;
    /// * with **Text**, the guarantee is *by precedence* — both underlying facts
    ///   can be true in Edit, and rung 2 decides. So the assertion there is not
    ///   an exclusive-or but the stronger and more specific one: the drag is
    ///   `TextSelect` and **never** a content meaning, in every mode.
    ///
    /// Written as one test rather than two because the property is one property
    /// — *one press, one meaning* — and splitting it would let a future reader
    /// change the branch order and fix only the half that failed.
    #[test]
    fn no_press_offers_both_a_text_sweep_and_a_content_marquee() {
        for caps in [
            Capabilities::NONE,
            Capabilities::FULL,
            Capabilities {
                edit_content: false,
                author_markup: true,
                author_measure: true,
            },
        ] {
            let text = matches!(
                press_kind(CanvasTool::Select, None, false, caps).drag,
                Some(DragKind::TextSelect)
            );
            let content = matches!(
                press_kind(CanvasTool::Select, None, false, caps).drag,
                Some(DragKind::Marquee(MarqueeIntent::Select))
            );
            assert!(text ^ content, "exactly one, for {caps:?}");

            // …and with the tool armed, the answer is text in every one of them,
            // whatever the pointer is over. The grip rows are what a build that
            // put the new rung *below* the content branch would fail — silently,
            // and only in Edit.
            for grip in [None, Some(Grip::SouthEast), Some(Grip::Move)] {
                assert_eq!(
                    press_kind(CanvasTool::Text, grip, false, caps),
                    PressMeaning {
                        drag: Some(DragKind::TextSelect),
                        click: true,
                    },
                    "an armed text tool sweeps text over {grip:?} in {caps:?}"
                );
            }
        }
    }

    /// ★ **The armed text tool takes the press in EDIT** — the row the whole
    /// tool exists for, asserted by itself so that a failure names it.
    ///
    /// `Capabilities::FULL` is Edit, whose primary drag is the content marquee.
    /// Every content meaning must be absent, and the click must still be
    /// reported — because three of the text gesture's four meanings are clicks
    /// (double-click takes a word, triple-click a line, Shift+click extends, a
    /// plain click clears), and a build that suppressed it would leave a sweep
    /// that selects and no way to unselect.
    ///
    /// The second half asserts the thing that must **not** have changed: with the
    /// tool retired, the same mode's press is the marquee it always was. Without
    /// it, a build that had simply deleted the mode gate would pass the first
    /// half perfectly while having removed the only content-selection gesture the
    /// product has.
    #[test]
    fn the_text_tool_sweeps_in_edit_and_retiring_it_gives_the_marquee_back() {
        let edit = Capabilities::FULL;
        assert_eq!(
            press_kind(CanvasTool::Text, None, false, edit),
            PressMeaning {
                drag: Some(DragKind::TextSelect),
                click: true,
            },
            "Edit is the mode this tool was built for"
        );
        assert_eq!(
            press_kind(CanvasTool::Select, None, false, edit),
            PressMeaning::dragging(DragKind::Marquee(MarqueeIntent::Select)),
            "…and putting it down restores the content marquee unchanged"
        );
        // A resize grip is still a resize with the tool down — the precedence
        // below rung 2 is untouched.
        assert_eq!(
            press_kind(CanvasTool::Select, Some(Grip::SouthEast), false, edit),
            PressMeaning::dragging(DragKind::Resize(Grip::SouthEast)),
        );
    }

    /// ★ **An armed region zoom outranks the armed text tool, and an armed pen
    /// outranks the zoom.**
    ///
    /// The two orderings around rung 2, asserted together because they point in
    /// opposite directions and the reason is stated once, at the branch: markup
    /// **authors**, so the loss of its drag is a mark that was never made, while
    /// a text sweep loses nothing an operator cannot re-make with one more drag —
    /// and the zoom is a one-shot the operator armed deliberately from the
    /// ribbon, spent by the very next drag.
    ///
    /// The text half is not a new rule: the *un-armed* reading-mode text row has
    /// yielded to the zoom since it shipped, and this asserts the armed tool
    /// borrows that ordering rather than inventing a second one. Both modes are
    /// covered, because a build that consulted `caps.edit_content` while
    /// deciding would answer differently in each.
    #[test]
    fn a_region_zoom_outranks_the_text_tool_but_not_a_pen() {
        for caps in [Capabilities::NONE, Capabilities::FULL] {
            assert_eq!(
                press_kind(CanvasTool::Text, None, true, caps),
                PressMeaning {
                    drag: Some(DragKind::Marquee(MarqueeIntent::Zoom)),
                    click: true,
                },
                "the zoom is a one-shot the operator armed; the text tool is back next press \
                 ({caps:?})"
            );
        }
        assert_eq!(
            press_kind(
                CanvasTool::Markup(MarkupKind::Rectangle),
                None,
                true,
                Capabilities::FULL
            ),
            PressMeaning::dragging(DragKind::Markup(MarkupKind::Rectangle)),
            "a pen still outranks the zoom, because a mark that is never made is a silent loss"
        );
    }

    /// ★ **A vertex markup tool takes the CLICK and offers no drag** — the row
    /// added on 2026-08-14, and the one a build that folded PolyLine into the
    /// band rung would fail.
    ///
    /// Three claims, and each has a distinct failure:
    ///
    /// * **`drag` is `None`** — a build that gave these a `DragKind::Markup`
    ///   would put a rubber band on screen for a gesture nothing implements, and
    ///   `band::drag`'s family guard would then draw and author nothing, so the
    ///   operator would see a band appear and vanish on every press.
    /// * **`click` is live**, gated on `author_markup` — a build that reused the
    ///   general `caps.edit_content || text` tail would leave these two tools
    ///   inert in **Review**, which is the mode a reviewer draws a cloud-shaped
    ///   polygon in, and would leave them placing vertices in Read.
    /// * **The grips and the armed zoom do not change the answer**, because the
    ///   early return is above both. A vertex click that fell through to the
    ///   marquee rung would place no vertex and replace the selection instead.
    #[test]
    fn a_vertex_markup_tool_takes_the_click_and_offers_no_drag() {
        let review = Capabilities {
            edit_content: false,
            author_markup: true,
            author_measure: true,
        };
        for kind in [MarkupKind::PolyLine, MarkupKind::Polygon] {
            let armed = CanvasTool::Markup(kind);
            for grip in [None, Some(Grip::SouthEast), Some(Grip::Move)] {
                for zoom in [false, true] {
                    for caps in [review, Capabilities::FULL] {
                        assert_eq!(
                            press_kind(armed, grip, zoom, caps),
                            PressMeaning::clicking(),
                            "{kind:?} grip={grip:?} zoom={zoom} {caps:?}"
                        );
                    }
                }
            }
            // …and a mode that cannot author markup gives it nothing at all,
            // which is the same answer an armed band kind gets in Read.
            assert_eq!(
                press_kind(armed, None, false, Capabilities::NONE),
                PressMeaning::NOTHING,
                "{kind:?} in a mode that authors no markup"
            );
        }
        // The freehand kind is the other half of the same routing rule and must
        // go the OTHER way: Ink is drag-shaped, so it keeps the band rung's
        // answer. A build that classified by "is it in the new set of kinds?"
        // rather than by the gesture would break exactly here.
        assert_eq!(
            press_kind(
                CanvasTool::Markup(MarkupKind::Ink),
                None,
                false,
                Capabilities::FULL
            ),
            PressMeaning::dragging(DragKind::Markup(MarkupKind::Ink)),
            "freehand is a DRAG, whatever else it shares with the vertex kinds"
        );
    }

    /// ★ **Review places markup and does not touch content** — the middle row
    /// of `MODES_AND_PANELS.md`'s gesture table, which is the row that proves
    /// the gate is per-capability rather than a single on/off.
    #[test]
    fn review_mode_places_markup_but_refuses_content() {
        let review = Capabilities {
            edit_content: false,
            author_markup: true,
            author_measure: true,
        };
        assert_eq!(
            press_kind(
                CanvasTool::Markup(MarkupKind::Rectangle),
                None,
                false,
                review
            ),
            PressMeaning {
                drag: Some(DragKind::Markup(MarkupKind::Rectangle)),
                click: false,
            },
            "a reviewer draws their own markup, and their click selects nothing"
        );
        assert!(
            !matches!(
                press_kind(CanvasTool::Select, Some(Grip::Move), false, review).drag,
                Some(DragKind::Move | DragKind::Resize(_))
            ),
            "a reviewer does not move the page's own content"
        );
        assert_eq!(
            press_kind(CanvasTool::Select, None, false, review),
            PressMeaning {
                drag: Some(DragKind::TextSelect),
                click: true,
            },
            "…and does not marquee-select it either — with the pen down, a \
             reviewer's bare press sweeps text, which is what an underline or a \
             strikeout will need"
        );
    }
}
