//! # `app::actions::annot` — the verbs whose subject is a whole annotation
//!
//! Move it, resize it, remove it. Split out of [`super::action`] under **R2**
//! on 2026-08-28, when `ResizeAnnotation` grew the operator's Tool-row scale
//! switches and took that file past 1,500 lines for the fifth time.
//!
//! ## ★★ Why THIS family, when the file's header still names markup
//!
//! [`super::action`]'s header pre-measured **markup** as the next sub-enum and
//! it is still the largest. The rule it states is *"the next family of variants
//! to **grow**"*, and today that was this one — the same reading that took the
//! text family out this morning. The markup measurement stands and is still the
//! answer the day markup grows.
//!
//! ## ★★★ What these three share, and it is not "they are annotations"
//!
//! **None of them takes a page index.** Every other authoring verb in this
//! crate does. The reason is a property of the engine's annotation verbs:
//! `move_annotation`, `resize_annotation` and `delete_annotation` all find
//! their operand by **stable object id**, so a page number would be a second
//! way of naming a thing that is already named — and one that goes wrong the
//! moment a page is reordered between the gesture and the queue draining.
//!
//! ⇒ That is why `Delete` carries a page and the other two do not: its page is
//! for the **trace and the disclosure**, not for finding the annotation. The
//! asymmetry is real and is documented on the variant rather than smoothed
//! away, because smoothing it would mean adding a page to two verbs that must
//! not use one.
//!
//! ## ★ `CommitMarkup` and `PasteMarkup` are deliberately NOT here
//!
//! They **author** an annotation, which needs a page, a spec and a pen. These
//! three act on one that exists. Authoring and editing are different subjects
//! however much they share a noun, and a sub-enum drawn around the noun rather
//! than around the subject would be the larger of the two families and the less
//! useful one.

/// The verbs whose subject is a whole annotation that already exists.
#[derive(Debug, Clone, PartialEq)]
pub enum AnnotAction {
    /// ★★★ **Move a markup annotation by a page-space delta**, as one undoable
    /// command.
    ///
    /// Raised by `crate::canvas::annotdrag` on the release of a drag, and by
    /// nothing else. **That module's header is the argument** for why this
    /// carries a delta rather than a new rectangle, and it is not repeated
    /// here: a `/Rect` names only the half of a move a renderer can see, and
    /// the absolute-coordinate geometry keys — which any *other* tool rebuilds
    /// an appearance from — are the half that would be silently left behind.
    ///
    /// ★ No page, for [`Self::DeleteAnnotation`]'s reason inverted: that one
    /// carries a page purely for its trace and its disclosure, and this one
    /// needs neither — `move_annotation` finds the annotation by id, and the
    /// disclosure it owes is about a pop-up rather than a sheet.
    Move {
        /// The annotation, by stable object id.
        id: pdfce_core::object::ObjId,
        /// Horizontal displacement, PDF points.
        dx: f64,
        /// Vertical displacement, PDF points. **Positive is up** -- y increases
        /// upward in PDF user space (§8.3.2.3).
        dy: f64,
    },
    /// ★★★ **Scale a markup annotation about an anchor**, as one undoable
    /// command. `OPERATOR_REQUESTS.md` **O51**.
    ///
    /// Raised by `crate::canvas::resizing` on the release of a grip drag, and
    /// by nothing else. **Anchor plus FACTORS, not a target rectangle** — the
    /// shape this shell asked the engine for so it would match
    /// `transform_objects`, and the argument is at that call site.
    ///
    /// ★★ [`Self::uniform`] travels because the engine **asked for it by
    /// name**, and it is not a number the engine could derive equally well: it
    /// reports what the operator did with their hand — a Shift-constrained
    /// corner drag versus a free edge drag. Neither PDF nor SVG has a per-axis
    /// stroke width, so a non-uniform scale of a foreign appearance produces an
    /// anisotropic border by arithmetic, and that case is refused rather than
    /// silently distorted.
    Resize {
        /// The annotation, by stable object id.
        id: pdfce_core::object::ObjId,
        /// The point that stays still, in PDF page space — the corner
        /// **opposite** the grip that was grabbed.
        anchor: (f64, f64),
        /// Horizontal scale factor.
        sx: f64,
        /// Vertical scale factor.
        sy: f64,
        /// Whether the two factors are equal. See the variant docs.
        uniform: bool,
        /// ★★ **The operator's Tool-row switches, CARRIED rather than read at
        /// apply time** — `OPERATOR_REQUESTS.md` O51.
        ///
        /// The same rule `CommitMarkup` follows and for its stated reason: a
        /// resize is raised by a gesture that completed frames before the queue
        /// drains, so a value read at apply time is a value that may have moved
        /// under it. `CommitTextAnnot` reads its pen live instead, and its own
        /// comment says why that is safe there — it is raised by a dialog the
        /// operator is sitting in, on the frame they press Accept.
        ///
        /// ★ Nobody can tick a checkbox during a drag, so the two would agree
        /// today. Carrying it is what keeps that an observation rather than a
        /// dependency.
        modifiers: crate::canvas::scaling::Modifiers,
    },
    Delete {
        /// The page it is on — for the trace and the disclosure, not for the
        /// verb, which finds the annotation by id wherever it lives. A reply
        /// may sit on a different page from the comment it replies to, so a
        /// page-scoped delete would miss it.
        page: usize,
        /// The annotation, by stable object id.
        id: pdfce_core::object::ObjId,
    },
}
