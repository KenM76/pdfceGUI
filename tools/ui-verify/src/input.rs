//! Drive the pointer and the keyboard **through the operating system**.
//!
//! # Why OS-level injection, and not the two easier alternatives
//!
//! This is the module's central design decision, and it is required by the
//! defect the harness exists to catch. Three ways to get input into an egui
//! application were available; two of them cannot see D1.
//!
//! ## Rejected: `PostMessage(WM_MOUSEMOVE / WM_LBUTTONDOWN)`
//!
//! Tried first in this project's predecessor, and it **does not work for an
//! off-screen window** — with a silent failure, which is the worst kind. winit
//! calls `TrackMouseEvent` on the move; Windows answers `WM_MOUSELEAVE`
//! because the physical cursor is elsewhere; `egui-winit` then drops the
//! button entirely, because it emits `PointerButton` only when it knows the
//! pointer position. The observed event list was `[PointerMoved, PointerGone]`
//! in **every** message ordering tried, including move and button posted back
//! to back. That finding is recorded in `D:\dev\rag\egui\`, and it is recorded
//! here too so nobody rebuilds it.
//!
//! ## Rejected as the *primary* driver: in-process injection
//!
//! The application already has one — `PDFCE_DIAG_SCRIPT` feeds steps through
//! eframe's `raw_input_hook`. It is excellent, it needs no screen, and it is
//! the right tool for a behavioural question on a machine the operator is
//! using. It is **the wrong oracle for D1**, and precisely because of what it
//! skips.
//!
//! D1's causal chain is:
//!
//! 1. the canvas calls `request_focus()` when `Response::clicked()` fires;
//! 2. `ctx.egui_wants_keyboard_input()` — which means *any widget has focus*,
//!    not *a text field has focus* — therefore returns `true` forever after;
//! 3. so the unmodified-key bindings, `Delete` among them, are never
//!    installed.
//!
//! Every link in that chain is about **what egui's focus machinery does with a
//! real click**. A harness that hands egui a synthetic `PointerButton` event
//! is asserting on the same layer that is broken. It might well reproduce the
//! bug — but a green result from it would not be evidence, because the thing
//! it skipped is the thing in question. The only way to be sure the click that
//! selects the object is the same click that focuses the canvas is to make the
//! window manager deliver it.
//!
//! Put plainly: **the harness that must catch D1 has to go through the OS,
//! because D1 is a defect in how the application responds to the OS.**
//!
//! ## Chosen: `SetCursorPos` + `mouse_event` + `keybd_event`
//!
//! System-level injection. The cursor really moves, the click lands on
//! whatever window is in front, and the keystroke goes to the foreground
//! window. Everything downstream — hit testing, focus, hover, capture — runs
//! exactly as it does for a person.
//!
//! `mouse_event`/`keybd_event` rather than `SendInput`: for a button at the
//! current position they are equivalent, and their signatures have no
//! variable-length array to get wrong. The one thing `SendInput` would buy is
//! atomic multi-event batches, which is not wanted — a real click is not
//! atomic either.
//!
//! ## What that costs, and how it is paid
//!
//! It commandeers the real desktop. Three mitigations, all mechanical:
//!
//! * [`Driver::new`] records the pointer position and [`Driver`]'s [`Drop`]
//!   puts it back, on every path including a panic.
//! * [`Driver::click_at`] and [`Driver::press`] raise the target window first,
//!   and `press` refuses if there is no window — a keystroke sent to the wrong
//!   window is not a failed keystroke, it is a keystroke into the operator's
//!   editor.
//! * Checks are short, and each one holds the desktop for a couple of seconds
//!   rather than a couple of minutes.
//!
//! The harness is honest about this rather than clever: it is a foreground
//! activity, it says so when it starts, and `--no-input` turns it off (whereupon
//! the checks that need it report SKIPPED, never PASS).
//!
//! # The PowerShell fallback
//!
//! [`PowerShellDriver`] does the same three operations by shelling out to
//! `Add-Type`'d `user32` P/Invokes. It exists for two reasons: it is what the
//! predecessor scripts used, so a finding reproduced there can be reproduced
//! here; and it keeps the harness usable if the `windows-sys` binding ever has
//! to be dropped. It is **not** the default — it costs a process per event,
//! which turns a three-event click into three process spawns and makes the
//! timing unlike a real click.

