//! # `text::print` — every word the print dialog shows
//!
//! The catalog area for [`crate::dialogs::print`]. One module per surface is
//! the rule this directory's `mod.rs` states; the print dialog is a surface,
//! and it is a large one — three tabs, a preview, a device selector and a
//! commit button whose label is itself a disclosure.
//!
//! ## The copy in here is doing three different jobs
//!
//! Distinguishing them is what keeps the voice consistent, so they are named:
//!
//! 1. **Names.** A radio's label, a heading, a tab. Sentence case, no
//!    trailing period, and they name the *thing*, not the act — "Actual
//!    size", not "Print at actual size".
//! 2. **Disclosures.** Sentences pdfce owes the operator because pdfce
//!    inferred something, capped something, or is about to lose something:
//!    [`clip_summary`], [`dpi_capped`], [`raster_note`],
//!    [`commit_with_clipping`]. These are full sentences with punctuation,
//!    they name the number, and they never apologise. `docs/core-api/03`
//!    §6.3 enumerates exactly which values are inferences; every one of them
//!    has a function here.
//! 3. **Refusals.** Why a control is absent or a job cannot go
//!    ([`spooler_unavailable`], [`no_printers`], [`no_pages_selected`],
//!    [`range_unparsable`]). These say what is true and what the operator
//!    can do, and they are deliberately *different sentences* for different
//!    causes — see the next section, which is the single most important
//!    convention in this file.
//!
//! ## ★ Three ways to have no printer, said three ways
//!
//! This mirrors [`crate::text`]'s own three-way open-failure distinction, and
//! for the same reason: an operator must be able to tell from the words alone
//! which of these is true, because the three have completely different
//! remedies.
//!
//! | function | what is actually true | what the operator does |
//! |---|---|---|
//! | [`spooler_unavailable`] | pdfce could not ask this system about printers **at all** | nothing, in this build |
//! | [`no_printers`] | the spooler answered, and reported none installed | install a printer |
//! | [`device_unavailable`] | this *particular* printer's driver would not describe itself | pick another printer |
//!
//! `pdfce-print` is explicit that collapsing the first two is a defect:
//! non-Windows `list_printers` returns `Err(Unsupported)` rather than an
//! empty `Vec`, because *"reporting the same value for 'this platform cannot
//! enumerate printers at all' would collapse two different facts into one and
//! send a caller looking for hardware"* (`lib.rs:1859-1866`). The error type
//! carries that distinction across the port in
//! [`crate::dialogs::print::spooler`]; these three sentences are what it is
//! carried *for*.
//!
//! ## Why the commit button's label is a format string
//!
//! [`commit_with_clipping`] exists because the print dialog **is** the
//! confirmation — there is no second gate — so the uncertainty has to be
//! stated *in the disclosure itself* rather than implied by a confirm step
//! existing. That is rule 4 applied to a button. A separate warning label
//! beside the button would be the version an operator can look past.

/// The dialog's window title.
#[must_use]
pub const fn dialog_title() -> &'static str {
    "Print"
}

// ---------------------------------------------------------------------------
// The device, and the three ways there is not one
// ---------------------------------------------------------------------------

/// Label for the printer selector.
#[must_use]
pub const fn printer_label() -> &'static str {
    "Printer"
}

/// **pdfce could not ask this system about its printers at all.**
///
/// The first of the three no-printer sentences (module docs). It is what this
/// build says today, because `pdfce-print` is not linked into this crate —
/// see [`crate::dialogs::print::spooler`]'s header for the one-line manifest
/// change that makes it unreachable.
///
/// Deliberately **not** "no printers were found": that would be a claim about
/// the operator's hardware made on evidence pdfce does not have. It also
/// covers the honest non-Windows case, where `pdfce-print`'s every entry
/// point returns `Unsupported`.
///
/// Says plainly that the capability is absent rather than showing controls
/// the shell would then ignore — the same choice, for the same reason, as
/// [`crate::text::open_needs_password`].
#[must_use]
pub const fn spooler_unavailable() -> &'static str {
    "This build cannot reach a print device, so there is nothing to print to. \
     Nothing has been sent and nothing will be."
}

