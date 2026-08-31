//! # `ribbon::sizing` — how much room one control asks for, and what it shows
//!
//! `RIBBON_SCALING.md`, and `OPERATOR_REQUESTS.md` O31.
//!
//! ## Why this file exists
//!
//! Until 2026-08-24 every control in the band was the same: icon, gap, label,
//! one row, always. [`crate::ribbon::band::render_command`] passed a hard-coded
//! `shows_label: true` and the comment beside it argued the case —
//! *"icon-only belongs to the QAT … in the band there are forty and the label
//! is the only thing that makes one findable"*.
//!
//! That argument is right about **findability** and was wrong about **every
//! control**, and driving Word settled it. Measured at 884 client points, on
//! the widest tab each application has:
//!
//! | | groups on the band |
//! |---|---|
//! | Word | **10** |
//! | this shell, before | **3** — four in a `⏷ 4 more` menu |
//!
//! Word gets there by mixing three sizes in one group: its Clipboard is one
//! Large button beside a column of three icon-only Small ones, its Font group
//! is two combos and thirteen Smalls, its Editing group is three Mediums. The
//! label is not what makes `B` findable; its position in a cluster of type
//! controls is.
//!
//! ## ★★★ The rule that keeps `Small` honest
//!
//! A control renders icon-only **only when it has earned it**: it names an
//! icon, it carries a **tooltip**, and a painter is actually installed. That
//! is [`crate::ribbon::qat::shows_label`]'s rule, unchanged, applied to a
//! second surface — and the reason is the same one that module gives at
//! length: the tooltip is the icon's **accessible name**. Without one, an
//! icon-only button is an unlabelled rectangle to a screen reader and a
//! guess to everybody else.
//!
//! A `Small` that has not earned it **falls back to `Medium`**. It does not
//! render a mystery, and it does not refuse to render. This is the same shape
//! as the QAT's fallback and it means a manifest can ask for `Small`
//! everywhere without an author having to audit which commands have tooltips.
//!
//! ## The layout rule for `Large`, and the one thing it changes about a group
//!
//! A Large control is **icon above label**, spanning the band's rows. It
//! therefore cannot live inside the row-wrapping that
//! [`crate::ribbon::plan::wrap_group`] does, because that partitions items
//! *into* rows and a Large item is beside them.
//!
//! So: **within a group, Large items lead.** They are drawn first, in a
//! horizontal run at the group's left, at full height; everything else wraps
//! into the rows to their right.
//!
//! ★ That is a real constraint on the manifest and it is worth stating rather
//! than discovering: a Large item written in the middle of a group is hoisted
//! to the front. It is also how every group in Word is actually built — Paste
//! leads Clipboard, the three Acrobat buttons are the whole group — so the
//! constraint costs nothing an author wanted, and the alternative is a
//! two-dimensional packing problem for a gain nobody asked for.

use egui::{Sense, TextStyle, Vec2, vec2};

use crate::commands::Command;
use crate::manifest::{Item, ItemSize};
use crate::ribbon::band::{button_padding, text_width};
use crate::ribbon::ctx::Ctx;
use crate::ribbon::ctx::IconRequest;
use crate::ribbon::plan::ItemWidths;

/// The gap between a Large control's icon and its label, in points.
///
/// Smaller than the horizontal `icon_spacing` a Medium control uses, because
/// vertically the two are already separated by the icon's own bottom edge and
/// the label's ascent — matching the horizontal figure reads as a gap rather
/// than as one control.
const LARGE_STACK_GAP: f32 = 2.0;

/// How much wider a Large control is than its widest part, in points.
///
/// A Large button's content is centred rather than left-aligned, so it needs
/// symmetric breathing room; the ordinary button padding is tuned for a row of
/// text and looks tight around a centred icon.
const LARGE_SIDE_PADDING: f32 = 10.0;

/// **Is this item drawn at all?** — `RIBBON_SCALING.md` §5.3.
///
/// The `visible_when` filter, applied **before measurement**, which is the
/// whole point: a hidden item must not merely be skipped when drawing, or the
/// group reserves space for a control that never appears and the band's plan
/// is wrong by exactly the width of every hidden item.
///
/// ★★★ This is **visibility**, not enablement, and R9 draws the line: *an
/// unavailable capability renders nothing; greying is reserved for
/// **temporarily** unavailable and is always explained on hover.*
/// [`Command::enable`] is the greying — no document open, empty undo stack.
/// This is the disappearing — the command does not apply on this surface, in
/// this mode, in this build.
///
/// An item with no condition is always visible, which is nearly all of them.
#[must_use]
pub(crate) fn visible(item: &Item, conditions: &crate::commands::ConditionSet) -> bool {
    item.visible_condition()
        .is_none_or(|name| conditions.is_set(name))
}

