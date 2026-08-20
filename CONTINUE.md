# CONTINUE — handoff, 2026-08-20 evening

**Clean tree. 16/16 gates. 1,549 + 385 + 10 tests. 60 driven checks.**
**Newest build: `OneDrive\pdfceGUI2`, 2026-08-20 17:26.**
`pdfceGUI1` holds 2026-08-19 17:44 — **the fallback is a day old**, because that
slot has been locked by OneDrive all day and the packager correctly refuses to
touch it. Not a fault; worth knowing.

34 commits today. The previous edition of this file is at
`.tmpwork/continue-old.md` if any detail is wanted back.

---

## 0. ★★★ READ THESE TWO FILES BEFORE ANYTHING ELSE

### `OPERATOR_REQUESTS.md` — the backlog, and the only truth about it

Created today because the operator asked, in effect, where his requests were
going:

> *"Where do you need to put these requests so they just get auto-repeated over
> and over again so I don't have to keep requesting they be done over and over
> again, or can you finally do them?"*

A request made in conversation lives exactly as long as the conversation. He had
been carrying the backlog in his head because nothing else was.

**The five rules, and they are not negotiable:**

1. Every ask goes in that file **the moment it is made**, before any work.
2. **Only he closes a row.** Built + gated + driven moves it to *Shipped —
   awaiting your verdict*. It leaves the file when he says it works.
3. A status is **evidence or the words NOT VERIFIED**. "Done" is not a status.
4. A blocked row names the request file in the engine channel.
5. Nothing is silently rescoped. Half an ask leaves the row open, saying which
   half.

14 open rows, 5 awaiting his verdict.

### `D:\dev\rag\ui-conventions\` — and the gate behind it

Built today, on his question: *"how can you learn from these other programs so
that you can build the missing parts more effectively?"*

The honest diagnosis is that **this was never missing knowledge**. Asked
directly *"what happens when you click inside an unfilled rectangle in a drawing
program?"* the answer comes back right every time. Nothing PROMPTS the question
while the code is being written.

Five gesture classes — `click-selects`, `drag-moves`, `handles`, `text-caret`,
`dialogs` — each a numbered list of rules carrying **where it comes from** and
**the failure mode when it is absent**, in his words where we have them.

`tools/gates/check-conventions.sh` makes every registered surface answer every
row of its class in its own source. **It cannot check behaviour and does not
pretend to** — it checks the question was asked, which is the whole of the
problem. It found **fourteen gaps on its first run**; they are row O14.

**When a new operator report arrives, the question is not only "what do I fix"
but "which rule was I missing, and what else violates it?"**

★ The highest-yield source turned out to be **a mature framework's API surface
read as a specification.** Qt documents `QGraphicsItem::shape()` as *"the
default returns `boundingRect()`; reimplement for a more accurate shape."* That
one sentence is a bug that shipped this morning.

---

## 1. What to do next, in his stated order

He approved this list. Items 1–3 are **not blocked by anything**.

1. **Shift preserves aspect on a resize** — `drag-moves` D5, and *the* resize
   convention. `move_nodes` takes a slice, so it is different scale factors, not
   a new verb. Also Shift-to-axis on the move, handle and dimension drags.
2. **Snapping on a vertex drag** — `drag-moves` D6. The tool that PLACED the
   vertex snaps; the drag that moves it does not, so an operator can pick a
   corner onto geometry and then be unable to put it back.
