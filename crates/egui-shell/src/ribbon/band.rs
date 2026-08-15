//! The band — the row of captioned groups beneath the active tab.
//!
//! # ★ The one closure every group goes through
//!
//! This module's central design decision is that `captioned_group` is
//! the **only** function in this crate that draws a ribbon group, and
//! that it emits the caption itself, after the body, with no branch that
//! can skip it.
//!
//! That is not defensive style. It is a fix for a defect that actually
//! shipped, recorded in the salvage source's own doc comment
//! (`D:\Dev\pdfce\crates\pdfce-gui\src\ribbon_ui.rs`):
//!
//! > Two sites previously bypassed the predicate and therefore drew no
//! > caption at all: `LayoutReset` used a bare `tab.shows(..)`, and
//! > `Show` and `Panels` shared one `shows(A) || shows(B)` block. Both
//! > were visible in the 2026-08-08 capture as unlabelled floating
//! > controls.
//!
//! Two caption-less groups shipped. They were found by a **screenshot**,
//! not by a test, and the reason is instructive: nothing was wrong. Each
//! site compiled, each drew its controls, each passed every test the
//! project had. The rule "a group has a caption" lived in a convention
//! that two call sites happened not to follow.
//!
//! The predecessor's fix was to make the caption a *consequence of
//! drawing the group* rather than a separate statement — the body is
//! handed in as a closure, so there is no code path that shows a group
//! without captioning it. That shape is carried across here, and
//! strengthened in three ways:
//!
//! 1. **The caption is never empty.** The manifest's caption is
//!    `Option<String>` because a *layer* may omit it (see
//!    [`crate::manifest`]); `caption_text` falls back to the group's
//!    **id**, which is never empty in a well-formed manifest. So even an
//!    unvalidated manifest cannot produce a bare band — it produces an
//!    ugly caption that names the group that needs fixing.
//! 2. **Overflowed groups go through the same closure.** A group that
//!    moved into the "⏷ N more" menu is still a group and still gets its
//!    caption. Routing the menu through a second, simpler drawing path is
//!    exactly how the two shipped defects happened.
//! 3. **The counts are returned and asserted.** `BandOutcome` carries
//!    `groups_rendered` and `captions_emitted`; they are `debug_assert`ed
//!    equal at the end of every band, and
//!    `every_rendered_group_emits_a_caption` asserts it in release too,
//!    against a manifest that deliberately includes a caption-less group.
//!
//! # ★ Two rows, and a height that does not depend on the tab
//!
//! A band is **[`plan::GROUP_ROWS`] control rows tall on every tab**, and a
//! group whose controls are wider than [`plan::GROUP_WRAP_WIDTH`] wraps onto
//! the second row rather than running on. Both halves are
//! `mockups/ribbon.html`'s:
//!
//! ```css
//! .band  { display:flex; align-items:stretch; padding:8px 10px 4px; min-height:86px }
//! .group { display:flex; flex-direction:column; justify-content:space-between;
//!          padding:0 13px }
//! .gcmds { display:flex; flex-wrap:wrap; gap:5px; align-items:flex-start; max-width:440px }
//! ```
//!
//! Three properties, and this module now has all three. `.gcmds` **wraps**
//! ([`plan::wrap_group`] decides where). The band has a **fixed height**
//! ([`band_height`]) rather than being as tall as its content. The group is a
//! **column with the caption pinned to the bottom** — `justify-content:
//! space-between` — which is what [`captioned_group`]'s `rows_height`
//! argument buys: every caption in the band sits on one baseline, whether
//! the group above it used one row or two.
//!
//! # ★ The two paddings, and the one that was free
//!
//! Both are in the CSS above and neither was drawn until 2026-08-14. They are
//! not the same kind of change and it is worth being explicit about which is
//! which, because one of them cost nothing and the other one moves the
//! canvas.
//!
//! **`.group { padding: 0 13px }` — free.** [`plan::GROUP_PADDING`] has
//! budgeted 6 pt per side in [`plan::group_width`] since the day the planner
//! was written, and its own doc comment recorded that the renderer never drew
//! it. The band was therefore reserving the space and then spending it as an
//! accidental margin *outside* the group boundary: measured in the running
//! application at 1,100 pt, the Markup tab's Text-markup group box began at
//! x = 322.5 and its first control began at 322.5 as well. Controls sat flush
//! against the group edge and against the rule dividing them from the next
//! group. [`captioned_group`] now insets its body by that same constant, so
//! **no group's planned width changed and no group moved into the overflow
//! menu** — the arithmetic was always right and only the ink was wrong. See
//! [`plan::GROUP_PADDING`] for why 6 pt is the mockup's 13 px rather than a
//! disagreement with it: the mockup's divider is a zero-width `border-right`
//! and this build's is a real `ui.separator()`, so 6 + 14 + 6 lands on the
//! mockup's 26 px from the other direction.
//!
//! **`.band { padding: … 4px }` — not free, and R128 governs it.**
//! [`BAND_PADDING_BOTTOM`] is a real four points of extra ribbon, and the
//! ribbon sits directly above the canvas, so it is added to [`band_height`]'s
//! **derivation** rather than being allowed to fall out of what a group drew.
//! The height stays a function of the theme, the font and two constants; it
//! is still identical on every tab, still identical when every group is in
//! the overflow menu, and `the_band_is_the_same_height_on_every_tab` and
//! `the_band_keeps_its_height_at_widths_where_every_group_overflows` still
//! say so.
//!
//! ## Why the height is fixed, which is the part that is not taste
//!
//! `PROJECT_PLAN.md`'s **R128**. A content-driven height adjacent to a
//! fit-to-viewport zoom is a feedback loop — measured at 230 % → 224 % →
//! 215 % zoom drift — and the ribbon sits in the top panel directly above
//! the canvas. A band that were one row tall on File and two on Markup
//! would therefore change the canvas's rectangle on **every tab click**,
//! and a fit-to-page zoom would chase it.
//!
//! So the height is computed from the theme and the font and **nothing
//! else**: not from how many rows this tab's widest group happened to need,
//! not from how many groups fitted, not from whether the overflow
//! affordance is showing. [`render_band`] reserves it before it draws, and
//! reserves it even when the plan puts *every* group in the menu — which is
//! the case a height derived from drawn content would silently get wrong.
//! `the_band_is_the_same_height_on_every_tab` asserts it, and asserts that
//! the measurement happened rather than that it was vacuously absent.
//!
//! Note what is *not* claimed: a [`crate::theme::Preset`] change does move
//! the band, through `control_height`. That is a deliberate, global, one-off
//! event and not something a tab click can cause, which is the distinction
//! R128 is actually about.
//!
//! # Why the caption is *beneath* the controls, centred
//!
//! Also carried from the salvage source, whose comment records what the
//! alternative looked like when captured from the running application:
//!
//! > ```text
//! > File [Open…] [Save a copy…] Document [Properties] Clipboard [Copy…] …
//! > ```
//! >
//! > — a ~26 px strip in which the captions read as just more small
//! > controls and **the grouping is invisible**.
//!
//! An inline caption is not a smaller version of a ribbon; it is a
//! toolbar with some extra words in it. The one structural cue a ribbon
//! has is a labelled block of related controls, and putting the label
//! beside the block instead of under it removes that cue entirely.
//!
//! Centring needs the row's measured width, which in immediate mode
//! exists only *after* the row is emitted — hence measure-then-allocate
//! rather than a `vertical_centered` wrapper, which would justify to the
//! whole remaining band and scatter the captions across the window.
//!
//! # Overflow
//!
//! The arithmetic lives in [`super::plan`], which explains at length why
//! it is a separate pure module. What happens here is the second half of
//! the enforcement: the overflow control's rectangle is computed **from
//! the band's right edge, before any group is drawn**, and the groups are
//! given a child `Ui` whose `max_rect` stops where that reservation
//! begins. Nothing the group loop does can reach it, because the group
//! loop is not laying out in that space.
//!
//! ## ★ "The band's right edge" is not `available_rect_before_wrap()`
//!
//! That sentence hid a defect for the whole of this module's life, and it
//! is worth spelling out because the wrong version reads perfectly.
//!
//! `egui`'s `Region::expand_to_include_rect` grows a `Ui`'s **`max_rect`**,
//! not only its `min_rect`, whenever a child widget lays out beyond it.
//! The ribbon draws the tab-strip row *before* the band, in the same
//! vertical `Ui`. When the QAT, the tabs and the mode selector do not fit
//! — which is the entire situation the overflow machinery exists for —
//! that row overflows, the enclosing vertical `Ui`'s `max_rect` silently
//! grows to contain it, and the band that is drawn next asks
//! `available_rect_before_wrap()` and is told it has a width the window
//! never had. Observed, at a 180 pt viewport with real font metrics:
//!
//! ```text
//! screen   [   0.0 ..  180.0 ]
//! max_rect [  -7.1 ..  258.1 ]   ← grown by the row above
//! overflow [ 192.7 ..  258.1 ]   ← reserved from a right edge off-screen
//! ```
//!
//! The reservation arithmetic was correct and the affordance was still
//! unreachable: failure mode #8, arrived at through a `Ui` that lied about
//! its width rather than through an ordering mistake. With no font data
//! installed the row always fitted, so no test could see it.
//!
//! The fix is [`entitled_bounds`]: the band lays out inside the rectangle the
//! ribbon was **handed**, intersected with what is actually on screen
//! (`clip_rect`), and never inside whatever a sibling's overflow grew the
//! parent to. `render_band` takes that rectangle as an argument rather
//! than deriving it, because the only `Ui` that knows it is the one the
//! application passed to [`super::Ribbon::render`], before anything was
//! drawn into it.
//!
//! ## When the band is narrower than the affordance itself
//!
//! Something has to give, and #8 dictates what: not the affordance. The
//! rectangle is clamped into the band (`left = max(band.left, band.right −
//! reserved)`), so the control is always fully on screen and always
//! hit-testable; what gives instead is its **label**, which truncates.
//! And it is disclosed — `ribbon-overflow-affordance-clamped` — because a
//! control silently rendering at less than the size it asked for is
//! exactly the kind of degradation that is invisible until somebody
//! screenshots it.

