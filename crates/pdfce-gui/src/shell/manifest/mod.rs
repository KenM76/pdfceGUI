//! # shell::manifest — pdfce's ribbon, as an `egui_shell::Shell` value
//!
//! [`built_in`] returns the complete pdfce shell: eight tabs (seven
//! ordinary plus the contextual Format tab), thirty-two groups, three
//! modes, the quick-access toolbar and the keymap. It is the **built-in
//! layer** of `SHELL_FRAMEWORK.md` §4's three-layer merge:
//!
//! 1. **Built-in** — this function. Compiled into the binary, always
//!    valid, and always available as the reset target.
//! 2. **Application override** — an optional file shipped beside the exe.
//! 3. **Operator customization** — `userdata/shell.ron`.
//!
//! Layers 2 and 3 override this one **per item**, never wholesale. That is
//! why this layer has to be complete and has to validate: it is the thing
//! every other layer is a patch against, and it is what an operator gets
//! back when they reset.
//!
//! One file per tab. The tab modules are where the *reasoning* lives —
//! why a command sits where it does, what moved, what was left out and
//! why — and they are worth reading before changing anything here.
//!
//! # ★ The no-placeholders rule, and the two registers that keep it honest
//!
//! `RIBBON_IA.md` P3: *an unavailable capability renders nothing, not a
//! disabled stub.* Greying is reserved for **temporarily** unavailable —
//! no document open, undo stack empty — and is always explained on hover.
//!
//! `RIBBON_IA.md` §5 marks every command it specifies with where it exists
//! today:
//!
//! | Mark | Meaning | In this manifest |
//! |---|---|---|
//! | **G** | exists in the GUI now | emitted |
//! | **C** | exists in `pdfce-core`/`pdfce-cli`, no GUI surface | **absent**, in [`PLANNED`] |
//! | **N** | exists nowhere | **absent**, in [`PLANNED`] |
//!
//! A **C** row is the cheapest kind of missing command — the hard half is
//! written and tested — and it is still absent, because P3 is about what
//! the operator can reach and an engine with no caller is not reachable.
//!
//! Absent is not forgotten. Two registers make the difference visible:
//!
//! - **[`PLANNED`]** — every specified command this manifest does *not*
//!   emit, with the reason. Tested in both directions: nothing in it is
//!   referenced by the manifest, and nothing in it is registered. That is
//!   the list a later stage reads to find its work.
//! - **[`DIRECTED`]** — the small set of commands emitted *despite* not
//!   carrying a **G** mark, each with the instruction that put it there.
//!   Without this list those seven entries would look like the manifest
//!   quietly ignoring P3.
//!
//! # Command ids
//!
//! Dotted lowercase, and the prefix is **the tab that owns the command**:
//! `view.zoom_fit_page`, `pages.rotate_left`, `markup.highlight`. That
//! makes P1 — one command, one tab — legible in the id itself, and it
//! makes a violation obvious on sight rather than only at validation.
//!
//! Two deliberate exceptions:
//!
//! - `edit.undo` and `edit.redo` sit on **no tab**. They live on the QAT
//!   alone, which `RIBBON_IA.md` §7 keeps unchanged. The `edit.` prefix
//!   says where they would go if they ever got one.
//! - `mode.read`, `mode.review` and `mode.edit` are not tab commands at
//!   all: they are the three positions of the selector at the far right of
//!   the tab row, reachable from the keymap.
//!
//! # What this module deliberately does not decide
//!
//! **Icons, labels, tooltips and enable predicates** — those are the
//! registry's half of the split, in [`super::commands`]. A manifest
//! contains command *ids* and nothing else about them, which is what stops
//! a customized ribbon from inventing a command and what makes an unknown
//! id a disclosed skip rather than a crash.
//!
//! **Behaviour.** Nothing here runs.

mod edit;
mod file;
mod format;
mod markup;
mod measure;
mod pages;
mod tools;
mod view;

use crate::text::ribbon;
use egui_shell::manifest::{Group, Item, Mode, Shell};

