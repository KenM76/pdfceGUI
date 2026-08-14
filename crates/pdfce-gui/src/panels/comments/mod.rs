//! # `panels::comments` — every annotation on this document, listed
//!
//! The comment list a reviewer works through. Salvaged from the old shell's
//! `main.rs:7028-7060` (`fn comments_panel`), whose **exclusion reasoning is
//! settled law** and is carried across below with its argument rather than as
//! a code snippet.
//!
//! The classification lives in [`model`]; this file is the drawing, the
//! disclosures and the one action the panel can raise.
//!
//! ## ★ What it deliberately excludes, decided by exclusion first
//!
//! Straight from the old shell, and the wording is kept because the wording is
//! the decision:
//!
//! - **`/Widget`** — form fields have their own first-class surface (the Forms
//!   panel). `Annotation::is_widget()` already exists as the exact predicate;
//!   a second one would be a divergence waiting to happen.
//! - **`/Popup`** — a reader-UI window attached to a `Text`/`FreeText`
//!   annotation, never independent content. One row per real annotation; its
//!   pop-up is implementation detail. §12.5.6.14 is a `shall`: a pop-up
//!   *"shall not appear alone but is associated with a markup annotation, its
//!   parent annotation."*
//! - **ce dimensions are NOT excluded by type**, and that is worth stating:
//!   they are `/Line` annotations and so they appear here. The spec excludes
//!   them conceptually because they have their own home, but excluding them by
//!   subtype would also hide a genuine `/Line` markup an operator drew.
//!   Showing them is the lesser wrong, and it is honest — they ARE annotations
//!   on the document.
//!
//! ### The one place this build departs, and why
//!
//! The old shell also excluded **`/TrapNet`**, and its reason was
//! *delete-shaped*: core refuses a `/TrapNet` deletion by name, so listing one
//! *"would put a row here whose only possible action is a refusal, which is the
//! affordance R83 forbids."* **This build has no Delete** (see below), so that
//! reason does not reach.
//!
//! It is still excluded, on the half of the old argument that survives without
//! a Delete button: a `/TrapNet` is **prepress output state** — it records the
//! trapping a RIP applied to the page — so it is not a comment, nobody wrote
//! it, and there is nothing in it for a reviewer to work through. That is the
//! same shape as the `/Widget` exclusion: not "we cannot act on it" but "this
//! surface is not about it."
//!
//! What the departure buys instead is that **nothing is silently omitted**.
//! Every exclusion is counted and disclosed by
//! [`crate::text::panels::comments::comments_excluded`], so a reviewer looking
//! at six rows on a drawing they know carries forty annotations is told the
//! arithmetic and where each missing kind went. The old shell stated the rule
//! only on the empty case; this states the numbers on every case.
//!
//! **When a Delete lands here**, the old shell's `/TrapNet` reasoning becomes
//! live again as well and nothing needs to change: the row is already absent,
//! for a reason that does not depend on the button.
//!
//! ## Ordering: page order, then `/Annots` order
//!
//! The ordering `pdfce-cli list-annotations` already produces, **reused by
//! name** rather than a second GUI-only rule that could disagree with it. See
//! [`model`]'s header for what that means concretely, and for why there is no
//! sort by date.
//!
//! ## ★ Read the SESSION, not the file on disk
//!
//! [`body`] hands [`model::collect`] `doc.session.view()` — the base revision
//! with **every unsaved edit applied**, which is the same thing the canvas
//! rasterizes. An operator who has just drawn three shapes must see three rows
//! without saving first. `crate::panels::forms`' body is the worked example
//! and carries the same sentence.
//!
//! ## Actions, not mutations — and this panel raises exactly one
//!
//! [`Action::GoToPage`], from a row's **Go to** control, exactly as
//! `crate::panels::bookmarks` does. The body is handed `&OpenDoc` — a
//! **shared** reference, so this is a compile-time fact and not a convention —
//! it reads, and it pushes. It never touches the document.
//!
//! ### ★ There is no Delete, and its absence is a decision rather than a gap
//!
//! The old shell's panel could delete an annotation, with a hover-computed
//! collateral preview, a per-row `Locked` refusal and a document-wide
//! certification gate. **None of it is carried here**, because
//! `crate::app::actions::Action` has no variant that could carry the intent
//! and `app/actions.rs` is not this work's to extend.
//!
//! So the control renders **nothing at all**, which is the no-placeholders
//! rule (`HANDOFF.md` §6): *"A capability that is absent renders nothing,
//! never a greyed control that explains itself badly."* A disabled Delete
//! whose tooltip said "not built yet" would be the half-built surface
//! `crate::panels`' own header is about, and the strings for it are
//! deliberately absent from the catalog too — see
//! `crate::text::panels::comments`' header.
//!
//! What the day it lands needs, so nobody re-derives it, is written up in this
//! module's report to the shell owner: one `Action` variant, one dispatch arm
//! calling `EditSession::delete_annotation`, and the three disclosures
//! `docs/core-api/03-capabilities.md` §3.4 requires — *"delete is not
//! redaction"*, the deletion preview's collateral **before** the click, and
//! the fact that the preview *"is not a perfect oracle"* and the real call can
//! still refuse.
//!
//! ## Rule 4: everything here is disclosure, and none of it is on the page
//!
//! `D:\Dev\FeatureRequests\pdfce_FeatureRequests\README.md`'s first
//! non-negotiable: *"Disclosure lives off-canvas."* **A panel is the right
//! home**, and this one draws not a single pixel on the canvas — no badge on a
//! hidden annotation, no tint on an unresolved appearance, no outline round the
//! row under the pointer. It must not start to. The one-line test is *would a
//! screenshot of the editing canvas differ from a screenshot of the same
//! document saved and reopened?*
//!
//! Three of this panel's row captions exist **only** because of that rule, and
//! each names the inference it discloses:
//!
//! | Caption | The inference |
//! |---|---|
//! | `comment_row_hidden` | none — this one is a *document fact* the file states and the page therefore cannot show. §3.4.5: *"list it and mark it hidden."* |
//! | `comment_row_appearance_unresolved` | pdfce chose to paint nothing, under a default core documents as **evidence tier (d), a reasoned guess** |
//! | `comment_row_is_group_member` | pdfce shows the raw `/Contents` where §12.5.6.2 says a reader should show the group primary's — so another viewer legitimately disagrees |
//!
//! The old shell's Forms panel highlighted a field's rectangle on the page on
//! hover, and rule 4's fourth clause would permit the equivalent here (a hover
//! highlight *"is the cursor"*). It is not built, for the same reason that one
//! was not carried: the mechanism — a channel from a panel to the canvas
//! overlay — does not exist in this build, and `crate::canvas` is not this
//! module's to extend. Named rather than silently dropped, and named as a
//! *permitted* affordance so nobody later reads its absence as a rule.
//!
//! ## The two layout rules, and which one applies
//!
//! 1. **Scrollbars must be visible.** `crate::panels::scroll_style` is applied
//!    by [`crate::panels::Panel::show`] before any body runs, so this panel
//!    inherits it. egui's default `floating()` bar allocates zero space and is
//!    fully transparent when the pointer is elsewhere, which makes a scrolling
//!    area indistinguishable in a capture from content clipped at the
//!    container edge.
//!
//! 2. **A fixed-size child inside a scroll area needs the container's width
//!    stated.** ★ **This panel has no fixed-size child**, so
//!    [`crate::panels::content_width`] is deliberately not called — and that
//!    is stated here rather than left to look like an omission, because it is
//!    the second layout rule and skipping it silently is exactly how the
//!    Objects panel shipped clipped rows.
//!
//!    Every child is a `Label`, which wraps to whatever width it is given, so
//!    the clamping defect cannot arise: there is nothing whose *requested*
//!    size could exceed the pane and be silently squeezed. The one fixed-width
//!    child is the **Go to** button, at a couple of dozen points against a
//!    dock that opens at 320. A note body is arbitrary operator text and can
//!    be a paragraph; stating a container width computed from it would either
//!    scroll a 4,000 pt row sideways or defeat its own wrapping. Vertical-only
//!    scrolling with wrapping labels is the correct shape here, and
//!    `crate::panels::forms` — whose rows have the same character — takes it
//!    too.
//!
//! ## Cost, stated rather than discovered
//!
//! [`body`] walks **every page's `/Annots`** every frame, and lays out every
//! row it finds. Both are the old shell's behaviour and both are bounded —
//! `pdfce_core::annot::MAX_ANNOTS_PER_PAGE` caps the walk, and the walk reads
//! one array plus one dictionary per annotation rather than decomposing any
//! content — so on the documents this project measures against
//! (`SW41177.pdf`, 36 sheets) it is nothing beside a raster.
//!
//! The one thing that would change the picture is a document with thousands of
//! comments, where the *layout* rather than the walk becomes the cost. The fix
//! is `ScrollArea::show_rows`, which needs a uniform row height, which these
//! rows do not have — a row is one to six labels depending on what the
//! annotation carries. Named here so the next hand does not have to measure it
//! twice; `crate::panels::forms` carries the same note for the same reason.
//!
//! ## `PDFCE_DIAG` proves what the panel computed
//!
//! One `comments-panel` line per frame carrying the counts: rows found, rows
//! with note text, rows with an author, ce dimensions, suppressed rows,
//! unresolved appearances, relations — and the three exclusion counts
//! separately, so *how many it excluded and why* is answerable from the trace
//! alone. That is the founding rule of this project applied to a surface whose
//! correctness is entirely arithmetic: a screenshot of this panel cannot tell
//! you that a widget was excluded, and the trace can.

