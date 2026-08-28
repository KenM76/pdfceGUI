//! # `canvas::handles` — eight grips plus move, and the cursor over each
//!
//! `GUI_ROADMAP.md` Phase 1.3: *"Eight handles plus move, per the convention
//! every drawing tool shares. Cursor changes over a handle, over a movable
//! object, over the canvas."*
//!
//! ## Rule 4 says these are welcome, and says exactly why
//!
//! `D:\Dev\FeatureRequests\pdfce_FeatureRequests\README.md`, fourth clause of
//! the disclosure rule:
//!
//! > **A pre-commit affordance is not content marking.** A snap indicator, a
//! > hover highlight, a rubber-band, a selection handle — these are the
//! > *cursor*; they describe what is about to happen and they are welcome.
//! > What is forbidden is styling content that has **already been applied**
//! > as though it were pending.
//!
//! So the grips are drawn, and nothing else is. No badge, no tint, no dashed
//! "provisional" layer over content, nothing that would make a screenshot of
//! the editing canvas differ from a screenshot of the same document saved and
//! reopened. The grips vanish with the selection because they are the
//! cursor's statement about the selection, not a property of the page.
//!
//! ## ★ These are SCREEN-space rects, deliberately, and it is the one place
//!
//! Everything else in `canvas/` past [`crate::canvas::mapping`] is page
//! space. A grip is the exception and must be: it is a **fixed number of
//! screen pixels**, because it is something the operator has to hit with a
//! mouse, and a grip sized in page units would be a 3-pixel speck at fit-page
//! and a slab the size of the object at 800%. It sits on the *output* side of
//! the boundary — the selection's bounds are converted to screen once, by
//! [`crate::canvas::mapping::PageMapping::rect_to_screen`], and the grips are
//! laid out on the result.
//!
//! ## What a grip drag does today, stated so it is not mistaken for an oversight
//!
//! [`Grip::Move`] is live: a drag on the selection's body moves it, through
//! `EditSession::move_objects`.
//!
//! The **eight resize grips change the cursor and consume the drag, and
//! perform no edit yet.** That is not a placeholder left in by accident, and
//! the reason is worth writing down rather than rediscovering: `pdfce-core`
//! has `move_object`, `move_objects`, `move_subpath`, `move_node`,
//! `move_nodes` and `move_handle` — and **no scale or resize verb for a
//! vector object at all**. `GUI_ROADMAP.md` 1.2 (*"move and resize anything
//! carrying a `/Rect`"*, `FEATURES.md:208`) is the row that gives them one,
//! and it covers annotations, form widgets, redaction marks, links and ce
//! dimensions — objects whose size is a rectangle in the file rather than a
//! consequence of their path data.
//!
//! Consuming the drag is the deliberate part. Without it, a drag that started
//! on a grip would fall through and become a **marquee**, so aiming at a
//! resize handle would silently replace the selection the operator was trying
//! to resize. Swallowing the gesture is the honest behaviour until the verb
//! exists.
//!
//! ## conventions: handles
//!
//! Corpus: `ui-conventions/handles.md`.
//!
//! - H1 appear-on-selection: the eight grips are drawn when something is
//!   selected at the Object rung, before any drag.
//! - H2 standard-set: **complete as of 2026-08-20** — eight resize grips, the
//!   body, and a rotate handle offset above the top edge on a stem, which is
//!   the arrangement PowerPoint, Illustrator, Figma, Inkscape, Visio and Konva
//!   all present. This row read *"GAP: no rotate handle, because no engine verb
//!   rotates anything"*, and it ended *"when that lands, the handle above the
//!   top edge is the shape to build, not a menu item."* `Pass 113.0` landed it
//!   and that is the shape that was built.
//! - H3 screen-sized: `GRIP_SIZE_PX` is in points and does not scale with zoom,
//!   so a corner on a plan at 20 % is as grabbable as one at 400 %.
//! - H4 target-not-smaller: `GRIP_GRAB_SLACK_PX` expands the live area beyond
//!   the drawn square. Never the reverse.
//! - H5 grips-outrank-body: checked first, because corner grips sit ON the
//!   box's edge and half of each square overlaps the interior — if the body won,
//!   each would be a half-size target on its outer half only.
//! - H6 cursor-names-it: `Grip::cursor` gives each grip its diagonal or axis
//!   arrow and the body a move cursor.
//! - H7 painted-equals-grabbable: the same predicate decides both. **This row
//!   exists because it failed on 2026-08-20**: a dimension's vertex handles were
//!   painted from the selection and hit-tested behind a capability the mode did
//!   not have, so they were visible and untouchable in the very mode that
//!   authors dimensions.
//! - H8 published: `SELECTION_OUTLINE_REGION` publishes the box every grip is
//!   derived from, and `dimdrag::VERTEX_REGION` publishes each vertex handle
//!   indexed — so a driven check aims at what the application says rather than
//!   at a guess.
//! - H9 vertex-editing: a perimeter ce dimension's corners are handles and drag
//!   to reshape. **GAP: no right-click to add or remove a point**, though both
//!   engine verbs and the preflight that greys the menu item already exist.

