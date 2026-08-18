//! # `shell::commands::reach::register` — the allow-list, and only the allow-list
//!
//! **The DATA half of [`super`].** Split out on 2026-08-17, when the file
//! crossed rule R2's 1,500-line ceiling for the third time in a week.
//!
//! ## The seam is real, and it is the one the file kept re-discovering
//!
//! `reach.rs` does two things that change for entirely different reasons:
//!
//! | half | what it is | changes when |
//! |---|---|---|
//! | **this file** | the register — every registered command with no dispatch arm, and *why* | a command is wired, deferred, or its reason expires |
//! | `mod.rs` | the CHECK — a `syn` parse of the dispatcher's `match`, the guard evaluation, and the tests | the dispatcher's shape changes, or the check gets sharper |
//!
//! The second is machinery and is nearly static. The first is a **living
//! document** that grows a paragraph every time somebody explains why a control
//! is inert and shrinks by an entry every time somebody fixes one — and every
//! one of those paragraphs is prose, so it is the half that pushes the line
//! count. Three separate sessions have trimmed a sentence out of this register
//! to get the file back under the ceiling, which is exactly the pressure R2
//! warns about: *"when a file approaches the limit, that is the signal to find
//! the seam, not to raise the limit"* — and trimming the reason is the worst
//! available response, because the reason is the entry's whole value.
//!
//! ## What did NOT move
//!
//! The counts. `super::tests::the_p3_tension_is_counted` still pins both
//! figures, and it reads them from here through the ordinary path — so the
//! number quoted in `super`'s header and the length of the list below still
//! move together or fail.

// ===========================================================================
// THE ALLOW-LIST
// ===========================================================================

