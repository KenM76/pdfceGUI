//! # `canvas::tool` — which pointer tool the canvas is in, and the space bar that borrows it
//!
//! ## What this module is for
//!
//! `GUI_ROADMAP.md` Phase 3.2: *"There is no hand tool at all; panning is
//! middle-drag only."* This is the hand tool, and the space bar that borrows
//! it for as long as it is held.
//!
//! It owns exactly one question — **what does the primary button mean right
//! now?** — and answers it as a pure function of two inputs: the tool the
//! operator *chose* ([`selected`]) and whether the space bar is *down*
//! ([`space_held`]). Everything else in `canvas/` reads [`active`] and
//! branches on the answer.
//!
//! ## ★ Why the space override is derived and never stored
//!
//! The requirement is *"space held = temporary pan, releasing returns to the
//! previous tool"*, and the obvious implementation — remember the previous
//! tool on key-down, restore it on key-up — is the one that fails. It fails
//! in the ordinary way (an interrupted key-up: the window loses focus mid-pan,
//! the operator alt-tabs, a dialog steals the release) and the failure is
//! *sticky*: the canvas is left in a hand tool the operator never chose and
//! cannot leave except by choosing something else. Every application that has
//! ever shipped a modal space-pan has shipped that bug at least once.
//!
//! So there is **no stored override and nothing to restore**. [`selected`] is
//! the only persistent value; the space bar is read fresh from
//! [`egui::InputState`] on every frame and composed with it by [`resolve`].
//! "Returning to the previous tool" is then not an action that can be missed —
//! it is what the next frame computes when the key is no longer down. A lost
//! key-up costs one frame of pan, not a stuck mode.
//!
//! ## ★ The text-field guard is not optional
//!
//! Space is a *character*. A canvas that panned on any Space keypress would
//! pan while the operator typed a page number into the status bar's page box
//! or a value into the Properties panel. The guard is
//! [`egui::Context::text_edit_focused`] — the same predicate, for the same
//! reason, as `DEFECTS.md` D1's Delete-key fix, and deliberately **not**
//! `egui_wants_keyboard_input()`, which is true whenever *any* widget has
//! focus and would therefore disable space-pan after a single click on the
//! canvas (the canvas takes focus on click, which is exactly how D1 happened).
//!
//! ## ★ This header used to say there would never be a third variant. What
//! changed
//!
//! Until the markup substrate landed, [`CanvasTool`]'s own doc comment read:
//!
//! > Deliberately two variants and not a general "tool" enum with markup,
//! > measure and text members. **Those are *modes* that arm a whole authoring
//! > surface and they will arrive with their own state**; this enum answers the
//! > narrow navigation question — does a primary drag select, or does it move
//! > the paper?
//!
//! That was right, and it is not being overturned — **its condition has been
//! met.** The sentence set a bar for admission ("arrives with their own
//! state"), and markup now clears it: it arrives with [`markup::MarkupKind`],
//! with a `DragKind` and a `GestureOutcome` of its own in
//! [`crate::canvas::gesture`], with a rubber band, a commit path and an
//! `Action`. What it does *not* have — and this is the part that decided the
//! shape — is any state that outlives a frame except **which kind is armed**,
//! which is precisely one enum value and is exactly the kind of thing this
//! module already stores.
//!
//! So the enum grows by one variant *carrying* the kind, rather than by four,
//! and the question it answers grows by one word: **does a primary drag select,
//! move the paper, or draw?** The two rules that made the old sentence true are
//! both still enforced here rather than at call sites —
//! [`CanvasTool::pans_with_primary`] is still the single predicate the pan and
//! gesture-suppression paths share, and [`CanvasTool::cursor`] is still the
//! single place a tool's cursor is decided.
//!
//! ## ★ …and that paragraph then named two exclusions, both of which have since
//! ## been overtaken. What is left of it, and what replaced it
//!
//! It used to close: *"Measure and text are **still** outside, and for the
//! original reason rather than by inertia."* That sentence was stale twice over
//! by 2026-08-14 and is kept here in quotation rather than deleted, because the
//! **bar** it set is the useful part and both admissions were argued against it.
//!
//! **Measure came in first**, as [`CanvasTool::Measure`], and the old sentence's
//! objection to it — *"a two-point pick with a snap indicator and a live
//! readout"* — turned out to describe the pick machinery in
//! [`crate::canvas::measure::pick`] rather than anything this enum has to hold.
//! What crossed the boundary was one [`MeasureKind`], exactly as markup's one
//! [`markup::MarkupKind`] had.
//!
//! **Text selection came in second, and it clears the bar more cleanly than
//! either.** The bar is *"arrives with its own state"*, and the standing set is:
//!
//! | it arrives with | where |
//! |---|---|
//! | a selection type, with its own staleness rule | [`crate::canvas::textsel::TextSelection`] |
//! | a [`PressMeaning`](crate::canvas::gesture::PressMeaning) and a `DragKind` | [`crate::canvas::gesture::DragKind::TextSelect`] |
//! | a resolver — one pass producing the string, the canvas boxes and the page quads | `canvas::textsel::resolve`, reached through `drag` / `click` / `select_all` |
//! | a commit path, in the only sense it has one: three markup kinds whose operand is the selection | [`crate::canvas::markup::text`] |
//! | two keyboard verbs of its own | [`crate::canvas::textsel::clipboard`] |
//!
//! …and **the only thing it needs to persist is that it is armed.** Not a range,
//! not a caret, not an anchor: the range lives on the document beside the object
//! selection, the anchor is re-derived from the press origin on every frame of a
//! sweep (`textsel::drag`'s own header says why that is exact rather than lazy),
//! and there is no caret at all (`textsel` §1.2 — a caret promises an insertion
//! point, and there is nothing to insert). So the variant carries **nothing**,
//! where `Markup` and `Measure` each carry a kind. That is the smallest thing
//! this enum can be asked to hold and still be worth holding.
//!
//! What the admission *buys* is two things at once, and the second is the one
//! that made it urgent. `canvas::textsel` §3 gave a press its text meaning
//! *"when the select tool is active and the mode cannot select content"*, which
//! yields Read ✓, Review ✓, **Edit ✗** — so a reviewer could sweep text and an
//! editor could not, and, worse, the three text-markup controls drawn on Edit's
//! Markup tab could **never enable**, because `selection.text` was never true
//! there. That is a live tension with `RIBBON_IA.md` P3, which reserves greying
//! for *temporarily* unavailable, and it could not be closed by hiding the
//! controls, because a command lives on exactly one tab and the Markup tab is in
//! both Review and Edit. One variant closes both.
//!
//! ### ★ The reference applications DISAGREE here, and Inkscape wins
//!
//! `HANDOFF.md` §3's standing instruction is to match Inkscape, Acrobat and
//! SolidWorks, and to say which won where they disagree. On this question they
//! genuinely do:
//!
//! * **Acrobat and SolidWorks resolve text-versus-object *contextually*, within
//!   one tool** — hover text, get an I-beam; hover an object, get an arrow.
//! * **Inkscape uses a separate Text tool**, distinct from its Selector.
//!
//! **Inkscape wins, and the reason is not a head-count.** An object marquee over
//! *vector content* is a surface Acrobat does not have at all: its "objects" are
//! annotations and form fields, never the page's own path and text operators, so
//! its contextual answer is not an answer to this conflict — it never has the
//! conflict. The conflict exists only in the Inkscape-shaped mode, which is what
//! makes Inkscape's resolution the applicable one rather than merely the
//! outvoted one.
//!
//! The concrete failure a contextual press would produce is the deciding
//! argument. In Edit the primary drag is the content marquee, and the commonest
//! gesture on a drawing sheet is a marquee over a *region* — which on any real
//! sheet contains text. Under a contextual rule that drag would mean "sweep
//! text" or "marquee objects" depending on whether the pixel under the button-
//! down happened to be inside a glyph's box, a distinction the operator cannot
//! see and cannot aim at. A tool makes the answer a thing they chose.
//!
//! ### What is STILL outside, and it is the half the old sentence was right about
//!
//! **Text *editing*** — Phase 5, the defect that began this project — remains
//! outside, and for exactly the original reason: it is a caret in a re-laid-out
//! box, it would drag a whole subsystem's state through this type, and
//! `HANDOFF.md` says in terms *do not start it early*. Selecting text and
//! editing text are different features with different state, and this variant is
//! the first one only. Whoever brings the second should have to make this
//! argument again, in this file.
//!
//! ## Where the state lives, and why `egui::Memory` is right here when it was
//! wrong for the selection
//!
//! `canvas/mod.rs`'s seam 1 records the selection being *moved out* of
//! `egui::Memory` because it is **document-scoped**: closing a document must
//! forget it, and `Memory` outlives documents. A tool is the opposite — it is
//! **application-scoped**, like the ribbon tab or the theme. An operator who
//! picks the hand tool, opens another drawing and finds themselves back in the
//! select tool would report that as a bug. So the tool stays in `Memory`
//! precisely *because* `Memory` outlives documents, which is the property that
//! disqualified it for the selection.

