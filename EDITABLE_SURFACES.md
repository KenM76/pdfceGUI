# EDITABLE_SURFACES.md — every verb `pdfce-core` implements, and where the operator reaches it

**Written 2026-08-28, in answer to a question this project could not answer from
its own documents:**

> *"confirm that you have built every editable surface into the GUI that has
> been implemented in pdfce"*

`FEATURES.md` says what the GUI does. `NO_SURFACE.md` lists compiled-in values
with no control. `GUI_ROADMAP.md` says what is planned. **None of the three is
keyed on the engine's verb list**, so none of them could answer *"is there a
verb `pdfce-core` implements that nothing in this shell calls?"*

The answer was **yes, twelve times**, and the pattern in the misses matters more
than the count: several were capabilities the engine had shipped **in answer to
this shell's own requests**, which this shell then never consumed. One was a
setting the operator could change that was honoured by nothing.

---

## ★★★ The instrument, and why this file is not a hand-written list

`tools/verb-coverage.py`. It parses `impl EditSession` out of
`D:\Dev\pdfce\crates\pdfce-core\src\edit.rs`, takes every `pub fn` declared in
it, and greps `crates/pdfce-gui/src` for each name.

```
python tools/verb-coverage.py            # the misses, one per line
python tools/verb-coverage.py --all      # every verb with its occurrence count
```

**Re-run it before quoting any number in this file.** The engine moves daily —
five of the rows below were written on the day the verb behind them shipped —
and a register that is trusted rather than re-measured becomes the seventh
stale blocker in a project that has already found six.

### What the measurement is worth, stated plainly

- **A hit means the NAME appears**, not that a reachable operator route calls
  it. A call site behind a condition nothing sets is a hit here and dead in the
  running program. Only `tools/ui-verify` answers that question.
- **A miss is stronger**: no occurrence of the identifier means nothing here
  calls it, full stop.
- **A miss is not automatically a gap.** Roughly half are session queries or
  alternate spellings of a verb the shell calls in another form. That is what
  the table below is for: **every miss owes a reason**, and a reason that is
  merely *"not built"* is a reason to go and look.

---

## The state on 2026-08-28, after the day's work

**157 `EditSession` verbs. 135 named somewhere in the shell. 22 named nowhere.**

The twelve gaps this audit found, and what happened to each:

| Verb | Engine Pass | Status after 2026-08-28 |
|---|---|---|
| `set_markup_note` / `clear_markup_note` | 154.0 | ✅ **Wired** — the Comments panel writes notes now |
| `add_markup_with` (opacity) | 81.1 | ✅ **Wired** — Markup ▸ Style ▸ Opacity, one undo entry |
| `set_outline_title` / `delete_outline_item` | 156.0 | ✅ **Wired** — Bookmarks panel renames and removes |
| `set_quad_point_order` | — | ✅ **Wired** — the fourth settings funnel; see below |
| `delete_pages_with` | — | ✅ **Wired** — the operator's separation policy now reaches the delete |
| `rotate_annotation` | 155.0 | 🔨 in progress |
| `rotate_dimension` | 159.0 | 🔨 in progress |
| `attach_file` / `detach_file` | — | 🔨 in progress |
| `unshare_form` | — | ✅ **Wired** — Format ▸ Selection ▸ *Give this page its own copy*, and the canvas right-click; seven worded refusals; the SHARED CONTENT disclosure now names it |
| `copy_annotations` | 120.x | ⬜ **open** — a fidelity gap in the object clipboard |
| `delete_field_group` | — | ⬜ **open** |
| `field_defaults` | — | ⬜ **open** |

### ★★★ The one that was a live defect rather than a missing feature

**`set_quad_point_order`.** `Settings::quad_point_order` was parsed, defaulted,
validated, persisted, drawn in the Settings window — and honoured by nothing,
because every session was opened with `EditSession::new(doc)`, which takes the
engine's default. An operator who chose *counterclockwise* got reading order in
every markup annotation this shell has ever authored.

⇒ ★★ **The lesson is about the shape of the guard, not about the field.**
`app::settings` exists precisely to prevent this class, and
`no_call_site_builds_its_own_options` parses every file in the crate to enforce
it — and both were built around **option constructors**. A setting delivered by
a **setter on the session** is invisible to that shape, and the check reported
green for the whole life of the shell.

The fix is a fourth funnel (`SettingsExt::open_session`) and `EditSession::new`
on the check's forbidden list. The finding that generalises: **a guard shaped
around one delivery mechanism cannot see a second one, and the way to find the
second is to ask what the engine offers rather than to re-read the guard.**

`Settings::separations` was the same defect one file along: chosen by the
operator, reported in the disclosure after a delete, and never passed to the
verb that would act on it.

---

## The 22 misses, each with its reason

### Not gaps — session queries the shell has no use for

| Verb | Why nothing calls it |
|---|---|
| `into_document` | Consumes the session to get the base document back. This shell's session lives as long as the tab does. |
| `authored_source` | A `base ++ staging` memcpy — ~14 MB per call on the benchmark document. Its own doc comment says it is for `pageops` callers that serialise a whole file anyway and is *"completely unacceptable on a render loop"*. |
| `dirty_set` | What the writer would emit as an incremental update. The shell never needs to know before saving; the writer asks it. |
| `dimension_rects` | Hit-testing ce dimensions from their `/Rect`s. This shell hit-tests through the **decomposition**, which resolves the object under the pointer for every kind at once. Two hit tests would be two answers to one question. |

