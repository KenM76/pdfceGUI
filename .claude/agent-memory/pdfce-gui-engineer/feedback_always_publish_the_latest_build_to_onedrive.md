---
name: always-publish-the-latest-build-to-onedrive
description: Every build worth keeping must be packaged and mirrored to OneDrive, alternating between the pdfceGUI1 and pdfceGUI2 slots — Ken picks it up from there.
metadata:
  type: feedback
---

**Every time you produce a build worth keeping, package it and mirror it to
Ken's OneDrive, alternating between the `pdfceGUI1` and `pdfceGUI2` slots.**

```bash
python tools/package-portable.py
```

That one command already does all of it — it builds the portable folder under
`D:\builds\`, mirrors it to `C:\Users\Ken\OneDrive\pdfceGUI<n>`, and **picks
the older of the two slots automatically**, so the alternation is a property of
the tool rather than something to track by hand. It also preserves the
`userdata/` folder already in the target slot, so settings survive a swap.

**Why:** stated by Ken on 2026-08-19. OneDrive is how he actually gets the
build — he runs it from there, on this machine and others. A build that exists
only in `target/release/` or `D:\builds\` has not reached him. The alternation
is the point: the previous build stays intact in the other slot, so if the new
one misbehaves there is always a working one beside it to fall back to and to
compare against. That is the same fallback property the whole project rests on
(`D:\Dev\pdfce\` keeps shipping while the rebuild happens), applied to the
day-to-day.

**How to apply:** run it at the end of any session that landed working changes,
and after any fix Ken might want to try immediately. Do not ask first — it
writes only to `D:\builds\` and the OneDrive slot, never to a repository. Say
in the report which slot it went to and which one holds the previous build, so
he knows which is which without opening either.

Two things that make the report useful rather than noise:

- **name the slot**, not just "packaged" — `pdfceGUI2`, and note that
  `pdfceGUI1` holds the previous one.
- if the build is one he asked for specifically, say what changed in it, since
  the slot name carries no version information.

Related: [[feedback_update_engine_before_every_build]] — `cargo update -p
pdfce-core -p pdfce-render -p pdfce-print` comes first, or the package carries
a stale engine.