use egui::{CursorIcon, Key};

use crate::app::modes::Capabilities;
use crate::canvas::markup::MarkupKind;
use crate::canvas::measure::MeasureKind;

/// `egui::Memory` key for the operator's chosen pointer tool.
const TOOL_MEMORY_KEY: &str = "pdfce-canvas-tool"; // ui-text-exempt: internal memory id, never displayed

/// What the primary button does over the page.
///
/// **Does a primary drag select, move the paper, or draw?** — the only question
/// the pan, marquee and markup paths need settled, and settling it here keeps
/// them from inventing three different answers.
///
/// Four variants, not nine: [`Self::Markup`] and [`Self::Measure`] each carry
/// **which** kind is armed rather than there being one variant per shape or per
/// dimension. See those variants' docs for the argument, and the module header
/// for what changed since this enum said it would stay at two.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CanvasTool {
    /// Click selects, drag rubber-bands. The shipped behaviour, and the
    /// default.
    #[default]
    Select,
    /// Click does nothing, drag moves the paper under the viewport.
    Hand,
    /// Drag authors a markup annotation of the carried kind.
    ///
    /// # ★ One variant carrying a kind, not one variant per shape
    ///
    /// The old shell settled this and its reasoning is carried across intact
    /// (`D:\Dev\pdfce\crates\pdfce-gui\src\canvas.rs:232-244`):
    ///
    /// > All markup kinds live in `MarkupToolState::kind` rather than becoming
    /// > separate `CanvasTool` entries […] Separate entries would put
    /// > mutually-exclusive states into a type that can express all their
    /// > combinations.
    ///
    /// That last clause is the whole argument, and it is a statement about
    /// *types* rather than about tidiness: the operator is drawing exactly one
    /// shape, so a type that can say `Rectangle` and `Ellipse` at once — which
    /// four booleans, or four variants plus a "which is active" rule spread
    /// across call sites, both can — is a type whose illegal states have to be
    /// prevented by discipline. Carrying the kind makes them unrepresentable.
    ///
    /// It also makes the *tool* branch total: every rule this enum owns —
    /// [`Self::pans_with_primary`], [`Self::cursor`], the press-kind decision in
    /// [`crate::canvas::gesture::press_kind`] — is written once for markup as a
    /// whole and cannot be written four times and forgotten once.
    ///
    /// **Changing the kind mid-drag is not possible here**, where the old shell
    /// had to discard an in-progress gesture on a kind change. Arming is a
    /// command, commands are dispatched between frames, and a drag in flight is
    /// owned by [`crate::canvas::gesture::GestureState`], which carries the kind
    /// it started with on its own `DragKind` — so a kind change mid-drag cannot
    /// reach the drag at all. The property the old shell had to enforce, this
    /// one gets from the gesture machine's existing "a drag keeps the kind it
    /// started with" rule.
    Markup(MarkupKind),
    /// Clicks author a **dimension** of the carried kind.
    ///
    /// One variant carrying a kind, for the argument spelled out on
    /// [`MeasureKind`] — which is the same argument `Markup` above makes, and
    /// which the old shell did *not* apply here: it had three separate
    /// `CanvasTool` variants for the measure tools plus five helper predicates
    /// to ask which was active.
    ///
    /// # ★ Unlike every other tool, this one works on CLICKS
    ///
    /// A ce dimension is picked, not dragged: point A, point B, and a third
    /// click saying how far off the geometry the dimension sits. So
    /// [`crate::canvas::gesture::press_kind`] returns a
    /// [`crate::canvas::gesture::PressMeaning`] with **no drag and a live
    /// click** for this tool, and the pick state machines in
    /// [`crate::canvas::measure::pick`] advance one click at a time.
    ///
    /// That is why the mode gate had to grow two answers rather than one. A
    /// gate that suppressed the click whenever it suppressed the drag would be
    /// correct in Read — where neither means anything — and would silently
    /// break **Review**, whose whole purpose is placing dimensions on a page
    /// whose content is not the reviewer's to select.
    Measure(MeasureKind),
    /// A drag sweeps a **range of text**, and a click resolves one — in every
    /// mode, including the ones whose primary button is otherwise the content
    /// marquee.
    ///
    /// # ★ The variant that carries nothing, and why that is the point
    ///
    /// `Markup` and `Measure` each carry a kind because the operator is drawing
    /// exactly one shape or placing exactly one dimension, and a type that could
    /// say two at once would need discipline to keep honest. There is no
    /// corresponding choice here: text selection has **one** meaning, so there is
    /// nothing to carry, and the module header's admission bar — *"arrives with
    /// its own state"* — is met by state that lives everywhere except in this
    /// enum. The range is on [`crate::app::state::OpenDoc`] beside the object
    /// selection; the sweep's anchor is re-derived from the press origin every
    /// frame; there is no caret. **Armed or not armed** is the whole of what has
    /// to persist, and that is exactly one bit in a value this module already
    /// stores in `egui::Memory`.
    ///
    /// # What arming it changes, and what it deliberately does not
    ///
    /// It changes exactly one predicate:
    /// [`crate::canvas::textsel::takes_the_press`] gains a disjunct, so a press
    /// means text when this is armed **or** under the pre-existing rule (the
    /// select tool, in a mode that cannot select content). Read and Review are
    /// therefore **unchanged** — their select tool already swept text, and an
    /// operator who never presses this control will not notice it exists. What
    /// changes is Edit, where the select tool keeps the content marquee and this
    /// tool is how an editor reaches a text range at all.
    ///
    /// It changes **no capability**. Selecting text authors nothing — it reads
    /// the page and writes to the clipboard, which is the operator's own
    /// *copying is not authoring* ruling of 2026-08-14 — so this tool is
    /// permitted in every mode, and [`retire_forbidden`] says so explicitly
    /// rather than by omission.
    ///
    /// # ★ It is exclusive with the content marquee by PRECEDENCE, where the
    /// mode rule was exclusive by construction
    ///
    /// This is the one property the addition genuinely weakens, so it is stated
    /// here rather than left to be discovered. Under the old rule the two
    /// meanings read the same flag on both sides of one branch — `edit_content`
    /// true meant content, false meant text — so no state could produce both and
    /// there was no ordering to get wrong. With this tool armed in Edit, *both*
    /// underlying facts are true: the mode can select content, and the operator
    /// has asked for text.
    ///
    /// The tie is broken where every other armed tool's is, in
    /// [`crate::canvas::gesture::press_kind`], by the rung that already reads
    /// **an armed tool takes the press**. That is not a new rule invented for
    /// this variant; it is the rule `Markup` has relied on since it landed, and
    /// the alternative — leaving the mode branch to win — would be a control that
    /// arms, shows an I-beam, and marquees objects.
    ///
    /// One consequence follows and is real: an object selection and a text
    /// selection can now both be non-empty at once, in Edit, which
    /// `canvas::textsel` §3 previously argued could never happen. That is why
    /// they are two fields on the document rather than one enum, and the shape
    /// turns out to have been right for a reason its own argument got wrong. See
    /// [`crate::canvas::keys`]'s rung 5, which orders them.
    Text,
}

