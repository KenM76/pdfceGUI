//! # `canvas::selection::annot` — clicking the things pdfce itself put on the page
//!
//! ## ★ The gap this closes, and how long it was open
//!
//! `FEATURES.md` recorded it on 2026-08-17, under the Format contextual tab:
//!
//! > *"**The canvas selection cannot address an annotation** — `Selection` is
//! > `page + object + subpath + node`, four integers naming a paint-order
//! > index into page *content*, which is what makes it immune to zoom and also
//! > means a markup or dimension **is not selectable at all**. The second is
//! > ours; the first is filed."*
//!
//! Both halves of that are now discharged. The engine's half — no verb that
//! modifies an annotation — cleared on 2026-08-18 with `set_markup_style`.
//! This is ours.
//!
//! The operator's report is what it cost: *"How do I edit a stamp I've
//! applied?"*, *"I still can't get to edit dimension groups when I click on
//! it"*, and *"it feels like nothing is moving forward on these things"* —
//! three symptoms of one missing capability. A stamp placed in the wrong spot
//! could not be moved, restyled, or even **deleted**, except by `Ctrl+Z`
//! immediately afterwards.
//!
//! ## Why this is a sibling of [`super::Selection`] and not a variant of it
//!
//! They look similar and are structurally different in four ways, every one of
//! which would have to be special-cased if they shared a type:
//!
//! | | page content | annotation |
//! |---|---|---|
//! | identity | a **paint-order index** — position in `PageObjects::objects` | an **`ObjId`**, stable across edits and across saves |
//! | arity | multi-select, built up over several clicks | one at a time |
//! | structure | a ladder — object ▸ subpath ▸ node, because one CAD path can hold 1,194 subpaths | flat; an annotation has no parts in pdfce's model |
//! | geometry | needs `decompose_page`, which resolves and walks every content stream | `/Rect`, read straight off the dictionary |
//!
//! The last row is why annotation selection needs no cache and no
//! `resolved_for` epoch key: the rectangle is four numbers in the annotation
//! dictionary, and asking for it costs a dictionary lookup rather than a
//! content-stream walk.
//!
//! **They are still mutually exclusive**, and [`super::SelectionState`]
//! enforces that in one place rather than by convention — see its `annot`
//! field. One canvas, one selection; `panels::ObjectTreeUi::focus`' refusal of
//! *"a second selection"* stands.
//!
//! ## ★ Why the KIND is in the type
//!
//! [`AnnotKind`] distinguishes a **ce dimension** from ordinary markup, and it
//! is carried on the target rather than re-derived where it is needed. That is
//! not tidiness — it is the shell's half of a refusal the engine makes by
//! name.
//!
//! A ce dimension is a `/Line` annotation with `/IT /LineDimension`. It passes
//! every *"is this markup pdfce can author?"* test, and restyling one through
//! `set_markup_style` would regenerate its appearance as a **bare line, with
//! its label and witness lines gone** — from an operator who asked only to
//! recolour it. `pdfce-core` refuses it by name
//! (`EditError::AnnotationIsCeDimension`) and points at `set_dimension_style`,
//! and the reply that shipped the verb said so in as many words: *"Your Format
//! tab must route ce dimensions there."*
//!
//! Carrying the kind on the target makes that routing a `match` the compiler
//! checks, rather than a condition somebody has to remember at each of the
//! places a style is applied. The engine's refusal stays as the backstop; this
//! is what stops it being reached.
//!
//! ## Rule 4: this draws nothing on the page that a save would not
//!
//! A selection outline is **the cursor**, which the rule permits by name — the
//! same class as a snap indicator, a rubber band or a resize handle, and the
//! same treatment content selection already gets. Nothing here tints, badges
//! or flags an annotation, and the one-line test still passes: a screenshot of
//! the canvas with a stamp selected differs from a screenshot of the saved
//! file only by the marching outline, which is where the pointer is and not
//! what the document says.

use std::collections::BTreeSet;

use egui::{Pos2, Rect};
use pdfce_core::annot::page_annotations;
use pdfce_core::graph::ObjectGraph;
use pdfce_core::object::ObjId;
use pdfce_core::page_tree::Page;

use crate::canvas::mapping::annot_canvas_rect;

/// Which family an annotation belongs to, and therefore **which verb may
/// restyle it**.
///
/// Two variants, and the distinction is load-bearing rather than descriptive —
/// see the module header. Deliberately not `is_ce_dimension: bool` on a struct:
/// a bool is a fact a caller may forget to read, while a variant is one the
/// compiler makes them handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AnnotKind {
    /// Ordinary markup — a shape, a note, a stamp, a text markup.
    /// `EditSession::set_markup_style` is its verb.
    Markup,
    /// A **ce dimension**: a `/Line` carrying `/IT /LineDimension` and a record
    /// in the document's `/PieceInfo` sidecar.
    ///
    /// `set_dimension_style` is its verb. Handing one to `set_markup_style`
    /// regenerates it as a bare line and loses its label and witness lines,
    /// which is why the engine refuses that by name.
    CeDimension,
}

