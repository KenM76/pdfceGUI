//! # `text::panels::properties` — the Properties panel
//!
//! `RIBBON_IA.md` §5.8 commissions two surfaces for a selection's
//! properties, and is explicit about which is built first:
//!
//! > The division of labour: the **tab** carries what a user changes *while
//! > working* — colour, width, style, align, delete. The **panel** carries
//! > everything, including the read-only facts (winding rule, node count,
//! > embedded-font status, exact geometry) that belong beside the Objects
//! > panel's inventory rather than in a ribbon band.
//! >
//! > Build order: **panel first, tab second.** The panel is the harder half
//! > and the tab's contents are a subset of it, so building the tab first
//! > would mean writing the property editors twice.
//!
//! This is that panel's copy — **the read-only half of it**, which is all of
//! it at stage S3.
//!
//! ## What is deliberately absent, and why it is absent rather than greyed
//!
//! §5.8 also says the panel is *"where the **editable geometry** lives — X,
//! Y, W, H as typed values"*, and calls that the surface through which
//! `/Rect` move-and-resize becomes reachable without a drag. **None of that
//! is here.**
//!
//! Not because typed geometry is hard, but because there is nothing to edit:
//! [`crate::app::actions::Action`] carries zoom and page navigation and
//! nothing else, and the panel that would host the editors has no selection
//! to host them for. Four spinners bound to nothing would render, accept
//! typing, and discard it — which is not a placeholder in the harmless sense
//! but a control that silently loses an operator's work.
//!
//! `RIBBON_IA.md` P3 states the rule this follows: *"An unavailable
//! capability renders nothing, not a disabled stub. Greying is reserved for
//! **temporarily** unavailable — no document open, document encrypted, undo
//! stack empty — and is always explained on hover."* "The selection model
//! does not exist" is not temporary unavailability; it is absence.
//!
//! So the geometry is stated as **facts**, in the same field list as
//! everything else, and becomes editable when there is something to edit.
//!
//! ## The panel is the disclosure surface
//!
//! Every `ObjectNote` an object carries is spelled out here in full, at the
//! foot of the field list. That placement is the disclosure rule's, not a
//! layout preference: inference reporting belongs **off-canvas** — *"a
//! status line, a results panel, a report after the command, a properties
//! field"* — and the page view must carry no badge, tint, dashed outline or
//! "provisional" layer at all.
//!
//! The one-line test the rule offers: *would a screenshot of the editing
//! canvas differ from a screenshot of the same document saved and reopened?*
//! Nothing in this panel can make it differ, because nothing in this panel
//! draws on the page.
//!
//! ## Field wording lives next door
//!
//! The *values* — kind names, paint dispositions, winding rules, colours,
//! font labels, note sentences — are all [`super::objects`]'s, and are
//! reached from here rather than re-worded. That is the same
//! single-description discipline
//! [`crate::panels::objects::summary`] exists to enforce, applied one layer
//! up: a path's fill colour must not be described one way in an Objects row
//! and another way in a Properties field.
//!
//! This module owns only the **labels** — the left-hand column — and the
//! panel's own chrome.

/// Heading over the selected object's properties.
///
/// Says **object**, and it matters. `RIBBON_IA.md` §5.1 gives `file.
/// properties` the tooltip *"The document's own title, author, subject and
/// keywords, and the properties of whatever is selected on the page"* — two
/// scopes under one command. An unheaded field list invites the reading
/// "these are the document's properties", which is exactly wrong for a fill
/// colour, and would be exactly wrong in the other direction for `/Title`.
///
/// The document half of that command is a separate surface and is not built
/// here; there is no `/Info` accessor on `Document` at all (the core API map
/// §12.6 calls that "the honest gap"), so it is a task rather than a
/// forgotten row.
#[must_use]
pub fn properties_object_heading() -> &'static str {
    "Object properties"
}

/// Shown when no object is being shown properties for.
///
/// The panel is never blanked: a blank region is indistinguishable from a
/// broken one, so the honest answer is a sentence naming the precondition —
/// and naming the surface that satisfies it, because the Objects panel is
/// the only route to this one at S3 and an operator has no way to guess
/// that.
#[must_use]
pub fn properties_nothing_focused() -> &'static str {
    "Pick a row in the Objects panel to see what it is made of."
}

/// The heading over the document-metadata section.
///
/// ★ **"This document" rather than "Document properties"**, because the panel
/// now has two subjects and the operator has to be able to tell which half
/// they are reading. The section above it is about *the thing they clicked*;
/// this one is about *the file*, and the two would be told apart by position
/// alone if the panel were never scrolled.
#[must_use]
pub const fn properties_document_heading() -> &'static str {
    "This document"
}

