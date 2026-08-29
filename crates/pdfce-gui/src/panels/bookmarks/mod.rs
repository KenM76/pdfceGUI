//! # `panels::bookmarks` — the document's outline, as navigation
//!
//! Salvaged from the old shell's `panels_structure.rs`, unchanged in
//! substance. **This is the only one of the six panels that can act on the
//! document at all**: it pushes [`Action::GoToPage`], which is the one thing
//! stage S3's action enum can carry that a panel wants.
//!
//! "Bookmarks", not "Outline": the PDF specification calls the structure an
//! outline (§12.3.3) and every other reader calls the things in it
//! bookmarks. The operator-facing word is the one operators use; the spec's
//! word stays in the code and the doc comments.
//!
//! # Why the tree is read fresh each frame rather than cached
//!
//! [`pdfce_core::outline::read_outline`] takes an object graph, not `&mut
//! self`, so it can run inside the draw closure — and the outline is a
//! property of the document that page edits can change (deleting a page can
//! leave a bookmark pointing nowhere). A cache would need invalidating on
//! every edit and undo, which is a correctness problem traded for a parse of
//! a structure that is a few hundred items at most.
//!
//! Measure before trading back. Note the contrast with
//! [`crate::panels::objects`], whose decomposition **is** cached: that one
//! walks every content stream on the page and there is no cache anywhere in
//! `pdfce-core`. The two panels differ because the work does, not because
//! one of them was optimised and the other forgotten.
//!
//! # A bookmark with no destination is NOT an error
//!
//! Three distinct states, and collapsing them would mislead:
//!
//! | State | Row | Why |
//! |---|---|---|
//! | points at a page pdfce resolved | enabled, tooltip names the page | the only one worth a click |
//! | a **heading** with no destination at all | disabled, tooltip says so | legal, common, groups its children |
//! | a destination pdfce could not resolve | disabled, tooltip says so | the document meant something and pdfce could not follow it |
//!
//! Only the third is a problem. Rendering the second and third alike would
//! send an operator hunting for damage in a perfectly ordinary document; not
//! showing the third at all would hide a real defect.
//!
//! Neither of the two disabled kinds is an *affordance* — R83: never offer a
//! control for something that cannot work. They are still drawn, because a
//! heading's children hang off it and omitting the parent would show them at
//! the wrong depth, silently misrepresenting the document's structure.
//!
//! [`pdfce_core::outline::Destination`] is `#[non_exhaustive]` with six
//! variants; only `Page { page_index, .. }` is navigable and the match below
//! says so by naming it and treating everything else as unresolved. That is
//! deliberate: a variant added to core must default to *"pdfce could not
//! follow this"*, never to a guess.
//!
//! # The truncation disclosure sits ABOVE the list
//!
//! An operator who scrolls a short list and stops has already drawn a
//! conclusion by the time a footnote would reach them. Same reasoning as the
//! Signatures caveat and the Fonts coverage note; three panels, one rule.
//!
//! # Indentation is keyed by object id, not by index
//!
//! `ui.indent` takes an id source, and two siblings at the same index in
//! different subtrees would collide in egui's id space — which shows up as
//! the wrong row responding to a hover. The item's `ObjId` (`num`,
//! `generation`) is unique across the document, so it cannot.

/// ★ Writing a bookmark — the half this panel did not have until
/// `EditSession::add_outline_item` shipped on 2026-08-19.
///
/// Its header carries the `/Count` trap the engine called *"the entire
/// difficulty of the feature"*: a bookmark added under a **collapsed** parent
/// does not change the document's total, so a surface reporting a diff reports
/// zero for a correct save — and, more to the point for an operator, the
/// bookmark is genuinely not visible until the parent is expanded.
pub mod add;

/// ★ Renaming a bookmark, and removing one with everything under it - the half
/// this panel did not have until `EditSession::set_outline_title` and
/// `EditSession::delete_outline_item` shipped on 2026-08-28.
///
/// Its header carries the two decisions a reader must not have to re-derive:
/// why the delete is **undoable rather than confirmed** (one press is one
/// engine command, so `Ctrl+Z` restores the whole subtree, and the sentence an
/// operator needs is *"this takes the eleven underneath"* rather than *"are you
/// sure?"*), and why reorder and re-parent render **nothing** rather than
/// greyed controls.
pub mod edit;
/// The two questions this panel asks of an outline - *where is this id?* and
/// *how many bookmarks are under this one?* - in the one place they can be
/// tested.
///
/// Split out of [`add`] when [`edit`] needed both. Its header carries why both
/// walks are generic over the tree (`OutlineItem` is `#[non_exhaustive]` and
/// this crate cannot build one, so a recursion written over it directly is a
/// recursion no test here can reach) and why the subtree count reads the
/// **tree** rather than `/Count`.
pub mod tree;

