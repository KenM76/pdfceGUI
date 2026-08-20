//! # shell::commands — every verb pdfce can perform
//!
//! [`register`] populates an `egui_shell::CommandRegistry` with the
//! **hundred and one** commands this build has, which fall into three groups:
//!
//! | group | count | how the operator reaches it |
//! |---|---|---|
//! | on a tab, the QAT or the keymap | 99 | a control [`super::manifest::built_in`] names by id |
//! | drawn by a **custom item** | 1 | `file.recent` — see [`super::manifest::CUSTOM_BACKED`] |
//! | drawn on the **status bar** | 1 | `edit.find` — `RIBBON_IA.md` §6 |
//!
//! (This header said *eighty-one* and *79* until 2026-08-14, while
//! [`tests::registration_succeeds_and_registers_every_command`] asserted 88
//! and passed. Prose drifting from a number a test pins is a defect this
//! project has now had three times; the test is the fact, and the table has
//! been corrected to it rather than the other way round.)
//!
//! ★ **And it happened a fourth time, in the four hours before `file.new` was
//! written.** This table read *ninety-nine* and *97* while the assertion below
//! held **100**: `file.ocr` had been registered and the two sentences above it
//! were not moved. It is recorded rather than quietly repaired for the reason
//! the paragraph above it was: the count is the one thing here that has drifted
//! every single time, and the running tally is the argument for why nobody
//! should trust a number in this file without re-reading the assertion. Both
//! were corrected together when `file.new` took the total to 101.
//!
//! The last two are the interesting ones, because both are reachable by an
//! operator and neither is a button on a tab. They are kept honest by
//! different mechanisms and the difference is worth knowing: a `Custom` item
//! carries no command id, so `Shell::command_references()` cannot see
//! `file.recent` at all and [`super::manifest::CUSTOM_BACKED`] is the
//! register that says why that is allowed; whereas the status bar is simply
//! not part of the manifest, and `edit.find` needs no exemption because the
//! keymap's `Ctrl+F` binding **is** a reference site — so the orphan check
//! still guards it against a rename.
//!
//! A command carries five things:
//!
//! | Field | Comes from | Why here rather than the manifest |
//! |---|---|---|
//! | `id` | this file | the manifest may only *reference* it |
//! | `label`, `tooltip` | [`crate::text::commands`] | copy is a design surface with one owner |
//! | `icon` | this file, as a key | the icon *set* is the application's; the shell only needs to know which one |
//! | `enable` | this file, as a condition name | *"predicates are safety, not decoration"* |
//! | `handler` | this file, as an opaque `u64` | the shell never interprets it |
//!
//! `SHELL_FRAMEWORK.md` §5 turns that split into the customization
//! contract. An operator may reorder tabs, rename them, hide them, move a
//! command between groups, create tabs, rebind keys, and define new modes.
//! An operator may **not** invent a command, change what a command does,
//! or bypass an enable predicate. Every one of those prohibitions is a
//! consequence of this half being code and the other half being data.
//!
//! # Handler tokens are opaque, and this file does not implement behaviour
//!
//! An `egui_shell::HandlerToken` is a `u64` the shell stores and hands back
//! when the command is invoked. The application dispatches on it at **one
//! choke point**, which is where a confirmation gate, an undo entry or a
//! trace belongs; a registry of closures would scatter that across as many
//! sites as there are commands, and would force the shell to name pdfce's
//! state type, which would end its reusability.
//!
//! The numbers are assigned here in blocks of one hundred, one block per
//! tab, and they are **stable**: a token is never reused for a different
//! command, because a persisted or traced token that silently changed
//! meaning between builds is a defect with no symptom at the site that
//! caused it. Gaps in the numbering are fine and expected — a command
//! removed leaves its number unused.
//!
//! # Enable conditions
//!
//! `Enable::When("doc.open")` names a condition the application publishes
//! once per frame in an `egui_shell::commands::ConditionSet`. Data rather
//! than a closure, because a name is serializable, testable headlessly,
//! and cannot capture state that makes a command's availability depend on
//! *when* it was registered.
//!
//! Seven conditions are used, and the whole vocabulary is listed here
//! because every one of them is a promise the application has to keep:
//!
//! | Condition | True when | Used by |
//! |---|---|---|
//! | *(none)* | always | commands with no precondition: Open, Settings, the batch tools, the window and render settings |
//! | `docs.multiple` | **more than one document is open** | Next / Previous document |
//! | `doc.open` | a document is open | document-level commands — close, save a copy, properties, print |
//! | `doc.pages` | …and it has at least one page | everything that acts on a page |
//! | `undo.available` / `redo.available` | the corresponding stack is non-empty | Undo, Redo |
//! | `selection.any` | something is selected | the contextual Format tab and its Delete |
//! | `selection.bounds` | …and it still resolves to a box on the page shown | Zoom to selection |
//!
//! `docs.multiple` is deliberately **not** a refinement of `doc.open`, and it
//! is published outside that arm. A tab whose file failed to open is still a
//! tab (`crate::app::documents` §2), so an operator can be sitting on a
//! damaged file — `doc.open` false — with three good documents behind it, and
//! that is the moment they most need a way back to one of them. Nesting the
//! condition would grey the only route out of a failed open.
//!
//! `selection.bounds` is separate from `selection.any` for the same shape
//! of reason, one level down. A selection here is an **identity** — page,
//! object, subpath, node — and an identity can outlive the box it once
//! described: it may name an object on a page that is not shown, or one an
//! edit has renumbered. Zoom to selection is the command where that gap is
//! visible, because framing nothing is not a no-op; it is a jump to the
//! origin that looks exactly like a bug.
//!
//! `doc.pages` is separate from `doc.open` because **a PDF with `/Count 0`
//! is a legal document**. pdfce opens it, shows "This document has no
//! pages", and must not offer to rotate one. Collapsing the two would make
//! that file arm tools that cannot run — the exact class of failure the
//! removal of the `Editing on` master toggle was meant to end.
//!
//! Greying is what a false predicate produces, and P3 permits it only for
//! *temporarily* unavailable, *always explained on hover*. Every command
//! here has a tooltip; [`crate::text::commands`] has no way to express a
//! command without one.
//!
//! # Where the list itself lives
//!
//! [`catalog::all`] — its own file since 2026-08-14, when wiring
//! `file.save_copy` took this one past the 1,500-line gate (**R2**). The seam
//! is the one this header already draws: everything above is the **contract**
//! (what a command is, what a token means, what conditions the application
//! promises to publish), and everything in [`catalog`] is the **list** (which
//! commands exist, with which glyph and which predicate, and why each of those
//! was chosen). The icon-coverage argument moved with the registrations it
//! justifies, because an argument belongs beside the thing it argues for.
//!
//! The list is still **one flat function in one file** — see [`catalog`]'s
//! header for why splitting *that* would have been the cheaper edit and the
//! wrong one.

