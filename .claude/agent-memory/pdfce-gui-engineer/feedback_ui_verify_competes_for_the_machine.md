---
name: ui-verify-competes-for-the-machine
description: ui-verify drives the real desktop, so it cannot run while Ken is using the PC — batch harness runs and ask for a go-ahead
metadata:
  type: feedback
---

**`tools/ui-verify/` may not be run while Ken is working at the machine.** It
launches the release binary, raises its window, and injects synthetic mouse and
keyboard input through the OS. That steals focus and clicks into whatever the
operator has in front of them.

**Why:** stated by Ken on 2026-08-17 — *"i am using the pc as well, so you can't
use the mouse display or keyboard until i give the go ahead."* This is not a
one-off: the harness is a single-desktop instrument by design (that is what
makes it the only oracle R1 trusts), so it will always contend with the operator
for the same machine.

**How to apply:**

- Treat display/keyboard/mouse as a **shared resource requiring an explicit
  go-ahead**, not as something to ask about per-run.
- Compute-only work stays available under the constraint: `cargo test`,
  `cargo clippy`, `tools/gates/run-all.sh`, source reading, and *writing* new
  `ui-verify` checks all run headless. Only *executing* the harness is blocked.
- Do the work anyway, then **batch the harness runs** and present them as a
  queue when the go-ahead comes. Do not stall a build waiting for the desktop.
- **Never let the constraint soften R1.** Work completed under it is
  *unverified*, and must be reported in exactly those words — this project was
  founded on a commit that said *"analysis-confirmed, NOT empirically
  verified"* and was treated as done anyway. A green `cargo test` is not a
  substitute and saying so is the whole point of [[r1-drive-the-binary]].

See also [[project-operator-report-2026-08-17]].
