//! # `canvas::textedit::keys` — what every key means inside a draft
//!
//! ## What this is
//!
//! One function, [`typing`], and the rules it enforces. It is the whole of the
//! keyboard's contract with a text draft: which keys insert, which move, which
//! commit, which abandon, and — since 2026-08-21 — which select.
//!
//! ## Why it is its own file
//!
//! R2. `textedit/mod.rs` reached 1,571 lines the day the selection landed, and
//! the seam was already drawn: everything else in that module is about *what a
//! draft is* and *where it came from*, and this is about *what happens when a
//! key goes down*. The old shell's 25,005-line `main.rs` is the argument, and
//! the rule that prevents it is to split at the seam rather than to raise the
//! limit.
//!
//! ## ★★ The four selection rules, and where each is enforced
//!
//! | # | rule | enforced by |
//! |---|---|---|
//! | 1 | a selection is the range between the mark and the caret | [`caret::range`] |
//! | 2 | typing replaces it | the `Text` arm, via [`take_selection`] |
//! | 3 | Backspace and Delete remove it and nothing else | their two arms |
//! | 4 | any movement without Shift drops it | [`caret::moved`], called by every movement arm |
//!
//! Rule 4 is the one that looks like a detail and is not: without it a
//! highlight stays on screen after the caret has walked out of it, and the next
//! keystroke deletes text the operator is no longer looking at.
//!
//! ## ★ What is NOT here, named rather than left to be discovered
//!
//! **Drag-select and double-click-to-select-a-word.** The draft is drawn in an
//! editor box in *screen* space by [`super::paint`], and hit-testing a pointer
//! into it needs that laid-out galley published where the click ladder can
//! reach it. Real work, not a line — and until it exists, a selection is made
//! with the keyboard only.

use egui::Ui;

use super::caret::{self, backspace, delete_forward, insert, word_left, word_right};
use super::{Anchor, DIAG_TYPE, Draft, abandon, blocks, commit_into, read, store};
use crate::app::state::OpenDoc;

/// **Consume this frame's keystrokes into the draft.**
///
/// Returns `true` when the draft was committed by Enter, so the caller knows the
/// caret is gone.
///
/// # Why the events are read raw rather than through a `TextEdit` widget
///
/// Because the caret is painted in PDF space, on the page, at the glyphs' own
/// scale — which is what *"just edit the existing box"* means. An `egui`
/// `TextEdit` would be a second box floating over the first, and the old shell's
/// one virtue here is worth keeping: it had a real caret in the page, and no
/// widget in the typing path.
/// **Remove whatever is selected**, and answer the caret.
///
/// Answers `draft.caret` unchanged when nothing is selected, so a caller may
/// call it unconditionally — which the `Text` arm does, because *"replace the
/// selection if there is one"* and *"insert here"* is one act.
///
/// ★ It clears the mark, and that is not tidying: a mark left pointing into
/// text that no longer exists is an index past the end of the string, and the
/// next Shift+Left would select a range that is not there.
fn take_selection(draft: &mut Draft) -> usize {
    let Some((from, to)) = caret::range(draft.mark, draft.caret) else {
        return draft.caret;
    };
    draft.mark = None;
    caret::delete_range(&mut draft.text, from, to)
}

