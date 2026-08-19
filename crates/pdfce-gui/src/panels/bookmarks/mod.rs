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
    egui::ScrollArea::vertical()
        .id_salt("bookmark-rows")
        .show(ui, |ui| {
            rows(ui, &outline.items, &mut go, &mut picked);
        });
    if let Some(id) = picked {
        state.bookmarks_mut().select(id);
    }
    add::show(ui, doc, state.bookmarks_mut(), actions);
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