use std::time::Duration;

use crate::coords::ScreenPoint;
use crate::error::{Error, Result};
use crate::sys::{self, WindowHandle};

/// How long the primary button stays down during a synthetic click.
///
/// Long enough that the application sees a press and a release on different
/// frames, which is what a real click looks like. Zero-length clicks have been
/// observed to be coalesced by frameworks that sample input once per frame,
/// and a coalesced click is a click the application never saw.
const CLICK_HOLD: Duration = Duration::from_millis(60);

/// How long to wait after moving the pointer before pressing.
///
/// The application needs at least one frame to process the move and update its
/// hover state; several widgets only respond to a click when they were hovered
/// on the preceding frame.
const MOVE_SETTLE: Duration = Duration::from_millis(80);

/// How many intermediate positions [`Driver::drag`] walks through.
///
/// Enough that the application sees the pointer *travel* rather than teleport —
/// see that method's docs on why a two-point drag can be delivered as a click.
/// Not more, because each step costs [`DRAG_STEP_SETTLE`] and a check that holds
/// the operator's desktop is one that should finish.
const DRAG_STEPS: u32 = 8;

/// How long to pause between the intermediate positions of a drag.
///
/// Shorter than [`MOVE_SETTLE`]: the application does not need to settle at each
/// waypoint, it only needs to *observe* each one, which is one frame at 60 Hz.
/// 25 ms is comfortably more than one frame on any machine that can render a
/// PDF page at all.
const DRAG_STEP_SETTLE: Duration = Duration::from_millis(25);

/// The OS-level input driver.
///
/// Owns the operator's pointer position for its lifetime and returns it on
/// drop.
pub struct Driver {
    original_cursor: Option<(i32, i32)>,
    target: Option<WindowHandle>,
}

impl Driver {
    /// Take the pointer, remembering where it was.
    ///
    /// `target` is the window every action is aimed at. It is required for
    /// keystrokes and used to raise before pointer actions.
    #[must_use]
    pub fn new(target: Option<WindowHandle>) -> Self {
        Self {
            original_cursor: sys::cursor_position().ok(),
            target,
        }
    }

    /// Move the pointer and click the primary button.
    ///
    /// The window is raised first: a click on a window that is not in front is
    /// consumed by the click-to-focus of whatever *is*, and the application
    /// under test sees nothing. That failure looks identical to a hit test
    /// returning nothing.
    pub fn click_at(&self, p: ScreenPoint) -> Result<()> {
        self.raise();
        sys::set_cursor_position(p.x(), p.y())?;
        std::thread::sleep(MOVE_SETTLE);
        sys::mouse_button(true);
        std::thread::sleep(CLICK_HOLD);
        sys::mouse_button(false);
        std::thread::sleep(MOVE_SETTLE);
        Ok(())
    }

    /// **Press at `from`, travel to `to`, release** — a real primary-button
    /// drag.
    ///
    /// # Why the harness had none until now
    ///
    /// It did not need one. Every check that drives the ribbon uses
    /// [`Self::click_at`], and even `markup_rectangle` — whose *subject* is a
    /// drag gesture — asserts on the ribbon arming rather than on the band,
    /// because a markup band is checkable from the command trace. Canvas **text
    /// selection** is the first feature whose entire behaviour is a drag: there
    /// is no button to press that produces one, and a click alone can only ever
    /// clear a selection.
    ///
    /// # The intermediate moves are the whole reason this is not three calls
    ///
    /// egui decides that a press has become a *drag* rather than a *click* by
    /// distance travelled, and it samples the pointer once per frame. A press
    /// followed immediately by a release at a distant point is delivered as a
    /// single jump: egui sees one position, then another, and may report a
    /// **click** at the far end rather than a drag at all — which for this
    /// feature is the difference between selecting a paragraph and clearing the
    /// selection.
    ///
    /// So the pointer is walked in [`DRAG_STEPS`] increments with a settle
    /// between each, which is what a hand does. The application gets several
    /// frames of `dragged_by(Primary)` with a moving position, which is
    /// precisely the sequence `canvas::gesture::GestureState` is written for.
    ///
    /// The button is held across the walk rather than being pressed at each
    /// step: a released-and-pressed pointer is *n* gestures, not one.
    pub fn drag(&self, from: ScreenPoint, to: ScreenPoint) -> Result<()> {
        self.raise();
        sys::set_cursor_position(from.x(), from.y())?;
        std::thread::sleep(MOVE_SETTLE);
        sys::mouse_button(true);
        std::thread::sleep(CLICK_HOLD);
        for step in 1..=DRAG_STEPS {
            // Integer arithmetic in i64 rather than f64: the endpoints are
            // whole pixels and the intermediate points should be too, so the
            // application is never handed a coordinate a real mouse could not
            // produce.
            let lerp = |a: i32, b: i32| -> i32 {
                let n = i64::from(DRAG_STEPS);
                let (wide_a, wide_b) = (i64::from(a), i64::from(b));
                // The endpoint on overflow rather than a clamp to `i32::MAX`: a
                // coordinate that far out is not a screen position, and
                // finishing where the drag was aimed is the only answer that is
                // not silently somewhere else.
                i32::try_from(wide_a + (wide_b - wide_a) * i64::from(step) / n).unwrap_or(b)
            };
            sys::set_cursor_position(lerp(from.x(), to.x()), lerp(from.y(), to.y()))?;
            std::thread::sleep(DRAG_STEP_SETTLE);
        }
        sys::mouse_button(false);
        std::thread::sleep(MOVE_SETTLE);
        Ok(())
    }

