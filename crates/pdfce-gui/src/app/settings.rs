//! # `app::settings` — the live configuration, and the funnel that makes it real
//!
//! ## ★ Why this module exists, and it is not "to hold a struct"
//!
//! `pdfce_core::settings::Settings` is thirteen operator choices about
//! questions the PDF standard declines to answer. Loading them is easy;
//! **honouring** them is where the old shell failed, and it failed silently.
//!
//! Measured against `D:\Dev\pdfce\crates\pdfce-gui` on 2026-08-17: of the
//! thirteen settings that window persists, **four are read anywhere in the
//! application and nine are not.** `separations`, `cmyk_intent`,
//! `parallel_epsilon_degrees` and `theme` reach the code that would act on
//! them. `word_gap_ratio`, `mask_resample`, `image_minify`,
//! `cmyk_jpeg_polarity`, `unmappable_code`, `actual_text`, `missing_as`,
//! `xref_entry_eol` and `trailing_eol` are written to disk, read back from
//! disk, shown in a window, edited by the operator — and then never consulted.
//!
//! The mechanism is not a bug anyone wrote. It is what happens when option
//! structs are built at the call site:
//!
//! ```text
//! ExtractOptions::default()      →  word_gap_ratio, unmappable_code, actual_text
//! RenderOptions::default()       →  mask_resample, image_minify,
//!                                   cmyk_jpeg_polarity, missing_as
//! SaveOptions::identity()        →  xref_entry_eol, trailing_eol
//! ```
//!
//! Every one of those constructors is correct in isolation and every one of
//! them silently discards the operator's configuration. There were twelve such
//! call sites in the old crate and there are fifteen in this one.
//!
//! The irony worth recording: `xref_entry_eol`'s whole *default* was changed on
//! an operator ruling, because a fixed `SP LF` produced a 10,000-byte diff on a
//! file nobody had edited — and the GUI could not honour anything but the
//! default anyway.
//!
//! ## The funnel
//!
//! Three functions — [`Settings::extract_options`], [`Settings::render_options`]
//! and [`Settings::save_options`] — and a rule: **no code in this crate
//! constructs those three types itself.** The rule is not a convention; it is
//! checked, by [`tests::no_call_site_builds_its_own_options`], which parses
//! every `.rs` in the crate with `syn` and fails on a bare constructor outside
//! this module.
//!
//! A grep would not do. `ExtractOptions::default()` appears in a dozen doc
//! comments in this crate — including several in this very header — and a grep
//! would count each one as a violation, or be loosened until it counted none of
//! the real ones. The same argument `shell::commands::reach` and
//! `redact::sealed` already make: the thing being counted is a *call*, and a
//! syntax tree contains no comments at all.
//!
//! ### The three exemptions, and why each is not a hole
//!
//! 1. **Tests and fixtures.** A test that pins the engine's own default
//!    behaviour must be able to say `ExtractOptions::default()`, or it is
//!    testing the operator's configuration instead of the engine's contract.
//!    The check skips `#[cfg(test)]` modules and `ocr/fixture.rs`.
//! 2. **`with_provenance(true)`.** Text editing needs provenance, which no
//!    setting controls. It is a *modifier* on the funnel's output rather than
//!    a second construction: `settings.extract_options().with_provenance(true)`.
//! 3. **Redaction's `SaveOptions::identity()`.** Deliberately NOT funnelled,
//!    and this is the interesting one — see [`Settings::save_options`].
//!
//! ## What is deliberately not here
//!
//! **A watcher on the settings file.** `pdfce-core` refuses one, and the
//! reason binds the shell too: live configuration that depends on when an
//! editor happened to flush is a source of irreproducible behaviour, not a
//! feature.
//!
//! **A save on exit.** `save` is called deliberately, from the Save button, so
//! a crash cannot persist half a session's accidental state and an operator's
//! hand-edited file is never rewritten behind their back with pdfce's own
//! formatting.

use pdfce_core::settings::Settings;
use pdfce_core::text_extract::ExtractOptions;
use pdfce_core::writer::SaveOptions;
use pdfce_render::RenderOptions;

