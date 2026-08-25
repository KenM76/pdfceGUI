//! # `dialogs::settings::colour` — the two questions about ink
//!
//! The group that **starts expanded**, because it holds the setting most likely
//! to have brought someone to this window: *"my black lines look grey."*
//!
//! It is also the only group containing a default that **knowingly departs from
//! what Acrobat and pdfium do**, on an explicit operator ruling — and that
//! departure is disclosed at the setting rather than in a footnote, because the
//! person reading this radio group is precisely the person who has noticed the
//! difference and is deciding whether it is a bug.

use egui::Ui;
use pdfce_core::settings::{CmykIntent, CmykJpegPolarity, MeshPatchPadding, PageBlendSpaceSource};

use super::{Draft, widgets};
use crate::text::settings as t;

/// How CMYK ink becomes screen colour.
///
/// # The default is a deliberate divergence, and the order says so
///
/// By the standing rule the default here would be `Calibrated`: that is tier
/// (a)/(c) evidence — Acrobat's shipped profile *and* pdfium both produce it —
/// which is the strongest evidence behind any default in this window. It is
/// `NeutralBlack` anyway, because the operator looked at what calibrated
/// rendering does to pure-K line art and overruled it.
///
/// **The default is listed first**, ahead of the better-sourced option, and
/// that ordering is the argument: an operator scanning this group should see
/// what pdfce is currently doing before they see the alternatives. Every other
/// setting in the window lists its default first for the same reason, so the
/// one place it would be tempting to make an exception is the one place the
/// consistency matters most.
///
/// The divergence is narrow by construction — only the pure-K axis moves, every
/// mixed colour still uses the measured table — and the option note says so,
/// because an operator worrying that pdfce has invented its own colour science
/// deserves to be answered from the window rather than from the source.
pub fn intent(ui: &mut Ui, draft: &mut Draft) {
    widgets::header(
        ui,
        t::cmyk_intent_title(),
        t::cmyk_intent_silence(),
        t::cmyk_intent_radius(),
    );
    widgets::option(
        ui,
        &mut draft.working.cmyk_intent,
        CmykIntent::NeutralBlack,
        t::cmyk_intent_neutral_label(),
        Some(t::cmyk_intent_neutral_note()),
    );
    widgets::option(
        ui,
        &mut draft.working.cmyk_intent,
        CmykIntent::Calibrated,
        t::cmyk_intent_calibrated_label(),
        Some(t::cmyk_intent_calibrated_note()),
    );
    widgets::option(
        ui,
        &mut draft.working.cmyk_intent,
        CmykIntent::Naive,
        t::cmyk_intent_naive_label(),
        Some(t::cmyk_intent_naive_note()),
    );
    // A disclosure rather than a note: it is about the SETTING, not about the
    // option it sits under, and it must not be weak-grey. A future session must
    // be able to see that pdfce chose differently on purpose, or the next
    // render-parity difference gets investigated as a defect.
    widgets::disclosure(ui, t::cmyk_intent_divergence());
}

/// Whether a CMYK JPEG's ink values are stored inverted.
///
/// # ★ The one well-sourced default in the whole window
///
/// Every other default here is *reasoned inference* — a guess — and says so.
/// This one is not, and says that instead: `"invert"` occurs **zero times** in
/// the Adobe technical note ISO 32000-1 makes normative, the APP14 marker
/// carries **no polarity flag at all** (so "invert on marker" keys off mere
/// presence), and all four reference engines accept the ambiguity rather than
/// inverting.
///
/// Keeping the two claims distinguishable is the point. "pdfce matched every
/// other implementation" and "pdfce guessed" must not read alike, or the
/// operator has no way to tell which of thirteen defaults to trust — and the
/// catalog has a test for each direction.
///
/// # It is also the only preview setting that can change saved bytes
///
/// A re-encode under the wrong polarity bakes the inversion in **permanently**,
/// which is why its radius line names the saved file and the other four preview
/// settings' do not.
pub fn polarity(ui: &mut Ui, draft: &mut Draft) {
    widgets::header(
        ui,
        t::polarity_title(),
        t::polarity_silence(),
        t::polarity_radius(),
    );
    widgets::option(
        ui,
        &mut draft.working.cmyk_jpeg_polarity,
        CmykJpegPolarity::NeverInvert,
        t::polarity_never_label(),
        Some(t::polarity_never_note()),
    );
    widgets::option(
        ui,
        &mut draft.working.cmyk_jpeg_polarity,
        CmykJpegPolarity::InvertOnApp14,
        t::polarity_invert_label(),
        Some(t::polarity_invert_note()),
    );
}