/// **The spooler answered, and this system has no printers installed.**
///
/// The second sentence. A true statement about the machine, which is why it
/// may not be used for the case above.
#[must_use]
pub const fn no_printers() -> &'static str {
    "This system reports no printers. Install one, then reopen this dialog."
}

/// **This particular printer's driver would not describe itself.**
///
/// The third. Everything the preview draws — the sheet, the printable
/// rectangle, the unprintable margins — comes from the device's own reported
/// geometry, so without it there is no honest picture to draw. Saying so is
/// better than drawing a plausible sheet: a guessed rectangle is exactly the
/// "confidently wrong" preview the whole feature exists to prevent.
#[must_use]
pub const fn device_unavailable() -> &'static str {
    "This printer's driver did not report its paper size, so pdfce cannot show \
     what the sheet will look like. Choose another printer."
}

/// The dialog was asked to draw with no document open.
///
/// Reachable only if a document is closed while the dialog is up. The dialog
/// closes itself in that case; this is the sentence for the spool path, which
/// must refuse rather than assume.
#[must_use]
pub const fn no_document() -> &'static str {
    "No document is open, so there is nothing to print."
}

// ---------------------------------------------------------------------------
// The tab strip
// ---------------------------------------------------------------------------

/// Tab 1's label.
#[must_use]
pub const fn tab_pages_layout() -> &'static str {
    "Pages & Layout"
}

/// Tab 1's hover text — the question the tab answers.
#[must_use]
pub const fn tab_pages_layout_tooltip() -> &'static str {
    "Which pages print, and how each one lands on the sheet."
}

/// Tab 2's label.
#[must_use]
pub const fn tab_copies_finishing() -> &'static str {
    "Copies & Finishing"
}

/// Tab 2's hover text.
#[must_use]
pub const fn tab_copies_finishing_tooltip() -> &'static str {
    "How many sheets come out, in what order, and on how many sides."
}

/// Tab 3's label.
#[must_use]
pub const fn tab_comments_resolution() -> &'static str {
    "Comments & Resolution"
}

/// Tab 3's hover text.
#[must_use]
pub const fn tab_comments_resolution_tooltip() -> &'static str {
    "What is painted onto each page, and how finely."
}

// ---------------------------------------------------------------------------
// Tab 1 — Pages & Layout
// ---------------------------------------------------------------------------

/// Heading over the page-range radios.
#[must_use]
pub const fn pages_heading() -> &'static str {
    "Pages"
}

/// "All N pages" — the count is in the label so the operator can see what
/// "all" costs before choosing it.
#[must_use]
pub fn range_all(pages: usize) -> String {
    if pages == 1 {
        "All 1 page".to_owned()
    } else {
        format!("All {pages} pages")
    }
}

/// The page currently on the canvas.
#[must_use]
pub const fn range_current() -> &'static str {
    "Current page"
}

/// The typed-range radio.
#[must_use]
pub const fn range_custom() -> &'static str {
    "Pages"
}

/// Hover text for the range box.
///
/// **States the syntax by example**, because the syntax is shared verbatim
/// with `pdfce-cli` — see [`crate::dialogs::print::tabs::parse_page_range`]
/// for why there is exactly one parser — and an operator who learns it here
/// can use it there.
#[must_use]
pub const fn range_hint() -> &'static str {
    "Page numbers or ranges, for example 3 or 1-4 or 5,1-2. \
     Numbers are the ones printed on the page, starting at 1."
}