/// One annotation, addressed the way the engine addresses it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnotTarget {
    /// The page it lives on.
    ///
    /// Carried for the same reason [`super::Selection::page`] is: it lets a
    /// selection survive navigating away and back, and Phase 4 puts several
    /// pages on screen at once.
    pub page: usize,
    /// The annotation's object id — **stable**, unlike a content object's
    /// paint-order index.
    ///
    /// This is what every `EditSession` annotation verb takes, so a selection
    /// made here can be acted on without a second lookup that could resolve
    /// differently.
    pub id: ObjId,
    /// Which verb may restyle it. See [`AnnotKind`].
    pub kind: AnnotKind,
    /// `/Subtype`, as the file spells it — `Stamp`, `Square`, `Line`, `Text`.
    ///
    /// Operator-facing, through [`crate::text`]: the status line and the
    /// Format tab both say *what* is selected, and "Stamp" is the word the
    /// operator used when they placed it.
    pub subtype: String,
    /// §12.5.3 Table 165 bit 8 — the file says the user interface may not
    /// change this annotation's properties.
    ///
    /// Carried on the target rather than checked at each verb, so a surface
    /// can **omit** the controls it governs rather than offer them and let the
    /// engine refuse. That is R83: an affordance that cannot be honoured is
    /// not drawn.
    pub locked: bool,
}

/// A selected annotation, with the outline to draw for it.
#[derive(Debug, Clone, PartialEq)]
pub struct AnnotSelection {
    /// What is selected.
    pub target: AnnotTarget,
    /// Its `/Rect`, in **canvas space** — the zoom-independent space the
    /// content selection's outlines are also cached in, so a zoom or a pan
    /// moves where this is drawn without changing what it is.
    pub outline: Rect,
}

/// Every annotation on `page_index` that a click may select, topmost last.
///
/// # What is excluded, and why each one
///
/// The same four exclusions [`crate::panels::comments`] makes, for the same
/// reasons, plus one this surface needs that the panel does not:
///
/// | excluded | why |
/// |---|---|
/// | `/Widget` | the form field surface owns it — a click there focuses an editor, and two owners of one press is how a field becomes unfillable |
/// | `/Popup` | §12.5.6.14 is a `shall`: a pop-up *"shall not appear alone but is associated with a markup annotation"*. It is a reader-UI window, not content |
/// | `/Link`, `/Movie`, `/PrinterMark`, `/TrapNet` | not authored by the operator and not restylable. `/TrapNet` in particular is prepress output state |
/// | **hidden** (§12.5.3 bit 2) | ★ **this surface's own**, and it is not shared with the panel |
///
/// The hidden case is the one worth stating. The Comments panel *lists* a
/// hidden annotation, deliberately — it is on the page and the operator has a
/// right to know. The canvas must not **select** one, because nothing is drawn
/// there: a click on blank paper would produce a selection outline around
/// nothing, and a Delete would remove something the operator cannot see. The
/// panel is where a hidden annotation is reached, which is exactly the split
/// the forms surface already makes for an undrawn field.
///
/// # Ordering
///
/// `/Annots` order, which is paint order — later entries draw on top. The
/// caller takes the **last** match, so the topmost annotation wins a click,
/// which is the rule page content already follows.
///
/// # Cost
///
/// One `/Annots` walk and one dictionary read per entry, bounded by
/// `pdfce_core::annot::MAX_ANNOTS_PER_PAGE`. No decomposition, no content
/// stream, no cache — see the module header's table.
pub fn selectable_on<G: ObjectGraph + ?Sized>(
    graph: &G,
    page: &Page,
    page_index: usize,
    ce_dimensions: &BTreeSet<ObjId>,
) -> Vec<(AnnotTarget, Rect)> {
    let mut out = Vec::new();
    for annot in page_annotations(graph, page.id) {
        if annot.is_widget() || annot.is_popup || annot.flags.hidden() {
            continue;
        }
        let subtype = String::from_utf8_lossy(&annot.subtype).into_owned();
        if matches!(
            subtype.as_str(),
            "Link" | "Movie" | "PrinterMark" | "TrapNet"
        ) {
            continue;
        }
        // No id means no verb can name it, so selecting it could only ever
        // lead to a refusal — R83 again, at the earliest point it can be
        // applied. `page_annotations` reports an inline (direct) annotation
        // this way; the Comments panel lists those and says so.
        let Some(id) = annot.id else { continue };
        let Some(rect) = annot.rect else { continue };
        let Some(outline) = annot_canvas_rect([rect.llx, rect.lly, rect.urx, rect.ury], page)
        else {
            continue;
        };
        let kind = if ce_dimensions.contains(&id) {
            AnnotKind::CeDimension
        } else {
            AnnotKind::Markup
        };
        out.push((
            AnnotTarget {
                page: page_index,
                id,
                kind,
                subtype,
                locked: annot.flags.locked(),
            },
            outline,
        ));
    }
    out
}

