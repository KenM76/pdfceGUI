//! # `panels::properties::text` — how the selected text LOOKS, and changing it
//!
//! `RIBBON_IA.md` §5.8's Font controls — `format.font`, `format.font_size` —
//! built where that section says to build them:
//!
//! > Build order: **panel first, tab second.** The panel is the harder half
//! > and the tab's contents are a subset of it, so building the tab first
//! > would mean writing the property editors twice.
//!
//! ## The operator's ask, twice
//!
//! > *"We should also have all the font tools available that Word does."*
//! > — O37, 2026-08-25
//!
//! > *"…when I have an object selected like text the Tool tab doesn't switch
//! > to giving me the editable stuff for that object."* — O46, 2026-08-26
//!
//! ## ★★★ The operand is the TEXT SELECTION, not the object selection
//!
//! This is the decision most likely to be read as a shortcut, so it is argued
//! rather than asserted.
//!
//! `EditSession::format_text` locates its operand by a **pinned byte span into
//! a decoded content buffer**, obtained from `GlyphProvenance` — which is
//! keyed on a *run* of the page's text extraction. The canvas object selection
//! is a `TargetId`: a **paint-order index** into `PageObjects`. The two index
//! spaces are unrelated, and nothing in either crate maps between them.
//!
//! So an object-selection operand would have to be inferred — by bounding box
//! overlap, most plausibly — and an inference that picks the wrong run
//! restyles text the operator did not select, silently, in a file they then
//! send to somebody. The text sweep *is* a run range, exactly, by construction.
//!
//! ★ That is a real gap and it is named rather than hidden: clicking a text
//! object with the Select tool does not raise this section; sweeping across the
//! text does. The empty state says so in those words, because an operator who
//! cannot find a control assumes it is missing.
//!
//! ## ★★ Why the read-back is stamped and not re-read every frame
//!
//! The values shown — face, size, colour — come from `GlyphProvenance`, and
//! provenance is **off** in the shared page-text cache. Reading it means an
//! extraction with `capture_provenance` on, which is the expensive thing this
//! shell does: **392 ms on the operator's benchmark sheet.**
//!
//! A panel that did that per frame would take the application to under three
//! frames a second on exactly the drawings this program is for. So
//! [`TextStyleDraft`] carries a stamp — `(page, first run, edit epoch)` — and
//! re-reads only when it moves, which is the same shape as
//! [`super::geometry::GeometryDraft`] and for a much larger reason.
//!
//! ## ★★★ Bold and Italic are NEVER greyed, and that is the engine's ruling
//!
//! `set_font` selects a real face and refuses when the page carries none.
//! `gate_synthesis` refuses synthesis when a real face **is** available.
//!
//! ★★★ **They are NOT exact complements, and this paragraph used to say they
//! were.** It read *"so between them every page is covered and there is no
//! page on which bold is unreachable"*, quoting the engine — who withdrew the
//! claim in writing on 2026-08-27 after reproducing the counter-example.
//! `gate_synthesis` prefers a real face by *family*, and the face it prefers
//! may not map every glyph in the run, in which case `set_font` refuses it and
//! synthesis is already gated off. On `textedit/format_family.pdf` bold is
//! reachable by neither verb. Filed, confirmed, and queued first by the
//! engine; `crate::app::actions::textstyle`'s header carries the whole of it.
//!
//! ★★ The conclusion survives its premise, which is why the two buttons still
//! do not grey. Greying them would mean predicting a refusal that depends on a
//! per-run glyph-coverage test this shell cannot run without doing the
//! engine's work. The honest behaviour is to try and to show the engine's own
//! named refusal, which is what happens.
//!
//! pdfce-core's instruction, verbatim: *"Do not grey out a bold button. Offer
//! it, and surface the disclosure when synthesis fires."*
//! `crate::app::actions::textstyle` takes whichever verb the page allows and
//! discloses which one it took.
//!
//! ★ This is also why the two toggles do **not** show the run's current state.
//! There is no "is this run bold" bit in a PDF: weight is a property of the
//! *face* (`Helvetica-Bold` is a different font from `Helvetica`), and a
//! synthetic weight is a stroke width in the content stream. A toggle drawn
//! pressed-in would be claiming to have read a fact that is not recorded. They
//! are **buttons that apply**, not switches that reflect — and the face name
//! beside them is where an operator reads what the text actually is.
//!
//! ## Rule 4: nothing here marks the canvas
//!
//! Every disclosure this section causes — a synthetic weight, a colour space
//! narrowed, a real face used instead — lands in the status bar through
//! `crate::app::actions::disclosure`. **The restyled text renders exactly as
//! the saved file will render it.** No badge, no tint, no "provisional"
//! styling: the one-line test is whether a screenshot of the canvas would
//! differ from a screenshot of the same document saved and reopened, and
//! nothing in this module can make it differ.

