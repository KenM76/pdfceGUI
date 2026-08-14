# pdfceGUI — what is built, and what is next

**Updated:** 2026-08-14 (fifth revision). **Scope:** the new shell only. `pdfce-core` and
`pdfce-cli` capabilities live in `D:\Dev\pdfce\docs\FEATURES.md`, whose
**`gui` column is this project's acceptance criteria** — nothing there may
regress at fold-in.

Two lists. The first is what works **today, in the running binary**. The
second is what does not, **in the order it is planned**.

**A row is ticked only when an operator can reach it in a real build.**
Not when the code exists, not when a test passes. Three panels in the old
shell shipped with a body, a rail entry and a diagnostic step and **no
control anyone could click**, and every verification passed for the whole
of their shipped life. That is what this bar exists to prevent.

**Legend:** ✅ reachable · 🔨 in progress · ⬜ planned · ⛔ blocked, with
the blocker named.

---

## Where it stands

| | |
|---|---|
| **Stages complete** | S0 skeleton · S1 `ui-verify` · S2 ribbon · S3 panels + dock · S4 selection · **S5 salvage** (print, forms, icons) · **Phase 3** (navigation, Find, thumbnails, rulers, grid, guides) · **Phase 4** (page display modes) · **Phase 6 begun** (markup substrate, four kinds, Comments panel) |
| **Tests** | 1,241 passing, 0 failing |
| **Gates** | 8 of 8, 0 skipped — `check-theme-colors` added 2026-08-13, with a self-test |
| **Source** | ~106,300 lines across three crates |
| **Commands** | 88 registered · 88 declared-and-deferred (`PLANNED`) |
| **Ribbon surface built** | ~52 % of `RIBBON_IA.md` §5 · 32 groups |

---

## Complete

### Shell and chrome

- ✅ **Ribbon, seven tabs** — File · View · Pages · Edit · Markup · Measure · Tools, plus the contextual **Format** tab that appears on selection
- ✅ **The ribbon is data, not code** — a serializable manifest, which is what makes it customizable *and* reusable by another application
- ✅ **Mode selector** — Read / Review / Edit, right-aligned on the tab row, driving both the tab set and the panel layout
- ✅ **Quick access toolbar** — Open, Save a copy, Undo, Redo, drawn **icon-only** now that a painter exists
- ✅ **Open, Recent, Close** — a second document can be opened. `Ctrl+O` was in the keymap and printed in the tooltip since the ribbon landed and **did nothing**: `DERIVED` held only digits, so the chord could not be spelled
- ✅ **Keyboard chords derive from the manifest keymap** — `keyboard.rs` no longer knows what `Ctrl+0` *means*; it spells the key, looks it up, and returns a command id through the same dispatcher a ribbon click reaches. Rebind it in a customization layer and the keyboard follows
- ✅ **`Ctrl+1/2/3` switch mode**, `Ctrl+0` is actual size — one owner per chord, with a test that fails naming any chord claimed twice
- ✅ **Every ribbon control that can be ON renders pressed** — the last two were the hand tool and marquee zoom, whose armed flag lives in `egui::Memory` rather than on the document, so `conditions()` now takes the `Context`. The obvious alternative, a shadow copy on `PdfceApp`, was refused: it fails as a ribbon claiming Hand while the canvas selects, and each half would be self-consistent so no test would catch it. Published outside the `Status::Open` arm on purpose — with no document the control is greyed **and** pressed, which is exactly "this is the tool you are in, and there is nothing to use it on"
- ✅ **Group captions, enforced by construction** — one closure draws every group, so a caption cannot be omitted
- ✅ **Band overflow** — reserved-space "⌄ N more", proven hit-testable at a width narrow enough to hide groups
- ✅ **Tab-strip overflow** — active tab pinned; collapses to the affordance below ~47 pt rather than hiding the tab you are looking at
- ✅ **Status bar** — render-notes disclosure (closed by default), Actual size / Fit width / Fit page, zoom −/%/+, page ⏴ n/N ⏵
- ✅ **Find** — `Ctrl+F` and a status-bar toggle open a floating box at the page's top right: search, `3 of 47`, Enter / Shift+Enter to step, hits highlighted with the current one distinguished, and Match case · Whole word · Wildcards behind an Options menu. Literal by default — the old shell's bar ran through `find_text`, which patterns, so typing `?` matched every character on the page. An edit clears the highlights and says so rather than drawing a mark that may no longer be over the text it names. Measured at **350 ms** on the 5.6 MB benchmark drawing, which is why nothing searches on a keystroke
- ✅ **Editable page box** — type `37`, press Enter. Commits on Enter or focus loss, clamps out-of-range and says so, rejects non-numeric without discarding what you typed
- ✅ **Three themes** with a *rendered-pair* contrast gate over all five widget states — **and, as of 2026-08-14, actually installed.** This row was ticked from the day the theme module landed and was false for all of it: `Theme::apply` was never called, so every colour an operator ever saw was `egui`'s stock style. Found by sampling pixels, not by a test — see `DEFECTS.md` D10. Enabling it immediately exposed a second defect the stock style had been hiding: the active ribbon tab borrowed `egui`'s *selection* visuals, which this theme points at the translucent **canvas** object-selection tint, so the tab drew pale text on a 27 % wash. It now paints `accent` + `on_accent`, as the mode selector beside it always did. **The preset is not yet choosable** — that is the settings dialog, still unsalvaged