use egui::{Align, Atoms, Layout, Rect, RichText, TextStyle, UiBuilder, Vec2, pos2, vec2};

use crate::manifest::{Group, Item};

use super::a11y;
use super::ctx::{Ctx, CustomItem, IconRequest};
use super::plan::{self, BandPlan, CUSTOM_ITEM_WIDTH, GroupRows, ItemWidths};
use super::report;

/// Vertical gap between a group's control row and its caption.
///
/// Small on purpose: the caption must read as belonging to the row above
/// it rather than as a line of its own. The salvage source used the same
/// constant for the same reason.
const CAPTION_GAP: f32 = 2.0;

/// Clear space between the band's captions and whatever the application puts
/// underneath the ribbon.
///
/// `mockups/ribbon.html`'s `.band { padding: 8px 10px 4px }` — the third
/// figure. Measured in the running application before it existed: the group
/// captions ended at y = 103 and the dock's tab bar began at y = 105.3, so a
/// 10 pt caption drawn `weak()` and `small()` was separated from the panel
/// header below it by rather less than a line of its own leading. The caption
/// is the one piece of text that says what a block of controls is *for* (see
/// this module's header on why it is beneath the controls at all), and a
/// caption sitting on the seam reads as a label for the thing below it.
///
/// # Why this is added to [`band_height`] and not to the group loop
///
/// `PROJECT_PLAN.md`'s **R128**: the band's height must be a function of the
/// theme and the font and of nothing a tab can vary. Padding drawn as "space
/// after the last group" would be exactly such a variation — a tab whose
/// groups all went into the overflow menu draws no group and would get no
/// padding, so the band would be 4 pt shorter on that tab than on the one
/// beside it and the canvas below would move on a tab click. Folding it into
/// the derivation instead keeps one number, reserved before anything is
/// drawn, spent identically whether the band holds five groups or none.
///
/// # Why the mockup's top and side padding are not here
///
/// Only the bottom edge was measured as wrong. The band's top is already
/// separated from the tab strip by [`super::tabs::strip_underline`] and the
/// enclosing layout's own spacing, and the band's horizontal padding is a
/// decision about the *band's* left edge that would shift every group in it —
/// see [`plan::GROUP_PADDING`]'s closing note. Adding either one would be
/// visual churn beyond the defect, and churn is harder to review than the
/// change it is mixed into.
pub(super) const BAND_PADDING_BOTTOM: f32 = 4.0;

/// The condition-name prefix that marks a command as currently *on*.
///
/// # Why toggles are expressed as a condition rather than as a field
///
/// A ribbon has toggles: "Single page" is either the current page-display
/// mode or it is not, and a control that cannot show which is a control
/// the operator has to test by clicking. But *which* toggle is on is
/// application state, and [`crate::commands::Command`] deliberately holds
/// no state — it is a registration, built once, shared, `Clone`.
///
/// The [`crate::commands::ConditionSet`] already exists, is already
/// republished every frame, and already carries exactly this kind of
/// fact. So a command with id `view.single` renders selected while the
/// condition `selected:view.single` is set. No new type, no new manifest
/// field, no per-frame registry mutation, and the state is inspectable in
/// the same place every other piece of frame state is.
///
/// The prefix uses `:` rather than `.` so it cannot collide with an
/// application's own dotted condition names.
pub const SELECTED_CONDITION_PREFIX: &str = "selected:";

