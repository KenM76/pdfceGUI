//! The named checks — the suite this harness exists to run.
//!
//! ## The three, and what each is for
//!
//! | Check | Defect | Oracle |
//! |---|---|---|
//! | [`delete_key`] | **D1** — Delete stops working after the first canvas click | the trace |
//! | [`ribbon_captions`] | group captions rendering illegibly, or not at all | the pixels |
//! | [`settings_headings`] | **D2** — section headings near-white on light grey | the pixels |
//!
//! These are the three `GUI_ROADMAP.md` names as "the smallest useful set",
//! and the three `PROJECT_PLAN.md` stage S1 makes the gate on the harness
//! itself. They are not a sample of what could be checked; they are the
//! specific defects that shipped past a green suite, turned into the tests
//! that would have caught them.
//!
//! ## What has been added since, and on what principle
//!
//! The suite is no longer three. [`all`] is the list; the additions are not a
//! drift away from the founding three but the same rule applied to each new
//! surface as it landed — [`qat_icons`] for the icon painter that was never
//! passed to the ribbon, [`find_bar`] for the chord that was in the keymap and
//! bound to nothing, [`markup_rectangle`] for a ribbon click whose whole
//! four-link chain had unit tests and had never been performed,
//! [`measure_linear`] for `SALVAGE.md`'s step 5 — *"assert it in `ui-verify`
//! before calling it done; a green unit test is the floor"* — and
//! [`read_mode`] for a mode gate whose one untested link is the one a refactor
//! breaks silently.
//!
//! ★ [`text_markup`] is the newest and adds a direction the suite did not have:
//! **it asserts that a control is correctly DISABLED** before asserting that it
//! works. Every check above it drives a control that should act; this one first
//! clicks a control that should not, and reads the *absence* of
//! `ribbon-command-invoked` as the evidence — which is admissible under rule 4
//! below precisely because the same control is then shown to invoke, in the same
//! run, once its operand exists.
//!
//! [`driving`] is not a check. It holds the moves the three ribbon-driving
//! checks share; its header carries the argument for why it exists and why
//! `markup_rectangle` deliberately keeps its own copies.
//!
//! The principle each satisfies, and the one to hold a proposed check to:
//! **it must fail against a build where the wiring is absent, and the wiring
//! must be something no unit test in the workspace can observe.** Every check
//! here has been run against such a build and seen to fail; that is what
//! `PROJECT_PLAN.md` §4 stage S1's acceptance criterion below asks for, and it
//! is not optional.
//!
//! ## The acceptance criterion for the harness
//!
//! > The three assertions **fail** against the old GUI (proving they detect
//! > the real defects) and **pass** against the new one.
//! > — `PROJECT_PLAN.md` §4, stage S1
//!
//! That is the reason [`crate::profile::PDFCE_LEGACY`] exists. A check suite
//! that has only ever been seen to pass is not evidence of anything: it is
//! indistinguishable from a suite that cannot fail. This is the same argument
//! that put a `--self-test` in `tools/gates/check-ui-strings.sh`, and it comes
//! from the same recorded incident — a deliberately planted violation that a
//! gate failed to detect, briefly making it look as though the fix had
//! produced a gate that could only pass.
//!
//! ## Writing a new check
//!
//! Four rules, each of which exists because breaking it produced a real
//! problem in this codebase:
//!
//! 1. **Say what you are about to do, in a note, before you do it.** The notes
//!    are what make a SKIP diagnosable and a PASS believable.
//! 2. **Only ever write [`crate::coords::DocPoint`] or
//!    [`crate::geom::FracRect`] literals.** Never a screen coordinate, never a
//!    window coordinate. See [`crate::coords`].
//! 3. **Establish the precondition explicitly, and SKIP on it.** "The click
//!    selected something" must be *asserted* before "Delete removed it" can be
//!    a failure rather than a mystery.
//! 4. **Never treat an absence as evidence unless you have shown the thing
//!    that would have produced it was working.** The distinction is drawn in
//!    [`crate::report`] and applied in [`delete_key`].
//! 5. **A SKIP reason names the component that is actually blocked, and gets
//!    re-audited whenever the application gains a capability.** This one was
//!    earned at S2. `ribbon_group_captions_legible` spent a stage reporting
//!    *"the trace declared no `ui-rect` regions"* about a binary that declares
//!    three of them on every frame — because nothing in this crate parsed the
//!    event, so the reason described the harness's own blindness as though it
//!    were the application's silence. A reader following it would have gone to
//!    `diag.rs`, which was finished, and found no defect to fix.
//!
//!    The rule that prevents a repeat is mechanical: **a reason may only
//!    assert what the check actually looked at.** [`legibility::resolve_set`]
//!    takes the trace evidence as an argument and builds its reason from it,
//!    with a distinct sentence for "nothing was consulted", "the application
//!    said nothing" and "the application said these things and none of them is
//!    what I need" — because those three send a reader to three different
//!    files.
//!
//! ## Where a check's evidence comes from, in preference order
//!
//! Both apply to every new check, and both say the same thing in two domains:
//! **prefer the evidence the application produced this run.**
//!
//! | Domain | Preferred | Fallback |
//! |---|---|---|
//! | *Where* to measure | a `ui-rect` the application declared this frame | a calibrated fraction in [`crate::profile`] |
//! | *Whether* state changed | a count of the thing itself (`objects n=`) | the event for the verb that should have changed it |
//!
//! The first pair is argued in [`legibility`]; the second in [`delete_key`].
//! In both cases the fallback is kept, because a dated screenshot cannot
//! declare its regions and the old binary cannot count its objects — and in
//! both cases the check says in its own output which one it used.

