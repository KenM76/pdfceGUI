//! # `app::conditions` — what the shell may ask about the application
//!
//! One function. It publishes the set of **named conditions** the ribbon
//! evaluates every frame to decide whether a control is enabled and whether
//! it renders pressed.
//!
//! ## Why the vocabulary is names rather than closures
//!
//! `egui_shell` stores `Enable::When("doc.pages")` — a string — and asks this
//! set whether it is present. Data rather than a closure, because a name is
//! serializable (so an operator's customized manifest can reference it),
//! testable headlessly, and cannot capture state that would make a command's
//! availability depend on *when* it was registered.
//!
//! The cost of that choice lands here: every name is a **promise the
//! application has to keep**, and the only thing keeping the two halves in
//! step is `shell::commands`' `KNOWN` list and the test that walks it.
//!
//! ## Why this is its own file
//!
//! Split from `app/mod.rs` when that file crossed the 1,500-line gate for the
//! second time — the first split produced `app/dispatch.rs`. The seam is the
//! same shape as that one and just as real: `mod.rs` composes a frame,
//! `dispatch.rs` answers *what does this verb do*, and this file answers
//! *what is true right now*. They change for different reasons.
//!
//! ## ★ Two sources, one convention
//!
//! Conditions come from two different places and it matters that they arrive
//! the same way:
//!
//! * **application state** — `doc.open`, `doc.pages`, `selection.any`,
//!   `selection.bounds`, and the page-display and view-chrome pressed states,
//!   all read from `PdfceApp` and the open document; and
//! * **`egui::Memory`** — the armed canvas tool and the armed region zoom,
//!   which is why this function takes an `egui::Context`.
//!
//! The second source is the reason this function grew a parameter. Three
//! separate pieces of work recorded that the hand tool and the region zoom had
//! no pressed state, and each declined to invent a mechanism for it — rightly.
//! The alternative was a shadow copy of the armed tool on `PdfceApp` that the
//! canvas would have to remember to update, which puts the truth in two places
//! and fails as a ribbon that says Hand while the canvas selects: a
//! disagreement no test catches, because each half is self-consistent.

use crate::app::PdfceApp;
use crate::app::state::Status;