### Panels and dock

- ✅ **Dock** — multiple columns per side, vertical stacks, tabbed groups, draggable splitters
- ✅ **Tab overflow** — reserved-space menu; the old two-panes-per-group cap is retired (nine panels in one stack, tested)
- ✅ **Layout persistence** — `<settings dir>/layout.ron`, debounced 750 ms with a 5 s ceiling, per-item fail-soft loading
- ✅ **Named workspaces**, and **a mode is a workspace** — leaving Edit and returning restores your arrangement, not a default
- ✅ **Scoped reset** — right dock alone, left alone, or all
- ✅ **Bookmarks** — navigation, raising `GoToPage`
- ✅ **Layers** — full `/RBGroups` radio semantics, locked-member handling, Reset to the document's own default
- ✅ **Signatures** — what each signature *covers*, opening with the sentence that pdfce performs no cryptographic verification
- ✅ **Fonts** — inventory, embed status, byte cost, font-folder resolution
- ✅ **Objects** — every object on the page, front-most first; 129,758 on the benchmark drawing
- ✅ **Properties** — read-only facts for a selection
- ✅ **Pages** — thumbnail grid, click to navigate, multi-select (click / Ctrl+click / Shift+extend), context menu of the six page verbs. Selection is marked by a *shape* change plus a written count, never colour alone
- ✅ **Comments** — every annotation in the document, page order then `/Annots` order, reusing `pdfce-cli list-annotations`' ordering by name. `/Widget` excluded because Forms owns it, `/Popup` excluded as an implementation detail of another annotation, and **ce dimensions deliberately *not* excluded** — they are `/Line` annotations, and filtering that subtype would also hide a real `/Line` markup. Every exclusion is disclosed **in numbers**, so nothing is silently omitted; the old shell stated the rule only on the empty case. Reads the session, so unsaved edits show. A markup with no note says so as a fact about the document, not as an error — pdfce cannot write `/Contents` on geometric markup yet, which is filed and accepted
- ✅ **Forms** — fill, **in all three modes including Read**. Reading the session, so unsaved edits show. Read was the operator's call on 2026-08-14 — *Acrobat Reader fills forms in its default view, and replacing it is the stated goal* — and it cost the taxonomy amendment plus a tab move: a command lives on exactly one tab and Read is shown File and View alone, so `edit.form_fill` became `view.panel_forms` in View ▸ Panels. Edit ▸ Forms keeps create, manage and flatten, which is the line the move draws: **filling is not authoring**

### Canvas

