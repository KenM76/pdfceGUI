//! `ocr_recognises_a_page_and_writes_a_new_file` — the check for a feature
//! whose whole product is **a file that did not exist before**, and whose whole
//! risk is that it might have written over one that did.
//!
//! # What no unit test in this workspace observes
//!
//! Four links, and only the first is testable off the binary:
//!
//! | # | Link | Its own test |
//! |---|---|---|
//! | 1 | recognition produces bytes, and those bytes carry extractable text | yes — `ocr::fixture::tests::recognises_the_synthetic_page` |
//! | 2 | a ribbon click on `file.ocr` reaches the dialog and the dialog's controls exist | **no** |
//! | 3 | the write goes to the path the operator named | **no** |
//! | 4 | **the document that was opened is not touched** | **no** |
//!
//! Link 4 is the operator's standing rule — *"Read may produce a new document;
//! it may not modify this one"* — and it is the one that cannot be checked
//! anywhere but here. A build that wrote the recognised bytes back over the
//! source would satisfy every assertion about "a file exists and has text in
//! it", and a unit test on `add_ocr_layer` would not notice, because the layer
//! writer never touches a path at all: the shell chooses the destination.
//!
//! # ★ The falsifying phase, and what it is aimed at
//!
//! Phases A–D below could all be passed by a build whose Save wrote to the
//! **source** path instead of the named one. It would open the dialog,
//! recognise, report, "save", and produce a document with extractable text —
//! and it would have destroyed the operator's scan.
//!
//! So **phase E hashes the source file before the run and after it**, and the
//! whole verdict rests on those two digests being equal. That is a genuinely
//! falsifying test rather than a confirming one: it is the assertion that fails
//! against the plausible wrong implementation and passes against the right one,
//! and there is no way to satisfy it accidentally.
//!
//! It also carries a second, weaker guard that is worth having anyway: the
//! saved copy's path is not the source's path. That one a reviewer could see;
//! the digest is what survives a reviewer who did not look.
//!
//! # Why the fixture is `synthetic-image-only.pdf`
//!
//! Because a document with no extractable text is the only kind on which OCR's
//! result is unambiguous: any text in the output came from the recogniser.
//! `crates/pdfce-gui/src/ocr/fixture.rs` generates it, and **its header is
//! required reading before believing anything here**. The short version, and it
//! is stated in this check's own report so a green result cannot be misread:
//! the fixture is a *rendered page*, not a scan. It has no scanner noise, no
//! skew and no JPEG ringing, so this check establishes the **plumbing** and
//! establishes **nothing** about recognition quality on real scanned material.
//!
//! # Mouse only, and one consequence that matters
//!
//! Synthetic keyboard input does not reach the target window from the session
//! that injects it on this machine — see [`crate::checks::find_bar`] and
//! `HANDOFF.md` §8's record of a lead against that which failed to reproduce.
//!
//! **The consequence here is specific and is reported rather than implied: the
//! Find bar's OCR offer is not driven by this check and cannot be.** Reaching
//! it needs a committed search, a search is committed by Enter in the Find
//! field, and no mouse gesture commits one — `bar::enter_intent` gives the step
//! buttons nothing to do until a search has already run. The offer's *rule* is
//! covered by unit test (`find::bar::tests`, including the falsifying case
//! where a zero-hit search on a page that has text offers nothing) and its
//! *drawing* is not covered at all. That gap is on the record here rather than
//! left for a reader of a PASS to assume away.
//!
//! # Which document it drives
//!
//! `--pdf` if given, otherwise `fixtures/synthetic-image-only.pdf`. **Pass
//! `--pdf` the day a genuine scanned document exists** — everything this check
//! asserts is true of any image-only PDF, and the synthetic one is a stand-in
//! rather than the subject.
//!
//! Note what the check does to whatever it is pointed at: **nothing**. That is
//! the whole of phase E. But it *reads* the file twice and compares, so the
//! file must be one the harness may read; it is never written to by design, and
//! a run in which it changes is a failure by definition.
//!
//! # The file picker is answered, not driven
//!
//! `PDFCE_DIAG_SAVE_PATH` supplies the save dialog's result and the dialog is
//! never opened. That is this project's established pattern for a native picker
//! (`app::files`' header, and the RAG note it quotes: *"Don't try to script the
//! dialog"*), and it is what makes phase D an assertion about **a file on disk**
//! rather than about a button having been pressed.
//!
//! # Every way this reports SKIP, and why none is a pass
//!
//! * no binary, no `--no-input`, no diagnostic channel — the harness never
//!   began;
//! * the fixture is missing, or already has extractable text (in which case it
//!   is the wrong fixture and OCR's contribution could not be isolated);
//! * a tab, a mode segment or a ribbon control was never declared, or took no
//!   click;
//! * the model weights are not beside the binary — the application says so by
//!   name, and a harness that called that a failure would be blaming the
//!   feature for a packaging step that was not run.