impl CanvasTool {
    /// Whether a primary-button drag pans the view rather than reaching the
    /// gesture machine.
    ///
    /// The whole branch, in one predicate, so the pan path and the
    /// gesture-suppression path cannot disagree about which tool pans — a
    /// disagreement whose symptom would be a drag that pans **and** marquees,
    /// which is one of the two things this stage must not ship.
    ///
    /// The markup tool answers `false`, which is what makes a markup drag reach
    /// the gesture machine at all: `canvas::interact` hands that machine a
    /// **blank** frame whenever this is `true`. Space-to-pan still works over
    /// the markup tool, because [`resolve`] composes the held space bar *before*
    /// this is asked — so a held space bar borrows the hand out of the markup
    /// tool exactly as it does out of the select tool, and releasing it hands
    /// the markup tool back with nothing stored and nothing to restore.
    #[must_use]
    pub fn pans_with_primary(self) -> bool {
        matches!(self, Self::Hand)
    }

    /// The cursor this tool shows, or `None` to leave the cursor to whatever
    /// else the canvas is doing with it (a grip, a marquee, a move drag).
    ///
    /// `Grab` when the hand is available and `Grabbing` while it is closed, in
    /// the direction every browser, CAD package and image editor uses. The
    /// pair matters: the requirement is that the cursor *changes and changes
    /// back*, and a single hand cursor for both states would leave an operator
    /// unable to tell a hand tool that is working from one that has run out of
    /// scroll range — the exact ambiguity the middle-drag path's own
    /// `Grabbing` was added to remove.
    ///
    /// `Select` returns `None` rather than `Default`: returning a cursor here
    /// would overwrite the grip cursors that [`crate::canvas::handles`] sets
    /// for the eight resize handles, and a resize grip that loses its cursor
    /// is a grip nobody can find.
    ///
    /// `Markup` returns `Crosshair` in **both** states, and the sameness is
    /// deliberate where the hand's pair is deliberately different. The hand
    /// needs to distinguish "available" from "closed" because a pan that has
    /// run out of scroll range is otherwise indistinguishable from a pan that
    /// is not working; a markup drag has no such failure — the band under the
    /// pointer is the feedback, and a cursor that changed under it would
    /// compete with the thing it is describing. What the crosshair says is
    /// *"this canvas draws now"*, which is true from the moment the tool is
    /// armed until it is retired, and returning it also **suppresses the grip
    /// cursors** — correctly, because a markup drag over a selected object
    /// draws a shape rather than resizing anything.
    ///
    /// ★ `Text` returns `CursorIcon::Text` in both states, on the same argument
    /// the crosshair makes and with one extra consequence worth naming. The
    /// I-beam is what Acrobat, Inkscape and SolidWorks all show over selectable
    /// text, and [`cursor_for`]'s own note records why this shell would not
    /// paint it on **hover** — answering *"is there a glyph under the pointer?"*
    /// per frame is a hit test against the page's extraction on every frame the
    /// pointer moves, paid on canvases nobody is selecting on. Armed, the
    /// question does not arise: the tool is a statement about what the next drag
    /// means, so the cursor is constant while it is armed and costs nothing.
    /// That is precisely the "becomes free on the day a `CanvasTool::Text`
    /// lands" this pair of comments anticipated.
    ///
    /// It suppresses the grip cursors too, and here that is load-bearing rather
    /// than incidental: in Edit a content selection can be on the page *while*
    /// this tool is armed, and [`crate::canvas::gesture::press_kind`] gives the
    /// armed tool the press. A grip that still showed its resize cursor would be
    /// promising a gesture the press rule has already decided against — the
    /// exact mismatch [`retire_forbidden`] exists to prevent at the other end of
    /// a tool's life.
    #[must_use]
    pub fn cursor(self, dragging: bool) -> Option<CursorIcon> {
        match self {
            Self::Select => None,
            Self::Hand if dragging => Some(CursorIcon::Grabbing),
            Self::Hand => Some(CursorIcon::Grab),
            // A crosshair for both authoring tools, and for the same reason:
            // it says "this canvas places something now" and it suppresses the
            // grip cursors, which is correct because a click with a measure
            // tool armed picks a point rather than grabbing a handle.
            Self::Markup(_) | Self::Measure(_) => Some(CursorIcon::Crosshair),
            // …and an I-beam for the one tool that places nothing. The pointer
            // says which of the two things a drag on this page is about to do,
            // which is the whole reason the tool exists.
            Self::Text => Some(CursorIcon::Text),
        }
    }

    /// Which markup kind is armed, if any.
    ///
    /// The accessor `crate::app::PdfceApp::conditions` needs in order to render
    /// exactly one Markup button pressed, and the accessor
    /// [`crate::canvas::gesture::press_kind`] needs in order to decide what a
    /// press means. Both would otherwise write the same `if let` — which is how
    /// a canvas ends up drawing one shape while the ribbon says another.
    #[must_use]
    pub fn markup_kind(self) -> Option<MarkupKind> {
        match self {
            Self::Markup(kind) => Some(kind),
            _ => None,
        }
    }

    /// Which measure kind is armed, if any.
    ///
    /// [`Self::markup_kind`]'s twin, and it exists for the identical two
    /// callers: `crate::app::PdfceApp::conditions`, so exactly one Measure
    /// button renders pressed, and
    /// [`crate::canvas::gesture::press_kind`], so a click is offered to the
    /// pick machines instead of to the selection.
    #[must_use]
    pub fn measure_kind(self) -> Option<MeasureKind> {
        match self {
            Self::Measure(kind) => Some(kind),
            _ => None,
        }
    }

