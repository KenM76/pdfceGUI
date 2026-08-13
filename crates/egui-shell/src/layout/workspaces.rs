//! Named workspaces — save, load, list, delete.
//!
//! # What a workspace is, and what it deliberately is not
//!
//! A workspace is **a name and an arrangement**. That is the whole type.
//!
//! `MODES_AND_PANELS.md` closes its analysis by identifying the two
//! requests that arrived together — a Read/Review/Edit mode selector, and
//! flexible panel areas — as one system:
//!
//! > **A mode is capability (g).** Read, Review and Edit are three
//! > built-in named workspaces, shipped as defaults, each remembering the
//! > operator's arrangement of it.
//!
//! So the three mode names live in the **application's** configuration,
//! not here. Nothing in this file knows them, and nothing in this crate
//! ever will: `SHELL_FRAMEWORK.md` §4 already states the same rule for
//! the manifest — *Read/Review/Edit is a **configuration**, not a
//! built-in* — and a workspace store that shipped three magic names would
//! be the same violation wearing a different hat. An application that
//! wants three modes registers three workspaces; one that wants eleven
//! registers eleven; one that wants none never calls this module.
//!
//! # Why this is table stakes rather than a luxury
//!
//! The peer table in `MODES_AND_PANELS.md` is unambiguous: Photoshop,
//! Krita and Affinity all ship named layouts, and the benchmarked
//! application does not — with the recorded consequence that its
//! community's workaround is *"copying `dialogs-state-ex.ini` aside and
//! back, awkward because the file is rewritten on every exit."* Failure
//! mode #12 names it directly: *no named workspaces, no in-app reset →
//! **both are table stakes, not luxuries.***
//!
//! Note what that workaround actually is: an operator hand-rolling this
//! module out of file copies, and losing to a race with the application's
//! own writer. Two functions here retire it.
//!
//! # The one rule that keeps the store honest
//!
//! **Names are unique and are matched exactly.** Saving over an existing
//! name replaces it in place — keeping its position in the list, so a
//! workspace does not jump to the end of a menu every time it is
//! updated — and loading a name that is not there returns `None` rather
//! than a nearest match. A store that guessed would make "load Review"
//! into a command whose effect depends on what else is saved.

use serde::{Deserialize, Serialize};

use super::skip::{LayoutSite, LayoutSkipReason, LoadReport};
use super::{LayoutDocument, Scope, sanitize};
use crate::dock::model::{DockLayout, PanelCatalog};

/// One named arrangement.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Workspace {
    /// What the operator called it.
    ///
    /// Free text, and deliberately not an id: a workspace is named by a
    /// person, for a person, and forcing `review_layout_2` on them buys
    /// nothing. Uniqueness is enforced on load; see the module header.
    pub name: String,
    /// The arrangement it restores.
    pub layout: DockLayout,
}

impl Workspace {
    /// Name an arrangement.
    #[must_use]
    pub fn new(name: impl Into<String>, layout: DockLayout) -> Self {
        Self {
            name: name.into(),
            layout,
        }
    }
}

impl LayoutDocument {
    /// Save `layout` under `name`, replacing any workspace already using
    /// that name **in place**.
    ///
    /// Returns whether an existing workspace was replaced, so an
    /// application can offer "overwrite?" *after* the fact in a status
    /// surface rather than asking before it — the same posture the rest
    /// of this crate takes towards modal interruptions.
    ///
    /// Replacing in place rather than appending is not cosmetic: a
    /// workspace that moved to the end of the list every time it was
    /// updated would reorder the operator's own menu behind their back,
    /// and a menu whose order changes when you use it is a menu you
    /// cannot build muscle memory for.
    pub fn save_workspace(&mut self, name: impl Into<String>, layout: DockLayout) -> bool {
        let name = name.into();
        let mut layout = layout;
        layout.normalize();
        if let Some(existing) = self.workspaces.iter_mut().find(|w| w.name == name) {
            existing.layout = layout;
            return true;
        }
        self.workspaces.push(Workspace::new(name, layout));
        false
    }

    /// The arrangement saved under `name`, if any.
    ///
    /// Returns a reference; the caller clones it into
    /// [`crate::dock::DockState::set_layout`]. Deliberately **not** a
    /// method that applies it: the store does not own the live state, and
    /// a function here that reached into a `DockState` would be a second
    /// path by which the arrangement changes.
    #[must_use]
    pub fn workspace(&self, name: &str) -> Option<&DockLayout> {
        self.workspaces
            .iter()
            .find(|w| w.name == name)
            .map(|w| &w.layout)
    }

    /// Every workspace name, in the order they were first saved.
    #[must_use]
    pub fn workspace_names(&self) -> Vec<&str> {
        self.workspaces.iter().map(|w| w.name.as_str()).collect()
    }