use egui::Ui;

use crate::app::actions::Action;
use crate::app::actions::textstyle::StyleChange;
use crate::app::state::OpenDoc;
use crate::text::panels::properties as t;

/// The trace region, so a driven check can find this section on screen.
// ui-text-exempt: trace region name, never displayed
pub const REGION: &str = "properties.text";
/// The Bold button's own region.
///
/// ★ Published per control rather than leaving a driven check to divide
/// [`REGION`] by eye — `geometry`'s own note on this is the precedent, and the
/// reason is that a check computing a control's position from a section's
/// bounds is a check that passes on a build where the controls moved.
// ui-text-exempt: trace region name, never displayed
pub const BOLD_REGION: &str = "properties.text.bold";
/// The size spinner's own region.
// ui-text-exempt: trace region name, never displayed
pub const SIZE_REGION: &str = "properties.text.size";
/// The face chooser's own region.
// ui-text-exempt: trace region name, never displayed
pub const FACE_REGION: &str = "properties.text.face";
/// The region of the sentence shown when text is selected as an **object** and
/// nothing has been swept — [`route`].
///
/// ★ Its own name rather than [`REGION`], because the two are mutually
/// exclusive states of one section and a driven check has to be able to tell
/// which one it is looking at. A single region would make *"the panel says
/// something about this text"* pass in the state where it says *"you cannot
/// change it from here"*, which is the state the check exists to detect.
// ui-text-exempt: trace region name, never displayed
pub const ROUTE_REGION: &str = "properties.text.route";

/// What the selected text looks like now, re-read only when it can have
/// changed.
///
/// # The stamp is three parts and every one is load-bearing
///
/// * **page** — a run ordinal means nothing without one;
/// * **first run** — the operator moved the selection to different text;
/// * **edit epoch** — the text is the same text and its style changed, which is
///   what happens on every press of a control in this section. Without this
///   term the panel would show the pre-edit size for ever after the first
///   change, which is the failure that makes a properties panel untrustworthy.
#[derive(Default)]
pub struct TextStyleDraft {
    /// `(page, first run, edit epoch)` the values below were read at.
    stamp: Option<(usize, usize, u64)>,
    /// The run's `/BaseFont`, subset tag and all, or `None` when the
    /// provenance carried no font resource.
    face: Option<String>,
    /// The `Tf` size in points.
    size: f64,
    /// The fill colour as sRGB bytes, or `None` when the run is painted in a
    /// space this control cannot round-trip.
    ///
    /// ★ `None` is shown as *"not a plain colour"* rather than as black. A
    /// swatch that renders a CMYK or Separation fill as its nearest RGB and
    /// then writes that back on the next press would silently convert the
    /// operator's ink — the exact narrowing pdfce refuses to do on their
    /// behalf elsewhere.
    colour: Option<[u8; 3]>,
    /// The size the operator is typing, kept separate from [`Self::size`] so a
    /// half-typed number does not become an edit.
    typed_size: f64,
}

