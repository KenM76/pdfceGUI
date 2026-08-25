//! # `app::prefs` — the shell's own preferences, as distinct from the engine's settings
//!
//! ## ★ Why this is not `pdfce_core::settings`
//!
//! That store has a stated purpose and this is not it. Its own window says so
//! in its first paragraph: *"The PDF standard leaves some things genuinely
//! undefined … Where that happens, pdfce asks you rather than deciding
//! quietly."* Every one of its thirteen entries exists because a **standard
//! declines to have an opinion**, and each one states what clause is silent.
//!
//! How sharp a page is rasterised is not that. Nothing in ISO 32000-1 is silent
//! about it; it is a **preference**, a trade of sharpness against time that
//! depends on the operator's machine and on how big their drawings are. Filing
//! it beside the CMYK conversion intent would make the settings file's own
//! framing dishonest — and would put a value with no clause number in a file
//! whose every entry cites one.
//!
//! `canvas::markup::pen`'s header already named this module before it existed:
//!
//! > Persisting it belongs with the ribbon layout and the keymap, under the
//! > same `userdata/` roof and in their own file.
//!
//! ## Same roof, same shape, same fail-soft contract
//!
//! It sits in the directory `pdfce_core::settings::resolve_store` resolves,
//! beside `settings.txt` and `layout.ron`, so the update instructions —
//! *"replace the program files, keep your `userdata` folder"* — cover it
//! without being reworded.
//!
//! The format is the engine's: flat `key = value`, `#` comments, and **per-key
//! recovery**. That last point is the one worth copying rather than a
//! convenience: an unknown key is left in the file and reported, a bad value
//! falls back for that key alone, and one bad line never discards the rest.
//! The file is meant to be hand-editable, and a parser that fails a whole
//! document over one typo punishes the operator for using it.
//!
//! ## ★ Why the RENDER settings were two and not seven
//!
//! `RIBBON_IA.md` §5.2 commissioned a View ▸ Render group of five, plus two
//! behaviour settings, and `shell::manifest`'s `DIRECTED` list carried all
//! seven as *"named individually, with their value sets and their defaults,
//! when this shell was commissioned"*. They were registered, drawn on the
//! ribbon, and inert.
//!
//! Checked against the engine on 2026-08-17, only two can be honoured:
//!
//! | commissioned | verdict |
//! |---|---|
//! | **Render quality** | ✅ [`RenderQuality`] — a raster-scale multiplier. `viewer::raster_scale` was `zoom × pixels_per_point` exactly, with no multiplier at all, so this is new capability rather than an exposed constant |
//! | **Zoom settle delay** | ✅ [`Prefs::zoom_settle_ms`] — `render::settle::ZOOM_SETTLE` was a compiled-in 150 ms |
//! | Render strategy (whole page · tiled progressive) | ❌ there is no tiled-progressive path in this shell. `pdfce_render::render_page_region` exists, so it is buildable — but it is a rendering **architecture**, not a setting, and a radio offering it would be an affordance for a code path that does not exist |
//! | Thin lines | ❌ `RenderOptions` has no such field. Verified by reading its eleven public fields |
//! | Antialiasing | ❌ `interpret.rs` sets `anti_alias: true` as a literal at two call sites and `RenderOptions` exposes no knob. (`shading.rs`'s `anti_alias` is the *document's* `/AntiAlias` key — a property of the shading pattern, not a viewer preference, and honouring it is correct.) |
//! | Floating panels (Off · Allowed) | ❌ `egui-shell`'s dock has no floating mode. Its only `floating` is `egui`'s scroll-bar style |
//! | App initiative (Never · Ask · Allowed) | ❌ **the setting has nothing to gate.** Nothing in this build opens a surface unasked — which is the specified default, *Never*, already true by construction. A control whose only value is the one already in force is a control that does nothing |
//!
//! The last row is the interesting one, and it is why this table is here
//! rather than in a commit message: `app_initiative`'s absence is not a gap.
//! It is a setting that would exist to switch off a behaviour pdfce does not
//! have. Building it would mean building the behaviour first.
//!
//! `DIRECTED`'s own doc comment anticipated exactly this outcome — *"if it
//! turns out to be wrong, the fix is deleting eight rows from one list rather
//! than re-deriving which entries were deliberate"* — and that is what
//! happened.
//!
//! ## ★ …and then two more arrived, from the opposite direction
//!
//! [`opening`]'s two preferences — how the first page is fitted, and which
//! overlays are already on — were **not** commissioned by `RIBBON_IA.md`. They
//! came out of the `NO_SURFACE.md` sweep, which is the inventory of *every
//! tunable an operator would plausibly want to change and cannot*, and they are
//! the two rows in it that cost an operator something on **every document they
//! ever open** rather than once.
//!
//! That contrast is worth carrying, because it says where the next preference
//! will come from. The commissioned list was written from the outside, before
//! the shell existed, and five of its seven turned out to name nothing. The
//! sweep was written from the inside, by reading the constants the code
//! actually holds, and both of its candidates were real. **An inventory of what
//! the program does beats a wishlist of what it might.**
//!
//! ## The two stores are two files, and the operator never finds out
//!
//! `dialogs::settings::Draft` edits both and the window has one Save and one
//! Cancel. See its `working_prefs` field, which states the rule: *"one Cancel
//! discards both, one Save writes both, and `is_dirty` is true if either
//! moved."*

/// How big the **program's own controls** are drawn — the one accessibility
/// preference, and the only one here that changes nothing about the document.
/// ★★ How much memory pdfce may spend so a page it has already drawn does not
/// have to be drawn again.
///
/// Its header carries the defect it exists for and the part of that defect a
/// reader would otherwise carry out wrongly: the cache pruned itself to the
/// VISIBLE SET on every frame, so the budget had never bitten, and raising the
/// number alone would have changed nothing at all.
pub mod cache;
pub mod chrome;
/// What an operator is shown when a page **first appears** — read once per
/// document open, never on the hot path.
pub mod opening;
// What a plain wheel does when the document is not one long scroll -- O30.
/// How sharply a page is drawn, and how long zoom waits before drawing it.
/// The two preferences that change what a **frame costs**.
pub mod quality;
pub mod smoothing;
pub mod wheel;

use std::path::PathBuf;

pub use cache::PageCache;
pub use chrome::{DEFAULT_UI_SCALE, MAX_UI_SCALE, MIN_UI_SCALE, UI_SCALE_STEP};
pub use opening::{OpeningFit, PageChrome};
pub use quality::{DEFAULT_SETTLE_MS, MAX_SETTLE_MS, MIN_SETTLE_MS, RenderQuality};
pub use wheel::WheelPaging;

