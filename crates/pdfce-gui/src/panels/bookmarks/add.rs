//! # `panels::bookmarks::add` — writing a bookmark, and the `/Count` trap that
//! comes with it
//!
//! ## The gap this closes
//!
//! pdfce has read bookmarks since the reader passes and had **zero authoring
//! verbs opposite them** — the engine's own words. `EditSession::add_outline_item`
//! shipped 2026-08-19 as `Pass 103.0`, the first ask of this shell's
//! `insert_pages` request, and this is the surface for it.
//!
//! ## ★★ The `/Count` trap, and why nothing here diffs a number
//!
//! The engine flagged it as *"not a footnote … the entire difficulty of the
//! feature"*, and it is the one thing that would produce a wrong disclosure:
//!
//! | | root `/Outlines` | an item |
//! |---|---|---|
//! | `/Count` counts | visible items at **every** level, including top-level | visible **descendants**, excluding itself |
//! | absent means | no open items | the item is a **leaf** |
//!
//! On an item the **sign is the open/closed flag** — §12.3.3 defines no `/Open`
//! key, so the sign is the only carrier. And the consequence:
//!
//! > Adding a bookmark under a **collapsed** ancestor does not change the
//! > document's total, because the new item is not visible.
//!
//! So a surface reporting *"added N bookmarks"* by diffing the root count
//! reports **zero for a correct save**. Nothing here diffs anything: one call
//! adds one bookmark, and that is what is said.
//!
//! ## ★ And the collapsed case is DISCLOSED, not merely survived
//!
//! Getting the count right is the low bar. The operator's actual problem is
//! that they will add a bookmark under a collapsed parent, look at the panel,
//! and **not see it** — because it genuinely is not visible, and the panel is
//! correct to show it that way.
//!
//! `OutlineItem::open` is read from the parent before the add, so the sentence
//! can be said. It is the same posture the ce-dimension group window takes
//! about re-measuring on a move: state the surprising consequence before the
//! press, not after the operator has gone looking.
//!
//! ## Why only the current page, and only `Fit`
//!
//! Because those are the two things the engine authors today and the only ones
//! it authors **without refusing**. `Destination::Named` and `Remote` are
//! refused by name, and `DestView::Unknown` is *"the one that looks writable
//! and is not"* — the reader keeps an extension's fit name and discards its
//! parameters, so re-emitting it writes a view that is not the one the source
//! had.
//!
//! A destination chooser offering fits pdfce cannot write would be a control
//! whose options are mostly refusals, which is R9 at the level of a combo box.
//! The page the operator is looking at is the destination every other
//! page-scoped surface in this application uses, and it needs no chooser.

use egui::Ui;
use pdfce_core::object::ObjId;
use pdfce_core::outline::OutlineItem;

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::text::panels as t;

/// The region the title field publishes.
pub const REGION_TITLE: &str = "bookmarks.new_title"; // ui-text-exempt: trace region name, never displayed
/// The region the Add button publishes.
pub const REGION_ADD: &str = "bookmarks.add"; // ui-text-exempt: trace region name, never displayed

/// The panel's authoring state, between frames.
///
/// ★ **The parent is an `ObjId`, not a path through the tree.** `OutlineItem::id`
/// carries it for exactly this, and its own doc says why: *"identity is what a
/// GUI needs and the tree cannot otherwise supply … selecting a bookmark …
/// keys off the object, not off a path through the tree that any edit
/// invalidates."*
///
/// An index into the walk would name a different bookmark after every add,
/// which is the hazard the engine hit **in its own CLI** — *"the indices shift
/// after every add … I got this wrong myself while driving the command and
/// nested something two levels deeper than intended, and the output looked
/// entirely plausible."*
#[derive(Default)]
pub struct BookmarksUi {
    /// What has been typed into the title field.
    title: String,
    /// The bookmark a new one goes under, or `None` for the top level.
    ///
    /// Set by clicking a row, which also navigates — a bookmark click means
    /// *"take me there"* first and always, and making it mean *"and this is
    /// now the parent"* as well is free because both are true of the row the
    /// operator pointed at.
    parent: Option<ObjId>,
}

impl std::fmt::Debug for BookmarksUi {
    /// The title's length, not its text: a bookmark's name is the operator's
    /// own words about their drawing, and this reaches a trace file a harness
    /// keeps. `panels::properties::info` makes the same choice for `/Info`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BookmarksUi")
            .field("title_len", &self.title.len())
            .field("parent", &self.parent)
            .finish()
    }
}

