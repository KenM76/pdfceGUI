//! # `text::status` — every string the status bar shows
//!
//! One area of the catalog described in [`crate::text`]'s header, and the
//! sole consumer is [`crate::app::status`]. Nothing here is read by the
//! ribbon: the status bar **mirrors** three View-tab commands under
//! amendment P1a (`RIBBON_IA.md` §2), and a mirror is a second surface for
//! one command, not a second command.
//!
//! ## ★ The one place this file deliberately repeats the ribbon
//!
//! [`fit_actual_size`] and [`fit_actual_size_tooltip`] say what
//! `crate::text::commands::view_zoom_actual` says, in the same words. That
//! is not an oversight and it is not a copy-paste that should be
//! de-duplicated into a shared constant:
//!
//! - The two surfaces are mirrors of **one** command, so an operator who
//!   reads the ribbon's tooltip and then hovers the status bar's button
//!   must be told the same thing. Two paraphrases of one command is how a
//!   product acquires two different mental models of the same verb.
//! - They are nevertheless two *entries*, because the ribbon's catalog is
//!   keyed by command id and consumed by `crate::shell::commands`, while
//!   this one is keyed by control and consumed by a widget. Reaching across
//!   would make `text::status` depend on `text::commands`' `CommandText`
//!   type for no gain, and would put the first cross-area dependency in a
//!   catalog whose whole organising principle is one area per consumer.
//!
//! **Both entries are now true**, and the wording being identical to the
//! ribbon's is what made fixing them one edit rather than two. The action
//! behind them raises `Action::ZoomTo(1.0)` (see the ★ section of
//! [`crate::app::status`]'s module docs for that half), and the chord they
//! name has one owner (see [`crate::app::keyboard`]'s ★ section for this
//! one). The same holds for the three mirrored fit tooltips: each is now
//! word-for-word its `crate::text::commands` twin, chord included.
//!
//! ## Why the arrows and the minus sign are in the catalog
//!
//! `⏴`, `⏵`, `⏷`, `−`, `+`, `·` are *labels*: they are the entire visible
//! text of a control, and a control's visible text is exactly what rule R1
//! governs. The `check-ui-strings.sh` heuristic would never catch them —
//! it flags literals containing whitespace, and these contain none — so
//! they are here by the rule rather than by the gate, which is the
//! distinction that file's own header draws.
//!
//! They are also the reason
//! [`crate::app::status::tests::every_glyph_the_status_bar_draws_has_a_glyph`]
//! exists, and **that test has already paid for itself**. This file was
//! written with the obvious glyphs — `◀` `▶` for the page steps and `▸` `▾`
//! for the disclosure — and every one of the four is **missing from egui's
//! bundled font set** (Ubuntu-Light + NotoEmoji + emoji-icon-font). On
//! screen they would have rendered as tofu boxes: defect D2's shape, an
//! invisible label, with the operator's page position behind it. What the
//! font set does carry, measured rather than assumed, is `⏴ ⏵ ⏶ ⏷ ‹ › « »
//! ○ • · – — − + % /`.
//!
//! ## Conventions, restated from [`crate::text`] because they bind here
//!
//! - **Sentence case, no trailing period on labels; full sentences with
//!   punctuation for prose.** Every tooltip below is prose and ends in a
//!   full stop; every label is a name and does not.
//! - **Name the chord only when the chord works.** Actual size names
//!   `Ctrl+0` because the manifest keymap binds it there and
//!   `crate::app::keyboard::commands` enacts what the keymap says. Fit page
//!   and Fit width name **none**, because none reaches them: `Ctrl+0` is
//!   actual size's and `Ctrl+2` is `mode.review`'s. Both used to be named
//!   here, and both were half of a chord with two owners — the rule is what
//!   caught it, and the rule is why the correction is an omission rather
//!   than a substitution. Do not invent a replacement chord to fill the gap;
//!   bind one in the manifest first, and it may be named the same day.
//! - **Never state a capability the build does not have.** The Find toggle
//!   `RIBBON_IA.md` §6 specifies has **no strings here**, and that is now a
//!   filing decision rather than an absence: the toggle exists, and its label
//!   and tooltip live in [`crate::text::find`] beside the rest of the Find
//!   surface's copy. One area per consumer is this catalog's organising
//!   principle, and the toggle's consumer is a control the Find module owns.
//!   (It used to say the toggle had no strings *because it had no command*.
//!   That is no longer true, and the sentence is corrected rather than
//!   deleted, because "this used to be absent and is now built" is exactly
//!   what a catalog header should make legible.)

