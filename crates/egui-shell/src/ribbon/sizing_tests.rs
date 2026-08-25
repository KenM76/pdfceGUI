//! Rendered geometry for the three item sizes and the `visible_when` filter.
//!
//! `RIBBON_SCALING.md`, `OPERATOR_REQUESTS.md` O31.
//!
//! # Why these are here and not in [`super::sizing`]'s own test module
//!
//! Because they measure **what was drawn**, and two of them need a font. This
//! crate depends on `egui` with `default-features = false`, so a plain test
//! process has no font data and every galley measures zero — which would make
//! *"an icon-only control is narrower than a labelled one"* pass against an
//! implementation that had never dropped the label, since both would measure
//! the icon and nothing else.
//!
//! [`super::testfont`] is the answer, and it is why [`super::width_tests`]
//! exists at all. This file borrows its harness rather than duplicating it:
//! one synthetic face, installed one way, asserted to actually measure
//! something before any test relies on it.
//!
//! ★ It is a **separate file** from `width_tests` for R2's reason and no
//! other: that one is 1,433 lines against a 1,500-line limit, and a rule that
//! is obeyed by writing the new tests somewhere else is a rule that is
//! working.

use egui::Rect;

use crate::commands::{Command, CommandRegistry, ConditionSet, HandlerToken};
use crate::manifest::{Group, Item, ItemSize, Mode, Shell, Tab};

use super::width_tests::context;

/// Two commands, both fully equipped, so `Small` is **earned** and the tests
/// below measure the size rule rather than the fallback.
///
/// ★ Each carries an icon **and** a tooltip. A fixture missing either would
/// make every `Small` in this file silently render as `Medium`, and the tests
/// asserting a narrower control would fail for a reason that has nothing to do
/// with what they are about — see [`super::sizing::resolved`].
fn registry() -> CommandRegistry {
    let mut r = CommandRegistry::new();
    r.register_all([
        Command::new("a.one", "Alpha command", HandlerToken::new(1))
            .with_icon("k1")
            .with_tooltip("The first"),
        Command::new("a.two", "Beta command", HandlerToken::new(2))
            .with_icon("k2")
            .with_tooltip("The second"),
    ])
    .expect("distinct ids");
    r
}

/// A one-tab, one-group manifest holding exactly `items`.
///
/// Deliberately minimal: every test below compares two renders that differ in
/// one property, and anything else on the tab would be width the comparison
/// has to reason about.
fn shell(items: impl IntoIterator<Item = Item>) -> Shell {
    Shell::new()
        .with_mode(Mode::new("only", "Only", ["t"]))
        .with_tab(Tab::new("t", "Tab").with_groups([Group::new("g", "Group").with_items(items)]))
}

/// The group's own rect, from the reported regions.
fn group_rect(rendered: &[(String, Rect)]) -> Option<Rect> {
    rendered
        .iter()
        .find(|(name, _)| name == "ribbon.group.t.g")
        .map(|(_, r)| *r)
}

/// One item's rect.
fn item_rect(rendered: &[(String, Rect)], id: &str) -> Option<Rect> {
    let want = format!("ribbon.item.{id}");
    rendered
        .iter()
        .find(|(name, _)| *name == want)
        .map(|(_, r)| *r)
}

/// Render a manifest at a comfortable width, **with an icon painter
/// installed**, and report every rect.
///
/// ★★★ The painter is the whole reason this file has its own render function
/// instead of calling [`render_shell_with`] like its neighbours. `Small` is
/// **earned** — it needs an icon, a tooltip *and* an installed painter — and
/// the shared harness installs no painter, so every `Small` in every test here
/// would silently render as `Medium` and the assertions would fail against a
/// perfectly correct implementation.
///
/// That is not a flaw in the shared harness: a ribbon with no icon painter is
/// a working ribbon, and its tests are right not to invent one. It is a
/// property of what this file measures.
///
/// The painter draws **nothing**. It exists to be `Some`. What is being
/// measured is the space a control reserves, and a painter that filled its
/// rect would be measuring `egui`'s compositor.
fn render_with_icons(
    items: impl IntoIterator<Item = Item>,
    registry: &CommandRegistry,
    conditions: &ConditionSet,
) -> Vec<(String, Rect)> {
    let ctx = context();
    let shell = shell(items);
    let mut state = crate::ribbon::RibbonState::new();
    state.set_active_tab("t");
    let mut rects = Vec::new();
    // Two frames, because `egui` resolves some geometry a frame late and the
    // second is the honest one — the same reason every other harness in this
    // crate runs twice.
    for _ in 0..2 {
        rects.clear();
        let mut sink = |name: &str, rect: Rect| rects.push((name.to_owned(), rect));
        let mut paint = |_: &egui::Painter, _: &crate::ribbon::IconRequest<'_>| {};
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::Vec2::new(1400.0, 400.0),
            )),
            ..Default::default()
        };
        let _ = ctx.run_ui(input, |ui| {
            let _ = crate::ribbon::Ribbon::new()
                .with_conditions(conditions)
                .with_icon_painter(&mut paint)
                .reporting_rects_to(&mut sink)
                .render(ui, &shell, registry, &mut state);
        });
    }
    rects
}

