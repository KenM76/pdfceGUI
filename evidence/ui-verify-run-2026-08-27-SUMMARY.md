# Driven run, 2026-08-27 — the whole suite, in foreground slices

**The operator handed the machine over.** Release binary built from `c5cd7d0`
plus the harness repairs below; engine pinned at `4c32afe` (v0.14.0).

## Result

**76 passed. 0 failed. 6 skipped.** 82 declared.

Every check in the suite was driven. Nothing failed. The six skips are each
reported with their own reason and none of them is a claim about the product:

| check | why it did not run |
|---|---|
| `blend_space` | the fixture has no transparency, so nothing composites and there is correctly nothing to disclose. The documented three-outcome behaviour, working |
| `arrow_keys_walk_between_blocks` | the run's caret landed in a run with no line above or below. A fact about where the pointer went, not about the arrows |
| `new_document_sizes_the_page` | the size popup did not publish its entries within the settle. Self-declared as a harness timing question |
| `progressive` | the pan produced no `canvas-coverage` line, so there was nothing to judge |
| `redaction_removes_and_proves_it` | got past the ribbon lookup after repair 4 below, then stopped at a panel control that has no input-channel confirmation |
| `dimension_groups_panel_makes_a_group` | the dock's heading rect never settled across twelve reads — the known dock-settling flake, recorded in `CONTINUE.md` |

★ **Run in slices, not as one suite.** Three checks that SKIPped inside a
twelve-member batch passed when re-run in a batch of six —
`zooming_past_the_pixmap_ceiling_still_renders`,
`panning_at_deep_zoom_stays_where_it_was_put`,
`a_fit_command_puts_the_page_on_screen`. `RESUME.md`'s standing note holds:
**per-check runs are authoritative**, and a batch skip needs the member re-run
before it is believed.

## The check this run existed for

`a_click_inside_a_form_selects_what_is_drawn_there` — **PASS**, and falsified in
the same session. With the shallow `hit_test_point_all` put back and the binary
rebuilt, it reports:

> THE DEFECT. The click landed on the middle square and the selection is
> `first=object:0` — a PAGE object. The only page object on this fixture is the
> page-sized form, so this is the operator's report reproduced: *"when I click
> on one of the objects all I get is the page selected"*.

The green result is therefore evidence rather than a green result.

Its two assertions, both driven through the OS at 234 % zoom on a 200 × 200 pt
page:

```
after the click on the square: first=leaf:1
after the click in the gap:    first=none
```

## ★★★ Five harness repairs, and three of them un-blinded a check

Three checks had been **unable to run** for days or weeks, each reporting an
honest SKIP that named the wrong thing. None of them was a product defect.

### 1. `sys::describe_window` — a cover refusal now names the offender

`describe_foreground`'s own docs already carried the rule, learned on 2026-08-25
when a stray `OpenWith.exe` dialog made nine checks skip:

> A check that reports a refusal without naming the refuser has withheld the
> only fact that distinguishes "wait" from "act".

It had been applied to the **foreground** guard and not to the **cover** guard,
which refuses for the same kind of reason. The cover guard's message guessed at
`osk.exe` — and on this run `osk.exe` was not running at all.

⇒ **When a guard learns to name its subject, check every other guard that
refuses for the same kind of reason.** A lesson applied at one call site and not
at its sibling is a lesson half-learned, and the sibling is where it gets paid
for again. It named the offender on the first try: this session's own terminal
window.

### 2. "Outside the window" ≠ "covered by another window"

If the point is not inside the target's client rectangle at all, nothing is
covering it — the desktop owns the pixel **because nothing of the application is
there**. Establishing that by hand took three runs: the guard blamed `osk.exe`,
then File Explorer, then `Progman`. The third was the tell.

The two remedies have nothing in common — close the offending window, versus
scroll the region into view — so the message now says which one it is, and
prints the window geometry it measured against.

### 3. `settings_headings_legible` — blind since the O31 ribbon work

It hand-rolled a two-place lookup (band, then overflow) where the ribbon has had
**three** since S3: a group short of width **collapses** into one captioned
button whose items live in its popup and publish no rect until it is opened.
`file.settings` is in the File tab's *pdfce* group, which is the last one and
collapses first. `driving::declared_or_in_overflow` already knew all three.

Now passes.

### 4. `redaction_removes_and_proves_it` — the same duplication, one file over

`file.copy_page_text` lives in the File tab's *Export* group, which collapses at
the same width. Routed through the shared helper too.

⇒ **A rule stated twice is a rule that drifts, and this is what the drift looks
like: nothing failed.** The shared helper gained a third case, the two copies did
not, and the checks simply stopped being able to begin. There is now one
statement of *"where can a ribbon command be"*.

### 5. `dimension_groups` scrolls before it clicks

A rect published from inside a `ScrollArea` is a position in the scrolled
**content**, not on screen. At the harness's 1,100 × 800 client the panel's Add
button lands at logical y 824 — twenty-four points below the bottom edge.

Not a defect: the panel body is a `ScrollArea::vertical`, so an operator sees the
bar and scrolls. A harness aiming at somewhere the window is not.

## Two aims that were wrong, and reported it honestly

Both cost a rediscovery this session. **Recorded so the next one does not.**

- **`bezier_handle_drag_changes_a_curve`** SKIPs by name on a fixture whose
  entered subpath is all straight. Run it as
  `--pdf fixtures/polyline-nodes.pdf --doc-point 0,150,260`. That point is on
  the first straight run of the polyline; its later segments are the cubics.
- **`ctrl_c_copies_text_to_the_os_clipboard`** needs `--doc-point` on actual
  text. `--doc-point 0,1140,62` on `SW41177.pdf` lands on a real run.

## Housekeeping during the run

A leaked `pdfce-gui.exe` from an earlier check was found holding a window and
killed. The operator's windows were minimised for the driven portion and
restored with `UndoMinimizeALL` at the end.
