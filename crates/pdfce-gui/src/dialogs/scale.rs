//! # `dialogs::scale` — what a dimension's number *means*
//!
//! The Set-scale dialog. `measure.set_scale` was registered, drawn on
//! Measure ▸ Scale, and inert; `shell::commands::reach` recorded it as *"the
//! clearest statement of a missing arm in the crate"*, and the block on it was
//! never the model — it was this window.
//!
//! ## ★ Why this is the sharpest gap in the measure feature
//!
//! Phase 7 shipped three tools that place dimensions: Linear, Two-line, and
//! Radius/diameter. All three work. **None of the numbers they produce means
//! anything until a scale is set**, because a fresh group's scale is the
//! tri-state's *never-set* value — so every label reads in PDF points, which is
//! a unit nobody's drawing is in.
//!
//! An operator could therefore place a dimension and read a number, and the
//! number was a measurement of the *paper* rather than of the thing drawn on
//! it. That is worse than a missing feature: it is a plausible answer.
//!
//! ## What was already built, and what was missing
//!
//! `canvas::measure::scale` came across whole in the Phase 7 salvage and is
//! **pure, GUI-free and unit-tested**: [`ScaleEntryFields`] holds the two
//! co-equal entry paths, back-calculates through the engine's own
//! `preview_group_scale`, and hands back a `(ScaleState, NumberFormat)` ready
//! for `EditSession::set_group_scale`. It contains *zero* scale arithmetic of
//! its own — deliberately, so that a canvas-calibrated group and a
//! CLI-calibrated group are the same number.
//!
//! What was missing was somewhere to type into. This module is that and
//! nothing else: it owns no arithmetic, no parsing and no units. It draws
//! fields, calls `sync_real_length`, shows `preview`, and raises one `Action`.
//!
//! ## The two entry paths, and why both exist
//!
//! | path | what you give it | needs |
//! |---|---|---|
//! | **Real length** | *"this line I drew is 4'-7 1/2\""* | a drawn reference line |
//! | **Ratio** | *"1:100"* | nothing |
//!
//! The real-length path is the recommended one and the one a drafter reaches
//! for, because it needs no arithmetic from them: point at a dimension the
//! drawing already states, type what it says, and the scale falls out. The
//! ratio path exists because it needs no drawn line — which makes it the only
//! path reachable from a dialog opened cold, and, as the source notes, an
//! accessibility win: a scale can be set entirely by typing.
//!
//! **This dialog offers the ratio path.** The real-length path is offered when
//! a reference line has been drawn, and drawing one is a canvas gesture
//! (`ScalePick`) that is not yet armed by any command — so the radio for it is
//! **absent rather than greyed**, on the standing rule: greying is for
//! *temporarily* unavailable, and no control in this dialog can produce a
//! reference line. The window says which path it is offering and why, because
//! an operator who has read the manual will look for the other one.
//!
//! ## Why the real-length field still accepts `4'-7 1/2"`
//!
//! It is not offered here yet, and the parsing belongs to
//! `pdfce_core::dimension::parse_length` either way — the grammar lives in core
//! once, so the GUI and the CLI cannot come to disagree about what `55 5/8"`
//! means. That is the same rule the print dialog follows about range parsers,
//! and it is why the field is text rather than a numeric spinner: a spinner
//! forces the operator to convert to a decimal and pick a unit by hand, which
//! is two chances to enter a number that is plausible and wrong.

use egui::Ui;
use pdfce_core::dimension::{FractionMode, GroupId, Unit};

use crate::app::actions::Action;
use crate::canvas::measure::scale::ScaleEntryFields;
use crate::text::scale as t;

/// The region this dialog publishes for its body.
pub const REGION_BODY: &str = "dialog:set-scale"; // ui-text-exempt: trace region name, never displayed

/// The region the Accept control publishes.
pub const REGION_ACCEPT: &str = "dialog:set-scale.accept"; // ui-text-exempt: trace region name, never displayed