    /// Move the pointer without clicking — for hover assertions, and for
    /// getting the pointer off a widget before a screenshot.
    pub fn move_to(&self, p: ScreenPoint) -> Result<()> {
        sys::set_cursor_position(p.x(), p.y())?;
        std::thread::sleep(MOVE_SETTLE);
        Ok(())
    }

    /// Scroll the pane under a point, then settle.
    ///
    /// Moves the pointer there first, because a wheel event goes to whatever is
    /// under the cursor — scrolling "the panel" means putting the pointer in it.
    ///
    /// # ★ Why a check needs this, and what its absence looked like
    ///
    /// A dock panel is a few hundred points tall and a real document's content
    /// is not, so a check that can only reach what is on screen at launch can
    /// only verify the top of any list. Worse, it reports everything below the
    /// fold as *"the control is drawn and inert"* — which is a **confident,
    /// specific, wrong defect report about a control that works**, and this
    /// harness produced three of those in one day before this existed.
    ///
    /// # Errors
    ///
    /// If the pointer cannot be moved.
    pub fn scroll_at(&self, p: ScreenPoint, notches: i32) -> Result<()> {
        self.move_to(p)?;
        sys::wheel(notches);
        std::thread::sleep(MOVE_SETTLE);
        Ok(())
    }

    /// Press and release a virtual key, in the target window.
    ///
    /// # Errors
    ///
    /// If there is no target window. Refusing is the whole point: keystrokes
    /// go to the foreground window, and if the harness does not know which
    /// window that should be, the keystroke lands in whatever the operator was
    /// typing in. There is no safe default here, so there is no default.
    pub fn press(&self, vk: u16) -> Result<()> {
        if self.target.is_none() {
            return Err(Error::new(
                "refusing to send a keystroke with no target window: it would go to whatever \
                 window is in front, which may be the operator's own",
            ));
        }
        self.raise();
        std::thread::sleep(MOVE_SETTLE);
        sys::key_stroke(vk);
        std::thread::sleep(MOVE_SETTLE);
        Ok(())
    }

    /// Press a **chord** — a virtual key with modifiers held — in the target
    /// window.
    ///
    /// The reason this exists: `Ctrl+F` and every other letter chord in the
    /// manifest keymap were unreachable from this harness, so the checks that
    /// would have driven them could not be written. `press` sends a bare
    /// virtual key with no modifiers, and a shell that binds a command to
    /// `Ctrl+F` cannot be reached by sending `F`.
    ///
    /// # Errors
    ///
    /// If there is no target window, for exactly the reason [`Self::press`]
    /// refuses — and more sharply. A bare keystroke into the operator's editor
    /// types a character. **A chord into the operator's editor runs a
    /// command**, and `Ctrl+W`, `Ctrl+Q` and `Ctrl+S` are all one letter away
    /// from a chord a UI test might plausibly send.
    ///
    /// Modifiers are released by [`sys::key_stroke_with`] on every path; see
    /// its docs for why that is not merely tidy.
    pub fn press_chord(&self, modifiers: &[u16], vk: u16) -> Result<()> {
        self.raise_and_confirm()?;
        sys::key_stroke_with(modifiers, vk);
        std::thread::sleep(MOVE_SETTLE);
        Ok(())
    }

