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
than the count: three were capabilities the engine had shipped **in answer to
this shell's own requests**, which this shell then never consumed, and **two
were settings the operator could change that were honoured by nothing**.

⇒ *A reply arriving is not a capability landing.* The engine session runs in
parallel and answers within the hour; its answers sat unread while this
project's own doc comments still recorded the capability as blocked.

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

## The state at the end of 2026-08-29

**157 `EditSession` verbs. 144 named somewhere in the shell. 13 named nowhere.**
At the start of the audit it was 135 / 22.

The twelve gaps this audit found, and what happened to each:

| Verb | Engine Pass | Status |
|---|---|---|
| `set_markup_note` / `clear_markup_note` | 154.0 | ✅ the Comments panel writes notes |
| `add_markup_with` (opacity) | 81.1 | ✅ Markup ▸ Style ▸ Opacity, one undo entry |
| `set_outline_title` / `delete_outline_item` | 156.0 | ✅ Bookmarks rename and remove |
| `set_quad_point_order` | — | ✅ the fourth settings funnel — **it was a live defect** |
| `delete_pages_with` | — | ✅ the separation policy reaches the delete — **also a live defect** |
| `rotate_annotation` | 155.0 | ✅ a ninth grip on the selection box |
| `rotate_dimension` | 159.0 | ✅ the same grip, routed by kind |
| `attach_file` / `detach_file` | — | ✅ Edit ▸ Insert ▸ Attachments, with extraction |
| `unshare_form` | — | ✅ *"Give this page its own copy"*, seven worded refusals |
| `delete_field_group` / `field_group_deletion_preview` | — | ✅ Forms ▸ Field groups, previewed before the press |
| `signature_impact_of_save` / `changes_structure` | — | ✅ a window before an invalidating save, a note after a preserved one |
| `copy_annotations` | 120.x | ⬜ **open** — asked of the engine; the interim loss is closed |

**Seven driven checks were written for this work and none has run.** A wired
verb is not a verified one; see the caveat at the top.

### ★★★ The two that were live defects rather than missing features

**`set_quad_point_order`.** `Settings::quad_point_order` was parsed, defaulted,
validated, persisted, drawn in the Settings window — and honoured by nothing,
because every session was opened with `EditSession::new(doc)`, which takes the
engine's default.

⇒ ★★ **The lesson is about the shape of the guard, not the field.**
`app::settings` exists precisely to prevent this class and a `syn` check
enforces it — and both were built around **option constructors**. A setting
delivered by a **setter on the session** is invisible to that shape, and the
check reported green for the whole life of the shell. `Settings::separations`
was the same defect one file along: chosen by the operator, reported in the
disclosure after a page delete, and never passed to the verb that would act on
it.

The fix is a fourth funnel (`SettingsExt::open_session`) with `EditSession::new`
on the check's forbidden list. **A guard shaped around one delivery mechanism
cannot see a second one, and the way to find the second is to ask what the
engine offers rather than to re-read the guard.**

### ★★ And one defect the audit did not find, which is worth saying

`Ctrl+S` **saved the file and then panicked the application** — every time,
since 2026-08-20, in the shipped build. It was found by an agent wiring the
signature guard into that arm, not by this register and not by any of the 105
driven checks. `DEFECTS.md` D16 carries the class.

⇒ A verb-coverage sweep answers *"is every capability reachable?"* It does not
answer *"does the route work?"*, and the two questions need different
instruments. This one is cheap; the other is `tools/ui-verify` and it needs the
operator's machine.

---

## The 13 remaining misses, each with its reason

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

### ⬜ / ⛔ What is left, and why

| Verb | Reason |
|---|---|
| `copy_annotations` | ⬜ **Open, and narrowed.** The object clipboard copied a markup by reading it into a `MarkupSpec` and authoring a new one, so everything a spec cannot express was lost — and on 2026-08-28 that came to include the note, the author, the date and the opacity, all of which this shell had just learned to author. `carried_options` closes those four. The general fix (`copy_annotations` → `ObjectClip` → `paste_objects`) is **asked of the engine rather than assumed**, because it is not known whether a `/Popup`, an `/IRT` reply chain or an `/RC` rich-text body survive that path either, and a paste that silently orphans a reply is worse than the loss it replaces. ⇒ The general form: **a copy implemented as a re-author loses ground every time the authoring side gains a key**, silently, in a direction no screenshot can see. |
| `add_named_destination` | ⛔ **Not a gap — a deliberate absence, and the engine agrees.** Nothing in this shell constructs a `Destination`: the one authoring call passes `Destination::Page { view: DestView::Fit }` and cannot pass anything else, because there is no destination chooser. The engine's own note says why that is right: *"a destination chooser offering fits pdfce cannot write would be a control whose options are mostly refusals."* The **reading** side already resolves named destinations, so the Bookmarks panel navigates them in CAD and Word exports today. |
| `field_defaults` | ⛔ **Not a gap.** *"Make another field like this one"* is already how this shell behaves — `FormDefaults::next` carries the previous field's settings forward, with the **name** the one thing that deliberately does not carry. What the verb adds is copying from *any named* field rather than the last one placed, which is a chooser. An operator call, not a hole. |

---|---|
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
