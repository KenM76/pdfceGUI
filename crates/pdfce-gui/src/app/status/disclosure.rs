//! # `app::status::disclosure` — the three rule-4 lines in the status bar
//!
//! Split out of [`super`] on 2026-08-26 when a third line took that file past
//! the 1,500-line ceiling (R2). The seam is a real one and was already drawn in
//! the parent's own prose, which distinguished *narration* from *disclosure*:
//!
//! > The left half carries four things, and only the first is the narrator. The
//! > others look similar and are governed by different rules.
//!
//! Everything here is **rule 4**: pdfce did something the operator did not ask
//! for and cannot see, so it says so — off-canvas, never on the page.
//!
//! ## ★★★ Three lines, and they are INDEPENDENT
//!
//! | line | answers |
//! |---|---|
//! | [`fill_disclosure`] | what a form fill had to **infer** — an auto-size chosen, characters that could not be encoded |
//! | [`edit_disclosure`] | what a move or delete had to change about an object's **form** to express the request |
//! | [`recovered_disclosure`] | how this **file** was assembled, before anything was drawn |
//!
//! The obvious mistake, adding a third beside two, is an `else if` chain that
//! shows whichever fires first. A document opened from a damaged index, then
//! edited, with a form filled, owes the operator all three —
//! `disclosure_independence` in the parent asserts they cannot collide.
//!
//! ★★ The third is the odd one out and the reason for this module's header: the
//! first two are about **something the operator just did**, and the last is
//! about **what the file was before they touched it**. It is also the only one
//! that persists for the life of the document rather than until the next edit.

use egui::{Align, Layout, Vec2};

use super::{
    NOTES_WIDTH_FRACTION, REGION_EDIT_DISCLOSURE, REGION_FILL_DISCLOSURE, REGION_RECOVERED,
    ROW_HEIGHT_PTS,
};
use crate::app::state::OpenDoc;
use crate::text::forms as t_forms;
use crate::text::status as t;

/// What the last fill **inferred**, in the bar, until the document moves on.
///
/// # Why this is not behind the disclosure triangle beside it
///
/// The render notes are *narration* — a census of what a raster contained —
/// and `DEFECTS.md` §5's complaint was their prominence: the first thing an
/// operator read was the application talking about itself. Demoting them was
/// right.
///
/// These two sentences are the opposite kind of thing. They are the surviving
/// half of rule 4: **an inference the operator cannot see still owes a
/// report.** `applied_autosize` means pdfce chose a point size the document
/// asked it to choose; `unencodable_chars` means the operator's own typing is
/// not what the page now says. Neither is re-derivable from the saved file
/// afterwards — both look exactly like the author's decision — so a
/// disclosure the operator has to *open something* to find is a disclosure
/// that did not happen.
///
/// # Why the status bar rather than the Forms panel alone
///
/// The panel shows them, and that was sufficient while the panel was the only
/// way to fill. Canvas filling landed 2026-08-14 and broke the assumption: a
/// fill can now happen in **Read mode with the panel closed**, and Read's
/// dock does not mount Forms unless the operator put it there. The bar is the
/// one surface present in every mode.
///
/// # It retires itself
///
/// Keyed on [`OpenDoc::edit_epoch`] **after** the fill, so any later edit —
/// including an undo — moves the document past it and the sentence
/// disappears with no code remembering to clear it. That is deliberate:
/// state that must be cleared is state that will one day be shown against
/// the wrong document.
///
/// Elided at the same fraction as the notes line, whole text on hover, and
/// **it does not make the bar taller** — R128, exactly as for its neighbour.
fn fill_disclosure(ui: &mut egui::Ui, doc: &OpenDoc) {
    let Some(d) = crate::panels::forms::edit::last_fill_disclosure(doc.edit_epoch) else {
        return;
    };

    // Both can be true of one fill. Joined rather than shown as two lines,
    // because two lines is two rows, which is the R128 loop.
    let mut line = String::new();
    if let Some(size) = d.applied_autosize {
        line.push_str(&t_forms::forms_fill_autosize_note(&d.field, size));
    }
    if d.unencodable_chars > 0 {
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(&t_forms::forms_fill_unencodable_note(
            &d.field,
            d.unencodable_chars,
        ));
    }
    if line.is_empty() {
        return;
    }

    disclosure_line(ui, REGION_FILL_DISCLOSURE, &line);
}