use egui::{CursorIcon, Pos2, Rect, Vec2};

/// The side length of a grip square, in screen points.
///
/// Large enough to hit with a mouse without a steady hand, small enough that
/// eight of them around a modest selection do not obscure it. It is also the
/// *drawn* size — grip and target are the same square, which is what makes
/// "aim at the thing you can see" true rather than approximately true.
pub const GRIP_SIZE_PX: f32 = 8.0;

/// Extra slack, in screen points, around a grip's drawn square when
/// hit-testing it.
///
/// Small and asymmetric with the selection catch radius on purpose: a grip is
/// a visible target the operator is aiming at, so it needs far less
/// forgiveness than an invisible hairline does, and every point of slack here
/// is a point stolen from the body-drag region just inside it.
pub const GRIP_GRAB_SLACK_PX: f32 = 2.0;

/// The smallest box, in screen points, that gets mid-edge grips on an axis.
///
/// Below three grip-widths the mid-edge grip would sit on top of its two
/// corner neighbours, producing an unaimable pile that looks like a rendering
/// fault. Corner grips are always offered — they are the ones that survive a
/// small box — so nothing is unreachable, there is simply less on screen.
pub const MIN_MID_GRIP_EXTENT_PX: f32 = GRIP_SIZE_PX * 3.0;

/// One grip on the selection's bounding box.
///
/// Named by compass point rather than by index, because an index would have
/// to be read against a table to know which corner it meant, and the cursor
/// mapping below is exactly such a table — written once, here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Grip {
    /// Top-left corner.
    NorthWest,
    /// Top edge, centred.
    North,
    /// Top-right corner.
    NorthEast,
    /// Right edge, centred.
    East,
    /// Bottom-right corner.
    SouthEast,
    /// Bottom edge, centred.
    South,
    /// Bottom-left corner.
    SouthWest,
    /// Left edge, centred.
    West,
    /// The body of the selection.
    ///
    /// Not drawn as a square: the whole interior *is* the target, which is
    /// what every drawing tool does and what an operator will try first.
    Move,
    /// ★★ **The rotate handle**, offset above the top edge on a stem.
    ///
    /// # Why above, and why on a stem
    ///
    /// Because that is where PowerPoint, Illustrator, Figma, Inkscape, Visio
    /// and Konva's `Transformer` all put it, and the standing tie-breaker for
    /// anything an operator compares against the tools they already use is to
    /// behave the way those tools behave.
    ///
    /// The **offset** is what makes it reachable on a selection whose top edge
    /// is already crowded by the north grip; the **stem** is what says the two
    /// belong together, without which the handle reads as an unrelated dot
    /// floating over the page.
    ///
    /// # ★ It is drawn as a CIRCLE
    ///
    /// Every square on this canvas resizes. A shape that resized in one place
    /// and rotated in another would be a private convention the operator has to
    /// learn, which is `handles.md` H2's stated failure mode — *"the operator
    /// has to learn a control they already knew."*
    ///
    /// # And it is not a resize
    ///
    /// [`Self::is_resize`] answers `false`, so `gesture::meaning` routes a press
    /// on it to its own drag kind rather than to `DragKind::Resize`. That
    /// predicate used to be `self != Self::Move`, which would have quietly made
    /// this the ninth resize grip — a rotate handle that scaled the object, and
    /// a defect nobody would have thought to test for.
    Rotate,
}

