//! # `shell::commands::catalog` — the list itself, and the argument for every
//! entry on it
//!
//! One function, [`all`], holding every command this build has in manifest
//! order, and one helper, [`command`], that builds each from a catalog entry.
//! Nothing else.
//!
//! ## ★ Why this is its own file
//!
//! `shell/commands/mod.rs` crossed the 1,500-line gate (standing rule **R2**)
//! when `file.save_copy` was wired. The rule's own justification is why the
//! split is *here* rather than at whichever line the count happened to reach:
//! *"the value of the limit is that the file has to have a single subject"*.
//!
//! The parent's subject is **the registry contract** — what a command is, why
//! its five fields are split between code and manifest, what a handler token
//! means, what vocabulary of enable conditions the application promises to
//! publish, and what must be true of the finished registry. This file's
//! subject is **the catalog**: which commands exist, in what order, with which
//! glyph and which predicate, and *why each of those was chosen*. The two
//! change for entirely different reasons — a new condition name is a parent
//! change, a new command is a change here — and they are read at different
//! times.
//!
//! It is the same seam `commands.rs` was already split along once, producing
//! [`super::mapping`] (the id ↔ operand bindings), and the same seam
//! `app/mod.rs` has been split along four times. The test for whether a split
//! was along a seam is whether the *reasoning* came with it, and it did: every
//! paragraph moved here is an argument about a registration.
//!
//! ## ★ The flat list survives the split, and that is the point
//!
//! [`all`]'s own doc comment has always refused a function per tab, because
//! *"a per-tab split would put the handler-token blocks in eight files where a
//! collision between two of them is invisible"*. That argument is untouched:
//! the list is still **one function in one file**, in manifest order, with all
//! nine hundred-blocks visible together and
//! `super::tests::every_handler_token_is_unique` still reading the whole
//! registry. What moved out is the registry's *contract*, not the list.
//!
//! Splitting the list instead would have been the cheaper edit and the wrong
//! one — it would have satisfied a line count by breaking the one property the
//! list's shape exists to protect.
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
//! ## Coverage — ★ the numbers live in the TEST, not in this sentence
//!
//! This heading read *"as of 2026-08-14: 86 of 101 named, 15 refused"* until
//! 2026-08-18, and every one of those three numbers was wrong: the registry
//! held 94, of which 85 were named and 9 refused. `86 + 15 = 101` is
//! internally consistent, which is why nobody looked twice.
//!
//! That is the **fifth** drift of this pair, and the four before it are
//! recorded on `super::tests::the_icon_coverage_split_adds_up_to_the_registry`
//! — which was added, after the fourth, to stop exactly this. It did not,
//! because it pins the split against its own literals and this sentence was
//! never one of them. A test that says *"update the header together"* is a
//! note asking a human to do the thing they just failed to do.
//!
//! So the sentence no longer carries numbers. `run --lib
//! the_icon_coverage_split_adds_up_to_the_registry` prints the current pair
//! and fails when it moves, which is a claim that cannot be stale by
//! construction. This project has now taken the same repair three times — the
//! gate runner's header, `README.md`'s test count, and this — and the shape is
//! always the same: **when prose and a measurement disagree, delete the prose's
//! copy of the measurement rather than correcting it.**
//!
//! What the sentence is still for: before 2026-08-14 the split was 47 named
//! and 41 not, with **no rule behind which was which** — a band drew glyphs and
//! bare words side by side, and the ribbon read as half-finished because it
//! was. Thirty commands gained a key in that pass (25 new glyphs plus two
//! reuses of `chevron-up`/`chevron-down` for page reordering, which is the
//! meaning `crate::icons::Icon::ChevronUp`'s doc comment already gave them).
//!
//! The rest are **recorded refusals**, each stated in full at its own
//! registration below and summarised in `crate::icons::assets` §5 deviation
//! #8. In one line each:
//!
//! | Command(s) | Why no glyph |
//! |---|---|
//! | `view.zoom_actual` | the icon ui-spec §3.2 argues against it by name |
//! | the five `view.render_*` | their labels are the parameter's whole content; no conventional glyph exists for any of them |
//! | `view.app_initiative` | any honest drawing pictures what its default forbids |
//! | `file.recent` | reusing `open` would draw one band control twice |
//! | `mode.read`/`review`/`edit` | the mode selector renders text segments and has no icon path |
//! | `measure.finish` | the set has no check/tick/accept glyph, and the `measure` ruler the three tools share would draw a fourth identical one for a command that places nothing |
//! | `markup.finish` | the same refusal, one tab over: no accept glyph exists, and reusing a shape glyph would draw a fourth near-identical shape in the Shapes band for a command that ends the drawing rather than doing any |
//! | `file.new` | the same refusal as `file.ocr` below, and for the same reason: the icon directory is declared the operator's **own art**, so a new glyph is not a build session's to add. Every reuse was worse than the word — `document` is Properties, `insert-pages` means *pages into this document*, `upload` is import |
//! | `file.ocr` | **the refusal with a different reason from all the others**, and worth reading: there is no recognition glyph and every reuse would mislead (`text-select` is the text *tool*, `search` is Find, `convert` is a format change), but the deciding fact is that the alternative is not available either — `icons/assets/PROVENANCE.md` declares that directory the **operator's own art**, which is what exempts it from `check-shipped-assets`, and adding a machine-drawn SVG would make that provenance note false. A false provenance note is a worse defect than a control that draws its own words |
//!
//! ★ …and moved a third time on 2026-08-14, when the three text-markup kinds
//! were registered **with** three new glyphs (`text-underline`,
//! `text-strikeout`, `text-squiggly`): 79-of-90 became 82-of-93, and the refusal
//! count is unchanged because none of the three refused one. They are new art
//! rather than a reuse of `shape-highlight` for the reason their registration
//! records: the four controls in the Text markup band differ only in the mark
//! they draw, so a shared glyph would leave four identical buttons carrying four
//! different words.
//!
//! ★ The counts above moved twice on 2026-08-14 and the second move is the
//! one to notice: `measure.two_line` was registered **with** a glyph and this
//! line was not updated, so it read 77-of-88 while the registry held 89. A
//! count quoted in prose is not pinned by the test that pins the registry —
//! `registration_succeeds_and_registers_every_command` would have stayed
//! green through any drift here. Both are corrected together.
//!
//! ★ …and a **fourth** move, later the same day, when `view.tool_text` was
//! registered with the new `text-select` glyph: 82-of-93 became 82-of-94 and the
//! refusal count stayed at twelve, because the text tool refused nothing.
//!
//! ★ **Fifth, later still on 2026-08-14**: the three unblocked Phase 6 markup
//! kinds — `markup.polyline`, `markup.polygon` and `markup.ink` — arrived **with**
//! three new glyphs, and `markup.finish` arrived **without** one, refusing it on
//! `measure.finish`'s own argument. 82-of-94 with twelve refusals became
//! **85-of-98 with thirteen**. This is the first of the five moves that was made
//! with the arithmetic under test rather than under advice: the split is now
//! pinned by `super::tests::the_icon_coverage_split_adds_up_to_the_registry`, so the
//! numbers in this section and the numbers in the registry cannot drift apart
//! silently again.
//!
//! ★ **That fourth pass also found the third line above to have been wrong**,
//! and it is worth stating rather than silently repairing, because it is the
//! same defect that line was written to record. It read *"82 of 93 named, 12
//! refused"* — and 82 + 12 is 94, not 93. The registry held 93 and **81** of
//! them named a glyph; the prose had been incremented one step too far in the
//! text-markup pass. Nothing detected it, for exactly the reason that pass
//! wrote down: the test pins the registry's size and nothing pins the split.
//! The arithmetic check that would have caught it — *named + refused must equal
//! the registry* — is the one property worth carrying forward here, and it is
//! now asserted by
//! `super::tests::the_icon_coverage_split_adds_up_to_the_registry` rather than being
//! left to a reader to do in their head.
//!
//! ★ **Sixth, 2026-08-14: `file.about` arrived with the new `info` glyph**,
//! making it **86-of-99 with thirteen** — About refused nothing, a circled `i`
//! being the most conventional glyph any toolbar has. First move made with the
//! arithmetic already under test; it cost one number in one assertion, which is
//! what the five paragraphs above were for. **Three of the four "count in prose"
//! incidents this module records were found by hand; the fourth was found by
//! the first three's own advice, which is why the advice is now a test.**
//!
//! **A band control's icon does not replace its label.**
//! `egui_shell::ribbon::band::command_button` is called with
//! `shows_label: true` from the band, always; only the QAT goes icon-only,
//! and only `file.open`, `file.save_copy`, `edit.undo` and `edit.redo` are
//! on it. Two of the notes retired in this pass had reasoned as though the
//! choice were "a glyph *or* a findable word", and it never was. That
//! misreading is worth keeping written down: it is what kept three Display
//! toggles and the Pages panel bare for longer than any decision did.

