# pdfce GUI — defect register

**Compiled:** 2026-08-12, against `D:\Dev\pdfce` at the release build
dated 2026-08-12 19:54 (`target/release/pdfce-gui.exe`).

Every entry below was verified against source at the quoted `file:line`,
or observed directly by driving the built binary. Screenshots are in
`evidence/`. Nothing here is inferred from documentation alone.

Ordering is by *cost to the user divided by cost to fix*, not by severity.

---

## D1 — The Delete key stops working the moment you click the canvas

**Severity:** critical · **Fix:** one line · **Regression dated:** 2026-08-10

This is the defect reported as *"I can't even click on an object and
delete it by hitting the delete key."* It is real, it is not a
discoverability problem, and the selection half works perfectly.

### Causal chain

1. **Click-select works with no gating.** With no tool armed the canvas
   falls to the modeless branch (`main.rs:17010-17041`) which hit-tests
   and assigns `doc.canvas_selection` (`main.rs:22123-22173`, applied at
   `22614`). No `editing_enabled` check, no armed-tool requirement, no
   Objects-panel requirement. The object visibly selects.

2. **`editing_enabled` is not the culprit.** It defaults to `true`
   (`main.rs:3624`), with the comment *"Editing starts ON… a new
   operator who finds every tool inert would reasonably conclude it is
   broken."* That instinct was right. It is not what is blocking Delete.

3. **The canvas grabs egui keyboard focus on every click**
   (`main.rs:16891-16895`):
   ```rust
   if image_response.clicked() || canvas::primary_drag_started(&image_response) {
       image_response.request_focus();
   }
   ```
   Deliberate, and reasonable — §1.4 wanted the canvas to be a real Tab
   stop rather than an inert image. Because the widget is recreated every
   frame its id stays live, so the focus never lapses.

4. **The keyboard guard tests the wrong thing** (`main.rs:13777`):
   ```rust
   let typing = ctx.egui_wants_keyboard_input();
   ```
   In egui 0.35 that is **not** "a text field is focused". Verified in
   the vendored source at
   `egui-0.35.0/src/context.rs:2884-2886`:
   ```rust
   pub fn egui_wants_keyboard_input(&self) -> bool {
       self.memory(|m| m.focused().is_some())
   }
   ```
   — *any* focused widget, including the canvas itself. The doc comment
   directly above it says *"egui is currently listening on text input
   (e.g. typing text in a `TextEdit`)"*, which is what the name and the
   comment both promise and what the implementation does not deliver.
   This is an egui API footgun, not a careless read.

5. **So the binding is never installed** (`main.rs:13875-13878`):
   ```rust
   if (!tool_active || canvas_delete_target) && !typing {
       pressed(Modifiers::NONE, Key::Delete, Action::DeleteSelection);
       pressed(Modifiers::NONE, Key::Backspace, Action::DeleteSelection);
   }
   ```
   `tool_active == false` and `canvas_delete_target == true` are both
   satisfied. `typing == true` from step 4. The branch never runs.

6. **The deletion logic downstream is correct and simply unreachable.**
   `Action::DeleteSelection` (`main.rs:11205-11290`) → `delete_selected_object()`
   (`main.rs:5250-5310`). Pass 47.0 had already removed an earlier
   `active_tool() == VectorEdit` gate here. That fix is intact; nothing
   calls it.

> **Root cause.** `collect_keyboard_actions` guards its unmodified-key
> bindings with a predicate that means "any widget has focus" rather
> than "a text field has focus", and the canvas takes focus on the very
> click that selects the object — so from the first canvas click onward
> the Delete key is permanently suppressed.

### Blast radius

The same `!typing` guard also suppresses, after any canvas click:

| Keys | Lost function | Line |
|---|---|---|
| `PageDown` / `PageUp` | Next / previous page | `13780-13782` |
| `Home` / `End` | First / last page | `13787-13790` |
| `[` / `]` | Rotate page | `13849-13852` |

So page navigation by keyboard is dead too, for the same reason and
from the same click.

### Why it was never caught

`collect_keyboard_actions` has exactly one test
(`main.rs:28338-28375`), which builds a bare `egui::Context::default()`
with **no widgets** — therefore `memory.focused()` is `None` and
`typing` is always `false`. The single property that breaks in the real
app is structurally absent from the only harness that exercises the
function. Object deletion is covered at the `Action` level, never
through the key.

