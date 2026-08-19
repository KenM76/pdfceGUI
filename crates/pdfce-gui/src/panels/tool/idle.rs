//! # `panels::tool::idle` — the state the application opens in, and the one
//! this panel was built for
//!
//! Blocks A and B of [`super`]. Both render whether or not a tool is armed —
//! A always, B only when none is — and between them they are the whole answer
//! to the operator's *"no side bar area showing what tool is active"*.
//!
//! ## ★ Block A is not a placeholder, and the test is that it says something
//! true and unavailable elsewhere
//!
//! *"Select tool active"* would be a placeholder: true, useless, and true 90 %
//! of the time. What Block A says instead is **what a press means in this
//! mode**, and that varies in a way nothing in this application has ever
//! disclosed.
//!
//! `canvas::textsel::takes_the_press` gives the select tool the text sweep *in
//! a mode that cannot select page content*. So the identical drag sweeps text
//! in Read and Review, and marquees objects in Edit. An operator who noticed
//! that has had no way to find out why except by guessing, and an operator who
//! has not noticed it is about to be surprised.
//!
//! ## ★★ Block B: one row per FAMILY, and two deliberate exceptions
//!
//! Six ribbon controls sit behind the Markup row and four behind Measure —
//! naming all of them would be the palette [`super`]'s header forbids.
//!
//! **The two text tools are listed separately anyway**, and the exception is
//! the entire point of the block:
//!
//! - they are **the reported defect**. `edit.text` and `edit.add_text` work,
//!   are on the ribbon, have chords, and the operator reported them as missing;
//! - they are a **confusable pair**, and one family row would hide exactly the
//!   distinction he needs — *change words already on the page* against *put new
//!   text wherever you click*;
//! - a family row would have to be named something like *"Text"*, which is
//!   already the name of a third, different tool (the sweep).
//!
//! ## Every row is a route, never an implementation
//!
//! `Action::Command(id)`, whose own documentation sanctions exactly this: *"it
//! exists so a second route to an existing command cannot become a second
//! implementation of it."* No panel-side arming, no second dispatch path, and
//! every guard, gate and refusal the ribbon control has applies unchanged —
//! including the mode gate, which is why a row for a tool this mode cannot use
//! is **absent** rather than greyed.
//!
//! ## ★ Absent rather than greyed, and how the panel knows
//!
//! R9: an unavailable capability renders **nothing**. A row is drawn only when
//! its command is **registered in this build** (`MenuHost::label` answers
//! `None` otherwise — `SHELL_FRAMEWORK.md` §5b, a compiled-out capability loses
//! its command) **and** its tab is one this mode is shown. The second half is
//! `crate::app::modes::Capabilities`, and it is what removes the two text rows
//! from Review and the whole authoring half from Read — with no capability flag
//! of this panel's own, which is R8 doing the work.

use egui::Ui;

use crate::app::actions::Action;
use crate::app::modes::Capabilities;
use crate::shell::menus::MenuHost;
use crate::text::tool as t;

/// **Block A — what the pointer does right now, in this mode.**
///
/// Two lines and no controls. It is the only block that renders in every state
/// of this panel, which is why it is first: the row an operator's eye lands on
/// must not move when they arm something.
pub(super) fn pointer(ui: &mut Ui, ctx: &egui::Context) {
    ui.label(t::pointer_heading());
    crate::diag::ui_rect(super::REGION_POINTER, ui.min_rect());
    // ★ Read from the same source the CANVAS reads, not from a second copy.
    //
    // `Capabilities::for_mode` is what `canvas::textsel::takes_the_press`
    // consults, so a panel that derived "can this mode select content" any
    // other way would eventually describe a canvas that does something else.
    // The failure would be silent and would look like the panel lying.
    let caps = crate::canvas::tool::capabilities(ctx);
    let sentence = if caps.edit_content {
        t::pointer_edit()
    } else {
        t::pointer_reading()
    };
    ui.label(egui::RichText::new(sentence).small().weak());
    ui.add_space(4.0);
}

/// One row of Block B: a command id, the sentence saying what it is for, and
/// the ribbon tab it lives on.
///
/// A struct rather than a tuple because the three fields are three different
/// kinds of thing and a tuple of three `&str`s is exactly the shape that gets
/// its arguments swapped — the tab and the sentence would both compile in
/// either position and the row would read as nonsense in a way no test sees.
struct Row {
    /// The command this row arms. **Not** a `CanvasTool`: the row is a route to
    /// a registered command, and R8 says that is the only way this panel may
    /// learn the tool exists.
    id: &'static str,
    /// What the tool is for, in the operator's terms.
    what: &'static str,
    /// The ribbon tab the command lives on, so the row teaches the ribbon
    /// rather than replacing it.
    tab: &'static str,
}

