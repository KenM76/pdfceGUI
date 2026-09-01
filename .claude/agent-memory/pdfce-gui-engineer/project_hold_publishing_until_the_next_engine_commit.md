---
name: hold-publishing-until-the-next-engine-commit
description: 2026-09-01 — Ken asked to hold the OneDrive publish until the pdfce engine lands its next commit; this SUSPENDS the standing publish-on-finish rule until it fires.
metadata:
  type: project
---

**2026-09-01, ~08:05: *"wait until the next commit of the pdfce engine before
publishing."*** The engine HEAD at that moment was `1a13640`; the last published
build (`pdfceGUI2`) was on `042e20e`.

**Why:** the engine session commits several times an hour and Ken tracks it. He
wants whatever is landing next in the build he actually receives, rather than a
build that is superseded before he opens it. He did not name the change, so do
not assume which one it is — wait for the commit, then take everything.

**How to apply:**

- This **suspends** [[always-publish-the-latest-build-to-onedrive]] — the rule
  that finishing work is itself the trigger. Keep finishing work; just do not
  package.
- A persistent `Monitor` on `git rev-parse HEAD` in `D:\Dev\pdfce` is the wake
  signal. When it fires: `cargo update -p pdfce-core -p pdfce-render -p pdfce-print`,
  build, re-measure `FEATURES.md`, then `tools/package-portable.py`.
- **It is a one-shot hold, not a new cadence.** Once that publish goes out, the
  standing rule resumes and this memory should be deleted.
- If a whole working session passes with no engine commit, say so rather than
  publishing anyway or silently sitting on it — the hold was about one specific
  imminent change, and an indefinite wait is not what he asked for.