/// Turning a document into a comment list — the classification, testable
/// without a `Ui`.
pub mod model;

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::panels::PanelsState;
use crate::text::panels::comments as t;

use self::model::{CommentRow, Listing, Note, Relation};

/// The ribbon command that opens this panel.
///
/// Named here as well as on [`crate::panels::Panel::command_id`] so this
/// module's own reachability test can assert it, exactly as
/// `crate::panels::forms` does.
///
/// # ★ Why `markup.comments` and not a `view.panel_*` id
///
/// `RIBBON_IA.md` names Comments **twice** and the two placements cannot both
/// be honoured, because P1 gives a command one tab:
///
/// - **§5.2** lists `Comments` among View ▸ Panels, beside Pages, Objects,
///   Bookmarks, Layers, Signatures and Forms.
/// - **§5.5** gives the Markup tab its own `Comments` group, with `Comments
///   panel` in it.
/// - **§7's migration map** then settles it explicitly, naming the source and
///   the destination: `Review ▸ Comments ▸ Comments` → `Markup ▸ Comments`.
///
/// The migration map is the more specific statement — §5.2's row is a list of
/// panel names, while §7 is a per-control ruling on this control — so Markup ▸
/// Comments it is. `crate::shell::manifest::markup` already reached the same
/// conclusion in the same words when the tab was built, and the command is
/// already registered and already on the ribbon; only the panel behind it was
/// missing.
///
/// **The mode taxonomy agrees, which is what makes this safe.** The Forms
/// panel had to move *off* the Edit tab because Read is shown `file` and
/// `view` alone and Read mounts Forms. Comments is mounted by **Review and
/// Edit only** (`crate::app::modes::defaults`), and both of those are shown
/// the `markup` tab. So no mode can mount this panel without also being able
/// to reopen it — which is the failure the Forms move existed to prevent.
pub const COMMAND_ID: &str = "markup.comments";