// ---------------------------------------------------------------------------
// The narrator — the render diagnostics disclosure
// ---------------------------------------------------------------------------

/// The disclosure control's label, closed and open.
///
/// ★ **Closed is the default, and the caption is still shown.** `DEFECTS.md`
/// records the old shell opening with a substitute-glyph census: *"The first
/// thing a user reads is the app talking about itself. Excellent
/// information, wrong prominence."* The fix is prominence, not deletion — so
/// the report is one click away and named, rather than hidden behind a bare
/// triangle nobody would think to press.
///
/// "Render notes" rather than "Diagnostics": the operator's question is
/// *"did pdfce draw my page faithfully?"*, and "diagnostics" is the word an
/// application uses about itself.
///
/// ★ **The triangles are `⏵` (U+23F5) and `⏷` (U+23F7), and the choice was
/// forced by measurement rather than taste.** The obvious glyphs for a
/// disclosure — `▸` U+25B8 and `▾` U+25BE — are **absent from egui's
/// bundled font set** (Ubuntu-Light + NotoEmoji + emoji-icon-font), as are
/// `▶`/`◀`. They were in this file first and
/// [`crate::app::status::tests::every_glyph_the_status_bar_draws_has_a_glyph`]
/// caught them: on screen they would have been tofu boxes, which on a
/// disclosure means an operator cannot tell open from closed.
///
/// `⏵` is therefore also [`next_page`]'s glyph, which is a real (small)
/// collision and is accepted rather than worked around: the two controls sit
/// at **opposite ends** of the bar, this one always carries the word "Render
/// notes" beside it, and this one alternates while the page arrows never do.
/// Substituting a non-triangle here — `›`, `»` — would trade a resolvable
/// ambiguity for a control that no longer looks like a disclosure at all.
#[must_use]
pub fn diagnostics_toggle(open: bool) -> &'static str {
    if open {
        "⏷ Render notes"
    } else {
        "⏵ Render notes"
    }
}

/// Hover text for the disclosure.
///
/// Says what the report is *about*, because the difference between "pdfce
/// approximated something" and "your document is damaged" is the single
/// most valuable thing this surface can teach.
#[must_use]
pub fn diagnostics_tooltip() -> &'static str {
    "What pdfce had to substitute or leave out when it drew this page. \
     These are facts about the renderer, not faults in your document."
}

/// Shown when the page drew with nothing substituted and nothing skipped.
///
/// Stated positively rather than left blank. An empty disclosure is
/// indistinguishable from a disclosure that failed to fill itself, and the
/// operator who opened it wanted an answer either way.
#[must_use]
pub fn diagnostics_clean() -> &'static str {
    "Drawn with nothing substituted or left out"
}

/// Glyphs painted from a **bundled** substitute face.
///
/// Positions are the document's own; the shapes are pdfce's. Worth its own
/// line rather than being folded into [`diagnostics_glyphs_supplied`],
/// because the two have different remedies: a bundled substitute is fixed by
/// supplying the real font, and a supplied one is already the operator's own
/// deliberate choice.
#[must_use]
pub fn diagnostics_glyphs_substituted(n: usize) -> String {
    if n == 1 {
        "1 glyph drawn with a bundled substitute face".to_owned()
    } else {
        format!("{n} glyphs drawn with a bundled substitute face")
    }
}