    /// **Whether the text tool is armed.**
    ///
    /// [`Self::markup_kind`]'s and [`Self::measure_kind`]'s third sibling,
    /// answering `bool` rather than `Option<Kind>` because [`Self::Text`] carries
    /// no kind — see that variant's docs for why it carries nothing at all.
    ///
    /// It exists for the same reason the other two do, and it has the same three
    /// callers, which is what stops them writing three `matches!` that could
    /// drift: [`crate::canvas::textsel::takes_the_press`], which decides what a
    /// press means; `crate::app::PdfceApp::conditions`, which decides whether the
    /// ribbon control renders **pressed**; and [`crate::canvas::gesture::press_kind`],
    /// which reads it through `takes_the_press` rather than directly, so the
    /// drag's meaning and the click's routing cannot disagree.
    ///
    /// Deliberately `selected`-agnostic: like its siblings it is a question about
    /// a [`CanvasTool`] value, and *which* value — the chosen one or the one a
    /// held space bar composes — is the caller's decision. [`active`] and
    /// [`selected`] answer differently on purpose.
    #[must_use]
    pub fn is_text(self) -> bool {
        matches!(self, Self::Text)
    }
}

/// **What the pointer looks like this frame** — the whole precedence, in one
/// pure function.
///
/// Lifted out of `canvas::interact` when the markup tool arrived, along the
/// same seam [`crate::canvas::gesture::press_kind`] was: the first rung of this
/// decision was already [`CanvasTool::cursor`], so the remaining three rungs
/// were the rest of one question living in the wiring, where they could not be
/// tested and where a fourth tool would have had to be remembered.
///
/// # The order is the rule
///
/// 1. **The armed tool**, when the pointer is over the canvas or a button is
///    down. This rung is the whole of *"the cursor must change, and must change
///    back"*: it changes because this branch is taken while the tool is active,
///    and it changes back because the answer is recomputed every frame from
///    [`active`] with nothing stored to restore. A dropped key-up costs one
///    frame of hand, not a canvas stuck showing a grab cursor over a select
///    tool.
/// 2. **A gesture in flight**, which keeps its own cursor even once the pointer
///    has wandered off the thing it started on — otherwise a drag that outruns
///    its object looks like it stopped working.
/// 3. **A hovered grip**, which is how the eight resize handles are findable at
///    all.
/// 4. **Nothing**, leaving the cursor to whatever else set it.
///
/// `pointer_down` is *any* button, because a middle-drag pan must show the
/// closed hand too; `over_canvas` is measured against the scroll viewport
/// rather than the page, because the hand pans the grey surround as readily as
/// the paper and a hand tool that shows no hand over half the canvas reads as a
/// tool that is not armed.
#[must_use]
pub fn cursor_for(
    tool: CanvasTool,
    gesture: Option<crate::canvas::gesture::DragKind>,
    hovered_grip: Option<crate::canvas::handles::Grip>,
    pointer_down: bool,
    over_canvas: bool,
) -> Option<CursorIcon> {
    use crate::canvas::gesture::DragKind;

    if let Some(icon) = tool
        .cursor(pointer_down)
        .filter(|_| over_canvas || pointer_down)
    {
        return Some(icon);
    }
    if let Some(kind) = gesture {
        return Some(match kind {
            // One crosshair for both marquee intents: the band is the same band
            // and `gesture`'s header refuses a second set of pixels for it. What
            // tells the operator a zoom is armed is the ribbon control that
            // armed it, off-canvas, where a mode indicator belongs. A markup
            // band answers the same way, and is stated rather than wildcarded
            // even though rung 1 already claimed it — a drag cannot be in flight
            // without the tool that started it, so this is unreachable today and
            // spelling it keeps the two answers one answer if that changes.
            DragKind::Marquee(_) | DragKind::Markup(_) => CursorIcon::Crosshair,
            DragKind::Move => CursorIcon::Grabbing,
            DragKind::Resize(grip) => grip.cursor(),
            // ★ The I-beam for a sweep that began under the MODE rule rather
            // than under an armed tool — and that distinction is now the whole
            // of what this arm is for.
            //
            // ★ **The paragraph that used to stand here has been half
            // discharged, and the discharged half is quoted rather than
            // deleted** because it predicted its own expiry: *"The hover I-beam
            // becomes free on the day a `CanvasTool::Text` lands, because it is
            // then rung 1's answer like every other tool's."* That day is
            // 2026-08-14. With the tool armed, `CanvasTool::Text`'s `cursor`
            // answers `Text` at rung 1, so the pointer is an I-beam from the
            // moment the tool is chosen — on hover, before any drag, over the
            // grey surround as readily as the paper — and it costs one match arm
            // per frame rather than a hit test.
            //
            // The undischarged half stands unchanged and is why this arm
            // survives: in **Read and Review** a press means text with *no tool
            // armed at all* (the select tool, under
            // `textsel::takes_the_press`'s original disjunct), so rung 1 has
            // nothing to answer with there and this rung is the only one that
            // can. Making it hover in those modes would still mean asking "is
            // there a glyph under the pointer?" on every frame the pointer moves
            // — a hit test against the page's extraction, paid on canvases
            // nobody is selecting on, which is most of them. And threading
            // `Capabilities` into this function to synthesise a tool from the
            // mode would put the mode gate in a second place, which is the thing
            // `canvas::textsel`'s header §3 spends its length arguing against.
            //
            // So the shipped rule is: **armed ⇒ I-beam always; un-armed ⇒ I-beam
            // once the sweep starts.** A reader may reasonably ask whether that
            // is an inconsistency an operator would notice, and the answer is
            // that they cannot: the two cases never coexist on one canvas,
            // because arming the tool is what moves a mode from the second to
            // the first.
            DragKind::TextSelect => CursorIcon::Text,
        });
    }
    hovered_grip.map(crate::canvas::handles::Grip::cursor)
}

/// Compose the chosen tool with the space bar — **the rule, and the only
/// place it exists**.
///
/// Space *borrows* the hand; it does not choose it. So this is a `max`, not a
/// swap: holding space over the hand tool changes nothing, and releasing it
/// returns whatever [`selected`] has said all along.
#[must_use]
pub fn resolve(selected: CanvasTool, space_held: bool) -> CanvasTool {
    if space_held {
        CanvasTool::Hand
    } else {
        selected
    }
}

/// The tool the operator chose — the persistent half, unaffected by the space
/// bar.
///
/// This is what a ribbon toggle or a tool palette should render as pressed:
/// showing the *active* tool there would make the button flicker under the
/// operator's thumb every time they held space.
#[must_use]
pub fn selected(ctx: &egui::Context) -> CanvasTool {
    let id = egui::Id::new(TOOL_MEMORY_KEY);
    ctx.data(|d| d.get_temp::<CanvasTool>(id).unwrap_or_default())
}

/// Choose a tool. **The entry point a `view.tool_hand` / `view.tool_select`
/// command calls.**
pub fn select(ctx: &egui::Context, tool: CanvasTool) {
    let id = egui::Id::new(TOOL_MEMORY_KEY);
    ctx.data_mut(|d| d.insert_temp(id, tool));
}

/// Flip between the hand and the select tool. **The entry point a single
/// `view.tool_hand` *toggle* command calls.**
///
/// Returns the tool now chosen, so a caller that wants to report or check the
/// new state does not have to ask again and risk reading a different frame's
/// answer.
pub fn toggle_hand(ctx: &egui::Context) -> CanvasTool {
    let next = match selected(ctx) {
        CanvasTool::Hand => CanvasTool::Select,
        // Any other tool is *left* by pressing Hand, not toggled through — the
        // operator asked for the hand, and returning them to Select would make
        // one press mean "put the pen down" and a second one mean "pick the
        // hand up". The text tool joins that arm rather than earning its own for
        // the identical reason: pressing Hand while sweeping text means Hand.
        CanvasTool::Select | CanvasTool::Markup(_) | CanvasTool::Measure(_) | CanvasTool::Text => {
            CanvasTool::Hand
        }
    };
    select(ctx, next);
    next
}