/// The condition name that reports command `id` as currently on.
#[must_use]
pub fn selected_condition(command_id: &str) -> String {
    format!("{SELECTED_CONDITION_PREFIX}{command_id}")
}

/// What one band did, for the frame report and for the caption
/// invariant's test.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct BandOutcome {
    /// How many groups were drawn, in the band and in the overflow menu
    /// together.
    pub groups_rendered: usize,
    /// How many of those were drawn **in the band itself**.
    ///
    /// Recorded as the value [`Self::groups_rendered`] had reached when
    /// the band's own loop finished, *not* as the plan's `shown`. The
    /// distinction is the whole reason this field exists: a counter's
    /// earlier value is provably ≤ its later value, so
    /// `groups_rendered − groups_in_band` — "how many are in the menu" —
    /// cannot underflow whatever the plan said and whatever the popup
    /// did. Deriving the same number by subtracting the *plan's* hidden
    /// count mixes a count of what was drawn with a count of what was
    /// intended, and those two disagree on every frame the menu is shut.
    pub groups_in_band: usize,
    /// How many captions were emitted. **Must equal
    /// [`Self::groups_rendered`].**
    pub captions_emitted: usize,
    /// How many groups the plan moved into the overflow menu.
    pub groups_overflowed: usize,
    /// Whether the overflow affordance was drawn.
    pub overflow_visible: bool,
    /// The `egui::Id` of the overflow affordance, when one was drawn.
    ///
    /// Carried out so a test — or a harness — can ask `egui` itself
    /// whether the control is hit-testable. A rectangle proves a thing
    /// was allocated; only a hit test proves it can be reached.
    pub overflow_id: Option<egui::Id>,
}

/// The caption a group will be drawn with — never empty.
///
/// The manifest's caption is optional because a *layer* may omit it (a
/// layer that says `Group(id: "render")` is reordering a group, not
/// blanking its caption). A complete manifest is required to have one by
/// [`crate::manifest::Shell::validate`].
///
/// This is what happens when an application renders a manifest it did not
/// validate. Falling back to the **id** rather than to `""` is the whole
/// point: an empty caption reproduces the exact defect this module exists
/// to prevent, whereas `page_display` in the caption slot is visibly
/// wrong, unmistakably diagnostic, and names the group whose manifest
/// entry needs fixing.
#[must_use]
pub(crate) fn caption_text(group: &Group) -> &str {
    match group.caption.as_deref() {
        Some(c) if !c.trim().is_empty() => c,
        _ if !group.id.is_empty() => &group.id,
        _ => "(unnamed group)",
    }
}

/// The vertical shape every group in a band is drawn to.
///
/// Two numbers rather than one, because they pin two different things and
/// a group that satisfied only the first would still make the band ragged:
///
/// - [`Self::rows`] pins the **captions** to one baseline — the mockup's
///   `justify-content: space-between`. A one-row group is padded out to the
///   height two rows would have taken, so its caption lands where its
///   two-row neighbour's does.
/// - [`Self::total`] pins the **band** to one height — R128. It closes the
///   gap between what the reservation promised and what the caption
///   actually measured, so that a band showing five groups and a band
///   showing none (everything in the overflow menu) come out identical
///   rather than identical-to-within-a-caption's-rounding.
///
/// [`Self::NATURAL`] — both zero — means "as tall as your content", which
/// is what the overflow menu wants: the band's height is a fact about the
/// band, and padding a popup entry out to it would put a hole under every
/// one-row group in the menu.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct GroupBox {
    /// Height the control rows are padded out to, before the caption.
    rows: f32,
    /// Height the whole group — rows, gap and caption — is padded out to.
    total: f32,
}

impl GroupBox {
    /// Pad to nothing: the group is as tall as what it drew.
    const NATURAL: Self = Self {
        rows: 0.0,
        total: 0.0,
    };
}

/// How far a group's **cursor** has advanced once [`plan::GROUP_ROWS`] rows
/// have been laid out — which is the number a shorter group is padded out
/// to, and what pins every caption in the band to one baseline.
///
/// # ★ `GROUP_ROWS × (control_height + item_spacing)`, and the trailing
/// ★ term is the one that matters
///
/// The obvious spelling is `rows × height + (rows − 1) × spacing`: two rows
/// with one gap between them. That is the right answer for the *ink* and
/// the wrong one for the **cursor**, because `egui` advances the cursor past
/// every laid-out rect by `item_spacing` — after the last row as much as
/// after the first. So a two-row group's cursor sits one gap beyond that
/// figure, its padding computes as zero, and the group ends up exactly
/// `item_spacing` taller than its one-row neighbour, whose padding *was*
/// applied and did land on the figure.
///
/// **That defect shipped into a build and no test in this crate could see
/// it.** `super::width_tests`' context installs a font but does not apply a
/// [`crate::theme::Theme`], so `egui`'s default `interact_size.y` (18 pt) is
/// well under the theme's `control_height` (24 pt) and every row had 6 pt of
/// slack for the stray gap to hide in. In the running application the two
/// are equal by construction — `Theme::apply` sets
/// `spacing.interact_size.y = control_height` — there is no slack, and the
/// band's own trace showed Shapes at 68 pt beside Text markup at 64.
/// `super::height_tests::context` now applies the theme for exactly this
/// reason: `HANDOFF.md` §10's *"a fixture can flatter the thing it
/// measures"*, arriving through spacing rather than through a curve.
fn rows_height(ui: &egui::Ui, ctx: &Ctx<'_>) -> f32 {
    #[allow(clippy::cast_precision_loss)] // single digits
    let n = plan::GROUP_ROWS.max(1) as f32;
    (ctx.theme.metrics.control_height + ui.spacing().item_spacing.y) * n
}

/// **The band's height, on every tab, whatever it contains.**
///
/// Four terms, in the order they are drawn: the control rows, the gap, one
/// line of caption, and [`BAND_PADDING_BOTTOM`]. Derived from the theme, the
/// font and two constants, and from nothing the manifest can vary — see the
/// module header on R128 for why that independence is the whole point.
///
/// The bottom padding belongs **in this derivation** rather than in the group
/// loop for the reason [`BAND_PADDING_BOTTOM`] gives at length: space emitted
/// after the last group would be absent on a tab that drew no group, which is
/// a reachable state (every group in the overflow menu) and would make the
/// height content-derived through the back door.
///
/// `pub(crate)` so a test can state the claim in the same terms the
/// renderer does rather than by re-deriving it.
pub(crate) fn band_height(ui: &egui::Ui, ctx: &Ctx<'_>) -> f32 {
    rows_height(ui, ctx)
        + CAPTION_GAP
        + ui.text_style_height(&TextStyle::Small)
        + BAND_PADDING_BOTTOM
}