impl Grip {
    /// The eight resize grips, clockwise from the top-left.
    ///
    /// Clockwise so the order is the one a reader traces with a finger, which
    /// makes an off-by-one in a table obvious rather than plausible.
    pub const RESIZE: [Self; 8] = [
        Self::NorthWest,
        Self::North,
        Self::NorthEast,
        Self::East,
        Self::SouthEast,
        Self::South,
        Self::SouthWest,
        Self::West,
    ];

    /// The cursor shown while the pointer is over this grip.
    ///
    /// The diagonal cursors are *shared between opposite corners* — NW and SE
    /// both read as `ResizeNwSe` — because that is what the cursor is
    /// describing: the **axis of the resize**, not which corner is under the
    /// hand. Every platform's own resize cursors work this way, and giving
    /// each corner its own arrow would be a private convention the operator
    /// has to learn.
    #[must_use]
    pub fn cursor(self) -> CursorIcon {
        match self {
            Self::NorthWest | Self::SouthEast => CursorIcon::ResizeNwSe,
            Self::NorthEast | Self::SouthWest => CursorIcon::ResizeNeSw,
            Self::North | Self::South => CursorIcon::ResizeVertical,
            Self::East | Self::West => CursorIcon::ResizeHorizontal,
            Self::Move => CursorIcon::Move,
            // ★ egui 0.35 has no rotate cursor, so this is the nearest honest
            // thing rather than the right thing: `Grab` says *"this is a handle
            // you take hold of"*, which is true, where `Default` would say
            // nothing and `Crosshair` would suggest precision placement.
            // Recorded as a compromise rather than a choice — `handles.md` H6
            // asks the cursor to NAME the gesture, and this one only hints at
            // it. A custom cursor is a texture and an atlas entry, which is a
            // real piece of work for one glyph.
            Self::Rotate => CursorIcon::Grab,
        }
    }

    /// Whether this grip resizes rather than moves or rotates.
    ///
    /// ★★ This was `self != Self::Move`, and leaving it that way when
    /// [`Self::Rotate`] arrived would have made the rotate handle **the ninth
    /// resize grip**: `gesture::meaning` asks exactly this question to decide
    /// between `DragKind::Resize` and everything else, so a press on the handle
    /// would have scaled the object about a corner. It would have looked like a
    /// deliberate feature and nothing in the suite asked about it.
    ///
    /// The enumeration is deliberate rather than a negation for that reason: a
    /// tenth affordance added later has to be classified rather than defaulting
    /// into the resize family.
    #[must_use]
    pub fn is_resize(self) -> bool {
        matches!(
            self,
            Self::NorthWest
                | Self::North
                | Self::NorthEast
                | Self::East
                | Self::SouthEast
                | Self::South
                | Self::SouthWest
                | Self::West
        )
    }

    /// Where this grip's centre sits on a screen-space bounding box.
    ///
    /// [`Self::Move`] answers with the box's centre. It has no drawn square,
    /// so the value is only meaningful as "the middle of the thing" — used by
    /// nothing that paints, and defined rather than left as an `Option` so
    /// every arm of the enum has an answer and a future caller cannot be
    /// surprised by a `None`.
    #[must_use]
    pub fn anchor(self, bounds: Rect) -> Pos2 {
        let mid = bounds.center();
        match self {
            Self::NorthWest => bounds.left_top(),
            Self::North => Pos2::new(mid.x, bounds.top()),
            Self::NorthEast => bounds.right_top(),
            Self::East => Pos2::new(bounds.right(), mid.y),
            Self::SouthEast => bounds.right_bottom(),
            Self::South => Pos2::new(mid.x, bounds.bottom()),
            Self::SouthWest => bounds.left_bottom(),
            Self::West => Pos2::new(bounds.left(), mid.y),
            Self::Move => mid,
            // Above the top edge, centred, by the stem's length. The one grip
            // whose centre is OUTSIDE the box, which is what the offset is for.
            Self::Rotate => Pos2::new(mid.x, bounds.top() - ROTATE_STEM_PX),
        }
    }