/// The typed range names no page in this document.
///
/// A refusal, not a correction. The parser yields *nothing* rather than a
/// guess for malformed input, precisely so this sentence can be shown and the
/// commit button can go absent — instead of printing a range nobody asked
/// for.
#[must_use]
pub const fn range_unparsable() -> &'static str {
    "That range does not name any page in this document."
}

/// Label in front of the odd/even radios.
#[must_use]
pub const fn subset_label() -> &'static str {
    "Subset"
}

/// No odd/even filtering.
#[must_use]
pub const fn subset_all() -> &'static str {
    "Every page"
}

/// Odd document pages only.
#[must_use]
pub const fn subset_odd() -> &'static str {
    "Odd only"
}

/// Even document pages only.
#[must_use]
pub const fn subset_even() -> &'static str {
    "Even only"
}

/// Hover text over the subset row.
///
/// **Says which numbering is meant**, because the answer is not obvious and
/// getting it wrong prints the wrong half of the document. `pdfce-print`
/// (`lib.rs:1217-1224`): *"an operator printing '2-9, odd' means document
/// pages 3, 5, 7, 9 — the numbers printed on the paper."*
#[must_use]
pub const fn subset_tooltip() -> &'static str {
    "Odd and even mean the page numbers printed on the paper, not positions \
     within the range above. The subset narrows the range; the two combine."
}

/// Heading over the sizing radios.
#[must_use]
pub const fn sizing_heading() -> &'static str {
    "Sizing"
}

/// Scale up or down to fill the printable area.
#[must_use]
pub const fn scale_fit() -> &'static str {
    "Fit to the printable area"
}

/// One PDF point to one point of paper.
#[must_use]
pub const fn scale_actual() -> &'static str {
    "Actual size"
}

/// Reduce an oversized page; never enlarge a small one.
#[must_use]
pub const fn scale_shrink() -> &'static str {
    "Shrink oversized pages only"
}

/// An explicit percentage.
#[must_use]
pub const fn scale_custom() -> &'static str {
    "Custom scale"
}

/// Hover text over the sizing group.
///
/// **Names the difference between Fit and Shrink**, which is the one thing
/// about this group an operator can get wrong without noticing. `pdfce-print`
/// keeps them as separate modes because collapsing them *"silently blows a
/// business card up to A4"* (`lib.rs:490-494`), and a UI that does not say so
/// re-creates the confusion the engine avoided.
#[must_use]
pub const fn sizing_tooltip() -> &'static str {
    "Fit scales in both directions, so a small page is enlarged to fill the \
     sheet. Shrink oversized pages only ever reduces."
}

/// Suffix on the custom-scale spinner.
#[must_use]
pub const fn percent_suffix() -> &'static str {
    " %"
}

/// Heading over the orientation radios.
#[must_use]
pub const fn orientation_heading() -> &'static str {
    "Orientation"
}

/// Decide per page from its own shape.
#[must_use]
pub const fn orientation_auto() -> &'static str {
    "Auto, from each page's shape"
}

/// Force portrait.
#[must_use]
pub const fn orientation_portrait() -> &'static str {
    "Portrait"
}

/// Force landscape.
#[must_use]
pub const fn orientation_landscape() -> &'static str {
    "Landscape"
}

// ---------------------------------------------------------------------------
// Tab 2 — Copies & Finishing
// ---------------------------------------------------------------------------

/// Label in front of the copy-count spinner.
#[must_use]
pub const fn copies_label() -> &'static str {
    "Copies"
}

/// The collation checkbox, phrased as the *un*-collated option.
///
/// Phrased this way round because collated is the default and the checkbox
/// therefore describes the change, not the state. "Collate" as a checked-by-
/// default box reads as a feature being switched off, which is the more
/// confusing of the two framings.
#[must_use]
pub const fn uncollated() -> &'static str {
    "Group each page's copies together, rather than repeating the whole set"
}

/// Print the sequence back to front.
#[must_use]
pub const fn reverse() -> &'static str {
    "Print back to front"
}

