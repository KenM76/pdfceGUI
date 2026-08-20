# Operator requests — the standing backlog

> **Ken, 2026-08-20:** *"Where do you need to put these requests so they just
> get auto-repeated over and over again so I don't have to keep requesting they
> be done over and over again?"*

**Here.** This file is the answer, and this section is the contract.

## The contract

1. **Every request you make goes in this file, the moment you make it**, before
   any work starts on it. Not into a chat reply, not into a session summary,
   not into an agent's memory — into this file, which is in git, on disk,
   backed up, and read at the start of every session.
2. **Only you close a row.** I may move a row to *shipped-and-driven*; the row
   does not leave this file until you have used it and said so. A row I believe
   is done but you have not confirmed sits under **Shipped — awaiting your
   verdict**, not deleted.
3. **A row carries evidence, not a claim.** *"Done"* is not a status. The
   status is either a driven check by name, or a dated note saying exactly what
   was verified and how. If nobody drove it, it says NOT VERIFIED, in those
   words.
4. **A blocked row names what blocks it and where that is filed.** If it is an
   engine gap, the row names the file in
   `D:\Dev\FeatureRequests\pdfce_FeatureRequests\open\`. A row that says
   "blocked" with nothing behind it is a row I have not done the work on.
5. **Nothing is silently rescoped.** If I ship half of what you asked for, the
   row stays open and says which half.

## Why this file and not something cleverer

The failure this fixes is real and it is mine: a request made in conversation
lives exactly as long as the conversation. Sessions end, context is compacted,
and an ask made in turn three of a long session is gone by turn thirty — which
is why you have had to repeat yourself. The agent-memory system is for *how you
work*; it is the wrong shape for *what you asked for*, because memories get
summarised and requests must not.

A file in the repository has none of those properties. It is read at session
start alongside `PROJECT_PLAN.md`, it survives compaction because it is on
disk, and its history is in git so a row cannot quietly disappear.

---

# OPEN

## O1 — Editing text on the canvas, and editing text in a text box

**Asked:** repeatedly; restated 2026-08-20 — *"Still no editing text on top of
the canvas. Or editing text on a text box."*
**Status:** OPEN. Under investigation 2026-08-20; nothing claimed.

Two distinct things and I have been conflating them, which is probably part of
why it keeps coming back:

- **(a) Editing text that is already on the page** — click a run of existing
  page text, get a caret, retype it.
- **(b) Editing the text inside a text box you have added** — a `/FreeText`
  annotation, or a text object this shell authored: double-click it, get a
  caret in it, retype.

**(a) — driven 2026-08-20 on your own CAD drawing. It is reproduced, and it is
not the shell.** `text_edit_on_a_real_drawing`:

```
text-edit-caret kind=Edit page=0 run=44 len=1     ← the caret lands on real text
text-edit-typing draft=true text_events=1 len=2   ← keystrokes reach the draft
text-edit-typing draft=true text_events=1 len=3
text-edit-plan page=0 run=44 disposition=Pin reason=Rotated pinned=true
edit-text-refused page=0 n=1
  detail=text to edit ("p") was not found in an editable run on the page
