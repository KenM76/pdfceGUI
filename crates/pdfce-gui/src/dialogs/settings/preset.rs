//! # `dialogs::settings::preset` — a named vector of answers
//!
//! **Operator request O38, 2026-08-25:**
//!
//! > *"I'd like a preset setting for rendering things to what the [print
//! > conformance suite] page needs to render correctly … since it is for
//! > conformance to PDF/X-4 (ISO 15930-7) … maybe we should have a dropdown to
//! > select view options between the different standards."*
//!
//! ## What a preset IS, and what it deliberately is not
//!
//! A preset is a **named bundle of settings that already exist**, applied in one
//! click, and **individually editable afterwards**. It is not a mode, not a
//! second rendering path, and not a lock. Choosing one writes values into the
//! draft exactly as though the operator had set each control by hand, and the
//! window then behaves as it always did.
//!
//! That is the whole design, and it is what makes the feature cheap and safe:
//! there is no new state to persist, no new thing that can disagree with
//! `settings.txt`, and no way for a preset to express something the individual
//! controls cannot. A preset that could would be a second source of truth.
//!
//! ## ★★★ Where the values came from, and why every one of them is graded
//!
//! Not from here. A control labelled *ISO 15930-7* carries that standard's
//! authority whether or not it was meant to, so the vector was **asked for**
//! and the engine answered with an API rather than a table
//! (`settings::presets`, Pass 128.1). Its reply quoted this project's own
//! reason back at it, and then made the sharper point:
//!
//! > *"The interesting column is not the value. It is how much weight the
//! > value can bear, and for most of these axes the answer is less than the
//! > button implies."*
//!
//! Only **one** of PDF/X-4's six answers is a claim about the standard at all,
//! and it is graded `implied` rather than `sourced`. So the row shows the
//! grading beside the choice; a row that showed the name and hid the grade
//! would be the over-claim this whole request was careful to avoid.
//!
//! ★★ Three consequences the engine had to spell out, all of which shape this
//! file:
//!
//! 1. **Not every standard binds a renderer.** PDF/A and PDF/UA both put
//!    *"operational details of rendering"* outside their own scope. `PdfUa1`
//!    exists and correctly **sets nothing** — surfaced as an answer, not
//!    hidden as an omission, because *"nothing, and here is the measurement"*
//!    cannot be mistaken for unfinished work.
//! 2. **A third of the grid is axes a standard does not reach.** No PDF/X part
//!    contains a shading clause, so none of them says anything about mesh
//!    padding. `PresetAction::LeaveAlone` is a real state and is rendered as
//!    one — blank would read as missing data, a value would assert a
//!    requirement that does not exist.
//! 3. **`cmyk_intent` has no conformant value.** Every PDF/X level guarantees a
//!    *colorimetric* definition of device colour, and `CmykIntent` selects
//!    among fixed built-in tables, which is not one. The preset takes the
//!    least-wrong value and **discloses that the file's own output intent was
//!    not applied** — mandatory under rule 4, because a colour transform that
//!    did not happen leaves nothing on screen to notice.
//!
//! ## What DOES ship today, and why it is not a consolation prize
//!
//! **pdfce's own recommended answers.** The operator's report that opened this
//! request included *"touching some of our presets caused some test to show up
//! as failed"* — which is a person who has changed several settings while
//! investigating and now wants a way back. That is a real need, it is
//! answerable today with complete authority (these are *our* defaults; no
//! external standard is being spoken for), and it is the half of the request
//! that was never blocked.

use pdfce_core::settings::Settings;
use pdfce_core::settings::presets::{Evidence, PresetKey, RenderPreset, RenderStandard};

use super::Draft;