impl BookmarksUi {
    /// Record the row the operator clicked as the parent for the next add.
    pub fn select(&mut self, id: ObjId) {
        self.parent = Some(id);
    }
}

/// Draw the add-a-bookmark row.
///
/// `items` is the outline as it currently stands, used for two things and
/// neither of them a count: naming the chosen parent, and reading whether it is
/// **collapsed**.
pub fn show(ui: &mut Ui, doc: &OpenDoc, ui_state: &mut BookmarksUi, actions: &mut Vec<Action>) {
    let page = doc.view.page_index;
    let outline = pdfce_core::outline::read_outline(&doc.session.view());
    // A parent that has gone — the document was edited, or reloaded — falls
    // back to the top level rather than naming an object that is not there.
    // Reachable through undo, which is the ordinary way an id stops resolving.
    let parent = ui_state.parent.and_then(|id| find(&outline.items, id));
    if ui_state.parent.is_some() && parent.is_none() {
        ui_state.parent = None;
    }

    ui.separator();
    ui.label(t::bookmark_add_heading());

    // --- where it goes -----------------------------------------------------
    ui.horizontal(|ui| {
        ui.label(match parent {
            Some(item) => t::bookmark_add_under(&display_title(&item.title)),
            None => t::bookmark_add_at_top().to_owned(),
        });
        // Offered only when there is something to clear, for the reason the
        // Rename button in the groups window is: a control whose only possible
        // effect is the state you are already in reads as broken.
        if parent.is_some() && ui.button(t::bookmark_add_to_top_button()).clicked() {
            ui_state.parent = None;
        }
    });
    ui.weak(t::bookmark_add_parent_hint());
    ui.weak(t::bookmark_add_destination(page.saturating_add(1)));

    // ★ The `/Count` disclosure, said BEFORE the press. See the module header:
    // a bookmark added under a collapsed parent is genuinely not visible, the
    // panel is correct to show it that way, and an operator who was not told
    // goes looking for a bookmark that is there.
    if parent.is_some_and(|item| !item.open) {
        ui.weak(t::bookmark_add_under_collapsed());
    }

    // --- the title and the button -----------------------------------------
    ui.horizontal(|ui| {
        let response = ui.add(
            egui::TextEdit::singleline(&mut ui_state.title)
                .desired_width(160.0)
                .hint_text(t::bookmark_add_title_hint()),
        );
        crate::diag::ui_rect(REGION_TITLE, response.rect);

        let title = ui_state.title.trim().to_owned();
        if title.is_empty() {
            // Greyed WITH an explanation rather than absent: unlike the Rename
            // button next door, this control is the *whole* of the feature, and
            // a row that vanished until you typed would leave an operator
            // looking for where bookmarks are added.
            let disabled = ui.add_enabled(false, egui::Button::new(t::bookmark_add_button()));
            crate::diag::ui_rect(REGION_ADD, disabled.rect);
            disabled.on_hover_text(t::bookmark_add_needs_a_title());
        } else {
            let button = ui.button(t::bookmark_add_button());
            crate::diag::ui_rect(REGION_ADD, button.rect);
            if button.clicked() {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed. The
                    // LENGTH, not the text — a bookmark's name is the
                    // operator's own words about their drawing.
                    format!(
                        "bookmark-add page={} under={:?} chars={}",
                        page + 1,
                        ui_state.parent.map(|id| id.num),
                        title.chars().count()
                    )
                });
                actions.push(Action::AddBookmark {
                    parent: ui_state.parent,
                    title,
                    page,
                });
                // Cleared so a second press cannot silently make a second
                // bookmark with the same name under the same parent — which
                // the engine would accept, and which would leave two
                // indistinguishable rows.
                ui_state.title.clear();
            }
        }
    });
}

/// Find an item by id, anywhere in the tree.
///
/// A depth-first walk rather than an index, for [`BookmarksUi::parent`]'s
/// reason: an id survives an edit and a position does not.
///
/// One line, because the recursion — the part with something to get wrong —
/// lives in [`find_in`], which **can be tested**. See that function.
fn find(items: &[OutlineItem], id: ObjId) -> Option<&OutlineItem> {
    find_in(items, id, |item| item.id, |item| item.children.as_slice())
}

