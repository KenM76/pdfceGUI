//! The manifest — the serializable definition of what the shell *is*.
//!
//! # The decision this module implements
//!
//! `SHELL_FRAMEWORK.md` §1, verbatim, because everything here follows
//! from it:
//!
//! > **The shell is data. Tabs, groups, commands, panels, layouts, modes
//! > and key bindings are a serializable document that the application
//! > *supplies* and the operator *edits* — not code that has to be
//! > recompiled to change.**
//!
//! Two requirements arrived together on this project: be **reusable**
//! across applications, and be **customizable** at runtime by the
//! operator. A ribbon defined in Rust `match` arms can be neither. A
//! ribbon defined as data can be both, and the same serializer that lets
//! an operator save a customized ribbon lets a different application ship
//! a completely different one.
//!
//! It also retires a deferral. The salvage source's ribbon deferred
//! customization on the grounds that *"a customisable ribbon that also
//! forgets itself would be worse than none."* That objection was about
//! persistence, and persistence is the first thing this design builds.
//!
//! # The type shape, and why one type is both document and patch
//!
//! [`Shell`] describes a complete ribbon. It is *also* the type of each
//! **layer** in the three-layer merge — the application-override file and
//! the operator's customization file are `Shell` values too, each
//! typically describing a handful of fields.
//!
//! That is why nearly every field is `Option`: the `Option` is what
//! distinguishes *"set this to empty"* from *"do not mention this"*, and
//! that distinction is the entire difference between a per-item override
//! and a wholesale replacement. A layer that says
//!
//! ```ron
//! Shell(tabs: [ Tab(id: "tools") ])
//! ```
//!
//! is saying "move the Tools tab to the front" — not "delete every other
//! tab", which is what a non-optional `tabs: Vec<Tab>` would have to mean.
//!
//! The cost is that a `Shell` can be *incomplete*, and the answer to that
//! is [`Shell::validate`]: **a complete manifest is one that validates.**
//! A layer is not expected to. The merged result is required to. One type,
//! two roles, and a checked boundary between them — rather than a second
//! `ShellPatch` type that would have to be kept in step with this one
//! field by field, forever.
//!
//! # What is checked, and where
//!
//! | Property | Enforced by |
//! |---|---|
//! | Structure: ids unique, labels present, groups captioned | [`Shell::validate`] |
//! | **One command appears on at most one tab** | [`Shell::validate`] |
//! | Every referenced command exists | [`Shell::validate_against`] |
//! | An operator's stale reference loses one item, not the layout | [`merge`] |
//!
//! The split is deliberate and is the difference between a *rejection*
//! and a *disclosure*:
//!
//! - **[`merge`] is fail-soft.** It is handed files an operator edited by
//!   hand and an application shipped two versions ago. A command that no
//!   longer exists loses that one item and produces a [`Skip`] the
//!   application can show in its status surface. It does not discard the
//!   layout, and it does not fail.
//! - **[`Shell::validate`] is strict.** It runs on the *merged* result,
//!   and what it rejects are contradictions no fail-soft rule can repair —
//!   two tabs with one id, a command on two tabs. `SHELL_FRAMEWORK.md`
//!   §5 makes the point that this is strictly more than the salvage
//!   source's compile-time ownership test could do: that test could only
//!   check the ribbon the developers wrote, and this one checks the ribbon
//!   the operator ends up with.
//!
//! # Commands are referenced, never defined
//!
//! A manifest contains command **ids**. It contains no labels for them, no
//! icons, no handlers, and no way to add any. Those live in
//! [`crate::commands::CommandRegistry`], in code.
//!
//! That is what stops a customized ribbon from inventing a command that
//! does not exist, and it is why an unknown id can be a disclosed skip
//! rather than a crash: there is nothing for the shell to try to run.
//!
//! # On-disk form
//!
//! RON, via [`Shell::from_ron`] and [`Shell::to_ron`]. RON rather than
//! JSON because the format has real enums (`Command("view.single")` beside
//! `Separator`), comments, and trailing commas — all three matter for a
//! file an operator edits by hand, which is the whole point of the
//! customization layer.
//!
//! ```ron
//! Shell(
//!     modes: [
//!         Mode(id: "read",   label: "Read",   tabs: ["file", "view"]),
//!         Mode(id: "review", label: "Review", tabs: ["file", "view", "pages", "markup", "measure"]),
//!     ],
//!     tabs: [
//!         Tab(id: "view", label: "View", question: "What is on my screen?", groups: [
//!             Group(id: "page_display", caption: "Page display", items: [
//!                 Command("view.single"), Command("view.continuous"),
//!             ]),
//!         ]),
//!     ],
//!     contextual_tabs: [
//!         Tab(id: "format", label: "Format", visible_when: "selection.any", groups: []),
//!     ],
//!     qat: ["file.open", "file.save_copy", "edit.undo", "edit.redo"],
//!     keymap: { "Ctrl+E": "edit.text", "Ctrl+1": "mode.read", "F11": "view.fullscreen" },
//! )
//! ```

