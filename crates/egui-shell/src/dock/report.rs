//! The names the dock publishes its drawn rectangles under, and the sink
//! that receives them.
//!
//! # Why a rect stream exists at all
//!
//! `MODES_AND_PANELS.md` Part 2 lists three prerequisites before any of
//! the flexible-panel work, and the second is the one this module
//! serves:
//!
//! > **A screenshot oracle for panel layout.** Two recorded instances
//! > where a traced rect was correct and the control was still clipped
//! > out of its pane: *"layout/clipping defects have exactly one oracle:
//! > a rendered screenshot."*
//!
//! Note carefully what that says and what it does not. A rect stream is
//! **not** the oracle for legibility or for clipping; the RAG entry
//! `headless_trace_asserts_reached_not_visible_a_clipped_widget_needs_a_pixel_oracle`
//! makes the same point from the other side. What a rect stream *is* good
//! for is the class of assertion where the geometry is the whole
//! question — *did the overflow affordance get drawn inside the tab bar,
//! or past its right edge?* — and that is precisely failure mode #8.
//!
//! The dock publishes the rects that make the twelve failure modes
//! checkable from outside, and leaves the pixel questions to
//! `tools/ui-verify`.
//!
//! # And why the names are structural rather than opaque
//!
//! Every name here is built from the *position* in the layout, not from
//! a generated id. A harness reading `dock.left.0.1.tabbar` knows which
//! compartment it is looking at without holding a map, and two runs of
//! the same layout produce the same names. That matters more than it
//! sounds: the RAG entry
//! `scripted_click_coordinates_go_stale_when_a_dock_width_changes` records
//! a whole class of false defect caused by harness coordinates that
//! outlived the layout they were measured in. Structural names let a
//! harness *re-read* rather than remember.
//!
//! Panel-scoped names carry the [`super::PanelId`] instead, because a
//! panel's identity survives being dragged to a different compartment and
//! a harness looking for "the Pages panel" should not have to know where
//! the operator put it.

use egui::Rect;

use super::model::{DockSide, PanelId};

/// The callback an application supplies to receive drawn rectangles.
///
/// Identical in shape to [`crate::ribbon::RectSink`] so an application
/// can pass the same closure to both surfaces and filter on the name
/// prefix.
pub type RectSink<'a> = dyn FnMut(&str, Rect) + 'a;

/// The name prefix every rect this module publishes begins with.
pub const PREFIX: &str = "dock";

/// The whole of one dock side.
#[must_use]
pub fn side(side: DockSide) -> String {
    format!("{PREFIX}.{}", side.key())
}

/// One column within a side.
#[must_use]
pub fn column(side: DockSide, column: usize) -> String {
    format!("{PREFIX}.{}.{column}", side.key())
}

/// One stack — tab bar and body together.
#[must_use]
pub fn stack(side: DockSide, column: usize, stack: usize) -> String {
    format!("{PREFIX}.{}.{column}.{stack}", side.key())
}

/// One stack's tab bar.
///
/// Published separately from the stack because failure mode #8 is a
/// statement about what fits *inside the bar*, and asserting it against
/// the whole compartment's rect would pass trivially.
#[must_use]
pub fn tab_bar(side: DockSide, column: usize, stack: usize) -> String {
    format!("{PREFIX}.{}.{column}.{stack}.tabbar", side.key())
}

/// One stack's overflow affordance.
///
/// **This is the rect failure mode #8 is asserted against.** At a width
/// where tabs are hidden, this must exist, have a positive area, and lie
/// within the tab bar published by [`tab_bar`].
#[must_use]
pub fn overflow(side: DockSide, column: usize, stack: usize) -> String {
    format!("{PREFIX}.{}.{column}.{stack}.overflow", side.key())
}

/// One tab button, named by the panel it selects.
#[must_use]
pub fn tab(panel: &PanelId) -> String {
    format!("{PREFIX}.tab.{panel}")
}

/// One panel's body region — the rectangle the application drew into.
#[must_use]
pub fn body(panel: &PanelId) -> String {
    format!("{PREFIX}.body.{panel}")
}

