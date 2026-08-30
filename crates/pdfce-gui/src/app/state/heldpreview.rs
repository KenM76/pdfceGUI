//! # `app::state::heldpreview` — **the preview that outlives the gesture**
//!
//! `OPERATOR_REQUESTS.md` **O63**, third piece. Split out of `app::state` under
//! **R2** on 2026-08-30, when adding it took that file to 1,672 lines.
//!
//! ## The subject, which is a seam rather than a size-driven cut
//!
//! `app::state` answers *"what does the shell know about the open document?"*.
//! This answers one much narrower question with an unusual property: **for how
//! long is a picture of the document still true?**
//!
//! Everything here is about a race between two things that are both correct —
//! an edit that has already happened, and a raster that has not caught up — and
//! the whole content of the module is the rule for deciding which of them the
//! operator should be looking at. That rule has three clauses, two of them
//! bounded by wall-clock time, and neither of those is obvious. It earns a file.
//!
//! ## What it exists to remove, in the operator's own words
//!
//! **Ken, 2026-08-30:** *"the live preview should remain while the update to the
//! pdf structure runs in the background."*
//!
//! Before this, releasing a drag discarded the preview **and the raster
//! underneath still showed the object where it started** — for one to two
//! seconds on his own CAD drawing. What that looks like is: the preview
//! vanishes, the object is back where it was, a pause, and then it jumps to
//! where he put it. **It appears to snap back**, which reads as the program
//! having refused the edit and then changed its mind.
//!
//! ## ★★ Why holding a picture is honest rather than optimistic
//!
//! The edit **has happened**. The document already reads the way the preview is
//! drawn; the only thing behind is the picture. So this is not a guess about a
//! commit that might fail — it is the true state of the document, drawn by the
//! one path that can produce it in under a second.
//!
//! ⇒ Which is why every clause below keys on evidence that the commit
//! *actually* landed, and why the one clause that cannot get that evidence
//! immediately is bounded by a quarter of a second rather than trusted.

use super::OpenDoc;

/// A live shape preview kept on screen while the page raster catches up.
///
/// See [`OpenDoc::held_preview`] for why this exists at all.
pub(crate) struct HeldPreview {
    /// The geometry, exactly as the gesture last drew it.
    pub shape: crate::canvas::shapes::ShapePreview,
    /// `edit_epoch` at the moment the gesture released — **before** the commit.
    ///
    /// ★ The liveness test compares against this rather than against the epoch
    /// the commit produced, because the commit has not happened yet when this is
    /// stored: actions are drained *after* the frame that raised them.
    pub captured_at_epoch: u64,
    /// When it was captured, for the backstop.
    pub since: std::time::Instant,
}

/// How long a held preview may survive before it is dropped regardless.
///
/// # ★★★ Why a wall-clock backstop, when the epoch test should be enough
///
/// Because *"should be enough"* is how a preview becomes permanent. The epoch
/// test drops the hold when the raster carrying the edit arrives — and if that
/// raster never arrives (a render that fails, a page that will not rasterise, a
/// strip path that leaves `page_texture_epoch` behind for a reason nobody has
/// thought of yet), the operator is left looking at a selection-coloured
/// tracing of their drawing with no way to clear it.
///
/// ⇒ A stuck preview is worse than a late one: it is indistinguishable from a
/// corrupted document. Four seconds is roughly four times the measured
/// whole-page raster on the operator's hardest drawing, so it cannot fire on a
/// render that is merely slow.
const HELD_PREVIEW_MAX: std::time::Duration = std::time::Duration::from_secs(4);

/// How long a hold may sit with the edit epoch **unmoved** before it is dropped.
///
/// # ★★★ This is the difference between "not applied yet" and "refused"
///
/// Actions are drained *after* the frame that raised them, so there is a real
/// window — one frame, ~16 ms — in which a hold is legitimate and the epoch has
/// not moved. There is also a state in which the epoch never moves at all: the
/// **engine refused the edit**. By epoch alone the two are identical.
///
/// By elapsed time they are not remotely alike. 250 ms is fifteen frames at
/// 60 Hz — far longer than the real window can be, far shorter than a refusal
/// stays wrong for.
///
/// ★ Getting this wrong ships a preview of a move that did not happen, sitting
/// over a document that disagrees with it, for the full four seconds of
/// [`HELD_PREVIEW_MAX`]. That is the single worst outcome available to this
/// feature, because it is a picture of a lie rather than a picture that is late.
const HELD_PREVIEW_GRACE: std::time::Duration = std::time::Duration::from_millis(250);

