//! # shell::commands — every verb pdfce can perform
//!
//! [`register`] populates an `egui_shell::CommandRegistry` with the
//! **eighty-eight** commands this build has, which fall into three groups:
//!
//! | group | count | how the operator reaches it |
//! |---|---|---|
//! | on a tab, the QAT or the keymap | 86 | a control [`super::manifest::built_in`] names by id |
//! | drawn by a **custom item** | 1 | `file.recent` — see [`super::manifest::CUSTOM_BACKED`] |
//! | drawn on the **status bar** | 1 | `edit.find` — `RIBBON_IA.md` §6 |
//!
//! (This header said *eighty-one* and *79* until 2026-08-14, while
//! [`tests::registration_succeeds_and_registers_every_command`] asserted 88
//! and passed. Prose drifting from a number a test pins is a defect this
//! project has now had three times; the test is the fact, and the table has
//! been corrected to it rather than the other way round.)
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
//! Six conditions are used, and the whole vocabulary is listed here
//! because every one of them is a promise the application has to keep:
//!
//! | Condition | True when | Used by |
//! |---|---|---|
//! | *(none)* | always | commands with no precondition: Open, Settings, the batch tools, the window and render settings |
//! | `doc.open` | a document is open | document-level commands — close, save a copy, properties, print |
//! | `doc.pages` | …and it has at least one page | everything that acts on a page |
//! | `undo.available` / `redo.available` | the corresponding stack is non-empty | Undo, Redo |
//! | `selection.any` | something is selected | the contextual Format tab and its Delete |
//! | `selection.bounds` | …and it still resolves to a box on the page shown | Zoom to selection |
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
//! # Icons
//!
//! A `String` key resolved by the application's icon set, not a texture:
//! icon rendering is a licensing and rasterization decision that belongs
//! to pdfce, and the shell only needs to know that a control has an icon
//! and which one. The keys are the ones the salvaged icon set already
//! uses (`open`, `rotate-ccw`, `shape-rect`, …).
//!
//! Two conventions worth stating, because both look like mistakes:
//!
//! - **Keys are shared.** `copy` is on both text-copy commands, `redact`
//!   on both redaction commands, `measure` on both dimension tools,
//!   `export` on both export verbs, `delete` on both delete verbs, `list`
//!   on both "manage a list of things" dialogs. A family of related
//!   commands sharing a glyph is how a ribbon reads as grouped; uniqueness
//!   is a property of ids, not of icons.
//! - **`None` is a real answer.** A command with no key renders as text.
//!   Every icon is a drawing somebody has to make, and inventing a key for
//!   an icon that does not exist would produce a missing-glyph box at run
//!   time — a placeholder, arriving through the back door.
//!
//! ## Coverage, as of 2026-08-14: 77 of 88 named, 11 refused
//!
//! Before that date the split was 47 named and 41 not, with **no rule
//! behind which was which** — a band drew glyphs and bare words side by
//! side, and the ribbon read as half-finished because it was. Thirty
//! commands gained a key in that pass (25 new glyphs plus two reuses of
//! `chevron-up`/`chevron-down` for page reordering, which is the meaning
//! `crate::icons::Icon::ChevronUp`'s doc comment already gave them).
//!
//! The other eleven are **recorded refusals**, each stated in full at its
//! own registration below and summarised in `crate::icons::assets` §5
//! deviation #8. In one line each:
//!
//! | Command(s) | Why no glyph |
//! |---|---|
//! | `view.zoom_actual` | the icon ui-spec §3.2 argues against it by name |
//! | the five `view.render_*` | their labels are the parameter's whole content; no conventional glyph exists for any of them |
//! | `view.app_initiative` | any honest drawing pictures what its default forbids |
//! | `file.recent` | reusing `open` would draw one band control twice |
//! | `mode.read`/`review`/`edit` | the mode selector renders text segments and has no icon path |
//!
//! **A band control's icon does not replace its label.**
//! `egui_shell::ribbon::band::command_button` is called with
//! `shows_label: true` from the band, always; only the QAT goes icon-only,
//! and only `file.open`, `file.save_copy`, `edit.undo` and `edit.redo` are
//! on it. Two of the notes retired in this pass had reasoned as though the
//! choice were "a glyph *or* a findable word", and it never was. That
//! misreading is worth keeping written down: it is what kept three Display
//! toggles and the Pages panel bare for longer than any decision did.

use crate::text::commands as t;
use crate::text::commands::CommandText;
use egui_shell::{Command, CommandRegistry, HandlerToken};

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
/// If two commands claim one id. That is a programming error in this file
/// and not a condition any input can produce, so it fails loudly at
/// start-up rather than being swallowed: the registry refuses a duplicate
/// precisely so that behaviour cannot come to depend on the order of
/// start-up code, and catching the error here to ignore it would give back
/// exactly the defect the refusal prevents.
pub fn register(reg: &mut CommandRegistry) {
    reg.register_all(all())
        // ui-text-exempt: a panic message, read by whoever is looking at
        // the stack trace. Never rendered to an operator — the process
        // does not reach a window if this fires.
        .expect("two shell commands claim the same id");
}

/// One command, with its label and tooltip taken from the catalog.
///
/// The two are always fetched together, from one catalog entry, so a
/// command cannot end up with one command's label and another's tooltip —
/// which is not a hypothetical: the salvage source's two adjacent Content
/// buttons both read `Aa`, and only their tooltips distinguished them.
fn command(id: &str, text: CommandText, handler: u64) -> Command {
    Command::new(id, text.label, HandlerToken::new(handler)).with_tooltip(text.tooltip)
}

