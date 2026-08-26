//! # text::commands — the label and tooltip of every ribbon command
//!
//! One function per command, each returning a [`CommandText`]. The ribbon's
//! *structural* strings — tab labels, tab questions, group captions, mode
//! labels — live next door in [`crate::text::ribbon`].
//!
//! ## Why a pair rather than two functions
//!
//! Every command needs both a label and a tooltip, and they are written
//! together: the tooltip's job is to say what the label cannot fit, so
//! reviewing one without the other is reviewing half a sentence. Two
//! functions per command would also double a file that is already the
//! longest in the catalog, for no gain — nothing ever wants one without
//! being able to reach the other.
//!
//! ## Every command has a tooltip. That is a rule, not an accident.
//!
//! `RIBBON_IA.md` P3 reserves greying for *temporarily* unavailable — no
//! document open, undo stack empty — and requires that it *"is always
//! explained on hover."* A command with no tooltip cannot honour that, so
//! [`CommandText`] has no way to express "no tooltip" and a test below
//! asserts none is empty.
//!
//! The salvage source got this wrong in exactly one place and it is
//! instructive: the four Measure buttons (`Linear Dimension`,
//! `Radius / Diameter Dimension`, `Set Group Scale…`, `Manage Dimension
//! Groups…`) were rendered as text-only selectables **with no tooltip at
//! all** — the four controls on the tab most likely to be used by someone
//! who has never used a PDF measuring tool.
//!
//! ## Voice, carried across from the salvage source deliberately
//!
//! pdfce's tooltips are unusually long and unusually specific, and that is
//! a deliberate quality of the product rather than an accident of who
//! wrote them. They say what a command *changes* ("This changes the
//! document, not just the view"), what it *cannot* do ("pdfce does not
//! check whether they are valid"), and what is *irreversible* ("Marking is
//! reversible; applying is not"). Where the salvage source's wording said
//! something worth keeping, it is kept close to verbatim.
//!
//! Two things are trimmed:
//!
//! 1. **Tooltips that enumerate the alternatives.** The old `Add Text`
//!    tooltip explained itself by contrast with three other commands over
//!    four sentences. One contrast is a clarification; three is a menu.
//! 2. **Tooltips that describe a defect.** `"click-to-place editing on the
//!    canvas is coming"` is a roadmap entry, not a tooltip.
//!
//! ## Labels: three renames that `RIBBON_IA.md` §5.4 requires
//!
//! `Aa`, `I⁺ Aa` and `Obj` become **Edit text**, **Add text** and **Edit
//! objects**. They are the primary content-editing tools and were the
//! least legible controls in the application — and the first two returned
//! the *same literal*, `"Aa"`, distinguished only by icon and tooltip.

/// The two operator-visible strings a ribbon command carries.
///
/// A plain pair of `&'static str` rather than owned `String`s because
/// every one of them is a literal in this file: the catalog is the
/// definition site, so there is nothing to allocate and a command's text
/// can be read in a `const` context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandText {
    /// What the control says. Sentence case, no trailing period; an
    /// ellipsis when activating it opens a dialog rather than acting.
    pub label: &'static str,
    /// What the control says on hover. A full sentence, with punctuation.
    pub tooltip: &'static str,
}

impl CommandText {
    /// Pair a label with its tooltip.
    #[must_use]
    pub const fn new(label: &'static str, tooltip: &'static str) -> Self {
        Self { label, tooltip }
    }
}

/// **The View tab's entries**, split out on 2026-08-20 when this file crossed
/// rule R2's ceiling.
///
/// Re-exported below, so nothing changed for a caller: every call site still
/// writes `crate::text::commands::view_zoom_in()`. See that module's header
/// for why the seam was drawn there and nowhere else.
mod view;

pub use view::*;

// ===========================================================================
// FILE TAB
// ===========================================================================

/// `file.open`
#[must_use]
pub const fn file_open() -> CommandText {
    CommandText::new("Open…", "Open a PDF document (Ctrl+O).")
}

/// `file.new`
///
/// **`New`, with no ellipsis**, and that is the label carrying a promise: an
/// ellipsis says *this will ask you something*, and this does not. Two of the
/// three reference applications create a document immediately from a default
/// (Acrobat from a locale default, Inkscape from its default template) and
/// only SolidWorks asks — and what it asks is which *kind* of document, a
/// question pdfce has no analogue for. See `crate::app::blank` §3.
///
/// The tooltip states the page size **because the command does not ask for
/// it**. A default that is never mentioned is a default an operator discovers
/// by measuring the page they just made, and A4 is a real choice — argued from
/// what the three reference applications do, and from this operator's own
/// A-series drawings — rather than an accident to be hidden.
///
/// ★ **The last sentence was a claim, and it stopped being true on
/// 2026-08-14.**
///
/// It read: *"This build cannot yet write a document to disk, so a new document
/// lasts as long as the window does."* That was accurate when it was written —
/// `file.save_copy` was registered with no dispatch arm — and `file.save_copy`
/// is now wired, so leaving it would have told the operator that the document
/// in front of them cannot be kept when a control two groups away keeps it.
///
/// It is replaced rather than deleted, because the thing an operator will
/// otherwise find out the hard way has changed rather than gone: New is still
/// the command where the *shape* of saving bites first. `Save a copy…` asks for
/// a destination every time and never adopts it, so a created document keeps
/// its `Untitled` name however often it is saved — which is what the new
/// sentence says, and which is Inkscape's behaviour for the same verb. See
/// `crate::app::save` §3.4.
///
/// The record of the correction is kept here for the reason `HANDOFF.md` §10
/// gives about prose that quotes a fact: this is the fifth such drift the
/// project has recorded, and the only defence that works is noticing them at
/// the site of the change that invalidated them.
#[must_use]
pub const fn file_new() -> CommandText {
    CommandText::new(
        "New",
        "Make a new document: one blank A4 page (Ctrl+N). It replaces what is open. Use Save a \
         copy to keep it; it is asked where to write every time, so the document itself stays \
         untitled.",
    )
}