/// The shipped maximum zoom, as a percentage.
///
/// ★★ **The maximum, on the operator's instruction of 2026-08-22** — *"Also
/// set the default to be able to hit the maximum zoom."*
///
/// It was 800 % for one build, chosen so a fresh install behaved exactly as
/// the shell had before the setting existed. That was the cautious call and he
/// overruled it, consistently with his earlier one: *"it is up to the user to
/// determine how much of a performance hit they want to take."* A capability
/// he has to find a preferences file to switch on is a capability most of its
/// users never have.
///
/// ★ What this does NOT change is the behaviour he cares about. The ceiling is
/// permission, not policy: `viewer::zoom_ceiling` still lets the whole-page
/// raster bind wherever it can, so **panning stays instant at every zoom that
/// could render whole-page before** — the region path engages only above it,
/// where the alternative is not a slower zoom but no zoom at all.
pub const DEFAULT_MAX_ZOOM_PERCENT: f32 = MAX_MAX_ZOOM_PERCENT;

/// The lowest a maximum-zoom setting may be. Below this the operator could
/// configure a document they cannot magnify at all.
pub const MIN_MAX_ZOOM_PERCENT: f32 = 10.0;

/// The highest a maximum-zoom setting may be — **a hundred billion percent**,
/// which is the deepest zoom the page has been confirmed to actually DRAW at.
///
/// ★★ The operator named a trillion, and a trillion very nearly works: driving
/// to it renders cleanly with no failed rasters. What it does not do is put a
/// page on screen. The limit there is no longer the scroll offset — tier 3's
/// `f64` anchor fixed that — but the **strip's own extent**, which is still
/// `page × zoom` in `f32` and reaches 6×10^12 points at a trillion percent on
/// US Letter. Measured by driving: drawn at 8.6×10^9× (859 billion percent),
/// not drawn at 1×10^10×.
///
/// ★ So this is set an order of magnitude inside the confirmed-working range
/// rather than at the edge of it. Offering a rung that renders without error
/// and shows a blank page would be the same defect this feature has refused
/// throughout: a control that accepts a number and then misbehaves.
///
/// Removing this needs the strip to stop being built in `page × zoom` space at
/// deep zoom — the same move tier 3 made for the offset, one layer out.
///
/// ★ It is not a judgement about what is sensible. He was explicit that the
/// performance trade is his to make; this is about what the shell can put on
/// the screen.
pub const MAX_MAX_ZOOM_PERCENT: f32 = 1e12;

/// Format a percentage for the preferences file without an exponent or a
/// trailing `.0`.
///
/// ★ `1e12` is what `f32::to_string` produces for a trillion, and a file the
/// operator opens in a text editor should say `1000000000000`. The file is
/// his to read and edit; a machine-shaped number there is a small rudeness
/// with a real cost, because he cannot tell at a glance what he set.
fn format_percent(value: f32) -> String {
    format!("{value:.0}")
}

/// The file this store is written to, beside `settings.txt`.
// ui-text-exempt: a file name, never displayed.
pub const PREFS_FILE: &str = "preferences.txt";

/// The shell's own preferences.
///
/// ## `PartialEq` but not `Eq` — and it was `Eq` until [`Self::ui_scale`] landed
///
/// A scale is a continuous quantity and `f32` has no total equality, so the
/// derive cannot be kept. Nothing is lost: the only thing that compares two
/// `Prefs` is `dialogs::settings::Draft::is_dirty`, which asks *"has the
/// operator changed anything?"* — and `PartialEq` answers that exactly. `Eq`
/// would additionally promise reflexivity, which the one field that could
/// break it (a `NaN` scale) cannot reach, because [`chrome::normalise_ui_scale`]
/// clamps every value that enters the struct.
#[derive(Debug, Clone, PartialEq)]
pub struct Prefs {
    /// How sharply a page is rasterised.
    pub render_quality: RenderQuality,
    /// ★★ **How much memory the page cache may hold**, so a page already drawn
    /// is not drawn again.
    ///
    /// Read every frame by `crate::render::settle::fill_strip`, which hands it
    /// to `StripRasters::retain` — the one place it is spent. Read live rather
    /// than at open, unlike [`Self::opening_fit`]: shrinking it must take effect
    /// at once, because an operator reaching for a smaller value is an operator
    /// whose machine is already struggling.
    pub page_cache: PageCache,
    /// How long a zoom must stop changing before it is committed to a real
    /// rasterisation, in milliseconds.
    ///
    /// Stored as a number rather than as a `Duration` because that is what the
    /// file holds and what the control edits; `render::settle` converts once,
    /// at the one place it is read.
    pub zoom_settle_ms: u64,
    /// ★★ **The highest zoom the operator wants to be able to reach**, as a
    /// percentage — `OPERATOR_REQUESTS.md` O24.
    ///
    /// > *"add a setting so the user can set the maximum zoom … I'm not
    /// > concerned about the practicality of offering such a high zoom. it is
    /// > up to the user to determine how much of a performance hit they want
    /// > to take."*
    ///
    /// That last sentence is why this has no guard, no warning and no
    /// preflight. The trade is explicitly his; the setting's whole job is to
    /// be honest about what it does and to actually do it.
    ///
    /// ★ It is also the control he asked for to **compare the two rendering
    /// paths**: the shell rasterizes the whole page while it can and switches
    /// to the visible region only when it cannot. Set this low and he never
    /// leaves the whole-page path; set it high and he exercises the region
    /// path. A threshold rather than a mode, which explains itself where a
    /// checkbox would have to be explained.
    ///
    /// Stored as a percentage because that is what the status bar shows and
    /// what he said — *"1,000,000,000,000%"*. `f32` is exact to 2^24, so a
    /// percentage stays whole to 16.7 million; beyond that the stored value
    /// rounds, which is immaterial at zooms where one screen pixel is a
    /// millionth of a point.
    pub max_zoom_percent: f32,
    /// How the first page of a newly opened document is sized to the window.
    ///
    /// ★ Read **once**, by [`Self::seed_view`], in the one place a document is
    /// adopted. Unlike the two above it is not consulted again — changing it
    /// while a document is open must not resize the page the operator is
    /// looking at, because they may have zoomed it deliberately since.
    pub opening_fit: OpeningFit,
    /// Whether pdfce-gui has already applied its own image-smoothing default.
    ///
    /// Bookkeeping, not a preference: it is written once and then only ever
    /// read. See [`smoothing`] for why the flip has to be a migration rather
    /// than a different default, and why the marker lives in THIS file rather
    /// than in the engine's `settings.txt`.
    pub image_smoothing_default_applied: bool,
    /// Which of the three View ▸ Display overlays are already on when a
    /// document opens.
    ///
    /// Read once, with [`Self::opening_fit`], and for the same reason.
    pub chrome: PageChrome,
    /// **What a plain mouse wheel does under a one-page-at-a-time display
    /// mode** -- `OPERATOR_REQUESTS.md` O30.
    ///
    /// Unlike the two above this one is consulted **every frame**, not once at
    /// open: it is a live preference about an input gesture, and an operator
    /// who changes it from the status bar expects the very next notch to obey.
    /// See [`WheelPaging`] for why the choice exists only under
    /// `PageDisplay::Single` and `Facing`.
    pub wheel_paging: WheelPaging,
    /// **How big the program's own controls are drawn**, as a multiplier on
    /// whatever the operating system already asked for.
    ///
    /// # ★ A multiplier, not a size, and the distinction is the whole design
    ///
    /// `egui`'s `Context::set_zoom_factor` multiplies the *native* pixels per
    /// point — the value the window system reports, which on Windows is the
    /// display-scaling percentage the operator set for every application on the
    /// machine. So `1.0` here does not mean *"draw at 96 dpi"*; it means
    /// **"whatever you already decided"**, and this preference expresses only
    /// the delta pdfce needs on top of it.
    ///
    /// That is the correct relationship and it is easy to get backwards.
    /// Storing an absolute point size would make pdfce the one application on
    /// the machine that ignores the display setting — so an operator who moved
    /// a 4K laptop to a 1080p monitor would fix every program but this one.
    ///
    /// # It is not stored as a `Duration`-style integer, unlike its neighbours
    ///
    /// [`Self::zoom_settle_ms`] is a `u64` because the file holds a whole
    /// number of milliseconds. A scale has no such natural unit, and rounding
    /// it to, say, whole percent in the struct would put the rounding rule in
    /// two places — the parser and the control. [`chrome::normalise_ui_scale`]
    /// is the one place instead, applied on the way in.
    ///
    /// # ★ Live-previewed, like the theme, and for the identical reason
    ///
    /// `app::frame`'s step 0 reads this from the **draft** while the settings
    /// window is open. A scale cannot be judged from a number — you choose it
    /// by seeing whether you can read the ribbon — so it is the second of the
    /// two settings in this window that take effect before Save. Cancel drops
    /// the draft and the size reverts with it; there is no separate preview
    /// state that could get out of step with what will be written.
    pub ui_scale: f32,
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            render_quality: RenderQuality::default(),
            page_cache: PageCache::default(),
            zoom_settle_ms: DEFAULT_SETTLE_MS,
            // ★ The shipped default is today's ceiling, so a fresh install
            // behaves exactly as the shell behaved before this existed.
            // Raising it is the operator's decision, which is the whole
            // point of the setting.
            max_zoom_percent: DEFAULT_MAX_ZOOM_PERCENT,
            opening_fit: OpeningFit::default(),
            // ★ FALSE, and it must stay false. This is what makes a fresh
            // preferences file — or one written by a build older than the
            // migration — trigger the flip exactly once. Defaulting it true
            // would silently skip every installation that has run pdfce
            // before, which is all of them, and the migration would be
            // shipped, reported done, and visible to nobody.
            image_smoothing_default_applied: false,
            wheel_paging: WheelPaging::default(),
            chrome: PageChrome::default(),
            ui_scale: DEFAULT_UI_SCALE,
        }
    }
}