    /// Raise the target and confirm it is actually in front.
    ///
    /// ★ **`raise()` is a request, not a result.** `SetForegroundWindow` is
    /// refused outright for a process without foreground rights — silently,
    /// via a boolean return nobody is obliged to read — so a window that was
    /// created behind an already-active one can stay behind it through any
    /// number of raise calls. Windows' foreground lock exists precisely to
    /// stop background processes stealing focus, and this harness IS a
    /// background process.
    ///
    /// Without this check the failure is not "the keystroke did not arrive".
    /// It is:
    ///
    /// * the keystroke arriving **in the operator's own window** — and for a
    ///   chord that means running one of their commands, not typing a
    ///   character; and
    /// * the check reporting that the FEATURE is broken, when the truth is
    ///   that nothing was ever typed at it. A false failure naming the wrong
    ///   subsystem is worse than no check, because somebody then goes and
    ///   looks at working code.
    ///
    /// Both were observed: a `find_opens_and_finds` run reported "Ctrl+F did
    /// not dispatch `edit.find`" against a build in which Ctrl+F works.
    fn raise_and_confirm(&self) -> Result<()> {
        let Some(w) = self.target else {
            return Err(Error::new(
                "refusing to send input with no target window: it would go to whatever window is in front, which may be the operator's own",
            ));
        };
        self.raise();
        std::thread::sleep(MOVE_SETTLE);
        if !sys::is_foreground(w) {
            return Err(Error::new(
                "the target window could not be brought to the front, so anything typed now would go to the operator's own window. Windows refuses SetForegroundWindow to a process without foreground rights, and this harness is a background process. Reported rather than typed: sending the keystroke anyway would both corrupt whatever IS in front and make this check report the feature as broken when nothing was ever typed at it.",
            ));
        }
        Ok(())
    }

    fn raise(&self) {
        if let Some(w) = self.target {
            sys::raise_window(w);
        }
    }
}

impl Drop for Driver {
    /// Put the operator's pointer back where it was.
    fn drop(&mut self) {
        if let Some((x, y)) = self.original_cursor {
            let _ = sys::set_cursor_position(x, y);
        }
    }
}

/// The same three operations, through PowerShell.
///
/// Kept for the reasons in the module docs. Not the default; one process per
/// event.
pub struct PowerShellDriver;

impl PowerShellDriver {
    /// Move the pointer and click, via `user32` P/Invokes in PowerShell.
    pub fn click_at(p: ScreenPoint) -> Result<()> {
        let script = format!(
            "Add-Type -Namespace UiVerify -Name U -MemberDefinition '\
             [DllImport(\"user32.dll\")] public static extern bool SetCursorPos(int x,int y);\
             [DllImport(\"user32.dll\")] public static extern void mouse_event(uint f,int x,int y,int d,System.UIntPtr e);'; \
             [UiVerify.U]::SetCursorPos({},{}) | Out-Null; Start-Sleep -Milliseconds 80; \
             [UiVerify.U]::mouse_event(0x0002,0,0,0,[System.UIntPtr]::Zero); \
             Start-Sleep -Milliseconds 60; \
             [UiVerify.U]::mouse_event(0x0004,0,0,0,[System.UIntPtr]::Zero)",
            p.x(),
            p.y()
        );
        run_powershell(&script)
    }

    /// Press and release a virtual key, via `user32` P/Invokes in PowerShell.
    pub fn press(vk: u16) -> Result<()> {
        let script = format!(
            "Add-Type -Namespace UiVerify -Name K -MemberDefinition '\
             [DllImport(\"user32.dll\")] public static extern void keybd_event(byte v,byte s,uint f,System.UIntPtr e);'; \
             [UiVerify.K]::keybd_event({vk},0,0,[System.UIntPtr]::Zero); \
             Start-Sleep -Milliseconds 40; \
             [UiVerify.K]::keybd_event({vk},0,2,[System.UIntPtr]::Zero)"
        );
        run_powershell(&script)
    }
}

fn run_powershell(script: &str) -> Result<()> {
    let out = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()
        .map_err(|e| Error::new(format!("cannot run powershell: {e}")))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(Error::new(format!(
            "powershell input step failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )))
    }
}
