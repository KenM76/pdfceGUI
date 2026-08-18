//! # `app::frame` — the per-frame update, in the one order it may happen in
//!
//! `eframe`'s entry point and nothing else. Split out of [`crate::app`] on
//! 2026-08-17, when that file crossed rule R2's 1,500-line ceiling.
//!
//! ## The seam is a real one, not a line count
//!
//! `app/mod.rs` answers *"what is the application, and how is it built?"* —
//! the state, its fields, its one constructor, and the two surfaces
//! (`ribbon_band`, `docks`) that are pure layout. This file answers *"what
//! happens, in what order, sixty times a second?"*, which is the question with
//! the ordering constraints in it.
//!
//! Those constraints are the reason the split is worth making rather than
//! merely necessary. Almost every comment in [`PdfceApp::ui`] is about
//! **sequence** — the theme before any widget, the keyboard before any widget
//! can consume a key, the dialogs after the docks so they are painted over
//! rather than under, the zoom anchor after the commands that raise one, the
//! rasterize last so it measures a settled frame. A reader auditing that order
//! now has it in one file with nothing else in it, which is the condition
//! under which an ordering bug is visible at all.
//!
//! The old shell is the argument: two independent regressions of the same key
//! landed two days apart in a 25,005-line `main.rs`, and neither noticed the
//! other.

use eframe::egui;

use super::actions::Action;
use super::state::Status;
use super::{PdfceApp, REGION_CENTRAL_PANEL, keyboard, modes, window};

impl eframe::App for PdfceApp {
    /// eframe 0.35's entry point is `ui`, **not** `update`.
    ///
    /// The trait hands a root [`egui::Ui`] rather than a [`egui::Context`]
    /// (`eframe-0.35.0/src/epi.rs:176`), and panels are added *inside* that
    /// `Ui` — `CentralPanel::show(ui, …)`, not `show(ctx, …)`. Anyone
    /// arriving from an older eframe, or from a code sample, will write
    /// `update` and get a "not a member of trait" error whose message does
    /// not say what to write instead; hence this note.
    ///
    /// The context is cloned out at the top because the raster bookkeeping
    /// needs it after the panel closure has ended, and `Ui::ctx()` borrows.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // ★ Step 0 — install the theme. See `DEFECTS.md` D10.
        //
        // **This call did not exist until 2026-08-14**, and the whole theme
        // subsystem — three presets, a palette, a role per colour, a
        // rendered-pair contrast gate over five widget states, and a gate
        // self-test — was compiled into the binary and never handed to the
        // `Context`. Every colour an operator has ever seen in this shell was
        // `egui`'s stock light style. Found by a `ui-verify` check sampling a
        // pressed ribbon button and getting `egui`'s `selection.bg_fill`
        // instead of the preset's.
        //
        // Two things are installed, and the second is the one whose absence
        // was invisible: `apply` writes the palette into **both** of egui's
        // light and dark `Style`s, and it stashes the whole `Theme` in
        // `ctx.data` where `Theme::of` retrieves it. `egui-shell`'s ribbon,
        // dock and splitter all call `Theme::of` for roles that have nowhere
        // to live in an `egui::Style` — the content backdrop, the label
        // plate. Without the stash they silently got the DEFAULT theme, so
        // the framework's chrome and egui's widgets painted from two
        // different palettes. `apply`'s own doc comment names that failure
        // and calls it the thing the module exists to prevent.
        //
        // **Per frame, not once at startup**, which is what that doc
        // prescribes: a theme change then takes effect immediately, with no
        // restart and no cache to invalidate. It is a handful of field writes
        // against a struct egui already owns.
        //
        // ★ The preset comes from the operator's settings — 2026-08-17, and
        // this is the second half of `DEFECTS.md` D10.
        //
        // The first half was fixed on 2026-08-14 by calling `apply` at all.
        // What that note said next, and what stayed true until now:
        //
        // > There is also **no way to choose a preset**: the settings dialog is
        // > one of the unsalvaged Class-B surfaces, so even once `apply` is
        // > wired, the preset is whatever the code picks until that dialog
        // > lands.
        //
        // # ★ The DRAFT wins over the live settings, and only for the theme
        //
        // Every other setting in that window is draft-until-Save. A theme
        // cannot be judged from a radio label — you choose it by *seeing* it —
        // so while the window is open the draft's token is the one installed.
        // The draft still governs what is SAVED; it just no longer governs
        // what is SHOWN, and the window's own radius line says so.
        //
        // Cancel drops the draft, so the look reverts with it. That is why
        // this is a two-line lookup rather than a separate "preview theme"
        // field with its own lifecycle: there is nothing to undo and nothing
        // that can get out of step with what will be written.
        //
        // # `unwrap_or_default`, and what it is covering
        //
        // A token this build does not recognise — from a settings file written
        // by a NEWER pdfce — falls back to the default preset and the token is
        // **kept, not overwritten**. The window says so, quoting the name. The
        // alternative, silently rewriting it to `quiet` on the next save, would
        // destroy a setting the operator made in a different version of the
        // program they also run from the same folder.
        let theme_token = self
            .settings_draft
            .as_ref()
            .map_or(self.settings.theme.as_str(), |draft| {
                draft.working.theme.as_str()
            });
        let preset = egui_shell::theme::Preset::from_key(theme_token).unwrap_or_default();
        egui_shell::theme::Theme::new(preset).apply(&ctx);

