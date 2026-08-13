//! # `panels::pages` — the document's pages, as pictures
//!
//! The thumbnail grid. `FEATURES.md`'s Phase 3 row — *"**Thumbnail grid** —
//! the Pages panel is not registered yet"* — and the last of the surfaces
//! `MODES_AND_PANELS.md` Part 1's table gives **all three** modes.
//!
//! | | |
//! |---|---|
//! | Ribbon command | `view.panel_pages` |
//! | Salvaged from | the old shell's `main.rs::thumbnail_rail` (~250 lines) and `raster::ThumbnailCache` |
//! | Acts on the document | [`Action::GoToPage`], and **nothing else** |
//! | Owns | [`select::PageSelection`] — the operand list the ribbon's Pages tab already promises |
//!
//! ## ★ Why a page panel sits in **Review**, and not only in Edit
//!
//! It is in all three default arrangements (`crate::app::modes::spec`), and
//! Review is the placement that needed an argument. `README.md` records the
//! operator's, and it is the reason this panel offers page verbs rather than
//! only navigation:
//!
//! > Reviewing a set means rotating a sheet to read it and extracting the
//! > pages you were asked about. The stance that matters is the content is
//! > not yours to alter, and page operations do not alter content.
//!
//! That is what separates rotate/extract/delete from the Edit tab's verbs. A
//! rotation changes `/Rotate`; an extraction writes a *different* file. Neither
//! touches a single content-stream operator, so neither breaches the stance
//! Review takes.
//!
//! ## What this panel draws, and what that costs
//!
//! The rendering and caching policy is [`thumbnails`]', and its header is the
//! one to read before changing anything here — it carries the measurements.
//! The one sentence that decides the shape of this file:
//!
//! > **A two-pixel render of the benchmark CAD drawing costs 691 ms.** ~99 %
//! > of a page's cost is resolution-independent, so a thumbnail is *not*
//! > cheap because it is small.
//!
//! Consequently this body renders **at most one page per frame**, only for
//! tiles that are actually on screen, and stops on its own the first time a
//! page proves expensive. An undrawn tile says so **in words**: a blank
//! rectangle the colour of paper is a picture of an *empty page*, which is a
//! thing a real PDF contains, so drawing one would assert something false
//! about the document rather than merely look unfinished.
//!
//! ## ★ Two surfaces this panel is built for and cannot reach in this build
//!
//! Both are `shell/`'s, and neither is this module's to add. They are named
//! here rather than left to be rediscovered, in the shape
//! `crate::shell::manifest::PLANNED` and `crate::app::modes::ABSENT_PANELS`
//! use for the same purpose.
//!
//! ### 1. `view.panel_pages` is not a registered command
//!
//! `crate::shell::manifest::PLANNED` carries it, with a reason written for
//! the *old* shell's furniture: *"page thumbnails are the sidebar rail's
//! first pane and have no independent toggle. `view.sidebar` shows the
//! rail."* There is no sidebar rail in this build — there is a dock, and
//! `crate::app::mod`'s panel registry registers a panel **only if its
//! command is registered**, so until that entry moves out of `PLANNED` and
//! into `crate::shell::commands::register`, this panel is filtered out of
//! every default arrangement by `SHELL_FRAMEWORK.md` §5b's capability rule
//! and an operator never sees it.
//!
//! That filter working correctly is why the panel could not simply be built
//! and forgotten about: it is invisible rather than broken, which is the
//! honest failure and also the silent one.
//!
//! ### 2. — closed
//!
//! `pages.row` had no definition when this panel was written: [`PAGES_ROW`] is
//! attached to every tile below, on every frame, through the same
//! [`MenuHost`] the canvas and the Objects panel use, but
//! `crate::shell::menus::built_in` defined four contexts and this was not one
//! of them — and `egui_shell::menu::Menu::attach` treats an unknown context as
//! *"this surface has no menu yet"*, so the right-click opened nothing at all.
//!
//! It is defined now, with the six verbs listed below, and **no edit was
//! needed here**: the attach site was already correct and the menu simply
//! started existing. That is the whole payoff of routing a right-click through
//! a context id rather than through a list of items at the call site.
//!
//! ## What a click does, and what it does not
//!
//! [`select`] owns the rule; the summary is that a plain click navigates and
//! picks one page, Ctrl+click toggles without navigating, and Shift+click
//! extends a range without navigating. **Only a plain click navigates**,
//! because building a five-page set that dragged the canvas through five
//! renders would cost ~4 s on a drawing set to perform a gesture that changes
//! nothing about what the operator is looking at.
//!
//! ## The selection is not a decoration
//!
//! Every one of the ribbon's Pages-tab tooltips already says *"the selected
//! pages"* — `pages.delete` is *"Remove **the selected pages** from this
//! document"* — and `crate::shell::commands`' own comment on that band says
//! those commands *"respect the thumbnail rail's selection when there is
//! one"*. This panel is where that selection comes from, and
//! [`crate::panels::PanelsState::selected_pages`] is how a dispatch arm will
//! read it. None of those arms exists yet (`crate::app::dispatch` has no
//! `pages.*` case), which is recorded here because the day the first one
//! lands it must read that accessor rather than invent a second selection.