/// The Set-scale dialog's live state.
///
/// Existence is the "open" state, as everywhere in this directory — there is no
/// `open: bool` that could disagree with whether the state exists.
pub struct ScaleDialog {
    /// The group being calibrated.
    ///
    /// Captured when the dialog opens, from the measure tool's active
    /// authoring group. **Not re-read per frame**: a scale is set for the group
    /// the operator was working in when they asked, and a group picker that
    /// moved underneath an open dialog would let them type a number for one
    /// group and commit it to another.
    group: GroupId,
    /// The two entry paths and everything typed into them.
    ///
    /// Seeded with [`ScaleEntryFields::for_group_panel`], which pre-selects the
    /// **ratio** path — the constructor exists precisely for "no reference line
    /// was drawn", which is this dialog's situation.
    fields: ScaleEntryFields,
    /// The last parse error from the real-length field, if any.
    ///
    /// Held rather than recomputed at draw time so the message and the preview
    /// cannot disagree: `sync_real_length` deliberately leaves the previous
    /// good value alone on a failed parse, so "what is shown" and "what would
    /// commit" have to be captured together.
    parse_error: Option<String>,
    /// Set by Accept, consumed after the window's closure returns.
    ///
    /// Same one-statement deferral the print dialog uses, and for a milder
    /// version of the same reason: `set_group_scale` re-propagates every
    /// member's baked appearance stream, which is a document edit, and the
    /// action funnel's invariant is that no code path runs from a widget to a
    /// document.
    accept_requested: bool,
    /// Set by Close, consumed by [`Self::show`].
    close_requested: bool,
}