3. **Dialogs as real OS windows** — `dialogs` G1, via
   `show_viewport_immediate`. Start with Print. Plus **Enter as the affirmative
   default** (G4), which no dialog has.
   - ★ He asked whether this loses our custom options. **It does not.** G1 is
     about the *container*; G2 (use the OS's own dialog) is about the
     *contents*, and for Print we deliberately do not — that dialog carries
     print-comments, the DPI cap, tray-by-size and pdfce's own sizing model,
     none of which `PrintDlgEx` can express. Driver settings already open the
     **native** properties sheet owned by our HWND. That split is correct and
     stays.

Then the rest of O14: unfilled-shape hit testing (only ce dimensions carry a
shape today), grapheme clusters in the caret, selection inside a draft, dialogs
remembering their position.

---

## 2. Blocked on the engine — all filed, none forgotten

`D:\Dev\FeatureRequests\pdfce_FeatureRequests\open\` — **read it every session;
empty means nothing is owed.** 6 open requests, 2 replies awaiting proper
reading.

| what | state |
|---|---|
| **`transform_objects`** — move / resize / rotate an image or text | Accepted in full. `Pass 112.0` shipped only the `Matrix` foundation (`scale`, `rotate`, `about`, `is_invertible`). **`113.0` is the verb and is NOT built** — the reply says plainly *"Nothing is unblocked yet."* 43 object×operation gaps are filed after the operator widened the scope himself. The whole shell side is built and waiting. |
| **Object clipboard** | Filed today, whole rather than the convenient subset, on his instruction: *"I might want all cases so we shouldn't be restrictive in our ask."* ★ Key finding: `EditSession::import_object` (`edit.rs:19367`) already does cross-document graph copying with id remapping and stream re-staging — so the ask is *expose it at object granularity*, not *build a copy engine*. |
| **Text inside a form XObject** | Asks 1 and 2 shipped (`Pass 118.0`) and are **already consumed**: `TextRun::editability()` replaced the shell's hand-rolled guard today. Ask 3 — editing inside forms — is `Pass 119.0`, not built. On a CAD sheet this is the MAJORITY of text: 1,696 show operators inside the form against 3,007 metadata glyphs in the page stream. |

**Two replies in `open/` still need reading properly** —
`2026-08-20-transform-scoped-*` and `2026-08-20-two-of-your-three-shipped-*`.
The second records a defect the engine found while writing it (§2) that has only
been skimmed here.

Closed and archived today: the `add_image` `/Contents` corruption, and the
polyline dimension kind. Both have rows in `INDEX.md`.

---

## 3. Standing rules this day earned, the hard way

- **A trace can say the verb ran. It cannot say the screen changed.** Three
  features were trace-green, gate-green and broken on screen. Every layout,
  repaint or clipping defect has exactly one oracle: a rendered screenshot. Put
  a capture on the failure branch of anything that draws.
- **A check asserting on an ABSENT line must first ask what else happened.**
  `edit-text-refused` is not `edit-text`, so a refusal satisfied a test for "no
  commit" and produced a confident, specific, wrong accusation about working
  code. And anything asserted about a *second* gesture in one process must be
  scoped to lines that arrived after the first one ended.
- **Two derivations of one position agree at first and separate under use.**
  Three instances now: the snap marker off by the scroll origin, the vertex drag
  tracking at `1/zoom`, and the caret measured from document metrics against
  text drawn in another font. `egui::Pos2` is screen, canvas AND page space.
  **The durable fix is typed coordinates — `euclid`, already in the dependency
  tree, MIT/Apache, no new licence — not care.**
- **A blocker is a measurement, and the question you measured is part of it.**
  Three times: *"no verb inserts content"* (markup could round-trip; shipped
  that evening); *"no polyline kind"* (one request away); *"in-place save is
  blocked on crash recovery"* (pdfce writes incrementally — the format already
  WAS the recovery; what was unsafe was the write, a three-line fix nobody had
  made because nobody was asking).
- **A predicate with two claimants must exist exactly once.**
  `text_edit_focused()` cannot see the canvas caret. It cost the Delete key,
  then the space bar — and the sweep found Delete mid-word would have deleted
  the selected *object*. `check-typing-guard.sh` now fails the build on a second
  copy.
- **A comment justifying a shortcut that enumerates its test cases is naming
  the specification of when it is safe.** *"a markup, a move, a form fill"* were
  exactly the three cases where a stale page vector happened to be right.

---

## 4. Environment gotchas

- **`osk.exe` — the on-screen keyboard — covers the ribbon and swallows
  synthetic clicks.** UIPI-protected: `taskkill`, `CloseMainWindow` and
  `ShowWindow` are all refused. The harness places windows at `(780, 40)` to
  clear it and refuses rather than mis-clicking. **A driven failure on this
  machine is a harness question before it is an application one.**
- **`ui-verify` takes the real cursor and keyboard.** He says when he is at the
  PC — he said *"I'm working on the pc"* mid-session today, and everything after
  that was headless until he said otherwise. **Ask, or verify headlessly.**
- **`python tools/package-portable.py` after every keeper build**, then read the
  build stamp out of **both** slots and print the two dates. It destroyed the
  fallback twice on 2026-08-20 before the atomic-rename fix, and reported
  success both times.
- **`cargo update -p pdfce-core -p pdfce-render -p pdfce-print` before every
  build.** The engine ships several times a day.
- `.tmpwork/edit.py` (git-ignored) is the CRLF-safe edit helper. Python
  heredocs mangle backslash-newline; use it or write to a file first.
- **Never `git checkout --` a dirty file.** It destroyed uncommitted work twice.

---

## 5. What shipped today, for context

Multi-document tabs with page drag between them · Shift-to-move · tab reorder
and a right-click menu · imperial sheet sizes · the dimension-group panel
overflow · **dimension drag with live preview** · **perimeter tool** · **length
tool** · **vertex editing with a re-measure disclosure** · **Save (Ctrl+S), in
place, temp-then-rename** · **Ctrl+P** · **a real text caret** (there had been
no index at all) · **the space bar** · **live text preview** · **Escape puts any
tool down** · **panel collapse tabs with a rail back** · dimension hit-testing
on ink rather than bounding box · insert-image fixed on both sides.

Three new gates: `check-typing-guard`, `check-conventions`, and the list-shaped
keymap test that asserts every chord a document application must have.

---

## 6. The operator's standing criticism — keep it in view

> *"it shouldn't take multiple 3 hour sessions each day to figure out how to get
> a cursor to move and edit text on it, or get shortcuts to work for basic
> functions."*

> *"I've never seen a program that doesn't live preview any change, and yet here
> I am having to ask for all the minute details as if you'd never been trained
> on it."*

He is right. The largest bucket of this fortnight's defects is **conventions
nobody audited** — not engine gaps and not hard problems. The conventions corpus
and its gate are the structural answer; use them **before** building an
interaction, not after he reports it.

He also asked whether egui is the wrong tool. The honest answer, recorded so it
is not re-litigated from scratch: egui has **no scene graph**, and four of nine
defects this session were things an item model would have handed us — `shape()`
hit-testing, an editable text item, transform handles, OS-window dialogs. It is
still a defensible choice, switching now would cost months, and the decision
turns on a question only he can answer: **is pdfce-gui a product to sell and
maintain for years, or the tool he uses to drive pdfce-core?** Qt (LGPL-3.0 or
commercial) is the answer to the first; finishing egui is the answer to the
second. Typed coordinates and a small scene/item layer help either way and are
portable to Qt if it ever happens.

And when he reports something, **believe him and go find it.** Every report this
fortnight was precise and correct, including the two that sounded at first like
misunderstandings.
