//! [`Ribbon`] — the builder and the one entry point that draws a frame.
//!
//! # ★ THE SEAM: the shell reports intent; the application dispatches
//!
//! [`Ribbon::show`] returns `Vec<HandlerToken>` — the tokens of the
//! commands the operator invoked this frame, in the order the controls
//! were drawn. **It runs nothing.** There is no handler in this crate, no
//! `dyn FnMut(&mut AppState)` in the registry, and no path by which
//! drawing a ribbon can change an application's document.
//!
//! That is a deliberate architectural choice with three consequences, and
//! it is worth being explicit about all three because the alternative — a
//! registry of closures — is the obvious design and is what most toolkits
//! do.
//!
//! 1. **The shell stays domain-free.** A registry holding
//!    `Box<dyn FnMut(&mut AppState)>` has to name `AppState`. A shell that
//!    names the application's state is not a reusable shell, and
//!    `tools/gates/check-shell-purity.sh` exists precisely to keep that
//!    edge from being drawn.
//! 2. **Dispatch stays at one choke point.** The application receives a
//!    list of tokens and matches on it in **one place** — which is where a
//!    confirmation gate, an undo entry, a trace or a "this document is
//!    encrypted" refusal belongs. A registry of closures scatters that
//!    across as many sites as there are commands, and the fifth one
//!    somebody adds is the one that forgets the undo entry.
//! 3. **The ribbon is testable with no application at all.** Every test in
//!    [`super::tests`] renders a real ribbon into a real `egui::Context`
//!    and asserts on the tokens that come back. Nothing has to be stubbed,
//!    because there is nothing to stub.
//!
//! The cost, accepted explicitly in `SHELL_FRAMEWORK.md` §6, is one
//! indirection between a button and its handler. In exchange, the sentence
//! *"the shell renders a tab called Measure and routes a command called
//! `measure.linear`; it has no idea what either means"* stays literally
//! true.
//!
//! ```no_run
//! # use egui_shell::{CommandRegistry, Shell, ribbon::{Ribbon, RibbonState}};
//! # fn dispatch(_: egui_shell::HandlerToken) {}
//! # fn frame(ui: &mut egui::Ui, shell: &Shell, registry: &CommandRegistry, state: &mut RibbonState) {
//! for token in Ribbon::show(ui, shell, registry, state) {
//!     dispatch(token); // ← the application's one choke point
//! }
//! # }
//! ```
//!
//! # ★ `entitled`: the one line of this file that is load-bearing
//!
//! ```text
//! let entitled = ui.max_rect();   // BEFORE a single widget is drawn
//! ```
//!
//! `egui`'s `Region::expand_to_include_rect` grows a `Ui`'s **`max_rect`**,
//! not merely its `min_rect`, whenever a child lays out beyond it. The
//! ribbon draws two rows into one vertical `Ui`. If the first row
//! overflowed — which is the entire situation the overflow machinery
//! exists for — the enclosing `Ui` would silently widen, and the band
//! drawn next would ask `available_rect_before_wrap()` and be told it has
//! a width the window never had. It would then reserve its overflow
//! affordance from a right edge off screen: the affordance present,
//! reported, and unclickable.
//!
//! So the entitlement is read here, once, before anything exists that
//! could have inflated it, and is threaded down to both rows. Both
//! [`super::strip::render`] and [`super::band::render_band`] take it as an
//! argument rather than deriving it, because the only `Ui` that knows it is
//! the one the application passed in. See
//! [`super::band::entitled_bounds`].
//!
//! # Frame ordering, and the one thing that lands next frame
//!
//! Everything the ribbon draws reflects the state at the **start** of the
//! frame. A tab click or a mode change is written into
//! [`super::RibbonState`] and takes effect on the **next** frame.
//!
//! That is deliberate. A mode change alters which tabs exist; applying it
//! mid-frame would mean the tab strip already drawn no longer matches the
//! band about to be drawn, and the operator would see one frame of a
//! ribbon that never validly existed. `egui` repaints after any click, so
//! the delay is one frame and is not perceptible — whereas an inconsistent
//! frame is exactly the kind of flicker that gets reported as "the ribbon
//! glitches when I switch modes".

