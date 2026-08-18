//! # `text::new_document` — the copy of the sized-New dialog
//!
//! Every operator-facing string `crate::dialogs::new_document` draws. One
//! function per string, per `crate::text`'s contract: the gate
//! `tools/gates/check-ui-strings.sh` fails the build for a literal that
//! reaches a widget from anywhere else.
//!
//! ## ★ What this surface is a size chooser FOR, which decides the wording
//!
//! Not for drafting. Nobody drafts a sheet in pdfce — documents arrive from
//! SolidWorks — and `crate::app::blank`'s header says so plainly while
//! arguing the A4 default. What this is for is the case the header also
//! names: *"A4 is very plausibly not the right size for this operator's next
//! new sheet"*, whose own drawings are **A3 and A1**.
//!
//! So the copy is short and assumes competence. An operator opening this
//! knows what A1 is; what they need is to find it quickly, see it confirmed,
//! and get out. There is no explanation of what a page size is, no advice
//! about which to pick, and no reassurance.
//!
//! ## Millimetres, and why there is no unit toggle
//!
//! The size list is ISO-first because the operator's corpus is, and every
//! entry states its size in **millimetres**. There is no inches toggle, and
//! that is a decision rather than an omission:
//!
//! - The A series is *defined* in millimetres. A1 in inches is 23.39 × 33.11,
//!   which is a number nobody recognises.
//! - The US and ANSI entries are in the same list and would want inches, so a
//!   toggle would be right for four entries of sixteen and wrong for twelve.
//! - This shell already has a units answer elsewhere and it is **not global**:
//!   the measure tools express a length in whatever the *dimension group's*
//!   own `NumberFormat` says, which is a per-document drafting convention, not
//!   an application preference. A unit switch here would be a second,
//!   unrelated units concept for a dialog that is open for four seconds.
//!
//! The custom fields are therefore millimetres, stated in the label rather
//! than in a suffix an operator can miss, and the resulting sheet is echoed in
//! **both** units by [`sheet_summary`] so a Letter-minded reader is not left
//! converting.

/// The dialog's window title.
#[must_use]
pub const fn window_title() -> &'static str {
    "New document"
}

/// The one-line introduction, above the size list.
///
/// It states what the command produces — **one blank page** — because the
/// name "New from template" (`RIBBON_IA.md` §5.1) leads a reader to expect a
/// template gallery, and this dialog offers page sizes. Saying what it does
/// in its first line is the cheapest available correction for a label this
/// project may not change on its own authority.
#[must_use]
pub const fn intro() -> &'static str {
    "One blank page, at the size you choose."
}

/// Heading over the size list.
#[must_use]
pub const fn size_heading() -> &'static str {
    "Page size"
}

/// The entry that opens the two width/height fields.
///
/// Listed **last**, after the sixteen standard sizes, rather than first. A
/// custom size is the rarer case and putting it at the top would make the
/// common case scroll.
#[must_use]
pub const fn size_custom() -> &'static str {
    "Custom…"
}

/// The presentable name of a standard size — `"A1"`, `"Letter"`, `"ANSI D"`.
///
/// # Why this is here and not in the engine
///
/// `pdfce_core::paper::PaperSize::id` is `"a1"`, `"ansi-d"`, `"letter"` —
/// ASCII, lowercase, hyphenated, and explicitly *"what a CLI flag value and a
/// settings file spell"*. It is an identifier, not a label, and the engine is
/// right not to carry operator copy. This crate's `text` module is where a
/// presentable name lives, for this and for everything else.
///
/// # The fallback, and why it is not a compile error
///
/// `PaperSize` is `#[non_exhaustive]` and the engine says the table will grow
/// — ARCH sizes, JIS B and the ISO B/C envelope series are all named as
/// plausible additions. A `match` with no wildcard would fail to compile the
/// day one lands, which sounds like the right failure until you notice what it
/// would be blocking: a size this shell could otherwise offer immediately and
/// correctly, under its identifier.
///
/// So an unrecognised size renders its `id()` **uppercased** and is listed.
/// That is a slightly ugly label for a real size, which beats a missing size
/// or a broken build. `tests` pins that every size in `PaperSize::ALL` today
/// has a proper name, so the fallback cannot quietly become the normal path.
#[must_use]
pub fn size_name(size: pdfce_core::paper::PaperSize) -> String {
    use pdfce_core::paper::PaperSize as P;
    match size {
        P::A0 => "A0".to_owned(),
        P::A1 => "A1".to_owned(),
        P::A2 => "A2".to_owned(),
        P::A3 => "A3".to_owned(),
        P::A4 => "A4".to_owned(),
        P::A5 => "A5".to_owned(),
        P::A6 => "A6".to_owned(),
        P::Letter => "Letter".to_owned(),
        P::Legal => "Legal".to_owned(),
        P::Tabloid => "Tabloid".to_owned(),
        P::Executive => "Executive".to_owned(),
        P::AnsiA => "ANSI A".to_owned(),
        P::AnsiB => "ANSI B".to_owned(),
        P::AnsiC => "ANSI C".to_owned(),
        P::AnsiD => "ANSI D".to_owned(),
        P::AnsiE => "ANSI E".to_owned(),
        // ui-text-exempt: not a written string — the engine's own identifier,
        // uppercased, for a size added to `PaperSize` after this build.
        other => other.id().to_uppercase(),
    }
}

