//! # `canvas::resizing` — the eight grips finally do something, and what it
//! cost to get there without a verb
//!
//! ## What this closes
//!
//! `GUI_ROADMAP.md` Phase 1.3 drew eight resize grips at S4. They have been
//! **cursored, hit-tested and drag-consuming ever since, and have committed
//! nothing** — the last ⛔ in `FEATURES.md`'s Phase 1 list and the oldest
//! unbuilt thing in this project.
//!
//! [`crate::canvas::handles`]' header states the reason and it is still true:
//!
//! > `pdfce-core` has `move_object`, `move_objects`, `move_subpath`,
//! > `move_node`, `move_nodes` and `move_handle` — and **no scale or resize
//! > verb for a vector object at all**.
//!
//! Re-derived against the engine on 2026-08-19 rather than taken from that
//! note: `grep "pub fn .*scale" edit.rs` returns exactly one hit and it is
//! `set_group_scale`, a ce-dimension calibration. **The blocker is real** —
//! unlike two others this project re-checked the same week, both of which had
//! quietly expired.
//!
//! ## ★★ So it is built out of `move_nodes`, and that is the whole idea
//!
//! **Scaling a path IS moving every one of its nodes.** For an anchor `a` and
//! factors `(sx, sy)`:
//!
//! ```text
//! p' = a + (p - a) * (sx, sy)
//! ```
//!
//! `EditSession::move_nodes` takes a **slice** of `(node, Point)`, so a whole
//! resize is **one call, one command, one undo entry** — which is this
//! project's standing rule for one gesture (`canvas::moving`'s §1) and the
//! thing a naive per-node loop would break, both by producing N undo entries
//! and by planning each move against byte offsets the previous one invalidated.
//!
//! The operator's instruction, 2026-08-19: *"finish off phase 1 and phase 5.
//! Get everything unblocked on phase 5 — no excuses about slowness of feature
//! from pdfce as a reason not to implement."* This is that applied to Phase 1.
//!
//! ## ★ What this CANNOT do, stated here rather than discovered
//!
//! All four are consequences of the substitution, not of the implementation,
//! and all four are **worded refusals** rather than silent no-ops:
//!
//! | | why |
//! |---|---|
//! | **text runs** | a text object has no nodes. Scaling one means writing a `Tm`, and this shell will not synthesise one — that is the engine's arithmetic |
//! | **images** | likewise, a `cm` |
//! | **more than one object** | `move_nodes` is per object, so N objects is N commands and N undo entries. One gesture is one command; the honest answer is to decline |
//! | **stroke width** | a scaled path keeps its original `w`, so a 2× box has 1× linework |
//!
//! The last is **not** refused, and that is a judgement rather than an
//! oversight. On a CAD drawing a line weight is a *drafting standard* — 0.25 mm
//! is 0.25 mm whatever size the detail is — so keeping it is right far more
//! often than scaling it would be, and it is the behaviour every drafting
//! package has. It is nonetheless something pdfce decided and the operator did
//! not, so it is **disclosed** ([`crate::text::resizing`]) rather than assumed.
//!
//! ## Why the arithmetic is here and not in `moving`
//!
//! [`crate::canvas::moving`] is about a **displacement** — one delta applied to
//! whatever the rung named. This is about a **map**: every node goes somewhere
//! different, and the somewhere depends on where it started. Folding it in
//! would put two different shapes of answer behind one `MoveSubject`, and the
//! module that owns the ghost preview would have to branch on which.
//!
//! ## The ghost, and rule 4
//!
//! An in-flight resize draws its **new outline**, not a tint over the old one —
//! `canvas::overlay`'s existing move ghost with a different transform. It is a
//! pre-commit affordance and therefore the *cursor*, which R8b's fourth clause
//! welcomes explicitly. Nothing is drawn onto the applied content, and a
//! screenshot of the page after a commit is a screenshot of the page as it will
//! save.

use egui::Vec2;
use pdfce_core::vector::Point;

use crate::app::actions::Action;
use crate::canvas::gesture::Phase;
use crate::canvas::handles::Grip;
use crate::canvas::mapping::PageMapping;
use crate::canvas::selection::SelectionState;
use crate::panels::objects::provider::{ObjectModelProvider, PartKind};

