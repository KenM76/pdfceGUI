//! # `app::dispatch` — one command in, one effect out
//!
//! The routing table. Every operator gesture that names a command — a
//! ribbon click, a quick-access button, a context-menu item, a keyboard
//! chord, a custom item reporting its own token — arrives here, and this is
//! the **one** place that decides what the application does about it.
//!
//! ## Why one choke point rather than a closure per command
//!
//! `egui_shell` stores an opaque `HandlerToken` and hands it back; it never
//! interprets it. That is what keeps the shell reusable — a registry of
//! closures would force it to name pdfce's state type. The consequence on
//! this side is a single `match`, and the consequence of *that* is the
//! property worth protecting: **a confirmation gate, an undo entry or a
//! trace has exactly one place to go.** Scatter dispatch across as many
//! sites as there are commands and each of those becomes something somebody
//! has to remember at every site.
//!
//! ## Why this is separate from `app::mod`
//!
//! Split out at Phase 3, when `app/mod.rs` reached 1,638 lines against the
//! 1,500-line gate (R2). The seam is a real one rather than arithmetic:
//! `mod.rs` composes a frame — panels in order, canvas, dialogs, then apply
//! — while this file answers *what does this verb do*. The two change for
//! different reasons and are read at different times.
//!
//! The gate's own rationale is the argument for splitting here rather than
//! anywhere that merely counts: the GUI this project replaces reached 25,005
//! lines in one `main.rs`, and two of the defects in `DEFECTS.md` are pairs
//! of lines thousands of lines apart that no reviewer could have been
//! expected to see together.
//!
//! ## ★ The arms route; they do not compute
//!
//! Almost every arm is one line: push an [`Action`], or call the one
//! function in the module that owns the rule. Zoom anchoring lives in
//! `crate::canvas::zoom`, the tool in `crate::canvas::tool`, the print
//! dialog in `crate::dialogs`. The moment an arm starts working out *how* to
//! do something, that rule exists in two places and only one of them will be
//! the one that gets fixed.
//!
//! The few exceptions are marked where they occur, and each is a routing
//! decision rather than a rule: `file.recent` chooses between a parked
//! operand and the newest reachable entry; the panel commands map an id to a
//! panel.

use super::PdfceApp;
use super::actions::Action;
use super::state::Status;

impl PdfceApp {
    /// Turn one invoked command into whatever the application does about it.
    ///
    /// Resolved token → id → [`Self::dispatch_command`], rather than matching
    /// on the raw token number. The numbers are assigned in per-tab blocks in
    /// `crate::shell::commands` and are meaningful only there; duplicating
    /// them here would create a second place to keep in step, and a silent
    /// mis-dispatch is the failure that would result.
    ///
    /// **The id is cloned rather than borrowed**, and it is not an
    /// oversight: the arms below need `&mut self` — a panel to activate, a
    /// dock to reset, a mode to select — and a `&str` borrowed out of
    /// `self.commands` would hold `self` shared for the whole match. One
    /// short allocation per *invoked command* (an operator click, not a
    /// frame) is the right price for arms that can act on the application.
    /// `pub(super)` rather than private: this method moved out of
    /// `app/mod.rs` and its callers stayed. It is deliberately NOT `pub` —
    /// nothing outside `app` may dispatch a command, because the choke
    /// point's whole value is that there is exactly one way in.
    pub(super) fn dispatch_token(
        &mut self,
        ctx: &egui::Context,
        token: egui_shell::commands::HandlerToken,
        actions: &mut Vec<Action>,
    ) {
        let Some(id) = self
            .commands
            .iter()
            .find(|c| c.handler == token)
            .map(|c| c.id.clone())
        else {
            return;
        };
        self.dispatch_command(ctx, &id, actions);
    }