/// `file.new_from_template`
///
/// # ★ The label follows `RIBBON_IA.md` and the tooltip corrects for it
///
/// §5.1 specifies the row as `New from template… (page size)`, following
/// Inkscape's `Ctrl+Alt+N`. What this shell offers is page sizes and not a
/// template gallery, so the word "template" over-promises — and the IA is
/// settled and reviewed, so a session may propose an amendment and may not
/// make one.
///
/// The tooltip is therefore doing real work rather than restating the label:
/// it says **page size** in its first four words, so an operator hovering
/// before they click learns what the window offers without opening it. See
/// `crate::dialogs::new_document`'s header for the full argument.
#[must_use]
pub const fn file_new_from_template() -> CommandText {
    CommandText::new(
        "New from template…",
        "Choose a page size and make a new document: A0 to A6, Letter, Legal, Tabloid, the ANSI \
         engineering sizes, or a size you type. It replaces what is open.",
    )
}

/// `file.close`
///
/// ★★ **The tooltip that was a promise nothing kept, from the day it shipped
/// until 2026-08-19.** *"You are asked what to do about unsaved edits first"* —
/// and nothing asked. `Action::Close` consulted `save_pending`, which is
/// permanently `false` by design, and then dropped the `EditSession`. Every
/// edit made since the file was opened went with it, silently, with no prompt
/// and no undo.
///
/// The sentence is **unchanged**, because it was never wrong about what pdfce
/// should do — it was a specification sitting on the ribbon, and the build had
/// not met it. `crate::dialogs::unsaved` is the surface that now does.
///
/// The generalisable half is worth keeping here, where the next tooltip gets
/// written: **an operator-visible string that describes behaviour is a claim,
/// and nothing in this project checks a claim of that shape.** The ui-strings
/// gate asserts the string lives in `text/`; the catalog tests assert it is a
/// sentence and that no two labels collide; no gate can ask whether it is
/// *true*. This one was found by an outside audit, three weeks after the fact,
/// by someone reading the tooltip and then reading the code.
#[must_use]
pub const fn file_close() -> CommandText {
    CommandText::new(
        "Close",
        "Close this document (Ctrl+W). You are asked what to do about unsaved edits \
         first. Your other open documents stay open.",
    )
}

/// `file.recent`
///
/// **The control this text belongs to is not a button.** `file.recent` is
/// drawn by the `recent_files` custom item in File ▸ File — a menu of the
/// documents the operator had open — so this label and tooltip are what the
/// *menu button* says, and the rows inside it are file names from
/// [`crate::text::files`]. See `crate::shell::manifest::CUSTOM_BACKED`.
///
/// The tooltip names the two behaviours an operator would otherwise have to
/// discover: the cap, and the fact that a document on a drive which is not
/// connected right now is hidden rather than forgotten.
#[must_use]
pub const fn file_recent() -> CommandText {
    CommandText::new(
        "Recent",
        "Open one of the last ten documents you had open. A document stored on a drive that \
         is not connected right now is hidden from the list until it comes back; it is not \
         forgotten.",
    )
}

/// `file.save`
///
/// ★★★ **Save. In place. Added 2026-08-20, on the operator:** *"can I please
/// have a save button like every other program in existence has? We're on week
/// two of this and just have a save as button."*
///
/// # The argument that used to stand here, and why it does not
///
/// [`file_save_copy`]'s doc comment said, and still says of itself:
///
/// > *"A button labelled `Save` would promise in-place saving, which cannot
/// > ship before autosave and crash recovery exist."*
///
/// That was a real position rather than an oversight, and it is **weaker than
/// it looks, for a reason specific to this application**: pdfce writes an
/// INCREMENTAL UPDATE. The new revision is appended; the previous one stays in
/// the file, byte for byte, reachable through its own cross-reference table.
/// An in-place save here does not overwrite the operator's document in the
/// sense the objection assumed — **the format is the crash recovery**, and it
/// was already shipping.
///
/// What remained genuinely unsafe was the WRITE, not the save: `fs::write`
/// truncates and then streams, so a crash mid-write leaves a partial file where
/// a whole one was. That is a solved problem, and `save::save_in_place` solves
/// it — materialise the replacement in a temporary beside the target, then
/// rename, which either happens or does not.
///
/// So the honest account is not *"the operator overruled a safety rule"*. It is
/// that the rule was aimed at the wrong hazard, and the right hazard has a
/// three-line answer that had not been written because nobody was asking the
/// question. That is the same shape as `Ctrl+P` never being bound.
///
/// # The description names the incremental behaviour on purpose
///
/// Because an operator who has been told for a fortnight that pdfce *never*
/// overwrites deserves to know exactly what changed, and because "the previous
/// version stays inside the file" is the fact that makes pressing this button
/// comfortable.
#[must_use]
pub const fn file_save() -> CommandText {
    CommandText::new(
        "Save",
        "Save this document over the file you opened (Ctrl+S). The edits are appended as an \
         update, so the previous version stays inside the file and nothing is thrown away. Use \
         Save a copy to write somewhere else instead.",
    )
}