pub mod select;
pub mod thumbnails;

use egui_shell::HandlerToken;

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::panels::PanelsState;
use crate::shell::menus::MenuHost;
use crate::text::pages as t;
use thumbnails::TileState;

/// Right-click on a page tile in the Pages panel.
///
/// Not yet defined by `crate::shell::menus::built_in` — see this module's
/// header. The constant lives here rather than being spelled at the attach
/// site for the reason that module gives for its own four: *"a context id is
/// used in exactly two places that must agree… a typo in either produces
/// silence rather than an error."* When the menu lands, this constant moves
/// to `shell::menus` beside the others and this one is deleted; until then it
/// is the single spelling this crate uses.
pub const PAGES_ROW: &str = "pages.row"; // ui-text-exempt: a menu context id, never displayed

/// The narrowest a tile may be drawn before the grid drops to one column.
///
/// Below roughly this width a drawing sheet's title block is no longer
/// legible and one thumbnail stops being distinguishable from the next, which
/// is the only job a thumbnail has. `crate::app::modes::NAVIGATOR_WIDTH` is
/// 280 pt *because* it fits two of these, and the two numbers are meant to
/// stay in step.
const MIN_TILE_WIDTH_PTS: f32 = 112.0;

/// The height reserved under each tile for its page number.
const CAPTION_HEIGHT_PTS: f32 = 16.0;

/// How thick the ring around the current page is drawn.
const CURRENT_RING_PTS: f32 = 2.0;

/// How much coloured mat a selected tile gets on each side.
///
/// A *shape* difference as well as a colour one — the tile visibly gains a
/// border where an unselected one has none. The old shell's rail put the same
/// reasoning behind its checkbox: *"a glyph AND a fill, never colour alone: a
/// colour-only state is invisible to a substantial fraction of operators."*
/// A mat is the version of that rule which needs no glyph, and therefore
/// cannot land on a font that has no glyph to draw — the failure that turned
/// the old rail's reorder arrows into empty boxes.
///
/// The header's *"N pages selected"* line is the third, wholly textual,
/// statement of the same fact.
const SELECTION_MAT_PTS: f32 = 3.0;

/// Points per millimetre, for the tooltip's sheet size.
///
/// A PDF user-space unit is 1/72 inch by definition (§8.3.2.3), and an inch
/// is 25.4 mm.
const PTS_PER_MM: f32 = 72.0 / 25.4;