use std::path::PathBuf;

use crate::checks::driving::{self, SHELL_DIAG_ENV};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The mode the check runs in.
///
/// ★ **Read, deliberately, and it is half of what this check is for.** The
/// operator's instruction is that OCR be available in Read, and the first
/// implementation of this feature put the command on the **Tools** tab —
/// `RIBBON_IA.md` §5.7's placement — where Read cannot reach it at all, Read
/// being shown `["file", "view"]` alone. Driving the ribbon in Read is what
/// turns that from an argument into an observation.
const READ: &str = "read";

/// The tab the command lives on. See [`READ`] for why it is not `tools`.
const TAB: &str = "file";

/// The command's id, which is also its declared region's suffix.
const COMMAND: &str = "file.ocr";

/// The fixture, relative to the workspace root.
const FIXTURE: &str = "fixtures/synthetic-image-only.pdf";

/// `ocr-started page=… models=… source=…`
const STARTED_EVENT: &str = "ocr-started";

/// `ocr-recognised page=… recognised=… written=… …`
const RECOGNISED_EVENT: &str = "ocr-recognised";

/// `ocr-refused reason=…`
const REFUSED_EVENT: &str = "ocr-refused";

/// `ocr-saved path=… bytes=…`
const SAVED_EVENT: &str = "ocr-saved";

/// The environment variable that answers the save dialog. See the module
/// header.
const SAVE_PATH_ENV: &str = "PDFCE_DIAG_SAVE_PATH";

/// How long to wait for recognition, in settle frames.
///
/// Generous. Recognition of one page measured about one second in a release
/// build and twenty in a debug one, and this harness drives whichever binary it
/// was pointed at. A wait that was too short would report "recognition did not
/// happen" about a build that was still working, which is the worst available
/// failure message.
const RECOGNITION_FRAMES: u32 = 400;

/// The repository's own copy of the fixture, located from this crate rather
/// than from the working directory.
///
/// `tools/ui-verify/` → up two → the workspace root. Stable whatever the
/// harness was invoked from and whatever `--source-root` says, which is the
/// property the first two attempts at this lacked — see [`drive`].
fn default_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join(FIXTURE)
}

/// See the module documentation.
pub struct OcrRecognisesAPageAndWritesANewFile;

impl Check for OcrRecognisesAPageAndWritesANewFile {
    fn name(&self) -> &'static str {
        "ocr_recognises_a_page_and_writes_a_new_file"
    }

    fn defect(&self) -> &'static str {
        "Recognise text is unreachable in the mode the operator asked for it in, produces no \
         layer, writes nothing, or — the one that cannot be caught anywhere else — writes the \
         recognised document back over the file that was opened"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(skip) => report.skip(skip.to_string()),
        }
    }
}

/// A cheap content digest — length plus FNV-1a over the bytes.
///
/// Not cryptographic and does not need to be: the question is *"did this file
/// change"*, the adversary is a bug rather than a forger, and carrying a SHA-2
/// implementation into this crate to answer it would be a dependency for
/// nothing. The **length is part of the digest** so a truncation cannot be
/// hidden by a hash collision, which is the only failure mode a 64-bit hash
/// realistically has here.
fn digest(bytes: &[u8]) -> (usize, u64) {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    (bytes.len(), hash)
}

