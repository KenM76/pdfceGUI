---
name: disk-is-tight-and-target-grows-unbounded
description: D: runs near-full and this project's target/ reaches 50GB+ of stale build cache within a week; clear it periodically without being asked
metadata:
  type: project
---

`D:\` is a 954 GB volume that sits in the low-90s percent used. This
project's `target/` grows to **50 GB+ within about a week of active
work** and cargo never reclaims any of it, so the project alone can be
the difference between comfortable and out-of-space.

Measured 2026-08-21, after ~8 days of work: `target/` was 56 GB, of
which `target/debug/incremental` was 30 GB across **305 separate
generation directories** and `target/debug/deps` was 24 GB of which
20 GB had not been touched in three days. `target/release/deps` was
2.6 GB and comparatively well-behaved.

**Why:** cargo's incremental cache and dep artifacts are append-only in
practice — every rebuild writes a new generation beside the old ones and
nothing garbage-collects. Debug is the offender because the gates
(`clippy --all-targets`, `cargo test`) build debug constantly while
almost nothing *runs* debug; the ui-verify harness drives the release
binary.

**How to apply:** treat `rm -rf target/debug` as routine housekeeping,
not a destructive act — it costs one full debug rebuild and nothing
else. Do the same for `target/doc` (regenerable) and for any
`target/ui-verify-*` scratch directories older than the current line of
work; anything worth keeping was already copied into `evidence/`, which
is tracked and small. **Leave `target/release` alone** — a release
rebuild is expensive and the harness depends on that binary; selectively
deleting files out of `release/deps` risks a half-valid cache for a
~2 GB return that is not worth it.

Expect the reclaimed figure to come in **well under** what `du` predicts
— on 2026-08-21 `du` accounted 53 GB deleted and `df` showed 29 GB
returned, with no NTFS compression on the folder to explain it. Quote
`df` before and after, not `du`, when reporting how much was actually
freed.

One-shot Python patch scripts have twice ended up committed at the repo
root (`patch15.py`, `patch_pixels.py`, `.tmp_text.py`, removed
2026-08-21). If a scratch edit-applier is needed, write it under
`.tmpwork/`, which is gitignored.

Related: [[always-publish-the-latest-build-to-onedrive]] — the packaging
step needs a valid `target/release`, another reason not to clear it.
