//! # `dialogs::settings` — where a spec ambiguity becomes a choice
//!
//! Ported from `D:\Dev\pdfce\crates\pdfce-gui\src\settings_panel.rs` (914
//! lines). `SALVAGE.md` classes it Class B — *"salvaged but unbuilt"* — and
//! `shell::commands::reach` recorded `file.settings` as blocked on exactly
//! this module.
//!
//! ## What this surface is for
//!
//! The standing rule behind it, an operator directive of 2026-08-08:
//!
//! > Where standards are ambiguous those should become settings that the user
//! > can choose direction one, with the initial installed default as the best
//! > guess of what is usually followed.
//!
//! `pdfce_core::settings` is the store that makes such a choice survive a
//! restart. This module is the only place an operator can *make* one without
//! opening a text editor — and the no-affordance-without-capability rule cuts
//! both ways here: **a setting the program honours but offers nowhere is just
//! as much a gap as a control that does nothing.**
//!
//! That is the whole reason this shipped late and the reason it had to ship: the
//! engine has honoured thirteen of these since before this project began, and
//! the operator's report was that features had been added with *"no surface for
//! changing or editing the settings for them."*
//!
//! ## Why a window and not a dock panel
//!
//! The dock's own test is **"selection state is watched, workflows are
//! entered."** Watched things — the page rail, the object tree, the armed
//! tool's options — earn a permanent compartment because they are consulted
//! continuously *while doing something else*. Settings are the opposite:
//! consulted in bursts, deliberately, when there is something to change. The
//! old shell moved Properties out of the dock on 2026-08-06 for precisely this
//! reason, after the operator used it and found the pairing wrong; putting
//! Settings *in* would be the same mistake with a different noun.
//!
//! So it is a window, opened from the File tab's pdfce group, in the shape an
//! operator arriving from Office or PDF-XChange already expects of *File →
//! Options*.
//!
//! ## ★ Seven groups, not six — and one of them is new
//!
//! Settings are collapsed into subject groups, and this is navigation rather
//! than tidiness: **an operator opens this window with a *symptom*** — "my
//! black lines look grey", "copied text has no spaces", "my dimension came out
//! as an angle" — and the group headings are how a symptom finds its setting. A
//! flat list of thirteen makes the reader scan every one.
//!
//! Which means a setting filed under the wrong heading is not untidy, it is
//! **unreachable**. The source had one: `parallel_epsilon_degrees`, which
//! governs whether two lines are dimensioned as a distance or an angle, sat
//! under *Copying and extracting text*, where it has nothing to do with either.
//! It was there because it happened to be a slider like the word-gap one beside
//! it. It now has its own group, [`measuring`].
//!
//! | # | Group | Settings |
//! |---|---|---|
//! | 1 | Appearance | theme |
//! | 2 | **Colour** *(open)* | CMYK intent · CMYK JPEG polarity |
//! | 3 | Images and transparency | mask resampling · minification |
//! | 4 | Copying and extracting text | word gap · unmappable codes · replacement text |
//! | 5 | **Measuring and dimensioning** *(new)* | parallel tolerance |
//! | 6 | Pages and printing | separations · missing appearance state |
//! | 7 | Saving files | index line endings · trailing newline |
//!
//! ### Which group starts expanded — a contradiction in the source, resolved
//!
//! `settings_panel.rs`'s header states, with a reason:
//!
//! > The **Colour** group starts expanded because it holds the setting most
//! > likely to have brought someone here — and the only one whose default
//! > knowingly differs from other PDF viewers.
//!
//! Its code opens **Appearance**. Appearance was added later and took the
//! expanded slot without the prose moving. The stated reasoning is still
//! right — Colour holds the knowingly-divergent default and answers the "my
//! black lines look grey" symptom — so **Colour is expanded here** and the
//! prose and the code now agree.
//!
//! ## The three obligations, enforced by a function signature
//!
//! A settings screen that listed keys and radio buttons would satisfy nobody
//! here. Three things must be visible that a conventional one omits:
//!
//! 1. **What the default rests on.** Most of these defaults are *reasoned
//!    inference* — a guess — and a guess must say it is a guess. Exactly one
//!    is well-sourced and says that too.
//! 2. **That a choice was made at all.** These are settings *because the
//!    standard declines to have an opinion*. An operator who does not know
//!    that reads a difference between pdfce and Acrobat as a pdfce bug.
//! 3. **Which way costs what.** A setting whose blast radius is the SAVED
//!    BYTES is a different kind of decision from one that only changes the
//!    preview.
//!
//! Obligations 2 and 3 are not left to discipline: [`widgets::header`] takes
//! `title`, `silence` and `radius` as **required arguments**, so a setting
//! cannot be added without answering all three. Obligation 1 lives in the
//! option notes and is pinned by tests in [`crate::text::settings`].
//!
//! ## Cancel is real
//!
//! The window edits a **working copy**. Nothing reaches the live configuration
//! or the disk until *Save*, and *Cancel* discards the lot. This is not
//! ceremony: four of the thirteen change **saved bytes**, so a radio click that
//! took effect immediately would be an edit the operator never intended and
//! cannot see.
//!
//! **Theme is the single exception**, and it is deliberate — see [`Draft`].

