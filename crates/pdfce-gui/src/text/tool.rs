//! # `text::tool` — every word the Tool panel says
//!
//! The strings for [`crate::panels::tool`]. Its header carries the design; this
//! file carries the copy, and the copy is doing more of the work here than in
//! any other panel, because **the Tool panel has almost no controls.** It is
//! nine-tenths sentences.
//!
//! ## ★ The three rules the whole file follows
//!
//! **1. No label is written here that the command registry already owns.**
//! The armed tool's name and every arming row's name come from
//! `CommandRegistry` through `crate::shell::menus::MenuHost::label`, and the
//! chord comes from the operator's own keymap. A second copy of a label
//! compiles, reads identically the day it is written, and drifts the first time
//! either is reworded — invisibly, because nothing renders both at once.
//! `NO_SURFACE.md` §1 records that exact failure with a colour.
//!
//! **2. Every sentence states a fact about the program, never a tip.** The
//! operator's own report about the shell this replaces: *"the nagging and red
//! flagging in the original GUI made for a lot of extra bugs in the visibility
//! when editing."* *"Drag marquees objects on this page"* is a statement.
//! *"Try dragging to select several objects!"* is a tip, and there are none
//! here.
//!
//! **3. An instruction says how the gesture ENDS.** Half the gestures in this
//! application do not end by themselves — a run of clicks does not, a text
//! caret does not — and *"click each corner"* is not a complete instruction
//! because nothing in it says when to stop. Every instruction below that
//! describes an open-ended gesture names its ending.

use crate::canvas::markup::MarkupKind;
use crate::canvas::measure::MeasureKind;
use crate::canvas::textannot::TextAnnotKind;
use crate::canvas::textedit::TextEditKind;

// ===========================================================================
// Block A — what the pointer does right now
// ===========================================================================

/// The heading over the "what the pointer does" block.
#[must_use]
pub const fn pointer_heading() -> &'static str {
    "The pointer"
}

/// What a press means in a mode that can select page content — Edit.
///
/// ★ **This sentence exists nowhere else in the application**, which is the
/// whole reason the unarmed panel is not a placeholder. The identical drag
/// means *marquee objects* here and *sweep text* in Read, decided by
/// `canvas::textsel::takes_the_press` reading the mode — and no surface has
/// ever said so. An operator who wonders why dragging behaves differently in
/// two modes has had no way to find out but to guess.
#[must_use]
pub const fn pointer_edit() -> &'static str {
    "Drag marquees objects on the page; click selects one. Hold Space to move \
     the paper."
}

/// What a press means in a mode that cannot select page content — Read and
/// Review.
#[must_use]
pub const fn pointer_reading() -> &'static str {
    "Drag selects text on the page; click puts the cursor in it. Hold Space to \
     move the paper."
}

// ===========================================================================
// Block B — the tools this mode has
// ===========================================================================

/// The heading over the list of tools.
#[must_use]
pub const fn tools_heading() -> &'static str {
    "Tools"
}

/// The sentence under that heading.
///
/// ★ It says **where the tools live**, and that is the anti-second-ribbon
/// mechanism written into the copy rather than only into the code. This panel
/// exists because an operator could not find a command; it must teach the
/// ribbon rather than replace it, and a list that never mentioned the ribbon
/// would become the place people go instead.
#[must_use]
pub const fn tools_hint() -> &'static str {
    "Each of these is also on the ribbon, on the tab named beside it."
}

/// One row's second line: where the command lives and what key reaches it.
///
/// `tab` is a ribbon tab's own label. The chord is omitted entirely when the
/// keymap binds none — never rendered as "no shortcut", which would be a row of
/// text saying nothing.
#[must_use]
pub fn row_home(tab: &str, chord: Option<&str>) -> String {
    match chord {
        Some(chord) => format!("{tab} · {chord}"),
        None => tab.to_owned(),
    }
}

/// What the Hand tool does, in the tool list.
#[must_use]
pub const fn row_hand() -> &'static str {
    "Move the paper without changing anything."
}