/// Glyphs painted from an **operator-supplied** face.
#[must_use]
pub fn diagnostics_glyphs_supplied(n: usize) -> String {
    if n == 1 {
        "1 glyph drawn from a supplied font".to_owned()
    } else {
        format!("{n} glyphs drawn from a supplied font")
    }
}

/// Glyphs that had no shape at all — `.notdef`, or nothing painted.
#[must_use]
pub fn diagnostics_glyphs_notdef(n: usize) -> String {
    if n == 1 {
        "1 glyph with no shape available".to_owned()
    } else {
        format!("{n} glyphs with no shape available")
    }
}

/// Whole fonts whose machinery this build does not implement; their text was
/// **skipped**, not approximated.
///
/// Worded as "text not drawn" rather than "fonts unsupported" because the
/// consequence is what the operator can see on the page. A count of
/// unsupported fonts is a fact about pdfce; missing text is a fact about the
/// picture in front of them.
#[must_use]
pub fn diagnostics_fonts_skipped(n: usize) -> String {
    if n == 1 {
        "text from 1 font not drawn".to_owned()
    } else {
        format!("text from {n} fonts not drawn")
    }
}

/// Images that could not be drawn at all.
#[must_use]
pub fn diagnostics_images_skipped(n: usize) -> String {
    if n == 1 {
        "1 image not drawn".to_owned()
    } else {
        format!("{n} images not drawn")
    }
}

/// Operators recognised but not yet implemented.
#[must_use]
pub fn diagnostics_ops_deferred(n: usize) -> String {
    if n == 1 {
        "1 drawing operator not yet implemented".to_owned()
    } else {
        format!("{n} drawing operators not yet implemented")
    }
}

/// Operators not recognised at all.
///
/// Distinct from [`diagnostics_ops_deferred`]: "not implemented" is a gap in
/// pdfce with a name, and "unrecognised" means the content stream contained
/// something no version of pdfce expects — which is usually a fact about the
/// file.
#[must_use]
pub fn diagnostics_ops_unknown(n: usize) -> String {
    if n == 1 {
        "1 unrecognised drawing operator".to_owned()
    } else {
        format!("{n} unrecognised drawing operators")
    }
}

/// Optional-content sections that were hidden and therefore not drawn.
///
/// Reported even though hiding a layer is usually the operator's own doing,
/// because the alternative reading of a suddenly-emptier page is "the render
/// failed". Naming the cause is the difference between a control working and
/// a control looking broken.
#[must_use]
pub fn diagnostics_layers_hidden(n: usize) -> String {
    if n == 1 {
        "1 hidden layer section not drawn".to_owned()
    } else {
        format!("{n} hidden layer sections not drawn")
    }
}

/// `/Contents` entries that named an object the file does not contain.
///
/// The one entry here that is a statement about the **document** rather than
/// about the renderer, and it is worded that way: the page is incomplete
/// because part of it is missing from the file, not because pdfce declined
/// to draw it.
#[must_use]
pub fn diagnostics_contents_missing(n: usize) -> String {
    if n == 1 {
        "1 content stream missing from the file".to_owned()
    } else {
        format!("{n} content streams missing from the file")
    }
}

/// Join the notes into the single line the disclosure shows.
///
/// The separator lives here rather than at the call site because it is
/// operator-visible punctuation, and because putting it in the widget would
/// be the first crack in "every string a human can read is defined here".
///
/// `·` (U+00B7) rather than a comma: the parts are independent facts, not a
/// list in a sentence, and a middle dot survives being read at a glance in a
/// small weak font better than a comma does.
#[must_use]
pub fn diagnostics_join(parts: &[String]) -> String {
    parts.join(" · ")
}

// ---------------------------------------------------------------------------
// Zoom
// ---------------------------------------------------------------------------

/// The zoom-out button's label — `−` (U+2212 MINUS SIGN).
///
/// Not the ASCII hyphen. A hyphen next to a `+` reads as a dash rather than
/// as an operator, and the two controls are meant to be seen as a pair.
#[must_use]
pub fn zoom_out() -> &'static str {
    "−"
}

