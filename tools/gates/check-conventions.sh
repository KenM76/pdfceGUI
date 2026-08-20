#!/usr/bin/env bash
# check-conventions.sh — every interactive surface has answered, row by row,
# the conventions its gesture class carries.
#
# The corpus is D:\dev\rag\ui-conventions\ — one file per gesture, each a
# numbered list of the rules every program in the class already follows. The
# registry is tools/gates/conventions.list, which says which surface answers to
# which class and carries the full argument for why this exists.
#
# ---------------------------------------------------------------------------
# WHAT IT CAN AND CANNOT CHECK, STATED PLAINLY
# ---------------------------------------------------------------------------
#
# It CANNOT check behaviour. No grep can tell whether a preview tracks the
# pointer at 1:1, and pretending otherwise would be worse than useless — a green
# gate asserting something it never measured is the exact failure this project
# exists to remove.
#
# It CAN check that somebody consciously answered the question for each row,
# and it makes an unanswered one visible. That is the whole of the value, and it
# is enough: every convention the operator has had to report was one nobody had
# asked about, not one somebody decided against.
#
# The rows that CAN be verified get a driven check or a unit test as well. This
# gate is the floor, not the ceiling.
#
# ---------------------------------------------------------------------------
# THE FORM OF AN ANSWER
# ---------------------------------------------------------------------------
#
#     // conventions: drag-moves
#     // - D1 live-preview: <how>
#     // - D3 escape-cancels: WAIVED — <why>
#
# A row is answered when its id appears after `- ` in a comment in the file.
# `WAIVED` is an answer and needs a reason; the gate checks the word `WAIVED`
# is not the end of the line.
#
# Exit 0 clean, 1 on a violation, 2 if the corpus cannot be read.

set -uo pipefail
cd "$(dirname "$0")/../.." || exit 1

CORPUS="${UI_CONVENTIONS_DIR:-D:/dev/rag/ui-conventions}"
REGISTRY="tools/gates/conventions.list"

echo "check-conventions: every interactive surface answers its gesture class…"

if [ ! -d "$CORPUS" ]; then
    echo "  the conventions corpus is not at $CORPUS."
    echo "  Set UI_CONVENTIONS_DIR, or clone it. This gate has no oracle without it,"
    echo "  and reporting PASS with no oracle is the failure it exists to prevent."
    exit 2
fi

# Rule ids for a class: every `## <ID>.` heading in its corpus file, with any
# leading ★ decoration stripped. `## ★★ D1. …` yields `D1`.
rows_for() {
    local class="$1"
    local file="$CORPUS/$class.md"
    [ -f "$file" ] || return 1
    grep -E '^## ' "$file" \
        | sed -E 's/^## //; s/^[★ ]*//; s/^([A-Z][0-9]+[a-z]?)\..*$/\1/' \
        | grep -E '^[A-Z][0-9]+[a-z]?$' \
        | sort -u
}

violations=0
surfaces=0

while read -r path class; do
    case "$path" in ''|'#'*) continue ;; esac
    surfaces=$((surfaces + 1))

    if [ ! -f "$path" ]; then
        echo "  $path: listed in the registry and not on disk."
        violations=$((violations + 1))
        continue
    fi
    if ! rows="$(rows_for "$class")"; then
        echo "  $path: names class '$class' and $CORPUS/$class.md does not exist."
        violations=$((violations + 1))
        continue
    fi

    # The file's answers: every `- <ID> ` inside a comment.
    answers="$(grep -oE '^[[:space:]]*(//[/!]?|#)[[:space:]]*-[[:space:]]*[A-Z][0-9]+[a-z]?[[:space:]:]' "$path" 2>/dev/null \
        | grep -oE '[A-Z][0-9]+[a-z]?' | sort -u)"

    if ! grep -qE '(//[/!]?|#)[[:space:]#]*conventions:[[:space:]]*'"$class" "$path"; then
        echo "  $path: no \`conventions: $class\` block."
        echo "      Its class carries these rows: $(echo "$rows" | tr '\n' ' ')"
        echo "      See $CORPUS/$class.md"
        violations=$((violations + 1))
        continue
    fi

    missing=""
    for row in $rows; do
        echo "$answers" | grep -qx "$row" || missing="$missing $row"
    done
    if [ -n "$missing" ]; then
        echo "  $path ($class): unanswered —$missing"
        violations=$((violations + 1))
    fi

    # A waiver with no reason is the exemption without the argument.
    while IFS= read -r bare; do
        [ -z "$bare" ] && continue
        echo "  $path: waived with no reason — $(printf '%s' "$bare" | sed 's/^[[:space:]]*//')"
        violations=$((violations + 1))
    done < <(grep -nE '^[[:space:]]*(//[/!]?|#).*WAIVED[[:space:]]*$' "$path" 2>/dev/null | cut -d: -f2-)

done < "$REGISTRY"

echo "  $surfaces surface(s) registered."

if [ "$violations" -gt 0 ]; then
    cat <<'MSG'

A surface listed in tools/gates/conventions.list must answer every row of its
gesture class, in a comment block:

    // conventions: drag-moves
    // - D1 live-preview: the polyline follows the pointer, previewed from the
    //   same function a committed dimension is drawn from.
    // - D3 escape-cancels: WAIVED — the gesture machine owns Escape.

This gate cannot check behaviour and does not pretend to. It checks that the
question was ASKED. Every convention the operator has had to report was one
nobody had asked about — not one somebody decided against.
MSG
    echo
    echo "check-conventions: FAIL — $violations surface(s) with an unanswered class."
    exit 1
fi

echo "check-conventions: PASS — every registered surface has answered."
exit 0
