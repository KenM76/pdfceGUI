//! The **View** tab — *what is on my screen, and how is the page laid
//! out?*
//!
//! `RIBBON_IA.md` §5.2, amended by `MODES_AND_PANELS.md`'s two new Window
//! settings. Six groups: Page display, Render, Zoom, Display, Panels,
//! Window.
//!
//! # The defect this tab exists to fix
//!
//! `RIBBON_IA.md` §3, on the shipped build:
//!
//! > **The View tab contains no view controls.** It has two groups:
//! > `Panels` and `Show`. There is no zoom, no page layout, no view
//! > rotation, no read mode, no full screen. Read mode and full screen
//! > have **no ribbon control at all** — they are keyboard-only (Ctrl+H,
//! > F11) on a tab literally named View. This is the single most confusing
//! > thing in the current ribbon.
//!
//! So this tab gains four groups and loses one command (`Fonts`, to File ▸
//! Document, because it describes the file rather than the screen).
//!
//! # Zoom here does not duplicate the status bar in spirit
//!
//! The status bar keeps the *continuous* controls a user reaches for
//! constantly: −/%/+ and the fit toggles. This tab mirrors the three
//! **named** zoom levels under P1a — actual size, fit page, fit width — so
//! that a user looking under View for zoom finds zoom. The two *targeted*
//! zooms that would have no status-bar home, zoom-to-selection and marquee
//! zoom-to-region, are **N** and therefore absent.
//!
//! Mirroring is legal because the status bar is not a tab; the same
//! amendment that lets the QAT carry Open lets the status bar carry Fit
//! page. What P1 forbids, and `egui-shell` still enforces, is one command
//! on two *tabs*.
//!
//! # Two documented conflicts, resolved here
//!
//! `RIBBON_IA.md` §5.2's table lists **`Thin lines` twice** — once under
//! Render and once under Display. One command cannot be on one tab twice;
//! `egui_shell::Shell::validate` refuses it by name. It is kept in
//! **Render**, because that is where the parameter acts (it is a
//! rasterization rule about minimum stroke width, not an overlay the
//! viewer draws) and because the Render group's contents were enumerated
//! explicitly, thin lines included, when this tab was commissioned. The
//! Display entry is treated as the duplicate.
//!
//! §5.2 also lists **`Comments`** among View ▸ Panels' panel toggles,
//! while §5.5 gives Markup a `Comments` group holding the Comments panel
//! and §7's migration map sends the existing control there explicitly
//! (`Review ▸ Comments ▸ Comments` → `Markup ▸ Comments`). The migration
//! map is the more specific statement, so the command lives on **Markup**
//! and this tab's Panels group does not list it.
//!
//! # The Render group is an operator decision, and it is a stated trade
//!
//! pdfce caches one whole-page texture and scales it with linear filtering
//! during the settle interval. Measured in use on a large drawing, that is
//! *smoother* to pan and zoom than the comparison product's progressive
//! tile rendering — no seams, no piece-by-piece fill-in — at the cost of a
//! full re-raster once motion stops.
//!
//! Those are two legitimate trades, not a better and a worse, and which
//! one wins depends on the sheet and the machine. So the strategy is a
//! **choice on this tab**, with whole-page as the default because it is
//! what measured better. `ZOOM_SETTLE` and the raster-scale multiplier are
//! constants in the shipped code and become the two knobs beside it.
//!
//! **Status note.** Three of the five Render entries and both new Window
//! settings are **N** in `RIBBON_IA.md`'s marking, and would be absent
//! under P3. They are present because the tab was commissioned with them
//! named individually and their defaults specified — see
//! [`super::DIRECTED`], which lists every such entry with the instruction
//! that put it there, so the exception is visible rather than inferred.

use super::{command, group};
use crate::text::ribbon;
use egui_shell::manifest::Tab;

