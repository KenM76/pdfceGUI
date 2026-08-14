//! Every `unsafe` line in the crate, behind one safe API.
//!
//! ## Why the platform code is quarantined here
//!
//! Two reasons.
//!
//! **Review surface.** Everything else in this crate is ordinary safe Rust
//! that a reviewer can read quickly. Interleaving `mouse_event` calls with
//! check logic would mean every future check has to be reviewed as if it
//! contained `unsafe`, which is how `unsafe` spreads.
//!
//! **Portability without pretence.** The harness only works on Windows — it
//! drives a Windows window through the Windows input API. But the *workspace*
//! must build elsewhere, or the crate gets dropped from the members list the
//! first time somebody runs `cargo check` on a laptop. So there are two
//! implementations of one API: [`win32`] on Windows, [`unsupported`]
//! everywhere else, which compiles and returns a clear error naming the
//! platform. The harness reports SKIPPED, not a build failure and not a pass.
//!
//! ## The API
//!
//! | Function | Job |
//! |---|---|
//! | [`find_window_for_pid`] | which top-level window belongs to the process we launched |
//! | [`window_frame`] | where its client area is, and at what DPI scale |
//! | [`raise_window`] | bring it to the foreground before driving or capturing it |
//! | [`cursor_position`] / [`set_cursor_position`] | the pointer |
//! | [`mouse_button`] | primary button down/up |
//! | [`key_stroke`] | a virtual key press and release |
//! | [`capture_screen`] | a desktop region as BGRA pixels |
//!
//! Note what is absent: there is no `send_message`, and there never will be.
//! See [`crate::input`] for why posting messages to the window was tried in
//! this project's predecessor and does not work.

#[cfg(windows)]
mod win32;
#[cfg(windows)]
pub use win32::*;

#[cfg(not(windows))]
mod unsupported;
#[cfg(not(windows))]
pub use unsupported::*;

/// Virtual-key codes the harness needs, named so that call sites read as
/// keystrokes rather than as magic numbers.
///
/// Deliberately a tiny closed list rather than a binding of the whole
/// `VIRTUAL_KEY` space: a harness that can press any key is a harness whose
/// scripts stop being readable.
pub mod vk {
    /// `Delete`. The key D1 is about.
    pub const DELETE: u16 = 0x2E;
    /// `Escape` — closes a dialog, cancels a tool.
    pub const ESCAPE: u16 = 0x1B;
    /// `Backspace`. Bound to the same action as Delete in this application,
    /// and suppressed by the same guard, so a check that presses one should
    /// usually be able to press the other.
    pub const BACKSPACE: u16 = 0x08;
    /// `Enter` — steps to the next Find hit, commits the page box.
    pub const ENTER: u16 = 0x0D;

    /// `Ctrl`, as a **modifier** for [`super::key_stroke_with`].
    ///
    /// Named `CONTROL` rather than `CTRL` because that is what Windows calls
    /// it (`VK_CONTROL`), and a constant that renames a platform's own
    /// vocabulary makes the next person check twice.
    pub const CONTROL: u16 = 0x11;
    /// `Shift`, as a modifier. `Ctrl+Shift+…` is two entries in the slice.
    pub const SHIFT: u16 = 0x10;

    /// `F` — the letter, for `Ctrl+F`.
    ///
    /// Letters are their ASCII uppercase code point on Windows, which is why
    /// this is `0x46` and not something derived. Only the letters the harness
    /// actually presses are listed: the closed-list rule above applies to
    /// letters more than to anything else, because `pub const A..Z` would be
    /// exactly the "can press any key" the doc comment refuses.
    pub const F: u16 = 0x46;
    /// `2` — the digit, for the `Ctrl+2` mode chord.
    ///
    /// Present only as a **control probe**: `Ctrl+2` is bound to
    /// `mode.review` and is a chord the application's key table has always
    /// been able to spell, so a check that gets nothing from it learns that
    /// the keystroke never arrived, rather than that the feature under test
    /// is broken.
    pub const DIGIT_2: u16 = 0x32;
}
