//! # `dialogs::settings::display` — how pdfce draws, as distinct from what it draws
//!
//! The eighth group, and the only one whose settings are **not** about the PDF
//! standard. Every other group in this window exists because a clause declines
//! to have an opinion; these two exist because a machine has a speed.
//!
//! ## ★ Two settings, out of seven commissioned
//!
//! `RIBBON_IA.md` §5.2 specified a View ▸ Render group of five, plus two
//! behaviour settings on the same tab, and `shell::manifest::DIRECTED` carried
//! all seven as *"named individually, with their value sets and their defaults,
//! when this shell was commissioned"* — which is a stronger statement of intent
//! than a status mark, and is why they were emitted despite carrying no `G`.
//!
//! They were registered, drawn, and inert. Checked against the engine on
//! 2026-08-17, **five of the seven have nothing behind them**:
//!
//! - **Render strategy** — there is no tiled-progressive path in this shell.
//!   `pdfce_render::render_page_region` exists, so it is buildable, but that is
//!   a rendering architecture and not a setting.
//! - **Thin lines** and **Antialiasing** — `RenderOptions` has neither field.
//!   `interpret.rs` sets `anti_alias: true` as a literal at two call sites.
//! - **Floating panels** — `egui-shell`'s dock has no floating mode.
//! - **App initiative** — *nothing in this build opens a surface unasked*. The
//!   specified default is **Never**, and it is already true by construction, so
//!   the control would exist to switch off a behaviour pdfce does not have.
//!
//! `DIRECTED`'s own doc comment anticipated this outcome and named the remedy:
//! *"if it turns out to be wrong, the fix is deleting eight rows from one list
//! rather than re-deriving which entries were deliberate."* Six rows went; the
//! two that survived became these controls and left the ribbon, because a
//! setting belongs in the settings window and `RIBBON_IA.md` §6's own list of
//! what does not go on the ribbon now has a real destination to point at.
//!
//! `crate::app::prefs`' header carries the full table with the evidence for
//! each verdict.
//!
//! ## Why these two are not in the engine's settings file
//!
//! They are **preferences**, not answers to a silent standard, and this
//! window's own opening paragraph promises the latter. They live in
//! `userdata/preferences.txt` beside `settings.txt` — same roof, same
//! fail-soft parser, different file — for the reason `crate::app::prefs`
//! states. The group sits in this window because a *window* is where an
//! operator looks for a choice, and which file a choice is stored in is not
//! their concern.

use egui::Ui;

use super::widgets;
use crate::app::prefs::{MAX_SETTLE_MS, MIN_SETTLE_MS, Prefs, RenderQuality};
use crate::text::settings as t;

/// How sharply a page is rasterised.
///
/// # Why this is a real setting on the drawings this shell is for
///
/// The benchmark sheet is 5.6 MB of dense vector site plan, and rasterising it
/// is the expensive thing this program does. The multiplier is the only control
/// an operator has over that cost, and both directions are wanted by real
/// people: someone panning a big sheet looking for a detail wants `Faster`, and
/// someone reading small text over a hairline grid wants `Sharper`.
///
/// The radius line says it affects speed as well as appearance, because that is
/// the trade being made and a control that mentioned only sharpness would be
/// describing half of itself.
pub fn render_quality(ui: &mut Ui, prefs: &mut Prefs) {
    widgets::header(
        ui,
        t::quality_title(),
        t::quality_silence(),
        t::quality_radius(),
    );
    for option in RenderQuality::ALL {
        widgets::option(
            ui,
            &mut prefs.render_quality,
            *option,
            t::quality_label(*option),
            Some(t::quality_note(*option)),
        );
    }
}

/// How long a zoom must stop changing before the page is redrawn sharply.
///
/// # A slider, and linear
///
/// Linear because the useful resolution is even: 50 ms against 150 ms matters
/// about as much as 500 against 600, since both answer *how long am I willing
/// to look at a soft page*. That is the same argument the parallel-tolerance
/// slider makes and the opposite of the word-gap one, whose useful range is all
/// at the low end.
///
/// The range is the store's own `MIN_SETTLE_MS..=MAX_SETTLE_MS`, not a local
/// pair of literals — the third instance of that rule in this window, and it
/// exists for the same reason each time: a control narrower than what the file
/// accepts silently rewrites a hand-edited value on open, and the operator
/// never touched the control.
pub fn zoom_settle(ui: &mut Ui, prefs: &mut Prefs) {
    widgets::header(
        ui,
        t::settle_title(),
        t::settle_silence(),
        t::settle_radius(),
    );
    ui.add(
        egui::Slider::new(&mut prefs.zoom_settle_ms, MIN_SETTLE_MS..=MAX_SETTLE_MS)
            .suffix(t::settle_suffix())
            .text(t::settle_slider_label()),
    );
    ui.label(egui::RichText::new(t::settle_note()).small().weak());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★ The shipped quality is the identity multiplier.
    ///
    /// The "a build that omits nothing behaves as it did before" rule, at the
    /// one place it can be checked cheaply. `viewer::raster_scale` was
    /// `zoom × pixels_per_point` exactly before this setting existed, so
    /// `Normal` must multiply by one or every raster in the application
    /// silently changed size the day the control landed.
    #[test]
    fn the_shipped_quality_changes_no_raster() {
        assert!((RenderQuality::default().multiplier() - 1.0).abs() < f32::EPSILON);
    }

    /// The three qualities are ordered less-to-more and are distinct.
    ///
    /// The control reads left to right as a scale, so a list whose middle
    /// entry was not between its neighbours would be a scale that does not
    /// scale.
    #[test]
    fn the_qualities_ascend() {
        let m: Vec<f32> = RenderQuality::ALL.iter().map(|q| q.multiplier()).collect();
        assert_eq!(m.len(), 3);
        assert!(m[0] < m[1] && m[1] < m[2], "{m:?}");
    }

    /// The shipped settle is reachable on its own slider.
    ///
    /// A default outside its control's bounds would be silently rewritten the
    /// first time anybody opened this window, on every machine, without a
    /// click. Third instance of this check in the window; third setting with a
    /// range that must be the store's.
    #[test]
    fn the_shipped_settle_is_reachable_on_the_slider() {
        let ms = Prefs::default().zoom_settle_ms;
        assert!(
            (MIN_SETTLE_MS..=MAX_SETTLE_MS).contains(&ms),
            "the shipped settle {ms} is outside the slider's range"
        );
    }
}