/// Draw the Pages panel.
///
/// Returns the handler tokens a right-click produced — **intent**, never an
/// executed command. See [`crate::panels::Panel::show`] on why a panel must
/// not translate a context-menu command into an [`Action`] for itself.
#[must_use]
pub fn body(
    ui: &mut egui::Ui,
    doc: &OpenDoc,
    state: &mut PanelsState,
    host: Option<&MenuHost<'_>>,
    actions: &mut Vec<Action>,
) -> Vec<HandlerToken> {
    let pixels_per_point = ui.ctx().pixels_per_point();
    let page_count = doc.pages.len();
    let current = doc.view.page_index;
    let pages = state.pages_mut();

    // Everything that must be true before a tile is drawn, in one place:
    // the cache describes this revision at this density, and no picked page
    // names a sheet that has stopped existing.
    pages.cache.sync(doc.edit_epoch, pixels_per_point);
    pages.selection.retain_below(page_count);

    ui.label(t::pages_count(page_count));
    if page_count == 0 {
        // Reachable: a damaged `/Pages` node can flatten to nothing while the
        // file still opens. An empty grid would read as a panel that failed.
        ui.label(t::pages_none());
        return Vec::new();
    }
    if !pages.selection.is_empty() {
        ui.label(t::pages_selected(pages.selection.len()));
    }

    // The previews control. Read from the cache and written straight back, so
    // "is the box ticked" and "will anything be drawn" are one expression
    // rather than two that can disagree — see `ThumbnailCache::previews_on`.
    let mut previews_on = pages.cache.previews_on();
    if ui
        .checkbox(&mut previews_on, t::previews_label())
        .on_hover_text(t::previews_tooltip())
        .changed()
    {
        pages.cache.force_on(previews_on);
    }
    // The disclosure sits ABOVE the grid, not below it — the same rule the
    // Bookmarks, Signatures and Fonts panels state: an operator who looks at
    // a grid of undrawn tiles and stops has already drawn a conclusion by the
    // time a footnote would reach them.
    if let Some(slow) = pages.cache.slow() {
        ui.label(
            egui::RichText::new(t::previews_paused_note(slow.page_index, slow.millis))
                .small()
                .weak(),
        );
    }
    ui.separator();

    let mut go: Option<usize> = None;
    let mut tokens: Vec<HandlerToken> = Vec::new();
    let mut visible: Vec<usize> = Vec::new();

    let grid = egui::ScrollArea::vertical()
        .id_salt("pages-grid")
        .show(ui, |ui| {
            grid_rows(
                ui,
                doc,
                pages,
                current,
                host,
                &mut visible,
                &mut go,
                &mut tokens,
            );
        });

    // The two named regions a pixel check aims at. The panel's own rect comes
    // from the `Ui` rather than from a response, because the body is a column
    // of widgets and not a single one.
    crate::diag::ui_rect("panel-pages", ui.min_rect());
    crate::diag::ui_rect("panel-pages-grid", grid.inner_rect);

    // ★ One page per frame, chosen from what is on screen. See `thumbnails`'
    // header for why this is one and not two, and why it is here rather than
    // on the render worker.
    //
    // AFTER the grid rather than during it: the scheduling rule wants the
    // whole visible set, and rendering mid-layout would hold the frame in the
    // middle of a scroll area with half its rows placed.
    if let Some(page_index) = pages.cache.next_to_render(&visible, current)
        && let Some(page) = doc.pages.get(page_index)
    {
        let centre = viewport_centre(&visible, current);
        let elapsed = pages
            .cache
            .render(ui.ctx(), doc, page_index, page, pixels_per_point, centre);
        crate::diag::trace(|| {
            format!(
                "pages-thumbnail page={} ms={} state={:?} cached={}",
                page_index + 1,
                elapsed.as_millis(),
                pages.cache.state(page_index),
                pages.cache.ready_count(),
            )
        });
        // A page still to draw means another frame is wanted even if nothing
        // moved — otherwise the grid would fill only while the operator
        // happened to be generating input.
        ui.ctx().request_repaint();
    }

    crate::diag::trace_changed(PANEL_SLOT, || {
        format!(
            "pages-panel pages={page_count} current={} selected={} visible={} \
             drawn={} previews={}",
            current + 1,
            pages.selection.len(),
            visible.len(),
            pages.cache.ready_count(),
            u8::from(pages.cache.previews_on()),
        )
    });

    if let Some(page) = go {
        actions.push(Action::GoToPage(page));
    }
    tokens
}