/// [`render_with_icons`] against the shared fixture registry.
fn render(items: impl IntoIterator<Item = Item>, conditions: &ConditionSet) -> Vec<(String, Rect)> {
    render_with_icons(items, &registry(), conditions)
}

/// ★★★ **An icon-only control is narrower than the same control labelled.**
///
/// The whole point of `Small`, and the measurement that moved the 884-point
/// number in `RIBBON_SCALING.md` §2. Asserted as a *comparison* between two
/// renders of the same command rather than against a number, because the
/// absolute width depends on the synthetic face's metrics and a literal here
/// would be pinning the fixture rather than the rule.
#[test]
fn an_icon_only_control_is_narrower_than_a_labelled_one() {
    let none = ConditionSet::new();
    let labelled = render([Item::command("a.one")], &none);
    let icon_only = render([Item::command("a.one").sized(ItemSize::Small)], &none);

    let wide = item_rect(&labelled, "a.one").expect("the labelled control drew");
    let narrow = item_rect(&icon_only, "a.one").expect("the icon-only control drew");
    assert!(
        narrow.width() < wide.width(),
        "icon-only must be narrower: {} vs {}",
        narrow.width(),
        wide.width()
    );
    // ★ And the GROUP narrowed with it. A control that shrank inside a group
    // whose width did not would have saved nothing — the band's plan is made
    // of group widths, and that is the number the operator feels.
    let wide_group = group_rect(&labelled).expect("group drew");
    let narrow_group = group_rect(&icon_only).expect("group drew");
    assert!(
        narrow_group.width() < wide_group.width(),
        "the group must narrow too: {} vs {}",
        narrow_group.width(),
        wide_group.width()
    );
}

/// **A Large control is taller than a Medium one**, because it spans the
/// band's rows rather than sitting in one of them.
///
/// Height needs no font, so this one would pass without the synthetic face —
/// it is here because it is the same subject, and because a reader comparing
/// the three sizes wants the three assertions together.
#[test]
fn a_large_control_spans_the_rows_a_medium_one_sits_in() {
    let none = ConditionSet::new();
    let medium = render([Item::command("a.one")], &none);
    let large = render([Item::command("a.one").sized(ItemSize::Large)], &none);

    let short = item_rect(&medium, "a.one").expect("the medium control drew");
    let tall = item_rect(&large, "a.one").expect("the large control drew");
    assert!(
        tall.height() > short.height(),
        "a large control must span the rows: {} vs {}",
        tall.height(),
        short.height()
    );
}

/// ★★★ **A hidden item is not drawn, and its space is reclaimed.**
///
/// Both halves, because only the first is obvious and only the second is the
/// operator's ask. A `visible_when` applied at draw time would satisfy the
/// first and leave a hole: the group would still be measured at its full
/// width, the groups to its right would not move left, and *"shift the space
/// used depending on what exists"* would be false.
#[test]
fn a_hidden_item_is_not_drawn_and_its_space_is_reclaimed() {
    let mut on = ConditionSet::new();
    on.set("show.two");
    let off = ConditionSet::new();
    let items = || {
        [
            Item::command("a.one"),
            Item::command("a.two").shown_when("show.two"),
        ]
    };

    let both = render(items(), &on);
    let one = render(items(), &off);

    assert!(
        item_rect(&both, "a.two").is_some(),
        "the conditioned item must draw while its condition holds"
    );
    assert!(
        item_rect(&one, "a.two").is_none(),
        "and must not draw when it does not"
    );
    assert!(
        item_rect(&one, "a.one").is_some(),
        "its neighbour is unaffected"
    );

    let wide = group_rect(&both).expect("group drew");
    let narrow = group_rect(&one).expect("group drew");
    assert!(
        narrow.width() < wide.width(),
        "the group must give the hidden item's width back: {} vs {}",
        narrow.width(),
        wide.width()
    );
}

/// **A group whose every item is hidden is not drawn at all** — R9, and the
/// end of the same rule.
///
/// ★ Not "drawn empty", and not "drawn with just its caption". A caption over
/// nothing is a promise of a control that is not there, and the separator
/// beside it is a rule between two things with nothing between them.
#[test]
fn a_group_with_nothing_left_is_not_drawn() {
    let off = ConditionSet::new();
    let rendered = render(
        [
            Item::command("a.one").shown_when("never"),
            Item::command("a.two").shown_when("never"),
        ],
        &off,
    );
    assert!(
        group_rect(&rendered).is_none(),
        "an emptied group must vanish, caption and all: {:?}",
        rendered.iter().map(|(n, _)| n).collect::<Vec<_>>()
    );
}