impl OpenDoc {
    /// **The held preview, if it should still be on screen.**
    ///
    /// Three conditions, in order, and each rejects a different way for a hold
    /// to be wrong:
    ///
    /// | test | what it rejects |
    /// |---|---|
    /// | the epoch moved | a gesture that was **refused** — nothing committed, so there is nothing to preview |
    /// | the raster has not caught up | a picture that is already correct — drawing over it would be strictly worse than the real thing |
    /// | it is younger than [`HELD_PREVIEW_MAX`] | a raster that will never arrive, leaving a preview nobody can clear |
    ///
    /// ★ There is a fourth state that deliberately draws: the frame **between**
    /// the release and the commit. Actions are drained after the frame that
    /// raised them, so for exactly one frame `edit_epoch` still equals
    /// `captured_at_epoch`. Rejecting that frame would blink the preview off and
    /// on again, which is the flicker this feature exists to remove.
    pub(crate) fn held_preview_to_draw(&self) -> Option<&crate::canvas::shapes::ShapePreview> {
        let held = self.held_preview.as_ref()?;
        if held.since.elapsed() > HELD_PREVIEW_MAX {
            return None;
        }
        // ★★★ The one-frame window described above — and it is bounded by TIME,
        // not by the epoch, and the difference is a defect that would otherwise
        // ship.
        //
        // Actions are drained after the frame that raised them, so for one frame
        // `edit_epoch` still equals `captured_at_epoch`. Accepting that state
        // unconditionally would also accept it **forever** — which is exactly
        // what happens when the engine REFUSES the edit: the epoch never moves,
        // and the operator is left looking at a preview of a move that did not
        // happen, for four seconds, with the document underneath disagreeing
        // with it.
        //
        // ⇒ A refusal is indistinguishable from "not applied yet" by epoch
        // alone. It is entirely distinguishable by *how long it has been*: one
        // frame is 16 ms and a refusal is forever. `moving::drag` already
        // declines to hold anything for the refusals IT can see; this covers the
        // ones only the apply phase can — the engine saying no after the Action
        // was raised.
        if self.edit_epoch == held.captured_at_epoch {
            return (held.since.elapsed() < HELD_PREVIEW_GRACE).then_some(&held.shape);
        }
        // The raster carrying the edit has landed. The document's own picture is
        // correct now, and it is better than this one in every way.
        if self.page_texture_epoch == self.edit_epoch {
            return None;
        }
        Some(&held.shape)
    }

    /// Drop a held preview that has stopped being live.
    ///
    /// ★ Separate from [`Self::held_preview_to_draw`] because that one takes
    /// `&self` — it is called from the painter, which holds the document
    /// immutably. This is called once a frame from `canvas::interact`, which
    /// does not, and it exists so a dead hold does not sit in memory carrying
    /// thousands of segments until the next gesture replaces it.
    pub(crate) fn retire_held_preview(&mut self) {
        if self.held_preview.is_some() && self.held_preview_to_draw().is_none() {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "canvas-held-preview-retired".to_owned()
            });
            self.held_preview = None;
        }
    }

    /// **Hold this preview until the page catches up.**
    ///
    /// Called on release, with the geometry the gesture last drew. Replaces any
    /// previous hold outright: a second edit supersedes the first, and two
    /// previews on screen would be two claims about one document.
    pub(crate) fn hold_preview(&mut self, shape: crate::canvas::shapes::ShapePreview) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!(
                "canvas-held-preview epoch={} segments={}",
                self.edit_epoch,
                shape.segment_count()
            )
        });
        self.held_preview = Some(HeldPreview {
            shape,
            captured_at_epoch: self.edit_epoch,
            since: std::time::Instant::now(),
        });
    }
}