/// The View tab.
pub(super) fn tab() -> Tab {
    Tab::new("view", ribbon::tab_view())
        .with_question(ribbon::question_view())
        .with_groups([
            // ---------------------------------------------------------------
            // Page display — a radio set of which exactly one is active.
            //
            // Single page stays the **default**: paging one drawing sheet
            // at a time is the right model for drafting review, and the
            // existing navigation is good. Continuous, facing and
            // facing-continuous are modes chosen here for the case where
            // the document is a 40-page specification rather than a sheet
            // set, and the choice persists per document so opening a
            // drawing set does not inherit a report's setting.
            //
            // All three are **N** and absent. This is the one group where
            // that leaves a single-item radio, which looks odd and is
            // honest: the alternative is three buttons that do nothing.
            // The build behind them is larger than it looks — the viewer
            // holds a single page index, and the object provider returns
            // nothing for any page but the current one.
            // ---------------------------------------------------------------
            group(
                "page_display",
                ribbon::group_view_page_display(),
                [command("view.page_single")],
            ),
            // ---------------------------------------------------------------
            // Render — see the module header. Five knobs, in the order the
            // operator meets them: what strategy, then how sharp, then how
            // long before it re-rasters, then the two correctness switches.
            // ---------------------------------------------------------------
            group(
                "render",
                ribbon::group_view_render(),
                [
                    command("view.render_strategy"),
                    command("view.render_quality"),
                    command("view.render_settle"),
                    command("view.render_thin_lines"),
                    command("view.render_antialias"),
                ],
            ),
            // ---------------------------------------------------------------
            // Zoom — the three named levels, mirrored from the status bar
            // under P1a. Zoom to selection and zoom to region are **N**.
            // ---------------------------------------------------------------
            group(
                "zoom",
                ribbon::group_view_zoom(),
                [
                    command("view.zoom_actual"),
                    command("view.zoom_fit_page"),
                    command("view.zoom_fit_width"),
                ],
            ),
            // ---------------------------------------------------------------
            // Display — what is drawn over the page. Rulers, grid and
            // guides are **N**; thin lines lives in Render (see header).
            // ---------------------------------------------------------------
            group(
                "display",
                ribbon::group_view_display(),
                [
                    command("view.show_annotations"),
                    command("view.show_points"),
                ],
            ),
            // ---------------------------------------------------------------
            // Panels.
            //
            // `Sidebar` is the rail toggle — page thumbnails and the
            // active tool's options — which is why there is no separate
            // `Pages` panel command: the thumbnails are the rail's first
            // pane, not an independently toggleable panel. `Forms` is
            // likewise not a panel toggle today; the forms surface is
            // reached from Edit ▸ Forms. Both are in PLANNED.
            // ---------------------------------------------------------------
            group(
                "panels",
                ribbon::group_view_panels(),
                [
                    command("view.sidebar"),
                    command("view.panel_bookmarks"),
                    command("view.panel_layers"),
                    command("view.panel_signatures"),
                    command("view.panel_objects"),
                ],
            ),
            // ---------------------------------------------------------------
            // Window — the shape of the application.
            //
            // Read mode and full screen are the two commands that exist
            // today with no control at all, which is the defect quoted in
            // the module header.
            //
            // The two settings after them retire `FEATURES.md`'s "nothing
            // floats over the canvas" as an absolute and replace it with a
            // pair of independent choices. The distinction between them is
            // the whole point:
            //
            //   Floating panels  Off · Allowed     default Allowed
            //     Whether the OPERATOR may tear a panel out. Off restores
            //     today's behaviour exactly.
            //
            //   App initiative   Never · Ask · Allowed   default NEVER
            //     Whether pdfce may float a surface over the canvas ON ITS
            //     OWN — tool option boxes, transient property bars,
            //     notifications.
            //
            // The second carries the original complaint (an accept/reject
            // box that appeared over the drawing and moved on every zoom),
            // and its default of Never preserves that decision's outcome
            // as the shipped behaviour while making it a choice rather
            // than a law. A panel the operator deliberately tears out is
            // not the same thing as a box the application decides to
            // float, and one setting each is what keeps them separable.
            //
            // Both are per-operator rather than per-document.
            //
            // `Save workspace…` and `Load workspace ⌄` are **N** and sit
            // between App initiative and Reset layout when they land.
            // ---------------------------------------------------------------
            group(
                "window",
                ribbon::group_view_window(),
                [
                    command("view.read_mode"),
                    command("view.fullscreen"),
                    command("view.floating_panels"),
                    command("view.app_initiative"),
                    command("view.reset_layout"),
                ],
            ),
        ])
}
