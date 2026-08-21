//! # `native-window` — the two or three things the window manager will only
//! # tell the operating system
//!
//! ## What this is for
//!
//! Almost nothing in this shell needs to know what a window manager is.
//! `eframe` opens the window, `egui` draws into it, and a viewport is a
//! platform-neutral idea. This module exists for the cases where that
//! abstraction has a **hole in it that costs the operator something**, and
//! where the toolkit exposes no way to say what needs saying.
//!
//! Today there is exactly one: **a dialog must be OWNED by the window it
//! belongs to.**
//!
//! ## ★★★ Why ownership, and why it is not cosmetic
//!
//! `ui-conventions/dialogs.md` G3 states the rule and this project has now
//! paid for its absence twice:
//!
//! 1. **The dialog can fall behind the application window**, which is the
//!    classic Windows bug — a program that appears to have frozen because the
//!    thing waiting for an answer is behind the thing it is blocking.
//! 2. **The dialog loses the keyboard a third of a second after it opens.**
//!    Measured 2026-08-21, with both windows reporting their own focus:
//!
//!    ```text
//!    dialog-focus  focused=Some(true)     the note window is given the keyboard
//!    root-focus    focused=Some(false)
//!    …17 idle passes: no resize, no reposition, no input, nothing asked for…
//!    root-focus    focused=Some(true)     and Windows hands it BACK
//!    dialog-focus  focused=Some(false)
//!    ```
//!
//!    The operator's version: *drag out a note box, type without clicking the
//!    field first, and the words go nowhere.*
//!
//! ★ **Asking again does not work, and that was tried.** Half a second of
//! `ViewportCommand::Focus`, one per pass, straight through the moment of the
//! loss — the root still takes the foreground back. Windows refuses the
//! foreground to a process that does not already hold it, silently, which is
//! the same rule `tools/ui-verify` documents at length about
//! `SetForegroundWindow`.
//!
//! **Ownership is not a request.** An owned window is *by definition* above its
//! owner in z-order, and activation follows the relationship rather than a call
//! that can be declined. It is the mechanism every native dialog on this
//! machine already uses, which is why none of them has this problem.
//!
//! ## ★ Why `eframe` cannot express it, and why this is not a workaround
//!
//! `ViewportBuilder` has thirty-odd options in `egui 0.35` and none of them is
//! an owner; `egui-winit` never passes down the parent relationship egui itself
//! tracks in `viewport_parents`. There is also no way to get the child window's
//! handle back out — `eframe::Frame` hands out the ROOT window's, once.
//!
//! So the handle is found the same way `tools/ui-verify` finds it: by asking
//! the operating system for **this process's** top-level windows and matching
//! on the title. That is a real lookup rather than a hack — the title is what
//! the application asked the platform to call the window, so it is the one name
//! both sides already agree on.
//!
//! ★★ **The process check is not optional.** `FindWindowExW` searches every
//! window on the desktop, so a title match alone could name another
//! application's window — and `SetWindowLongPtrW` on somebody else's window is
//! a real thing to do to somebody else's program. Every function here confirms
//! the window belongs to this process before touching it.
//!
//! ## What this module refuses to become
//!
//! A platform layer. There is no trait, no abstraction, and no second backend
//! beyond a no-op for every non-Windows target. When the toolkit grows an owner
//! option, this module is **deleted** rather than ported — which is why it is
//! one file with one public function and no state.

#![cfg_attr(not(windows), allow(unused))]

#[cfg(windows)]
mod win32;

#[cfg(windows)]
pub use win32::own_window;

/// **Make the window titled `title` owned by `owner`.** No-op off Windows.
///
/// See the module header. `owner` is a raw window handle as
/// [`crate::app::window_handle`] produces it.
#[cfg(not(windows))]
pub fn own_window(_owner: isize, _title: &str) -> bool {
    // Every other platform: dialogs are already handled correctly by their
    // window managers, or the port has not happened. Answering `false` rather
    // than `true` keeps a caller that logs the outcome honest.
    false
}
