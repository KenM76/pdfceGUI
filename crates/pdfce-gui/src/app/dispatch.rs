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
            "file.open" => crate::app::files::raise(crate::app::files::pick_document(), actions),
            // ★ Close. `doc.open` gates the control, so the no-document case
            // is unreachable from the ribbon — and the action handles it
            // anyway, because a customized keymap can reach any command from
            // any state.
            "file.close" => actions.push(Action::Close),
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
            "view.zoom_selection" => {
                if let Status::Open(doc) = &mut self.status {
                    // The command is gated on `selection.bounds`, so a
                    // pressable control always has something to frame. This
                    // still handles the no-bounds answer, because a keymap
                    // can reach any command from any state and the honest
                    // response there is "do nothing", not "frame nothing".
                    let _ = crate::canvas::zoom::zoom_to_selection(
                        ctx,
                        doc,
                        crate::canvas::CANVAS_MARGIN,
                        actions,
                    );
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
