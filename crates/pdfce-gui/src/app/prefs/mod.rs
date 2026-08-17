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
//! ## ★ Why this is TWO settings and not seven
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
//! | **Zoom settle delay** | ✅ [`Prefs::zoom_settle`] — `render::settle::ZOOM_SETTLE` was a compiled-in 150 ms |
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

use std::path::PathBuf;

/// How sharply a page is rasterised, as a multiplier on the natural scale.
///
/// # What "natural" is, and why this multiplies rather than replaces
///
/// `viewer::raster_scale` is `zoom × pixels_per_point`: one raster pixel per
/// *device* pixel, which is the scale at which a page is exactly as sharp as
/// the display can show and no sharper. That is the right default and it is
/// what [`RenderQuality::Normal`] means.
///
/// The two other values trade against it in opposite directions, and both are
/// real needs on the drawings this shell is for:
///
/// - **Faster** renders at 0.75× and lets the GPU upscale. On the benchmark
///   CAD sheet — 5.6 MB of dense vector site plan — that is roughly half the
///   pixels and therefore roughly half the rasterisation time, at the cost of
///   softness that is most visible on the thin linework such a drawing is
///   made of. An operator panning around a big sheet looking for something may
///   well want it; an operator checking a dimension will not.
/// - **Sharper** renders at 1.5×. Pointless on most content and genuinely
///   better on small text over a hairline grid, where a device pixel straddles
///   two strokes and neither survives.
///
/// # Why three values and not a slider
///
/// Because the useful range is narrow and the middle of it is almost always
/// right. A slider invites an operator to spend attention tuning a number that
/// will not repay it, and — more practically — every intermediate value costs a
/// full re-raster of every visible page to evaluate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RenderQuality {
    /// 0.75× — fewer pixels, softer lines, quicker.
    Faster,
    /// 1× — one raster pixel per device pixel. The shipped answer.
    #[default]
    Normal,
    /// 1.5× — more pixels than the display can show, for small text.
    Sharper,
}

impl RenderQuality {
    /// Every value, in the order the settings window lists them.
    ///
    /// Worst-to-best rather than best-to-worst, so the control reads left to
    /// right as *less … more* — which is the direction a reader expects of a
    /// quality scale and the opposite of the order the enum's own reasoning
    /// arrived in.
    pub const ALL: &'static [Self] = &[Self::Faster, Self::Normal, Self::Sharper];

    /// The multiplier applied to the natural raster scale.
    #[must_use]
    pub const fn multiplier(self) -> f32 {
        match self {
            Self::Faster => 0.75,
            Self::Normal => 1.0,
            Self::Sharper => 1.5,
        }
    }

    /// The token written to the preferences file.
    ///
    /// Stable across releases and deliberately not the display name: a display
    /// name is operator copy and may be reworded or translated, and a file
    /// whose keys moved when the wording did would silently reset everybody's
    /// preference. Same rule `egui_shell::theme::Preset::key` follows.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            // ui-text-exempt: a file token, never displayed.
            Self::Faster => "faster",
            // ui-text-exempt: a file token, never displayed.
            Self::Normal => "normal",
            // ui-text-exempt: a file token, never displayed.
            Self::Sharper => "sharper",
        }
    }

    /// Read a token back, or `None` if it names nothing.
    ///
    /// `None` rather than a default, so the loader can *report* an unreadable
    /// value rather than silently substituting one — the per-key recovery
    /// contract in the module header.
    #[must_use]
    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|q| q.key() == key)
    }
}

/// The shortest zoom-settle delay offered, in milliseconds.
///
/// Zero is excluded and that is a decision. A settle of zero means *rasterise
/// every intermediate value of a wheel gesture*, which on a dense CAD sheet is
/// dozens of full-page renders producing images nobody sees — the exact cost
/// the debounce exists to avoid. 20 ms is short enough to feel immediate and
/// long enough to swallow the burst of events one wheel notch produces.
pub const MIN_SETTLE_MS: u64 = 20;

/// The longest offered.
///
/// Beyond about a second the interim scaled texture stops reading as "still
/// settling" and starts reading as "stuck", which is a worse impression than
/// the CPU cost it saves.
pub const MAX_SETTLE_MS: u64 = 1000;