- ✅ **Render** — off-thread, generation-counted, cancellable between content-stream operators
- ✅ **Zoom ladder** with a per-page raster ceiling that accounts for `pixels_per_point`
- ✅ **Cursor-anchored Ctrl+wheel zoom** — under 0.01 px drift
- ✅ **Middle-drag pan**, wheel scroll, **hand tool and space-to-pan**
- ✅ **Four page-display modes** — Single · Continuous · Facing · Facing-continuous, as a radio whose active position renders pressed. **Single page is provably unchanged**: its strip is one row with no gap, so the scroll range, centring margin, pan clamp and zoom anchor are the same arithmetic, asserted as an equality against the pre-Phase-4 expression
- ✅ **An undrawn page says so** — its real boundary, a fill that is visibly not paper, and a sentence centred in the part of the page **on screen**
- ✅ **Selection as identity, not position** — page + object + subpath + node, four integers and no coordinate
- ✅ **Selection survives navigation** — zoom, pan, fit, view rotation, page-display mode and tab change, all asserted byte-identical
- ✅ **Multi-select**, marquee, level ladder with Escape ascending one rung
- ✅ **Delete** — click then Delete, verified end to end by `ui-verify` against the real binary
- ✅ **Context menus** — canvas object, canvas empty, Objects row, Pages row, dock tab; a menu with nothing to offer never opens
- ✅ **Find** — `Ctrl+F` or the status-bar toggle. Case, whole-word (with a configurable word rule, because ISO 32000-1 declines to define "word"), and **wildcards off by default**: `find_text` enables them, which is why typing `?` into the old shell's Find bar matched every character on the page. Exactly one `TextSearchOptions` is constructed in the whole crate
- ✅ **Find never searches on a keystroke** — measured at 331–449 ms per search on the benchmark sheet, because the engine re-extracts the document's text every call. Enter, the step buttons, or an option change after a search has run
- ✅ **Stale hits are cleared and said** — an edit clears the highlights, keeps the query, and the readout reads *Document changed*. A quad recorded before a delete can cover different glyphs afterwards
- ✅ **Move** — drag a selection; live ghost preview with no re-raster; multi-select moves as **one** command
- ✅ **Subpath and node moves** — Part rung → `move_subpath`, Node rung → `move_node`; a text run declines rather than borrowing the Object rung's verb
- ✅ **Escape cancels a drag in flight** without committing, and never both cancels and ascends a rung
- ✅ **Eight handles** — drawn, cursors correct, drag consumed so it cannot fall through to a marquee *(commits nothing — see ⛔ resize)*
- ✅ **Dock-tab context menu** — `egui-shell` hands the tab's `Response` to the application; the built-in Close survives for consumers that do not opt in

### Verification

- ✅ **`ui-verify`** — drives the real binary through the OS, asserts on the diagnostic trace **and** the pixels
- ✅ **Three checks**, each of which **fails against the old binary** and passes against the new one
- ✅ **Eight CI gates** — ui-strings and theme-colours (each with a self-test proving it catches its own planted violation), file size, shell purity, fmt, clippy. The theme gate was **missing entirely** until 2026-08-13 and found five unmarked colours on its first run
- ✅ **`PDFCE_DIAG`** — canvas layout, pointer in three coordinate spaces, object counts, named UI rects

---

## Planned, in order

### Next — small, unblocked

| | |
|---|---|
| ⬜ | **Panel toggles** — `view.panel_bookmarks|_layers|_signatures|_objects` and `file.fonts` read as *toggles*; `show_panel` is show-only, so toggle semantics need deciding first |
| ⬜ | **Edit-disclosure surface** — the move verbs return operator-facing disclosures when the surgery changed an operator's *form* (an `re` rectangle expanded into explicit segments). They are traced but **not surfaced**, and Rule 4 says tracing is not surfacing |
| ⬜ | **Scoped reset chooser** — reset currently applies `All`; the scoped variants need three commands and a split-button item kind |

### Phase 1 remainder — making selection mean everything it should

| | |
|---|---|
| ⛔ | **Resize** — `EditSession` has the whole `move_*` family and **no scale verb at all**. Blocked on a capability, not on identity |
| ⬜ | **Object clipboard** — cut, copy, paste, paste-in-place |
| ⬜ | **Format tab contents** — colour, width, style, opacity for a *placed* markup |
| ⬜ | **Editable geometry** — X/Y/W/H in the Properties panel, typed rather than dragged |
| ⬜ | **Undo/redo** — the action funnel exists precisely so this is possible; the command log is not yet surfaced |

### Phase 3 — viewer conventions

