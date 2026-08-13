//! # icons::catalog — which glyphs exist, and what each one means
//!
//! The [`Icon`] enum is the whole vocabulary: one variant per drawn glyph,
//! named for the **role** the icon plays rather than for the artwork, so a
//! future re-draw changes one constant in [`super::assets`] and touches no
//! call site.
//!
//! Salvaged from `D:\Dev\pdfce\crates\pdfce-gui\src\icons.rs` (Class A,
//! `SALVAGE.md`). Every variant's doc comment is carried across, because
//! several of them are not descriptions at all — they are *rulings*. Three
//! kinds recur and each one is a decision somebody paid for:
//!
//! * **"This glyph was authored because a text character had no face."**
//!   [`Icon::Back`], [`Icon::Close`], [`Icon::ChevronUp`],
//!   [`Icon::ChevronDown`] each replace a Unicode character that was
//!   VERIFIED to render as a tofu box in the shipped font stack. The
//!   operator's standing ruling (2026-08-06) is that a missing glyph is
//!   **authored**, not worked around by rewording the control.
//! * **"This glyph must not be that other glyph."** [`Icon::Back`] vs
//!   [`Icon::ChevronLeft`], [`Icon::ShowPoints`] vs [`Icon::EditObjects`],
//!   [`Icon::Layers`] vs [`Icon::Combine`]. Each pair states the shape cue
//!   that keeps them apart at 16 px, and losing that note is how the pair
//!   quietly converges in a later "consistency" pass.
//! * **"An icon is a claim."** [`Icon::Signatures`] must not be a seal,
//!   badge, shield or checkmark, because pdfce performs no cryptographic
//!   verification and those shapes read as VALIDATED. [`Icon::Fonts`] must
//!   not be a pencil or an I-beam, because the Fonts panel writes nothing.
//!   A glyph reaches the operator's eye before the panel's first line does.
//!
//! ## The one key namespace
//!
//! [`Icon::name`] is the string an `egui_shell::Command` names with
//! `.with_icon("…")`, and [`Icon::from_key`] is the reverse. There is
//! exactly one spelling of each key and it lives in `name`; `from_key`
//! searches [`Icon::ALL`] rather than carrying a second `match`, so the two
//! cannot drift. `every_name_round_trips_through_from_key` pins it anyway,
//! because "cannot drift" is a property of today's implementation and the
//! test is a property of the contract.

use super::assets;