/// What the last **vector edit** disclosed, in the bar, until the document
/// moves on.
///
/// # What it says, and who wrote it
///
/// Every vector verb — the three move verbs and Delete — returns a list of
/// operator-facing sentences alongside its success, non-empty when the surgery
/// had to change an operator's *form* to express the request: an `re` rectangle
/// rewritten as four lines so one corner could move independently, an
/// implicitly-started subpath's `m` materialised, a curve dropped along with
/// the point it ran into. **The drawing is unchanged and the bytes are not
/// recoverable by reversing the gesture** — dragging the corner back does not
/// restore the rectangle form — which is precisely the condition rule 4 exists
/// for: pdfce inferred a representation, and the operator would otherwise
/// learn it from a diff.
///
/// The sentences are `pdfce-core`'s own and are passed through verbatim; this
/// module contributes the framing, and only the framing. See
/// [`crate::text::status::edit_disclosure_line`].
///
/// # Why it is here rather than only in the trace
///
/// It *was* only in the trace. `crate::app::actions::vector_edit`'s header
/// named that as the outstanding half in as many words — *"a disclosure that
/// only ever reaches `PDFCE_DIAG` has been recorded and not disclosed"* — and
/// this function is the half it was waiting for. The trace is unchanged and
/// still carries the full list; what has changed is that an operator who is
/// not running with `PDFCE_DIAG` set can now read it, which is every operator.
///
/// # Why the status bar rather than a panel or the canvas
///
/// Two constraints, and together they leave one surface. Rule 4 puts a
/// disclosure **off-canvas** — the one-line test is whether a screenshot of the
/// editing canvas would differ from a screenshot of the same document saved and
/// reopened, and a note drawn over the page would make it differ. And the
/// gesture that raises one is a **canvas drag**, available in Edit and Review
/// with any panel arrangement including none, so a panel could not be relied on
/// to be mounted. The bar is the one surface present in every mode.
///
/// # It retires itself, and it cannot collide with its neighbour
///
/// Keyed on [`OpenDoc::edit_epoch`] **after** the edit, exactly as
/// [`fill_disclosure`] is: any later edit — including an undo — moves the
/// document past it and the sentence disappears with no code remembering to
/// clear it. One edit bumps the epoch once and records at most one kind of
/// disclosure, so the fill line and this one can never both be live for the
/// same revision; see
/// [`crate::app::actions::last_edit_disclosure`]'s ★ section.
///
/// **It does not make the bar taller** — R128, asserted by
/// [`tests::the_bar_is_exactly_as_tall_open_as_closed`].
fn edit_disclosure(ui: &mut egui::Ui, doc: &OpenDoc) {
    let Some(d) = crate::app::actions::last_edit_disclosure(doc.edit_epoch) else {
        return;
    };
    disclosure_line(
        ui,
        REGION_EDIT_DISCLOSURE,
        &t::edit_disclosure_line(&d.notes),
    );
}

/// Draw one disclosure sentence into the bar's single row, and publish its
/// rect.
///
/// The shared body of [`fill_disclosure`] and [`edit_disclosure`], written once
/// for the reason `crate::app::actions::vector_edit` is written once: the
/// R128 defence here is not one rule but four small ones that only work
/// together — a **bounded** sub-region so a long sentence cannot push the
/// navigation controls off the right of the bar, a **fixed** row height,
/// `truncate()` rather than wrapping (wrapping is how a one-row bar becomes a
/// two-row bar, which is the feedback loop with extra steps), and the full text
/// on **hover** so eliding defers rather than loses. Two hand-written copies
/// would be two chances to omit one of the four, and the omission would show up
/// as a page that re-fits itself at the moment an operator finishes a gesture.
///
/// `region` is published so `ui-verify` can assert the sentence is on screen
/// and legible rather than merely constructed — which, for a disclosure, is the
/// whole of the requirement.
pub(super) fn disclosure_line(ui: &mut egui::Ui, region: &str, line: &str) {
    let width = (ui.available_width() * NOTES_WIDTH_FRACTION).max(0.0);
    let rect = ui
        .allocate_ui_with_layout(
            Vec2::new(width, ROW_HEIGHT_PTS),
            Layout::left_to_right(Align::Center),
            |ui| {
                ui.add(egui::Label::new(egui::RichText::new(line).small()).truncate())
                    .on_hover_text(line.to_owned());
            },
        )
        .response
        .rect;
    crate::diag::ui_rect(region, rect);
}

/// **This file's index was damaged and pdfce rebuilt it** — the only line here
/// that is about the FILE rather than about something the operator just did.
///
/// # ★★★ Why it is in the status bar as well as in Properties
///
/// Operator ruling, 2026-08-26: *"disclose it."*
///
/// Properties already carries the detail — how many objects were recovered, how
/// many were defined more than once, how many needed repairing. But **a
/// disclosure the operator has to go looking for is half a disclosure**, and
/// this is the one fact that changes how much they should trust what is on
/// screen. A rebuilt index is a *best reading of damaged bytes*: where an object
/// was defined twice pdfce had to pick one, and on a drawing a wrong pick is a
/// line in the wrong place on a page that renders perfectly.
///
/// # ★★ How it avoids being the nagging the old shell was criticised for
///
/// 1. **Off-canvas.** A line in the status bar, never a badge on the page. The
///    document is not in doubt as *drawn*; what is in doubt is how it was
///    *assembled*, and marking the page would be a second rendering path for
///    content that is fine — decision 059's whole subject.
/// 2. **It only appears for a file that was actually rebuilt**, which is rare. A
///    healthy document shows nothing; verified by opening one.
/// 3. **It states the fact and stops.** No icon, no colour alarm, no modal at
///    open. One sentence, and the operator decides whether it matters to the job
///    in front of them.
///
/// ★ The counters stay in Properties. The status bar answers *"is there
/// something I should know?"*; the panel answers *"what exactly?"* — and a line
/// long enough to carry three numbers would push the zoom and page controls off
/// a narrow window.
fn recovered_disclosure(ui: &mut egui::Ui, doc: &OpenDoc) {
    if doc.session.document().recovery().is_none() {
        return;
    }
    disclosure_line(ui, REGION_RECOVERED, t::recovered_status_line());
}

/// Draw all three, in the order the parent expects.
pub(super) fn all(ui: &mut egui::Ui, doc: &OpenDoc) {
    fill_disclosure(ui, doc);
    edit_disclosure(ui, doc);
    recovered_disclosure(ui, doc);
}