/// The rectangle a ribbon row is entitled to lay out in.
///
/// Shared by the band and by [`super::strip`], because both rows have the
/// same obligation and the same trap: whatever a row is *offered* by the
/// layout is not necessarily a width the window has.
///
/// Three candidates, and the row gets the **narrowest** of them, because
/// each is an upper bound on where a control can be both drawn and
/// clicked:
///
/// 1. `ui.available_rect_before_wrap()` — where this `Ui`'s cursor is now.
///    Supplies the top edge and the left edge in the ordinary case.
/// 2. `entitled` — the rectangle the application handed
///    [`super::Ribbon::render`], captured **before** anything was drawn
///    into it. This is the one that matters: see the module header on
///    `max_rect` growth. Nothing a sibling row does can inflate it,
///    because it was read before the sibling existed.
/// 3. `ui.clip_rect()` — what is on screen. `egui` never grows a clip rect
///    to fit overflowing content, so it is the honest answer to "would the
///    operator see a pixel painted here", which is what failure mode #8 is
///    ultimately about.
///
/// Only the horizontal extent is negotiated. The vertical extent is the
/// caller's — [`render_band`] replaces the bottom edge with
/// `top + `[`band_height`] immediately, and clamping the height to the clip
/// rect instead would make a partially-scrolled ribbon lay its captions out
/// differently from an unscrolled one.
///
/// A degenerate result (right ≤ left) is returned as a zero-width rect at
/// the left edge rather than as an inverted one: [`plan::plan_band`] reads
/// zero as "nothing fits, everything goes to the menu", which is the safe
/// answer, whereas an inverted rect would produce a negative width and a
/// nonsense plan.
pub(crate) fn entitled_bounds(ui: &egui::Ui, entitled: Rect) -> Rect {
    let cursor = ui.available_rect_before_wrap();
    let clip = ui.clip_rect();
    let left = cursor.left().max(entitled.left()).max(clip.left());
    let right = cursor
        .right()
        .min(entitled.right())
        .min(clip.right())
        .max(left);
    Rect::from_min_max(pos2(left, cursor.top()), pos2(right, cursor.bottom()))
}

/// Draw the band for one tab: its groups, left to right, with a vertical
/// rule between them, and an overflow affordance if they do not fit.
///
/// `entitled` is the rectangle the application handed the ribbon, read
/// before any of the ribbon was drawn. It is a parameter rather than
/// something this function derives because by the time the band runs, the
/// `Ui` it is given can no longer report it — see [`entitled_bounds`] and the
/// module header.
pub(crate) fn render_band(
    ui: &mut egui::Ui,
    ctx: &mut Ctx<'_>,
    tab_id: &str,
    groups: &[Group],
    entitled: Rect,
) -> BandOutcome {
    let mut outcome = BandOutcome::default();

    let gutter = ctx.theme.metrics.gutter;
    let separator = separator_width(ui);
    let measured: Vec<(GroupRows, f32)> =
        groups.iter().map(|g| measure_group(ui, ctx, g)).collect();
    let widths: Vec<f32> = measured.iter().map(|(_, w)| *w).collect();
    let reserve = plan::overflow_width(groups.len(), button_padding(ui), |s| {
        text_width(ui, s, &TextStyle::Button)
    });

    ui.horizontal(|ui| {
        // ★ R128. The band's height is reserved here, from the theme, before
        // anything is drawn and regardless of what the plan is about to
        // decide — so a tab whose groups all went into the overflow menu is
        // exactly as tall as one whose groups all fitted. A height taken
        // from what was drawn would differ between those two, and the
        // difference would move the canvas underneath. See the module
        // header.
        let height = band_height(ui, ctx);
        let box_ = GroupBox {
            rows: rows_height(ui, ctx),
            total: height,
        };
        ui.set_min_height(height);

        // The vertical extent is the band's own, not whatever the enclosing
        // `Ui` had left over: `entitled_bounds` negotiates width alone and
        // hands back the caller's bottom edge.
        let offered = entitled_bounds(ui, entitled);
        let full = Rect::from_min_max(
            offered.min,
            pos2(offered.right(), offered.top() + height.max(0.0)),
        );
        let band_plan = plan::plan_band(full.width(), &widths, separator, reserve);

        // ★ The reservation, taken from the right edge BEFORE any group
        // is drawn. `overflow_rect` is computed from `full.right()` and
        // from nothing the group loop can influence, so failure mode #8
        // — the overflow control being the thing that gets squeezed out —
        // is not reachable from here. See `plan`'s module header.
        //
        // `left` is clamped into the band. When the band is narrower than
        // the affordance the subtraction alone would put the control's
        // left edge off screen, which is #8 again with the affordance
        // present-but-unreachable rather than absent. Clamping spends the
        // shortfall on the label instead — see the module header.
        let overflow_rect = band_plan.has_overflow().then(|| {
            let desired_left = full.right() - band_plan.overflow_width;
            let left = desired_left.max(full.left());
            if left > desired_left {
                crate::verify::event("ribbon-overflow-affordance-clamped")
                    .kv("tab", tab_id)
                    .kv("band_width", format!("{:.1}", full.width()))
                    .kv("reserved", format!("{:.1}", band_plan.overflow_width))
                    .emit();
            }
            Rect::from_min_max(
                pos2(left, full.top()),
                pos2(full.right(), full.top() + ctx.theme.metrics.control_height),
            )
        });

        let groups_rect = Rect::from_min_max(
            full.min,
            pos2(
                (full.left() + band_plan.group_budget).min(full.right()),
                full.bottom(),
            ),
        );

        ui.scope_builder(
            UiBuilder::new()
                .id_salt("egui-shell-ribbon-groups")
                .max_rect(groups_rect)
                .layout(Layout::left_to_right(Align::Min)),
            |ui| {
                ui.set_max_width(groups_rect.width());
                for (index, group) in groups.iter().take(band_plan.shown).enumerate() {
                    // Separator BEFORE the group rather than after, so the
                    // band never ends with a trailing rule and the first
                    // group never starts with one.
                    if index > 0 {
                        ui.separator();
                    }
                    captioned_group(
                        ui,
                        ctx,
                        tab_id,
                        group,
                        gutter,
                        &measured[index].0,
                        box_,
                        &mut outcome,
                    );
                }
            },
        );

        // Snapshotted here, between the band's loop and the menu's, so the
        // "how many are in the menu" subtraction downstream is a counter
        // minus its own earlier value. See `BandOutcome::groups_in_band`.
        outcome.groups_in_band = outcome.groups_rendered;

        if let Some(rect) = overflow_rect {
            render_overflow(
                ui,
                ctx,
                tab_id,
                groups,
                &measured,
                &band_plan,
                gutter,
                &mut outcome,
                rect,
            );
        }
    });

    // The invariant, restated where it can fail loudly in a debug build.
    // The release-mode guarantee is structural (there is one drawing
    // path and it emits the caption itself); this is the tripwire for an
    // edit that adds a second one.
    debug_assert_eq!(
        outcome.groups_rendered, outcome.captions_emitted,
        "a ribbon group was drawn without a caption — every group must go \
         through `captioned_group`, which is the only function that draws one"
    );
    outcome
}

