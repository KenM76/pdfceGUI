# pdfceGUI — what is built, and what is next

**Updated:** 2026-08-13 (second revision). **Scope:** the new shell only. `pdfce-core` and
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
| **Stages complete** | S0 skeleton · S1 `ui-verify` · S2 ribbon · S3 panels + dock · S4 selection · **S5 salvage** (print, forms, icons) · **Phase 3 navigation** |
| **Tests** | 1,008 passing, 0 failing |
| **Gates** | 8 of 8, 0 skipped — `check-theme-colors` added 2026-08-13, with a self-test |
| **Source** | ~86,000 lines across three crates |
| **Commands** | 80 registered · 97 declared-and-deferred (`PLANNED`) |
| **Ribbon surface built** | ~48 % of `RIBBON_IA.md` §5 · 32 groups |

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
- ✅ **Group captions, enforced by construction** — one closure draws every group, so a caption cannot be omitted
- ✅ **Band overflow** — reserved-space "⌄ N more", proven hit-testable at a width narrow enough to hide groups
- ✅ **Tab-strip overflow** — active tab pinned; collapses to the affordance below ~47 pt rather than hiding the tab you are looking at
- ✅ **Status bar** — render-notes disclosure (closed by default), Actual size / Fit width / Fit page, zoom −/%/+, page ⏴ n/N ⏵
- ✅ **Editable page box** — type `37`, press Enter. Commits on Enter or focus loss, clamps out-of-range and says so, rejects non-numeric without discarding what you typed
- ✅ **Three themes** with a *rendered-pair* contrast gate over all five widget states

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

### Canvas

- ✅ **Render** — off-thread, generation-counted, cancellable between content-stream operators
- ✅ **Zoom ladder** with a per-page raster ceiling that accounts for `pixels_per_point`
- ✅ **Cursor-anchored Ctrl+wheel zoom** — under 0.01 px drift
- ✅ **Middle-drag pan**, wheel scroll
- ✅ **Selection as identity, not position** — page + object + subpath + node, four integers and no coordinate
- ✅ **Selection survives navigation** — zoom, pan, fit, view rotation, page-display mode and tab change, all asserted byte-identical
- ✅ **Multi-select**, marquee, level ladder with Escape ascending one rung
- ✅ **Delete** — click then Delete, verified end to end by `ui-verify` against the real binary
- ✅ **Context menus** — canvas object, canvas empty, Objects row; a menu with nothing to offer never opens
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
| ⬜ | **Rulers, grid, guides** |
| ⬜ | **Thumbnail grid** — the Pages panel is not registered yet |
| ⬜ | **Worded decline** — zoom-to-selection with no bounds, and a region zoom the raster ceiling clamped, are *traced* and greyed but never worded. Blocked on the edit-disclosure surface below |

### Phase 4 — page display modes

| | |
|---|---|
| ⬜ | **Continuous scroll**, and **Read defaults to it** |
| ⬜ | **Facing** and **facing-continuous** |
| ⬜ | Per-document persistence of the choice, so a sheet set does not inherit a report's setting |

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
| ⬜ | **Revision clouds** — table stakes for drawing markup, and not even on the old shell's deferred list |
| ⬜ | Polyline, polygon, ink, underline, strikeout, squiggly |
| ⬜ | **Note text** — markup cannot carry `/Contents` |
| ⬜ | Style: width, fill, opacity *(colour works)* |

### Phase 7 — measure completeness

| | |
|---|---|
| ⬜ | **Two-line ce dimension** — core and CLI shipped and measured; the canvas gesture has no caller. Cheapest real feature in the backlog |
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
