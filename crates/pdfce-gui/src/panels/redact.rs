//! # `panels::redact` — marking content for removal, and reviewing what is
//! marked
//!
//! The body of `edit.redact`, and the **reversible** half of the redaction
//! feature. Its irreversible twin is [`crate::dialogs::redact`], and the split
//! between the two surfaces is the operator-facing distinction
//! `crate::text::commands::edit_redact` already ships in four words:
//!
//! > **Marking is reversible; applying is not.**
//!
//! Everything in this panel can be taken back — by the Remove button on a row,
//! by `Ctrl+Z`, or by simply not applying. Nothing here removes a byte from
//! anything.
//!
//! ## ★ Why a panel and not a dialog, when apply is a dialog
//!
//! [`crate::dialogs`]' header draws the line: *"a dialog is a single
//! transaction with a start and an end… a panel is somewhere an operator dips
//! in and out of while working."*
//!
//! Marking is unmistakably the second. An operator searches for a name, pages
//! through the document checking what the search caught, marks a title block by
//! hand, takes one mark off again, and only then applies. That is a working
//! surface, not a transaction — and the two commands split along exactly that
//! seam, which is why `edit.redact` opens a panel and `edit.redact_apply` opens
//! a dialog.
//!
//! It holds no document state of its own: the marks are `/Redact` annotations
//! **in the document**, so the list below is rebuilt from
//! `pdfce_core::redact::redaction_marks` every frame and there is nothing to
//! keep in step. What [`RedactUi`] holds is the operator's half-typed search —
//! their state, not the document's.
//!
//! ## ★ The census is read from the SESSION GRAPH, never the base document
//!
//! `redaction_marks(&doc.session.graph())`, and the distinction is not
//! academic: it is the shape of a real defect in the shell this one replaces,
//! where a status-bar census read `session.document()` and therefore reported
//! **zero marks** for every mark the operator had just made and not yet saved.
//! A redaction census that silently omits this session's marks is the worst
//! possible reading of the worst possible counter.
//!
//! `crate::redact::prepare_redaction_apply` reads the same walk for the same
//! reason, so the number this panel shows and the number the apply acts on
//! cannot disagree.
//!
//! ## ★ Layout: state, then action, then detail — and it was measured, twice
//!
//! Deliberately **not** the usual detail-then-action order. The count and the
//! *Review & apply* control come **first**, above the marking controls and
//! above the mark list.
//!
//! The old shell reached the same conclusion from one direction: at a realistic
//! panel height the conventional order pushed *"Review & apply"* below the
//! fold, so an operator with eleven marks could not see the control that acts
//! on them without scrolling past all eleven.
//!
//! **This build's first cut put the apply control second rather than last, and
//! that was still not enough** — a finding from driving the binary, which is
//! `HANDOFF.md` §2's founding rule paying for itself again. With the marking
//! controls above it (a heading, a button, a field, a second button, a
//! two-position switch and a four-line hint), `tools/ui-verify`'s redaction
//! check reported the control declared at `y = 801.7` inside a panel whose body
//! ended at `y = 770.0`: **off the bottom of its own pane**, on a 1100×800
//! window, with one mark made. Every unit test passed, because a unit test
//! cannot see where a control landed.
//!
//! So the rule is stronger than *"above the list"*: the census and the apply
//! control are **the first things in the panel**, and everything that can grow
//! — the hint text, the mark rows — is below them. A surface whose primary verb
//! can be pushed out of view by its own explanatory copy is one an operator
//! will conclude is broken.
//!
//! ## What this panel does NOT have, and why each is deliberate
//!
//! | absent | why |
//! |---|---|
//! | **Canvas drag-to-mark** | Shipped panel-only. See this module's *"Marking by drag"* section below — it is the salvage row's "change needed", and it is a canvas-tool build rather than a panel addition. |
//! | **A confirmation on Remove** | Removing a mark is reversible twice over (the mark can be re-made, and `Ctrl+Z` restores it) and nothing has been removed from the document. A confirmation on a reversible action is how operators learn to dismiss confirmations — including the one that matters, three controls away. |
//! | **A confirmation on Mark whole page** | Same argument, and its tooltip says so in words. |
//! | **An Apply button that applies** | The control opens a **report**. The click that opens it must not feel like the click that commits, which is why its label ends in an ellipsis and the commit lives behind two checkboxes in another surface. |
//!
//! ## ★ Marking by drag: shipped panel-only, and the reasoning
//!
//! `SALVAGE.md`'s row for `redact_apply.rs` names *"Canvas drag-to-mark
//! (currently panel-driven only)"* as the change needed. It is **not** in this
//! landing, and the brief's own instruction is the one being followed: *"if the
//! canvas gesture is more than a modest addition, ship the panel-driven version
//! and say so."*
//!
//! Three reasons it is more than modest here:
//!
//! 1. **The shipped tooltip does not promise it.**
//!    `crate::text::commands::edit_redact` enumerates what marking offers — *"a
//!    whole page, every occurrence of some text, or everything matching a
//!    pattern"* — and none of the three is a drag. All three are built.
//! 2. **It is a canvas-tool build, not a panel one.** It would need a
//!    `CanvasTool` variant, an `app::modes::capability` entry so Read cannot
//!    reach it, a rung on `canvas::keys`' Escape ladder, an overlay preview in
//!    `canvas::overlay`, and an `Action` carrying page-space quads —
//!    `HANDOFF.md` §8's warning that a tool substrate is bigger than its row
//!    implies, applied to a substrate that would be arming the one irreversible
//!    verb in the program.
//! 3. **The proof is what was blocking, and the proof is now here.** A correct,
//!    verified, panel-driven redaction is the thing `FEATURES.md`'s row was
//!    waiting on. A half-built canvas gesture on top of it would add a way to
//!    make marks, not a way to trust them.
//!
//! What it would take, so the next hand does not re-derive it: the whole-page
//! marking path below already builds a `RedactSpec` from a `Rect` and pushes it
//! through the one action arm. A canvas gesture is that same arm with a
//! different rectangle — `canvas::markup::band` is the drag machine and
//! `canvas::mapping` is the screen-to-page conversion — plus the five pieces of
//! tool substrate in item 2.