/// Why a preference was not applied as written.
///
/// The same shape as `pdfce_core::settings::SettingNote` and for the same
/// reason: the file is hand-editable, so a mistake in it must be findable, and
/// **at its line number**. A message saying only "something was wrong" sends
/// the operator to read the whole file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrefNote {
    /// A key this build does not know. Left in the file, never deleted — it
    /// may belong to a newer pdfce the operator also runs from this folder.
    UnknownKey {
        /// The key as written.
        key: String,
        /// Its 1-based line.
        line: usize,
    },
    /// A value this build could not read. That key alone falls back.
    BadValue {
        /// The key.
        key: String,
        /// What was written.
        value: String,
        /// Its 1-based line.
        line: usize,
    },
    /// A value outside the accepted range, clamped.
    Clamped {
        /// The key.
        key: String,
        /// What was written.
        value: String,
        /// Its 1-based line.
        line: usize,
    },
    /// A line that is not `name = value`. Skipped.
    Malformed {
        /// Its 1-based line.
        line: usize,
    },
}

impl Prefs {
    /// Where the preferences file lives, or `None` if there is nowhere
    /// writable.
    ///
    /// Derived from the same `pdfce_core::settings::resolve_store()` the
    /// settings and the layout use, so the three cannot drift apart — which is
    /// the failure this project already found once, when two callers in one
    /// process disagreed about which home was live and put two files that
    /// belong together in two places.
    #[must_use]
    pub fn path() -> Option<PathBuf> {
        pdfce_core::settings::resolve_store()
            .directory()
            .map(|dir| dir.join(PREFS_FILE))
    }

    /// Load, never failing.
    ///
    /// A missing file, an unreadable one, a broken line or a value out of range
    /// all yield usable preferences with a reason in the returned notes. **A
    /// missing file produces no note**, deliberately: a first run is the
    /// expected state, not a fault, and reporting it would train the operator
    /// to ignore the channel that carries the real problems.
    #[must_use]
    pub fn load() -> (Self, Vec<PrefNote>) {
        let Some(path) = Self::path() else {
            return (Self::default(), Vec::new());
        };
        let Ok(text) = std::fs::read_to_string(&path) else {
            // Unreadable and absent are collapsed here, unlike in the engine's
            // store, and the reason is proportion: that store holds thirteen
            // choices whose blast radius includes saved bytes, so it owes the
            // operator a distinct sentence. This holds four display
            // preferences, and the honest cost of an unreadable file is that
            // the page is drawn at the shipped sharpness.
            return (Self::default(), Vec::new());
        };
        Self::parse(&text)
    }