    /// Delete the workspace called `name`.
    ///
    /// Returns whether one was removed. Deleting something that is not
    /// there is not an error — a second click on a delete command, or a
    /// stale menu, must not produce a failure the operator has to read.
    pub fn delete_workspace(&mut self, name: &str) -> bool {
        let before = self.workspaces.len();
        self.workspaces.retain(|w| w.name != name);
        self.workspaces.len() != before
    }

    /// Rename a workspace, refusing if the new name is taken or empty.
    ///
    /// Refusing rather than merging: a rename that silently absorbed
    /// another workspace would destroy an arrangement the operator did
    /// not mention.
    pub fn rename_workspace(&mut self, from: &str, to: impl Into<String>) -> bool {
        let to = to.into();
        if to.trim().is_empty() || self.workspaces.iter().any(|w| w.name == to) {
            return false;
        }
        match self.workspaces.iter_mut().find(|w| w.name == from) {
            Some(w) => {
                w.name = to;
                true
            }
            None => false,
        }
    }
}

/// Sanitize every workspace in a loaded document, dropping the ones that
/// cannot survive and repairing the ones that can.
///
/// **Per workspace**, which is the whole point: one saved arrangement
/// naming a panel this build does not offer loses that tab; one with no
/// name at all is dropped; a second one claiming a name already used is
/// dropped; and every other workspace in the file is untouched. That is
/// the per-item promise applied at the granularity an operator thinks in
/// — *"my Review layout came back and my Proofing one did not"* is a
/// sentence they can act on.
pub(crate) fn sanitize_all(
    workspaces: &mut Vec<Workspace>,
    catalog: &dyn PanelCatalog,
    report: &mut LoadReport,
) {
    let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut kept: Vec<Workspace> = Vec::with_capacity(workspaces.len());

    for mut workspace in std::mem::take(workspaces) {
        if workspace.name.trim().is_empty() {
            report.push(
                LayoutSite::Document,
                LayoutSkipReason::WorkspaceName {
                    name: String::new(),
                },
            );
            continue;
        }
        if !seen.insert(workspace.name.clone()) {
            report.push(
                LayoutSite::Document,
                LayoutSkipReason::WorkspaceName {
                    name: workspace.name.clone(),
                },
            );
            continue;
        }

        let name = workspace.name.clone();
        sanitize(
            &mut workspace.layout,
            catalog,
            &Scope::Workspace(&name),
            report,
        );

        // A workspace sanitized to nothing is dropped rather than kept as
        // an empty arrangement. Restoring it would give the operator a
        // dock with no panels and no explanation; the per-panel skips
        // already recorded say why it went.
        if workspace.layout.panels().next().is_none() {
            report.push(
                LayoutSite::Workspace { name },
                LayoutSkipReason::EmptyContainer,
            );
            continue;
        }

        kept.push(workspace);
    }

    *workspaces = kept;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dock::model::{
        AnyPanel, Column, PanelId, PanelInfo, PanelRegistry, SideLayout, Stack,
    };

    fn registry(ids: &[&str]) -> PanelRegistry {
        let mut r = PanelRegistry::new();
        for id in ids {
            r.register(PanelInfo::new(*id, *id));
        }
        r
    }

    fn one(panel: &str) -> DockLayout {
        DockLayout::new(SideLayout::single(panel), SideLayout::none())
    }

    /// Save, load, list, delete — the four operations, in one pass.
    #[test]
    fn the_four_operations_do_what_they_say() {
        let mut doc = LayoutDocument::new(one("pages"));
        assert!(doc.workspace_names().is_empty());

        assert!(!doc.save_workspace("Reading", one("pages")));
        assert!(!doc.save_workspace("Marking up", one("layers")));
        assert_eq!(doc.workspace_names(), vec!["Reading", "Marking up"]);

        assert!(
            doc.workspace("Reading")
                .expect("saved")
                .contains(&PanelId::new("pages"))
        );
        assert!(doc.workspace("Nothing").is_none(), "no nearest match");

        assert!(doc.delete_workspace("Reading"));
        assert!(
            !doc.delete_workspace("Reading"),
            "a second delete is not an error"
        );
        assert_eq!(doc.workspace_names(), vec!["Marking up"]);
    }

    /// ★ **Saving over a name replaces it in place**, so the operator's
    /// own menu does not reorder itself every time they use it.
    #[test]
    fn saving_over_a_name_keeps_its_position_in_the_list() {
        let mut doc = LayoutDocument::new(one("pages"));
        doc.save_workspace("A", one("pages"));
        doc.save_workspace("B", one("layers"));
        doc.save_workspace("C", one("tools"));

        assert!(
            doc.save_workspace("A", one("objects")),
            "reported as a replace"
        );
        assert_eq!(doc.workspace_names(), vec!["A", "B", "C"], "order held");
        assert!(
            doc.workspace("A")
                .expect("still there")
                .contains(&PanelId::new("objects"))
        );
    }

    /// A saved arrangement is normalized on the way in, so a workspace
    /// cannot carry a defect the live layout would have repaired.
    #[test]
    fn a_saved_workspace_is_normalized_on_the_way_in() {
        let mut doc = LayoutDocument::new(one("pages"));
        let messy = DockLayout::new(
            SideLayout::new([Column::new([Stack::new("a"), Stack::new("a")])]),
            SideLayout::none(),
        );
        doc.save_workspace("Messy", messy);
        let saved = doc.workspace("Messy").expect("saved");
        assert!(saved.is_normalized());
        assert_eq!(saved.panels().count(), 1);
    }

    /// A rename refuses a collision and an empty name, rather than
    /// absorbing an arrangement the operator did not mention.
    #[test]
    fn a_rename_refuses_a_collision_and_an_empty_name() {
        let mut doc = LayoutDocument::new(one("pages"));
        doc.save_workspace("A", one("pages"));
        doc.save_workspace("B", one("layers"));
        assert!(!doc.rename_workspace("A", "B"), "would have destroyed B");
        assert!(!doc.rename_workspace("A", "   "));
        assert!(!doc.rename_workspace("missing", "C"));
        assert!(doc.rename_workspace("A", "Reading"));
        assert_eq!(doc.workspace_names(), vec!["Reading", "B"]);
    }

    /// Workspaces survive a round trip through text with the live
    /// arrangement.
    #[test]
    fn workspaces_round_trip_with_the_document() {
        let mut doc = LayoutDocument::new(one("pages"));
        doc.save_workspace("Reading", one("pages"));
        doc.save_workspace("Marking up", one("layers"));
        let text = doc.to_ron_pretty().expect("serializes");
        let loaded = LayoutDocument::from_ron(&text, &one("pages"), &AnyPanel);
        assert!(loaded.report.is_empty(), "{:?}", loaded.report.skips());
        assert_eq!(loaded.document, doc);
    }

    /// ★ **One bad workspace does not cost the others.**
    ///
    /// The per-item promise at the granularity an operator thinks in.
    /// Here: an unnamed one, a duplicate one, one whose only panel this
    /// build does not offer — and two good ones that come back intact.
    #[test]
    fn one_bad_workspace_does_not_cost_the_others() {
        let mut doc = LayoutDocument::new(one("pages"));
        doc.workspaces = vec![
            Workspace::new("Reading", one("pages")),
            Workspace::new("", one("pages")),
            Workspace::new("Reading", one("layers")),
            Workspace::new("Signing", one("signatures")),
            Workspace::new("Marking up", one("layers")),
        ];
        let text = doc.to_ron_pretty().expect("serializes");
        let loaded =
            LayoutDocument::from_ron(&text, &one("pages"), &registry(&["pages", "layers"]));

        assert_eq!(
            loaded.document.workspace_names(),
            vec!["Reading", "Marking up"],
            "the good ones survived and kept their order"
        );
        assert!(
            loaded
                .document
                .workspace("Reading")
                .expect("kept")
                .contains(&PanelId::new("pages")),
            "the FIRST `Reading` won, not the duplicate"
        );

        let reasons = loaded.report.skips();
        assert!(reasons.iter().any(|s| matches!(&s.reason,
            LayoutSkipReason::WorkspaceName { name } if name.is_empty())));
        assert!(reasons.iter().any(|s| matches!(&s.reason,
            LayoutSkipReason::WorkspaceName { name } if name == "Reading")));
        assert!(reasons.iter().any(|s| matches!(&s.reason,
            LayoutSkipReason::UnknownPanel { panel } if panel.as_str() == "signatures")));
    }

    /// A workspace that lost every panel is dropped rather than restored
    /// as an empty dock — and the skip names the workspace, so the
    /// operator can connect it to the panel that went.
    #[test]
    fn a_workspace_emptied_by_a_missing_capability_is_dropped_by_name() {
        let mut doc = LayoutDocument::new(one("pages"));
        doc.save_workspace("Signing", one("signatures"));
        let text = doc.to_ron_pretty().expect("serializes");
        let loaded = LayoutDocument::from_ron(&text, &one("pages"), &registry(&["pages"]));
        assert!(loaded.document.workspaces.is_empty());
        assert!(
            loaded.report.skips().iter().any(|s| matches!(
                &s.site,
                LayoutSite::Workspace { name } if name == "Signing"
            )),
            "the skip does not name the workspace: {:?}",
            loaded.report.skips()
        );
    }

    /// ★ **The shell ships no workspace names of its own.**
    ///
    /// `MODES_AND_PANELS.md` makes a mode *a named workspace*, and
    /// `SHELL_FRAMEWORK.md` makes Read/Review/Edit a configuration rather
    /// than a built-in. A default store with three magic names would
    /// quietly weld one application's modes into the framework, which is
    /// the exact class of coupling the purity gate exists to prevent —
    /// and the gate greps for PDF crate names, so it would not catch
    /// this one.
    #[test]
    fn a_fresh_document_ships_no_workspaces_at_all() {
        assert!(LayoutDocument::default().workspaces.is_empty());
        assert!(
            LayoutDocument::new(one("pages"))
                .workspace_names()
                .is_empty()
        );
    }
}