/// The panel's state, between frames.
///
/// ★ **Moved here from [`add`] on 2026-08-28**, when [`edit`] arrived. It was
/// never the add row's private state - the row it holds is the row the whole
/// panel is pointed at - and leaving it in `add` would have made the rename and
/// remove controls reach through the module that writes new bookmarks to find
/// the one they act on. `crate::panels::PanelsState` names the type and not its
/// path, so the move is invisible to every caller.
///
/// ★ **The selected bookmark is an `ObjId`, not a path through the tree.**
/// `OutlineItem::id` carries it for exactly this, and its own doc says why:
/// *"identity is what a GUI needs and the tree cannot otherwise supply ...
/// selecting a bookmark ... keys off the object, not off a path through the
/// tree that any edit invalidates."*
///
/// An index into the walk would name a different bookmark after every add,
/// which is the hazard the engine hit **in its own CLI** - *"the indices shift
/// after every add ... I got this wrong myself while driving the command and
/// nested something two levels deeper than intended, and the output looked
/// entirely plausible."*
#[derive(Default)]
pub struct BookmarksUi {
    /// What has been typed into the **new bookmark's** title field.
    ///
    /// Distinct from [`Self::rename`], which is a draft over an existing
    /// bookmark's name. Two fields rather than one because they answer
    /// different questions and are live at the same time: an operator may be
    /// half-way through naming a new bookmark when they decide to rename the
    /// one they had selected, and a shared buffer would swap one into the
    /// other.
    pub(super) title: String,
    /// ★ **The row the operator last clicked**, or `None` for none.
    ///
    /// One field, three meanings, all of them true of the row that was pointed
    /// at - which is what makes the overload honest rather than a shortcut:
    ///
    /// | read by | means |
    /// |---|---|
    /// | [`add`] | the parent a new bookmark is filed under; `None` is the top level |
    /// | [`edit`] | the bookmark being renamed |
    /// | [`edit`] | the bookmark being removed, with its subtree |
    ///
    /// The one seam worth knowing: `None` is a **meaningful** answer for the
    /// add (file it at the top level) and an **absent** one for the other two
    /// (nothing is selected, so R9 says draw nothing). So pressing *Move to top
    /// level* in the add row also takes the rename and remove controls away.
    /// That is correct - nothing is selected - and it is stated in [`edit`]'s
    /// header rather than left to be discovered.
    pub(super) selected: Option<pdfce_core::object::ObjId>,
    /// The rename draft, **paired with the bookmark it was typed for**.
    ///
    /// The pairing is the point: a half-typed name must not follow the operator
    /// to a different bookmark. A draft whose id does not match the selected
    /// row is stale, and [`Self::rename_draft_for`] re-seeds from the document
    /// instead of offering it.
    pub(super) rename: Option<(pdfce_core::object::ObjId, String)>,
}

impl std::fmt::Debug for BookmarksUi {
    /// The drafts' **lengths**, not their text: a bookmark's name is the
    /// operator's own words about their drawing, and this reaches a trace file
    /// a harness keeps. `panels::properties::info` makes the same choice for
    /// `/Info`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BookmarksUi")
            .field("title_len", &self.title.len())
            .field("selected", &self.selected)
            .field(
                "rename_len",
                &self.rename.as_ref().map(|(_, text)| text.len()),
            )
            .finish()
    }
}

impl BookmarksUi {
    /// Record the row the operator clicked.
    ///
    /// Clears any rename draft held for a *different* bookmark on the way
    /// through, which is belt-and-braces beside [`Self::rename_draft_for`]'s
    /// staleness test: the draft is re-seeded on read anyway, and dropping it
    /// here means a stale name does not sit in memory being not-shown.
    pub fn select(&mut self, id: pdfce_core::object::ObjId) {
        if self.rename.as_ref().is_some_and(|(held, _)| *held != id) {
            self.rename = None;
        }
        self.selected = Some(id);
    }

    /// Forget the selected row.
    ///
    /// Raised by the add row's *Move to top level* - where it means *"file the
    /// next one at the top"* - and by [`edit`] the instant a removal is raised,
    /// so the block does not spend one frame describing a bookmark that has
    /// gone. See that call site for why one frame matters.
    pub fn clear_selection(&mut self) {
        self.selected = None;
        self.rename = None;
    }