    /// Parse, with per-key recovery.
    ///
    /// # The `match` is the file format
    ///
    /// There is no key table, no `HashMap` and no derive: every key this build
    /// understands is an arm below, and the `_` arm reports everything else as
    /// [`PrefNote::UnknownKey`] and **keeps it in the file**. That last part is
    /// what makes it safe for an operator to run two versions of pdfce out of
    /// one `userdata` folder — the older one does not delete the newer one's
    /// settings on its next Save, because [`Self::write_to_string`] writes what
    /// this build knows and the loader never rewrites on load.
    ///
    /// The honest limit of that: an unknown key survives until the operator
    /// presses Save in the older build, which writes a fresh file from the
    /// fields it has. Preserving unknown lines across a *write* would mean
    /// carrying them on `Prefs`, and a struct holding values it cannot use is
    /// worse than the narrow case it protects.
    #[must_use]
    pub fn parse(text: &str) -> (Self, Vec<PrefNote>) {
        let mut prefs = Self::default();
        let mut notes = Vec::new();
        for (index, raw) in text.lines().enumerate() {
            let line = index + 1;
            let trimmed = raw.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let Some((key, value)) = trimmed.split_once('=') else {
                notes.push(PrefNote::Malformed { line });
                continue;
            };
            let (key, value) = (key.trim(), value.trim());
            if key.is_empty() {
                notes.push(PrefNote::Malformed { line });
                continue;
            }
            match key {
                // ui-text-exempt: a file KEY, parsed out of preferences.txt.
                "page_cache" => match PageCache::from_key(value) {
                    Some(c) => prefs.page_cache = c,
                    None => notes.push(PrefNote::BadValue {
                        key: key.to_owned(),
                        value: value.to_owned(),
                        line,
                    }),
                },
                "render_quality" => match RenderQuality::from_key(value) {
                    Some(q) => prefs.render_quality = q,
                    None => notes.push(PrefNote::BadValue {
                        key: key.to_owned(),
                        value: value.to_owned(),
                        line,
                    }),
                },
                // ui-text-exempt: a file KEY, matched literally.
                "max_zoom_percent" => match value.parse::<f32>() {
                    Ok(pct) if pct.is_finite() => {
                        let clamped = pct.clamp(MIN_MAX_ZOOM_PERCENT, MAX_MAX_ZOOM_PERCENT);
                        if (clamped - pct).abs() > f32::EPSILON {
                            notes.push(PrefNote::Clamped {
                                key: key.to_owned(),
                                value: value.to_owned(),
                                line,
                            });
                        }
                        prefs.max_zoom_percent = clamped;
                    }
                    // ★ A non-finite value is a BadValue rather than a clamp.
                    // `inf` would propagate into a scroll extent and blank the
                    // canvas, and reporting it as "clamped" would imply the
                    // operator wrote something reasonable.
                    _ => notes.push(PrefNote::BadValue {
                        key: key.to_owned(),
                        value: value.to_owned(),
                        line,
                    }),
                },
                "zoom_settle_ms" => match value.parse::<u64>() {
                    Ok(ms) => {
                        let clamped = ms.clamp(MIN_SETTLE_MS, MAX_SETTLE_MS);
                        if clamped != ms {
                            notes.push(PrefNote::Clamped {
                                key: key.to_owned(),
                                value: value.to_owned(),
                                line,
                            });
                        }
                        prefs.zoom_settle_ms = clamped;
                    }
                    Err(_) => notes.push(PrefNote::BadValue {
                        key: key.to_owned(),
                        value: value.to_owned(),
                        line,
                    }),
                },
                "ui_scale" => match value.parse::<f32>() {
                    // ★ `is_finite` first, and it is not defensive padding.
                    // `"nan"` and `"inf"` both parse successfully as `f32`, so
                    // without this a hand-edited `ui_scale = nan` would reach
                    // `normalise_ui_scale`, where `clamp` propagates NaN rather
                    // than rejecting it — and a NaN zoom factor is a window
                    // that draws nothing. It is reported as a bad value, which
                    // is what it is, rather than clamped to an end the operator
                    // did not name.
                    Ok(raw) if raw.is_finite() => {
                        let scale = chrome::normalise_ui_scale(raw);
                        // Reported when the file's value is not one the control
                        // can produce — see `normalise_ui_scale` on why the
                        // rounding happens at all. The epsilon is a tenth of a
                        // step, comfortably finer than any difference that
                        // matters and coarse enough that float noise from the
                        // round trip does not raise a note on a clean file.
                        if (scale - raw).abs() > UI_SCALE_STEP / 10.0 {
                            notes.push(PrefNote::Clamped {
                                key: key.to_owned(),
                                value: value.to_owned(),
                                line,
                            });
                        }
                        prefs.ui_scale = scale;
                    }
                    _ => notes.push(PrefNote::BadValue {
                        key: key.to_owned(),
                        value: value.to_owned(),
                        line,
                    }),
                },
                "wheel_paging" => match WheelPaging::from_key(value) {
                    Some(w) => prefs.wheel_paging = w,
                    None => notes.push(PrefNote::BadValue {
                        key: key.to_owned(),
                        value: value.to_owned(),
                        line,
                    }),
                },
                "opening_fit" => match OpeningFit::from_key(value) {
                    Some(f) => prefs.opening_fit = f,
                    None => notes.push(PrefNote::BadValue {
                        key: key.to_owned(),
                        value: value.to_owned(),
                        line,
                    }),
                },
                // The three overlays share one parse shape and differ only in
                // which field they land in, so the destination is picked first
                // and the reading is written once. Three near-identical arms is
                // how the fourth overlay gets a subtly different parser.
                k if k == smoothing::KEY => match opening::bool_from_key(value) {
                    Some(on) => prefs.image_smoothing_default_applied = on,
                    None => notes.push(PrefNote::BadValue {
                        key: key.to_owned(),
                        value: value.to_owned(),
                        line,
                    }),
                },
                "show_rulers" | "show_grid" | "show_guides" => {
                    let target = match key {
                        "show_rulers" => &mut prefs.chrome.rulers,
                        "show_grid" => &mut prefs.chrome.grid,
                        // Exhaustive by the arm's own pattern; the compiler
                        // cannot see that, and a `_` here would silently absorb
                        // a fourth overlay added to the pattern above and never
                        // given a field.
                        _ => &mut prefs.chrome.guides,
                    };
                    match opening::bool_from_key(value) {
                        Some(on) => *target = on,
                        None => notes.push(PrefNote::BadValue {
                            key: key.to_owned(),
                            value: value.to_owned(),
                            line,
                        }),
                    }
                }
                _ => notes.push(PrefNote::UnknownKey {
                    key: key.to_owned(),
                    line,
                }),
            }
        }
        (prefs, notes)
    }

