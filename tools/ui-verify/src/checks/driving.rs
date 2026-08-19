//! `checks::driving` — the moves that **every check which drives the ribbon**
//! has to make, in one place.
//!
//! # Why this module exists
//!
//! [`crate::checks::markup_rectangle`] was the first check to click a ribbon
//! control, and it had to invent five small things to do it: read the last
//! rect the application declared for a name, list the names it *did* declare
//! for a SKIP reason, re-parse the same captured stderr under the **shell's**
//! line prefix, measure a control's fill out of a capture, and compare two
//! fills. All five are properties of *driving an `egui-shell` ribbon*, not of
//! markup.
//!
//! The second and third such checks — [`crate::checks::measure_linear`] and
//! [`crate::checks::read_mode`] — needed the same five, and a third copy of a
//! function is where copies start to disagree. So they live here, with the
//! reasoning that shaped them carried across rather than summarised.
//!
//! `markup_rectangle` deliberately keeps its own copies. Rewriting a check
//! that is already known to detect its defect, in the same change that adds
//! two new ones, would mean the three checks stopped being independent
//! evidence of each other at exactly the moment the harness grew. This module
//! is a *widening*, not a refactor; when someone next has cause to touch
//! `markup_rectangle` for its own sake, folding it onto these is a one-line
//! change per helper.
//!
//! # The two diagnostic channels, and why a check reads both
//!
//! One captured stderr file, two vocabularies:
//!
//! | Channel | Switch | Prefix | Says |
//! |---|---|---|---|
//! | the shell | [`SHELL_DIAG_ENV`] | [`SHELL_TRACE_PREFIX`] | a segment/tab/control took a click |
//! | the application | the profile's `diag_env` | the profile's `trace_prefix` | what the application did about it |
//!
//! The split is not an accident of this build. `egui_shell::verify`'s header
//! explains that one environment variable name lets a harness arm tracing on
//! *any* `egui-shell` application without first discovering its name, and the
//! prefix is the application's so two crates' lines never blur together. The
//! consequence for a check is the thing that makes a failure attributable: a
//! present `ribbon-command-invoked` with an absent application-side effect
//! names the application's dispatch and nothing else, and an absent
//! `ribbon-command-invoked` means no click was ever delivered — which is a
//! SKIP, because a check that could not deliver a click has learned nothing.

use crate::coords::WindowFrame;
use crate::error::{Error, Result};
use crate::geom::LRect;
use crate::image::{Image, Rgb};
use crate::input::Driver;
use crate::launch::Session;
use crate::trace::Trace;

/// The shell's own diagnostic switch, and its value.
///
/// See the module header. `pdfce-gui` does not call
/// `egui_shell::verify::set_prefix`, so the shell's lines arrive under the
/// crate's default prefix, [`SHELL_TRACE_PREFIX`].
pub const SHELL_DIAG_ENV: (&str, &str) = ("EGUI_SHELL_DIAG", "1");

/// The line prefix `egui-shell` uses when the application has not set one.
pub const SHELL_TRACE_PREFIX: &str = "egui-shell-diag";

/// `ribbon-mode-selected mode=…` — the shell reporting a mode-segment click.
///
/// Emitted on **every** click of a segment, including a click on the segment
/// that is already selected (`ribbon::mode_selector` sets `chosen` from the
/// click and filters for "did this change anything" only in its *return
/// value*, after the line is written). That is what makes it usable as the
/// input-channel proof for a mode a check merely wants to be *in*, rather than
/// only for a mode it is switching *to*.
pub const MODE_EVENT: &str = "ribbon-mode-selected";

/// `ribbon-tab-activated tab=…` — the shell reporting a tab click.
pub const TAB_EVENT: &str = "ribbon-tab-activated";

/// `ribbon-command-invoked id=… handler=…` — the shell reporting that a band
/// control was clicked and its token handed to the application.
pub const INVOKE_EVENT: &str = "ribbon-command-invoked";

/// `command-unimplemented id=…` — `app/dispatch.rs`'s fall-through arm.
///
/// Read only to *improve a failure message*: its presence alongside a missing
/// application-side effect is the signature of a dispatch that received the
/// command and had no arm for it, which is a different fix from a dispatch
/// that never received it at all.
pub const UNIMPLEMENTED_EVENT: &str = "command-unimplemented";

/// The namespace one ribbon command control's rect is published under.
pub const ITEM_PREFIX: &str = "ribbon.item.";