/// What the text-selection tool does, in the tool list.
#[must_use]
pub const fn row_select_text() -> &'static str {
    "Sweep a range of the page's words, to copy or mark them."
}

/// What Edit text does — half of the pair the operator could not find.
///
/// ★ The distinction from [`row_add_text`] is the entire point of these two
/// rows, and it is carried by the first three words of each: *change words
/// already* against *put new text*. An operator who reads only the beginnings
/// of lines — which is most operators — still gets it.
#[must_use]
pub const fn row_edit_text() -> &'static str {
    "Change words already on the page."
}

/// What Add text does — the other half.
#[must_use]
pub const fn row_add_text() -> &'static str {
    "Put new text wherever you click."
}

/// What the markup family does, in the tool list.
///
/// Names three kinds and stops. A row that listed all eight would be the
/// palette this panel refuses to become, and three is enough to say what the
/// family is for.
#[must_use]
pub const fn row_markup() -> &'static str {
    "Draw a comment on the page — rectangle, revision cloud, arrow, freehand."
}

/// What the measure family does, in the tool list.
#[must_use]
pub const fn row_measure() -> &'static str {
    "Add a dimension the drawing does not already carry."
}

// ===========================================================================
// The armed frame
// ===========================================================================

/// The heading over the armed tool's own block.
#[must_use]
pub const fn armed_heading() -> &'static str {
    "Armed"
}

/// The put-the-tool-down button.
///
/// ★ Named for what it does to the **tool**, never "Close" — the dock tab's ✕
/// closes the panel and this does not, and two controls a click apart that both
/// read as closing something is how an operator loses a surface they wanted.
#[must_use]
pub const fn put_down_button() -> &'static str {
    "Put this tool down"
}

/// The hint on that button, naming the key that does the same thing.
#[must_use]
pub const fn put_down_hint() -> &'static str {
    "Esc does the same."
}

/// What the **Node tool** does, in the Tool panel's live stage.
///
/// ★ Written as the two gestures in the order an operator performs them, and
/// naming the *thing* rather than the rung. "Anchor" is what a draughtsman
/// calls the point; `SelectionLevel::Node` is what this program calls the
/// state, and the panel speaks the first vocabulary — `text::commands`' rule
/// that a label is the operator's word and an id is the format's.
#[must_use]
pub const fn node_instruction() -> &'static str {
    "Click a shape to show its points. Click a point to select it, then drag to move it."
}

/// The line under it: how to take more than one.
#[must_use]
pub const fn node_shift() -> &'static str {
    "Shift-click to add more points. A point on a curve also shows its handles."
}

/// The Hand tool's instruction.
#[must_use]
pub const fn hand_instruction() -> &'static str {
    "Drag to move the paper. Nothing on the page changes."
}

/// The Hand tool's second line — the borrow every other tool can do.
#[must_use]
pub const fn hand_borrow() -> &'static str {
    "Holding Space borrows this from any other tool, so you rarely need to arm it."
}

/// The text-sweep tool's instruction.
#[must_use]
pub const fn text_select_instruction() -> &'static str {
    "Drag across the words you want. Click once to put the cursor in a word."
}

/// ★ What arming the sweep takes away, in the mode where it takes something.
///
/// Rendered only in a mode that can select page content. Everywhere else the
/// select tool already swept text, so there is nothing to disclose and the line
/// is **absent** rather than reworded — R9's rule applied to a sentence.
#[must_use]
pub const fn text_select_takes_the_press() -> &'static str {
    "While this is armed a drag selects text instead of marqueeing objects."
}