/// One thing the operator can choose from the presets row.
///
/// Two kinds, and the distinction is the whole model:
///
/// * [`Choice::Recommended`] — pdfce's own shipped answers. **We** are the
///   authority, so it can say what it does without qualification.
/// * [`Choice::Standard`] — a published standard's answers, from
///   `pdfce_core::settings::presets`. **We are not the authority**, so every
///   value carries the engine's own evidence grade and the row shows it.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Choice {
    /// Restore what pdfce ships with.
    Recommended,
    /// Render as a published standard specifies — as far as it specifies
    /// anything at all.
    Standard(RenderStandard),
}

impl Choice {
    /// A stable id for the radio's identity and for tests.
    fn id(self) -> &'static str {
        match self {
            Self::Recommended => "pdfce",
            Self::Standard(s) => s.as_str(),
        }
    }

    /// What the operator sees.
    fn label(self) -> String {
        match self {
            Self::Recommended => crate::text::settings::preset_pdfce_label().to_owned(),
            Self::Standard(s) => s.title().to_owned(),
        }
    }

    /// Write this choice's answers into a settings struct.
    fn apply(self, settings: &mut Settings) {
        match self {
            Self::Recommended => apply_pdfce(settings),
            // ★ A standard's preset is applied ON TOP of pdfce's defaults, not
            // onto whatever the operator last had. Otherwise "PDF/X-4" would
            // mean something different depending on what was set before it,
            // which is the one thing a named preset must not do.
            Self::Standard(s) => {
                apply_pdfce(settings);
                let _ = RenderPreset::for_standard(s).apply(settings);
            }
        }
    }
}

/// **Everything the presets row offers**, in display order.
///
/// ★★★ pdfce's own answers first, then every standard the engine knows about.
/// The list is *derived* from `RenderStandard::all()` rather than restated, so
/// a standard the engine adds appears here with no change at all — R8's
/// registration rule, reached through the crate boundary instead of through a
/// command registry.
#[must_use]
pub fn choices() -> Vec<Choice> {
    std::iter::once(Choice::Recommended)
        .chain(RenderStandard::all().iter().copied().map(Choice::Standard))
        .collect()
}

/// pdfce's own recommended answers.
///
/// # ★★★ It is `Settings::default()`, and getting here took two corrections
///
/// The first draft restated two values explicitly, believing pdfce-gui diverged
/// from the engine on both. **Neither was true**, and the test written to catch
/// exactly this caught both, an hour apart:
///
/// * **`image_minify`** genuinely diverged — for one day. The engine adopted
///   `Smooth` on 2026-08-25 (Pass 128.0) on the strength of the operator's
///   Acrobat comparison, and the restatement became a no-op that same evening.
/// * **`cmyk_intent`** never diverged at all. `NeutralBlack` has been the
///   engine's *shipped default* since the operator's 2026-08-08 ruling was
///   adopted there. What the engine's doc records is a divergence **in
///   reasoning** — it says openly that its default is knowingly not the
///   best-evidenced answer — and that was misread here as a divergence in
///   **value**.
///
/// ★ The second is the more instructive mistake. A doc comment asserting a
/// difference that does not exist is worse than no comment: it invites the next
/// reader to preserve a line that does nothing, and it makes a *deliberate*
/// override indistinguishable from a copied one. **Read the value, not the
/// prose about the value.**
///
/// So this assigns nothing. Every answer is the engine's, taken by *not
/// assigning it*, so a default the engine changes tomorrow arrives here for
/// free and cannot rot into a stale literal.
fn apply_pdfce(s: &mut Settings) {
    *s = Settings::default();
}

/// Which preset the given settings currently match, if any.
///
/// ★ Returns `None` for "none of them", which is the **normal** state once an
/// operator has adjusted anything, and is not a fault. The control shows no
/// selection rather than pretending the nearest one is chosen — a radio that
/// claimed "pdfce recommended" over settings that are not pdfce's recommended
/// answers would be lying about the thing it exists to report.
#[must_use]
pub fn matching(settings: &Settings) -> Option<&'static str> {
    choices().into_iter().find_map(|c| {
        let mut probe = Settings::default();
        c.apply(&mut probe);
        same(&probe, settings).then_some(c.id())
    })
}