/// Hover text for reverse — names the reason it exists.
#[must_use]
pub const fn reverse_tooltip() -> &'static str {
    "For a printer that stacks face-up, so the finished pile is in order."
}

/// Heading over the duplex radios.
///
/// The whole group is **absent** on a device whose driver does not report
/// duplex support, rather than greyed — see the tab body for why, and
/// `docs/core-api/03` §6.3 item 4 for the engine's side of it. There is
/// deliberately no "your printer cannot do this" sentence here: no setting in
/// this dialog would ever make it possible, so there is nothing to explain
/// and nothing to hope for.
#[must_use]
pub const fn duplex_heading() -> &'static str {
    "Two-sided"
}

/// One side only.
#[must_use]
pub const fn duplex_off() -> &'static str {
    "One-sided"
}

/// Two-sided, flipped on the long edge — the usual book binding.
#[must_use]
pub const fn duplex_long() -> &'static str {
    "Two-sided, long-edge binding"
}

/// Two-sided, flipped on the short edge — notepad binding.
#[must_use]
pub const fn duplex_short() -> &'static str {
    "Two-sided, short-edge binding"
}

/// The tray checkbox.
#[must_use]
pub const fn pick_tray() -> &'static str {
    "Let the printer choose the tray from each page's size"
}

// ---------------------------------------------------------------------------
// Tab 3 — Comments & Resolution
// ---------------------------------------------------------------------------

/// Heading over the annotation-scope radios.
#[must_use]
pub const fn comments_heading() -> &'static str {
    "Comments and forms"
}

/// Page content, links and form-field widgets — no review markup.
///
/// The **default for printing**, which differs from the renderer's own
/// `DocumentAndMarkups` default. Deliberate on both sides: the canvas should
/// show markup, and a print should not carry review comments unless asked.
#[must_use]
pub const fn scope_document() -> &'static str {
    "Document"
}

/// Everything above, plus review markup.
#[must_use]
pub const fn scope_markups() -> &'static str {
    "Document and markups"
}

/// Everything above, restricted to stamps.
#[must_use]
pub const fn scope_stamps() -> &'static str {
    "Document and stamps"
}

/// Form-field widgets only, over blank page content.
#[must_use]
pub const fn scope_fields_only() -> &'static str {
    "Form fields only"
}

/// Heading over the resolution disclosure.
#[must_use]
pub const fn resolution_heading() -> &'static str {
    "Resolution"
}

/// The standing note that pdfce prints rasters, not vectors.
///
/// **Always true, so a caption rather than a warning.** A banner that fires
/// on every job trains an operator to stop reading banners — which is how the
/// *conditional* disclosure beneath it ([`dpi_capped`]) would come to be
/// ignored too.
#[must_use]
pub const fn raster_note() -> &'static str {
    "pdfce renders each page to an image at the resolution below and sends \
     that image. Text and lines are not sent as vectors."
}

/// pdfce chose a resolution the operator did not.
///
/// The conditional half of the resolution disclosure, and it exists because
/// `JobResolution::capped` is pdfce's own memory judgement rather than
/// anything the device or the document asked for (`docs/core-api/03` §6.3
/// item 3). It names all three numbers — what will be used, what the device
/// could do, and what lifting the cap would cost — because an operator
/// deciding whether to raise it needs the cost, not just the fact.
#[must_use]
pub fn dpi_capped(dpi: u32, device_dpi: u32, uncapped_page_mb: u64) -> String {
    format!(
        "Printing at {dpi} DPI. This printer can do {device_dpi} DPI, but one page \
         at that resolution costs pdfce about {uncapped_page_mb} MB of memory, so \
         pdfce capped it. Raise the cap if you need the detail."
    )
}

/// Suffix on the DPI spinner.
#[must_use]
pub const fn dpi_suffix() -> &'static str {
    " DPI"
}

// ---------------------------------------------------------------------------
// The preview
// ---------------------------------------------------------------------------