/// **Registered, deliberately without a dispatch arm, and why.**
///
/// ★ This list is the deliverable, not the leftovers. Every entry is a control
/// an operator can press today that does nothing, and writing the reason down
/// is what forces somebody to say *why it is drawn at all*.
///
/// # How this differs from `super::super::manifest::PLANNED`
///
/// `PLANNED` names commands that are **not registered** — ids `RIBBON_IA.md`
/// mentions that this build does not have, so nothing draws them and nothing
/// can invoke them. These are the opposite and worse state: **registered, drawn
/// and inert.** The two lists are disjoint by construction, because everything
/// here is in the registry and nothing in `PLANNED` is; that is asserted by
/// [`tests::no_scaffolded_command_is_also_planned`].
///
/// # What an entry has to carry
///
/// The **reason**, not the name. Where a reason already exists in the codebase
/// — a doc comment at the registration, a table in `app::dispatch`, a
/// `SALVAGE.md` class — the entry cites it rather than inventing a second
/// wording that can drift from the first.
///
/// # ★ Several of these should probably not be drawn yet, and this list is
/// where that becomes visible
///
/// `RIBBON_IA.md` P3 says an unavailable capability renders **nothing**, and a
/// control that is drawn, enabled, pressable and inert breaches it more
/// severely than a greyed one does — a greyed control at least explains itself
/// on hover. Entries whose honest answer is *"this should not be on the ribbon
/// yet"* are marked **★ P3** in their reason. Removing them is a taxonomy
/// decision and is the operator's, not this module's; what this module can do
/// is make the count impossible to lose track of, which
/// [`tests::the_p3_tension_is_counted`] does.
///
/// # Ordering
///
/// Registry order, so this list and [`super::catalog::all`] read side by side.
/// [`tests::no_scaffolded_entry_is_stale`] asserts every entry is registered,
/// is genuinely unreachable, and carries a reason rather than a restatement of
/// its own id.
pub(crate) const SCAFFOLDED: &[(&str, &str)] = &[
    // ===================================================================
    // FILE
    // ===================================================================
    (
        "file.export_dxf",
        "★ P3 — NO RECORDED REASON ANYWHERE, and that is the entry. `RIBBON_IA.md` §5.1 \
         marks the row G, meaning the capability exists in the shell this one replaces; \
         neither `FEATURES.md`'s backlog, nor `manifest::PLANNED`, nor `manifest::DIRECTED`, \
         nor any doc comment says why the control was drawn before the arm was written. The \
         only comments at its registration are about which glyph it shares and which group \
         it sits in. Scaffolded by omission rather than by decision.",
    ),
    (
        "file.export_form_data",
        "Blocked on a writer that does not exist. `FEATURES.md`'s Forms row: “fill ✅ (all \
         three modes since 2026-08-14); create field, flatten and FDF/XFDF/CSV still ⬜” — \
         and this command IS the FDF/XFDF/CSV half. Filling a form and serialising the \
         values to a foreign format are different builds; the first shipped.",
    ),
    (
        "file.shortcuts",
        "Blocked on the same salvage, with a defect attached. The reference this would show \
         lives in the old shell's 7,912-line `ui_text.rs`, and `SALVAGE.md`'s row for that \
         file carries the instruction rather than a straight copy: “Fix `shortcuts_reference()` \
         — it omits six live bindings (DEFECTS.md D5) — and derive it from the keyboard map \
         so it cannot drift again.” Salvaging it unfixed would import D5.",
    ),
    // ===================================================================
    // VIEW — the seven settings-shaped controls
    //
    // ★ These seven already have a register of their own, and it is the
    // one place in this list where a reason for DRAWING the control was
    // written before the control was drawn: `manifest::DIRECTED`, eight
    // entries, each naming the value set and the default the operator
    // specified when this shell was commissioned.
    //
    // Its argument, verbatim: "P3 exists so an operator is never shown a
    // control that does nothing. These eight are settings rather than
    // actions, every one of them has a specified default, and **a setting
    // showing its default is not a stub**."
    //
    // ★ THE PREMISE OF THAT ARGUMENT IS NOT YET TRUE, and it is worth
    // saying here rather than leaving for whoever wires the first one.
    // None of the seven shows a value: they are registered as ordinary
    // commands, drawn by `band::command_button` as ordinary buttons, and
    // a press produces `command-unimplemented`. "A setting showing its
    // default is not a stub" is a defence of a control that DISPLAYS
    // something; a button that displays nothing and does nothing is not
    // the thing being defended. The argument becomes true the day these
    // render as value controls, and `DIRECTED`'s own closing sentence
    // already says what to do if it does not: "the fix is deleting eight
    // rows from one list".
    //
    // They are therefore cited to `DIRECTED` rather than marked ★ P3 —
    // the decision to draw them is recorded, argued and the operator's —
    // with the gap between the argument and the artefact stated at each.
    // ===================================================================
    // ===================================================================
    // VIEW — the two that are not settings
    //
    // ★ It was four until 2026-08-15. `view.read_mode` and `view.fullscreen`
    // were wired that day — see `app::window` — and their entries are gone
    // rather than reworded, which is what `no_scaffolded_entry_is_stale`'s
    // middle assertion exists to force. What is left is genuinely blocked
    // (`show_points`, on an unsalvaged tool) and genuinely contested
    // (`sidebar`, a taxonomy question).
    // ===================================================================
    (
        "view.show_points",
        "★ P3 — and the reason was found on 2026-08-15 rather than written: **there is nothing \
         for it to show.** Its own tooltip says what it governs — “the editable points of every \
         part of the object you are working inside… Points always appear for the selected part” \
         — which is the old shell's node-mark population, where `canvas.rs` carries `NODE_MARK_PX`, \
         `NODE_MARK_OTHER_PART_PX` (“drawn only while the ‘show points’ view option is on”) and \
         `MAX_DRAWN_NODES`. **This build draws no anchor mark at any rung.** `canvas::overlay`'s \
         whole output is outlines, eight grips, a move ghost, find hits, a text wash and a \
         marquee, and `CanvasTargetProvider` offers `nearest_node` — a *query* — with no way to \
         enumerate a part's anchors to paint them. So the baseline the toggle is defined against \
         does not exist, and the substrate is unsalvaged: `SALVAGE.md` lists `vector_edit_tool.rs` \
         (“Node/handle editing. Keeps.”) and a ~1,200-line “Vector object editing, node/handle” \
         unit, neither brought across. It was ALSO checked against `chrome_for_command` and does \
         not belong there: every `ViewChrome` variant is a `crate::viewer::ViewState` field the \
         canvas reads in order to *draw* something, and joining that family would add the one \
         member whose flag nothing reads — a toggle that toggles nothing, which is worse than \
         an inert button because it looks like it worked.",
    ),
    (
        "view.sidebar",
        "★ P3 — the only justification on record is provably stale. `panels/pages/mod.rs` \
         quotes the old shell's note — “page thumbnails are the sidebar rail's first pane \
         and have no independent toggle; the rail toggle shows the rail” — and then \
         answers it: “**There is no sidebar rail in this build — there is a dock.**” \
         `manifest/view.rs` still claims this id is in PLANNED, which is false: it is \
         registered, drawn first in View ▸ Panels, and available with nothing open.",
    ),
    // ===================================================================
    // PAGES — the three the dispatcher already argues in place
    // ===================================================================
    (
        "pages.split",
        "★ P3 — already recorded as such. `app::dispatch` carries a ★ table where these \
         three arms would be, and `FEATURES.md` calls the trio “the P3 breach the audit was \
         about”. What is missing here is a BOUNDARY CHOOSER: `plan_split` takes a plan \
         (every N pages, at bookmarks, at an explicit list) plus a destination directory and \
         a name template, and “there is no honest default: splitting a 36-sheet drawing set \
         into 36 files because nobody was asked is not a lesser version of the feature”.",
    ),
    (
        "pages.merge_into",
        "★ P3 — needs a file picker and a destination, and is NOT blocked the way it was. \
         Its old entry said `insert` “returns the bytes of a NEW document rather than \
         mutating the session … which discards the command log the undo work is building”. \
         That was true, was filed rather than worked around, and `pdfce-core` answered on \
         2026-08-18 with `EditSession::insert_pages` — which `pages.insert_from_file` now \
         uses. What remains here is a genuine question rather than a blocker: merge_into and \
         insert_from_file differ by WHERE the pages land, so this one wants a destination \
         document, and a shell that can only edit the open document has nowhere to put it \
         yet.",
    ),
    (
        "edit.objects",
        "★ P3 — NO RECORDED REASON ANYWHERE. It appears in a test list and in an argument \
         about its LABEL (the abbreviation `Obj` being illegible), and nowhere else. It is \
         plausibly the third member of the deferred text/authoring phase above, but nothing \
         in the repository says so, and inferring a deferral is not the same as recording \
         one.",
    ),
    (
        "edit.insert_image",
        "★ P3 — NO RECORDED REASON for the missing arm. `manifest/edit.rs` argues only why \
         the COMMAND should exist — “placing an image works today only by drag and drop, \
         which is a gesture with no discoverable equivalent… A command is the affordance” — \
         which is a description of the shell being replaced, and is an argument for a \
         control that works.",
    ),
    (
        "edit.form_create_field",
        "Blocked on a different engine gate from the one filling passes. `panels/forms/rows.rs`: \
         these are “Edit ▸ Forms AUTHORING commands… they answer to core's STRUCTURAL \
         certification gate rather than the fill gate… They land with the commands that name \
         them.” Filling a field and creating one are different permissions on the document.",
    ),
    (
        "edit.form_manage_fields",
        "The same structural gate as the entry above, cited at the same place, and the same \
         backlog row: `FEATURES.md` records forms as “fill ✅ …; create field, flatten and \
         FDF/XFDF/CSV still ⬜”. Its dialog does not exist either.",
    ),
    (
        "edit.form_flatten",
        "The third of the unbuilt forms-authoring verbs, on `FEATURES.md`'s same row — and \
         the one that is irreversible on the document, so it also needs the disclosure \
         surface a destructive verb takes before it can honestly be offered.",
    ),
    // ★ `edit.redact` and `edit.redact_apply` were here until 2026-08-15, and
    // their entries are **deleted rather than reworded**, which is what
    // `no_scaffolded_entry_is_stale`'s middle assertion exists to force. The
    // pair read:
    //
    //     edit.redact — "Salvage. `FEATURES.md`: 'Redaction — mark, review,
    //     apply, with the true-removal proof that exists only in the old
    //     shell.' `SALVAGE.md`'s row for `redact_apply.rs` is stronger still:
    //     '★ This file is currently the ONLY place the proof exists.' Marking
    //     without the proof would be the worse half to ship first."
    //
    //     edit.redact_apply — "The other half of the one unsalvaged unit
    //     above… This is the verb the true-removal proof belongs to, and the
    //     proof is the part that has not crossed over."
    //
    // The proof has crossed over. `crate::redact` carries it whole, and
    // `crate::redact::sealed` asserts that the engine's removal is called from
    // exactly one place — so the reason both entries gave has stopped being
    // true, and an entry that states a false reason is the failure this list's
    // staleness test is about.
    //
    // `edit.redact` is now routed by the `Panel::from_command_id` GUARD arm
    // rather than by a literal one (`crate::panels::Panel::Redact`), which is
    // worth noting because it is the reason no arm bearing its name appears in
    // `dispatch.rs`. `edit.redact_apply` has a literal arm and opens
    // `crate::dialogs::redact`.
    // ===================================================================
    // MARKUP
    // ===================================================================
    // ★★ text_box, sticky_note and stamp were HERE until 2026-08-18.
    //
    // Their recorded reason was accurate and is worth keeping, because it is
    // what the work turned out to be rather than an excuse: `canvas::markup`'s
    // own table of kinds it does not carry — *"Note · text box · sticky ·
    // stamp — Text-bearing, not geometric. A different gesture (place, then
    // type) and a different spec type (`TextAnnotSpec`)."*
    //
    // Both halves were true and neither was small. Building them meant a
    // fourth `CanvasTool` family, a `DragKind` whose release does NOT author,
    // a dialog, two actions and a click path — because the seven geometric
    // kinds author on release from geometry alone, and these cannot: releasing
    // produces an empty box, and an empty box is not an annotation.
    //
    // The stamp additionally needed the gallery `manifest/markup.rs` called
    // for: *"a stamp with no chooser has no operand."*
    // ===================================================================
    // MEASURE
    // ===================================================================
    (
        "measure.manage_groups",
        "★ The dialog it waited on LANDED on 2026-08-17 and this entry stays, which \
         is the interesting part. `crate::dialogs::scale` is a *calibration* window — \
         one group, one scale — and Manage groups is a *list* you add to, rename in \
         and remove from. Two of those four verbs do not exist to call: `EditSession` \
         has `add_dimension_group`, `set_group_scale`, `set_group_standard` and \
         `toggle_dimension_layer`, and `canvas::measure::scale::GroupAction`’s own \
         docs record why the other two are absent — “not in the shipped `EditSession` \
         surface and deliberately NOT reimplemented in the GUI — that would push \
         sidecar-rewriting logic out of core”. A management window missing half its \
         verbs is a worse surface than none. `super::mapping`’s test still records \
         what this must not become meanwhile: “deliberately NOT a tool: it opens a \
         dialog. If it ever answered here it would arm a picking state the operator \
         never asked for.”",
    ),
    // ===================================================================
    // TOOLS
    // ===================================================================
    (
        "tools.merge_files",
        "Salvage, Class C. `SALVAGE.md`'s inventory carries one row for the whole surface — \
         “Batch pane — merge, split, insert, font folders | ~700 lines | `panels/batch/`” — \
         and the pane has not been brought across. `manifest/tools.rs` explains only that the \
         pane's contents were surfaced AS COMMANDS, which is the ribbon half of a job whose \
         other half is a panel.",
    ),
    (
        "tools.split_files",
        "The second verb on the same unsalvaged batch pane row in `SALVAGE.md`. It is the \
         document-set twin of the Pages tab's split, and it inherits that one's missing \
         boundary chooser on top of the missing pane.",
    ),
    (
        "tools.font_folders",
        "The fourth verb on that same unsalvaged batch pane row. `manifest/tools.rs` \
         describes it as a session-scoped setting, which is what it will be — a directory \
         list an operator edits — and a list needs the pane it lives in.",
    ),
    (
        "tools.embed_fonts",
        "`panels/fonts.rs` argues it, and the argument is DATED: “Both push a mutation \
         through `pdfce_core::edit::EditSession`, and at S3 `Action` carries zoom and page \
         navigation and nothing else… A control that cannot commit is an affordance for \
         something that cannot work.” ★ That premise has since expired — the mutation \
         funnel and the undo log both landed on 2026-08-14 — so this is now closer to \
         unwritten than to blocked, and the note wants revisiting.",
    ),
    (
        "tools.unembed_fonts",
        "Its sibling above, plus a reason of its own that has NOT expired: `panels/fonts.rs` \
         records that the old shell's confirmation window exists because “three of \
         unembedding's four consequences are invisible on the canvas (a broken PDF/A claim, \
         an invalidated signature, a renamed font)”. That disclosure surface is rule 4 work \
         and is not built.",
    ),
];

