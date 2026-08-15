//! # `app::conditions` — what the shell may ask about the application
//!
//! One function. It publishes the set of **named conditions** the ribbon
//! evaluates every frame to decide whether a control is enabled and whether
//! it renders pressed.
//!
//! ## Why the vocabulary is names rather than closures
//!
//! `egui_shell` stores `Enable::When("doc.pages")` — a string — and asks this
//! set whether it is present. Data rather than a closure, because a name is
//! serializable (so an operator's customized manifest can reference it),
//! testable headlessly, and cannot capture state that would make a command's
//! availability depend on *when* it was registered.
//!
//! The cost of that choice lands here: every name is a **promise the
//! application has to keep**, and the only thing keeping the two halves in
//! step is `shell::commands`' `KNOWN` list and the test that walks it.
//!
//! ## Why this is its own file
//!
//! Split from `app/mod.rs` when that file crossed the 1,500-line gate for the
//! second time — the first split produced `app/dispatch.rs`. The seam is the
//! same shape as that one and just as real: `mod.rs` composes a frame,
//! `dispatch.rs` answers *what does this verb do*, and this file answers
//! *what is true right now*. They change for different reasons.
//!
//! ## ★ Two sources, one convention
//!
//! Conditions come from two different places and it matters that they arrive
//! the same way:
//!
//! * **application state** — `doc.open`, `doc.pages`, `selection.any`,
//!   `selection.bounds`, and the page-display and view-chrome pressed states,
//!   all read from `PdfceApp` and the open document; and
//! * **`egui::Memory`** — the armed canvas tool and the armed region zoom,
//!   which is why this function takes an `egui::Context`.
//!
//! The second source is the reason this function grew a parameter. Three
//! separate pieces of work recorded that the hand tool and the region zoom had
//! no pressed state, and each declined to invent a mechanism for it — rightly.
//! The alternative was a shadow copy of the armed tool on `PdfceApp` that the
//! canvas would have to remember to update, which puts the truth in two places
//! and fails as a ribbon that says Hand while the canvas selects: a
//! disagreement no test catches, because each half is self-consistent.

use crate::app::PdfceApp;
use crate::app::state::Status;