pub mod delete_key;
pub mod driving;
pub mod find_bar;
pub mod legibility;
pub mod markup_rectangle;
/// ★ The three Phase 6 markup kinds that are **not drag-shaped** — Freehand,
/// Polyline and Polygon — and the one control in this application whose
/// availability is decided by a gesture in progress rather than by the document.
/// It carries the only measurement of the ink simplification taken against a real
/// pointer, and the only falsifier in the suite that needs a control to be
/// **greyed at a specific moment mid-gesture**. Its header carries the argument.
pub mod markup_shapes;

/// ★ Markup ▸ Style — a ribbon group whose one item the manifest declared at S2
/// and no renderer ever drew, so it shipped as a caption over an empty band.
///
/// A **third** shape of invisible wiring, and the quietest: the manifest test
/// asserted the item was *declared* and passed correctly, and the reachability
/// check could not see it at all because a `Custom` item carries no command id.
pub mod markup_style;
pub mod measure_linear;
/// ★ File ▸ New — the first command that makes a document out of **compiled-in
/// bytes** rather than out of a file the operator named, and the only check in
/// the suite whose subject is a page that is *supposed* to be blank. That is
/// what makes it worth having: a blank page and a page that failed to
/// rasterize are the same screenshot, so it reads the canvas's own `drawn=`
/// count instead of a pixel. Its header carries the argument and the
/// falsifying phase.
pub mod new_document;
/// `ocr_recognises_a_page_and_writes_a_new_file` — the whole Recognise-text
/// chain against a genuinely image-only document, ending in the one assertion
/// no unit test can make: **the file that was opened is byte-identical
/// afterwards.**
pub mod ocr;
/// ★ The **Pages tab**, all of which did nothing: six verbs registered, drawn,
/// offered by a context menu and four of them bound to chords, with no dispatch
/// arm between them. The only check in the suite whose subject is a
/// **structural** change to a document rather than a mark drawn on it, and
/// therefore the only one that can assert the thing a page delete uniquely
/// breaks — that the shell's page vector, its rasters and its two selections
/// stop describing a document that no longer exists. Its header carries the
/// argument and the three falsifying phases.
pub mod page_ops;

/// ★ `file.print` — the dialog that told every operator this build could not
/// print, on a machine with twelve printers, in a build that had the printing
/// crate linked into it.
///
/// A new shape of the founding failure and the reason this module exists: the
/// adapter's own unit test asserted that all four of its calls **refused**,
/// which was correct while `pdfce-print` was unlinked and became a lock
/// holding the defect in place the moment the manifest line landed. A green
/// suite defended the absence of the feature. See the module header.
pub mod print_dialog;
pub mod qat_icons;
pub mod read_mode;