/// Where a page's blending colour space comes from when its own group
/// declares none — the engine's `PGB-A1`, and the setting that decides
/// whether **overprint** is simulated at all.
///
/// # Why this belongs in the Colour group and not in Images
///
/// Because of the symptom that sends somebody looking for it. It is not an
/// image question; it is *"the overprinted areas in my print file look wrong"*
/// — or, from the other direction, *"this file renders differently in pdfce
/// than it did last month."* Both are ink questions, and this is the ink
/// group.
///
/// # What the operator is actually choosing between
///
/// ISO 32000-1 §11.4.7 says a page with no declared blending space uses the
/// **device's native** space. pdfce's pixmap is RGBA8, which is additive — and
/// in an additive space overprint is not merely unsimulated, it is
/// **unrepresentable**: §11.7.4.3 makes the blend function return the source
/// colour for every component *"specified in the current colour space"*, and
/// in sRGB every component is always specified. The engine measured the
/// consequence on the Ghent PDF Output Suite: **24 of its 51 patches request
/// overprint**, and under the literal reading all 24 are wrong.
///
/// So the shipped default consults the file's **output intent**, but only when
/// that intent is subtractive. That conditional is what makes it safe: an RGB
/// or greyscale intent cannot drag a page into ink, so the only files it moves
/// are ones that already declare themselves destined for print.
///
/// ★ The three options are ordered **strict → shipped → most literal reading
/// of Annex P**, and the middle one is the default. That is deliberate: an
/// operator scanning this group meets the conforming-but-degenerate answer
/// first, so *why is the default not the one the standard says* is answered
/// before it is asked.
pub fn page_blend_space(ui: &mut Ui, draft: &mut Draft) {
    widgets::header(
        ui,
        t::blend_space_title(),
        t::blend_space_silence(),
        t::blend_space_radius(),
    );
    widgets::option(
        ui,
        &mut draft.working.page_blend_space_source,
        PageBlendSpaceSource::DeviceNative,
        t::blend_space_label(PageBlendSpaceSource::DeviceNative),
        Some(t::blend_space_note(PageBlendSpaceSource::DeviceNative)),
    );
    widgets::option(
        ui,
        &mut draft.working.page_blend_space_source,
        PageBlendSpaceSource::OutputIntentIfSubtractive,
        t::blend_space_label(PageBlendSpaceSource::OutputIntentIfSubtractive),
        Some(t::blend_space_note(
            PageBlendSpaceSource::OutputIntentIfSubtractive,
        )),
    );
    widgets::option(
        ui,
        &mut draft.working.page_blend_space_source,
        PageBlendSpaceSource::OutputIntentAlways,
        t::blend_space_label(PageBlendSpaceSource::OutputIntentAlways),
        Some(t::blend_space_note(
            PageBlendSpaceSource::OutputIntentAlways,
        )),
    );
}

/// How a mesh-shading patch record is byte-padded (spec ambiguity `MSH-A1`).
///
/// # Why this is in Colour and not in Images
///
/// Because the thing that goes wrong is a **gradient**, and a gradient is
/// colour. The mechanism is a bit-alignment question about a binary stream,
/// which would file it under nothing an operator can name; the symptom is
/// *"this smooth fill came out as noise"*, and that is what the title says.
///
/// # ★ It is observable in very few files, and the note says so honestly
///
/// The two readings agree unless `BitsPerFlag + k·BitsPerCoordinate +
/// m·BitsPerComponent` fails to be a multiple of 8. Every combination the
/// engine has measured in real files — 8/32/8 in the print-conformance suite's
/// type 7 meshes, and the common 8/16/8 — is byte-aligned for every record
/// shape, so the two render identically. A file with `BitsPerFlag` 2 or 4, or
/// 12-bit coordinates, is where they diverge, **and there the divergence is
/// total**: one record out of step desynchronises every record after it.
///
/// That is why the second option's note is phrased as a remedy to try rather
/// than as a preference to hold — an operator will only ever reach this control
/// because something on screen is already wrong.
pub fn mesh_patch_padding(ui: &mut Ui, draft: &mut Draft) {
    widgets::header(
        ui,
        t::mesh_padding_title(),
        t::mesh_padding_silence(),
        t::mesh_padding_radius(),
    );
    widgets::option(
        ui,
        &mut draft.working.mesh_patch_padding,
        MeshPatchPadding::PerRecord,
        t::mesh_padding_record_label(),
        Some(t::mesh_padding_record_note()),
    );
    widgets::option(
        ui,
        &mut draft.working.mesh_patch_padding,
        MeshPatchPadding::None,
        t::mesh_padding_none_label(),
        Some(t::mesh_padding_none_note()),
    );
}