/// Every icon pdfce ships, one variant per drawn glyph.
///
/// Two roles deliberately share one asset: [`Icon::Open`] and
/// [`Icon::FontFolders`] are both the plain folder glyph — Open is a
/// top-level action and Font Folders is a labelled row three levels into a
/// dock, and they are never on screen together.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Icon {
    /// Open a file. ScripTree `icon-folder.svg`.
    Open,
    /// Save a copy.
    Save,
    /// Thumbnail-rail visibility toggle ("sidebar").
    Sidebar,
    /// Annotation-visibility toggle ("comment-bubble").
    Comment,
    /// Previous page ("chevron").
    ChevronLeft,
    /// Leave a ribbon-opened surface and return to the armed tools' own
    /// options ("back-arrow").
    ///
    /// Authored 2026-08-06 under the operator's standing ruling that a
    /// missing glyph is AUTHORED, not worked around: the control wanted `←`
    /// (U+2190), the coverage gate correctly rejected it as having no glyph
    /// in the shipped stack, and the first fix reworded the button to plain
    /// text. Rewording spends the operator's affordance to protect the font
    /// stack; an icon costs one asset and keeps both.
    ///
    /// Distinct from [`Icon::ChevronLeft`] by its SHAFT — see `back.svg`'s
    /// own embedded note. Same reasoning that made [`Icon::ChevronUp`] and
    /// [`Icon::ChevronDown`] exist: those two were also authored precisely
    /// because their text glyphs were tofu.
    Back,
    /// Next page.
    ChevronRight,
    /// "Move selection up" in the page rail and the Combine-files list,
    /// drawn instead of the text glyph `▲` (U+25B2) — VERIFIED tofu in the
    /// running build 2026-08-03, same Geometric Shapes block as `▾`. Those
    /// buttons are glyph-ONLY, so a missing glyph left them with no visible
    /// identity at all.
    ChevronUp,
    /// Menu-disclosure marker on a dropdown button, drawn instead of the
    /// text glyph `▾` (U+25BE) — which is absent from every font in egui's
    /// default Proportional chain and rendered as a tofu box on four shipped
    /// toolbar controls.
    ChevronDown,
    /// A magnifier — Find. Empty lens: [`Icon::ZoomIn`]/[`Icon::ZoomOut`]
    /// are the same shape carrying a `+`/`-`, so the unmarked lens is what
    /// says "search" rather than "magnify by".
    Search,
    /// Dismiss / remove — drawn instead of the text glyph `✕` (U+2715),
    /// which is absent from every font of the shipped stack.
    ///
    /// Authored rather than reworded, per the operator's 2026-08-06 ruling
    /// that a missing glyph for a real control gets an icon created for it.
    Close,
    /// Zoom out ("magnifier±").
    ZoomOut,
    /// Zoom in.
    ZoomIn,
    /// Fit whole page ("frame-fit").
    FitPage,
    /// Fit page width ("frame-fit-width").
    FitWidth,
    /// Rotate page counter-clockwise ("rotate-page").
    RotateCcw,
    /// Rotate page clockwise.
    RotateCw,
    /// Document properties. ScripTree `icon-document.svg`.
    Properties,
    /// Markup menu ("shapes").
    Markup,
    /// Text menu ("note").
    Text,
    /// Edit page text. ScripTree `icon-edit.svg`.
    EditText,
    /// Add page text ("text-cursor-plus").
    AddText,
    /// Edit vector objects — the "Obj" tool. Not in the ui-spec (that tool
    /// shipped after the spec was written); authored in the same style
    /// contract, see [`super::assets`] §5.
    EditObjects,
    /// Create an interactive form field — the "Create Field" tool. Authored
    /// 2026-08-07; not in the ui-spec, same style contract as
    /// [`Icon::EditObjects`].
    ///
    /// Deliberately NOT reusing any existing asset. The nearest candidates
    /// each would have said something false: `edit-objects.svg` promises
    /// vector editing, and a plain box would read as the FILL surface, which
    /// is a different capability in a different ribbon group.
    FormField,
    /// Measure/dimension menu. ScripTree `icon-ruler.svg`.
    Measure,
    /// Undo ("history-arrow").
    Undo,
    /// Redo.
    Redo,
    /// Copy-text menu ("copy").
    Copy,
    /// Tools dock toggle. ScripTree `icon-tool.svg`.
    Tools,
    /// Keyboard-shortcuts window ("keyboard").
    Keyboard,
    /// "Show points" view toggle — draws every anchor of the object being
    /// worked inside, so the points can be aimed at BEFORE one is selected.
    ///
    /// Deliberately close to, and deliberately distinct from,
    /// [`Icon::EditObjects`]: both show square node marks, because both are
    /// about the same points. That one puts two on a Bézier (shape editing);
    /// this one puts three on a straight run with the middle one offset (the
    /// points themselves, and the canvas's selected-node vocabulary).
    ShowPoints,
    /// Bookmarks panel toggle — the document's outline.
    ///
    /// A ribbon with a notch, which is the one shape read as "bookmark"
    /// without a label. Deliberately not a page-with-lines: that is
    /// [`Icon::Properties`]/document territory, and this panel is about
    /// places IN a document rather than the document itself.
    Bookmarks,
    /// Layers panel toggle — optional content.
    ///
    /// Three stacked sheets. Three rather than two so it does not read as
    /// [`Icon::Combine`]'s linked pair at 16 px.
    Layers,
    /// Signatures panel toggle.
    ///
    /// A written flourish over a signing rule, and emphatically **not** a
    /// seal, badge, shield or checkmark: each of those reads as VALIDATED,
    /// and pdfce performs no cryptographic verification. The panel's first
    /// line says so; the glyph must not contradict it before the panel is
    /// open. An icon is a claim too.
    Signatures,
    /// Fonts panel toggle — the document's font inventory.
    ///
    /// A capital A on a baseline rule. The letterform reads as "type"; the
    /// rule under it is what stops it reading as "a text tool", which
    /// matters because this panel writes nothing and an icon borrowed from
    /// an editing tool would suggest otherwise. Deliberately not
    /// [`Icon::AddText`]'s I-beam-plus or [`Icon::EditText`]'s pencil for
    /// that reason.
    Fonts,
    /// Markup → Rectangle.
    ShapeRect,
    /// Markup → Ellipse.
    ShapeEllipse,
    /// Markup → Arrow line.
    ShapeArrow,
    /// Markup → Highlight band.
    ShapeHighlight,
    /// Text → FreeText box.
    TextFreeText,
    /// Text → Sticky note.
    TextSticky,
    /// Text → Stamp, and the reserved Bates-numbering glyph.
    Stamp,
    /// Combine files…. ScripTree `icon-link.svg`.
    Combine,
    /// Split this document…. ScripTree `icon-scissors.svg`.
    Split,
    /// Insert pages from a file…. ScripTree `icon-upload.svg`.
    InsertPages,
    /// Font folders… — the same folder art as [`Icon::Open`].
    FontFolders,
    /// Redaction.
    ///
    /// It is the one intentionally solid-FILLED glyph in an otherwise
    /// all-outline set, which is also why it is the pipeline's only coverage
    /// of the fill path (see `redaction_is_the_only_filled_icon`). The fill
    /// is not decoration: every other tool in this app draws or measures,
    /// and this one obliterates, so its glyph reads as a solid bar rather
    /// than an outline of one.
    Redact,
}