mod merge;
mod validate;

pub use merge::{Layer, MergeInput, MergeReport, Merged, Skip, SkipReason, merge};
pub use validate::{ManifestError, Site};

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Anything that can say whether a command id is real.
///
/// [`crate::commands::CommandRegistry`] implements it. The trait exists so
/// this module does not depend on that one: a manifest must be parseable,
/// mergeable and round-trippable by a tool that has no registry at all —
/// a schema linter, a diff viewer, `tools/ui-verify` inspecting a `.ron`
/// file without linking the application.
pub trait CommandCatalog {
    /// Whether this id names a real command.
    fn contains(&self, id: &str) -> bool;
}

/// A catalog that accepts every id.
///
/// For tests, for tooling that has no registry, and for the first stage
/// of an application's own bring-up. Using it in production would disable
/// the check that makes an unknown id a disclosed skip, which is why it is
/// a named type at a call site rather than a default.
#[derive(Debug, Clone, Copy, Default)]
pub struct AnyCommand;

impl CommandCatalog for AnyCommand {
    fn contains(&self, _id: &str) -> bool {
        true
    }
}

/// The whole shell definition — or one layer of it. See the module header
/// on why those are the same type.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Shell {
    /// The schema version this document was written against.
    ///
    /// `0` means "unstated", which is what a hand-written operator file
    /// will usually be, and is treated as the current version. A value
    /// **above** [`Shell::SCHEMA`] means the file came from a newer build:
    /// [`merge`] skips that whole layer with a disclosure rather than
    /// guessing, because a field it does not understand may be the one
    /// that makes the rest of the file mean what it says.
    #[serde(skip_serializing_if = "is_zero")]
    pub schema: u32,
    /// Named workspaces. Each names the tabs it contains.
    ///
    /// `MODES_AND_PANELS.md`: *a mode is a named workspace layout*, and
    /// Read/Review/Edit is a **configuration**, not a built-in. Nothing in
    /// this crate knows those three names.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modes: Option<Vec<Mode>>,
    /// The ordinary tabs, in display order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tabs: Option<Vec<Tab>>,
    /// Tabs that appear only while their [`Tab::visible_when`] condition
    /// holds — a Format tab that appears on selection, say.
    ///
    /// Separate from [`Self::tabs`] because they are *not* mode members:
    /// a mode names a fixed tab set, and a contextual tab's whole nature
    /// is that its presence is decided by application state rather than by
    /// configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contextual_tabs: Option<Vec<Tab>>,
    /// The quick-access toolbar: command ids, in order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qat: Option<Qat>,
    /// Key chord → command id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keymap: Option<Keymap>,
    /// Context menus, keyed by the context id the application supplies at
    /// the right-click site — `"canvas.object"`, `"dock.tab"`, and so on.
    ///
    /// A menu is a [`Group`] in every respect except what keys it, so it
    /// carries the same [`Item`] list and is customized the same way. See
    /// [`crate::menu`].
    ///
    /// **Deliberately not covered by [`Shell::validate`]'s
    /// one-command-one-tab rule.** `RIBBON_IA.md` §6 is explicit that a
    /// context menu carrying the same commands as a tab *"is not
    /// duplication in the P1 sense — context menus are not tabs"*: the rule
    /// exists so a command has one discoverable **home**, and a menu is a
    /// shortcut to that home rather than a rival to it. `validate` walks
    /// `all_tabs()` only, and `one_command_may_appear_in_several_menus`
    /// says so in a test.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub menus: Option<crate::menu::Menus>,
}