/// Draw the Comments panel.
///
/// The one entry point. Shape and signature match every other panel body — see
/// [`crate::panels::Panel::show`].
///
/// `state` is unused, and that is a property of the panel rather than an
/// oversight: it is a pure function of the document. Nothing in it is expanded,
/// picked, drafted or remembered, so there is no inter-frame state to keep and
/// none is invented. A panel that stored a "selected comment" would be growing
/// the second selection [`crate::panels::ObjectTreeUi::focus`]' docs refuse.
pub fn body(ui: &mut egui::Ui, doc: &OpenDoc, _state: &mut PanelsState, actions: &mut Vec<Action>) {
    // Asked ONCE per frame, never per row. `dimension_model` walks the catalog
    // to the `/PieceInfo` sidecar and deserializes it — cheap, and bounded by
    // the number of ce dimensions rather than by the document — but calling it
    // per row would make the panel O(rows x sidecar), which is the shape of
    // defect the old shell's hover-gated deletion preview was fixing.
    let ce_dimensions = model::ce_dimension_annots(&doc.session);
    // Read the SESSION, not the file on disk — see the module header.
    let view = doc.session.view();
    let listing = model::collect(&view, &doc.pages, &ce_dimensions);

    trace(doc, &listing);

    let excluded = t::comments_excluded(
        listing.excluded.widgets,
        listing.excluded.popups,
        listing.excluded.trap_nets,
    );

    if listing.rows.is_empty() {
        // The empty case still discloses the filter. A drawing whose every
        // annotation is a form field is a real and common shape, and "no notes
        // or markup" alone would leave an operator who can *see* annotations on
        // the page believing the panel had failed.
        ui.label(t::comments_none());
        if let Some(line) = excluded {
            ui.label(egui::RichText::new(line).small().weak());
        }
        return;
    }

    // ★ EVERY DISCLOSURE SITS ABOVE THE LIST, without exception.
    //
    // The same rule the Bookmarks truncation note, the Signatures caveat and
    // the Fonts coverage note follow — four panels, one reason: an operator
    // who scrolls a short list and stops has already drawn their conclusion by
    // the time a footnote would reach them.
    //
    // The order is by how much it changes what the operator should do: the
    // count first (how big is this job), then what is missing from it, then why
    // the rows below look emptier than expected.
    ui.label(t::comments_count(listing.rows.len()));
    if let Some(line) = excluded {
        ui.label(egui::RichText::new(line).small().weak());
    }
    if listing.every_row_lacks_note_text() {
        ui.label(
            egui::RichText::new(t::comments_all_without_notes())
                .small()
                .weak(),
        );
    }
    ui.separator();

    // Collected during the draw and applied after it — the actions-not-
    // mutations discipline at its smallest, and the same shape
    // `crate::panels::bookmarks` uses. One `Option`, not a `Vec`: two rows
    // cannot be clicked in one frame, and a `Vec` would invite a future reader
    // to push two navigations that would fight.
    let mut go: Option<usize> = None;
    egui::ScrollArea::vertical()
        .id_salt("comment-rows")
        .show(ui, |ui| {
            let last = listing.rows.len() - 1;
            for (i, comment) in listing.rows.iter().enumerate() {
                // `push_id` per row, because two rows of the same subtype on
                // the same page would otherwise give their **Go to** buttons
                // the same egui id — which shows up as the wrong button
                // responding to a hover, the same collision
                // `crate::panels::bookmarks` keys its indent against.
                ui.push_id(i, |ui| row(ui, comment, &mut go));
                if i != last {
                    ui.separator();
                }
            }
        });

    if let Some(page) = go {
        actions.push(Action::GoToPage(page));
    }
}