/// Hover text for zoom out.
#[must_use]
pub fn zoom_out_tooltip() -> &'static str {
    "Zoom out one step (Ctrl+Minus)."
}

/// The zoom-in button's label.
#[must_use]
pub fn zoom_in() -> &'static str {
    "+"
}

/// Hover text for zoom in.
#[must_use]
pub fn zoom_in_tooltip() -> &'static str {
    "Zoom in one step (Ctrl+Plus)."
}

/// The current zoom, as a whole percentage.
///
/// A **readout**, not a control: this build has no way to set an arbitrary
/// zoom by typing, so an editable box here would be an affordance for
/// something that cannot happen. The page number beside it *is* editable
/// because `Action::GoToPage` exists; there is no `Action` that sets a zoom
/// to a named value, and inventing a text box in front of one would be the
/// placeholder the project's invariants forbid.
#[must_use]
pub fn zoom_percent(percent: u32) -> String {
    format!("{percent}%")
}

/// Hover text for the zoom readout.
///
/// Explains the ladder, because "why did 137% become 150%?" is the question
/// the readout provokes and the answer is a deliberate design choice
/// (`crate::viewer`'s module docs: a fixed ladder makes zoom-in-then-out
/// exactly reversible).
#[must_use]
pub fn zoom_percent_tooltip() -> &'static str {
    "The current zoom. The − and + buttons step a fixed ladder of familiar \
     percentages, so zooming in and back out returns to exactly where you \
     started."
}

// ---------------------------------------------------------------------------
// Fit — the three View-tab mirrors (amendment P1a)
// ---------------------------------------------------------------------------

/// The Actual size button's label.
///
/// **Identical to `crate::text::commands::view_zoom_actual`'s label**, on
/// purpose — see this module's header for why a mirror repeats rather than
/// paraphrases, and [`crate::app::status`]'s ★ section for why the claim it
/// makes is not yet true.
#[must_use]
pub fn fit_actual_size() -> &'static str {
    "Actual size"
}

/// Hover text for Actual size.
///
/// ★ **It names `Ctrl+0` again, and that sentence is now true.** The chord
/// had two owners — the manifest keymap bound it to `view.zoom_actual` while
/// `crate::app::keyboard` bound it to Fit page and reached it first — so this
/// tooltip had to advertise no chord at all, with a test pinning the
/// omission. `crate::app::keyboard`'s ★ section has the whole account; the
/// outcome is that the manifest is the only place a chord is bound, and it
/// binds this one here.
///
/// Word for word `crate::text::commands::view_zoom_actual`'s tooltip,
/// including the chord — see this module's header on why a mirror repeats
/// rather than paraphrases.
#[must_use]
pub fn fit_actual_size_tooltip() -> &'static str {
    "Show the page at actual size — one PDF point per screen point (Ctrl+0)."
}

/// The Fit width button's label.
#[must_use]
pub fn fit_width() -> &'static str {
    "Fit width"
}

/// Hover text for Fit width.
///
/// Says "and keep it fitted", because a fit is a **mode** here rather than a
/// one-shot: resizing the window re-fits. A viewer that stopped fitting on
/// the first resize would be conspicuously wrong, and the tooltip is where
/// the operator learns which of the two this is.
///
/// ★ **It no longer names `Ctrl+2`.** That chord belongs to `mode.review`
/// (`MODES_AND_PANELS.md` Part 1 §6, and `crate::text::commands::mode_review`
/// names it), and `crate::app::keyboard` bound it here as well — one chord,
/// two owners, of which this was the half nothing but this string admitted
/// to. Fit width keeps this button, its View ▸ Zoom control and its
/// `canvas.empty` context-menu entry; what it does not have is a chord, and
/// the rule in this module's header is to say so by omission rather than to
/// name one that does something else.
#[must_use]
pub fn fit_width_tooltip() -> &'static str {
    "Scale the page so its full width is visible, and keep it fitted as the \
     window resizes."
}