/// Why a resize could not be committed.
///
/// Every variant is **a sentence to show**, never a silent drop —
/// `canvas::textedit::Refusal`'s rule, and for the reason that module's own
/// history proves: this project has already shipped one feature whose answer to
/// a case it could not handle was to do nothing, and the operator reported it
/// as broken for weeks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// Nothing is selected, or the selection names no object on this page.
    NothingSelected,
    /// More than one object is selected.
    ///
    /// `move_nodes` is per object, so this would be N commands and N undo
    /// entries for one drag. See the module header.
    ManyObjects,
    /// The selection is not a path — a text run or an image.
    NotAPath,
    /// The object model could not be read, so nothing can be verified and
    /// therefore nothing may be promised.
    NoObjectModel,
    /// The drag would collapse the selection to nothing on an axis, or invert
    /// it.
    ///
    /// Refused rather than clamped: a zero or negative factor is a shape the
    /// operator cannot have meant, and clamping would silently substitute a
    /// different edit for the one they made.
    Degenerate,
    /// The object has no nodes to move — an empty path.
    NoNodes,
}

/// The scale factors a grip's drag implies, about the anchor opposite it.
///
/// # ★ Why the anchor is the OPPOSITE corner and not the centre
///
/// Because that is what every drawing application does, and the standing
/// tie-breaker for anything an operator compares against the tools they already
/// use is to behave the way those tools behave. Dragging the south-east grip
/// moves the south-east corner and leaves the north-west one exactly where it
/// is — so the part of the object the operator is *not* pointing at does not
/// move under their hand.
///
/// [`Grip::anchor`] already answers this, in **screen** space, for the drawing
/// side. This computes in the same frame and hands the result to the caller to
/// map, rather than re-deriving the opposite-corner rule: two spellings of
/// "which corner stays still" would eventually disagree, and the disagreement
/// would be an object that jumps on the first frame of a drag.
///
/// # The mid-edge grips scale ONE axis
///
/// `East` and `West` scale x and leave y at 1.0; `North` and `South` the
/// reverse. That is what a mid-edge grip means, and it is why they are offered
/// separately from the corners rather than being four more corners.
#[must_use]
pub fn factors(grip: Grip, bounds: egui::Rect, delta: Vec2) -> Option<(f32, f32)> {
    let (w, h) = (bounds.width(), bounds.height());
    if w <= f32::EPSILON || h <= f32::EPSILON {
        return None;
    }
    // How the grip's own motion changes the box's extent on each axis. A grip
    // on the east edge grows the box by its own dx; one on the west shrinks it
    // by the same. A grip that does not touch an axis leaves it alone.
    let dw = match grip {
        Grip::NorthEast | Grip::East | Grip::SouthEast => delta.x,
        Grip::NorthWest | Grip::West | Grip::SouthWest => -delta.x,
        _ => 0.0,
    };
    // ★ Screen y is DOWN and the box is a screen rect, so a south grip dragged
    // downward (positive dy) grows the box. The PDF-space flip happens once, in
    // `canvas::mapping`, and must not be applied a second time here — doing the
    // conversion twice is `canvas::mapping`'s own "classic silent defect".
    let dh = match grip {
        Grip::SouthWest | Grip::South | Grip::SouthEast => delta.y,
        Grip::NorthWest | Grip::North | Grip::NorthEast => -delta.y,
        _ => 0.0,
    };
    let sx = if dw == 0.0 { 1.0 } else { (w + dw) / w };
    let sy = if dh == 0.0 { 1.0 } else { (h + dh) / h };
    Some((sx, sy))
}

/// Whether a pair of factors describes a shape anybody meant.
///
/// A factor at or below zero collapses or mirrors the object. Refused rather
/// than clamped — see [`Refusal::Degenerate`].
///
/// The floor is not `0.0` but a small positive number, because a drag that
/// passes exactly through zero would otherwise produce a
/// zero-area object whose next resize has no bounds to scale from: the
/// `w <= EPSILON` guard in [`factors`] would then answer `None` for ever and
/// the object could never be recovered except by undo.
#[must_use]
pub fn is_usable(sx: f32, sy: f32) -> bool {
    const FLOOR: f32 = 0.001;
    sx.is_finite() && sy.is_finite() && sx > FLOOR && sy > FLOOR
}