| | |
|---|---|
| ✅ | **Hand tool**, space-to-pan — Space is read by the canvas itself, so it needs no keymap entry and cannot be unbound by accident |
| ✅ | **Cursor-anchored discrete zoom** — the rule was decided once, in `canvas::zoom::anchor_point`, and **all five** paths route through it: wheel, the three commands, and the framing verbs. Armed from one statement in the frame, not from the six sites that raise the actions |
| ✅ | **Zoom to selection** — gated on `selection.bounds`, which is *not* `selection.any`: an identity can outlive the box it described, and framing nothing is a jump to the origin that looks like a bug |
| ✅ | **Marquee zoom to region** — one rubber band shared with marquee-select, branched only at release; a zoom marquee never touches the selection and decomposes nothing |
| ✅ | **Recent files** — `recent.txt` beside `layout.ron`, capped at 10, move-to-front; missing entries dropped at display time only, throttled so a dead network path cannot block the UI thread |
| ✅ | **Rulers** — in **points**, or in the document's own unit at its own scale when its dimension sidecar carries one, through the *same* `pdfce-core` `format_measurement` a dimension label uses, so a ruler and a dimension across one span agree to the digit. The gutters take a **constant** bite out of the viewport (R128): switching them on costs exactly one re-fit, and the trace shows it. Zero is the current page's own top-left, which is the frame the pointer readout already reports |
| ✅ | **Grid** — **per page, in page space**, clipped to the sheet, so it scrolls with the drawing and the row gaps between sheets carry none. Every numbered ruler tick has a grid line under it, because both come from one 1-2-5 ladder. Its pitch is bounded on the **drawn** step, which a measurement of the running binary corrected: bounding the labelled step put a line every 1.4 screen pixels on the benchmark A3 sheet, and neither a screenshot nor the suite saw it |
| ✅ | **Guides** — belong to a **page**, dragged out of a ruler, moved on the canvas, deleted by dragging off it or by a double-click. A guide's catch band is registered after every page widget, so grabbing one cannot also rubber-band a selection. Persisted per document in `guides.txt` — a fourth store beside `layout.ron`, `recent.txt` and `page-display.txt` — and a document that has remembered guides opens showing them, because the presence of the work is the preference. **Escape abandons a guide drag and abandons exactly that**: it is the fourth claimant on Escape and it sits above an armed region zoom, because a drag following the pointer this frame is the more transient of the two. Nothing is rolled back — a drag holds a *proposed* position and only release raises `SetGuides` |
| ✅ | **A new panel reaches an operator who upgrades** — a layout records which panels *existed* when it was written, so a genuinely new one appears while one closed on purpose stays closed. Verified against a real layout file written before the Pages panel existed |
| ✅ | **Thumbnail grid** — one tile per frame, on-screen only, current page first, 64-texture cap, and a **hard stop past 400 ms** naming the page and its cost. Measured: 918 ms for the benchmark CAD sheet, 238 ms for one sheet of a 36-sheet set against 58–72 ms for its siblings |
| ⬜ | **Worded decline** — zoom-to-selection with no bounds, and a region zoom the raster ceiling clamped, are *traced* and greyed but never worded. Blocked on the edit-disclosure surface below |

### Phase 4 — page display modes

| | |
|---|---|
| ✅ | **Continuous scroll**, and **Read defaults to it**. Single page stays the default everywhere else and is unchanged — the strip lays out one row for it, so its size, scroll range and centring margin are the same arithmetic they were, asserted as an equality rather than as an intention |
| ✅ | **Facing** and **facing-continuous**, cover page alone. Fit and the raster ceiling became per-*row*: a spread fits as a spread, and the ceiling is a minimum over the row's pages because a spread is two pixmaps rather than one |
| ✅ | **A continuous fit does not depend on where you scrolled to** — under a continuous mode the fit is taken over the document's tightest row, per axis; under Single and Facing it is the current row, which the operator chose. `page_index` is *derived* from the scroll in a continuous mode, so fitting the current row made the zoom depend on the scroll and the scroll depend on the zoom: a mixed-size document oscillated between `zoom=1.4773` and `zoom=0.9559` for as long as the wheel moved. `PROJECT_PLAN.md` R128 in a new place |
| ✅ | Per-document persistence of the choice, so a sheet set does not inherit a report's setting. A third store, `page-display.txt`, beside `layout.ron` and `recent.txt` — `recent.rs`'s own header had already refused to become one |
| ✅ | **Only visible pages rasterize**, one at a time, nearest the viewport centre first, bounded by a texel budget. Measured: a scroll across four pages of a 20-page document whose pages cost ~460 ms each spent **four** renders, none cancelled and none evicted |
| ✅ | **An undrawn page says so** — its real boundary, a fill that is visibly not paper, and a sentence naming the page and its state, centred in the part of the page on screen. Both the fill and the placement were corrected from a screenshot of a driven scroll after every test was green |