/// Draw one comment.
///
/// Every line below the heading is conditional, and each condition is a real
/// state of a real document rather than a formatting choice. A row is between
/// two and seven lines tall depending on what the annotation actually carries,
/// which is why this panel cannot use `ScrollArea::show_rows` — see the module
/// header.
fn row(ui: &mut egui::Ui, comment: &CommentRow, go: &mut Option<usize>) {
    // The page number is 1-based **only here**, where a human reads it. The
    // index itself travels 0-based to `Action::GoToPage`; see
    // [`tests::the_page_index_travels_zero_based_and_prints_one_based`].
    let page_number = comment.page_index + 1;

    // ★ THE HEADING SAYS WHAT THE ANNOTATION ACTUALLY IS.
    //
    // A ce dimension is named as one — project rule 15, and the constructive
    // half of the exclusion argument in this module's header: the sidecar can
    // tell a ce dimension from a `/Line` markup, so the panel does not have to
    // choose between mislabelling one and hiding the other.
    let heading = if comment.is_ce_dimension {
        t::comment_row_ce_dimension_heading(&comment.subtype, page_number)
    } else {
        t::comment_row_heading(&comment.subtype, page_number)
    };
    ui.label(egui::RichText::new(heading).strong());

    // Author and modification date, when the annotation carries either.
    //
    // `/T` is a Table 170 MARKUP key, so its absence on a `/Link` or a
    // `/PrinterMark` means "this subtype has no such concept", not "anonymous"
    // — which is why an absent one prints nothing rather than a placeholder
    // that would read as a claim about a person.
    if let Some(byline) =
        t::comment_row_byline(comment.author.as_deref(), comment.modified.as_deref())
    {
        let resp = ui.label(egui::RichText::new(byline).small().weak());
        // The tooltip explains the *date*, so it is attached only when there
        // is one. Hanging it off an author-only byline would answer a question
        // that line does not raise.
        if comment.modified.is_some() {
            resp.on_hover_text(t::comment_row_modified_tooltip());
        }
    }

    // The note itself — three states, and collapsing any two would mislead.
    match &comment.note {
        Note::Text(text) => {
            ui.label(t::comment_row_body(text));
        }
        Note::Description(text) => {
            ui.label(t::comment_row_body(text));
            // §12.5.2's other meaning. Below the text rather than above it,
            // deliberately and against this panel's own disclosure-first rule:
            // the caption is *about* the string, and a reader has to have seen
            // the string for "this is not a note somebody wrote" to attach to
            // anything. The disclosure-first rule is about caveats that change
            // what you conclude from a LIST; this one qualifies one line.
            ui.label(
                egui::RichText::new(t::comment_row_description_caption())
                    .small()
                    .weak(),
            );
        }
        Note::Absent => {
            // Worded as a fact about the document, never as missing data. On
            // markup pdfce itself drew this is the *expected* state — the
            // engine cannot yet write `/Contents` on geometric markup — and
            // the document-wide sentence above the list has already said so
            // when it is true of every row.
            let caption = if comment.is_ce_dimension {
                t::comment_row_ce_dimension_no_note()
            } else {
                t::comment_row_no_note()
            };
            ui.label(egui::RichText::new(caption).small().weak());
        }
    }

    // The disclosures. Each is drawn only when it is true, so the marker means
    // something when it appears; a row of "not hidden / appearance fine /
    // not a reply" captions would be noise with the same information content
    // as nothing at all.
    if comment.suppressed {
        ui.label(egui::RichText::new(t::comment_row_hidden()).small().weak());
    }
    if comment.appearance_unresolved {
        ui.label(
            egui::RichText::new(t::comment_row_appearance_unresolved())
                .small()
                .weak(),
        );
    }
    match &comment.relation {
        Some(Relation::Reply) => {
            ui.label(
                egui::RichText::new(t::comment_row_is_reply())
                    .small()
                    .weak(),
            );
        }
        Some(Relation::GroupMember) => {
            ui.label(
                egui::RichText::new(t::comment_row_is_group_member())
                    .small()
                    .weak(),
            );
        }
        // An `/RT` name pdfce has never seen, and nothing to say about it —
        // see `model::Relation::Other`. Saying "this has an unrecognised
        // relationship" would be a placeholder for a fact with no consequence.
        Some(Relation::Other) | None => {}
    }

    if ui
        .button(t::comment_row_goto())
        .on_hover_text(t::comment_row_goto_tooltip(page_number))
        .clicked()
    {
        *go = Some(comment.page_index);
    }
}

