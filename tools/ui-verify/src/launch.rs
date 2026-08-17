//! Start the built binary, capture its diagnostic trace, find its window, and
//! never leave it running.
//!
//! ## The binary, not a test harness
//!
//! This module launches `pdfce-gui.exe` — the actual artefact, built in
//! release, opening an actual file. That is the entire premise of the crate:
//! D1 and D2 both live in the space between our code and the framework, and
//! neither is reachable from inside a test binary that constructs an
//! `egui::Context` by hand.
//!
//! ## Two guarantees this module owes the operator
//!
//! **1. It never kills a process it did not start.** The operator may well
//! have the application open for their own work — this harness drives the real
//! desktop, which is precisely the situation where that is most likely — and a
//! harness that tidied up by killing "all pdfce-gui processes" would close
//! their document. So [`Session`] holds a child handle and kills exactly that.
//!
//! **2. It never leaks the process it did start.** pdfce's predecessor script
//! killed its child on its last line, so any error before that line left a
//! window running: parked off-screen, invisible, and still consuming pointer
//! input on the operator's desktop. The operator reported it as *"do you have
//! some gui processes leftover that are interfering with my mouse?"* — twice
//! in one session, which is what made it a defect in the tool rather than an
//! operating mistake. Here the kill is in [`Drop`], so it happens on every
//! path including a panic.
//!
//! ## The staleness gate
//!
//! [`Session::launch`] refuses a binary older than the newest source file
//! under `crates/`, unless explicitly told not to. The failure this prevents
//! is the worst kind: the traces a developer expects are simply **absent**,
//! which reads as "the feature does not work" rather than "the feature was
//! never compiled". pdfce recorded an agent nearly concluding a panel did not
//! render, when the binary predated every change it had made.
//!
//! An absence is only evidence when the thing that would have produced it was
//! actually built.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use crate::coords::WindowFrame;
use crate::error::{Error, Result};
use crate::sys::{self, WindowHandle};
use crate::trace::Trace;

/// How to start the application.
#[derive(Clone, Debug)]
pub struct LaunchSpec {
    /// The binary to run.
    pub exe: PathBuf,
    /// The document to open — passed as `argv[1]`, the way an operator would
    /// open it from the shell.
    pub pdf: Option<PathBuf>,
    /// Environment to add. The diagnostic switch goes here.
    pub env: Vec<(String, String)>,
    /// Where the captured stderr is written.
    pub stderr_path: PathBuf,
    /// How long to wait for a window to appear.
    pub window_timeout: Duration,
    /// Skip the staleness gate. An escape hatch, and it is opt-in for the
    /// reason in the module docs.
    pub allow_stale: bool,
    /// Source tree to check the binary's age against.
    pub source_root: Option<PathBuf>,
}

impl LaunchSpec {
    /// A spec with the harness's defaults.
    #[must_use]
    pub fn new(exe: impl Into<PathBuf>, stderr_path: impl Into<PathBuf>) -> Self {
        Self {
            exe: exe.into(),
            pdf: None,
            env: Vec::new(),
            stderr_path: stderr_path.into(),
            // Generous: a cold start that also has to parse and raster a large
            // CAD drawing is slow, and a timeout that fires early produces a
            // SKIP that looks like a hang.
            window_timeout: Duration::from_secs(30),
            allow_stale: false,
            source_root: None,
        }
    }
}

/// The smallest client area the harness will accept as "the window is up".
///
/// Not a guess at the application's size — a floor below which the window
/// cannot be a laid-out application window. See the polling loop in
/// [`Session::launch`] for what happens without it.
const MIN_CLIENT_PX: u32 = 200;

/// A running application, its captured trace, and its window.
pub struct Session {
    child: Child,
    pid: u32,
    stderr_path: PathBuf,
    window: Option<WindowHandle>,
    trace_prefix: String,
}