/// `skip_serializing_if` predicate for [`Shell::schema`].
///
/// Takes a reference because that is serde's required signature for a
/// `skip_serializing_if` path, not because a `u32` wants one.
#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_zero(v: &u32) -> bool {
    *v == 0
}

impl Shell {
    /// The schema version this build writes and understands.
    ///
    /// Bump when a change would make an *older* build misread a newer
    /// file — not for an added optional field, which an older build
    /// already ignores safely.
    pub const SCHEMA: u32 = 1;

    /// An empty manifest stamped with the current schema.
    #[must_use]
    pub fn new() -> Self {
        Self {
            schema: Self::SCHEMA,
            ..Self::default()
        }
    }

    /// The ordinary tabs, or an empty slice if unstated.
    #[must_use]
    pub fn tabs(&self) -> &[Tab] {
        self.tabs.as_deref().unwrap_or(&[])
    }

    /// The contextual tabs, or an empty slice if unstated.
    #[must_use]
    pub fn contextual_tabs(&self) -> &[Tab] {
        self.contextual_tabs.as_deref().unwrap_or(&[])
    }

    /// The modes, or an empty slice if unstated.
    #[must_use]
    pub fn modes(&self) -> &[Mode] {
        self.modes.as_deref().unwrap_or(&[])
    }

    /// Every tab, ordinary then contextual.
    ///
    /// The one-command-one-tab rule counts contextual tabs, so most
    /// checks want this rather than [`Self::tabs`].
    pub fn all_tabs(&self) -> impl Iterator<Item = &Tab> {
        self.tabs().iter().chain(self.contextual_tabs())
    }

    /// Add an ordinary tab.
    #[must_use]
    pub fn with_tab(mut self, tab: Tab) -> Self {
        self.tabs.get_or_insert_with(Vec::new).push(tab);
        self
    }

    /// Add a contextual tab.
    #[must_use]
    pub fn with_contextual_tab(mut self, tab: Tab) -> Self {
        self.contextual_tabs.get_or_insert_with(Vec::new).push(tab);
        self
    }

    /// Add a mode.
    #[must_use]
    pub fn with_mode(mut self, mode: Mode) -> Self {
        self.modes.get_or_insert_with(Vec::new).push(mode);
        self
    }

    /// Set the quick-access toolbar.
    #[must_use]
    pub fn with_qat<I, S>(mut self, ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.qat = Some(Qat(ids.into_iter().map(Into::into).collect()));
        self
    }

    /// Bind a key chord to a command.
    #[must_use]
    pub fn with_binding(mut self, chord: impl Into<String>, command: impl Into<String>) -> Self {
        self.keymap
            .get_or_insert_with(Keymap::default)
            .0
            .insert(chord.into(), command.into());
        self
    }

    /// Parse a manifest from RON.
    ///
    /// # Errors
    ///
    /// [`ManifestError::Parse`], carrying RON's own line and column. The
    /// span is the useful part: this file is hand-edited, and "expected
    /// `)` at 14:3" is the difference between a fixable typo and a file
    /// the operator reverts wholesale.
    ///
    /// Parsing does **not** validate. A layer is not expected to be a
    /// complete manifest, so refusing to parse one that is incomplete
    /// would make the layered design unrepresentable. Call
    /// [`Self::validate`] on the merged result.
    pub fn from_ron(text: &str) -> Result<Self, ManifestError> {
        Ok(ron_options().from_str(text)?)
    }

    /// Serialize to compact RON.
    ///
    /// # Errors
    ///
    /// [`ManifestError::Serialize`] if RON refuses the value, which for
    /// this type's fields should not be reachable.
    pub fn to_ron(&self) -> Result<String, ManifestError> {
        Ok(ron_options().to_string(self)?)
    }