/// One entry in the size list: the standard's own name, then its millimetres.
///
/// `name` comes from [`size_name`]; the millimetres are computed here so the
/// list and [`sheet_summary`] round identically.
#[must_use]
pub fn size_entry(name: &str, size_pt: (f64, f64)) -> String {
    let mm = |pt: f64| (pt * 25.4 / 72.0).round() as i64;
    format!("{name} — {} × {} mm", mm(size_pt.0), mm(size_pt.1))
}

/// Heading over the orientation pair.
#[must_use]
pub const fn orientation_heading() -> &'static str {
    "Orientation"
}

/// Taller than wide.
#[must_use]
pub const fn orientation_portrait() -> &'static str {
    "Portrait"
}

/// Wider than tall.
///
/// **The normal orientation for a drawing sheet** — a CAD sheet called "A1"
/// is A1 landscape in every practical case, which is `pdfce_core::paper`'s own
/// observation. It is not made the default here: `file.new`'s A4 portrait is
/// the shipped default and this dialog opens on it, so the operator's first
/// sight of the window matches the command beside it rather than second-
/// guessing them.
#[must_use]
pub const fn orientation_landscape() -> &'static str {
    "Landscape"
}

/// Label for the custom width field. States its unit.
#[must_use]
pub const fn custom_width() -> &'static str {
    "Width (mm)"
}

/// Label for the custom height field.
#[must_use]
pub const fn custom_height() -> &'static str {
    "Height (mm)"
}

/// The resulting sheet, echoed under the controls in both units.
///
/// # Why it is echoed at all when the list entry already says it
///
/// Because the list entry says the size **portrait**, and the orientation
/// toggle beside it can transpose it. A reader who picks A1 and then Landscape
/// has been shown "841 × 1189 mm" and is about to get 1189 × 841. One line
/// that reports the actual outcome removes the arithmetic.
///
/// It also reports **points**, which the list deliberately does not. Points
/// are what the `/MediaBox` will say and what every other measurement in this
/// application is in, so an operator comparing this sheet against a drawing
/// that arrived from CAD needs them. Millimetres first because that is the
/// unit the decision was made in.
#[must_use]
pub fn sheet_summary(width_pt: f64, height_pt: f64) -> String {
    let mm = |pt: f64| (pt * 25.4 / 72.0).round() as i64;
    format!(
        "Sheet: {} × {} mm ({:.0} × {:.0} pt)",
        mm(width_pt),
        mm(height_pt),
        width_pt,
        height_pt,
    )
}

/// ★ Why a custom size is being refused, shown in place of [`sheet_summary`].
///
/// # The refusal is the shell's, and it is made BEFORE the engine's
///
/// `EditSession::set_media_box` normalizes and then refuses a degenerate
/// rectangle by name (`EditError::MediaBoxDegenerate`), so a zero-width sheet
/// cannot reach a file whatever this dialog does. That refusal is the right
/// backstop and the wrong operator experience: it arrives *after* the
/// document has been created and failed, as a `Status::Failed` where a working
/// document used to be.
///
/// So the dialog checks first and simply does not offer Create. The engine's
/// guard stays where it is — a shell-side check that replaced it would be the
/// second implementation this project keeps warning about — and this sentence
/// exists so the missing button is not a mystery.
///
/// # Why the ceiling is stated rather than clamped
///
/// A sheet larger than the ceiling is refused, not silently reduced. A
/// silently shortened sheet is a wrong document that looks like a pdfce
/// scaling bug — the same reasoning `pdfce-print` gives for refusing rather
/// than clamping a custom `DEVMODE` sheet, arrived at independently on the
/// other side of the application.
///
/// # ★ Where the ceiling comes from, and the caveat on it
///
/// **14,400 default user space units = 200 inches = 5,080 mm**, from
/// ISO 32000-1 Annex C.2: *"The minimum page size should be 3 by 3 units in
/// default user space; the maximum should be 14,400 by 14,400 units."*
///
/// That is a **`should`, not a `shall`**, and the caveat matters enough to
/// write down: ISO 32000-2:2020 retitles Annex C *"Advice on maximising
/// portability"*, makes it informative, and **drops every numeric limit in
/// it** — the page-size range included. So this is 1.7-era portability advice
/// with no 2.0 successor, and pdfce is choosing to honour it.
///
/// The choice is defensible on its own terms rather than on the standard's: a
/// New command exists to make a sheet somebody will work on, 5 m covers every
/// drafting size that exists by a factor of four (A0 is 1,189 mm, ANSI E is
/// 1,118 mm), and a page beyond it is one that widely-deployed readers have
/// historically refused to open. An operator who genuinely needs a 10 m banner
/// is not served by a *New* dialog either way.
///
/// Sourced from `D:\Dev\Rag-Specialized\PDF_Spec\iso32000\iso32000__annex__c.md`,
/// which carries both the 1.7 text and the measured 2.0 delta. It is written
/// down here because a number in a validity check with no provenance is
/// indistinguishable from a number somebody guessed.
#[must_use]
pub fn custom_refused(min_mm: i64, max_mm: i64) -> String {
    format!("Each side must be between {min_mm} and {max_mm} mm. Nothing is made until both are.")
}