/// **Block B — the tools this mode has.**
///
/// Capped at seven rows, and the cap is written down rather than merely
/// observed: exceeding it means somebody has rebuilt the ribbon in a dock
/// column. Today's maximum is six, in Edit.
pub(super) fn tools(
    ui: &mut Ui,
    ctx: &egui::Context,
    host: Option<&MenuHost<'_>>,
    actions: &mut Vec<Action>,
) {
    let caps = crate::canvas::tool::capabilities(ctx);
    ui.label(t::tools_heading());
    crate::diag::ui_rect(super::REGION_TOOLS, ui.min_rect());
    ui.label(egui::RichText::new(t::tools_hint()).small().weak());
    ui.add_space(4.0);

    for row in rows(caps) {
        draw_row(ui, host, actions, &row);
    }
}

/// The rows this mode offers, in the order an operator meets them.
///
/// # ★ The order is navigation-first, authoring-second, and it is deliberate
///
/// Hand and Select-text change nothing about the document and are available in
/// every mode; the four below them all write. An operator scanning the list
/// meets the harmless tools first, which is the same ordering argument
/// `crate::dialogs::settings` makes about its own groups and the same one that
/// puts Redact last in Edit's dock stack.
///
/// **Edit text and Add text are third and fourth rather than last**, above
/// Markup and Measure, and that placement is the whole reason this list exists:
/// they are the two the operator reported as missing, and a row he has to
/// scroll to is a row he does not read.
fn rows(caps: Capabilities) -> Vec<Row> {
    let mut rows = vec![
        Row {
            // ui-text-exempt: command id, never displayed
            id: "view.tool_hand",
            what: t::row_hand(),
            tab: crate::text::ribbon::tab_view(),
        },
        Row {
            // ui-text-exempt: command id, never displayed
            id: "view.tool_text",
            what: t::row_select_text(),
            tab: crate::text::ribbon::tab_view(),
        },
    ];
    // ★ The two text tools, and the ONE capability that gates them.
    //
    // `edit_content` is `Capabilities::for_mode`'s answer for "is this mode
    // shown the Edit tab", which is exactly the condition
    // `canvas::tool::retire_forbidden` uses to disarm a `TextEdit` when the
    // mode changes. Reading the same flag is what stops the panel offering a
    // row that arms a tool the canvas immediately puts down again.
    if caps.edit_content {
        rows.push(Row {
            // ui-text-exempt: command id, never displayed
            id: "edit.text",
            what: t::row_edit_text(),
            tab: crate::text::ribbon::tab_edit(),
        });
        rows.push(Row {
            // ui-text-exempt: command id, never displayed
            id: "edit.add_text",
            what: t::row_add_text(),
            tab: crate::text::ribbon::tab_edit(),
        });
    }
    if caps.author_markup {
        rows.push(Row {
            // ★ The family row arms `markup.rectangle`, which is the kind an
            // operator reaches for most and the one every drawing package
            // defaults to. It deliberately does NOT remember the last kind
            // used: a row whose effect varies with history is a row you cannot
            // aim at, which is the same objection this panel's header makes to
            // a recently-used list.
            // ui-text-exempt: command id, never displayed
            id: "markup.rectangle",
            what: t::row_markup(),
            tab: crate::text::ribbon::tab_markup(),
        });
    }
    if caps.author_measure {
        rows.push(Row {
            // ui-text-exempt: command id, never displayed
            id: "measure.linear",
            what: t::row_measure(),
            tab: crate::text::ribbon::tab_measure(),
        });
    }
    debug_assert!(
        rows.len() <= MAX_ROWS,
        // ui-text-exempt: a debug-assertion message, read from a stack trace by
        // whoever added the eighth row. Never rendered.
        "the tool list has grown past its cap — see this module's header"
    );
    rows
}

/// The hard cap on Block B, from [`super`]'s header.
///
/// Seven. Not a layout constraint — a **design** one: past seven rows this
/// stops being a list of what an operator can do and becomes a palette, which
/// is the failure mode the whole panel is written against.
const MAX_ROWS: usize = 7;