/// A `Small` that has not earned icon-only rendering draws at `Medium` width
/// — the fallback, measured rather than asserted about the resolver.
///
/// ★ This is the guard that lets a manifest ask for `Small` freely. Without
/// it, marking a tooltip-less command `Small` would ship an unlabelled
/// rectangle, and the author would have no way to know except by looking.
#[test]
fn a_small_that_has_not_earned_it_renders_at_medium_width() {
    let mut r = CommandRegistry::new();
    // Icon, but no tooltip — so no accessible name, so no icon-only.
    r.register_all([Command::new("a.one", "Alpha command", HandlerToken::new(1)).with_icon("k1")])
        .expect("distinct ids");
    let none = ConditionSet::new();
    let asked_small = render_with_icons([Item::command("a.one").sized(ItemSize::Small)], &r, &none);
    let plain = render_with_icons([Item::command("a.one")], &r, &none);

    let a = item_rect(&asked_small, "a.one").expect("drew");
    let b = item_rect(&plain, "a.one").expect("drew");
    assert!(
        (a.width() - b.width()).abs() < 0.5,
        "an unearned Small must fall back to the labelled width: {} vs {}",
        a.width(),
        b.width()
    );
}

/// ★★★ **A Large control in the OVERFLOW MENU is still tall enough to click.**
///
/// The regression this file exists to hold, and the one thing here that was a
/// shipped defect rather than a hypothetical.
///
/// A group drawn in the menu uses `GroupBox::NATURAL`, whose row height is
/// `0.0` **on purpose** — so a one-row group in the popup has no hole beneath
/// it. The first `render_large` allocated exactly the height it was handed, so
/// a Large control in the menu got a rect of **zero height**: it painted (the
/// icon and label are placed from the rect's centre, which still exists), it
/// reported its rect as required, and it **could not be clicked**.
///
/// ★ Every unit test passed, because the band path hands a real row height and
/// only the menu path does not. `ui-verify`'s `print_dialog_reaches_the_spooler`
/// found it, at the width the harness drives, and said exactly the right
/// thing: `ribbon.item.file.print` declared at `y 148.0 .. 148.0`, *"which has
/// no usable area — the control is laid out and not on screen"*.
///
/// This drives the same path: a band too narrow for the group, a click on the
/// affordance, and an assertion about the rect the menu reported.
#[test]
fn a_large_control_in_a_popup_is_tall_enough_to_click() {
    let ctx = context();
    let registry = registry();
    // ★★ RETARGETED 2026-08-25 from the `⏷ N more` dropdown to a COLLAPSED
    // GROUP's popup, which S4 left as the only popup that renders groups. The
    // defect guarded is unchanged and is one of the sharpest in this crate: a
    // Large control handed `GroupBox::NATURAL` — whose `rows` is 0.0 — used to
    // allocate a rect of ZERO HEIGHT. It painted, it published its rect, and it
    // was not clickable, because a zero-height rect has no area to hit.
    // `ui-verify` found it in the honest way, reporting `ribbon.item.file.print`
    // at `y 148.0 .. 148.0`. Every unit test passed, because only the popup
    // path passes a zero.
    let mut shell = shell([Item::command("a.one").sized(ItemSize::Large)]);
    if let Some(g) = shell
        .tabs
        .iter_mut()
        .flatten()
        .flat_map(|t| t.groups.iter_mut().flatten())
        .next()
    {
        g.collapse = Some(1);
    }
    let mut state = crate::ribbon::RibbonState::new();
    state.set_active_tab("t");
    // Narrow enough that the only group cannot fit beside the affordance.
    let narrow = 60.0_f32;

    let render = |ctx: &egui::Context,
                  state: &mut crate::ribbon::RibbonState,
                  input: egui::RawInput,
                  rects: &mut Vec<(String, Rect)>| {
        rects.clear();
        let mut sink = |name: &str, rect: Rect| rects.push((name.to_owned(), rect));
        let mut input = input;
        input.screen_rect = Some(Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::Vec2::new(narrow, 400.0),
        ));
        let _ = ctx.run_ui(input, |ui| {
            let _ = crate::ribbon::Ribbon::new()
                .reporting_rects_to(&mut sink)
                .render(ui, &shell, &registry, state);
        });
    };

    let mut rects = Vec::new();
    render(&ctx, &mut state, egui::RawInput::default(), &mut rects);
    let affordance = rects
        .iter()
        .find(|(n, _)| n == "ribbon.group.t.g.collapsed")
        .map(|(_, r)| *r)
        .expect("the band is too narrow for the group, so it must have collapsed");
    assert!(
        item_rect(&rects, "a.one").is_none(),
        "with the popup closed the control must not be on the band, or the assertion below could be satisfied without the popup ever opening"
    );

    // Click the affordance, then let the popup render and settle.
    let at = affordance.center();
    let mut input = egui::RawInput::default();
    input.events.extend([
        egui::Event::PointerMoved(at),
        egui::Event::PointerButton {
            pos: at,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        },
        egui::Event::PointerButton {
            pos: at,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        },
    ]);
    render(&ctx, &mut state, input, &mut rects);
    let mut input = egui::RawInput::default();
    input.events.push(egui::Event::PointerMoved(at));
    render(&ctx, &mut state, input, &mut rects);

    let drawn = item_rect(&rects, "a.one")
        .expect("with the menu open the group's control must have been drawn");
    assert!(
        drawn.height() > 0.0,
        "a Large control in the overflow menu must have a clickable height, got {:?}",
        drawn
    );
}