use pdfce_core::object::ObjId;

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::panels::PanelsState;
use crate::text::redact as t;

/// Named region: the whole panel body.
///
/// Matched **literally** by `tools/ui-verify/src/checks/redaction.rs`, so
/// renaming one of these silently un-aims the check that drives it. The same
/// contract `crate::dialogs::ocr`'s region names carry.
const REGION_PANEL: &str = "redact-panel"; // ui-text-exempt: trace region name, never displayed

/// Named region: the search field.
const REGION_QUERY: &str = "redact-query"; // ui-text-exempt: trace region name, never displayed

/// Named region: the control that searches and marks.
const REGION_SEARCH: &str = "redact-search"; // ui-text-exempt: trace region name, never displayed

/// Named region: the control that marks the whole page.
const REGION_WHOLE_PAGE: &str = "redact-whole-page"; // ui-text-exempt: trace region name, never displayed

/// Named region: the control that opens the apply report.
///
/// Declared **only while it is enabled**, which is itself an assertion a
/// harness wants: its absence from the trace is evidence that nothing is
/// marked, rather than that a click missed.
const REGION_APPLY: &str = "redact-apply"; // ui-text-exempt: trace region name, never displayed

/// The command the apply control invokes.
///
/// Raised as [`Action::Command`] rather than opening the dialog here, on
/// `crate::app::mod`'s stated rule for the Find bar's OCR offer: a surface
/// outside the ribbon that means an existing *command* routes through the one
/// dispatch choke point, so the command's guards live in one place. A panel
/// that called `DialogsState::open_redact` itself would be a second
/// implementation of `edit.redact_apply`, and the two would drift the first
/// time the command grew a precondition.
const APPLY_COMMAND: &str = "edit.redact_apply"; // ui-text-exempt: a command id, never displayed

/// The operator's own state in this panel.
///
/// Not document state and not derived from anything — a half-typed search
/// query and which way the mode switch is set — but it has to outlive a frame.
/// It lives on [`PanelsState`] for the reason that struct's header gives about
/// the Pages panel's: `&mut PanelsState` is already threaded to every body, so
/// a panel's own state needs no interior mutability and the forgetting is free.
///
/// **The query is deliberately cleared with the document** (through
/// `PanelsState::forget_document`, which resets this struct whole). A search
/// term left over from a previous file is one an operator could run against a
/// document it was never meant for, and on this feature that authors marks over
/// whatever it happens to hit.
#[derive(Default)]
pub struct RedactUi {
    /// What the operator has typed into the search field.
    pub(super) query: String,
    /// Whether the search field is read as a pattern rather than as literal
    /// text.
    ///
    /// One field and one box, rather than two search fields: the two modes
    /// answer the same question — *what should be marked?* — and offering two
    /// boxes would invite an operator to fill both and wonder which won.
    pub(super) pattern: bool,
}