    /// ★★ **The corner a drag on this grip must leave EXACTLY WHERE IT IS.**
    ///
    /// [`Self::anchor`] answers where the grip *is*; this answers what it pivots
    /// about, and the two are opposite corners. Dragging the south-east grip moves
    /// the south-east corner and leaves the north-west one still — which is what
    /// every drawing application does, and what the standing *"behave the way the
    /// tools they already use behave"* tie-breaker asks for.
    ///
    /// # Why it is a method here and not arithmetic in `canvas::resizing`
    ///
    /// Because it is the same fact as `anchor`, mirrored, and the two must agree:
    /// the ghost is drawn about this point and the commit is computed about it, so
    /// a second spelling would be a preview and an edit that disagreed about which
    /// corner stayed still — an object that jumps on release by exactly the box's
    /// size.
    ///
    /// ★ A mid-edge grip pivots about the **opposite edge**, keeping the axis it
    /// does not scale centred. `East` returns the west edge at the same y, so the
    /// unscaled axis's factor of 1.0 leaves every point on it unmoved whatever y
    /// this returns — but returning the mid-point rather than a corner keeps the
    /// value meaningful if a future edit ever scales both.
    ///
    /// [`Self::Move`] pivots about itself: it does not resize, and a caller that
    /// reached here for it has already gone wrong. Returning the centre is the
    /// harmless answer — a scale about the centre with factors of 1.0 is the
    /// identity — rather than a panic in a frame that is trying to draw.
    #[must_use]
    pub fn pivot(self, bounds: Rect) -> Pos2 {
        let mid = bounds.center();
        match self {
            Self::NorthWest => bounds.right_bottom(),
            Self::North => Pos2::new(mid.x, bounds.bottom()),
            Self::NorthEast => bounds.left_bottom(),
            Self::East => Pos2::new(bounds.left(), mid.y),
            Self::SouthEast => bounds.left_top(),
            Self::South => Pos2::new(mid.x, bounds.top()),
            Self::SouthWest => bounds.right_top(),
            Self::West => Pos2::new(bounds.right(), mid.y),
            Self::Move => mid,
            // ★ The CENTRE, and for this grip it is the real answer rather than
            // a harmless one. A rotation turns the selection about its middle —
            // which is what every drawing program does, and the only choice that
            // leaves the object where the operator can still see it. The eight
            // resize grips pivot about an opposite corner because a resize has
            // an edge that must not move; a rotation has no such edge.
            Self::Rotate => mid,
        }
    }
}

/// How far above the selection box the rotate handle's centre sits, in points.
///
/// ★ Far enough that its grab area (the handle plus [`GRIP_GRAB_SLACK_PX`])
/// cannot overlap the north grip's, or the two would fight for the same press
/// and which one won would depend on the order they are checked in — the
/// failure `handles.md` H5's corollary is about. With a 7 pt handle and 2 pt of
/// slack on each, 20 pt clears both by a comfortable margin.
///
/// Screen-space, like every other number here (H3), so the handle sits the same
/// distance from the box at 20 % as at 400 %.
pub const ROTATE_STEM_PX: f32 = 20.0;

/// **The rotate handle's square**, above the top edge on its stem.
///
/// Separate from [`grip_rects`] rather than an entry in it, because every
/// consumer of that list treats its members as resize grips: the painter draws
/// them as squares and the hit test routes them to `DragKind::Resize`. Adding a
/// ninth entry would have made the rotate handle a square that resizes — the
/// same collision `Grip::is_resize`'s own note describes, arriving through the
/// list instead of through the predicate.
///
/// Drawn as a circle at this rect's centre; see [`Grip::Rotate`].
#[must_use]
pub fn rotate_rect(bounds: Rect) -> Rect {
    Rect::from_center_size(Grip::Rotate.anchor(bounds), Vec2::splat(GRIP_SIZE_PX))
}

