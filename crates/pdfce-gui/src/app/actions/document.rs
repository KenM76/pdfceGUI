//! # `app::actions::document` — the four actions that replace the open
//! document, and the two guards all four share
//!
//! ## Why this is a file of its own
//!
//! `apply.rs` crossed R2's 1,500-line gate on 2026-08-19 when the unsaved-edits
//! guard landed, and `tools/gates/check-file-size.sh`'s own header says what
//! not to do about that: *"Split the module along its seams — one subject per
//! file — rather than raising the limit."*
//!
//! This is the seam, and it was already drawn in prose before it was drawn in
//! files. `apply`'s match has a block at the top whose own comment reads: *"The
//! three actions that are about WHICH document is open, matched BEFORE the
//! guard below."* Everything below that block acts **on** the open document;
//! everything in it decides **which** document is open, or whether there is
//! one. That is a different subject with a different failure mode — the arms
//! below can be wrong about a page, and these can be wrong about an afternoon's
//! work.
//!
//! ## ★ The two guards, in order, and why the order is not interchangeable
//!
//! All four arms ask the same two questions in the same sequence:
//!
//! | # | question | answer | why first / second |
//! |---|---|---|---|
//! | 1 | **Is a save in flight?** (`PdfceApp::save_pending`) | decline outright, trace it | the document's bytes are mid-write; there is no answer the operator could give that would make proceeding safe |
//! | 2 | **Are there unsaved edits?** (`DialogsState::ask_unsaved`) | **ask**, and resume afterwards | there is an answer the operator can give, and it is theirs to give |
//!
//! Reversing them would put a question in front of an operator whose answer
//! cannot be honoured — they would press *Close without saving* and be declined
//! anyway, which reads as a broken button.
//!
//! **They are two predicates, not one, and conflating them is the mistake this
//! file exists to prevent.** `crate::app::lifecycle::save_pending` carries the
//! whole argument: it asks *"is a save in flight"*, is permanently `false`
//! because `file.save_copy` is synchronous, and is **not** *"are there unsaved
//! edits?"* — a successful save-a-copy leaves the document exactly as unsaved
//! as it was, because the copy went somewhere else. `dialogs::ocr`'s
//! `UnsavedEdits` refusal reads `edit_epoch != 0` and would break the moment
//! somebody merged them.
//!
//! ## ★★ The defect this file's shape closes
//!
//! Guard 2 did not exist until 2026-08-19. Every one of these four arms
//! destroyed every edit made since the file was opened, silently, with no
//! prompt and no undo — while `file.close`'s shipped tooltip promised the
//! operator *"You are asked what to do about unsaved edits first."*
//!
//! It was found by an audit against `pdfce`'s capability register, not by a
//! test and not by use, and the reason it survived so long is worth keeping:
//! **the guard that should have caught it existed, was well argued, was
//! correct, and was answering a different question.** A reader arriving at
//! `Action::Close` saw a guard, saw a doc comment explaining the guard, and had
//! no reason to ask whether it was the guard the tooltip was describing.
//!
//! Putting all four arms in one file with the guard table above is the
//! structural half of not repeating that. The other half is
//! [`tests::every_document_replacing_action_asks_about_unsaved_edits`], which
//! fails when a fifth one arrives without it.

use crate::app::PdfceApp;
use crate::dialogs::unsaved::PendingIntent;

impl PdfceApp {
    /// `Action::Open` — replace the open document with the one at `path`.
    ///
    /// With nothing open this is the **ordinary** case: it is how an operator
    /// gets their first document after launching with no argument. That is why
    /// this arm is matched before `apply`'s document guard rather than being
    /// subject to it.
    ///
    /// ★ **The arm that needed the unsaved-edits question most**, and it is
    /// worth saying why it is not Close. An operator who has marked up a
    /// drawing and then opens the next one has destroyed exactly as much work
    /// as one who pressed Close — and is far more likely to do it, because
    /// opening the next file is what you do all day, whereas closing a document
    /// deliberately is something you do at the end of one.
    pub(super) fn apply_open(&mut self, path: std::path::PathBuf) {
        if self.save_pending() {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("open-declined path={path:?} reason=save-pending")
            });
            return;
        }
        if self
            .dialogs
            .ask_unsaved(&self.status, PendingIntent::Open(path.clone()))
        {
            return;
        }
        self.open_path(path);
    }

    /// `Action::New` — replace the open document with a blank one.
    pub(super) fn apply_new(&mut self) {
        if self.save_pending() {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "new-declined reason=save-pending".to_owned()
            });
            return;
        }
        if self.dialogs.ask_unsaved(&self.status, PendingIntent::New) {
            return;
        }
        self.new_document();
    }

    /// `Action::NewSized` — the same replacement, with a page box the operator
    /// chose.
    ///
    /// ★ Beside the plain New and behind the **same two guards**, spelled out
    /// rather than shared, and adjacent rather than merged. Not a copy: the
    /// same arm shape, deliberately next to its twin, so that a change to what
    /// either guard means cannot be applied to one New and missed on the other.
    /// A shared helper would remove the duplication and would also remove the
    /// two `trace` lines that name which New declined — and a trace that cannot
    /// say which of two commands ran is a trace that cannot diagnose either.
    pub(super) fn apply_new_sized(&mut self, width_pt: f64, height_pt: f64) {
        if self.save_pending() {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "new-sized-declined reason=save-pending".to_owned()
            });
            return;
        }
        if self.dialogs.ask_unsaved(
            &self.status,
            PendingIntent::NewSized {
                width_pt,
                height_pt,
            },
        ) {
            return;
        }
        // The lower-left corner is the origin: a new page has nothing to offset
        // from, and `Action::NewSized`'s own docs say why the action carries a
        // size rather than a rectangle.
        self.new_document_sized(pdfce_core::page_tree::Rect::from_corners(
            0.0, 0.0, width_pt, height_pt,
        ));
    }

    /// `Action::Close` — put the document away.
    ///
    /// With nothing open this is a no-op that must still not be reached through
    /// a path that assumes a document, which is the other half of why this
    /// family is matched before `apply`'s guard.
    pub(super) fn apply_close(&mut self) {
        if self.save_pending() {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "close-declined reason=save-pending".to_owned()
            });
            return;
        }
        if self.dialogs.ask_unsaved(&self.status, PendingIntent::Close) {
            return;
        }
        self.close_document();
    }
}