/// One `comments-panel` line per frame, carrying what the panel computed.
///
/// # Why this is more than a debug print
///
/// `HANDOFF.md` §2: *"Verify by driving the binary, not by a passing test"* —
/// and the eighth defect on its list was found **only** by printing what the
/// running application had chosen, because *"2,450 hairlines and a wash are
/// the same picture"*. This panel has the same property in a different
/// direction: a screenshot of it cannot tell you that four widgets were
/// excluded, that two rows are hidden annotations, or that the `/Line` on
/// page 3 was recognised as a ce dimension. Every one of those is arithmetic,
/// and arithmetic is what a trace is for.
///
/// Every count that drives a *decision* is here, which is the test for what
/// belongs: `with_note` decides the document-wide disclosure, the three
/// exclusion counts decide the exclusion line, and `ce_dimensions`,
/// `suppressed`, `unresolved`, `replies` and `group_members` each decide a row
/// caption. If a number here is wrong, something on screen is wrong with it.
fn trace(doc: &OpenDoc, listing: &Listing) {
    crate::diag::trace(|| {
        let ce = listing.rows.iter().filter(|r| r.is_ce_dimension).count();
        let suppressed = listing.rows.iter().filter(|r| r.suppressed).count();
        let unresolved = listing
            .rows
            .iter()
            .filter(|r| r.appearance_unresolved)
            .count();
        let replies = listing
            .rows
            .iter()
            .filter(|r| matches!(r.relation, Some(Relation::Reply)))
            .count();
        let group_members = listing
            .rows
            .iter()
            .filter(|r| matches!(r.relation, Some(Relation::GroupMember)))
            .count();
        let descriptions = listing
            .rows
            .iter()
            .filter(|r| matches!(r.note, Note::Description(_)))
            .count();
        format!(
            "comments-panel pages={} listed={} with_note={} descriptions={} authors={} \
             ce_dimensions={ce} suppressed={suppressed} unresolved={unresolved} \
             replies={replies} group_members={group_members} \
             excluded_widgets={} excluded_popups={} excluded_trapnet={} excluded_total={}",
            doc.pages.len(),
            listing.rows.len(),
            listing.with_note_text(),
            descriptions,
            listing.rows.iter().filter(|r| r.author.is_some()).count(),
            listing.excluded.widgets,
            listing.excluded.popups,
            listing.excluded.trap_nets,
            listing.excluded.total(),
        )
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::panels::Panel;
    use crate::shell::{commands, manifest};
    use egui_shell::CommandRegistry;
    use std::collections::BTreeSet;

    /// **★ The command that opens this panel exists and is on the ribbon.**
    ///
    /// The check three panels in the old shell shipped without: they had a
    /// body, a rail entry and a diagnostic step, and *"no control an operator
    /// could click"*, so every verification passed while they were unreachable
    /// in a real build.
    ///
    /// Two assertions, and both are needed. A command **the manifest
    /// references** is one the ribbon draws a control for; a command **the
    /// registry holds** is one that has a label, a tooltip and an enable
    /// predicate. Either alone is half a control.
    ///
    /// `crate::panels::tests::every_panel_is_reachable_from_the_ribbon` sweeps
    /// the same property across every panel; this one names *this* panel in
    /// its failure message, which is what a reader who has just added it
    /// wants to see.
    #[test]
    fn the_comments_command_is_reachable_from_the_ribbon() {
        let shell = manifest::built_in();
        let mut registry = CommandRegistry::new();
        commands::register(&mut registry);
        let referenced: BTreeSet<String> = shell
            .command_references()
            .into_iter()
            .map(|(_, id)| id)
            .collect();

        assert!(
            referenced.contains(COMMAND_ID),
            "no tab, QAT slot or key binding references `{COMMAND_ID}`, so an \
             operator cannot open the Comments panel. `RIBBON_IA.md` §7 puts it \
             on Markup ▸ Comments."
        );
        assert!(
            registry.get(COMMAND_ID).is_some(),
            "`{COMMAND_ID}` is not registered, so the ribbon has an id with no \
             label, no tooltip and no enable predicate, and draws nothing for it."
        );
    }

    /// **The panel and this module name the same command.**
    ///
    /// Two spellings of one id is two things to keep in step, and the failure
    /// when they drift is a panel that opens from the ribbon and draws nothing
    /// in the dock — which looks like a rendering bug and is not.
    #[test]
    fn the_panel_enum_and_this_module_agree() {
        assert_eq!(Panel::Comments.command_id(), COMMAND_ID);
    }

    /// **★ The page index travels 0-based and prints 1-based.**
    ///
    /// The off-by-one that would otherwise be invisible.
    /// [`crate::app::actions::Action::GoToPage`] takes a 0-based index — the
    /// same convention `crate::panels::bookmarks` pins from its own side — and
    /// every string a human reads takes the number one higher. Getting it
    /// backwards produces a panel that navigates one page past every comment,
    /// which looks like a document defect.
    ///
    /// Asserted against a real fixture rather than a constructed row, so the
    /// indices are ones the collector actually produced.
    #[test]
    fn the_page_index_travels_zero_based_and_prints_one_based() {
        use crate::panels::objects::test_support::engine_fixture;

        let path = engine_fixture("annot/thread.pdf");
        let doc = pdfce_core::document::Document::load(&path).expect("the fixture loads");
        let pages = pdfce_core::page_tree::pages(&doc).expect("a page tree");
        let session = pdfce_core::edit::EditSession::new(doc);
        let listing = model::collect(
            &session.view(),
            &pages,
            &model::ce_dimension_annots(&session),
        );
        assert!(
            !listing.rows.is_empty(),
            "the fixture must carry annotations, or this test proves nothing"
        );

        for comment in &listing.rows {
            // What the row would push …
            let action = Action::GoToPage(comment.page_index);
            assert_eq!(action, Action::GoToPage(comment.page_index));
            // … and what it prints, which is one higher, in both the heading
            // and the button's tooltip.
            let human = comment.page_index + 1;
            let heading = t::comment_row_heading(&comment.subtype, human);
            assert!(heading.contains(&human.to_string()), "{heading}");
            let tip = t::comment_row_goto_tooltip(human);
            assert!(tip.contains(&human.to_string()), "{tip}");
        }
    }

    /// **A ce dimension's heading names it as one and keeps the subtype.**
    ///
    /// Rule 15 at the point of use. The bracketed `/Line` is not decoration:
    /// the exclusion argument in this module's header turns on ce dimensions
    /// *being* `/Line` annotations, and a heading that hid that would quietly
    /// contradict the argument that put the row in the list.
    #[test]
    fn a_ce_dimension_row_says_ce_dimension_and_still_says_line() {
        let heading = t::comment_row_ce_dimension_heading("Line", 3);
        assert!(heading.contains("ce dimension"), "{heading}");
        assert!(heading.contains("Line"), "{heading}");
        // …and an ordinary `/Line` markup is not relabelled.
        let plain = t::comment_row_heading("Line", 3);
        assert!(!plain.contains("dimension"), "{plain}");
    }
}