/// **The complete pdfce shell.**
///
/// Deterministic and side-effect free: called from tests, from the RON
/// round-trip check, and once at start-up. It allocates a few kilobytes of
/// `String` and does nothing else.
///
/// # Order is presentation
///
/// Tabs appear in the order they are added, groups in the order they are
/// listed, and items in the order they are written. `RIBBON_IA.md` §4's
/// table is the tab order — File, View, Pages, Edit, Markup, Measure,
/// Tools — and it is not arbitrary: it runs from what you do to the file,
/// through what you look at, to what you change, to what you add, ending
/// at the things that run across other files.
///
/// # The menus are part of this value, not a second document
///
/// `SHELL_FRAMEWORK.md` §1 says the shell is *one* serializable document,
/// and `egui_shell::Shell` carries `menus` beside `tabs` for exactly that
/// reason: one file to ship, one file to merge, one file for the operator
/// to edit, and — the payoff that decided it — **one keymap**, so a menu
/// row's chord hint is derived from the same bindings the ribbon uses
/// rather than written down twice. See [`super::menus`] for what is in
/// them and why.
#[must_use]
pub fn built_in() -> Shell {
    let mut shell = Shell::new()
        // -------------------------------------------------------------------
        // MODES — `MODES_AND_PANELS.md` Part 1.
        //
        // Three positions of one control, ORDERED BY CAPABILITY: each
        // mode's tab set is a superset of the one before it. That ordering
        // is the whole premise —
        //
        //   > The three positions are ordered by capability… A slider says
        //   > that; three toggle buttons do not, and a dropdown hides the
        //   > current position behind a click. The ordering is the
        //   > information.
        //
        // — and `each_mode_is_a_subset_of_the_next` in `super`'s tests is
        // what keeps it true. `egui-shell` cannot check it: a different
        // application may ship three unrelated workspaces, and assuming an
        // ordering there would be the framework legislating about content.
        //
        // Modes are declared before tabs so this list reads as the summary
        // of the ribbon that follows.
        //
        // **Pages is in Review.** Operator decision, 2026-08-13, reversing
        // an earlier draft that excluded it on the reasoning that delete,
        // extract and merge are structural. Reviewing a drawing set means
        // rotating a sheet to read it, extracting the two pages you were
        // asked about, and inserting a marked-up revision — all reviewer
        // work. The stance that matters is *the page content is not yours
        // to alter*, and page operations do not alter content.
        //
        // The contextual Format tab is in NO mode's list, and is present
        // in all three. See `format.rs`.
        // -------------------------------------------------------------------
        .with_mode(Mode::new("read", ribbon::mode_read(), ["file", "view"]))
        .with_mode(Mode::new(
            "review",
            ribbon::mode_review(),
            ["file", "view", "pages", "markup", "measure"],
        ))
        .with_mode(Mode::new(
            "edit",
            ribbon::mode_edit(),
            [
                "file", "view", "pages", "edit", "markup", "measure", "tools",
            ],
        ))
        // -------------------------------------------------------------------
        // TABS
        // -------------------------------------------------------------------
        .with_tab(file::tab())
        .with_tab(view::tab())
        .with_tab(pages::tab())
        .with_tab(edit::tab())
        .with_tab(markup::tab())
        .with_tab(measure::tab())
        .with_tab(tools::tab())
        .with_contextual_tab(format::tab())
        // -------------------------------------------------------------------
        // QUICK-ACCESS TOOLBAR — `RIBBON_IA.md` §6, unchanged from today.
        //
        // Open, Save a copy, Undo, Redo. Two of them mirror the File tab,
        // which amendment P1a permits explicitly: *the QAT and the status
        // bar are shortcut surfaces, not tabs. A command may appear on
        // exactly one tab and additionally on the QAT and/or the status
        // bar.* The other two appear nowhere else, which is why the QAT
        // being always visible is load-bearing rather than convenient.
        // -------------------------------------------------------------------
        .with_qat(["file.open", "file.save_copy", "edit.undo", "edit.redo"])
        // -------------------------------------------------------------------
        // KEYMAP
        //
        // Chords are opaque strings here; parsing them into modifiers and a
        // key is the renderer's job, and doing it in the manifest would
        // mean a manifest could not be read by a tool that does not link
        // egui.
        //
        // Every binding below is one the shipped build already honours and
        // already documents in its own keyboard-shortcuts window, EXCEPT
        // the last five, which are new and are marked. Carrying the
        // existing spellings verbatim — including `Ctrl+Y or Ctrl+Shift+Z`
        // as two separate bindings for redo — is what stops the shortcut
        // list and the keymap from being two sources of truth.
        //
        // ★ THIS KEYMAP IS THE ONLY PLACE A CHORD IS BOUND TO A MEANING.
        //
        // `crate::app::keyboard::commands` reads it at run time and hands the
        // command id to `PdfceApp::dispatch_command` — the same dispatcher a
        // ribbon click reaches — so a chord cannot disagree with the control
        // that shares its command. It used to: this file bound `Ctrl+0` to
        // `view.zoom_actual` and `Ctrl+2` to `mode.review` while
        // `app::keyboard` bound the same two chords to fit page and fit
        // width and got there first, because nothing dispatched this keymap
        // at all. Two operator-visible surfaces named the chords, and both
        // were lying.
        //
        // The list below is therefore load-bearing rather than explanatory,
        // and `app::keyboard::tests::no_chord_has_two_owners` enforces it:
        // every chord `app::keyboard::collect` binds outright is checked
        // against this map, in every spelling, and the test fails naming the
        // chord and both claimants. Add a binding here for one of them and it
        // goes red.
        //
        // NOT bound here, deliberately:
        //
        //   Delete / Backspace   The shortcut window lists these for
        //                        "delete the selected pages", but the same
        //                        keys delete a selected OBJECT on the
        //                        canvas. Which one applies depends on where
        //                        focus is, and a global binding cannot
        //                        express that. It stays canvas-scoped.
        //   PageUp / PageDown / Home / End / Ctrl+Plus / Ctrl+Minus
        //                        Viewer navigation, handled in the app's
        //                        own keyboard layer against the view state.
        //                        They are not ribbon commands and putting
        //                        them here would give them a second owner.
        // ★ Ctrl+F IS bound here now, and the comment it replaces is worth
        // keeping visible because it was right about the control and wrong
        // about the chord:
        //
        //     Ctrl+F   Find lives in the status bar, which this manifest
        //              does not describe.
        //
        // The first clause still holds — `RIBBON_IA.md` §6 puts the Find
        // TOGGLE on the status bar, and `edit.find` is on no tab. The second
        // does not follow from it. A keymap is not a description of controls;
        // it is the ONE place a chord is bound to a meaning, which is the
        // property the two-owner defect was fixed by establishing. Leaving
        // Ctrl+F out of it would have meant binding it in
        // `crate::app::keyboard` instead — a second owner, in the module whose
        // whole header is about why there must not be one.
        //
        // So the rule is: the manifest binds every chord, whatever surface
        // the control lives on; a surface this manifest does not describe is
        // a reason for the command to be on no TAB, not a reason for its chord
        // to be bound somewhere else.
        // -------------------------------------------------------------------
        // ★ Ctrl+N — the universal chord, bound the day its command landed.
        //
        // Acrobat, Inkscape and SolidWorks all bind Ctrl+N to New, as does
        // every other document application; there was nothing to decide here
        // beyond whether it was allowed to be bound at all, and the rule in
        // `crate::app::keyboard::DERIVED`'s header says it is: *"a chord here
        // dispatches a command, and a command with no dispatch arm would trace
        // `command-unimplemented` on a keypress that used to do nothing
        // quietly. They land with their commands."* `file.new` has an arm, so
        // the chord lands with it — and `Key::N` joins `DERIVED`'s spelling
        // table in the same edit, because a chord this file binds and that
        // table cannot spell is a chord no keypress delivers. That is the
        // defect `Ctrl+O` sat in for the whole life of the ribbon.
        .with_binding("Ctrl+N", "file.new")
        .with_binding("Ctrl+O", "file.open")
        .with_binding("Ctrl+S", "file.save_copy")
        .with_binding("Ctrl+Z", "edit.undo")
        .with_binding("Ctrl+Y", "edit.redo")
        .with_binding("Ctrl+Shift+Z", "edit.redo")
        .with_binding("Ctrl+E", "edit.text")
        .with_binding("Ctrl+Shift+E", "edit.add_text")
        // ★ Was bound to `edit.copy_page_text` until 2026-08-14. The COMMAND
        // moved to File ▸ Export and the chord followed it here, in the same
        // edit, because this keymap is the only place a chord is bound to a
        // meaning: a binding left pointing at the old id would not fail the
        // build — an unknown id is a disclosed skip, not an error — it would
        // simply make `Ctrl+Shift+C` do nothing, which is the silent failure
        // this block's header is entirely about.
        //
        // The move is what makes the chord work in **Read**: the gate in
        // `crate::app::modes::capability::offers_command` lets a chord reach a
        // command the active mode shows, and Read shows File.
        .with_binding("Ctrl+Shift+C", "file.copy_page_text")
        .with_binding("Ctrl+F", "edit.find")
        .with_binding("Ctrl+0", "view.zoom_actual")
        .with_binding("[", "pages.rotate_left")
        .with_binding("]", "pages.rotate_right")
        .with_binding("Alt+Up", "pages.move_up")
        .with_binding("Alt+Down", "pages.move_down")
        // New. `RIBBON_IA.md` §3 records Ctrl+H and F11 as the only way to
        // reach read mode and full screen in the shipped build — they have
        // no ribbon control at all — but no such string appears anywhere in
        // its source, so the operator has no way to discover them. Binding
        // them here alongside the View ▸ Window controls gives them both a
        // visible home and a documented chord.
        .with_binding("Ctrl+H", "view.read_mode")
        .with_binding("F11", "view.fullscreen")
        // New. `MODES_AND_PANELS.md` Part 1 §6 specifies these three, and
        // adds that the selector must also be a real focusable control with
        // arrow-key movement — not a mouse-only affordance.
        .with_binding("Ctrl+1", "mode.read")
        .with_binding("Ctrl+2", "mode.review")
        .with_binding("Ctrl+3", "mode.edit");
    // -------------------------------------------------------------------
    // CONTEXT MENUS — `RIBBON_IA.md` §6, "the other half of making
    // selection meaningful".
    //
    // Assigned rather than chained because `egui_shell::Shell` has no
    // `with_menus` builder, and adding one is `egui-shell`'s change to
    // make, not this crate's. The field is public and the assignment is
    // one line; a builder method would be nicer and is not worth reaching
    // across a crate boundary for.
    //
    // `Shell::validate` does NOT check this half — it walks tabs, the QAT
    // and the keymap — so `super::menus`' own tests carry the checks that
    // matter: every command a menu names is registered, and every menu has
    // something to offer.
    // -------------------------------------------------------------------
    shell.menus = Some(super::menus::built_in());
    shell
}

