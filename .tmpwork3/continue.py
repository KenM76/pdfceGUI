p = 'CONTINUE.md'
s = open(p, encoding='utf-8').read()

s = s.replace(
 '**Written 2026-08-19 at `ae5d0d4`, clean tree, gates 14/14, 1,432 tests.**',
 '**Rewritten 2026-08-19 at `f06ee2d`, clean tree, gates 14/14, 1,431 tests.**\n'
 '**Newest portable build: `OneDrive\\pdfceGUI2`.**')

old_rows = '''| 3 | *“the groups editor popup … too long for some screens so can't close it … should come up in the side bar and be scrollable and each section should be able to fold up like the settings one”* | ⬜ **NEXT.** The growth bug is fixed; the redesign is not |
| 4 | *“no side bar area showing what tool is active and its options”* | ⬜ then this |
| 5 | *“no text editing or adding text on the canvas”* | ⚠ **they exist and he cannot find them** — see §4 |
| 6 | *“still no revision cloud tool”* | ⬜ engine has `MarkupSpec::Cloud`; the GUI command was never built |'''
new_rows = '''| 3 | *“the groups editor popup … too long for some screens so can't close it … should come up in the side bar and be scrollable and each section should be able to fold up like the settings one”* | ✅ **done**, `cbb3469`. `panels::dimension_groups`, six folds, five shut. **Not driven** — he was on the machine |
| 4 | *“no side bar area showing what tool is active and its options”* | ⬜ **NEXT.** §3.2. Fixes #5 too |
| 5 | *“no text editing or adding text on the canvas”* | ⚠ **they exist and he cannot find them** — see §4.1. #4 is the fix |
| 6 | *“still no revision cloud tool”* | ✅ **done**, `c972dfd`. `MarkupKind::Cloud`, `/BE /I 1.0`, its own glyph, ribbon row after Polygon. **Not driven** |'''
assert old_rows in s
s = s.replace(old_rows, new_rows)

# --- section 3 rewrite ---------------------------------------------------
start = s.index('## 3. What to do next, in order, without asking')
end = s.index('## 4. Two things that are true and surprising')
new3 = '''## 3. What to do next, in order, without asking

### 3.1 ✅ Dimension groups — done, `cbb3469`

`panels/dimension_groups/`, mounted in Review and Edit, six foldable sections
of which **one** starts open. Its module header carries the fold policy and is
worth reading before building any other panel: the rule is *what does an
operator need to READ without asking*, not *what do they most often change* —
and the second question is the trap that made this surface taller than his
screen in the first place.

`measure.manage_groups` is the panel's command, with **no** second
`view.panel_*` id. Redact's precedent: one surface, one id, and the mode
taxonomy does the gating for free.

⚠ **Not verified by driving.** `dimension_groups_panel_makes_a_group` was
rewritten to press fold headings and to open-then-shut the Appearance section,
and it compiles and has never been run.

### 3.2 ⬜ A tool panel — what is armed, and its options — **NEXT**

The biggest single win left and it fixes two complaints at once (#4 and #5).
There is still no surface anywhere that says which canvas tool is armed.

- `crate::canvas::tool::CanvasTool` already knows.
- `MODES_AND_PANELS.md` and `RIBBON_IA.md` are the spec — **do not improvise
  the IA.**
- `panels::dimension_groups` is now the worked example of a panel with folds;
  copy its layout discipline, not its content.
- Get a UX critique before designing it. The `pdfce-ui-specialist` agent is
  **not on this session's roster** — dispatch `general-purpose` with the
  specialist's framing instead.

★ The hard part is **the empty state**, which is what the application opens in
and where the operator spends most of their time. An empty panel teaches people
to close it; a panel of *“nothing selected”* placeholders breaks R9. Solve that
first, not last.

### 3.3 ✅ Revision clouds — done, `c972dfd`

`MarkupKind::Cloud`, `markup.cloud` at token 507, `shape-cloud.svg`, ribbon row
directly after Polygon. It joins `is_vertex` and nowhere else in that impl;
`markup::action` takes Polygon and Cloud through one arm; the whole difference
is `/BE /I 1.0` in `spec`, which is Acrobat's default cloud.

`markup_shapes` gained **phase F**, which asserts one field — `kind=Cloud` on
`markup-commit`. A build whose control armed `Polygon` would pass every other
assertion in that check.

⚠ **Not verified by driving.**

### 3.4 ⬜ Three things this session left on the floor, in order of size

1. **Run the driven suite.** Three checks changed today and none has been run:
   `dimension_groups_panel_makes_a_group` (rewritten for the panel),
   `markup_freehand_and_vertex_kinds` (phase F), and everything the new
   Dimension-groups tab might have shifted in the dock for other panels'
   coordinates. **Ask before driving — it needs the machine.**
2. **A Format-tab surface for a selected markup.** `set_markup_style` shipped in
   the engine and takes `/C`, `/IC`, `/BS /W`, `/CA` and `/LE`. This shell has
   no surface for any of it. That is now the largest engine capability with no
   route from this GUI.
3. **`NO_SURFACE.md` §1c wants a sweep.** Every remaining blocker in that file
   that names `pdfce-core` is a claim this project cannot re-check. Two were
   found false in one session. Re-derive the rest, and rewrite each as a dated
   citation.

'''
s = s[:start] + new3 + s[end:]

open(p, 'w', encoding='utf-8').write(s)
print('done')