/// Trace slot for the panel's once-per-change summary.
const PANEL_SLOT: &str = "pages-panel"; // ui-text-exempt: trace slot name, never displayed

/// Lay the grid out row by row, drawing only the rows that are on screen.
///
/// # Why rows are laid out by hand rather than with `horizontal_wrapped`
///
/// Two reasons, and the second is the load-bearing one.
///
/// A wrapped layout decides where the break falls from the widths it is
/// handed, so a grid of pages with **different sheet sizes** — which is what
/// a real drawing set is — wraps at a different column count on different
/// rows. The eye reads that as a fault in the panel.
///
/// And a wrapped layout gives no seam at which to ask *"is this row on
/// screen?"*. Culling is the whole reason a 900-page document is affordable
/// here: every row is *allocated* (so the scroll bar is honest and nothing
/// jumps as pictures arrive) but only a visible row is painted, interacted
/// with, or considered for rendering.
#[allow(
    clippy::too_many_arguments,
    reason = "each argument is a distinct output the body collects during one \
              layout pass — navigation, menu tokens and the visible set are \
              three different answers, and bundling them into a struct would \
              name a type whose only purpose is to be destructured immediately" // ui-text-exempt: clippy lint justification, never displayed
)]
fn grid_rows(
    ui: &mut egui::Ui,
    doc: &OpenDoc,
    pages: &mut PagesUi,
    current: usize,
    host: Option<&MenuHost<'_>>,
    visible: &mut Vec<usize>,
    go: &mut Option<usize>,
    tokens: &mut Vec<HandlerToken>,
) {
    let spacing = ui.spacing().item_spacing.x;
    let full_width = ui.available_width();
    let columns = columns_for(full_width, spacing);
    let tile_width = tile_width_for(full_width, spacing, columns);

    let mut first = 0usize;
    while first < doc.pages.len() {
        let last = (first + columns).min(doc.pages.len());
        let row = first..last;

        // The row's height is the tallest sheet in it, so a landscape A1
        // beside a portrait A4 sit on one baseline instead of stepping.
        let thumb_height = row
            .clone()
            .filter_map(|i| doc.pages.get(i))
            .map(|p| tile_height_for(p, tile_width))
            .fold(1.0f32, f32::max);
        let row_height = thumb_height + CAPTION_HEIGHT_PTS;

        let (row_rect, _) =
            ui.allocate_exact_size(egui::vec2(full_width, row_height), egui::Sense::hover());
        if ui.is_rect_visible(row_rect) {
            for (column, page_index) in row.clone().enumerate() {
                let Some(page) = doc.pages.get(page_index) else {
                    continue;
                };
                visible.push(page_index);
                let height = tile_height_for(page, tile_width);
                let origin = egui::pos2(
                    row_rect.left() + column as f32 * (tile_width + spacing),
                    // Bottom-aligned within the row, so the captions line up
                    // and the sheets stand on a common baseline.
                    row_rect.top() + (thumb_height - height),
                );
                let rect = egui::Rect::from_min_size(origin, egui::vec2(tile_width, height));
                tile(ui, doc, pages, page_index, current, rect, host, go, tokens);
            }
        }
        first = last;
    }
}