/// What is editable here, and what an empty box means.
///
/// ★ The second sentence is the one that could not be guessed. Clearing a box
/// **removes the key from the file** rather than storing an empty string —
/// `set_info_field(field, None)` against `Some("")` — and they are different
/// documents. It is also the only action in this section that removes
/// anything, which is why it is stated at the top rather than left to be
/// discovered.
#[must_use]
pub const fn properties_document_note() -> &'static str {
    "These are stored in the file and travel with it. Type to change one; \
     empty a box to remove it from the document altogether."
}

/// The label on one document-information field.
///
/// Takes the engine's `InfoField` rather than a string, so the panel
/// enumerates `InfoField::all()` and this maps the result — which is the
/// discipline that enum's own doc comment asks for: *"so a front end
/// enumerates the real list instead of hard-coding one that drifts when a
/// field is added."*
///
/// The words are the PDF spec's own field names in ordinary English. "Subject"
/// and "Keywords" are not obvious, and neither is improved by inventing
/// something friendlier: an operator who has met these fields in any other PDF
/// tool has met them under these names, and a novel word would be a novel
/// thing to learn for no gain.
///
/// # ★ `InfoField` is `#[non_exhaustive]`, so the compiler CANNOT catch a new
/// field here
///
/// This function was first written as a `const fn` with four arms and no
/// wildcard, on the assumption that a fifth variant would break the build. It
/// would not: `#[non_exhaustive]` forces a downstream crate to write `_`, and
/// with a `_` arm the match compiles for ever no matter what the engine adds.
/// The protection people expect from an exhaustive match is **not available
/// across a crate boundary** when the enum is marked that way, and assuming it
/// is available is how a new field ends up silently unlabelled.
///
/// So the safety is built two other ways instead, and both are needed:
///
/// 1. **The fallback is the field's own PDF key**, taken from
///    `InfoField::key()` — the engine's answer, not a guess. A field added
///    upstream appears in the panel labelled `Producer` or `Creator` rather
///    than disappearing or reading "Unknown". Imperfect English, correct, and
///    reachable, which beats all three alternatives.
/// 2. **A test asserts none of the four known fields reaches the fallback.**
///    That is the alarm the compiler cannot raise: if the mapping is ever
///    broken the four named fields start rendering as their raw keys, and the
///    test says which.
#[must_use]
pub fn properties_info_label(field: pdfce_core::edit::InfoField) -> &'static str {
    match field {
        pdfce_core::edit::InfoField::Title => "Title",
        pdfce_core::edit::InfoField::Author => "Author",
        pdfce_core::edit::InfoField::Subject => "Subject",
        pdfce_core::edit::InfoField::Keywords => "Keywords",
        // A field this build does not know a word for. `key()` is
        // `&'static [u8]` of ASCII, so the decode cannot fail — and it is
        // spelled `unwrap_or` rather than `expect` because a panic in a label
        // takes the window with it, and a blank box is a less bad outcome than
        // no window. // ui-text-exempt: the fallback is the engine's own PDF key, not authored copy
        _ => core::str::from_utf8(field.key()).unwrap_or(""),
    }
}

/// ★ **The value shown is pdfce's reading of bytes it could not fully
/// decode.**
///
/// Drawn under a field whose `InfoText::exact` is `false`. That flag means
/// re-encoding the displayed text would **not** reproduce the file's own
/// bytes, so what is on screen is a rendering with substitutions in it, not a
/// copy.
///
/// This is rule 4's surviving half in its purest form: an inference the
/// operator **cannot see**. A replacement character in a metadata field looks
/// like a character rather than like a gap, and without this sentence the
/// operator's only clue that pdfce is guessing would be a glyph they might
/// read as the document's own.
///
/// Worded as a fact about the **document**, not as a pdfce failure — the file
/// really does carry bytes in an encoding it does not declare well enough to
/// resolve — and it says what to do about it, which is the part that makes it
/// actionable rather than alarming: leaving it alone is safe.
#[must_use]
pub const fn properties_info_not_exact() -> &'static str {
    "Some characters in this value could not be read with certainty and are \
     shown as substitutes. Leave the box alone and the file keeps its own \
     bytes; type in it and what you type replaces them."
}

/// The label on the file row.
#[must_use]
pub const fn properties_file_label() -> &'static str {
    "File"
}

/// A document that has never been written to disk.
///
/// `OpenDoc::has_file` is `false` for a document `file.new` created, and its
/// `path` in that state is a *name*, not a location. Showing the name as
/// though it were a file would tell the operator their work is somewhere it is
/// not.
#[must_use]
pub const fn properties_file_unsaved() -> &'static str {
    "not saved to a file yet"
}

/// The label on the size row.
#[must_use]
pub const fn properties_size_label() -> &'static str {
    "Size on disk"
}

