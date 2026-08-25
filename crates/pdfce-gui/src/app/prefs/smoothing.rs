//! # `app::prefs::smoothing` — pdfce-gui's own answer to image minification
//!
//! **Operator report, 2026-08-25:**
//!
//! > *"there was also an update to an image quality setting to discard smaller
//! > details than the screen sees a while ago that I think has been enabled by
//! > default because image quality is a little worse on normal pages than it
//! > was whereas before it was on par with acrobat reader — this setting should
//! > be an option in our settings and disabled by default."*
//!
//! ## What the setting is, and why his description is exactly right
//!
//! `pdfce_core::settings::MinifyFilter` decides how an image drawn **smaller
//! than its own pixel grid** is sampled — which is what happens to every scan,
//! photograph or CAD raster on a page displayed at anything under 1:1.
//!
//! * `PointSample` takes **one texel per output pixel** and throws the rest
//!   away. That is *"discard smaller details than the screen sees"*, precisely.
//! * `Smooth` averages the area the output pixel covers.
//!
//! The engine's own doc comment states the cost of the first in the same terms
//! the operator used: *"aliasing (shimmer, dropped hairlines) on a heavily
//! downscaled image"*, and the settings window has said so to his face —
//! *"Fine detail such as thin lines or small text in a scan can shimmer or
//! disappear entirely."*
//!
//! ## ★★ The half of his report that is a hypothesis, checked and corrected
//!
//! He attributed it to a recent change enabling something. **It was not
//! enabled recently; it has been the shipped default all along**, and wiring
//! the setting through the GUI changed nothing, because
//! `RenderOptions::default()` and `Settings::default()` carry the *same*
//! `MinifyFilter::default()` — `PointSample`. Verified by reading both.
//!
//! That distinction matters and is not pedantry: had it been a regression,
//! the fix would be to find and revert the change. It is not one, so the fix
//! is a **decision about a default**, which is his to make and which he has now
//! made. The report was right; the diagnosis attached to it was not, and acting
//! on the diagnosis would have sent somebody looking through the engine's
//! history for a commit that does not exist.
//!
//! ## Why the engine's default is not simply wrong
//!
//! `pdfce-core` grades this **evidence tier (d)** — reasoned inference, and it
//! says so out loud: *"i.e. a guess"*. It declines to flip on pdfce's own
//! unverified assertion that most production viewers smooth on minification,
//! because that assertion is exactly the shape of claim the claim-bearing-copy
//! rule targets. And it names its own escape hatch:
//!
//! > *"A viewer-behaviour check filed to `C:\personal_rag\pdf\` would raise
//! > this to tier (c) and, if it confirms, flip the default."*
//!
//! **The operator has now supplied that check**, from the one comparison that
//! settles it: pdfce against Acrobat Reader, on his own drawings, on his own
//! screen. The engine hand-off is written up separately; this module is what
//! makes his screen right today without waiting for it.
//!
//! ## ★★★ Why a one-time migration, and not just a different default
//!
//! Because a different default would do **nothing** for anybody who already
//! has a `settings.txt`, and that is everybody who has run pdfce once.
//!
//! `Settings::save` writes **every** key with its current value, and the
//! engine's store writes a fully commented template on first run. So a real
//! installation contains the line
//!
//! ```text
//! image_minify = point_sample
//! ```
//!
//! not because anyone chose it, but because it was the default at the moment
//! the template was generated. Both of the operator's own settings files carry
//! that line — verified before this was written, and the reason a
//! defaults-only change was rejected: it would have been shipped, reported as
//! done, and changed nothing he could see.
//!
//! A merged value is indistinguishable from a chosen one after the fact, so the
//! migration is deliberately **once, ever, and recorded**:
//!
//! * a marker key in `preferences.txt` — pdfce-gui's own file, not the
//!   engine's — says whether it has run;
//! * absent marker ⇒ set `image_minify = Smooth`, save, write the marker;
//! * present marker ⇒ never touch the setting again, whatever its value.
//!
//! So an operator who *deliberately* wants point sampling changes it once, and
//! it stays changed. The cost is that anyone who had deliberately chosen point
//! sampling before today gets flipped one time — accepted, because the number
//! of such people is one, he is the one asking for the flip, and it is
//! disclosed in the release note rather than done quietly.
//!
//! ★ The marker lives in `preferences.txt` rather than in `settings.txt` on
//! purpose. `settings.txt` is the **engine's** file, its keys are the engine's
//! vocabulary, and a GUI bookkeeping flag in it would be an unknown key to
//! every other pdfce surface — reported as a fault by the CLI, and one more
//! thing the engine has to agree not to delete.