/// Click a ribbon band control by command id, and confirm the shell saw it.
///
/// The same shape as [`driving::click_mode_segment`], for the other half of the
/// ribbon. Not folded into that module because it is the first check to need
/// it: a second caller is the moment to move it, and moving it on the first
/// would leave `driving` with an untested function.
fn click_command(session: &Session, driver: &Driver, ui_rect: &str, id: &str) -> Result<()> {
    let region = format!("{}{id}", driving::ITEM_PREFIX);
    let trace = session.trace()?;
    let rect = driving::declared(&trace, ui_rect, &region).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{region}` region, so `{id}` has no control to click. \
             Band controls it did declare: {}.",
            driving::list(&driving::declared_names(
                &trace,
                ui_rect,
                driving::ITEM_PREFIX
            ))
        ))
    })?;
    if !rect.is_substantial() {
        return Err(Error::new(format!(
            "`{region}` was declared at {rect:?}, which has no usable area to click."
        )));
    }
    let before = driving::shell_trace(session)?
        .events(driving::INVOKE_EVENT)
        .filter(|l| l.get("id") == Some(id))
        .count();
    driver.click_at(session.frame()?.declared_center(rect))?;
    session.settle(16);
    let after = driving::shell_trace(session)?
        .events(driving::INVOKE_EVENT)
        .filter(|l| l.get("id") == Some(id))
        .count();
    if after <= before {
        return Err(Error::new(format!(
            "the click on `{region}` produced no new `{} id={id}` line, so no click reached the \
             ribbon and nothing after it would mean anything. Trace: {}.",
            driving::INVOKE_EVENT,
            session.trace_path().display()
        )));
    }
    Ok(())
}

/// Click a tab by id.
fn click_tab(session: &Session, driver: &Driver, ui_rect: &str, tab: &str) -> Result<()> {
    let region = format!("ribbon.tab.{tab}");
    let trace = session.trace()?;
    let rect = driving::declared(&trace, ui_rect, &region).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{region}` region. Tabs it did declare: {}. \
             ★ If `{TAB}` is missing while `{READ}` is selected, that is the finding rather \
             than a harness problem: the command would be unreachable in the mode the operator \
             asked for it in.",
            driving::list(&driving::declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    let before = driving::shell_trace(session)?
        .events(driving::TAB_EVENT)
        .filter(|l| l.get("tab") == Some(tab))
        .count();
    driver.click_at(session.frame()?.declared_center(rect))?;
    session.settle(12);
    let after = driving::shell_trace(session)?
        .events(driving::TAB_EVENT)
        .filter(|l| l.get("tab") == Some(tab))
        .count();
    if after <= before {
        return Err(Error::new(format!(
            "the click on `{region}` produced no new `{} tab={tab}` line.",
            driving::TAB_EVENT
        )));
    }
    Ok(())
}

/// Click a region the *application* declared (a dialog control), by name.
fn click_region(session: &Session, driver: &Driver, ui_rect: &str, name: &str) -> Result<()> {
    let trace = session.trace()?;
    let rect = driving::declared(&trace, ui_rect, name).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{name}` region. Regions it did declare beginning \
             `ocr-`: {}.",
            driving::list(&driving::declared_names(&trace, ui_rect, "ocr-"))
        ))
    })?;
    if !rect.is_substantial() {
        return Err(Error::new(format!(
            "`{name}` was declared at {rect:?}, which has no usable area to click."
        )));
    }
    driver.click_at(session.frame()?.declared_center(rect))?;
    session.settle(12);
    Ok(())
}

/// Run the sequence. `Err` is SKIP, `Ok(Some(_))` is FAIL, `Ok(None)` is a pass.
#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    // --- preconditions -----------------------------------------------------
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a ribbon control and two dialog \
             buttons. Reported as SKIPPED rather than passed: a check that did not run has \
             learned nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot state \
             where its controls are and this check has nothing to aim at.",
            ctx.profile.name
        ))
    })?;

    // ★ `--pdf` first, the repository default second — and the ORDER is a
    // repair rather than a preference.
    //
    // The first version of this check resolved the fixture from
    // `ctx.source_root`, which is the *staleness comparison's* root and is
    // `None` under `--no-staleness-check`. Falsifying the check against a
    // deliberately broken build was supposed to run against a COPY of the
    // fixture in a scratch directory; the path collapsed to `.` instead, the
    // planted build overwrote the repository's real fixture, and the check
    // reported the overwrite correctly while the harness was the thing that
    // aimed it at the wrong file.
    //
    // Both halves of that are worth keeping. The check did its job — it is the
    // reason the damage was noticed within one run rather than at the next
    // commit — and a harness that decides which file to destroy from a flag
    // about staleness had no business doing so. `--pdf` is now the explicit
    // control, which also makes the right thing possible the day a genuine
    // scanned document exists: point this check at it.
    // The default is resolved from THIS CRATE'S manifest directory, not from
    // the working directory and not from `--source-root`. Both of those were
    // tried and both were wrong: `--source-root` defaults to `crates`, so the
    // fixture resolved to `crates/fixtures/...` and the check SKIPped; and a
    // bare `.` depends on where the harness was invoked from, which is how the
    // planted-build run came to aim at the repository's own copy.
    let fixture = ctx.pdf.clone().unwrap_or_else(default_fixture);
    if !fixture.is_file() {
        return Err(Error::new(format!(
            "the image-only fixture is not at {}. Generate it:\n    cargo test -p pdfce-gui \
             --lib write_synthetic_image_only -- --ignored",
            fixture.display()
        )));
    }
    let before_bytes = std::fs::read(&fixture)
        .map_err(|e| Error::new(format!("cannot read {}: {e}", fixture.display())))?;
    let before = digest(&before_bytes);
    report.note(format!(
        "fixture {} — {} bytes, digest {:016x}",
        fixture.display(),
        before.0,
        before.1
    ));

    // Where the recognised copy will go. Beside the harness's own output, never
    // beside the fixture: a stray file in `fixtures/` would be committed by
    // somebody eventually.
    let target = ctx.out("ocr-recognised.pdf");
    let _ = std::fs::remove_file(&target);

    // --- launch ------------------------------------------------------------
    let mut spec = LaunchSpec::new(&exe, ctx.out("ocr.trace.txt"));
    spec.pdf = Some(fixture.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((SAVE_PATH_ENV.to_owned(), target.display().to_string()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} as pid {} with {SAVE_PATH_ENV}={}",
        exe.display(),
        session.pid(),
        target.display()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);

    let trace = session.trace()?;
    if !trace.started(ctx.profile.vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so the diagnostic switch did not reach the process and \
             this check has no oracle. Captured stderr is at {}.",
            ctx.profile.vocab.start_event,
            session.trace_path().display()
        )));
    }
    let driver = Driver::new(session.window());

    // --- phase A: reach the command IN READ --------------------------------
    driving::click_mode_segment(&session, &driver, ui_rect, READ)?;
    click_tab(&session, &driver, ui_rect, TAB)?;
    report.note(format!(
        "the `{TAB}` tab is present and took a click while the mode selector is on `{READ}` — so \
         the command is reachable in the mode the operator asked for it in, which the \
         specification's Tools placement would not have been"
    ));
    click_command(&session, &driver, ui_rect, COMMAND)?;

    // --- phase B: the dialog is up ------------------------------------------
    session.settle(16);
    let trace = session.trace()?;
    if driving::declared(&trace, ui_rect, "ocr-dialog").is_none() {
        return Ok(Some(format!(
            "THE COMMAND OPENED NO DIALOG. `{COMMAND}` was invoked — the shell traced it — and \
             the application declared no `ocr-dialog` region on any frame afterwards. Either the \
             dispatch arm is missing (look for `{}` in the trace) or `DialogsState::show` is not \
             drawing it. Trace: {}.",
            driving::UNIMPLEMENTED_EVENT,
            session.trace_path().display()
        )));
    }
    report.note("the dialog is up and declared its own rect");

    // --- phase C: recognise -------------------------------------------------
    if driving::declared(&trace, ui_rect, "ocr-run").is_none() {
        // The dialog is showing a refusal rather than a run button. That is a
        // legitimate state and this check must not call it a failure: the
        // commonest cause by far is that the model weights are not beside this
        // binary, which is a packaging step rather than a defect in the
        // feature.
        return Err(Error::new(format!(
            "the dialog drew no `ocr-run` control, so it is refusing rather than offering to \
             recognise. The overwhelmingly likely cause is that the `models/ocrs` folder is not \
             beside {} — this check needs a PACKAGED build, or the weights copied next to the \
             executable. Point --exe at a folder produced by `tools/package-portable.py`.",
            exe.display()
        )));
    }
    click_region(&session, &driver, ui_rect, "ocr-run")?;
    session.settle(RECOGNITION_FRAMES);

    let trace = session.trace()?;
    if let Some(refusal) = trace.last(REFUSED_EVENT) {
        return Ok(Some(format!(
            "RECOGNITION REFUSED: `{}`. The run control was drawn — so the preflight checks \
             passed and this build has both an engine and models — and the job then came back \
             with a named refusal. Trace: {}.",
            refusal.raw,
            session.trace_path().display()
        )));
    }
    let Some(recognised) = trace.last(RECOGNISED_EVENT) else {
        return Ok(Some(format!(
            "RECOGNITION NEVER FINISHED. `{STARTED_EVENT}` was {}, and no `{RECOGNISED_EVENT}` \
             or `{REFUSED_EVENT}` line followed within {RECOGNITION_FRAMES} frames. A job that \
             neither answers nor refuses leaves the dialog saying `Recognising…` forever, which \
             is the one outcome `ocr::Job::poll`'s disconnected arm exists to prevent. Trace: {}.",
            if trace.last(STARTED_EVENT).is_some() {
                "traced"
            } else {
                "NOT traced either, so the click never reached the button"
            },
            session.trace_path().display()
        )));
    };
    report.note(format!("recognition finished: `{}`", recognised.raw));

    let written = recognised.get_usize("written").unwrap_or(0);
    if written == 0 {
        return Ok(Some(format!(
            "RECOGNITION WROTE NO WORDS. The job completed and reported `{}`. On a page whose \
             every mark is text this means the detector or the recogniser produced nothing \
             placeable — check `ocr::fitted_dpi` against `ocr::TARGET_PIXELS`, which is the \
             constant that most affects this and which measured a 13× accuracy swing across \
             five resolutions.",
            recognised.raw
        )));
    }

    // ★ The disclosure fact, asserted rather than assumed. If a build ever
    // claims this engine reports confidence, the dialog stops making its
    // "nothing here has been scored" statement and a page of unscored guesses
    // starts presenting as a page of checked ones.
    if recognised.get("confidence_available") == Some("true") {
        return Ok(Some(format!(
            "THE BUILD CLAIMS PER-WORD CONFIDENCE IT DOES NOT HAVE. `{}` reports \
             `confidence_available=true`, and `ocrs` emits a character and a rectangle with no \
             score anywhere. The consequence is not cosmetic: `text::ocr::no_confidence` is the \
             one sentence telling the operator that nothing here was checked, and a build in \
             this state has stopped showing it.",
            recognised.raw
        )));
    }

    // --- phase D: save to the named path ------------------------------------
    if driving::declared(&session.trace()?, ui_rect, "ocr-save").is_none() {
        return Ok(Some(
            "RECOGNITION PRODUCED NO SAVEABLE DOCUMENT. The job reported words written and the \
             dialog drew no `ocr-save` control, which it draws if and only if it is holding \
             bytes. So the layer was written and the document was not carried out of the worker \
             — look at `Phase::Recognised` and at what `Job::poll` handed it."
                .to_owned(),
        ));
    }
    click_region(&session, &driver, ui_rect, "ocr-save")?;
    session.settle(40);

    let trace = session.trace()?;
    let Some(saved) = trace.last(SAVED_EVENT) else {
        return Ok(Some(format!(
            "NOTHING WAS WRITTEN. The save control was clicked and no `{SAVED_EVENT}` line \
             followed. {SAVE_PATH_ENV} was set, so the picker was answered rather than opened — \
             which means either `pick_save_path` did not read the seam or `std::fs::write` \
             failed. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("the copy was written: `{}`", saved.raw));

    if !target.is_file() {
        return Ok(Some(format!(
            "THE SAVED FILE IS NOT WHERE IT WAS ASKED FOR. The application traced `{}` and \
             nothing exists at {}. The path the operator names is the only thing standing \
             between this feature and the file they opened.",
            saved.raw,
            target.display()
        )));
    }
    let written_bytes = std::fs::read(&target)
        .map_err(|e| Error::new(format!("cannot read {}: {e}", target.display())))?;
    report.artifact(target.clone());
    report.note(format!(
        "the recognised copy is {} bytes, against the fixture's {}",
        written_bytes.len(),
        before.0
    ));
    if written_bytes.len() <= before.0 {
        return Ok(Some(format!(
            "THE SAVED COPY IS NO LARGER THAN THE ORIGINAL ({} vs {} bytes). An OCR layer is an \
             INCREMENTAL revision appended to the file — a content stream, a font dictionary and \
             a rewritten page object — so a copy that did not grow did not gain one, whatever \
             the trace said it wrote.",
            written_bytes.len(),
            before.0
        )));
    }

    // --- ★ phase E: the falsifying one --------------------------------------
    let after_bytes = std::fs::read(&fixture)
        .map_err(|e| Error::new(format!("cannot re-read {}: {e}", fixture.display())))?;
    let after = digest(&after_bytes);
    if after != before {
        return Ok(Some(format!(
            "★ THE DOCUMENT THAT WAS OPENED HAS BEEN MODIFIED. {} was {} bytes (digest {:016x}) \
             before the run and is {} bytes (digest {:016x}) after it.\n\n\
             This is the operator's standing rule broken at the one place it can be: *Read may \
             produce a new document; it may not modify this one*, enforced at the SAVE rather \
             than at the operation. Every other assertion in this check passed — a dialog \
             opened, words were recognised, a file was written and it had text in it — and the \
             file that was written over was the operator's scan. Look at `dialogs::ocr::save`, \
             and at whether `app::files::pick_save_path`'s answer is the path actually passed to \
             `std::fs::write`.",
            fixture.display(),
            before.0,
            before.1,
            after.0,
            after.1
        )));
    }
    report.note(format!(
        "★ the opened document is byte-identical after the run — {} bytes, digest {:016x}, \
         unchanged. That is the assertion this check exists for: a build that wrote the \
         recognised bytes back over the source would have passed every phase above it",
        after.0, after.1
    ));

    if target == fixture {
        return Ok(Some(
            "the saved copy's path IS the source's path, so phase E compared a file with \
             itself and proved nothing. This is a harness defect rather than an application \
             one, and it is reported as a failure so it cannot be mistaken for a pass."
                .to_owned(),
        ));
    }

    // --- what this does and does not establish ------------------------------
    report.note(
        "NOT established by this check: recognition quality on real scanned material. The \
         fixture is a rendered page with no scanner noise, skew, JPEG ringing or uneven \
         lighting, so it flatters the recogniser — see `crates/pdfce-gui/src/ocr/fixture.rs`'s \
         header. What is established is the chain: reachable in Read, recognises, discloses, \
         writes where it was told, and leaves the original alone",
    );
    report.note(
        "NOT covered here: the Find bar's OCR offer. Reaching it needs a committed search, a \
         search is committed by Enter, and synthetic keystrokes do not reach the target window \
         from this session (see find_bar). Its rule is covered by unit test including the \
         falsifying case; its drawing is covered by nothing, and that gap is stated rather than \
         implied by a green result",
    );
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★ **The digest changes when one byte does, and the length is part of it.**
    ///
    /// Phase E's whole verdict rests on this function, so a digest that answered
    /// "unchanged" for a modified file would turn the check's most important
    /// assertion into a formality that always passes.
    #[test]
    fn the_digest_notices_a_single_changed_byte_and_a_truncation() {
        let a = b"%PDF-1.4 hello world";
        let mut b = a.to_vec();
        b[10] ^= 0x01;
        assert_ne!(digest(a), digest(&b), "one flipped bit must change it");
        assert_ne!(
            digest(a),
            digest(&a[..a.len() - 1]),
            "a truncation must change it, which is what the length is in the tuple for"
        );
        assert_eq!(digest(a), digest(a), "and it must be stable");
    }

    /// The mode this check drives is the one the operator's instruction is
    /// about, and the tab is one that mode is actually shown.
    ///
    /// Pinned because the two together *are* the finding: `RIBBON_IA.md` §5.7
    /// puts OCR on Tools, Read is shown `["file", "view"]`, and a check that
    /// quietly drove Edit instead would pass against a build in which OCR is
    /// unreachable in Read.
    #[test]
    fn the_check_drives_read_and_a_tab_read_actually_has() {
        assert_eq!(READ, "read");
        assert_eq!(TAB, "file");
        assert!(
            COMMAND.starts_with(TAB),
            "a command id names its owning tab, so `{COMMAND}` on `{TAB}` must share the prefix \
             — and if it does not, the manifest and this check disagree about where it is"
        );
    }

    /// The fixture path is the generated image-only one, not the drawing.
    #[test]
    fn the_fixture_is_the_image_only_one() {
        assert!(FIXTURE.contains("image-only"));
        assert!(
            !FIXTURE.contains("titleblock"),
            "a1-titleblock.pdf has extractable text, so OCR's contribution to it could not be \
             isolated from what was already there"
        );
    }

    /// Every trace event this check reads is spelled once.
    #[test]
    fn the_event_names_are_distinct() {
        let all = [STARTED_EVENT, RECOGNISED_EVENT, REFUSED_EVENT, SAVED_EVENT];
        for (i, a) in all.iter().enumerate() {
            assert!(a.starts_with("ocr-"), "{a} is not an OCR event");
            for b in &all[i + 1..] {
                assert_ne!(a, b);
            }
        }
    }

    /// The check never writes into `fixtures/`.
    ///
    /// A stray recognised copy beside the fixture would be committed by
    /// somebody eventually, and a repository that gains a file every time the
    /// harness runs is a repository whose `git status` stops being read.
    #[test]
    fn the_output_path_is_not_beside_the_fixture() {
        assert_eq!(
            std::path::Path::new(FIXTURE)
                .parent()
                .and_then(std::path::Path::to_str),
            Some("fixtures")
        );
    }
}