/// `file.save_copy`
///
/// The label is `Save a copy…`, not `Save`, and that is load-bearing:
/// pdfce writes the edits as an incremental update to a file you name, and
/// never overwrites the original unless you pick it. A button labelled
/// `Save` would promise in-place saving, which cannot ship before autosave
/// and crash recovery exist.
#[must_use]
pub const fn file_save_copy() -> CommandText {
    CommandText::new(
        "Save a copy…",
        "Write the document, including unsaved edits, to a file you choose (Ctrl+Shift+S). The \
         original is never overwritten unless you pick it, and the edits are appended as an \
         update so the previous version stays intact inside the file.",
    )
}

/// `file.export_dxf`
#[must_use]
pub const fn file_export_dxf() -> CommandText {
    CommandText::new(
        "Export DXF…",
        "Write this page's lines, curves and text out as a DXF file that CAD and CNC software \
         can open.",
    )
}

/// `file.export_form_data`
#[must_use]
pub const fn file_export_form_data() -> CommandText {
    CommandText::new(
        "Export form data…",
        "Write this document's filled form values out as FDF, XFDF or CSV.",
    )
}

/// `file.copy_page_text`
///
/// ★ **Was `edit_copy_page_text`, in the EDIT TAB section, until 2026-08-14.**
/// The command moved to File ▸ Export by operator decision — copying is not
/// authoring, and the File tab is the one tab every mode shows — so this
/// catalog entry moved with it, because this file is ordered by tab and a
/// command's text sitting under the wrong heading is how the next reader
/// concludes the command is somewhere it is not.
///
/// **The wording is unchanged, deliberately.** Nothing about what the command
/// does has moved, and a tooltip rewritten during a re-parenting is a change
/// nobody asked for arriving inside one they did. The chord it names is still
/// `Ctrl+Shift+C`, still bound in `crate::shell::manifest`'s keymap — now to
/// this id — and the sentence about inferred word and line breaks is still the
/// thing an operator cannot guess: a PDF is under no obligation to record where
/// a word ends, so pdfce infers it from letter positions and says how much of
/// the copy was inferred.
#[must_use]
pub const fn file_copy_page_text() -> CommandText {
    CommandText::new(
        "Copy page text",
        "Copy this page's text to the clipboard (Ctrl+Shift+C). Where a PDF does not say where \
         words and lines end, pdfce works it out from the position of the letters, and says \
         how much of the copy that was.",
    )
}

/// `file.copy_document_text`
///
/// Was `edit_copy_document_text`; see [`file_copy_page_text`] for the move.
/// Wording unchanged, including the warning about the window not responding,
/// which is the honest description of a synchronous extraction over every page
/// and is the kind of sentence a re-parenting must not quietly lose.
#[must_use]
pub const fn file_copy_document_text() -> CommandText {
    CommandText::new(
        "Copy document text",
        "Copy every page's text to the clipboard. On a long document this can take a few \
         seconds, during which the window will not respond.",
    )
}

/// `file.print`
#[must_use]
pub const fn file_print() -> CommandText {
    CommandText::new(
        "Print…",
        "Set up and print this document. Nothing prints until you press Print in the dialog.",
    )
}

/// `file.properties`
#[must_use]
pub const fn file_properties() -> CommandText {
    CommandText::new(
        "Properties",
        "The document's own title, author, subject and keywords, and the properties of \
         whatever is selected on the page.",
    )
}

/// `file.fonts`
///
/// Moved here from View ▸ Panels. The Fonts panel answers "what is inside
/// this file", not "what is on my screen", so it belongs beside Properties
/// as document-level inspection.
#[must_use]
pub const fn file_fonts() -> CommandText {
    CommandText::new(
        "Fonts",
        "Show every font this document declares — type, encoding, embedded size, and whether \
         its embedded program could be removed.",
    )
}

/// `file.settings`
#[must_use]
pub const fn file_settings() -> CommandText {
    CommandText::new(
        "Settings…",
        "Choose how pdfce reads and writes documents where the PDF standard leaves the answer \
         open — colour, printing separations, text extraction. Your choices are kept in a file \
         beside the program and survive restarts.",
    )
}