#[cfg(test)]
mod tests {
    /// ★★ **Every action that replaces the open document asks about unsaved
    /// edits.**
    ///
    /// The gate that would have caught the 2026-08-19 defect, written as the
    /// module header promises: *the test that will fail when a fifth one
    /// arrives without it.*
    ///
    /// # Why it reads the source rather than driving the four functions
    ///
    /// Driving them is not possible in a unit test and the reason is the point:
    /// three of the four end in `open_path` / `new_document` /
    /// `new_document_sized`, which build real `EditSession`s, and the one that
    /// does not — `apply_close` — would pass this test trivially by having no
    /// document to ask about. A behavioural test here would exercise the
    /// **absence** of the guard's precondition rather than the presence of the
    /// guard.
    ///
    /// So it asserts the structural property directly: **each of the four
    /// functions in this file names `ask_unsaved`.** Crude, and deliberately so
    /// — the same trade this project made for the settings-coverage gate. A
    /// crude check that fails when the guard is dropped beats an exact one that
    /// cannot run.
    ///
    /// The floor assertion is not optional: without it, a rename of these
    /// functions makes the loop iterate zero times and the test reports full
    /// coverage. `CONTINUE.md` §7 — *an instrument that can only return one
    /// answer cannot detect the thing it was added to detect.*
    #[test]
    fn every_document_replacing_action_asks_about_unsaved_edits() {
        const SRC: &str = include_str!("document.rs");
        // The four function bodies, split on their own signatures. `[1..]`
        // drops everything before the first, which is the module header.
        // ★★ The marker is ASSEMBLED from two pieces rather than written as one
        // literal, and this test's first two drafts are why.
        //
        // The scan looks for the four function signatures. Writing that
        // signature out as a single string — here, or in a comment explaining
        // why not to — puts a fifth copy of it into the very file being
        // scanned, and the split finds five bodies instead of four. Both drafts
        // did it: the first in the `split` call, the second in the comment
        // warning about the first.
        //
        // Funny, and the shape is not. **The instrument was counting itself**,
        // and the spurious body would have contained `ask_unsaved` and
        // `save_pending` — they appear in the assertion messages — so it would
        // have passed every check below. Without the arity assertion above,
        // this test would have reported success while measuring one thing that
        // was not code, which is `CONTINUE.md` §7's rule arriving from a
        // direction nobody predicted: a source-scanning test is part of its own
        // corpus, and the floor assertion is what noticed.
        let marker = format!("    pub(super) {}", "fn apply_");
        let bodies: Vec<&str> = SRC.split(marker.as_str()).skip(1).collect();
        assert_eq!(
            bodies.len(),
            4,
            "expected four document-replacing arms in this file and found {}. \
             If a fifth landed, it needs both guards and this number needs to move; \
             if the naming convention changed, this test has stopped measuring anything.",
            bodies.len()
        );
        for body in bodies {
            let name = body.split('(').next().unwrap_or("<unnamed>");
            assert!(
                body.contains("ask_unsaved"),
                "`apply_{name}` replaces the open document and never calls \
                 `ask_unsaved`. Until 2026-08-19 all four were like that, and all four \
                 destroyed every edit made since the file was opened, silently, while \
                 `file.close`'s tooltip promised otherwise."
            );
            assert!(
                body.contains("save_pending"),
                "`apply_{name}` replaces the open document and never checks \
                 `save_pending`. A save in flight must decline outright — it is the \
                 guard the operator cannot answer."
            );
            // ★ And in that order. Reversed, the operator would be asked a
            // question whose answer cannot be honoured: they press *Close
            // without saving* and are declined anyway, which reads as a broken
            // button rather than as a busy program.
            let pending = body.find("save_pending");
            let ask = body.find("ask_unsaved");
            assert!(
                pending < ask,
                "`apply_{name}` asks about unsaved edits before checking whether a save \
                 is in flight. See this module's guard table: only one of the two has an \
                 answer the operator can give."
            );
        }
    }
}