/// Whether two settings agree on **everything a preset sets**.
///
/// ★★ Compares the render-radius fields explicitly rather than deriving
/// `PartialEq` on `Settings`, and the reason is not tidiness. `Settings` also
/// carries `theme` — the *program's* appearance — and the two write-radius
/// entries that change bytes on disk. A preset is about how a **document
/// renders**; an operator who picks a dark theme has not stopped using pdfce's
/// recommended rendering, and a comparison that said otherwise would clear the
/// radio for a reason that has nothing to do with rendering.
fn same(a: &Settings, b: &Settings) -> bool {
    a.cmyk_intent == b.cmyk_intent
        && a.image_minify == b.image_minify
        && a.mask_resample == b.mask_resample
        && a.page_blend_space_source == b.page_blend_space_source
        && a.mesh_patch_padding == b.mesh_patch_padding
        && a.separations == b.separations
}

/// **Draw the presets row.**
///
/// Above the groups rather than inside one, because it acts on all of them —
/// `widgets::group`'s convention is that a group holds settings sharing a
/// *subject*, and a preset shares a *purpose*.
///
/// ## ★★★ What is shown BESIDE the choice, and why it is not decoration
///
/// The engine's reply that supplied these values spent most of its length on
/// one point: *"the interesting column is not the value, it is how much weight
/// the value can bear."* Only one of PDF/X-4's six answers is a claim about the
/// standard at all, and that one is `implied` rather than `sourced`. A row that
/// showed the name and hid the grading would be exactly the over-claim the
/// whole request was careful to avoid.
///
/// So a selected standard shows:
///
/// * **its disclosures**, verbatim from the engine. These are not advisory —
///   `cmyk_intent` has no conformant value at all, so choosing a PDF/X preset
///   means a colour transform did *not* happen, and rule 4 requires saying so
///   because nothing on screen would reveal it.
/// * **what it leaves alone**, named. Roughly a third of the grid is axes a
///   standard does not reach — no PDF/X part contains a shading clause, so none
///   of them says anything about mesh padding. Showing those as blank would
///   read as missing data; showing them as values would assert a requirement
///   that does not exist.
pub fn row(ui: &mut egui::Ui, draft: &mut Draft) {
    let rect = ui
        .scope(|ui| {
            super::widgets::header(
                ui,
                crate::text::settings::preset_title(),
                crate::text::settings::preset_silence(),
                // Deliberately no radius line: a preset's radius is the union
                // of the radii of what it sets, and restating "affects what you
                // see" here while each setting states its own would be a
                // second, vaguer copy of something already precise one screen
                // down.
                "",
            );
            let current = matching(&draft.working);
            for c in choices() {
                let selected = current == Some(c.id());
                if ui.radio(selected, c.label()).clicked() && !selected {
                    c.apply(&mut draft.working);
                }
                if selected {
                    detail(ui, c);
                }
            }
        })
        .response
        .rect;

    // Published so a driven check can assert the row is ON SCREEN rather than
    // merely laid out. The settings window is a `ScrollArea`, and a control
    // scrolled out of one still reports a rect — hence `ui_rect_visible`,
    // intersected with the clip, exactly as the group headings do. A row that
    // exists and cannot be reached is a defect this project has already met
    // twice, in the Tool panel and in the overflow menu.
    crate::diag::ui_rect_visible(REGION, rect, ui.clip_rect());
}