pub mod appearance;
pub mod colour;
pub mod images;
pub mod measuring;
pub mod pages;
pub mod saving;
pub mod text;
pub mod widgets;

use egui::RichText;
use pdfce_core::settings::{Settings, StoreLocation};

use crate::text::settings as t;

/// The region the window publishes for its whole body.
///
/// ★ **This is what makes `ui-verify`'s `settings_headings_legible` a LIVE
/// check.** That check is the regression test for `DEFECTS.md` D2 — every
/// collapsible heading in the old shell's Settings dialog rendering near-white
/// on light grey, at around **1.1:1** against a 3:1 floor — and it has been
/// running in offline mode against a dated screenshot for the whole project,
/// because the sentence in its own header was true: *"the new application has
/// no Settings dialog."*
///
/// It does now, and a check that measures the running program is worth more
/// than one that measures a photograph of a program that no longer exists.
/// Renaming this constant un-aims that check.
pub const REGION_BODY: &str = "dialog:settings"; // ui-text-exempt: trace region name, never displayed

/// The region each group heading publishes, suffixed with the group's key.
///
/// One per collapsible header, so the contrast check can measure **each**
/// heading against its own background rather than sampling the window and
/// hoping. D2's defect was a foreground/background *pairing*, and a pairing
/// only exists once something is drawn — so the check needs the rectangle the
/// application actually laid the text into, not a rectangle derived from a
/// palette.
pub const REGION_HEADING_PREFIX: &str = "settings.heading."; // ui-text-exempt: trace region name, never displayed

/// The region each theme radio publishes, suffixed with the preset's key.
///
/// Exists so a check can **click** one. Proving that a theme picker is on
/// screen is not the property anybody cares about; the property is that
/// choosing Dark makes the window dark, which needs a rect to aim at and a
/// capture afterwards. See `DEFECTS.md` D10.
pub const REGION_THEME_PREFIX: &str = "settings.theme."; // ui-text-exempt: trace region name, never displayed

/// How far the working copy has drifted from what the window opened on.
///
/// # ★ Theme is the one setting that breaks the draft contract, on purpose
///
/// Every other setting here is draft-until-Save. A theme cannot be judged from
/// a radio label — you choose it by *seeing* it — so the selection takes effect
/// on the next frame. The draft still governs what is **saved**; it just no
/// longer governs what is **shown**.
///
/// The mechanism is one line in the application's per-frame `ui()`, before any
/// widget is built: the theme token is read from the draft when a draft exists
/// and from the live settings otherwise. Cancel drops the draft, so the look
/// reverts with it — no separate undo path, and nothing that can get out of
/// step. The window says so in the theme setting's own radius line rather than
/// leaving it to be discovered.
#[derive(Debug, Clone, PartialEq)]
pub struct Draft {
    /// The edits in progress.
    pub working: Settings,
    /// What the settings were when the window opened.
    pub original: Settings,
}

impl Draft {
    /// Start editing from the **live** configuration.
    ///
    /// Live rather than a re-read of the file, and the difference matters in
    /// exactly one case: if a previous save failed, the session is honouring a
    /// choice the disk does not have. The window must show what pdfce is
    /// actually doing, not what it wished it had written.
    #[must_use]
    pub fn new(current: &Settings) -> Self {
        Self {
            working: current.clone(),
            original: current.clone(),
        }
    }

    /// Whether anything has actually changed since the window opened.
    ///
    /// Drives whether *Save* is offered at all. A Save button that is always
    /// live cannot tell the operator whether they have unsaved changes, and
    /// this is a window someone may open just to read.
    ///
    /// **Not latched.** Click a radio and click it back, and the draft is clean
    /// again — because it is, and a dirty flag that only ever went one way
    /// would make Save mean "you visited this window".
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.working != self.original
    }

    /// Whether every value is still pdfce's own answer.
    ///
    /// # Why this is not the same question as [`Self::is_dirty`]
    ///
    /// A draft opened from non-default settings is **clean but not
    /// all-default**: loading is not editing. Collapsing the two predicates
    /// would disable *Restore defaults* for exactly the operator who most needs
    /// it — the one who changed something in a previous session and wants it
    /// back.
    #[must_use]
    pub fn is_all_default(&self) -> bool {
        self.working == Settings::default()
    }
}