/// Draw one tile and read whatever the operator did to it.
#[allow(
    clippy::too_many_arguments,
    reason = "same as `grid_rows` — three independent outputs plus the four \
              inputs a tile is a function of" // ui-text-exempt: clippy lint justification, never displayed
)]
fn tile(
    ui: &mut egui::Ui,
    doc: &OpenDoc,
    pages: &mut PagesUi,
    page_index: usize,
    current: usize,
    rect: egui::Rect,
    host: Option<&MenuHost<'_>>,
    go: &mut Option<usize>,
    tokens: &mut Vec<HandlerToken>,
) {
    let id = ui.id().with(("pages-tile", page_index));
    let response = ui.interact(rect, id, egui::Sense::click());
    let visuals = ui.visuals().clone();
    let painter = ui.painter();

    // The selection mat, painted first so everything else sits on top of it.
    if pages.selection.contains(page_index) {
        painter.rect_filled(
            rect.expand(SELECTION_MAT_PTS),
            2.0,
            visuals.selection.bg_fill,
        );
    }
    // The sheet itself: paper, then a hairline, then either the picture or
    // the words that say why there is not one.
    painter.rect_filled(rect, 2.0, visuals.extreme_bg_color);
    painter.rect_stroke(
        rect,
        2.0,
        visuals.widgets.noninteractive.bg_stroke,
        egui::StrokeKind::Inside,
    );

    match pages.cache.state(page_index) {
        TileState::Ready => {
            if let Some(texture) = pages.cache.texture(page_index) {
                egui::Image::from_texture(texture)
                    .fit_to_exact_size(rect.size())
                    .paint_at(ui, rect);
            }
        }
        state => {
            // ★ Words, never a blank rectangle. See this module's header and
            // `crate::text::pages`': paper-coloured emptiness is a picture of
            // an empty page, and an empty page is a thing a PDF can contain.
            let words = match state {
                // ui-text-exempt: a panic message for an arm the match above
                // already took; never rendered.
                TileState::Ready => unreachable!("handled above"),
                TileState::NotDrawnYet => t::thumbnail_not_drawn_yet(),
                TileState::PreviewsOff => t::thumbnail_previews_off(),
                TileState::Abandoned => t::thumbnail_abandoned(),
                TileState::Failed => t::thumbnail_failed(),
            };
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                words,
                egui::TextStyle::Small.resolve(ui.style()),
                visuals.weak_text_color(),
            );
        }
    }

    // The current page's ring, outside the sheet so it cannot hide a hairline
    // of the picture. "Which page am I on" must be answerable at a glance,
    // and it must be answerable on an undrawn tile too — which is why it is
    // drawn after the words rather than only over a picture.
    if page_index == current {
        ui.painter().rect_stroke(
            rect,
            2.0,
            egui::Stroke::new(CURRENT_RING_PTS, visuals.selection.bg_fill),
            egui::StrokeKind::Outside,
        );
        crate::diag::ui_rect("panel-pages-current-tile", rect);
    }

    // The caption. Always drawn, for every state — the number and the sheet's
    // shape both come from the page tree rather than from rendering, so a
    // tile is never a row that says nothing.
    ui.painter().text(
        egui::pos2(rect.center().x, rect.bottom() + 2.0),
        egui::Align2::CENTER_TOP,
        t::page_number(page_index),
        egui::TextStyle::Small.resolve(ui.style()),
        if page_index == current {
            visuals.strong_text_color()
        } else {
            visuals.text_color()
        },
    );

    let (width_pts, height_pts) = doc
        .pages
        .get(page_index)
        .map_or((0.0, 0.0), crate::viewer::page_extent_pts);
    let response = response.on_hover_text(t::page_tile_tooltip(
        page_index,
        width_pts / PTS_PER_MM,
        height_pts / PTS_PER_MM,
    ));

    if response.clicked() {
        let modifiers = ui.input(|i| i.modifiers);
        // `command` rather than `ctrl`: it is Ctrl everywhere and Cmd on
        // macOS, which is what an operator's hand expects on the machine
        // they are using.
        let outcome = pages
            .selection
            .click(page_index, modifiers.command, modifiers.shift);
        if outcome.navigate {
            *go = Some(page_index);
        }
        crate::diag::trace(|| {
            format!(
                "pages-tile-click page={} ctrl={} shift={} navigate={} selected={}",
                page_index + 1,
                u8::from(modifiers.command),
                u8::from(modifiers.shift),
                u8::from(outcome.navigate),
                pages.selection.len(),
            )
        });
    }
    // The operand rule, before the attach: a menu's verbs must apply to the
    // tile the operator pointed at. See `select::PageSelection::right_click`.
    if response.secondary_clicked() && pages.selection.right_click(page_index) {
        crate::diag::trace(|| {
            format!(
                "pages-tile-right-click page={} selected={}",
                page_index + 1,
                pages.selection.len(),
            )
        });
    }
    if let Some(host) = host {
        tokens.extend(host.attach(&response, PAGES_ROW));
    }
}

