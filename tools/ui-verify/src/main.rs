//! `ui-verify` — the command line.
//!
//! A thin shell over [`ui_verify`]: parse arguments, build a
//! [`ui_verify::checks::CheckContext`], run the selected checks, print the
//! report, exit with its code.
//!
//! ## Why the argument parser is hand-written
//!
//! Eighty lines, versus a `clap` dependency. Same reasoning as the rest of the
//! crate's dependency posture (see `Cargo.toml`): this is the tool reached for
//! when the application is misbehaving, and every dependency it carries is one
//! more way for it to fail to build on the day it is most needed. The argument
//! surface here is a dozen flags and it is not going to grow into something
//! that needs subcommands.
//!
//! ## Exit codes
//!
//! | Code | Meaning |
//! |---|---|
//! | 0 | every check that ran passed, and at least one ran |
//! | 1 | a check drove the application and its assertion did not hold |
//! | 2 | the command line itself was wrong |
//! | 3 | nothing failed, but something did not run — an INCOMPLETE run |
//!
//! 3 is non-zero on purpose. CI must not go green on a suite that did not
//! execute. See [`ui_verify::report`] for the full argument, and
//! `tools/gates/run-all.sh` for the same three-state model applied to the
//! shell gates.

use std::path::PathBuf;
use std::process::ExitCode;

use ui_verify::checks::{self, CheckContext};
use ui_verify::coords::DocPoint;
use ui_verify::pixels;
use ui_verify::profile;
use ui_verify::report::RunReport;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(code) => code,
        Err(message) => {
            eprintln!("ui-verify: {message}");
            eprintln!();
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

fn run(args: &[String]) -> Result<ExitCode, String> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("{USAGE}");
        return Ok(ExitCode::SUCCESS);
    }
    if args.iter().any(|a| a == "--list") {
        list();
        return Ok(ExitCode::SUCCESS);
    }

    let mut profile_name = profile::PDFCE_GUI.name.to_owned();
    let mut exe: Option<PathBuf> = None;
    let mut pdf: Option<PathBuf> = None;
    let mut image: Option<PathBuf> = None;
    let mut out_dir = PathBuf::from("target/ui-verify");
    let mut selected: Vec<String> = Vec::new();
    let mut threshold = pixels::AA_LARGE;
    let mut allow_input = true;
    let mut allow_stale = false;
    let mut source_root: Option<PathBuf> = Some(PathBuf::from("crates"));
    let mut page_size: Option<(f64, f64)> = None;
    let mut target: Option<DocPoint> = None;

    let mut i = 0usize;
    while i < args.len() {
        let arg = args[i].as_str();
        let mut value = |what: &str| -> Result<String, String> {
            i += 1;
            args.get(i)
                .cloned()
                .ok_or_else(|| format!("{what} needs a value"))
        };
        match arg {
            "--profile" => profile_name = value("--profile")?,
            "--exe" => exe = Some(PathBuf::from(value("--exe")?)),
            "--pdf" => pdf = Some(PathBuf::from(value("--pdf")?)),
            "--image" => image = Some(PathBuf::from(value("--image")?)),
            "--out" => out_dir = PathBuf::from(value("--out")?),
            "--check" => selected.push(value("--check")?),
            "--contrast" => {
                let v = value("--contrast")?;
                threshold = v
                    .parse::<f64>()
                    .map_err(|_| format!("--contrast wants a number, got `{v}`"))?;
            }
            "--doc-point" => {
                let v = value("--doc-point")?;
                target = Some(parse_doc_point(&v)?);
            }
            "--page-size" => {
                let v = value("--page-size")?;
                page_size = Some(parse_page_size(&v)?);
            }
            "--no-input" => allow_input = false,
            "--allow-stale" => allow_stale = true,
            "--source-root" => source_root = Some(PathBuf::from(value("--source-root")?)),
            "--no-staleness-check" => source_root = None,
            other => return Err(format!("unknown argument `{other}`")),
        }
        i += 1;
    }

    let profile = profile::by_name(&profile_name).ok_or_else(|| {
        format!(
            "no profile named `{profile_name}`. Known: {}",
            profile::all()
                .iter()
                .map(|p| p.name)
                .collect::<Vec<_>>()
                .join(", ")
        )
    })?;

    let ctx = CheckContext {
        profile,
        exe,
        pdf,
        image,
        out_dir,
        contrast_threshold: threshold,
        allow_input,
        allow_stale,
        source_root,
        page_size,
        target,
    };

    let all = checks::all();
    if !selected.is_empty() {
        for want in &selected {
            if !all.iter().any(|c| c.name() == want) {
                return Err(format!(
                    "no check named `{want}`. Known: {}",
                    all.iter().map(|c| c.name()).collect::<Vec<_>>().join(", ")
                ));
            }
        }
    }

    println!(
        "ui-verify — profile `{}` ({})",
        profile.name, profile.description
    );
    if allow_input {
        println!(
            "  NOTE: this harness drives the REAL cursor and keyboard. It raises the target \
             window, moves the pointer, and types into it. The pointer is put back where it \
             was when the run ends. Pass --no-input to disable (checks that need input then \
             report SKIPPED, never PASS)."
        );
    }
    println!();

    let mut run = RunReport::default();
    for check in &all {
        if !selected.is_empty() && !selected.iter().any(|s| s == check.name()) {
            continue;
        }
        run.checks.push(check.run(&ctx));
    }
    run.print();
    Ok(ExitCode::from(u8::try_from(run.exit_code()).unwrap_or(1)))
}

