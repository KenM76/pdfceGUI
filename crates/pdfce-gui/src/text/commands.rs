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

// ===========================================================================
// FILE TAB
// ===========================================================================

/// `file.open`
#[must_use]
pub const fn file_open() -> CommandText {
    CommandText::new("Open…", "Open a PDF document (Ctrl+O).")
}

/// `file.close`
#[must_use]
pub const fn file_close() -> CommandText {
    CommandText::new(
        "Close",
        "Close this document. You are asked what to do about unsaved edits first.",
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
        "Write the document, including unsaved edits, to a file you choose (Ctrl+S). The \
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

// ===========================================================================
// VIEW TAB
// ===========================================================================

/// `view.page_single`
#[must_use]
pub const fn view_page_single() -> CommandText {
    CommandText::new(
        "Single page",
        "Show one page at a time. This is pdfce's default, because paging one drawing sheet at \
         a time is the right model for reading a sheet set.",
    )
}

/// `view.page_continuous`
///
/// The operator's instruction of 2026-08-12 is what this control is, and the
/// tooltip carries its reasoning rather than a feature description: *"continuous
/// scroll should be an option under the view tab as the way I move around a
/// page is great when working with drafting drawings."* So the words say what
/// it is **for** — a document you read through — and leave single page
/// standing as the right answer for a sheet set, which it is.
///
/// The second sentence states the per-document persistence, because it is
/// behaviour the operator cannot see until it surprises them: choosing this on
/// a report and then opening a drawing set must not carry the setting across,
/// and a control that silently remembers something should say so.
#[must_use]
pub const fn view_page_continuous() -> CommandText {
    CommandText::new(
        "Continuous",
        "Scroll through every page in one run, for a document you read rather than a sheet set \
         you page through. pdfce remembers this choice for this document, so another file keeps \
         its own.",
    )
}

/// `view.page_facing`
#[must_use]
pub const fn view_page_facing() -> CommandText {
    CommandText::new(
        "Facing",
        "Show two pages side by side, as an open book. The first page sits alone, so every \
         later spread pairs the way a bound document does.",
    )
}

/// `view.page_facing_continuous`
#[must_use]
pub const fn view_page_facing_continuous() -> CommandText {
    CommandText::new(
        "Facing continuous",
        "Scroll through every spread in one run — facing pages, without stopping at each one.",
    )
}

/// `view.panel_pages`
///
/// The one panel toggle whose tooltip has to say what the panel is *for*
/// rather than what it contains: a grid of thumbnails is self-explanatory to
/// look at and not to read about, so the sentence spends its words on the two
/// verbs — go to a page, and act on several at once — that are not obvious
/// from the picture.
#[must_use]
pub const fn view_panel_pages() -> CommandText {
    CommandText::new(
        "Pages",
        "Show or hide the panel of page thumbnails: click one to go there, and pick several to \
         act on them together.",
    )
}

/// `view.render_strategy`
///
/// The operator decision of 2026-08-12, in one control. Measured on a
/// large drawing, pdfce's whole-page raster is *smoother* to pan and zoom
/// than progressive tiling — no seams, no piece-by-piece fill-in — at the
/// cost of a full re-raster once motion stops. Those are two legitimate
/// trades, not a better and a worse, so the tooltip states the trade
/// rather than recommending one.
#[must_use]
pub const fn view_render_strategy() -> CommandText {
    CommandText::new(
        "Strategy",
        "Whether pdfce rasterises the whole page at once and scales that while you move, or \
         fills the page in as progressive tiles. Whole page is smoother to pan and re-rasters \
         once you stop; tiles show detail sooner on a very large sheet.",
    )
}

/// `view.render_quality`
#[must_use]
pub const fn view_render_quality() -> CommandText {
    CommandText::new(
        "Raster scale",
        "How many pixels pdfce rasterises for each screen pixel. Higher stays sharper while \
         you zoom, and costs memory and time on every re-raster.",
    )
}

/// `view.render_settle`
#[must_use]
pub const fn view_render_settle() -> CommandText {
    CommandText::new(
        "Settle delay",
        "How long pdfce waits after you stop moving before it rasterises the page again at \
         full quality.",
    )
}

/// `view.render_thin_lines`
#[must_use]
pub const fn view_render_thin_lines() -> CommandText {
    CommandText::new(
        "Thin lines",
        "Draw hairline strokes at least one pixel wide, so a line the drawing defines as \
         zero-width does not vanish when you zoom out.",
    )
}

/// `view.render_antialias`
#[must_use]
pub const fn view_render_antialias() -> CommandText {
    CommandText::new(
        "Antialias",
        "Whether text and vector edges are smoothed. Turning it off makes a dense drawing \
         crisper and a page of body text harder to read.",
    )
}

/// `view.zoom_actual`
#[must_use]
pub const fn view_zoom_actual() -> CommandText {
    CommandText::new(
        "Actual size",
        "Show the page at actual size — one PDF point per screen point (Ctrl+0).",
    )
}

/// `view.zoom_selection`
#[must_use]
pub const fn view_zoom_selection() -> CommandText {
    CommandText::new(
        "Zoom to selection",
        "Scale and centre the view on what is selected.",
    )
}

/// `view.zoom_region`
///
/// The tooltip says *"drag"* because arming this command does not zoom
/// anything — it changes what the next drag on the page means. A control
/// that arms rather than acts has to say so, or its first press reads as
/// broken.
#[must_use]
pub const fn view_zoom_region() -> CommandText {
    CommandText::new(
        "Zoom to region",
        "Drag a rectangle on the page to zoom to it. The selection is left alone.",
    )
}

/// `view.tool_hand`
#[must_use]
pub const fn view_tool_hand() -> CommandText {
    CommandText::new(
        "Hand",
        "Drag to pan the page instead of selecting. Hold Space to pan without switching tools.",
    )
}

/// `view.zoom_fit_page`
#[must_use]
pub const fn view_zoom_fit_page() -> CommandText {
    CommandText::new(
        "Fit page",
        "Scale the page so all of it is visible, and keep it fitted as the window resizes.",
    )
}

/// `view.zoom_fit_width`
#[must_use]
pub const fn view_zoom_fit_width() -> CommandText {
    CommandText::new(
        "Fit width",
        "Scale the page so its full width is visible, and keep it fitted as the window resizes.",
    )
}

/// `view.show_annotations`
#[must_use]
pub const fn view_show_annotations() -> CommandText {
    CommandText::new(
        "Annotations",
        "Show or hide the markup, stamps and form-field appearances stored in this document, \
         so the page content can be seen alone.",
    )
}

/// `view.show_points`
#[must_use]
pub const fn view_show_points() -> CommandText {
    CommandText::new(
        "Points",
        "Show the editable points of every part of the object you are working inside, not just \
         the part you have selected. Points always appear for the selected part.",
    )
}

/// `view.sidebar`
#[must_use]
pub const fn view_sidebar() -> CommandText {
    CommandText::new(
        "Sidebar",
        "Show or hide the left panel — page thumbnails and the active tool's options.",
    )
}

/// `view.panel_bookmarks`
#[must_use]
pub const fn view_panel_bookmarks() -> CommandText {
    CommandText::new(
        "Bookmarks",
        "Show the document's bookmarks. Click one to jump to its page.",
    )
}

/// `view.panel_layers`
///
/// ## ★ Reworded at S4, because the old tooltip undersold a capability
///
/// It read:
///
/// > Show the document's layers and which of them a reader draws by default.
///
/// which was a claim about the document with **no verb in it** — accurate for
/// the S3 panel, which was a report. S4 restored the visibility control
/// (`crate::app::actions::Action::SetLayerVisible`), and a tooltip that
/// describes a panel as read-only when it is not costs the operator the
/// capability: they read it, conclude there is nothing to click, and never
/// open the panel.
///
/// The new wording follows [`view_panel_bookmarks`]'s shape — *what it shows,
/// then what you can do in it* — because that is the shape of the only other
/// panel in this build with a verb, and two panels that answer the same
/// question should answer it the same way.
///
/// The third clause is not padding. It is the same boundary
/// `crate::text::panels::layers_session_only_note` states inside the panel,
/// and it is repeated here because the ribbon tooltip is read **before** the
/// panel opens — which is the moment an operator decides whether clicking
/// this is a safe thing to do to someone else's file.
#[must_use]
pub const fn view_panel_layers() -> CommandText {
    CommandText::new(
        "Layers",
        "Show the document's layers, and switch any of them on or off while you look at it. \
         The document is not changed.",
    )
}

/// `view.panel_signatures`
#[must_use]
pub const fn view_panel_signatures() -> CommandText {
    CommandText::new(
        "Signatures",
        "Show what each digital signature covers. pdfce does not check whether they are valid.",
    )
}

/// `view.panel_objects`
#[must_use]
pub const fn view_panel_objects() -> CommandText {
    CommandText::new(
        "Objects",
        "Show or hide the right-hand panel listing everything on the page, nested into parts \
         and points.",
    )
}

/// `view.read_mode`
#[must_use]
pub const fn view_read_mode() -> CommandText {
    CommandText::new(
        "Read mode",
        "Hide the ribbon and the panels and give the whole window to the page (Ctrl+H).",
    )
}

/// `view.fullscreen`
#[must_use]
pub const fn view_fullscreen() -> CommandText {
    CommandText::new("Full screen", "Fill the whole display with pdfce (F11).")
}

/// `view.floating_panels`
///
/// One half of the pair that retires `FEATURES.md`'s "nothing floats over
/// the canvas" invariant, per the operator decision of 2026-08-13. This
/// one governs what the **operator** may do; [`view_app_initiative`]
/// governs what **pdfce** may do unasked, and only the second carries the
/// original complaint.
#[must_use]
pub const fn view_floating_panels() -> CommandText {
    CommandText::new(
        "Floating panels",
        "Whether you may tear a panel out into a window of its own. Off keeps every panel \
         docked, which is how earlier builds behaved.",
    )
}

/// `view.app_initiative`
///
/// Default **Never**, which preserves the shipped behaviour the operator
/// asked for — no accept/reject box appearing over the drawing — while
/// making it a choice rather than a law.
#[must_use]
pub const fn view_app_initiative() -> CommandText {
    CommandText::new(
        "App initiative",
        "Whether pdfce may float a surface over the page on its own, without you having asked \
         for it — a tool's option box, a transient property bar, a notification. Never means \
         it does not, and is the default.",
    )
}

/// `view.reset_layout`
///
/// ★ **This entry lost an ellipsis and a promise, and both losses are the
/// same correction.** It used to read *"Reset layout…"* / *"Put the panels
/// back where they started. You choose which ones — the left panel, the
/// right panel, or just whether they are open."*
///
/// The choice is real and specified — `RIBBON_IA.md`: *"an operator who only
/// wanted the right dock back must not lose their left one"* — and
/// `egui_shell::layout::ResetScope` implements all three scopes. What does
/// not exist is anywhere to **ask**: this build has no modal, no popup and no
/// split-button item kind, so the command was wired to `ResetScope::All` (see
/// `crate::app::PdfceApp::dispatch_command`, whose arm records what a chooser
/// would take). A tooltip offering a choice the operator is never given is
/// exactly the "never state a capability the build does not have" failure
/// this catalog's header forbids, and the trailing `…` made the same promise
/// in punctuation.
///
/// **Restore both the moment the chooser lands** — the original wording is
/// quoted above so that is a copy rather than a rewrite.
#[must_use]
pub const fn view_reset_layout() -> CommandText {
    CommandText::new(
        "Reset layout",
        "Put both panel docks back where this mode started them. Your other modes keep the \
         arrangements you gave them.",
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
         one instead, leaving this document alone, use Tools ▸ Merge files.",
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
         removable comment instead, use Markup ▸ Text box.",
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

/// `edit.copy_page_text`
#[must_use]
pub const fn edit_copy_page_text() -> CommandText {
    CommandText::new(
        "Copy page text",
        "Copy this page's text to the clipboard (Ctrl+Shift+C). Where a PDF does not say where \
         words and lines end, pdfce works it out from the position of the letters, and says \
         how much of the copy that was.",
    )
}

/// `edit.copy_document_text`
#[must_use]
pub const fn edit_copy_document_text() -> CommandText {
    CommandText::new(
        "Copy document text",
        "Copy every page's text to the clipboard. On a long document this can take a few \
         seconds, during which the window will not respond.",
    )
}

/// `edit.form_fill`
#[must_use]
pub const fn edit_form_fill() -> CommandText {
    CommandText::new(
        "Fill form",
        "List this document's fillable fields and type into them. Nothing is written to disk \
         until you save.",
    )
}

/// `edit.form_create_field`
#[must_use]
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
/// keymap binds it AND `crate::app::keyboard::DERIVED` can spell it. Both
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
        "Search the text drawn on this document's pages, and highlight every hit (Ctrl+F).          Form fields, comments, bookmarks and attachments are not searched, and a word the          producer split across two text runs is not found.",
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
// MARKUP TAB
//
// The four shapes shared one tooltip in the salvage source — "Draw this
// shape on the page. Click the button, then drag on the page where you
// want it." Each gets its own here, because the gesture is not the same
// for all four: a highlight is dragged across words, an arrow from tail to
// head, a rectangle corner to corner.
// ===========================================================================

/// `markup.rectangle`
#[must_use]
pub const fn markup_rectangle() -> CommandText {
    CommandText::new(
        "Rectangle",
        "Draw a rectangle on the page. Click the button, then drag from one corner to the \
         other.",
    )
}

/// `markup.ellipse`
#[must_use]
pub const fn markup_ellipse() -> CommandText {
    CommandText::new(
        "Ellipse",
        "Draw an ellipse on the page. Click the button, then drag out the box it fits inside.",
    )
}

/// `markup.arrow`
#[must_use]
pub const fn markup_arrow() -> CommandText {
    CommandText::new(
        "Arrow",
        "Draw an arrow on the page. Click the button, then drag from the tail to the head.",
    )
}

/// `markup.highlight`
#[must_use]
pub const fn markup_highlight() -> CommandText {
    CommandText::new(
        "Highlight",
        "Draw a highlight band over the page. Click the button, then drag across what you want \
         marked.",
    )
}

/// `markup.text_box`
#[must_use]
pub const fn markup_text_box() -> CommandText {
    CommandText::new(
        "Text box",
        "Place a box of text on the page as an annotation. It sits on top of the document \
         rather than becoming part of it, and takes the markup colour.",
    )
}

/// `markup.sticky_note`
#[must_use]
pub const fn markup_sticky_note() -> CommandText {
    CommandText::new(
        "Sticky note",
        "Place a collapsed note on the page, which opens when a reader clicks it. Sticky notes \
         use their own standard colours.",
    )
}

/// `markup.stamp`
#[must_use]
pub const fn markup_stamp() -> CommandText {
    CommandText::new(
        "Stamp",
        "Place a stamp on the page. Stamps use their own standard colours.",
    )
}

/// `markup.comments`
#[must_use]
pub const fn markup_comments() -> CommandText {
    CommandText::new(
        "Comments",
        "List the notes and markup on this document and jump to any of them.",
    )
}

// ===========================================================================
// MEASURE TAB
//
// The four controls that had no tooltip at all in the salvage source. Each
// one now says what it measures and what the measurement is read against,
// because the group model — named groups carrying a shared scale, number
// format and drafting standard — is the part of pdfce's measuring that a
// user of any other product will not expect.
// ===========================================================================

/// `measure.linear`
#[must_use]
pub const fn measure_linear() -> CommandText {
    CommandText::new(
        "Linear",
        "Measure a straight distance and place a dimension on the page. The result is read \
         against the current dimension group's scale.",
    )
}

/// `measure.radius_diameter`
#[must_use]
pub const fn measure_radius_diameter() -> CommandText {
    CommandText::new(
        "Radius / diameter",
        "Measure a circle or an arc and place a radius or diameter dimension on the page.",
    )
}

/// `measure.set_scale`
#[must_use]
pub const fn measure_set_scale() -> CommandText {
    CommandText::new(
        "Set scale",
        "Set the scale the current dimension group's measurements are read against — how much \
         real-world length one unit on the drawing stands for.",
    )
}

/// `measure.manage_groups`
#[must_use]
pub const fn measure_manage_groups() -> CommandText {
    CommandText::new(
        "Manage dimension groups…",
        "Add, rename and remove dimension groups, and see the scale, number format and \
         drafting standard each one carries.",
    )
}

// ===========================================================================
// TOOLS TAB
// ===========================================================================

/// `tools.merge_files`
#[must_use]
pub const fn tools_merge_files() -> CommandText {
    CommandText::new(
        "Merge files…",
        "Combine several PDFs into one new file. This document is not changed — to add pages \
         to it instead, use Pages ▸ Merge into this document.",
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
            file_open(),
            file_close(),
            file_recent(),
            file_save_copy(),
            file_export_dxf(),
            file_export_form_data(),
            file_print(),
            file_properties(),
            file_fonts(),
            file_settings(),
            file_shortcuts(),
            view_page_single(),
            view_page_continuous(),
            view_page_facing(),
            view_page_facing_continuous(),
            view_render_strategy(),
            view_render_quality(),
            view_render_settle(),
            view_render_thin_lines(),
            view_render_antialias(),
            view_zoom_actual(),
            view_zoom_fit_page(),
            view_zoom_fit_width(),
            view_show_annotations(),
            view_show_points(),
            view_sidebar(),
            view_panel_pages(),
            view_panel_bookmarks(),
            view_panel_layers(),
            view_panel_signatures(),
            view_panel_objects(),
            view_read_mode(),
            view_fullscreen(),
            view_floating_panels(),
            view_app_initiative(),
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
            edit_copy_page_text(),
            edit_copy_document_text(),
            edit_form_fill(),
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
            markup_highlight(),
            markup_text_box(),
            markup_sticky_note(),
            markup_stamp(),
            markup_comments(),
            measure_linear(),
            measure_radius_diameter(),
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