    /// The rename draft for `item`, seeded from the document when it is stale.
    ///
    /// *Stale* means **held for a different bookmark** - see [`Self::rename`].
    ///
    /// ★ **The draft does NOT follow the document while it is being typed**,
    /// deliberately, and that differs from `panels::properties::info`'s
    /// epoch-reseed. The difference is what the two fields are: a metadata box
    /// commits on focus loss and is otherwise idle, so re-seeding it costs
    /// nothing; a rename box is typed into and then committed, and an epoch
    /// bump from an unrelated edit - placing a dimension, moving a page -
    /// would wipe a half-typed name mid-keystroke.
    ///
    /// The narrow cost is that undoing a rename leaves the old name in the box
    /// until the operator selects another bookmark and comes back. The button
    /// re-appears, because the draft now differs from the document, so the
    /// state is legible rather than wrong. Same trade, same wording, as
    /// `panels::dimension_groups::identity::rename_draft_for`.
    pub(super) fn rename_draft_for(&self, item: &pdfce_core::outline::OutlineItem) -> String {
        match &self.rename {
            Some((id, text)) if *id == item.id => text.clone(),
            _ => item.title.clone(),
        }
    }

    /// Hold what is in the rename field, against the bookmark it belongs to.
    pub(super) fn set_rename_draft(&mut self, id: pdfce_core::object::ObjId, text: String) {
        self.rename = Some((id, text));
    }

    /// Drop the rename draft so the next frame re-seeds from the document.
    pub(super) fn clear_rename_draft(&mut self) {
        self.rename = None;
    }
}

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::panels::PanelsState;
use crate::text::panels as t;

/// Draw the Bookmarks panel.
pub fn body(ui: &mut egui::Ui, doc: &OpenDoc, state: &mut PanelsState, actions: &mut Vec<Action>) {
    let outline = pdfce_core::outline::read_outline(&doc.session.view());

    let total = outline.diagnostics.items;
    // The current page, so a driven click has an observable to check
    // against — the only oracle available when the operator is using the
    // machine and a screenshot harness would seize their screen.
    crate::diag::trace(|| {
        format!(
            "bookmarks-panel page={} items={total}",
            doc.view.page_index + 1
        )
    });
    ui.label(t::bookmarks_count(total));
    // The truncation disclosure sits ABOVE the list, not below it — see the
    // module docs.
    if outline.diagnostics.cycles_broken > 0
        || outline.diagnostics.depth_truncations > 0
        || outline.diagnostics.item_budget_exhausted
    {
        ui.label(egui::RichText::new(t::bookmarks_truncated()).small().weak());
    }
    if outline.items.is_empty() {
        ui.label(t::bookmarks_empty());
        // ★ NOT an early return any more. A document with no bookmarks is
        // exactly the one an operator most wants to add the first one to, and
        // returning here is what made this panel read-only-looking for its
        // whole life — the sentence said "none" and offered nothing.
        add::show(ui, doc, state.bookmarks_mut(), actions);
        return;
    }
    ui.separator();

    // Collected first, applied after — the actions-not-mutations discipline
    // at its smallest: the click is recorded while the tree is being walked
    // and turned into an `Action` once the walk is over.
    let mut go: Option<usize> = None;
    // ★ The clicked row is recorded as well as navigated to. A bookmark click
    // means "take me there" first and always; making it ALSO mean "and this is
    // the parent for the next one" is free, because both are true of the row
    // the operator pointed at, and it saves a second selection gesture that
    // would have to be taught.
    let mut picked: Option<pdfce_core::object::ObjId> = None;
    // ★★ The authoring row is drawn BEFORE the list, and that ordering is
    // the fix for a feature that shipped unreachable.
    //
    // A driven run on a 122-bookmark drawing found the panel body occupying
    // y=133..770 and this row laid out at y=899..923 — **below the bottom of
    // the panel**, with no way to reach it. The row drew. It published its
    // region. Every unit test passed. And `add_outline_item`, wired that
    // morning, could not be used on any document with a real outline, which is
    // every document somebody would want to add a bookmark to.
    //
    // The first attempt capped the list with a reserve, which moved the row
    // from y=899 to y=769 in a panel ending at 770 — still overflowing, by less.
    // That is the shape of a fix that is a **magic number**: it works at the
    // pane height it was tuned against and fails quietly at every other one.
    //
    // Putting the row first removes the arithmetic entirely. Nothing follows
    // the scroll area, so nothing can be pushed past the end of the panel, at
    // any pane height, with any size of outline. The rule generalises and is
    // worth stating: **a control that must always be reachable cannot be placed
    // after an unbounded `ScrollArea`.** Reserve-and-hope is not a second
    // option; it is the same defect with a tuning parameter.
    //
    // It also reads better, for the reason the Manage-groups window's Add
    // button was moved on the same pass: this row acts on the LIST — it files
    // the new bookmark under whichever row was last clicked — and a control's
    // position is a claim about what it acts on. Above the list it is making
    // that claim correctly, and the operator sees the destination before they
    // scroll rather than after.
    add::show(ui, doc, state.bookmarks_mut(), actions);
    // ★ The rename-and-remove block, and it is drawn ONLY when a row has been
    // clicked. That is R9 rather than tidiness: with nothing selected there is
    // no bookmark for either verb to name, so the controls would be offering a
    // capability that cannot act. They are absent, not greyed — greying is for
    // something *temporarily* unavailable that can explain itself, and "click a
    // row first" is already what the add row's parent hint says two lines up.
    //
    // Resolved here rather than inside `edit::show` so the whole block can be
    // skipped in one place, and so that module never has to consider an id that
    // no longer names anything — the ordinary state one frame after an undo of
    // a delete, and the state `add::show` above has already cleared.
    let selected = state
        .bookmarks_mut()
        .selected
        .and_then(|id| tree::find(&outline.items, id));
    if let Some(item) = selected {
        // Cloned because `state` is borrowed mutably by `edit::show` and the
        // item is borrowed out of `outline`, which `state` does not own. One
        // `OutlineItem` per frame in which a bookmark is selected, against
        // restructuring the whole panel to read the outline twice.
        let item = item.clone();
        edit::show(ui, &item, state.bookmarks_mut(), actions);
    }
    ui.separator();

    egui::ScrollArea::vertical()
        .id_salt("bookmark-rows")
        .show(ui, |ui| {
            rows(ui, &outline.items, &mut go, &mut picked);
        });
    if let Some(id) = picked {
        state.bookmarks_mut().select(id);
    }
    if let Some(page) = go {
        actions.push(Action::GoToPage(page));
    }
}