use pdfce_core::settings::{MinifyFilter, Settings, StoreLocation};

/// The `preferences.txt` key that records whether the migration has run.
///
/// Named for what it *is* rather than for what it did, so that a reader
/// meeting it in the file learns something: it is the marker for pdfce-gui
/// having applied its own image-smoothing default.
pub const KEY: &str = "image_smoothing_default_applied"; // ui-text-exempt: a file KEY, parsed and written, never displayed.

/// What pdfce-gui considers the right answer, as distinct from the engine's.
pub const GUI_DEFAULT: MinifyFilter = MinifyFilter::Smooth;

/// The outcome of one migration attempt, for the trace and for the tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// The marker was already present. Nothing was read, changed or written.
    AlreadyApplied,
    /// The setting already held [`GUI_DEFAULT`]; the marker was written so
    /// this is never asked again, but the settings file was left alone.
    ///
    /// ★ Distinct from [`Self::Changed`] deliberately. A migration that
    /// reports "changed" when it changed nothing makes the trace useless for
    /// the one question worth asking afterwards — *did this actually do
    /// anything on this machine?*
    AlreadySmooth,
    /// The setting was flipped to [`GUI_DEFAULT`] and must now be saved.
    Changed,
}

/// **Decide what the migration should do**, without touching the disk.
///
/// Pure, so the decision can be tested exhaustively and the IO can be tested
/// once. `applied` is the marker's current state; `current` is what
/// `settings.txt` yielded.
#[must_use]
pub fn decide(applied: bool, current: MinifyFilter) -> Outcome {
    if applied {
        Outcome::AlreadyApplied
    } else if current == GUI_DEFAULT {
        Outcome::AlreadySmooth
    } else {
        Outcome::Changed
    }
}

/// Run the migration against a live `Settings`, saving if it changed anything.
///
/// Returns the outcome and whether the marker now needs writing back — the
/// caller owns `Prefs` and does that, because this module deliberately knows
/// nothing about how preferences are stored.
///
/// ★ A failed save is traced and otherwise ignored, and the marker is **still**
/// written. The alternative — retry every launch — turns a read-only settings
/// folder into a write attempt on every start, and the operator has already
/// been told once that their settings could not be saved. One disclosure per
/// cause, not one per frame.
pub fn apply(settings: &mut Settings, store: &StoreLocation, applied: bool) -> Outcome {
    let outcome = decide(applied, settings.image_minify);
    if outcome == Outcome::Changed {
        settings.image_minify = GUI_DEFAULT;
        if let Err(e) = settings.save(store) {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!("image-smoothing-migration save-failed detail={e:?}")
            });
        }
    }
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("image-smoothing-migration outcome={outcome:?}")
    });
    outcome
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The marker is absolute.** Once set, the operator's choice stands, and
    /// that is the property that makes flipping a default acceptable at all.
    #[test]
    fn a_marked_installation_is_never_touched_again() {
        assert_eq!(
            decide(true, MinifyFilter::PointSample),
            Outcome::AlreadyApplied,
            "an operator who deliberately went back to point sampling must \
             keep it — a migration that runs twice is not a migration, it is \
             an override"
        );
        assert_eq!(decide(true, MinifyFilter::Smooth), Outcome::AlreadyApplied);
    }

    /// **An unmarked installation carrying the engine default is flipped.**
    /// This is the case every real installation is in today.
    #[test]
    fn an_unmarked_installation_gets_the_gui_default() {
        assert_eq!(decide(false, MinifyFilter::PointSample), Outcome::Changed);
    }

    /// **…and one that already smooths is marked without being rewritten.**
    ///
    /// Worth its own case because the lazy implementation — write the value
    /// unconditionally — would rewrite `settings.txt` on the first launch after
    /// an update for every operator on earth, including the ones for whom
    /// nothing changed, and a settings file whose timestamp moves for no reason
    /// is a settings file nobody trusts.
    #[test]
    fn an_unmarked_installation_that_already_smooths_is_only_marked() {
        assert_eq!(decide(false, MinifyFilter::Smooth), Outcome::AlreadySmooth);
    }

    /// ★ **The GUI's default is not the engine's**, which is the entire premise
    /// of this module. If the engine ever flips its own — the hand-off asks it
    /// to — this test fails, and that failure is the reminder to delete all of
    /// this rather than leave two mechanisms arguing about one value.
    #[test]
    fn this_module_exists_because_the_defaults_differ() {
        assert_ne!(
            GUI_DEFAULT,
            MinifyFilter::default(),
            "pdfce-core's default now matches pdfce-gui's, so the migration \
             and its marker are dead weight — remove this module, its key, and \
             the call site in `app::mod`"
        );
    }
}