/// Every command, in manifest order.
///
/// One flat list rather than a function per tab: the registry is a flat
/// namespace, the ordering here mirrors the ribbon so the two can be read
/// side by side, and a per-tab split would put the handler-token blocks in
/// eight files where a collision between two of them is invisible.
fn all() -> Vec<Command> {
    vec![
        // ===================================================================
        // FILE — tokens 100-199
        // ===================================================================
        command("file.open", t::file_open(), 100).with_icon("open"),
        command("file.close", t::file_close(), 101)
            .with_icon("close")
            .enabled_when("doc.open"),
        // ★ Recent — the one command whose ribbon control is NOT a button.
        //
        // It is drawn by the `recent_files` custom item in File ▸ File, which
        // is what asks *which* of the ten documents; this command is the verb
        // that opens the answer. `super::manifest::CUSTOM_BACKED` records the
        // arrangement and `super::tests::no_registered_command_is_orphaned`
        // consults that register, so a command with no route at all still
        // fails while this one passes for a stated reason.
        //
        // **No enable predicate**, and that is deliberate rather than an
        // omission. The vocabulary of conditions is five names published once
        // per frame, and "the operator has opened something before" would be a
        // sixth that only one control reads — while that control is a menu
        // that has to decide its own greying anyway, from a list it already
        // holds. So the availability rule lives with the control
        // (`app::recent::menu`, which greys the button on an empty list and
        // explains it on hover, exactly as P3 requires) and this command stays
        // available to a keymap or a customized quick-access toolbar, where it
        // opens the newest document it can still see.
        //
        // No icon: `open` belongs to `file.open` and reusing it would make two
        // adjacent controls in one band look like one control drawn twice. A
        // command with no key renders as text, which is a real answer — see
        // the header — and the right one for a menu button whose label is a
        // word.
        command(FILE_RECENT, t::file_recent(), 102),
        command("file.save_copy", t::file_save_copy(), 110)
            .with_icon("save")
            .enabled_when("doc.open"),
        // Both export verbs share `export`, and that is the header's
        // shared-key convention rather than an oversight: the glyph is the
        // download twin of `insert-pages`' upload art, reserved for exactly
        // this by the icon ui-spec §3.1, and what it says — "out of this
        // document, into a file" — is equally and completely true of both.
        // What differs is the format, which is a word only the label can say.
        command("file.export_dxf", t::file_export_dxf(), 120)
            .with_icon("export")
            .enabled_when("doc.pages"),
        command("file.export_form_data", t::file_export_form_data(), 121)
            .with_icon("export")
            .enabled_when("doc.open"),
        // ★ Print had no icon because the salvage source drew it with the
        // *stamp* glyph, and that was a mis-assignment rather than a
        // convention to carry — `stamp` means "a mark applied with a stamp"
        // (icon ui-spec §3.4) and is shared with the reserved Bates glyph.
        //
        // Declining the wrong glyph was right; it was never a reason to have
        // none. `print` is the printer art the ui-spec §8.12 reserved, and it
        // collides with nothing.
        command("file.print", t::file_print(), 130)
            .with_icon("print")
            .enabled_when("doc.open"),
        command("file.properties", t::file_properties(), 140)
            .with_icon("properties")
            .enabled_when("doc.open"),
        command("file.fonts", t::file_fonts(), 141)
            .with_icon("fonts")
            .enabled_when("doc.open"),
        // Settings and the shortcut list are always available: they are
        // about pdfce, not about a document.
        command("file.settings", t::file_settings(), 150).with_icon("settings"),
        command("file.shortcuts", t::file_shortcuts(), 151).with_icon("keyboard"),
        // ===================================================================
        // VIEW — tokens 200-299
        // ===================================================================
        // ★ **Page display — a radio, not four toggles.**
        //
        // Exactly one is active at a time, and which one is published as a
        // `selected:` condition by `PdfceApp::conditions` so the active
        // position renders pressed (`egui_shell::ribbon::selected_condition`).
        // Four independent toggles would admit states that mean nothing —
        // "facing, but also single" — and would leave the ribbon reconstructing
        // which of them is on.
        //
        // All four are gated on `doc.pages` rather than `doc.open`: an
        // arrangement of pages is meaningless without pages, and a document
        // with `/Count 0` is legal.
        //
        // The tokens are contiguous (200-203) because they are one control.
        //
        // ★ **All four carry an icon or none would.** These are the positions
        // of one radio, and a radio whose positions are three glyphs and one
        // bare word does not read as a radio — the eye groups by shape before
        // it reads. The four glyphs are drawn as a set for the same reason:
        // left-to-right says how many pages are across, a cut bottom edge
        // says whether they keep coming, and those two axes are the whole
        // control (see `crate::icons::Icon::PageSingle`).
        command("view.page_single", t::view_page_single(), 200)
            .with_icon("page-single")
            .enabled_when("doc.pages"),
        command("view.page_continuous", t::view_page_continuous(), 201)
            .with_icon("page-continuous")
            .enabled_when("doc.pages"),
        command("view.page_facing", t::view_page_facing(), 202)
            .with_icon("page-facing")
            .enabled_when("doc.pages"),
        command(
            "view.page_facing_continuous",
            t::view_page_facing_continuous(),
            203,
        )
        .with_icon("page-facing-continuous")
        .enabled_when("doc.pages"),
        // The Render group is settings, not actions. They are available
        // with no document open because they are what the *next* document
        // will be drawn with, and a setting you can only change while
        // something is open is a setting you cannot prepare.
        //
        // ★ **No icons on any of the five, and that is a decision about the
        // whole group rather than five separate omissions.**
        //
        // Their labels ARE the control's content: "Strategy", "Raster scale",
        // "Settle delay", "Thin lines", "Antialias" each name a parameter
        // whose value is the thing an operator came here to read. None of the
        // five has an industry-conventional glyph — there is no picture of a
        // settle delay that anybody has already learned — so every candidate
        // would be art invented here, decoding to nothing the word beside it
        // did not already say.
        //
        // This is the reasoning the icon ui-spec §3.2 applied to Actual size
        // ("a numeral read at a glance is clearer than any glyph substitute
        // could be… both add a decode step a bare percentage does not need"),
        // applied to a group instead of a control. The 2026-08-14 icon pass
        // considered each of the five and refused each: an invented glyph on
        // a settings knob is decoration, and decoration on a ribbon costs the
        // legibility of the glyphs that mean something.
        //
        // It also leaves Render reading as the settings group it is — a band
        // of labelled values among bands of pictured verbs — which is a
        // property worth keeping rather than an inconsistency to fix.
        command("view.render_strategy", t::view_render_strategy(), 210),
        command("view.render_quality", t::view_render_quality(), 211),
        command("view.render_settle", t::view_render_settle(), 212),
        command("view.render_thin_lines", t::view_render_thin_lines(), 213),
        command("view.render_antialias", t::view_render_antialias(), 214),
        // ★ **No icon, on the icon ui-spec's own explicit instruction.**
        //
        // §3.2 is a whole section devoted to this one control: "Recommend
        // leaving `zoom_100_button()` as plain text ('100%'), not iconified…
        // a magnifier-with-'1' badge or a '1:1' pictograph both add a decode
        // step a bare percentage does not need… Flagged explicitly so the
        // engineer does not feel obligated to force an icon here against the
        // better outcome."
        //
        // The 2026-08-14 pass gave the other four Zoom entries glyphs and
        // deliberately left this one alone. A spec that anticipated being
        // overruled by a completeness drive, and argued against it in
        // advance, is the strongest kind of recorded decision there is.
        command("view.zoom_actual", t::view_zoom_actual(), 220).enabled_when("doc.pages"),
        // Zoom to selection is gated on `selection.bounds`, not on
        // `selection.any` — the two differ, and the difference is the
        // command's whole failure mode. A selection can exist and resolve to
        // no box (it names an object on another page, or one an edit has
        // renumbered), and the honest answer there is a greyed control, not
        // a press that silently frames nothing.
        //
        // Its glyph is a diagonal PAIR of corner brackets closing on an
        // object, not the four `fit-page` uses. Four here would differ from
        // Fit page — two buttons away in this same group — only by a small
        // rect in the middle, which is the same-group collision the icon
        // ui-spec §2.1 calls its one ❌-grade risk.
        command("view.zoom_selection", t::view_zoom_selection(), 223)
            .with_icon("zoom-selection")
            .enabled_when("selection.bounds"),
        // Arming, not acting: this changes what the next drag means. It
        // renders pressed while armed through the `selected:` convention,
        // and the canvas disarms it on release.
        //
        // `zoom-region` is the fourth member of the icon ui-spec §3.1
        // magnifier family, whose grammar is that the lens names the member:
        // empty is Find, a minus is zoom out, a plus is zoom in, a box is
        // "magnify the box you drag".
        command("view.zoom_region", t::view_zoom_region(), 224)
            .with_icon("zoom-region")
            .enabled_when("doc.pages"),
        command("view.tool_hand", t::view_tool_hand(), 225)
            .with_icon("hand")
            .enabled_when("doc.pages"),
        command("view.zoom_fit_page", t::view_zoom_fit_page(), 221)
            .with_icon("fit-page")
            .enabled_when("doc.pages"),
        command("view.zoom_fit_width", t::view_zoom_fit_width(), 222)
            .with_icon("fit-width")
            .enabled_when("doc.pages"),
        command("view.show_annotations", t::view_show_annotations(), 230)
            .with_icon("comment")
            .enabled_when("doc.pages"),
        command("view.show_points", t::view_show_points(), 231)
            .with_icon("show-points")
            .enabled_when("doc.pages"),
        // ★ **The three chrome toggles**, and all three render pressed while
        // they are on, through the `selected:` convention `view.tool_hand`
        // documents and the page-display radio uses.
        //
        // They *can*, where the hand tool and the region zoom still cannot,
        // and the difference is worth naming because it is the reason the
        // state lives where it does: a `selected:` condition is published from
        // `PdfceApp::conditions`, which is handed `&self` and **no
        // `egui::Context`** — so a toggle whose state lives in `egui::Memory`
        // has no route to the ribbon. These three live on
        // `crate::viewer::ViewState`, which `conditions` can read, so no
        // second mechanism was needed.
        //
        // ★ **All three now carry a glyph, and the note that used to stand
        // here is retired rather than reworded.** It read: "No icons: there
        // is no ruler, grid or guide key in `crate::icons::catalog`, and
        // naming one would draw the catalogue's deliberate slashed mark for
        // an unknown key. A command with no icon renders as its label, which
        // is the right answer here… the control's name is a word, and the
        // word is what makes it findable."
        //
        // The first half was a true statement about the catalogue and is now
        // false: `rulers`, `grid` and `guides` exist, authored 2026-08-14.
        //
        // The second half was a **misreading of the ribbon**, and it is worth
        // saying so plainly because it is the reason three controls stayed
        // bare longer than they had to. In a band, `egui_shell`'s
        // `band::command_button` is called with `shows_label: true` always —
        // an icon there is drawn *beside* the label, never instead of it.
        // Only the QAT goes icon-only, and only these four ids are on it:
        // `file.open`, `file.save_copy`, `edit.undo`, `edit.redo`. So an icon
        // on a band control costs the word nothing; the choice was never
        // "glyph or findable name", and reasoning as though it were produced
        // a group of three bare words in a row of pictures.
        //
        // `doc.pages`, like the rest of the Display group: a ruler with no
        // page to measure and a grid with no paper to rule are both chrome
        // about nothing.
        //
        // The tokens are contiguous (232-234) because they are one row of the
        // specification.
        command("view.rulers", t::view_rulers(), 232)
            .with_icon("rulers")
            .enabled_when("doc.pages"),
        command("view.grid", t::view_grid(), 233)
            .with_icon("grid")
            .enabled_when("doc.pages"),
        command("view.guides", t::view_guides(), 234)
            .with_icon("guides")
            .enabled_when("doc.pages"),
        // The sidebar is the application's own furniture and toggles with
        // or without a document; the panels inside it need one to describe.
        command("view.sidebar", t::view_sidebar(), 240).with_icon("sidebar"),
        command("view.panel_bookmarks", t::view_panel_bookmarks(), 241)
            .with_icon("bookmarks")
            .enabled_when("doc.open"),
        // ★ **This carried a recorded "no icon" decision, and the decision
        // has expired.** It read: "There is no `document` (or `pages`) key in
        // `crate::icons::catalog`, and naming one would draw the catalogue's
        // deliberate visible slashed mark for an unknown key on a control an
        // operator uses constantly. A command with no icon renders as its
        // label, which is a real answer and the right one here — the panel's
        // name is a word, and the word is what makes it findable."
        //
        // Both halves have been overtaken:
        //
        // * The premise is gone. `pages` was authored on 2026-08-14 (three
        //   sheets, front one whole and two behind showing only the edges
        //   that clear it — `crate::icons::Icon::Pages` records what it was
        //   drawn to stay distinguishable from). Naming it draws a glyph.
        // * The fallback argument was a misreading of the ribbon, the same
        //   one the Display group's note made: a band control draws its icon
        //   BESIDE its label, never instead of it. Nothing about the word was
        //   ever at stake.
        //
        // Left standing, that comment would have read as a live reason to
        // keep a bare button in a row of glyphs. A decision whose premise has
        // been removed is not a decision any more.
        //
        // `doc.open`, not `doc.pages`, unlike every other entry in this group:
        // the Pages panel's own body handles a `/Count 0` document and says so,
        // which is more useful than a greyed toggle that cannot explain why a
        // legal PDF has no pages.
        command("view.panel_pages", t::view_panel_pages(), 245)
            .with_icon("pages")
            .enabled_when("doc.open"),
        command("view.panel_layers", t::view_panel_layers(), 242)
            .with_icon("layers")
            .enabled_when("doc.open"),
        command("view.panel_signatures", t::view_panel_signatures(), 243)
            .with_icon("signatures")
            .enabled_when("doc.open"),
        command("view.panel_objects", t::view_panel_objects(), 244)
            .with_icon("edit-objects")
            .enabled_when("doc.pages"),
        // ★ Was `edit.form_fill`, token 430, until the operator answered the
        // question `crate::app::modes` had been carrying: Read should fill
        // forms, because that is what Acrobat Reader does in its default
        // view. Read is shown `file` and `view` alone, and P1 gives a
        // command exactly one tab — so the verb moved to a tab Read has,
        // which meant a new id and a token in the `view.` block.
        //
        // It keeps `doc.pages` rather than `doc.open`: an AcroForm's fields
        // carry page-relative rectangles, so a document with no pages has
        // nowhere for a field to be.
        //
        // `forms` is a page carrying two input boxes — not `form-field`,
        // which makes a field and belongs to Edit. That distinction is the
        // same line this command's placement draws: filling is not authoring.
        // It does not contradict the icon ui-spec §8.14's "no dedicated
        // toolbar icon" for form filling either; that ruling is about there
        // being no fill TOOL to arm, and this is a panel toggle.
        command("view.panel_forms", t::view_panel_forms(), 246)
            .with_icon("forms")
            .enabled_when("doc.pages"),
        // Read mode and full screen are the two commands `RIBBON_IA.md` §3
        // named as having "no ribbon control at all" on a tab literally
        // called View. They have controls now, so they have glyphs.
        command("view.read_mode", t::view_read_mode(), 250).with_icon("read-mode"),
        command("view.fullscreen", t::view_fullscreen(), 251).with_icon("fullscreen"),
        command("view.floating_panels", t::view_floating_panels(), 252)
            .with_icon("floating-panels"),
        // ★ **No icon**, and it is the one refusal in this group.
        //
        // The setting is a three-position policy — Never · Ask · Allowed —
        // about whether pdfce may float a surface over the page **on its own
        // initiative**. Every honest drawing of it is a picture of that
        // floating surface, which is to say a picture of what the default
        // (Never) forbids. `crate::icons::catalog`'s header states the rule
        // this runs into: an icon is a claim, and it reaches the operator
        // before the tooltip does.
        //
        // Its neighbour `view.floating_panels` looks like the same kind of
        // thing and is not: that one governs whether the OPERATOR may tear a
        // panel out, so a torn-out panel is a picture of the capability
        // rather than of the prohibition, and it gets a glyph. The two
        // settings being separable is the whole point of there being two, and
        // the icons follow the distinction instead of blurring it.
        command("view.app_initiative", t::view_app_initiative(), 253),
        command("view.reset_layout", t::view_reset_layout(), 254).with_icon("reset-layout"),
        // ===================================================================
        // PAGES — tokens 300-399
        //
        // Every one of these needs a page to act on, so `doc.pages`
        // throughout. They additionally respect the thumbnail rail's
        // selection when there is one, which is a property of the handler
        // rather than of availability: with no selection they act on the
        // current page, which is a defined answer and not a disabled state.
        // ===================================================================
        command("pages.insert_from_file", t::pages_insert_from_file(), 300)
            .with_icon("insert-pages")
            .enabled_when("doc.pages"),
        // `delete` is the waste-bin glyph, shared with `format.delete` under
        // the header's shared-key convention: the verb is the same one and
        // the two are never drawn together, because Format is contextual and
        // one tab's band shows at a time. What differs is the target, which
        // is what the label says.
        command("pages.delete", t::pages_delete(), 310)
            .with_icon("delete")
            .enabled_when("doc.pages"),
        command("pages.extract", t::pages_extract(), 311)
            .with_icon("page-extract")
            .enabled_when("doc.pages"),
        // ★ These two REUSE existing keys rather than gaining art, and the
        // reuse is the catalogue's own documented meaning rather than a
        // near-enough substitution. `crate::icons::Icon::ChevronUp`'s doc
        // comment already reads: *"'Move selection up' in the page rail and
        // the Combine-files list"* — it was authored 2026-08-03 for exactly
        // this verb, because `▲` (U+25B2) was VERIFIED tofu in the shipped
        // font stack.
        //
        // Drawing page-shaped art for reorder would have been the worse
        // answer twice over: two more assets to keep in step with the rest of
        // the Pages tab, and a departure from the up/down chevron pair, which
        // is the reorder convention in every list control an operator has
        // used. The pair sits side by side here with its labels, which is
        // what disambiguates `chevron-down` from its other role as a menu
        // disclosure marker.
        command("pages.move_up", t::pages_move_up(), 312)
            .with_icon("chevron-up")
            .enabled_when("doc.pages"),
        command("pages.move_down", t::pages_move_down(), 313)
            .with_icon("chevron-down")
            .enabled_when("doc.pages"),
        command("pages.split", t::pages_split(), 314)
            .with_icon("split")
            .enabled_when("doc.pages"),
        command("pages.merge_into", t::pages_merge_into(), 315)
            .with_icon("combine")
            .enabled_when("doc.pages"),
        command("pages.rotate_left", t::pages_rotate_left(), 320)
            .with_icon("rotate-ccw")
            .enabled_when("doc.pages"),
        command("pages.rotate_right", t::pages_rotate_right(), 321)
            .with_icon("rotate-cw")
            .enabled_when("doc.pages"),
        // ===================================================================
        // EDIT — tokens 400-499
        // ===================================================================
        command("edit.text", t::edit_text(), 400)
            .with_icon("edit-text")
            .enabled_when("doc.pages"),
        command("edit.add_text", t::edit_add_text(), 401)
            .with_icon("add-text")
            .enabled_when("doc.pages"),
        command("edit.objects", t::edit_objects(), 402)
            .with_icon("edit-objects")
            .enabled_when("doc.pages"),
        // `insert-image` is the picture glyph the icon ui-spec §8.5 reserved
        // for OCR. This command is the earlier and primary claim on it: it
        // places an actual raster on the page, where OCR only reads one.
        command("edit.insert_image", t::edit_insert_image(), 410)
            .with_icon("insert-image")
            .enabled_when("doc.pages"),
        command("edit.copy_page_text", t::edit_copy_page_text(), 420)
            .with_icon("copy")
            .enabled_when("doc.pages"),
        command("edit.copy_document_text", t::edit_copy_document_text(), 421)
            .with_icon("copy")
            .enabled_when("doc.pages"),
        // `edit.form_fill` was here, token 430. It is now `view.panel_forms`
        // — see that registration. Token 430 stays unused rather than being
        // handed to the next Edit command: a token is what a trace prints,
        // and reusing this one would make an old trace of a form fill read
        // as whatever took its number.
        command("edit.form_create_field", t::edit_form_create_field(), 431)
            .with_icon("form-field")
            .enabled_when("doc.pages"),
        // `list` is shared with `measure.manage_groups`, and the family it
        // belongs to is one of ACTION rather than of subject: form fields and
        // dimension groups have nothing to do with each other, but both
        // commands answer a click by opening a list you add to, rename in and
        // remove from — which is the only thing a glyph can honestly promise
        // where "fields" and "dimension groups" are words only a label can
        // say. Different tabs, so never drawn together.
        command("edit.form_manage_fields", t::edit_form_manage_fields(), 432)
            .with_icon("list")
            .enabled_when("doc.pages"),
        // Drawn to the icon ui-spec §8.14's own construction for this exact
        // command: "a form-field rectangle with a small downward chevron
        // pressing onto it (burn-in metaphor)".
        command("edit.form_flatten", t::edit_form_flatten(), 433)
            .with_icon("form-flatten")
            .enabled_when("doc.pages"),
        // ★ Find — registered, bound to Ctrl+F, and on **no tab**.
        //
        // A third documented exception to the "every command is on its owning
        // tab" convention, alongside `edit.undo`/`edit.redo` (QAT only). Its
        // control is the **status bar's Find toggle**: `RIBBON_IA.md` §6 lists
        // the status bar's contents and puts Find first among them, in the
        // section headed "what deliberately does not go on the ribbon". The
        // `edit.` prefix says where it would go if it ever got a tab, which is
        // the same thing undo's and redo's prefixes say.
        //
        // It is not orphaned, and it needs no `CUSTOM_BACKED` exemption: the
        // manifest keymap binds `Ctrl+F` to it, and a keymap entry is a
        // reference site `Shell::command_references()` walks. So
        // `no_registered_command_is_orphaned` sees it, and a rename that lost
        // the binding would fail that test rather than silently producing a
        // command nothing can reach.
        //
        // `doc.pages`, not `doc.open`: there is no page text to search in a
        // document with no pages, and a Find bar over one is a control whose
        // every input is refused — the exact case that predicate exists to
        // separate.
        command("edit.find", t::edit_find(), 450)
            .with_icon("search")
            .enabled_when("doc.pages"),
        command("edit.redact", t::edit_redact(), 440)
            .with_icon("redact")
            .enabled_when("doc.pages"),
        command("edit.redact_apply", t::edit_redact_apply(), 441)
            .with_icon("redact")
            .enabled_when("doc.pages"),
        // Undo and redo live on the QAT alone. Their predicates are the
        // canonical example of "greying is for temporarily unavailable":
        // an empty stack is a state that ends the moment anything happens,
        // and the tooltip is what explains it.
        command("edit.undo", t::edit_undo(), 490)
            .with_icon("undo")
            .enabled_when("undo.available"),
        command("edit.redo", t::edit_redo(), 491)
            .with_icon("redo")
            .enabled_when("redo.available"),
        // ===================================================================
        // MARKUP — tokens 500-599
        // ===================================================================
        command("markup.rectangle", t::markup_rectangle(), 500)
            .with_icon("shape-rect")
            .enabled_when("doc.pages"),
        command("markup.ellipse", t::markup_ellipse(), 501)
            .with_icon("shape-ellipse")
            .enabled_when("doc.pages"),
        command("markup.arrow", t::markup_arrow(), 502)
            .with_icon("shape-arrow")
            .enabled_when("doc.pages"),
        command("markup.highlight", t::markup_highlight(), 510)
            .with_icon("shape-highlight")
            .enabled_when("doc.pages"),
        command("markup.text_box", t::markup_text_box(), 520)
            .with_icon("text-freetext")
            .enabled_when("doc.pages"),
        command("markup.sticky_note", t::markup_sticky_note(), 521)
            .with_icon("text-sticky")
            .enabled_when("doc.pages"),
        command("markup.stamp", t::markup_stamp(), 522)
            .with_icon("stamp")
            .enabled_when("doc.pages"),
        command("markup.comments", t::markup_comments(), 540)
            .with_icon("comment")
            .enabled_when("doc.open"),
        // ===================================================================
        // MEASURE — tokens 600-699
        // ===================================================================
        command("measure.linear", t::measure_linear(), 600)
            .with_icon("measure")
            .enabled_when("doc.pages"),
        command("measure.radius_diameter", t::measure_radius_diameter(), 601)
            .with_icon("measure")
            .enabled_when("doc.pages"),
        // `set-scale` is the conversion glyph the icon ui-spec §8.2 assigned
        // — two arrows chasing each other round a circle. Deliberately not a
        // third `measure`: this command measures nothing, it changes what
        // measurements are read against.
        command("measure.set_scale", t::measure_set_scale(), 610)
            .with_icon("set-scale")
            .enabled_when("doc.pages"),
        // §8.2 also assigned `icon-ring.svg` here, and that half is a
        // **recorded deviation**: two concentric circles read as a target or
        // a radio button at 16 px, not as a list of named things. The row was
        // written at reservation depth before the Measure surface existed and
        // states no reasoning to weigh against. `list` is shared with
        // `edit.form_manage_fields`; see that registration.
        command("measure.manage_groups", t::measure_manage_groups(), 611)
            .with_icon("list")
            .enabled_when("doc.open"),
        // ===================================================================
        // TOOLS — tokens 700-799
        //
        // The batch commands and the font folders take their inputs from
        // disk, so they are available with nothing open. That is the whole
        // distinction between this tab and Pages, expressed as a predicate.
        // ===================================================================
        command("tools.merge_files", t::tools_merge_files(), 700).with_icon("combine"),
        command("tools.split_files", t::tools_split_files(), 701).with_icon("split"),
        command("tools.font_folders", t::tools_font_folders(), 710).with_icon("font-folders"),
        command("tools.embed_fonts", t::tools_embed_fonts(), 711)
            .with_icon("fonts")
            .enabled_when("doc.open"),
        command("tools.unembed_fonts", t::tools_unembed_fonts(), 712)
            .with_icon("fonts")
            .enabled_when("doc.open"),
        command(
            "tools.render_diagnostics",
            t::tools_render_diagnostics(),
            720,
        )
        .with_icon("tools")
        .enabled_when("doc.open"),
        // ===================================================================
        // FORMAT (contextual) — tokens 800-899
        //
        // The tab is visible when `selection.any` and the command inside it
        // is enabled by the same condition. That is not redundant: the tab
        // and its contents are evaluated independently, and a Format tab
        // that appeared with a greyed Delete would be the placeholder P3
        // forbids, arriving through a mismatch rather than a decision.
        // ===================================================================
        command("format.delete", t::format_delete(), 800)
            .with_icon("delete")
            .enabled_when("selection.any"),
        // ===================================================================
        // MODES — tokens 900-999
        //
        // Not ribbon commands: the three positions of the selector, bound
        // to Ctrl+1/2/3. Always available — a mode is an interface-
        // complexity control, not a permission, and there is no document
        // state in which changing your own view stance should be refused.
        //
        // ★ **No icons, and this is the one entry in the whole "which
        // commands get a glyph" question that is settled by the renderer
        // rather than by taste.** `egui_shell::ribbon::mode_selector` draws
        // the modes as **text segments** of an N-position segmented control,
        // taking each one's `Mode::label` from the manifest — it never looks
        // at a `Command`, and the module contains no icon path at all (the
        // string `icon` does not occur in the file). `MODES_AND_PANELS.md`
        // Part 1 is why: the control must render "as a real segmented control
        // with all three labels visible — not a bare track with a knob, where
        // the available positions are invisible until you drag."
        //
        // So a key here would resolve to art nothing draws. Worse, it would
        // look like a wiring bug to the next reader — a command that names a
        // glyph and never shows one — which is the failure mode the visible
        // slashed mark exists to make loud, arriving in the one place the
        // mark cannot appear.
        // ===================================================================
        command("mode.read", t::mode_read(), 900),
        command("mode.review", t::mode_review(), 901),
        command("mode.edit", t::mode_edit(), 902),
    ]
}