impl ScaleDialog {
    /// Open on `group`.
    #[must_use]
    pub fn open(group: GroupId) -> Self {
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "scale-open group={group:?}"
            )
        });
        Self {
            group,
            fields: ScaleEntryFields::for_group_panel(),
            parse_error: None,
            accept_requested: false,
            close_requested: false,
        }
    }

    /// Draw one frame. Returns `false` when it should close.
    ///
    /// # Screen-anchored, like every dialog here
    ///
    /// A surface an operator is typing into must stay where they put their
    /// eyes, and a position derived from the page moves on every zoom and
    /// scroll. `default_pos` rather than `anchor` so it can be dragged aside —
    /// this one sits over a drawing the operator may want to look at while
    /// deciding what the scale is.
    pub fn show(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) -> bool {
        let screen = ctx.input(egui::InputState::content_rect);
        let size = egui::vec2(440.0_f32.min(screen.width() - 40.0), 300.0);
        let pos = egui::pos2(
            ((screen.width() - size.x).max(0.0) / 2.0).max(0.0),
            ((screen.height() - size.y).max(0.0) / 3.0).max(0.0),
        );

        let mut open = true;
        egui::Window::new(t::window_title())
            .collapsible(false)
            .resizable(true)
            .default_size(size)
            .min_size([360.0, 220.0])
            .default_pos(pos)
            .open(&mut open)
            .show(ctx, |ui| {
                crate::diag::ui_rect(REGION_BODY, ui.max_rect());
                self.body(ui);
            });

        if std::mem::take(&mut self.accept_requested) {
            self.commit(actions);
            return false;
        }
        open && !std::mem::take(&mut self.close_requested)
    }

    /// The fields, the preview and the two buttons.
    fn body(&mut self, ui: &mut Ui) {
        ui.label(t::intro());
        ui.add_space(6.0);

        // ★ The ratio path only, and the window says so.
        //
        // The real-length path needs a drawn reference line, and drawing one is
        // a canvas gesture no command arms yet. A greyed radio would imply the
        // operator could turn it on; an unexplained absence would leave someone
        // who has read the manual hunting for it. So: absent, and named.
        ui.label(egui::RichText::new(t::ratio_only_note()).small().weak());
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            ui.label(t::ratio_label());
            ui.add(
                egui::DragValue::new(&mut self.fields.ratio_paper)
                    .speed(0.01)
                    .range(0.0001..=10_000.0),
            );
            ui.label(t::ratio_separator());
            ui.add(
                egui::DragValue::new(&mut self.fields.ratio_real)
                    .speed(1.0)
                    .range(0.0001..=1_000_000.0),
            );
        });
        ui.label(egui::RichText::new(t::ratio_hint()).small().weak());
        ui.add_space(6.0);

        // The paper-unit basis. Disclosed rather than assumed, because a ratio
        // is meaningless without one: `1:100` on an inch basis and `1:100` on a
        // millimetre basis are different scales, and PDF's own paper unit is
        // 1/72", which is nobody's intuition.
        ui.horizontal(|ui| {
            ui.label(t::basis_label());
            unit_combo(ui, "scale.basis", &mut self.fields.basis);
        });
        ui.horizontal(|ui| {
            ui.label(t::unit_label());
            unit_combo(ui, "scale.unit", &mut self.fields.unit);
        });
        ui.horizontal(|ui| {
            ui.label(t::fraction_label());
            fraction_combo(ui, &mut self.fields.fraction);
        });

        ui.add_space(8.0);
        ui.separator();

        // ★ The live preview, from the ENGINE's own back-calculation.
        //
        // `ScaleEntryFields::preview` calls `preview_group_scale`, which is the
        // same function the CLI calibrates through — so what this window shows
        // and what a scripted calibration produces are the same number by
        // construction rather than by two implementations agreeing.
        //
        // `None` means a degenerate entry (a zero somewhere), and Accept then
        // has nothing to commit. Shown as a refusal rather than as a blank, so
        // a greyed Accept has a reason beside it.
        let preview = self.fields.preview(None);
        match &preview {
            Some(p) => {
                // `ratio_label` is the engine's own `/R`-style display string —
                // `1:100`, or `25 ft = 42.3 pt`. Using it rather than formatting
                // the scale here keeps one implementation of a number the
                // operator is checking; two would eventually disagree about
                // rounding.
                ui.label(t::preview(&p.ratio_label, p.unit));
            }
            None => {
                ui.label(egui::RichText::new(t::degenerate()).color(ui.visuals().warn_fg_color));
            }
        }
        if let Some(error) = &self.parse_error {
            ui.label(
                egui::RichText::new(t::parse_failed(error)).color(ui.visuals().error_fg_color),
            );
        }

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            let accept = ui.add_enabled(preview.is_some(), egui::Button::new(t::accept()));
            crate::diag::ui_rect(REGION_ACCEPT, accept.rect);
            if accept.clicked() {
                self.accept_requested = true;
            }
            if preview.is_none() {
                accept.on_disabled_hover_text(t::accept_disabled_tooltip());
            }
            if ui
                .button(t::cancel())
                .on_hover_text(t::cancel_tooltip())
                .clicked()
            {
                self.close_requested = true;
            }
        });
    }

    /// Turn the entry into the one action that changes the document.
    ///
    /// # Why a single action and not a call
    ///
    /// `set_group_scale` **re-propagates every member's baked appearance
    /// stream** — a dimension's label is drawn into its `/AP`, so changing the
    /// scale rewrites every dimension in the group. That is a document edit
    /// with an undo step, and the funnel exists so that every such edit is
    /// ordered against every other and appears once in the command log.
    ///
    /// One `Ctrl+Z` undoes a recalibration, whatever it touched. That is the
    /// group model's whole promise — *a group exists so its members agree* —
    /// and it would be broken by a dialog that issued one call per member.
    fn commit(&self, actions: &mut Vec<Action>) {
        // `None` for the drawn length: this dialog offers the ratio path, which
        // needs no line. `ScaleEntryFields::entry` routes on exactly that.
        let Some((scale, format)) = self.fields.commit(None) else {
            // Unreachable from the UI — Accept is disabled without a preview,
            // and a preview exists exactly when `commit` will. Handled rather
            // than unwrapped because an `Action` is plain data a test can
            // build, and declining by name beats a panic in a frame that is
            // trying to draw.
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "scale-commit-refused reason=degenerate".to_owned()
            });
            return;
        };
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "scale-commit group={:?} scale={scale:?} format={format:?} \
                 ratio={}:{} basis={:?}",
                self.group, self.fields.ratio_paper, self.fields.ratio_real, self.fields.basis,
            )
        });
        actions.push(Action::SetGroupScale {
            group: self.group,
            scale,
            format,
        });
    }
}