/// **The condition, published by the application each frame, under which
/// something is selected on the page.**
///
/// One spelling, one source. It is the Format tab's `visible_when`, the
/// enable predicate of the `format.delete` inside it, and the condition
/// [`super::menus::MenuHost::with_condition`] corrects when a right-click
/// selects the object under the pointer. Three surfaces reading three
/// spellings of one condition is a defect whose only symptom is a tab that
/// appears holding one greyed control.
///
/// Re-exported from [`format`] rather than declared here, because the tab
/// is where the condition is *decided* and this is where it is *reachable*
/// from outside the manifest.
pub const SELECTION_ANY: &str = format::VISIBLE_WHEN;

/// **The `Item::Custom` kind of the Recent-documents control.**
///
/// One spelling, one source: the manifest writes it in File ▸ File and
/// [`crate::app::PdfceApp::ribbon_band`]'s custom-item renderer matches on
/// it. A mismatch between those two is invisible — the shell reserves the
/// item's space, the application declines to draw it, and the band shows a
/// gap — so the string is a constant rather than a literal in two files.
pub const RECENT_FILES: &str = "recent_files"; // ui-text-exempt: a custom-item kind, never displayed

// ===========================================================================
// CUSTOM_BACKED
// ===========================================================================

/// **Registered commands whose only ribbon control is an [`Item::Custom`],
/// and the item that draws each.**
///
/// `(command id, custom kind, why)`.
///
/// # Why this register has to exist
///
/// `egui_shell::Shell::command_references()` walks tab groups, the QAT and
/// the keymap — the places a command *id* can appear. A `Custom` item carries
/// no id (that is the whole point of it: the shell reserves space and the
/// application draws whatever it likes), so a command reachable only through
/// one is invisible to every reachability check built on that function.
///
/// `super::tests::no_registered_command_is_orphaned` is exactly such a check,
/// and it is a good one: it catches the rename that leaves a command
/// registered and referenced by nothing, which nothing in `egui-shell` can
/// see. Without this register it would have to be either weakened — which
/// gives up the rename check for every other command — or satisfied by
/// putting a second, redundant button on the tab.
///
/// So the exception is **data**, exactly as [`PLANNED`] and [`DIRECTED`] are:
/// enumerable, tested in both directions (the id is registered, and the kind
/// really appears in the manifest), and carrying its reason. A command listed
/// here whose custom item was deleted fails the suite rather than becoming an
/// unreachable command with a note explaining why it used to be fine.
///
/// # The bar for an entry
///
/// The control must genuinely be one a **button cannot be**. `file.recent`
/// qualifies because the command needs an operand — *which* of ten documents
/// — that a button has no way to ask for, and the alternatives are ten
/// commands or a command that opens whichever file it feels like. A command
/// that could have been a button and was drawn some other way for taste does
/// not belong here; it belongs on the tab.
pub const CUSTOM_BACKED: &[(&str, &str, &str)] = &[(
    crate::shell::commands::FILE_RECENT,
    RECENT_FILES,
    "The Recent menu in File ▸ File. The command opens a document from the recent list, and \
     WHICH document is a ten-way choice a button cannot express — so the ribbon control is a \
     menu the application draws (`app::recent::menu`), which parks the chosen path and returns \
     this command's token. Same shape as `file.open`, whose operand comes from a file dialog: \
     the picker asks, the command acts.",
)];

/// A captioned band of items.
///
/// A two-line convenience over `Group::new(..).with_items(..)`, because
/// this manifest writes thirty-two of them and the builder chain is the
/// noisiest thing on the page when every group is one expression.
fn group(id: &str, caption: &str, items: impl IntoIterator<Item = Item>) -> Group {
    Group::new(id, caption).with_items(items)
}

/// A command reference, by id.
///
/// Named `command` rather than used as `Item::command` so that a tab
/// module's item lists read as a list of commands, which is what they are.
fn command(id: &str) -> Item {
    Item::command(id)
}

// ===========================================================================
// PLANNED
// ===========================================================================