        // ★ Step 0b — install the UI scale. The theme's twin, added 2026-08-17.
        //
        // # Why it is here and not in `configure_context`
        //
        // Same reason the theme is: the operator can change it, so a one-time
        // call at start-up would mean a restart to see the effect.
        //
        // # ★ Why the epsilon guard, given that egui already guards
        //
        // `Context::set_zoom_factor` (`context.rs:2269-2280` in 0.35) does test
        // before acting — but on **exact float equality**. A bit-identical
        // `f32` handed straight back is absorbed and costs nothing, so this
        // guard is not covering a naive upstream.
        //
        // What `!=` misses is every *derived* value, which is what a
        // continuous, operator-settable quantity actually produces: a
        // percentage that has been formatted and re-parsed through the
        // preferences file, a slider mid-drag, anything that has been through
        // `normalise_ui_scale`. Those land a hair off the stored value, egui's
        // equality test sees a change, and the cost is real — a trip requests a
        // repaint on **every viewport** and re-derives `screen_rect` on the
        // next pass (`context.rs:431-443`). An epsilon is the right *kind* of
        // guard for a quantity with no exact representation; exact equality is
        // the right kind for a token.
        //
        // Note also that `zoom_factor()` does not reflect a set until the pass
        // **ends** (`context.rs:2258`), so this read returns what the previous
        // frame settled on — which is exactly what "has it changed since last
        // frame?" wants, and is why a set-then-read-back within one frame would
        // prove nothing.
        //
        // # ★ The draft wins, exactly as it does for the theme
        //
        // These two are the only settings in the window that take effect
        // before Save, and the argument is identical in both cases: **you
        // cannot judge either from a label.** A theme is chosen by seeing it;
        // a scale is chosen by seeing whether you can read the ribbon at it.
        // The draft still governs what is SAVED — Cancel drops it and the size
        // reverts with it, with no separate preview state to get out of step.
        //
        // The settings window's own radius line says so, so this is disclosed
        // rather than merely true.
        //
        // # What it does NOT do, and why that is right
        //
        // It does not touch the page. `set_zoom_factor` moves
        // `ctx.pixels_per_point`, which `viewer::raster_scale` already reads —
        // so the canvas re-rasterises at the new device density and the
        // document stays exactly the same size **relative to the window**. A
        // bigger UI genuinely does mean a smaller visible page, because the
        // ribbon and the panels take more of the window; that is the honest
        // consequence of the setting and not something to compensate for.
        //
        // The rasters keyed on `pixels_per_point` — the page texture, the
        // strip, the Pages thumbnails — invalidate through their own existing
        // keys, so nothing here has to know about them.
        let ui_scale = self
            .settings_draft
            .as_ref()
            .map_or(self.prefs.ui_scale, |draft| draft.working_prefs.ui_scale);
        // A tenth of a step: finer than any change an operator can make with
        // the control, coarse enough that float noise never trips the setter.
        if (ctx.zoom_factor() - ui_scale).abs() > crate::app::prefs::UI_SCALE_STEP / 10.0 {
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "ui-scale from={:.2} to={ui_scale:.2} ppp={:.3}",
                    ctx.zoom_factor(),
                    ctx.pixels_per_point(),
                )
            });
            ctx.set_zoom_factor(ui_scale);
        }

        // Step 1 — keyboard, before any widget can consume a key.
        let page_count = match &self.status {
            Status::Open(doc) => Some(doc.pages.len()),
            _ => None,
        };
        let mut actions = keyboard::collect(&ctx, page_count);

        // Step 1a — the chords the MANIFEST binds.
        //
        // ★ This is the second half of the one-owner-per-chord fix. The
        // keymap is data — `egui-shell` deliberately does not dispatch it,
        // because "the application owns the question of what has focus and
        // what a chord means" — so until now every binding in it was a
        // documented promise with nothing behind it, and `keyboard::collect`
        // quietly bound two of the same chords to something else.
        //
        // `keyboard::commands` returns command *ids*, which go through the
        // same dispatcher a ribbon click does. That is what makes a chord and
        // its button incapable of disagreeing.
        //
        // Owned rather than borrowed (`Vec<String>`) because dispatching
        // needs `&mut self` and the keymap lives in `self.shell`. It is empty
        // on all but the handful of frames where a chord was actually
        // pressed.
        //
        // ★ **Filtered by the active mode**, which is the keymap's share of
        // the mode gate. Operator decision, 2026-08-14.
        //
        // The ribbon hides a tab and the canvas asks `Capabilities`; between
        // them sat this, dispatching by id and consulting neither — so Read
        // hid the Edit tab and `Ctrl+E` still reached `edit.text`. The rule
        // lives in `modes::capability::offers_command`, beside the other
        // statement of what a mode permits, rather than here: this is the
        // choke point that *applies* it, and a second copy of the rule at the
        // point of application is how the two come to disagree.
        //
        // Filtered rather than refused inside `dispatch_command`, because a
        // command the mode does not offer is not a command that *failed* —
        // there is nothing to report and nothing to trace as declined. The
        // chord simply is not bound in this mode, which is what the operator
        // sees: no tab, no button, no effect.
        let chord_commands =
            keyboard::commands(&ctx, self.shell.as_ref().and_then(|s| s.keymap.as_ref()));
        let mode = self.ribbon.mode().map(str::to_owned);
        for id in chord_commands {
            if !modes::capability::offers_command(self.shell.as_ref(), mode.as_deref(), &id) {
                crate::diag::trace(|| {
                    format!(
                        // ui-text-exempt: diagnostic trace, never displayed.
                        "chord-not-offered id={id} mode={}",
                        mode.as_deref().unwrap_or("-")
                    )
                });
                continue;
            }
            self.dispatch_command(&ctx, &id, &mut actions);
        }

        // Step 1b — the ribbon, above the canvas.
        //
        // Added before the `CentralPanel` because panel composition order is
        // load-bearing for both geometry and Tab focus: a panel added later
        // carves its space from what is left, so the canvas must be last or
        // it takes the whole window and the ribbon draws over it.
        //
        // The shell executes nothing. `show` returns the handler tokens the
        // operator invoked this frame, and they are translated into
        // `Action`s here — the same one-choke-point discipline every other
        // surface follows. A token with no arm yet is not an error; at S2
        // most of the ribbon is scaffolding for behaviour that lands later,
        // and `dispatch_token` says so per token rather than silently.
        // ★ Read mode's whole effect is here: the ribbon and the docks are not
        // added to the frame. Why it is not `mode.read`, why the status bar
        // below stays and how the operator gets back out are all in [`window`];
        // a composition step deciding any of that would be a second rule.
        let chrome = window::draws_chrome(&ctx);
        if chrome {
            self.ribbon_band(ui, &mut actions);
        }

        // Step 1b² — the status bar, before the docks.
        //
        // **Order, not preference.** This module's own header states the rule
        // the old shell was bitten by: *"a full-width bar must be added
        // before any side panel, or it starts at the side panel's edge
        // instead of spanning the window."* A status bar that stops at the
        // dock is not a status bar.
        //
        // `exact_size`, never `default_height`: a content-driven status
        // height and a per-frame fit-to-viewport zoom form a measured
        // feedback loop — 230 % → 224 % → 215 % drift from a status line
        // that grew (R128, `D:\dev\rag\egui\bottom_panel_height_...md`).
        egui::Panel::bottom("status")
            .exact_size(crate::app::status::HEIGHT_PTS)
            .show(ui, |ui| {
                // Two disjoint field borrows through `self`, as at the canvas
                // call site below: the bar reads the status and writes the
                // Find toggle's own state.
                crate::app::status::show(ui, &self.status, &mut self.find, &mut actions);
            });

        // Step 1c — the docks, between the ribbon and the canvas.
        //
        // Order is load-bearing twice over. The ribbon is a full-width bar
        // and must be added *before* any side panel, or it would start at
        // the dock's edge instead of spanning the window. The canvas must
        // be added *after* both, because a `CentralPanel` takes whatever is
        // left and there must be something left for it to take.
        if chrome {
            self.docks(ui, &mut actions);
        }

        // Step 1c² — the debounced workspace write, dock drawn or not. ★ Moved
        // out of `Self::docks` when read mode landed; [`window`] §3 has why the
        // debounce belongs to the frame and what quitting from read mode would
        // otherwise lose.
        if let Some(after) = self.layout.tick(std::time::Instant::now()) {
            ctx.request_repaint_after(after);
        }

        // Step 2 — compose. Nothing here mutates a document; surfaces push
        // onto `actions`.
        egui::CentralPanel::default().show(ui, |ui| {
            // Declare the panel's own rect before drawing into it. This is
            // the outermost named region the application owns, and it is the
            // one a screenshot oracle uses to tell "the control is drawn but
            // clipped out of its pane" from "the control is not drawn" —
            // `PROJECT_PLAN.md` §4.2 prerequisite 2 records two cases where a
            // traced rect was correct and the control was still clipped.
            crate::diag::ui_rect(REGION_CENTRAL_PANEL, ui.max_rect());
            self.central(ui, &mut actions);
        });

        // Step 2a² — the FIND OVERLAY, over the page.
        //
        // ★ After the canvas, and the order IS the placement. The box is an
        // `egui::Area` positioned from the CANVAS VIEWPORT's rect, which
        // `canvas::show` records through `zoom::remember_frame` as the last
        // thing it does — so drawing it before the canvas would position this
        // frame's box from last frame's layout, visible as a one-frame lag
        // every time a dock splitter is dragged.
        //
        // Before the dialogs, because a modal takes the frame and must be over
        // everything, this included.
        //
        // It draws nothing when the bar is closed and nothing when no document
        // is open, so on the overwhelming majority of frames this line costs
        // one boolean. Two disjoint field borrows through `self`, as at the
        // canvas call site: `&mut self.find` and `&self.status`.
        crate::find::bar::show(ui, &mut self.find, &self.status, &mut actions);

        // Step 2a³ — drain any `Action::Command` raised by a surface that is
        // not the ribbon, and route it through the one dispatch choke point.
        //
        // ★ Here, and not in the apply phase, and the position is the design.
        //
        // The Find bar's OCR offer is the first control outside the ribbon that
        // means an existing *command* rather than a document change. Wiring it
        // straight to `DialogsState::open_ocr` would have been one line and
        // would have put `file.ocr`'s guards in two places — the failure this
        // module's "one choke point for dispatch" invariant exists to prevent.
        //
        // It has to run **now** rather than at step 3 for two reasons, both
        // hard rather than stylistic: `dispatch_command` needs an
        // `&egui::Context` and the apply phase is deliberately given none, and
        // a dialog opened by the dispatch must be drawn by `DialogsState::show`
        // three lines below — on this frame, not the next one.
        //
        // The drain is unconditional and cheap: on the overwhelming majority of
        // frames `actions` is empty and this is one `iter().any`. Dispatched
        // commands may themselves raise actions, which is why the loop pushes
        // into the same vector the apply phase will read.
        if actions.iter().any(|a| matches!(a, Action::Command(_))) {
            let mut invoked: Vec<String> = Vec::new();
            actions.retain(|a| match a {
                Action::Command(id) => {
                    invoked.push(id.clone());
                    false
                }
                _ => true,
            });
            for id in invoked {
                self.dispatch_command(&ctx, &id, &mut actions);
            }
        }

        // Step 2b — modal dialogs, LAST among the surfaces.
        //
        // After the canvas and the docks, because egui draws in call order
        // and a dialog shown before them would be painted under the very
        // content it is modal over. It takes `&self.status` so it can close
        // itself when the document does — a print dialog outliving its
        // document would offer to print pages that are gone.
        self.dialogs.show(&ctx, &self.status, &mut actions);

        // ★ The calibration round trip: dialog -> canvas gesture -> dialog.
        //
        // Two edges, read once per frame, in this order.
        //
        // FIRST, "the operator pressed *Measure it on the drawing*": close the
        // window so it is not over the page they are about to click, and arm
        // the two-point pick. Read-and-clear, so a request cannot re-arm on
        // every subsequent frame.
        //
        // SECOND, "the pick just completed": put the window back with the
        // measured length in it, and disarm, so the next click is an ordinary
        // one rather than the start of another reference line.
        //
        // Here rather than in `dispatch` because neither edge is a command —
        // one is a button inside a dialog and the other is the canvas noticing
        // its own state machine finished. Both are frame-level observations,
        // which is what this function is for.
        if self.dialogs.take_scale_calibrate_request() {
            self.dialogs.close_scale();
            crate::canvas::tool::select(
                &ctx,
                crate::canvas::tool::CanvasTool::Measure(
                    crate::canvas::measure::MeasureKind::Scale,
                ),
            );
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "scale-calibrate armed=true".to_owned()
            });
        }
        // ★ Both halves must be present, and the group is the one that can be
        // absent. `active_group` answers `None` when no measure state exists —
        // which cannot happen on the frame a pick completes, since completing
        // one requires the state. Handled rather than unwrapped anyway: an
        // `expect` here would turn an impossible ordering into a crash in the
        // one gesture whose whole output is a number the operator is trusting.
        if let Some(measured) = crate::canvas::measure::take_completed_scale_line(&ctx)
            && let Some(group) = crate::canvas::measure::active_group(&ctx)
        {
            self.dialogs
                .open_scale_calibrated(&self.status, group, measured);
            crate::canvas::tool::select(&ctx, crate::canvas::tool::CanvasTool::Select);
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "scale-calibrate measured_pt={measured:.3}"
                )
            });
        }

        // Step 2b(ii) — the Settings window.
        //
        // Drawn beside the other dialogs but held separately, because its draft
        // has to be readable at the TOP of the frame where the theme is
        // installed. See `crate::dialogs::settings`' header.
        self.settings_window(&ctx);

        // Step 2c — give every pending zoom an anchor, in ONE place.
        //
        // `ZoomIn`, `ZoomOut` and `ZoomTo` are raised from five call sites —
        // `view.zoom_actual` in the dispatcher, the keyboard, and three
        // status-bar controls. Anchoring them where they are raised would
        // mean the same rule spelled six times, and a seventh surface added
        // later would silently zoom to the top-left corner: the exact defect
        // this closes, which is why it is one statement here rather than six
        // there.
        //
        // The rule it applies lives in `canvas::zoom::anchor_point` and
        // nowhere else: **a zoom holds one page point still, and that point
        // is where the operator is looking** — the pointer when it is over
        // the canvas, the viewport's centre when it is not.
        //
        // It skips an action whose anchor is already armed, so the framing
        // verbs (fit, zoom-to-selection, region zoom) and the Ctrl+wheel keep
        // the anchors they set deliberately.
        if let Status::Open(doc) = &mut self.status {
            crate::canvas::zoom::arm_for_actions(&ctx, doc, &actions);
        }

        // Step 3 — apply, after the frame is drawn.
        let pixels_per_point = ctx.pixels_per_point();
        self.apply_actions(actions, pixels_per_point);

        // Step 4 — decide whether the picture on screen still matches the
        // state that was just updated, and start a render if not.
        self.settle_and_rasterize(&ctx, pixels_per_point);

        // ★ LAST — close the frame's region census.
        //
        // Every `diag::ui_rect` call for this frame has happened by now, so
        // this is the first moment at which "which regions were NOT drawn this
        // frame?" has an answer. It emits `ui-rect-gone` for each, and
        // `crate::diag::end_ui_frame` carries the argument for why a trace
        // that only reports appearances is not merely incomplete but
        // actively misleading.
        //
        // After `settle_and_rasterize` rather than before it, because that
        // call can still declare regions — and a census closed one line early
        // would retire whatever it was about to draw, every frame, forever.
        crate::diag::end_ui_frame();
    }
}
