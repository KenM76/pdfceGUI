SECTION = '''### 3b. ★★ The one gap in this file that is a DISCLOSURE, not a tunable

Found **2026-08-19**, while answering the `pdfce` session's `gui`-column re-base.
It is listed separately from the table above because it is a different kind of
thing, and because it is the most serious entry in this document.

**A document whose cross-reference table pdfce REBUILT BY SCANNING opens with
no indication whatsoever.**

`pdfce_core::document::Document::recovery()` returns
`Option<&recover::RecoveryReport>` — `document.rs:1057` — and this shell
**never calls it**. Nothing greps to it. The report carries, among others:

| field | what it says |
|---|---|
| `reason` | why the normal load path was abandoned |
| `file_level_objects` / `objstm_objects` | how much was recovered, and from where |
| `last_wins_collisions` | how many objects were defined more than once, and pdfce picked one |
| `stream_lengths_recovered` | streams whose `/Length` was wrong and was re-derived from the bytes |
| `missing_endobj_recovered` | objects with no `endobj`, terminated by inference |
| `trailer_source` | whether the trailer is the file's own or was synthesized |
| `offset_start` | whether the whole file is shifted from where its offsets claim |

Every one of those is **an inference pdfce made that the operator cannot see**,
which is precisely the half of rule 4 that survives the "never mark the canvas"
clause:

> Inferences the operator *cannot* see — invisible OCR text, a plausible font
> substitution, a best-fit residual, an over-eager snap — still owe an
> off-canvas report. **Render normally; report separately. Both.**

`last_wins_collisions` is the one that should have caught someone's attention
soonest. A non-zero count means **two definitions of one object existed and
pdfce chose between them**. The operator is looking at one of two possible
documents and has not been told there was a choice.

**It is not blocked on anything.** The accessor is `pub`, the report's fields
are `pub`, and it needs no verb. It is a status-line note and a Diagnostics
section, and it is the cheapest high-value surface left in this file.

Recorded here rather than filed as a request because **there is nothing to ask
`pdfce-core` for** — see §1c on how easily a gap on this side gets
mis-recorded as a blocker on theirs.

'''

p = 'NO_SURFACE.md'
s = open(p, encoding='utf-8').read()
anchor = '\n## 4. Zero surface'
assert anchor in s
s = s.replace(anchor, '\n' + SECTION.rstrip('\n') + '\n\n---\n' + anchor, 1)
open(p, 'w', encoding='utf-8').write(s)
print('done')