/// `file.shortcuts`
#[must_use]
pub const fn file_shortcuts() -> CommandText {
    CommandText::new("Keyboard shortcuts", "Show every keyboard shortcut.")
}

/// `file.about`
///
/// ★ **No ellipsis, deliberately.** This catalog's `…` means *you will be
/// asked something before anything happens* — the reading `view_reset_layout`
/// had its ellipsis taken away for getting wrong. About asks nothing; it
/// shows. Its neighbour `file_shortcuts` is the same kind of window and is
/// spelled the same way, and all three reference applications agree: Acrobat,
/// Inkscape and SolidWorks all write "About <product>" plain.
///
/// The tooltip names **all three** things the window carries rather than just
/// the version, because the version is the least of them. The reason this
/// command exists is the attribution surface — see [`crate::text::about`] —
/// and an operator looking for licence terms has to be able to tell from the
/// hover that this is where they live.
#[must_use]
pub const fn file_about() -> CommandText {
    CommandText::new(
        "About pdfce",
        "Show this build's version, pdfce's own licence, and the third-party material included \
         in the program.",
    )
}

/// `file.ocr`
///
/// ★ **The tooltip states the uncertainty, and that is not optional here.**
/// OCR is the single largest inference pdfce makes — `pdfce-core`'s own
/// `ocr::layer` header says *"every word here is a guess"* — and rule 4 asks
/// that an inherently uncertain inference say so rather than imply otherwise.
/// A hover is the first place an operator meets this command, and a tooltip
/// that described only the benefit would be the sentence they remember.
///
/// It also states what does **not** change, because that is the question a
/// scanned document raises: nothing visible is added and the image is never
/// re-encoded, so a scan that is the record of something stays exactly the
/// bytes it was. That is `ocr::layer`'s own guarantee rather than a claim this
/// catalog is making on its behalf.
///
/// The dialog's fuller disclosure lives in [`crate::text::ocr`]; this is the
/// one-line version, and the two must not drift apart in what they promise.
#[must_use]
pub const fn file_ocr() -> CommandText {
    CommandText::new(
        "Recognise text…",
        "Read the words in a scanned page and add them as invisible text behind the image, so \
         Find and copy work. Every word is a guess and this recogniser scores none of them, so \
         you are shown what it read before anything is saved. The page still looks the same and \
         the scan is never re-encoded.",
    )
}

// ===========================================================================
// PAGES TAB
//
// Every command here operates on THIS document's page set and respects the
// thumbnail rail's selection when there is one. That is the tab's
// organising rule and it is what distinguishes it from Tools, which
// produces new files. The tooltips say so where the distinction is easy to
// get wrong — `pages.merge_into` against `tools.merge_files` especially.
// ===========================================================================

/// `pages.insert_from_file`
#[must_use]
pub const fn pages_insert_from_file() -> CommandText {
    CommandText::new(
        "Insert from file…",
        "Insert the pages of another PDF into this document, before or after the page you have \
         selected.",
    )
}

/// `pages.delete`
///
/// **`Delete pages`, not `Delete`.** `RIBBON_IA.md` §5.3 writes the row as
/// `Delete`, which is unambiguous *in its band* — it sits under a tab
/// called Pages, in a group called Organise, beside Extract and Move.
/// It is not unambiguous against the contextual Format tab's `Delete`,
/// which removes the selected object and can appear over any tab at any
/// time. Two controls reading `Delete`, one of which removes a sheet from
/// a drawing set, is a collision worth two extra characters.
#[must_use]
pub const fn pages_delete() -> CommandText {
    CommandText::new(
        "Delete pages",
        "Remove the selected pages from this document. Undo reverses it.",
    )
}

/// `pages.extract`
#[must_use]
pub const fn pages_extract() -> CommandText {
    CommandText::new(
        "Extract…",
        "Write the selected pages out as a new PDF. This document is left unchanged.",
    )
}

/// `pages.move_up`
#[must_use]
pub const fn pages_move_up() -> CommandText {
    CommandText::new(
        "Move up",
        "Move the selected pages one place earlier in the document (Alt+Up).",
    )
}

/// `pages.move_down`
#[must_use]
pub const fn pages_move_down() -> CommandText {
    CommandText::new(
        "Move down",
        "Move the selected pages one place later in the document (Alt+Down).",
    )
}

/// `pages.split`
#[must_use]
pub const fn pages_split() -> CommandText {
    CommandText::new(
        "Split…",
        "Split this document into several files at page boundaries you choose.",
    )
}

/// `pages.merge_into`
#[must_use]
pub const fn pages_merge_into() -> CommandText {
    CommandText::new(
        "Merge into this document…",
        "Add the pages of one or more other PDFs to this document. To combine files into a new \
         one instead, leaving this document alone, use Tools > Merge files.",
    )
}

/// `pages.rotate_left`
#[must_use]
pub const fn pages_rotate_left() -> CommandText {
    CommandText::new(
        "Rotate left",
        "Turn the selected pages 90° counter-clockwise ([). This changes the document, not \
         just the view, and is saved with it — use Undo to reverse it.",
    )
}