/// ★ **The command id that names a page-display mode**, and its inverse.
///
/// One binding between [`crate::viewer::PageDisplay`] and the ribbon, written
/// down once. The two directions are used by different surfaces and would drift
/// apart if each spelled the mapping for itself:
///
/// * `crate::app::dispatch` turns an invoked command into a mode;
/// * `PdfceApp::conditions` turns the active mode into the `selected:`
///   condition that makes its ribbon button render pressed.
///
/// It lives here rather than on the enum for the reason the enum's own
/// `id`/`from_id` pair lives *there*: `viewer` must not know what a ribbon is.
/// `viewer::PageDisplay::id` is the **on-disk** spelling and this is the
/// **command** spelling, and keeping them separate is what lets either change
/// without silently rewriting the other's files.
///
/// [`tests::every_page_display_mode_has_a_registered_command`] asserts both
/// directions against the live registry, so a fifth mode that is added and not
/// registered fails the suite rather than becoming a mode with no control.
#[must_use]
pub fn page_display_command(display: crate::viewer::PageDisplay) -> &'static str {
    use crate::viewer::PageDisplay as D;
    match display {
        // ui-text-exempt: command ids, never displayed
        D::Single => "view.page_single",
        // ui-text-exempt: command ids, never displayed
        D::Continuous => "view.page_continuous",
        // ui-text-exempt: command ids, never displayed
        D::Facing => "view.page_facing",
        // ui-text-exempt: command ids, never displayed
        D::FacingContinuous => "view.page_facing_continuous",
    }
}