### Phase 5 — text editing

| | |
|---|---|
| ⬜ | **Alignment and rotation fixes** — right/centre/justified tails move the wrong way; rotated `Tm` is shifted along the wrong axis |
| ⬜ | **Live re-layout while typing** — the draft is currently ghost text in the wrong typeface at the wrong widths |
| ⬜ | **Reflow reachability** — three gates, one of which demands save-and-reopen *after* showing a correct-looking preview |
| ⬜ | **Multi-run editing** — the edit unit is one show-text operator, so a paragraph split across four `Tj` runs is four edits |

### Phase 6 — markup completeness

| | |
|---|---|
**★ The substrate comes first, and it does not exist.** This shell can place **no markup at all** today: the eight `markup.*` commands are registered and drawn, and every one falls through to `command-unimplemented`. `CanvasTool` has two variants and there is no `canvas/markup.rs`. So Phase 6 is *build the tool, then the kinds* — the six "engine-ready" rows below are blocked on our own substrate, not on pdfce.

| | |
|---|---|
| ✅ | **Markup tool substrate, and four kinds placing** — Rectangle, Ellipse, Arrow, Highlight. One `CanvasTool::Markup(MarkupKind)` carrying a kind rather than one variant per shape, so a tool that is two kinds at once is unrepresentable and the four ribbon controls behave as a radio with nothing enforcing it. Rubber-band preview, `Action::CommitMarkup`, an apply arm calling `EditSession::add_markup`. **A click with no drag places nothing** — the old shell dropped a 120×60 default box justified as "a placeholder the operator will resize", and there is no resize verb here, so that box could only be corrected by undo. Arrow keeps **raw** endpoints rather than a normalised rect, verified by driving a reversed drag and getting the same two page points in the opposite order — which a normalising implementation cannot produce |
| ✅ | **Drag origins are the button-down point** — `Response::drag_started()` fires only once egui has *decided* an interaction is a drag, by which time `interact_pointer_pos()` reports where the pointer has travelled to. Measured at **94 PDF points** of error on A1 at 0.21× zoom. The fix went into `PointerFrame::press_origin`, not into markup, because **the marquee and the move drag had it too**: a marquee that starts late encloses less than the operator drew round, and a move under-shoots by the same distance. Neither had ever been reported, because both fail by a plausible amount rather than a visible one |
| ⬜ | Polyline, polygon, ink — **engine-ready**, but not drag-shaped; each needs its own gesture |
| ⬜ | Underline, strikeout, squiggly — **engine-ready**, but they mark *text* and there is no text-selection gesture yet |
| ⛔ | **Revision clouds** — table stakes for drawing markup, and not even on the old shell's deferred list. **Filed and accepted 2026-08-14, scheduled, not started.** `/BE` appears nowhere in `pdfce-core`, `MarkupSpec` is `#[non_exhaustive]`, and there is no verb taking a caller-built appearance — so all three routes were closed. Landing as `MarkupSpec::Cloud` **plus `Square { border_effect }`, and the rectangular cloud ships first**: drag-a-box-make-it-cloudy is the gesture, the free-form polygon is what people reach for second |
| ⛔ | **Note text** — markup cannot carry `/Contents`. **Filed and accepted 2026-08-14**, landing as `/Contents` + `/T` + `/M` **together**, on the argument that a note with no author reads as a broken panel rather than an absent field. `/M` is engine-stamped (a caller-supplied date is a caller-supplied lie); `/T` is optional and `None` omits the key — **no placeholder author is invented** |
| ⬜ | Style: **width and fill** *(colour works)* — both engine-ready |
| ⛔ | Style: **opacity** — `/CA` is never written. **Filed and answered 2026-08-14: write `/CA` alone, never `/ca` in the appearance stream.** The control must not be drawn until the renderer lands, because a control that visibly does nothing is not a partial feature (P3). The question uncovered a live viewer defect — see **`DEFECTS.md` D9** |

