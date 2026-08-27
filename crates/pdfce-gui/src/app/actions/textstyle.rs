//! # `app::actions::textstyle` — changing how EXISTING text looks
//!
//! ## The operator's ask, and the table that answered it wrong
//!
//! > *"We should also have all the font tools available that Word does."*
//! > — 2026-08-25, `OPERATOR_REQUESTS.md` O37
//!
//! O37's inventory read the engine's text verbs — `add_text`, `edit_text`,
//! `delete_text_run` — and concluded, in a table with fourteen rows and a
//! column of crosses, that **pdfce could choose how text looked when it was
//! created and could not change how existing text looked at all.**
//!
//! Every cross in that column was wrong. `EditSession::format_text` shipped as
//! `Pass 14.2`, was extended through `Pass 19.x`, and became retargetable into
//! form XObjects as `Pass 119.2` — five days *before* the request that said it
//! did not exist. It had reached this project only as a paragraph inside a note
//! about something else, which is the engine's own recorded defect (`R220`):
//! *"a verb whose only description is something somebody said once, in
//! passing, while writing about something else."*
//!
//! ★ Worth keeping, because half the lesson is ours: **an absence claim about a
//! crate you do not build is a claim about every route, and one verb is not
//! every route.** This module exists because somebody eventually asked instead
//! of inferring.
//!
//! ## What can actually be changed, measured rather than described
//!
//! | control | verb | limit |
//! |---|---|---|
//! | **size** | `set_size` | none — the `Tf` operand changes and the line is relaid out |
//! | **colour** | `set_fill` | none. pdfce stores the SPACE the operator chose (`rg`/`g`/`k`) instead of force-converting to DeviceRGB the way Acrobat does |
//! | **face** | `set_font` | the target must **already be a font resource on the page**; refused by name otherwise (`FF-C`) |
//! | **bold / italic** | `set_synthetic`, *or* `set_font` — see below | one named refusal, on italic only |
//!
//! ## ★★★ Why Bold is never greyed, and why one press can take either verb
//!
//! `set_font` selects a real face and refuses when the page carries none.
//! `gate_synthesis` is its **exact complement**: it refuses synthesis when a
//! real face *is* available, and its own first branch reads *"No font resources
//! to search: nothing better exists, so the fallback is genuinely the only
//! option. Proceed."*
//!
//! ⇒ Between the two verbs **every page is covered**, and there is no page on
//! which bold is unreachable. The engine's instruction was explicit: *"Do not
//! grey out a bold button. Offer it, and surface the disclosure when synthesis
//! fires."*
//!
//! So [`apply`] asks for synthesis first and, when the engine refuses **because
//! a real face is available**, retries with that face — which the refusal names
//! (`RealFaceAvailable { real_font, .. }`). The operator presses one button and
//! gets the best weight the page can give: a genuine typeface where one exists,
//! a disclosed synthetic one where it does not.
//!
//! ★ That retry is the one place this module reacts to an error variant rather
//! than reporting it, and it is not cleverness papering over a refusal: the
//! refusal's own prose *names the remedy and asks for it to be applied*. Not
//! taking it would mean showing the operator a sentence telling them to do
//! something the program could have done.
//!
//! A synthetic weight is the regular face thickened by stroking; a synthetic
//! slant is the upright face sheared. `R90` means neither is ever a preference
//! — only an explicit, per-use request — and the report says which fired, in
//! words passed straight through rather than re-written.
//!
//! ## ★★★ Why the runs are edited in DESCENDING order
//!
//! The load-bearing decision in the file, and invisible until it is wrong.
//!
//! `format_text` rewrites one **show operator**. A sweep can cover several, so
//! a restyle is several calls. Each call rewrites the content stream, so every
//! pin taken before it is stale afterwards — which is why
//! [`crate::canvas::textedit::pin::resolve`] is re-run between steps instead of
//! the pins being batched up front.
//!
//! Re-resolving fixes the *spans*. It does not fix the **indices**: synthetic
//! italic brackets its run with two absolute `Tm` operators, and a `Tm` can
//! split a run, so an edit at index *k* may renumber everything after it.
//!
//! Descending order makes that harmless by construction. Editing run *k* can
//! only insert operators at or after *k*'s position in the buffer, so runs
//! `0..k` keep both their bytes and their ordinals. Working downwards, every run
//! still to be done is always *before* the one just done, and its index is still
//! the index it was measured at.
//!
//! Ascending order would work for four of the five controls and fail for italic,
//! on multi-run selections only, by restyling the wrong text — the shape of
//! defect that ships because the case that breaks it is the one nobody tries.
//!
//! ## What is NOT here, and is filed rather than hidden
//!
//! **One gesture is N undo entries** when the selection covers N runs.
//! `EditSession` has no grouping verb; the engine solves multi-verb undo by
//! adding a *combined* verb per case, which is how `Pass 81.1` gave markup
//! authoring an opacity in one entry rather than two. A restyle across a
//! paragraph therefore takes several `Ctrl+Z` presses to take back.
//!
//! Disclosed by the count rather than left to be discovered, and filed on the
//! request channel. **Not** worked around here: a shell-side coalesce would
//! work and would leave every other consumer with the same defect, which is
//! decision 058's whole argument and is quoted in the engine's own docs about
//! the last time it happened.