/// The grips to draw for a screen-space selection box, with their squares.
///
/// Mid-edge grips are omitted on an axis shorter than
/// [`MIN_MID_GRIP_EXTENT_PX`] — see that constant for why. The corners are
/// always present, so a selection is never left with nothing to grab.
///
/// `bounds` must already be the **visible** box, i.e. after
/// [`crate::canvas::overlay::visible_outline_rect`] has grown a degenerate
/// one. A zero-height rule would otherwise get eight grips stacked along a
/// line, which is both unaimable and a fair description of nothing.
#[must_use]
pub fn grip_rects(bounds: Rect) -> Vec<(Grip, Rect)> {
    let wide = bounds.width() >= MIN_MID_GRIP_EXTENT_PX;
    let tall = bounds.height() >= MIN_MID_GRIP_EXTENT_PX;
    Grip::RESIZE
        .into_iter()
        .filter(|g| match g {
            Grip::North | Grip::South => wide,
            Grip::East | Grip::West => tall,
            _ => true,
        })
        .map(|g| {
            (
                g,
                Rect::from_center_size(g.anchor(bounds), Vec2::splat(GRIP_SIZE_PX)),
            )
        })
        .collect()
}

/// Which grip a screen-space `pointer` is over, or `None` if it is over
/// neither a grip nor the selection's body.
///
/// # Resize grips win over the body, and that is not arbitrary
///
/// The corner grips sit *on* the box's edge, so half of each square overlaps
/// the interior. If the body won, the corner grips would be half-size targets
/// on their outer halves only — the operator would aim at a square and get a
/// move. Checking the grips first makes the drawn square and the live target
/// the same shape, which is the same argument that puts Bézier handles ahead
/// of the nodes they belong to.
#[must_use]
/// Which grips a selection offers, because it has a verb behind each.
///
/// ★★★ Two flags rather than one, added 2026-08-28 when annotations and form
/// fields gained a resize verb (`resize_annotation`, `edit_widget … with_rect`)
/// and neither gained a rotate one.
///
/// The single `offer_resize` bool this replaces was correct while exactly one
/// kind of thing could be resized. It cannot express *"eight grips, no rotate
/// handle"*, and the alternative — painting a rotate handle that does nothing —
/// is the **visible control, silently inert** failure this project spends its
/// time removing.
///
/// ★★ It is one value passed to BOTH the painter and the hit test, which is
/// rule H7 and is why it is a struct rather than two arguments threaded
/// separately. That row exists because it failed on 2026-08-20: a dimension's
/// vertex handles were painted from the selection and hit-tested behind a
/// capability the mode did not have, so they were visible and untouchable in
/// the very mode that authors dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GripSet {
    /// The eight scale grips.
    pub resize: bool,
    /// The rotate handle above the top edge.
    ///
    /// ★ Never true without [`Self::resize`] today, and deliberately not
    /// collapsed into it: *"can this be scaled"* and *"can this be turned"* are
    /// two questions about the engine's verb list, and a shell that inferred
    /// one from the other would offer rotation to the next kind that gains a
    /// resize verb without anybody deciding.
    pub rotate: bool,
}

impl GripSet {
    /// Everything — page content at the Object rung.
    pub const fn all() -> Self {
        Self {
            resize: true,
            rotate: true,
        }
    }

    /// The eight scale grips and no rotate handle — an annotation or a form
    /// field's box, both of which pdfce can scale and cannot turn.
    pub const fn scale_only() -> Self {
        Self {
            resize: true,
            rotate: false,
        }
    }
}