/// **Every command `RIBBON_IA.md` specifies that this manifest does not
/// emit, and why.**
///
/// `(id, reason)`. The reason is the entry's whole value: it is what lets
/// a later stage tell a **C** row — engine written and tested, shell
/// missing, a day's work — from an **N** row that is a month, without
/// re-deriving the analysis from the specification each time.
///
/// # Why this exists rather than a comment
///
/// P3 says an unavailable capability renders nothing. Applied literally
/// and alone, that turns a specification of 180-odd commands into a
/// manifest of 76 with no record of the other 100, and the next person to
/// read this module cannot tell a command that was *considered and
/// deferred* from one that was *never noticed*. Those are very different
/// facts and only one of them is a plan.
///
/// So the omissions are data:
///
/// - **tested**, in both directions — `planned_commands_are_genuinely_absent`
///   asserts nothing here is referenced by the manifest *and* nothing here
///   is registered, so an entry that gets built and not removed fails the
///   suite rather than becoming a stale comment;
/// - **enumerable**, so a diagnostic surface or a roadmap tool can list
///   the gap;
/// - **greppable by id**, so the search that finds `measure.two_line` in
///   the manifest also finds the note saying where it went.
///
/// # Ordering
///
/// By tab, in the tab order of [`built_in`], then in the order
/// `RIBBON_IA.md` §5 lists them within their group. Not sorted
/// alphabetically: this list is read against the specification, and a
/// reader checking §5.3 against it wants the Pages entries together and in
/// the document's order.
pub const PLANNED: &[(&str, &str)] = &[
    // -- File -- `RIBBON_IA.md` §5.1 ----------------------------------------
    //
    // ★ `file.new` was here — "N — a blank or from-template document. pdfce has
    // no document-creation path at all." It shipped on 2026-08-14, and the note
    // is kept as a comment for the same reason `file.recent`'s is: "this used to
    // be planned and is now built" is the one transition this list exists to
    // make legible. Its second sentence remains true of the ENGINE and always
    // will — `pdfce-core`'s `document.rs:10-19` states "no separate
    // builder/generation model may ever be introduced" as a named invariant —
    // which is exactly why the shipped command opens a bundled blank template
    // instead of asking pdfce to grow a creation path. See `crate::app::blank`.
    //
    // What did NOT ship is the other half of §5.1's row, and it has its own
    // entry below rather than being folded into a comment, because it is a
    // capability an operator will ask for by name.
    (
        "file.new_from_template",
        "N — §5.1's `New (blank / from template)` row shipped only its BLANK half. This is \
         where a page-SIZE choice belongs: `file.new` makes A4 with no dialog, which is what \
         Acrobat and Inkscape do, and an operator whose sheets are A3 and A1 will want to say \
         so. Inkscape's own split is the shape to copy — Ctrl+N makes a document, Ctrl+Alt+N \
         chooses what kind — so this is a second command and not a dialog bolted onto the \
         first. It needs one template asset per offered size (each ~450 bytes, own work, \
         covered by `crates/pdfce-gui/src/app/assets/PROVENANCE.md`) and a chooser; no engine \
         work at all.",
    ),
    // `file.recent` was here — "N — needs a persisted recent-files list;
    // nothing writes one today." Something writes one now
    // (`crate::app::recent`), so the command is registered, the `recent_files`
    // custom item in File ▸ File draws it, and the entry moved to
    // `CUSTOM_BACKED`. Recorded as a comment rather than silently deleted
    // because "this used to be planned and is now built" is the one transition
    // this list exists to make legible.
    (
        "file.save",
        "N — in-place save is blocked on autosave and crash recovery. Until then the Save \
         group holds `Save a copy…` alone and this does not render at all, per P3.",
    ),
    (
        "file.revert",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — meaningless until there is a save point to revert to, so it follows `file.save`.",
    ),
    (
        "file.export_image",
        "C — pdfce-core rasterises to PNG/JPEG/TIFF already. Needs a DPI picker and a save \
         dialog; no engine work.",
    ),
    (
        "file.export_text",
        "C — pdfce-core extracts text already. Needs a save dialog and nothing else. Not to be \
         confused with `file.copy_page_text` / `file.copy_document_text`, which sit in the same \
         band and are shipped: those write the extracted text to the CLIPBOARD, this one writes \
         it to a file the operator names.",
    ),
    (
        "file.imposition",
        "C — n-up, booklet and poster imposition exist in core and the CLI. Needs a \
         print-time dialog.",
    ),
    (
        "file.security",
        "N — no encryption or permissions surface. Encryption is disclosed in the status bar \
         today, and opening a signed or encrypted document into Read mode is the nearer fix.",
    ),
    // `file.about` was here — "N — there is no about box." There is one now
    // (`crate::dialogs::about`), so the command is registered and draws in
    // File ▸ pdfce. Recorded as a comment rather than silently deleted, on the
    // `file.recent` precedent above: "this used to be planned and is now
    // built" is the one transition this list exists to make legible.
    //
    // Worth keeping the reason it stopped being optional. The box was N for as
    // long as this shell redistributed only permissively-licensed code, whose
    // notices the shipped `LICENSE` covers. The operator's 2026-08-14 decision
    // to ship CC-BY-SA-4.0 OCR model weights ends that: BY requires the notice
    // to reach the RECIPIENT of the work, and nothing in this program reached
    // them. See `crate::text::about`.
    // -- View -- `RIBBON_IA.md` §5.2 ----------------------------------------
    // ★ `view.page_continuous`, `view.page_facing` and
    // `view.page_facing_continuous` were here until Phase 4, marked N with the
    // note that the build was *"larger than it looks: the viewer holds a single
    // page index and the object provider returns nothing for any page but the
    // current one"*. All three are now emitted by `view.rs` and registered by
    // `super::commands`, so they are removed from this list rather than left
    // with a stale reason — `planned_commands_are_genuinely_absent` asserts in
    // both directions and fails on an entry that has shipped.
    (
        "view.rotate_view_left",
        "N — rotates the VIEW without changing the document, which is a different command \
         from `pages.rotate_left` and is the one a reader wants.",
    ),
    (
        "view.rotate_view_right",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — as `view.rotate_view_left`, clockwise.",
    ),
    // ★ `view.rulers`, `view.grid` and `view.guides` were here until the
    // rulers landed, marked N. Their notes were *"rulers along the canvas
    // edges, in the document's units"*, *"a drawing grid drawn over the
    // page"*, and — the one with a condition attached — *"draggable guides,
    // which need a per-document store to survive a reopen."*
    //
    // All three are now emitted by `view.rs` and registered by
    // `super::commands`, so they are removed from this list rather than left
    // with a stale reason: `planned_commands_are_genuinely_absent` asserts in
    // both directions and fails on an entry that has shipped. The condition on
    // the third is discharged by `crate::canvas::guides`, whose header records
    // why `guides.txt` is a fourth store beside `layout.ron`, `recent.txt` and
    // `page-display.txt` rather than a field in any of them.
    //
    // "The document's units" turned out to be the interesting half of that
    // first note; `crate::canvas::rulers`' header §1 is the answer.
    // ★ `view.panel_pages` was here, with the reason *"page thumbnails are the
    // sidebar rail's first pane and have no independent toggle;
    // `view.sidebar` shows the rail"*. That reason described the OLD shell's
    // rail, which this build does not have — the Pages panel is an ordinary
    // dock panel like Bookmarks and Layers, so it needs an ordinary panel
    // toggle. The entry was stale rather than early, and it is removed rather
    // than reworded because the command is now registered and drawn.
    //
    // `every_panel_is_reachable_from_the_ribbon` is the test that made the
    // staleness visible: the panel existed, was filtered out of every mode by
    // the §5b capability rule, and no operator could open it.
    // ★ `view.panel_forms` was here too, with the reason *"there is no
    // standalone Forms panel; the forms surface is reached from Edit ▸
    // Forms"*. Both halves were true when written and the first stopped
    // being true when the Forms panel shipped — the entry survived because
    // `edit.form_fill` was still the way in, so nothing forced the question.
    //
    // What forced it was the operator's answer on 2026-08-14 that Read
    // fills forms. Read is shown `file` and `view` alone, so this id — the
    // one this list had reserved to say the panel had no toggle of its own
    // — is now that toggle, and `edit.form_fill` is the entry that no
    // longer exists. Recorded as a comment rather than silently deleted for
    // the same reason as `file.recent` above: *"this used to be planned and
    // is now built"* is the one transition this list exists to make legible.
    (
        "view.save_workspace",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — named workspaces are a superset of layout persistence, which lands at stage S3.",
    ),
    (
        "view.load_workspace",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — as `view.save_workspace`.",
    ),
    // -- Pages -- `RIBBON_IA.md` §5.3 ---------------------------------------
    (
        "pages.insert_blank",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "C — pdfce-core inserts blank pages already. Needs a size-and-count dialog only.",
    ),
    (
        "pages.insert_scan",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — there is no scanner acquisition path of any kind.",
    ),
    (
        "pages.replace",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — replace the selected pages with pages from another file.",
    ),
    (
        "pages.crop",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — needs an interactive crop-box gesture and a /CropBox writer.",
    ),
    (
        "pages.resize",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — rescale or re-media-box a set of pages.",
    ),
    (
        "pages.watermark",
        "N — the whole Pages ▸ Stamp group is unbuilt, so the GROUP is absent too rather \
         than present and empty.",
    ),
    (
        "pages.header_footer",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — as `pages.watermark`, in the same absent group.",
    ),
    (
        "pages.bates",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — Bates numbering, in the same absent group. See DEFECTS.md §2.",
    ),
    // -- Edit -- `RIBBON_IA.md` §5.4 ----------------------------------------
    (
        "edit.insert_shape",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — real page shapes, as distinct from the markup shapes on the Markup tab.",
    ),
    (
        "edit.align",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — the whole Edit ▸ Arrange group is unbuilt, so the GROUP is absent too.",
    ),
    // ui-text-exempt: developer note about an ABSENT command; never rendered.
    ("edit.distribute", "N — as `edit.align`, same absent group."),
    (
        "edit.bring_forward",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — needs a content-stream reordering primitive that does not exist.",
    ),
    (
        "edit.send_backward",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — as `edit.bring_forward`, in the other direction.",
    ),
    (
        "edit.group",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — object grouping has no representation in the object model yet.",
    ),
    // ui-text-exempt: developer note about an ABSENT command; never rendered.
    ("edit.ungroup", "N — as `edit.group`."),
    (
        "edit.flip_horizontal",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — as `edit.align`, same absent group.",
    ),
    (
        "edit.flip_vertical",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — as `edit.align`, same absent group.",
    ),
    (
        "edit.cut",
        "N — there is no object clipboard. The two text-copy commands in File ▸ Export are a \
         different mechanism and do not imply one. (They were in Edit ▸ Clipboard until \
         2026-08-14; that group is deleted, so this row no longer has a band waiting for it.)",
    ),
    // ui-text-exempt: developer note about an ABSENT command; never rendered.
    ("edit.copy", "N — as `edit.cut`, the object clipboard."),
    // ui-text-exempt: developer note about an ABSENT command; never rendered.
    ("edit.paste", "N — as `edit.cut`, the object clipboard."),
    (
        "edit.paste_in_place",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — as `edit.cut`, and needs a notion of the source's page coordinates.",
    ),
    (
        "edit.sanitise",
        "N — strip metadata, scripts and hidden content. Distinct from redaction, which \
         removes what a mark covers.",
    ),
    // -- Markup -- `RIBBON_IA.md` §5.5 --------------------------------------
    (
        "markup.line",
        "N — the shipped build has four markup kinds and none is a plain line; the existing \
         `Arrow line` is the arrow. See markup.rs.",
    ),
    // ★ `markup.polyline`, `markup.polygon` and `markup.ink` were here and are
    // now REGISTERED — Phase 6, 2026-08-14, on the two gestures that were their
    // only blocker. Their shared reason read:
    //
    //   "N — not drag-shaped: deferred in the canvas alongside Ink and Polygon,
    //    all three needing a multi-click or freehand gesture the two-point band
    //    cannot express."
    //
    // Every word of which was true, and all of it was about a **gesture** rather
    // than about the engine: `MarkupSpec::PolyLine`, `Polygon` and `Ink` have
    // been in `pdfce-core` since Pass 6.1. `canvas::markup::vertex` built the
    // multi-click gesture — with two endings, on the operator's own 2026-08-14
    // ruling for `measure.finish` — and `canvas::markup::ink` built the freehand
    // one. `markup.finish` is registered with them and was never in this list,
    // because the problem it solves did not exist until the tools did.
    //
    // Removed rather than annotated, because this list's contract is that
    // everything in it is absent and a "planned" row for a shipped command is the
    // drift the list exists to prevent — the same treatment the three text-markup
    // rows below got earlier the same day. The removal is recorded in
    // `manifest::markup`'s header instead, where a reader is looking at the band
    // that gained them.
    (
        "markup.cloud",
        "N — revision clouds, and now the ONLY markup kind still absent for an ENGINE reason \
         rather than a gesture one. The one this audience will miss first: AEC table stakes.",
    ),
    // ★ `markup.underline`, `markup.strikeout` and `markup.squiggly` were here
    // and are now REGISTERED — Phase 6, 2026-08-14, on the text-selection
    // gesture that was their only blocker. Their entries are removed rather
    // than annotated, because this list's contract is that everything in it is
    // absent, and a "planned" row for a shipped command is the drift the list
    // exists to prevent. The removal is recorded in `manifest::markup`'s header
    // instead, where a reader is looking at the band that gained them.
    (
        "markup.callout",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — a note with a leader line to what it refers to.",
    ),
    (
        "markup.line_width",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — Style sets the NEXT markup's properties and only colour has a control today.",
    ),
    // ui-text-exempt: developer note about an ABSENT command; never rendered.
    ("markup.fill", "N — as `markup.line_width`."),
    // ui-text-exempt: developer note about an ABSENT command; never rendered.
    ("markup.opacity", "N — as `markup.line_width`."),
    (
        "markup.clear_page",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — remove every markup on this page in one action.",
    ),
    (
        "markup.clear_all",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — remove every markup in the document in one action.",
    ),
    // -- Measure -- `RIBBON_IA.md` §5.6 -------------------------------------
    (
        "measure.aligned",
        "partial G — the constraint exists inside the linear tool, but there is no separate \
         tool to arm, and a button that arms nothing is the placeholder P3 forbids.",
    ),
    (
        "measure.angular",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — angular dimensions. One of the two conspicuous absences for takeoff work.",
    ),
    // ★ `measure.two_line` was here and is now REGISTERED — Phase 7,
    // 2026-08-14. Its entry is removed rather than annotated, because this
    // list's contract is that everything in it is absent, and a "planned"
    // row for a shipped command is the drift the list exists to prevent.
    //
    // Worth recording where it went, because this entry had already been
    // wrong once: it read *"the pick gesture has no caller"*, which was false
    // in five documents at once — the old shell calls `pick_line_in_page` at
    // `main.rs:23564` and pdfce's own ledger marks the row `gui [x]`. The
    // caller that was missing was ours, and it now exists:
    // `crate::canvas::measure` hosts the pick and `TwoLinePick` came across
    // with it. See `SALVAGE.md`'s correction note for the full account.
    (
        "measure.calibrate",
        "partial G — calibrate from a known length. The least certain judgement in this \
         list: it may already be reachable through the scale entry, in which case this \
         moves into Measure ▸ Scale. See measure.rs.",
    ),
    (
        "measure.distance",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — the whole Measure ▸ Quantity group is unbuilt, so the GROUP is absent too.",
    ),
    // ui-text-exempt: developer note about an ABSENT command; never rendered.
    ("measure.perimeter", "N — as `measure.distance`."),
    (
        "measure.area",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — as `measure.distance`, and the other conspicuous absence for takeoff work.",
    ),
    // ui-text-exempt: developer note about an ABSENT command; never rendered.
    ("measure.count", "N — as `measure.distance`."),
    (
        "measure.takeoff_schedule",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — the whole Measure ▸ Takeoff group is unbuilt, so the GROUP is absent too.",
    ),
    (
        "measure.takeoff_export",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — as `measure.takeoff_schedule`; a schedule to export must exist first.",
    ),
    // -- Tools -- `RIBBON_IA.md` §5.7 ---------------------------------------
    (
        "tools.batch_print",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — printing a set of files unattended, with one print setup.",
    ),
    (
        "tools.compare",
        "N — document comparison. A large build, and an OPEN QUESTION in RIBBON_IA.md §8 \
         rather than a scheduled item: it is the one absence an AEC reviewer names first.",
    ),
    // `tools.ocr` was here — "N — blocked on an OCR engine decision; see the
    // roadmap." It is registered now as **`file.ocr`** (`super::commands`, token
    // 160) and draws in File ▸ Recognise — not Tools ▸ Recognise, for the
    // reason `super::tools`'s header gives. The entry is removed rather than
    // left with a
    // stale reason: `planned_commands_are_genuinely_absent` asserts in both
    // directions and fails on an entry that has shipped. Recorded as a comment
    // on the `file.recent` and `file.about` precedents above — "this used to be
    // planned and is now built" is the one transition this list exists to make
    // legible.
    //
    // ★ The reason is worth keeping, because the note was wrong about what the
    // blocker was. It said "an OCR engine decision"; the engine had been chosen
    // (`ocrs`, for being the only surveyed candidate that passes pdfce's wasm32
    // gate) and the whole recognition path had shipped in `pdfce-core`. What was
    // actually blocked was **redistributing CC-BY-SA-4.0 model weights from an
    // MIT repository**, which is a licensing question and not a GUI one at all.
    // The operator answered it on 2026-08-14 — "yes ship that model in the mit
    // repo with proper credit" — and the credit mechanism was built first, so
    // `about.hbs`, `crates/pdfce-gui/src/text/about.rs` and
    // `tools/package-portable.py` now fail together if any one of them forgets.
    //
    // The lesson is the general one this list is for: a one-line reason is a
    // claim, and a claim that names the wrong blocker sends the next reader to
    // solve the wrong problem.
    (
        "tools.pdfa_validate",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — PDF/A validation and conversion. See DEFECTS.md §2.",
    ),
    (
        "tools.optimise",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — resample, subset and recompress to shrink a file.",
    ),
    // -- Format (contextual) -- `RIBBON_IA.md` §5.8 -------------------------
    //
    // Twenty-four entries from one section, and they are all one decision:
    // "build order: panel first, tab second. The panel is the harder half
    // and the tab's contents are a subset of it, so building the tab first
    // would mean writing the property editors twice."
    (
        "format.colour",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — markup property editors are built panel-first; see format.rs.",
    ),
    // ui-text-exempt: developer note about an ABSENT command; never rendered.
    ("format.fill", "N — markup property; panel first."),
    // ui-text-exempt: developer note about an ABSENT command; never rendered.
    ("format.line_width", "N — markup property; panel first."),
    // ui-text-exempt: developer note about an ABSENT command; never rendered.
    ("format.line_style", "N — markup property; panel first."),
    // ui-text-exempt: developer note about an ABSENT command; never rendered.
    ("format.opacity", "N — markup property; panel first."),
    (
        "format.arrowheads",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — markup property, arrow and line only; panel first.",
    ),
    (
        "format.note_text",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — the text of a placed note; panel first.",
    ),
    (
        "format.dimension_group",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — which group a placed dimension belongs to; panel first.",
    ),
    (
        "format.dimension_scale",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — a placed dimension's scale, as distinct from the current group's; panel first.",
    ),
    (
        "format.precision",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — a placed dimension's number format; panel first.",
    ),
    (
        "format.units",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — a placed dimension's units; panel first.",
    ),
    (
        "format.standard",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — a placed dimension's drafting standard; panel first.",
    ),
    (
        "format.witness_lines",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — a placed dimension's witness lines; panel first.",
    ),
    (
        "format.size",
        "N — a selected image's size. The panel carries the typed W/H, which is the surface \
         that makes /Rect resize reachable without a drag.",
    ),
    (
        "format.position",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — a selected object's position, typed rather than dragged; panel first.",
    ),
    // ui-text-exempt: developer note about an ABSENT command; never rendered.
    ("format.crop", "N — cropping a placed image; panel first."),
    (
        "format.replace_image",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — swap the image behind a placed image object; panel first.",
    ),
    (
        "format.stroke",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — a vector object's stroke; panel first.",
    ),
    (
        "format.winding_rule",
        "N — a vector object's winding rule. A read-only fact more often than an edit, which \
         is precisely why it belongs in the panel rather than the tab.",
    ),
    (
        "format.node_tools",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — add, remove and convert a vector object's nodes; panel first.",
    ),
    // ui-text-exempt: developer note about an ABSENT command; never rendered.
    ("format.font", "N — a text run's font; panel first."),
    // ui-text-exempt: developer note about an ABSENT command; never rendered.
    ("format.font_size", "N — a text run's size; panel first."),
    (
        "format.spacing",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — a text run's character and line spacing; panel first.",
    ),
    (
        "format.alignment",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "N — a text run's alignment; panel first.",
    ),
    // -- Not from `RIBBON_IA.md` §5: commanded by a context menu ------------
    //
    // One entry, and it is here rather than in a register of its own because
    // the question it answers is the same one every row above answers —
    // *"why is this not on the surface that obviously wants it?"* — and a
    // second list would be a second place to look.
    //
    // `super::menus`' `dock.tab` menu is where a Close belongs, and §6 does
    // not specify one because §6 is about the ribbon. The dock closes a tab
    // today through its own hard-coded button and its own internal intent,
    // which is a dock mechanism rather than a pdfce command.
    (
        "dock.close_panel",
        "N — closing one panel is an `egui_shell::dock` INTENT, not a command: the dock \
         draws its own tabs, owns their secondary click, and exposes no seam for an \
         application menu. Registering an id with no way to reach the dock's intent from \
         `dispatch_token` would be a command that cannot work. See `shell::menus`' header \
         for what would close the gap.",
    ),
];