impl Session {
    /// Launch, and wait for a window.
    ///
    /// # Errors
    ///
    /// Every error here is a **precondition** failure — the harness could not
    /// begin — so callers report SKIPPED, not FAIL. Each message names the
    /// specific thing that was missing.
    pub fn launch(spec: &LaunchSpec, trace_prefix: &str) -> Result<Self> {
        if !spec.exe.is_file() {
            return Err(Error::new(format!(
                "no binary at {}. Build it first (cargo build --release), or point the \
                 harness at one with --exe.",
                spec.exe.display()
            )));
        }
        if let Some(pdf) = &spec.pdf
            && !pdf.is_file()
        {
            return Err(Error::new(format!("no document at {}", pdf.display())));
        }
        if !spec.allow_stale
            && let Some(root) = &spec.source_root
            && let Some(msg) = staleness_complaint(&spec.exe, root)
        {
            return Err(Error::new(msg));
        }

        if let Some(dir) = spec.stderr_path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let stderr = std::fs::File::create(&spec.stderr_path).map_err(|e| {
            Error::new(format!(
                "cannot create the trace file {}: {e}",
                spec.stderr_path.display()
            ))
        })?;

        let mut cmd = Command::new(&spec.exe);
        if let Some(pdf) = &spec.pdf {
            cmd.arg(pdf);
        }
        for (k, v) in &spec.env {
            cmd.env(k, v);
        }
        cmd.stderr(Stdio::from(stderr));
        cmd.stdout(Stdio::null());
        cmd.stdin(Stdio::null());

        let child = cmd
            .spawn()
            .map_err(|e| Error::new(format!("cannot start {}: {e}", spec.exe.display())))?;
        let pid = child.id();

        let mut session = Self {
            child,
            pid,
            stderr_path: spec.stderr_path.clone(),
            window: None,
            trace_prefix: trace_prefix.to_owned(),
        };

        // Poll for the window rather than sleeping a fixed time. A fixed sleep
        // is either too short on a slow machine (a SKIP that looks like a
        // crash) or wasted on a fast one, and the harness runs this per check.
        let deadline = Instant::now() + spec.window_timeout;
        while Instant::now() < deadline {
            if let Some(status) = session.child.try_wait()? {
                return Err(Error::new(format!(
                    "the application exited with {status} before showing a window. Its \
                     stderr is at {}.",
                    session.stderr_path.display()
                )));
            }
            // A window is only accepted once it has a REAL client area.
            //
            // Found the expensive way, by this harness, against the old GUI: a
            // winit window is created, becomes `IsWindowVisible`, and is
            // enumerable **while its client rect is still 0x0**. Accepting it
            // there produced a window frame whose centre was the window's own
            // top-left corner, so the layout-probe click landed on the desktop
            // behind the application, the application received no input at all,
            // and the check reported "this build does not trace its canvas
            // layout" — a confident, wrong diagnosis of the program under test
            // caused entirely by the harness measuring too early.
            //
            // That is the failure class this whole crate is about, committed by
            // the crate itself, so the fix is a precondition rather than a
            // longer sleep: keep polling until the client area is big enough to
            // be a real application window.
            if let Some(w) = sys::find_window_for_pid(pid)
                && let Ok(frame) = sys::window_frame(w)
                && frame.client_size.0 >= MIN_CLIENT_PX
                && frame.client_size.1 >= MIN_CLIENT_PX
            {
                session.window = Some(w);
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }

        if session.window.is_none() {
            return Err(Error::new(format!(
                "no window appeared for pid {pid} within {:?}. On a platform that cannot \
                 enumerate windows this is always the outcome, and the check is correctly \
                 reported as SKIPPED rather than failed.",
                spec.window_timeout
            )));
        }
        Ok(session)
    }

    /// The window handle.
    #[must_use]
    pub fn window(&self) -> Option<WindowHandle> {
        self.window
    }

    /// Measure the window's client area and DPI scale, now.
    ///
    /// Re-measured on demand rather than cached: the window can be moved or
    /// resized between one assertion and the next, and a cached frame would
    /// convert against a geometry that no longer exists — which produces
    /// clicks that land near the target, the hardest failure to diagnose.
    pub fn frame(&self) -> Result<WindowFrame> {
        let w = self
            .window
            .ok_or_else(|| Error::new("the session has no window"))?;
        sys::window_frame(w)
    }

    /// Bring the window to the front.
    pub fn raise(&self) {
        if let Some(w) = self.window {
            sys::raise_window(w);
        }
    }

    /// Maximise the window, so a ribbon control past the fold is on screen
    /// rather than in the overflow menu.
    ///
    /// # ★ Call this before looking for a control the tab lists LAST
    ///
    /// A ribbon overflows when it is wider than its window, and a control in
    /// the overflow **stops publishing a rect** — which a check cannot tell
    /// apart from a control that does not exist. `settings_theme` found this
    /// the hard way: it asked the File tab for `ribbon.item.file.settings`, was
    /// handed ten controls ending at `file.print`, and would have reported a
    /// shipped feature as missing.
    ///
    /// It is opt-in per check rather than done on every launch, because a
    /// maximised window is a **different layout**, and several checks measure
    /// things — the canvas rect, the find bar's placement, the page strip — for
    /// which the size is part of the subject. Making it universal would change
    /// what those are testing without changing a line of them.
    ///
    /// A no-op on platforms with no window control, exactly as [`Self::raise`]
    /// is: a check that cannot maximise still runs, against whatever size the
    /// window opened at.
    pub fn maximize(&self) {
        if let Some(w) = self.window {
            sys::maximize_window(w);
        }
    }

    /// Read and parse everything the application has written so far.
    ///
    /// Safe to call while it is still running: the trace goes to stderr
    /// unbuffered, one line per event, and reading a file another process has
    /// open for writing is permitted on Windows with the share mode Rust's
    /// `File::open` requests.
    pub fn trace(&self) -> Result<Trace> {
        Trace::read(&self.stderr_path, &self.trace_prefix)
    }

    /// Where the trace file is, for a failure report to point at.
    #[must_use]
    pub fn trace_path(&self) -> &Path {
        &self.stderr_path
    }

    /// The process id, for messages.
    #[must_use]
    pub fn pid(&self) -> u32 {
        self.pid
    }

    /// Give the application `n` frames' worth of time to settle.
    ///
    /// Named in frames rather than milliseconds because that is the unit the
    /// thing being waited for is measured in: a raster rebuild, a layout pass,
    /// a provider swap. At 60 Hz a frame is about 17 ms; the extra margin is
    /// for the frames that are not.
    pub fn settle(&self, frames: u32) {
        std::thread::sleep(Duration::from_millis(u64::from(frames) * 25));
    }
}

impl Drop for Session {
    /// Kill the child on every path — normal return, early error, or panic.
    ///
    /// See the module docs: the alternative left an invisible window
    /// consuming the operator's pointer input, twice in one session.
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Is the binary older than the sources? Returns the complaint, or `None`.
///
/// Deliberately a *complaint string* rather than a bool: the message has to
/// carry both timestamps and the rebuild command, because whoever sees it is
/// about to spend an hour diagnosing a feature that was never compiled.
fn staleness_complaint(exe: &Path, source_root: &Path) -> Option<String> {
    let exe_time = std::fs::metadata(exe).ok()?.modified().ok()?;
    let mut newest: Option<(PathBuf, std::time::SystemTime)> = None;
    let mut stack = vec![source_root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // `target/` is build output; its timestamps are always newer
                // than the binary and would make this gate fire on every run.
                if path.file_name().is_some_and(|n| n == "target") {
                    continue;
                }
                stack.push(path);
                continue;
            }
            let is_source = path.extension().is_some_and(|e| e == "rs" || e == "toml");
            if !is_source {
                continue;
            }
            if let Ok(t) = entry.metadata().and_then(|m| m.modified())
                && newest.as_ref().is_none_or(|(_, best)| t > *best)
            {
                newest = Some((path, t));
            }
        }
    }

    let (path, t) = newest?;
    if t <= exe_time {
        return None;
    }
    Some(format!(
        "STALE BINARY — refusing to run.\n  binary : {}\n           built {:?}\n  newest : {}\n \
          edited {:?}\n\nThe traces you are about to collect would describe code that is NOT \
         the code you just wrote, and a missing trace looks exactly like a broken feature.\n\n \
          cargo build --release\n\nPass --allow-stale only if you intend to drive the older \
         build.",
        exe.display(),
        exe_time,
        path.display(),
        t
    ))
}