    /// The file's whole text.
    ///
    /// Commented, because the file is meant to be opened in a text editor and
    /// a bare `render_quality = faster` tells an operator nothing about what
    /// else they could write. Same posture as the engine's store, which spends
    /// a comment block per key for exactly this reason.
    #[must_use]
    pub fn write_to_string(&self) -> String {
        let mut out = String::new();
        out.push_str(
            "# pdfce display preferences\n\
             #\n\
             # How pdfce draws, as distinct from how it reads and writes PDFs —\n\
             # those live in settings.txt beside this file. Plain text, one\n\
             # `key = value` per line, # for comments. An unknown key is reported\n\
             # and kept, not deleted, and a value pdfce cannot read falls back for\n\
             # that key alone.\n\
             #\n\
             # KEEP THIS FOLDER when you update pdfce.\n\
             \n\
             # How sharply a page is drawn: faster | normal | sharper\n\
             # faster  = three quarter scale. Softer lines, quicker on a big sheet.\n\
             # normal  = one pixel per screen pixel. The shipped answer.\n\
             # sharper = one and a half times. For small text over fine linework.\n",
        );
        // ui-text-exempt: a file KEY, written into preferences.txt and parsed
        // back out of it. Never displayed — the operator meets this setting as
        // "How sharply pages are drawn" in the Settings window.
        out.push_str("render_quality = ");
        out.push_str(self.render_quality.key());
        out.push('\n');
        out.push_str(
            "\n\
             # How much memory pdfce may use to remember pages it has already\n\
             # drawn, so that scrolling back to one does not draw it again:\n\
             #   small   = about 190 MB. What pdfce used before 2026-08-19.\n\
             #   medium  = about 490 MB.\n\
             #   large   = about 980 MB. The shipped answer.\n\
             #   maximum = about 1950 MB. A whole drawing set kept resident.\n\
             # Pages furthest from the one you are looking at are dropped first.\n",
        );
        // ui-text-exempt: a file KEY, as above.
        out.push_str("page_cache = ");
        out.push_str(self.page_cache.key());
        out.push('\n');
        out.push_str(
            "\n\
             # How long a zoom must stop changing before the page is redrawn\n\
             # sharply, in milliseconds. 20 to 1000. Lower feels more immediate\n\
             # and redraws more; higher swallows a whole wheel gesture.\n",
        );
        out.push_str(
            "\n\
             # The highest zoom you can reach, as a percentage. 800 is the\n\
             # shipped default and is what earlier versions allowed.\n\
             #\n\
             # Above roughly 1000% pdfce stops drawing the whole page and draws\n\
             # only what is on screen, because a whole-page image would exceed\n\
             # what can be rasterized. Panning is free below that point and\n\
             # costs a redraw above it -- so this is also the dial for trying\n\
             # the two out against each other.\n\
             # 10 to 1000000000000.\n",
        );
        // ui-text-exempt: a file KEY, as above.
        out.push_str("max_zoom_percent = ");
        out.push_str(&format_percent(self.max_zoom_percent));
        out.push('\n');
        out.push_str(
            "\n\
             # How long a zoom must stop changing before the page is redrawn\n\
             # sharply, in milliseconds. 20 to 1000. Lower feels more immediate\n\
             # and redraws more; higher swallows a whole wheel gesture.\n",
        );
        // ui-text-exempt: a file KEY, as above.
        out.push_str("zoom_settle_ms = ");
        out.push_str(&self.zoom_settle_ms.to_string());
        out.push('\n');
        out.push_str(
            "\n\
             # How big pdfce's own menus, buttons and labels are drawn, as a\n\
             # MULTIPLIER on your Windows display setting -- not a replacement\n\
             # for it. 0.8 to 2.0, in steps of 0.05. A value of 1 means exactly\n\
             # what Windows asked for. Changes the program, never the page.\n",
        );
        // ui-text-exempt: a file KEY, as above.
        out.push_str("ui_scale = ");
        // Two decimals: the step is 0.05, so two places represent every value
        // the control can produce exactly and none that it cannot. The default
        // `f32` formatting would write `1` for 1.0 and `1.1500001` for a value
        // that arrived through a slider, and the second of those is a number no
        // operator should have to read in a file they are invited to edit.
        out.push_str(&format!("{:.2}", self.ui_scale));
        out.push('\n');
        out.push_str(
            "\n\
             # ---------------------------------------------------------------\n\
             # What you see when a document first opens. Both of these apply to\n\
             # the NEXT document opened, not to the one already on screen.\n\
             # ---------------------------------------------------------------\n\
             \n\
             # How the first page is sized: page | width | height | actual\n\
             # page   = the whole page fits the window. The shipped answer.\n\
             # width  = the full width fits; the bottom may run off screen.\n\
             # height = the full height fits; the side may run off screen.\n\
             # actual = one page point per screen point, whatever that shows.\n",
        );
        // ui-text-exempt: a file KEY, as above.
        out.push_str("opening_fit = ");
        out.push_str(self.opening_fit.key());
        out.push('\n');
        out.push_str(
            // ui-text-exempt: file comments, never displayed in the UI.
            "\n\
             # What the mouse wheel does on a single page: scroll | flip\n\
             # scroll = move within the sheet. The shipped answer.\n\
             # flip   = turn to the next or previous page.\n\
             # Ignored under a continuous display mode, where the wheel\n\
             # scrolls the whole document by definition.\n",
        );
        // ui-text-exempt: a file KEY, as above.
        out.push_str("wheel_paging = ");
        out.push_str(self.wheel_paging.key());
        out.push('\n');
        out.push_str(
            "\n\
             # Which overlays are already switched on: true | false.\n\
             # Rulers take a strip off the top and left of the drawing area.\n\
             # Guides are dragged OUT OF a ruler, so placing one needs both\n\
             # show_guides and show_rulers on.\n",
        );
        // ui-text-exempt: a file KEY, as above. Three keys, written together
        // under one comment block because they are one setting in the window
        // and a reader meeting them apart would not know they interlock.
        for (key, value) in [
            ("show_rulers", self.chrome.rulers),
            ("show_grid", self.chrome.grid),
            ("show_guides", self.chrome.guides),
        ] {
            out.push_str(key);
            // ui-text-exempt: the file format's own `key = value` separator,
            // never displayed. The three single-key writes above spell it into
            // their key literal; a loop cannot, so it is its own push.
            out.push_str(" = ");
            out.push_str(opening::bool_key(value));
            out.push('\n');
        }

        // ★ Bookkeeping rather than a preference, and written last so it reads
        // as the footnote it is. Without this line the migration in
        // `smoothing` has no memory and runs on EVERY launch — which is not a
        // migration but an override, and would silently undo an operator who
        // went back to point sampling on purpose. `smoothing`'s own
        // `a_marked_installation_is_never_touched_again` is the assertion; this
        // is what makes it reachable.
        out.push_str(
            "\n\
             # Bookkeeping, not a setting you need to change.\n\
             # pdfce-gui smooths images drawn smaller than their own pixel grid,\n\
             # which the PDF standard does not legislate either way. This records\n\
             # that the choice has been applied once, so if you change it back in\n\
             # Settings it stays changed.\n",
        );
        // ui-text-exempt: a file KEY, written into preferences.txt and parsed
        // back out of it. Never displayed.
        out.push_str(smoothing::KEY);
        // ui-text-exempt: the file format's own `key = value` separator, never displayed.
        out.push_str(" = ");
        out.push_str(opening::bool_key(self.image_smoothing_default_applied));
        out.push('\n');
        out
    }

    /// Write, reporting failure.
    ///
    /// Unlike loading, saving fails **loudly**: the operator asked for
    /// something to be remembered and is owed the truth if it was not. Same
    /// asymmetry the engine's store holds itself to.
    ///
    /// # Errors
    ///
    /// The path could not be resolved, its directory could not be created, or
    /// the write was refused. Carried as a `String` because the caller's only
    /// use for it is a trace line — the operator-facing half is a fixed
    /// sentence, for the reason `text::status::settings_not_saved` documents.
    pub fn save(&self) -> Result<(), String> {
        let path = Self::path().ok_or_else(|| {
            // ui-text-exempt: a trace/diagnostic string, never displayed.
            "no writable location for preferences".to_owned()
        })?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(&path, self.write_to_string()).map_err(|e| e.to_string())
    }