/// ★ The size shown is the file as it was OPENED, not as it would be saved.
///
/// `Document::bytes()` is documented as *"the base revision, not the edited
/// state"*, so with unsaved edits this number is the file on disk and not what
/// a save would produce. Shown only while edits are pending, because on an
/// unedited document the two are the same and the sentence would be noise that
/// trains the operator to skip it.
///
/// "Size on disk" rather than "File size" for the same reason: the label
/// itself carries most of the distinction, and the sentence carries the rest
/// when it matters.
#[must_use]
pub const fn properties_size_is_base() -> &'static str {
    "This is the file as it was opened. Your unsaved changes are not counted."
}

/// The label on the PDF-version row.
#[must_use]
pub const fn properties_version_label() -> &'static str {
    "PDF version"
}

/// The label on the page-count row.
#[must_use]
pub const fn properties_pages_label() -> &'static str {
    "Pages"
}

/// The label on the sheet-size row.
#[must_use]
pub const fn properties_page_size_label() -> &'static str {
    "Sheet size"
}

/// One sheet size, in millimetres.
///
/// Millimetres rather than points, because a drafter knows an A3 by
/// `420 × 297` and nobody's intuition is in 72nds of an inch. The page tile's
/// tooltip made the same choice for the same reason.
#[must_use]
pub fn properties_page_size(width_mm: f32, height_mm: f32) -> String {
    format!("{width_mm:.0} × {height_mm:.0} mm")
}

/// ★ A document whose sheets are not all the same size.
///
/// **The common case for this operator**, not an edge case: a drawing set is
/// an A1 general arrangement with A3 details behind it. Reporting page one's
/// size alone would be a true number that reads as a claim about the document,
/// so the mixed case says so and gives the first sheet's size as an example
/// rather than as the answer.
#[must_use]
pub fn properties_page_size_mixed(width_mm: f32, height_mm: f32) -> String {
    format!("mixed — page 1 is {width_mm:.0} × {height_mm:.0} mm")
}

/// The label on the encryption row.
#[must_use]
pub const fn properties_encryption_label() -> &'static str {
    "Encryption"
}

/// An encrypted document.
#[must_use]
pub const fn properties_encrypted() -> &'static str {
    "Encrypted"
}

/// An unencrypted document.
///
/// ★ Stated rather than left blank. This panel's posture is that its silences
/// must be as legible as its numbers, and an absent encryption row is
/// indistinguishable from a panel that does not check — which on this
/// particular question is exactly the wrong impression to leave.
#[must_use]
pub const fn properties_not_encrypted() -> &'static str {
    "Not encrypted"
}

/// ★ What pdfce does NOT tell you about an encrypted document.
///
/// The Signatures panel's discipline applied here: *say what you cannot tell
/// you, first*. A row reading "Encrypted" invites the operator to conclude
/// something about what the document permits, and pdfce reports nothing about
/// permissions in this panel.
///
/// The reason is in `pdfce-core`'s own `DocumentEncryption::perms` doc: `/Perms`
/// is *"the only integrity check in PDF encryption"*, it is a `should` rather
/// than a `shall`, and it is `NotApplicable` for every `/R` ≤ 4 document —
/// which is *"the ordinary answer, not a failed check, and a front end must
/// not render it as one."* Reporting permissions properly means reporting that
/// distinction properly, and this build does not.
#[must_use]
pub const fn properties_encryption_note() -> &'static str {
    "pdfce opened it, so it could read it. This panel does not report what the \
     encryption permits — printing, copying, changing — and an encrypted \
     document may restrict any of them."
}

/// The line stating that this panel reports and does not change.
///
/// **Shown once, at the top, and never repeated per field.** An operator
/// looking at a list of exact numbers with no input boxes will reasonably
/// wonder whether the boxes failed to draw; saying so costs one line and
/// removes the question.
///
/// It states the boundary without naming a future control (P3 again — a
/// promise is a placeholder made of prose).
#[must_use]
pub fn properties_read_only_note() -> &'static str {
    "These are the facts pdfce read from the file. Nothing here can be changed in this build."
}

/// Sub-heading over the disclosure sentences at the foot of the list.
///
/// A heading rather than an unlabelled run of paragraphs, because the
/// sentences are long and an operator scanning for a number needs to know
/// where the numbers stop. "Worth knowing" rather than "Warnings": every one
/// of these is a fact about the document, and warning styling would make a
/// property of the file read as a pdfce failure.
#[must_use]
pub fn properties_notes_heading() -> &'static str {
    "Worth knowing about this object"
}