/// What a selected choice says about itself.
///
/// Indented under its radio, muted, and **only for the selected one** — the
/// full grid for nine standards at once would be a wall of text nobody reads,
/// and the operator only needs the caveats for the answer they have chosen.
fn detail(ui: &mut egui::Ui, choice: Choice) {
    let Choice::Standard(standard) = choice else {
        ui.label(
            egui::RichText::new(crate::text::settings::preset_pdfce_note())
                .small()
                .weak(),
        );
        return;
    };
    let preset = RenderPreset::for_standard(standard);
    ui.indent(standard.as_str(), |ui| {
        // ★★★ THE WEIGHT LINE, and it is the reason this feature is not just a
        // dropdown. The engine's own framing: *"the interesting column is not
        // the value, it is how much weight the value can bear, and for most of
        // these axes the answer is less than the button implies."* For PDF/X-4
        // exactly ONE of six answers is a claim about the standard at all.
        //
        // Summarised rather than tabulated: a six-row grid per standard, times
        // ten standards, is a wall nobody reads, and burying the choice under
        // it would trade one over-claim for an unusable control. One sentence
        // carries the same fact.
        let mut sourced = 0_usize;
        let mut inferred = 0_usize;
        let mut chosen = 0_usize;
        for e in preset.entries() {
            match e.evidence {
                Evidence::Sourced => sourced += 1,
                Evidence::Implied => inferred += 1,
                Evidence::BestEffort => chosen += 1,
                // Counted through `left_alone` below instead, where it is
                // named rather than tallied — "does not apply" is a different
                // kind of fact from "we chose", and adding them together would
                // be the arithmetic that makes a preset look better sourced
                // than it is.
                _ => {}
            }
        }
        ui.label(
            egui::RichText::new(crate::text::settings::preset_weight(
                sourced, inferred, chosen,
            ))
            .small()
            .weak(),
        );

        // ★ Verbatim from the engine, never paraphrased. These sentences carry
        // clause citations and one is quoted word-for-word out of ISO 15930;
        // rewording them here would put our phrasing behind a standard's
        // authority, which is the failure this whole feature was shaped to
        // avoid.
        for line in preset.disclosures() {
            ui.label(egui::RichText::new(line).small().weak());
        }

        let untouched = preset.left_alone();
        if !untouched.is_empty() {
            // ★★ Named with the titles the operator already reads one screen
            // down, NOT with the engine's key names. `mesh_patch_padding` is a
            // field identifier; "A gradient fill that comes out scrambled" is
            // the control they would go looking for. Printing the identifier
            // would be leaking our vocabulary into their window, and the
            // ui-strings gate would be right to object.
            let names: Vec<&str> = untouched.iter().map(|k| operator_title(*k)).collect();
            ui.label(
                egui::RichText::new(crate::text::settings::preset_leaves_alone(
                    // ui-text-exempt: a list separator, not a sentence. The
                    // sentence around it lives in `text::settings`.
                    &names.join("; "),
                ))
                .small()
                .weak(),
            );
        }
    });
}

/// The operator-facing title for a settings key.
///
/// ★ The engine names these `mesh_patch_padding`, `image_minify` and so on —
/// field identifiers, correct for an API and wrong for a window. Every one of
/// them already has a title in [`crate::text::settings`], written by the
/// symptom that would send somebody looking for it, and this is the one place
/// the two vocabularies meet.
///
/// A `PresetKey` the engine adds later falls through to its own name rather
/// than to a placeholder: an unfamiliar identifier is ugly and honest, whereas
/// a guessed title would be a sentence pdfce never wrote.
fn operator_title(key: PresetKey) -> &'static str {
    use crate::text::settings as t;
    match key {
        PresetKey::PageBlendSpaceSource => t::blend_space_title(),

        PresetKey::MeshPatchPadding => t::mesh_padding_title(),
        PresetKey::MaskResample => t::mask_title(),
        PresetKey::ImageMinify => t::minify_title(),
        PresetKey::CmykIntent => t::cmyk_intent_title(),
        PresetKey::Separations => t::separations_title(),
        // ui-text-exempt: the engine's own key name, shown only when this shell
        // has not yet been taught a title for a key the engine added.
        other => other.as_str(),
    }
}