/// Flip between the text tool and the select tool. **The entry point the
/// `view.tool_text` toggle command calls.**
///
/// [`toggle_hand`]'s twin, deliberately down to the shape of the `match`: these
/// are the two pointer tools that carry no kind, they sit in the same ribbon
/// group, and a single press of either is how an operator both enters and leaves
/// it. The same-press-retires rule is [`arm_markup`]'s argument applied to a tool
/// with one kind instead of four — *the button is pressed, so pressing it is how
/// you un-press it* — and without it an operator who armed Text by mistake would
/// have no way back to the select tool except by arming something else.
///
/// # Why it returns to `Select` and not to whatever was armed before
///
/// Because nothing is stored to return to, and that is the same refusal this
/// module's header makes about the space bar: a "previous tool" is state that can
/// be lost, and losing it leaves the canvas in a tool the operator never chose.
/// [`CanvasTool::Select`] is this enum's `#[default]` and the stance every other
/// retirement path in this file returns to ([`disarm_markup`],
/// [`disarm_measure`], [`retire_forbidden`]), so a reader has one answer to learn
/// rather than four.
///
/// Note what that means in a **reading** mode, and it is deliberate rather than a
/// gap: in Read and Review the select tool already sweeps text
/// ([`crate::canvas::textsel::takes_the_press`]'s original rule), so toggling
/// this off there changes the pressed control and changes no behaviour. The tool
/// is not suppressed in those modes for that reason — a control that vanished
/// from View in two of three modes would be a per-mode visibility rule invented
/// to hide a redundancy, and View is shown in every mode precisely so its
/// contents do not have to be.
///
/// Returns the tool now chosen, honouring the same report-rather-than-re-ask
/// contract [`toggle_hand`] and [`arm_markup`] do.
pub fn toggle_text(ctx: &egui::Context) -> CanvasTool {
    let next = match selected(ctx) {
        CanvasTool::Text => CanvasTool::Select,
        // …and from any other tool this *takes* the text tool rather than
        // returning to Select, which is `toggle_hand`'s rule above and
        // `arm_markup`'s different-kind-re-arms rule, spelled once more.
        CanvasTool::Select | CanvasTool::Hand | CanvasTool::Markup(_) | CanvasTool::Measure(_) => {
            CanvasTool::Text
        }
    };
    select(ctx, next);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI.
        //
        // The same argument `markup-tool` and `measure-tool` carry, and it is
        // sharper here than for either: an armed text tool changes the CURSOR and
        // nothing else on screen, so an armed canvas and an un-armed one are not
        // merely the same screenshot — they are the same screenshot even with the
        // pointer in it, because a captured window does not carry the cursor.
        // This line is the only way a harness can prove the ribbon button armed
        // anything.
        format!("text-tool tool={next:?}")
    });
    next
}

/// Arm the markup tool with `kind`, or retire it if that kind is already
/// armed. **The entry point every `markup.*` shape command calls.**
///
/// # ★ Why pressing the armed button again retires the tool
///
/// *"Make it work the way other programs do"* is the operator's stated
/// tie-breaker, and every drawing application treats a tool button as a toggle:
/// the button is **pressed**, so pressing it is how you un-press it. The
/// alternative — a button that only ever arms — leaves an operator who armed
/// Rectangle by mistake with no way back to the select tool except Escape,
/// which they have to know about, or arming some other tool, which is not what
/// they want either.
///
/// Choosing a *different* kind is not a toggle; it is a change of kind, and it
/// arms. So the rule is: same kind ⇒ retire, different kind ⇒ re-arm. That is
/// what makes the four Markup buttons behave as a radio you can switch off,
/// which is what they look like once each renders pressed.
///
/// Returns the tool now chosen, so a caller that wants to report or check the
/// new state does not have to ask again and risk reading a different frame's
/// answer — the same contract [`toggle_hand`] honours.
pub fn arm_markup(ctx: &egui::Context, kind: MarkupKind) -> CanvasTool {
    let next = if selected(ctx) == CanvasTool::Markup(kind) {
        CanvasTool::Select
    } else {
        CanvasTool::Markup(kind)
    };
    select(ctx, next);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI.
        //
        // The tool a canvas is armed with is otherwise invisible from outside:
        // a crosshair is a cursor, and a screenshot of an armed canvas and an
        // un-armed one are the same picture — which is defect 8's lesson
        // exactly. This line is how a harness proves the button armed anything.
        format!("markup-tool tool={next:?}")
    });
    next
}

/// Arm the measure tool with `kind`, or retire it if that kind is already
/// armed. **The entry point every `measure.*` tool command calls.**
///
/// [`arm_markup`]'s twin, with the identical same-kind-retires rule and for the
/// identical reason — see that function's header, which is the argument for
/// both. The two are separate functions rather than one generic over the kind
/// because the tools are separate: a shared one would have to take a
/// `CanvasTool` already built, which moves the "which variant" decision back out
/// to the four call sites this pair exists to keep it away from.
///
/// **It arms a tool; it authors nothing.** The clicks are taken by
/// [`crate::canvas::measure`], and only the pick that completes a dimension
/// raises an `Action`.
pub fn arm_measure(ctx: &egui::Context, kind: MeasureKind) -> CanvasTool {
    let next = if selected(ctx) == CanvasTool::Measure(kind) {
        CanvasTool::Select
    } else {
        CanvasTool::Measure(kind)
    };
    select(ctx, next);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI.
        //
        // Same argument as `markup-tool` above: an armed canvas and an un-armed
        // one are the same screenshot, so this line is the only way a harness
        // can prove the ribbon button armed anything.
        format!("measure-tool tool={next:?}")
    });
    next
}

/// Retire the measure tool, returning to [`CanvasTool::Select`], and report
/// whether there was one to retire.
///
/// **Escape's claimant, alongside [`disarm_markup`]**, and it sits at the same
/// rung for the same reason — see [`crate::canvas::keys`]'s precedence table.
///
/// ★ Note what this does **not** do: it does not discard a half-finished pick.
/// A linear dimension with point A taken and point B not is in-progress work
/// held by [`crate::canvas::measure::pick`], and Escape retires *one* thing per
/// press (decision 025's L1). So the first Escape abandons the pick and the
/// second puts the tool down — which is the order the operator means, because
/// the pick is the more transient of the two.
pub fn disarm_measure(ctx: &egui::Context) -> bool {
    if selected(ctx).measure_kind().is_none() {
        return false;
    }
    select(ctx, CanvasTool::Select);
    true
}

/// Retire the markup tool, returning to [`CanvasTool::Select`], and report
/// whether there was one to retire.
///
/// **Escape's claimant.** Reports rather than being asked twice, for the same
/// reason `zoom::disarm_region_zoom` does: the caller cannot know whether the
/// key was spent here without asking, and a caller that re-derived it would be
/// the version that retires the tool *and* ascends a selection rung. See
/// [`crate::canvas::keys`]'s precedence table for where this sits and why.
///
/// Deliberately reads [`selected`] rather than [`active`]: a held space bar
/// borrows the hand, and Escape pressed mid-space must retire the markup tool
/// underneath it rather than doing nothing because the *active* tool happened
/// to be the hand at that instant.
pub fn disarm_markup(ctx: &egui::Context) -> bool {
    if selected(ctx).markup_kind().is_none() {
        return false;
    }
    select(ctx, CanvasTool::Select);
    true
}