/// The Fit page button's label.
#[must_use]
pub fn fit_page() -> &'static str {
    "Fit page"
}

/// Hover text for Fit page.
///
/// ★ **It no longer names `Ctrl+0`.** See [`fit_actual_size_tooltip`]: that
/// chord now has one owner, the manifest keymap, and the manifest binds it to
/// actual size. Fit page is reached from this button, from View ▸ Zoom and
/// from the `canvas.empty` context menu.
///
/// Word for word `crate::text::commands::view_zoom_fit_page`'s tooltip, which
/// has never named a chord — the two mirrors of one command now say exactly
/// the same thing, which is what the header claims they should.
#[must_use]
pub fn fit_page_tooltip() -> &'static str {
    "Scale the page so all of it is visible, and keep it fitted as the \
     window resizes."
}

// ---------------------------------------------------------------------------
// Page navigation, and the editable page box
// ---------------------------------------------------------------------------

/// The previous-page button's label — `⏴` (U+23F4).
///
/// **Not `◀` (U+25C0)**, which `RIBBON_IA.md` §6 spells the control with and
/// which egui's bundled fonts cannot draw — see [`diagnostics_toggle`] for
/// the measurement and the test that caught it. `⏴`/`⏵` are the same shape
/// at a slightly smaller optical size, and they are what this font set has.
#[must_use]
pub fn prev_page() -> &'static str {
    "⏴"
}

/// Hover text for the previous-page button.
#[must_use]
pub fn prev_page_tooltip() -> &'static str {
    "Previous page (Page Up)."
}

/// The next-page button's label — `⏵` (U+23F5). See [`prev_page`].
#[must_use]
pub fn next_page() -> &'static str {
    "⏵"
}

/// Hover text for the next-page button.
#[must_use]
pub fn next_page_tooltip() -> &'static str {
    "Next page (Page Down)."
}

/// The page number, as the editable box shows it.
///
/// **1-based.** `crate::viewer::ViewState::page_index` is 0-based and the
/// conversion happens here, once, exactly as that module's own docs
/// prescribe: *"The UI displays it 1-based; the conversion happens once, in
/// the string catalog."*
#[must_use]
pub fn page_number(page_1_based: usize) -> String {
    format!("{page_1_based}")
}

/// The total, shown to the right of the editable box.
///
/// `/ 42` rather than `of 42`: `RIBBON_IA.md` §6 spells the control
/// `page ◀ n/N ▶`, and the slash is narrower — which matters on a control
/// that sits between two buttons in a fixed-height bar.
#[must_use]
pub fn page_of_total(total: usize) -> String {
    format!("/ {total}")
}

/// Hover text for the editable page box.
///
/// States the commit rule, because it is the one thing about this control
/// that is not visible: nothing happens per keystroke, so an operator typing
/// `42` must be able to trust that passing through `4` did not move them.
#[must_use]
pub fn page_box_tooltip() -> &'static str {
    "Type a page number and press Enter. Nothing moves while you type, and \
     a number past the end of the document goes to the nearest page and \
     says so."
}

/// Shown beside the box when a committed number was outside the document.
///
/// ★ **The point of this string is that the clamp is not silent.** Typing
/// `99` into a 42-page document and landing on 42 with no explanation is
/// indistinguishable from the box ignoring what was typed — and an operator
/// who cannot tell those apart stops trusting the control. Naming the number
/// that does not exist, and the page they got instead, makes the clamp a
/// *report* rather than a shrug.
///
/// `asked` is the 1-based number typed; `landed` and `total` are 1-based
/// page numbers.
#[must_use]
pub fn page_clamped_note(asked: usize, landed: usize, total: usize) -> String {
    format!("No page {asked} — went to {landed} of {total}")
}

