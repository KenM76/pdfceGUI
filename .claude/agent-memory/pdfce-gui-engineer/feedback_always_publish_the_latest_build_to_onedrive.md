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

**★★ ALWAYS read the build stamp out of BOTH slots after packaging, and do not
trust the tool's own report.**

```bash
for d in pdfceGUI1 pdfceGUI2; do
  printf "%s: " "$d"; grep -m1 "^Built:" "C:/Users/Ken/OneDrive/$d/BUILD-INFO.txt"
done
```

**Why:** the mirror destroyed Ken's fallback build **twice on 2026-08-20**, and
the second time was after a fix. First it cleared the slot before copying;
then, repaired to stage-then-clear-then-swap, it failed identically — because
`shutil.rmtree` is itself non-atomic and a lock on one file leaves everything
already removed removed. Both times the tool printed a message asserting
nothing had been replaced, and both times that was false. `pdfceGUI1` was
restored by hand from `D:\builds\` on both occasions.

The lock is **OneDrive's own sync client**, which is permanent on a synced
folder. The tool now never deletes in place — copy to `.slot-incoming`,
`os.rename` the slot aside, `os.rename` the staging in, then delete — because a
failed directory rename moves nothing. Full finding in
`D:\dev\rag\rust\`.

The two-date check is the only reason either failure was noticed. It costs two
lines and it is not optional.

Related: [[feedback_update_engine_before_every_build]] — `cargo update -p
pdfce-core -p pdfce-render -p pdfce-print` comes first, or the package carries
a stale engine.