use super::FILE_RECENT;
use crate::text::commands as t;
use crate::text::commands::CommandText;
use egui_shell::{Command, HandlerToken};

/// One command, with its label and tooltip taken from the catalog.
///
/// The two are always fetched together, from one catalog entry, so a
/// command cannot end up with one command's label and another's tooltip —
/// which is not a hypothetical: the salvage source's two adjacent Content
/// buttons both read `Aa`, and only their tooltips distinguished them.
pub(super) fn command(id: &str, text: CommandText, handler: u64) -> Command {
    Command::new(id, text.label, HandlerToken::new(handler)).with_tooltip(text.tooltip)
}

/// Every command, in manifest order.
///
/// One flat list rather than a function per tab: the registry is a flat
/// namespace, the ordering here mirrors the ribbon so the two can be read
/// side by side, and a per-tab split would put the handler-token blocks in
/// eight files where a collision between two of them is invisible.
pub(super) fn all() -> Vec<Command> {
    vec![
        // ===================================================================
        // FILE — tokens 100-199
        // ===================================================================
        // ★ **New — first in the band, and with no glyph.**
        //
        // Order: New, Open, Recent, Close. All three reference applications
        // open their File menu with New and follow it with Open, and this is
        // also the useful order — the two ways to *get* a document, then the
        // two ways to get one *back*, then the way to put one away.
        //
        // **No icon, and it is a recorded refusal rather than an oversight.**
        // The refusal has `file.ocr`'s reason, which is the one reason on the
        // list that is not about the drawing being hard: `icons/assets/`
        // declares itself the **operator's own art**, and that declaration is
        // exactly what exempts the directory from `check-shipped-assets`'
        // notice surfaces. A machine-drawn SVG added by this session would make
        // that provenance note false, and a false provenance note is a worse
        // defect than a control that draws its own word. Reusing an existing
        // key was considered and refused too: `document` is the Properties
        // glyph, `insert-pages` means *pages into this document*, and `upload`
        // is the import half of the export pair — each would say something New
        // does not do. A blank-page glyph is the operator's to draw, and until
        // it exists this control reads `New`, which nobody has ever had to look
        // up.
        //
        // **No enable predicate**, like `file.open` and for the same reason: an
        // operator with nothing open is exactly the operator most likely to
        // want this.
        command("file.new", t::file_new(), 103),
        // ★ The sized New, immediately after the plain one.
        //
        // Order matters here in the way §5.1's own table does: New, New from
        // template, Open, Recent, Close. The two ways to MAKE a document sit
        // together, then the two ways to get one back, then the way to put one
        // away.
        //
        // **No icon**, for `file.new`'s reason and not a new one: the icon
        // directory is declared the operator's own art, and the two New
        // controls sharing a glyph they do not have would be worse than the
        // two of them reading `New` and `New from template…`.
        //
        // **No enable predicate**, for the strongest version of `file.new`'s:
        // an operator with nothing open is not somebody this is tolerated for,
        // they are the operator it exists for.
        command("file.new_from_template", t::file_new_from_template(), 104),
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
        // ★ **Copy page text / Copy document text — were `edit.copy_page_text`
        // and `edit.copy_document_text`, tokens 420 and 421, until the operator
        // decided on 2026-08-14 that they belong here.**
        //
        // This is the same taxonomy move `view.panel_forms` records one block
        // down, applied to the same line from the other side. Filling a form is
        // not authoring; **copying text out is not authoring either**. Both
        // verbs read the document and write somewhere that is not the document,
        // and neither can change a byte of the file.
        //
        // # What forced it, and why the Edit tab was the wrong home
        //
        // The chord/mode gate landed the same day
        // (`crate::app::modes::capability::offers_command`): a chord may reach a
        // command the active mode **shows**, or one that lives on **no ordinary
        // tab**. `Ctrl+Shift+C` is bound to the page-text copy, which sat on the
        // Edit tab — a tab Read does not show — so Read refused it. Acrobat
        // Reader copies text, and *replacing Acrobat Reader* is this project's
        // stated goal for Read, so a Read that cannot copy is wrong about the
        // one thing that mode exists to be. The gate SURFACED that; it did not
        // cause it. The command had been on the wrong tab since the day it
        // arrived there.
        //
        // # Why File ▸ Export rather than a new group or View
        //
        // The File tab is in every mode's tab list, so a command here is
        // reachable from Read, Review and Edit without any exception list — the
        // gate's own second clause never has to be invoked. And this group is
        // the right group rather than merely an available one: `file.export_dxf`
        // writes the page's geometry out to a file another program reads, and
        // `file.export_form_data` writes the filled values out the same way.
        // **Copying the page's text out is an export of content**, differing
        // only in the destination — a clipboard rather than a path — which is a
        // difference the labels carry and the caption does not need to.
        //
        // # Tokens 122 and 123, and the two gaps left behind
        //
        // New ids get new numbers in the `file.` block; 420 and 421 stay unused
        // for the reason the header states and `edit.form_fill`'s vacated 430
        // already demonstrates — a token is what a trace prints, and reusing one
        // would make an old trace of a text copy read as whatever inherited its
        // number. Gaps in the numbering are fine and expected.
        //
        // The `copy` icon, the `doc.pages` predicate and both tooltips come
        // across unchanged: nothing about what these commands DO has moved, only
        // where an operator finds them. `doc.pages` in particular is still the
        // right predicate rather than `doc.open` — text is drawn on pages, and a
        // legal `/Count 0` document has none to copy from.
        command("file.copy_page_text", t::file_copy_page_text(), 122)
            .with_icon("copy")
            .enabled_when("doc.pages"),
        command("file.copy_document_text", t::file_copy_document_text(), 123)
            .with_icon("copy")
            .enabled_when("doc.pages"),
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
        // Settings, the shortcut list and About are always available: they
        // are about pdfce, not about a document.
        command("file.settings", t::file_settings(), 150).with_icon("settings"),
        command("file.shortcuts", t::file_shortcuts(), 151).with_icon("keyboard"),
        // ★ `file.about` carries an OBLIGATION, not a courtesy: it is the
        // in-application half of the attribution surface that shipping
        // CC-BY-SA-4.0 OCR model weights requires, BY needing the notice to
        // reach the RECIPIENT rather than a reader of the repository. The
        // argument is in `crate::text::about`; the gate that keeps both
        // halves true is `tools/gates/check-shipped-assets.py`.
        command("file.about", t::file_about(), 152).with_icon("info"),
        // ★ `file.ocr` — REGISTERED WITH NO ICON. The refusal's full argument
        // is the `file.ocr` row of this module's header table; in one line, the
        // icon directory is declared the operator's OWN ART, so a new glyph is
        // not a build session's to add, and every available reuse would tell an
        // operator the button does something it does not.
        //
        // `doc.pages` rather than `doc.open`: recognition needs a page to
        // rasterize, and a document with none would open a dialog whose only
        // possible outcome is a refusal.
        //
        // ★ On the FILE tab, where `RIBBON_IA.md` §5.7 says Tools. Read's tab
        // list is `["file", "view"]`, so Tools would put OCR out of reach in
        // the one mode the operator asked for it in. Argued in full in
        // `super::manifest::tools`'s header.
        command("file.ocr", t::file_ocr(), 160).enabled_when("doc.pages"),
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
        // ★ **All five were UNREGISTERED on 2026-08-17**, and tokens 210-214
        // are retired rather than reused — a token is an operator's saved
        // keybinding, and handing 211 to something else would silently rebind
        // whatever they had put on it.
        //
        // Three of the five had nothing behind them: there is no
        // tiled-progressive path in this shell, and `RenderOptions` has neither
        // a thin-lines nor an antialiasing field (`interpret.rs` sets
        // `anti_alias: true` as a literal). The other two were real and became
        // **settings** — Settings ▸ Drawing the page — because a value an
        // operator sets once and forgets is not an activity, which is what P2
        // says a ribbon tab picks.
        //
        // R8 is the rule that makes this a deletion rather than a hidden
        // control: *registering a command is the only way the GUI may learn
        // that a capability exists*. Three of these named no capability, and
        // the other two are no longer reached by a command at all.
        // `crate::app::prefs`' header carries the evidence per verdict.
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
        // ★★ **The two pointer tools that make the canvas predictable**, added
        // 2026-08-19 on the operator's report:
        //
        // > *"The selector should be predictable like other programs. It seems a
        // > lot of ideas are getting invented instead of just using the … most
        // > common method expected."*
        //
        // He is right, and `view.tool_select` had been **deliberately absent** —
        // the comment that used to sit here read *"There is deliberately no
        // `view.tool_select` beside them"*, on the argument that Select is the
        // default you return to rather than a thing you pick. That argument is
        // sound and it produced an unusable surface: with no Select control
        // there was no *row of tools*, so the Hand and the Text tool read as two
        // unrelated toggles rather than as members of a set, and there was
        // nowhere for a third and fourth to join. A tool palette is the most
        // conventional object in this product class; not having one is the
        // invention.
        // ★★ **The object clipboard, 2026-08-19** — the operator's report:
        // *"also the standard copy/paste and I didn't try cut so possibly that
        // one too aren't implemented."* They were not.
        //
        // ★ Scoped to **markup and comments**, because that is what the engine
        // can express: `annot_author::spec_from_dict` reads one and `add_markup`
        // writes one back. Page content cannot be pasted — 157 verbs in
        // `edit.rs` and none inserts content, checked 2026-08-19 — so a copy of
        // a path would be offering a paste that could never happen. The labels
        // say "comment or markup" rather than "object" for exactly that reason.
        //
        // `enabled_when("doc.pages")` rather than a selection condition: what is
        // selected changes every click, and a control that greys and un-greys
        // under the pointer is harder to aim at than one that answers in a
        // sentence when pressed. The refusals are `canvas::clipboard::Refusal`,
        // on the status row, which is the same posture the six resize refusals
        // take.
        command("edit.cut", t::edit_cut(), 403)
            .with_icon("cut")
            .enabled_when("doc.pages"),
        command("edit.copy", t::edit_copy(), 404)
            .with_icon("copy")
            .enabled_when("doc.pages"),
        command("edit.paste", t::edit_paste(), 405)
            .with_icon("paste")
            .enabled_when("doc.pages"),
        command("view.tool_select", t::view_tool_select(), 252)
            .with_icon("cursor")
            .enabled_when("doc.pages"),
        command("view.tool_node", t::view_tool_node(), 253)
            .with_icon("cursor-node")
            .enabled_when("doc.pages"),
        command("view.tool_hand", t::view_tool_hand(), 225)
            .with_icon("hand")
            .enabled_when("doc.pages"),
        // ★ **The text tool** — 2026-08-14, and it closes two things at once.
        //
        // Beside `view.tool_hand` because View ▸ Navigate is where the *other*
        // pointer-tool toggle already lives, and because View is the one tab
        // every mode is shown. Both halves of that matter: a tool is a mode the
        // page is in rather than an action taken on it (which is why Navigate is
        // its own group and not a fourth button in Zoom), and a command lives on
        // exactly one tab — so a text tool on the **Edit** tab would be
        // unreachable from Read and Review, which is the shape of mistake the
        // operator has already had to correct twice (`edit.form_fill` →
        // `view.panel_forms`, `edit.copy_page_text` → `file.copy_page_text`).
        //
        // What it closes:
        //
        // 1. `canvas::textsel::takes_the_press` gave a press its text meaning
        //    only for the select tool in a mode that cannot select content, so
        //    Read ✓, Review ✓, **Edit ✗** — a reviewer could sweep text and an
        //    editor could not.
        // 2. The three `markup.*` text-markup commands are drawn on the Markup
        //    tab in Edit and could **never enable** there, because
        //    `selection.text` was never true. That is a live tension with
        //    `RIBBON_IA.md` P3 — greying is for *temporarily* unavailable — and
        //    it was not fixable by hiding them, because the Markup tab is in both
        //    Review and Edit and a command has one tab.
        //
        // ★ **The reference applications disagree here and Inkscape won.**
        // Acrobat and SolidWorks resolve text-versus-object *contextually*
        // inside one tool; only Inkscape uses a separate Text tool. The full
        // argument is at `crate::canvas::tool::CanvasTool::Text` and is not
        // restated here — in one line, an object marquee over *vector content*
        // is a surface Acrobat does not have at all, so its contextual answer is
        // not an answer to this conflict.
        //
        // **It arms a tool; it authors nothing**, so it takes no capability and
        // `retire_forbidden` permits it in every mode. It renders **pressed**
        // while armed through the same `selected:` convention `view.tool_hand`
        // documents, published from `PdfceApp::conditions` — the step that was
        // once forgotten for the measure tools and shipped a tool that armed
        // without looking armed.
        //
        // `text-select` is a new glyph rather than a reuse of `add-text`: that
        // one is this I-beam **plus a badge**, and the badge is the difference
        // between creating text and selecting it. `doc.pages`, like every other
        // entry in this group — a pointer tool with no page under it has nothing
        // to point at.
        command("view.tool_text", t::view_tool_text(), 226)
            .with_icon("text-select")
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
        // ★★ **The Tool panel**, registered 2026-08-19 — the operator's item 4,
        // *"no side bar area showing what tool is active and its options"*, and
        // the fix for his item 5 as well.
        //
        // Token 247, out of the `view.panel_*` run, because tokens are never
        // reused and 240-246 are taken. The ORDER on the ribbon is
        // the manifest's, not the token's: this sits FIRST in View ▸ Panels,
        // ahead of every navigator, because it is the only panel there that
        // answers *"what can I do"* rather than *"what is in this file"* — and
        // because an operator looking for a missing command scans that group
        // from the top.
        command("view.panel_tool", t::view_panel_tool(), 247).with_icon("pointer"), // ★ NO `enabled_when`. A panel toggle is about the operator's own
        // screen, and the Tool panel is the one panel whose body says
        // something useful with nothing open — Block B still names the
        // tools this mode has and where they live. Gating it on
        // `doc.open` would hide the surface at exactly the moment somebody
        // has launched pdfce and is looking for what it does.
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
        // ★ `view.floating_panels` (252) and `view.app_initiative` (253) were
        // UNREGISTERED on 2026-08-17, tokens retired rather than reused.
        //
        // Neither had anything behind it. `egui-shell`'s dock has no floating
        // mode at all — its only `floating` is `egui`'s scroll-bar style — so
        // the first governed a capability that does not exist.
        //
        // The second is the more interesting deletion, and worth keeping the
        // reasoning for. `view.app_initiative` was a three-position policy —
        // Never · Ask · Allowed — about whether pdfce may float a surface over
        // the page **on its own initiative**. Its specified default was
        // **Never**, and *nothing in this build does that*: the default is
        // already true by construction. So the control existed to switch off a
        // behaviour pdfce does not have, which is a control that cannot do
        // anything whichever way it is set.
        //
        // Building it would mean building the behaviour first, and the
        // behaviour is the thing the operator objected to. It goes back on the
        // list the day something wants to float unasked, and not before.
        command("view.reset_layout", t::view_reset_layout(), 254).with_icon("reset-layout"),
        // ★ **The two document-switching verbs**, registered 2026-08-19 with
        // the document tab strip.
        //
        // `enabled_when("docs.multiple")` and not `doc.open`: with one document
        // open there is nothing to switch to, and R9 reserves greying for
        // *temporarily* unavailable — which this is, exactly. Opening a second
        // document arms both, and the hover says what they would do.
        //
        // They exist as commands rather than as bare keyboard handling because
        // `R8` allows no other way for the shell to learn a capability is
        // present: the chords in the manifest resolve against this registry,
        // so a build without them would have Ctrl+Tab bound to nothing rather
        // than bound to something that silently does nothing.
        command("view.next_document", t::view_next_document(), 255)
            .with_icon("chevron-right")
            .enabled_when("docs.multiple"),
        command("view.previous_document", t::view_previous_document(), 256)
            .with_icon("chevron-left")
            .enabled_when("docs.multiple"),
        // ★ **Close others**, 2026-08-20, with the document tab strip.
        //
        // Its operand depends on the route: from a tab's context menu it keeps
        // the tab that was right-clicked, from the ribbon it keeps the one on
        // screen. `crate::app::PdfceApp::tab_menu_target` is how the first is
        // supplied and `unwrap_or(active_slot)` is how the second falls back —
        // and the tooltip says *"the one you opened this on"* rather than
        // naming either, because that sentence is true from both routes.
        //
        // ★ There is deliberately **no** `close_document` beside it. The
        // conventional tab menu has three rows — Close, Close others, Close to
        // the right — and a Close here would be a second command with
        // `file.close`'s label and `file.close`'s behaviour from the ribbon,
        // differing only in a parked operand. `no_two_commands_share_a_label`
        // and `every_menu_command_is_also_reachable_from_the_ribbon` both
        // caught the attempt, and between them they are right: closing the tab
        // you right-clicked is already the ✕ on that tab and a middle click on
        // it, which are the two gestures every operator reaches for first.
        //
        // No icon. `catalog`'s coverage table calls a context-menu row's glyph
        // decoration: a menu is a list of words, read rather than scanned, and
        // a half-iconed menu is worse than none.
        command(
            "view.close_other_documents",
            t::view_close_other_documents(),
            257,
        )
        .enabled_when("docs.multiple"),
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
        // `edit.copy_page_text` and `edit.copy_document_text` were here, tokens
        // 420 and 421. They are now `file.copy_page_text` and
        // `file.copy_document_text` in File ▸ Export — operator decision,
        // 2026-08-14; see those registrations for the argument. Both numbers
        // stay unused, exactly as 430 below does, and for the same reason.
        //
        // The Edit ▸ Clipboard group went with them, because those two were its
        // only members and an empty group must not ship. `super::manifest`'s
        // group count moved 32 → 31 with it.
        //
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
        // ★ **The three unblocked Phase 6 kinds** — Phase 6, 2026-08-14, moving
        // out of `manifest::PLANNED`.
        //
        // `FEATURES.md` carried all three as *"engine-ready, but not drag-shaped;
        // each needs its own gesture"* for the whole project, and that is exactly
        // what they got: `canvas::markup::vertex` for the two click-shaped kinds
        // and `canvas::markup::ink` for the freehand one. Nothing about the
        // engine changed — `MarkupSpec::PolyLine`, `Polygon` and `Ink` have been
        // there since Pass 6.1.
        //
        // # `doc.pages`, like the four kinds above and unlike the three below
        //
        // These **arm a tool**; they do not act. That is the line the enable
        // predicate draws: a shape command is live wherever there is a page to
        // draw on, and a *mark* command is live only when there is a selection to
        // mark (`selection.text`). Getting that backwards would grey Polygon
        // until something unrelated was selected.
        //
        // # The icons are three new glyphs, and two of them are one idea
        //
        // `shape-polyline` is `shape-polygon` with its closing segment removed,
        // which is exactly how the two annotations differ (§12.5.6.13). Drawing
        // them as a pair is what makes the band teachable: an operator who learns
        // one has learned the other. `shape-ink` is deliberately *not* a reuse of
        // `text-squiggly` — that one is a periodic wave in a band under two text
        // lines and means "mark these words"; this one is an aperiodic full-tile
        // stroke and means "the path your hand took". See
        // `crate::icons::Icon::ShapePolyline` and its two siblings.
        command("markup.polyline", t::markup_polyline(), 503)
            .with_icon("shape-polyline")
            .enabled_when("doc.pages"),
        command("markup.polygon", t::markup_polygon(), 504)
            .with_icon("shape-polygon")
            .enabled_when("doc.pages"),
        // ★ **Revision cloud**, registered 2026-08-19 — the operator's item 6,
        // raised three times in his own words: *"still no revision cloud
        // tool."*
        //
        // Token **507**, out of the band's own run, because 506 is
        // `markup.finish` and tokens are never reused. The ORDER on the ribbon
        // is the manifest's, not the token's, so this sits between Polygon and
        // Freehand where it belongs — beside the tool it is a variant of.
        //
        // It was in `crate::shell::manifest::PLANNED` with the reason *"the
        // ONLY markup kind still absent for an ENGINE reason rather than a
        // gesture one"*, and that had quietly stopped being true:
        // `MarkupSpec::Cloud` shipped in `pdfce-core` and nothing in this shell
        // noticed for weeks. A PLANNED entry is a claim about the world and it
        // decays; this one cost three weeks of the operator asking for a tool
        // whose only blocker had already been removed.
        command("markup.cloud", t::markup_cloud(), 507)
            .with_icon("shape-cloud")
            .enabled_when("doc.pages"),
        command("markup.ink", t::markup_ink(), 505)
            .with_icon("shape-ink")
            .enabled_when("doc.pages"),
        // ★ **Finish shape** — the ribbon half of the vertex tools' ending, and
        // `measure.finish`'s twin in every respect that matters.
        //
        // Polyline and Polygon are the only markup gestures with no natural end:
        // a band drag ends when the button comes up and a freehand stroke ends
        // the same way, but a run of clicks does not end itself. The operator
        // settled that shape of problem on 2026-08-14 for the radius/diameter
        // tool — **two endings through one commit path** — and this is that
        // answer applied to the second tool with the same problem, deliberately
        // rather than inventing a third. A double-click on the canvas is the
        // ending most operators will use; this is the discoverable one, and the
        // one that works when the last corner sits somewhere awkward to
        // double-click.
        //
        // # Why `markup.finishable` and not `doc.pages`
        //
        // Because a Finish that is always enabled is a control that does nothing
        // on almost every press, and P3 reserves greying for *temporarily
        // unavailable* — which is exactly what this is. The predicate is the same
        // question the arm asks (`canvas::markup::vertex::finishable`, one
        // derivation shared with `vertex::finish`), so the control is live
        // precisely when pressing it would author an annotation.
        //
        // It is also where the polygon/polyline difference becomes visible: a
        // polygon needs three vertices where a polyline needs two, so after two
        // clicks this control is live for one tool and greyed for the other. The
        // operator is told the rule before they press, rather than refused after.
        //
        // # No icon, and it is the same deliberate refusal `measure.finish` makes
        //
        // There is no check-mark, tick or accept glyph in the set, and no
        // existing key means "complete this gesture". Reusing one of the three
        // shape glyphs would draw a fourth near-identical shape in the same band
        // for a command that draws nothing — it *ends* the drawing — and would
        // undermine the pairing argument the polyline/polygon glyphs above rest
        // on. Naming a key that does not exist draws a visible slashed mark,
        // which is a placeholder arriving through the back door. So it renders as
        // its words, which for a completion verb is the clearest thing it could
        // be.
        command("markup.finish", t::markup_finish(), 506).enabled_when("markup.finishable"),
        command("markup.highlight", t::markup_highlight(), 510)
            .with_icon("shape-highlight")
            .enabled_when("doc.pages"),
        // ★ **The three text-markup kinds** — Phase 6, 2026-08-14, moving out of
        // `manifest::PLANNED`.
        //
        // # Why `selection.text` and not `doc.pages`
        //
        // Because these three do not arm a tool: they act **at once**, on the
        // text selection the operator has already made
        // (`canvas::markup::text` §1, which records that this is Acrobat's
        // model and why it was chosen over arm-then-sweep). A control gated on
        // `doc.pages` would therefore be live on every open document and would
        // do nothing on almost every press — which is what `RIBBON_IA.md` P3
        // forbids and what `measure.finish` set the precedent for answering with
        // a condition of its own.
        //
        // The predicate is the same question the dispatch arm asks, so the
        // control cannot be enabled while pressing it would decline: `conditions`
        // publishes `selection.text` from a **live** selection on the open
        // document, and `markup::text::mark` refuses anything else.
        //
        // # ★ Where they are reachable, which is narrower than the tab suggests
        //
        // **Review, and Review alone.** Read cannot author markup (its tab list
        // is File and View, so the Markup tab is not there at all), and Edit
        // cannot make a text selection (its primary button is the content
        // marquee — `canvas::textsel::takes_the_press`), so in Edit these three
        // are drawn and permanently greyed. That is an inversion, it is
        // recorded rather than smoothed over, and it closes the day
        // `CanvasTool::Text` lands. See `canvas::markup::text` §2.
        //
        // # The icons
        //
        // Three new glyphs rather than a reuse of `shape-highlight`: the four
        // controls in the Text markup band differ *only* in the mark they draw,
        // so a shared glyph would make the band four identical buttons with
        // four different words — the exact opposite of the "family shares a
        // glyph" convention this module's header describes, which is for
        // commands whose difference is carried by the label.
        command("markup.underline", t::markup_underline(), 511)
            .with_icon("text-underline")
            .enabled_when("selection.text"),
        command("markup.strikeout", t::markup_strikeout(), 512)
            .with_icon("text-strikeout")
            .enabled_when("selection.text"),
        command("markup.squiggly", t::markup_squiggly(), 513)
            .with_icon("text-squiggly")
            .enabled_when("selection.text"),
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
        // ★ **Perimeter** - the operator's ask of 2026-08-20.
        //
        // Shares the `measure` glyph with its three neighbours for the reason
        // the note below gives about Two-line: all four place a dimension, and
        // what differs is what they measure FROM. Four near-identical rulers
        // would make the group harder to read, not easier, and the label is
        // where the distinction belongs.
        command("measure.perimeter", t::measure_perimeter(), 604)
            .with_icon("measure")
            .enabled_when("doc.pages"),
        // Registered as part of Phase 7, moving out of `manifest::PLANNED`.
        //
        // It shares the `measure` glyph with Linear and Radius/diameter rather
        // than getting a third: all three place a dimension, and what differs
        // is what they measure *from* — two clicked points, an arc, or two
        // lines already on the drawing. That distinction is what the label
        // says, and drawing three near-identical rulers would make the group
        // harder to read rather than easier.
        command("measure.two_line", t::measure_two_line(), 602)
            .with_icon("measure")
            .enabled_when("doc.pages"),
        // ★ **Finish** — the ribbon half of the radius/diameter tool's ending.
        //
        // The radius/diameter gesture is the only one on this tab with no
        // natural end: Linear finishes at three clicks and Two-line at two,
        // because both are picks of a known arity, and a best-fit circle is
        // finished when the operator says it is. A double-click on the canvas
        // is the other half of the answer and is the one most operators will
        // use; this is the discoverable one, and the one that works when the
        // last picked arc is somewhere awkward to double-click.
        //
        // # Why `measure.finishable` and not `doc.pages`
        //
        // Because a Finish that is always enabled is a control that does
        // nothing on almost every press, and P3 reserves greying for
        // *temporarily unavailable* — which is exactly what this is. The
        // predicate is the same question the arm asks
        // (`canvas::measure::finishable`, one derivation shared with
        // `canvas::measure::finish`), so the control is live precisely when
        // pressing it would author a dimension: the circular tool armed, a pick
        // set on the page, and a fit that is not degenerate. Two picked arcs on
        // a straight line leave it greyed, correctly — there is no circle in
        // them to commit.
        //
        // # No icon, and it is a deliberate refusal
        //
        // There is no check-mark, tick or accept glyph in the set, and no
        // existing key means "complete this gesture". Reusing `measure` — the
        // key the three tools share — would draw a fourth identical ruler in
        // the same group and undermine the very argument the two-line
        // registration above makes for sharing it: the family shares a glyph
        // because all three *place a dimension*, and this one places nothing,
        // it ends the placing. Naming a key that does not exist draws a visible
        // slashed mark, which is a placeholder arriving through the back door.
        // So it renders as its word, which for a one-word completion verb is
        // the clearest thing it could be.
        command("measure.finish", t::measure_finish(), 603).enabled_when("measure.finishable"),
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
        // ★ A second ROUTE to `file.properties`, not a second command that
        // opens the panel. Its arm raises `Action::Command("file.properties")`,
        // which is the mechanism that keeps one command's guards in one place.
        //
        // Registered as its own id because the shell enforces one command, one
        // tab — and the two placements answer different questions: File ▸
        // Document is "tell me about this file", Format is "tell me about the
        // thing I just clicked".
        //
        // The icon is `properties`, shared with `file.properties` under the
        // header's shared-key convention: same panel, same glyph, and the two
        // are never drawn together because Format is contextual.
        command("format.properties", t::format_properties(), 801)
            .with_icon("properties")
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