use pdfce_core::text_edit::{FormatError, FormatOptions, FormatRequest, NewFill, StyleSynthesis};

use crate::app::state::OpenDoc;
use crate::app::status::decline;
use crate::text::status as t;

/// One property of a text run, and the value the operator chose for it.
///
/// One variant per control, because **one control press is one undo entry**. A
/// struct carrying five `Option`s would let the panel batch a size and a colour
/// into a single request — which the engine supports — and would make `Ctrl+Z`
/// after two separate presses take back a state the operator never saw. The
/// panel commits on `drag_stopped` / `lost_focus` for the same reason.
#[derive(Debug, Clone, PartialEq)]
pub enum StyleChange {
    /// A new size in points, changing the `Tf` operand.
    Size(f64),
    /// A new fill colour, stored in the space the operator chose.
    Fill(NewFill),
    /// A new face, named by `/Resources /Font` key or by `/BaseFont`.
    ///
    /// A `String` rather than a `FontSelector` because [`super::action::Action`]
    /// derives `PartialEq` and `FontSelector` is `#[non_exhaustive]`. The
    /// conversion is one line at the point of use.
    Face(String),
    /// Weight and slant, as two independent flags.
    ///
    /// ★ Deliberately **not** named `Synthetic`, because whether it ends up
    /// synthetic is the engine's decision and not the operator's: see the
    /// module header on the two-verb retry. The operator asked for bold; how
    /// bold is achieved on this page is a fact they are told afterwards.
    Weight {
        /// Bold wanted.
        bold: bool,
        /// Italic wanted.
        italic: bool,
    },
}

impl StyleChange {
    /// Stamp this change onto a request that is already pinned.
    ///
    /// Its own function so the mapping from *a control the operator pressed* to
    /// *a field on `FormatRequest`* is one readable table, and so a sixth
    /// control cannot be added without appearing in it.
    fn stamp(&self, req: FormatRequest) -> FormatRequest {
        match self {
            Self::Size(points) => req.size(*points),
            Self::Fill(fill) => req.fill(fill.clone()),
            Self::Face(selector) => req.font(pdfce_core::text_edit::FontSelector::new(selector)),
            Self::Weight { bold, italic } => req.synthetic(StyleSynthesis::new(*bold, *italic)),
        }
    }

    /// The trace word for this change, for `PDFCE_DIAG`.
    const fn label(&self) -> &'static str {
        match self {
            Self::Size(_) => "size",
            Self::Fill(_) => "fill",
            Self::Face(_) => "face",
            Self::Weight { .. } => "weight",
        }
    }
}