    /// Serialize to indented RON, for a file a human will open.
    ///
    /// # Errors
    ///
    /// As [`Self::to_ron`].
    pub fn to_ron_pretty(&self) -> Result<String, ManifestError> {
        Ok(ron_options().to_string_pretty(self, pretty_config())?)
    }
}

/// The RON dialect this manifest is read and written in.
///
/// # ★ Why `IMPLICIT_SOME`, and why it is not a cosmetic preference
///
/// Nearly every field of [`Shell`], [`Tab`], [`Group`] and [`Mode`] is an
/// `Option`, because the `Option` is what distinguishes *"set this to
/// empty"* from *"do not mention this"* — see the module header. That is
/// the right model in Rust and, in stock RON, a disaster on disk: a
/// present value has to be written `tabs: Some([…])`, and the operator's
/// customization file — the whole point of the format being editable —
/// fills up with a wrapper that carries no information at all.
///
/// Worse, it is a wrapper that is *easy to forget*. Writing
///
/// ```ron
/// Shell(tabs: [ Tab(id: "tools") ])
/// ```
///
/// is the obvious thing, it is what every example in this crate's
/// documentation shows, and under stock RON it fails to parse with
/// `ExpectedOption` — a message that means nothing to someone who has
/// never seen a Rust `Option`.
///
/// `IMPLICIT_SOME` makes the obvious spelling the correct one. It is set
/// on the [`ron::Options`] used for **both** directions rather than only
/// emitted as an `#![enable(implicit_some)]` header, because a header
/// only helps files this crate wrote. A file the operator wrote from
/// scratch, or pasted from documentation, has no header — and that file
/// is exactly the one that must not fail.
///
/// The header is emitted as well, by [`pretty_config`], so that a file
/// this crate writes is also readable by RON tooling that honours it.
fn ron_options() -> ron::Options {
    ron::Options::default().with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
}

/// Formatting for a manifest a person will open.
///
/// Carries the same extension as [`ron_options`] so the written file
/// declares the dialect it is in.
fn pretty_config() -> ron::ser::PrettyConfig {
    ron::ser::PrettyConfig::default().extensions(ron::extensions::Extensions::IMPLICIT_SOME)
}

/// A named workspace: a label and the tabs it contains.
///
/// `MODES_AND_PANELS.md` Part 1 describes what a mode is for, and one
/// rule from it binds anything rendering this type:
///
/// > **A mode changes what is *visible*. It never makes a visible control
/// > silently inert.**
///
/// That is the difference between a mode and the master toggle it
/// replaced: the toggle left the editing tools on screen and made
/// gestures quietly do nothing. A mode *removes* the tools it disables, so
/// there is no click that mysteriously fails.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Mode {
    /// Stable id, e.g. `"review"`. Never displayed.
    pub id: String,
    /// The operator-visible label. Required in a complete manifest.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The ids of the ordinary tabs this mode contains, in order.
    ///
    /// A reference to a tab that does not exist is dropped by [`merge`]
    /// with a disclosure — an operator's mode surviving a tab's removal
    /// minus one entry is better than the mode failing to load.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tabs: Option<Vec<String>>,
}

impl Mode {
    /// A complete mode.
    #[must_use]
    pub fn new<I, S>(id: impl Into<String>, label: impl Into<String>, tabs: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            id: id.into(),
            label: Some(label.into()),
            tabs: Some(tabs.into_iter().map(Into::into).collect()),
        }
    }

    /// A mode reference for a layer: names the id and overrides nothing.
    #[must_use]
    pub fn patch(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            ..Self::default()
        }
    }

    /// The tabs this mode names, or an empty slice if unstated.
    #[must_use]
    pub fn tabs(&self) -> &[String] {
        self.tabs.as_deref().unwrap_or(&[])
    }
}