/// `pages.rotate_right`
#[must_use]
pub const fn pages_rotate_right() -> CommandText {
    CommandText::new(
        "Rotate right",
        "Turn the selected pages 90° clockwise (]). This changes the document, not just the \
         view, and is saved with it — use Undo to reverse it.",
    )
}

// ===========================================================================
// EDIT TAB
// ===========================================================================

/// `edit.text`
#[must_use]
pub const fn edit_text() -> CommandText {
    CommandText::new(
        "Edit text",
        "Edit words already on this page — fix a typo, resize, or recolour existing text \
         (Ctrl+E). To add brand-new page text instead, use Add text.",
    )
}

/// `edit.add_text`
#[must_use]
pub const fn edit_add_text() -> CommandText {
    CommandText::new(
        "Add text",
        "Add new text to the page itself — a label, caption or note that becomes real, \
         permanent page content, exactly like the text already here (Ctrl+Shift+E). For a \
         removable comment instead, use Markup > Text box.",
    )
}

/// `edit.objects`
#[must_use]
pub const fn edit_objects() -> CommandText {
    CommandText::new(
        "Edit objects",
        "Edit vector drawing objects on the page: click to select, drag to move the object, \
         drag an anchor to move that node, or press Delete to remove it. Delete removes a \
         drawing object — it is not redaction, and does not securely remove covered content.",
    )
}

/// `edit.insert_image`
#[must_use]
pub const fn edit_insert_image() -> CommandText {
    CommandText::new("Image…", "Place an image file on this page.")
}

/// **Text field** — the box an operator types into.
#[must_use]
pub const fn edit_form_text_field() -> CommandText {
    CommandText::new(
        "Text field",
        "A box to type into. Click where you want it, or drag out the exact size.",
    )
}

/// **Check box** — one independent on/off box.
#[must_use]
pub const fn edit_form_check_box() -> CommandText {
    CommandText::new(
        "Check box",
        "A single box that is either ticked or not. Click where you want it, or drag out the exact size.",
    )
}

/// **Radio button** — one of a group.
///
/// ★ The tooltip names the grouping rule, because it is the only one of the
/// five whose behaviour depends on another field: two radios sharing a name are
/// one control. An operator who does not know that places two buttons that both
/// stay on and reasonably calls it a bug.
#[must_use]
pub const fn edit_form_radio_button() -> CommandText {
    CommandText::new(
        "Radio button",
        "One of a set, where choosing one clears the others. Give them the same group name to make them alternatives.",
    )
}

/// **Choice** — a drop-down or list.
#[must_use]
pub const fn edit_form_choice() -> CommandText {
    CommandText::new(
        "Drop-down",
        "A list of options to choose from. Click where you want it, or drag out the exact size.",
    )
}

/// **Push button** — authorable, inert, and greyed until pdfce can run actions.
#[must_use]
pub const fn edit_form_push_button() -> CommandText {
    CommandText::new("Button", "A button that runs an action when pressed.")
}

/// Why the push button is greyed.
///
/// ★★★ R9 permits greying only for a **temporarily** unavailable capability
/// that is **always explained on hover**, and this is the explanation. It draws
/// the distinction that matters: pdfce can *place* a button perfectly well —
/// what it cannot do is *run* what the button would do, because it executes no
/// PDF actions. Placing one would give the operator a control that looks
/// finished and does nothing, which is worse than not offering it.
///
/// ★ It says what is missing rather than apologising, so an operator can judge
/// whether it matters to them and can ask for it if it does.
#[must_use]
pub const fn edit_form_push_button_unavailable() -> &'static str {
    "pdfce can place a button but cannot yet run what a button does, so one placed now would do nothing when pressed."
}

pub const fn edit_form_create_field() -> CommandText {
    CommandText::new(
        "Create field",
        "Add a new form field to the page. Click where you want it, or drag out the exact size.",
    )
}

/// `edit.form_manage_fields`
#[must_use]
pub const fn edit_form_manage_fields() -> CommandText {
    CommandText::new(
        "Manage fields",
        "List every form field in this document, and rename, retype or remove them.",
    )
}

/// `edit.form_flatten`
#[must_use]
pub const fn edit_form_flatten() -> CommandText {
    CommandText::new(
        "Flatten",
        "Turn the filled values into ordinary page content, so they draw everywhere but can no \
         longer be edited as fields.",
    )
}

/// `edit.find`
///
/// ★ **The one command in this catalog whose only control is on the status
/// bar.** `RIBBON_IA.md` §6 puts the Find toggle there rather than on the
/// ribbon, so this label and tooltip are what that toggle's *command* says —
/// reachable from a keymap, from a customized quick-access toolbar, and from
/// the shortcut list — while `crate::text::find` holds the copy the bar's own
/// controls own. The two are not duplicates: this one is keyed by command id
/// and consumed by `crate::shell::commands`, that one is keyed by control and
/// consumed by a widget.
///
/// The tooltip names `Ctrl+F` because the chord genuinely works: the manifest
/// keymap binds it AND `crate::app::keyboard::parse_chord` can spell it. Both
/// halves are required — `Ctrl+O` was in the keymap and printed in a tooltip
/// for the whole of the ribbon's first life while pressing it did nothing,
/// because the spelling table held only digits.
///
/// It also names the two limits an operator has no way to guess and that
/// account for almost every surprising empty result: the search is over the
/// text **drawn on the pages**, and it matches within one text run at a time.
#[must_use]
pub const fn edit_find() -> CommandText {
    CommandText::new(
        "Find",
        "Search the text drawn on this document's pages, and highlight every hit (Ctrl+F). Form fields, comments, bookmarks and attachments are not searched, and a word the producer split across two text runs is not found.",
    )
}

