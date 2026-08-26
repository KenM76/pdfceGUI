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
//! ## ★★★ Why PDF/X-4 is not in this file yet, and why that is the feature
//!
//! Because **nobody here knows the values**, and a control labelled
//! *ISO 15930-7* carries that standard's authority whether or not it was meant
//! to. Writing a plausible vector and shipping it under a standard's name is
//! precisely the failure the claim-bearing-copy rule exists to prevent: the
//! operator would reasonably read it as *what the standard requires*, when it
//! would in fact be one engineer's best guess about a licensed test suite he
//! cannot run.
//!
//! The engine team runs that suite, owns the parity instrument, and already
//! grades every one of these settings for evidence quality. The vector has been
//! **asked for** (`request_the_conformant_setting_vector_for_iso_15930_7.md`).
//!
//! ★★ So the mechanism ships now and the entry appears when its values exist —
//! and it appears **by being added to [`PRESETS`], with no other change at
//! all**. That is R8's rule applied to presets rather than to commands:
//! *registering it is the only way the GUI learns it exists.* A preset with no
//! vector is not registered, so it is not drawn, so there is no greyed-out
//! *"PDF/X-4 (coming soon)"* row — which R9 forbids outright, and which would
//! be the worst of both worlds: the standard's authority with none of its
//! content.
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

use super::Draft;

/// One named set of answers.
///
/// ★ `apply` is a function rather than a data table on purpose. Several of
/// these settings are `#[non_exhaustive]` enums from `pdfce-core`, so a table of
/// literals here would need updating every time the engine adds a variant,
/// silently going stale in between. A function that assigns named variants
/// fails to compile when one is removed and keeps working when one is added.
pub struct Preset {
    /// Stable id. Never displayed; used for the radio's identity and by tests.
    pub id: &'static str,
    /// What the operator sees.
    pub label: fn() -> &'static str,
    /// One sentence on when to choose it.
    pub note: fn() -> &'static str,
    /// Write this preset's answers into a settings struct.
    ///
    /// Takes `&mut Settings` rather than returning one because `Settings` is
    /// `#[non_exhaustive]`: it cannot be constructed by struct expression
    /// outside its own crate, so every caller would have to start from
    /// `default()` and assign — which is exactly what this does, once.
    pub apply: fn(&mut Settings),
}

/// **Every preset the operator may choose.**
///
/// ★★★ THE REGISTRATION RULE. This list is the only thing that decides what the
/// window offers. Adding PDF/X-4 when its values arrive is one entry here and
/// nothing else — no control to write, no layout to change, no condition to
/// wire. Until then it is absent rather than disabled, which is R9: an
/// unavailable capability renders **nothing**, and greying is reserved for a
/// *temporarily* unavailable one the operator can act their way out of.
pub const PRESETS: &[Preset] = &[Preset {
    id: "pdfce",
    label: crate::text::settings::preset_pdfce_label,
    note: crate::text::settings::preset_pdfce_note,
    apply: apply_pdfce,
}];