/// Shown beside the box when the committed text was not a page number.
///
/// The operator's text is deliberately **left in the box** when this
/// appears, so the note explains something still visible rather than
/// describing a value that has already been thrown away.
#[must_use]
pub fn page_rejected_note() -> &'static str {
    "Not a page number — type digits, then Enter"
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The disclosure must say which state it is in.
    ///
    /// A toggle whose two states read identically is a toggle whose state
    /// the operator has to discover by clicking it, which is exactly the
    /// affordance a disclosure triangle exists to remove.
    #[test]
    fn the_disclosure_reads_differently_open_and_closed() {
        assert_ne!(diagnostics_toggle(false), diagnostics_toggle(true));
    }

    /// Every counted note in this module, so a new one cannot be added
    /// without inheriting the checks below.
    const COUNTERS: [fn(usize) -> String; 9] = [
        diagnostics_contents_missing,
        diagnostics_fonts_skipped,
        diagnostics_images_skipped,
        diagnostics_glyphs_notdef,
        diagnostics_glyphs_substituted,
        diagnostics_glyphs_supplied,
        diagnostics_layers_hidden,
        diagnostics_ops_deferred,
        diagnostics_ops_unknown,
    ];

    /// ★ **One is singular everywhere it can be.**
    ///
    /// Not pedantry: these lines are read in a small weak font at the edge
    /// of the window, and "1 glyphs" is the kind of thing a reader notices
    /// *instead of* the number, which is the part that matters.
    ///
    /// The property is asserted structurally rather than against a table of
    /// expected sentences: **the singular must not be the plural with the
    /// digit swapped.** That catches a missing branch on every entry,
    /// including the ones whose noun is not the first word ("text from 1
    /// font not drawn"), and it keeps working when the copy is edited.
    #[test]
    fn every_counted_note_is_singular_at_one() {
        for f in COUNTERS {
            let one = f(1);
            let two = f(2);
            assert!(one.contains('1'), "a count must show its number: {one}");
            assert!(two.contains('2'), "a count must show its number: {two}");
            assert_ne!(
                one,
                two.replacen('2', "1", 1),
                "the singular form is missing — this note reads as a plural at \
                 a count of one: {one}"
            );
        }
    }

    /// The join is what makes several notes one line.
    #[test]
    fn notes_join_into_one_line() {
        let joined = diagnostics_join(&[
            diagnostics_glyphs_substituted(3),
            diagnostics_images_skipped(1),
        ]);
        assert!(joined.contains('·'));
        assert!(
            !joined.contains('\n'),
            "the bar has exactly one row: {joined}"
        );
    }

    /// **★ The clamp note names both numbers.**
    ///
    /// The whole value of the note is that it distinguishes "your number was
    /// out of range" from "the box ignored you". A note that named only the
    /// page landed on could not do that.
    #[test]
    fn the_clamp_note_names_what_was_asked_and_what_was_given() {
        let note = page_clamped_note(99, 42, 42);
        assert!(note.contains("99"), "{note}");
        assert!(note.contains("42"), "{note}");
    }

    /// The two failure notes must not read alike.
    ///
    /// "I typed a page that does not exist" and "I typed something that is
    /// not a page number" are different mistakes with different fixes, and
    /// the operator gets one line to tell them apart.
    #[test]
    fn the_two_page_box_notes_read_differently() {
        assert_ne!(page_clamped_note(99, 42, 42), page_rejected_note());
    }

    /// The page number shown is 1-based, and the total reads as a total.
    #[test]
    fn the_page_number_is_one_based_and_the_total_is_labelled() {
        assert_eq!(page_number(1), "1");
        assert_eq!(page_number(42), "42");
        assert!(page_of_total(42).contains("42"));
        assert!(
            page_of_total(42).starts_with('/'),
            "the total must read as a denominator, not as a second page number"
        );
    }

    /// A zoom readout is a percentage.
    #[test]
    fn the_zoom_readout_carries_its_unit() {
        assert_eq!(zoom_percent(100), "100%");
        assert_eq!(zoom_percent(8), "8%");
    }

    /// **The three fit labels are distinct, and so are their tooltips.**
    ///
    /// Three controls in a row that read alike is the failure the ribbon's
    /// own salvage notes record (two adjacent Content buttons both reading
    /// `Aa`, distinguished only by their tooltips). Here both halves are
    /// asserted.
    #[test]
    fn the_three_fit_controls_are_distinguishable() {
        let labels = [fit_actual_size(), fit_width(), fit_page()];
        let tooltips = [
            fit_actual_size_tooltip(),
            fit_width_tooltip(),
            fit_page_tooltip(),
        ];
        for i in 0..3 {
            for j in (i + 1)..3 {
                assert_ne!(labels[i], labels[j]);
                assert_ne!(tooltips[i], tooltips[j]);
            }
        }
    }

    /// **★ Each fit tooltip names exactly the chord that reaches it.**
    ///
    /// This test used to pin the *opposite* facts — Actual size naming no
    /// chord, Fit page naming `Ctrl+0`, Fit width naming `Ctrl+2` — because
    /// `Ctrl+0` and `Ctrl+2` each had two owners and this file was the
    /// surface that had to keep quiet about it. With one owner per chord
    /// (`crate::app::keyboard`'s ★ section) the three claims invert, and the
    /// test inverts with them rather than being deleted: the property worth
    /// defending was never "Actual size is silent", it was **"a status-bar
    /// tooltip names a chord if and only if that chord reaches the control"**.
    ///
    /// The three assertions below are the direct expression of that, and the
    /// last two are the ones that matter most — a chord *removed* from the
    /// keyboard leaves no compile error behind, so the only thing standing
    /// between the operator and a tooltip that lies is an assertion that the
    /// string stayed silent.
    #[test]
    fn each_fit_tooltip_names_exactly_the_chord_that_reaches_it() {
        assert!(
            fit_actual_size_tooltip().contains("Ctrl+0"),
            "the manifest binds Ctrl+0 to view.zoom_actual and the keyboard enacts it: {}",
            fit_actual_size_tooltip()
        );
        assert!(
            !fit_page_tooltip().contains("Ctrl"),
            "no chord reaches Fit page in this build: {}",
            fit_page_tooltip()
        );
        assert!(
            !fit_width_tooltip().contains("Ctrl"),
            "Ctrl+2 belongs to mode.review, not to Fit width: {}",
            fit_width_tooltip()
        );
    }

    /// **★ The mirrors say exactly what the ribbon says.**
    ///
    /// The header's claim, asserted rather than trusted. Three status-bar
    /// controls mirror three View ▸ Zoom commands under amendment P1a, and a
    /// mirror that paraphrases is how a product acquires two mental models of
    /// one verb. It is also what let the chord defect live on one surface and
    /// not the other for as long as it did.
    #[test]
    fn the_fit_mirrors_repeat_the_ribbon_word_for_word() {
        use crate::text::commands as c;
        assert_eq!(fit_actual_size(), c::view_zoom_actual().label);
        assert_eq!(fit_actual_size_tooltip(), c::view_zoom_actual().tooltip);
        assert_eq!(fit_page(), c::view_zoom_fit_page().label);
        assert_eq!(fit_page_tooltip(), c::view_zoom_fit_page().tooltip);
        assert_eq!(fit_width(), c::view_zoom_fit_width().label);
        assert_eq!(fit_width_tooltip(), c::view_zoom_fit_width().tooltip);
    }

    /// The page-box tooltip must state the commit rule.
    ///
    /// It is the only place the operator can learn that typing does not
    /// navigate, and that property is the reason the control is usable at
    /// all — see `crate::app::status`.
    #[test]
    fn the_page_box_tooltip_states_the_commit_rule() {
        let t = page_box_tooltip();
        assert!(t.contains("Enter"), "{t}");
        assert!(t.contains("while you type"), "{t}");
    }
}