/// **The size this control will actually render at**, which is not always the
/// size the manifest asked for.
///
/// See the module header: `Small` is earned. `can_paint` is whether the
/// application installed an icon painter at all — a manifest asking for
/// icon-only controls in a build with no icons would otherwise draw a band of
/// empty rectangles.
///
/// ★ `Large` is **not** conditional on an icon. A large button with no icon is
/// a large label, which is odd-looking but legible and unambiguous; a large
/// button with an icon and no label would be the mystery, and `Large` always
/// draws its label.
#[must_use]
pub(crate) fn resolved(command: &Command, asked: ItemSize, can_paint: bool) -> ItemSize {
    match asked {
        ItemSize::Small if !can_paint || command.icon.is_none() || command.tooltip.is_none() => {
            ItemSize::Medium
        }
        other => other,
    }
}

/// The width one command control occupies at `size`.
///
/// ★★ This and [`render`] are one decision written twice, and they must not
/// diverge: a control measured at one width and drawn at another is how a band
/// that "claims to fit" clips its last group. `band`'s own comment makes the
/// same point about the icon slot. Every branch here has a matching branch
/// there, in the same order, and
/// [`crate::ribbon::width_tests`]'s `a_band_that_claims_to_fit_really_does_fit`
/// is what would catch a drift between them.
#[must_use]
pub(crate) fn width(ui: &egui::Ui, ctx: &Ctx<'_>, command: &Command, size: ItemSize) -> f32 {
    let icon = if command.icon.is_some() {
        ctx.theme.metrics.icon_pts
    } else {
        0.0
    };
    match size {
        ItemSize::Medium => ItemWidths {
            icon,
            text: text_width(ui, &command.label, &TextStyle::Button),
            gap: ui.spacing().icon_spacing,
            padding: button_padding(ui),
        }
        .total(),
        // Icon only. The text is measured as zero rather than omitted, so the
        // `gap` term switches itself off through `ItemWidths`' own rule rather
        // than through a second copy of it here.
        ItemSize::Small => ItemWidths {
            icon,
            text: 0.0,
            gap: ui.spacing().icon_spacing,
            padding: button_padding(ui),
        }
        .total(),
        // Stacked: the wider of the two parts decides, and neither is a gap
        // away from the other horizontally.
        ItemSize::Large => {
            let text = text_width(ui, &command.label, &TextStyle::Button);
            icon.max(text) + LARGE_SIDE_PADDING * 2.0
        }
    }
}