impl TextStyleDraft {
    /// Re-read from the document when the stamp has moved; otherwise keep what
    /// is on screen.
    ///
    /// Returns `true` when there is something to draw — i.e. the run resolved.
    pub(crate) fn sync(&mut self, doc: &OpenDoc, page: usize, run: usize) -> bool {
        let stamp = (page, run, doc.edit_epoch);
        if self.stamp == Some(stamp) {
            return self.face.is_some();
        }
        self.stamp = Some(stamp);
        self.face = None;
        self.size = 0.0;
        self.colour = None;

        // ★ The expensive call, made exactly here and nowhere else in this
        // module. See the module header on the 392 ms.
        let Some(read) = crate::canvas::textedit::pin::inspect(doc, page, run) else {
            return false;
        };
        self.size = f64::from(read.style.size);
        self.typed_size = self.size;
        // ★ The join. `GlyphProvenance` records the RESOURCE KEY the content
        // stream used — `F1` — and an operator needs the `/BaseFont`. The
        // document's font inventory is the only place both appear, so this is
        // the one hop that turns a machine name into a human one.
        //
        // `None` when the key resolves to nothing, which is a real state on a
        // malformed page and is shown as such rather than as a blank combo.
        self.face = read.style.font_resource.as_ref().and_then(|key| {
            doc.font_inventory()
                .fonts
                .iter()
                .find(|record| record.resource_names.iter().any(|name| name == key))
                .and_then(|record| record.base_font.clone())
        });
        self.colour = read.style.fill.and_then(rgb_of);
        self.face.is_some()
    }

    // -----------------------------------------------------------------------
    // Accessors, added 2026-08-27 when the Format ▸ Font group shipped.
    //
    // ★★ **One draft, two surfaces, and that is the whole reason these exist.**
    //
    // The ribbon's Font group and this panel's *This text* section show the
    // same four values and both need them read back from the document. The
    // read costs 392 ms on the operator's benchmark sheet — a text extraction
    // with provenance capture on — so a second draft would double it on every
    // selection change, on exactly the drawings this program is for.
    //
    // So `PanelsState` owns the one draft, `app::surfaces` borrows it for the
    // ribbon and `panels::properties` borrows it for the panel, and whichever
    // draws first in the frame pays for the read while the other gets a stamp
    // hit. These accessors are what let the ribbon read it without the fields
    // becoming public and without `app::fontband` learning how the stamp works.
    // -----------------------------------------------------------------------

    /// The run's `/BaseFont`, subset tag and all, or `None` when the
    /// provenance carried no font resource.
    #[must_use]
    pub(crate) fn face(&self) -> Option<&str> {
        self.face.as_deref()
    }

    /// The `Tf` size the document currently holds, in points.
    ///
    /// ★ Not [`Self::typed_size`]. This is what was **read**; that is what the
    /// operator is **typing**, and the difference between them is what decides
    /// whether a release is an edit or a no-op.
    #[must_use]
    pub(crate) fn size(&self) -> f64 {
        self.size
    }

    /// The size the operator is typing.
    #[must_use]
    pub(crate) fn typed_size(&self) -> f64 {
        self.typed_size
    }

    /// The size the operator is typing, for a widget to write into.
    pub(crate) fn typed_size_mut(&mut self) -> &mut f64 {
        &mut self.typed_size
    }

    /// The fill colour as sRGB bytes, or `None` when the run is painted in a
    /// space this control cannot round-trip. See [`rgb_of`].
    #[must_use]
    pub(crate) fn colour(&self) -> Option<[u8; 3]> {
        self.colour
    }
}

/// Draw the section, or nothing.
///
/// Returns whether it drew, so [`super::body_sections`] knows the panel is
/// already saying something about a selection.
pub fn section(
    ui: &mut Ui,
    doc: &OpenDoc,
    draft: &mut TextStyleDraft,
    actions: &mut Vec<Action>,
) -> bool {
    // ★ The staleness gate is inside `runs`, not here — a stale run ordinal
    // restyles the WRONG text, so the check lives with the data rather than
    // with each of its readers.
    let Some(selection) = doc.text_selection.as_ref() else {
        return route(ui, doc);
    };
    let runs = selection.runs(doc.edit_epoch);
    let Some(&first) = runs.first() else {
        return route(ui, doc);
    };
    let page = selection.page;

    if !draft.sync(doc, page, first) {
        // The selection is real and the run would not pin. Saying nothing here
        // would be the "control that is silently missing" defect: the operator
        // has text selected and the section they saw last time is gone. One
        // sentence, no controls.
        ui.heading(t::text_heading());
        ui.label(t::text_unreadable());
        crate::diag::ui_rect_visible(REGION, ui.min_rect(), ui.clip_rect());
        return true;
    }

    ui.heading(t::text_heading());
    ui.label(t::text_covers(runs.len()));

    face_row(ui, doc, draft, page, &runs, actions);
    size_row(ui, draft, page, &runs, actions);
    weight_row(ui, page, &runs, actions);
    colour_row(ui, draft, page, &runs, actions);

    crate::diag::ui_rect_visible(REGION, ui.min_rect(), ui.clip_rect());
    ui.separator();
    true
}