// ===========================================================================
// DIRECTED
// ===========================================================================

/// **Commands emitted despite not carrying a `G` mark, and the instruction
/// that put them there.**
///
/// `(id, why)`. Eight entries, and they exist as a list rather than as
/// prose because otherwise this manifest would look like it applied P3
/// everywhere except in two groups, for no stated reason.
///
/// Two of them — the render quality and settle knobs — are `partial G`:
/// the *value* exists as a compiled-in constant today and what is new is
/// the control that exposes it. The rest were named individually, with
/// their value sets and their defaults, when this shell was commissioned,
/// which is a stronger statement of intent than a status mark in a table:
/// a specification detailed enough to say *"App initiative: Never · Ask ·
/// Allowed, default **Never**"* is describing something decided, not
/// something wished for.
///
/// The honest reading of the tension: P3 exists so an operator is never
/// shown a control that does nothing. These eight are settings rather than
/// actions, every one of them has a specified default, and a setting
/// showing its default is not a stub. That is the argument. It is written
/// down here so that if it turns out to be wrong, the fix is deleting
/// eight rows from one list rather than re-deriving which entries were
/// deliberate.
pub const DIRECTED: &[(&str, &str)] = &[
    (
        "view.render_strategy",
        "Whole page · Tiled progressive. Named explicitly when the View ▸ Render group was \
         commissioned; whole-page is the default because it measured better in use.",
    ),
    (
        "view.render_quality",
        "partial G — the raster-scale multiplier is a compiled-in constant today. What is new \
         is the knob, not the value.",
    ),
    (
        "view.render_settle",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "partial G — `ZOOM_SETTLE` is a compiled-in constant today. As `view.render_quality`.",
    ),
    (
        "view.render_thin_lines",
        "Named explicitly when the View ▸ Render group was commissioned. RIBBON_IA.md §5.2 \
         lists it under both Render and Display; it is kept here, once.",
    ),
    (
        "view.render_antialias",
        // ui-text-exempt: developer note about an ABSENT command; never rendered.
        "Named explicitly when the View ▸ Render group was commissioned (text / vector).",
    ),
    (
        "view.floating_panels",
        "Off · Allowed, default Allowed. Operator decision 2026-08-13, retiring \"nothing \
         // ui-text-exempt: developer note about an ABSENT command; never rendered.
         floats over the canvas\" as an absolute. Off restores today's behaviour exactly.",
    ),
    (
        "view.app_initiative",
        "Never · Ask · Allowed, default NEVER. The half that carries the original complaint; \
         its default preserves the shipped behaviour as a choice rather than a law.",
    ),
    (
        "format.delete",
        "Not status-marked: RIBBON_IA.md §5.8 lists Delete in every selection type's row \
         without a mark. Modeless select-and-delete works today — it is what the removal of \
         the `Editing on` toggle relies on — so the command is real.",
    ),
];