/// Draw one Large control — icon above label, spanning `height`.
///
/// # Why this is built by hand rather than from `egui::Button`
///
/// `egui::Atoms` lays out with `push_right`; there is no vertical form, so a
/// `Button` cannot stack an icon over a label. The alternatives were a nested
/// `Ui` inside a frame that *looks* like a button — which does not respond
/// like one, and gets the hover and pressed visuals wrong on the day the theme
/// changes — or this: allocate the rect, take a real `Response`, and paint the
/// button's own `WidgetVisuals` into it.
///
/// ★ Painting from `ui.style().interact(&response)` rather than from theme
/// colours directly is what keeps a Large control identical to every other
/// button under hover, focus, disabled and selected. A hand-drawn control that
/// picked its own colours is the shape of the defect this project's
/// `check-theme-colors` gate exists to refuse.
pub(crate) fn render_large(
    ui: &mut egui::Ui,
    ctx: &mut Ctx<'_>,
    command: &Command,
    selected: bool,
    enabled: bool,
    height: f32,
) -> egui::Response {
    let icon_size = ctx.theme.metrics.icon_pts;
    // ★★★ NEVER SHORTER THAN ITS OWN CONTENT — and this was a shipped defect,
    // caught by driving.
    //
    // `height` is the band's row area, which a Large control spans. In the
    // **overflow menu** there is no row area: a group in the menu is drawn
    // with `GroupBox::NATURAL`, whose `rows` is `0.0` deliberately, so that a
    // one-row group in the popup does not get a hole under it. A Large control
    // handed that zero allocated a rect of zero height — it painted (the icon
    // and label are placed from the rect's centre, which still exists), it
    // reported its rect as required, and it was **not clickable**, because a
    // zero-height rect has no area to hit.
    //
    // `ui-verify` caught it in the honest way: `print_dialog_reaches_the_spooler`
    // opened the overflow menu, found `ribbon.item.file.print` declared at
    // `y 148.0 .. 148.0`, and said so — *"which has no usable area — the
    // control is laid out and not on screen"*. Every unit test passed, because
    // the band path hands a real row height and only the menu path does not.
    //
    // So: span the rows when there are rows, and be as tall as the content
    // otherwise. Both are the same expression.
    let content_height = icon_size
        + LARGE_STACK_GAP
        + ui.text_style_height(&TextStyle::Button)
        + LARGE_STACK_GAP * 2.0;
    let want = vec2(
        width(ui, ctx, command, ItemSize::Large),
        height.max(content_height),
    );
    // ★★★ **ALLOCATED FROM A DISABLED SCOPE WHEN IT IS DISABLED**, since
    // 2026-08-31, and the line it replaces was wrong about the one thing it
    // claimed.
    //
    // It read `ui.allocate_exact_size(want, Sense::click())` followed by
    // `response.on_disabled_hover_text(...)`, with a comment promising *"the
    // response is neutered … and still refuses the click."*
    //
    // **It refused nothing.** `Ui::interact` passes `self.enabled` into the
    // response's `ENABLED` flag (egui 0.35 `ui.rs:928`, `context.rs:1385`),
    // and this allocated from an *enabled* `Ui` — it only painted greyed,
    // choosing `visuals.widgets.inactive` by hand fifteen lines below. So
    // `response.enabled()` was **always true**, with two consequences:
    //
    // 1. **The tooltip was dead.** `on_disabled_hover_text` opens only when
    //    `!response.enabled()`, so it never ran — here, and again at the
    //    caller in `ribbon::control`, which attaches the same explanation the
    //    same way. Every Large band command is greyed with no explanation, and
    //    R9 requires one.
    // 2. ★★★ **And the click still fired.** `ribbon::control` does
    //    `if response.clicked() { ctx.invoke(command.handler) }` with no
    //    second gate, so pressing a greyed Large control **invoked its
    //    command**. The band said no and the shell did it anyway.
    //
    // ⇒ The scope is the fix for both, because both read one flag. It wraps
    // the ALLOCATION only; the painting below still uses the outer `ui`'s
    // painter, so the greyed appearance is unchanged to the pixel and the
    // hand-picked `inactive` visuals keep working. Wrapping the painting too
    // would multiply the disabled alpha a second time and dim every greyed
    // Large control twice over.
    //
    // ★ Found by `OPERATOR_REQUESTS.md` O77's sweep for dead hover
    // explanations. The sweep was looking for silence and found a control that
    // acts.
    let (rect, response) = if enabled {
        ui.allocate_exact_size(want, Sense::click())
    } else {
        ui.scope(|ui| {
            ui.disable();
            ui.allocate_exact_size(want, Sense::click())
        })
        .inner
    };

    let visuals = if enabled {
        ui.style().interact_selectable(&response, selected)
    } else {
        ui.style().visuals.widgets.inactive
    };
    ui.painter().rect(
        rect,
        visuals.corner_radius,
        visuals.weak_bg_fill,
        visuals.bg_stroke,
        egui::StrokeKind::Inside,
    );

    // The icon occupies a square at the top, centred; the label sits beneath
    // it, centred. Both are placed from the rect rather than from a cursor, so
    // the two halves cannot drift apart when the height changes.
    let label_height = ui.text_style_height(&TextStyle::Button);
    let stack = icon_size + LARGE_STACK_GAP + label_height;
    let top = rect.top() + ((rect.height() - stack) / 2.0).max(0.0);
    if let Some(key) = command.icon.clone()
        && let Some(painter) = ctx.icons.take()
    {
        let icon_rect = egui::Rect::from_min_size(
            egui::pos2(rect.center().x - icon_size / 2.0, top),
            Vec2::splat(icon_size),
        );
        painter(
            ui.painter(),
            &IconRequest {
                key: &key,
                rect: icon_rect,
                tint: visuals.fg_stroke.color,
                enabled,
                selected,
            },
        );
        ctx.icons = Some(painter);
    }
    ui.painter().text(
        egui::pos2(rect.center().x, top + icon_size + LARGE_STACK_GAP),
        egui::Align2::CENTER_TOP,
        &command.label,
        TextStyle::Button.resolve(ui.style()),
        visuals.fg_stroke.color,
    );

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::{Command, ConditionSet, HandlerToken};
    use crate::manifest::Item;

    fn command(id: &str) -> Command {
        Command::new(id, "Label", HandlerToken::new(1))
    }

    /// ★★★ **A response allocated from an ENABLED `Ui` is enabled, however it
    /// is painted** — the assumption `render_large` made and that was false.
    ///
    /// This is a claim about **egui**, so it is asserted against egui rather
    /// than reasoned about. `render_large` painted itself greyed by choosing
    /// `visuals.widgets.inactive` by hand, and allocated its response from the
    /// ordinary `Ui` — so `response.enabled()` stayed true, its
    /// `on_disabled_hover_text` never opened, and `ribbon::control`'s
    /// `if response.clicked() { ctx.invoke(…) }` **still invoked the command**.
    /// The band said no and the shell did it anyway.
    ///
    /// The second half is the fix: allocating inside `ui.disable()`'s scope
    /// produces a response that reports itself disabled, which is what both
    /// the tooltip and the click gate read.
    ///
    /// ★ Written as a table over the two cases rather than asserting only the
    /// fixed one, because a build in which BOTH were disabled would satisfy a
    /// one-sided assertion and would grey every Large control permanently.
    #[test]
    fn only_a_disabled_scope_produces_a_disabled_response() {
        let ctx = egui::Context::default();
        let mut plain = None;
        let mut scoped = None;
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            let (_, response) =
                ui.allocate_exact_size(egui::vec2(40.0, 20.0), egui::Sense::click());
            plain = Some(response.enabled());

            let (_, response) = ui
                .scope(|ui| {
                    ui.disable();
                    ui.allocate_exact_size(egui::vec2(40.0, 20.0), egui::Sense::click())
                })
                .inner;
            scoped = Some(response.enabled());
        });
        assert_eq!(
            plain,
            Some(true),
            "painting a control greyed does not disable its response — that was the bug"
        );
        assert_eq!(
            scoped,
            Some(false),
            "…and allocating inside a disabled scope is what does, which is what              `on_disabled_hover_text` and the click gate both read"
        );
    }

    /// ★★★ `Small` is earned three ways, and failing any one of them falls
    /// back to `Medium` rather than drawing an unlabelled rectangle.
    ///
    /// Asserted as a table over all eight combinations, because the rule is a
    /// conjunction and a test of one clause at a time would pass against an
    /// implementation that had dropped a different one.
    #[test]
    fn small_is_earned_and_falls_back_when_it_is_not() {
        for icon in [false, true] {
            for tooltip in [false, true] {
                for painter in [false, true] {
                    let mut c = command("x");
                    if icon {
                        c = c.with_icon("k");
                    }
                    if tooltip {
                        c = c.with_tooltip("t");
                    }
                    let got = resolved(&c, ItemSize::Small, painter);
                    let earned = icon && tooltip && painter;
                    assert_eq!(
                        got,
                        if earned {
                            ItemSize::Small
                        } else {
                            ItemSize::Medium
                        },
                        "icon={icon} tooltip={tooltip} painter={painter}"
                    );
                }
            }
        }
    }

    /// `Medium` and `Large` are never downgraded — only `Small` is earned.
    ///
    /// ★ `Large` deliberately does not require an icon: a large button with no
    /// icon is a large label, which is legible. The mystery this rule guards
    /// against is an icon with no name, and `Large` always draws its label.
    #[test]
    fn only_small_is_ever_downgraded() {
        let bare = command("x");
        for painter in [false, true] {
            assert_eq!(resolved(&bare, ItemSize::Medium, painter), ItemSize::Medium);
            assert_eq!(resolved(&bare, ItemSize::Large, painter), ItemSize::Large);
        }
    }

    /// An item with no condition is always visible; one with a condition is
    /// visible exactly while it holds.
    #[test]
    fn visibility_follows_the_condition_and_defaults_to_shown() {
        let plain = Item::command("a");
        let gated = Item::command("b").shown_when("mode.edit");
        let mut set = ConditionSet::default();

        assert!(
            visible(&plain, &set),
            "an unconditioned item is always shown"
        );
        assert!(
            !visible(&gated, &set),
            "a condition that is not set hides it"
        );
        set.set("mode.edit");
        assert!(visible(&gated, &set));
        assert!(
            visible(&plain, &set),
            "setting an unrelated condition changes nothing"
        );
    }

    /// A separator and a custom item carry no condition and are always
    /// visible — the honest answer, since neither can state one yet.
    #[test]
    fn an_item_that_cannot_be_conditioned_is_shown() {
        let set = ConditionSet::default();
        assert!(visible(&Item::Separator, &set));
        assert!(visible(&Item::custom("swatch"), &set));
    }
}