/// **The only function in this crate that draws a ribbon group.**
///
/// Lays one group out as Office lays one out: its controls in up to
/// [`plan::GROUP_ROWS`] rows, its **caption beneath them, centred on the
/// widest row**. The body is a closure rather than a predicate for the
/// reason the salvage source records: to put the caption *under* the
/// controls, the controls must be emitted inside a vertical container that
/// is still open when the caption is written, and a predicate returning
/// `bool` has already returned before the body runs.
///
/// See the module header for why this being the only such function is the
/// point.
///
/// # `rows`
///
/// The split [`plan::wrap_group`] decided, handed in rather than recomputed
/// — see [`GroupRows`] for why the plan and the renderer must not each own
/// a copy of that arithmetic.
///
/// # `box_`
///
/// The heights this group is padded out to — see [`GroupBox`], and
/// [`GroupBox::NATURAL`] for the overflow menu, where neither applies.
///
/// The padding is measured against the `Ui`'s own **cursor** rather than
/// predicted, so a control that turned out taller than
/// [`crate::theme::Metrics::control_height`] shortens the gap instead of
/// pushing the caption out of the band. The cursor, specifically, and not
/// `min_rect`: `egui` advances the cursor past a laid-out rect by
/// `item_spacing`, so the two differ by exactly one gap after every row and
/// padding against the wrong one leaves each group a gap taller than the
/// height the band reserved for it. That is a 3 pt discrepancy that shows
/// up only when a tab has *no* group in the band to compare against — which
/// is to say, only in the R128 case.
///
/// # The horizontal inset
///
/// The mockup's `.group { padding: 0 13px }`, drawn at
/// [`plan::GROUP_PADDING`] — the width [`plan::group_width`] has budgeted for
/// it all along, so this inset is paid for out of a reservation that already
/// existed and costs the band nothing. See the module header.
///
/// The reported rectangle **includes** the inset, deliberately: the group box
/// is what the operator perceives as the group, and a report that named only
/// the content would make "is there padding" unanswerable from outside the
/// process, which is the question that produced this change.
#[allow(clippy::too_many_arguments)]
fn captioned_group(
    ui: &mut egui::Ui,
    ctx: &mut Ctx<'_>,
    tab_id: &str,
    group: &Group,
    gutter: f32,
    rows: &GroupRows,
    box_: GroupBox,
    outcome: &mut BandOutcome,
) {
    outcome.groups_rendered += 1;

    // ★ The group's own horizontal padding — `.group { padding: 0 13px }`.
    //
    // `horizontal_top` rather than `horizontal`: the latter is
    // `left_to_right(Align::Center)` and would centre a group vertically
    // within the band's reserved height, which is precisely the pinning the
    // `box_` arithmetic below exists to do by hand. `horizontal_top` is the
    // same layout with `Align::Min`, i.e. exactly what a bare `ui.vertical`
    // in the band's own `left_to_right(Align::Min)` did before this wrapper
    // existed.
    //
    // `item_spacing.x = 0` because `egui` advances the cursor past a
    // laid-out rect by `item_spacing` and `add_space` adds to that: without
    // this the trailing pad would be `GROUP_PADDING + item_spacing.x` and
    // the group would be one gutter wider than the plan budgeted for it.
    // The leading pad has no such term (nothing has been laid out yet), so
    // an asymmetric group is exactly what forgetting this line produces.
    // Setting `.x` alone leaves `item_spacing.y` — which `rows_height`
    // budgets against — untouched.
    let whole = ui
        .horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.add_space(plan::GROUP_PADDING);
            group_body(ui, ctx, tab_id, group, gutter, rows, box_, outcome);
            ui.add_space(plan::GROUP_PADDING);
        })
        .response
        .rect;

    ctx.reporter
        .report(whole, || report::group(tab_id, &group.id));
}

/// The inside of a group: its control rows, then its caption, padded to
/// [`GroupBox`].
///
/// Split out of [`captioned_group`] only so the horizontal inset above reads
/// as one line rather than as a closure wrapping a closure. **It is not a
/// second drawing path** — [`captioned_group`] is the only caller and the
/// only function that can reach it, so the module header's invariant ("every
/// group goes through one closure, which emits the caption itself") is
/// unchanged: `outcome.captions_emitted` is still incremented on a line that
/// cannot be reached without the label having been drawn.
#[allow(clippy::too_many_arguments)]
fn group_body(
    ui: &mut egui::Ui,
    ctx: &mut Ctx<'_>,
    tab_id: &str,
    group: &Group,
    gutter: f32,
    rows: &GroupRows,
    box_: GroupBox,
    outcome: &mut BandOutcome,
) {
    ui.vertical(|ui| {
        // The controls FIRST: the widest row's width is what the
        // caption is then centred within, and that width only exists
        // after the rows have been emitted.
        let items = group.items();
        let top = ui.cursor().top();
        let mut widest = 0.0_f32;
        let mut at = 0_usize;
        for &count in &rows.counts {
            let end = (at + count).min(items.len());
            let row = ui
                .horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = gutter;
                    for item in &items[at..end] {
                        render_item(ui, ctx, tab_id, &group.id, item);
                    }
                })
                .response
                .rect;
            widest = widest.max(row.width());
            at = end;
        }

        // ★ The caption is pinned to the bottom of the band's row area,
        // not to the bottom of whatever this group happened to draw. A
        // one-row group and a two-row group therefore caption on the
        // same baseline — `justify-content: space-between`, and the
        // reason the mockup's band reads as staged rather than ragged.
        ui.add_space((box_.rows - (ui.cursor().top() - top)).max(0.0));
        ui.add_space(CAPTION_GAP);

        let caption = ui
            .allocate_ui_with_layout(vec2(widest, 0.0), Layout::top_down(Align::Center), |ui| {
                ui.label(RichText::new(caption_text(group)).weak().small())
            })
            .inner;

        // Counted here, one line after the label that cannot be
        // reached without emitting it.
        outcome.captions_emitted += 1;
        ctx.reporter
            .report(caption.rect, || report::group_caption(tab_id, &group.id));

        // ★ And out to the band's own height. `allocate_space` rather
        // than `add_space`, because only an allocation grows a `Ui`'s
        // `min_rect` — `add_space` moves the cursor, which is what the
        // padding above it wanted and is exactly not what this wants.
        // Zero-width, so it changes nothing horizontally.
        //
        // What this closes: `band_height` predicts the caption's height
        // from `TextStyle::Small`'s row height, and the label allocates
        // whatever its galley measured. The two agree in every font this
        // crate has been run against, and if they ever stop agreeing the
        // band would be a fraction taller with groups in it than
        // without — R128 by a hair rather than by a row.
        // `the_band_keeps_its_height_at_widths_where_every_group_overflows`
        // is the tripwire.
        let drawn = ui.cursor().top() - top;
        if box_.total > drawn {
            ui.allocate_space(vec2(0.0, box_.total - drawn));
        }
    });
}