/// The Create button.
///
/// Its own label rather than "OK", on the same rule the print dialog's commit
/// button follows: a button that *does the thing* should say the thing. "OK"
/// on a dialog with a size list reads as "keep this setting", and this one
/// makes a document and replaces what is open.
#[must_use]
pub const fn create() -> &'static str {
    "Create"
}

/// Hover text for [`create`], stating the consequence.
///
/// **It replaces what is open**, which is `file.new`'s behaviour and is stated
/// in that command's tooltip too. Repeated here rather than referenced,
/// because an operator who reached this window from the ribbon has not
/// necessarily read the other control's tooltip — and the consequence is the
/// one thing about this dialog that is not undoable.
///
/// A document with unsaved edits is not replaced: the action is declined at
/// `crate::app::actions::apply`, exactly as `file.new` is. That is not stated
/// here, because a tooltip is not the place to describe a guard the operator
/// will only meet if it saves them.
#[must_use]
pub const fn create_tooltip() -> &'static str {
    "Makes the document and replaces what is open."
}

/// The Cancel button.
#[must_use]
pub const fn cancel() -> &'static str {
    "Cancel"
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★ The millimetre conversion agrees with the engine's own table.
    ///
    /// Not a tautology: [`size_entry`] and [`sheet_summary`] each convert
    /// points to millimetres, and `pdfce_core::paper` builds its points *from*
    /// millimetres by the inverse constant. A rounding or a transposed
    /// constant here would show A1 as "593 × 840" beside a file that really is
    /// A1 — a discrepancy an operator would read as pdfce getting the standard
    /// wrong.
    #[test]
    fn a_named_size_reads_back_as_its_own_millimetres() {
        let a1 = pdfce_core::paper::PaperSize::A1.size_pt();
        let label = size_entry("A1", a1);
        assert!(
            label.contains("594 × 841"),
            "A1 must read as its defining millimetres: {label}"
        );

        let a4 = pdfce_core::paper::PaperSize::A4.size_pt();
        let summary = sheet_summary(a4.0, a4.1);
        assert!(
            summary.contains("210 × 297"),
            "A4 must read as its defining millimetres: {summary}"
        );
        // And the points are there too, because a CAD comparison needs them.
        assert!(
            summary.contains("595"),
            "the summary must state points: {summary}"
        );
    }

    /// ★ The refusal names the ceiling.
    ///
    /// An operator told only that their number is wrong has to guess. The
    /// number is what turns a refusal into an instruction, and it is the
    /// single thing most likely to be dropped by a later rewording.
    #[test]
    fn the_custom_refusal_states_the_limits() {
        let message = custom_refused(1, 5080);
        assert!(message.contains("5080"), "no upper limit in {message}");
        assert!(message.contains('1'), "no lower limit in {message}");
    }

    /// ★ No size in the list reads like an identifier.
    ///
    /// [`size_name`]'s wildcard exists so a size added to `PaperSize` after
    /// this build still appears in the list, under `id().to_uppercase()`. This
    /// pins that the wildcard is the **exception**.
    ///
    /// # Why it cannot be written as "the name differs from the fallback"
    ///
    /// Because for seven of the sixteen it does not, and correctly: the
    /// uppercased id of `A0` is `"A0"`, which is also its right name. That
    /// version of this test was written first and failed on its first run,
    /// reporting `A0` as a defect. Which was useful — it is the same shape as
    /// a test asserting a refusal that outlives its premise, caught early.
    ///
    /// What distinguishes a fallback that is *wrong* is that an identifier is
    /// hyphenated where a name is spaced: `ansi-d` becomes `"ANSI-D"` and
    /// should read `"ANSI D"`. So the property asserted is that **no name
    /// contains a hyphen** — which holds for every size today, fails for any
    /// multi-word size the engine adds, and says something true rather than
    /// something merely checkable.
    #[test]
    fn no_size_in_the_list_reads_like_an_identifier() {
        for size in pdfce_core::paper::PaperSize::ALL {
            let name = size_name(*size);
            assert!(
                !name.contains('-'),
                "{size:?} rendered as {name:?} — a hyphen means it fell through to the \
                 identifier fallback and needs a written name"
            );
            assert!(!name.is_empty(), "{size:?} has an empty name");
        }
        // And the ANSI pair specifically, because they are the ones the
        // fallback would visibly mangle and the ones most likely to be added
        // to in future (ARCH A-E are named as plausible).
        assert_eq!(size_name(pdfce_core::paper::PaperSize::AnsiD), "ANSI D");
    }
}