/// How many columns fit in `available` points.
///
/// At least one, always: a dock dragged narrower than a single tile must show
/// a column of squeezed thumbnails rather than none at all, because zero
/// columns is a panel that has silently emptied itself.
#[must_use]
pub fn columns_for(available: f32, spacing: f32) -> usize {
    if !available.is_finite() || available <= 0.0 {
        return 1;
    }
    let per_column = MIN_TILE_WIDTH_PTS + spacing;
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "the value is clamped to at least 1.0 and bounded above by a \
                  window width in points, so it cannot exceed a few hundred" // ui-text-exempt: clippy lint justification, never displayed
    )]
    let columns = (((available + spacing) / per_column).floor() as usize).max(1);
    columns
}

/// How wide each tile is, once `columns` of them and their gaps share
/// `available`.
#[must_use]
pub fn tile_width_for(available: f32, spacing: f32, columns: usize) -> f32 {
    #[allow(
        clippy::cast_precision_loss,
        reason = "a column count is at most a few hundred and is exact in f32" // ui-text-exempt: clippy lint justification, never displayed
    )]
    let n = columns as f32;
    let gaps = spacing * (n - 1.0);
    // A floor of one point rather than zero: a zero-width rect makes egui lay
    // nothing out and the panel goes blank, which reads as a crash.
    ((available - gaps) / n).max(1.0)
}

/// How tall a tile is, from its page's own aspect ratio.
///
/// The shape is free — it comes from the page tree, not from rendering — so
/// every tile is the right shape from the first frame, before any picture
/// exists. That is what makes the scroll bar honest while the grid fills:
/// each row occupies its final height whether or not its pictures have
/// arrived, so nothing jumps.
#[must_use]
pub fn tile_height_for(page: &pdfce_core::page_tree::Page, tile_width: f32) -> f32 {
    let (width, height) = crate::viewer::page_extent_pts(page);
    if width > 0.0 && height > 0.0 {
        (tile_width * height / width).max(1.0)
    } else {
        // A degenerate `/CropBox`. Square is a visibly odd shape rather than
        // a plausible-looking wrong one, which is the right failure for a
        // page whose geometry the file did not state.
        tile_width
    }
}

/// The page at the middle of what is on screen — the centre eviction
/// measures distance from.
///
/// Falls back to the current page when nothing is visible, which happens on
/// the frame a panel is first mounted and on any frame the dock gives it no
/// height.
#[must_use]
pub fn viewport_centre(visible: &[usize], current: usize) -> usize {
    if visible.is_empty() {
        return current;
    }
    visible[visible.len() / 2]
}

/// The Pages panel's own state, between frames.
///
/// Held by [`crate::panels::PanelsState`], which owns every panel's
/// inter-frame state — see its header for why that is there rather than on
/// `PdfceApp`.
#[derive(Default)]
pub struct PagesUi {
    /// Which pages the operator has picked.
    pub selection: select::PageSelection,
    /// The pictures, and the policy that fills them.
    pub cache: thumbnails::ThumbnailCache,
}