/// A pinned request for one run, ready to hand to the engine.
///
/// Built fresh per run per step; see the module header on why a batch of these
/// taken up front would be wrong.
fn request(page: usize, pinned: crate::canvas::textedit::pin::Pinned, find: &str) -> FormatRequest {
    // ★★★ `find` is the RUN'S OWN TEXT, and it is required.
    //
    // The obvious shape — pin the operator and leave `find` empty, because the
    // pin already says which operator — does not work, and finding that out is
    // what the first driven test of this module was for. `match_run` refuses an
    // empty `find` by name (*"empty find text"*), because the two locators
    // answer different questions: the **pin** names the show operator, and
    // **`find`** names a contiguous sub-range *within* it. A restyle of the
    // whole run is therefore a `find` of the whole run's text, not of nothing.
    //
    // ★ That is also the door to restyling part of a run later, without any
    // engine work: a shorter `find` restyles a shorter span. This shell does
    // not offer it yet because a text sweep's byte offsets would have to be
    // trusted across an extraction, and `TextSelection::runs` deliberately
    // drops them.
    let mut req = FormatRequest::new(page, find);
    req.pinned_span = Some(pinned.span);
    req.target(pinned.target)
}

/// Restyle every run the selection covers.
///
/// # The ordering is done here, not asked of the caller
///
/// `runs` arrives in whatever order the caller measured it; this sorts,
/// deduplicates and reverses. A caller that had to remember to pass them
/// backwards is a caller that will one day forget, and the failure would be
/// silent and rare — see the module header.
///
/// # What a refusal does
///
/// **Stops.** A restyle that half-applies and carries on is worse than one that
/// half-applies and says so: the operator sees some of their text change, has no
/// way to tell how much, and the undo stack holds an unknown number of entries.
pub(super) fn apply(doc: &mut OpenDoc, page: usize, runs: &[usize], change: &StyleChange) {
    let mut ordered: Vec<usize> = runs.to_vec();
    ordered.sort_unstable();
    ordered.dedup();
    ordered.reverse();

    let total = ordered.len();
    if total == 0 {
        decline::record_text_style(t::TextStyleRefusal::NoRun);
        return;
    }

    // Accumulated across the whole gesture and surfaced on the LAST successful
    // step, because `super::apply::vector_edit` records the disclosure slot per
    // call and the last write wins. Collecting and emitting once is what stops
    // a three-run restyle showing only the third run's sentence.
    let mut carried: Vec<String> = Vec::new();
    let mut applied = 0_usize;

    for (position, run) in ordered.into_iter().enumerate() {
        let last = position + 1 == total;
        // ★ A FRESH pin per step: the previous iteration rewrote the content
        // stream, so any span measured before it names bytes that have moved.
        let Some(read) = crate::canvas::textedit::pin::inspect(doc, page, run) else {
            stop(doc, applied, t::TextStyleRefusal::Unpinnable);
            return;
        };
        let (pinned, find) = (read.pin, read.style.text);

        let mut outcome: Option<FormatError> = None;
        let mut notes: Vec<String> = Vec::new();
        super::apply::vector_edit(doc, "text-style", page, 1, |session| {
            match session.format_text(
                &change.stamp(request(page, pinned, &find)),
                &FormatOptions::default(),
            ) {
                Ok(report) => {
                    notes.extend(report.disclosures);
                    Ok(notes.clone())
                }
                // ★★ The two-verb retry. The engine refused synthesis *because
                // a real face is available* and named it; taking that offer is
                // what makes one Bold button work on every page. See the module
                // header — the alternative is showing the operator a sentence
                // telling them to do a thing the program could have done.
                Err(FormatError::RealFaceAvailable {
                    real_font, style, ..
                }) => {
                    let retry = request(page, pinned, &find)
                        .font(pdfce_core::text_edit::FontSelector::new(&real_font));
                    match session.format_text(&retry, &FormatOptions::default()) {
                        Ok(report) => {
                            notes.push(t::text_style_used_real_face(style, &real_font));
                            notes.extend(report.disclosures);
                            Ok(notes.clone())
                        }
                        // ★★★ The RETRY's refusal is the one reported, not the
                        // synthesis refusal that sent us here — and that is the
                        // opposite of what this code did when it was written.
                        //
                        // A test on `textedit/format_family.pdf` proved it. The
                        // page carries a `/Times-Roman` run and two bold
                        // resources: `/F2` `Calibri-Bold`, which covers the run,
                        // and `/F3` `Times-Bold`, whose `/Differences` remaps
                        // `o` to `/bullet` and therefore does NOT. `gate_synthesis`
                        // refuses synthesis and names `Times-Bold` — matching the
                        // run's family, which is the sensible-looking choice —
                        // and `set_font Times-Bold` is then refused for
                        // coverage. **Bold is unreachable on that page**, which
                        // contradicts the engine's own "between the two verbs
                        // every page is covered", and `--set-font F2` succeeds
                        // on the command line the whole time.
                        //
                        // Reported to the engine rather than worked around: a
                        // shell-side search for a *different* bold resource
                        // would be this project second-guessing pdfce's font
                        // selection, and decision 058 says a workaround the GUI
                        // keeps quiet is a boundary defect that stays.
                        //
                        // What the shell owes meanwhile is the ACTIONABLE half.
                        // "There is a real bold face, use it" is useless when
                        // using it is what just failed; "that face has no shape
                        // for one of these characters" is a fact the operator
                        // can act on.
                        Err(error) => {
                            decline::record_text_style(refusal_of(&error));
                            outcome = Some(error);
                            Err(FormatError::NoOp)
                        }
                    }
                }
                Err(error) => {
                    decline::record_text_style(refusal_of(&error));
                    // Handed back unchanged so the trace keeps the engine's own
                    // `Display` prose — the decline is a sentence for an
                    // operator, the trace is the record for whoever is
                    // debugging, and `check-ui-strings.sh` exclusion 3 says
                    // they must not become each other.
                    //
                    // ★ `outcome` is SET and not taken. It was `outcome.take()`
                    // for one commit, which handed the error onward correctly
                    // and left the flag `None` — so the caller below read the
                    // refusal as a success and counted it applied. Found by the
                    // first test that read the DOCUMENT back instead of the
                    // function's own report of itself.
                    outcome = Some(error);
                    Err(FormatError::NoOp)
                }
            }
        });

        if outcome.is_some() && notes.is_empty() {
            // Refused. The decline is already recorded; say how far it got.
            if applied > 0 {
                decline::record_text_style(t::TextStyleRefusal::PartOnly);
            }
            return;
        }
        applied += 1;
        carried.extend(notes);

        if last && (total > 1 || !carried.is_empty()) {
            emit_carried(doc, page, applied, total, &carried);
        }
    }

    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "text-style page={page} change={} applied={applied} of={total}",
            change.label()
        )
    });
}