#[cfg(test)]
mod tests {
    use super::*;
    use egui_shell::manifest::{Group, Item, Tab};
    use std::collections::BTreeSet;

    /// Every command id the manifest emits, in document order.
    fn emitted() -> Vec<String> {
        built_in()
            .command_references()
            .into_iter()
            .map(|(_, id)| id)
            .collect()
    }

    /// The shape of the ribbon, pinned.
    ///
    /// Not a change-detector for its own sake: these numbers are quoted in
    /// prose, in five module headers, as the description of the layout
    /// `RIBBON_IA.md` §5 specifies. A count that drifts silently makes
    /// every one of them wrong, and the failure message says which way it
    /// moved.
    ///
    /// **Failing here means editing prose, not just the literal.** The
    /// group count went 31 → 32 with this test passing on the new number
    /// and five headers still saying "thirty-one", because pinning a value
    /// does not pin the sentences that repeat it. The sites are:
    ///
    /// - this module's header, and [`group`]'s;
    /// - [`crate::shell`]'s submodule table;
    /// - [`crate::shell::ron`]'s header (groups **and** key bindings);
    /// - [`crate::text::ribbon`]'s header.
    ///
    /// ★ It went back **32 → 31** on 2026-08-14, and the same five sites were
    /// edited with it. The cause was a *deletion*, which is the direction that
    /// makes this test most valuable: the two text-copy commands moved to
    /// File ▸ Export, Edit ▸ Clipboard was left with no members, and an empty
    /// group is a captioned band offering nothing — the placeholder P3
    /// forbids. Deleting it is what the rule requires; editing the number in
    /// six places is what this test makes unavoidable.
    ///
    /// The keymap is counted here for the same reason: `ron`'s header
    /// argues that the format can express *the real ribbon* and then lists
    /// its parts, so a binding added without that list moving turns the
    /// argument into a claim about a smaller shell than the one shipped.
    #[test]
    fn the_ribbon_has_the_documented_shape() {
        let shell = built_in();
        assert_eq!(shell.tabs().len(), 7, "seven ordinary tabs");
        assert_eq!(shell.contextual_tabs().len(), 1, "one contextual tab");
        assert_eq!(
            shell.all_tabs().flat_map(Tab::groups).count(),
            32,
            "thirty-two groups"
        );
        assert_eq!(shell.modes().len(), 3, "three modes");
        assert_eq!(
            shell.keymap.as_ref().expect("a keymap").len(),
            20,
            "twenty key bindings"
        );
    }