/// The last rect the application declared under `name`, if any.
///
/// **Last wins.** A region is re-declared whenever it moves, and an early
/// frame can carry a rect from before the layout settled — the find bar's
/// one-frame misplacement was exactly that, and taking the first occurrence
/// would aim a check's clicks at it.
#[must_use]
pub fn declared(trace: &Trace, ui_rect: &str, name: &str) -> Option<LRect> {
    // ★ A region that was RETIRED after its last declaration is not declared.
    //
    // The application's `ui-rect` channel is a CHANGE LOG — it emits only when
    // a rect moves — so a control that stops being drawn leaves its last rect
    // standing in the trace with nothing after it. Reading `.last()` alone
    // therefore returns a fossil, and a caller cannot tell it from a live
    // region.
    //
    // That is not hypothetical: it made the UI-scale check report eighteen
    // ribbon controls as lying outside the window at a large scale, when the
    // ribbon's overflow had correctly swallowed every one of them and the
    // screenshot showed a clean layout with a *5 more* button. A confident,
    // detailed, entirely wrong layout-defect report, produced by reading a
    // change log as a snapshot.
    //
    // The application now closes each frame with a `ui-rect-gone name=…` line
    // per region it stopped drawing, so the log reports both directions. This
    // compares positions in the trace: a `gone` after the last `ui-rect` means
    // the region is not on screen, whatever rect it last had.
    //
    // Older traces — captured before that line existed — carry no `gone`
    // events at all, so this degrades to the previous behaviour rather than
    // to an error. That matters because `--image` runs replay dated captures.
    // `TraceLine::lineno` is the position in the FILE, so the two event
    // streams are comparable. Enumerating each iterator separately would give
    // two independent counters and compare a ui-rect's ordinal against a
    // gone-event's ordinal, which is meaningless — and would silently be
    // *mostly* right, since there are far more of the former.
    let (line_of_last_rect, rect) = trace
        .events(ui_rect)
        .filter(|l| l.get("name") == Some(name))
        .filter_map(|l| l.get_rect("rect").map(|r| (l.lineno, r)))
        .last()?;
    let retired_after = trace
        .events(UI_RECT_GONE_EVENT)
        .any(|l| l.lineno > line_of_last_rect && l.get("name") == Some(name));
    if retired_after { None } else { Some(rect) }
}

/// **A region's rectangle, once it has stopped moving.**
///
/// Reads the region, settles, reads it again, and repeats until two consecutive
/// reads agree — or until it gives up and returns the last one it saw.
///
/// # ★★ Why this exists, and it is a defect report
///
/// `ui-rect` is a **change log**: the application emits a line when a rect
/// moves, so [`declared`] answers *where that control was as of the last frame
/// the application drew*. That is exactly right for a settled window and
/// exactly wrong for one in motion, and the difference is invisible — a stale
/// coordinate is a number, not an error.
///
/// Measured, on `dimension_groups_panel_makes_a_group`, 2026-08-19: raising a
/// dock panel changes the **dock's own** layout, and it lands over several
/// frames. The check read a fold heading at `x=786..1009, y=610`, the dock
/// then re-laid out, and by the time the click was injected the panel's left
/// edge had moved past the point being aimed at — so the click landed **on the
/// canvas** and the check reported the fold as broken. Adding settle time did
/// not fix it, because the motion is triggered by the very act being measured
/// rather than by the passage of time.
///
/// > **A harness that reads a coordinate and then acts on it owns the interval
/// > between the two.** The only honest way to close that interval is to watch
/// > the coordinate until it stops.
///
/// # Why it gives up rather than failing
///
/// A rect that never settles is a real state — an animation, a spinner, a
/// progress bar — and this helper cannot know whether the caller is aiming at
/// one. Returning the last observation lets the caller's own assertion produce
/// the verdict, in its own words, with its own diagnosis. A `Result` here would
/// make every call site handle a failure mode most of them cannot describe.
pub fn stable_rect(
    session: &Session,
    ui_rect: &str,
    name: &str,
    tries: u32,
) -> Result<Option<LRect>> {
    let mut previous = declared(&session.trace()?, ui_rect, name);
    for _ in 0..tries {
        session.settle(8);
        let now = declared(&session.trace()?, ui_rect, name);
        // `None` twice is stable too, and is the honest answer for a region
        // that is not on screen — the caller's own message is what says so.
        if now == previous {
            return Ok(now);
        }
        previous = now;
    }
    Ok(previous)
}

