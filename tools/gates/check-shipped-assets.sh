#!/usr/bin/env bash
#
# check-shipped-assets.sh — wrapper around `check-shipped-assets.py`.
#
# ===========================================================================
# WHY A WRAPPER AT ALL
# ===========================================================================
#
# `run-all.sh` invokes every gate as `bash <gate>`, and that uniformity is
# worth keeping: a runner with one special case acquires a second one. The
# gate itself is Python — its own header argues why, in short because its
# central check reads `PAYLOAD_DOCS` and `PAYLOAD_ASSET_DIRS` **out of
# `tools/package-portable.py` by importing it**, and reading a Python list
# with a grep is the silently-rotting pattern every gate here exists to avoid.
#
# So this file's whole job is: find an interpreter, or say so honestly.
#
# ===========================================================================
# "PYTHON IS NOT ON PATH" IS NOT "THE ASSETS ARE FINE"
# ===========================================================================
#
# The lesson is already written down in `run-all.sh`, against `cargo`:
#
#   > A skip reason is read precisely when someone cannot see the machine. It
#   > has to name the actual fact.
#
# `tools/package-portable.py` runs `run-all.sh` through `subprocess`, and the
# bash it spawns is neither a login nor an interactive shell — it has already
# been observed not to inherit `~/.cargo/bin`. There is no reason to assume
# Python fares better, and a gate that exited 0 because it could not run would
# be the worst available outcome: a licence obligation reported as discharged
# by a check that never happened.
#
# Hence exit 2 (SKIPPED), which `run-all.sh` renders in its own block and
# which makes the whole run exit 3 rather than 0.
#
# Three spellings are tried because Windows ships the `py` launcher, some
# environments have only `python3`, and Git Bash usually has `python`.
#
# EXIT CODES — passed through from the Python gate, plus:
#   2  SKIPPED — no Python interpreter was found. NOT a pass.
set -uo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GATE="$HERE/check-shipped-assets.py"

if [ ! -f "$GATE" ]; then
    echo "shipped-assets: SKIPPED — $GATE is missing."
    echo "  Nothing was checked, and 'nothing checked' is not 'nothing wrong'."
    exit 2
fi

for candidate in python python3 py; do
    if command -v "$candidate" >/dev/null 2>&1; then
        exec "$candidate" "$GATE" "$@"
    fi
done

echo "shipped-assets: SKIPPED — no Python interpreter on PATH."
echo ""
echo "  Tried: python, python3, py."
echo ""
echo "  The asset licensing is NOT implicated: nothing was read, because the"
echo "  tool that would read it was never found. If this ran from a script,"
echo "  the spawned shell probably did not inherit the interpreter's"
echo "  directory — the same shape as run-all.sh's cargo-on-PATH note."
exit 2
