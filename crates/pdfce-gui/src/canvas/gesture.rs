//! # `canvas::gesture` — press, drag, release, and the clear that must not happen on a press
//!
//! ## ★ Invariant 2, and it lives entirely in this file
//!
//! `GUI_ROADMAP.md` Phase 1, the second of the three ways a selection model
//! loses *"selection survives navigation"*:
//!
//! > **Selection cleared by a click that was really a drag.** A pan gesture
//! > begins with a press on the canvas. If press-on-empty clears the
//! > selection, every pan that starts on blank paper destroys it. The clear
//! > must be driven by a *completed click* with no drag, not by a press.
//!
//! [`GestureState::update`] returns [`GestureOutcome::Idle`] on the press
//! frame — always, unconditionally, whatever the press landed on. A press
//! records where the gesture began and does nothing else. Only a *completed*
//! interaction produces an outcome, and only [`GestureOutcome::Click`] can
//! reach [`crate::canvas::selection::SelectionState::click`].
//!
//! The distinction is egui's to make and it already makes it correctly:
//! `Response::clicked()` is true for a press-and-release that did **not**
//! exceed the drag threshold, and `drag_started`/`dragged`/`drag_stopped` are
//! true for one that did. The two are mutually exclusive on one interaction.
//! What this module adds is the guarantee that **nothing else is consulted**
//! — in particular not `is_pointer_button_down_on`, which is true on the
//! press frame and is exactly how the defect above gets written.
//!
//! ## Primary button only, and why every canvas gesture must say so
//!
//! `Response::drag_started()` is button-agnostic: it is true for a middle-
//! and a right-drag as well as a left one. That is harmless until the middle
//! button means something — and here it means **pan**. A pan read as a
//! selection gesture would make dragging across a drawing replace the
//! selection, or, once a move verb is wired, silently rewrite the page.
//!
//! So the canvas reads `..._by(PointerButton::Primary)` and this module never
//! sees any other button. The right button is excluded for the same reason,
//! before the context menus of Phase 1.1 give it a job.
//!
//! ## Marquee versus pan: settled by the button, not by a heuristic
//!
//! The old shell left this open (*"a drag starting on empty canvas is
//! ambiguous between pan and marquee-select"*). It is not ambiguous here, and
//! it was decided at S0 rather than now: `canvas/mod.rs` switches egui's
//! button-agnostic drag-to-scroll **off** and implements panning against the
//! scroll offset on the middle button, with the stated reason *"the left
//! button is reserved for the selection marquee that arrives at S4"*. Left
//! drags marquee; middle drags pan; neither can be mistaken for the other,
//! and no distance threshold or modal state is involved.

use egui::{Pos2, Rect, Vec2};

use crate::canvas::handles::Grip;

/// What the pointer did over the page this frame, already converted to
/// **canvas space**.
///
/// Assembled in `canvas/mod.rs` from one egui [`egui::Response`] and handed
/// here as plain data, which is what makes the whole state machine testable
/// without a window. Every field is a question egui has already answered; the
/// value of naming them is that the *set* is closed — a future gesture that
/// wants some other signal has to add it here, in front of this module's
/// docs, rather than reaching into a `Response` at a call site.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PointerFrame {
    /// A primary-button drag began this frame.
    pub drag_started: bool,
    /// A primary-button drag is in flight.
    pub dragging: bool,
    /// A primary-button drag ended this frame.
    pub drag_stopped: bool,
    /// A primary-button **click** completed this frame — pressed and
    /// released without exceeding the drag threshold.
    pub clicked: bool,
    /// The completed click was the second of a double-click.
    pub double_clicked: bool,
    /// Where the pointer is, in canvas space, if it is anywhere the canvas
    /// can see. `None` for a gesture whose pointer has left the window.
    pub pos: Option<Pos2>,
    /// Whether Shift was held. Read once, here, so every gesture agrees about
    /// what "extend" means.
    pub shift: bool,
    /// **Escape was pressed this frame**, and the canvas is entitled to it —
    /// i.e. no text field has focus.
    ///
    /// # ★ Why the abort arrives as an input rather than as a method call
    ///
    /// Because a drag in flight and the selection ladder both want Escape, and
    /// exactly one of them may have it per press. Routing the key through the
    /// same `PointerFrame` every other signal arrives on is what makes the
    /// precedence a single readable branch at the top of
    /// [`GestureState::update`] — the drag wins, and it says so by returning
    /// [`GestureOutcome::Cancelled`], which is the caller's cue to leave the
    /// ladder alone this frame.
    ///
    /// A `GestureState::cancel()` method would have worked and would have put
    /// the precedence at the *call site*, where the next reader has to
    /// reconstruct it from two `if`s in different functions. That is how
    /// "Escape cancels the drag AND ascends a rung" ships.
    pub cancel: bool,
}