/// The last rect a region was published with **after** a given trace line,
/// whether or not it has since been retired.
///
/// # ★★ Why [`declared`] is the wrong question for a gesture-only overlay
///
/// `declared` asks *"is this on screen now?"*, and it is right to: a region
/// retired after its last declaration is a fossil, and reading one produced a
/// confident, detailed, entirely wrong layout-defect report once already.
///
/// But an overlay that exists **only while the pointer is down** — a drop
/// caret, a rubber band, a snap indicator — is *guaranteed* to be retired by
/// the time a check can look at it. The harness cannot read the trace mid-drag:
/// `Driver::drag` presses, moves and releases before it returns. So `declared`
/// answers `None` for a caret that drew perfectly, and the check reports the
/// feature missing.
///
/// That is not hypothetical either. It is exactly what happened on
/// 2026-08-19: `pages_drag_shows_where_it_lands` failed with *"NO
/// `panel-pages-drop-caret` region was ever published"* while the trace
/// carried `ui-rect name=panel-pages-drop-caret rect=[[258.0 239.1] - [262.0
/// 331.9]]` four lines above the release. The indicator worked. The check was
/// reading a change log as a snapshot in the other direction — asking for
/// presence *now* about a thing whose whole nature is to be gone now.
///
/// # What this asks instead, and why the anchor is required rather than optional
///
/// *"Was it published during THIS gesture?"* The `after` line number is the
/// gesture's own start event, so a caret left over from an earlier drag in the
/// same run cannot satisfy it. Without that anchor this would be
/// `last-rect-ignoring-retirement`, which is the fossil-reading bug wearing a
/// helpful name — and it would pass on a build where the caret drew once at
/// startup and never again.
///
/// `TraceLine::lineno` is the position in the file, so the two streams are
/// directly comparable — the same property [`declared`] relies on.
#[must_use]
pub fn declared_since(trace: &Trace, ui_rect: &str, name: &str, after: usize) -> Option<LRect> {
    trace
        .events(ui_rect)
        .filter(|l| l.lineno > after && l.get("name") == Some(name))
        .filter_map(|l| l.get_rect("rect"))
        .last()
}

/// The event the application emits for a region it has stopped drawing.
///
/// Matched literally, like every other event name in this crate, so renaming
/// it in `crate::diag` without changing it here silently returns [`declared`]
/// to reading fossils.
pub const UI_RECT_GONE_EVENT: &str = "ui-rect-gone";

/// Every region name beginning with `prefix` that is **on screen now**.
///
/// # ★★ Why this exists beside [`declared_names`], which counts fossils
///
/// [`declared_names`]'s own documentation says *"used only for SKIP reasons"*,
/// and it means it: it collects every name that has **ever** appeared, because
/// an error message listing what the application *did* declare is more useful
/// the more it lists. Retirement is irrelevant to that job.
///
/// It is exactly wrong for a **count**. The `ui-rect` channel is a change log,
/// so a row that was deleted leaves its last declaration standing for ever, and
/// counting names therefore counts rows that are gone.
///
/// That is not hypothetical. On 2026-08-19 the Manage-groups check reported
/// *"the round trip did not close: 1 row before, 2 after the delete"* over a
/// trace containing `dimension-group-delete id=1`, `delete-dimension-group
/// epoch=3` **and** `ui-rect-gone name=dimension-groups.draw_into.1`. The
/// delete had worked, at every level, and the check said it had not — a
/// confident, specific, entirely wrong defect report about a feature that was
/// correct, produced by a helper being used outside the job its own doc comment
/// names.
///
/// So: **[`declared_names`] to say what was seen, this to say what is there.**
/// If a check compares two numbers, it wants this one.
#[must_use]
pub fn live_names(trace: &Trace, ui_rect: &str, prefix: &str) -> Vec<String> {
    declared_names(trace, ui_rect, prefix)
        .into_iter()
        .filter(|name| declared(trace, ui_rect, name).is_some())
        .collect()
}

/// Every distinct region name the application declared beginning with
/// `prefix`, in first-seen order.
///
/// Used only for SKIP reasons. A reason that says "I did not find X" and does
/// not say what it *did* find sends its reader to guess; this crate has a
/// standing rule about that ([`crate::checks`] rule 5).
#[must_use]
pub fn declared_names(trace: &Trace, ui_rect: &str, prefix: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for line in trace.events(ui_rect) {
        let Some(name) = line.get("name") else {
            continue;
        };
        if name.starts_with(prefix) && !out.iter().any(|n| n == name) {
            out.push(name.to_owned());
        }
    }
    out
}