/// **Retire an armed tool the mode being entered does not permit**, and report
/// whether there was one.
///
/// Called from `PdfceApp`'s mode-change arm, once, on the frame the operator
/// moves the selector.
///
/// # ★ Why arming has to be undone rather than merely refused
///
/// The armed tool lives in `egui::Memory` and is **application**-scoped, not
/// per-mode — see this module's header. So a Rectangle armed in Edit is still
/// armed after a switch to Read, and it survives a switch back. That is the
/// right lifetime for a tool (an operator who returns to Edit expects their
/// pen), and it is exactly wrong across a mode that forbids the pen.
///
/// [`crate::canvas::gesture::press_kind`] already refuses to give a forbidden
/// tool a meaning, so nothing would be drawn either way. What that refusal
/// cannot fix is the **cursor**: [`CanvasTool::cursor`] gives an armed markup
/// tool a crosshair, so without this the operator would be shown a drawing
/// cursor over every page of a document they cannot draw on — a promise the
/// canvas has already decided not to keep. Retiring the tool is what makes the
/// pointer tell the truth.
///
/// Returns to [`CanvasTool::Select`] rather than to `Hand`, matching
/// [`disarm_markup`]: Select is this enum's `#[default]` and the stance every
/// other retirement path returns to, and a mode change that silently swapped in
/// a *different* tool would be a second surprise on top of the first.
pub fn retire_forbidden(ctx: &egui::Context, caps: Capabilities) -> bool {
    let armed = selected(ctx);
    let permitted = match armed {
        // None of the three touches the document: Select is inert in a mode that
        // cannot select (`press_kind` gives its presses no meaning), Hand only
        // pans, and Text reads the page and writes to the clipboard. Retiring any
        // of them would take a navigation or reading tool away from the mode that
        // navigates and reads.
        //
        // ★ **Text is on this arm and NOT on the markup arm below, and the
        // difference is the operator's own ruling rather than a judgement made
        // here.** The obvious move when adding a tool is to copy the line above
        // the cursor and swap the capability — `CanvasTool::Text => caps.???` —
        // and there is no capability to put there. Three steps:
        //
        // 1. **Selecting text authors nothing.** It changes no byte, bumps no
        //    `edit_epoch`, and touches no `EditSession`. `app::modes::capability`
        //    §4's not-gated list is exactly that class — *"pan, zoom, the hand
        //    tool, marquee zoom, Find, guides, rulers, grid: navigation and
        //    inspection, none of which touches the document"* — and its nearest
        //    neighbour there is Find, which also extracts the page's text, also
        //    derives quads from it, and also washes the result.
        // 2. **The operator settled it for the commands already.** On 2026-08-14
        //    both text-copy verbs moved off the authoring tab under the sentence
        //    *copying is not authoring*. A capability invented here would be that
        //    ruling restated in a second place, free to disagree with it — which
        //    is the same argument `canvas::textsel` §3 makes for why there is no
        //    `select_text` flag.
        // 3. **The retirement would be actively wrong in both directions.**
        //    Retiring it on the way into Read would take away a tool that mode
        //    plainly permits (its select tool already sweeps text). Retiring it on
        //    the way into **Edit** would be worse: Edit is the one mode this tool
        //    exists for, so a capability check that failed there would delete the
        //    feature on the frame the operator entered the mode that needs it.
        //
        // So the honest answer is *none*, and it is written as membership of this
        // arm — where the reason is stated — rather than as a `true` on a line of
        // its own, so that a future reader adding a fifth tool has to decide which
        // of the two groups it joins.
        CanvasTool::Select | CanvasTool::Hand | CanvasTool::Text => true,
        CanvasTool::Markup(_) => caps.author_markup,
        CanvasTool::Measure(_) => caps.author_measure,
    };
    if permitted {
        return false;
    }
    select(ctx, CanvasTool::Select);
    true
}

/// Whether the space bar is down **and the canvas is entitled to it**.
///
/// See the module docs on the text-field guard.
#[must_use]
pub fn space_held(ctx: &egui::Context) -> bool {
    !ctx.text_edit_focused() && ctx.input(|i| i.key_down(Key::Space))
}