/// `edit.redact`
#[must_use]
pub const fn edit_redact() -> CommandText {
    CommandText::new(
        "Redact",
        "Mark what is to be permanently removed — a whole page, every occurrence of some text, \
         or everything matching a pattern. Marking is reversible; applying is not.",
    )
}

/// `edit.redact_apply`
#[must_use]
pub const fn edit_redact_apply() -> CommandText {
    CommandText::new(
        "Apply redactions",
        "Permanently remove everything the redaction marks cover. This cannot be undone.",
    )
}

/// `edit.undo`
///
/// # ★ Why this does NOT name the operation, and what it would take to
///
/// `SALVAGE.md` records the old shell as having *"undo tooltips naming the
/// specific operation"*, and the engine still supplies everything needed for
/// one: `EditSession::undo_kind` answers *what would be undone* without
/// undoing it, over 44 `CommandKind` variants. Writing *"Undo add annotation
/// (Ctrl+Z)"* is therefore catalog work and nothing more — a
/// `CommandKind → &'static str` mapping in this file, with a fallback for the
/// kinds this shell cannot author.
///
/// **The blocker is the registry, not the catalog.** `egui_shell`'s
/// `Command::tooltip` is a `String` fixed at registration;
/// `CommandRegistry` exposes `get`, `iter` and `register` and **no mutable
/// accessor and no removal**, and `PdfceApp::commands` is built once in
/// `PdfceApp::new` and handed to the ribbon by shared reference every frame.
/// So a tooltip that changes with the log needs one of two things:
///
/// 1. a `get_mut` (or a `tooltip` closure) on `CommandRegistry` — which is a
///    change to `crates/egui-shell`, the crate `check-shell-purity.sh` keeps
///    application-agnostic and which this work is not permitted to touch; or
/// 2. rebuilding the whole 101-command registry every frame so one string can
///    differ — which pays a hundred allocations a frame for one tooltip, and
///    changes the **accessible name** of an icon-only control under the
///    operator's pointer, since `egui_shell::ribbon::a11y` promotes the
///    tooltip to the name when there is no visible label.
///
/// Half-doing it — naming the operation in the status bar instead, say — would
/// put the answer somewhere the operator is not looking when they hover the
/// control that asks the question. So the plain label ships, deliberately, and
/// the operation *is* named on the diagnostic channel (`undo kind=…`), which is
/// where it is currently readable. The right fix is (1), by whoever next has
/// cause to open `egui-shell`'s command registry.
///
/// The chord is still printed, and that is the part P3 actually requires: this
/// command is greyed whenever the log is empty, and a greyed control must
/// explain itself on hover. `egui_shell::ribbon::qat` uses
/// `on_disabled_hover_text`, so this sentence is read in exactly the state it
/// most needs to be.
#[must_use]
pub const fn edit_undo() -> CommandText {
    CommandText::new("Undo", "Undo the last change (Ctrl+Z).")
}

/// `edit.redo`
#[must_use]
pub const fn edit_redo() -> CommandText {
    CommandText::new(
        "Redo",
        "Redo the change you just undid (Ctrl+Y or Ctrl+Shift+Z).",
    )
}

// ===========================================================================
// MARKUP AND MEASURE — in `annotate`
//
// ★ **Moved out on 2026-08-14 under R2**, at the seam that module's header
// argues for: these two tabs are what an operator *adds on top of* the page,
// which is the line `app::modes::Capabilities` already draws between
// `edit_content` and the two authoring flags, and the line `shell::manifest`
// already draws by keeping `markup.rs` and `measure.rs` as files of their own.
//
// Re-exported by name — not by glob — so every call site still writes
// `t::markup_rectangle()` and nothing outside `text/` learns the catalog was
// split, while a function added over there still has to be named here to reach
// the crate. The catalog's discipline is that every operator-visible string is
// named somewhere a reviewer looks.
// ===========================================================================
pub mod annotate;

pub use annotate::{
    markup_arrow, markup_cloud, markup_comments, markup_ellipse, markup_finish, markup_highlight,
    markup_ink, markup_polygon, markup_polyline, markup_rectangle, markup_squiggly, markup_stamp,
    markup_sticky_note, markup_strikeout, markup_text_box, markup_underline, measure_finish,
    measure_length, measure_linear, measure_manage_groups, measure_perimeter,
    measure_radius_diameter, measure_set_scale, measure_two_line,
};

// ===========================================================================
// TOOLS TAB
// ===========================================================================

/// `tools.merge_files`
#[must_use]
pub const fn tools_merge_files() -> CommandText {
    CommandText::new(
        "Merge files…",
        "Combine several PDFs into one new file. This document is not changed — to add pages \
         to it instead, use Pages > Merge into this document.",
    )
}