/// The shipped settle, in milliseconds.
///
/// 150 ms is the value the old shell settled on against real CAD sheets, and it
/// was `render::settle::ZOOM_SETTLE`'s compiled-in constant before this module
/// existed. It stays the default for the standing reason: a build that omits
/// nothing must behave as it did before the choice existed.
pub const DEFAULT_SETTLE_MS: u64 = 150;

/// The file this store is written to, beside `settings.txt`.
// ui-text-exempt: a file name, never displayed.
pub const PREFS_FILE: &str = "preferences.txt";

/// The shell's own preferences.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prefs {
    /// How sharply a page is rasterised.
    pub render_quality: RenderQuality,
    /// How long a zoom must stop changing before it is committed to a real
    /// rasterisation, in milliseconds.
    ///
    /// Stored as a number rather than as a `Duration` because that is what the
    /// file holds and what the control edits; `render::settle` converts once,
    /// at the one place it is read.
    pub zoom_settle_ms: u64,
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            render_quality: RenderQuality::default(),
            zoom_settle_ms: DEFAULT_SETTLE_MS,
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
            // operator a distinct sentence. This holds two display preferences,
            // and the honest cost of an unreadable file is that the page is
            // rendered at the shipped sharpness.
            return (Self::default(), Vec::new());
        };
        Self::parse(&text)
    }

    /// Parse, with per-key recovery.
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
                "render_quality" => match RenderQuality::from_key(value) {
                    Some(q) => prefs.render_quality = q,
                    None => notes.push(PrefNote::BadValue {
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
             # How long a zoom must stop changing before the page is redrawn\n\
             # sharply, in milliseconds. 20 to 1000. Lower feels more immediate\n\
             # and redraws more; higher swallows a whole wheel gesture.\n",
        );
        // ui-text-exempt: a file KEY, as above.
        out.push_str("zoom_settle_ms = ");
        out.push_str(&self.zoom_settle_ms.to_string());
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
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★ Every value round-trips through the file.
    ///
    /// The property a preferences store exists for, and the one a hand-written
    /// writer and a hand-written parser get wrong first: they are two spellings
    /// of the same vocabulary, and this is what stops them drifting.
    #[test]
    fn every_preference_round_trips_through_the_file() {
        for quality in RenderQuality::ALL {
            let original = Prefs {
                render_quality: *quality,
                zoom_settle_ms: 275,
            };
            let (read_back, notes) = Prefs::parse(&original.write_to_string());
            assert!(
                notes.is_empty(),
                "a written file did not read cleanly: {notes:?}"
            );
            assert_eq!(read_back, original, "{quality:?} did not survive the trip");
        }
    }

    /// The shipped defaults are what the constants they replaced held.
    ///
    /// `ZOOM_SETTLE` was a compiled-in 150 ms and `raster_scale` had no
    /// multiplier at all. A build that never opens the Settings window has to
    /// behave exactly as the build before this module did — the standing rule
    /// for a capability becoming choosable.
    #[test]
    fn the_defaults_are_the_constants_they_replaced() {
        let prefs = Prefs::default();
        assert_eq!(prefs.zoom_settle_ms, 150);
        assert!((prefs.render_quality.multiplier() - 1.0).abs() < f32::EPSILON);
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
            notes
                .iter()
                .any(|n| matches!(n, PrefNote::Malformed { .. }))
        );
        assert!(notes.iter().any(|n| matches!(n, PrefNote::BadValue { .. })));
        assert!(
            notes
                .iter()
                .any(|n| matches!(n, PrefNote::UnknownKey { .. }))
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

    /// The tokens are stable and distinct.
    ///
    /// They are what the file holds, so two quality values sharing a token
    /// would make one of them unreachable from a hand-edited file, and a token
    /// that changed with a display name would reset everybody's preference on
    /// upgrade.
    #[test]
    fn every_quality_has_a_distinct_stable_token() {
        for q in RenderQuality::ALL {
            assert_eq!(RenderQuality::from_key(q.key()), Some(*q));
        }
        let keys: Vec<&str> = RenderQuality::ALL.iter().map(|q| q.key()).collect();
        for i in 0..keys.len() {
            for j in (i + 1)..keys.len() {
                assert_ne!(keys[i], keys[j]);
            }
        }
        assert!(RenderQuality::from_key("nonesuch").is_none());
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