/// Whether a drag is still happening or has just finished.
///
/// Both matter and they mean different things: an in-flight drag draws a
/// rubber-band or a ghost outline (a pre-commit affordance — the cursor
/// describing what is about to happen), while a completed one changes the
/// selection or raises an action. Collapsing them into one signal is how a
/// marquee ends up committing on every frame it is dragged across.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// The pointer is still down. Draw, do not commit.
    InFlight,
    /// The pointer has been released. Commit.
    Complete,
}

/// What a press landed on — decided once, on the press frame, by the caller.
///
/// # Why it is decided at press time and then remembered
///
/// A drag that began on a grip stays a resize even when the pointer wanders
/// off the grip, off the object, and off the page. Re-deciding per frame
/// would turn a resize into a marquee the instant the operator's hand moved
/// faster than the box, which is exactly when they are dragging hardest.
///
/// This is also what makes the grips *consume* their drags. See
/// [`crate::canvas::handles`]: without it, a drag aimed at a resize grip
/// would fall through to a marquee and silently replace the selection the
/// operator was trying to resize.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragKind {
    /// The press was on empty paper, or on unselected content: rubber-band.
    Marquee,
    /// The press was inside the selection's body: move it.
    Move,
    /// The press was on one of the eight resize grips.
    Resize(Grip),
}

/// What the canvas should do about the pointer this frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GestureOutcome {
    /// Nothing to do. **This is what a press produces** — see invariant 2 in
    /// the module docs.
    Idle,
    /// A drag in flight was **abandoned by Escape** without committing.
    ///
    /// Distinct from [`Self::Idle`] on purpose, and the distinction is
    /// load-bearing in both directions:
    ///
    /// * it tells the caller the key was **consumed**, so the same press must
    ///   not also ascend the selection ladder — one press, one effect, which
    ///   is decision 025's L1 applied to the gesture layer;
    /// * it is raised only when a drag was genuinely in flight, so Escape with
    ///   an idle pointer falls straight through to the ladder and the operator
    ///   never has to press it twice to leave a rung.
    ///
    /// Nothing is committed and nothing is drawn. A cancelled move puts the
    /// object back where it was for the same reason an *interrupted* drag
    /// does — see [`GestureState::update`]'s last branch — except that this one
    /// is the operator asking, so it is reported rather than silent.
    Cancelled,
    /// A completed click with no drag: the only outcome that may change the
    /// selection by hit test, and the only one that may clear it.
    Click {
        /// Canvas-space position of the click.
        point: Pos2,
        /// Whether Shift was held (extend rather than replace).
        shift: bool,
        /// Whether this was the second click of a double-click (descend a
        /// rung rather than pick at the current one).
        double: bool,
    },
    /// A rubber-band, in canvas space.
    Marquee {
        /// The band, normalised — dragged in any of four directions.
        rect: Rect,
        /// Whether Shift was held at the press (extend rather than replace).
        shift: bool,
        /// Draw, or commit.
        phase: Phase,
    },
    /// A move drag of the current selection, as a canvas-space delta.
    Move {
        /// How far the pointer has travelled since the press.
        delta: Vec2,
        /// Draw the ghost, or commit the move.
        phase: Phase,
    },
    /// A resize drag on one of the eight grips.
    ///
    /// Raised so the drag is **consumed** rather than falling through to a
    /// marquee. `pdfce-core` has no scale verb for a vector object, so
    /// nothing commits on `Complete` yet — see [`crate::canvas::handles`] for
    /// the whole reasoning and the roadmap row that gives it a verb.
    Resize {
        /// Which grip is being dragged.
        grip: Grip,
        /// How far the pointer has travelled since the press.
        delta: Vec2,
        /// Draw, or commit.
        phase: Phase,
    },
}