/// "Sheet i of n" — which sheet of the **job** is showing.
///
/// Says *sheet*, not *page*, and the distinction is load-bearing: the stepper
/// walks the job's own sequence, which may be a custom range, odd/even
/// filtered, reversed, or repeated for copies. Calling position 3 "page 3"
/// would name a document page the job might not even contain.
#[must_use]
pub fn preview_position(index: usize, total: usize) -> String {
    format!("Sheet {index} of {total}")
}

/// Step to the previous sheet of the job.
#[must_use]
pub const fn preview_previous() -> &'static str {
    "Previous"
}

/// Step to the next sheet of the job.
#[must_use]
pub const fn preview_next() -> &'static str {
    "Next"
}

/// Put the preview back to fit, centred.
#[must_use]
pub const fn preview_zoom_fit() -> &'static str {
    "Fit"
}

/// Hover text for Fit.
#[must_use]
pub const fn preview_zoom_fit_tooltip() -> &'static str {
    "Show the whole sheet, centred."
}

/// Zoom the preview out one step.
#[must_use]
pub const fn preview_zoom_out() -> &'static str {
    "Zoom out"
}

/// Zoom the preview in one step.
#[must_use]
pub const fn preview_zoom_in() -> &'static str {
    "Zoom in"
}

/// Draw one PDF point as one screen point.
#[must_use]
pub const fn preview_zoom_actual() -> &'static str {
    "100%"
}

/// Hover text for the actual-size button.
#[must_use]
pub const fn preview_zoom_actual_tooltip() -> &'static str {
    "Draw the sheet at its true size on this screen."
}

/// The magnification readout.
///
/// **A percentage of ACTUAL size, never of the fit.** A number expressed
/// against the fit would change whenever the window was dragged, without the
/// operator touching a zoom control — so it would report the window, not the
/// sheet, and would be useless for the one question the preview exists to
/// answer ("will this fine print clear the margin?").
#[must_use]
pub fn preview_zoom_percent(percent: u32) -> String {
    format!("{percent}% of actual size")
}

/// The gesture hint under the preview.
#[must_use]
pub const fn preview_pan_hint() -> &'static str {
    "Drag to pan, Ctrl+wheel to zoom"
}

/// The job selects no pages, so there is nothing to preview.
#[must_use]
pub const fn no_pages_selected() -> &'static str {
    "This job selects no pages, so there is nothing to preview and nothing to print."
}

/// **Content will be lost off the edge of the printable area.**
///
/// Shown for the whole job, always, not only for the sheet on screen — a
/// multi-page job's clip is frequently on a sheet the operator is not looking
/// at, and a count that only appeared when you happened to step onto the
/// offending sheet would be a disclosure you could miss by not scrolling.
///
/// This is the GUI half of the divergence `pdfce-print` was built for:
/// *"Acrobat's documented behaviour here is to clip SILENTLY … pdfce reports
/// it instead"* (`lib.rs:522-528`). That divergence is worth nothing if the
/// shell reduces it to a number an operator can look past, which is why the
/// same fact also reaches [`commit_with_clipping`].
#[must_use]
pub fn clip_summary(clipped: usize, total: usize) -> String {
    if clipped == 1 {
        format!("1 of these {total} sheets will lose content outside the printable area.")
    } else {
        format!("{clipped} of these {total} sheets will lose content outside the printable area.")
    }
}

// ---------------------------------------------------------------------------
// The footer — the one irreversible control in the application
// ---------------------------------------------------------------------------

/// Leave without printing.
///
/// Says **Close**, not Cancel: nothing has started, so there is nothing to
/// cancel, and a Cancel button next to a Print button invites the reading
/// that a job is in flight and this stops it.
#[must_use]
pub const fn close() -> &'static str {
    "Close"
}

/// Send the job. The plain label, when nothing will be clipped.
#[must_use]
pub const fn commit() -> &'static str {
    "Print"
}