/// Draw the panel.
///
/// The standard body signature: `&OpenDoc` is **shared**, so the
/// actions-not-mutations invariant is a compile-time fact here rather than a
/// convention, and every verb below is an [`Action`] pushed for the apply phase.
pub fn body(ui: &mut egui::Ui, doc: &OpenDoc, state: &mut PanelsState, actions: &mut Vec<Action>) {
    crate::diag::ui_rect(REGION_PANEL, ui.max_rect());
    ui.label(t::panel_intro());
    ui.add_space(6.0);
    ui.separator();

    // The one walk both this panel and `crate::redact::prepare_redaction_apply`
    // read — see the module header on why it must be the session graph.
    let marks = pdfce_core::redact::redaction_marks(&doc.session.graph());
    let page_count = doc.pages.len();

    // ★ **State, then action, then detail** — and the order is measured rather
    // than preferred. See the module header's layout section.
    census_and_apply(ui, &marks, actions);
    ui.add_space(8.0);
    ui.separator();
    marking_controls(ui, doc, state, actions, page_count);
    ui.add_space(8.0);
    ui.separator();
    mark_rows(ui, &marks, actions);

    crate::diag::trace_changed("redact-panel", || {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed.
            //
            // The field a harness reads to know how many marks exist without
            // counting pixels. `pages=` beside `marks=` because two marks on
            // one page and two marks on two pages are different situations and
            // the list alone cannot be parsed for the difference.
            "redact-panel marks={} pages={} epoch={}",
            marks.len(),
            {
                let mut pages: Vec<usize> = marks.iter().map(|m| m.page_index).collect();
                pages.sort_unstable();
                pages.dedup();
                pages.len()
            },
            doc.edit_epoch,
        )
    });
}

/// The three ways to make a mark.
///
/// Split out because [`body`] would otherwise be one long function whose two
/// halves — *make marks* and *review marks* — change for entirely different
/// reasons, which is the same seam `app/actions.rs` was split along.
fn marking_controls(
    ui: &mut egui::Ui,
    doc: &OpenDoc,
    state: &mut PanelsState,
    actions: &mut Vec<Action>,
    page_count: usize,
) {
    ui.add_space(6.0);
    ui.label(t::mark_heading());
    ui.add_space(4.0);

    // ---- whole page ---------------------------------------------------
    //
    // Enabled on there being a page at all. The command that opens this panel
    // is gated on `doc.pages`, so a document with none cannot normally get
    // here — and the guard is applied anyway, because a saved dock layout can
    // put this panel on screen with anything open.
    let whole = ui
        .add_enabled(page_count > 0, egui::Button::new(t::mark_whole_page()))
        .on_hover_text(t::mark_whole_page_tooltip());
    crate::diag::ui_rect(REGION_WHOLE_PAGE, whole.rect);
    if whole.clicked() {
        actions.push(Action::MarkPageForRedaction {
            page: doc.view.page_index,
        });
    }

    ui.add_space(6.0);

    // ---- find and mark --------------------------------------------------
    let redact_ui = state.redact_mut();
    ui.horizontal(|ui| {
        ui.label(t::search_label());
        let field = ui.text_edit_singleline(&mut redact_ui.query);
        crate::diag::ui_rect(REGION_QUERY, field.rect);
    });
    let query = redact_ui.query.trim().to_owned();
    let pattern = redact_ui.pattern;

    ui.horizontal(|ui| {
        let search = ui
            .add_enabled(
                !query.is_empty() && page_count > 0,
                egui::Button::new(t::search_button()),
            )
            .on_hover_text(t::search_button_tooltip(!query.is_empty()));
        crate::diag::ui_rect(REGION_SEARCH, search.rect);
        if search.clicked() {
            actions.push(Action::MarkRedactionsBySearch { query, pattern });
        }
    });

    // ---- the mode switch -------------------------------------------------
    //
    // Read after the button so that a click and a mode change in the same
    // frame use the mode that was on screen when the operator clicked. The
    // other order would mark by pattern a query the operator typed as literal
    // text, on the frame they changed their mind — a marking difference, on the
    // one feature where over-marking is the safe direction and under-marking is
    // not.
    let redact_ui = state.redact_mut();
    ui.horizontal(|ui| {
        ui.label(t::match_mode_label());
        ui.selectable_value(&mut redact_ui.pattern, false, t::match_literal())
            .on_hover_text(t::match_literal_tooltip());
        ui.selectable_value(&mut redact_ui.pattern, true, t::match_pattern())
            .on_hover_text(t::match_pattern_tooltip());
    });
    ui.label(
        egui::RichText::new(t::search_hint(redact_ui.pattern))
            .small()
            .weak(),
    );
}