/// ★ `tools.render_diagnostics` — the inert control whose data was already
/// being computed. `shell::commands::reach` called it *"the least defensible
/// kind — the work behind it is done"*: the renderer has produced the report
/// since S0, and what was missing was a `match` arm and a window.
pub mod render_diagnostics;

/// ★ `view.read_mode` — the command with a control, a glyph, a group, `Ctrl+H`
/// and a line in the shortcuts reference, and **no dispatch arm** for the whole
/// life of the project. Its whole behaviour is one `if` in the frame
/// composition, which every unit test in the workspace is blind to.
///
/// Named `read_mode_chrome` rather than `read_mode` because that name is
/// already taken by the check one line up, and the two are about genuinely
/// different things: that one is `mode.read`'s **capability** gate (a click in
/// Read must not select), this one is `view.read_mode`'s **chrome** toggle (the
/// ribbon and the docks stop being drawn). `app::window` §1 carries the
/// argument for why those are two commands rather than a duplicate.
pub mod read_mode_chrome;
/// ★★ **Redaction** — the one operation in this program that cannot be undone,
/// and the only check in the suite whose verdict is a **byte scan of a file on
/// disk** rather than a trace field or a pixel. The application's own absence
/// proof reports `verified=true` from inside the process that performed the
/// removal; this asks the same question from outside it, three times, over two
/// strings, in two processes — and it says which of the three answers is the
/// verdict and which two exist to stop the verdict passing vacuously. Its
/// header carries the falsification table.
pub mod redaction;
pub mod ribbon_captions;
/// ★ `file.save_copy` — the command that was registered, drawn, on the
/// quick-access toolbar and bound to `Ctrl+S` with **no dispatch arm**, so
/// nothing this shell could author could reach a disk. The only check in the
/// suite that spans **two processes**: it authors an annotation with a real
/// drag, saves a copy, and then re-opens the saved file in a fresh binary to ask
/// whether the annotation is in it. Its header carries the three falsifying
/// phases and the different wrong build each one catches.
pub mod save_copy;
pub mod settings_headings;

/// ★ `DEFECTS.md` **D10**'s second half — three themes shipped and nothing an
/// operator could press chose one. Proved the only way a theme can be proved:
/// two captures of one window, before and after the click.
pub mod settings_theme;
/// Marking a text selection — underline, strikeout, squiggly. The first
/// commands in this shell whose operand is **not the pointer**, and therefore
/// the first check that asserts a control is *correctly disabled* as well as
/// that it works. Its header carries the argument.
/// **The text-editing round trip** — `DEFECTS.md` D4b, driven: a chord arms the
/// caret tool, a click resolves a run, a commit plans the follower disposition,
/// a save writes it, and a second process reads it back. Its verdict is a byte
/// scan for an operator the operator did NOT touch.
pub mod text_edit;
pub mod text_markup;
/// The canvas text-selection sweep: the one feature whose entire behaviour is a
/// drag and whose entire feedback is a translucent wash, so a screenshot cannot
/// tell it from a page with nothing selected. Its header carries the argument.
pub mod text_selection;
/// ★ The **text tool** in Edit, and the `RIBBON_IA.md` P3 tension it closes. The
/// only check in the suite that observes one control **dead and then live in the
/// same mode in the same run**, and the only one whose subject changes nothing an
/// operator can see except the mouse pointer — which a window capture does not
/// carry at all. Its header carries the argument.
pub mod text_tool;
/// ★ `edit.undo` and `edit.redo` — the pair that was registered, drawn on the
/// quick-access toolbar in **every** mode and bound to three chords with **no
/// dispatch arm**, so an operator could author dimensions, seven markup kinds,
/// text marks and form fills and take none of it back. The only check in the
/// suite that asserts a document change was **un-made**, and the only one whose
/// oracles include two *invalidation* signals — a fresh `objects` line and a
/// fresh `render-spawn` — because the build it exists to catch is one whose
/// every count is already correct. Its header carries the argument.
pub mod undo_redo;

use std::path::{Path, PathBuf};

use crate::coords::DocPoint;
use crate::profile::Profile;
use crate::report::CheckReport;

