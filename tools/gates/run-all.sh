#!/usr/bin/env bash
# run-all.sh — every gate in this directory, plus fmt and clippy, in one run.
#
# ===========================================================================
# WHAT THIS IS FOR
# ===========================================================================
#
# One command a developer runs before pushing, and the one CI runs. The reason
# it exists as a script rather than a list of steps in a CI YAML is the lesson
# recorded at the top of `check-ui-strings.sh`: pdfce's string rule lived as an
# inline CI grep, was red at baseline for months, and therefore enforced
# nothing. A gate has to be runnable locally, in one command, or it becomes
# scenery.
#
# ===========================================================================
# THE THREE-STATE MODEL, AND WHY "SKIPPED" IS NOT "PASSED"
# ===========================================================================
#
# Every gate here returns one of three things:
#
#   0  PASS     — it ran, and found nothing wrong.
#   1  FAIL     — it ran, and found something wrong.
#   2  SKIPPED  — its PRECONDITION was absent. The crate does not exist yet,
#                 the tree has no source files, the binary was never built.
#
# Most gate runners have two states and fold the third into the first. That is
# the single defect this project is most determined not to repeat:
# PROJECT_PLAN.md §4.1 documents a gate that "would print `ui-strings: clean`
# while checking a handful of files", because finding nothing looks exactly
# like finding no violations.
#
# So SKIPPED is tracked separately, printed in its own block, and — critically
# — a run containing any skip exits **3**, not 0. It is not a failure, and it
# is not a pass either. CI must not go green on a gate set that did not fully
# run. If a skip is expected (another crate is mid-write, this is a partial
# checkout), the human reads the reason and decides; the machine does not get
# to decide it for them.
#
# ===========================================================================
# ORDER
# ===========================================================================
#
# The self-tests run FIRST, before any gate is trusted. If a gate cannot detect
# its own planted violation, its verdict on the real crate is worth nothing,
# and finding that out after a green run is finding it out too late.
#
# Three gates carry one: `check-ui-strings.sh`, `check-theme-colors.sh` and
# `check-shipped-assets.sh`. The first two are greps over source, which is the
# category that fails SILENTLY — a pattern that stops matching, a path that
# stops resolving, and a find that walks an empty tree all print exactly what a
# clean run prints.
#
# The third fails silently for a different and worse reason. It checks that
# every redistributed third-party asset's licence reaches the operator, and a
# repository with no asset directories, or a scan that finds none, prints
# "clean" just as loudly as one where every obligation is discharged. That gate
# ALSO has no natural failure in daily use: assets are added rarely, so it
# could sit green for months while quietly checking nothing. Its self-test
# plants four separate violations, exempts a fifth, and passes a sixth.
#
# fmt and clippy run LAST, because they are the slow ones and because a
# formatting complaint is the least interesting thing this script can tell you.
#
# ===========================================================================
# USAGE / EXIT CODES
# ===========================================================================
#   tools/gates/run-all.sh              everything
#   tools/gates/run-all.sh --no-cargo   gates only, no fmt/clippy (fast)
#
#   0  everything ran and everything passed
#   1  at least one gate FAILED
#   3  nothing failed, but at least one gate was SKIPPED — an incomplete run

set -uo pipefail          # NOT -e: a failing gate must be recorded, not fatal

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
cd "$ROOT" || exit 1

RUN_CARGO=1
[ "${1:-}" = "--no-cargo" ] && RUN_CARGO=0

PASSED=(); FAILED=(); SKIPPED=()

rule() { printf '%s\n' "------------------------------------------------------------------------"; }

# run <label> <command...> — run one gate, classify by exit code, record it.
run() {
    local label="$1"; shift
    rule
    echo ">> $label"
    rule
    "$@"
    local rc=$?
    case "$rc" in
        0) PASSED+=("$label") ;;
        2) SKIPPED+=("$label") ;;
        *) FAILED+=("$label") ;;
    esac
    echo ""
    return 0
}

echo ""
echo "pdfceGUI gates — $(date '+%Y-%m-%d %H:%M:%S') — $ROOT"
echo ""

# --- 0. the gates prove they can fail, before their verdicts are believed ---
run "check-ui-strings --self-test"  bash "$HERE/check-ui-strings.sh" --self-test
run "check-theme-colors --self-test" bash "$HERE/check-theme-colors.sh" --self-test
run "check-shipped-assets --self-test" bash "$HERE/check-shipped-assets.sh" --self-test