/// **The route, when a piece of text is selected as an OBJECT and nothing has
/// been swept.**
///
/// Returns whether it drew, so [`section`]'s callers see the panel as having
/// said something about the selection.
///
/// # ★★★ The state this exists for, and why silence was the wrong answer
///
/// The operator clicks a piece of text. The Select tool picks the *object* —
/// a paint-order index — and the Format tab appears, and this section, whose
/// whole subject is how the text looks, drew **nothing**. Which is honest
/// about the operand (there is no run range, so there is nothing to restyle)
/// and dishonest about the capability, because an operator who cannot find a
/// control concludes it is missing. That is O37's report, in his own words:
/// *"the Tool tab doesn't switch to giving me the editable stuff for that
/// object."*
///
/// This module's header has claimed since it shipped that *"the empty state
/// says so in those words"*. It did not; there was no empty state. The
/// sentence is [`t::text_object_route`] and the claim is now true.
///
/// # ★★ Why it is gated on the object being TEXT
///
/// Because a rectangle is not text, and a panel that offered a route to the
/// font controls whenever anything at all was selected would be offering it
/// wrongly nine times in ten. `summary::object_kind` is the same
/// classification the Objects panel row and the read-only object section use,
/// so what this section calls text and what the panel beside it calls text
/// cannot disagree.
///
/// ★ **Only when exactly one object is selected**, matching
/// [`super::geometry::section`]'s rule and for its reason: a mixed
/// multi-selection has no single subject, and a sentence about *"these words"*
/// over a selection of eleven shapes and one label would be describing
/// something the operator did not do.
fn route(ui: &mut Ui, doc: &OpenDoc) -> bool {
    // An annotation is never a page-content text run, and its own sections
    // have already drawn above this one.
    if doc.selection.annot().is_some() {
        return false;
    }
    let page = doc.view.page_index;
    let objects = doc.selection.object_indices_on(page);
    let [object] = objects.as_slice() else {
        return false;
    };
    let object = *object;
    let is_text = doc.page_objects().is_some_and(|provider| {
        provider
            .page_objects()
            .objects
            .get(object)
            .is_some_and(|o| {
                crate::panels::objects::summary::object_kind(o)
                    == crate::panels::objects::summary::ObjectKind::Text
            })
    });
    if !is_text {
        return false;
    }
    ui.heading(t::text_heading());
    ui.label(t::text_object_route());
    crate::diag::ui_rect_visible(ROUTE_REGION, ui.min_rect(), ui.clip_rect());
    ui.separator();
    true
}