### Not gaps — alternate spellings of a verb the shell already calls

| Verb | What the shell calls instead |
|---|---|
| `rotate_page_by` | `rotate_pages(&[…], delta)` — the same act for a set rather than for one page, which is what the Pages panel's selection is. |
| `search_and_mark_redactions` | `search_and_mark_redactions_styled` — the styled variant, because a redaction mark whose appearance the operator cannot choose is a mark they cannot see against their own drawing. |
| `mark_redactions_by_pattern` | `mark_redactions_by_pattern_styled`, for the same reason. |
| `copy_annotations` | ⚠ **This one is a real fidelity gap and is listed again below.** The shell calls `copy_objects`, which does not carry annotations, and round-trips a copied markup through `MarkupSpec` instead. |

### ★★ Not gaps — the preview and refusal queries, and why their absence is a real cost

These four are `&self`, side-effect-free, and share one body with the verb they
describe, so `preview(..).is_ok()` **is** the predicate rather than a second
implementation that agrees until somebody changes one.

| Verb | What it would buy |
|---|---|
| `annotation_deletion_refusal` | R83 — *ask before offering the control*. The Format tab's Delete is enabled on a certified document and then refuses. |
| `rename_refusal` | The same for a form-field rename. |
| `annotation_deletion_preview` | The collateral of a delete **before** the click: the pop-up that goes with it, the replies orphaned, the group members promoted. The old shell had a hover-computed version of exactly this. |
| `field_group_deletion_preview` | *"how many fields is this and what are they called"*, before a grouping-node delete. |
| `paste_preview` | Whether a paste will work, before the drop, in the requesting shell's own words: *"a greyed-out menu item needs the answer BEFORE the gesture."* |
| `preview_style_resolution` | What a synthetic bold/italic would do to a run, before applying it. |
| `signature_impact_of_save` / `changes_structure` | ★★★ **What saving would do to the document's signatures.** A signed drawing saved through the wrong path is invalidated, and pdfce reports the invalidation rather than preventing it. This shell says nothing before the save. |

⇒ Each is R9 and R83 quality work rather than a missing capability: the verb
runs either way, and the difference is whether the operator learns the answer
from a greyed control with a hover explanation, or from a refusal after the
gesture. **`signature_impact_of_save` is the one with a consequence that
survives the session** and is the first of these to build.

### ⬜ Real gaps still open

| Verb | What it is, and why it matters here |
|---|---|
| `unshare_form` | ★★★ **Give this page its own private copy of a shared form XObject.** A CAD title block is one form invoked from thirty-six sheets — §8.10.1 names that as the feature's purpose — and this shell can edit text inside a form, so an operator fixing a typo on sheet 12 changes all thirty-six. The engine discloses that after the fact (*"SHARED CONTENT: …"*), and this is the remedy the disclosure should point at. `pdfce-core` withdrew its own "do not offer this" note by name: *"Please un-suppress it rather than leaving the suppression in place — a control withheld on the strength of a note that has since been withdrawn is exactly the kind of thing that stays withheld for months."* |
| `copy_annotations` | The object clipboard copies a markup by reading it into a `MarkupSpec` and authoring a new one, so **everything `MarkupSpec` cannot express is lost**: the note, the author, the date, the opacity, the reply threading. Two of those became losses *today* — the note editor and the opacity control both write keys the clipboard cannot carry, which is a fidelity gap that widens every time the authoring side gains a key. `copy_annotations` returns an `ObjectClip` that owns the annotation and its whole resource closure by value. ⚠ Asked of the engine before rewriting: whether a `/Popup`, an `/IRT` reply chain and a `/RC` rich-text body survive that path, or are the same loss in a different place. |
| `delete_field_group` | Deleting a grouping node and every field beneath it as one undoable command. The Forms panel can delete a terminal field and not a group. |
| `field_defaults` | ⚠ **Re-derived 2026-08-28 and it is NOT a gap — the row is kept because the correction is the useful part.** *"Make another field like this one"* is already how this shell behaves: `FormDefaults::next(kind, &existing)` carries the previous field's settings to the next one, deliberately, with the **name** the one thing that does not carry (two widgets sharing a fully-qualified name are one field). What the engine verb adds is copying from *any named* field rather than from the last one placed, which is a chooser listing every field in the document. That is a real difference and a small one, and it is an operator call rather than a hole. |

---

## What this register does NOT cover, said so nobody reads it as complete

- **`pdfce-render` and `pdfce-print`.** This is the editing surface only.
- **Verbs on other engine types** — `MarkupNote`, `NewTextField`,
  `FieldEdit`, `MarkupStyle` and their builders. The tool scopes itself to
  `impl EditSession` deliberately: those types are *operands*, and an unused
  builder means a field of an operand the shell never sets, which is a
  different and much longer question.
- **Whether a wired verb is reachable.** See the caveat at the top. A hit is a
  name, not a route. `tools/ui-verify` is the instrument for reachability, and
  the standing rule stands: **a capability is not verified until the running
  binary has been driven through it.**