/// `tools.split_files`
#[must_use]
pub const fn tools_split_files() -> CommandText {
    CommandText::new(
        "Split files…",
        "Split one or more PDFs into separate files. The originals are not changed.",
    )
}

/// `tools.font_folders`
#[must_use]
pub const fn tools_font_folders() -> CommandText {
    CommandText::new(
        "Font folders…",
        "Point pdfce at folders of your own font files (.ttf/.otf) so it can draw a document's \
         missing text with the real typeface instead of a bundled substitute. This changes how \
         missing fonts look, not where text sits on the page.",
    )
}

/// `tools.embed_fonts`
#[must_use]
pub const fn tools_embed_fonts() -> CommandText {
    CommandText::new(
        "Embed fonts",
        "Copy the font programs this document relies on into the file itself, so it draws the \
         same on a machine that does not have them.",
    )
}

/// `tools.unembed_fonts`
#[must_use]
pub const fn tools_unembed_fonts() -> CommandText {
    CommandText::new(
        "Unembed fonts",
        "Remove embedded font programs from the file. The document gets smaller and starts \
         depending on the reader having those fonts.",
    )
}

/// `tools.render_diagnostics`
#[must_use]
pub const fn tools_render_diagnostics() -> CommandText {
    CommandText::new(
        "Render diagnostics",
        "Show what the renderer did with the last page — how long it took, at what raster \
         size, and anything it could not draw.",
    )
}

// ===========================================================================
// FORMAT TAB (contextual)
// ===========================================================================

/// `format.delete`
#[must_use]
pub const fn format_delete() -> CommandText {
    CommandText::new(
        "Delete",
        "Remove what is selected from the page. Undo reverses it.",
    )
}

/// `format.properties`
///
/// ★ **A second route to `file.properties`, not a second implementation of
/// it.** Its dispatch arm raises `Action::Command("file.properties")`, which is
/// the mechanism that exists so exactly this cannot become two ways of opening
/// one panel with two sets of guards — the Find bar's OCR offer is the
/// precedent.
///
/// It is registered as its own id rather than listing `file.properties` twice
/// because the shell enforces **one command, one tab**, and the two placements
/// answer different questions: File ▸ Document is *"tell me about this file"*
/// and Format is *"tell me about the thing I just clicked"*.
///
/// The tooltip names the ce dimension case explicitly. That is the capability
/// the panel gained on 2026-08-18 — the style cascade, the tolerance and the
/// radius/diameter switch — and it is the one an operator has no other way to
/// discover, because a selected ce dimension looks exactly like an unselected
/// one apart from its outline.
#[must_use]
pub const fn format_properties() -> CommandText {
    CommandText::new(
        "Properties",
        "Show the Properties panel for what is selected — for a dimension, its \
         group, what it measured, and every setting it inherits from its group \
         or overrides for itself.",
    )
}

// ===========================================================================
// MODES
//
// Not ribbon commands: these are the three positions of the selector at the
// far right of the tab row, reachable from the keymap. They are registered
// commands because a key binding resolves against the registry, and because
// the mode selector is a control like any other.
//
// Each tooltip states the rule that makes the feature safe — a mode changes
// what is VISIBLE and never makes a visible control silently inert. That
// distinction is the whole difference between this and the `editing_enabled`
// master toggle it replaces.
// ===========================================================================

/// `mode.read`
#[must_use]
pub const fn mode_read() -> CommandText {
    CommandText::new(
        "Read",
        "Show only what a reader needs: File and View (Ctrl+1). Nothing is hidden from the \
         document — only from the interface — and your edits are untouched.",
    )
}

/// `mode.review`
#[must_use]
pub const fn mode_review() -> CommandText {
    CommandText::new(
        "Review",
        "Add the Pages, Markup and Measure tabs (Ctrl+2) — comment on a drawing, measure it, \
         and reorganise the sheets, without the content-editing tools.",
    )
}