/// Read the same captured stderr a second time, under the **shell's** line
/// prefix.
///
/// One file, two vocabularies — see the module header. `Session::trace` parses
/// with the profile's prefix; everything `egui-shell` writes carries its own
/// and lands in [`Trace::other`] on that parse. Re-parsing is cheap next to a
/// click and keeps both streams honest: a line is attributed to whichever
/// crate actually wrote it.
///
/// # Errors
///
/// If the captured stderr cannot be read at all.
/// The control that holds the groups a narrow ribbon could not fit.
pub const OVERFLOW: &str = "ribbon.overflow";

/// **Find a ribbon item, opening the overflow if that is where it went.**
///
/// ★ The fix for a whole class of false SKIPs, and it is worth understanding
/// why they were false rather than treating this as a convenience.
///
/// The harness drives a **1100 pt** window. At that width the ribbon correctly
/// folds its rightmost groups into an overflow menu — that is the responsive
/// behaviour working, not failing. A check that looked only at the tab surface
/// then reported *"no `ribbon.item.file.print` region on the File tab"*, which
/// is true and reads as *"the command is missing"*, which is false. It cost
/// `print_dialog_reaches_the_spooler` a standing FAIL that was written up as a
/// harness gap and left, and it would have cost `about` the same.
///
/// So: look on the tab; if it is not there and an overflow control is, click
/// the overflow and look again. A caller gets `None` only when the item is
/// genuinely absent from both, which is the finding it meant to make.
///
/// # Why this returns the rect rather than clicking
///
/// Because *"where is it"* and *"press it"* are different decisions, and some
/// callers want to measure a control rather than invoke it. The overflow is
/// left OPEN on return, which is what a caller that is about to click wants;
/// a caller that is not can dismiss it with Escape.
pub fn declared_or_in_overflow(
    session: &Session,
    driver: &crate::input::Driver,
    ui_rect: &str,
    name: &str,
) -> Result<Option<LRect>> {
    let trace = session.trace()?;
    if let Some(rect) = declared(&trace, ui_rect, name) {
        return Ok(Some(rect));
    }
    let Some(overflow) = declared(&trace, ui_rect, OVERFLOW) else {
        return Ok(None);
    };
    driver.click_at(session.frame()?.declared_center(overflow))?;
    session.settle(16);
    Ok(declared(&session.trace()?, ui_rect, name))
}

pub fn shell_trace(session: &Session) -> Result<Trace> {
    Trace::read(session.trace_path(), SHELL_TRACE_PREFIX)
}

/// Render a list of names for a reason string, or say plainly that there were
/// none.
///
/// `"none"` rather than `""`, because an empty list printed as nothing reads
/// as a formatting bug and hides the fact that was being reported.
#[must_use]
pub fn list(names: &[String]) -> String {
    if names.is_empty() {
        "none".to_owned()
    } else {
        names.join(", ")
    }
}

/// [`list`] for borrowed strings.
#[must_use]
pub fn list_str(names: &[&str]) -> String {
    if names.is_empty() {
        "none".to_owned()
    } else {
        names.join(", ")
    }
}

/// The dominant colour of a declared region in a capture — a control's fill.
///
/// `None` when the region resolved to no pixels, which means the application
/// declared it outside its own client area. That is a finding rather than a
/// measurement and the caller reports it as one.
#[must_use]
pub fn fill_of(image: &Image, frame: &WindowFrame, rect: LRect) -> Option<Rgb> {
    let px = frame.logical_to_capture_pixels(rect);
    if px.area() == 0 {
        return None;
    }
    let report = crate::pixels::contrast_at(image, px);
    (report.sampled > 0).then_some(report.background)
}

/// Maximum absolute per-channel difference between two colours.
#[must_use]
pub fn delta(a: Rgb, b: Rgb) -> u16 {
    let d = |x: u8, y: u8| u16::from(x.abs_diff(y));
    d(a.r, b.r).max(d(a.g, b.g)).max(d(a.b, b.b))
}