    /// **Apply the opening preferences to a freshly assembled view.**
    ///
    /// Called once per document, from `PdfceApp::adopt`, and from nowhere else.
    ///
    /// # ★ Why this is a method here rather than a field read in `ViewState::default`
    ///
    /// Because `ViewState::default()` cannot see the application. `OpenDoc::assemble`
    /// builds a document without a `PdfceApp` in reach — its own comment says
    /// so — which is the same constraint that put `adopt_settings` in the open
    /// path rather than in the constructor. Seeding here keeps `ViewState`'s
    /// `Default` the **conservative** answer, which is what every test that
    /// builds one without a configuration relies on.
    ///
    /// # ★ The remembered-guides override still wins, and that is not a
    /// coincidence of ordering
    ///
    /// `OpenDoc::assemble` may already have set `view.guides = true`, because
    /// `canvas::guides::opening` turns the layer on for a document that has
    /// guides saved against it — *"the presence of the work is the
    /// preference"*. This function therefore **ORs** rather than assigns for
    /// that one field:
    ///
    /// | remembered guides | preference | result |
    /// |---|---|---|
    /// | yes | on | shown |
    /// | yes | off | **shown** — the work outranks the default |
    /// | no | on | shown, and empty until the first is placed |
    /// | no | off | hidden |
    ///
    /// Row two is the one that matters and it is the reason this is not three
    /// plain assignments. A preference is a statement about documents in
    /// general; a document that carries guides is a statement about *that*
    /// document, and the specific beats the general. Assigning would hide work
    /// the operator did, on the document they did it on, because of a switch
    /// they set weeks earlier about something else.
    ///
    /// Rulers and grid have no per-document memory at all, so they assign.
    ///
    /// # What it deliberately does not touch
    ///
    /// [`crate::viewer::ViewState::display`] — the single/continuous/facing
    /// arrangement. That has its own per-document store and its own operator
    /// requirement; see [`opening`]'s header for why a global default for it
    /// would be a second axis colliding with the one that was asked for.
    pub fn seed_view(&self, view: &mut crate::viewer::ViewState) {
        let (fit, zoom) = self.opening_fit.to_view();
        view.fit = fit;
        view.zoom = zoom;
        view.rulers = self.chrome.rulers;
        view.grid = self.chrome.grid;
        // OR, not assign — see the table in this function's docs.
        view.guides = view.guides || self.chrome.guides;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::viewer::{FitMode, ViewState};

    /// ★ Every value round-trips through the file.
    ///
    /// The property a preferences store exists for, and the one a hand-written
    /// writer and a hand-written parser get wrong first: they are two spellings
    /// of the same vocabulary, and this is what stops them drifting.
    ///
    /// Every field is varied, and the two enums are varied over **all** their
    /// values rather than one apiece — a writer that emitted a constant token
    /// would pass a single-value check.
    #[test]
    fn every_preference_round_trips_through_the_file() {
        for quality in RenderQuality::ALL {
            for fit in OpeningFit::ALL {
                let original = Prefs {
                    // The migration marker round-trips like every other key.
                    // `true` rather than the default `false`, per this test's
                    // own rule: a non-default in every field, so no emitted
                    // value can coincide with what a failed parse left behind.
                    image_smoothing_default_applied: true,
                    render_quality: *quality,
                    page_cache: PageCache::default(),
                    zoom_settle_ms: 275,
                    // A non-default well past the shipped ceiling, so the
                    // round trip is proved on a value that MATTERS rather
                    // than on 800.
                    max_zoom_percent: 1_000_000.0,
                    opening_fit: *fit,
                    // ★ The non-default, so a writer that emitted no
                    // `wheel_paging` key at all would fail here rather than
                    // pass by landing back on `Scroll`.
                    wheel_paging: WheelPaging::FlipPages,
                    // Deliberately not all-true and not all-false: an assignment
                    // that crossed two of the three fields would survive either.
                    chrome: PageChrome {
                        rulers: true,
                        grid: false,
                        guides: true,
                    },
                    // A non-default that is ON the control's step, so the round
                    // trip tests the writer's formatting rather than the
                    // loader's rounding — that is `an_off_step_ui_scale_is_rounded_and_reported`'s job.
                    ui_scale: 1.25,
                };
                let (read_back, notes) = Prefs::parse(&original.write_to_string());
                assert!(
                    notes.is_empty(),
                    "a written file did not read cleanly: {notes:?}"
                );
                assert_eq!(
                    read_back, original,
                    "{quality:?}/{fit:?} did not survive the trip"
                );
            }
        }
    }

    /// ★ The three overlays are three independent keys.
    ///
    /// The failure this catches is a copy-paste in either the writer or the
    /// parser sending two overlays to one field — which the round-trip above
    /// would only catch for the specific combination it happens to use. Here
    /// each is set alone and the other two are asserted to have stayed off.
    #[test]
    fn each_overlay_is_written_and_read_on_its_own_key() {
        for (name, build) in [
            (
                "rulers",
                PageChrome {
                    rulers: true,
                    ..PageChrome::default()
                },
            ),
            (
                "grid",
                PageChrome {
                    grid: true,
                    ..PageChrome::default()
                },
            ),
            (
                "guides",
                PageChrome {
                    guides: true,
                    ..PageChrome::default()
                },
            ),
        ] {
            let original = Prefs {
                chrome: build,
                ..Prefs::default()
            };
            let (read_back, notes) = Prefs::parse(&original.write_to_string());
            assert!(notes.is_empty(), "{name}: {notes:?}");
            assert_eq!(read_back.chrome, build, "{name} landed in the wrong field");
        }
    }

    /// The shipped defaults are what the constants they replaced held.
    ///
    /// `ZOOM_SETTLE` was a compiled-in 150 ms, `raster_scale` had no
    /// multiplier at all, and `ViewState::default` was fit-page with all three
    /// overlays off. A build that never opens the Settings window has to
    /// behave exactly as the build before this module did — the standing rule
    /// for a capability becoming choosable.
    #[test]
    fn the_defaults_are_the_constants_they_replaced() {
        let prefs = Prefs::default();
        assert_eq!(prefs.zoom_settle_ms, 150);
        assert!((prefs.render_quality.multiplier() - 1.0).abs() < f32::EPSILON);
        assert_eq!(prefs.opening_fit, OpeningFit::Page);
        assert!(prefs.chrome.all_hidden());
    }

    /// ★ **The shipped preferences change nothing about a freshly opened view.**
    ///
    /// The strongest form of the rule above, and the one a reordering or a
    /// typo in [`Prefs::seed_view`] would break: seeding a default `ViewState`
    /// from default preferences must leave it **byte-identical**. Asserting
    /// the fields one at a time would pass while a fourth field was silently
    /// clobbered; asserting the whole struct will not.
    #[test]
    fn seeding_from_the_shipped_preferences_changes_nothing() {
        let mut view = ViewState::default();
        Prefs::default().seed_view(&mut view);
        assert_eq!(
            view,
            ViewState::default(),
            "the shipped preferences moved a freshly opened view"
        );
    }

    /// Each opening fit reaches the view it names.
    #[test]
    fn the_opening_fit_reaches_the_view() {
        for (fit, expected) in [
            (OpeningFit::Page, FitMode::Page),
            (OpeningFit::Width, FitMode::Width),
            (OpeningFit::Height, FitMode::Height),
            (OpeningFit::ActualSize, FitMode::None),
        ] {
            let mut view = ViewState::default();
            Prefs {
                opening_fit: fit,
                ..Prefs::default()
            }
            .seed_view(&mut view);
            assert_eq!(view.fit, expected, "{fit:?}");
            assert!(view.zoom > 0.0, "{fit:?} seeded a zoom of {}", view.zoom);
        }
    }

    /// ★ **A document's remembered guides survive a preference that hides them.**
    ///
    /// Row two of [`Prefs::seed_view`]'s table, and the whole reason that one
    /// field ORs. `canvas::guides::opening` turns the layer on for a document
    /// that has guides saved against it, because *"the presence of the work is
    /// the preference"* — and an assignment here would hide work the operator
    /// did, on the document they did it on, because of a switch they set weeks
    /// earlier about documents in general.
    ///
    /// This is the failing direction: preference **off**, view already **on**.
    #[test]
    fn a_preference_that_hides_guides_does_not_hide_remembered_ones() {
        // What `OpenDoc::assemble` hands over for a document with saved guides.
        let mut view = ViewState {
            guides: true,
            ..ViewState::default()
        };
        Prefs {
            chrome: PageChrome {
                guides: false,
                ..PageChrome::default()
            },
            ..Prefs::default()
        }
        .seed_view(&mut view);
        assert!(
            view.guides,
            "a document's own remembered guides were hidden by a global default"
        );
    }

    /// …and rulers and grid do NOT get that treatment.
    ///
    /// The counterpart, and it is what stops the OR being copied to all three
    /// out of symmetry. Neither has any per-document memory, so a `true`
    /// arriving in the view is not evidence of anything the operator did — it
    /// would just be a stale value that the preference could then never turn
    /// off.
    #[test]
    fn rulers_and_grid_follow_the_preference_in_both_directions() {
        let mut view = ViewState {
            rulers: true,
            grid: true,
            ..ViewState::default()
        };
        Prefs::default().seed_view(&mut view);
        assert!(
            !view.rulers,
            "the rulers preference could not turn them off"
        );
        assert!(!view.grid, "the grid preference could not turn it off");
    }

    /// ★ One bad line never discards the rest of the file.
    ///
    /// The fail-soft contract, and the reason it matters here rather than being
    /// inherited politeness: this file is *meant* to be hand-edited, and a
    /// parser that failed a whole document over one typo would punish the
    /// operator for doing the thing the file invites.
    #[test]
    fn a_bad_line_costs_only_its_own_key() {
        let (prefs, notes) = Prefs::parse(
            "render_quality = sharper\n\
             this line is not a setting\n\
             zoom_settle_ms = purple\n\
             show_rulers = ture\n\
             opening_fit = width\n\
             unknown_key = 3\n",
        );
        assert_eq!(
            prefs.render_quality,
            RenderQuality::Sharper,
            "a good key was discarded because a later line was bad"
        );
        assert_eq!(
            prefs.zoom_settle_ms, DEFAULT_SETTLE_MS,
            "an unreadable value must fall back for its own key"
        );
        assert!(
            !prefs.chrome.rulers,
            "a misspelt bool must fall back, not be read as true"
        );
        assert_eq!(
            prefs.opening_fit,
            OpeningFit::Width,
            "a good key AFTER a bad one was discarded"
        );
        assert!(
            notes
                .iter()
                .any(|n| matches!(n, PrefNote::Malformed { .. }))
        );
        // Two bad values, not one: the settle and the misspelt bool.
        assert_eq!(
            notes
                .iter()
                .filter(|n| matches!(n, PrefNote::BadValue { .. }))
                .count(),
            2,
            "{notes:?}"
        );
        assert!(
            notes
                .iter()
                .any(|n| matches!(n, PrefNote::UnknownKey { .. }))
        );
    }

    /// ★★★ **A trillion percent is accepted**, which is the figure the
    /// operator named — `OPERATOR_REQUESTS.md` O24.
    ///
    /// The point of the setting is that the performance trade is his; a ceiling
    /// exists only because `f32` must stay finite — and since both precision
    /// ceilings were removed, the page actually draws there.
    #[test]
    fn a_trillion_percent_is_accepted_and_the_page_actually_draws_there() {
        let (prefs, notes) = Prefs::parse(
            "max_zoom_percent = 1000000000000
",
        );
        // ★ Accepted in full, and NOT clamped. It was clamped for part of
        // 2026-08-22, while a trillion percent rendered cleanly and showed a
        // blank page; removing the two precision ceilings made the figure he
        // named actually draw, so the clamp went with them.
        assert!((prefs.max_zoom_percent - 1e12).abs() / 1e12 < 1e-6);
        assert!(
            notes.is_empty(),
            "a stated maximum the shell can honour must not be second-guessed"
        );
    }

    /// ★★ **A non-finite value is refused, not clamped.**
    ///
    /// `inf` would propagate into a scroll extent and blank the canvas, which is
    /// the failure `canvas::geometry`'s guards exist for. Reporting it as
    /// *clamped* would also imply the operator wrote something reasonable.
    #[test]
    fn an_infinite_maximum_is_a_bad_value_rather_than_a_clamp() {
        for text in [
            "max_zoom_percent = inf
",
            "max_zoom_percent = NaN
",
        ] {
            let (prefs, notes) = Prefs::parse(text);
            assert_eq!(
                prefs.max_zoom_percent, DEFAULT_MAX_ZOOM_PERCENT,
                "{text:?} must leave the default in place"
            );
            assert!(
                notes.iter().any(|n| matches!(n, PrefNote::BadValue { .. })),
                "{text:?} should be reported as a bad value"
            );
        }
    }

    /// The default is the MAXIMUM, on the operator's instruction of the
    /// shell behaved before this setting existed.
    #[test]
    fn the_default_maximum_is_the_highest_available() {
        let (prefs, _) = Prefs::parse("");
        assert!(
            (prefs.max_zoom_percent - MAX_MAX_ZOOM_PERCENT).abs() < f32::EPSILON,
            "the operator asked for the default to reach the maximum"
        );
    }

    /// ★ **The file says a whole number, not `1e12`.**
    ///
    /// The preferences file is the operator's to read and edit; a machine-shaped
    /// number there means he cannot tell at a glance what he set.
    ///
    /// ★★ And it records something the operator will otherwise discover by
    /// reading his own file: **`f32` cannot hold a trillion exactly.** It
    /// stores `999,999,995,904` — a rounding of four thousand parts in a
    /// trillion, four ten-millionths of one percent. At a zoom where one screen
    /// pixel is a millionth of a point, that difference is unobservable; but a
    /// value written back as a number he did not type is worth knowing about
    /// rather than being mistaken for a bug.
    #[test]
    fn the_file_writes_a_readable_number_rather_than_an_exponent() {
        let prefs = Prefs {
            max_zoom_percent: 1e12,
            ..Prefs::default()
        };
        let text = prefs.write_to_string();
        assert!(
            text.contains("max_zoom_percent = 999999995904"),
            "the file should spell the number out rather than using an exponent: {text}"
        );
        assert!(
            !text.contains("e12"),
            "no exponent should reach the file: {text}"
        );
    }

    /// An out-of-range settle is clamped and the clamp is reported.
    ///
    /// Reported, not silent: the operator wrote a number and is getting a
    /// different one, which is exactly the kind of quiet substitution the
    /// engine's store spends a note variant on.
    #[test]
    fn an_out_of_range_settle_clamps_and_says_so() {
        let (prefs, notes) = Prefs::parse("zoom_settle_ms = 99999\n");
        assert_eq!(prefs.zoom_settle_ms, MAX_SETTLE_MS);
        assert!(notes.iter().any(|n| matches!(n, PrefNote::Clamped { .. })));

        let (prefs, notes) = Prefs::parse("zoom_settle_ms = 0\n");
        assert_eq!(prefs.zoom_settle_ms, MIN_SETTLE_MS);
        assert!(notes.iter().any(|n| matches!(n, PrefNote::Clamped { .. })));
    }

    /// An off-step UI scale is rounded to one the control can produce, and the
    /// substitution is reported.
    ///
    /// Rounding rather than accepting, because the file is hand-editable and
    /// the slider is not: a value of `1.234` would sit in the control until the
    /// operator touched it, at which point it would jump — a change they did
    /// not make, to a setting they did. Reported for the same reason the settle
    /// clamp is: the operator wrote a number and is getting a different one.
    #[test]
    fn an_off_step_ui_scale_is_rounded_and_reported() {
        let (prefs, notes) = Prefs::parse("ui_scale = 1.234\n");
        assert!(
            (prefs.ui_scale - 1.25).abs() < 1e-5,
            "1.234 became {}",
            prefs.ui_scale
        );
        assert!(notes.iter().any(|n| matches!(n, PrefNote::Clamped { .. })));

        // …and a value already on the step is NOT reported. The other half:
        // a note on every clean file would train the operator to ignore notes.
        let (prefs, notes) = Prefs::parse("ui_scale = 1.25\n");
        assert!((prefs.ui_scale - 1.25).abs() < 1e-5);
        assert!(notes.is_empty(), "a clean value was reported: {notes:?}");
    }

    /// ★ **A UI scale of `nan` or `inf` is refused, not clamped.**
    ///
    /// The one parse arm in this file that needs a guard beyond `parse()`
    /// succeeding. `"nan"` and `"inf"` are both valid `f32` literals, and
    /// `f32::clamp` **propagates** NaN rather than rejecting it — so without
    /// the `is_finite` check a hand-edited `ui_scale = nan` would flow through
    /// `normalise_ui_scale` untouched and reach `Context::set_zoom_factor`,
    /// which is a window that draws nothing.
    ///
    /// Reported as a bad value rather than clamped to an end, because the
    /// operator did not name an end. `inf` is included for the same reason
    /// even though clamping would in fact handle it: two spellings of "this is
    /// not a size" should not get two different treatments.
    #[test]
    fn a_non_finite_ui_scale_is_refused_rather_than_clamped() {
        for spelling in ["nan", "NaN", "inf", "-inf", "infinity"] {
            let (prefs, notes) = Prefs::parse(&format!("ui_scale = {spelling}\n"));
            assert!(
                (prefs.ui_scale - DEFAULT_UI_SCALE).abs() < 1e-6,
                "{spelling:?} produced a scale of {}",
                prefs.ui_scale
            );
            assert!(
                prefs.ui_scale.is_finite(),
                "{spelling:?} reached the zoom factor"
            );
            assert!(
                notes.iter().any(|n| matches!(n, PrefNote::BadValue { .. })),
                "{spelling:?} was substituted silently: {notes:?}"
            );
        }
    }

    /// A missing file is silent.
    ///
    /// A first run is the expected state, not a fault. Reporting it would train
    /// the operator to ignore the channel that carries the real problems — the
    /// engine's store makes the same distinction and states it in a table.
    #[test]
    fn an_empty_file_produces_defaults_and_no_notes() {
        let (prefs, notes) = Prefs::parse("");
        assert_eq!(prefs, Prefs::default());
        assert!(notes.is_empty());
    }

    /// ★ Every key the writer emits is a key the parser knows.
    ///
    /// The drift this catches is the one that would be silent in both
    /// directions: a key added to [`Prefs::write_to_string`] and not to
    /// [`Prefs::parse`] makes pdfce report its **own** file as containing an
    /// unknown key, on every start, forever — and the operator would have no
    /// way to tell that the file they never edited was written by the program
    /// complaining about it.
    ///
    /// The round-trip test above cannot see this: it compares the parsed struct
    /// and would pass on a key that was written, unread and defaulted back to
    /// the same value.
    #[test]
    fn the_writer_emits_no_key_the_parser_rejects() {
        // A non-default in every field, so no emitted value can coincide with
        // what a failed parse would have left behind.
        let prefs = Prefs {
            // Non-default, for the reason stated below about every other field.
            image_smoothing_default_applied: true,
            render_quality: RenderQuality::Sharper,
            // ★ Not the default, deliberately, and this test's own comment says
            // why: "a non-default in every field, so no emitted value can
            // coincide with what a failed parse would have left behind". A
            // `PageCache::Large` here would pass on a build whose writer emitted
            // no `page_cache` key at all.
            page_cache: PageCache::Maximum,
            zoom_settle_ms: 400,
            max_zoom_percent: 25_000.0,
            opening_fit: OpeningFit::ActualSize,
            wheel_paging: WheelPaging::FlipPages,
            chrome: PageChrome {
                rulers: true,
                grid: true,
                guides: true,
            },
            ui_scale: 1.65,
        };
        let (_, notes) = Prefs::parse(&prefs.write_to_string());
        assert!(
            notes.is_empty(),
            "pdfce's own preferences file does not read cleanly: {notes:?}"
        );
    }

    /// The preferences file sits beside the settings file.
    ///
    /// Asserted against `pdfce-core`'s own answer rather than by re-deriving a
    /// path, so the two cannot drift — which is the failure this project
    /// already found once, when two callers in one process disagreed about
    /// which home was live.
    #[test]
    fn the_preferences_file_lives_beside_the_settings_file() {
        let store = pdfce_core::settings::resolve_store();
        let (Some(settings), Some(prefs)) = (store.path.as_deref(), Prefs::path()) else {
            // No writable location on this machine — the session still runs,
            // and there is nothing to compare. Not a failure.
            return;
        };
        assert_eq!(settings.parent(), prefs.parent());
    }
}