pub mod catalog;
pub mod mapping;

/// ★ **The sixth obligation, and the only one the five above cannot express:
/// a registered command must be REACHABLE by some arm of `app::dispatch`.**
///
/// `HANDOFF.md` §5's five obligations are all about the *registration* being
/// consistent — a count, a group count, a `PLANNED` removal, a RON
/// regeneration, a `KNOWN` condition name. Every one of them was satisfied by
/// `file.save_copy` on the day it was drawn on the quick-access toolbar, bound
/// to `Ctrl+S`, printed "(Ctrl+S)" in its own tooltip, and **did nothing**,
/// because no dispatch arm existed. [`reach`] is the assertion that closes
/// that: every id in this registry is routed by a literal arm, claimed by a
/// guard arm, or listed in [`reach::SCAFFOLDED`] with a written reason.
///
/// `#[cfg(test)]` because the reader parses `app/dispatch.rs` with `syn`, a
/// **dev**-dependency — see this crate's `Cargo.toml` for why a real parser and
/// not a grep, and [`reach`]'s own header for the two mechanisms that lost.
/// Nothing here is compiled into `pdfce-gui.exe`.
#[cfg(test)]
mod reach;

/// ★ Re-exported flat, so every caller still writes
/// `shell::commands::measure_command` and nothing outside `shell/` learns that
/// the module was split.
///
/// A `pub use` rather than moving the callers, deliberately: the split is an
/// **R2** consequence, not a change to what the shell offers, and a file-size
/// rule that rewrote fifteen call sites in `app/` would be a rule that makes
/// unrelated diffs. See `mapping`'s own header for what the seam is.
pub use mapping::{
    chrome_command, chrome_for_command, markup_command, markup_for_command, measure_command,
    measure_for_command, page_display_command, page_display_for_command, text_mark_command,
    text_mark_for_command,
};