```

So the tool arms, the caret lands on the right run, the typing arrives, the plan
is built and the commit reaches the engine — **and `pdfce-core` refuses it.**
From your chair that is precisely "the tool responds and the page does not
change".

Root cause under investigation 2026-08-20. It will be either an engine gap (a
request, filed) or a wrong call on my side (a fix). Either way this row records
the answer.

★ And a second defect found in the process, on my side: the driven check for
this was reporting *"THE COMMIT NEVER REACHED THE ENGINE"* — which was **false**.
`edit-text-refused` is not `edit-text`, so a check asserting on the absence of a
line produced a confident, specific, wrong accusation about working code. Fixed:
a refusal is now asked about first and quoted verbatim.

**(b) — not yet driven.** Nothing claimed.

## O2 — Cut / copy / paste of PAGE CONTENT (`Ctrl+X` / `Ctrl+C` / `Ctrl+V`)

**Asked:** repeatedly; restated 2026-08-20.
**Status:** OPEN — partially shipped, and the part that is missing is the part
you are pressing the key for.

- Markup / comments: **works** (`edit.cut`, `edit.copy`, `edit.paste`, all
  three chords bound and driven).
- Swept text: **works** (copy to the system clipboard).
- **Page content — a path, a line, a block of drawing:** refused. The
  refusal is a sentence on the status row, which from your chair reads as
  "the shortcut does nothing".
- A placed image: can be pasted in, cannot be picked back up.

**Blocked by the engine.** `pdfce-core` has no verb that inserts page content;
this was measured over `edit.rs` and the narrower half was corrected on
2026-08-19 (`note_correcting_our_own_clipboard_blocker_and_one_small_ask.md`).
The note ended *"we are not asking for it yet"* — **that was my call to make and
it was the wrong one**, because you had already asked. A proper request is owed
and this row is not closed until the round trip works on a path.

## O3 — Perimeter measuring tool

**Asked:** 2026-08-20 — click around to make a shape, sum the segment lengths
into one dimension; right-click to add segments; drag the endpoints to adjust
the shape; all the scaling options of the other dimension tools.
**Status:** OPEN. Blocked on the engine, filed the same day:
`request_there_is_no_polyline_dimension_kind_so_a_perimeter_cannot_be_measured.md`.

`DimensionKind` has three variants (Linear, Circular, Angular), all fixed-arity,
and no polyline. The pick machine, the preview, the right-click menu and the
vertex handles are mine to build and I am not waiting on the engine for the
design — but nothing can be committed to a document until the variant exists.

## O4 — Insert image does nothing

**Asked:** 2026-08-19, restated 2026-08-20 — *"No it always hasn't worked."*
**Status:** OPEN, **cause found 2026-08-20**, blocked on the engine.

You were right and it was never a misunderstanding. `EditSession::add_image`
returns success, the status bar reports the resolution, and the picture is not
on the page — because `add_image` corrupts the page's `/Contents` when it is an
indirect reference to an array, which is what every CAD-exported sheet uses.
The saved file then cannot be reopened by pdfce at all.

Eight-line repro and the exact bytes are in
`request_add_image_corrupts_a_page_whose_contents_is_an_indirect_array.md`.
Works on a page whose `/Contents` is a plain stream; fails on one whose
`/Contents` is an array. That is the entire difference.

## O5 — Horizontal / vertical dimension constraint, from a drop-down

**Asked:** 2026-08-20.
**Status:** OPEN, not started. `LinearPick::constraint` exists in the shell and
is never written outside tests — there is no control for it. No engine work
needed.

## O6 — The scale ratio field follows the dimension you set

**Asked:** 2026-08-20 — *"when I set the dimension the editable ratio one shown
should change to match it."*
**Status:** OPEN, not started.

---

# SHIPPED — awaiting your verdict

Rows here are built, gated and driven. **They stay here until you have used
them and said so**, then they move to CLOSED with the date you confirmed.

## S1 — Move a placed dimension, with a live preview

**Asked:** 2026-08-20. **Shipped:** 2026-08-20, commit `469d4d7`.
Press inside a selected ce dimension and drag: the dimension line follows,
previewed from the same function a committed dimension is drawn from, and the
release commits `place_dimension`. The measured points never move, so the
printed number cannot change. Linear dimensions only — angular refuses at the
press rather than starting a drag that could not finish.
**Verified:** unit tests only (4). **NOT yet driven through the harness.**

## S2 — The measure sidebar no longer hides half its controls

**Asked:** 2026-08-20. **Shipped:** 2026-08-20, commit `c2de963`.
The group list was a four-column grid 209 pt wider than its own column, clipped
with no scrollbar in that axis. Now a block per group, and the panel measures
its own overflow so it cannot come back quietly.
**Verified:** `no_row_in_this_panel_outruns_a_narrow_dock`, which failed at
209 pt before the change. **NOT yet looked at on screen.**

## S3 — `Ctrl+P` opens Print

**Asked:** 2026-08-20. **Shipped:** 2026-08-20.
It had never been bound. Print was on the ribbon, in the QAT and in a menu, so
every surface that lists commands showed it and only the keyboard did not.
`the_keymap_offers_the_chords_a_document_application_must` now asserts the whole
list of universal chords rather than this one line.
**Verified:** unit test. **NOT yet driven.**

## S4 — Imperial sheet sizes

**Asked:** 2026-08-19. **Shipped:** 2026-08-19, commit `815036a`.

## S5 — Multiple documents, page drag between them, tab reorder

**Asked:** 2026-08-19/20. **Shipped:** 2026-08-20.
**Verified:** driven — `document_tabs`, `page_drag_between_documents`,
`tab_reorder`.

---

# CLOSED

*(Nothing yet. A row lands here only when you have said it works.)*