/// The page-display mode `id` names, or `None` if it names none.
///
/// The inverse of [`page_display_command`], derived from it rather than
/// written out a second time — so the two cannot disagree even in principle.
#[must_use]
pub fn page_display_for_command(id: &str) -> Option<crate::viewer::PageDisplay> {
    crate::viewer::PageDisplay::ALL
        .iter()
        .copied()
        .find(|&m| page_display_command(m) == id)
}

/// ★ **The command id that names a piece of View ▸ Display chrome**, and its
/// inverse.
///
/// Exactly the shape [`page_display_command`] has, and here for exactly the
/// same reasons: two surfaces need the mapping in opposite directions —
/// `crate::app::dispatch` turns an invoked command into a
/// [`crate::app::actions::ViewChrome`], and `PdfceApp::conditions` turns each
/// toggle's state into the `selected:` condition that renders its button
/// pressed — and a mapping spelled twice is a mapping that drifts.
///
/// The difference from the page-display pair is that these three are
/// **independent toggles rather than a radio**: all, none or any two may be
/// on at once, so `conditions` publishes between zero and three of these
/// conditions where it publishes exactly one page-display condition. That is
/// the whole of what makes them read as three switches instead of one
/// three-position control.
#[must_use]
pub fn chrome_command(chrome: crate::app::actions::ViewChrome) -> &'static str {
    use crate::app::actions::ViewChrome as C;
    match chrome {
        // ui-text-exempt: command ids, never displayed
        C::Rulers => "view.rulers",
        // ui-text-exempt: command ids, never displayed
        C::Grid => "view.grid",
        // ui-text-exempt: command ids, never displayed
        C::Guides => "view.guides",
    }
}