# --- 1. the gates themselves ------------------------------------------------
run "check-ui-strings"   bash "$HERE/check-ui-strings.sh"
run "check-theme-colors" bash "$HERE/check-theme-colors.sh"
run "check-file-size"    bash "$HERE/check-file-size.sh"
run "check-shell-purity" bash "$HERE/check-shell-purity.sh"
run "check-shipped-assets" bash "$HERE/check-shipped-assets.sh"

# --- 2. cargo fmt / clippy --------------------------------------------------
#
# Both are wrapped in a workspace-loadability probe. If a member crate listed
# in the root Cargo.toml has no manifest yet — normal while several agents are
# building different crates — cargo cannot load the workspace at all, and its
# error has nothing to do with formatting or lints. Reporting that as a fmt
# FAILURE would be a false accusation against whoever is mid-write, so it is
# reported as a SKIP with the real reason.
#
# ★ "cargo is not on PATH" is NOT "the workspace does not load", and the two
# are separated here because merging them produced a flatly false message.
#
# `tools/package-portable.py` runs this script through `subprocess`, and the
# bash it spawns does not inherit the PATH entry for `~/.cargo/bin`. The probe
# below then failed with `cargo: command not found` and this script reported
# "the workspace does not currently load", followed by advice about a member
# crate being mid-write. Every word of that was wrong: the workspace was fine,
# nothing was mid-write, and the reader was pointed at the one place the
# problem was not.
#
# A skip reason is read precisely when someone cannot see the machine. It has
# to name the actual fact.
if [ "$RUN_CARGO" -eq 1 ] && ! command -v cargo >/dev/null 2>&1; then
    rule
    echo ">> cargo fmt / cargo clippy"
    rule
    echo "SKIPPED — cargo is not on PATH in this shell."
    echo ""
    echo "  The workspace is not implicated: nothing was parsed, because the"
    echo "  tool that would parse it was never found. If this ran from a script,"
    echo "  the spawned shell probably did not inherit ~/.cargo/bin."
    echo ""
    SKIPPED+=("cargo fmt (cargo not on PATH)")
    SKIPPED+=("cargo clippy (cargo not on PATH)")
elif [ "$RUN_CARGO" -eq 1 ]; then
    if ! probe=$(cargo metadata --no-deps --format-version 1 2>&1 >/dev/null); then
        rule
        echo ">> cargo fmt / cargo clippy"
        rule
        echo "SKIPPED — the workspace does not currently load:"
        printf '%s\n' "$probe" | sed 's/^/  /' | head -20
        echo ""
        echo "  This is expected while a member crate is being written. Neither fmt"
        echo "  nor clippy can say anything about a workspace cargo cannot parse,"
        echo "  and calling that a formatting failure would blame the wrong file."
        echo ""
        SKIPPED+=("cargo fmt")
        SKIPPED+=("cargo clippy")
    else
        run "cargo fmt" cargo fmt --all --check
        run "cargo clippy" cargo clippy --workspace --all-targets -- -D warnings
    fi
else
    SKIPPED+=("cargo fmt (--no-cargo)")
    SKIPPED+=("cargo clippy (--no-cargo)")
fi

# ---------------------------------------------------------------------------
# SUMMARY
# ---------------------------------------------------------------------------
rule
echo "SUMMARY"
rule
for g in "${PASSED[@]:-}";  do [ -n "$g" ] && echo "  PASS     $g"; done
for g in "${SKIPPED[@]:-}"; do [ -n "$g" ] && echo "  SKIPPED  $g"; done
for g in "${FAILED[@]:-}";  do [ -n "$g" ] && echo "  FAIL     $g"; done
echo ""
np=${#PASSED[@]}; nf=${#FAILED[@]}; ns=${#SKIPPED[@]}
echo "  $np passed, $nf failed, $ns skipped"
echo ""

if [ "$nf" -gt 0 ]; then
    echo "RESULT: FAIL — $nf gate(s) found a violation."
    exit 1
fi
if [ "$ns" -gt 0 ]; then
    echo "RESULT: INCOMPLETE — nothing failed, but $ns gate(s) never ran."
    echo ""
    echo "  This is NOT a pass. A gate whose precondition was absent has told you"
    echo "  nothing, and 'told you nothing' printed as green is the exact defect"
    echo "  PROJECT_PLAN.md §4.1 exists to remove. Read each SKIPPED reason above"
    echo "  and decide whether it is expected."
    exit 3
fi
echo "RESULT: PASS — every gate ran and every gate is clean."
exit 0