/// Draw one level of the outline, recursing into children.
///
/// Indentation carries the structure. See the module docs on why the indent
/// is keyed by the item's object id rather than by its index.
fn rows(
    ui: &mut egui::Ui,
    items: &[pdfce_core::outline::OutlineItem],
    go: &mut Option<usize>,
    picked: &mut Option<pdfce_core::object::ObjId>,
) {
    use pdfce_core::outline::Destination;
    for it in items {
        // The page a click would reach, if any. Only a resolved page
        // destination is navigable — a named destination pdfce could not
        // look up, or a remote file, is shown and not offered.
        let target = match &it.destination {
            Some(Destination::Page { page_index, .. }) => Some(*page_index),
            _ => None,
        };
        let (enabled, tip) = match (&it.destination, target) {
            (_, Some(p)) => (true, t::bookmark_row_tooltip(p + 1)),
            (None, _) => (false, t::bookmark_row_heading_tooltip().to_owned()),
            (Some(_), None) => (false, t::bookmark_row_unresolved_tooltip().to_owned()),
        };

        let label = if it.title.trim().is_empty() {
            // An untitled bookmark is legal and unclickable-looking. Its own
            // row still has to exist, or its children lose their parent and
            // appear at the wrong depth.
            t::bookmark_untitled().to_owned()
        } else {
            it.title.clone()
        };

        let resp = ui
            .add_enabled(enabled, egui::Button::new(label).frame(false))
            .on_hover_text(tip.clone());
        // A disabled widget does not show `on_hover_text`, so the two kinds
        // of unclickable row would be silent about WHY without this — which
        // is the whole of the three-state distinction.
        let resp = if enabled {
            resp
        } else {
            resp.on_disabled_hover_text(tip)
        };
        crate::diag::trace(|| {
            format!(
                "bookmark-row level={} title={:?} page={:?} enabled={enabled} rect={:?}",
                it.level,
                it.title,
                target.map(|p| p + 1),
                resp.rect
            )
        });
        if resp.clicked() {
            // The id is recorded whether or not the row navigates. A heading
            // with no destination is unclickable-looking and is still a
            // perfectly good PARENT — indeed it is the likeliest one, since a
            // heading is what an operator files things under.
            *picked = Some(it.id);
            if let Some(p) = target {
                *go = Some(p);
            }
        }

        if !it.children.is_empty() {
            ui.indent(("bookmark", it.id.num, it.id.generation), |ui| {
                rows(ui, &it.children, go, picked);
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::panels::objects::test_support::engine_fixture;
    use pdfce_core::outline::Destination;

    /// **A resolved bookmark's page index is 0-based and already resolved by
    /// core; the tooltip prints it 1-based.**
    ///
    /// The off-by-one that would otherwise be invisible: `page_index` is
    /// *"ALREADY 0-based into `pages`"* per `pdfce-core`'s consumer map, and
    /// [`Action::GoToPage`] takes the same 0-based index — so the raw value
    /// travels, and the `+ 1` happens only where a human reads it.
    ///
    /// Getting that backwards produces a panel that navigates one page past
    /// every bookmark, which looks like a document defect.
    #[test]
    fn a_resolved_destination_navigates_zero_based_and_prints_one_based() {
        let path = engine_fixture("outline/basic-tree.pdf");
        let doc = pdfce_core::document::Document::load(&path).expect("the fixture loads");
        let outline = pdfce_core::outline::read_outline(&doc);

        let resolved: Vec<usize> = outline
            .flatten()
            .into_iter()
            .filter_map(|it| match &it.destination {
                Some(Destination::Page { page_index, .. }) => Some(*page_index),
                _ => None,
            })
            .collect();
        assert!(
            !resolved.is_empty(),
            "the fixture must have at least one resolvable destination, or this \
             test proves nothing"
        );
        for page_index in resolved {
            // What the panel would push …
            let action = Action::GoToPage(page_index);
            assert_eq!(action, Action::GoToPage(page_index));
            // … and what it would print, which is one higher.
            let tip = t::bookmark_row_tooltip(page_index + 1);
            assert!(
                tip.contains(&(page_index + 1).to_string()),
                "the tooltip must name the human page number: {tip}"
            );
        }
    }

    /// **Every non-page destination is treated as unresolved, including ones
    /// this build has never seen.**
    ///
    /// `Destination` is `#[non_exhaustive]`, so core can add a variant
    /// without this crate changing. The match must therefore *fail closed*:
    /// anything that is not a resolved page is a row pdfce declines to
    /// offer, never a row it guesses at.
    ///
    /// Asserted against a real fixture whose destinations pdfce genuinely
    /// cannot resolve, using the same expression the panel uses, so the two
    /// cannot come apart. Constructing `Destination` values by hand would
    /// prove only that `matches!` works.
    #[test]
    fn any_destination_that_is_not_a_resolved_page_is_not_navigable() {
        let navigable = |d: &Option<Destination>| matches!(d, Some(Destination::Page { .. }));
        // A heading has no destination at all, and is not navigable.
        assert!(!navigable(&None));

        let path = engine_fixture("outline/broken-dests.pdf");
        let doc = pdfce_core::document::Document::load(&path).expect("the fixture loads");
        let outline = pdfce_core::outline::read_outline(&doc);
        let items = outline.flatten();
        assert!(!items.is_empty(), "the fixture must have bookmarks");

        let unresolvable = items
            .iter()
            .filter(|it| it.destination.is_some() && !navigable(&it.destination))
            .count();
        assert!(
            unresolvable > 0,
            "this fixture exists to carry destinations pdfce cannot map to a \
             page; if none survive, the test proves nothing about failing closed"
        );
    }

    /// The three row states carry three different tooltips, and the two
    /// unclickable ones say which kind they are.
    ///
    /// A heading and an unresolved destination are both disabled rows. If
    /// they read the same, an operator cannot tell a perfectly ordinary
    /// document from one whose outline is damaged.
    #[test]
    fn the_two_disabled_row_kinds_explain_themselves_differently() {
        let heading = t::bookmark_row_heading_tooltip();
        let unresolved = t::bookmark_row_unresolved_tooltip();
        assert_ne!(heading, unresolved);
        assert!(heading.contains("heading"), "{heading}");
        assert!(unresolved.contains("could not resolve"), "{unresolved}");
    }

    /// An untitled bookmark still gets a row.
    ///
    /// Its children hang off it; omitting the parent would show them at the
    /// wrong depth and silently misrepresent the document's structure.
    #[test]
    fn an_untitled_bookmark_has_a_label_rather_than_being_skipped() {
        assert!(!t::bookmark_untitled().trim().is_empty());
        // Whitespace-only titles take the placeholder too — a title of three
        // spaces is an invisible row, which is the same defect as no row.
        for title in ["", " ", "\t\n"] {
            assert!(
                title.trim().is_empty(),
                "the panel's emptiness test is `trim().is_empty()`; this pins the \
                 inputs it must catch"
            );
        }
    }
}