/// The chrome toggle `id` names, or `None` if it names none.
///
/// Derived from [`chrome_command`] rather than written out a second time, so
/// the two cannot disagree even in principle.
#[must_use]
pub fn chrome_for_command(id: &str) -> Option<crate::app::actions::ViewChrome> {
    crate::app::actions::ViewChrome::ALL
        .iter()
        .copied()
        .find(|&c| chrome_command(c) == id)
}

/// The command id that arms `kind`.
///
/// The **single** binding between a `markup.*` id and a
/// [`crate::canvas::markup::MarkupKind`], in the shape [`chrome_command`]
/// established and for the same reason: a match here plus a derived inverse
/// cannot disagree, where two hand-written tables can.
///
/// ## Why these four and not the ten `RIBBON_IA.md` §5.5 names
///
/// [`crate::canvas::markup::MarkupKind`] enumerates what the rubber-band
/// gesture can draw and the engine can author — nothing else. Underline,
/// strikeout and squiggly mark **text** and need a text-selection gesture
/// that does not exist; polygon, polyline and ink are not drag-shaped;
/// clouds are blocked on `MarkupSpec::Cloud`, which pdfce accepted on
/// 2026-08-14 and has not started.
///
/// Declaring the other six here early would put six dead arms in a type
/// whose job is to say what the tool is doing — the same argument the old
/// shell made about its own tool enum, applied at the gesture boundary.
/// They arrive with the gestures that can draw them.
#[must_use]
pub fn markup_command(kind: crate::canvas::markup::MarkupKind) -> &'static str {
    use crate::canvas::markup::MarkupKind as K;
    match kind {
        // ui-text-exempt: command ids, never displayed
        K::Rectangle => "markup.rectangle",
        // ui-text-exempt: command ids, never displayed
        K::Ellipse => "markup.ellipse",
        // ui-text-exempt: command ids, never displayed
        K::Arrow => "markup.arrow",
        // ui-text-exempt: command ids, never displayed
        K::Highlight => "markup.highlight",
    }
}