/// One ribbon tab.
///
/// `RIBBON_IA.md` §4 keeps an idiom worth preserving: every tab carries a
/// one-line **question** it exists to answer — *"What is on my screen, and
/// how is the page laid out?"* That is what [`Self::question`] is, and it
/// is not decoration: a tab whose question cannot be written in one line
/// is a tab carrying two unrelated jobs, which is the defect that split
/// six tabs into seven in that document.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Tab {
    /// Stable id, e.g. `"view"`. Never displayed.
    pub id: String,
    /// The operator-visible label. Required in a complete manifest.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The one-line question this tab exists to answer. Optional; a
    /// renderer may show it as a hint, and a reviewer should read it as a
    /// test of whether the tab is coherent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub question: Option<String>,
    /// For a contextual tab: the condition under which it appears, in the
    /// language of [`crate::commands::ConditionSet`], e.g.
    /// `"selection.any"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visible_when: Option<String>,
    /// Hidden tabs keep their definition and are not rendered.
    ///
    /// Hiding rather than deleting is what makes "unhide it again" a
    /// possible operation. An operator who deletes a tab from their
    /// customization file gets the built-in one back at the next merge,
    /// which is surprising; an operator who hides it gets what they asked
    /// for and can undo it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
    /// The groups on this tab, in display order. Required in a complete
    /// manifest — a tab with no `groups` key at all is a layer's
    /// reference to a tab, not an empty tab.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub groups: Option<Vec<Group>>,
}

impl Tab {
    /// A tab with a label and no groups yet.
    #[must_use]
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: Some(label.into()),
            groups: Some(Vec::new()),
            ..Self::default()
        }
    }

    /// A tab reference for a layer: names the id and overrides nothing.
    ///
    /// This is the whole vocabulary needed to reorder tabs — see
    /// [`merge`]'s ordering rule.
    #[must_use]
    pub fn patch(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            ..Self::default()
        }
    }

    /// Set the groups.
    #[must_use]
    pub fn with_groups(mut self, groups: impl IntoIterator<Item = Group>) -> Self {
        self.groups = Some(groups.into_iter().collect());
        self
    }

    /// Set the one-line question.
    #[must_use]
    pub fn with_question(mut self, question: impl Into<String>) -> Self {
        self.question = Some(question.into());
        self
    }

    /// Set the visibility condition, making this a contextual tab.
    #[must_use]
    pub fn with_visible_when(mut self, condition: impl Into<String>) -> Self {
        self.visible_when = Some(condition.into());
        self
    }

    /// Hide or show.
    #[must_use]
    pub fn with_hidden(mut self, hidden: bool) -> Self {
        self.hidden = Some(hidden);
        self
    }

    /// Whether this tab is hidden. Unstated means visible.
    #[must_use]
    pub fn is_hidden(&self) -> bool {
        self.hidden.unwrap_or(false)
    }

    /// The groups, or an empty slice if unstated.
    #[must_use]
    pub fn groups(&self) -> &[Group] {
        self.groups.as_deref().unwrap_or(&[])
    }
}

/// A captioned band of items within a tab.
///
/// The caption is required in a complete manifest, and that is a rule
/// carried across from the salvage source, which enforced it with a
/// single closure through which every group had to be rendered. An
/// uncaptioned group is a row of controls whose relationship the operator
/// has to infer.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Group {
    /// Stable id, unique within its tab. Never displayed.
    pub id: String,
    /// The operator-visible caption. Required in a complete manifest.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    /// The items in this group, in display order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<Item>>,
}

impl Group {
    /// A group with a caption and no items yet.
    #[must_use]
    pub fn new(id: impl Into<String>, caption: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            caption: Some(caption.into()),
            items: Some(Vec::new()),
        }
    }

    /// A group reference for a layer: names the id and overrides nothing.
    #[must_use]
    pub fn patch(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            ..Self::default()
        }
    }

    /// Set the items.
    #[must_use]
    pub fn with_items(mut self, items: impl IntoIterator<Item = Item>) -> Self {
        self.items = Some(items.into_iter().collect());
        self
    }

    /// The items, or an empty slice if unstated.
    #[must_use]
    pub fn items(&self) -> &[Item] {
        self.items.as_deref().unwrap_or(&[])
    }
}