/// The "⏷ N more" affordance and the menu behind it.
///
/// The rectangle is supplied by the caller, computed from the band's
/// right edge before any group was laid out — see [`render_band`].
#[allow(clippy::too_many_arguments)]
fn render_overflow(
    ui: &mut egui::Ui,
    ctx: &mut Ctx<'_>,
    tab_id: &str,
    groups: &[Group],
    measured: &[(GroupRows, f32)],
    band_plan: &BandPlan,
    gutter: f32,
    outcome: &mut BandOutcome,
    rect: Rect,
) {
    outcome.groups_overflowed = band_plan.hidden;

    let label = plan::overflow_label(band_plan.hidden);
    let id = ctx.id("overflow", tab_id);
    let response = ui
        .scope_builder(
            UiBuilder::new()
                .id_salt(id)
                .max_rect(rect)
                .layout(Layout::left_to_right(Align::Center)),
            |ui| {
                ui.set_max_width(rect.width());
                // `min_size` fills the reservation, so the control is
                // exactly as big as the arithmetic promised. `truncate`
                // is the other half: without it a label wider than the
                // rect makes the button wider than the rect, and the
                // affordance would hang off the band's right edge in the
                // one situation — a band narrower than its own
                // reservation — where it most needs to be reachable.
                // Truncating spends the shortfall on characters, which is
                // recoverable (the tooltip states the count in full),
                // where spending it on position is not.
                ui.add(
                    egui::Button::new(RichText::new(&label))
                        .min_size(rect.size())
                        .truncate(),
                )
            },
        )
        .inner;

    outcome.overflow_visible = true;
    outcome.overflow_id = Some(response.id);
    ctx.reporter
        .report_static(response.rect, report::overflow());

    // The affordance is a real control with a real accessible name: the
    // count is the information, and "button" would be as useless here as
    // it is on an icon.
    let announced = format!("{label} ribbon groups");
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, true, announced.clone())
    });
    let response = response.on_hover_text(format!(
        "{} group{} do not fit; open them here",
        band_plan.hidden,
        if band_plan.hidden == 1 { "" } else { "s" }
    ));

    egui::Popup::menu(&response).show(|ui| {
        // Overflowed groups go through the SAME closure as visible ones,
        // and with the SAME row split, so a group reads identically
        // whichever side of the affordance it is on. A second, simpler
        // drawing path for the menu is exactly how the two caption-less
        // groups in the salvage source happened.
        //
        // `rows_height` is 0 here, and deliberately: the band's fixed height
        // is a fact about the band, and padding a menu entry out to it would
        // put a hole under every one-row group in the popup.
        for ((rows, _), group) in measured.iter().zip(groups).skip(band_plan.shown) {
            captioned_group(
                ui,
                ctx,
                tab_id,
                group,
                gutter,
                rows,
                GroupBox::NATURAL,
                outcome,
            );
            ui.separator();
        }
    });
}

/// Draw one item of a group.
fn render_item(ui: &mut egui::Ui, ctx: &mut Ctx<'_>, tab_id: &str, group_id: &str, item: &Item) {
    match item {
        Item::Separator => {
            ui.separator();
        }
        Item::Command(id) => {
            render_command(ui, ctx, id);
        }
        Item::Custom { kind, payload } => {
            let request = CustomItem {
                kind,
                payload: payload.as_deref(),
                tab: tab_id,
                group: group_id,
            };
            // `take` so the borrow of `ctx.custom` does not conflict with
            // `ctx.invoke`; put back immediately, because a renderer that
            // vanished after the first custom item would be a very
            // confusing bug.
            if let Some(renderer) = ctx.custom.take() {
                let token = renderer(ui, &request);
                ctx.custom = Some(renderer);
                if let Some(token) = token {
                    ctx.invoke(token);
                }
            } else {
                // No renderer: reserve the space the plan budgeted for
                // it, so the band's arithmetic stays true and the gap is
                // visible rather than silently closing up. An application
                // that put a custom item in its manifest and supplied no
                // renderer has a defect, and a hole is how it finds out.
                crate::verify::event("ribbon-custom-item-unrendered")
                    .kv("kind", kind)
                    .kv("group", group_id)
                    .emit();
                ui.allocate_space(vec2(CUSTOM_ITEM_WIDTH, ctx.theme.metrics.control_height));
            }
        }
    }
}

/// Draw one command control, honouring its enable predicate and its
/// selected condition.
fn render_command(ui: &mut egui::Ui, ctx: &mut Ctx<'_>, id: &str) {
    let Some(command) = ctx.command(id).cloned() else {
        return;
    };
    let enabled = command.is_enabled(ctx.conditions);
    let selected = ctx.conditions.is_set(&selected_condition(&command.id));

    // A band control always shows its label. Icon-only belongs to the
    // QAT, where the operator has four controls they use constantly and
    // has learned the glyphs; in the band there are forty and the label
    // is the only thing that makes one findable.
    let response = command_button(ui, ctx, &command, true, selected, enabled, false);

    // ★ **Where this control was drawn** — published on the frame it was
    // drawn, under the stable name [`report::band_item`] builds.
    //
    // The band used to report its groups and their captions and nothing
    // else, which made every *command* in the ribbon unlocatable from
    // outside the process. A caption's rect answers "is this label
    // legible"; it cannot answer "did clicking Rectangle arm anything",
    // because nothing outside the window could find the Rectangle button
    // in order to click it. So the only evidence available for a ribbon
    // click's whole chain — click → dispatch → tool armed → control
    // renders pressed — was a set of unit tests, one per link, none of
    // which observes the links being connected. That is precisely the
    // shape of the icon-painter defect this crate already shipped: every
    // part tested, the join untested, the join wrong.
    //
    // Reported for **every** command, enabled or disabled, selected or
    // not, in the band and in the overflow menu alike — because the
    // question a consumer asks is *where is this control*, and a control
    // that is greyed is still a control that was drawn somewhere. A
    // report conditioned on state would go quiet in exactly the cases a
    // harness most wants to look at.
    //
    // The shell learns nothing about what the id *means*. It publishes
    // that a control registered under some id occupied some rectangle;
    // what `markup.rectangle` is for is the application's business, and
    // this crate could not name it without becoming a PDF viewer.
    ctx.reporter
        .report(response.rect, || report::band_item(&command.id));

    a11y::describe_command(&response, &command, true, enabled);
    let response = match (&command.tooltip, enabled) {
        (Some(tip), true) => response.on_hover_text(tip),
        (Some(tip), false) => response.on_disabled_hover_text(tip),
        (None, _) => response,
    };

    if response.clicked() {
        ctx.invoke(command.handler);
        crate::verify::event("ribbon-command-invoked")
            .kv("id", &command.id)
            .kv("handler", command.handler.get())
            .emit();
    }
}