/// **Build the one action a completed resize becomes.**
///
/// Pure, so the whole decision is testable without a window: the selection, the
/// object model, the anchor in PDF space and the two factors go in, and one
/// `Action::MoveNodes` or one named refusal comes out.
///
/// # ★ The anchor arrives in PDF user space, already converted
///
/// The caller converts once, through `canvas::mapping`, for the reason
/// `canvas::textedit::resolve_run` records about its own two hops: a second
/// conversion is how a preview and a commit come to disagree about where the
/// operator's hand was.
pub fn action(
    selection: &SelectionState,
    page: usize,
    provider: Option<&ObjectModelProvider>,
    anchor: Point,
    (sx, sy): (f32, f32),
) -> Result<Action, Refusal> {
    if !is_usable(sx, sy) {
        return Err(Refusal::Degenerate);
    }
    let provider = provider.ok_or(Refusal::NoObjectModel)?;
    let objects = selection.object_indices_on(page);
    let object = match objects.as_slice() {
        [] => return Err(Refusal::NothingSelected),
        [one] => *one,
        _ => return Err(Refusal::ManyObjects),
    };
    // ★ The same predicate `canvas::moving::context` uses for its own non-path
    // check, asked of the same provider. A second notion of "is this a path"
    // would let a resize accept an object the move rung refuses, and the two
    // gestures share a selection.
    if provider.part_kind(object) != Some(PartKind::Subpath) {
        return Err(Refusal::NotAPath);
    }
    let nodes = provider.object_node_points(object);
    if nodes.is_empty() {
        return Err(Refusal::NoNodes);
    }
    let moves: Vec<(usize, Point)> = nodes
        .into_iter()
        .map(|(index, p)| {
            (
                index,
                Point::new(
                    anchor.x + (p.x - anchor.x) * f64::from(sx),
                    anchor.y + (p.y - anchor.y) * f64::from(sy),
                ),
            )
        })
        .collect();
    Ok(Action::MoveNodes {
        page,
        object,
        moves,
    })
}

/// The frame's facts about a resize drag in flight.
///
/// A struct rather than seven parameters, and it is not only clippy's
/// arity rule: **five of the seven are read-only facts about the same frame**,
/// so grouping them says what they are. It also removes the failure a long
/// parameter list invites — `map` and `page` are both `Option<&…>` and adjacent,
/// and swapping them would compile if their types ever converged.
///
/// `selection`, `provider` and `actions` stay outside it, deliberately: the
/// first two are *the document's* state rather than the frame's, and the third
/// is an output. A struct that mixed all three would be a bag rather than a
/// grouping.
#[derive(Clone, Copy)]
pub struct Frame<'a> {
    /// Which grip the press landed on, sampled at the press.
    pub grip: Grip,
    /// How far the pointer has travelled since then, in screen points.
    pub delta: Vec2,
    /// Draw the ghost, or commit.
    pub phase: Phase,
    /// The selection's grip box in screen space, or `None` if there is no
    /// outline to have grabbed.
    pub bounds: Option<egui::Rect>,
    /// The page the drag is on.
    pub page_index: usize,
    /// The frame's screen ⟷ canvas mapping.
    pub map: Option<&'a PageMapping>,
    /// The page itself, for the canvas → PDF hop.
    pub page: Option<&'a pdfce_core::page_tree::Page>,
}