/// One entry in a group.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Item {
    /// A command, by id. The id is resolved against the registry; the
    /// manifest carries nothing else about it.
    Command(String),
    /// A vertical rule between neighbours. Presentation only.
    Separator,
    /// Something the application draws itself.
    ///
    /// The extension point for controls that are not a button: a colour
    /// swatch, a zoom slider, a scale picker, a split button with a
    /// gallery. The shell reserves the space and hands `kind` and
    /// `payload` back; it draws nothing and interprets neither.
    ///
    /// This is what keeps the item vocabulary from growing a variant per
    /// widget an application happens to want — which is the road by which
    /// a reusable shell acquires a `ColourSwatch` variant and stops being
    /// reusable.
    Custom {
        /// An application-defined kind, e.g. `"colour_swatch"`.
        kind: String,
        /// Optional application-defined payload.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        payload: Option<String>,
    },
}

impl Item {
    /// A command item.
    #[must_use]
    pub fn command(id: impl Into<String>) -> Self {
        Item::Command(id.into())
    }

    /// A custom item with no payload.
    #[must_use]
    pub fn custom(kind: impl Into<String>) -> Self {
        Item::Custom {
            kind: kind.into(),
            payload: None,
        }
    }

    /// The command id, if this item is a command.
    #[must_use]
    pub fn command_id(&self) -> Option<&str> {
        match self {
            Item::Command(id) => Some(id),
            Item::Separator | Item::Custom { .. } => None,
        }
    }
}

/// The quick-access toolbar: command ids, in order.
///
/// `SHELL_FRAMEWORK.md` §5 amends the salvage source's one-command-one-tab
/// rule specifically to allow this: *"a command may appear on exactly one
/// **tab**; the QAT and status bar may mirror it."* A QAT that could not
/// mirror would be a second place to hunt for a command rather than a
/// shortcut to a known one.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Qat(pub Vec<String>);

impl Qat {
    /// The command ids, in order.
    #[must_use]
    pub fn ids(&self) -> &[String] {
        &self.0
    }
}

impl<S: Into<String>> FromIterator<S> for Qat {
    fn from_iter<I: IntoIterator<Item = S>>(iter: I) -> Self {
        Qat(iter.into_iter().map(Into::into).collect())
    }
}

/// Key chord → command id.
///
/// The chord is an opaque string here — `"Ctrl+E"`, `"F11"`. Parsing it
/// into modifiers and a key is the renderer's job, and doing it in this
/// type would mean a manifest could not be read by a tool that does not
/// link `egui`.
///
/// Ordered (`BTreeMap`) so a serialized manifest is byte-stable: an
/// operator's customization file that reordered itself on every save
/// would produce a diff on every run and make version control useless for
/// exactly the file most worth versioning.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Keymap(pub BTreeMap<String, String>);

impl Keymap {
    /// The command bound to a chord, if any.
    #[must_use]
    pub fn get(&self, chord: &str) -> Option<&str> {
        self.0.get(chord).map(String::as_str)
    }

    /// Every binding, in chord order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.0.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    /// How many chords are bound.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether nothing is bound.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A manifest in the shape of `SHELL_FRAMEWORK.md` §4's sketch,
    /// reduced to two tabs. Shared by the round-trip and validation
    /// tests so they cannot disagree about what a well-formed manifest
    /// looks like.
    pub(super) fn sketch() -> Shell {
        Shell::new()
            .with_mode(Mode::new("read", "Read", ["file", "view"]))
            .with_mode(Mode::new("edit", "Edit", ["file", "view"]))
            .with_tab(
                Tab::new("file", "File")
                    .with_question("What do I do with the file as a whole?")
                    .with_groups([Group::new("file", "File").with_items([
                        Item::command("file.open"),
                        Item::Separator,
                        Item::command("file.save_copy"),
                    ])]),
            )
            .with_tab(
                Tab::new("view", "View")
                    .with_question("What is on my screen?")
                    .with_groups([
                        Group::new("page_display", "Page display").with_items([
                            Item::command("view.single"),
                            Item::command("view.continuous"),
                        ]),
                        Group::new("window", "Window").with_items([
                            Item::command("view.fullscreen"),
                            Item::custom("zoom_slider"),
                        ]),
                    ]),
            )
            .with_contextual_tab(
                Tab::new("format", "Format")
                    .with_visible_when("selection.any")
                    .with_groups([
                        Group::new("style", "Style").with_items([Item::command("format.colour")])
                    ]),
            )
            .with_qat(["file.open", "file.save_copy"])
            .with_binding("Ctrl+E", "edit.text")
            .with_binding("F11", "view.fullscreen")
    }