/// A primary-button drag in flight.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Drag {
    /// Canvas-space position of the press.
    origin: Pos2,
    /// Canvas-space position of the most recent frame that had one.
    ///
    /// Held rather than re-read, so a frame in which the pointer left the
    /// window continues the gesture from where it was last seen instead of
    /// collapsing it to the origin — which would look like the object
    /// snapping back to where it started.
    latest: Pos2,
    /// What the press landed on.
    kind: DragKind,
    /// Whether Shift was held **at the press**.
    ///
    /// At the press, not at the release: an operator who lets go of Shift
    /// before the mouse button has still asked for an extending marquee, and
    /// sampling the modifier at the end would make the gesture's meaning
    /// depend on the order two fingers came up.
    shift: bool,
}

impl Drag {
    /// This drag's outcome at `phase`.
    fn outcome(self, phase: Phase) -> GestureOutcome {
        let delta = self.latest - self.origin;
        match self.kind {
            DragKind::Marquee => GestureOutcome::Marquee {
                rect: Rect::from_two_pos(self.origin, self.latest),
                shift: self.shift,
                phase,
            },
            DragKind::Move => GestureOutcome::Move { delta, phase },
            DragKind::Resize(grip) => GestureOutcome::Resize { grip, delta, phase },
        }
    }
}

/// The canvas's pointer-gesture state between frames.
///
/// One `Option`. Everything else is derived from the frame's own signals,
/// which is deliberate: gesture state that outlives its gesture is how a
/// canvas ends up in a mode the operator cannot see and cannot leave.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct GestureState {
    drag: Option<Drag>,
}

impl GestureState {
    /// Advance the machine by one frame.
    ///
    /// `press_kind` is consulted **only** on the frame a drag starts; on
    /// every other frame it is ignored, so the caller may compute it however
    /// cheaply it likes without worrying about which frame it is.
    ///
    /// # The order of the branches is the invariant
    ///
    /// 0. **Escape abandons a drag in flight**, and only then. `Option::take`
    ///    does both halves in one expression: it clears the gesture and
    ///    reports whether there was one to clear, so an Escape with no drag
    ///    under it changes nothing here and reaches the ladder untouched.
    /// 1. **A press starts a drag and returns `Idle`.** Nothing else. No hit
    ///    test, no clear, no selection change. This is invariant 2.
    /// 2. **A drag in flight owns the frame**, so a stray `clicked` cannot be
    ///    read out of the middle of one.
    /// 3. **A completed click** is the only thing that reaches the selection
    ///    by hit test.
    ///
    /// Reordering 1 and 3 is exactly the defect: `clicked` would still be
    /// false on the press frame, but a future edit that "helpfully" hit-tested
    /// on press would have nowhere obvious to be wrong. Keeping the press arm
    /// first, and empty, is what makes the rule visible to the next reader.
    ///
    /// Branch 0 sits above the press for a smaller but real reason: a frame
    /// carrying both a cancel and a fresh `drag_started` is a *new* gesture,
    /// and the abandoned one must not be able to resurrect itself by having
    /// its origin overwritten.
    pub fn update(&mut self, frame: PointerFrame, press_kind: DragKind) -> GestureOutcome {
        if frame.cancel && self.drag.take().is_some() {
            return GestureOutcome::Cancelled;
        }

        if frame.drag_started {
            if let Some(pos) = frame.pos {
                self.drag = Some(Drag {
                    origin: pos,
                    latest: pos,
                    kind: press_kind,
                    shift: frame.shift,
                });
            }
            return GestureOutcome::Idle;
        }

        if let Some(drag) = &mut self.drag {
            if let Some(pos) = frame.pos {
                drag.latest = pos;
            }
            let drag = *drag;
            if frame.drag_stopped {
                self.drag = None;
                return drag.outcome(Phase::Complete);
            }
            if frame.dragging {
                return drag.outcome(Phase::InFlight);
            }
            // Neither dragging nor stopped: the gesture was interrupted —
            // the window lost focus, the pointer left, a dialog opened.
            // Abandon it rather than commit it. An interrupted drag whose
            // release nobody saw must not resize a drawing.
            self.drag = None;
            return GestureOutcome::Idle;
        }

        if (frame.clicked || frame.double_clicked)
            && let Some(point) = frame.pos
        {
            return GestureOutcome::Click {
                point,
                shift: frame.shift,
                double: frame.double_clicked,
            };
        }

        GestureOutcome::Idle
    }

