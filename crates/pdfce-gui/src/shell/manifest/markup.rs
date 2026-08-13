//! The **Markup** tab — *what am I adding for someone else to read?*
//!
//! `RIBBON_IA.md` §5.5. Five groups: Shapes, Text markup, Notes, Style,
//! Comments.
//!
//! # Why it is not called Review
//!
//! What lives here is markup *authoring* — shapes, notes, stamps.
//! "Review" promises a review *workflow*: compare revisions, resolve
//! comments, track changes. pdfce does not have that yet, and when it does
//! it will want the name. `Markup` is also the term this project's
//! audience uses; Bluebeam and every drafting office call it that.
//!
//! Note that the *mode* called Review is a different thing and the
//! collision is deliberate rather than accidental: Review mode is the
//! stance in which a reviewer works, and this tab is one of the five it
//! contains.
//!
//! # The Style group sets the style of the *next* markup
//!
//! Not of the selected one. Changing an existing markup's style happens on
//! the contextual Format tab, and `RIBBON_IA.md` §5.5 is explicit that
//! both surfaces must exist — *"today only the first does, which is why a
//! placed markup feels final."*
//!
//! Colour is the only style property with a control today, and it is not a
//! button: it is a swatch that opens a colour picker. That is what
//! `egui_shell::manifest::Item::Custom` is for. The shell reserves the
//! space and hands the `kind` back; it draws nothing and interprets
//! nothing. Modelling the swatch as a `Command` would have meant either
//! lying about what the control is or growing the framework a
//! `ColourSwatch` item variant, which is the road by which a reusable
//! shell stops being reusable.
//!
//! Line width, fill and opacity are **N** and join the swatch when they
//! exist.
//!
//! # Six of ten markup kinds are missing, and one of them matters most
//!
//! Ink, Polygon, PolyLine, Underline, StrikeOut and Squiggly are deferred
//! in the canvas code itself. **Cloud** — revision clouds — is not even in
//! that list, and it is the one this audience will name first: it is AEC
//! table stakes. All seven are in [`super::PLANNED`].
//!
//! `RIBBON_IA.md` §5.5 lists `Line` among the four **G** shapes. The
//! shipped build has four markup kinds — Rectangle, Ellipse, `Arrow line`
//! and `Highlight band` — and no plain line: the parenthetical *"(Arrow is
//! `Arrow line`)"* resolves which of the two the existing control is, and
//! the answer is the arrow. A plain line is therefore **N** and is in
//! PLANNED rather than emitted, which is the conservative reading and the
//! one P3 requires: a button that arms a tool that does not exist is
//! exactly the placeholder the rule forbids.

use super::{command, group};
use crate::text::ribbon;
use egui_shell::manifest::{Item, Tab};

/// The Markup tab.
pub(super) fn tab() -> Tab {
    Tab::new("markup", ribbon::tab_markup())
        .with_question(ribbon::question_markup())
        .with_groups([
            // ---------------------------------------------------------------
            // Shapes. Polyline, Polygon, Cloud and Ink are **N**.
            // ---------------------------------------------------------------
            group(
                "shapes",
                ribbon::group_markup_shapes(),
                [
                    command("markup.rectangle"),
                    command("markup.ellipse"),
                    command("markup.arrow"),
                ],
            ),
            // ---------------------------------------------------------------
            // Text markup — markup that attaches to words already on the
            // page. Underline, Strikeout and Squiggly are **N**, which is
            // why this band holds one control; splitting Highlight out of
            // Shapes is still right, because a highlight is dragged across
            // text and a rectangle is dragged across space.
            // ---------------------------------------------------------------
            group(
                "text_markup",
                ribbon::group_markup_text(),
                [command("markup.highlight")],
            ),
            // ---------------------------------------------------------------
            // Notes. `Callout` is **N**; the stamp control exists and
            // needs a gallery, which is a change to the control rather
            // than a new command.
            // ---------------------------------------------------------------
            group(
                "notes",
                ribbon::group_markup_notes(),
                [
                    command("markup.text_box"),
                    command("markup.sticky_note"),
                    command("markup.stamp"),
                ],
            ),
            // ---------------------------------------------------------------
            // Style — see the module header on why this is a Custom item
            // rather than a command.
            // ---------------------------------------------------------------
            group(
                "style",
                ribbon::group_markup_style(),
                [Item::custom("colour_swatch")],
            ),
            // ---------------------------------------------------------------
            // Comments.
            //
            // `RIBBON_IA.md` §5.2 also lists a `Comments` entry under
            // View ▸ Panels. It cannot be in both places — one command,
            // one tab — and §7's migration map settles it explicitly:
            // `Review ▸ Comments ▸ Comments` → `Markup ▸ Comments`. Here.
            //
            // `Clear page` and `Clear all` are **N**.
            // ---------------------------------------------------------------
            group(
                "comments",
                ribbon::group_markup_comments(),
                [command("markup.comments")],
            ),
        ])
}