/// Send the job, **with the clip count in the button's own label**.
///
/// The dialog is the confirmation and there is no second gate, so the
/// uncertainty is stated in the disclosure rather than implied by a confirm
/// step existing. Putting it *in the label* rather than beside the button is
/// the difference between a warning the operator has to have read and one
/// they can have looked past — it is on the control their hand is already on.
#[must_use]
pub fn commit_with_clipping(clipped: usize) -> String {
    if clipped == 1 {
        "Print — 1 sheet will be clipped".to_owned()
    } else {
        format!("Print — {clipped} sheets will be clipped")
    }
}

/// Confirmation that the job went out.
#[must_use]
pub fn sent(pages: usize) -> String {
    if pages == 1 {
        "Sent 1 page to the printer.".to_owned()
    } else {
        format!("Sent {pages} pages to the printer.")
    }
}

/// The job did not go out, and why.
///
/// `detail` is `pdfce-print`'s own error `Display`, passed through rather than
/// rewritten — for the same reason [`crate::text::canvas_render_failed`] does
/// it: those errors are structured, specific diagnostics, and replacing one
/// with "an error occurred" throws away the only part of the sentence that
/// helps.
///
/// **Says nothing came out.** A failed spool can leave an operator wondering
/// whether half a job reached the tray, and the first line of the answer
/// belongs in the message.
#[must_use]
pub fn failed(detail: &str) -> String {
    format!("Nothing was sent to the printer. {detail}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★ The three no-printer sentences must be genuinely different.
    ///
    /// Not a tautology test — the same argument as
    /// `crate::text::tests::the_three_open_failures_read_differently`. The
    /// value of the distinction is that an operator can tell from the words
    /// alone which of "this build cannot print", "you have no printers" and
    /// "this printer would not answer" is true, because the three have
    /// different remedies. Three functions producing near-identical prose
    /// would satisfy the type system and defeat the design.
    #[test]
    fn the_three_no_printer_sentences_read_differently() {
        let a = spooler_unavailable();
        let b = no_printers();
        let c = device_unavailable();
        assert_ne!(a, b);
        assert_ne!(b, c);
        assert_ne!(a, c);
    }

    /// The commit label carries the count, and the count is visible in it.
    ///
    /// This is the whole disclosure mechanism: if the number ever stopped
    /// appearing in the string, the button would silently become an ordinary
    /// Print button on a job that loses content.
    #[test]
    fn the_commit_label_states_the_clip_count() {
        assert!(commit_with_clipping(7).contains('7'));
        assert!(commit_with_clipping(1).contains('1'));
        // And it must not read like the plain label, or the disclosure is
        // invisible at a glance.
        assert_ne!(commit_with_clipping(1), commit());
    }

    /// Singular and plural are both grammatical.
    ///
    /// Cheap to get wrong ("1 sheets will be clipped"), and prose that reads
    /// as machine output is prose an operator trusts less — which matters
    /// most on exactly the sentences that are trying to warn them.
    #[test]
    fn the_counted_sentences_are_grammatical_at_one() {
        assert!(commit_with_clipping(1).contains("1 sheet will"));
        assert!(clip_summary(1, 4).contains("1 of these 4 sheets"));
        assert!(sent(1).contains("1 page to"));
        assert!(range_all(1).contains("1 page"));
        assert!(!range_all(1).contains("1 pages"));
    }

    /// The capped-resolution disclosure names all three numbers.
    ///
    /// An operator deciding whether to raise the cap needs the cost of doing
    /// so, not merely the fact that a cap exists. Dropping any one of the
    /// three turns a decision aid back into a notification.
    #[test]
    fn the_dpi_disclosure_names_what_it_costs() {
        let message = dpi_capped(300, 1200, 139);
        for number in ["300", "1200", "139"] {
            assert!(message.contains(number), "missing {number} in {message}");
        }
    }
}