/// `mode.edit`
#[must_use]
pub const fn mode_edit() -> CommandText {
    CommandText::new("Edit", "Show every tab (Ctrl+3).")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every command in this catalog, so the rules below are checked
    /// against all of them rather than against whichever ones somebody
    /// remembered to list.
    ///
    /// Maintained by hand, and that is the point: adding a command means
    /// adding a line here, and the count assertion in
    /// `crate::shell::commands` cross-checks this list against the
    /// registry, so a command that is registered but never appears here
    /// fails a test rather than shipping with unreviewed copy.
    fn all() -> Vec<CommandText> {
        vec![
            file_new(),
            file_open(),
            file_close(),
            file_recent(),
            file_save_copy(),
            file_export_dxf(),
            file_export_form_data(),
            // Moved from the Edit block below on 2026-08-14 with the commands
            // themselves; this list is in tab order for the same reason the
            // catalog is.
            file_copy_page_text(),
            file_copy_document_text(),
            file_print(),
            file_properties(),
            file_fonts(),
            file_settings(),
            file_shortcuts(),
            file_about(),
            file_ocr(),
            view_page_single(),
            view_page_continuous(),
            view_page_facing(),
            view_page_facing_continuous(),
            view_zoom_actual(),
            view_zoom_fit_page(),
            view_zoom_fit_width(),
            view_zoom_fit_height(),
            view_show_annotations(),
            view_show_points(),
            view_rulers(),
            view_grid(),
            view_guides(),
            view_sidebar(),
            view_panel_pages(),
            view_panel_bookmarks(),
            view_panel_layers(),
            view_panel_signatures(),
            view_panel_objects(),
            view_panel_forms(),
            view_read_mode(),
            view_fullscreen(),
            view_next_document(),
            view_previous_document(),
            view_close_other_documents(),
            view_reset_layout(),
            pages_insert_from_file(),
            pages_delete(),
            pages_extract(),
            pages_move_up(),
            pages_move_down(),
            pages_split(),
            pages_merge_into(),
            pages_rotate_left(),
            pages_rotate_right(),
            edit_text(),
            edit_add_text(),
            edit_objects(),
            edit_insert_image(),
            edit_form_create_field(),
            edit_form_manage_fields(),
            edit_form_flatten(),
            edit_redact(),
            edit_redact_apply(),
            edit_undo(),
            edit_redo(),
            markup_rectangle(),
            markup_ellipse(),
            markup_arrow(),
            markup_polyline(),
            markup_polygon(),
            markup_cloud(),
            markup_ink(),
            markup_finish(),
            markup_highlight(),
            markup_text_box(),
            markup_sticky_note(),
            markup_stamp(),
            markup_comments(),
            measure_linear(),
            measure_length(),
            measure_perimeter(),
            measure_radius_diameter(),
            // `measure_two_line` was registered on 2026-08-14 and was not
            // added here, so for one day the label-uniqueness and
            // tooltip-is-a-sentence rules were being asserted over a list that
            // did not contain it. Both new Measure entries are here.
            measure_two_line(),
            measure_finish(),
            measure_set_scale(),
            measure_manage_groups(),
            tools_merge_files(),
            tools_split_files(),
            tools_font_folders(),
            tools_embed_fonts(),
            tools_unembed_fonts(),
            tools_render_diagnostics(),
            format_delete(),
            mode_read(),
            mode_review(),
            mode_edit(),
        ]
    }

    /// **Every command has a non-empty label and a non-empty tooltip.**
    ///
    /// P3 reserves greying for temporarily unavailable and requires that
    /// it always be explained on hover. A command with no tooltip cannot
    /// honour that, and the salvage source shipped four such controls on
    /// the Measure tab.
    #[test]
    fn every_command_has_a_label_and_a_tooltip() {
        for t in all() {
            assert!(!t.label.trim().is_empty(), "empty label: {t:?}");
            assert!(!t.tooltip.trim().is_empty(), "empty tooltip: {t:?}");
        }
    }

    /// **No two commands share a label.**
    ///
    /// The defect this prevents shipped: `edit_text_tool_button()` and
    /// `add_text_tool_button()` both returned the literal `"Aa"`, and the
    /// two adjacent buttons in the Content group were distinguishable only
    /// by icon and tooltip. Two identical labels side by side is not a
    /// style problem; it is two controls the operator cannot tell apart.
    ///
    /// The check is deliberately global rather than per-group. A label
    /// duplicated across two tabs is less confusing than one duplicated
    /// within a group, but it is still a search result with two answers,
    /// and the moment customization lets an operator move a command
    /// between tabs the per-group version of this rule stops holding.
    #[test]
    fn no_two_commands_share_a_label() {
        let mut labels: Vec<&str> = all().iter().map(|t| t.label).collect();
        let total = labels.len();
        labels.sort_unstable();
        labels.dedup();
        assert_eq!(
            labels.len(),
            total,
            "two commands share a label — an operator cannot tell them apart"
        );
    }

    /// A tooltip is a sentence: it ends in punctuation.
    ///
    /// A label is a name and takes no trailing period; a tooltip is prose
    /// and does. Stated as a rule in [`crate::text`] and worth checking,
    /// because the two conventions sit two lines apart in this file and
    /// the wrong one is easy to copy.
    #[test]
    fn tooltips_are_sentences_and_labels_are_not() {
        for t in all() {
            assert!(
                t.tooltip.ends_with('.'),
                "a tooltip is prose and ends in a full stop: {:?}",
                t.tooltip
            );
            assert!(
                !t.label.ends_with('.'),
                "a label is a name and takes no trailing period: {:?}",
                t.label
            );
        }
    }

    /// **The three illegible labels are gone.**
    ///
    /// `RIBBON_IA.md` §5.4 requires `Aa`, `I⁺ Aa` and `Obj` to become
    /// real words. This asserts the outcome rather than trusting that
    /// nobody copies the old literals back in — `Obj` is not a word, and
    /// it was the label on one of the three primary editing tools.
    #[test]
    fn the_content_tools_have_real_labels() {
        assert_eq!(edit_text().label, "Edit text");
        assert_eq!(edit_add_text().label, "Add text");
        assert_eq!(edit_objects().label, "Edit objects");
    }
}