pub fn grip_at(bounds: Rect, pointer: Pos2, offer: GripSet) -> Option<Grip> {
    if offer.rotate {
        // ★★ The rotate handle FIRST, and the reason is H7 rather than
        // geometry: it sits outside the box, so it collides with nothing and
        // the order could not matter for correctness. It is first because
        // **the same predicate decides painting and hit-testing**, and that
        // predicate is `GripSet` — so a handle painted here is grabbable
        // here, in one place, with nothing in between for a future edit to slip
        // a capability check into.
        //
        // That row exists because it failed on 2026-08-20: a dimension's vertex
        // handles were painted from the selection and hit-tested behind a
        // capability the mode did not have, so they were visible and untouchable
        // in the very mode that authors dimensions.
        if rotate_rect(bounds)
            .expand(GRIP_GRAB_SLACK_PX)
            .contains(pointer)
        {
            return Some(Grip::Rotate);
        }
    }
    // ★ The eight scale grips are gated separately from the rotate handle
    // above, which is the whole reason `GripSet` has two fields. An annotation
    // offers these and not that one.
    if offer.resize {
        for (grip, rect) in grip_rects(bounds) {
            if rect.expand(GRIP_GRAB_SLACK_PX).contains(pointer) {
                return Some(grip);
            }
        }
    }
    bounds.contains(pointer).then_some(Grip::Move)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn box_of(w: f32, h: f32) -> Rect {
        Rect::from_min_size(Pos2::new(100.0, 200.0), Vec2::new(w, h))
    }

    /// A comfortable selection offers all eight grips, and each one sits
    /// where its name says.
    #[test]
    fn a_comfortable_box_offers_all_eight_grips_in_the_right_places() {
        let b = box_of(200.0, 100.0);
        let grips = grip_rects(b);
        assert_eq!(grips.len(), 8);

        let at = |g: Grip| {
            grips
                .iter()
                .find(|(k, _)| *k == g)
                .map(|(_, r)| r.center())
                .expect("grip present")
        };
        assert_eq!(at(Grip::NorthWest), b.left_top());
        assert_eq!(at(Grip::SouthEast), b.right_bottom());
        assert_eq!(at(Grip::North), Pos2::new(b.center().x, b.top()));
        assert_eq!(at(Grip::West), Pos2::new(b.left(), b.center().y));
    }

    /// A box too narrow for a mid-edge grip drops it rather than piling it
    /// on top of the corners — but keeps every corner, so nothing becomes
    /// unreachable.
    #[test]
    fn a_narrow_box_drops_its_mid_edge_grips_and_keeps_its_corners() {
        let narrow = box_of(10.0, 200.0);
        let kinds: Vec<Grip> = grip_rects(narrow).into_iter().map(|(g, _)| g).collect();
        assert!(!kinds.contains(&Grip::North));
        assert!(!kinds.contains(&Grip::South));
        assert!(kinds.contains(&Grip::East), "the tall axis keeps its grips");
        for corner in [
            Grip::NorthWest,
            Grip::NorthEast,
            Grip::SouthEast,
            Grip::SouthWest,
        ] {
            assert!(kinds.contains(&corner), "{corner:?} must always be offered");
        }

        // …and symmetrically for a short one.
        let short = box_of(200.0, 10.0);
        let kinds: Vec<Grip> = grip_rects(short).into_iter().map(|(g, _)| g).collect();
        assert!(!kinds.contains(&Grip::East));
        assert!(kinds.contains(&Grip::North));
    }

    /// A grip wins over the body where they overlap, so the drawn square and
    /// the live target are the same shape.
    #[test]
    fn a_grip_wins_over_the_body_where_they_overlap() {
        let b = box_of(200.0, 100.0);
        // Just inside the top-left corner — inside the body, and inside the
        // NW grip's square.
        assert_eq!(
            grip_at(b, b.left_top() + Vec2::splat(2.0), GripSet::all()),
            Some(Grip::NorthWest)
        );
        // Well inside: the body.
        assert_eq!(grip_at(b, b.center(), GripSet::all()), Some(Grip::Move));
        // Well outside: nothing.
        assert_eq!(
            grip_at(b, b.left_top() - Vec2::splat(60.0), GripSet::all()),
            None
        );
    }

    /// Every grip has a cursor, opposite corners share an axis cursor, and
    /// the move grip is the only one that is not a resize.
    #[test]
    fn opposite_corners_share_a_resize_axis_and_move_stands_apart() {
        assert_eq!(Grip::NorthWest.cursor(), Grip::SouthEast.cursor());
        assert_eq!(Grip::NorthEast.cursor(), Grip::SouthWest.cursor());
        assert_eq!(Grip::North.cursor(), Grip::South.cursor());
        assert_eq!(Grip::East.cursor(), Grip::West.cursor());
        assert_ne!(Grip::NorthWest.cursor(), Grip::NorthEast.cursor());
        assert_eq!(Grip::Move.cursor(), CursorIcon::Move);
        assert!(!Grip::Move.is_resize());
        assert!(Grip::RESIZE.iter().all(|g| g.is_resize()));
        assert_eq!(Grip::RESIZE.len(), 8, "eight grips, plus move");
    }

    /// The grips are a fixed number of SCREEN points, so they do not change
    /// size with the zoom — the one place screen space is used inside the
    /// selection layer, and the property that makes it correct.
    #[test]
    fn grips_are_the_same_size_however_big_the_selection_is() {
        for (w, h) in [(40.0, 40.0), (2_000.0, 1_400.0), (60.0, 5_000.0)] {
            for (_, r) in grip_rects(box_of(w, h)) {
                assert!((r.width() - GRIP_SIZE_PX).abs() < f32::EPSILON);
                assert!((r.height() - GRIP_SIZE_PX).abs() < f32::EPSILON);
            }
        }
    }

    /// ★★ **`pivot` is the OPPOSITE of `anchor`, for every resize grip.**
    ///
    /// The property the whole resize rests on, asserted as a relation rather
    /// than against a table of corners — a table would pass for a build whose
    /// `pivot` returned `anchor` unchanged if somebody wrote the table from the
    /// same wrong function.
    ///
    /// The failure it forbids is specific and looks plausible: scaling about
    /// the grip being dragged makes the shape grow *away* from the operator's
    /// hand instead of towards it. It resizes; it is wrong; and it is the kind
    /// of wrong that survives a screenshot.
    #[test]
    fn every_resize_grip_pivots_about_the_opposite_point() {
        let b = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(100.0, 60.0));
        let mid = b.center();
        for g in Grip::RESIZE {
            let a = g.anchor(b);
            let p = g.pivot(b);
            // Reflecting the anchor through the box centre gives the pivot, on
            // every axis the grip actually scales. A mid-edge grip's other axis
            // is the centre in both, so the relation holds on both axes for all
            // eight without a special case.
            assert!(
                (a.x + p.x - 2.0 * mid.x).abs() < 1e-4,
                "{g:?}: anchor.x={} pivot.x={} do not straddle the centre",
                a.x,
                p.x
            );
            assert!(
                (a.y + p.y - 2.0 * mid.y).abs() < 1e-4,
                "{g:?}: anchor.y={} pivot.y={} do not straddle the centre",
                a.y,
                p.y
            );
            assert_ne!(
                a, p,
                "{g:?} pivots about itself, so a drag would scale about the hand"
            );
        }
    }

    /// ★★ **An inner rung offers `Move` and none of the eight.**
    ///
    /// The regression test for a defect found by driving: an anchor mark is
    /// centred on its point, so an anchor at a corner of the object's bounding
    /// box is half outside it — and the corner grip, with two points of grab
    /// slack, covers exactly that spot. A drag from a selected corner anchor
    /// raised no move at all, because the press had been claimed by the
    /// north-west grip.
    ///
    /// The operator's version is *"I can drag the middle nodes and not the end
    /// ones"*, which reads as a broken hit test rather than as two features
    /// competing for one pixel.
    #[test]
    fn an_inner_rung_offers_move_and_no_scale_handles() {
        let b = box_of(200.0, 100.0);
        let corner = b.min;
        assert_eq!(
            grip_at(b, corner, GripSet::all()),
            Some(Grip::NorthWest),
            "the Object rung still offers all eight"
        );
        assert_eq!(
            grip_at(b, corner, GripSet::default()),
            Some(Grip::Move),
            "an inner rung must hand the corner press to the MOVE gesture"
        );
        // And the interior is a move either way — that is how a move drag is
        // recognised at every rung, so withholding the eight must not withhold
        // it.
        assert_eq!(grip_at(b, b.center(), GripSet::default()), Some(Grip::Move));
        assert_eq!(grip_at(b, b.center(), GripSet::all()), Some(Grip::Move));
        // Outside is still nothing.
        assert_eq!(
            grip_at(
                b,
                Pos2::new(b.max.x + 50.0, b.max.y + 50.0),
                GripSet::default()
            ),
            None
        );
    }
}