/// The face, chosen from the fonts this page already carries.
///
/// # ★★ Why the list is the page's fonts and not a list of typefaces
///
/// `set_font` **selects** an existing resource; it does not **create** one.
/// Offering Helvetica on a page that carries only Arial would produce a
/// refusal on press — a control whose entries may not work, which is precisely
/// what this project spends its time removing.
///
/// The list is therefore built from `fontinfo::FontInventory`, filtered to the
/// records that name this page. ★ That filter is a **name join** and it is not
/// exact: `fontinfo` is keyed on the font *dictionary* and `set_font` matches
/// on `/BaseFont` with the §9.6.4 subset tag stripped, and one page can carry
/// two dictionaries sharing a `/BaseFont` — two subsets of one face, which the
/// survey behind the Fonts panel found in 87 % of embedding files. So this is a
/// superset that is usually exactly right, and when it is not, the press earns
/// a named refusal rather than silence. A proper pre-flight is filed with the
/// engine as `Pass 142.1`.
fn face_row(
    ui: &mut Ui,
    doc: &OpenDoc,
    draft: &TextStyleDraft,
    page: usize,
    runs: &[usize],
    actions: &mut Vec<Action>,
) {
    let current = draft.face.clone().unwrap_or_default();
    let faces = faces_on_page(doc, page);
    ui.horizontal(|ui| {
        ui.label(t::text_face_label());
        let combo = egui::ComboBox::from_id_salt("properties-text-face")
            .selected_text(shorten(&current))
            .show_ui(ui, |ui| {
                for face in &faces {
                    // `selectable_label` rather than `selectable_value`: the
                    // value is not held anywhere between frames, because the
                    // document is the state. A press is an edit, not a
                    // selection to be committed later.
                    if ui
                        .selectable_label(*face == current, shorten(face))
                        .clicked()
                        && *face != current
                    {
                        actions.push(Action::TextStyle {
                            page,
                            runs: runs.to_vec(),
                            change: StyleChange::Face(face.clone()),
                        });
                    }
                }
            });
        crate::diag::ui_rect_visible(FACE_REGION, combo.response.rect, ui.clip_rect());
    });
}

/// The size, in points.
///
/// ★ Committed on `drag_stopped` or `lost_focus`, never on `.changed()`. Each
/// commit is a content-stream rewrite and an undo entry, so a drag across the
/// spinner would author one edit per pixel — the same rule
/// [`super::markup`]'s width and opacity rows follow, for the same reason.
fn size_row(
    ui: &mut Ui,
    draft: &mut TextStyleDraft,
    page: usize,
    runs: &[usize],
    actions: &mut Vec<Action>,
) {
    ui.horizontal(|ui| {
        ui.label(t::text_size_label());
        let response = ui.add(
            egui::DragValue::new(&mut draft.typed_size)
                .speed(0.25)
                .range(1.0..=1440.0)
                .suffix(t::text_size_suffix()),
        );
        crate::diag::ui_rect_visible(SIZE_REGION, response.rect, ui.clip_rect());
        if (response.drag_stopped() || response.lost_focus())
            && (draft.typed_size - draft.size).abs() > f64::EPSILON
        {
            actions.push(Action::TextStyle {
                page,
                runs: runs.to_vec(),
                change: StyleChange::Size(draft.typed_size),
            });
        }
    });
}

/// Bold and Italic — buttons that apply, not switches that reflect.
///
/// See the module header: there is no "is this run bold" bit in a PDF, so a
/// pressed-in toggle would claim to have read a fact that is not recorded.
/// Neither is ever greyed; the engine's two verbs cover every page between
/// them.
fn weight_row(ui: &mut Ui, page: usize, runs: &[usize], actions: &mut Vec<Action>) {
    ui.horizontal(|ui| {
        ui.label(t::text_weight_label());
        let bold = ui.button(t::text_bold());
        crate::diag::ui_rect_visible(BOLD_REGION, bold.rect, ui.clip_rect());
        if bold.on_hover_text(t::text_bold_hint()).clicked() {
            actions.push(Action::TextStyle {
                page,
                runs: runs.to_vec(),
                change: StyleChange::Weight {
                    bold: true,
                    italic: false,
                },
            });
        }
        if ui
            .button(t::text_italic())
            .on_hover_text(t::text_italic_hint())
            .clicked()
        {
            actions.push(Action::TextStyle {
                page,
                runs: runs.to_vec(),
                change: StyleChange::Weight {
                    bold: false,
                    italic: true,
                },
            });
        }
    });
}