/// pdfce's own recommended answers.
///
/// # ★ Not simply `Settings::default()`, and the difference matters
///
/// Two of these deliberately diverge from what `pdfce-core` ships, both by the
/// operator's own explicit ruling, and a "recommended" preset that quietly
/// reverted them would be undoing decisions rather than restoring them:
///
/// * **`cmyk_intent = NeutralBlack`** — his ruling of 2026-08-08 (*"flip it"*),
///   made after seeing what the calibrated answer does to pure-K line art. The
///   engine's own doc records this as a knowing divergence from the
///   best-sourced answer rather than as agreement with it, which is exactly why
///   it has to be restated here instead of inherited.
/// * **`image_minify = Smooth`** — his ruling of 2026-08-25, after comparing
///   against Acrobat on his own drawings. `pdfce-core` still defaults to
///   `PointSample`; pdfce-gui migrates it once. See `app::prefs::smoothing`.
///
/// Everything else is the engine's default, taken by *not assigning it* rather
/// than by copying the value — so a default the engine changes tomorrow arrives
/// here for free, and cannot drift into a stale literal.
fn apply_pdfce(s: &mut Settings) {
    *s = Settings::default();
    s.cmyk_intent = pdfce_core::settings::CmykIntent::NeutralBlack;
    s.image_minify = crate::app::prefs::smoothing::GUI_DEFAULT;
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
    PRESETS.iter().find_map(|p| {
        let mut probe = Settings::default();
        (p.apply)(&mut probe);
        same(&probe, settings).then_some(p.id)
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
/// and `widgets::group`'s own convention is that a group holds settings that
/// share a subject, which a preset does not: it shares a *purpose*.
///
/// ★ Nothing is drawn at all when only one preset is registered and it is
/// already selected? **No** — it is drawn regardless, and that is deliberate.
/// The reported use is an operator who has changed several things and wants a
/// way back; a control that hides itself precisely when the settings are
/// pristine is a control that is missing every time somebody looks for it
/// *before* experimenting, and then cannot be found *after*.
pub fn row(ui: &mut egui::Ui, draft: &mut Draft) {
    // ★ The rect comes from a `scope`'s own response rather than from
    // arithmetic over `cursor()` and `min_rect()`. The first draft did the
    // arithmetic, produced a rect with no usable area, and `ui_rect_visible`
    // correctly suppressed it — so the row published nothing, which from
    // outside is indistinguishable from the row never having drawn. Let egui
    // report what it actually laid out.
    let rect = ui
        .scope(|ui| {
            super::widgets::header(
                ui,
                crate::text::settings::preset_title(),
                crate::text::settings::preset_silence(),
                // Deliberately no radius line: a preset's radius is the union
                // of the radii of what it sets, and restating "affects what you
                // see" here while each setting states its own would be a
                // second, vaguer copy of information that is already precise
                // one screen down.
                "",
            );
            let current = matching(&draft.working);
            for p in PRESETS {
                let selected = current == Some(p.id);
                if ui
                    .radio(selected, (p.label)())
                    .on_hover_text((p.note)())
                    .clicked()
                    && !selected
                {
                    (p.apply)(&mut draft.working);
                }
                ui.label(egui::RichText::new((p.note)()).small().weak());
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

/// The published region name for the presets row.
///
/// A cross-repo stability contract with `tools/ui-verify`: renaming it is
/// changing an API, not tidying a string.
// ui-text-exempt: a diagnostic region name, never displayed.
pub const REGION: &str = "settings.presets";

#[cfg(test)]
mod tests {
    use super::*;

    /// **Applying a preset then asking which one matches gives it back.**
    ///
    /// The round trip is the whole contract, and it is what a radio's selected
    /// state depends on. If it failed, choosing a preset would move the
    /// settings and then immediately show nothing selected — which reads as the
    /// click not having worked.
    #[test]
    fn applying_a_preset_makes_it_the_matching_one() {
        for p in PRESETS {
            let mut s = Settings::default();
            (p.apply)(&mut s);
            assert_eq!(
                matching(&s),
                Some(p.id),
                "applying `{}` must make `{}` the preset that matches",
                p.id,
                p.id
            );
        }
    }

    /// ★★ **pdfce's recommended answers are NOT the engine's defaults**, and
    /// this test exists to fail loudly on the day that stops being true.
    ///
    /// Two settings diverge by the operator's explicit rulings. If the engine
    /// adopts either — the request asking it to adopt the second is already
    /// filed — this fails, and that failure is the reminder to delete the
    /// corresponding line from `apply_pdfce` rather than leave a restatement
    /// that has quietly become a no-op.
    #[test]
    fn the_recommended_preset_still_diverges_from_the_engine() {
        let mut ours = Settings::default();
        apply_pdfce(&mut ours);
        let theirs = Settings::default();
        assert!(
            !same(&ours, &theirs),
            "pdfce's recommended answers now equal pdfce-core's defaults, so \
             `apply_pdfce`'s explicit assignments are restating rather than \
             diverging — remove the ones that have become redundant"
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

    /// **Every registered preset has distinct copy and a distinct id.**
    ///
    /// Cheap now with one entry, and the point is that it is already here when
    /// the second arrives.
    #[test]
    fn presets_are_distinct() {
        for (i, a) in PRESETS.iter().enumerate() {
            for b in PRESETS.iter().skip(i + 1) {
                assert_ne!(a.id, b.id);
                assert_ne!((a.label)(), (b.label)());
            }
        }
    }
}