/// How far apart two dominant fills must be to count as "one of these is
/// pressed", as a maximum absolute per-channel difference in 0–255.
///
/// The derivation is [`crate::checks::markup_rectangle`]'s `MIN_PRESSED_DELTA`
/// and is not restated here, because restating it would create two accounts of
/// one measurement that can drift apart. In summary, and only as a pointer
/// into that argument:
///
/// * `egui`'s stock light palette — which is what the built binary actually
///   paints with, because nothing in `crates/pdfce-gui` calls
///   `egui_shell::theme::Theme::apply` — separates unpressed `#E5E5E5` from
///   pressed `#90D1FF` by **85**;
/// * `egui-shell`'s `quiet` preset, if it were installed, would separate them
///   by **39**;
/// * two identically filled controls in a lossless BGRA capture differ by
///   **0**, not by a small number.
///
/// Twelve sits above zero and a factor of three below the smaller of the two
/// real differences, so the verdict is the same whichever palette is in force.
///
/// A channel difference rather than a contrast ratio, because both pairs are
/// near-equal in luminance (about 1.5:1 and 1.3:1) and would therefore be
/// called *identical* by [`crate::pixels::AA_LARGE`]. Contrast answers "can
/// this be read"; the question here is "is this a different colour".
pub const MIN_PRESSED_DELTA: u16 = 12;