### Phase 7 — measure completeness

| | |
|---|---|
| ⬜ | **Two-line ce dimension** — core and CLI shipped and measured. **Corrected 2026-08-14:** this row used to read *"the canvas gesture has no caller"*, which was false in five documents at once. The gesture is fully built in the **old** shell (`main.rs:23564` calls `pick_line_in_page`, `:23592` takes the pick; `TwoLinePick` in `measure_tool.rs:361`, 814 lines of tests) and pdfce's own ledger marks the row `gui [x]`. The caller that is missing is **ours** — this shell has no measure tool at all. So it is a salvage of Class A `measure_tool.rs` plus its canvas hosting, and it is **not** the cheapest feature in the backlog. See `RIBBON_IA.md` §5.6 |
| ⬜ | **Area** and **Angular** — the two a takeoff needs |
| ⬜ | Count tool and a takeoff schedule |

### Phase 6/S6 — rendering

| | |
|---|---|
| ⛔ | **Deep zoom via `render_page_region`** — the API shipped, but per-viewport regions cost ~700 ms each on a dense sheet. **Blocked on the reusable parsed handle**, filed in `open/request_reusable_parsed_handle.md`. Without it this trades smooth pan for zoom range |
| ⬜ | **`MAX_ZOOM` from measured performance** — not from `f32` (sub-pixel accuracy holds to ~5,000×) and not from the pixmap guard |
| ❌ | ~~Tiled rendering~~ — **cancelled.** A 1×1 *point* region costs 691 ms; a 3×3 ring is a 9× regression. See `BENCHMARK.md` |

### Standing backlog — shell-only work

Exists in `pdfce-core` or `pdfce-cli`; needs a surface, not an engine.

| | |
|---|---|
| ⬜ | Attachments panel · page image export · canvas text selection and copy |
| ⬜ | Imposition in the print dialog · insert blank page · push-button field creation |
| ⬜ | Script-driven-field census · unencrypted-wrapper warning |

### Not salvaged yet

Present in the old shell, not yet carried across. All are Class A or B in
`SALVAGE.md` and none is a rewrite.

| | |
|---|---|
| ✅ | **Print dialog** with live preview — split 2,022 → five files at the three questions it answers. Fixed on the way: stale device capabilities after a printer change, `/Rotate` ignored so a rotated page was planned portrait and rendered landscape, and a texture-name collision |
| ⬜ | **Measure tools** — dimension groups, Taubin best-fit circle, snapping *(1,230 lines)* |
| ⬜ | **Redaction** — mark, review, apply, with the true-removal proof that **exists only in the old shell** |
| 🔨 | **Forms** — **fill** ✅ (Review and Edit modes); create field, flatten and FDF/XFDF/CSV still ⬜. Filling found a genuine `pdfce-core` defect: `fill_refusal()` omits two of the three guards `fill_guards()` applies |
| ⬜ | **Settings dialog** — the spec-ambiguity model, plus the new Render group |
| ✅ | **Icons** — 47 glyphs, rasterized at physical pixel size, tinted from the theme. All 37 keys named by commands resolve, asserted against the live registry. An unknown key draws a **visible slashed mark**, never a blank: the label fallback is decided upstream of the painter, so drawing nothing silently reproduces the blank boxes it exists to prevent |
| ⬜ | **Text editing** — the whole tool |

---

## Not planned, and why

| | |
|---|---|
| ❌ | **A Home tab** — would mirror commands across tabs and re-create the Pass 47.1 defect |
| ❌ | **Automatic reflow on edit** — R75: reflow invents line breaks the file never stated. Make it reachable, not silent |
| ❌ | **Tiled rendering** — measured as a 9× regression |
| ❌ | **JavaScript execution** — standing refusal |
| ❌ | **Provisional styling on the canvas** — Rule 4. Disclosure lives off-canvas; a second rendering path for the same content drifts |