/// One markup kind's instruction. The gesture, and how it ends.
///
/// ★ These came from `MarkupKind`'s own variant doc comments, where they had
/// been written — correctly, and in the operator's words — since the day each
/// kind landed, with no surface able to render them. Moving them here is what
/// `check-ui-strings.sh` requires and is also what makes them reachable.
#[must_use]
pub const fn markup_instruction(kind: MarkupKind) -> &'static str {
    match kind {
        MarkupKind::Rectangle => "Drag from one corner to the other.",
        MarkupKind::Ellipse => "Drag out the box it fits inside.",
        MarkupKind::Arrow => "Drag from the tail to the head.",
        MarkupKind::PolyLine | MarkupKind::Polygon | MarkupKind::Cloud => {
            "Click each corner in turn, and double-click the last one."
        }
        MarkupKind::Ink => "Press and draw. Let go when you are done.",
        MarkupKind::Highlight => "Drag across what you want marked.",
    }
}

/// How many corners are down, and what ends the run.
///
/// ★ **The number is why this panel exists rather than a canvas readout.** A
/// rubber band and a snap indicator are the cursor and are welcome; a *number*
/// floated near the pointer would be pdfce putting a surface over the drawing
/// on its own initiative, which `MODES_AND_PANELS.md` sets to **never**. So the
/// count has exactly one legal home, and it is a real need: a polygon and a
/// revision cloud both refuse at two corners, and an operator who double-clicks
/// one click early gets silence.
#[must_use]
pub fn vertices_placed(n: usize) -> String {
    match n {
        0 => "No corners placed yet.".to_owned(),
        1 => "1 corner placed. Double-click the last one to finish.".to_owned(),
        _ => format!("{n} corners placed. Double-click the last one to finish."),
    }
}

/// One text-annotation kind's instruction.
#[must_use]
pub const fn text_annot_instruction(kind: TextAnnotKind) -> &'static str {
    match kind {
        TextAnnotKind::TextBox => "Drag out the box, then type into it.",
        TextAnnotKind::Sticky => "Click where the note should sit, then type into it.",
        TextAnnotKind::Stamp => "Drag out the area the stamp should cover.",
    }
}

/// ★ What a release does for a text-bearing annotation, which is NOT what it
/// does for a shape.
///
/// The distinction `CanvasTool` was split for: *"A markup band authors on
/// release, from geometry alone. These cannot: releasing produces an empty box,
/// and an empty box is not an annotation."* An operator who does not know that
/// reads a release-that-authors-nothing as a broken tool — which is the same
/// failure shape as the text-editing complaint that produced this panel.
#[must_use]
pub const fn text_annot_release() -> &'static str {
    "Nothing is added to the page until you accept what you have typed."
}

/// Edit-text's instruction, before there is a caret.
#[must_use]
pub const fn text_edit_instruction(kind: TextEditKind) -> &'static str {
    match kind {
        TextEditKind::Edit => "Click a word already on the page to put the cursor in it.",
        TextEditKind::Add => "Click an empty spot to start typing new text there.",
    }
}

/// Edit-text's instruction while a caret is live.
#[must_use]
pub const fn text_edit_live() -> &'static str {
    "Enter commits what you have typed. Esc abandons it."
}

/// ★★ The heading over the refusal, when a click was declined.
///
/// The refusal sentences themselves are `crate::text::textedit::refusal`'s and
/// are **not** duplicated here. They were written well, are tested, and have
/// never had a surface wide enough to show them: their own module records that
/// they were aimed at the status bar, and that *"it shares the status row with
/// everything else and R128 forbids that row growing."* A dock panel's width is
/// the dock's, decided before the body draws, so that constraint does not apply
/// here at all.
///
/// This is very likely the actual cause of *"no text editing or adding text on
/// the canvas"*: on a dense CAD sheet the first click lands where the operator
/// wants text rather than where text is, the tool declines with an explanation
/// nobody could read, and they conclude the feature does not exist.
#[must_use]
pub const fn refusal_heading() -> &'static str {
    "That click was declined"
}