/// Everything a check needs to know about this run.
#[derive(Clone, Debug)]
pub struct CheckContext {
    /// The target binary's vocabulary and regions.
    pub profile: &'static Profile,
    /// The binary to drive. `None` means the checks that drive one SKIP.
    pub exe: Option<PathBuf>,
    /// The document to open.
    pub pdf: Option<PathBuf>,
    /// An already-captured image to assert against instead of driving the
    /// application — the offline mode for pixel checks. Its purpose is
    /// falsification against a dated artefact; see
    /// [`crate::profile::Calibration`].
    pub image: Option<PathBuf>,
    /// Where screenshots and trace copies are written.
    pub out_dir: PathBuf,
    /// WCAG contrast floor. Defaults to [`crate::pixels::AA_LARGE`].
    pub contrast_threshold: f64,
    /// Whether the harness may take the operator's pointer and keyboard.
    /// `false` makes every driving check SKIP — never pass.
    pub allow_input: bool,
    /// Drive a binary older than its sources.
    pub allow_stale: bool,
    /// Source tree the staleness gate compares against.
    pub source_root: Option<PathBuf>,
    /// Explicit page size, when the fixture's `/MediaBox` cannot be read.
    pub page_size: Option<(f64, f64)>,
    /// The document point a driving check aims at.
    ///
    /// **There is deliberately no default.** A default would be a guess about
    /// where the fixture keeps an object, and a click on empty page is
    /// symptom-identical to a broken hit test — the confusion that produced a
    /// filed-then-retracted defect in this codebase. Absent, the driving
    /// checks SKIP and say what to pass.
    pub target: Option<DocPoint>,
}

impl CheckContext {
    /// A path under the run's output directory.
    #[must_use]
    pub fn out(&self, name: &str) -> PathBuf {
        self.out_dir.join(name)
    }

    /// The exe to use: the explicit one, or the profile's default if it is
    /// actually there.
    ///
    /// A default that does not exist is `None` rather than a path, so the SKIP
    /// reason says "no binary" once rather than describing a path the caller
    /// never chose.
    #[must_use]
    pub fn resolve_exe(&self) -> Option<PathBuf> {
        if let Some(e) = &self.exe {
            return Some(e.clone());
        }
        let default = Path::new(self.profile.default_exe);
        default.is_file().then(|| default.to_path_buf())
    }
}

/// One check.
pub trait Check {
    /// The name `--check` accepts.
    fn name(&self) -> &'static str;
    /// Which defect it detects, in one line, for the report.
    fn defect(&self) -> &'static str;
    /// Run it. A check never panics and never returns an error: every outcome,
    /// including "I could not start", is a [`CheckReport`].
    fn run(&self, ctx: &CheckContext) -> CheckReport;
}