impl PdfceApp {
    /// The conditions the ribbon evaluates its predicates against.
    ///
    /// Rebuilt every frame because that is what it describes — the state
    /// *this* frame is drawn from. The set is **closed**, and the vocabulary
    /// is written down once in `crate::shell::commands`' `KNOWN` list rather
    /// than counted here — a count in prose drifts the moment a condition is
    /// added, and this sentence has already been wrong once for saying
    /// "five". That module has a test asserting no predicate names anything
    /// outside the list, so a typo in a manifest cannot silently produce a
    /// control that is disabled forever.
    ///
    /// # ★ `selection.any` is published from here, and only now
    ///
    /// It was deliberately absent while the selection lived in
    /// `egui::Memory`: this function has no `egui::Context`, so it could not
    /// have read the selection even if it wanted to, and publishing a
    /// condition it could not evaluate would have armed a **destructive**
    /// control that could not work — the inverse of the no-placeholders rule
    /// and the exact shape of defect D1.
    ///
    /// The selection now lives on [`state::OpenDoc`], so the answer is one
    /// field read. Two surfaces come alive with it, both of which the manifest
    /// has been carrying unpowered: the contextual **Format** tab
    /// (`visible_when: "selection.any"`, which is the appear-on-selection
    /// affordance `RIBBON_IA.md` §5.8 calls the single largest usability
    /// change) and the **Delete** inside it (`enabled_when` the same). One
    /// spelling, one source — see `shell::manifest::format::VISIBLE_WHEN`.
    ///
    /// **The Objects panel's focus is not a selection and must never satisfy
    /// this**, which is what `panels::PanelsState::focus`'s own test asserts
    /// through the enable machinery: a panel row being focused must not arm a
    /// destructive command, because the operator would have no way to tell
    /// which of two "selections" it was about to act on. This reads
    /// `doc.selection` and nothing else.
    /// `pub(super)` rather than private: this moved out of `app/mod.rs` and
    /// its three callers stayed. Deliberately NOT `pub` — nothing outside
    /// `app` may publish or read the condition set, because a second producer
    /// is how a control comes to be enabled by one rule and drawn by another.
    pub(super) fn conditions(&self, ctx: &egui::Context) -> egui_shell::commands::ConditionSet {
        let mut set = egui_shell::commands::ConditionSet::new();
        if let Status::Open(doc) = &self.status {
            set.set("doc.open");
            if !doc.pages.is_empty() {
                set.set("doc.pages");
            }
            if !doc.selection.is_empty() {
                set.set("selection.any");
            }
            // ★ **A live text selection** — the operand the three Text markup
            // commands act on, and the condition that keeps them from being
            // controls that do nothing on almost every press.
            //
            // # It is NOT a refinement of `selection.any`, and confusing them
            // would grey the controls exactly where they work
            //
            // `selection.any` is the **object** selection — page content, the
            // thing Edit's marquee builds. This is the **text** selection, and
            // the two are mutually exclusive by construction: a press means text
            // only when the mode cannot select content
            // (`canvas::textsel::takes_the_press`), so in every mode at most one
            // of these two conditions can ever be set. A predicate written as
            // `selection.any` on a text-markup command would therefore be false
            // in Review — the one mode where marking text works — and true in
            // Edit, where it cannot.
            //
            // # Why `live`, and why the same question the command asks
            //
            // A selection records the revision it was resolved against
            // (`canvas::textsel` §7), and after an edit its stored boxes may sit
            // over different glyphs. `markup::text::mark` refuses a stale one
            // rather than authoring a `/QuadPoints` annotation over
            // possibly-wrong words, so the condition asks the *same* question —
            // otherwise the control would be live at exactly the moment pressing
            // it declines, which is the disagreement `selection.bounds` was
            // added to prevent for zoom-to-selection.
            //
            // Note the visible consequence, which is deliberate: authoring a
            // text markup is itself an edit, so the selection that authored it
            // is stale on the next frame and these three controls **grey
            // themselves** immediately afterwards. That reads as the operator's
            // work being finished, and it is honest — marking the same words a
            // second way needs a second sweep.
            if doc
                .text_selection
                .as_ref()
                .is_some_and(|s| s.live(doc.edit_epoch))
            {
                set.set("selection.text");
            }
            // ★ `selection.bounds` is NOT `selection.any`, and the gap
            // between them is a real state rather than a defensive check.
            //
            // A selection is an identity — page, object, subpath, node —
            // and identities can outlive the box they described: the
            // selection may name an object on a page that is no longer
            // shown, or one whose index an edit renumbered. `selection.any`
            // is then true and there is nothing to frame.
            //
            // Zoom-to-selection is the one command where that difference is
            // visible, because framing "nothing" is not a no-op — it is a
            // jump to the origin at some arbitrary scale, which looks
            // exactly like a bug and loses the operator's place. So the
            // control greys instead, and it asks the same function the
            // grips are laid out on, so what greys and what is drawn can
            // never disagree.
            if crate::canvas::zoom::can_zoom_to_selection(doc) {
                set.set("selection.bounds");
            }
            // ★ **The page-display radio's pressed position.**
            //
            // `egui_shell::ribbon::selected_condition` is the framework's
            // convention for "this command is currently ON", and
            // `render_command` reads it to draw the button pressed. Without
            // this line View ▸ Page display is four buttons with no indication
            // of which one you are in — which for a radio is not a cosmetic
            // gap, it is the control's entire state.
            //
            // Exactly one is ever set, because `view.display` is one enum
            // value and `page_display_command` is a total function over it.
            // That is what makes it a radio rather than four toggles, and it
            // is asserted from the registry side by
            // `shell::commands::tests::every_page_display_mode_has_a_registered_command`.
            set.set(egui_shell::ribbon::selected_condition(
                crate::shell::commands::page_display_command(doc.view.display),
            ));
            // ★ **The three View ▸ Display toggles' pressed state.**
            //
            // Between zero and three of these are set, where exactly one
            // page-display condition above always is — which is the whole
            // difference between three switches and one three-position
            // control, expressed in the conditions rather than in the drawing.
            //
            // Rulers, grid and guides live on `doc.view`; the hand tool and
            // the armed region zoom live in `egui::Memory` and are published
            // below. Both routes end in the same `selected_condition`, which
            // is what kept a second mechanism from being invented for either.
            for &chrome in crate::app::actions::ViewChrome::ALL {
                if chrome.read(&doc.view) {
                    set.set(egui_shell::ribbon::selected_condition(
                        crate::shell::commands::chrome_command(chrome),
                    ));
                }
            }
            // ★ **Is there a circle fit waiting to be committed?** — the one
            // condition on this list that is about a *gesture in progress*
            // rather than about the document or the view.
            //
            // `measure.finish` is the ribbon half of the radius/diameter
            // tool's ending. That gesture has no natural end (see
            // `canvas::measure::MeasureKind::Circular`), so the operator
            // supplies one — and a Finish that were always enabled would be a
            // control that does nothing on almost every press, which P3
            // forbids and which is the placeholder shape this project refuses.
            //
            // # ★ Why it is INSIDE the `Status::Open` arm when the armed-tool
            // conditions below are deliberately outside it
            //
            // Those publish *"which tool you are in"*, which is true of the
            // application and survives closing a document — a ribbon that
            // forgot your tool the moment you closed a file would be reporting
            // something untrue about itself. This one publishes *"there is a
            // pick set on a page that is ready to become a dimension"*, and
            // the action it leads to names that page. With no document open
            // there is no page for it to name: the pick set would still be
            // sitting in `egui::Memory`, the control would be live, and
            // pressing it would raise a `CommitDimension` against a document
            // that is not there. Two different kinds of fact, so two different
            // scopes — and this is the one place in this function where that
            // distinction has had to be drawn.
            //
            // Costs one memory lookup per frame with nothing armed, which is
            // what `canvas::measure::finishable` reduces to when the tool is
            // not the circular one.
            if crate::canvas::measure::finishable(ctx) {
                set.set("measure.finishable");
            }
            // ★ …and the same fact for the two **vertex markup** tools, which
            // have the same problem and were given the same answer.
            //
            // PolyLine and Polygon are runs of clicks with no natural end, so
            // `markup.finish` is their ribbon ending — and a Finish that were
            // always enabled would be the same control-that-does-nothing P3
            // forbids. Everything the paragraph above says about scope applies
            // here unchanged and for the identical reason: this publishes *"there
            // is a run on a page that is ready to become an annotation"*, and the
            // action it leads to names that page, so with no document open there
            // is no page for it to name. Inside the `Status::Open` arm,
            // therefore, beside its twin.
            //
            // ★ It is also where the polygon/polyline difference reaches the
            // operator: `finishable` asks `markup::action`, which needs **three**
            // vertices for a polygon and two for a polyline, so after two clicks
            // this control is live for one tool and greyed for the other — the
            // rule stated where they can see it before pressing anything, rather
            // than as a refusal after.
            //
            // Costs one memory lookup per frame with nothing armed, which is what
            // `markup::vertex::finishable` reduces to when the armed tool is not
            // one of the two.
            if crate::canvas::markup::vertex::finishable(ctx) {
                set.set("markup.finishable");
            }
        }

        // ★ **The two toggles whose state lives in `egui::Memory`.**
        //
        // These were the last controls in the ribbon with no pressed state,
        // and the reason was structural rather than an oversight: this
        // function took `&self` and no `egui::Context`, so a toggle whose
        // state is in egui's own memory had no route here at all. Three
        // separate pieces of work recorded the gap and declined to invent a
        // second mechanism for it, which was right — the fix is to hand this
        // function the context, not to keep a shadow copy of the tool on
        // `PdfceApp` that the canvas would then have to remember to update.
        //
        // A shadow copy is worth naming as the road not taken, because it is
        // the obvious one: it would put the truth about which tool is armed in
        // two places, and the failure mode is a ribbon that says Hand while
        // the canvas selects — a disagreement no test would catch, because
        // each half would be self-consistent.
        //
        // **Outside the `Status::Open` arm on purpose.** The armed tool and
        // the armed zoom survive closing a document, so a ribbon that forgot
        // which tool you were in the moment you closed a file would be
        // reporting something untrue about its own state. The commands
        // themselves are gated on `doc.pages`, so they still grey out with
        // nothing open — greyed and pressed is exactly right for "this is the
        // tool you are in, and there is nothing to use it on".
        if crate::canvas::tool::selected(ctx) == crate::canvas::tool::CanvasTool::Hand {
            set.set(egui_shell::ribbon::selected_condition("view.tool_hand"));
        }
        // ★ **The text tool's pressed state**, published exactly as the hand's
        // is, from the same `egui::Memory`-backed value and outside the
        // `Status::Open` arm for the same reason.
        //
        // # This is the step that was forgotten once, and the reason it has a
        // test of its own
        //
        // Phase 7 shipped `CanvasTool::Measure`, `arm_measure`, `measure_command`
        // and a dispatch arm using its inverse — every one with a passing unit
        // test — and did **not** publish the condition here, so Measure ▸ Linear
        // armed the tool, placed a dimension the engine accepted, and the button
        // never lit up. `ui-verify` found it in a running window, because the
        // missing link was a *call site*.
        //
        // The text tool is more exposed to that failure than either family
        // before it, and the reason is worth stating: arming it changes the
        // **cursor and nothing else**. A markup tool at least draws a band the
        // moment you use it; an armed text tool that did not light its control
        // would leave an operator with no on-screen evidence of the mode they are
        // in at all — and a captured window does not carry the pointer, so not
        // even a screenshot would show it.
        //
        // `selected` rather than `active`, matching the hand: a held space bar
        // borrows the hand for as long as it is down, and a control that
        // un-pressed itself under the operator's thumb every time they panned
        // would be reporting a tool they did not choose.
        if crate::canvas::tool::selected(ctx).is_text() {
            set.set(egui_shell::ribbon::selected_condition("view.tool_text"));
        }
        if crate::canvas::zoom::region_zoom_armed(ctx) {
            set.set(egui_shell::ribbon::selected_condition("view.zoom_region"));
        }
        // ★ The armed markup tool, published the same way and outside the
        // `Status::Open` arm for the same reason as the two above.
        //
        // **At most one**, because `CanvasTool::Markup` carries the kind
        // rather than there being one variant per shape — so the four
        // controls behave as a radio without anything having to enforce it.
        // That is the payoff of the enum shape the canvas chose: a tool that
        // could be two kinds at once is unrepresentable, so a ribbon showing
        // two pressed shape buttons is unrepresentable too.
        if let Some(kind) = crate::canvas::tool::selected(ctx).markup_kind() {
            set.set(egui_shell::ribbon::selected_condition(
                crate::shell::commands::markup_command(kind),
            ));
        }
        // ★ …and the armed **measure** tool, for the identical reason.
        //
        // # This arm was missing, and `ui-verify` is what found it
        //
        // Phase 7 shipped `CanvasTool::Measure(MeasureKind)`, `arm_measure`,
        // `measure_command` — the exact twin of `markup_command` — and a
        // dispatch arm that uses its inverse. Every one of those has a passing
        // unit test. What nothing tested is that *this function* hands the
        // second to the first, because that is a property of a **call site**,
        // and a call site's effect is observable only in a running window.
        //
        // So Measure ▸ Linear armed the tool, placed a dimension the engine
        // accepted, and **the button never lit up**. That is `HANDOFF.md`
        // defect 2's shape one layer up: the thing works, and the surface the
        // operator looks at does not say so. It was found by
        // `ui_verify::checks::measure_linear`, which compares the control's fill
        // against its sibling's *in one capture* — a differential nothing that
        // happens to both controls can satisfy.
        //
        // The lesson worth keeping: adding a tool is not four changes, it is
        // five, and the fifth is the one with no unit test to remind you.
        if let Some(kind) = crate::canvas::tool::selected(ctx).measure_kind() {
            set.set(egui_shell::ribbon::selected_condition(
                crate::shell::commands::measure_command(kind),
            ));
        }

        // `undo.available` and `redo.available` are still deliberately absent:
        // there is no undo stack to report on yet. Setting them would arm
        // controls that cannot work. They arrive with their subsystem.
        set
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★ **Every armable tool kind reports a pressed state** — asserted over
    /// `ALL` for both families rather than over a list written here.
    ///
    /// # The defect this exists for, which shipped
    ///
    /// `app::conditions` published the armed **markup** kind and did not
    /// publish the armed **measure** kind. Phase 7 had `CanvasTool::Measure`,
    /// `arm_measure`, `measure_command` and a dispatch arm using its inverse —
    /// all four with passing unit tests — so Linear armed the tool, placed a
    /// dimension the engine accepted, and the button never lit up. It was found
    /// by `ui-verify` driving the real window, because the missing link was a
    /// **call site**, and no unit test observed two adjacent links being
    /// connected.
    ///
    /// Iterating `MarkupKind::ALL` and `MeasureKind::ALL` is what stops the
    /// same omission recurring: a fifth kind added to either enum with no
    /// `selected_condition` fails here rather than shipping as a control that
    /// arms without looking armed. A list of ids spelled out in this test would
    /// have to be remembered, which is the thing that was not.
    #[test]
    fn every_armable_tool_kind_reports_a_pressed_state() {
        use crate::canvas::markup::MarkupKind;
        use crate::canvas::measure::MeasureKind;
        use crate::canvas::tool::{self, CanvasTool};

        let app = PdfceApp::new();
        let ctx = egui::Context::default();

        for &kind in MarkupKind::ALL {
            let id = crate::shell::commands::markup_command(kind);
            let cond = egui_shell::ribbon::selected_condition(id);
            tool::select(&ctx, CanvasTool::Select);
            assert!(
                !app.conditions(&ctx).is_set(&cond),
                "`{id}` must not read pressed while the select tool is armed"
            );
            tool::select(&ctx, CanvasTool::Markup(kind));
            assert!(
                app.conditions(&ctx).is_set(&cond),
                "`{id}` names {kind:?}, which is armed, and the ribbon does not say so"
            );
        }

        for &kind in MeasureKind::ALL {
            let id = crate::shell::commands::measure_command(kind);
            let cond = egui_shell::ribbon::selected_condition(id);
            tool::select(&ctx, CanvasTool::Select);
            assert!(
                !app.conditions(&ctx).is_set(&cond),
                "`{id}` must not read pressed while the select tool is armed"
            );
            tool::select(&ctx, CanvasTool::Measure(kind));
            assert!(
                app.conditions(&ctx).is_set(&cond),
                "`{id}` names {kind:?}, which is armed, and the ribbon does not say so"
            );
        }

        // ★ …and the two tools that carry **no** kind, which is why they cannot
        // be reached by walking an `ALL`. They are the ones this test's own
        // mechanism would silently miss, so they are named — and naming them is
        // exactly the "list of ids that has to be remembered" this test's header
        // warns about, which is why the warning is narrowed rather than ignored:
        // walk the kinds where kinds exist, and enumerate the kindless ones,
        // because there is nothing else to enumerate them from.
        //
        // The text tool is the more exposed of the pair. Arming it changes the
        // cursor and nothing else on the canvas, so a missing pressed state
        // would leave an operator with no evidence at all of the mode they are
        // in — where a hand at least shows a grab cursor and moves the page.
        for (tool, id) in [
            (CanvasTool::Hand, "view.tool_hand"),
            (CanvasTool::Text, "view.tool_text"),
        ] {
            let cond = egui_shell::ribbon::selected_condition(id);
            tool::select(&ctx, CanvasTool::Select);
            assert!(
                !app.conditions(&ctx).is_set(&cond),
                "`{id}` must not read pressed while the select tool is armed"
            );
            tool::select(&ctx, tool);
            assert!(
                app.conditions(&ctx).is_set(&cond),
                "`{id}` names {tool:?}, which is armed, and the ribbon does not say so"
            );
            // …and arming it must not leave the OTHER kindless tool pressed,
            // which is the property that makes them behave as a radio without
            // anything enforcing it — the same payoff the kind-carrying enum
            // gives the two families above.
            let other = if tool == CanvasTool::Hand {
                "view.tool_text"
            } else {
                "view.tool_hand"
            };
            assert!(
                !app.conditions(&ctx)
                    .is_set(&egui_shell::ribbon::selected_condition(other)),
                "arming {tool:?} must not leave `{other}` pressed"
            );
        }

        // …and exactly one is pressed at a time, which is the payoff of the
        // kind-carrying enum shape: a tool that could be two kinds at once is
        // unrepresentable, so two pressed buttons are too.
        tool::select(&ctx, CanvasTool::Measure(MeasureKind::Linear));
        let set = app.conditions(&ctx);
        let pressed = MeasureKind::ALL
            .iter()
            .chain(std::iter::empty())
            .filter(|&&k| {
                set.is_set(&egui_shell::ribbon::selected_condition(
                    crate::shell::commands::measure_command(k),
                ))
            })
            .count();
        assert_eq!(pressed, 1, "exactly one measure control renders pressed");
        for &kind in MarkupKind::ALL {
            assert!(
                !set.is_set(&egui_shell::ribbon::selected_condition(
                    crate::shell::commands::markup_command(kind)
                )),
                "arming a measure tool must not leave a markup control pressed"
            );
        }
    }

    /// ★ **The three Text markup controls are live exactly when there is a
    /// live text selection**, and are asserted through the **registry's own
    /// enable machinery** rather than by reading the condition name.
    ///
    /// Reading `set.is_set("selection.text")` would assert that this function
    /// agrees with itself. What matters is whether the *control* comes alive,
    /// which is the registration's predicate and this function's publication
    /// joined — the same join `every_armable_tool_kind_reports_a_pressed_state`
    /// exists for, and the same join `ui-verify` had to find in a running window
    /// when the measure tools armed without lighting up.
    ///
    /// Three states, and the third is the one a build would plausibly get wrong:
    /// a selection made *before* an edit is not an operand, because its recorded
    /// boxes may now sit over other glyphs — and a control that is live while
    /// the press would decline is the disagreement `selection.bounds` was
    /// invented to prevent one command over.
    #[test]
    fn the_text_markup_controls_need_a_live_text_selection() {
        use crate::app::tests::opened;
        use crate::canvas::markup::text::TextMarkKind;
        use crate::canvas::textsel::TextSelection;
        use pdfce_core::annot_author::Quad;
        use pdfce_core::page_tree::Rect as PageRect;

        let ctx = egui::Context::default();
        let mut app = opened();
        let ids: Vec<&str> = TextMarkKind::ALL
            .iter()
            .map(|&k| crate::shell::commands::text_mark_command(k))
            .collect();
        let mut reg = egui_shell::CommandRegistry::new();
        crate::shell::commands::register(&mut reg);

        let live = |app: &PdfceApp, ctx: &egui::Context, id: &str| {
            reg.get(id)
                .expect("registered")
                .is_enabled(&app.conditions(ctx))
        };

        for id in &ids {
            assert!(
                !live(&app, &ctx, id),
                "`{id}` must be greyed with nothing selected — it would do nothing"
            );
        }

        let Status::Open(doc) = &mut app.status else {
            unreachable!("`opened` opens a document")
        };
        let epoch = doc.edit_epoch;
        doc.text_selection = Some(TextSelection::for_test(
            0,
            epoch,
            vec![Quad::from_rect(PageRect::from_corners(
                72.0, 700.0, 300.0, 710.0,
            ))],
        ));
        for id in &ids {
            assert!(
                live(&app, &ctx, id),
                "`{id}` acts on the text selection there now is"
            );
        }

        // One edit later the same selection is not an operand.
        let Status::Open(doc) = &mut app.status else {
            unreachable!()
        };
        doc.edit_epoch = epoch.wrapping_add(1);
        for id in &ids {
            assert!(
                !live(&app, &ctx, id),
                "`{id}` must not offer to mark boxes recorded against an older revision"
            );
        }
    }

    /// ★ **THE P3 TENSION, CLOSED** — in Edit, with the text tool armed and a
    /// live text selection, the three text-markup controls come alive and their
    /// press authors an annotation.
    ///
    /// # What was wrong, and why it was a rule violation rather than a gap
    ///
    /// Edit shows the Markup tab, so `markup.underline`, `markup.strikeout` and
    /// `markup.squiggly` were **drawn** there — and `selection.text` could never
    /// be true in Edit, because `canvas::textsel::takes_the_press` gave the press
    /// its text meaning only where the mode could *not* select content. So three
    /// controls rendered, greyed, in every Edit session for the life of the
    /// build, with no state that could ever enable them.
    ///
    /// `RIBBON_IA.md` **P3** reserves greying for *temporarily* unavailable and
    /// says an absent capability renders nothing. Permanently greyed is neither,
    /// and it could not be fixed by hiding: a command lives on exactly one tab,
    /// and the Markup tab is in **both** Review and Edit, so hiding them in Edit
    /// would have required a per-command per-mode visibility rule that this
    /// manifest does not have and that would have been a mechanism invented to
    /// conceal a gap rather than to close one.
    ///
    /// # Why this test is worth its length
    ///
    /// It is the only assertion in the workspace that joins **four** things that
    /// each have their own passing tests: the mode's capabilities, the armed
    /// tool, the condition, and the dispatch. `the_text_markup_controls_need_a_
    /// live_text_selection` above proves the condition-to-enable join and says
    /// nothing about the mode; `canvas::textsel`'s tests prove the press rule and
    /// know nothing about the ribbon. A build that armed the tool and left
    /// `press_kind` reading the mode first would pass both of those and fail
    /// here.
    ///
    /// The **negative** half is asserted first and is what makes the positive
    /// half mean something: with the tool down, the same mode with the same
    /// selection must still refuse, because that is the state the operator was
    /// in before this feature and it must not have been quietly widened. Note it
    /// is the *press rule* that is asserted there, not the condition — the
    /// condition reads only whether a selection exists and is live, and in Edit
    /// without the tool no gesture could have made one.
    #[test]
    fn in_edit_the_text_tool_makes_the_text_markup_controls_reachable() {
        use crate::app::tests::opened;
        use crate::canvas::markup::text::TextMarkKind;
        use crate::canvas::textsel::{self, TextSelection};
        use crate::canvas::tool::{self, CanvasTool};
        use pdfce_core::annot_author::Quad;
        use pdfce_core::page_tree::Rect as PageRect;

        let ctx = egui::Context::default();
        let mut app = opened();
        app.dispatch_command(&ctx, "mode.edit", &mut Vec::new());
        let caps = app.capabilities();
        assert!(
            caps.edit_content && caps.author_markup,
            "the premise: Edit both selects content and authors markup — which is exactly why \
             the two halves collided"
        );

        // The old world: no tool, no text gesture, so no operand could exist.
        tool::select(&ctx, CanvasTool::Select);
        assert!(
            !textsel::takes_the_press(tool::selected(&ctx), caps),
            "with the select tool, Edit's primary button is still the content marquee"
        );

        // Arm the tool, and the gesture that makes the operand exists.
        tool::select(&ctx, CanvasTool::Text);
        app.on_mode_capabilities_changed(&ctx);
        assert!(
            textsel::takes_the_press(tool::selected(&ctx), caps),
            "the armed text tool takes the press in Edit — that is the whole feature"
        );

        // A sweep would produce this. Planted rather than driven, because
        // `interact` needs a window; the sweep itself is asserted against a real
        // extraction in `canvas::textsel` and against the real binary in
        // `ui-verify`'s `text_tool_selects_and_marks_in_edit`.
        let Status::Open(doc) = &mut app.status else {
            unreachable!("`opened` opens a document")
        };
        let epoch = doc.edit_epoch;
        doc.text_selection = Some(TextSelection::for_test(
            0,
            epoch,
            vec![Quad::from_rect(PageRect::from_corners(
                72.0, 700.0, 300.0, 710.0,
            ))],
        ));

        let mut reg = egui_shell::CommandRegistry::new();
        crate::shell::commands::register(&mut reg);
        for &kind in TextMarkKind::ALL {
            let id = crate::shell::commands::text_mark_command(kind);
            assert!(
                reg.get(id)
                    .expect("registered")
                    .is_enabled(&app.conditions(&ctx)),
                "`{id}` is drawn on Edit's Markup tab and must now be able to enable there — \
                 this is the P3 tension the text tool exists to close"
            );
        }

        // …and pressing one really authors, rather than merely lighting up. The
        // action is the proof that Edit's `author_markup` and the text selection
        // meet: a control that enabled and then declined would be the
        // `selection.bounds` failure one command over.
        let mut actions = Vec::new();
        app.dispatch_command(
            &ctx,
            crate::shell::commands::text_mark_command(TextMarkKind::Underline),
            &mut actions,
        );
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, crate::app::actions::Action::CommitTextMarkup { .. })),
            "the press must raise CommitTextMarkup in Edit, not decline: {actions:?}"
        );
    }

    /// ★ **`measure.finishable` needs a document, not merely a pick set.**
    ///
    /// The one condition published from inside the `Status::Open` arm that is
    /// about a gesture rather than about the document, and this is the reason
    /// it is inside it. A circular pick set lives in `egui::Memory`, which
    /// **outlives documents** — that is the property the armed-tool conditions
    /// below it are published outside the arm to preserve. Here it is exactly
    /// the hazard: the action Finish raises names a page, and with the document
    /// closed there is no page for it to name. A live control that raises a
    /// `CommitDimension` against nothing is the placeholder shape this project
    /// refuses.
    ///
    /// Both directions are asserted, because the first alone would pass on a
    /// build where the condition was never published at all.
    #[test]
    fn finish_is_not_offered_with_no_document_open() {
        use crate::app::state::{FOUR_PAGES, open_fixture};
        use crate::canvas::measure::{self, MeasureKind};
        use crate::canvas::tool::{self, CanvasTool};

        let mut app = PdfceApp::new();
        let ctx = egui::Context::default();
        tool::select(&ctx, CanvasTool::Measure(MeasureKind::Circular));
        measure::circular::plant_pick_for_test(&ctx, 0);
        assert!(
            measure::finishable(&ctx),
            "the canvas really does have a finishable fit"
        );
        assert!(
            !app.conditions(&ctx).is_set("measure.finishable"),
            "…and the ribbon must still not offer to place it into no document"
        );

        // Open one, and the same fit is offered.
        app.status = Status::Open(Box::new(open_fixture(FOUR_PAGES)));
        assert!(
            app.conditions(&ctx).is_set("measure.finishable"),
            "with a document open the control is live"
        );
    }

    /// ★ **`markup.finishable` is the same fact for the vertex tools, and it is
    /// scoped the same way** — plus the one thing that is genuinely different
    /// about it.
    ///
    /// The document half is a near-copy of the test above, deliberately: the two
    /// conditions have the same shape, the same hazard and the same argument for
    /// living inside `Status::Open`, so a build that got the scope right for one
    /// and wrong for the other is what a near-copy catches.
    ///
    /// What is **not** a copy is the last section, and it is the interesting
    /// half: this condition is where the polygon/polyline difference reaches the
    /// operator. `markup::action` needs **three** vertices for a `/Polygon` and
    /// two for a `/PolyLine`, so the same two-click run leaves the ribbon's
    /// Finish live for one tool and greyed for the other. Asserting it here
    /// rather than only in `markup::vertex` is the point — the rule is worth
    /// nothing until it reaches the control.
    #[test]
    fn markup_finish_needs_a_document_and_enough_corners_for_its_kind() {
        use crate::app::state::{FOUR_PAGES, open_fixture};
        use crate::canvas::markup::{MarkupKind, vertex};
        use crate::canvas::tool::{self, CanvasTool};

        let mut app = PdfceApp::new();
        let ctx = egui::Context::default();
        tool::select(&ctx, CanvasTool::Markup(MarkupKind::Polygon));
        vertex::plant_run_for_test(&ctx, 0, MarkupKind::Polygon);
        assert!(
            vertex::finishable(&ctx),
            "the canvas really does have a finishable run"
        );
        assert!(
            !app.conditions(&ctx).is_set("markup.finishable"),
            "…and the ribbon must still not offer to place it into no document"
        );

        app.status = Status::Open(Box::new(open_fixture(FOUR_PAGES)));
        assert!(
            app.conditions(&ctx).is_set("markup.finishable"),
            "with a document open the control is live"
        );
        // …and it is not the measure tab's condition wearing another name: a
        // measure tool and a markup tool cannot both be armed, so exactly one of
        // the two may ever be set, and a build that collapsed them would light
        // one tab's Finish from the other tab's gesture.
        assert!(!app.conditions(&ctx).is_set("measure.finishable"));

        // ★ The polygon/polyline difference, at the control. Two vertices is a
        // polyline and is not a polygon.
        vertex::plant_short_run_for_test(&ctx, 0, MarkupKind::Polygon);
        assert!(
            !app.conditions(&ctx).is_set("markup.finishable"),
            "two corners are a line drawn there and back, not a polygon"
        );
        tool::select(&ctx, CanvasTool::Markup(MarkupKind::PolyLine));
        vertex::plant_short_run_for_test(&ctx, 0, MarkupKind::PolyLine);
        assert!(
            app.conditions(&ctx).is_set("markup.finishable"),
            "…and the same two corners ARE a polyline"
        );
    }
}