    /// Do whatever this build does about the command named `id`.
    ///
    /// **The one dispatcher.** A ribbon click, a QAT click, a context-menu
    /// click and a keyboard chord all arrive here, which is what makes it
    /// impossible for a chord and a button that share a command to do
    /// different things — the defect `crate::app::keyboard`'s header is
    /// about, closed structurally rather than by agreement.
    ///
    /// **A command with no arm is not an error.** At S2 most of the ribbon is
    /// scaffolding for behaviour that lands at S3 and later, and the
    /// honest thing is to say so once per invocation in the trace rather
    /// than to pretend the click did something. Where a command is *known*
    /// not to be implementable yet, its arm says why in the trace rather
    /// than falling through to the generic line — a reader of a trace from a
    /// machine they cannot see should not have to guess which kind of
    /// nothing happened.
    /// `pub(super)` rather than private: this method moved out of
    /// `app/mod.rs` and its callers stayed. It is deliberately NOT `pub` —
    /// nothing outside `app` may dispatch a command, because the choke
    /// point's whole value is that there is exactly one way in.
    pub(super) fn dispatch_command(
        &mut self,
        ctx: &egui::Context,
        id: &str,
        actions: &mut Vec<Action>,
    ) {
        // ★ **The operator's next act retires the last worded decline.**
        //
        // A decline — "Nothing to zoom to" — is a sentence about *one*
        // gesture, and this is the one place in the application that knows an
        // operator has just invoked something. That makes it the only honest
        // lifetime available: the sentence stands until the next thing they
        // do, and then it is gone.
        //
        // It is deliberately **not** keyed on `edit_epoch` the way the two
        // rule-4 disclosure lines beside it are. A decline changes no
        // document, so the epoch never moves and an epoch-keyed sentence would
        // never retire; and a decline must be **repeatable**, which an epoch
        // key cannot express because nothing changed between the two presses.
        // `crate::app::status::decline`'s header carries the whole argument.
        //
        // Placement above the match is what makes the repeat work: pressing
        // the declining chord twice retires the first sentence here and the
        // arm below records a second one, so two presses are two events rather
        // than one press and one swallowed keystroke.
        //
        // This is still routing rather than computing. The arm hands over a
        // value; it does not decide what a decline is, how long one lives, or
        // what it says — all three live in the module that owns them.
        crate::app::status::decline::retire();

        match id {
            // ★ Open. The command that makes pdfce a reader rather than a
            // viewer of one file.
            //
            // It was registered, drawn on the File tab, drawn on the QAT,
            // bound to Ctrl+O in the keymap — and had no arm, so the only way
            // to open a document was `argv`. That is defect D1's shape with
            // the most consequential verb in the application behind it.
            //
            // The dialog runs HERE, during dispatch, and only its *result*
            // becomes an action. See `crate::app::files` for why that line is
            // where it is, and for the `PDFCE_DIAG_OPEN_PATH` seam that lets
            // a scripted harness answer the dialog without a human — a native
            // dialog is a hard wall for synthetic input, and substituting the
            // answer is the only thing that gets past it.
            // ★ New. One line, and the whole of the arm — which is what a
            // routing table looks like when the rule lives somewhere else.
            //
            // Everything a reader is likely to want is in
            // `crate::app::PdfceApp::new_document` and `crate::app::blank`:
            // where the bytes come from, why the engine has no way to make a
            // document and never will, why the page is A4, why the mode is
            // left alone, and why the dirty-document question is `save_pending`
            // rather than a second rule.
            //
            // Note what it does NOT do: open a dialog. Two of the three
            // reference applications create immediately from a default and
            // only SolidWorks asks — and what SolidWorks asks is *which kind
            // of document*, which pdfce has no analogue for. See
            // `crate::app::blank` §3.
            "file.new" => actions.push(Action::New),
            "file.open" => crate::app::files::raise(crate::app::files::pick_document(), actions),
            // ★ Close. `doc.open` gates the control, so the no-document case
            // is unreachable from the ribbon — and the action handles it
            // anyway, because a customized keymap can reach any command from
            // any state.
            "file.close" => actions.push(Action::Close),
            // ★ **Save a copy.** Registered, on the quick-access toolbar, bound
            // to `Ctrl+S`, printing "(Ctrl+S)" in its own tooltip — and until
            // 2026-08-14 it had **no arm**, so it traced `command-unimplemented`
            // and nothing this shell could author could be written to disk at
            // all. D1's shape, with the one verb that makes an editor an editor.
            //
            // One line, because the rule lives in `crate::app::save`: what the
            // copy is called, which mode the bytes are written in and why it is
            // not up for renegotiation, which `SaveOptions` were chosen, what
            // happens to `edit_epoch` (nothing, in both directions), and what
            // the operator sees when it fails.
            //
            // ★ **It does NOT open the picker here**, and that is the one thing
            // worth knowing at this site — `file.open` two arms above does. The
            // difference is not inconsistency: `crate::app::files::pick_save_path`
            // carries a **frame-timing requirement** that dispatch cannot
            // guarantee, because `PdfceApp::central` dispatches the canvas's
            // context-menu tokens from inside `egui::CentralPanel::show`, and a
            // native modal opened mid-layout blocks the frame it is being drawn
            // in. The apply phase is always outside every closure, so the picker
            // runs there. `Action::SaveCopy`'s own docs carry the full argument,
            // including why it needs no operand where `Action::Open` needs one.
            "file.save_copy" => actions.push(Action::SaveCopy),
            // ★ Print, and the one command in this match that raises no
            // action.
            //
            // Everything else here funnels through `Action` so that a
            // mutation is applied once, after the frame, in one place that
            // an undo log can be built from. Printing mutates nothing — it
            // has nothing to contribute to that log, and an action variant
            // could only be serviced by reaching back into the dialog for
            // the state it holds anyway.
            //
            // The funnel's actual *reason* — do no irreversible work in the
            // middle of laying out a frame — is honoured inside the dialog:
            // the commit button sets a flag, and the spool runs after the
            // window's closure has returned. Paper is as irreversible as it
            // gets; it is the rule being kept, not the mechanism.
            "file.print" => self.dialogs.open_print(&self.status),
            // About. Takes no `&self.status`, unlike every other dialog on
            // this list, and that asymmetry is the point rather than an
            // omission: it describes the program, so it opens with nothing
            // loaded and stays open when the document closes. The guard that
            // would make it document-scoped is deliberately absent at BOTH
            // ends — here and in `DialogsState::show` — because a guard in
            // one place and not the other is how a dialog ends up opening and
            // vanishing on the same frame.
            "file.about" => self.dialogs.open_about(),
            // ★ Recognise text. A dialog rather than an immediate action, and
            // rather than the `file.copy_document_text` shape one arm below.
            //
            // Three things had to be true of this arm and none of them can be
            // true of a `match` limb that just does the work:
            //
            // 1. **It must not block.** Copying the document's text blocks the
            //    UI thread on purpose — 331–449 ms, a stutter. Recognition
            //    rasterizes a page at 300 DPI and runs two neural networks over
            //    it, which is *seconds*, and a window frozen for seconds is
            //    indistinguishable from a hung program. The work is on a thread;
            //    see `crate::ocr::Job`.
            // 2. **It must disclose before it writes.** Every word OCR produces
            //    is a guess and this recogniser scores none of them, so the
            //    operator reads what it inferred while still holding the ability
            //    not to save it. That needs a surface with three states.
            // 3. **It must ask where the result goes.** The operator's rule is
            //    that Read may produce a new document and may not modify this
            //    one, enforced at the save — and this is the first write to disk
            //    this shell performs, so it is the first place that can bite.
            //
            // No mode check, deliberately. OCR is offered in Read exactly as in
            // Edit: `app::modes::capability` governs canvas *gestures*, and OCR
            // is not a gesture. The rule it has to honour is about what a save
            // may overwrite, and that is enforced by the destination being a
            // path the operator names — which holds in every mode without any
            // mode being consulted.
            "file.ocr" => self.dialogs.open_ocr(&self.status),
            // ★ Recent. The operand comes from the `recent_files` custom item
            // (see `Self::ribbon_band`), which parked it before returning this
            // command's token.
            //
            // Reaching this arm with nothing parked is not an error and not
            // unreachable: an operator may bind a chord to `file.recent` or
            // put it on their quick-access toolbar, neither of which draws a
            // menu. The defined answer is **the newest document that can be
            // seen right now** — which is what "recent" means with no further
            // qualification, and which skips an entry on a drive that is not
            // connected rather than reporting a failure the operator did not
            // cause.
            crate::shell::commands::FILE_RECENT => {
                let path = self
                    .recent_choice
                    .take()
                    .or_else(|| self.recent.newest_present(std::time::Instant::now()));
                match path {
                    Some(path) => actions.push(Action::Open(path)),
                    None => crate::diag::trace(|| {
                        // ui-text-exempt: diagnostic trace, never displayed.
                        "recent-declined reason=nothing-reachable".to_owned()
                    }),
                }
            }
            // ★ The three Phase 3 navigation verbs.
            //
            // Each is one call because the rule each obeys lives in
            // `canvas::zoom` or `canvas::tool`, not here. This match is a
            // routing table; the moment it starts computing an anchor or
            // deciding what a drag means, the same rule exists in two places
            // and one of them will be the one that gets fixed.
            // ★ …and this is the one arm whose RETURN VALUE matters.
            //
            // `ZoomOutcome` is `#[must_use]` precisely because its declining
            // variants are the point, and this arm used to discard it with a
            // `let _ =` — which is how "there is nothing to zoom to" became
            // "the command did nothing", the difference between a control that
            // declines and one that looks broken. `FEATURES.md` recorded the
            // gap as *traced and greyed but never worded*.
            //
            // The outcome is now carried into `status::decline`, which decides
            // whether it is a decline at all (a ceiling-clamped zoom is a
            // partial grant and is not worded), which sentence it gets, and
            // how long it lives. This arm decides none of that; it routes.
            //
            // The command is gated on `selection.bounds`, so a pressable
            // control usually has something to frame. The no-bounds answer is
            // still reachable two ways, and the second is why the sentence
            // exists: **by chord**, since a keymap reaches any command from any
            // state, and **in the race** where the bounds evaporate between the
            // frame that drew the enabled control and the frame that applied
            // it. In that second case the operator clicked something that was
            // offered to them and got nothing, which is exactly the situation
            // that must not be answered with silence.
            "view.zoom_selection" => {
                if let Status::Open(doc) = &mut self.status {
                    let outcome = crate::canvas::zoom::zoom_to_selection(
                        ctx,
                        doc,
                        crate::canvas::CANVAS_MARGIN,
                        actions,
                    );
                    crate::app::status::decline::record(outcome);
                }
            }
            // Arms; does not act. The canvas disarms it when the drag ends,
            // so there is no "turn it off" arm to write.
            "view.zoom_region" => crate::canvas::zoom::arm_region_zoom(ctx),
            // Toggles, and returns the tool now chosen — which is discarded
            // here because the pressed state is published from `conditions`
            // by asking `tool::selected`, not from a copy kept in the app.
            "view.tool_hand" => {
                let _ = crate::canvas::tool::toggle_hand(ctx);
            }
            // ★ **The text tool**, and it is the `tool_hand` arm's twin down to
            // the discarded return value — the pressed state is published from
            // `conditions` by asking `tool::selected`, never from a copy kept on
            // the app, because a shadow copy is how a ribbon comes to say Text
            // while the canvas marquees.
            //
            // ★ **No capability check, and the absence is the decision** rather
            // than an oversight — which is worth saying here because every arm
            // around it has one. `markup_for_command` declines on
            // `author_markup`, `measure_for_command` on `author_measure`, and
            // the obvious symmetry would be a third. There is nothing to put
            // there: selecting text authors nothing, so `canvas::tool::
            // retire_forbidden` permits this tool in every mode and a decline
            // here would contradict it. The full argument is at that function's
            // `Select | Hand | Text` arm; in one line, it is the operator's own
            // *copying is not authoring* ruling of 2026-08-14, which already
            // moved both text-copy verbs off the authoring tab.
            //
            // It therefore has no decline trace either, and that is consistent
            // rather than lax: a trace line exists to say *which* nothing
            // happened, and there is no state in which pressing this does
            // nothing.
            "view.tool_text" => {
                let _ = crate::canvas::tool::toggle_text(ctx);
            }
            // ★ **The markup shape tools — one arm for all four.**
            //
            // The same shape as the page-display radio below, for the same
            // reason: the id *is* the operand, and
            // `crate::shell::commands::markup_for_command` is the single
            // binding between an id and a kind. Four literal arms would be
            // four places to forget the fifth.
            //
            // **It arms a tool; it authors nothing.** The canvas draws the
            // band, the release raises `Action::CommitMarkup`, and pressing
            // the armed button again puts the pen down — `arm_markup`
            // toggles on the same kind and re-arms on a different one, so a
            // second press of Rectangle leaves the select tool rather than
            // arming Rectangle twice.
            //
            // The returned tool is discarded for the reason the `tool_hand`
            // arm above states: the pressed state is published from
            // `conditions` by asking `tool::selected`, never from a copy
            // kept on the app. A shadow copy is how a ribbon comes to say
            // Rectangle while the canvas is selecting.
            // ★ …and it declines in a mode that does not author markup.
            //
            // Unreachable through the shipped manifest — Read is shown File and
            // View alone, and no chord binds a `markup.*` id — so this is the
            // belt to `retire_forbidden`'s braces, which covers only the
            // *transition* into such a mode and cannot cover an arming that
            // happens while already in one. A customized manifest that binds a
            // chord to Rectangle is all it takes to reach this.
            //
            // Declining rather than arming-and-refusing is what keeps the
            // cursor honest: an armed markup tool paints a crosshair over every
            // page, which promises a drawing gesture `press_kind` has already
            // decided not to give. The same argument `retire_forbidden` makes,
            // at the other end of the tool's life.
            //
            // This is still routing rather than computing: the arm asks one
            // published predicate and either calls the one function or does
            // not. It does not work out *what* a markup is, and the trace
            // spelling matches the `mode.*` arm below, which already declines.
            // ★ **The measure tools — one arm for all four.**
            //
            // The same shape as the markup arm below, and it declines in a mode
            // that does not author dimensions for the same reasons — see there.
            // Read grants neither; **Review grants this one and not that one**,
            // which is why they are two capabilities rather than an "authoring"
            // flag.
            //
            // **It arms a tool; it authors nothing.** A ce dimension is placed
            // by clicks that `crate::canvas::measure` takes, and only the pick
            // that completes one raises an `Action`.
            // ★ **Finish** — the ribbon half of the radius/diameter tool's
            // ending, and the one `measure.*` command that is not a tool.
            //
            // It must sit ahead of the arm below rather than inside it:
            // `measure_for_command` maps ids to *kinds*, this id names no kind,
            // and if it ever did, pressing Finish would toggle the tool off
            // (`arm_measure`'s same-kind-retires rule) instead of committing.
            //
            // The arm routes and does not compute. Everything about what a
            // finish *is* — whether there is a fit, which page it belongs to,
            // which group it joins, emptying the pick set afterwards — lives in
            // `canvas::measure::finish`, which is the same function the
            // canvas's double-click ending reaches. One commit path, two
            // entrances; a second derivation here is exactly how the two
            // endings would come to author different dimensions.
            //
            // The capability check mirrors the tool arm below. It is
            // unreachable through the shipped manifest — Read is shown File and
            // View alone, and no chord binds a `measure.*` id — but a
            // customized manifest can bind a chord to anything, and a mode that
            // cannot author dimensions must not author one because the pick set
            // predates the mode change.
            //
            // Both refusals are traced, and they are traced separately: "the
            // mode says no" and "there was nothing to finish" are different
            // facts, and a reader of a trace from a machine they cannot see
            // should not have to guess which kind of nothing happened.
            "measure.finish" => {
                if !self.capabilities().author_measure {
                    crate::diag::trace(|| {
                        // ui-text-exempt: diagnostic trace, never displayed.
                        format!("command-declined id={id} reason=mode-cannot-author-measure")
                    });
                } else if !crate::canvas::measure::finish(ctx, actions) {
                    crate::diag::trace(|| {
                        // ui-text-exempt: diagnostic trace, never displayed.
                        //
                        // Reachable only by a chord or a customized manifest:
                        // the ribbon control is greyed unless there is a
                        // non-degenerate fit, by the same predicate `finish`
                        // itself asks.
                        format!("command-declined id={id} reason=no-circle-fit-to-finish")
                    });
                }
            }
            id if crate::shell::commands::measure_for_command(id).is_some() => {
                if !self.capabilities().author_measure {
                    crate::diag::trace(|| {
                        format!(
                            // ui-text-exempt: diagnostic trace, never displayed.
                            "command-declined id={id} reason=mode-cannot-author-measure"
                        )
                    });
                } else if let Some(kind) = crate::shell::commands::measure_for_command(id) {
                    let _ = crate::canvas::tool::arm_measure(ctx, kind);
                }
            }
            // ★ **The three text-markup commands — one arm for all three.**
            //
            // The same one-arm shape the two families below have, and for the
            // same reason: the id IS the operand, and
            // `crate::shell::commands::text_mark_for_command` is the single
            // binding between an id and a `TextMarkKind`.
            //
            // ★ **It authors immediately; it arms nothing.** That is the whole
            // difference from the `markup_for_command` arm below it, and it is
            // the interaction decision recorded at
            // `canvas::markup::text`'s header §1: these kinds mark **an existing
            // text selection**, which is Acrobat's answer and needs no tool, no
            // gesture and no `CanvasTool` variant. The operand is on the
            // document, visible as a wash, at the moment the button is pressed.
            //
            // It must sit **ahead** of the `markup_for_command` arm. Both are
            // guard arms on `markup.*` ids and `match` takes the first that
            // matches; the two mappings are asserted disjoint in both directions
            // (`shell::commands::mapping`), so the order is belt to that
            // braces — but the order is also the cheaper of the two guarantees
            // and costs nothing to state.
            //
            // Two refusals, traced separately, because they have different
            // answers and a reader of a trace from a machine they cannot see
            // should not have to guess which nothing happened:
            //
            // * **the mode cannot author markup** — unreachable through the
            //   shipped manifest (Read is shown File and View alone), and
            //   reachable from a chord in a customized one, exactly as the
            //   markup and measure arms below;
            // * **there was nothing markable** — no selection, or one made
            //   against a revision that has since moved. The ribbon control is
            //   greyed in both cases, by the same `selection.text` condition the
            //   rule here asks about, so this is reachable only by a chord.
            //
            // The arm still routes rather than computes: `markup::text::mark` is
            // a pure function that owns every rule about which selection is
            // eligible and what a stale one means, and this reads one published
            // capability, calls it once, and pushes what comes back.
            id if crate::shell::commands::text_mark_for_command(id).is_some() => {
                let Some(kind) = crate::shell::commands::text_mark_for_command(id) else {
                    return;
                };
                if !self.capabilities().author_markup {
                    crate::diag::trace(|| {
                        format!(
                            // ui-text-exempt: diagnostic trace, never displayed.
                            "command-declined id={id} reason=mode-cannot-author-markup"
                        )
                    });
                    return;
                }
                // Every state but `Open` is *no document*, and therefore no
                // selection to mark and no page for the action to name — the
                // same hazard `measure.finishable` is published inside the
                // `Status::Open` arm to avoid. Written as an `if let` with one
                // fallback rather than an exhaustive `match`, so that a sixth
                // failure state (`Unsupported`, `NeedsPassword`, …) does not
                // arrive here asking to be classified: none of them holds a
                // document, and that is the only property this arm reads.
                let selected = if let Status::Open(doc) = &self.status {
                    crate::canvas::markup::text::mark(
                        kind,
                        doc.text_selection.as_ref(),
                        doc.edit_epoch,
                    )
                } else {
                    Err(crate::canvas::markup::text::Refusal::NoSelection)
                };
                match selected {
                    Ok(action) => {
                        if let Action::CommitTextMarkup { page, quads, .. } = &action {
                            crate::canvas::markup::text::trace_commit(kind, *page, quads.len());
                        }
                        actions.push(action);
                    }
                    Err(reason) => crate::canvas::markup::text::decline(kind, reason),
                }
            }
            // ★ **Finish** — the ribbon half of the vertex tools' ending, and the
            // one `markup.*` command that is neither a tool nor a mark.
            //
            // It is `measure.finish`'s twin, deliberately down to the shape of
            // this arm, because it answers the identical problem: PolyLine and
            // Polygon are runs of clicks with no natural end, exactly as the
            // radius/diameter pick set has none, and the operator settled that
            // on 2026-08-14 with **two endings through one commit path**. A
            // double-click on the canvas is the other half and is the one most
            // operators will use; this is the discoverable one, and the one that
            // works when the last vertex sits somewhere awkward to double-click.
            //
            // It must sit ahead of the `markup_for_command` arm below rather
            // than inside it, for the reason `measure.finish` states in its own
            // words: that mapping takes ids to *kinds*, this id names no kind,
            // and if it ever did, pressing Finish would toggle the tool off
            // (`arm_markup`'s same-kind-retires rule) instead of committing.
            //
            // The arm routes and does not compute. Everything about what a
            // finish *is* — whether the run is long enough for its kind, which
            // page it belongs to, emptying it afterwards — lives in
            // `canvas::markup::vertex::finish`, which is the same commit path
            // the canvas's double-click reaches. One commit path, two entrances;
            // a second derivation here is exactly how the two endings would come
            // to author different annotations.
            //
            // Both refusals are traced separately, because "the mode says no"
            // and "there was nothing to finish" are different facts with
            // different answers, and a reader of a trace from a machine they
            // cannot see should not have to guess which nothing happened.
            "markup.finish" => {
                if !self.capabilities().author_markup {
                    crate::diag::trace(|| {
                        // ui-text-exempt: diagnostic trace, never displayed.
                        format!("command-declined id={id} reason=mode-cannot-author-markup")
                    });
                } else if !crate::canvas::markup::vertex::finish(ctx, actions) {
                    crate::diag::trace(|| {
                        // ui-text-exempt: diagnostic trace, never displayed.
                        //
                        // Reachable only by a chord or a customized manifest:
                        // the ribbon control is greyed unless there is a run
                        // long enough for its kind, by the same predicate
                        // `finish` itself asks.
                        format!("command-declined id={id} reason=no-vertex-run-to-finish")
                    });
                }
            }
            id if crate::shell::commands::markup_for_command(id).is_some() => {
                if !self.capabilities().author_markup {
                    crate::diag::trace(|| {
                        format!(
                            // ui-text-exempt: diagnostic trace, never displayed.
                            "command-declined id={id} reason=mode-cannot-author-markup"
                        )
                    });
                } else if let Some(kind) = crate::shell::commands::markup_for_command(id) {
                    let _ = crate::canvas::tool::arm_markup(ctx, kind);
                }
            }
            // ★ **The four positions of View ▸ Page display.**
            //
            // One arm for the whole radio, because the id *is* the operand:
            // `crate::shell::commands::page_display_for_command` is the single
            // binding between a command id and a
            // `crate::viewer::PageDisplay`, and its inverse is what publishes
            // the `selected:` condition that renders the active position
            // pressed. Four arms would be four places for that mapping to be
            // spelled, and the fifth mode would be added to three of them.
            //
            // An id the mapping does not know cannot reach here — the match is
            // gated on the same function — so there is no "unknown mode" arm
            // to write.
            id if crate::shell::commands::page_display_for_command(id).is_some() => {
                if let Some(display) = crate::shell::commands::page_display_for_command(id) {
                    actions.push(Action::SetPageDisplay(display));
                }
            }
            // ★ **The three View ▸ Display chrome toggles**, one arm, for the
            // identical reason the page-display radio has one: the id IS the
            // operand, `chrome_for_command` is the single binding between an
            // id and a `ViewChrome`, and its inverse is what publishes the
            // `selected:` condition that renders each one pressed. Three arms
            // would be three places to spell one mapping.
            //
            // Unlike the radio these are independent — a click means "flip
            // this one", not "select this position" — so the action carries
            // which toggle and the apply reads the current value. Reading it
            // *there* rather than here is what keeps the dispatcher free of
            // `self.status`: a chord can reach this command with no document
            // open, and the arm must not have to decide what that means.
            id if crate::shell::commands::chrome_for_command(id).is_some() => {
                if let Some(chrome) = crate::shell::commands::chrome_for_command(id) {
                    actions.push(Action::ToggleViewChrome(chrome));
                }
            }
            "view.zoom_in" => actions.push(Action::ZoomIn),
            "view.zoom_out" => actions.push(Action::ZoomOut),
            // `ZoomTo(1.0)`, not `Fit(FitMode::None)`. The latter only stops the
            // per-frame re-fit and leaves the zoom where it was, so this
            // control used to pin whatever magnification happened to be
            // showing while promising one PDF point per screen point.
            "view.zoom_actual" => actions.push(Action::ZoomTo(1.0)),
            "view.zoom_fit_page" => actions.push(Action::Fit(crate::viewer::FitMode::Page)),
            "view.zoom_fit_width" => actions.push(Action::Fit(crate::viewer::FitMode::Width)),
            "view.next_page" => actions.push(Action::NextPage),
            "view.prev_page" => actions.push(Action::PrevPage),
            // This control was drawn and enabled from the moment the ribbon
            // landed, and did nothing — a live instance of D1's shape: an
            // affordance that looks available and is inert. It became
            // wirable when `RenderKey` gained `annotations`.
            "view.show_annotations" => actions.push(Action::ToggleAnnotations),
            // ★ The ribbon's Delete — the contextual Format tab's one command.
            //
            // The id is `format.delete`, not `edit.delete`: `RIBBON_IA.md`
            // §5.8 puts Delete on the **Format** tab, which is contextual and
            // appears only while something is selected, and
            // `shell::commands` registers exactly that id gated on
            // `selection.any`. There is no `edit.delete` in this build, and
            // adding an arm for one would be an arm no token can ever reach —
            // dead code wearing a design pattern, which is what the
            // no-placeholders invariant forbids.
            //
            // It became wirable when the selection moved onto `OpenDoc`: this
            // function has no `egui::Context`, so while the selection lived in
            // `egui::Memory` there was no route from a ribbon click to the
            // thing it was about to delete. That is the whole of why the
            // control has been drawn-but-unwired until now.
            //
            // **The rule is not restated here.**
            // `SelectionState::deletable_objects_on` decides what a Delete may
            // act on — Object rung only, ascending, de-duplicated, this page
            // only — and the canvas's Delete key reads the same method. Two
            // statements of a destructive rule is one too many.
            //
            // An empty list raises nothing rather than an empty action the
            // engine would have to refuse. That is reachable in practice: the
            // Format tab is visible whenever *anything* is selected, including
            // at a rung whose delete verb does not exist yet.
            "format.delete" => {
                if let Status::Open(doc) = &self.status {
                    let page = doc.view.page_index;
                    let objects = doc.selection.deletable_objects_on(page);
                    if !objects.is_empty() {
                        actions.push(Action::DeleteSelection { page, objects });
                    }
                }
            }
            // ★ **The two text-copy verbs — registered since 2026-08-14 and
            // dead until now.**
            //
            // They were drawn on File ▸ Export, `Ctrl+Shift+C` was bound to the
            // page one, and neither had an arm: a live control that does
            // nothing, which is defect D1's shape and which this project's
            // own `both_text_copy_commands_are_offered_by_every_mode` test could
            // not see, because offering a command and implementing it are
            // different facts.
            //
            // What made them wirable was the per-page extraction cache
            // (`app::cache::PageTextCache`) arriving for canvas text selection.
            // Before it, `file.copy_page_text` had no cheap route to one page's
            // text: `EditSession::find_text_with` is the only text verb on the
            // session, it needs `&mut`, and it walks the **whole document**.
            //
            // ★ Both arms read `page_text()` / `extract_*_view`, so the string
            // an operator copies from the ribbon and the string a canvas
            // selection copies come from **one** extraction of one revision.
            // Two paths to "the text of this page" is how a Copy and a
            // selection come to disagree about what is on it.
            //
            // Neither raises an `Action`: a clipboard write touches no document
            // and needs no frame boundary — the same call `file.print` makes,
            // for the same stated reason. `canvas::textsel::copy` is the one
            // place the clipboard is written and the one place a copy is traced.
            "file.copy_page_text" => {
                if let Status::Open(doc) = &self.status {
                    match doc.page_text() {
                        // `plain_text()` rather than `sourced_text()`: it
                        // carries the engine's derived word spaces and line
                        // breaks, so a copied page reads as a page. `sourced_`
                        // is the honest lower bound for a *test* asserting what
                        // the file provides, and it would paste as one
                        // unbroken word.
                        Some(text) => crate::canvas::textsel::copy(
                            ctx,
                            &text.plain_text(),
                            // ui-text-exempt: diagnostic trace field, never displayed
                            "page",
                        ),
                        None => {
                            // ★ The engine's own reason where there is one, and
                            // a distinct token where there is not.
                            //
                            // Three states reach here and they are three
                            // different facts: the page's content stream would
                            // not walk (`detail=` carries `pdfce-core`'s error),
                            // there is no such page at all, and — a fourth,
                            // handled by `copy` rather than here — the page
                            // extracted fine and has no text on it. A reader of
                            // a trace from a machine they cannot see should not
                            // have to guess which kind of nothing happened;
                            // that is the same argument `objects-unavailable`
                            // makes one module over.
                            let detail = doc.page_text_failure().map(|e| e.clone());
                            crate::diag::trace(|| match &detail {
                                // ui-text-exempt: diagnostic trace, never displayed
                                Some(reason) => format!(
                                    "command-declined id={id} reason=extract-failed \
                                     detail={reason:?}"
                                ),
                                // ui-text-exempt: diagnostic trace, never displayed
                                None => {
                                    format!("command-declined id={id} reason=no-such-page")
                                }
                            });
                        }
                    }
                }
            }
            // The whole-document twin. It really can block the window on a long
            // file — its own tooltip says so — because
            // `extract_document_view` walks every page, which `crate::find`
            // measured at 331–449 ms on this project's fixtures. That cost is
            // paid here and nowhere else: it is a verb the operator invoked
            // once, not a per-frame derivation, which is exactly the line the
            // page-level cache exists to draw.
            //
            // Deliberately NOT cached: a document-wide extraction keyed on the
            // edit epoch would hold the whole document's text alive for the life
            // of the session to serve a command pressed at most a handful of
            // times.
            "file.copy_document_text" => {
                if let Status::Open(doc) = &self.status {
                    match pdfce_core::text_extract::extract_document_view(
                        // The SESSION's revision, as everywhere else: the
                        // operator is copying the document they are looking at,
                        // unsaved edits included (decision 018).
                        &doc.session.view(),
                        &pdfce_core::text_extract::ExtractOptions::default(),
                    ) {
                        Ok(text) => crate::canvas::textsel::copy(
                            ctx,
                            &text.plain_text(),
                            // ui-text-exempt: diagnostic trace field, never displayed
                            "document",
                        ),
                        Err(e) => crate::diag::trace(|| {
                            // ui-text-exempt: diagnostic trace, never displayed
                            format!("command-declined id={id} reason=extract-failed detail={e}")
                        }),
                    }
                }
            }
            // ★ Find. `Ctrl+F`, and the status bar's Find toggle.
            //
            // A **toggle**, not a show: Ctrl+F is the chord every application
            // in the class uses to open a find bar, and the operator whose
            // fingers already know it expects the second press to put the
            // canvas back. That is the opposite of `file.properties`
            // immediately below, which is deliberately *idempotent* — and the
            // difference is not inconsistency. Properties is offered from a
            // context menu to *describe the row just clicked*, so a second
            // invocation that hid the description would be actively hostile;
            // Find is offered from a chord whose whole idiom is a toggle.
            //
            // Raises **no action**. Opening a bar changes no document and
            // needs no frame boundary, exactly as mounting a panel does not —
            // the funnel is for work that touches a document or that must not
            // happen mid-frame, and this is neither. What *does* go through
            // the funnel is the search itself; see `crate::find`.
            //
            // The command is gated on `doc.pages`, so the ribbon and the
            // status bar cannot reach it without a document. A customized
            // keymap can, which is why the no-document case is answered here
            // rather than assumed away: the bar would draw nothing over an
            // empty shell, so opening it would be a control the operator
            // cannot see, and a trace line is the honest response.
            "edit.find" => {
                if matches!(self.status, Status::Open(_)) {
                    let open = self.find.toggle();
                    crate::diag::trace(|| {
                        // ui-text-exempt: diagnostic trace, never displayed in the UI
                        format!("find-toggled open={open}")
                    });
                } else {
                    crate::diag::trace(|| {
                        // ui-text-exempt: diagnostic trace, never displayed in the UI
                        "find-declined reason=no-document".to_owned()
                    });
                }
            }
            // ★ The Properties panel. See [`Self::show_panel`] for the
            // mount-versus-nothing decision, which is the only interesting
            // part of this.
            //
            // `file.properties` rather than a `view.panel_*` id because
            // `RIBBON_IA.md` §5.1 puts Properties in File ▸ Document — it
            // describes the document, not the screen — and
            // `crate::panels::Panel::command_id` is the one place that
            // binding is written down.
            "file.properties" => self.show_panel(crate::panels::Panel::Properties),
            // ★ The Comments panel, and its id is the interesting part.
            //
            // `markup.comments` rather than `view.panel_comments`, which is
            // what `crate::app::modes::defaults` named for the whole time the
            // panel did not exist. `RIBBON_IA.md` §5.2 lists Comments among
            // View ▸ Panels' toggles AND §5.5 gives Markup a `Comments` group;
            // §7's migration map settles it by naming the control —
            // `Review ▸ Comments ▸ Comments` → `Markup ▸ Comments`. A ruling
            // about one control beats a list that merely contains its name.
            //
            // Unlike `view.panel_forms`, this needed no move and no tab
            // argument: Comments mounts in Review and Edit only, and both are
            // shown the `markup` tab, so no mode can mount this panel without
            // being able to reopen it. Forms had the opposite problem.
            //
            // The command has been registered and drawn since the Markup tab
            // was built (`shell::commands`, token 540) with nothing behind it.
            // This arm is the body arriving, which is why none of the five
            // registration obligations apply here.
            "markup.comments" => self.show_panel(crate::panels::Panel::Comments),
            // ★ **The panel toggles — one arm for the whole family.**
            //
            // `view.panel_bookmarks|_layers|_signatures|_objects|_forms` and
            // `file.fonts`. Registered and drawn since the ribbon landed with
            // **nothing behind them** — this arm is the body arriving, which is
            // why none of the five registration obligations apply.
            //
            // `Panel::from_command_id` is the single binding between an id and
            // a panel, exactly as `markup_for_command` and `measure_for_command`
            // are for their families, so there is no second table here to drift.
            //
            // # ★ Placement below the two literal arms is load-bearing
            //
            // `from_command_id` also answers for `file.properties` and
            // `markup.comments`, because they name panels too. A `match` takes
            // the first arm that matches, so those two are claimed above by
            // their own literals and never reach this guard — which is exactly
            // right, because **they are not toggles**. `file.properties` is
            // offered by the Objects row context menu to describe the row just
            // clicked, and a second invocation that closed the description
            // would be, in that test's own word, hostile. See
            // [`Self::toggle_panel`] for the rule this distinction expresses:
            // a control asking *"is this panel open?"* toggles; a control
            // asking *"tell me about this thing"* shows.
            //
            // Moving this arm above either of them would silently turn both
            // into toggles, which is why the ordering is written down rather
            // than left to be noticed.
            id if crate::panels::Panel::from_command_id(id).is_some() => {
                if let Some(panel) = crate::panels::Panel::from_command_id(id) {
                    self.toggle_panel(panel);
                }
            }
            // ★ Reset layout. `ResetScope::All`, and the scope is a decision.
            //
            // `RIBBON_IA.md`'s rule is why a scope exists at all: *"an
            // operator who only wanted the right dock back must not lose
            // their left one."* Honouring that properly needs a **chooser**,
            // and this build has no modal, no popup and no split-button
            // affordance to put one in — see the note on
            // `crate::text::commands::view_reset_layout`, which used to
            // promise the choice in its tooltip and no longer does.
            //
            // Given one button and no chooser, `All` is the only scope whose
            // behaviour matches the words on it: a control named "Reset
            // layout" that reset half the layout would be the more surprising
            // of the two failures. It is also the least destructive it looks:
            // `Modes::reset` restores *this mode's* default and leaves every
            // other mode's saved workspace alone.
            //
            // What a chooser needs, so the next hand does not have to
            // re-derive it: three commands (`view.reset_layout_left`,
            // `_right`, `_all`) with their own `CommandText`, an
            // `egui_shell::manifest::Item` kind that renders a split button
            // or a submenu, and this arm becoming three that pass the
            // matching `ResetScope`. `ResetScope::ALL` already lists them
            // narrowest-first, in the order such a menu should offer them.
            "view.reset_layout" => {
                let scope = egui_shell::layout::ResetScope::All;
                let changed = self.modes.reset(
                    scope,
                    &mut self.dock,
                    &mut self.layout,
                    &self.panel_registry,
                );
                crate::diag::trace(|| {
                    format!(
                        // ui-text-exempt: diagnostic trace, never displayed.
                        "layout-reset scope={scope} changed={changed}"
                    )
                });
            }
            // ★ The mode selector's three keyboard positions.
            //
            // `RibbonState::set_mode`'s own doc commissions exactly this:
            // *"This is what an application calls when the operator presses
            // the Ctrl+1 its manifest bound to `mode.read` — the shell
            // reports the command's token, the application dispatches it,
            // and dispatching it means calling this."*
            //
            // Wired here because the keymap route now reaches it. The mode
            // ids are the command ids without their `mode.` prefix, which is
            // the manifest's own convention — see
            // `crate::shell::manifest::built_in`'s mode list beside its
            // keymap — and an id the manifest does not declare is declined
            // rather than adopted, so a customized keymap naming a fourth
            // mode cannot put the ribbon into a state it has no tab list for.
            //
            // Nothing else happens here: the dock follows on the same frame,
            // in `Self::docks`, which compares `ribbon.mode()` against
            // `modes.active()` and moves the workspace across. One place does
            // that, and it must stay one place — see its ★ comment on why the
            // order of *record* and *restore* is load-bearing.
            "mode.read" | "mode.review" | "mode.edit" => {
                if let Some(mode) = id.strip_prefix("mode.") {
                    if self.modes.is_known(mode) {
                        self.ribbon.set_mode(mode.to_owned());
                    } else {
                        crate::diag::trace(|| {
                            format!(
                                // ui-text-exempt: diagnostic trace, never displayed.
                                "command-declined id={id} reason=mode-not-declared"
                            )
                        });
                    }
                }
            }
            other => {
                crate::diag::trace(|| {
                    format!(
                        // ui-text-exempt: diagnostic trace, never displayed.
                        "command-unimplemented id={other}"
                    )
                });
            }
        }
    }
}