/// The annotation under `point`, or `None`.
///
/// `point` is **canvas space**, the same space `selectable_on` returns and the
/// same space the content hit test works in.
///
/// # ★ No tolerance, and why that is right here when it is wrong elsewhere
///
/// The snap and content hit tests take a tolerance so a hairline can be hit.
/// This one does not, and the reason is the engine's:
///
/// > *"`bounds_of` applies the pen half-width at **authoring** time, so the
/// > stored `/Rect` already contains it. A shell hit-testing `/Rect` is
/// > already correct today."*
///
/// So the rectangle is not the ideal geometry — it is the geometry **plus**
/// the margin a tolerance would be trying to add. Adding a second one would
/// make two adjacent markups claim each other's clicks, which is the failure
/// the forms surface refuses a tolerance for by name.
///
/// # Topmost wins
///
/// The **last** match in `/Annots` order, which is the last one painted. A
/// stamp dropped on top of a rectangle is the thing the operator sees and
/// therefore the thing they mean.
#[must_use]
pub fn hit(candidates: &[(AnnotTarget, Rect)], point: Pos2) -> Option<AnnotSelection> {
    candidates
        .iter()
        .rev()
        .find(|(_, rect)| rect.contains(point))
        .map(|(target, rect)| AnnotSelection {
            target: target.clone(),
            outline: *rect,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target(id: u32, kind: AnnotKind) -> AnnotTarget {
        AnnotTarget {
            page: 0,
            id: ObjId::new(id, 0),
            kind,
            subtype: "Square".to_owned(),
            locked: false,
        }
    }

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::from_min_size(Pos2::new(x, y), egui::vec2(w, h))
    }

    /// ★ **The topmost annotation wins, not the first one found.**
    ///
    /// `/Annots` is paint order, so a stamp dropped over a rectangle is drawn
    /// last and is what the operator sees. A hit test that took the first
    /// match would select the thing underneath — which looks like the click
    /// missing entirely, because the outline appears somewhere the operator
    /// was not pointing.
    #[test]
    fn the_last_painted_annotation_takes_the_click() {
        let candidates = vec![
            (target(1, AnnotKind::Markup), rect(0.0, 0.0, 100.0, 100.0)),
            (target(2, AnnotKind::Markup), rect(20.0, 20.0, 40.0, 40.0)),
        ];
        let hit = hit(&candidates, Pos2::new(30.0, 30.0)).expect("the overlap is a hit");
        assert_eq!(hit.target.id, ObjId::new(2, 0), "the topmost must win");

        // …and outside the upper one, the lower one still takes it.
        let hit = hit_outside(&candidates);
        assert_eq!(hit.target.id, ObjId::new(1, 0));
    }

    fn hit_outside(candidates: &[(AnnotTarget, Rect)]) -> AnnotSelection {
        hit(candidates, Pos2::new(5.0, 5.0)).expect("inside the lower one only")
    }

    /// A click on blank paper selects nothing.
    ///
    /// Stated because the alternative — nearest-match — is a plausible
    /// implementation that would make it impossible to *deselect* by clicking
    /// away, which is the gesture every operator tries first.
    #[test]
    fn a_click_outside_every_annotation_is_not_a_hit() {
        let candidates = vec![(target(1, AnnotKind::Markup), rect(0.0, 0.0, 10.0, 10.0))];
        assert!(hit(&candidates, Pos2::new(50.0, 50.0)).is_none());
        assert!(hit(&[], Pos2::new(0.0, 0.0)).is_none());
    }

    /// ★ The kind survives the hit test.
    ///
    /// The one property that routes a later restyle to `set_dimension_style`
    /// rather than `set_markup_style`. If it were dropped here and re-derived
    /// downstream, the re-derivation would be the thing that could be
    /// forgotten — and forgetting it turns a recolour into a dimension that
    /// loses its label.
    #[test]
    fn a_ce_dimension_stays_a_ce_dimension() {
        let candidates = vec![(
            target(7, AnnotKind::CeDimension),
            rect(0.0, 0.0, 50.0, 50.0),
        )];
        let hit = hit(&candidates, Pos2::new(10.0, 10.0)).expect("a hit");
        assert_eq!(hit.target.kind, AnnotKind::CeDimension);
    }
}