/// The census line and the control that opens the apply report.
///
/// Above the list, deliberately — see the module header's layout note.
fn census_and_apply(
    ui: &mut egui::Ui,
    marks: &[pdfce_core::redact::RedactionMark],
    actions: &mut Vec<Action>,
) {
    ui.add_space(6.0);
    let theme = egui_shell::theme::Theme::of(ui.ctx());
    // ★ Coloured only when there is something to warn about. A census that is
    // permanently in the warning role is one an operator stops seeing, and
    // "nothing is marked" is a reassuring answer rather than a warning.
    //
    // The role is the theme's, never a literal — `check-theme-colors.sh`
    // enforces that from the other side, and `DEFECTS.md` D11 records that
    // `.strong()` is unusable in this theme, which is why the emphasis is a
    // colour role rather than a weight.
    if marks.is_empty() {
        ui.label(t::marks_count(0));
    } else {
        ui.label(egui::RichText::new(t::marks_count(marks.len())).color(theme.palette.danger));
    }
    ui.add_space(6.0);

    let apply = ui
        .add_enabled(!marks.is_empty(), egui::Button::new(t::review_and_apply()))
        .on_hover_text(t::review_and_apply_tooltip(!marks.is_empty()));
    if !marks.is_empty() {
        crate::diag::ui_rect(REGION_APPLY, apply.rect);
    }
    if apply.clicked() {
        actions.push(Action::Command(APPLY_COMMAND.to_owned()));
    }
}

/// One row per mark: where it is, and a way to take it off.
///
/// Two controls and no third. The row itself navigates — a plain button rather
/// than a `selectable_label`, because a row is a **navigation command** and not
/// a selection, and a highlighted row would imply a selected-mark concept this
/// panel deliberately does not have.
fn mark_rows(
    ui: &mut egui::Ui,
    marks: &[pdfce_core::redact::RedactionMark],
    actions: &mut Vec<Action>,
) {
    // No nested scroll area: `Panel::show` already put one around this body,
    // and a second would give the operator two bars to choose between and a
    // list that scrolls the wrong one.
    for mark in marks {
        ui.horizontal(|ui| {
            let size = mark.rect.map(|[llx, lly, urx, ury]| (urx - llx, ury - lly));
            if ui
                .button(t::mark_row(mark.page_index + 1, size))
                .on_hover_text(t::mark_row_tooltip())
                .clicked()
            {
                actions.push(Action::GoToPage(mark.page_index));
            }
            if ui
                .button(t::mark_remove())
                .on_hover_text(t::mark_remove_tooltip())
                .clicked()
            {
                actions.push(Action::RemoveRedactionMark {
                    annot_id: mark.annot_id,
                });
            }
        });
    }
}

/// **Build the specification for a whole-page mark.**
///
/// Pure, and separate from the action arm for the reason every geometry rule in
/// this crate is: it is the part that could be wrong in a way an operator would
/// notice, and a `&mut EditSession` is not available to a test that only wants
/// to ask what rectangle was chosen.
///
/// # ★ The crop box, not the media box
///
/// `Page::crop_box` is what a reader **displays** (ISO 32000-1 Table 30:
/// content is clipped to it at display time), and it defaults to the media box
/// when the document does not state one — so this is the larger of the two
/// answers in every case where they differ *for what the operator can see*, and
/// identical otherwise.
///
/// Marking the media box instead would cover area the operator has never been
/// shown, which sounds harmless and is not: the whole-page control's tooltip
/// promises to mark *"this entire page"*, and a mark extending past what the
/// page displays is a claim about content nobody reviewed. Where the two
/// genuinely differ — a trimmed drawing sheet, an imposed signature — content
/// outside the crop box is content the operator did not know was there, and
/// telling them it is covered by a mark they made deliberately would be the
/// same false-confidence failure the whole feature exists to prevent. That is a
/// **sanitise** verb, and `crate::shell::manifest`'s own note keeps the two
/// apart: *"strip metadata, scripts and hidden content. Distinct from
/// redaction."*
///
/// `fill`, `overlay_text` and `quadding` are left at their neutral values
/// because this build has no surface to choose them from, and inventing a
/// default overlay caption would put words on the operator's page that they did
/// not write.
#[must_use]
pub fn whole_page_spec(page: &pdfce_core::page_tree::Page) -> pdfce_core::annot_author::RedactSpec {
    pdfce_core::annot_author::RedactSpec {
        quads: vec![pdfce_core::annot_author::Quad::from_rect(page.crop_box)],
        fill: None,
        overlay_text: None,
        quadding: pdfce_core::vartext::Quadding::Left,
    }
}

