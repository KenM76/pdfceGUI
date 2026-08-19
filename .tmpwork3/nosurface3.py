SECTION = '''### 3c. ★ Render diagnostics — **11 of the engine's 65 counters reach an operator**

Measured **2026-08-19**, answering the `pdfce` session's question *"which of the
twelve counters does your diagnostics surface actually read?"* The honest answer
turned out to be a different shape from the question.

`pdfce_render::Diagnostics` (`interpret.rs:192`) has **65 top-level fields**,
several of which are whole sub-structs. This shell reads exactly **eleven**, by
two routes:

| route | counters |
|---|---|
| `app::status::notes::findings()` — a fixed 9-entry table, filtered to non-zero, shown in **both** the status line and the Diagnostics dialog | `contents_streams_unresolved`, `fonts_unsupported`, `images_unsupported`, `glyphs_notdef`, `glyphs_substituted`, `glyphs_supplied`, `oc_sections_hidden`, `deferred_ops`, `unknown_ops` |
| the dialog alone | `tolerated`, `compat_skipped` |

**Fifty-four are read by nothing.** A repo-wide grep for `.diagnostics.<field>`
returns two hits.

#### This is NOT simply "add 54 rows"

Most of the 54 are **measurements** — `images_rendered`, `annotations_painted`,
`ramps_sampled`, `overprint_pixels`. A dialog that listed every one would be the
noise that trains an operator to stop reading it, which is the failure the
9-entry table was designed against (`app/status/notes.rs`'s own header).

**But a subset of them are refusals and silent degradations**, and those are
rule 4's surviving half — *an inference the operator cannot see still owes an
off-canvas report.* Grouped by what an operator would want to know:

| what happened | the counters |
|---|---|
| pdfce was asked to composite and **did not** | `blend_modes_ignored`, `soft_masks_ignored`, `soft_mask_transfer_ignored`, `transparency_groups_knockout_approximated`, `overprint_refused` |
| pdfce could not paint something it found | `shading.refused`, `shading.missing_function`, `shading.function_unloadable`, `shading.function_arity_mismatch`, `color.patterns_unpainted`, `images_codec_unsupported`, `codec_feature_unsupported`, `mask_refused`, `images_mask_unsupported` |
| pdfce **approximated a colour** | `color.tint_transform_not_applied`, `color.separation_all_approximated`, `color.indexed_index_clamped`, `color.indexed_lookup_short`, `color.icc_alternate_used`, `color.icc_device_fallback_used`, `images_uncalibrated_colorimetry` |
| an **annotation** is not on screen | `annotations_without_ap`, `annotations_hidden`, `annotations_appearance_state_missing`, `annotations_placement_degenerate`, `page_content_suppressed` |
| the file is malformed and pdfce coped | `lzw_framing_anomalies`, `codec_geometry_mismatch`, `xobject_depth_overflows` |

`annotations_without_ap` is the one that should go first. It means **a comment
is in the file and is not being drawn** — and on a drawing an operator is
reviewing, a comment they cannot see is worse than a colour that is slightly
off.

The engine also carries `image_notes`, `annotation_notes`, `color.notes` and
`shading.notes` — *per-occurrence* explanations, not counts — which are the
natural body of a Diagnostics section and are read by nothing at all.

#### Not blocked on anything

Every field is `pub` on a report this shell already holds:
`texture.diagnostics` is in hand at `dialogs/diagnostics.rs:202`. This is a
layout decision, not a capability gap.

'''

p = 'NO_SURFACE.md'
s = open(p, encoding='utf-8').read()
anchor = '\n## 4. Zero surface'
assert anchor in s
s = s.replace(anchor, '\n' + SECTION.rstrip('\n') + '\n\n---\n' + anchor, 1)
open(p, 'w', encoding='utf-8').write(s)
print('done')
