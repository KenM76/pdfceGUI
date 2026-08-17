//! # `text::settings` — every word the Settings window shows
//!
//! The catalog area for [`crate::dialogs::settings`]. Ported from the old
//! shell's `ui_text.rs`, where these strings occupied roughly 700 lines in the
//! middle of a 7,912-line file.
//!
//! ## ★ The one rule this module has that the rest of the catalog does not
//!
//! Carried across verbatim from the source, because it is the reason the copy
//! is written the way it is:
//!
//! > Every string here must be readable by someone who has never opened the
//! > PDF standard. These settings exist BECAUSE the standard is silent, so the
//! > operator is being asked to make a judgement — and a judgement cannot be
//! > made from a clause number. The clause is named for traceability; the
//! > SENTENCE has to stand on its own.
//!
//! So `§8.6.4.4` appears in exactly one place per setting, inside a sentence
//! that would still make sense with the number deleted. An operator who has
//! never heard of ISO 32000-1 must be able to choose correctly.
//!
//! ## The three obligations, and how they are enforced by shape
//!
//! `settings_panel.rs`'s header names three things a settings screen must
//! show that a conventional one omits. Two of them are enforced here by
//! **function naming**, not by review: every setting has a `*_title`, a
//! `*_silence` and a `*_radius`, and
//! [`crate::dialogs::settings::widgets::header`] takes all three as required
//! arguments. A setting cannot be added without answering all three.
//!
//! | obligation | where it lives |
//! |---|---|
//! | 1. What the default rests on | inside the chosen option's `_note`, and only where it is true |
//! | 2. That a choice was made at all | `*_silence` — what the standard leaves open |
//! | 3. Which way costs what | `*_radius` — preview, extraction, or **saved bytes** |
//!
//! ### Obligation 1 is the one the source got wrong, and it is fixed here
//!
//! The ambiguity register grades each recommended default: **(a)** observed
//! Acrobat behaviour, **(b)** corpus census, **(c)** other implementations,
//! **(d)** reasoned inference — *a guess*. Most are (d), and the source's own
//! header says a guess must say it is a guess.
//!
//! It said so for five settings and not for five others that `pdfce-core`
//! grades (d) just as explicitly: `image_minify`, `unmappable_code`,
//! `actual_text`, `missing_as` and `trailing_eol` all read as confident
//! recommendations. **Their notes now carry the disclosure**, in the same
//! words the settings that had it already use, so the contract the window
//! states about itself is true of all thirteen rather than of eight.
//!
//! The one *positively* sourced default — CMYK JPEG polarity, tier (c) —
//! says so too, because "pdfce matched every other engine" and "pdfce
//! guessed" are different claims and must not read alike.
//!
//! ## Two disclosures the source documented in the engine and showed nowhere
//!
//! Both are added here, and both are facts rather than directions:
//!
//! - [`unmappable_omit_note`] now says that a run whose codes are **all**
//!   unmappable disappears entirely under *Leave it out* — not merely that
//!   characters go missing. The layout pass drops a run with no characters,
//!   so a page of `Identity-H` text with no `/ToUnicode` yields *zero runs*.
//!   That is the surprising half and the source's note omitted it.
//! - [`actual_text_bound`] is new. No length correspondence exists between
//!   `/ActualText` and the content it replaces, so character-level mapping
//!   back to glyph positions is **impossible across such a run whichever
//!   option is chosen** — which bounds search highlighting, selection and
//!   redaction-by-text to sequence granularity. `pdfce-core` calls this *"a
//!   fact to disclose, not a direction to pick"* and the old window disclosed
//!   it nowhere.
//!
//! Both settings' radius lines also now name **redaction**, because R35 is
//! explicit that a redaction built under one value is not equivalent under
//! another, and "affects copied and extracted text" does not tell an operator
//! that.

pub mod bytes;
pub mod extract;
pub mod look;

pub use bytes::*;
pub use extract::*;
pub use look::*;

use egui_shell::theme::Preset;
use pdfce_core::settings::StoreKind;
use pdfce_core::settings::StoreLocation;