fn list() {
    println!("Checks:");
    for c in checks::all() {
        println!("  {}", c.name());
        println!("      {}", c.defect());
    }
    println!();
    println!("Profiles:");
    for p in profile::all() {
        println!("  {:<14} {}", p.name, p.description);
        println!("      default exe: {}", p.default_exe);
        for set in p.region_sets {
            println!(
                "      region set `{}` ({} region(s)): {}",
                set.name,
                set.regions.len(),
                set.provenance
            );
        }
    }
}

/// `PAGE,X,Y` in PDF user space.
fn parse_doc_point(v: &str) -> Result<DocPoint, String> {
    let parts: Vec<&str> = v.split(',').map(str::trim).collect();
    if parts.len() != 3 {
        return Err(format!(
            "--doc-point wants PAGE,X,Y (PDF user space, origin bottom-left), got `{v}`"
        ));
    }
    let page = parts[0]
        .parse::<usize>()
        .map_err(|_| format!("--doc-point page `{}` is not a number", parts[0]))?;
    let x = parts[1]
        .parse::<f64>()
        .map_err(|_| format!("--doc-point x `{}` is not a number", parts[1]))?;
    let y = parts[2]
        .parse::<f64>()
        .map_err(|_| format!("--doc-point y `{}` is not a number", parts[2]))?;
    Ok(DocPoint::new(page, x, y))
}

/// `WxH` in points.
fn parse_page_size(v: &str) -> Result<(f64, f64), String> {
    let (w, h) = v
        .split_once(['x', 'X'])
        .ok_or_else(|| format!("--page-size wants WxH in points, got `{v}`"))?;
    let w = w
        .trim()
        .parse::<f64>()
        .map_err(|_| format!("--page-size width `{w}` is not a number"))?;
    let h = h
        .trim()
        .parse::<f64>()
        .map_err(|_| format!("--page-size height `{h}` is not a number"))?;
    Ok((w, h))
}

const USAGE: &str = "\
ui-verify — drive the built GUI and assert on its trace and its pixels.

USAGE
  ui-verify [OPTIONS]

OPTIONS
  --profile NAME     target profile (default: pdfce-gui). --list shows them.
  --exe PATH         the binary to drive. Defaults to the profile's.
  --pdf PATH         the fixture document to open.
  --doc-point P,X,Y  where a driving check aims, in PDF user space (origin
                     bottom-left). NO DEFAULT, on purpose: a guessed point
                     produces a click on empty page, which is symptom-identical
                     to a broken hit test.
  --page-size WxH    page size in points, when the fixture's /MediaBox cannot
                     be read from the file.
  --image PATH       assert a pixel check against an already-captured PNG
                     instead of driving the application. Used for
                     falsification against dated evidence.
  --check NAME       run only this check. Repeatable.
  --contrast RATIO   WCAG contrast floor (default 3.0, AA large text).
  --out DIR          where screenshots and trace captures go
                     (default target/ui-verify).
  --no-input         do not touch the real pointer or keyboard. Checks that
                     need input then report SKIPPED — never PASS.
  --allow-stale      drive a binary older than its sources. Off by default:
                     a missing trace from an unbuilt change looks exactly like
                     a broken feature.
  --source-root DIR  what the staleness check compares against (default
                     crates).
  --no-staleness-check   disable it entirely.
  --list             list checks and profiles.
  -h, --help         this text.

EXIT CODES
  0  everything that ran passed, and at least one thing ran
  1  a check drove the application and its assertion did not hold
  2  the command line was wrong
  3  nothing failed, but something did not run — an INCOMPLETE run, which is
     not a pass

EXAMPLES
  # What is available?
  ui-verify --list

  # Falsify the pixel oracle against the dated evidence for D2.
  ui-verify --profile pdfce-legacy --check settings_headings_legible \\
            --image evidence/crop_settings.png

  # Drive the Delete-key check against a built binary.
  ui-verify --exe target/release/pdfce-gui.exe --pdf fixture.pdf \\
            --doc-point 0,300,500 --check delete_key_after_canvas_click
";