impl PdfceApp {
    /// The conditions the ribbon evaluates its predicates against.
    ///
    /// Rebuilt every frame because that is what it describes — the state
    /// *this* frame is drawn from. Five conditions, and the set is closed:
    /// `crate::shell::commands` has a test asserting no predicate names
    /// anything outside it, so a typo in a manifest cannot silently
    /// produce a control that is disabled forever.
    ///
    /// # ★ `selection.any` is published from here, and only now
    ///
    /// It was deliberately absent while the selection lived in
    /// `egui::Memory`: this function has no `egui::Context`, so it could not
    /// have read the selection even if it wanted to, and publishing a
    /// condition it could not evaluate would have armed a **destructive**
    /// control that could not work — the inverse of the no-placeholders rule
    /// and the exact shape of defect D1.
    ///
    /// The selection now lives on [`state::OpenDoc`], so the answer is one
    /// field read. Two surfaces come alive with it, both of which the manifest
    /// has been carrying unpowered: the contextual **Format** tab
    /// (`visible_when: "selection.any"`, which is the appear-on-selection
    /// affordance `RIBBON_IA.md` §5.8 calls the single largest usability
    /// change) and the **Delete** inside it (`enabled_when` the same). One
    /// spelling, one source — see `shell::manifest::format::VISIBLE_WHEN`.
    ///
    /// **The Objects panel's focus is not a selection and must never satisfy
    /// this**, which is what `panels::PanelsState::focus`'s own test asserts
    /// through the enable machinery: a panel row being focused must not arm a
    /// destructive command, because the operator would have no way to tell
    /// which of two "selections" it was about to act on. This reads
    /// `doc.selection` and nothing else.
    /// `pub(super)` rather than private: this moved out of `app/mod.rs` and
    /// its three callers stayed. Deliberately NOT `pub` — nothing outside
    /// `app` may publish or read the condition set, because a second producer
    /// is how a control comes to be enabled by one rule and drawn by another.
    pub(super) fn conditions(&self, ctx: &egui::Context) -> egui_shell::commands::ConditionSet {
        let mut set = egui_shell::commands::ConditionSet::new();
        if let Status::Open(doc) = &self.status {
            set.set("doc.open");
            if !doc.pages.is_empty() {
                set.set("doc.pages");
            }
            if !doc.selection.is_empty() {
                set.set("selection.any");
            }
            // ★ `selection.bounds` is NOT `selection.any`, and the gap
            // between them is a real state rather than a defensive check.
            //
            // A selection is an identity — page, object, subpath, node —
            // and identities can outlive the box they described: the
            // selection may name an object on a page that is no longer
            // shown, or one whose index an edit renumbered. `selection.any`
            // is then true and there is nothing to frame.
            //
            // Zoom-to-selection is the one command where that difference is
            // visible, because framing "nothing" is not a no-op — it is a
            // jump to the origin at some arbitrary scale, which looks
            // exactly like a bug and loses the operator's place. So the
            // control greys instead, and it asks the same function the
            // grips are laid out on, so what greys and what is drawn can
            // never disagree.
            if crate::canvas::zoom::can_zoom_to_selection(doc) {
                set.set("selection.bounds");
            }
            // ★ **The page-display radio's pressed position.**
            //
            // `egui_shell::ribbon::selected_condition` is the framework's
            // convention for "this command is currently ON", and
            // `render_command` reads it to draw the button pressed. Without
            // this line View ▸ Page display is four buttons with no indication
            // of which one you are in — which for a radio is not a cosmetic
            // gap, it is the control's entire state.
            //
            // Exactly one is ever set, because `view.display` is one enum
            // value and `page_display_command` is a total function over it.
            // That is what makes it a radio rather than four toggles, and it
            // is asserted from the registry side by
            // `shell::commands::tests::every_page_display_mode_has_a_registered_command`.
            set.set(egui_shell::ribbon::selected_condition(
                crate::shell::commands::page_display_command(doc.view.display),
            ));
            // ★ **The three View ▸ Display toggles' pressed state.**
            //
            // Between zero and three of these are set, where exactly one
            // page-display condition above always is — which is the whole
            // difference between three switches and one three-position
            // control, expressed in the conditions rather than in the drawing.
            //
            // Rulers, grid and guides live on `doc.view`; the hand tool and
            // the armed region zoom live in `egui::Memory` and are published
            // below. Both routes end in the same `selected_condition`, which
            // is what kept a second mechanism from being invented for either.
            for &chrome in crate::app::actions::ViewChrome::ALL {
                if chrome.read(&doc.view) {
                    set.set(egui_shell::ribbon::selected_condition(
                        crate::shell::commands::chrome_command(chrome),
                    ));
                }
            }
        }

        // ★ **The two toggles whose state lives in `egui::Memory`.**
        //
        // These were the last controls in the ribbon with no pressed state,
        // and the reason was structural rather than an oversight: this
        // function took `&self` and no `egui::Context`, so a toggle whose
        // state is in egui's own memory had no route here at all. Three
        // separate pieces of work recorded the gap and declined to invent a
        // second mechanism for it, which was right — the fix is to hand this
        // function the context, not to keep a shadow copy of the tool on
        // `PdfceApp` that the canvas would then have to remember to update.
        //
        // A shadow copy is worth naming as the road not taken, because it is
        // the obvious one: it would put the truth about which tool is armed in
        // two places, and the failure mode is a ribbon that says Hand while
        // the canvas selects — a disagreement no test would catch, because
        // each half would be self-consistent.
        //
        // **Outside the `Status::Open` arm on purpose.** The armed tool and
        // the armed zoom survive closing a document, so a ribbon that forgot
        // which tool you were in the moment you closed a file would be
        // reporting something untrue about its own state. The commands
        // themselves are gated on `doc.pages`, so they still grey out with
        // nothing open — greyed and pressed is exactly right for "this is the
        // tool you are in, and there is nothing to use it on".
        if crate::canvas::tool::selected(ctx) == crate::canvas::tool::CanvasTool::Hand {
            set.set(egui_shell::ribbon::selected_condition("view.tool_hand"));
        }
        if crate::canvas::zoom::region_zoom_armed(ctx) {
            set.set(egui_shell::ribbon::selected_condition("view.zoom_region"));
        }

        // `undo.available` and `redo.available` are still deliberately absent:
        // there is no undo stack to report on yet. Setting them would arm
        // controls that cannot work. They arrive with their subsystem.
        set
    }
}