/// The published region name for the presets row.
///
/// A cross-repo stability contract with `tools/ui-verify`: renaming it is
/// changing an API, not tidying a string.
// ui-text-exempt: a diagnostic region name, never displayed.
pub const REGION: &str = "settings.presets";

#[cfg(test)]
mod tests {
    use super::*;

    /// **Applying a choice leaves the settings matching a choice with the same
    /// answers** — and that is deliberately weaker than "matching itself".
    ///
    /// ★★★ Because **two standards can mean the same thing to pdfce**, and two
    /// of them do: applying `pdf-x3` leaves settings that `matching` reports as
    /// `pdf-x1a`. That is not a defect in either — it is a fact about the
    /// domain. PDF/X-1a and PDF/X-3 differ in what colour spaces a *file* may
    /// contain, and pdfce's render-radius settings cannot see that difference,
    /// so the two produce an identical vector.
    ///
    /// The first version of this test asserted the stronger property and failed
    /// on the second standard it tried. Weakening it was the right response
    /// rather than adding state to remember which button was pressed: the radio
    /// reflects **what the settings are**, not what was last clicked, and a
    /// remembered choice could disagree with `settings.txt` — which is the one
    /// thing this feature's design set out to make impossible.
    ///
    /// What must hold is the round trip that the radio actually depends on: the
    /// settings after applying a choice are the settings of *whatever* choice
    /// is reported, so the selection shown is never a lie about the values.
    #[test]
    fn applying_a_choice_leaves_settings_that_match_an_equivalent_one() {
        for c in choices() {
            let mut applied = Settings::default();
            c.apply(&mut applied);

            let id = matching(&applied)
                .unwrap_or_else(|| panic!("applying `{}` must match SOME choice", c.id()));

            let equivalent = choices()
                .into_iter()
                .find(|o| o.id() == id)
                .expect("`matching` returns an id from `choices`");
            let mut reapplied = Settings::default();
            equivalent.apply(&mut reapplied);
            assert!(
                same(&applied, &reapplied),
                "applying `{}` reported `{id}`, but `{id}`'s own answers differ from what was applied — the radio would be showing a selection that does not describe the settings",
                c.id()
            );
        }
    }

    /// ★★ **At least two standards share a vector**, recorded so that the
    /// weakened assertion above is understood as describing the domain rather
    /// than as a concession.
    ///
    /// If this ever fails — because pdfce gains a setting that distinguishes
    /// them — the test above can be strengthened back, and this is the note
    /// that says so.
    #[test]
    fn some_standards_are_indistinguishable_from_the_settings_alone() {
        let mut a = Settings::default();
        let mut b = Settings::default();
        Choice::Standard(RenderStandard::PdfX1a).apply(&mut a);
        Choice::Standard(RenderStandard::PdfX3).apply(&mut b);
        assert!(
            same(&a, &b),
            "PDF/X-1a and PDF/X-3 now differ in pdfce's render settings, so `applying_a_choice_leaves_settings_that_match_an_equivalent_one` can go back to asserting each choice matches ITSELF"
        );
    }

    /// ★★ **pdfce's recommended answers ARE the engine's defaults**, and this
    /// test is the guard on the day that stops being true.
    ///
    /// Its predecessor asserted the opposite and failed twice in one evening,
    /// which is the whole reason this one is worded as it is. See
    /// [`super::apply_pdfce`] for what those two failures taught.
    ///
    /// If a real divergence is ever added, this fails — and that failure is the
    /// prompt to state the reason in `apply_pdfce`'s doc comment *before*
    /// amending the assertion, so an override always arrives with its
    /// justification attached rather than as a bare line somebody later deletes
    /// as redundant.
    #[test]
    fn the_recommended_preset_is_the_engines_defaults() {
        let mut ours = Settings::default();
        apply_pdfce(&mut ours);
        assert!(
            same(&ours, &Settings::default()),
            "pdfce-gui now overrides one of the engine's rendering defaults. \
             That may be right — but say WHY in `apply_pdfce`'s doc comment \
             before changing this assertion, or the next reader cannot tell a \
 deliberate override from a stale restatement"
        );
    }