/// **Click a mode segment and confirm the shell saw the click.**
///
/// The move both new checks make repeatedly, with the counting that makes it
/// honest folded in.
///
/// # Why the count rather than "is there a line for this mode?"
///
/// Because a run switches modes more than once, and a check that asked
/// "did the shell ever report `mode=read`?" would be satisfied by a click it
/// made a minute ago. The event is emitted on every segment click — including
/// a click on the already-selected segment — so the number of them is the only
/// thing that distinguishes *this* click from the previous one.
///
/// # Why a failure here is a SKIP and not a FAIL
///
/// Same reason [`crate::checks::find_bar`]'s chord control exists: a check
/// that could not deliver a click has learned nothing about the application,
/// and naming a feature as the culprit when nothing was ever clicked at it is
/// worse than no check at all. The two readings — pointer injection is not
/// reaching this window, or the shell diagnostic switch did not reach the
/// process — are both stated, and this function declines to choose between
/// them.
///
/// # Errors
///
/// * the application declared no rect for the segment, so there is nothing to
///   aim at (the reason lists the segments it *did* declare);
/// * the segment was declared at no usable size;
/// * the pointer could not be driven;
/// * the shell traced no new [`MODE_EVENT`] for this mode after the click.
pub fn click_mode_segment(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    mode_id: &str,
) -> Result<()> {
    let region = format!("ribbon.mode.{mode_id}");
    let trace = session.trace()?;
    let rect = declared(&trace, ui_rect, &region).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{region}` region, so there is no mode segment to \
             click and this check cannot put the application into the mode it is about. \
             Regions it did declare under `ribbon.mode.`: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.mode."))
        ))
    })?;
    if !rect.is_substantial() {
        return Err(Error::new(format!(
            "`{region}` was declared at {rect:?}, which has no usable area. A click aimed at a \
             degenerate rectangle proves nothing, so this is reported rather than driven — and \
             it is itself the finding: `MODES_AND_PANELS.md` Part 1 requires the selector to \
             render as a real segmented control with every label visible."
        )));
    }

    let before = shell_trace(session)?
        .events(MODE_EVENT)
        .filter(|l| l.get("mode") == Some(mode_id))
        .count();
    driver.click_at(session.frame()?.declared_center(rect))?;
    session.settle(12);
    let after = shell_trace(session)?
        .events(MODE_EVENT)
        .filter(|l| l.get("mode") == Some(mode_id))
        .count();
    if after <= before {
        let shell = shell_trace(session)?;
        return Err(Error::new(format!(
            "the click on `{region}` produced no new `{MODE_EVENT} mode={mode_id}` line, so no \
             click reached the ribbon and nothing after it would mean anything. Two readings, \
             and this check declines to choose between them: the pointer injection is not \
             reaching this window, or the shell diagnostic switch {}={} did not reach the \
             process — the shell trace carries {} line(s) under `{SHELL_TRACE_PREFIX}`. \
             Trace: {}.",
            SHELL_DIAG_ENV.0,
            SHELL_DIAG_ENV.1,
            shell.lines.len(),
            session.trace_path().display()
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geom::Pt;

    /// Last wins, per name, and a name that was never declared is `None`.
    ///
    /// The same property [`crate::checks::markup_rectangle`] pins for its own
    /// copy. Pinned twice on purpose: the two copies exist to be independent,
    /// and an independent copy with no test of its own is not independent
    /// evidence, it is an untested duplicate.
    #[test]
    fn a_regions_last_declaration_is_the_one_that_is_used() {
        let trace = Trace::parse(
            "pdfce-diag start argv1=None\n\
             pdfce-diag ui-rect name=ribbon.item.measure.linear rect=[[0.0 0.0] - [10.0 10.0]]\n\
             pdfce-diag ui-rect name=ribbon.item.measure.two_line rect=[[20.0 0.0] - [30.0 10.0]]\n\
             pdfce-diag ui-rect name=ribbon.item.measure.linear rect=[[4.0 30.0] - [84.0 54.0]]",
            "pdfce-diag",
        );
        assert_eq!(
            declared(&trace, "ui-rect", "ribbon.item.measure.linear"),
            Some(LRect::new(Pt::new(4.0, 30.0), Pt::new(84.0, 54.0))),
            "an early frame can carry a rect from before the layout settled"
        );
        assert_eq!(
            declared(&trace, "ui-rect", "ribbon.item.measure.area"),
            None
        );
        assert_eq!(
            declared_names(&trace, "ui-rect", ITEM_PREFIX),
            vec![
                "ribbon.item.measure.linear".to_owned(),
                "ribbon.item.measure.two_line".to_owned()
            ],
            "each name once, in first-seen order"
        );
    }

    /// **The two channels are parsed out of one file without contaminating
    /// each other.**
    ///
    /// If a future prefix change made one a prefix of the other, this test is
    /// what says so — and the symptom otherwise would be a check that reads a
    /// `ribbon-command-invoked` that is not there, or misses one that is.
    #[test]
    fn the_application_and_shell_streams_do_not_contaminate_each_other() {
        let text = "pdfce-diag start argv1=None\n\
                    egui-shell-diag ribbon-mode-selected mode=review\n\
                    egui-shell-diag ribbon-command-invoked id=measure.linear handler=600\n\
                    pdfce-diag measure-tool tool=Measure(Linear)\n";
        let app = Trace::parse(text, "pdfce-diag");
        let shell = Trace::parse(text, SHELL_TRACE_PREFIX);

        assert!(app.started("start"));
        assert!(
            app.events(INVOKE_EVENT).next().is_none(),
            "the shell's line must not be read as the application's"
        );
        assert_eq!(
            app.last("measure-tool").and_then(|l| l.get("tool")),
            Some("Measure(Linear)")
        );
        assert!(
            shell
                .events(MODE_EVENT)
                .any(|l| l.get("mode") == Some("review"))
        );
        assert!(
            shell.events("measure-tool").next().is_none(),
            "the application's line must not be read as the shell's"
        );
    }

    /// The difference is symmetric and takes the largest channel, so a shift
    /// confined to one channel still registers.
    #[test]
    fn the_difference_is_the_largest_channel_and_is_symmetric() {
        let a = Rgb::new(200, 100, 50);
        let b = Rgb::new(190, 100, 90);
        assert_eq!(delta(a, b), 40);
        assert_eq!(delta(b, a), 40);
        assert_eq!(delta(a, a), 0, "identical fills differ by nothing at all");
    }

    /// **The threshold separates pressed from unpressed under both palettes
    /// this build might paint with — and a contrast ratio separates neither.**
    ///
    /// The second assertion is the one that matters: `AA_LARGE` is 3.0 and
    /// these fills are 1.5:1 and 1.3:1 apart, so a check written against the
    /// harness's usual legibility oracle would report "no difference" about a
    /// control that is visibly blue.
    #[test]
    fn the_threshold_separates_pressed_from_unpressed_under_both_palettes() {
        let pairs = [
            (
                Rgb::new(229, 229, 229),
                Rgb::new(144, 209, 255),
                85_u16,
                "egui's stock light palette — MEASURED from a real capture",
            ),
            (
                Rgb::new(232, 232, 234),
                Rgb::new(193, 207, 230),
                39_u16,
                "egui-shell's `quiet` preset, composited — computed",
            ),
        ];
        for (unpressed, pressed, expected, what) in pairs {
            assert_eq!(delta(unpressed, pressed), expected, "{what}");
            assert!(
                expected > MIN_PRESSED_DELTA * 3,
                "the threshold must sit well below the difference produced by {what}"
            );
            let ratio = crate::pixels::contrast_ratio(unpressed, pressed);
            assert!(
                ratio < crate::pixels::AA_LARGE,
                "a contrast threshold would call these two fills the same colour \
                 ({ratio:.2}:1) under {what}, which is why this module measures a channel \
                 difference instead"
            );
        }
    }

    /// A list with nothing in it says so in words.
    #[test]
    fn an_empty_list_reads_as_none_rather_than_as_nothing() {
        assert_eq!(list(&[]), "none");
        assert_eq!(list_str(&[]), "none");
        assert_eq!(list_str(&["a", "b"]), "a, b");
    }
}