/// Re-record the whole gesture's disclosures, on the last step.
///
/// A second `vector_edit` would be a second undo entry, so this writes the
/// disclosure slot directly. That is the one place in this module that reaches
/// past the funnel, and it is sound because it changes **no document**: the
/// epoch it stamps is the one the final edit already bumped.
fn emit_carried(doc: &OpenDoc, page: usize, applied: usize, total: usize, carried: &[String]) {
    let mut notes: Vec<String> = carried.to_vec();
    if total > 1 {
        notes.push(t::text_style_multi(applied));
    }
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("text-style-disclosed page={page} n={}", notes.len())
    });
    super::disclosure::record_edit_disclosure(Some(super::disclosure::EditDisclosure {
        epoch: doc.edit_epoch,
        notes,
    }));
}

/// Record that the gesture stopped, and how far it got.
fn stop(doc: &mut OpenDoc, applied: usize, why: t::TextStyleRefusal) {
    let _ = doc;
    decline::record_text_style(why);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("text-style-declined applied={applied}")
    });
}

/// Which operator-facing sentence a refusal earns.
///
/// ★ The engine's own `Display` prose is deliberately **not** the sentence.
/// `check-ui-strings.sh` exclusion 3 says in as many words that an error type's
/// prose is not permission to route UI text through it; the prose goes to the
/// trace, where whoever is debugging wants it, and the catalog says the same
/// thing in the operator's terms with the remedy first.
///
/// Three named cases and a catch-all, chosen because they are the three an
/// operator can *do something about*. Everything else — encryption, a no-op
/// request, a page index — is either impossible from this surface or is not
/// improved by being subdivided.
fn refusal_of(error: &FormatError) -> t::TextStyleRefusal {
    match error {
        FormatError::TargetFontMissing(_) => t::TextStyleRefusal::FaceNotOnPage,
        FormatError::ShearUnsupported(_) => t::TextStyleRefusal::ItalicWouldMove,
        FormatError::CoverageFailure(_) => t::TextStyleRefusal::FaceLacksCharacters,
        _ => t::TextStyleRefusal::Other,
    }
}

#[cfg(test)]
mod tests;