The regression is self-declared in its own commit message: `e46c3a8`,
2026-08-10, *"a focused text field keeps its unmodified keys —
analysis-confirmed, NOT empirically verified."* It landed two days after
Pass 47.0 fixed the same key by a different route.

### Fix

**Primary** — `main.rs:13777`:
```rust
let typing = ctx.text_edit_focused();
```
This preserves `e46c3a8`'s intent exactly. `text_edit_focused()`
(`egui-0.35.0/src/context.rs:2889-2895`) resolves the focused id and
checks whether a `TextEditState` exists for it. A `DragValue` in
keyboard-edit mode registers its `TextEdit` under the *same* id it
focuses, so property-bar drag values still count as typing.

**Secondary, required alongside** — `main.rs:13341-13348`. Once `typing`
stops masking it, the `canvas_delete_target` escape hatch becomes
reachable while a text tool is armed with a stale `canvas_selection`,
which would steal forward-delete from the caret. The comment at
`main.rs:13872-13874` already promises this cannot happen (*"The text
tools are deliberately NOT given this hole"*) but nothing enforces it:
```rust
Status::Open(doc) => !matches!(
        doc.active_tool(),
        Some(CanvasTool::TextEdit | CanvasTool::AddText)
    ) && (doc.selected_dimension.is_some()
        || doc.entered.is_some_and(|e| e.subpath.is_some())
        || !doc.canvas_selection.is_empty()),
```

**Test that would have caught it, and should be added:** drive
`collect_keyboard_actions` through a context where a widget holds focus
(`ctx.memory_mut(|m| m.request_focus(id))`) and assert `Key::Delete`
still yields `Action::DeleteSelection` when
`CanvasKeys { delete_target: true, tool_active: false, .. }`.

### Two workarounds, until it lands

Both work today and both explain why this survived dev testing (egui's
default is `SurrenderFocusOn::Clicks`):

- Click the object, then click **any ribbon or panel chrome** — that
  surrenders canvas focus without clearing the selection — then Delete.
- Select from the **Objects panel** tree row instead of the canvas. A
  plain `Button` never calls `request_focus`, so Delete works at once.

---

## D2 — Section headings and dock tab labels are invisible in the default theme

**Severity:** high · **Fix:** small · **Evidence:** `evidence/crop_settings.png`, `evidence/crop_tabs_left.png`

Every collapsible section heading in the Settings dialog — *Appearance,
Theme, Colour, Images and transparency, Copying and extracting text,
Pages and printing, Saving files* — renders near-white on light grey. So
do the dock tab labels "Pages" and "Objects". At 1× they are simply not
readable.

### Cause

`theme.rs:434-444` loops over all five widget states setting
`corner_radius`, `bg_stroke` and `fg_stroke`. Then:

```rust
v.widgets.inactive.weak_bg_fill = p.panel;     // 447
v.widgets.hovered.weak_bg_fill  = p.surface;   // 448
v.widgets.active.weak_bg_fill   = p.accent;    // 449
v.widgets.active.fg_stroke = Stroke::new(1.0, p.label_backdrop); // 450
```

`label_backdrop` is `rgba(250,250,250,220)` (`theme.rs:290`). Pairing it
with the accent is correct — light text on an accent fill. But only
`weak_bg_fill` is assigned the accent. **`widgets.active.bg_fill` is
never set at all.** Widgets that paint their background with `bg_fill`
rather than `weak_bg_fill` — `egui_tiles` tab buttons, `CollapsingHeader`
headers — get the near-white foreground on a light background.

### Why CI did not catch it

Two tests look adjacent to this and neither covers it:

- `text_contrasts_with_its_background_in_every_preset` (`theme.rs:521`)
  checks `text` against `surface`/`panel` and `text_muted` against
  `surface`. It never tests `label_backdrop`.
- `label_plates_stay_page_facing_not_chrome_facing` (`theme.rs:553`)
  *asserts `label_backdrop` stays light* — correct for its stated
  purpose (labels sit over the white page) — without checking what is
  actually behind it in chrome.

`tools/check-theme-colors.sh` bans raw `Color32` literals outside
`theme.rs`. It never measures a rendered pair. The gate is structural,
not perceptual.

### Fix

Either set `v.widgets.active.bg_fill = p.accent` alongside line 449, or
stop overriding `active.fg_stroke` and let the accent-filled case handle
itself. Then add a test that asserts every place `label_backdrop` is
used as a foreground has the accent as its background — or, more
robustly, a contrast assertion over the actual `(fg_stroke, bg_fill)`
pairs of all five widget states in all three presets.

---

## D3 — README claims two capabilities that FEATURES.md says are stubs

**Severity:** high (it is a published claim) · **Fix:** edit three words

`README.md:20-22` lists under **"Working today"**:

> …markup annotations; redaction (mark, review and apply); **Bates
> numbering; PDF/A validation and conversion**; digital-signature
> inspection…

`FEATURES.md:29-31` states:

> `to-pdfa`, `validate-pdfa`, `sign` and `bates-stamp` exist in
> `pdfce-cli --help` as **stubs that print "not implemented"**. Not
> ticked anywhere; listed under *Planned*.

Confirmed at `FEATURES.md:224-225`, where both Bates numbering and PDF/A
conformance are unticked on core, CLI **and** GUI.

The same sentence claims printing *"with page placement, orientation,
duplex, copies and n-up/booklet/poster imposition"*. Imposition is real
in the CLI but `FEATURES.md:164` says it has **"No GUI surface at
all"**, and the sentence is describing the application.

`digital-signature inspection` is accurate — inspection only, no
cryptographic verification — and should stay.

This matters more than a normal doc error because the README's own
selling point, two lines above, is that it *"says plainly what does and
does not work today."*

---

## D4 — Text editing: three separate problems behind one complaint

Reported as *"text editing is weird and doesn't just edit the existing
box and move the text correctly as you type plus flow to the next line
doesn't work."* All three parts are correct. They have different causes
and very different costs.

### D4a — The edit unit is one PDF show-text operator, not a text box

**Architectural limit, honestly documented.** Editing genuinely is
in-place on the canvas — there is a real blinking caret painted in PDF
space (`main.rs:17820-17830`), keystrokes are consumed as raw
`egui::Event::Text` (`main.rs:18227-18243`), and no `TextEdit` widget is
in the typing path. But `PendingEdit` pins to one run
(`main.rs:2386-2400`): *"a commit may only span ONE run (§4.4)"*, and a
`TJ` array is one operator.

So a visual paragraph split across several `Tj` runs — the ordinary
output of CAD title blocks, Word and LibreOffice — must be edited run by
run. Dragging a selection across runs sets `cross_run`, which **silently
disables the whole typing loop** (`main.rs:18227`,
`canvas.rs:1489-1510`) behind this notice (`ui_text.rs:5770`):

> *"This selection spans more than one text run … pdfce's first-cut
> editor edits one run at a time. Narrow the selection to edit or format
> it."*

**Second contributor to "weird":** while composing, what you see is not
your glyphs. It is ghost text in an egui proportional font over a
translucent mask (`main.rs:17868-17899` — *"NEVER a re-raster; the real
glyphs appear only after a real commit"*). You type in the wrong
typeface at the wrong widths, then it snaps to reality on Accept.

**To change it:** a multi-run edit request in core that groups runs into
a line or block and re-emits them as a set, plus dropping the
`cross_run` typing lock.

### D4b — Nothing moves as you type; two cases move wrongly on commit

The metrics path is **correct**: advance widths come from real font
metrics — `/Widths` for simple fonts, `/W` + `/DW` for composite
(`text_extract/font.rs:687-700`) — and §9.4.4 is implemented properly
(`edit.rs:1950-1967`) with `Tc`, `Tw` and `Tz` all tracked, `Tw`
correctly restricted to single-byte code 32. The 500/1000 fallback is
the third rung only and is disclosed. `TJ` kerning numbers are preserved
verbatim (`edit.rs:1983-2036`), not dropped.

But: **there is no re-layout per keystroke.** `main.rs:18208-18210` —
*"Typing → build/extend the `PendingEdit` (§6.1). **No core call per
keystroke.**"* Real layout runs once, in `commit_text_edit_draft`. So
"as you type", nothing moves at all. That alone accounts for much of the
complaint.

Two cases are then genuinely **wrong** on commit:

1. **Right-aligned, centred and justified text moves the wrong way.**
   `FollowerDisposition::Pin` exists precisely *"for a justified /
   right-aligned tail that must not move"* (`edit.rs:301-303`), but the
   GUI always passes `EditOptions::default()` — i.e. `Reflow` — at
   `main.rs:12438`, its only call site. Alignment is never detected on
   the edit path.
2. **Rotated or skewed text is shifted along the wrong axis.** The
   follower shift adds the advance delta straight to the translation
   component: `emit_tm([*a, *b, *c, *d, *e + delta, *f])`
   (`edit.rs:1503`), with **no rotation guard**. The reflow path does
   refuse rotated text (`reflow_apply.rs:757-760`); the edit path does
   not. This bites rotated CAD title-block text specifically.

There is also **no collision or margin-fit check anywhere in the edit
path** — the response to an overrun is a disclosure string
(`edit.rs:1527-1534`), not a re-layout.

**To change it:** re-measure and re-render the draft with real metrics
per keystroke; detect alignment and select `Pin` for right/centre/
justified tails; port the rotation guard `reflow_apply` already has.

### D4c — Reflow is unreachable in the sequence a user actually performs

Reflow is implemented and shipped. It is blocked by three gates in a row.

**By design it never happens while typing.** Decision 015 §3.3 and
standing rule **R75**: *"Within-block re-wrap is never automatic on
edit; it is an operator-invoked action producing a DERIVED preview
accepted/rejected before any mutation."* The reasoning — that reflow
invents line breaks the file never stated — is sound and should not be
overturned. But it means the line simply grows past the margin and you
must go and press a button.

**Gate 1.** The "Reflow paragraph…" button is disabled *while you are
typing*: `reflow_button_enabled` is `target.is_some() && !pending_is_some`
(`main.rs:2462-2464`). You must Accept first.

**Gate 2 — the serious one.** Having accepted, reflow then refuses
outright (`edit.rs:4279-4285`):
```
"the page's content was already edited this session; reflow is planned
 against the base content, so save and reopen before reflowing this page"
```
And the **preview still renders**, because it is computed from
`state.page_text` against the base document (`main.rs:18501-18520`). So
you see a correct-looking ghost, click Accept, and only then are refused
(`main.rs:18660-18669`). Edit text → reflow is a dead end that requires
save-and-reopen.

**Gate 3 — an open filed defect.** Pass 33.0 (`ROADMAP.md:43419`). Even
on a fresh open, the auto-detected wrap width is wrong after an
overflowing edit, because the block bbox is a union over its lines and
the one over-long line has already widened it (`reflow.rs:605`:
`req.wrap_width.unwrap_or_else(|| old_bbox.width())`). Measured on the
project's own fixture: a 156 pt block became 930 pt and the re-wrap ran
text off a 612 pt page. Only the *disclosure* option shipped; the
roadmap says plainly that *"an operator who does not read the disclosure
still gets a re-wrap to a width they never chose."*

**Additional refusals that hit real CAD and Word content hard**
(`reflow_apply.rs`): text inside a form XObject (`:658`), more than one
font resource in the block (`:669`), rotated or skewed `Tm`/CTM
(`:757`), more than one text-matrix scale — i.e. **mixed font sizes**
(`:768`), and composite/CID fonts.

**And a tokenisation limit that matters more than any of them**
(`reflow.rs:42-54`): word breaks are found at **real U+0020 space glyphs
only**. Producers that position words with `Td`/`TJ` offsets instead of
emitting a space glyph — extremely common in CAD output — present reflow
with one unbreakable word, so nothing wraps at all.

**To change it:** pick option (b) or (d) for Pass 33.0's wrap width;
make reflow plannable against staged session content so Gate 2
disappears; treat `DerivedWordSpace` as a break opportunity; relax the
uniform-font and uniform-size refusals.

### Why the tests do not catch any of this

`fixtures/synthetic/reflow/reflow.pdf` is 5 pages of one paragraph each,
Courier, emitted as **one `Tj` per line with real space glyphs and a
uniform font and size** (`tools/gen-reflow-fixtures.py:114-124`). The
most complex text-edit fixture, `tm_follower.pdf`, has **two** runs on
one line. No fixture has a paragraph split across many runs, mixed sizes
or fonts in a block, rotated text, or words separated by positioning
rather than space glyphs. Every condition that fails in the field is
absent by construction.

---

## D5 — The keyboard-shortcuts reference omits six live bindings

`ui_text::shortcuts_reference()` (`ui_text.rs:5143-5158`) lists 14
chords. Missing: **Ctrl+F** (Find), **Ctrl+P** (Print), **Ctrl+E** (Edit
text), **Ctrl+Shift+E** (Add text), **F11** (full screen), **Ctrl+H**
(read mode). The doc comment immediately above it
(`ui_text.rs:5138-5141`) says it *must* be kept in step with
`collect_keyboard_actions`.

**Fix:** derive the list from `collect_keyboard_actions`, or add a test
asserting the two agree. A hand-maintained list with a comment telling
you to hand-maintain it has already failed once.

---

## D6 — Review mode does not actually block object deletion

> **Superseded 2026-08-12.** The operator's decision is to remove the
> `Editing on` master toggle entirely and work the way other editors do
> (`RIBBON_IA.md` §5.4, `GUI_ROADMAP.md` Phase 1.7). With no review mode
> there is nothing to enforce, so the fix becomes *delete the four gate
> sites*, not *add the missing fifth*. The analysis below is kept
> because it documents the inconsistency, and because **if D1 ships
> before Phase 1.7 the hole is briefly live** — sequence them together
> or land 1.7 first.

**Latent today; becomes live the moment D1 is fixed.**

Neither `Action::DeleteSelection` (`main.rs:11205`) nor
`delete_selected_object` (`main.rs:5250`) checks `doc.editing_enabled`.
With editing toggled **off**, a canvas selection plus Delete still
rewrites the content stream. Every other authoring surface does check
(`main.rs:7095`, `8169`, `8194`, `16920`).

`main.rs:3225-3235` states the guarantee this breaks: *"no gesture able
to change it by accident."* Add the check before shipping D1's fix,
or the fix turns a dormant hole into a live one.

---

## D7 — Documentation drift

Three items, all small, all in files the project treats as authoritative.

**D7a.** `ROADMAP.md:43419` (Pass 33.0) states as a load-bearing
correction: *"**There is no on-canvas caret at all.** Text entry is a
**panel field**, not an overlaid editable widget."* This is false —
`main.rs:17820-17830` paints a blinking caret in PDF space, and the Pass
14.3 comment at `main.rs:16904` says *"the canvas is its own
caret/selection surface."* It appears to have been written to rebut a
third-party guess and over-corrected. It should be fixed, because
`ROADMAP.md` is declared to win any disagreement.

**D7b.** `FEATURES.md:73` marks reflow `[x]` on core, CLI and GUI with
no caveat, while Pass 33.0 is open and the session gate (D4c, Gate 2)
exists. At minimum it needs a footnote.

**D7c.** `FEATURES.md:119` says form flatten has no GUI surface. It does
— `Action::FlattenForm` at `main.rs:4701`, pushed by a button in the
Forms panel at `main.rs:8112-8116`. The doc understates the build.

---

## D8 — Housekeeping

A stale worktree at
`D:\Dev\pdfce\.claude\worktrees\agent-ad491473a5659e3eb\` contains an
older `main.rs` in which `editing_enabled` defaults differently and a
test asserts `!doc.editing_enabled` (line 23274). It pollutes repo-wide
greps and will mislead the next investigation. Delete it.

---

## Not defects — deliberate choices worth re-examining anyway

These are working as designed. They are listed because the design is
what generates the complaint.

| Behaviour | Where | Why it reads as broken |
|---|---|---|
| Zoom buttons pin the page's **top-left**, not the centre or the cursor | observed; `viewer.rs` ladder | Every mainstream viewer zooms about the centre or the pointer. Zooming in loses your place. Note this is about the *anchor*, not the smoothness — the whole-page-texture model is a deliberate and well-judged trade, see `GUI_ROADMAP.md` § Rendering. |
| The status bar opens with a substitute-glyph census | `main.rs:15576-15960` | The first thing a user reads is the app talking about itself. Excellent information, wrong prominence — put it behind the disclosure triangle that is already there. |
| Dock layout resets every launch | `dock.rs:50-67` — disclosed in-app | Being told your layout will be lost is better than losing it silently, and worse than keeping it. |
| No context menus anywhere | `grep context_menu` → 0 hits | Right-click is where users look for Delete after the keyboard fails them. Fixing D1 without adding these leaves the second-choice path also missing. |