// ---------------------------------------------------------------------------
// Field labels
//
// The left-hand column, and nothing else. Every VALUE in this panel is
// worded by `super::objects`, so a fact cannot be described one way in an
// Objects row and another way in a Properties field.
//
// Each is a noun, sentence case, with no trailing colon: the colon is
// layout, and putting it in the string means a future two-column layout has
// to strip it back out.
// ---------------------------------------------------------------------------

/// The object's kind.
#[must_use]
pub fn field_type() -> &'static str {
    "Type"
}

/// The object's paint-order index — the handle every command-line verb
/// takes.
#[must_use]
pub fn field_index() -> &'static str {
    "Index"
}

/// How the path is painted (§8.5.3, Table 60).
#[must_use]
pub fn field_paint() -> &'static str {
    "Paint"
}

/// The colour a viewer actually sees for this object.
///
/// "Colour", not "Fill" or "Stroke", because which of the two is showing
/// depends on the paint disposition — a stroke-only path never shows its
/// fill colour, so a field labelled "Fill" would name a colour that appears
/// nowhere on the page. The Paint field directly above says which it is.
#[must_use]
pub fn field_colour() -> &'static str {
    "Colour"
}

/// The fill winding rule (§8.5.3.3).
#[must_use]
pub fn field_winding() -> &'static str {
    "Winding rule"
}

/// Stroke width in user-space units at paint time.
#[must_use]
pub fn field_line_width() -> &'static str {
    "Line width"
}

/// Anchor count across every part of the object.
#[must_use]
pub fn field_nodes() -> &'static str {
    "Points"
}

/// How many separate pieces the object is drawn from.
#[must_use]
pub fn field_parts() -> &'static str {
    "Parts"
}

/// The text a text object shows.
#[must_use]
pub fn field_text() -> &'static str {
    "Text"
}

/// The font in effect at the object's first show operator.
#[must_use]
pub fn field_font() -> &'static str {
    "Font"
}

/// Whether the document carries the font's program.
#[must_use]
pub fn field_font_embedded() -> &'static str {
    "Font embedded"
}

/// An image's sample count.
#[must_use]
pub fn field_pixels() -> &'static str {
    "Image samples"
}

/// The object's lower-left corner in PDF user space.
#[must_use]
pub fn field_position() -> &'static str {
    "Position"
}

/// The object's width and height in PDF points.
#[must_use]
pub fn field_size() -> &'static str {
    "Size"
}

// ---------------------------------------------------------------------------
// Field values that are this panel's own
// ---------------------------------------------------------------------------

/// A position, in PDF points.
///
/// **PDF user space, y-UP, origin at the page's lower left** — the same
/// frame `pdfce-cli` prints and the same frame the object model stores. Not
/// the screen's y-down frame, and not adjusted for `/CropBox` or `/Rotate`.
/// An operator comparing this number against one from the CLI must get the
/// same number, and that is worth more than matching the direction their
/// mouse moves.
///
/// One decimal: enough to tell a 0.0-pt-tall rule from a 0.5-pt one, which
/// is precisely the distinction that makes a hairline look like nothing at
/// all.
#[must_use]
pub fn value_position(x: f64, y: f64) -> String {
    format!("{x:.1}, {y:.1} pt")
}

/// The object's paint-order index, as an operator reads it.
///
/// The `#` is not decoration: it is the form the Objects panel's row label
/// uses and the form `pdfce-cli object-list` prints, so an operator can
/// match a properties field against a row and against a command line without
/// translating. Formatting a number is a catalog decision for exactly this
/// reason — one place decides, and every surface inherits it.
#[must_use]
pub fn value_index(index: usize) -> String {
    format!("#{index}")
}

/// A stroke width, in PDF points.
///
/// Two decimals, unlike the one [`value_size`] uses, and the difference is
/// deliberate: a line width is routinely 0.25 or 0.75 pt, and rounding to
/// one decimal makes a quarter-point hairline and a half-point one the same
/// number. A bounding box is never that fine.
#[must_use]
pub fn value_line_width(width: f64) -> String {
    format!("{width:.2} pt")
}

/// A width and height, in PDF points.
///
/// `×` rather than `x`, and one decimal for the same reason
/// [`value_position`] uses one. A zero on either axis is a real answer, not
/// a missing measurement — the note list below the fields says which shape
/// it is.
#[must_use]
pub fn value_size(width: f64, height: f64) -> String {
    format!("{width:.1} × {height:.1} pt")
}

/// An image's sample count.
///
/// "px" and never "pt": these are SAMPLES (§8.9.5, Table 89), and the Size
/// field a few rows above is in points. An image occupies the unit square
/// under the CTM, so the two numbers describe genuinely different things —
/// where it is, and what it is made of — and the pair is what lets an
/// operator judge effective resolution. They must not look alike.
#[must_use]
pub fn value_pixels(width: u32, height: u32) -> String {
    format!("{width} × {height} px")
}