// ===========================================================================
// Window chrome
// ===========================================================================

/// The window's title.
#[must_use]
pub const fn window_title() -> &'static str {
    "Settings"
}

/// The paragraph under the title.
///
/// Load-bearing rather than decorative: it is the sentence that tells an
/// operator why this window is full of questions instead of being full of
/// answers. Without it, thirteen radio groups read as thirteen things pdfce
/// could not decide.
#[must_use]
pub const fn intro() -> &'static str {
    "The PDF standard leaves some things genuinely undefined, so different \
     programs can open the same file and be equally correct while showing you \
     different results. Where that happens, pdfce asks you rather than deciding \
     quietly. Each choice below says what the standard does not settle, what \
     pdfce ships as its answer and why, and what changing it affects."
}

/// Where the settings file lives, said in the operator's terms.
///
/// # Why this line is always shown
///
/// An operator who does not know which of the two homes is live cannot follow
/// the update instructions, and those instructions are the one place a wrong
/// guess costs them their configuration: *"replace the program files, keep
/// your `userdata` folder"* means nothing if the settings are not in it.
///
/// [`StoreKind`] is `#[non_exhaustive]`, so the catch-all arm is required by
/// the compiler — and it still says something useful rather than falling
/// silent, because a variant this build does not know about is still a home
/// the operator's settings are in.
#[must_use]
pub fn store_location(store: &StoreLocation) -> String {
    match (store.kind, store.path.as_deref()) {
        (StoreKind::Portable, Some(path)) => format!(
            "Kept in {} — this folder is yours. When you update pdfce by replacing \
             the program files, keep it.",
            path.display()
        ),
        (StoreKind::Portable, None) => "Your choices are kept beside the program.".to_owned(),
        (StoreKind::PlatformFallback, Some(path)) => format!(
            "Kept in {} because pdfce's own folder is not writable. These choices \
             will NOT travel with the program folder if you move or copy it.",
            path.display()
        ),
        (StoreKind::PlatformFallback, None) => {
            "Kept in your system settings folder, because pdfce's own folder is not writable."
                .to_owned()
        }
        _ => "No writable location was found, so anything you change here lasts only \
              until you close pdfce."
            .to_owned(),
    }
}

// ===========================================================================
// Buttons
// ===========================================================================

/// The commit button.
#[must_use]
pub const fn save() -> &'static str {
    "Save"
}

/// Why Save is greyed.
///
/// Greyed rather than absent, which is the one place this window departs from
/// the no-placeholders rule and is entitled to: Save is *temporarily*
/// unavailable — one radio click makes it live — and greying with a reason on
/// hover is exactly what that rule reserves greying for.
#[must_use]
pub const fn save_disabled_tooltip() -> &'static str {
    "Nothing has changed yet."
}

/// The abort button.
#[must_use]
pub const fn cancel() -> &'static str {
    "Cancel"
}

/// What Cancel promises, said plainly and unconditionally.
///
/// Not a courtesy. Four of the thirteen settings change **saved bytes**, so an
/// operator who has been clicking radio buttons for a minute needs to know
/// that none of it has taken effect — and needs to know it *before* they
/// decide whether to click Cancel, which is why it is a tooltip on an
/// always-enabled control rather than a confirmation after the fact.
#[must_use]
pub const fn cancel_tooltip() -> &'static str {
    "Close without changing anything. Nothing you have clicked here has taken \
     effect yet."
}

/// The reset control.
#[must_use]
pub const fn restore_defaults() -> &'static str {
    "Restore defaults"
}

/// Why *Restore defaults* is greyed.
#[must_use]
pub const fn restore_defaults_disabled_tooltip() -> &'static str {
    "Everything is already set to pdfce's own answer."
}

/// What *Restore defaults* actually does, on hover when it is live.
///
/// It replaces the **draft** and does not save. Said out loud because the
/// button's name suggests otherwise: "restore defaults" in most programs is
/// immediate and irreversible, and this one is neither.
#[must_use]
pub const fn restore_defaults_tooltip() -> &'static str {
    "Sets every choice below back to pdfce's own answer. Nothing is written \
     until you press Save, and Cancel still puts everything back."
}