/// The ids of marks currently in `session`, for a caller that needs to know
/// what a marking verb added.
///
/// One walk, exposed so `crate::app::actions` can report *how many* marks a
/// search created without the panel and the action arm deriving the census two
/// different ways.
#[must_use]
pub fn mark_ids(session: &pdfce_core::edit::EditSession) -> Vec<ObjId> {
    pdfce_core::redact::redaction_marks(&session.graph())
        .into_iter()
        .map(|m| m.annot_id)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::{FOUR_PAGES, open_fixture};

    /// ★ **A whole-page mark covers what the page displays.**
    ///
    /// The crop box rather than the media box, argued at
    /// [`whole_page_spec`]. The failure this catches is silent in both
    /// directions: a media-box mark covers content the operator was never
    /// shown, and a hand-shrunk rectangle would leave a margin of live text
    /// under a mark labelled "whole page".
    #[test]
    fn a_whole_page_mark_covers_the_displayed_page() {
        let doc = open_fixture(FOUR_PAGES);
        let page = &doc.pages[0];
        let spec = whole_page_spec(page);
        assert_eq!(spec.quads.len(), 1, "one region, not one per corner");
        let quad = &spec.quads[0];
        assert!(
            (quad.ll.0 - page.crop_box.llx).abs() < f64::EPSILON
                && (quad.ur.0 - page.crop_box.urx).abs() < f64::EPSILON
                && (quad.ll.1 - page.crop_box.lly).abs() < f64::EPSILON
                && (quad.ur.1 - page.crop_box.ury).abs() < f64::EPSILON,
            "the mark must be the crop box exactly: {quad:?} vs {:?}",
            page.crop_box
        );
        assert!(
            spec.overlay_text.is_none(),
            "no surface chooses overlay text, and inventing one would put words \
             on the operator's page that they did not write"
        );
    }

    /// **Marking really does add a mark the census can see.**
    ///
    /// The end-to-end shape in the smallest form a headless test can hold, and
    /// the reason it is worth having: the census reads
    /// `session.graph()` while the mark is authored into the session overlay,
    /// and a build that read `session.document()` anywhere in that chain would
    /// report zero for every mark the operator had just made.
    #[test]
    fn a_mark_authored_into_the_session_is_visible_to_the_census() {
        let mut doc = open_fixture(FOUR_PAGES);
        assert!(mark_ids(&doc.session).is_empty());
        let spec = whole_page_spec(&doc.pages[0]);
        let session = std::sync::Arc::get_mut(&mut doc.session)
            .expect("nothing else holds the session in a test");
        session
            .add_redaction(0, &spec)
            .expect("marking a page of the fixture must be expressible");

        let ids = mark_ids(&doc.session);
        assert_eq!(
            ids.len(),
            1,
            "the census read the BASE revision rather than the session graph, \
             so every unsaved mark is invisible to it — which is the defect the \
             module header records from the shell this one replaces"
        );
        assert_eq!(
            pdfce_core::redact::count_redaction_marks(&doc.session.graph()),
            1,
            "the list and the count must be the same walk"
        );
    }

    /// The panel's own state is forgotten with the document.
    ///
    /// A search term left over from a previous file is one an operator could
    /// run against a document it was never meant for — and this feature answers
    /// a search by authoring marks over whatever it hits.
    #[test]
    fn the_search_query_does_not_survive_a_new_document() {
        let mut state = PanelsState::default();
        state.redact_mut().query = "MARGARETHALE".to_owned();
        state.redact_mut().pattern = true;
        state.forget_document();
        assert!(state.redact_mut().query.is_empty());
        assert!(
            !state.redact_mut().pattern,
            "the mode goes with the query: a pattern left armed would read the \
             next document's search term as a wildcard"
        );
    }
}