/// Shown for a field whose value the file does not state.
///
/// One sentence fragment for every such field rather than a per-field
/// wording, because the answer is the same in every case and the *reason*
/// belongs in the note list rather than duplicated across four rows.
///
/// It is not a blank. A blank field is indistinguishable from a field pdfce
/// forgot to fill in, and this panel's entire value is that its silences are
/// as legible as its numbers.
#[must_use]
pub fn value_not_stated() -> &'static str {
    "not stated in the file"
}

/// The font's program is in the document.
#[must_use]
pub fn value_font_embedded_yes() -> &'static str {
    "Yes — the document carries this font's program."
}

/// The font's program is not in the document.
///
/// States the consequence, not just the fact: a font the reader has to
/// supply is the difference between a file that prints as designed anywhere
/// and one that does so only on the machine it was made on. That is the
/// question an operator is actually asking when they look at this field.
#[must_use]
pub fn value_font_embedded_no() -> &'static str {
    "No — this document relies on the reader having a copy of it."
}

/// pdfce could not decide whether the font is embedded.
///
/// ★ **The honest answer to a name-matching problem, and it is disclosed
/// rather than resolved.**
///
/// A text object records the `/BaseFont` in effect; the document's font
/// inventory records a program per font *dictionary*. Joining the two by
/// name is the only join available — the object model does not carry the
/// font dictionary's object id — and a name is not a key: one document can
/// declare two font dictionaries with the same `/BaseFont` (two independent
/// subsets of one face, which the survey behind the Fonts panel found in
/// 87 % of embedding files), and they can differ in whether they embed.
///
/// So when the name matches more than one record, or none, pdfce says it
/// could not tell rather than picking one. Picking would be an inference
/// presented as a fact, which is precisely what rule 4 exists to stop — and
/// unlike most inferences this one is invisible: a confidently wrong "Yes"
/// looks exactly like a right one.
///
/// The Fonts panel is where the per-dictionary truth lives, so this points
/// at it.
#[must_use]
pub fn value_font_embedded_ambiguous() -> &'static str {
    "pdfce could not tell — this document declares more than one font under that name, and they need not agree. The Fonts panel lists each one separately."
}

// ===========================================================================
// The selected markup's style — `set_markup_style`
// ===========================================================================

/// The heading over the markup restyle controls.
#[must_use]
pub const fn markup_heading() -> &'static str {
    "This markup"
}

/// The line under it: what kind of mark is selected.
///
/// ★ The file's own `/Subtype`, translated. An operator placed a *rectangle*
/// and the file calls it `Square`; they placed an *arrow* and the file calls it
/// `Line`. Showing the file's word would be correct and useless — the standing
/// rule in `text::commands` is that a label is the operator's vocabulary and an
/// id is the format's.
#[must_use]
pub fn markup_subtype(subtype: &str) -> String {
    let name = match subtype {
        "Square" => "Rectangle",
        "Circle" => "Ellipse",
        "Line" => "Arrow or line",
        "Polygon" => "Polygon or revision cloud",
        "PolyLine" => "Polyline",
        "Ink" => "Freehand",
        "Highlight" => "Highlight",
        "Underline" => "Underline",
        "StrikeOut" => "Strikeout",
        "Squiggly" => "Squiggly",
        "FreeText" => "Text box",
        "Text" => "Sticky note",
        "Stamp" => "Stamp",
        // ★ Not "Unknown". A subtype this catalogue has no word for is still a
        // real mark the operator can see and is about to restyle, and the
        // file's own spelling is the most honest thing left to show them.
        other => other,
    };
    format!("{name} on this page")
}

/// The colour control's label.
#[must_use]
pub const fn markup_colour_label() -> &'static str {
    "Colour"
}

/// The width control's label.
#[must_use]
pub const fn markup_width_label() -> &'static str {
    "Line width"
}

/// The suffix on the width control.
#[must_use]
pub const fn markup_width_suffix() -> &'static str {
    " pt"
}

/// The opacity control's label.
#[must_use]
pub const fn markup_opacity_label() -> &'static str {
    "Opacity"
}

/// The suffix on the opacity control.
///
/// A percentage, because that is the unit every application an operator has
/// used states opacity in. `/CA`'s own `0.0..=1.0` is a file-format detail they
/// should never meet.
#[must_use]
pub const fn markup_opacity_suffix() -> &'static str {
    " %"
}

/// The button that removes a property, restoring the file's own default.
///
/// ★ *"Clear"*, not *"Reset"* or *"Default"*. It removes the key from the
/// annotation dictionary, and what happens then is that the **standard's**
/// default applies — which is not necessarily what the mark looked like when
/// the operator placed it. "Reset" would promise a return to a previous state
/// that pdfce does not remember.
#[must_use]
pub const fn markup_clear() -> &'static str {
    "Clear"
}