/// One measure kind's instruction, before any pick.
#[must_use]
pub const fn measure_instruction(kind: MeasureKind) -> &'static str {
    match kind {
        MeasureKind::Linear => {
            "Click the first point, then the second, then where the \
                                dimension line should sit."
        }
        MeasureKind::Circular => "Click three or more points around the arc, then finish it.",
        MeasureKind::TwoLine => "Click one line, then the other.",
        // ★ The calibration pick, which is armed from inside the Set-scale
        // window rather than from the Measure tab — it is deliberately absent
        // from `MeasureKind::ALL` for that reason.
        //
        // It gets its own sentence rather than borrowing Linear's, even though
        // it reuses `LinearPick` verbatim, because the two picks mean opposite
        // things: Linear AUTHORS a ce dimension onto the page and this one
        // authors nothing at all — it measures a length the operator is about
        // to tell pdfce the real-world value of. An operator who read
        // "then where the dimension line should sit" would wait for a third
        // click that never comes.
        MeasureKind::Scale => "Click each end of something whose real length you know.",
    }
}

/// The label over the group the next dimension will join.
///
/// ★ **Read-only here, and the button beside it is a route rather than a
/// picker.** A second group picker would be two copies of the one control that
/// decides where every ce dimension goes, which is precisely the duplication
/// this project has already been bitten by. The panel that owns it is one click
/// away and is the only place it can be changed.
#[must_use]
pub const fn draw_into_label() -> &'static str {
    "Drawing into"
}

/// The button that opens the panel which owns the group picker.
#[must_use]
pub const fn manage_groups_button() -> &'static str {
    "Groups…"
}

// ===========================================================================
// The text pen — what NEW page text is written in
// ===========================================================================

/// The heading over the Add-text options.
///
/// ★ Says **new text**, not "text", and the distinction is the whole reason
/// these controls are in the Tool panel rather than on the Format tab: they
/// decide what the *next* thing typed looks like, not what a run already on the
/// page looks like. An operator who reads "Text" here and expects it to restyle
/// the word they clicked has been misled by one word.
#[must_use]
pub const fn text_pen_heading() -> &'static str {
    "New text"
}

/// The font combo's label.
#[must_use]
pub const fn text_pen_font_label() -> &'static str {
    "Font"
}

/// One bundled face's name, as an operator would say it.
///
/// ★ *"Helvetica Bold"*, not `HelveticaBold` — the engine's identifier is a
/// Rust variant and this is a font menu. The four Courier faces say
/// *"Courier Oblique"* rather than *"Courier Italic"*, because oblique is what
/// the Standard-14 set actually contains and a menu that renamed it would
/// promise a true italic pdfce cannot write.
#[must_use]
pub const fn text_pen_font_name(face: pdfce_core::fontdata::Std14) -> &'static str {
    use pdfce_core::fontdata::Std14 as F;
    match face {
        F::Helvetica => "Helvetica",
        F::HelveticaBold => "Helvetica Bold",
        F::HelveticaOblique => "Helvetica Oblique",
        F::HelveticaBoldOblique => "Helvetica Bold Oblique",
        F::TimesRoman => "Times Roman",
        F::TimesBold => "Times Bold",
        F::TimesItalic => "Times Italic",
        F::TimesBoldItalic => "Times Bold Italic",
        F::Courier => "Courier",
        F::CourierBold => "Courier Bold",
        F::CourierOblique => "Courier Oblique",
        F::CourierBoldOblique => "Courier Bold Oblique",
        F::Symbol => "Symbol",
        F::ZapfDingbats => "Zapf Dingbats",
        // ★ NO wildcard, and its absence is deliberate. `Std14` is not
        // `#[non_exhaustive]` — checked, rather than assumed from its
        // neighbours in that module, several of which are — so this match is
        // exhaustive by the compiler's own count and a fifteenth face would be
        // a build error here rather than a combo entry reading "Another
        // bundled face". That is the stronger arrangement and it is available,
        // so it is taken.
    }
}

/// The size control's label.
#[must_use]
pub const fn text_pen_size_label() -> &'static str {
    "Size"
}

