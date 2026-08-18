//! # `dialogs::print::spooler::device` — what a printer IS, and how it is configured
//!
//! ## The seam this file is on the other side of
//!
//! [`super`] is the adapter for **the job**: which pages, at what size, in
//! what order, placed where on a sheet. This file is the adapter for **the
//! device**: which printers exist, what each one can do, which sheets it
//! offers, and what its driver currently holds.
//!
//! The two change for different reasons, which is the test R2 asks for when
//! a file comes due for splitting. A change to how a job is *laid out* — a
//! new scale mode, an imposition tab, a different resolution ceiling — never
//! touches this file. A change to how a device is *interrogated* — a paper
//! list, a properties dialog, a tray capability — never touches the
//! placement arithmetic next door. They were one file until 2026-08-18, and
//! the split happened because the paper work would have carried the total
//! past the 1,500-line limit; the seam was already there.
//!
//! ## What is still true of both halves
//!
//! **This module and its parent are the only files in the crate that name
//! `pdfce_print`.** Everything else in [`crate::dialogs::print`] — the three
//! tabs, the preview, the footer — is written against the mirrored types
//! here. That is what confined "make printing work" to one module in
//! August 2026, and it is worth keeping: see [`super`]'s header for the full
//! reasoning, including why no arithmetic is ever mirrored.
//!
//! ## ★ The rule that governs every capability query here
//!
//! **A query that answers "I do not know" is not a query that answered
//! "no".** `pdfce-print` was explicit about this when it declined this
//! project's proposal to gate the tray control on a `bool`:
//!
//! > *"`DC_BINS` on Microsoft Print to PDF returns nothing at all, while
//! > that same device's `dmDefaultSource` is already `DMBIN_FORMSOURCE` — it
//! > picks by form by default. A bool would have collapsed 'the driver said
//! > nothing' into 'no', and told the operator a device cannot do the thing
//! > it was already doing."*
//!
//! So [`FormSourceSupport`] has three states and not two, and the shell's
//! reading of them is the inverse of R83's usual direction: **`NotListed`
//! and `Unknown` still get the control**, with the disclosure. R83 forbids
//! offering an affordance the hardware *cannot* honour; it does not forbid
//! offering one the driver merely declined to advertise.
//!
//! Contrast [`DeviceFeatures::supports_duplex`], which is a genuine
//! capability answer and *is* gated: `DC_DUPLEX` returning zero means the
//! device is simplex, and no setting in the dialog will change that.

use super::Unavailable;

// ---------------------------------------------------------------------------
// The device, and what it says about itself
// ---------------------------------------------------------------------------

/// One printer the system knows about. Maps to `pdfce_print::Printer`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Printer {
    /// The name the spooler reports, and the one a job is addressed to.
    pub(crate) name: String,
    /// The driver's name.
    ///
    /// Carried because two printers can share a human-readable name closely
    /// enough that an operator cannot tell them apart, and the driver usually
    /// distinguishes them. Traced rather than shown today: the selector is a
    /// combo of names, and a two-line row is a change to make on evidence
    /// that the ambiguity actually bites.
    pub(crate) driver: String,
    /// The port, for the same reason as [`Self::driver`].
    pub(crate) port: String,
    /// Whether this is the system default — the dialog's initial selection.
    pub(crate) is_default: bool,
}

/// What a device says it can do, beyond geometry.
///
/// Maps to `pdfce_print::DeviceFeatures`. Read **once**, when the dialog
/// opens: asking a driver this question sixty times a second while a dialog
/// sits open would be rude to a service other applications share.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct DeviceFeatures {
    /// The driver reports duplex support. The dialog draws no duplex control
    /// without it (R83).
    pub(crate) supports_duplex: bool,
    /// How many copies the driver can produce itself.
    ///
    /// **Reported, not used.** pdfce sends its own sequence today, so this is
    /// carried to the trace so a later decision about hardware collation can
    /// be made on evidence rather than on assumption.
    pub(crate) max_copies: u16,
}
// ---------------------------------------------------------------------------
// The queries into the engine
// ---------------------------------------------------------------------------
//
// Each one is called on a CHANGE — the dialog opening, or the selected
// printer changing — and never per frame. Asking a driver these questions
// sixty times a second while a dialog sits open would be rude to a service
// other applications share, and two of them (`printer_configuration`,
// `printer_forms`) open a device context to do it.

/// Enumerate the system's printers.
///
/// Called **once**, when the dialog opens — enumerating printers touches the
/// spooler, and doing it per frame while a dialog sits open would be rude to
/// a service other applications share. [`super::PrintDialog::new`] is the
/// only caller and it stores the result.
///
/// # Errors
///
/// [`Unavailable::Spooler`] when the spooler could not be queried at all,
/// which on a non-Windows target is always (`PrintError::Unsupported`).
///
/// **An empty `Vec` is `Ok`, not an error.** A machine with no printers
/// installed is a normal machine; see [`Unavailable`]'s own documentation for
/// why the type has nowhere to put that case.
pub(crate) fn list_printers() -> Result<Vec<Printer>, Unavailable> {
    match pdfce_print::list_printers() {
        Ok(found) => Ok(found
            .into_iter()
            .map(|printer| Printer {
                name: printer.name,
                driver: printer.driver,
                port: printer.port,
                is_default: printer.is_default,
            })
            .collect()),
        Err(error) => Err(Unavailable::Spooler(error.to_string())),
    }
}

/// Read one device's non-geometric capabilities.
///
/// Consulted **before** offering the duplex control at all (R83), never
/// after. [`super::PrintDialog::refresh_features`] calls it once per change
/// of the selected printer — which is the fix for a defect the old shell
/// still carries: it read features only for the *initially* selected device
/// and never again, so switching printers left the duplex control gated on
/// the previous one's capabilities.
///
/// # Errors
///
/// [`Unavailable::Spooler`] when the driver would not answer. The caller
/// falls back to [`DeviceFeatures::default`] — `supports_duplex: false` —
/// which is the safe direction: a device that cannot describe itself gets no
/// duplex control, rather than a control that may silently do nothing.
pub(crate) fn device_features(printer: &str) -> Result<DeviceFeatures, Unavailable> {
    match pdfce_print::device_features(printer) {
        Ok(features) => Ok(DeviceFeatures {
            supports_duplex: features.supports_duplex,
            max_copies: features.max_copies,
        }),
        Err(error) => Err(Unavailable::Spooler(error.to_string())),
    }
}