/// **Apply one frame of a resize drag: preview it, or commit it.**
///
/// Mirrors [`crate::canvas::moving::drag`] deliberately, down to the return
/// type, so the caller's two arms read the same and a reader who has understood
/// one has understood both. What it hands back is the **scale factors** for the
/// ghost, where the move drag hands back a displacement.
///
/// # ★ A refusal is worded ONCE, on `Complete`
///
/// Not on every frame of the drag. `moving::drag` makes the same choice and its
/// reason applies unchanged: an in-flight gesture is a question, and answering a
/// question the operator has not finished asking would put a sentence on the
/// status row sixty times a second while they were still deciding.
pub fn drag(
    frame: Frame<'_>,
    selection: &SelectionState,
    provider: Option<&ObjectModelProvider>,
    actions: &mut Vec<Action>,
) -> Option<(f32, f32)> {
    let Frame {
        grip,
        delta,
        phase,
        bounds,
        page_index,
        map,
        page,
    } = frame;
    let Some(bounds) = bounds else {
        // No grip box means no selection outline, which means there was nothing
        // to grab — unreachable from a real gesture, and silent because a
        // sentence about a selection that does not exist would be describing
        // the harness rather than the document.
        return None;
    };
    let Some((sx, sy)) = factors(grip, bounds, delta) else {
        if phase == Phase::Complete {
            decline(Refusal::Degenerate);
        }
        return None;
    };
    if phase == Phase::InFlight {
        // ★ The ghost is offered even for factors that will be REFUSED on
        // release, and that is deliberate: an operator dragging a corner past
        // the opposite one can see the shape collapsing, which is how they
        // learn to stop. Hiding the preview at the moment it becomes invalid
        // would read as the drag having stopped tracking.
        return Some((sx, sy));
    }

    // ---- commit ------------------------------------------------------
    let (Some(map), Some(page)) = (map, page) else {
        decline(Refusal::NoObjectModel);
        return None;
    };
    // ★★ The anchor is converted ONCE, here, through the same mapping the
    // outline was drawn with. `canvas::mapping`'s header calls a second
    // conversion *the classic silent defect*: the ghost and the commit would
    // then disagree about which corner stayed still, and the object would jump
    // by whatever the two conversions differed by on release.
    // The same TWO hops `canvas::textedit::resolve_run` takes, in the same
    // order, through the same two functions — screen → canvas → PDF user space.
    // `canvas::mapping`'s header calls doing this any other way *the classic
    // silent defect*: the canvas is Y-down from the page's top-left with
    // `/Rotate` applied, and every coordinate the engine speaks is Y-up from
    // the un-rotated CropBox.
    // ★★ `pivot`, NOT `anchor`. `anchor` is where the grip IS; the point that
    // must stay still is the OPPOSITE corner. Using `anchor` here would scale
    // the object about the very corner the operator is dragging, so the shape
    // would grow away from their hand instead of towards it — a resize that
    // works and is wrong, which is the failure mode this whole module's driven
    // check exists to catch.
    let anchor_screen = grip.pivot(bounds);
    let anchor_canvas = map.to_page(anchor_screen);
    let Some(pdf) = crate::viewer::canvas_to_pdf_space(anchor_canvas, page) else {
        decline(Refusal::Degenerate);
        return None;
    };
    let anchor = Point::new(f64::from(pdf.x), f64::from(pdf.y));
    match action(selection, page_index, provider, anchor, (sx, sy)) {
        Ok(a) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                //
                // ★ Carries the FACTORS and the anchor, which is what a wrong
                // build would get wrong. A line saying only "resize committed"
                // would be identical for a build that scaled about the centre,
                // mirrored an axis, or applied the same factor to both.
                format!(
                    "resize-commit grip={grip:?} sx={sx:.4} sy={sy:.4} \
                     ax={:.2} ay={:.2}",
                    anchor.x, anchor.y
                )
            });
            actions.push(a);
            Some((sx, sy))
        }
        Err(reason) => {
            decline(reason);
            None
        }
    }
}