impl Icon {
    /// Every icon, in catalogue order.
    ///
    /// This is the list the catalogue-wide tests walk, and it is what makes
    /// "every shipped asset is valid" an enforced property rather than a
    /// hope — so a new [`Icon`] variant MUST be added here or it ships
    /// unverified. `all_is_exhaustive` guards the omission that would
    /// otherwise be invisible.
    pub const ALL: &'static [Icon] = &[
        Icon::Open,
        Icon::Save,
        Icon::Sidebar,
        Icon::Comment,
        Icon::ChevronLeft,
        Icon::Back,
        Icon::ChevronRight,
        Icon::ChevronDown,
        Icon::Search,
        Icon::ChevronUp,
        Icon::Close,
        Icon::ZoomOut,
        Icon::ZoomIn,
        Icon::FitPage,
        Icon::FitWidth,
        Icon::RotateCcw,
        Icon::RotateCw,
        Icon::Properties,
        Icon::Markup,
        Icon::Text,
        Icon::EditText,
        Icon::AddText,
        Icon::EditObjects,
        Icon::FormField,
        Icon::Measure,
        Icon::Undo,
        Icon::Redo,
        Icon::Copy,
        Icon::Tools,
        Icon::Keyboard,
        Icon::ShowPoints,
        Icon::Bookmarks,
        Icon::Layers,
        Icon::Signatures,
        Icon::Fonts,
        Icon::ShapeRect,
        Icon::ShapeEllipse,
        Icon::ShapeArrow,
        Icon::ShapeHighlight,
        Icon::TextFreeText,
        Icon::TextSticky,
        Icon::Stamp,
        Icon::Combine,
        Icon::Split,
        Icon::InsertPages,
        Icon::FontFolders,
        Icon::Redact,
    ];

    /// The asset's SVG source.
    ///
    /// A compiled-in constant rather than a runtime file read because pdfce
    /// ships single-folder portable: the executable must not depend on an
    /// `assets/` directory travelling beside it, and an icon that fails to
    /// load at startup is not a failure mode worth having when the whole set
    /// is 34 KB of text. See [`super::assets`] for why the text is a Rust
    /// constant rather than an `include_str!` of a sibling directory.
    #[must_use]
    pub const fn source(self) -> &'static str {
        match self {
            Icon::Open | Icon::FontFolders => assets::FOLDER,
            Icon::Save => assets::SAVE,
            Icon::Sidebar => assets::SIDEBAR,
            Icon::Comment => assets::COMMENT,
            Icon::ChevronLeft => assets::CHEVRON_LEFT,
            Icon::Back => assets::BACK,
            Icon::ChevronRight => assets::CHEVRON_RIGHT,
            Icon::ChevronDown => assets::CHEVRON_DOWN,
            Icon::Search => assets::SEARCH,
            Icon::ChevronUp => assets::CHEVRON_UP,
            Icon::Close => assets::CLOSE,
            Icon::ZoomOut => assets::ZOOM_OUT,
            Icon::ZoomIn => assets::ZOOM_IN,
            Icon::FitPage => assets::FIT_PAGE,
            Icon::FitWidth => assets::FIT_WIDTH,
            Icon::RotateCcw => assets::ROTATE_CCW,
            Icon::RotateCw => assets::ROTATE_CW,
            Icon::Properties => assets::DOCUMENT,
            Icon::Markup => assets::MARKUP,
            Icon::Text => assets::TEXT,
            Icon::EditText => assets::EDIT,
            Icon::AddText => assets::ADD_TEXT,
            Icon::FormField => assets::FORM_FIELD,
            Icon::EditObjects => assets::EDIT_OBJECTS,
            Icon::ShowPoints => assets::SHOW_POINTS,
            Icon::Bookmarks => assets::BOOKMARKS,
            Icon::Layers => assets::LAYERS,
            Icon::Signatures => assets::SIGNATURES,
            Icon::Fonts => assets::FONTS,
            Icon::Measure => assets::RULER,
            Icon::Undo => assets::UNDO,
            Icon::Redo => assets::REDO,
            Icon::Copy => assets::COPY,
            Icon::Tools => assets::TOOL,
            Icon::Keyboard => assets::KEYBOARD,
            Icon::ShapeRect => assets::SHAPE_RECT,
            Icon::ShapeEllipse => assets::SHAPE_ELLIPSE,
            Icon::ShapeArrow => assets::SHAPE_ARROW,
            Icon::ShapeHighlight => assets::SHAPE_HIGHLIGHT,
            Icon::TextFreeText => assets::TEXT_FREETEXT,
            Icon::TextSticky => assets::TEXT_STICKY,
            Icon::Stamp => assets::STAMP,
            Icon::Combine => assets::LINK,
            Icon::Split => assets::SCISSORS,
            Icon::InsertPages => assets::UPLOAD,
            Icon::Redact => assets::REDACT,
        }
    }

    /// The stable key this icon answers to.
    ///
    /// Two jobs, and they are the same string on purpose:
    ///
    /// 1. **It is the application's icon key**, the thing a command names
    ///    with `.with_icon("…")` and the thing `egui-shell` hands back in
    ///    `IconRequest::key`. The shell never interprets it — an icon set is
    ///    a licensing and rasterization decision, which is the application's
    ///    business — so this is the only place the vocabulary is defined.
    /// 2. **It is the texture's debug name.** egui keys textures by handle,
    ///    not by name, so that part is purely for debuggers and texture
    ///    inspectors — but a texture list full of "icon" tells you nothing,
    ///    and one full of `icon:rotate-ccw@32:Bold` tells you everything.
    ///
    /// Kebab-case throughout, matching the command ids and the asset
    /// filenames it was salvaged from.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Icon::Open => "open",
            Icon::Save => "save",
            Icon::Sidebar => "sidebar",
            Icon::Comment => "comment",
            Icon::Close => "close",
            Icon::ChevronLeft => "chevron-left",
            Icon::Back => "back",
            Icon::ChevronRight => "chevron-right",
            Icon::ChevronDown => "chevron-down",
            Icon::Search => "search",
            Icon::ChevronUp => "chevron-up",
            Icon::ZoomOut => "zoom-out",
            Icon::ZoomIn => "zoom-in",
            Icon::FitPage => "fit-page",
            Icon::FitWidth => "fit-width",
            Icon::RotateCcw => "rotate-ccw",
            Icon::RotateCw => "rotate-cw",
            Icon::Properties => "properties",
            Icon::Markup => "markup",
            Icon::Text => "text",
            Icon::EditText => "edit-text",
            Icon::AddText => "add-text",
            Icon::FormField => "form-field",
            Icon::EditObjects => "edit-objects",
            Icon::ShowPoints => "show-points",
            Icon::Bookmarks => "bookmarks",
            Icon::Layers => "layers",
            Icon::Signatures => "signatures",
            Icon::Fonts => "fonts",
            Icon::Measure => "measure",
            Icon::Undo => "undo",
            Icon::Redo => "redo",
            Icon::Copy => "copy",
            Icon::Tools => "tools",
            Icon::Keyboard => "keyboard",
            Icon::ShapeRect => "shape-rect",
            Icon::ShapeEllipse => "shape-ellipse",
            Icon::ShapeArrow => "shape-arrow",
            Icon::ShapeHighlight => "shape-highlight",
            Icon::TextFreeText => "text-freetext",
            Icon::TextSticky => "text-sticky",
            Icon::Stamp => "stamp",
            Icon::Combine => "combine",
            Icon::Split => "split",
            Icon::InsertPages => "insert-pages",
            Icon::FontFolders => "font-folders",
            Icon::Redact => "redact",
        }
    }

    /// Resolve an application icon key back to an [`Icon`].
    ///
    /// This is the lookup [`super::paint_ribbon_icon`] performs on every
    /// icon-bearing ribbon control, every frame.
    ///
    /// # Why a linear scan and not a `match` or a `HashMap`
    ///
    /// A reverse `match` would be a second copy of the key vocabulary, and
    /// two copies of a mapping is exactly how a rename lands in one of them.
    /// [`Icon::name`] stays the single source of truth and this walks it.
    ///
    /// The cost is 47 pointer-length comparisons with an early exit, for the
    /// handful of icons a ribbon draws per frame — comfortably under a
    /// microsecond, against a frame budget of 16 ms. A `HashMap` would need
    /// a lazily-initialised static, would hash the key anyway, and would buy
    /// nothing measurable. If the set ever reaches the hundreds, revisit;
    /// `every_name_round_trips_through_from_key` makes the swap safe.
    #[must_use]
    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|icon| icon.name() == key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// ★ [`Icon::ALL`] must really be all of them.
    ///
    /// Everything catalogue-wide — "every asset parses", "every asset
    /// rasterizes to something visible", "redaction is the only filled one"
    /// — iterates `ALL`. A variant left out of it is therefore not merely
    /// untested: it is *silently* untested, and a broken asset behind it
    /// ships green.
    ///
    /// There is no reflection in Rust to count enum variants, so this checks
    /// the two things that would actually go wrong: a duplicate entry (a
    /// copy-paste that hid the variant it was meant to add) and a count that
    /// no longer matches the number of distinct keys.
    #[test]
    fn all_is_exhaustive_and_free_of_duplicates() {
        let unique: HashSet<Icon> = Icon::ALL.iter().copied().collect();
        assert_eq!(
            unique.len(),
            Icon::ALL.len(),
            "Icon::ALL contains a duplicate variant"
        );
        assert_eq!(
            Icon::ALL.len(),
            47,
            "the catalogue changed size: add the new variant to Icon::ALL and update this count"
        );
    }

    /// Every key is unique. Two icons answering to one key would make
    /// [`Icon::from_key`] return whichever came first in `ALL`, which is a
    /// silently-wrong glyph rather than a missing one — the worse failure.
    #[test]
    fn every_name_is_distinct() {
        let mut seen: HashSet<&str> = HashSet::new();
        for &icon in Icon::ALL {
            assert!(
                seen.insert(icon.name()),
                "duplicate icon key '{}'",
                icon.name()
            );
        }
    }

    /// ★ The key vocabulary has exactly one definition.
    ///
    /// [`Icon::from_key`] is documented as the inverse of [`Icon::name`].
    /// This is what keeps that true if `from_key` is ever rewritten as a
    /// `match` or a map for speed.
    #[test]
    fn every_name_round_trips_through_from_key() {
        for &icon in Icon::ALL {
            assert_eq!(
                Icon::from_key(icon.name()),
                Some(icon),
                "'{}' did not round-trip",
                icon.name()
            );
        }
    }

    /// An unknown key resolves to nothing rather than to something plausible.
    ///
    /// The whole missing-icon story downstream ([`super::super::paint`])
    /// depends on this returning `None` instead of guessing at a nearest
    /// match: a fuzzy resolver would draw the *wrong* glyph for a typo,
    /// which is undetectable, where `None` is drawn as a visible mark and
    /// traced.
    #[test]
    fn an_unknown_key_resolves_to_nothing() {
        assert_eq!(Icon::from_key("no-such-icon"), None);
        assert_eq!(Icon::from_key(""), None);
        // Case and separator variants are NOT accepted: the vocabulary is
        // kebab-case, exactly, and a near-miss should be reported rather
        // than silently repaired.
        assert_eq!(Icon::from_key("Open"), None);
        assert_eq!(Icon::from_key("fit_page"), None);
    }

    /// Keys are kebab-case with no surprises, because they appear verbatim
    /// in command definitions that a human types by hand.
    #[test]
    fn keys_are_lowercase_kebab_case() {
        for &icon in Icon::ALL {
            let name = icon.name();
            assert!(!name.is_empty(), "{icon:?} has an empty key");
            assert!(
                name.bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-'),
                "icon key '{name}' is not lowercase kebab-case"
            );
        }
    }

    /// The two roles that deliberately share one asset still share it, and
    /// nothing else accidentally does.
    ///
    /// Asset sharing is a real decision (one folder glyph, two places it
    /// appears, never simultaneously) but an *accidental* share means two
    /// controls that should be distinguishable are not — which reads to an
    /// operator as a wiring bug in whichever control they clicked second.
    #[test]
    fn only_the_folder_asset_is_shared() {
        let mut by_source: std::collections::HashMap<&str, Vec<Icon>> =
            std::collections::HashMap::new();
        for &icon in Icon::ALL {
            by_source.entry(icon.source()).or_default().push(icon);
        }
        for (_, icons) in by_source {
            if icons.len() > 1 {
                let mut names: Vec<&str> = icons.iter().map(|i| i.name()).collect();
                names.sort_unstable();
                assert_eq!(
                    names,
                    vec!["font-folders", "open"],
                    "an unexpected pair of icons shares one asset"
                );
            }
        }
    }

    /// Every variant has non-empty art. A `source()` arm wired to the wrong
    /// (or an empty) constant would otherwise only show up as a blank
    /// button.
    #[test]
    fn every_icon_has_source_text() {
        for &icon in Icon::ALL {
            let src = icon.source();
            assert!(
                src.contains("<svg"),
                "icon '{}' has no <svg> root in its source",
                icon.name()
            );
        }
    }
}