/// **The mirror defect: a literal arm that no token can reach, and why each is
/// tolerated.**
///
/// ★ **Empty since 2026-08-15, and that is the entry.** The list is kept rather
/// than deleted because an empty allow-list is still a gate: a fifth dead arm
/// cannot be added quietly, it has to be written here with a reason, and
/// [`tests::the_p3_tension_is_counted`] pins the length at zero so shortening
/// or lengthening it is a visible act.
///
/// ## What was here, and where it went
///
/// ★ Found by building the check above rather than by looking for it. The first
/// planted violation used to prove the reader bites deleted
/// `"view.zoom_in" => actions.push(Action::ZoomIn)` from the dispatcher — and
/// **nothing was reported**, because `view.zoom_in` is not in the registry at
/// all. Asserting the converse turned up **four** such arms: `view.zoom_in`,
/// `view.zoom_out`, `view.next_page` and `view.prev_page`. There was no catalog
/// entry, no manifest item, no [`crate::text::commands`] copy and no
/// `RIBBON_IA.md` row for any of them, so no token existed and no operator
/// gesture had ever reached one — dispatch begins at a registered command's
/// token, and there was none to begin at.
///
/// That is the exact thing `app::dispatch`'s own `format.delete` arm forbids in
/// writing — *"adding an arm for one would be an arm no token can ever reach —
/// dead code wearing a design pattern, which is what the no-placeholders
/// invariant forbids"* — and it is the inverse of the defect [`SCAFFOLDED`] is
/// about: there, a control with no arm; here, an arm with no control.
///
/// **All four arms were deleted**, after each of the four verbs was checked to
/// have a live route that is not the dispatcher:
///
/// | verb | keyboard | status bar |
/// |---|---|---|
/// | `Action::ZoomIn` | `app::keyboard`, `Ctrl` `+` | `status::zoom_group`'s `+` |
/// | `Action::ZoomOut` | `app::keyboard`, `Ctrl` `-` | `status::zoom_group`'s `−` |
/// | `Action::NextPage` | `app::keyboard`, `PageDown` | `status::page_box`'s `▶` |
/// | `Action::PrevPage` | `app::keyboard`, `PageUp` | `status::page_box`'s `◀` |
///
/// So the deletion removed **duplicate entrances, not behaviour**, and
/// `RIBBON_IA.md` §6 says the entrances that remain are the specified ones:
/// *"Find toggle, actual size, fit width, fit page, zoom −/%/+, page ◀ n/N ▶
/// … These are the controls a user touches constantly; they belong where they
/// never disappear behind a tab change."* Both pairs are status-bar verbs by
/// specification, and the arms were the redundant half.
///
/// ## The other answer, and what it would take
///
/// Registering the four instead is a **ribbon** decision and the operator's,
/// and it is recorded here so it can be made rather than inherited. It would be
/// four `catalog::all` entries in the `view.` token block, four
/// [`crate::text::commands`] pairs, a `RIBBON_IA.md` row each and a place to
/// draw them — View ▸ Zoom for the step pair, which currently draws Actual size,
/// Fit page, Fit width, Region and Selection and conspicuously not these two.
/// Page navigation would need a group that does not exist, against §6's
/// deliberate placement of it on the bar. The arms would then come back, and
/// each would be one line pushing the `Action` its two live routes already push.
///
/// ## The quieter failure, and why it is worth a list of its own
///
/// An inert control at least *looks* wrong when an operator presses it. A dead
/// arm reads as working code: it will be maintained, reviewed and reasoned
/// about by everyone who passes it, and no test in the suite touched those four
/// before this one.
pub(crate) const UNREACHED_ARMS: &[(&str, &str)] = &[];