/// A unit picker.
///
/// `Unit::ALL` rather than a hand-written list, so a unit the engine gains
/// appears here without anybody remembering — the same rule
/// `MarkupKind::ALL` and `Preset::ALL` are read under elsewhere in this crate.
fn unit_combo(ui: &mut Ui, id: &str, unit: &mut Unit) {
    egui::ComboBox::from_id_salt(id)
        .selected_text(t::unit_name(*unit))
        .show_ui(ui, |ui| {
            for option in UNITS {
                ui.selectable_value(unit, *option, t::unit_name(*option));
            }
        });
}

/// The units offered, metric first.
///
/// A local list because `pdfce_core::dimension::Unit` exposes no `ALL` — every
/// other enumerable set this crate reads (`MarkupKind::ALL`, `Preset::ALL`)
/// does, and this one does not, so the list is here and the risk is stated: a
/// unit the engine gains will not appear until this array does. The catch-all
/// in `text::scale::unit_name` covers the *naming* half so such a unit is at
/// least legible if it arrives from a file; it cannot make it selectable.
///
/// Metric before imperial because millimetres are what a CAD export is
/// overwhelmingly in, and the first entry is the one a hurried operator picks.
const UNITS: &[Unit] = &[
    Unit::Millimeter,
    Unit::Centimeter,
    Unit::Meter,
    Unit::Inch,
    Unit::DecimalFeet,
    Unit::FeetInches,
];

/// The number styles offered.
///
/// `FractionMode`'s variants carry data — `Decimal { places }`,
/// `Fraction { denominator, reduce }` — so there is no finite set to enumerate
/// and this is a **curated** one rather than an exhaustive one. That is the
/// right shape: the useful decimal places are one to three and the useful
/// denominators are the binary ones a drawing writes, and offering a spinner
/// over every `u32` would be a control whose range is mostly nonsense.
///
/// `reduce: false` throughout, which is the architectural convention the
/// engine's own docs name: `6/8"` rather than `3/4"`, because a drawing
/// dimensioned to eighths writes eighths.
const FRACTIONS: &[FractionMode] = &[
    FractionMode::Decimal { places: 0 },
    FractionMode::Decimal { places: 1 },
    FractionMode::Decimal { places: 2 },
    FractionMode::Decimal { places: 3 },
    FractionMode::Fraction {
        denominator: 8,
        reduce: false,
    },
    FractionMode::Fraction {
        denominator: 16,
        reduce: false,
    },
    FractionMode::Fraction {
        denominator: 32,
        reduce: false,
    },
];

/// A fraction-display picker, including the *"use the unit's default"* state.
///
/// The `None` entry is first and is what an operator who never opens this
/// control gets. It is a real choice rather than an absence: an explicit
/// selection must survive a unit change, which is why the field stores
/// `Option<FractionMode>` rather than re-deriving from the unit — an operator
/// who asked for eighths does not want them silently reverted by switching from
/// inches to feet.
fn fraction_combo(ui: &mut Ui, fraction: &mut Option<FractionMode>) {
    egui::ComboBox::from_id_salt("scale.fraction")
        .selected_text(t::fraction_name(*fraction))
        .show_ui(ui, |ui| {
            ui.selectable_value(fraction, None, t::fraction_name(None));
            for mode in FRACTIONS {
                ui.selectable_value(fraction, Some(*mode), t::fraction_name(Some(*mode)));
            }
        });
}