/// The fill colour.
///
/// ★ `None` renders a sentence, not a swatch. A run painted in DeviceCMYK, a
/// Separation or an ICC space has no faithful `[u8; 3]`, and a swatch showing
/// its nearest RGB would write that RGB back on the next press — converting
/// the operator's ink without being asked. `pdfce-core` deliberately does not
/// force-convert to DeviceRGB the way Acrobat does, and this control must not
/// undo that on its behalf.
fn colour_row(
    ui: &mut Ui,
    draft: &TextStyleDraft,
    page: usize,
    runs: &[usize],
    actions: &mut Vec<Action>,
) {
    ui.horizontal(|ui| {
        ui.label(t::text_colour_label());
        let Some(current) = draft.colour else {
            ui.label(t::text_colour_not_plain());
            return;
        };
        let mut rgb = current;
        if ui.color_edit_button_srgb(&mut rgb).changed() && rgb != current {
            let components = vec![
                f64::from(rgb[0]) / 255.0,
                f64::from(rgb[1]) / 255.0,
                f64::from(rgb[2]) / 255.0,
            ];
            if let Ok(fill) = pdfce_core::text_edit::NewFill::new(
                pdfce_core::text_edit::FillModel::Rgb,
                components,
            ) {
                actions.push(Action::TextStyle {
                    page,
                    runs: runs.to_vec(),
                    change: StyleChange::Fill(fill),
                });
            }
        }
    });
}

/// The `/BaseFont` names this page carries, deduplicated and sorted.
///
/// See [`face_row`] on why this is a superset rather than an oracle.
pub(crate) fn faces_on_page(doc: &OpenDoc, page: usize) -> Vec<String> {
    // `fontinfo` numbers pages from 1; a run ordinal's page is 0-based.
    let one_based = u32::try_from(page + 1).unwrap_or(u32::MAX);
    let mut names: Vec<String> = doc
        .font_inventory()
        .fonts
        .iter()
        .filter(|record| record.pages.contains(&one_based))
        .filter_map(|record| record.base_font.clone())
        .collect();
    names.sort_unstable();
    names.dedup();
    names
}

/// A `/BaseFont` without its §9.6.4 subset tag, for display only.
///
/// ★ Display only, and the distinction matters: the **value** pushed on the
/// action is the full name, because `set_font` accepts either and handing it
/// the full one keeps the shell from having to know the stripping rule. What
/// an operator gains from `ABCDEF+ArialMT` being shown as `ArialMT` is the
/// ability to read the list at all.
pub(crate) fn shorten(base_font: &str) -> &str {
    match base_font.split_once('+') {
        // A subset tag is exactly six uppercase letters (§9.6.4). Anything else
        // before a `+` is part of the name and is kept — `Foo+Bar` is a legal,
        // if unusual, font name.
        Some((tag, rest)) if tag.len() == 6 && tag.bytes().all(|b| b.is_ascii_uppercase()) => rest,
        _ => base_font,
    }
}

/// sRGB bytes for a fill colour, or `None` when the space cannot round-trip.
///
/// # ★★ Why CMYK is `None` rather than converted
///
/// A conversion here would be a **one-way** trip the operator never asked for.
/// The swatch would show DeviceCMYK ink as its nearest RGB; the next press
/// would write that RGB back through `set_fill`; and the run would leave its
/// original space for ever, on a document heading for a printer that cares.
///
/// `pdfce-core` deliberately does not force-convert to DeviceRGB the way
/// Acrobat does — it stores the space the caller chose — and a control that
/// undid that on the operator's behalf would make the engine's care pointless.
/// Gray round-trips exactly, so it is offered.
fn rgb_of(colour: pdfce_core::text_extract::TextColor) -> Option<[u8; 3]> {
    use pdfce_core::text_extract::TextColor;
    let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    match colour {
        TextColor::Rgb(r, g, b) => Some([byte(r), byte(g), byte(b)]),
        // DeviceGray IS a subset of DeviceRGB with r == g == b, so this is a
        // faithful reading. The write-back is a widening, disclosed by the
        // engine, and the operator asked for a colour.
        TextColor::Gray(v) => Some([byte(v), byte(v), byte(v)]),
        TextColor::Cmyk(..) | TextColor::Other => None,
        // `TextColor` is `#[non_exhaustive]`: a space added later is unknown,
        // and unknown means do not guess.
        _ => None,
    }
}
