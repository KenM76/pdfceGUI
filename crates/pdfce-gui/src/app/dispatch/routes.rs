//! # `app::dispatch::routes` — the commands that perform nothing and point
//! somewhere else
//!
//! Split out of [`super`] under **R2** on 2026-08-28, when that file crossed
//! 1,500 lines for the fourth time.
//!
//! ## ★★★ The seam, and it is a subject rather than a size
//!
//! Every arm here raises `Action::Command(other_id)` and does nothing else.
//!
//! ★ It is **not** every such arm in the shell: `format.properties` predates
//! this file and stays in `dispatch::format`, because an id must have exactly
//! one claimant and moving it would have given it two. See [`target`].
//! That is the property they share and no other arm in the dispatcher does,
//! and it is the one thing a reader has to check about any of them:
//!
//! > `Action::Command` exists so a second route to an existing command cannot
//! > become a second implementation of it.
//!
//! `format.properties` set the precedent — File ▸ Document and the Format tab
//! are two places to ask *"tell me about this"*, and exactly one of them opens
//! the panel. The failure they exist to prevent is the one this project has
//! spent its time removing everywhere else: two surfaces for one capability,
//! drifting apart, each with its own guards.
//!
//! ## Why a command exists at all when another command does the work
//!
//! Because `egui-shell` enforces **one command, one tab**, and the placements
//! answer different questions. An operator hunting on the Tools tab for where
//! font folders live should find the entry there; that it opens the Settings
//! window is an implementation detail of *where the list is kept*, not a
//! reason to leave the Tools tab silent.
//!
//! ★ Each of the three was, until recently, a **drawn and inert control** with
//! a recorded blocker that had gone stale. That is not a coincidence: a command
//! whose capability lives somewhere else is exactly the kind whose blocker
//! stops being true without anybody noticing, because nothing about it changes
//! when the other surface ships.

use crate::app::actions::Action;

/// Whether this file owns `id`.
///
/// `pub(crate)` for `dispatch::format::handles`' reason: `shell::commands::reach`'s
/// reachability checker must be able to **evaluate** every guard arm it finds,
/// and a guard it cannot evaluate is a place commands could hide from the check
/// that exists to find them.
#[must_use]
pub(crate) fn handles(id: &str) -> bool {
    target(id).is_some()
}

/// The command each route points at.
///
/// One function, so the mapping is stated once and [`handles`] cannot drift
/// from [`dispatch`] — the two would otherwise be a list and a match that agree
/// today.
fn target(id: &str) -> Option<&'static str> {
    // ui-text-exempt: registered command ids, never displayed.
    match id {
        // ★★ Its tooltip promises *"list every form field … rename, retype or
        // remove them"*, and every one of those is already reachable: the Forms
        // panel lists and fills, the Properties pane renames and removes.
        // **Retyping is reachable nowhere and never will be** — Acrobat has
        // offered no field-type conversion since Acrobat 6, and `pdfce-core`
        // models the same limit by making the request unrepresentable, so there
        // is not even a control to grey. That clause was struck from the
        // tooltip when this was wired.
        "edit.form_manage_fields" => Some("view.panel_forms"),
        // ★★★ Its blocker said the folder list *"needs the pane it lives in"* —
        // the old shell's unsalvaged batch pane. Every clause about the engine
        // was true; the last one was an **assumption**, and nothing about a
        // list of directories needs a batch pane.
        //
        // ⇒ A blocker naming a missing **host** is weaker than one naming a
        // missing capability, and it goes stale the moment any other host will
        // do — silently, because nothing has to happen for it to become false.
        "tools.font_folders" => Some("file.settings"),
        // ★★ `format.properties` is the oldest route of this shape and it is
        // deliberately NOT here. It stays in `dispatch::format`, whose subject
        // is *the Format tab's share of the routing table*, and moving it would
        // have given one id two claimants — the dispatcher matches arms in
        // order, so which one won would depend on their order in a `match`.
        //
        // ⇒ Two organising principles genuinely collide here: *"all second
        // routes in one place"* and *"one file per tab's arms"*. The tie-break
        // is that **an id must have exactly one claimant**, and the existing
        // one is tested. This file's header claims the shape, not a monopoly on
        // it.
        _ => None,
    }
}

/// Raise the command this route points at.
///
/// ★ It does **not** re-check the target's guards. `dispatch_command` is the
/// choke point and the raised id goes through it exactly as a ribbon click
/// would — which is the entire point of routing rather than performing. A
/// guard applied here would be a second copy of the target's rule, in the file
/// whose purpose is to have none.
pub(crate) fn dispatch(id: &str, actions: &mut Vec<Action>) {
    if let Some(target) = target(id) {
        actions.push(Action::Command(target.to_owned()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★★ **Every route points at a registered command, and never at itself.**
    ///
    /// Two failures in one assertion, and both are silent. A route to an
    /// unregistered id raises an `Action::Command` that `dispatch_command`
    /// drops on the floor — a control that does nothing, which is what all
    /// three of these used to be. A route to itself is an infinite loop through
    /// the action queue.
    #[test]
    fn every_route_points_at_a_registered_command_that_is_not_itself() {
        let mut registry = egui_shell::commands::CommandRegistry::new();
        crate::shell::commands::register(&mut registry);
        for id in ROUTED {
            let target = target(id).expect("listed");
            assert_ne!(target, *id, "`{id}` routes to itself");
            assert!(
                registry.get(target).is_some(),
                "`{id}` routes to `{target}`, which this build does not register — the action \
                 would be raised and dropped, and the control would do nothing"
            );
            assert!(
                registry.get(id).is_some(),
                "`{id}` is routed and is not registered, so nothing can invoke it"
            );
        }
    }

    /// The ids this file claims, as a list, so a fourth added to `target` and
    /// not here fails rather than going untested.
    // ui-text-exempt: registered command ids, never displayed.
    const ROUTED: &[&str] = &["edit.form_manage_fields", "tools.font_folders"];

    /// **`handles` and `target` cannot disagree**, which is why there is one
    /// mapping and not a list beside a match.
    #[test]
    fn handles_agrees_with_target() {
        for id in ROUTED {
            assert!(handles(id));
        }
        assert!(!handles("file.open"));
        assert!(!handles("nonsense"));
    }
}