/// What the primary button means on this frame — [`resolve`] applied to the
/// live context.
///
/// The one call the canvas makes. Everything downstream branches on the
/// result and nothing downstream reads the space bar for itself.
#[must_use]
pub fn active(ctx: &egui::Context) -> CanvasTool {
    resolve(selected(ctx), space_held(ctx))
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Context, Event, Modifiers, RawInput};

    /// ★ **Space borrows the hand and gives it back** — the requirement,
    /// stated as the pure rule it is implemented as.
    ///
    /// The third case is the one that matters: releasing space returns to
    /// `Select`, and it does so without anything having been stored, so there
    /// is no restore step that can be skipped.
    #[test]
    fn space_borrows_the_hand_and_releasing_returns_the_previous_tool() {
        assert_eq!(resolve(CanvasTool::Select, false), CanvasTool::Select);
        assert_eq!(resolve(CanvasTool::Select, true), CanvasTool::Hand);
        assert_eq!(resolve(CanvasTool::Select, false), CanvasTool::Select);
    }

    /// Holding space while the hand tool is already chosen changes nothing,
    /// and releasing it does not drop the operator back into Select.
    #[test]
    fn space_over_the_hand_tool_is_a_no_op_in_both_directions() {
        assert_eq!(resolve(CanvasTool::Hand, true), CanvasTool::Hand);
        assert_eq!(resolve(CanvasTool::Hand, false), CanvasTool::Hand);
    }

    /// Only the hand pans, and each tool's cursor is what it should be — the
    /// two halves of the branch, asserted together so a future fourth tool
    /// cannot answer one and forget the other.
    ///
    /// The markup rows are the ones that matter now: a markup tool that
    /// answered `true` to `pans_with_primary` would be handed a blank pointer
    /// frame by `canvas::interact` and could never draw anything at all — a
    /// tool that arms, shows a crosshair and does nothing, which is the exact
    /// shape of an affordance that looks available and is inert.
    #[test]
    fn only_the_hand_pans_and_each_tool_paints_its_own_cursor() {
        assert!(!CanvasTool::Select.pans_with_primary());
        assert!(CanvasTool::Hand.pans_with_primary());
        assert_eq!(CanvasTool::Select.cursor(false), None);
        assert_eq!(CanvasTool::Select.cursor(true), None);
        assert_eq!(CanvasTool::Hand.cursor(false), Some(CursorIcon::Grab));
        assert_eq!(CanvasTool::Hand.cursor(true), Some(CursorIcon::Grabbing));
        for &kind in MarkupKind::ALL {
            let tool = CanvasTool::Markup(kind);
            assert!(!tool.pans_with_primary(), "{kind:?} must not pan");
            assert_eq!(tool.cursor(false), Some(CursorIcon::Crosshair), "{kind:?}");
            assert_eq!(tool.cursor(true), Some(CursorIcon::Crosshair), "{kind:?}");
            assert_eq!(tool.markup_kind(), Some(kind));
        }
        assert_eq!(CanvasTool::Select.markup_kind(), None);
        assert_eq!(CanvasTool::Hand.markup_kind(), None);
    }

    /// ★ **The text tool does not pan, shows an I-beam in both states, and is
    /// the only tool `is_text` answers `true` for.**
    ///
    /// All three halves, because each has a distinct and plausible failure. A
    /// text tool that answered `true` to `pans_with_primary` would be handed a
    /// **blank** pointer frame by `canvas::interact` and could never sweep
    /// anything at all — a tool that arms, shows an I-beam and does nothing,
    /// which is the exact shape of an affordance that looks available and is
    /// inert. A tool with no cursor would be indistinguishable from the select
    /// tool it replaces, on a control whose entire visible effect is the pointer.
    /// And an `is_text` that answered `true` for a *markup* tool would hand an
    /// armed pen's press to the text gesture.
    #[test]
    fn the_text_tool_sweeps_rather_than_pans_and_says_so_with_the_pointer() {
        let text = CanvasTool::Text;
        assert!(!text.pans_with_primary());
        assert_eq!(text.cursor(false), Some(CursorIcon::Text));
        assert_eq!(text.cursor(true), Some(CursorIcon::Text));
        assert!(text.is_text());
        assert_eq!(text.markup_kind(), None);
        assert_eq!(text.measure_kind(), None);

        for other in [
            CanvasTool::Select,
            CanvasTool::Hand,
            CanvasTool::Markup(MarkupKind::Rectangle),
            CanvasTool::Measure(MeasureKind::Linear),
        ] {
            assert!(!other.is_text(), "{other:?} is not the text tool");
        }

        // ★ Rung 1 of `cursor_for`, on hover with no button down and no gesture
        // — which is the whole difference the tool makes to the pointer, and the
        // half the un-armed rule could not pay for. It also outranks a hovered
        // grip, which is load-bearing in Edit: a content selection can be on the
        // page while this tool is armed, and `press_kind` gives the armed tool
        // the press, so a grip that kept its resize cursor would promise a
        // gesture already decided against.
        assert_eq!(
            cursor_for(CanvasTool::Text, None, None, false, true),
            Some(CursorIcon::Text),
            "an armed text tool paints the I-beam ON HOVER, before any drag"
        );
        assert_eq!(
            cursor_for(
                CanvasTool::Text,
                None,
                Some(crate::canvas::handles::Grip::SouthEast),
                true,
                true,
            ),
            Some(CursorIcon::Text),
            "…and it suppresses the grip cursors, as every armed tool does"
        );
        assert_eq!(
            cursor_for(CanvasTool::Text, None, None, false, false),
            None,
            "but it does not claim the pointer over the ribbon"
        );
    }

    /// ★ **Pressing the armed Text button again retires it; pressing it from
    /// another tool takes it.**
    ///
    /// `arm_markup`'s two halves for a tool with no kind, and both matter for the
    /// reason that test gives: a build that only ever armed would pass a test of
    /// the first press alone, and the operator's complaint would be that the tool
    /// cannot be put down.
    ///
    /// The third case is the one `toggle_text`'s own docs argue: arriving from a
    /// markup tool must **take** the text tool rather than dropping to Select,
    /// or one press would mean "put the pen down" and a second one "pick up the
    /// I-beam".
    #[test]
    fn toggling_the_text_tool_arms_it_retires_it_and_takes_it_from_another_tool() {
        let ctx = Context::default();
        assert_eq!(selected(&ctx), CanvasTool::Select);

        assert_eq!(toggle_text(&ctx), CanvasTool::Text);
        assert_eq!(selected(&ctx), CanvasTool::Text);
        assert_eq!(
            toggle_text(&ctx),
            CanvasTool::Select,
            "a second press retires"
        );

        arm_markup(&ctx, MarkupKind::Ellipse);
        assert_eq!(
            toggle_text(&ctx),
            CanvasTool::Text,
            "from a pen, Text takes the tool rather than returning to Select"
        );
        // …and Hand still takes it back from Text, which is `toggle_hand`'s
        // matching arm and would silently answer `Select` if the variant had been
        // added to the wrong side of that match.
        assert_eq!(toggle_hand(&ctx), CanvasTool::Hand);
        assert_eq!(toggle_hand(&ctx), CanvasTool::Select);
    }

    /// ★ **Space borrows the hand out of the text tool and gives it back**, and
    /// **Escape does not claim the text tool.**
    ///
    /// The first is the property the derived-never-stored design exists for,
    /// asserted for the new tool exactly as it is for markup above.
    ///
    /// The second is a **deliberate absence**, asserted so that adding an Escape
    /// rung later is a decision rather than an accident. `canvas::keys`' rung 3b
    /// retires an armed markup or measure tool, and the text tool is
    /// deliberately not on it: those two paint a crosshair promising a gesture
    /// that *writes to the document*, while this one promises a selection, and —
    /// the deciding half — Escape's rung 5 already means "clear the selection"
    /// in this tool. A further press that silently moved the operator from
    /// sweeping text to marqueeing objects would be a change of gesture they did
    /// not ask for, on the key they pressed to clear something. See
    /// `canvas::keys`' header.
    #[test]
    fn space_borrows_the_hand_out_of_the_text_tool_and_escape_leaves_it_alone() {
        assert_eq!(resolve(CanvasTool::Text, true), CanvasTool::Hand);
        assert_eq!(resolve(CanvasTool::Text, false), CanvasTool::Text);

        let ctx = Context::default();
        select(&ctx, CanvasTool::Text);
        assert!(
            !disarm_markup(&ctx),
            "the markup claimant must not take Escape for a tool that is not a pen"
        );
        assert!(!disarm_measure(&ctx));
        assert_eq!(
            selected(&ctx),
            CanvasTool::Text,
            "and the tool is still armed afterwards"
        );
    }

    /// ★ **No mode retires the text tool** — the `retire_forbidden` decision,
    /// asserted over every capability combination rather than over the three
    /// shipped modes.
    ///
    /// The Edit row (`FULL`) is the one that would break the feature outright: a
    /// capability check copied from the markup arm would fail on the frame the
    /// operator entered the one mode this tool exists for. The Read row
    /// (`NONE`) is the one that would break it quietly, by taking a reading tool
    /// away from the reading mode.
    ///
    /// Asserted beside the *markup* tool in the same loop, so this is a statement
    /// about the difference rather than about the text tool alone: a build that
    /// stopped retiring anything would pass the first half and fail the second.
    #[test]
    fn the_text_tool_is_permitted_in_every_mode_where_a_pen_is_not() {
        for markup in [false, true] {
            for measure in [false, true] {
                for content in [false, true] {
                    let caps = Capabilities {
                        edit_content: content,
                        author_markup: markup,
                        author_measure: measure,
                    };
                    let ctx = Context::default();
                    select(&ctx, CanvasTool::Text);
                    assert!(
                        !retire_forbidden(&ctx, caps),
                        "the text tool authors nothing, so {caps:?} has nothing to forbid"
                    );
                    assert_eq!(selected(&ctx), CanvasTool::Text);

                    select(&ctx, CanvasTool::Markup(MarkupKind::Rectangle));
                    assert_eq!(
                        retire_forbidden(&ctx, caps),
                        !markup,
                        "a pen IS retired by a mode that cannot author markup: {caps:?}"
                    );
                }
            }
        }
    }

    /// ★ **The cursor precedence**, all four rungs, in one test that would
    /// have caught each of them being reordered.
    ///
    /// This rule was four `if`s in the middle of `canvas::interact` and had no
    /// test at all — it needed a window to reach. Moving it here is what makes
    /// it assertable, and the rungs are asserted **against each other**: each
    /// case supplies a lower rung that would answer differently, so a build
    /// that consulted them in the wrong order fails rather than merely
    /// producing *a* cursor.
    #[test]
    fn the_cursor_precedence_runs_tool_then_gesture_then_grip() {
        use crate::canvas::gesture::{DragKind, MarqueeIntent};
        use crate::canvas::handles::Grip;

        // 1. The armed tool wins over a gesture AND a hovered grip.
        assert_eq!(
            cursor_for(
                CanvasTool::Markup(MarkupKind::Arrow),
                Some(DragKind::Move),
                Some(Grip::SouthEast),
                true,
                true,
            ),
            Some(CursorIcon::Crosshair),
        );
        assert_eq!(
            cursor_for(CanvasTool::Hand, Some(DragKind::Move), None, true, true),
            Some(CursorIcon::Grabbing),
        );
        // …but only while the pointer is over the canvas or a button is down,
        // so the hand does not claim the cursor over the ribbon.
        assert_eq!(cursor_for(CanvasTool::Hand, None, None, false, false), None);

        // 2. With the select tool, a gesture in flight wins over a grip the
        //    pointer happens to be over.
        assert_eq!(
            cursor_for(
                CanvasTool::Select,
                Some(DragKind::Marquee(MarqueeIntent::Select)),
                Some(Grip::SouthEast),
                true,
                true,
            ),
            Some(CursorIcon::Crosshair),
        );
        assert_eq!(
            cursor_for(
                CanvasTool::Select,
                Some(DragKind::Resize(Grip::NorthWest)),
                Some(Grip::SouthEast),
                true,
                true,
            ),
            Some(Grip::NorthWest.cursor()),
            "an in-flight resize keeps ITS grip's cursor, not the hovered one"
        );

        // 3. Then a hovered grip, and 4. then nothing.
        assert_eq!(
            cursor_for(CanvasTool::Select, None, Some(Grip::East), false, true),
            Some(Grip::East.cursor()),
        );
        assert_eq!(
            cursor_for(CanvasTool::Select, None, None, false, true),
            None
        );
    }

    /// ★ **Pressing an armed markup button again retires the tool; pressing a
    /// different one changes kind.**
    ///
    /// Both halves, because a build that only armed would pass a test of the
    /// first press alone — and the operator's complaint would be that the tool
    /// cannot be put down.
    #[test]
    fn arming_a_markup_kind_toggles_that_kind_and_switches_between_kinds() {
        let ctx = Context::default();
        assert_eq!(selected(&ctx), CanvasTool::Select);

        assert_eq!(
            arm_markup(&ctx, MarkupKind::Rectangle),
            CanvasTool::Markup(MarkupKind::Rectangle)
        );
        // A different kind re-arms rather than retiring.
        assert_eq!(
            arm_markup(&ctx, MarkupKind::Arrow),
            CanvasTool::Markup(MarkupKind::Arrow)
        );
        assert_eq!(selected(&ctx), CanvasTool::Markup(MarkupKind::Arrow));
        // The same kind again retires.
        assert_eq!(arm_markup(&ctx, MarkupKind::Arrow), CanvasTool::Select);
        assert_eq!(selected(&ctx), CanvasTool::Select);
    }

    /// ★ **Escape's claimant reports whether it took the key.**
    ///
    /// `false` with nothing armed is the load-bearing half: without it Escape
    /// would be consumed by a tool that was not armed, and the selection ladder
    /// would need two presses to leave a rung.
    #[test]
    fn disarming_markup_reports_whether_there_was_anything_to_disarm() {
        let ctx = Context::default();
        assert!(!disarm_markup(&ctx), "nothing armed: the key is not ours");

        arm_markup(&ctx, MarkupKind::Ellipse);
        assert!(disarm_markup(&ctx));
        assert_eq!(selected(&ctx), CanvasTool::Select);
        assert!(!disarm_markup(&ctx), "and it is not claimed twice");

        // The hand tool is not ours to retire either — Escape must not silently
        // put an operator who chose the hand back into Select.
        select(&ctx, CanvasTool::Hand);
        assert!(!disarm_markup(&ctx));
        assert_eq!(selected(&ctx), CanvasTool::Hand);
    }

    /// ★ **Space borrows the hand out of the markup tool and gives it back.**
    ///
    /// The property the whole "derived, never stored" design exists for,
    /// asserted for the new tool: an operator drawing a rectangle who holds
    /// space to reposition the page must get the rectangle tool back on
    /// release, with its kind intact.
    #[test]
    fn space_borrows_the_hand_out_of_the_markup_tool_and_returns_the_kind() {
        let armed = CanvasTool::Markup(MarkupKind::Rectangle);
        assert_eq!(resolve(armed, true), CanvasTool::Hand);
        assert_eq!(resolve(armed, false), armed);
    }

    /// The chosen tool survives a frame, and the toggle alternates rather
    /// than latching.
    #[test]
    fn the_chosen_tool_persists_and_the_toggle_alternates() {
        let ctx = Context::default();
        assert_eq!(selected(&ctx), CanvasTool::Select);
        assert_eq!(toggle_hand(&ctx), CanvasTool::Hand);
        assert_eq!(selected(&ctx), CanvasTool::Hand);
        assert_eq!(toggle_hand(&ctx), CanvasTool::Select);
        select(&ctx, CanvasTool::Hand);
        assert_eq!(selected(&ctx), CanvasTool::Hand);
    }

    /// ★ **A focused text field keeps the space bar**, so typing a page
    /// number into the status bar does not pan the drawing under the
    /// operator.
    ///
    /// Built against a real `TextEdit` for the same reason
    /// `canvas::tests::a_focused_text_field_keeps_delete_for_itself` is:
    /// `text_edit_focused()` resolves the focused id and looks for a
    /// `TextEditState` under it, so a hand-requested focus on a bare id would
    /// pass vacuously.
    #[test]
    fn a_focused_text_field_keeps_the_space_bar() {
        let ctx = Context::default();
        let mut buffer = String::from("37");

        // Frame 1: build the field and take focus.
        let _ = ctx.run_ui(RawInput::default(), |ui| {
            ui.add(egui::TextEdit::singleline(&mut buffer))
                .request_focus();
        });

        // Frame 2: the field holds focus and space is down.
        let input = RawInput {
            events: vec![Event::Key {
                key: Key::Space,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
            ..Default::default()
        };
        let mut typing = false;
        let mut held = true;
        let _ = ctx.run_ui(input, |ui| {
            ui.add(egui::TextEdit::singleline(&mut buffer));
            typing = ui.ctx().text_edit_focused();
            held = space_held(ui.ctx());
        });

        assert!(
            typing,
            "the test is vacuous unless a TEXT field really holds focus"
        );
        assert!(!held, "a focused text field must keep the space bar");
        assert_eq!(
            resolve(selected(&ctx), held),
            CanvasTool::Select,
            "and the tool must therefore not have changed"
        );
    }

    /// With no text field in the way, a held space bar really does reach the
    /// canvas — the other direction of the guard above, without which the
    /// previous test would pass on a build where space-pan never worked at
    /// all.
    #[test]
    fn a_held_space_bar_reaches_the_canvas_when_nothing_is_typing() {
        let ctx = Context::default();
        let input = RawInput {
            events: vec![Event::Key {
                key: Key::Space,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
            ..Default::default()
        };
        let mut tool = CanvasTool::Select;
        let _ = ctx.run_ui(input, |ui| {
            tool = active(ui.ctx());
        });
        assert_eq!(tool, CanvasTool::Hand);
    }
}