/// A splitter between two columns of a side.
#[must_use]
pub fn column_splitter(side: DockSide, boundary: usize) -> String {
    format!("{PREFIX}.{}.split.col.{boundary}", side.key())
}

/// A splitter between two stacks of a column.
#[must_use]
pub fn stack_splitter(side: DockSide, column: usize, boundary: usize) -> String {
    format!("{PREFIX}.{}.{column}.split.row.{boundary}", side.key())
}

/// The **collapse control** on an open side — the little tab that minimises it.
#[must_use]
pub fn collapse(side: DockSide) -> String {
    format!("{PREFIX}.{}.collapse", side.key())
}

/// The rail a collapsed side leaves behind — the way back.
#[must_use]
pub fn rail(side: DockSide) -> String {
    format!("{PREFIX}.{}.rail", side.key())
}

/// The splitter between a side and the central area.
#[must_use]
pub fn side_splitter(side: DockSide) -> String {
    format!("{PREFIX}.{}.split.side", side.key())
}

/// Holds the application's rect sink, if there is one.
///
/// A struct rather than a bare `Option` so the "do not format the name
/// unless someone is listening" rule lives in one place. Every call site
/// in the dock goes through it, and the rule matters here more than in
/// the ribbon: a dock draws a name per tab per stack per column per side
/// per frame, and `format!` on a hot path with nobody reading it is pure
/// waste.
pub struct Reporter<'a> {
    sink: Option<&'a mut RectSink<'a>>,
}

impl<'a> Reporter<'a> {
    /// Wrap a sink, or nothing.
    #[must_use]
    pub fn new(sink: Option<&'a mut RectSink<'a>>) -> Self {
        Self { sink }
    }

    /// Whether anyone is listening.
    ///
    /// Public so a caller can skip an expensive *measurement* — not only
    /// an allocation — when nothing will read it.
    #[must_use]
    pub fn is_listening(&self) -> bool {
        self.sink.is_some()
    }

    /// Publish a rect under a lazily-formatted name.
    pub fn report(&mut self, rect: Rect, name: impl FnOnce() -> String) {
        if let Some(sink) = self.sink.as_deref_mut() {
            sink(&name(), rect);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every published name begins with the prefix, so a harness can
    /// separate the dock's stream from the ribbon's.
    #[test]
    fn every_name_carries_the_prefix() {
        let names = [
            side(DockSide::Left),
            column(DockSide::Right, 0),
            stack(DockSide::Left, 1, 2),
            tab_bar(DockSide::Left, 0, 0),
            overflow(DockSide::Right, 0, 0),
            tab(&PanelId::new("pages")),
            body(&PanelId::new("pages")),
            column_splitter(DockSide::Left, 0),
            stack_splitter(DockSide::Left, 0, 1),
            side_splitter(DockSide::Right),
        ];
        for name in &names {
            assert!(name.starts_with(PREFIX), "{name} is not prefixed");
        }
    }

    /// Names distinguish the two sides and the two axes of splitter, so a
    /// harness cannot mistake one for another.
    #[test]
    fn names_are_distinct_across_sides_and_axes() {
        assert_ne!(side(DockSide::Left), side(DockSide::Right));
        assert_ne!(
            column_splitter(DockSide::Left, 0),
            stack_splitter(DockSide::Left, 0, 0)
        );
        assert_ne!(stack(DockSide::Left, 0, 1), stack(DockSide::Left, 1, 0));
    }

    /// A panel-scoped name follows the panel rather than its position, so
    /// a harness that finds "the Pages panel" keeps finding it after the
    /// operator moves it.
    #[test]
    fn panel_names_do_not_encode_a_position() {
        let name = tab(&PanelId::new("pages"));
        assert_eq!(name, "dock.tab.pages");
        assert!(!name.contains("left"));
    }

    /// A reporter with no sink formats nothing.
    #[test]
    fn a_silent_reporter_does_not_format_names() {
        let mut reporter = Reporter::new(None);
        assert!(!reporter.is_listening());
        reporter.report(Rect::ZERO, || panic!("the name was formatted"));
    }
}