/// What the window is asking the application to do.
///
/// Returned rather than performed. This module renders and does not own
/// application state — the split every other dialog and panel in this shell
/// uses — and the three verbs have consequences (adopting a configuration,
/// writing a file, invalidating every cached raster) that belong in the
/// dispatcher where they can be seen together.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Nothing was pressed this frame.
    Idle,
    /// Adopt the working copy and write it.
    Save,
    /// Discard the working copy.
    Cancel,
    /// Replace the working copy with the defaults.
    ///
    /// **Does not save.** The operator still has to confirm, and still has
    /// Cancel. "Restore defaults" is not the kind of button that should be able
    /// to discard a configuration in one click with no way back.
    RestoreDefaults,
}

/// Draw one frame of the window. Returns what the operator asked for.
///
/// # Geometry, and why every number has a reason
///
/// | aspect | value | why |
/// |---|---|---|
/// | screen source | `content_rect` | not `viewport_rect`: it subtracts safe-area insets, so centring uses *usable* space |
/// | width | `620` clamped to `[420, screen − 40]` | wide enough for a full sentence at the body's text size |
/// | height | `82 %` of screen, clamped `[420, 900]` | a fixed 620 left half the height unused *while still scrolling* — the worst combination |
/// | position | `default_pos`, centred, `−20` vertically | see below |
///
/// **`default_pos` rather than `anchor`**, so the operator can drag it aside.
/// A window pinned in the middle of the screen is a window in the way of the
/// document it is about. And egui's own default position put it top-left, over
/// the quick-access toolbar and the ribbon tabs — so *opening Settings hid the
/// control that opened it*. The `−20` keeps the button row on a short screen.
///
/// # The scroll area's height is computed, not fixed
///
/// `available − 96`, floored at 180. The source used a fixed 460 and it clipped
/// the last group's heading in half, so *"Saving files"* read as a rendering
/// fault rather than as a group. The reserved 96 is the intro, the store line,
/// two separators and the button row — everything that is not the list.
pub fn show(
    ctx: &egui::Context,
    draft: &mut Draft,
    store: &StoreLocation,
    open: &mut bool,
) -> Outcome {
    let screen = ctx.input(egui::InputState::content_rect);
    let width = 620.0_f32.min(screen.width() - 40.0).max(420.0);
    let height = (screen.height() * 0.82).clamp(420.0, 900.0);
    let pos = egui::pos2(
        ((screen.width() - width).max(0.0) / 2.0).max(0.0),
        (((screen.height() - height).max(0.0) / 2.0) - 20.0).max(0.0),
    );

    let mut outcome = Outcome::Idle;
    egui::Window::new(t::window_title())
        .collapsible(false)
        .resizable(true)
        .default_size([width, height])
        .default_pos(pos)
        .open(open)
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            ui.label(t::intro());
            ui.add_space(4.0);
            // Always shown, never a control. An operator who does not know
            // which of the two homes is live cannot follow the update
            // instructions, and those instructions are the one place a wrong
            // guess costs them their configuration.
            ui.label(RichText::new(t::store_location(store)).small().weak());
            ui.separator();

            let reserved = 96.0;
            let available = (ui.available_height() - reserved).max(180.0);
            egui::ScrollArea::vertical()
                .max_height(available)
                .show(ui, |ui| {
                    // The order below is the operator-facing order and is the
                    // contract. It is deliberately NOT the order the fields
                    // appear in `Settings`, nor the order they are written to
                    // the file — three orders that the source let drift apart,
                    // which is how `theme` ended up emitted between the two
                    // image settings and splitting them.
                    widgets::group(ui, "appearance", t::group_appearance(), false, |ui| {
                        appearance::theme(ui, draft);
                    });
                    // ★ Colour is the expanded one. See the module header for
                    // the contradiction in the source this resolves.
                    widgets::group(ui, "colour", t::group_colour(), true, |ui| {
                        colour::intent(ui, draft);
                        ui.add_space(10.0);
                        colour::polarity(ui, draft);
                    });
                    widgets::group(ui, "images", t::group_images(), false, |ui| {
                        images::mask_resample(ui, draft);
                        ui.add_space(10.0);
                        images::minify(ui, draft);
                    });
                    widgets::group(ui, "text", t::group_text(), false, |ui| {
                        text::word_gap(ui, draft);
                        // The source omitted this one space, so the word-gap
                        // slider and the setting under it ran together. Every
                        // other adjacent pair in every group has it.
                        ui.add_space(10.0);
                        text::unmappable(ui, draft);
                        ui.add_space(10.0);
                        text::actual_text(ui, draft);
                    });
                    widgets::group(ui, "measuring", t::group_measuring(), false, |ui| {
                        measuring::parallel(ui, draft);
                    });
                    widgets::group(ui, "pages", t::group_pages(), false, |ui| {
                        pages::separations(ui, draft);
                        ui.add_space(10.0);
                        pages::missing_as(ui, draft);
                    });
                    widgets::group(ui, "saving", t::group_saving(), false, |ui| {
                        saving::xref_entry_eol(ui, draft);
                        ui.add_space(10.0);
                        saving::trailing_eol(ui, draft);
                    });
                });

            ui.separator();
            ui.horizontal(|ui| {
                let dirty = draft.is_dirty();
                let save = ui.add_enabled(dirty, egui::Button::new(t::save()));
                if save.clicked() {
                    outcome = Outcome::Save;
                }
                if !dirty {
                    save.on_disabled_hover_text(t::save_disabled_tooltip());
                }
                if ui
                    .button(t::cancel())
                    .on_hover_text(t::cancel_tooltip())
                    .clicked()
                {
                    outcome = Outcome::Cancel;
                }
                // Separated from the two commit/abort controls, because it is
                // the destructive one and a mis-click on it costs the operator
                // every choice they have ever made here.
                ui.add_space(12.0);
                let all_default = draft.is_all_default();
                let restore =
                    ui.add_enabled(!all_default, egui::Button::new(t::restore_defaults()));
                if restore.clicked() {
                    outcome = Outcome::RestoreDefaults;
                }
                if all_default {
                    restore.on_disabled_hover_text(t::restore_defaults_disabled_tooltip());
                } else {
                    restore.on_hover_text(t::restore_defaults_tooltip());
                }
            });
        });

    outcome
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfce_core::pageops::SeparationPolicy;
    use pdfce_core::settings::CmykIntent;

    /// Opening the window is not an edit.
    #[test]
    fn a_fresh_draft_is_not_dirty() {
        let draft = Draft::new(&Settings::default());
        assert!(!draft.is_dirty());
        assert!(draft.is_all_default());
    }

    /// Changing a value makes Save live and Restore live.
    #[test]
    fn changing_a_value_makes_the_draft_dirty() {
        let mut draft = Draft::new(&Settings::default());
        draft.working.cmyk_intent = CmykIntent::Calibrated;
        assert!(draft.is_dirty());
        assert!(!draft.is_all_default());
    }

    /// ★ The dirty flag does not latch.
    ///
    /// A radio click and a click back is not an edit, and a Save button that
    /// stayed live afterwards would be telling the operator they have unsaved
    /// changes when they have none — which is the same lie as a Save button
    /// that is always live, arrived at by a different route.
    #[test]
    fn changing_a_value_back_makes_it_clean_again() {
        let mut draft = Draft::new(&Settings::default());
        let was = draft.working.cmyk_intent;
        draft.working.cmyk_intent = CmykIntent::Naive;
        assert!(draft.is_dirty());
        draft.working.cmyk_intent = was;
        assert!(!draft.is_dirty(), "the dirty flag latched");
    }

    /// ★ The two predicates must not collapse into one.
    ///
    /// A draft opened from non-default settings is **clean** (nothing has been
    /// edited) and **not all-default** (something is not pdfce's answer).
    /// Collapsing them would grey *Restore defaults* for the operator who
    /// changed something last week and wants it back — which is most of the
    /// people who will ever press it.
    #[test]
    fn a_draft_started_from_non_default_settings_is_clean_but_not_all_default() {
        // `Settings` is `#[non_exhaustive]`, so a struct expression is illegal
        // out of crate: the only shape available is start-from-default and
        // assign, which is what this seeds through.
        let mut seed = Settings::default();
        seed.separations = SeparationPolicy::Refuse;
        let draft = Draft::new(&seed);
        assert!(!draft.is_dirty(), "loading is not editing");
        assert!(
            !draft.is_all_default(),
            "Restore defaults must stay available"
        );
    }

    /// Restore defaults reaches all-default without going through Save.
    #[test]
    fn restoring_defaults_is_a_draft_edit_and_nothing_else() {
        let mut seed = Settings::default();
        seed.separations = SeparationPolicy::Refuse;
        let mut draft = Draft::new(&seed);
        draft.working = Settings::default();
        assert!(draft.is_all_default());
        // …and it is an *edit*, so Save is offered. The operator must confirm.
        assert!(
            draft.is_dirty(),
            "restoring defaults must leave something to save"
        );
    }
}
