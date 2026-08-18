---
name: update-engine-before-every-build
description: Always run cargo update on pdfce-core/render/print before packaging a build — Ken's standing instruction, 2026-08-17
metadata:
  type: feedback
---

**Always `cargo update -p pdfce-core -p pdfce-render -p pdfce-print` before
building a release.** Ken, 2026-08-17: *"always update core render and print
before building the latest."*

**Why:** the engine dependency is `git = "file:///D:/Dev/pdfce", branch = "main"`,
so `Cargo.lock` pins a revision and only `cargo update` moves it. Ken works the
`D:\Dev\pdfce` repo in a parallel session and it moves **fast** — it went 8, then
12, then 4, then 6 commits ahead of the locked revision within a single afternoon.
A build taken without updating silently ships an engine older than the repository
has.

This has already cost something concrete: a stale GitHub pin left the shell eight
commits behind `1e7a0be`, the fix that made `Separation`/`DeviceN`/`Lab`/`CalGray`/
`CalRGB` **images** decode instead of being dropped from the raster. Eighteen
pictures were missing from Ken's own file, and the *old* shell rendered it
correctly while the rebuild did not — the reverse of what anyone expects.

**How to apply:**

- `tools/package-portable.py` now does the update itself as a build step, so the
  normal path is automatic. Do not remove that step; `--no-update` exists for the
  rare case where an exact revision must be reproduced.
- If the update breaks the build or the tests, **report it and do not ship** —
  Ken's parallel session sometimes has uncommitted work in flight. A path
  dependency once failed to compile mid-rewrite of `redact.rs`; the `file://` +
  branch form takes committed history only, which is why it is that form.
- `BUILD-INFO.txt` records the revision actually linked, read from `Cargo.lock`
  rather than from the engine tree's HEAD. Those differ.

Related: [[ui-verify-competes-for-the-machine]].