use egui_shell::CommandRegistry;

/// **Open a document from the recent list.**
///
/// A constant rather than a literal because this id is used in four places
/// that must agree and two of them are not obvious: the registration below,
/// the `CUSTOM_BACKED` entry that records why it is on no tab, the registry
/// lookup in [`crate::app::PdfceApp::ribbon_band`] that turns the operator's
/// menu choice back into this command's token, and the dispatch arm. A typo
/// in any of them produces silence — a menu that draws and reports nothing —
/// rather than an error.
///
/// The other command ids stay literals at their (single) use sites, which is
/// this file's existing convention; this one earns a name by being spelled in
/// two modules.
pub const FILE_RECENT: &str = "file.recent"; // ui-text-exempt: a command id, never displayed

/// **Register every command the built-in manifest names.**
///
/// # Panics
///
/// If two commands claim one id. That is a programming error in
/// [`catalog::all`] and not a condition any input can produce, so it fails
/// loudly at
/// start-up rather than being swallowed: the registry refuses a duplicate
/// precisely so that behaviour cannot come to depend on the order of
/// start-up code, and catching the error here to ignore it would give back
/// exactly the defect the refusal prevents.
pub fn register(reg: &mut CommandRegistry) {
    reg.register_all(catalog::all())
        // ui-text-exempt: a panic message, read by whoever is looking at
        // the stack trace. Never rendered to an operator — the process
        // does not reach a window if this fires.
        .expect("two shell commands claim the same id");
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_shell::commands::ConditionSet;
    use std::collections::BTreeSet;

    fn registry() -> CommandRegistry {
        let mut reg = CommandRegistry::new();
        register(&mut reg);
        reg
    }

    /// Registration succeeds and produces the documented count.
    ///
    /// The number is quoted in this module's header and in
    /// `super::manifest`'s, and a silent drift makes both wrong.
    #[test]
    fn registration_succeeds_and_registers_every_command() {
        assert_eq!(registry().len(), 108);
    }

    /// ★ **The icon-coverage split adds up to the registry.**
    ///
    /// This module's header quotes a split — *"N of M named, K refused"* — and
    /// the three dated notes below it are a record of that pair drifting. It
    /// drifted a fourth time and in a way the earlier notes make embarrassing:
    /// the header read *82 of 93 named, 12 refused*, and 82 + 12 = **94**, while
    /// the registry held 93. The truth was 81 named.
    ///
    /// Nothing caught it, and the reason is stated in the note it sits under: the
    /// registry's *size* is pinned by
    /// [`Self::registration_succeeds_and_registers_every_command`] and its
    /// *split* was pinned by nothing. So this pins the split — as an identity
    /// rather than as two more literals, which is deliberate. Two literals would
    /// be two more numbers to drift; an identity is a property, and the only way
    /// to make it false is to make one of its terms genuinely wrong.
    ///
    /// The refused list lives in this module's header table rather than in code,
    /// because each entry is an **argument** and arguments belong at the
    /// registration they justify. What this asserts is that the arithmetic
    /// closes: every command either names a glyph or is one of the refusals, and
    /// the two counts partition the registry with nothing left over.
    #[test]
    fn the_icon_coverage_split_adds_up_to_the_registry() {
        let reg = registry();
        let total = reg.len();
        let named = reg.iter().filter(|c| c.icon.is_some()).count();
        let refused = total - named;
        assert_eq!(
            named + refused,
            total,
            "the split must partition the registry"
        );
        // ★ The literals, and they are now the ONLY copy of these numbers.
        //
        // This block used to end "update that sentence together", pointing at
        // `catalog`'s `## Coverage` heading. It drifted anyway — a fifth time,
        // to *86 of 101 named, 15 refused* against a registry of 94 — because a
        // test cannot enforce a comment. The heading no longer carries numbers
        // and this is where they live: a literal that fails loudly beats a
        // sentence that is wrong quietly.
        //
        // Failing here means the registry changed. Read the diff, decide
        // whether the new command should have a glyph, and move the number
        // that is genuinely wrong.
        assert_eq!(named, 96, "commands naming an icon");
        assert_eq!(
            refused, 12,
            "commands with no icon, each argued at its registration"
        );
        // Each refusal is argued at its own registration and listed in the
        // header's table. Asserting the ids too would be a third copy of that
        // list; asserting the *count* is what stops a glyph being quietly
        // dropped from a control that had one.
    }

    /// **★ No two commands share a handler token.**
    ///
    /// The shell explicitly permits it — two ids may share a token if the
    /// application wants two names for one handler — which is exactly why
    /// this needs asserting on *our* side. pdfce has no such pair, so a
    /// collision here is a typo in a hand-assigned number, and its symptom
    /// would be one command silently doing another's work. Nothing else in
    /// the system can detect that.
    #[test]
    fn every_handler_token_is_unique() {
        let mut seen: BTreeSet<u64> = BTreeSet::new();
        for command in registry().iter() {
            assert!(
                seen.insert(command.handler.get()),
                "handler token {} is assigned twice; `{}` collides with an earlier command",
                command.handler.get(),
                command.id
            );
        }
    }

    /// Handler tokens sit in their tab's hundred-block.
    ///
    /// The blocks are what make a collision improbable in the first place
    /// and what makes a raw token in a trace readable — `4xx` is an Edit
    /// command without looking anything up. A number in the wrong block is
    /// how the next one gets assigned on top of an existing command.
    #[test]
    fn every_handler_token_is_in_its_tabs_block() {
        let blocks = [
            ("file.", 100),
            ("view.", 200),
            ("pages.", 300),
            ("edit.", 400),
            ("markup.", 500),
            ("measure.", 600),
            ("tools.", 700),
            ("format.", 800),
            ("mode.", 900),
        ];
        for command in registry().iter() {
            let (prefix, base) = blocks
                .iter()
                .find(|(p, _)| command.id.starts_with(p))
                .unwrap_or_else(|| panic!("`{}` has no known prefix", command.id));
            let token = command.handler.get();
            assert!(
                (*base..base + 100).contains(&token),
                "`{}` has token {token}, outside the `{prefix}` block {base}..{}",
                command.id,
                base + 100
            );
        }
    }

    /// Every enable condition is one of the five documented names.
    ///
    /// A predicate naming a condition the application never publishes is a
    /// command that is permanently greyed — and it fails silently, because
    /// an unset condition and a false condition are the same value. The
    /// vocabulary is small on purpose; this is what keeps it small.
    #[test]
    fn every_predicate_names_a_documented_condition() {
        const KNOWN: &[&str] = &[
            // ★ NOT nested inside `doc.open` where it is published, and the
            // header says why: the one state that needs it most is a failed
            // open with other documents behind it.
            "docs.multiple",
            "doc.open",
            "doc.pages",
            "undo.available",
            "redo.available",
            "selection.any",
            // Not a refinement of `selection.any` — see `PdfceApp::conditions`.
            // A selection can exist and resolve to no box.
            "selection.bounds",
            // ★ The only condition about a **gesture in progress** rather
            // than about the document, the selection or the view.
            //
            // `measure.finish` ends the radius/diameter gesture, which is the
            // one gesture on the canvas with no natural end, so its control
            // must be live exactly when there is something to end — a Finish
            // that is always enabled is a control that does nothing on almost
            // every press. Published by `PdfceApp::conditions` from
            // `canvas::measure::finishable`, which is the same derivation the
            // command's own arm asks, so the control cannot be enabled while
            // pressing it would do nothing.
            "measure.finishable",
            // ★ **A live text selection**, and the second condition here about
            // something other than the document or the view.
            //
            // The three Text markup commands act on the selection rather than
            // arming a tool (`canvas::markup::text` §1), so without one they
            // would be controls that do nothing on almost every press. It is
            // **not** a refinement of `selection.any`, which is the *object*
            // selection: the two are mutually exclusive by construction
            // (`canvas::textsel` §3), so a build that confused them would grey
            // these three in exactly the mode where they work.
            //
            // "Live" is part of the name's meaning rather than a detail: a
            // selection resolved against a revision that has since moved is
            // refused by `markup::text::mark`, and the condition asks the same
            // question so the control cannot be enabled while the press would
            // decline.
            "selection.text",
            // ★ **A vertex run ready to be committed** — `measure.finishable`'s
            // twin, and the second condition here about a **gesture in progress**.
            //
            // `markup.finish` ends the PolyLine and Polygon gestures, which are
            // the only markup gestures with no natural end: a band drag and a
            // freehand stroke both end when the button comes up, and a run of
            // clicks does not end itself. So its control must be live exactly
            // when there is a run to end — a Finish that is always enabled is a
            // control that does nothing on almost every press.
            //
            // Published by `PdfceApp::conditions` from
            // `canvas::markup::vertex::finishable`, which is the same derivation
            // the command's own arm asks, so the control cannot be enabled while
            // pressing it would do nothing. It is **not** a refinement of
            // `measure.finishable`: a measure tool and a markup tool cannot both
            // be armed, so exactly one of the two can ever be true, and a build
            // that collapsed them would light one tab's Finish from the other
            // tab's gesture.
            "markup.finishable",
        ];
        for command in registry().iter() {
            if let egui_shell::commands::Enable::When(name) = &command.enable {
                let bare = name.strip_prefix('!').unwrap_or(name);
                assert!(
                    KNOWN.contains(&bare),
                    "`{}` waits on `{name}`, which is not a published condition",
                    command.id
                );
            }
        }
    }

    /// **With no document open, only the commands that make sense without
    /// one are available.**
    ///
    /// The headless equivalent of launching pdfce and looking at the
    /// ribbon. It is asserted as an exact set rather than a count, because
    /// the interesting failure is a *specific* command escaping its
    /// predicate — `pages.delete` live with nothing open — and a count
    /// would pass as long as some other command lost one.
    #[test]
    fn with_no_document_only_the_document_free_commands_are_enabled() {
        let nothing = ConditionSet::new();
        let reg = registry();
        let live: BTreeSet<&str> = reg
            .iter()
            .filter(|c| c.is_enabled(&nothing))
            .map(|c| c.id.as_str())
            .collect();

        let expected: BTreeSet<&str> = [
            // About describes pdfce, so it is offered before anything is
            // open — see its registration.
            "file.about",
            // ★ New has no predicate for the strongest version of `file.open`'s
            // reason: an empty shell is not a state New is *tolerated* in, it is
            // the state New exists for. A `doc.open` gate here would grey the
            // one control that answers "there is nothing here".
            "file.new",
            // The sized New, for exactly `file.new`'s reason. Two commands
            // that both answer "there is nothing here" must both be reachable
            // from that state, and a predicate on one of them would be a
            // difference between siblings with no argument behind it.
            "file.new_from_template",
            "file.open",
            // Available with nothing open, like `file.open`, and for the same
            // reason: it is how you GET a document. Its own control greys
            // itself when the list is empty — see the registration's comment
            // on why that rule lives with the menu rather than in a sixth
            // published condition.
            "file.recent",
            "file.settings",
            "file.shortcuts",
            "mode.edit",
            "mode.read",
            "mode.review",
            "tools.font_folders",
            "tools.merge_files",
            "tools.split_files",
            "view.fullscreen",
            // ★ The Tool panel is live with NOTHING OPEN, and it is the only
            // panel toggle that is. Its body still names the tools this mode
            // has and where they live on the ribbon, which is exactly what an
            // operator who has just launched pdfce is looking for. Gating it on
            // `doc.open` would hide the surface at the one moment it answers
            // the question it was built for.
            "view.panel_tool",
            "view.read_mode",
            "view.reset_layout",
            "view.sidebar",
        ]
        .into_iter()
        .collect();

        assert_eq!(live, expected);
    }

    /// A document with no pages is a legal document, and it must not arm
    /// anything that acts on a page.
    ///
    /// `/Count 0` is valid PDF. pdfce opens such a file and says "This
    /// document has no pages" rather than reporting a failure — so the
    /// condition set it publishes has `doc.open` and not `doc.pages`, and
    /// this asserts the consequence.
    #[test]
    fn an_empty_document_arms_nothing_that_needs_a_page() {
        let empty_doc = ConditionSet::new().with("doc.open");
        let reg = registry();
        for id in [
            "pages.rotate_left",
            "pages.delete",
            "edit.text",
            "markup.rectangle",
            "measure.linear",
            "view.zoom_fit_page",
        ] {
            assert!(
                !reg.get(id).expect("registered").is_enabled(&empty_doc),
                "`{id}` acts on a page and must not be armed by a document with none"
            );
        }
        // …while the document-level commands are live, because there is a
        // document: its properties, its fonts and its metadata all exist.
        for id in ["file.properties", "file.fonts", "file.close"] {
            assert!(reg.get(id).expect("registered").is_enabled(&empty_doc));
        }
    }

    /// Undo and redo are the canonical *temporarily* unavailable pair.
    #[test]
    fn undo_and_redo_follow_their_stacks() {
        let reg = registry();
        let undo = reg.get("edit.undo").expect("registered");
        let redo = reg.get("edit.redo").expect("registered");
        let nothing = ConditionSet::new();
        assert!(!undo.is_enabled(&nothing));
        assert!(!redo.is_enabled(&nothing));
        assert!(undo.is_enabled(&ConditionSet::new().with("undo.available")));
        assert!(redo.is_enabled(&ConditionSet::new().with("redo.available")));
        // And each has a tooltip, which is what P3 requires of anything
        // that can be greyed.
        assert!(undo.tooltip.is_some());
        assert!(redo.tooltip.is_some());
    }

    /// Every registered command has a tooltip.
    ///
    /// The catalog type makes this structurally true, so the test is
    /// guarding the *wiring*: a command built with `Command::new` and
    /// never given `.with_tooltip` would compile.
    #[test]
    fn every_command_has_a_tooltip() {
        for command in registry().iter() {
            assert!(
                command
                    .tooltip
                    .as_ref()
                    .is_some_and(|t| !t.trim().is_empty()),
                "`{}` has no tooltip; greying it would be unexplainable",
                command.id
            );
        }
    }

    /// Icon keys are lower-case kebab, matching the salvaged icon set's
    /// naming.
    ///
    /// A key that does not match the set's spelling resolves to nothing at
    /// run time and renders as a missing glyph — a placeholder arriving
    /// through the back door, and one that no headless test would
    /// otherwise see.
    #[test]
    fn icon_keys_are_kebab_case() {
        for command in registry().iter() {
            let Some(icon) = &command.icon else { continue };
            assert!(
                icon.chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
                "`{}` names icon `{icon}`, which is not lower-case kebab",
                command.id
            );
        }
    }
}