/// The button itself: an optional icon slot, an optional label, the
/// selected state, and the icon painting seam.
///
/// Shared with [`super::qat`], which is why it lives here and takes
/// `shows_label`.
///
/// # `truncate`
///
/// Whether the label may lose characters rather than the button losing
/// its place. `true` on the tab-strip row, `false` in the band, and the
/// asymmetry is deliberate:
///
/// - A **band** control that does not fit is in a group the plan has
///   already decided is visible, inside a `Ui` whose `max_rect` stops
///   before the overflow affordance. Truncating it would hide a command's
///   name to save a few points that the reservation has already accounted
///   for.
/// - A **strip** control has nowhere to go. The QAT is a fixed cost with
///   no menu behind it, and the active tab is pinned out of the strip's
///   own menu ([`plan::plan_tab_strip`]). When either is wider than the
///   room the row can give it, the only alternatives are "truncate" and
///   "draw off the edge of the window", and the second one is the defect.
pub(crate) fn command_button(
    ui: &mut egui::Ui,
    ctx: &mut Ctx<'_>,
    command: &crate::commands::Command,
    shows_label: bool,
    selected: bool,
    enabled: bool,
    truncate: bool,
) -> egui::Response {
    let icon_size = ctx.theme.metrics.icon_pts;
    let icon_slot = command
        .icon
        .as_ref()
        .map(|key| (key.clone(), ctx.id("icon", &command.id)));

    let mut atoms = Atoms::default();
    if let Some((_, slot_id)) = &icon_slot {
        atoms.push_right(egui::Atom::custom(*slot_id, Vec2::splat(icon_size)));
    }
    if shows_label || icon_slot.is_none() {
        // The `||` is the accessibility floor: a command with no icon key
        // draws its label even in an icon-only context, because a control
        // with neither an icon nor a label is an empty rectangle.
        atoms.push_right(RichText::new(&command.label));
    }

    let laid_out = ui
        .scope(|ui| {
            if !enabled {
                ui.disable();
            }
            let mut button = egui::Button::new(atoms).selected(selected);
            if truncate {
                button = button.truncate();
            }
            button.atom_ui(ui)
        })
        .inner;

    if let Some((key, slot_id)) = icon_slot
        && let Some(rect) = laid_out.rect(slot_id)
        && let Some(painter) = ctx.icons.take()
    {
        let visuals = ui.style().interact(&laid_out.response);
        painter(
            ui.painter(),
            &IconRequest {
                key: &key,
                rect,
                tint: visuals.fg_stroke.color,
                enabled,
                selected,
            },
        );
        ctx.icons = Some(painter);
    }

    laid_out.response
}

// ---------------------------------------------------------------------
// Measurement
// ---------------------------------------------------------------------

/// **How one group wraps, and the width it will occupy once it has.**
///
/// The two answers come from one call because they are one decision: a
/// group's width *is* its widest row, and its widest row is decided by the
/// wrap. Returning them together is what makes it impossible for
/// [`render_band`] to plan against one split and draw another — the failure
/// that would show up as a clipped group and never as anything a reader
/// would recognise as a bug.
fn measure_group(ui: &egui::Ui, ctx: &Ctx<'_>, group: &Group) -> (GroupRows, f32) {
    let widths: Vec<f32> = group
        .items()
        .iter()
        .map(|item| measure_item(ui, ctx, item))
        .collect();
    let rows = plan::wrap_group(
        &widths,
        ctx.theme.metrics.gutter,
        plan::GROUP_ROWS,
        plan::GROUP_WRAP_WIDTH,
    );
    let caption = text_width(ui, caption_text(group), &TextStyle::Small);
    let width = plan::group_width(&rows, caption);
    (rows, width)
}

/// The width one item will occupy.
///
/// # Two corrections that only matter once text has a width
///
/// **The icon/label gap is `icon_spacing`, not `gutter`.** A control with
/// both halves is drawn as an `egui::Atoms` row, and `AtomLayout` spaces
/// its atoms by `ui.spacing().icon_spacing`
/// (`egui-0.35.0/src/atomics/atom_layout.rs`), which this crate's theme
/// does not set and therefore leaves at `egui`'s 4 pt. Estimating the gap
/// as the theme's `gutter` agrees with that by coincidence at the compact
/// density and **under**-estimates by 4 pt per control at the comfortable
/// one. Under-estimating is the dangerous direction — it is the direction
/// that lets a group spill into space the band has already promised to
/// something else — so the estimate asks `egui` for the number `egui` will
/// use.
///
/// **A separator inside a group costs its line, not its line plus two
/// gaps.** [`separator_width`] is the cost of a rule *between two groups*,
/// which includes the `item_spacing` either side of it. Inside a group,
/// [`plan::group_width`] already adds one gutter between every adjacent
/// pair — including the pairs the separator forms with its neighbours — so
/// charging the full inter-group figure here counts those two gaps twice.
/// It over-estimated rather than under-estimated, so it hid a group early
/// instead of clipping one, which is why nothing caught it; it was still
/// wrong by 2 × `gutter` for every separator in the manifest.
fn measure_item(ui: &egui::Ui, ctx: &Ctx<'_>, item: &Item) -> f32 {
    match item {
        Item::Separator => SEPARATOR_LINE,
        Item::Custom { .. } => CUSTOM_ITEM_WIDTH,
        Item::Command(id) => match ctx.registry.get(id) {
            // An unknown id draws nothing (see `Ctx::command`), so it must
            // also measure nothing — otherwise the band reserves space for
            // a control that will not appear and the plan is wrong by
            // exactly the width of every stale reference in the manifest.
            None => 0.0,
            Some(command) => ItemWidths {
                icon: if command.icon.is_some() {
                    ctx.theme.metrics.icon_pts
                } else {
                    0.0
                },
                text: text_width(ui, &command.label, &TextStyle::Button),
                gap: ui.spacing().icon_spacing,
                padding: button_padding(ui),
            }
            .total(),
        },
    }
}