/// The application's view of the operator's configuration.
///
/// A **trait on the engine's type** rather than a wrapper struct. The engine's
/// `Settings` is `#[non_exhaustive]`, so a wrapper would have to re-expose
/// thirteen fields by hand and would go stale the day a fourteenth arrived —
/// whereas an extension trait grows only where it must, which is in the three
/// option builders below.
pub trait SettingsExt {
    /// Text extraction, configured.
    fn extract_options(&self) -> ExtractOptions;
    /// Rasterisation, configured. `annotations` is the caller's, not a
    /// setting's — see the method.
    fn render_options(&self) -> RenderOptions;
    /// Writing, configured.
    fn save_options(&self) -> SaveOptions;
}

impl SettingsExt for Settings {
    /// Every extraction in the application starts here.
    ///
    /// # The three fields, and the one that is a correctness knob
    ///
    /// - `word_gap_ratio` decides where extracted text gets its spaces.
    /// - `actual_text` decides how far a document's own replacement text is
    ///   trusted over the glyphs drawn.
    /// - `unmappable_code` decides what stands in for text pdfce cannot read —
    ///   and it is **not** a cosmetic choice. Downstream of extraction sit
    ///   search, clipboard copy and **redaction-by-text**. Changing the
    ///   sentinel changes character offsets, therefore changes which runs a
    ///   redaction pattern matches. `pdfce-core`'s R35 states it plainly: *a
    ///   redaction built under one value is not equivalent under another.*
    ///
    /// That last point is why both this and `actual_text` have radius lines in
    /// the settings window that name redaction, which the old shell's did not.
    ///
    /// # Why fields rather than builders
    ///
    /// `ExtractOptions` exposes no `with_word_gap_ratio` / `with_unmappable_code`
    /// / `with_actual_text` — checked, not assumed. The fields are `pub` and
    /// the struct is `#[non_exhaustive]`, so the only legal shape out of crate
    /// is *start from `default()` and assign*. Assigning after `default()` is
    /// what `clippy::field_reassign_with_default` complains about, which is why
    /// the binding is `let mut options` on its own line rather than a struct
    /// expression: the lint is about the pattern that *looks like* a struct
    /// literal and is not, and `#[non_exhaustive]` makes the real literal
    /// illegal here.
    fn extract_options(&self) -> ExtractOptions {
        let mut options = ExtractOptions::default();
        options.word_gap_ratio = self.word_gap_ratio;
        options.unmappable_code = self.unmappable_code;
        options.actual_text = self.actual_text;
        options
    }

    /// Every rasterisation in the application starts here.
    ///
    /// # Five settings, and one deliberate absence
    ///
    /// `cmyk_intent`, `mask_resample`, `image_minify`, `cmyk_jpeg_polarity` and
    /// `missing_as` are all read. Four of the five were persisted and ignored
    /// by the old shell, which chained only `.with_annotations()` and
    /// `.with_cmyk_intent()` onto a bare default.
    ///
    /// **Annotation scope is NOT set here**, and that is the absence worth
    /// stating. Whether annotations are drawn is a property of *what is being
    /// rendered for* — the canvas draws them, a print job may not, an export
    /// may be asked either way — and it is passed at the call site. Folding it
    /// in here would give the canvas and the print preview one answer, which is
    /// the opposite of what they need.
    ///
    /// # `missing_as` reaches paper, not just the screen
    ///
    /// It decides what a form control with no stated appearance state looks
    /// like, and the print path renders through this same function. An
    /// operator checking a form before printing it is exactly who that setting
    /// is for, which is why its radius line is the only one that separately
    /// names printing.
    fn render_options(&self) -> RenderOptions {
        RenderOptions::default()
            .with_cmyk_intent(self.cmyk_intent)
            .with_mask_resample(self.mask_resample)
            .with_image_minify(self.image_minify)
            .with_cmyk_jpeg_polarity(self.cmyk_jpeg_polarity)
            .with_missing_as(self.missing_as)
    }