/// ★★ What restyling costs, said once under the whole section.
///
/// Two facts an operator cannot see and would otherwise discover from a
/// changed file:
///
/// 1. **The appearance is regenerated.** `set_markup_style` redraws the mark
///    from the geometry pdfce models, so anything the original expressed
///    *outside* that model — a border effect pdfce does not author, a producer's
///    own decoration — is gone from the new appearance even though its
///    dictionary key survives. The engine reports each one, and those arrive
///    verbatim on the status row; this sentence is the standing warning that
///    such a report is possible at all.
/// 2. **A wider line moves the box.** For every subtype except a rectangle and
///    an ellipse, `/Rect` is derived from the geometry plus a margin that
///    contains the stroke and any arrowheads — so widening the pen makes the
///    annotation's rectangle bigger. That is the engine's own ⚠, and it is the
///    difference between a mark that looks the same and a mark that occupies
///    the same space.
#[must_use]
pub const fn markup_note() -> &'static str {
    "Changing any of these redraws the mark from the shape pdfce has recorded for it. A wider \
     line also makes the mark's own box bigger, except on rectangles and ellipses."
}

/// Why the controls are greyed on a locked annotation.
///
/// Names the standard, because an operator who meets this wants to know whether
/// pdfce is refusing or the document is — and it is the document. It also names
/// the one thing that is still possible, which is the rule a refusal follows
/// everywhere in this shell.
#[must_use]
pub const fn markup_locked() -> &'static str {
    "This mark is locked by the document, so its appearance cannot be changed here. You can \
     still delete it."
}

/// ★★ What regenerating an appearance LOST, in the operator's terms.
///
/// `set_markup_style` redraws a mark from the geometry pdfce models, so
/// anything the original expressed *outside* that model is gone from the new
/// appearance even though its dictionary key survives. The engine names each
/// one; this is the sentence that reaches the operator, and it is owed under
/// rule 4's surviving half — **an inference the operator cannot see still owes
/// an off-canvas report.**
///
/// ★ Every sentence says **what they will see**, not what a key is called. An
/// operator who is told *"the `/BE` border effect was dropped"* has been told
/// nothing; one who is told *"its cloudy edge is now a plain outline"* can look
/// at the page and decide whether they mind.
#[must_use]
pub const fn markup_dropped(dropped: pdfce_core::edit::DroppedProperty) -> &'static str {
    use pdfce_core::edit::DroppedProperty as D;
    match dropped {
        D::BorderEffect => {
            "This mark had a cloudy or hand-drawn edge that pdfce does not redraw. It is now a plain outline."
        }
        D::BorderStyle => {
            "This mark had a border style pdfce does not redraw — a bevel, an inset or an underline. It is now a plain outline."
        }
        D::DashPattern => "This mark's dashed outline is now a solid one.",
        D::RectDifferences => {
            "This mark's own box was inset from the area it covered, and that inset is gone. The mark is drawn to the box now."
        }
        D::LineEnding => {
            "This mark had arrowheads or line ends pdfce does not redraw, and they are gone."
        }
        // `DroppedProperty` is `#[non_exhaustive]`, so a wildcard is required
        // rather than optional. It answers with the general form of the same
        // fact, which is true of every member: something the file expressed is
        // not in the picture pdfce just drew, and saying so imprecisely is far
        // better than saying nothing.
        _ => {
            "This mark carried something pdfce does not redraw, and it is not in the new appearance."
        }
    }
}

// ===========================================================================
// ★ The selected object's geometry — X, Y, W, H typed rather than dragged
//
// Every string here names a **PDF user-space point**, and none of them says
// so more than once. The units live in one note under the heading rather than
// as a suffix on four fields, because "40.00 pt" repeated four times is three
// repetitions of a fact the operator learned from the first one, and a
// properties panel is read top to bottom.
// ===========================================================================

/// The heading over the four geometry fields.
///
/// *"Position and size"* rather than *"Geometry"*: the second is the word a
/// draughtsman uses for the shape of the thing, and this section changes where
/// it is and how big it is. The standing rule in `text::commands` is that a
/// label is the operator's vocabulary.
#[must_use]
pub const fn geometry_heading() -> &'static str {
    "Position and size"
}

/// The units line under the heading.
///
/// ★ It names the corner as well as the unit, and that is the load-bearing
/// half. PDF's Y axis points **up**, so a panel showing `Y` without saying
/// which edge it measures is ambiguous in the one direction that matters — an
/// operator who reads it as a top edge and types a smaller number to move the
/// object up will watch it go down.
#[must_use]
pub const fn geometry_units_note() -> &'static str {
    "Points, measured to the bottom-left corner. Y increases upward."
}