use crate::commands::{CommandRegistry, ConditionSet, HandlerToken};
use crate::manifest::Shell;
use crate::theme::Theme;

use super::ctx::{Ctx, CustomItem, CustomItemRenderer, IconPainter, IconRequest};
use super::report::{RectSink, Reporter};
use super::{FrameReport, RibbonState, band, strip, tabs};

/// Draws a [`Shell`]. See this module's header for the seam it sits on.
///
/// The plain entry point is [`Ribbon::show`]. The builder exists for the
/// four optional capabilities an application may supply, each of which is
/// a seam that keeps a domain concern out of the shell:
///
/// | Builder method | Supplies | Why the shell cannot do it itself |
/// |---|---|---|
/// | [`Self::with_conditions`] | what is true this frame | The shell has no state to derive it from. |
/// | [`Self::with_icon_painter`] | how to draw an icon key | An icon set is a licensing and rasterization decision. |
/// | [`Self::with_custom_items`] | how to draw a non-button control | Otherwise the item vocabulary grows a variant per widget. |
/// | [`Self::reporting_rects_to`] | where to publish drawn rects | Only the harness knows what it wants to assert. |
///
/// All four are optional and all four default to "off", so the
/// four-argument form is a complete, working ribbon.
#[derive(Default)]
pub struct Ribbon<'a> {
    conditions: Option<&'a ConditionSet>,
    rects: Option<&'a mut RectSink<'a>>,
    icons: Option<&'a mut IconPainter<'a>>,
    custom: Option<&'a mut CustomItemRenderer<'a>>,
}

impl<'a> Ribbon<'a> {
    /// A ribbon with no optional capabilities.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Publish what is true this frame, for enable predicates, contextual
    /// tab visibility and toggle state.
    ///
    /// Without this every [`crate::commands::Enable::When`] command is
    /// disabled and every contextual tab is hidden — which is the correct
    /// answer for an empty condition set, and is why an application that
    /// forgets this sees a greyed-out ribbon rather than a wrong one.
    #[must_use]
    pub fn with_conditions(mut self, conditions: &'a ConditionSet) -> Self {
        self.conditions = Some(conditions);
        self
    }