    /// The tabs are the seven of `RIBBON_IA.md` §4, in its order.
    #[test]
    fn the_tabs_are_the_seven_in_specification_order() {
        let shell = built_in();
        let ids: Vec<&str> = shell.tabs().iter().map(|t| t.id.as_str()).collect();
        assert_eq!(
            ids,
            [
                "file", "view", "pages", "edit", "markup", "measure", "tools"
            ]
        );
        assert_eq!(shell.contextual_tabs()[0].id, "format");
    }

    /// **Every command id is prefixed with the tab that owns it.**
    ///
    /// The convention that makes P1 legible in the id itself. Two
    /// documented exceptions, and they are named here rather than
    /// hand-waved so that a third one has to be added deliberately.
    #[test]
    fn every_command_id_names_its_owning_tab() {
        for tab in built_in().all_tabs() {
            for group in tab.groups() {
                for id in group.items().iter().filter_map(Item::command_id) {
                    assert!(
                        id.starts_with(&format!("{}.", tab.id)),
                        "`{id}` is on tab `{}` but is not prefixed with it",
                        tab.id
                    );
                }
            }
        }
    }

    /// The two ids that live on no tab do live somewhere.
    #[test]
    fn the_two_tabless_commands_are_the_documented_ones() {
        let shell = built_in();
        let on_a_tab: BTreeSet<&str> = shell
            .all_tabs()
            .flat_map(Tab::groups)
            .flat_map(Group::items)
            .filter_map(Item::command_id)
            .collect();
        let qat = shell.qat.as_ref().expect("the QAT exists");
        for id in ["edit.undo", "edit.redo"] {
            assert!(!on_a_tab.contains(id));
            assert!(qat.ids().iter().any(|q| q == id));
        }
    }