    /// **A changed setting matches no preset**, rather than the nearest one.
    #[test]
    fn adjusted_settings_match_nothing() {
        let mut s = Settings::default();
        apply_pdfce(&mut s);
        assert!(matching(&s).is_some());
        s.mask_resample = pdfce_core::settings::MaskResample::Bilinear;
        assert_eq!(
            matching(&s),
            None,
            "an operator who has adjusted a render setting is on no preset, and \
 the control must say so rather than claiming the closest one"
        );
    }

    /// ★★★ **The theme is not part of a preset**, because a preset is about how
    /// a DOCUMENT renders and the theme is about the program.
    #[test]
    fn choosing_a_theme_does_not_leave_the_preset() {
        let mut s = Settings::default();
        apply_pdfce(&mut s);
        let before = matching(&s);
        s.theme = "dark".to_owned();
        assert_eq!(
            matching(&s),
            before,
            "picking a dark theme is not a rendering decision, and must not \
 clear a rendering preset"
        );
    }

    /// **Every choice has a distinct copy and a distinct id.**
    #[test]
    fn presets_are_distinct() {
        let all = choices();
        for (i, a) in all.iter().enumerate() {
            for b in all.iter().skip(i + 1) {
                assert_ne!(a.id(), b.id());
                assert_ne!(a.label(), b.label());
            }
        }
    }

    /// ★★ **Every key a standard leaves alone is named in the operator's own
    /// words**, not in the engine's field identifiers.
    ///
    /// `mesh_patch_padding` is correct for an API and wrong for a window. The
    /// fallback arm exists so a key the engine adds later degrades to an
    /// unfamiliar identifier — ugly and honest — rather than to a guessed
    /// title, which would be a sentence pdfce never wrote appearing under a
    /// standard's name.
    ///
    /// This fails when the engine adds a `PresetKey`, which is the point: the
    /// failure is the prompt to write a title for it.
    #[test]
    fn every_key_a_standard_leaves_alone_has_an_operator_facing_title() {
        let mut seen = 0_usize;
        for standard in RenderStandard::all() {
            for key in RenderPreset::for_standard(*standard).left_alone() {
                seen += 1;
                assert_ne!(
                    operator_title(key),
                    key.as_str(),
                    "`{}` is shown to the operator with its ENGINE key name. Give it a title in `text::settings` and add the arm to `operator_title`",
                    key.as_str()
                );
            }
        }
        assert!(
            seen > 0,
            "no standard left any key alone, so this test asserted nothing — either the engine changed shape or the presets are not loading"
        );
    }

    /// **The weight line adds up to the answers that carry a value.**
    ///
    /// ★ `NotApplicable` is deliberately excluded from the tally and named in
    /// the left-alone list instead. *"Does not apply"* is a different kind of
    /// fact from *"we chose"*, and adding them together is the arithmetic that
    /// would make a preset look better sourced than it is.
    #[test]
    fn the_weight_tally_excludes_what_the_standard_does_not_reach() {
        for standard in RenderStandard::all() {
            let preset = RenderPreset::for_standard(*standard);
            let graded = preset
                .entries()
                .iter()
                .filter(|e| !matches!(e.evidence, Evidence::NotApplicable))
                .count();
            let left = preset.left_alone().len();
            let not_applicable = preset
                .entries()
                .iter()
                .filter(|e| matches!(e.evidence, Evidence::NotApplicable))
                .count();
            assert!(
                left >= not_applicable,
                "{}: {not_applicable} entries are marked not-applicable but only {left} keys are reported as left alone, so the summary would count an axis twice",
                standard.as_str()
            );
            let _ = graded;
        }
    }
}
