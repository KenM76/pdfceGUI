#!/usr/bin/env bash
# check-file-size.sh — no .rs file may exceed 1,500 lines. Standing rule R2.
#
# ===========================================================================
# WHY THIS GATE EXISTS
# ===========================================================================
#
# The GUI this project replaces has a single `main.rs` of 25,005 code lines.
# That number is not an aesthetic complaint; it is the direct cause of several
# entries in DEFECTS.md, and it is why this rebuild exists at all:
#
#   * Nobody can hold 25,000 lines in their head, so the same concept gets
#     re-implemented in two places and the copies drift. D5 (the shortcut
#     reference disagreeing with the actual key bindings) is exactly that.
#   * A reviewer cannot see that a keyboard guard at line 13,777 interacts
#     with a focus request at line 16,891. D1 is exactly that: two correct-
#     looking lines three thousand lines apart that together break the Delete
#     key.
#   * Tooling degrades. `cargo fmt` on a file that size, an LLM reading it, a
#     grep for a symbol — all become approximate.
#
# 1,500 lines is not a magic number. It is roughly "one sitting", and it is
# small enough that the file has to have a single subject. The value of the
# limit is that it is enforced from the first commit rather than adopted after
# the file is already unmanageable — 25,005 lines is not a decision anybody
# made, it is a decision nobody ever made.
#
# ===========================================================================
# WHAT IS COUNTED, AND WHY IT IS TOTAL LINES
# ===========================================================================
#
# Total physical lines, comments and blanks included. Deliberately NOT "code
# lines":
#
#   * This project asks for verbose documentation (a standing instruction), so
#     a "code lines" metric would be the one that quietly rewards deleting the
#     docs to get under a threshold. That is the wrong incentive to build into
#     a gate.
#   * A file whose 1,400 comment lines make it 2,900 lines long is still a file
#     nobody can navigate. The reading cost is the thing being limited.
#
# The right response to this gate firing is to SPLIT THE MODULE, not to shrink
# the prose. If a file genuinely needs to be longer — a generated table, a
# large const catalog — that is an operator decision and it belongs in the
# EXEMPT list below with a reason, not in a silent threshold bump.
#
# ===========================================================================
# USAGE / EXIT CODES
# ===========================================================================
#   tools/gates/check-file-size.sh [LIMIT]
#
#   0  every .rs file is within the limit
#   1  at least one file is over
#   2  PRECONDITION ABSENT — nothing to scan (no .rs files anywhere yet)

set -euo pipefail

LIMIT="${1:-1500}"

# Roots to scan. `target/` is excluded below: build artefacts include generated
# .rs files that nobody wrote and nobody can split.
ROOTS=()
[ -d crates ] && ROOTS+=(crates)
[ -d tools ] && ROOTS+=(tools)

if [ "${#ROOTS[@]}" -eq 0 ]; then
    echo "file-size: SKIPPED — no crates/ or tools/ directory here." >&2
    echo "  Run from the repository root. Exiting 2, not 0: an unscanned tree" >&2
    echo "  is not a compliant tree." >&2
    exit 2
fi

# EXEMPT — paths whose size is not a maintenance problem. One entry, one
# reason, reviewed like any other rule change.
#
#   fixtures/  — gate fixtures are inputs to shell scripts, not modules anybody
#                navigates. (They are all tiny anyway; the exclusion is for the
#                day one of them is a generated corpus.)
is_exempt() {
    case "$1" in
        */fixtures/*) return 0 ;;
        */target/*) return 0 ;;
        *) return 1 ;;
    esac
}

scanned=0
offenders=""
# Every file with its count, for the "largest files" report. Kept in one string
# rather than an array so the sort below is a single pipeline.
all=""

while IFS= read -r -d '' file; do
    is_exempt "$file" && continue
    scanned=$((scanned + 1))
    n=$(wc -l < "$file" | tr -d ' ')
    all="${all}${n} ${file}
"
    if [ "$n" -gt "$LIMIT" ]; then
        offenders="${offenders}${n} ${file}
"
    fi
done < <(find "${ROOTS[@]}" -type f -name '*.rs' -not -path '*/target/*' -print0 | sort -z)

if [ "$scanned" -eq 0 ]; then
    echo "file-size: SKIPPED — no .rs files found under ${ROOTS[*]}." >&2
    echo "  Exiting 2, not 0: 'nothing over the limit' and 'nothing at all'" >&2
    echo "  must not produce the same green tick." >&2
    exit 2
fi

if [ -n "$offenders" ]; then
    echo "file-size: FAIL — $(printf '%s' "$offenders" | grep -c '^') file(s) over $LIMIT lines:"
    printf '%s' "$offenders" | sort -rn | awk '{ printf "  %7d  %s\n", $1, $2 }'
    cat <<EOF

Rule R2: no .rs file over $LIMIT lines. Split the module along its seams —
one subject per file — rather than raising the limit.

The GUI this project replaces reached 25,005 lines in a single main.rs. That
was never a decision anybody made; it was a limit nobody ever set. Two of the
defects in DEFECTS.md are pairs of lines thousands of lines apart that no
reviewer could have been expected to see together.

If a file genuinely cannot be split (a generated table, a large const
catalog), that is an operator decision: add it to is_exempt() in this script
with the reason written down.
EOF
    exit 1
fi

# Report the three largest even on success. A gate that only speaks when it
# fails gives no warning that a file is at 1,480 lines and one feature away
# from firing — and the cheapest moment to split a module is before it has to
# be split in a hurry.
echo "file-size: clean — $scanned .rs file(s) scanned, none over $LIMIT lines"
if [ -n "$all" ]; then
    echo "           largest:"
    printf '%s' "$all" | sort -rn | head -3 | awk '{ printf "             %7d  %s\n", $1, $2 }'
fi
exit 0