/// The markup kind `id` arms, or `None` if it names none.
///
/// Derived from [`markup_command`] rather than written out a second time, so
/// the two cannot disagree even in principle.
#[must_use]
pub fn markup_for_command(id: &str) -> Option<crate::canvas::markup::MarkupKind> {
    crate::canvas::markup::MarkupKind::ALL
        .iter()
        .copied()
        .find(|&k| markup_command(k) == id)
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
        assert_eq!(registry().len(), 88);
    }

    /// ★ **Every chrome toggle has a registered command, and every one of
    /// those commands names a toggle.**
    ///
    /// The twin of [`every_page_display_mode_has_a_registered_command`], and
    /// it catches the same failure: a fourth toggle added to
    /// [`crate::app::actions::ViewChrome`] with no registration would be a
    /// piece of chrome no operator could reach, and nothing else in the suite
    /// would notice. Asserted against the **live registry** rather than
    /// against the mapping's own table, which is the difference between the
    /// code agreeing with itself and the control existing.
    #[test]
    fn every_chrome_toggle_has_a_registered_command() {
        let reg = registry();
        for &chrome in crate::app::actions::ViewChrome::ALL {
            let id = chrome_command(chrome);
            assert!(
                reg.get(id).is_some(),
                "`{id}` names {chrome:?} and is not registered"
            );
            assert_eq!(chrome_for_command(id), Some(chrome), "round trip");
        }
        let mut ids: Vec<&str> = crate::app::actions::ViewChrome::ALL
            .iter()
            .map(|&c| chrome_command(c))
            .collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), crate::app::actions::ViewChrome::ALL.len());
        // …and the two mappings do not overlap, which is what keeps a
        // page-display click from toggling a ruler.
        assert_eq!(chrome_for_command("view.page_single"), None);
        assert_eq!(page_display_for_command("view.rulers"), None);
    }

    /// ★ **Every markup kind the canvas can draw has a registered command,
    /// and no other mapping claims a `markup.*` id.**
    ///
    /// The third of this family, and the one with the most room to go wrong,
    /// because the kinds and the commands were built by different hands: the
    /// canvas enumerates what the *gesture* can draw, the manifest enumerates
    /// what `RIBBON_IA.md` §5.5 *names*, and those two sets are deliberately
    /// different sizes today — ten names, four kinds. This asserts the four
    /// are a genuine subset and reach real controls, not that the sets match.
    ///
    /// The failure it exists to catch is a fifth kind added to `MarkupKind`
    /// with no registration: a tool an operator could not arm, which is
    /// precisely the class of half-built surface `panels`' header is about.
    /// Asserted against the **live registry** rather than the mapping's own
    /// table — the difference between the code agreeing with itself and the
    /// control existing.
    #[test]
    fn every_markup_kind_has_a_registered_command() {
        let reg = registry();
        for &kind in crate::canvas::markup::MarkupKind::ALL {
            let id = markup_command(kind);
            assert!(
                reg.get(id).is_some(),
                "`{id}` names {kind:?} and is not registered"
            );
            assert_eq!(markup_for_command(id), Some(kind), "round trip");
        }
        let mut ids: Vec<&str> = crate::canvas::markup::MarkupKind::ALL
            .iter()
            .map(|&k| markup_command(k))
            .collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(
            ids.len(),
            crate::canvas::markup::MarkupKind::ALL.len(),
            "two kinds sharing one id would arm the wrong tool from one button"
        );
        // …and no other mapping answers to a markup id, nor this one to
        // theirs. The dispatch arm for markup is a GUARD arm — `id if
        // markup_for_command(id).is_some()` — so an overlap here would not
        // merely confuse a lookup, it would swallow another command's arm
        // entirely, and the arm it swallowed would simply stop happening.
        assert_eq!(markup_for_command("view.rulers"), None);
        assert_eq!(markup_for_command("view.tool_hand"), None);
        assert_eq!(markup_for_command("markup.comments"), None);
        assert_eq!(chrome_for_command("markup.rectangle"), None);
        assert_eq!(page_display_for_command("markup.rectangle"), None);
    }

    /// ★ **Every page-display mode has a registered command, and every one of
    /// those commands names a mode.**
    ///
    /// Both directions, against the **live registry** rather than against the
    /// mapping's own table — which is the difference between asserting the
    /// code agrees with itself and asserting that the control exists. The
    /// failure this catches is a fifth mode added to the enum with no
    /// registration: the ribbon would draw three buttons, the fourth would be
    /// unreachable, and nothing else in the suite would notice.
    #[test]
    fn every_page_display_mode_has_a_registered_command() {
        let reg = registry();
        for &mode in crate::viewer::PageDisplay::ALL {
            let id = page_display_command(mode);
            assert!(
                reg.get(id).is_some(),
                "`{id}` names {mode:?} and is not registered"
            );
            assert_eq!(page_display_for_command(id), Some(mode), "round trip");
        }
        // …and the ids are distinct, which the round trip alone would not
        // prove if two modes shared one command.
        let mut ids: Vec<&str> = crate::viewer::PageDisplay::ALL
            .iter()
            .map(|&m| page_display_command(m))
            .collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), crate::viewer::PageDisplay::ALL.len());
        assert_eq!(page_display_for_command("view.zoom_actual"), None);
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
            "doc.open",
            "doc.pages",
            "undo.available",
            "redo.available",
            "selection.any",
            // Not a refinement of `selection.any` — see `PdfceApp::conditions`.
            // A selection can exist and resolve to no box.
            "selection.bounds",
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
            "view.app_initiative",
            "view.floating_panels",
            "view.fullscreen",
            "view.read_mode",
            "view.render_antialias",
            "view.render_quality",
            "view.render_settle",
            "view.render_strategy",
            "view.render_thin_lines",
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