    /// Whether a drag is in flight — the canvas asks before setting a cursor,
    /// so a gesture keeps its own cursor even when the pointer has wandered
    /// off the thing it started on.
    #[must_use]
    pub fn active(&self) -> Option<DragKind> {
        self.drag.map(|d| d.kind)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(x: f32, y: f32) -> Option<Pos2> {
        Some(Pos2::new(x, y))
    }

    /// ★ **A press produces nothing at all** — invariant 2, at its source.
    ///
    /// Whatever the press landed on, and whatever else the frame carries, the
    /// press frame is `Idle`. Nothing downstream of this can clear a
    /// selection, because nothing downstream of this is called.
    #[test]
    fn a_press_alone_produces_no_outcome() {
        for kind in [DragKind::Marquee, DragKind::Move] {
            let mut g = GestureState::default();
            let out = g.update(
                PointerFrame {
                    drag_started: true,
                    pos: at(10.0, 10.0),
                    ..PointerFrame::default()
                },
                kind,
            );
            assert_eq!(out, GestureOutcome::Idle, "a press must decide nothing");
        }
    }

    /// ★ **A press on blank paper that becomes a drag never yields a click.**
    ///
    /// The whole sequence, frame by frame, as the roadmap describes it: press
    /// on empty canvas, move, release. If any frame produced a `Click`, the
    /// selection would be cleared by hit test — which is the defect.
    #[test]
    fn a_press_that_becomes_a_drag_never_yields_a_click() {
        let mut g = GestureState::default();
        let mut outcomes = Vec::new();
        outcomes.push(g.update(
            PointerFrame {
                drag_started: true,
                pos: at(0.0, 0.0),
                ..PointerFrame::default()
            },
            DragKind::Marquee,
        ));
        for step in 1..=5u8 {
            outcomes.push(g.update(
                PointerFrame {
                    dragging: true,
                    pos: at(f32::from(step) * 10.0, 0.0),
                    ..PointerFrame::default()
                },
                DragKind::Marquee,
            ));
        }
        outcomes.push(g.update(
            PointerFrame {
                drag_stopped: true,
                pos: at(50.0, 20.0),
                ..PointerFrame::default()
            },
            DragKind::Marquee,
        ));

        assert!(
            !outcomes
                .iter()
                .any(|o| matches!(o, GestureOutcome::Click { .. })),
            "a drag produced a click: {outcomes:?}"
        );
        assert_eq!(
            outcomes.last(),
            Some(&GestureOutcome::Marquee {
                rect: Rect::from_two_pos(Pos2::ZERO, Pos2::new(50.0, 20.0)),
                shift: false,
                phase: Phase::Complete,
            })
        );
    }

    /// A completed click with no drag is the one thing that reaches the
    /// selection by hit test.
    #[test]
    fn a_completed_click_is_reported_once_with_its_modifiers() {
        let mut g = GestureState::default();
        assert_eq!(
            g.update(
                PointerFrame {
                    clicked: true,
                    pos: at(7.0, 9.0),
                    shift: true,
                    ..PointerFrame::default()
                },
                DragKind::Marquee,
            ),
            GestureOutcome::Click {
                point: Pos2::new(7.0, 9.0),
                shift: true,
                double: false,
            }
        );
        // The frame after carries nothing, so the click is applied once.
        assert_eq!(
            g.update(PointerFrame::default(), DragKind::Marquee),
            GestureOutcome::Idle
        );
    }

    /// A double-click is reported as one, so the ladder descends instead of
    /// re-picking at the same rung.
    #[test]
    fn a_double_click_is_reported_as_a_double() {
        let mut g = GestureState::default();
        let out = g.update(
            PointerFrame {
                clicked: true,
                double_clicked: true,
                pos: at(1.0, 2.0),
                ..PointerFrame::default()
            },
            DragKind::Marquee,
        );
        assert_eq!(
            out,
            GestureOutcome::Click {
                point: Pos2::new(1.0, 2.0),
                shift: false,
                double: true,
            }
        );
    }

    /// The marquee draws while in flight and commits once, on release — not
    /// on every frame it is dragged across.
    #[test]
    fn a_marquee_draws_in_flight_and_commits_once() {
        let mut g = GestureState::default();
        g.update(
            PointerFrame {
                drag_started: true,
                pos: at(100.0, 100.0),
                shift: true,
                ..PointerFrame::default()
            },
            DragKind::Marquee,
        );
        let mid = g.update(
            PointerFrame {
                dragging: true,
                pos: at(40.0, 30.0),
                ..PointerFrame::default()
            },
            DragKind::Marquee,
        );
        assert_eq!(
            mid,
            GestureOutcome::Marquee {
                // Dragged up and left: normalised, or it would contain nothing.
                rect: Rect::from_two_pos(Pos2::new(100.0, 100.0), Pos2::new(40.0, 30.0)),
                shift: true,
                phase: Phase::InFlight,
            },
            "an in-flight marquee must be drawn, and must not commit"
        );
        let end = g.update(
            PointerFrame {
                drag_stopped: true,
                pos: at(40.0, 30.0),
                // Shift released before the button: the gesture keeps the
                // meaning it started with.
                shift: false,
                ..PointerFrame::default()
            },
            DragKind::Marquee,
        );
        assert_eq!(
            end,
            GestureOutcome::Marquee {
                rect: Rect::from_two_pos(Pos2::new(100.0, 100.0), Pos2::new(40.0, 30.0)),
                shift: true,
                phase: Phase::Complete,
            },
            "the modifier is sampled at the press, not at the release"
        );
    }

    /// A drag keeps the kind it started with, even when the pointer leaves
    /// the grip, the object and the page.
    #[test]
    fn a_drag_keeps_the_kind_it_started_with() {
        let mut g = GestureState::default();
        g.update(
            PointerFrame {
                drag_started: true,
                pos: at(0.0, 0.0),
                ..PointerFrame::default()
            },
            DragKind::Resize(Grip::SouthEast),
        );
        // The caller now says "Marquee" every frame — the pointer has left
        // the grip. The gesture must not change under it.
        let out = g.update(
            PointerFrame {
                dragging: true,
                pos: at(-500.0, -900.0),
                ..PointerFrame::default()
            },
            DragKind::Marquee,
        );
        assert_eq!(
            out,
            GestureOutcome::Resize {
                grip: Grip::SouthEast,
                delta: Vec2::new(-500.0, -900.0),
                phase: Phase::InFlight,
            }
        );
        assert_eq!(g.active(), Some(DragKind::Resize(Grip::SouthEast)));
    }

    /// A move drag reports the travel since the press, not since last frame —
    /// so a caller applying it once on `Complete` moves the object by the
    /// whole gesture rather than by its last twitch.
    #[test]
    fn a_move_drag_reports_the_whole_travel() {
        let mut g = GestureState::default();
        g.update(
            PointerFrame {
                drag_started: true,
                pos: at(10.0, 10.0),
                ..PointerFrame::default()
            },
            DragKind::Move,
        );
        for step in 1..=3u8 {
            g.update(
                PointerFrame {
                    dragging: true,
                    pos: at(10.0 + f32::from(step) * 5.0, 10.0),
                    ..PointerFrame::default()
                },
                DragKind::Move,
            );
        }
        let end = g.update(
            PointerFrame {
                drag_stopped: true,
                pos: at(40.0, 25.0),
                ..PointerFrame::default()
            },
            DragKind::Move,
        );
        assert_eq!(
            end,
            GestureOutcome::Move {
                delta: Vec2::new(30.0, 15.0),
                phase: Phase::Complete,
            }
        );
    }

    /// ★ **An interrupted drag is abandoned, never committed.**
    ///
    /// Focus loss, a dialog, the pointer leaving the window: egui stops
    /// reporting the drag without ever reporting a stop. Committing on the
    /// next frame that happens to look like a release would apply an edit the
    /// operator never finished.
    #[test]
    fn an_interrupted_drag_is_abandoned_rather_than_committed() {
        let mut g = GestureState::default();
        g.update(
            PointerFrame {
                drag_started: true,
                pos: at(0.0, 0.0),
                ..PointerFrame::default()
            },
            DragKind::Move,
        );
        let out = g.update(PointerFrame::default(), DragKind::Move);
        assert_eq!(out, GestureOutcome::Idle);
        assert_eq!(
            g.active(),
            None,
            "the gesture must not survive its interruption"
        );
    }

    /// ★ **Escape abandons a drag in flight, and it commits nothing.**
    ///
    /// The gesture ladder's escape hatch: a move drag that is halfway across
    /// the page and clearly wrong must be abandonable without an undo. The
    /// frame that carries the cancel produces `Cancelled` — never a
    /// `Complete`, which is the outcome that would have rewritten the page.
    #[test]
    fn escape_abandons_a_drag_in_flight_without_committing() {
        for kind in [
            DragKind::Move,
            DragKind::Marquee,
            DragKind::Resize(Grip::SouthEast),
        ] {
            let mut g = GestureState::default();
            g.update(
                PointerFrame {
                    drag_started: true,
                    pos: at(0.0, 0.0),
                    ..PointerFrame::default()
                },
                kind,
            );
            g.update(
                PointerFrame {
                    dragging: true,
                    pos: at(80.0, 40.0),
                    ..PointerFrame::default()
                },
                kind,
            );
            let out = g.update(
                PointerFrame {
                    // egui still reports the button as down: the operator has
                    // not let go, they have changed their mind.
                    dragging: true,
                    pos: at(80.0, 40.0),
                    cancel: true,
                    ..PointerFrame::default()
                },
                kind,
            );
            assert_eq!(out, GestureOutcome::Cancelled, "{kind:?}");
            assert_eq!(g.active(), None, "{kind:?} survived its cancellation");
        }
    }

    /// …and the release that follows a cancel commits nothing either. The
    /// operator's finger comes off the button some frames later, and by then
    /// there is no gesture for `drag_stopped` to complete.
    #[test]
    fn the_release_after_a_cancel_commits_nothing() {
        let mut g = GestureState::default();
        g.update(
            PointerFrame {
                drag_started: true,
                pos: at(0.0, 0.0),
                ..PointerFrame::default()
            },
            DragKind::Move,
        );
        g.update(
            PointerFrame {
                dragging: true,
                pos: at(90.0, 0.0),
                cancel: true,
                ..PointerFrame::default()
            },
            DragKind::Move,
        );
        let out = g.update(
            PointerFrame {
                drag_stopped: true,
                pos: at(90.0, 0.0),
                ..PointerFrame::default()
            },
            DragKind::Move,
        );
        assert_eq!(
            out,
            GestureOutcome::Idle,
            "a cancelled drag must not commit on its release"
        );
    }

    /// ★ **Escape with no drag under it is NOT consumed**, so it reaches the
    /// selection ladder and one press still ascends exactly one rung.
    #[test]
    fn escape_with_no_drag_leaves_the_key_for_the_ladder() {
        let mut g = GestureState::default();
        assert_eq!(
            g.update(
                PointerFrame {
                    cancel: true,
                    pos: at(5.0, 5.0),
                    ..PointerFrame::default()
                },
                DragKind::Marquee,
            ),
            GestureOutcome::Idle,
            "reporting Cancelled here would make Escape need two presses to \
             leave a rung"
        );
    }

    /// A cancel arriving on the same frame as a fresh press abandons the old
    /// gesture and does not let the new one inherit it — the new press starts
    /// cleanly on the frame after.
    #[test]
    fn a_cancel_on_a_press_frame_does_not_resurrect_the_old_drag() {
        let mut g = GestureState::default();
        g.update(
            PointerFrame {
                drag_started: true,
                pos: at(0.0, 0.0),
                ..PointerFrame::default()
            },
            DragKind::Move,
        );
        let out = g.update(
            PointerFrame {
                drag_started: true,
                pos: at(500.0, 500.0),
                cancel: true,
                ..PointerFrame::default()
            },
            DragKind::Move,
        );
        assert_eq!(out, GestureOutcome::Cancelled);
        assert_eq!(g.active(), None);
    }

    /// A press with no pointer position starts nothing — a trackpad gesture
    /// can arrive with the pointer off-window, and a drag anchored at a
    /// fabricated origin would move an object by a page's width.
    #[test]
    fn a_press_with_no_position_starts_no_drag() {
        let mut g = GestureState::default();
        let out = g.update(
            PointerFrame {
                drag_started: true,
                pos: None,
                ..PointerFrame::default()
            },
            DragKind::Move,
        );
        assert_eq!(out, GestureOutcome::Idle);
        assert_eq!(g.active(), None);
    }
}