    /// Every save in the application starts here — **except one, on purpose.**
    ///
    /// # The two settings
    ///
    /// `xref_entry_eol` and `trailing_eol`. Neither is visible in a viewer;
    /// both change the bytes on disk, which is why the settings window files
    /// them under *Saving files* and says "nothing visible" rather than
    /// pretending they are cosmetic.
    ///
    /// `ProducerPolicy::Preserve` is carried over from `identity()` rather than
    /// chosen here: it is not a setting, and changing what pdfce writes into
    /// `/Producer` is a decision about attribution rather than about bytes.
    ///
    /// # ★ Redaction does not use this, and must not
    ///
    /// `redact::apply_redactions` is handed `SaveOptions::identity()` directly,
    /// and the [`tests::no_call_site_builds_its_own_options`] check exempts
    /// that one file by name.
    ///
    /// The reason is not that redaction is special-cased for convenience. A
    /// redaction is the one operation in the program whose output is checked,
    /// byte by byte, against a claim — that the removed content is *gone*. The
    /// proof runs over the exact buffer between the constructor and the
    /// syscall. Letting an operator's line-ending preference into that buffer
    /// would mean the bytes proved and the bytes written could differ by a
    /// setting, and the whole guarantee is that they cannot differ by anything.
    ///
    /// A redaction is also not a document the operator is *editing*: it is a
    /// new file produced from an old one, always written as a save-as, and the
    /// "leave untouched objects byte-identical" invariant that motivates
    /// `MatchSource` does not apply to a full rewrite that has deliberately
    /// changed content on every affected page.
    ///
    /// So the exemption is a statement about redaction, not a gap in the
    /// funnel — and it is written down here rather than only in the check,
    /// because a reader who finds the exemption first needs the argument.
    fn save_options(&self) -> SaveOptions {
        let mut options = SaveOptions::identity();
        options.xref_entry_eol = self.xref_entry_eol;
        options.trailing_eol = self.trailing_eol;
        options
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfce_core::settings::{
        ActualTextPrecedence, CmykIntent, CmykJpegPolarity, MaskResample, MinifyFilter,
        MissingAppearanceState, TrailingEol, UnmappableCode, XrefEntryEol,
    };
    use std::path::Path;

    /// A `Settings` whose every funnelled field differs from its default.
    ///
    /// `Settings` is `#[non_exhaustive]`, so a struct expression is illegal out
    /// of crate and this is the only shape available: start from the default
    /// and assign. That is also exactly what the funnel's own implementations
    /// have to do, so the awkwardness is shared rather than incidental.
    fn every_field_moved() -> Settings {
        let mut s = Settings::default();
        s.word_gap_ratio = 0.42;
        s.unmappable_code = UnmappableCode::Omit;
        s.actual_text = ActualTextPrecedence::Glyphs;
        s.cmyk_intent = CmykIntent::Calibrated;
        s.mask_resample = MaskResample::Bilinear;
        s.image_minify = MinifyFilter::Smooth;
        s.cmyk_jpeg_polarity = CmykJpegPolarity::InvertOnApp14;
        s.missing_as = MissingAppearanceState::FirstEntry;
        s.xref_entry_eol = XrefEntryEol::CrLf;
        s.trailing_eol = TrailingEol::None;
        s
    }

    /// ★ **The regression test for the defect this module exists to prevent.**
    ///
    /// Nine of thirteen settings in the old shell were persisted, shown, edited
    /// and never read. This asserts that every field the funnel is responsible
    /// for actually reaches the option struct it belongs to — for all ten of
    /// them at once, from one non-default `Settings`.
    ///
    /// It compares against the value **set**, not against a hard-coded
    /// expectation, so it cannot go stale if an engine default moves. And it
    /// asserts each field individually rather than comparing whole structs,
    /// because a whole-struct comparison would need a second construction and
    /// would then be asserting that two copies of the same code agree.
    #[test]
    fn every_setting_reaches_the_options_it_configures() {
        let s = every_field_moved();

        let extract = s.extract_options();
        assert!((extract.word_gap_ratio - 0.42).abs() < f32::EPSILON);
        assert_eq!(extract.unmappable_code, UnmappableCode::Omit);
        assert_eq!(extract.actual_text, ActualTextPrecedence::Glyphs);

        let save = s.save_options();
        assert_eq!(save.xref_entry_eol, XrefEntryEol::CrLf);
        assert_eq!(save.trailing_eol, TrailingEol::None);

        // `RenderOptions` has no `PartialEq` and its fields are read through
        // the renderer rather than compared here; what is assertable from
        // outside is that the builder chain is total. A default-valued render
        // options and a fully-moved one must not be the same picture, and the
        // cheapest honest statement of that is the debug rendering, which
        // names every field.
        let moved = format!("{:?}", s.render_options());
        let plain = format!("{:?}", Settings::default().render_options());
        assert_ne!(
            moved, plain,
            "render options ignore every setting fed to them"
        );
        for expected in [
            "Calibrated",
            "Bilinear",
            "Smooth",
            "InvertOnApp14",
            "FirstEntry",
        ] {
            assert!(
                moved.contains(expected),
                "render options dropped {expected}: {moved}"
            );
        }
    }

    /// The default settings produce the engine's own defaults, unchanged.
    ///
    /// The other half of the property, and not a tautology: a funnel that
    /// accidentally *forced* a value — say by writing `MaskResample::Nearest`
    /// as a literal instead of reading the field — would pass the test above
    /// whenever the operator happened to want that value, and would pin the
    /// application to one answer forever. This catches it by asserting the
    /// funnel is transparent when it has nothing to say.
    #[test]
    fn default_settings_change_nothing_about_the_engines_own_defaults() {
        let s = Settings::default();
        let extract = s.extract_options();
        let plain = ExtractOptions::default();
        assert!((extract.word_gap_ratio - plain.word_gap_ratio).abs() < f32::EPSILON);
        assert_eq!(extract.unmappable_code, plain.unmappable_code);
        assert_eq!(extract.actual_text, plain.actual_text);

        let save = s.save_options();
        let identity = SaveOptions::identity();
        assert_eq!(save.xref_entry_eol, identity.xref_entry_eol);
        assert_eq!(save.trailing_eol, identity.trailing_eol);
    }

    /// ★ **No call site in this crate builds its own option struct.**
    ///
    /// The rule that keeps the funnel from being a suggestion. Without it, one
    /// new `ExtractOptions::default()` written in good faith next year silently
    /// restores the defect for whichever surface it is on — and, being correct
    /// in isolation, survives review.
    ///
    /// # Why the AST and not a grep
    ///
    /// The identifier `ExtractOptions::default()` appears in a dozen **doc
    /// comments** in this crate, several of them in this module's own header
    /// explaining why it must not be called. A grep counts those and reports
    /// violations that are prose, or is loosened past the point where it
    /// catches the real ones. A syntax tree contains no comments.
    ///
    /// This is the third such check in the crate — `shell::commands::reach`
    /// parses dispatch arms, `redact::sealed` counts one call — and they share
    /// the argument and the `syn` dev-dependency.
    ///
    /// # The exemptions, restated where they are enforced
    ///
    /// - **`app/settings.rs`** — this file. It is the funnel.
    /// - **`ocr/fixture.rs`** — a synthetic-document generator, not a surface.
    /// - **`redact/`** — see [`SettingsExt::save_options`]. The proof must run
    ///   over bytes no setting can vary.
    /// - **`#[cfg(test)]` modules anywhere** — a test pinning the engine's own
    ///   default behaviour must be able to name it, or it is testing the
    ///   operator's configuration instead of the engine's contract.
    #[test]
    fn no_call_site_builds_its_own_options() {
        use syn::visit::Visit;

        /// Constructors that discard the operator's configuration.
        const FORBIDDEN: &[(&str, &str)] = &[
            ("ExtractOptions", "default"),
            ("RenderOptions", "default"),
            ("SaveOptions", "default"),
            ("SaveOptions", "identity"),
        ];

        struct Finder {
            hits: Vec<String>,
        }

        impl<'ast> Visit<'ast> for Finder {
            /// Skip `#[cfg(test)]` modules whole.
            fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
                let is_test_mod = node.attrs.iter().any(|a| {
                    a.path().is_ident("cfg") && a.to_token_stream_string().contains("test")
                });
                if !is_test_mod {
                    syn::visit::visit_item_mod(self, node);
                }
            }

            fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
                if let syn::Expr::Path(path) = &*node.func {
                    let segs: Vec<String> = path
                        .path
                        .segments
                        .iter()
                        .map(|s| s.ident.to_string())
                        .collect();
                    if segs.len() >= 2 {
                        let ty = &segs[segs.len() - 2];
                        let func = &segs[segs.len() - 1];
                        if FORBIDDEN.iter().any(|(t, f)| t == ty && f == func) {
                            self.hits.push(format!("{ty}::{func}()"));
                        }
                    }
                }
                syn::visit::visit_expr_call(self, node);
            }
        }

