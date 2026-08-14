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
use super::plan::{self, BandPlan, CUSTOM_ITEM_WIDTH, ItemWidths};
use super::report;

/// Vertical gap between a group's control row and its caption.
///
/// Small on purpose: the caption must read as belonging to the row above
/// it rather than as a line of its own. The salvage source used the same
/// constant for the same reason.
const CAPTION_GAP: f32 = 2.0;

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
/// caller's — a band is as tall as its content, and clamping its height to
/// the clip rect would make a partially-scrolled ribbon lay its captions
/// out differently from an unscrolled one.
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
    if groups.is_empty() {
        return outcome;
    }

    let gutter = ctx.theme.metrics.gutter;
    let separator = separator_width(ui);
    let widths: Vec<f32> = groups.iter().map(|g| measure_group(ui, ctx, g)).collect();
    let reserve = plan::overflow_width(groups.len(), button_padding(ui), |s| {
        text_width(ui, s, &TextStyle::Button)
    });

    ui.horizontal(|ui| {
        let full = entitled_bounds(ui, entitled);
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
                    captioned_group(ui, ctx, tab_id, group, gutter, &mut outcome);
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
/// Lays one group out as Office lays one out: its controls in a row, its
/// **caption beneath them, centred on the row**. The body is a closure
/// rather than a predicate for the reason the salvage source records: to
/// put the caption *under* the controls, the controls must be emitted
/// inside a vertical container that is still open when the caption is
/// written, and a predicate returning `bool` has already returned before
/// the body runs.
///
/// See the module header for why this being the only such function is the
/// point.
fn captioned_group(
    ui: &mut egui::Ui,
    ctx: &mut Ctx<'_>,
    tab_id: &str,
    group: &Group,
    gutter: f32,
    outcome: &mut BandOutcome,
) {
    outcome.groups_rendered += 1;

    let whole = ui
        .vertical(|ui| {
            // The controls row FIRST: its measured width is what the
            // caption is then centred within, and that width only exists
            // after the row has been emitted.
            let row = ui
                .horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = gutter;
                    for item in group.items() {
                        render_item(ui, ctx, tab_id, &group.id, item);
                    }
                })
                .response
                .rect;

            ui.add_space(CAPTION_GAP);

            let caption = ui
                .allocate_ui_with_layout(
                    vec2(row.width(), 0.0),
                    Layout::top_down(Align::Center),
                    |ui| ui.label(RichText::new(caption_text(group)).weak().small()),
                )
                .inner;

            // Counted here, one line after the label that cannot be
            // reached without emitting it.
            outcome.captions_emitted += 1;
            ctx.reporter
                .report(caption.rect, || report::group_caption(tab_id, &group.id));
        })
        .response
        .rect;

    ctx.reporter
        .report(whole, || report::group(tab_id, &group.id));
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
        // Overflowed groups go through the SAME closure as visible ones.
        // A second, simpler drawing path for the menu is exactly how the
        // two caption-less groups in the salvage source happened.
        for group in groups.iter().skip(band_plan.shown) {
            captioned_group(ui, ctx, tab_id, group, gutter, outcome);
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

/// The width one group will occupy, caption included.
fn measure_group(ui: &egui::Ui, ctx: &Ctx<'_>, group: &Group) -> f32 {
    let widths: Vec<f32> = group
        .items()
        .iter()
        .map(|item| measure_item(ui, ctx, item))
        .collect();
    let caption = text_width(ui, caption_text(group), &TextStyle::Small);
    plan::group_width(&widths, ctx.theme.metrics.gutter, caption)
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