/// Draw one row, or nothing at all.
///
/// # ★ `None` from `label` renders NOTHING, and that is R8 rather than
/// defensiveness
///
/// `MenuHost::label` answers `None` for a command this build does not register.
/// `SHELL_FRAMEWORK.md` §5b: a capability compiled out loses its command, and a
/// surface naming an unregistered command must drop the item rather than
/// invent a name for it. So a build without, say, the markup family shows six
/// rows minus one and says nothing about the absence — which is exactly what
/// the ribbon does with the same command.
///
/// The same `None` covers a build with no validated manifest, where `host` is
/// `None` and there is no ribbon either. A panel offering routes to commands
/// the ribbon cannot show would be the only way to reach them, which is worse
/// than none.
fn draw_row(ui: &mut Ui, host: Option<&MenuHost<'_>>, actions: &mut Vec<Action>, row: &Row) {
    let Some(host) = host else {
        return;
    };
    let Some(label) = host.label(row.id) else {
        return;
    };
    let chord = host.chord(row.id);

    let response = ui.add(egui::Button::new(label).wrap());
    crate::diag::ui_rect(
        // ui-text-exempt: trace region name, never displayed
        &format!("{}{}", super::REGION_ROW_PREFIX, row.id),
        response.rect,
    );
    if response.clicked() {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("tool-panel-armed id={}", row.id)
        });
        actions.push(Action::Command(row.id.to_owned()));
    }
    // ★ The sentence and the home go UNDER the button rather than in its
    // tooltip. A tooltip requires already hovering the thing you cannot find,
    // which is precisely the operator's situation: the command's own tooltip
    // has carried the Edit-text / Add-text distinction all along and he never
    // saw it.
    ui.label(egui::RichText::new(row.what).small().weak());
    ui.label(
        egui::RichText::new(t::row_home(row.tab, chord.as_deref()))
            .small()
            .weak(),
    );
    ui.add_space(6.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★ The list never exceeds its cap, in any mode.
    ///
    /// The cap is the anti-palette rule mechanised. `rows` has a
    /// `debug_assert!` for the same property; this is the version that runs in
    /// a release test build and names the mode that broke it.
    #[test]
    fn no_mode_offers_more_than_seven_rows() {
        for caps in [
            Capabilities::NONE,
            Capabilities::FULL,
            Capabilities {
                edit_content: false,
                author_markup: true,
                author_measure: true,
            },
        ] {
            let n = rows(caps).len();
            assert!(
                n <= MAX_ROWS,
                "{caps:?} offers {n} rows, past the cap of {MAX_ROWS}. Past seven this stops \
                 being a list of what an operator can do and becomes a palette."
            );
        }
    }

    /// ★★ **Every mode that can edit page content offers both text rows.**
    ///
    /// The assertion this panel exists to make pass. The operator reported
    /// `edit.text` and `edit.add_text` as missing while both were registered,
    /// drawn and chord-bound, so the property that matters is not *"the
    /// commands exist"* — it is *"a surface an operator sees without asking
    /// names them"*.
    #[test]
    fn a_mode_that_can_edit_text_says_so_without_being_asked() {
        let ids: Vec<&str> = rows(Capabilities::FULL).iter().map(|r| r.id).collect();
        assert!(ids.contains(&"edit.text"), "{ids:?}");
        assert!(ids.contains(&"edit.add_text"), "{ids:?}");
        // And they are near the top, not scrolled past. A row an operator has
        // to scroll to is a row they do not read, which is the failure this
        // whole panel is a fix for.
        let first_text = ids
            .iter()
            .position(|id| *id == "edit.text")
            .expect("present");
        assert!(
            first_text < 4,
            "the text tools are row {first_text} of {}, which is far enough down to be \
             scrolled past in a short dock",
            ids.len()
        );
    }

    /// A mode with no authoring capability offers only the two harmless tools.
    ///
    /// Read's list. Both change nothing about the document — the hand moves the
    /// paper and the sweep writes to the clipboard, which is the operator's own
    /// *copying is not authoring* ruling — so both are legitimate in a mode
    /// whose stated point is that it authors nothing.
    #[test]
    fn a_reading_mode_offers_nothing_that_writes() {
        let ids: Vec<&str> = rows(Capabilities::NONE).iter().map(|r| r.id).collect();
        assert_eq!(ids, ["view.tool_hand", "view.tool_text"]);
    }

    /// Every row names a distinct command.
    ///
    /// Two rows arming the same command would be two buttons that do the same
    /// thing with different sentences under them — which reads as one of them
    /// being broken.
    #[test]
    fn no_two_rows_arm_the_same_command() {
        let mut seen = std::collections::BTreeSet::new();
        for row in rows(Capabilities::FULL) {
            assert!(seen.insert(row.id), "{} is listed twice", row.id);
        }
    }
}