/// The horizontal padding `egui` will add inside a button, both sides.
///
/// `pub(crate)` because the tab strip budgets buttons too — a tab, a QAT
/// control and a band control are all `egui::Button`s and must be measured
/// with the same constants, or one row's estimate disagrees with another's
/// for no reason a reader could find.
pub(crate) fn button_padding(ui: &egui::Ui) -> f32 {
    ui.spacing().button_padding.x * 2.0
}

/// **★ The narrowest an `egui::Button` can be drawn — the floor
/// `truncate()` cannot go below.**
///
/// # Why this number decides the whole tab-strip row
///
/// `Button::truncate()` shortens a label to the room available, which
/// sounds like it can shrink to nothing and cannot. `egui` lays a
/// truncated label out as *the ellipsis* plus the button's own padding,
/// and stops there. Measured against the synthetic face of
/// [`super::testfont`], asking a `"Save a copy…"` button to lay itself out
/// in rooms from 0 to 80 pt:
///
/// ```text
/// room     0     2     6    10    14    20    26    40    80
/// width  19.7  19.7  19.7  19.7  19.7  19.7  19.7  34.7  74.7
///        └──────────── the floor ────────────┘ └─ room − 5.3 ─┘
/// ```
///
/// 19.6875 = 4 + 4 of `button_padding` plus 11.6875 of `…`. Below about
/// 25 pt of room the button simply **overflows the space it was given**,
/// silently, because `egui` does not clip children to a `Ui`'s `max_rect`.
///
/// The consequence is the one rule the tab-strip row is built on: a region
/// gets either **at least this much width, or none at all**. Granting a
/// sliver produces a control drawn outside its own rectangle, on top of
/// its neighbour — which is exactly the class of defect
/// [`super::strip`] exists to retire, arrived at by trying to be
/// accommodating. See [`super::plan::plan_strip_row`], which takes this as
/// its `button_floor`.
///
/// Measured from the live style rather than written down as a constant,
/// because both terms are theme- and font-dependent: `button_padding` is
/// the theme's, and the ellipsis's advance is the face's.
pub(crate) fn min_button_width(ui: &egui::Ui) -> f32 {
    button_padding(ui) + text_width(ui, "…", &TextStyle::Button)
}

/// The space a `ui.separator()` allocates for itself in a horizontal
/// layout, excluding the layout gaps around it.
///
/// `egui::Separator`'s default `spacing` is 6 pt in the cross direction,
/// with the 1 pt rule painted down the middle of it. It is not exposed as
/// a constant, so it is named here rather than left as a bare literal at a
/// call site.
const SEPARATOR_LINE: f32 = 6.0;

/// The full cost of putting a `ui.separator()` **between two things** in a
/// horizontal layout: its own width plus the `item_spacing` `egui` puts on
/// each side of it.
///
/// This is the band's inter-group figure — `[group][gap][rule][gap][group]`
/// — and is what [`plan::plan_band`] is handed as `separator`. It is *not*
/// the right number for a separator that is an item inside a group; see
/// [`measure_item`].
///
/// `pub(crate)` because [`super::qat`] ends with the same `ui.separator()`
/// and must charge itself the same figure for it.
pub(crate) fn separator_width(ui: &egui::Ui) -> f32 {
    SEPARATOR_LINE + ui.spacing().item_spacing.x * 2.0
}

/// Measure a string in the font `egui` will draw it in.
///
/// Uses [`egui::Color32::PLACEHOLDER`] so the galley this produces is the
/// **same cache entry** the widget will later ask for with its real
/// colour — `egui` memoizes layout jobs, and a placeholder-coloured
/// galley is the form it stores. Measuring therefore costs a hash lookup
/// rather than a second text layout.
///
/// `pub(crate)` for the reason [`button_padding`] gives: every row of the
/// ribbon that plans its own width must measure text the same way.
pub(crate) fn text_width(ui: &egui::Ui, text: &str, style: &TextStyle) -> f32 {
    if text.is_empty() {
        return 0.0;
    }
    let font_id = style.resolve(ui.style());
    ui.ctx().fonts_mut(|fonts| {
        fonts
            .layout_no_wrap(text.to_owned(), font_id, egui::Color32::PLACEHOLDER)
            .size()
            .x
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **★ A caption is never empty, whatever the manifest says.**
    ///
    /// The fallback chain — caption → id → a literal — is what makes
    /// "every rendered group emits a caption" a *total* claim rather than
    /// one that holds only for validated manifests. An unvalidated
    /// manifest is exactly the input this module has to survive, because
    /// the defect it exists to prevent is a band of unlabelled controls
    /// and an empty caption reproduces it precisely.
    ///
    /// Falling back to the id is also diagnostic: `page_display` sitting
    /// in a caption slot names the manifest entry that needs fixing,
    /// where a blank names nothing.
    #[test]
    fn a_caption_is_never_empty_even_for_an_unvalidated_group() {
        assert_eq!(caption_text(&Group::new("render", "Render")), "Render");
        assert_eq!(
            caption_text(&Group::patch("page_display")),
            "page_display",
            "a caption-less group must announce its own id, not a blank"
        );
        let blank = Group {
            id: "window".to_owned(),
            caption: Some("   ".to_owned()),
            items: None,
        };
        assert_eq!(
            caption_text(&blank),
            "window",
            "whitespace is as invisible as an empty string and must fall through"
        );
        let nameless = Group {
            id: String::new(),
            caption: None,
            items: None,
        };
        assert_eq!(caption_text(&nameless), "(unnamed group)");

        // Total claim: no input produces an empty caption.
        for g in [
            Group::new("a", "A"),
            Group::patch("b"),
            blank,
            nameless,
            Group {
                id: String::new(),
                caption: Some(String::new()),
                items: None,
            },
        ] {
            assert!(!caption_text(&g).trim().is_empty(), "{g:?}");
        }
    }

    /// The selected-condition convention is stable and cannot collide
    /// with an application's own dotted condition names.
    ///
    /// The `:` is load-bearing: with a `.` an application could
    /// accidentally define a real condition called
    /// `selected.view.single` and turn a toggle on from a distance.
    #[test]
    fn the_selected_condition_name_is_namespaced() {
        assert_eq!(selected_condition("view.single"), "selected:view.single");
        assert!(selected_condition("x").starts_with(SELECTED_CONDITION_PREFIX));
        assert!(
            !SELECTED_CONDITION_PREFIX.contains('.'),
            "a dotted prefix could collide with an application's own condition names"
        );
    }
}