    /// **A manifest survives a trip through RON unchanged.**
    ///
    /// The manifest's whole value proposition is that it is a file: an
    /// operator edits it, an application ships one, a workspace is saved
    /// as one, and `SHELL_FRAMEWORK.md` §6 lists "inspectable, diffable,
    /// testable without a GUI, and serializable" as what the design buys.
    /// Every one of those claims fails if the round trip is lossy.
    ///
    /// Both forms are checked. The compact form is what a save writes;
    /// the pretty form is what an operator opens, and a pretty printer
    /// that emits something its own parser rejects would be discovered by
    /// the operator rather than by CI.
    #[test]
    fn a_manifest_round_trips_through_ron() {
        let original = sketch();

        let compact = original.to_ron().expect("serializes");
        assert_eq!(
            Shell::from_ron(&compact).expect("compact form parses"),
            original,
            "the compact round trip lost or changed something"
        );

        let pretty = original.to_ron_pretty().expect("serializes");
        assert_eq!(
            Shell::from_ron(&pretty).expect("pretty form parses"),
            original,
            "the pretty round trip lost or changed something"
        );

        // The shapes the module header advertises must actually appear,
        // or the documented example is fiction.
        assert!(pretty.contains("Command(\"file.open\")"), "{pretty}");
        assert!(pretty.contains("Separator"), "{pretty}");
        assert!(pretty.contains("\"Ctrl+E\""), "{pretty}");
    }

    /// **An unstated field stays unstated through a round trip.**
    ///
    /// This is the property the whole layered design rests on: `None`
    /// means "do not mention this" and must not come back as
    /// `Some(empty)`. If a layer's omitted `groups` round-tripped into
    /// `Some(vec![])`, saving and reloading an operator's customization
    /// would silently empty every tab it mentioned.
    #[test]
    fn an_unstated_field_stays_unstated_through_a_round_trip() {
        let layer = Shell::default().with_tab(Tab::patch("tools"));
        let text = layer.to_ron().expect("serializes");
        assert!(
            !text.contains("groups"),
            "an unstated field must not be written at all; got {text}"
        );
        let back = Shell::from_ron(&text).expect("parses");
        assert_eq!(back, layer);
        assert!(
            back.tabs()[0].groups.is_none(),
            "`None` must not resurrect as `Some(empty)` — that turns a \
             reference to a tab into an instruction to empty it"
        );
    }

    /// A hand-written operator file, with comments and a trailing comma,
    /// parses. This is the ergonomics claim RON was chosen for.
    #[test]
    fn a_hand_written_file_with_comments_parses() {
        let text = r#"
            Shell(
                // Move Tools to the front; leave everything else alone.
                tabs: [
                    Tab(id: "tools"),
                ],
                keymap: { "Ctrl+K": "tools.batch" },
            )
        "#;
        let shell = Shell::from_ron(text).expect("comments and trailing commas are allowed");
        assert_eq!(shell.tabs().len(), 1);
        assert_eq!(shell.tabs()[0].id, "tools");
        assert_eq!(
            shell.keymap.as_ref().and_then(|k| k.get("Ctrl+K")),
            Some("tools.batch")
        );
    }

    /// Accessors treat "unstated" as empty rather than panicking, so a
    /// layer can be read by the same code that reads a complete manifest.
    #[test]
    fn accessors_read_an_incomplete_layer_as_empty() {
        let layer = Shell::default();
        assert!(layer.tabs().is_empty());
        assert!(layer.modes().is_empty());
        assert!(layer.contextual_tabs().is_empty());
        assert_eq!(layer.all_tabs().count(), 0);
    }

    /// `Item::command_id` distinguishes the three variants, which is what
    /// every reference walk in `validate` and `merge` relies on.
    #[test]
    fn only_command_items_carry_a_command_id() {
        assert_eq!(Item::command("a.b").command_id(), Some("a.b"));
        assert_eq!(Item::Separator.command_id(), None);
        assert_eq!(Item::custom("swatch").command_id(), None);
    }
}
