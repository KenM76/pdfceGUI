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
pub mod find_bar;
pub mod legibility;
pub mod qat_icons;
pub mod ribbon_captions;
pub mod settings_headings;

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
        // Last, because it is the only check that TYPES. Everything above
        // either reads a trace or captures a window; this one presses a
        // chord, types a needle and presses Enter into a real foreground
        // window, so it costs the operator their focus for a few seconds.
        // A run that fails earlier should fail before paying that.
        Box::new(find_bar::FindOpensAndFinds),
        Box::new(settings_headings::SettingsHeadingsLegible),
    ]
}