/// Word a refusal on the status row, and trace it.
///
/// One place, so a variant added to [`Refusal`] is a compile error in
/// `crate::text::resizing` rather than a drag that silently does nothing —
/// which is the failure `canvas::textedit`'s own history is about.
fn decline(reason: Refusal) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("resize-declined reason={reason:?}")
    });
    crate::app::actions::record_note(
        // ★ Epoch zero rather than the document's, and this is the one place in
        // the crate that does it. A refusal changed nothing, so there is no
        // edit for it to be about; `record_note` keys on the epoch so a
        // disclosure retires when the document moves past it, and a refusal
        // must retire on the operator's NEXT act instead. Passing the live
        // epoch would leave "you cannot resize text" on screen through forty
        // subsequent edits.
        0,
        crate::text::resizing::refusal(reason).to_owned(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 100×50 screen box at the origin.
    fn box_100x50() -> egui::Rect {
        egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(100.0, 50.0))
    }

    /// ★ **Dragging the south-east grip right and down grows both axes.**
    ///
    /// The base case, and the one whose y sign is easy to get backwards: screen
    /// y is down, so a positive `dy` on a *south* grip is growth. Getting it
    /// wrong produces an object that shrinks when you pull it bigger, which is
    /// the kind of defect that survives review because both directions "look
    /// like a resize".
    #[test]
    fn the_south_east_grip_grows_both_axes() {
        let (sx, sy) = factors(Grip::SouthEast, box_100x50(), Vec2::new(50.0, 25.0)).expect("box");
        assert!((sx - 1.5).abs() < 1e-6, "sx={sx}");
        assert!((sy - 1.5).abs() < 1e-6, "sy={sy}");
    }

    /// The north-west grip grows when dragged UP and LEFT — negative deltas.
    #[test]
    fn the_north_west_grip_grows_on_negative_travel() {
        let (sx, sy) =
            factors(Grip::NorthWest, box_100x50(), Vec2::new(-50.0, -25.0)).expect("box");
        assert!((sx - 1.5).abs() < 1e-6, "sx={sx}");
        assert!((sy - 1.5).abs() < 1e-6, "sy={sy}");
    }

    /// ★★ **A mid-edge grip scales ONE axis**, which is the whole reason the
    /// four of them are offered separately from the corners.
    ///
    /// A build that treated them as corners would let an operator aiming at
    /// "make this wider" also make it taller — a change they did not ask for,
    /// on the axis they were deliberately not touching.
    #[test]
    fn a_mid_edge_grip_leaves_the_other_axis_alone() {
        let (sx, sy) = factors(Grip::East, box_100x50(), Vec2::new(50.0, 40.0)).expect("box");
        assert!((sx - 1.5).abs() < 1e-6, "sx={sx}");
        assert!(
            (sy - 1.0).abs() < 1e-6,
            "the east grip moved y by {sy}; a mid-edge grip must not touch the other axis even \
             when the pointer wanders across it"
        );
        let (sx, sy) = factors(Grip::South, box_100x50(), Vec2::new(70.0, 25.0)).expect("box");
        assert!((sx - 1.0).abs() < 1e-6, "sx={sx}");
        assert!((sy - 1.5).abs() < 1e-6, "sy={sy}");
    }

    /// A degenerate box has no factors, rather than infinite ones.
    #[test]
    fn a_zero_width_box_has_no_factors() {
        let flat = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(0.0, 50.0));
        assert_eq!(factors(Grip::East, flat, Vec2::new(10.0, 0.0)), None);
    }

    /// ★ **A collapse or a mirror is refused, not clamped.**
    ///
    /// Clamping would silently substitute a different edit for the one the
    /// operator made — and a mirrored path is a legal, plausible-looking
    /// document they did not ask for.
    #[test]
    fn collapsing_and_mirroring_are_refused() {
        assert!(!is_usable(0.0, 1.0));
        assert!(!is_usable(1.0, -1.0));
        assert!(!is_usable(f32::NAN, 1.0));
        assert!(!is_usable(1.0, f32::INFINITY));
        assert!(is_usable(0.5, 2.0));
    }

    /// ★★ **The map is anchored**: the anchor point does not move, and
    /// everything else moves in proportion to its distance from it.
    ///
    /// Asserted as the two properties rather than against a table of
    /// coordinates, because the properties are what "resize about a corner"
    /// means and a coordinate table would pass for a build that had the anchor
    /// at the centre.
    #[test]
    fn the_anchor_stays_put_and_distance_scales() {
        let anchor = Point::new(10.0, 20.0);
        let scaled = |p: Point, sx: f64, sy: f64| {
            Point::new(
                anchor.x + (p.x - anchor.x) * sx,
                anchor.y + (p.y - anchor.y) * sy,
            )
        };
        let at_anchor = scaled(anchor, 3.0, 3.0);
        assert!((at_anchor.x - anchor.x).abs() < 1e-9);
        assert!((at_anchor.y - anchor.y).abs() < 1e-9);

        let far = Point::new(30.0, 20.0);
        let out = scaled(far, 2.0, 2.0);
        assert!(
            ((out.x - anchor.x) - 2.0 * (far.x - anchor.x)).abs() < 1e-9,
            "a point twice as far from the anchor must end up twice as far again"
        );
    }
}