/// The suffix on the size control.
#[must_use]
pub const fn text_pen_size_suffix() -> &'static str {
    " pt"
}

/// The colour swatch's label.
#[must_use]
pub const fn text_pen_colour_label() -> &'static str {
    "Colour"
}

/// ★ The sentence under the three controls.
///
/// It says what they DO NOT do, because that is the thing an operator will
/// otherwise assume: these set the next run's appearance and change nothing
/// already on the page. `Edit text` beside them replaces the words in a run and
/// keeps its existing face — pdfce cannot restyle a placed run at all yet, and
/// a control group that stayed silent about that would be read as offering it.
#[must_use]
pub const fn text_pen_note() -> &'static str {
    "These apply to the next text you add. They do not change text already on \
     the page — pdfce cannot restyle a run it did not write."
}

// ===========================================================================
// Block C — what pdfce last inferred
// ===========================================================================

/// The heading over the disclosures block.
///
/// ★ Rendered only when there is something under it. R9: an unavailable
/// capability renders **nothing**, and a heading over an empty region is the
/// placeholder that rule exists to forbid.
#[must_use]
pub const fn disclosures_heading() -> &'static str {
    "What pdfce worked out"
}

// ===========================================================================
// The empty case
// ===========================================================================

/// What the panel says with no document open.
///
/// The one sentence in this file that is a placeholder, and it is the panel
/// host's own — `Panel::show` answers the no-document case once for every
/// panel rather than eleven times. It is here only so the Tool panel's own
/// no-document state is not a second, differently-worded sentence.
#[must_use]
pub const fn no_document() -> &'static str {
    "Open a document to use the drawing and measuring tools."
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every markup kind has an instruction, and every instruction says how the
    /// gesture ends.
    ///
    /// ★ The second half is the assertion worth having. *"Click each corner"*
    /// is not a complete instruction — nothing in it says when to stop — and
    /// the failure it produces is an operator clicking forever, which is
    /// exactly what the two endings exist to prevent. Asserted as a property
    /// (the sentence names a release, a double-click or a stop) rather than
    /// against the literals, which would pass just as well if every kind
    /// returned the same string.
    #[test]
    fn every_markup_instruction_says_how_the_gesture_ends() {
        for kind in MarkupKind::ALL.iter().copied() {
            let s = markup_instruction(kind);
            assert!(!s.is_empty(), "{kind:?} has no instruction");
            let ends = s.contains("double-click")
                || s.contains("Let go")
                || s.contains("Drag from")
                || s.contains("Drag out")
                || s.contains("Drag across");
            assert!(
                ends,
                "{kind:?}'s instruction {s:?} never says how the gesture ends, so an \
                 operator following it has no way to know when to stop"
            );
        }
    }

    /// The two text tools' rows are distinguishable from their first words.
    ///
    /// The pair the operator confuses, and the row that fixes it. Asserted on
    /// the *prefix* because that is what gets read: two sentences differing
    /// only in their last clause would look identical in a narrow column.
    #[test]
    fn the_two_text_rows_differ_where_a_reader_looks() {
        let edit = row_edit_text();
        let add = row_add_text();
        assert_ne!(edit, add);
        let (a, b) = (&edit[..8], &add[..8]);
        assert_ne!(
            a, b,
            "the two rows begin with the same words, so in a narrow column they read as \
             one repeated line"
        );
    }

    /// The corner count reads as English at one and at many.
    #[test]
    fn the_corner_count_reads_as_english() {
        assert!(vertices_placed(1).starts_with("1 corner placed"));
        assert!(vertices_placed(3).starts_with("3 corners placed"));
        assert!(!vertices_placed(0).contains('0'));
    }

    /// A row with no chord renders the tab alone, not "no shortcut".
    #[test]
    fn a_row_with_no_chord_says_nothing_about_chords() {
        assert_eq!(row_home("Edit", None), "Edit");
        assert_eq!(row_home("Edit", Some("Ctrl+E")), "Edit · Ctrl+E");
    }
}