pub fn typing(
    ui: &Ui,
    ctx: &egui::Context,
    doc: &OpenDoc,
    focused: bool,
    actions: &mut Vec<crate::app::actions::Action>,
) -> bool {
    let Some(mut draft) = read(ctx) else {
        return false;
    };
    let mut changed = false;
    // The diagnostic seam, consumed exactly once per draft. See [`DIAG_TYPE`].
    if !draft.seeded {
        draft.seeded = true;
        changed = true;
        if let Ok(seed) = std::env::var(DIAG_TYPE)
            && !seed.is_empty()
        {
            draft.text.clear();
            draft.caret = insert(&mut draft.text, 0, &seed);
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!("text-edit-seeded len={}", draft.text.chars().count())
            });
        }
    }
    if focused {
        // ★★ Read once, outside the loop: see [`caret::shifted`] for why the
        // frame's own modifier state is consulted at all, and why ignoring it
        // cost this shell its whole first driven run of Shift+arrow.
        let frame_shift = ui.input(|i| i.modifiers.shift);
        for ev in ui.input(|i| i.events.clone()) {
            match ev {
                // ★★ TYPING REPLACES THE SELECTION. Rule 2 of the four in
                // `caret`'s selection section, and the one an operator notices
                // first: select a word, type a word, and the old one is gone.
                egui::Event::Text(t) if !t.is_empty() => {
                    draft.caret = take_selection(&mut draft);
                    draft.caret = insert(&mut draft.text, draft.caret, &t);
                    changed = true;
                }
                // ★ Rule 3: with a selection, Backspace and Delete remove
                // THAT and nothing else — they stop being different keys, which
                // is what every text field does and is why both arms are the
                // same two lines.
                egui::Event::Key {
                    key: egui::Key::Backspace,
                    pressed: true,
                    ..
                } => {
                    draft.caret = if caret::range(draft.mark, draft.caret).is_some() {
                        take_selection(&mut draft)
                    } else {
                        backspace(&mut draft.text, draft.caret)
                    };
                    changed = true;
                }
                egui::Event::Key {
                    key: egui::Key::Delete,
                    pressed: true,
                    ..
                } => {
                    draft.caret = if caret::range(draft.mark, draft.caret).is_some() {
                        take_selection(&mut draft)
                    } else {
                        delete_forward(&mut draft.text, draft.caret)
                    };
                    changed = true;
                }
                // ★★ SELECT ALL. `Ctrl+A` is not in the keymap and must not be:
                // the application's own Select-all acts on OBJECTS, and while a
                // draft is live the operator means the text they are typing.
                // The draft takes the chord first and the event is consumed, so
                // the two never both fire.
                egui::Event::Key {
                    key: egui::Key::A,
                    pressed: true,
                    modifiers,
                    ..
                } if modifiers.command => {
                    draft.mark = Some(0);
                    draft.caret = draft.text.chars().count();
                    changed = true;
                }
                // ★★ **Caret movement**, 2026-08-20, on the operator's report
                // that *"the cursor just sits at the end of a text line. It
                // can't be moved to the center of an existing text block."*
                //
                // These five arms are what makes the caret a caret. Before
                // them the draft had no position at all: text was appended and
                // Backspace popped, so changing `SHEET 1 OF 4` to `SHEET 2 OF
                // 4` meant deleting back to `SHEET ` and retyping the rest.
                //
                // ★ `changed` is set for a pure movement, and that is
                // deliberate rather than sloppy. It is the flag that decides
                // whether the draft is written back to `egui::Memory`, and a
                // moved caret IS a changed draft - without this the arrow keys
                // would appear to work for one frame and then snap back on the
                // next load. It does NOT put anything on the undo stack:
                // `commit_into` compares the TEXT with the original, so a draft
                // whose caret moved and whose characters did not still pushes
                // no action.
                egui::Event::Key {
                    key: egui::Key::ArrowLeft,
                    pressed: true,
                    modifiers,
                    ..
                } => {
                    // ★ Rule 4, applied by one function so every movement arm
                    // agrees: Shift plants or keeps the mark, no Shift drops it.
                    draft.mark = caret::moved(
                        draft.mark,
                        draft.caret,
                        caret::shifted(modifiers.shift, frame_shift),
                    );
                    draft.caret = if modifiers.command {
                        word_left(&draft.text, draft.caret)
                    } else {
                        draft.caret.saturating_sub(1)
                    };
                    changed = true;
                }
                egui::Event::Key {
                    key: egui::Key::ArrowRight,
                    pressed: true,
                    modifiers,
                    ..
                } => {
                    draft.mark = caret::moved(
                        draft.mark,
                        draft.caret,
                        caret::shifted(modifiers.shift, frame_shift),
                    );
                    let end = draft.text.chars().count();
                    draft.caret = if modifiers.command {
                        word_right(&draft.text, draft.caret)
                    } else {
                        (draft.caret + 1).min(end)
                    };
                    changed = true;
                }
                // ★★★ UP AND DOWN WALK THE PAGE'S OWN LINES, AND CROSS INTO
                // THE NEXT PARAGRAPH.
                //
                // The operator, 2026-08-21: *"there was an acrobat feature in
                // the original pdfce-gui that attempted to reassemble
                // individual lines into paragraphs and the cursor would move to
                // the next block of text using the navigation keys."*
                //
                // **Salvage.** `canvas::textedit::blocks` carries the four
                // lines it came from and the argument; the short form is that
                // the reassembly is `pdfce-core`'s — `caret_up` walks the
                // model's *lines*, and a block is a group of lines, so the
                // caret steps into the next paragraph without anything here
                // knowing what a paragraph is. The old shell's whole
                // contribution was **asking**, and this shell had not been.
                //
                // ★ It was not bound at all before today, and that was right at
                // the time: the caret is a character index into ONE run, and a
                // single run has no line above it. What changed is not the
                // draft — it is that the *page* is now the thing being
                // navigated.
                //
                // ★★ THE DRAFT IS COMMITTED ON THE WAY OUT. A caret that left
                // a run with unsaved keystrokes in it would silently discard
                // them, which is the defect class this whole module exists
                // against — and `commit_into` writes nothing when the text is
                // unchanged, so an operator who is merely reading with the
                // arrow keys puts nothing on the undo stack.
                //
                // ★ A BOX draft is deliberately excluded. Its lines are the
                // shell's wrap rather than the page's, so this model would move
                // the caret to a run somewhere else on the sheet mid-paragraph.
                // Named in `blocks`' header rather than left to be discovered.
                egui::Event::Key {
                    key: key @ (egui::Key::ArrowUp | egui::Key::ArrowDown),
                    pressed: true,
                    ..
                } => {
                    let dir = if key == egui::Key::ArrowUp {
                        blocks::Vertical::Up
                    } else {
                        blocks::Vertical::Down
                    };
                    if blocks::step(ctx, doc, &draft, dir, actions) {
                        return true;
                    }
                }
                // ★★ HOME AND END REACH THE ENDS OF THE LINE THE OPERATOR CAN
                // SEE, which on a CAD sheet is usually several show operators
                // wide. `blocks::line` answers `false` when the line is this
                // run — the common case, and the cheap one — and the two
                // assignments below are what happens then.
                egui::Event::Key {
                    key: egui::Key::Home,
                    pressed: true,
                    modifiers,
                    ..
                } => {
                    // ★ Shift+Home selects to the start of the draft and stays
                    // in it, rather than walking to another run: a selection
                    // that spanned two show operators would be a selection this
                    // shell cannot commit, and offering it would be a gesture
                    // whose result is a refusal.
                    let shift = caret::shifted(modifiers.shift, frame_shift);
                    if !shift && blocks::line(ctx, doc, &draft, false, actions) {
                        return true;
                    }
                    draft.mark = caret::moved(draft.mark, draft.caret, shift);
                    draft.caret = 0;
                    changed = true;
                }
                egui::Event::Key {
                    key: egui::Key::End,
                    pressed: true,
                    modifiers,
                    ..
                } => {
                    let shift = caret::shifted(modifiers.shift, frame_shift);
                    if !shift && blocks::line(ctx, doc, &draft, true, actions) {
                        return true;
                    }
                    draft.mark = caret::moved(draft.mark, draft.caret, shift);
                    draft.caret = draft.text.chars().count();
                    changed = true;
                }
                // ★★★ ENTER MEANS TWO THINGS, AND THE ANCHOR DECIDES WHICH.
                //
                // The operator, 2026-08-21: *"I should be able to make it multi
                // line."*
                //
                // | anchor | plain Enter | Ctrl+Enter |
                // |---|---|---|
                // | a **box** | a paragraph break | commit |
                // | a point, or an existing run | commit | commit |
                //
                // ★ This is the old shell's own split, carried across verbatim:
                // *"in box mode a plain Enter is a paragraph break; Ctrl+Enter
                // accepts. In point mode Enter accepts (single line)."* It is
                // also what every program in the class does, which is the
                // standing tie-breaker.
                //
                // ★★ And it is why `Anchor::Box` is a variant rather than an
                // `Option<Rect>` on `Origin`. Enter cannot mean *insert* and
                // *commit* in one draft, so the keystroke handler has to know
                // which gesture started it — and asking the TEXT ("does it
                // already contain a newline?") would make the first Enter
                // commit and every one after it insert, which is the worst
                // possible answer.
                //
                // ★ A newline in an EXISTING run is refused by construction
                // rather than by a check: `Anchor::Run` is not a box, so plain
                // Enter commits there. That is correct and not a limitation
                // being hidden — `edit_text` replaces the text of ONE show
                // operator, and a show operator cannot contain a line break. A
                // run that should become two lines is a *reflow*, which is a
                // different verb with its own preconditions.
                egui::Event::Key {
                    key: egui::Key::Enter,
                    pressed: true,
                    modifiers,
                    ..
                } => {
                    crate::diag::trace(|| {
                        // ui-text-exempt: diagnostic trace, never displayed.
                        //
                        // ★ Enter is the one keystroke in this handler with TWO
                        // meanings, so its ARRIVAL is worth reporting separately
                        // from its effect. The multi-line work spent a driven
                        // run on *"did the key arrive, or did the branch pick
                        // wrong?"*, which the `text-edit-typing` line cannot
                        // answer: it reports a length, and both failures leave
                        // the length unchanged.
                        format!(
                            "text-edit-enter boxed={} command={}",
                            u8::from(matches!(draft.anchor, Anchor::Box { .. })),
                            u8::from(modifiers.command),
                        )
                    });
                    if matches!(draft.anchor, Anchor::Box { .. }) && !modifiers.command {
                        // ★ `newline`, NOT `insert` — see its docs. `insert`
                        // drops control characters, correctly, and ate this
                        // exact keystroke for one driven run.
                        draft.caret = caret::newline(&mut draft.text, draft.caret);
                        changed = true;
                    } else {
                        commit_into(ctx, &draft, actions);
                        abandon(ctx);
                        return true;
                    }
                }
                _ => {}
            }
        }
    }
    if changed {
        // The selection, published for the harness.
        //
        // ★ A TRACE RATHER THAN A MUTATION, and that is the point of it. The
        // honest way to prove Shift+Right selected three characters is to type
        // over them and see the text shrink — but a driven check runs on the
        // operator's own drawing, and proving a *selection* by making an
        // *edit* is a bad trade. This line carries the two numbers a wrong
        // build would get wrong, so nothing has to be changed to read them.
        //
        // ★★ It reports the EMPTY case too, in its own words rather than by
        // going quiet. Rule 4 — an unshifted move drops the selection — is
        // exactly as important as the selecting, and an absent line cannot be
        // told from a build where the trace stopped being emitted.
        crate::diag::trace_on_change("text-select", || {
            // ui-text-exempt: diagnostic trace, never displayed.
            match caret::range(draft.mark, draft.caret) {
                // ★ The KEY IS NOT REPEATED in the value. `trace_on_change`
                // prints `pdfce-diag <key> <value>`, so a value beginning with
                // the key produces `text-select text-select from=0 …` — which
                // parses, reads as a typo, and was one until this line.
                Some((from, to)) => {
                    let n = to - from;
                    // ui-text-exempt: diagnostic trace, never displayed.
                    format!("from={from} to={to} n={n}")
                }
                // ui-text-exempt: diagnostic trace, never displayed.
                None => format!("none caret={}", draft.caret),
            }
        });
        store(ctx, draft);
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::textedit::TextEditKind;

    /// **The oracle for *"it doesn't type anything in the box when I type"*.**
    ///
    /// Every existing text-edit check seeds the draft through `PDFCE_DIAG_TYPE`,
    /// which is the ONE path that bypasses the event loop — so all of them pass
    /// on a build where real typing is dead. This one drives a real
    /// `egui::Context` with a real `Event::Text` and asserts the draft grew.
    #[test]
    fn a_real_text_event_lands_in_the_draft() {
        let ctx = egui::Context::default();
        store(
            &ctx,
            Draft {
                page: 0,
                kind: TextEditKind::Add,
                anchor: Anchor::Origin { x: 10.0, y: 10.0 },
                text: String::new(),
                caret: 0,
                mark: None,
                seeded: true,
            },
        );
        let mut input = egui::RawInput::default();
        input.events.push(egui::Event::Text("h".to_owned()));
        let mut actions = Vec::new();
        let inner = ctx.clone();
        // ★ A real document, because `typing` now takes one: Up and Down ask
        // the PAGE where the next line is (see `blocks`). This test's own event
        // is a `Text`, which never reaches that path — the document is here to
        // satisfy the signature, and passing a real one rather than inventing a
        // stub is what keeps the test honest if the typing path ever grows a
        // second document read.
        let doc = crate::app::state::open_fixture(crate::app::state::FOUR_PAGES);
        let _ = ctx.run_ui(input, move |c| {
            egui::CentralPanel::default().show(c, |ui| {
                typing(ui, &inner, &doc, true, &mut actions);
            });
        });
        assert_eq!(read(&ctx).map(|d| d.text), Some("h".to_owned()));
        assert_ne!(
            TextEditKind::Edit.command_id(),
            TextEditKind::Add.command_id()
        );
    }

    /// ★★ **Shift+Right selects, and a plain Right drops it** — the two halves
    /// of the selection, driven through the same event loop the keyboard uses.
    ///
    /// A unit test rather than only a driven one, because the driven check
    /// cannot tell "the shell ignored Shift" from "the harness never sent it",
    /// and those live in different repositories. This one is unambiguous: the
    /// event carries `shift: true` by construction.
    #[test]
    fn shift_right_selects_and_a_plain_right_drops_it() {
        let ctx = egui::Context::default();
        store(
            &ctx,
            Draft {
                page: 0,
                kind: TextEditKind::Add,
                anchor: Anchor::Origin { x: 10.0, y: 10.0 },
                text: "abcdef".to_owned(),
                caret: 0,
                mark: None,
                seeded: true,
            },
        );
        let doc = crate::app::state::open_fixture(crate::app::state::FOUR_PAGES);

        let press = |ctx: &egui::Context, doc: &crate::app::state::OpenDoc, shift: bool| {
            // ui-text-exempt: nothing below is displayed; this is a driver.
            let mut input = egui::RawInput::default();
            let modifiers = egui::Modifiers {
                shift,
                ..Default::default()
            };
            input.modifiers = modifiers;
            input.events.push(egui::Event::Key {
                key: egui::Key::ArrowRight,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers,
            });
            let mut actions = Vec::new();
            let inner = ctx.clone();
            let _ = ctx.run_ui(input, move |c| {
                egui::CentralPanel::default().show(c, |ui| {
                    typing(ui, &inner, doc, true, &mut actions);
                });
            });
        };

        press(&ctx, &doc, true);
        press(&ctx, &doc, true);
        let after = read(&ctx).expect("the draft survives a movement");
        assert_eq!(after.caret, 2);
        assert_eq!(
            caret::range(after.mark, after.caret),
            Some((0, 2)),
            "two shifted presses select two characters, from where the caret started"
        );

        press(&ctx, &doc, false);
        let after = read(&ctx).expect("the draft survives a movement");
        assert_eq!(after.caret, 3);
        assert_eq!(
            caret::range(after.mark, after.caret),
            None,
            "an unshifted move drops the selection - rule 4"
        );
    }
}