/// The X field's label.
#[must_use]
pub const fn geometry_x() -> &'static str {
    "Left"
}

/// The Y field's label.
///
/// *"Bottom"* rather than *"Y"*, for the reason [`geometry_units_note`] gives:
/// naming the edge makes the axis direction unmistakable at the point of use,
/// not just in a note the operator may have scrolled past.
#[must_use]
pub const fn geometry_y() -> &'static str {
    "Bottom"
}

/// The width field's label.
#[must_use]
pub const fn geometry_w() -> &'static str {
    "Width"
}

/// The height field's label.
#[must_use]
pub const fn geometry_h() -> &'static str {
    "Height"
}

/// The commit button.
///
/// One button for up to two commands, and it does not say how many — *"Apply"*
/// is what the operator is doing; *"raise a move and a scale"* is what the
/// program is doing, and `RIBBON_IA.md` §2's rule is that a control is named
/// for the first.
#[must_use]
pub const fn geometry_apply() -> &'static str {
    "Apply"
}

/// Why Apply is greyed when nothing was typed.
///
/// R9 reserves greying for *temporarily* unavailable and requires the reason on
/// hover. This is the ordinary case — the section has just drawn, the fields
/// hold the object's current numbers, and there is nothing to do until one of
/// them changes.
#[must_use]
pub const fn geometry_nothing_typed() -> &'static str {
    "Type a different number in one of the four fields first."
}

/// Why Apply is greyed when a typed extent would collapse the object.
///
/// ★ It says what the floor IS rather than only that one was hit, because
/// *"too small"* leaves the operator guessing at a threshold, and the whole
/// point of a typed field is that they can hit an exact number.
#[must_use]
pub const fn geometry_too_small() -> &'static str {
    "Width and height must each be at least a quarter of a point — a smaller \
     value would collapse the object onto a line."
}

/// Heading for the recovered-file disclosure.
///
/// ★ Plain, and about the FILE rather than about pdfce. "pdfce had to repair
/// this file" would read as pdfce struggling; the file is the thing that is
/// damaged, and the operator's next question is about the file.
#[must_use]
pub const fn recovered_heading() -> &'static str {
    "This file's index was damaged, and pdfce rebuilt it to open it"
}

/// The detail line: what the rebuild involved.
///
/// ★★ Three numbers, and only the middle one is a warning. Objects recovered
/// says how big the job was; **objects defined more than once** is the one that
/// can put a line in the wrong place, because pdfce had to choose; and repaired
/// says how much else needed inference. Naming them separately lets an operator
/// tell "large but clean" from "small and ambiguous", which a single "recovered
/// N objects" cannot.
#[must_use]
pub fn recovered_detail(objects: usize, collisions: usize, repaired: usize) -> String {
    format!(
        "{objects} objects were recovered by scanning the file. {collisions} were defined more than once, so pdfce chose one of each. {repaired} needed repairing."
    )
}