/// The status-bar line after a successful save.
#[must_use]
pub fn saved(path: &str) -> String {
    format!("Settings saved to {path}.")
}

/// The status-bar line after a failed save.
///
/// Loud, and deliberately not softened: the operator asked for something to be
/// remembered and it was not. The session still honours the choice — see the
/// dispatch arm — so the sentence has to carry the distinction between "this
/// did not happen" and "this will not survive a restart".
#[must_use]
pub fn save_failed(reason: &str) -> String {
    format!(
        "Settings could NOT be saved: {reason} — this session is using your \
         choices, but they will be gone when pdfce restarts."
    )
}

// ===========================================================================
// Group headings
// ===========================================================================

/// Group 1.
#[must_use]
pub const fn group_appearance() -> &'static str {
    "Appearance"
}

/// Group 2 — the one that starts expanded.
#[must_use]
pub const fn group_colour() -> &'static str {
    "Colour"
}

/// Group 3.
#[must_use]
pub const fn group_images() -> &'static str {
    "Images and transparency"
}

/// Group 4.
#[must_use]
pub const fn group_text() -> &'static str {
    "Copying and extracting text"
}

/// Group 5.
///
/// ★ **New in this port.** In the old shell `parallel_epsilon_degrees` sat
/// under *Copying and extracting text* — where it has nothing to do with
/// either — purely because it happened to be a slider like the word-gap one
/// beside it. The operator symptom is *"my dimension came out as an angle"*,
/// and nobody with that symptom looks under a heading about copying.
///
/// The group headings are the whole navigation model of this window: an
/// operator arrives with a symptom and the headings are how a symptom finds
/// its setting. A setting filed under the wrong one is not untidy, it is
/// unreachable.
#[must_use]
pub const fn group_measuring() -> &'static str {
    "Measuring and dimensioning"
}

/// Group 6.
#[must_use]
pub const fn group_pages() -> &'static str {
    "Pages and printing"
}