impl std::fmt::Debug for PagesUi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PagesUi")
            .field("selection", &self.selection.len())
            .field("cache", &self.cache)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The dock's default navigator width fits **two** columns, which is the
    /// number `crate::app::modes::NAVIGATOR_WIDTH`'s own doc comment claims.
    ///
    /// Asserted here rather than trusted, because the two constants live in
    /// different modules and the claim is only true for a particular tile
    /// width: *"a thumbnail rail one column wide wastes the dock, and three
    /// columns makes each too small to recognise a drawing by."*
    #[test]
    fn the_default_navigator_width_fits_two_columns() {
        // The dock's 280 pt, less the panel's own margins and the scroll bar
        // this panel reserves 10 pt for.
        assert_eq!(columns_for(250.0, 8.0), 2);
        assert_eq!(columns_for(120.0, 8.0), 1, "a narrow dock is one column");
        assert!(
            columns_for(500.0, 8.0) >= 4,
            "a wide dock must use the width it was given"
        );
    }

    /// A dock dragged to nothing still asks for one column.
    ///
    /// Zero columns divides by zero in [`tile_width_for`] and lays out
    /// nothing, which reads as a panel that crashed rather than one that ran
    /// out of room.
    #[test]
    fn a_degenerate_width_still_produces_a_usable_grid() {
        for available in [0.0, -5.0, f32::NAN, 1.0] {
            let columns = columns_for(available, 8.0);
            assert!(columns >= 1, "{available} gave {columns} columns");
            let width = tile_width_for(available, 8.0, columns);
            assert!(width.is_finite() && width > 0.0, "{available} gave {width}");
        }
    }

    /// The columns and their gaps fill the width they were given, and never
    /// exceed it.
    ///
    /// Exceeding it is the defect `crate::panels::content_width`'s docs
    /// describe from the other side: a row wider than the viewport is
    /// silently squeezed and the overflow is clipped with nothing to say so.
    #[test]
    fn the_columns_and_their_gaps_exactly_fill_the_width() {
        for available in [250.0f32, 300.0, 512.0, 1000.0] {
            let spacing = 8.0;
            let columns = columns_for(available, spacing);
            let width = tile_width_for(available, spacing, columns);
            #[allow(
                clippy::cast_precision_loss,
                reason = "a small column count is exact in f32" // ui-text-exempt: clippy lint justification, never displayed
            )]
            let used = width * columns as f32 + spacing * (columns as f32 - 1.0);
            assert!(
                (used - available).abs() < 0.01,
                "{columns} columns of {width} used {used} of {available}"
            );
        }
    }

    /// **★ A tile is the shape of its page before any picture exists.**
    ///
    /// The property that keeps the scroll bar honest while the grid fills:
    /// the aspect ratio comes from the page tree, which is free, so each row
    /// occupies its final height from the first frame and nothing jumps as
    /// pictures arrive.
    #[test]
    fn a_tile_takes_its_shape_from_the_page_tree() {
        use crate::panels::objects::test_support::engine_fixture;
        let path = engine_fixture("pageops/four-pages.pdf");
        let doc = pdfce_core::document::Document::load(&path).expect("the fixture loads");
        let pages = pdfce_core::page_tree::pages(&doc).expect("a page tree");
        assert!(!pages.is_empty());
        for page in &pages {
            let (w, h) = crate::viewer::page_extent_pts(page);
            let height = tile_height_for(page, 120.0);
            assert!(height.is_finite() && height > 0.0);
            assert!(
                (height - 120.0 * h / w).abs() < 0.01,
                "a tile must be the page's own shape, not a fixed box"
            );
        }
    }

    /// The eviction centre is the middle of what is on screen, and falls back
    /// to the current page when nothing is.
    #[test]
    fn the_viewport_centre_is_the_middle_of_what_is_on_screen() {
        assert_eq!(viewport_centre(&[10, 11, 12, 13, 14], 0), 12);
        assert_eq!(
            viewport_centre(&[], 7),
            7,
            "a panel with no height must still name a centre"
        );
    }

    /// **★ The menu context this panel attaches is spelled once, and is
    /// now defined.**
    ///
    /// `crate::shell::menus`' own rule: *"a context id is used in exactly two
    /// places that must agree… a typo in either produces silence rather than
    /// an error."* This pins the spelling on both sides.
    ///
    /// It used to assert the opposite — that the context was **not** yet
    /// defined — so that the day it was, this test would fail and be updated
    /// in the same commit that made the right-click work. That day is this
    /// commit, and the assertion is inverted rather than deleted: the pairing
    /// it guards is the same pairing either way, and a menu attached to a
    /// context nobody defines opens nothing at all, silently.
    #[test]
    fn the_page_tile_menu_context_is_named_and_defined() {
        assert_eq!(PAGES_ROW, "pages.row");
        assert_eq!(
            PAGES_ROW,
            crate::shell::menus::PAGES_ROW,
            "the two spellings of this context id have drifted apart, which              detaches every tile's menu with no error anywhere"
        );
        let menus = crate::shell::menus::built_in();
        assert!(
            menus.get(PAGES_ROW).is_some(),
            "`{PAGES_ROW}` is attached by every tile and defined by nothing,              so the right-click opens nothing"
        );
        assert!(
            crate::shell::menus::CONTEXTS.contains(&PAGES_ROW),
            "the context list and the menu document disagree"
        );
    }

    /// **★ Every page verb this panel means to offer is a registered
    /// command.**
    ///
    /// The rule `crate::shell::menus`' header states — *only real commands* —
    /// checked from the panel's side before the menu exists, so the menu can
    /// be written from this list rather than from memory. A verb that failed
    /// here would be one to leave out, not one to add and grey.
    #[test]
    fn every_page_verb_the_menu_would_offer_is_registered() {
        use egui_shell::CommandRegistry;
        let mut registry = CommandRegistry::new();
        crate::shell::commands::register(&mut registry);
        for id in [
            "pages.rotate_left",
            "pages.rotate_right",
            "pages.delete",
            "pages.extract",
            "pages.move_up",
            "pages.move_down",
        ] {
            assert!(
                registry.get(id).is_some(),
                "`{id}` is not registered, so a menu row for it would render \
                 nothing at all"
            );
        }
        // …and the two the menu deliberately leaves out are absent from that
        // list rather than forgotten: `pages.split` and `pages.merge_into`
        // are document-level verbs that act on the whole file rather than on
        // the sheets pointed at, and both open a dialog this build has not
        // built. They stay on the ribbon's Pages tab.
        for id in ["pages.split", "pages.merge_into"] {
            assert!(
                registry.get(id).is_some(),
                "`{id}` is expected to exist on the ribbon even though the \
                 tile menu does not offer it"
            );
        }
    }

    /// **★ The measurement behind this panel's policy, on the real
    /// documents.**
    ///
    /// Ignored by default: it rasterizes whole pages and takes seconds, which
    /// is not a unit test's job. It is kept because the numbers in
    /// [`thumbnails`]' header are the entire argument for the design, and a
    /// claim about performance with no way to re-run it is a claim that
    /// quietly stops being true.
    ///
    /// ```text
    /// cargo test -p pdfce-gui -- --ignored --nocapture thumbnail_cost
    /// ```
    #[test]
    #[ignore = "measurement: rasterizes real pages, takes seconds"]
    fn thumbnail_cost_on_the_benchmark_documents() {
        use std::path::PathBuf;
        use std::time::Instant;

        let candidates = [
            PathBuf::from(r"D:\Dev\temp\pdfce\ncored-benchmark-cad-drawing.pdf"),
            PathBuf::from(r"D:\Dev\temp\pdfce\SW41177.pdf"),
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/a1-titleblock.pdf"),
            crate::panels::objects::test_support::engine_fixture("pageops/four-pages.pdf"),
        ];

        for path in candidates {
            if !path.exists() {
                println!("skipped (absent): {}", path.display());
                continue;
            }
            let document = pdfce_core::document::Document::load(&path).expect("loads");
            let pages = pdfce_core::page_tree::pages(&document).expect("a page tree");
            let session = pdfce_core::edit::EditSession::new(document);
            let view = session.view();
            let mut options = pdfce_render::RenderOptions::default();
            options.annotations = true;

            for (index, page) in pages.iter().enumerate().take(4) {
                let scale = thumbnails::raster_scale_for(page, 2.0);
                let started = Instant::now();
                let outcome = pdfce_render::render_page_with_view(&view, page, scale, &options);
                let ms = started.elapsed().as_millis();
                let size = outcome
                    .as_ref()
                    .map(|r| format!("{}x{}", r.pixmap.width(), r.pixmap.height()))
                    .unwrap_or_else(|e| e.to_string());
                println!(
                    "{}  page {} scale {scale:.3}  {ms} ms  {size}",
                    path.file_name().unwrap_or_default().to_string_lossy(),
                    index + 1,
                );
            }
        }
    }
}