/// The hover explanation.
///
/// ★★★ States the consequence in the operator's terms and stops. It does not
/// tell them to do anything, because there is nothing reliable to tell them:
/// the file may be perfectly fine, and the only real remedy is a good copy from
/// whoever produced it. Inventing an action would be worse than naming the
/// uncertainty.
#[must_use]
pub const fn recovered_tooltip() -> &'static str {
    "Every PDF carries an index saying where its contents are. This one's was wrong or missing — usually an interrupted download, a crashed writer, or a tool that appended to it badly — so pdfce scanned the whole file and rebuilt the index from what it found. The document opens and prints normally. Where something was defined more than once pdfce had to pick one, so if anything looks out of place, check it against the original before relying on it."
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Every field label is a bare noun phrase with no trailing colon.**
    ///
    /// The colon is layout. Baking it into the string means a future
    /// two-column or grid layout has to strip it back out of every entry,
    /// and the one that gets missed renders as `Type::`.
    #[test]
    fn no_field_label_carries_its_own_punctuation() {
        for label in ALL_FIELD_LABELS {
            assert!(!label.ends_with(':'), "`{label}` carries a colon");
            assert!(
                !label.ends_with('.'),
                "`{label}` is a label, not a sentence"
            );
            assert!(!label.is_empty());
        }
    }

    /// **No two fields share a label.**
    ///
    /// Two rows reading "Size" — one for the bounding box and one for the
    /// image's samples — is exactly the confusion [`value_pixels`]'s "px vs
    /// pt" comment is about, arriving through the label column instead of
    /// the value column.
    #[test]
    fn every_field_label_is_distinct() {
        let mut seen: Vec<&str> = Vec::new();
        for label in ALL_FIELD_LABELS {
            assert!(!seen.contains(&label), "two fields share the label {label}");
            seen.push(label);
        }
    }

    /// The catalog of field labels, for the sweeps above.
    ///
    /// Hand-written, like every enumeration of things Rust cannot enumerate
    /// for us. It is only used by tests, so an entry missed here weakens a
    /// check rather than shipping a defect — but it is listed in the same
    /// order as the panel draws them so a reader can diff the two.
    const ALL_FIELD_LABELS: [&str; 15] = [
        "Type",
        "Index",
        "Paint",
        "Colour",
        "Winding rule",
        "Line width",
        "Points",
        "Parts",
        "Text",
        "Font",
        "Font embedded",
        "Image samples",
        "Position",
        "Size",
        // Not a field: the note heading. Included so a rename of it is
        // caught by the distinctness sweep alongside the fields, since it
        // shares the same column.
        "Worth knowing about this object",
    ];

    /// The label list and the functions agree.
    ///
    /// Without this the sweeps above would silently test a stale copy of the
    /// catalog — the classic failure of a hand-written enumeration.
    #[test]
    fn the_label_catalog_matches_the_functions() {
        let from_fns = [
            field_type(),
            field_index(),
            field_paint(),
            field_colour(),
            field_winding(),
            field_line_width(),
            field_nodes(),
            field_parts(),
            field_text(),
            field_font(),
            field_font_embedded(),
            field_pixels(),
            field_position(),
            field_size(),
            properties_notes_heading(),
        ];
        assert_eq!(from_fns, ALL_FIELD_LABELS);
    }

    /// Position and size are in points, to one decimal, and a zero extent is
    /// a real answer.
    ///
    /// The decimal is not decoration: a horizontal rule is 0.0 pt tall and a
    /// hairline is 0.5 pt tall, and rounding to whole points makes those the
    /// same object.
    #[test]
    fn geometry_values_keep_one_decimal_and_state_their_unit() {
        assert_eq!(value_position(72.0, 144.26), "72.0, 144.3 pt");
        assert_eq!(value_size(200.0, 0.0), "200.0 × 0.0 pt");
        assert!(value_size(1.0, 1.0).ends_with(" pt"));
    }

    /// An image's samples are labelled px, never pt.
    ///
    /// The Size field a few rows above is in points and describes a
    /// different thing. Two numbers of the same shape with the same unit
    /// would read as one measurement stated twice.
    #[test]
    fn image_samples_are_never_labelled_in_points() {
        let px = value_pixels(640, 480);
        assert_eq!(px, "640 × 480 px");
        assert!(!px.contains("pt"));
    }

    /// **The three embedded-font answers are three different answers.**
    ///
    /// The ambiguous one is the load-bearing case: a confidently wrong "Yes"
    /// is indistinguishable from a right one, so the panel has to be able to
    /// decline. It must not read like either of the definite answers, and it
    /// must point at the surface that can be definite.
    #[test]
    fn the_embedded_font_answers_include_an_honest_dont_know() {
        let yes = value_font_embedded_yes();
        let no = value_font_embedded_no();
        let dunno = value_font_embedded_ambiguous();
        assert_ne!(yes, no);
        assert_ne!(no, dunno);
        assert_ne!(yes, dunno);
        assert!(
            dunno.contains("could not tell"),
            "the ambiguous answer must decline in words: {dunno}"
        );
        assert!(
            dunno.contains("Fonts panel"),
            "an honest don't-know has to say where the answer is: {dunno}"
        );
    }

    /// An unstated value is a sentence, never a blank.
    ///
    /// A blank field is indistinguishable from one pdfce forgot to fill in,
    /// and this panel's whole value is that its silences are as legible as
    /// its numbers.
    #[test]
    fn an_absent_value_says_so() {
        assert!(!value_not_stated().trim().is_empty());
    }

    /// **The panel must not promise typed geometry it cannot accept.**
    ///
    /// `RIBBON_IA.md` §5.8 specifies editable X/Y/W/H here, and it is not
    /// built: there is no selection model and no mutating action to carry
    /// the edit. The read-only note is the one string that says so, and a
    /// well-meaning copy edit that turns it into "editing coming soon" would
    /// make it a promise — which P3 forbids in prose exactly as it forbids
    /// in a widget.
    #[test]
    fn the_read_only_note_states_the_boundary_without_promising_a_control() {
        let note = properties_read_only_note();
        assert!(note.contains("can be changed"), "{note}");
        assert!(note.contains("Nothing here"), "{note}");
        for promise in ["coming soon", "not yet available", "will be", "future"] {
            assert!(
                !note.to_lowercase().contains(promise),
                "the note promises a control instead of stating a boundary: {note}"
            );
        }
    }
}
