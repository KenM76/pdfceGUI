---
name: a-measurement-of-the-wrong-surface-looks-exactly-like-a-broken-one
description: Before believing a pixel check's verdict, prove it sampled the surface it names — a wrong-surface reading is indistinguishable from a defect.
metadata:
  type: feedback
---

A pixel measurement that landed on the wrong surface **reads exactly like a
measurement of a broken one**. Before acting on a contrast, colour or layout
verdict, establish that the sampler was pointed at the thing it names.

**Why:** on 2026-08-21 the same mistake was made twice within an hour.

- **The wrong WINDOW.** After thirteen dialogs became real OS windows, a
  contrast check went on capturing the *application's* window and measured the
  drawing where the dialog used to be — reporting a confident **1.51:1** about
  two headings that actually render at **15.07:1**.
- **The wrong PART of the right one.** `diag::ui_rect_visible` published any
  region that *intersected* the clip, on the stated argument that a
  half-scrolled heading is still worth measuring. A heading two points inside a
  scroll area's bottom edge measured **1.53:1**, read off the anti-aliased top
  rows of glyphs whose bodies had been clipped away, at 5.3 % coverage.

Both verdicts were specific, quantitative, and about working code.

**How to apply:** when a pixel check fails, ask *"what did it sample?"* before
*"what is broken?"* — read the artefact PNG, which every such check writes. In
the harness, capture the window a frame describes rather than the application's,
and raise it by matching **client origins**, never by z-order (the raise is
about to change z-order). In the application, a diagnostic channel that
publishes a region nobody can read is not being generous — it is manufacturing
false failures; require a region to be *mostly* inside its clip, not merely to
touch it.

Related: [[ui-verify-competes-for-the-machine]].