    /// Publish every drawn caption, segment, tab and QAT control rect to
    /// `sink`, under the stable names in [`super::report`].
    #[must_use]
    pub fn reporting_rects_to(mut self, sink: &'a mut (impl FnMut(&str, egui::Rect) + 'a)) -> Self {
        self.rects = Some(sink);
        self
    }

    /// Supply the painter for [`crate::commands::Command::icon`] keys.
    ///
    /// Without one, controls draw their labels and no glyphs. That is a
    /// working ribbon, which is the point: an application can bring the
    /// ribbon up before it has an icon set.
    #[must_use]
    pub fn with_icon_painter(
        mut self,
        painter: &'a mut (impl FnMut(&egui::Painter, &IconRequest<'_>) + 'a),
    ) -> Self {
        self.icons = Some(painter);
        self
    }

    /// Supply the renderer for [`crate::manifest::Item::Custom`].
    #[must_use]
    pub fn with_custom_items(
        mut self,
        renderer: &'a mut (impl FnMut(&mut egui::Ui, &CustomItem<'_>) -> Option<HandlerToken> + 'a),
    ) -> Self {
        self.custom = Some(renderer);
        self
    }

    /// Draw a ribbon with no optional capabilities and report the commands
    /// the operator invoked.
    ///
    /// The shell **executes nothing**: the returned tokens are intent, and
    /// the application dispatches them at its own choke point. See this
    /// module's header.
    pub fn show(
        ui: &mut egui::Ui,
        shell: &Shell,
        registry: &CommandRegistry,
        state: &mut RibbonState,
    ) -> Vec<HandlerToken> {
        Ribbon::new().render(ui, shell, registry, state)
    }

    /// Draw the ribbon and report the commands the operator invoked.
    ///
    /// Consumes the builder, because it holds `&mut` borrows of the
    /// application's callbacks for exactly the duration of one frame.
    pub fn render(
        self,
        ui: &mut egui::Ui,
        shell: &Shell,
        registry: &'a CommandRegistry,
        state: &mut RibbonState,
    ) -> Vec<HandlerToken> {
        // The stand-in for an application that supplied no conditions.
        //
        // A `static` rather than a local because `Ctx` holds `&'a
        // ConditionSet` for the same `'a` as the application's callbacks,
        // and a local would not live that long. It is an immutable empty
        // set, built at most once per process, and reading it says exactly
        // what it means: nothing is true, so every `Enable::When` command
        // is disabled and every contextual tab is hidden.
        static NO_CONDITIONS: std::sync::OnceLock<ConditionSet> = std::sync::OnceLock::new();
        let conditions = self
            .conditions
            .unwrap_or_else(|| NO_CONDITIONS.get_or_init(ConditionSet::new));

        let mut ctx = Ctx {
            registry,
            conditions,
            theme: Theme::of(ui.ctx()),
            reporter: Reporter::new(self.rects),
            icons: self.icons,
            custom: self.custom,
            base_id: state.base_id,
            invoked: Vec::new(),
        };

        // Resolve what is on screen BEFORE anything is drawn, so the tab
        // strip and the band cannot disagree about it.
        let visible = tabs::visible_tabs(shell, state.mode.as_deref(), conditions);
        let active = tabs::resolve_active(&visible, state.active_tab.as_deref());
        let active_id = active.map(|t| t.id.clone());
        state.active_tab = active_id.clone();

        // ★ The width the ribbon was actually given, read BEFORE a single
        // widget is drawn into `ui`. See this module's header — this line
        // is why both rows take a rectangle rather than asking for one.
        let entitled = ui.max_rect();

        let mut band_outcome = band::BandOutcome::default();
        let mut strip_outcome = strip::StripOutcome::default();

        ui.vertical(|ui| {
            strip_outcome = strip::render(
                ui,
                &mut ctx,
                shell,
                &visible,
                active_id.as_deref(),
                state.mode.as_deref(),
                entitled,
            );

            tabs::strip_underline(ui, &ctx);

            if let Some(tab) = active {
                band_outcome = band::render_band(ui, &mut ctx, &tab.id, tab.groups(), entitled);
            }
        });

        // Applied after drawing, so they land on the next frame — see the
        // module header on why a mid-frame mode change would produce a
        // ribbon that never validly existed.
        if let Some(id) = strip_outcome.clicked_tab.take() {
            state.active_tab = Some(id);
        }
        if let Some(id) = strip_outcome.chosen_mode.take() {
            state.mode = Some(id);
            // A mode change may retire the active tab; `resolve_active`
            // handles that on the next frame, which is why nothing is
            // cleared here.
        }

        let invoked = ctx.invoked;
        state.last_frame = FrameReport {
            active_tab: state.active_tab.clone(),
            mode: state
                .mode
                .clone()
                .or_else(|| shell.modes().first().map(|m| m.id.clone())),
            tabs_visible: visible.len(),
            tabs_in_strip: strip_outcome.tabs_in_strip,
            tabs_overflowed: strip_outcome.tabs_overflowed,
            tab_overflow_visible: strip_outcome.tab_overflow_visible,
            tab_strip_collapsed: strip_outcome.tab_strip_collapsed,
            tab_overflow_id: strip_outcome.tab_overflow_id,
            groups_rendered: band_outcome.groups_rendered,
            groups_in_band: band_outcome.groups_in_band,
            captions_emitted: band_outcome.captions_emitted,
            groups_overflowed: band_outcome.groups_overflowed,
            overflow_visible: band_outcome.overflow_visible,
            overflow_id: band_outcome.overflow_id,
            commands_invoked: invoked.len(),
        };
        invoked
    }
}

impl std::fmt::Debug for Ribbon<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ribbon")
            .field("conditions", &self.conditions.is_some())
            .field("rects", &self.rects.is_some())
            .field("icons", &self.icons.is_some())
            .field("custom", &self.custom.is_some())
            .finish()
    }
}