/// Group 7.
#[must_use]
pub const fn group_saving() -> &'static str {
    "Saving files"
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★ Every setting answers all three obligations, and none of the answers
    /// is empty.
    ///
    /// The mechanical half of the window's stated contract. A setting added
    /// with a title and no silence line would compile — the helper takes
    /// `&str` — and would ship a control that says what it is and never says
    /// why the operator is being asked.
    #[test]
    fn every_setting_states_its_silence_and_its_radius() {
        let triples: &[(&str, &str, &str)] = &[
            (theme_title(), theme_silence(), theme_radius()),
            (
                cmyk_intent_title(),
                cmyk_intent_silence(),
                cmyk_intent_radius(),
            ),
            (polarity_title(), polarity_silence(), polarity_radius()),
            (mask_title(), mask_silence(), mask_radius()),
            (minify_title(), minify_silence(), minify_radius()),
            (word_gap_title(), word_gap_silence(), word_gap_radius()),
            (parallel_title(), parallel_silence(), parallel_radius()),
            (
                unmappable_title(),
                unmappable_silence(),
                unmappable_radius(),
            ),
            (
                actual_text_title(),
                actual_text_silence(),
                actual_text_radius(),
            ),
            (
                separations_title(),
                separations_silence(),
                separations_radius(),
            ),
            (
                missing_as_title(),
                missing_as_silence(),
                missing_as_radius(),
            ),
            (xref_eol_title(), xref_eol_silence(), xref_eol_radius()),
            (
                trailing_eol_title(),
                trailing_eol_silence(),
                trailing_eol_radius(),
            ),
        ];
        assert_eq!(triples.len(), 13, "thirteen settings, one triple each");
        for (title, silence, radius) in triples {
            assert!(!title.is_empty(), "a setting with no title");
            assert!(!silence.is_empty(), "{title:?} does not say what is open");
            assert!(!radius.is_empty(), "{title:?} does not say what it costs");
        }
    }

    /// ★ Every setting that changes SAVED BYTES says so, and no other does.
    ///
    /// The distinction the window exists to make legible: a setting whose blast
    /// radius is the file on disk is a different kind of decision from one that
    /// only changes the preview, and an operator must be able to tell them
    /// apart from the words. Four of thirteen touch bytes.
    ///
    /// Asserted in both directions. A preview setting whose radius grew the
    /// word "file" would be quietly claiming a consequence it does not have —
    /// which trains the operator to stop reading these lines, and the four that
    /// matter are the ones that then get skipped.
    #[test]
    fn exactly_the_byte_changing_settings_say_they_change_the_file() {
        // "the file you save" / "the bytes pdfce writes" / "the saved file".
        let touches_bytes = |radius: &str| {
            radius.contains("the file you save")
                || radius.contains("bytes pdfce writes")
                || radius.contains("the saved file")
        };

        for radius in [
            separations_radius(),
            xref_eol_radius(),
            trailing_eol_radius(),
            polarity_radius(),
        ] {
            assert!(touches_bytes(radius), "a byte setting hides it: {radius:?}");
        }

        // ★ The theme is checked against BOTH its lines, and it is the only one.
        //
        // Every other setting makes its "and the file is untouched" promise in
        // its radius line. The theme makes it in its **silence** line —
        // *"nothing here is written into a PDF you save"* — because that is the
        // sentence explaining why a window-chrome setting is in a window full
        // of file-format questions at all, and repeating it one line later
        // would be padding. Its radius line has a different and more useful job:
        // saying that this one setting takes effect **before** Save, which is
        // the exception to the whole window's contract.
        //
        // So the pair is joined here rather than the theme being exempted. An
        // exemption would let a future edit delete the promise from both.
        let theme_says = format!("{} {}", theme_silence(), theme_radius());
        assert!(!touches_bytes(&theme_says), "{theme_says:?}");
        assert!(
            theme_says.contains("written into a PDF"),
            "the theme no longer promises it leaves documents alone: {theme_says:?}"
        );

        for radius in [
            cmyk_intent_radius(),
            mask_radius(),
            minify_radius(),
            word_gap_radius(),
            parallel_radius(),
            unmappable_radius(),
            actual_text_radius(),
            missing_as_radius(),
        ] {
            assert!(
                !touches_bytes(radius),
                "a preview-only setting claims it changes the file: {radius:?}"
            );
            assert!(
                radius.contains("does not change the file")
                    || radius.contains("Does not change the file"),
                "a preview-only setting does not say it leaves the file alone: {radius:?}"
            );
        }
    }

    /// ★ Every default that is a GUESS admits it, in its own note.
    ///
    /// Obligation 1, mechanised — and the test that would have failed on the
    /// old shell for five of these. `pdfce-core` grades `image_minify`,
    /// `unmappable_code`, `actual_text`, `missing_as` and `trailing_eol` tier
    /// (d), reasoned inference, exactly as explicitly as it grades the two that
    /// already said so, and all five read as confident recommendations.
    ///
    /// The predicate is deliberately loose about wording and strict about
    /// presence: what matters is that the sentence disclaims external
    /// authority, not that it uses one phrasing.
    #[test]
    fn every_guessed_default_says_it_is_a_guess() {
        let admits = |note: &str| {
            note.contains("pdfce's own")
                || note.contains("considered guess")
                || note.contains("are guesses")
                || note.contains("has not been checked")
                || note.contains("pdfce reading")
                || note.contains("pdfce taking")
        };
        for (name, note) in [
            ("mask_resample", mask_nearest_note()),
            ("image_minify", minify_point_note()),
            ("word_gap_ratio", word_gap_note()),
            ("unmappable_code", unmappable_replacement_note()),
            ("actual_text", actual_text_always_note()),
            ("missing_as", missing_as_nothing_note()),
            ("trailing_eol", trailing_eol_lf_note()),
        ] {
            assert!(
                admits(note),
                "{name}'s default is a guess and its note does not say so: {note:?}"
            );
        }
    }

    /// ★ The one SOURCED default says it is sourced, and says it differently.
    ///
    /// The counterpart to the test above, and the reason that one is not
    /// enough. If every note hedged, the operator would have no way to tell
    /// which of thirteen defaults rests on evidence. CMYK JPEG polarity is
    /// tier (c) — every reference engine agrees — and it must not read like a
    /// guess.
    #[test]
    fn the_sourced_default_claims_its_evidence() {
        let note = polarity_never_note();
        assert!(
            note.contains("best-supported"),
            "the one sourced default no longer claims its evidence: {note:?}"
        );
        assert!(
            !note.contains("guess") && !note.contains("pdfce's own"),
            "the sourced default hedges like a guessed one: {note:?}"
        );
    }

    /// The divergence from Acrobat is stated, and states which way to go back.
    ///
    /// A note saying only "pdfce differs" would leave the operator who wants
    /// parity with nothing to click. It has to name the option.
    #[test]
    fn the_acrobat_divergence_names_the_option_that_matches() {
        let note = cmyk_intent_divergence();
        assert!(note.contains("Acrobat"));
        assert!(
            note.contains(cmyk_intent_calibrated_label()),
            "the divergence note does not name the matching option: {note:?}"
        );
    }

    /// The unknown-theme sentence quotes the token and promises to keep it.
    ///
    /// Both halves matter. Quoting is what makes the cause legible; the promise
    /// is what stops the operator "fixing" it by picking one of the three,
    /// which would discard a newer version's setting.
    #[test]
    fn an_unknown_theme_is_named_and_kept() {
        let said = theme_unknown("midnight");
        assert!(said.contains("\"midnight\""), "{said:?}");
        assert!(said.contains("kept"), "{said:?}");
    }

    /// Every store location says something, including the one with no home.
    ///
    /// The `None` case is the one worth pinning: an operator whose settings
    /// cannot be written anywhere must be told before they spend a minute
    /// choosing, not after they press Save.
    #[test]
    fn every_store_location_is_described() {
        use std::path::PathBuf;
        let portable = StoreLocation {
            path: Some(PathBuf::from("C:\\pdfce\\userdata\\settings.txt")),
            kind: StoreKind::Portable,
        };
        assert!(store_location(&portable).contains("userdata"));

        let nowhere = StoreLocation {
            path: None,
            kind: StoreKind::None,
        };
        let said = store_location(&nowhere);
        assert!(!said.is_empty());
        assert!(
            said.contains("until you close"),
            "a session with no writable store must say the choices are temporary: {said:?}"
        );
    }

    /// Each theme preset has a distinct name and a distinct description.
    #[test]
    fn the_presets_are_distinguishable() {
        let labels: Vec<&str> = Preset::ALL.iter().map(|p| theme_preset_label(*p)).collect();
        let notes: Vec<&str> = Preset::ALL.iter().map(|p| theme_preset_note(*p)).collect();
        for i in 0..labels.len() {
            for j in (i + 1)..labels.len() {
                assert_ne!(labels[i], labels[j]);
                assert_ne!(notes[i], notes[j]);
            }
        }
    }

    /// The two disclosures added in this port are actually present.
    ///
    /// Both were documented in `pdfce-core` and shown nowhere in the old
    /// window. A test rather than a comment, because "we should surface that"
    /// is the kind of intention that survives one session.
    #[test]
    fn the_two_engine_facts_the_old_window_hid_are_disclosed() {
        assert!(
            unmappable_omit_note().contains("disappears altogether"),
            "the disappearing-run consequence is not disclosed: {:?}",
            unmappable_omit_note()
        );
        let bound = actual_text_bound();
        assert!(
            bound.contains("Whichever you choose"),
            "the ActualText bound reads as an argument for one option: {bound:?}"
        );
        assert!(
            bound.contains("redact"),
            "the ActualText bound does not name redaction: {bound:?}"
        );
    }
}