    /// **The three View ▸ Render entries with no `G` mark, and the two new
    /// Window settings, are actually present.**
    ///
    /// [`DIRECTED`] is a claim about this manifest. If an entry were listed
    /// there and then not emitted, the list would be documenting a
    /// deviation that had been quietly reverted — which is worse than
    /// either state on its own, because the note would still be there
    /// explaining a decision nobody could see.
    #[test]
    fn every_directed_entry_is_emitted() {
        let emitted: BTreeSet<String> = emitted().into_iter().collect();
        for (id, why) in DIRECTED {
            assert!(
                emitted.contains(*id),
                "`{id}` is listed as a directed inclusion ({why}) but is not in the manifest"
            );
        }
    }

    /// `DIRECTED` and `PLANNED` are disjoint.
    ///
    /// An id in both would be claiming to be emitted and absent at once.
    #[test]
    fn directed_and_planned_do_not_overlap() {
        let planned: BTreeSet<&str> = PLANNED.iter().map(|(id, _)| *id).collect();
        for (id, _) in DIRECTED {
            assert!(
                !planned.contains(*id),
                "`{id}` is listed as both directed and planned"
            );
        }
    }

    /// The View ▸ Window group carries the two new settings, in the
    /// specified order, between the two existing window commands and the
    /// layout reset.
    ///
    /// Order is checked rather than mere presence because the group reads
    /// as a progression — what the window shows, then what may float in
    /// it, then how to put it all back — and the reset belongs last for
    /// the same reason a reset button always does.
    #[test]
    fn the_window_group_holds_the_two_new_settings_in_order() {
        let shell = built_in();
        let view = shell
            .tabs()
            .iter()
            .find(|t| t.id == "view")
            .expect("the View tab");
        let window = view
            .groups()
            .iter()
            .find(|g| g.id == "window")
            .expect("the Window group");
        let ids: Vec<&str> = window.items().iter().filter_map(Item::command_id).collect();
        assert_eq!(
            ids,
            [
                "view.read_mode",
                "view.fullscreen",
                "view.floating_panels",
                "view.app_initiative",
                "view.reset_layout",
            ]
        );
    }

    /// The Markup ▸ Style band holds the colour swatch as a `Custom` item.
    ///
    /// Asserted because the alternative — modelling a colour picker as a
    /// `Command` — is the easy mistake, and it is the one that would push
    /// a `ColourSwatch` variant into `egui-shell` the first time the
    /// renderer needed to tell the two apart.
    #[test]
    fn the_markup_style_band_is_a_custom_item_not_a_command() {
        let shell = built_in();
        let style = shell
            .tabs()
            .iter()
            .find(|t| t.id == "markup")
            .expect("the Markup tab")
            .groups()
            .iter()
            .find(|g| g.id == "style")
            .expect("the Style group")
            .items()
            .to_vec();
        assert_eq!(style, vec![Item::custom("colour_swatch")]);
    }

    /// The keymap binds every chord to a command the manifest knows, and
    /// binds the four chords that are new in this layout.
    #[test]
    fn the_keymap_binds_the_new_chords() {
        let shell = built_in();
        let keymap = shell.keymap.as_ref().expect("a keymap");
        assert_eq!(keymap.get("Ctrl+H"), Some("view.read_mode"));
        assert_eq!(keymap.get("F11"), Some("view.fullscreen"));
        assert_eq!(keymap.get("Ctrl+1"), Some("mode.read"));
        assert_eq!(keymap.get("Ctrl+2"), Some("mode.review"));
        assert_eq!(keymap.get("Ctrl+3"), Some("mode.edit"));
        // Two chords, one command: redo is reachable both ways, exactly as
        // the shipped shortcut window already promises.
        assert_eq!(keymap.get("Ctrl+Y"), Some("edit.redo"));
        assert_eq!(keymap.get("Ctrl+Shift+Z"), Some("edit.redo"));
    }

    /// No command is emitted twice anywhere, including across the QAT and
    /// the keymap.
    ///
    /// `Shell::validate` enforces one-command-one-*tab* and separately
    /// forbids the QAT listing one id twice. This is the remaining case:
    /// a command that appears once on a tab, once on the QAT and twice in
    /// the keymap is legal and intended (redo), so what is checked is the
    /// narrower thing — no id appears twice within the tab set.
    #[test]
    fn no_command_appears_twice_on_the_tabs() {
        let shell = built_in();
        let mut on_tabs: Vec<&str> = shell
            .all_tabs()
            .flat_map(Tab::groups)
            .flat_map(Group::items)
            .filter_map(Item::command_id)
            .collect();
        let total = on_tabs.len();
        on_tabs.sort_unstable();
        on_tabs.dedup();
        assert_eq!(on_tabs.len(), total, "a command is on the tabs twice");
    }
}