/// Every check, in the order the suite runs them.
#[must_use]
pub fn all() -> Vec<Box<dyn Check>> {
    vec![
        Box::new(delete_key::DeleteKeyAfterCanvasClick),
        Box::new(ribbon_captions::RibbonGroupCaptionsLegible),
        // Reads the trace only — no window is raised and no capture is taken,
        // so it costs nothing and cannot take the operator's focus. Placed
        // after the captions check because both launch, and a reader
        // comparing two ribbon verdicts wants them adjacent.
        Box::new(qat_icons::QatControlsAreIconOnly),
        // ★ The first *driving* check, and it goes first among them on
        // purpose: it is the cheapest — two clicks on one always-enabled
        // control, no canvas gesture, no keystroke, no capture — and it is the
        // one whose failure most changes what a later failure means. Every
        // check below assumes a document is on screen; this one is the only
        // one that makes a document rather than being handed one, so if the
        // ribbon-click channel is broken it says so here, in seconds, instead
        // of at the end of a canvas drag.
        Box::new(new_document::NewDocumentMakesAPage),
        // Clicks and captures, so it takes the desktop — but only with the
        // mouse, and only for a few seconds. Placed after the three ribbon
        // chrome checks because it depends on the same rects they read and a
        // reader comparing ribbon verdicts wants them together; placed before
        // the two typing checks because a run that fails here should fail
        // before paying for a keystroke that may never arrive.
        Box::new(markup_rectangle::MarkupRectangleArmsFromTheRibbon),
        // Beside it, because it is the same shape of check on the same
        // surface — a ribbon control that arms a canvas tool — and a reader
        // comparing the two verdicts wants them adjacent. It goes second
        // because it is the longer of the two: it clicks the page as well as
        // the ribbon, and a snap candidate that needs confirming costs it an
        // extra click.
        // Directly after `markup_rectangle`, because it is the same surface and
        // the longer of the two: it arms three tools, clicks out two runs and
        // drives one drag. A run where the four-link chain itself is broken
        // should report that as `markup_rectangle`'s failure first — this one
        // would fail for the same reason with three more candidate causes in
        // front of it.
        Box::new(markup_shapes::MarkupFreehandAndVertexKinds),
        // ★ Directly after the markup checks, and for the same dependency
        // reason: this one is about the pen those gestures author WITH, so a
        // run in which the ribbon-click channel is broken should report that as
        // `markup_rectangle`'s failure first.
        Box::new(markup_style::MarkupStyleGroupIsDrawn),
        Box::new(measure_linear::MeasureLinearPlacesADimension),
        // ★ Directly after the markup checks, and the order is a **dependency**
        // rather than a preference: this one begins by arming Rectangle and
        // dragging, which is `markup_rectangle`'s and `markup_shapes`' whole
        // subject. A run in which the four-link arm chain is broken should
        // report that as their failure and this one's second — here the same
        // symptom has the entire save path stacked behind it, and a reader who
        // has already seen Rectangle fail knows to ignore the rest.
        //
        // It is the most expensive check in the suite by some way: it launches
        // the binary **twice**, because the round trip it exists to prove is
        // that a second process can read what the first one wrote. Placed
        // before the two typing-dependent checks all the same, because it is
        // the only one whose subject is a file on disk and a run that cannot
        // save should say so before it spends anything on a keystroke that may
        // never arrive.
        Box::new(save_copy::SaveCopyRoundTrip),
        // ★ Directly after it, and the order is a **dependency** rather than a
        // preference: this check ends by saving a copy and re-opening it, so a
        // run in which `file.save_copy` itself is broken should report that as
        // `save_copy_round_trip`'s failure and this one's second. A reader who
        // has already seen the save fail knows to ignore everything below
        // phase E here.
        //
        // It is the second most expensive check in the suite, and for the same
        // reason: it launches the binary **twice**, because a page count read
        // in the process that deleted the page is a count the code under test
        // wrote about itself.
        Box::new(page_ops::PageOpsRoundTrip),
        // ★ Two ribbon clicks and one trace line — cheap, no capture, no
        // canvas gesture, no keystroke — so its position is chosen for what a
        // reader wants adjacent rather than for cost.
        //
        // It sits here, among the checks that drive a real document, because
        // its precondition is one: `file.print` is gated on `doc.open`. It
        // must NOT move up among the chrome checks, which run without a
        // fixture.
        //
        // ★ It never presses the commit button, and no future edit may make it
        // do so. That button is the one control in the application that
        // consumes paper and cannot be undone; a harness that can start a
        // print job will eventually start one by accident. The module header
        // states what that costs and why the cost is worth paying.
        Box::new(print_dialog::PrintDialogReachesTheSpooler),
        // ★ Beside it because it is the same shape — two ribbon clicks into a
        // dialog — and because both are checks whose subject is a control that
        // was drawn and did nothing.
        //
        // It launches with NO fixture, deliberately: `file.settings` is
        // application-scoped and must work with nothing open. That also makes
        // it the cheapest driving check in the suite, so a run whose ribbon
        // channel is broken says so here without paying for a render.
        Box::new(settings_theme::SettingsThemeTakesEffect),
        // ★ Directly after it, and for the same dependency reason it sits
        // after the markup checks: this one also begins by arming Rectangle and
        // dragging, so a run in which the four-link arm chain is broken should
        // report that as `markup_rectangle`'s failure and this one's last.
        //
        // Placed AFTER `save_copy` rather than before it, although it is the
        // cheaper of the two (one process, no file on disk): a shell that
        // cannot write what an operator authored is a worse finding than one
        // that cannot take it back, and a run is likelier to be read from the
        // top than from the bottom.
        Box::new(undo_redo::UndoRedoRoundTrip),
        // ★ Third of the two-process checks, and placed here for the same
        // dependency reason the two above it are: it ends by writing a file and
        // re-opening it, so a run in which writing itself is broken should
        // report that as `save_copy_round_trip`'s failure and this one's
        // second.
        //
        // It is the most expensive check in the suite by a small margin —
        // two launches, eight clicks, and a full rewrite of a document
        // performed synchronously inside one of them — and the most valuable
        // per second spent, because its subject is the only irreversible
        // operation the program has. Its fixture is **generated**, so unlike
        // every other driving check it does not consult `--pdf` and cannot be
        // aimed at a document that lacks the strings it scans for.
        Box::new(redaction::RedactionRemovesAndProvesIt),
        // ★ Directly after `redaction`, and before the two selection checks,
        // because it is the second most expensive check in the suite — it
        // launches the binary twice, for the same reason `save_copy` does — and
        // because its subject is the one this project exists for. A run in which
        // `save_copy` failed should be read first: every link from Ctrl+S
        // onwards is that check's, and this one has the whole text-edit path
        // stacked in front of them.
        Box::new(text_edit::TextEditPinsAnAlignedTail),
        // After both, because it is the only driving check that does not touch
        // the ribbon band at all — it clicks mode segments and the page — and
        // because it is the slowest: it searches for a point with content
        // under it, and every candidate costs four clicks.
        Box::new(read_mode::ReadModeRefusesCanvasEdits),
        Box::new(text_selection::TextSelectionSweepsAndCopies),
        // Directly after it, and the order is a dependency rather than a
        // preference: this one *begins* by making a text selection, so a run
        // where the sweep itself is broken should report that as the sweep's
        // failure first and this one's SKIP second. It is also the longer of
        // the two — three ribbon clicks and a drag, one of which authors an
        // annotation into the open document (never onto disk: nothing here
        // saves).
        Box::new(text_markup::TextMarkupMarksASelection),
        // Directly after it, and again the order is a dependency rather than a
        // preference: this one does everything `text_markup` does and then some,
        // in a different mode and behind a tool that has to arm first. A run
        // where the marking path itself is broken should report that as
        // `text_markup`'s failure, and this one's second — because here the same
        // symptom has one more candidate cause (the tool), and a reader who has
        // already seen Review fail knows to ignore it.
        //
        // It is the longest driving check in the suite: five ribbon clicks
        // across three tabs, two drags, and one annotation authored into the
        // open document (never onto disk; nothing here saves).
        // ★ Last of the driving checks, and the slowest: it runs a real
        // recognition, which is a second in a release build. Placed after the
        // cheap ones so a run that is going to fail on something structural
        // fails before spending it.
        Box::new(ocr::OcrRecognisesAPageAndWritesANewFile),
        Box::new(text_tool::TextToolSelectsAndMarksInEdit),
        // Last, because it is the only check that TYPES. Everything above
        // either reads a trace or captures a window; this one presses a
        // chord, types a needle and presses Enter into a real foreground
        // window, so it costs the operator their focus for a few seconds.
        // A run that fails earlier should fail before paying that.
        // Cheap and non-destructive: two ribbon clicks, no canvas gesture, no
        // keystroke, and a window that changes nothing. Placed here rather than
        // among the first driving checks only because it depends on a raster
        // having landed, and everything above has already waited for one.
        Box::new(render_diagnostics::RenderDiagnosticsOpensItsReport),
        Box::new(find_bar::FindOpensAndFinds),
        // ★ Second to last among the driving checks, and the placement is a
        // property of what it does rather than of what it costs: it is the only
        // check that **cannot put the application back**. Read mode's exit is
        // `Ctrl+H` and this machine cannot inject keystrokes, so the session
        // ends with the chrome hidden. That harms nothing — every check launches
        // its own process and read mode is per-session by design — but a reader
        // scanning a run for the first failure should not meet a check whose
        // window looks broken in its artefacts before the ones whose windows
        // look ordinary.
        //
        // Cheap otherwise: two ribbon clicks, two captures, no canvas gesture
        // and no keystroke.
        Box::new(read_mode_chrome::ReadModeHidesTheChrome),
        Box::new(settings_headings::SettingsHeadingsLegible),
    ]
}