/// Depth-first search of a tree, given the two things a tree is.
///
/// ## ★ Generic because `OutlineItem` is `#[non_exhaustive]`
///
/// This crate **cannot construct an `OutlineItem`**, so a search written
/// directly over one is a search no unit test in this crate can reach — and the
/// recursion is the only part of this module with something to get wrong. The
/// nested case is exactly the one an index gets wrong, and it is the one the
/// engine's own CLI got wrong: *"I … nested something two levels deeper than
/// intended, and the output looked entirely plausible."*
///
/// Taking the id and the children as accessors moves the algorithm somewhere a
/// test can build a tree for it. That is the fourth remedy in
/// `D:/dev/rag/rust/`'s `#[non_exhaustive]` finding — restructure so the logic
/// does not touch the unconstructible type — and it is the second time in this
/// codebase that the constraint pushed toward the better shape rather than
/// merely around it: `dialogs::insert_image`'s arithmetic went the same way for
/// the same reason.
///
/// **Depth first, and the order is load-bearing.** A breadth-first walk would
/// return a shallower item carrying a duplicate id before a deeper one — and
/// while `ObjId`s are unique in a well-formed document, an outline that made
/// them not so is exactly the malformed case `read_outline`'s cycle-breaking
/// exists for.
fn find_in<'a, T>(
    items: &'a [T],
    id: ObjId,
    id_of: impl Fn(&T) -> ObjId + Copy,
    children: impl Fn(&'a T) -> &'a [T] + Copy,
) -> Option<&'a T> {
    for item in items {
        if id_of(item) == id {
            return Some(item);
        }
        if let Some(found) = find_in(children(item), id, id_of, children) {
            return Some(found);
        }
    }
    None
}

/// A bookmark's title, or the stand-in for one that has none.
///
/// An untitled bookmark is **legal** — `OutlineItem::title`'s own doc says a
/// file may legitimately carry one — so naming it as the parent needs the same
/// stand-in the row does rather than an empty gap in a sentence.
fn display_title(title: &str) -> String {
    if title.trim().is_empty() {
        t::bookmark_untitled().to_owned()
    } else {
        title.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A tree this crate CAN build, standing in for the engine's.
    ///
    /// `OutlineItem` is `#[non_exhaustive]`, so the real one cannot be
    /// constructed here — which is why [`find_in`] takes accessors rather than
    /// the type. This is the tree it is exercised against.
    struct Node {
        id: ObjId,
        open: bool,
        children: Vec<Node>,
    }

    fn node(num: u32, open: bool, children: Vec<Node>) -> Node {
        Node {
            id: ObjId::new(num, 0),
            open,
            children,
        }
    }

    fn find_node(items: &[Node], num: u32) -> Option<&Node> {
        find_in(
            items,
            ObjId::new(num, 0),
            |n| n.id,
            |n| n.children.as_slice(),
        )
    }

    /// ★ A parent is found at any depth.
    ///
    /// Depth is the point. The hazard this search replaces is an **index**, and
    /// an index is wrong precisely for the nested case — which is the one the
    /// engine hit in its own CLI: *"I got this wrong myself while driving the
    /// command and nested something two levels deeper than intended, and the
    /// output looked entirely plausible."*
    #[test]
    fn a_parent_is_found_at_any_depth() {
        let tree = vec![
            node(1, true, vec![]),
            node(2, true, vec![node(3, false, vec![node(4, true, vec![])])]),
        ];
        assert_eq!(
            find_node(&tree, 4).map(|n| n.id.num),
            Some(4),
            "three levels down"
        );
        assert_eq!(
            find_node(&tree, 1).map(|n| n.id.num),
            Some(1),
            "the first sibling"
        );
        assert!(find_node(&tree, 99).is_none(), "an id that is not there");
    }

    /// ★ A collapsed parent is readable, which is what makes the disclosure
    /// possible at all.
    ///
    /// `open` is the shell's read of the **sign** on `/Count` — §12.3.3 defines
    /// no `/Open` key, so the sign is the only carrier — and it is the one
    /// field that decides whether an operator will be able to see what they
    /// just added.
    #[test]
    fn a_collapsed_parent_is_visible_to_the_disclosure() {
        let tree = vec![node(2, true, vec![node(3, false, vec![])])];
        assert!(!find_node(&tree, 3).expect("present").open);
        assert!(find_node(&tree, 2).expect("present").open);
    }

    /// An untitled parent is named by the stand-in, not by a gap.
    ///
    /// An untitled bookmark is **legal** — `OutlineItem::title`'s own doc says
    /// a file may carry one — so naming it as the parent needs the same
    /// stand-in the row uses rather than an empty space in a sentence.
    #[test]
    fn an_untitled_parent_is_still_nameable() {
        assert_eq!(display_title("   "), t::bookmark_untitled());
        assert_eq!(display_title(""), t::bookmark_untitled());
        assert_eq!(display_title("Chapter 3"), "Chapter 3");
    }
}