        /// `syn`'s `Attribute` has no direct "text of the tokens" accessor, so
        /// this trait supplies the one thing the module filter needs. Kept
        /// local because it is a detail of this test and not a facility.
        trait TokensAsString {
            fn to_token_stream_string(&self) -> String;
        }
        impl TokensAsString for syn::Attribute {
            fn to_token_stream_string(&self) -> String {
                match &self.meta {
                    syn::Meta::List(list) => list.tokens.to_string(),
                    _ => String::new(),
                }
            }
        }

        fn walk(dir: &Path, out: &mut Vec<(String, Vec<String>)>) {
            let Ok(entries) = std::fs::read_dir(dir) else {
                return;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk(&path, out);
                    continue;
                }
                if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                    continue;
                }
                let name = path.to_string_lossy().replace('\\', "/");
                // The three exempt files, matched on their path suffix so the
                // check works from any working directory.
                if name.ends_with("app/settings.rs")
                    || name.ends_with("ocr/fixture.rs")
                    || name.contains("/redact/")
                {
                    continue;
                }
                let Ok(text) = std::fs::read_to_string(&path) else {
                    continue;
                };
                let Ok(parsed) = syn::parse_file(&text) else {
                    continue;
                };
                // ★ A whole file gated out of release builds is exempt, and it
                // must be recognised from the AST rather than from the path.
                //
                // `#![cfg(test)]` as an INNER attribute is how this crate marks
                // a module that compiles to nothing in a release build —
                // `canvas::textedit::proof` and `canvas::textedit::cost` both
                // use it, and both must be able to name the engine's own
                // defaults, because what they exist to measure is the *engine's*
                // behaviour and not the operator's configuration. A `cost.rs`
                // that benchmarked extraction under whatever the developer
                // happened to have set would be a benchmark of a preference.
                //
                // This is checked here and not by adding two more filenames to
                // the list above, because the property that earns the exemption
                // is "not in the shipped binary" — and a filename is a
                // restatement of that which goes stale the moment a third such
                // module is written.
                let file_is_test_only = parsed.attrs.iter().any(|attr| {
                    matches!(attr.style, syn::AttrStyle::Inner(_))
                        && attr.path().is_ident("cfg")
                        && attr.to_token_stream_string().contains("test")
                });
                if file_is_test_only {
                    continue;
                }
                let mut finder = Finder { hits: Vec::new() };
                finder.visit_file(&parsed);
                if !finder.hits.is_empty() {
                    out.push((name, finder.hits));
                }
            }
        }

        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        assert!(root.is_dir(), "cannot find src at {}", root.display());
        let mut violations = Vec::new();
        walk(&root, &mut violations);

        assert!(
            violations.is_empty(),
            "these call sites build their own option struct and therefore discard every \
             setting the operator chose — route them through `SettingsExt` instead:\n{}",
            violations
                .iter()
                .map(|(file, hits)| format!("  {file}: {}", hits.join(", ")))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}
