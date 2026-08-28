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
    // ★★★ `file.export_form_data` was HERE until 2026-08-27, and its entry is
    // **deleted rather than reworded**. It read:
    //
    //   "Blocked on a writer that does not exist. `FEATURES.md`'s Forms row:
    //   “fill ✅ …; create field, flatten and FDF/XFDF/CSV still ⬜” — and this
    //   command IS the FDF/XFDF/CSV half."
    //
    // **The writer exists three times over**, and two of them since `Pass 7.1`:
    // `fdf::FormData::to_fdf`, `fdf::FormData::to_xfdf` and `formcsv::to_csv`,
    // reached through `EditSession::export_form_data`. The `FEATURES.md` row it
    // cites was itself stale.
    //
    // ★★ That makes it the **second citation-of-a-citation in one evening** —
    // `edit.form_flatten` was the first, two hours earlier, and the sixth stale
    // blocker this project has found. Both were discovered by the rule written
    // on this list's own count assertion: *when you touch this list for any
    // purpose, re-derive the reason of the entry beside the one you came for.*
    // It has now paid for itself twice on the day it was written.
    //
    // The arm is in `app::dispatch`; the verb is `app::actions::export::form_data`.
    // ★★★ `view.show_points` was HERE until 2026-08-28. Its reason ran to
    // twenty lines and said, in substance:
    //
    //   "★ P3 — and the reason was found on 2026-08-15 rather than written:
    //   **there is nothing for it to show.** This build draws no anchor mark at
    //   any rung … `CanvasTargetProvider` offers `nearest_node` — a *query* —
    //   with no way to enumerate a part's anchors to paint them."
    //
    // **True on 2026-08-15 and false four days later.** The multi-node move
    // landed on 2026-08-19 with `canvas::overlay::draw_anchors`, and with the
    // enumeration the reason said did not exist — `object_node_points` and
    // `subpath_node_points`, both already called by `canvas::painting`.
    //
    // ★★ The dead sentence had **three copies**: this entry, a `FEATURES.md`
    // row, and a third — while `FEATURES.md` itself says twelve lines from one
    // of them *"the anchors are VISIBLE, and until 2026-08-19 they were not, at
    // any rung."* **A file contradicting itself is the shape a scheduled audit
    // finds and an opportunistic one does not.**
    //
    // ★ What is wired is the toggle at the draw's EXISTING scope. Widening it
    // to every object on the page is a separate decision and it is the
    // operator's: `MAX_UNSELECTED_ANCHORS` is 400 and has already fired blank
    // once on his own SW41177, so "show all the points" on a CAD sheet is a
    // question about what to do with five thousand of them rather than a flag.
    //
    // No arm in `dispatch` — it is a `ViewChrome` variant, so
    // `shell::commands::mapping::chrome_command` routes it with the other
    // three, and `canvas::painting::draw_anchors` reads it.
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
    // ★★★ `pages.merge_into` was HERE until 2026-08-28, and its entry is
    // **deleted rather than reworded** — the fourth such deletion in two days.
    // It is also the entry whose history best shows what this list is FOR and
    // where it fails, because it had two reasons and they were wrong in
    // opposite ways.
    //
    // **The first was right and was answered.** It read: *"`insert` returns the
    // bytes of a NEW document rather than mutating the session … wiring it
    // means replacing `OpenDoc::session` wholesale, which discards the command
    // log the undo work is building."* True when written, filed rather than
    // worked around, and answered on 2026-08-18 with `merge_document` —
    // in-session, one undo entry, field collisions renamed. **That is this list
    // working exactly as intended.**
    //
    // **The second was written to replace it and had the destination
    // backwards**: *"merge_into wants a destination document, and a shell that
    // can only edit the open document has nowhere to put it yet."* The
    // manifest's own taxonomy says the opposite two files away — Pages ▸ Merge
    // *adds pages to this document*, Tools ▸ Merge *combines files into a new
    // one* — so the open document **is** the destination and always was.
    //
    // ⇒ The lesson is not "check harder when you write one". It is that a
    // reason **rewritten after a blocker clears** gets none of the scrutiny the
    // original had, and lands in a list nothing re-reads. Found by re-deriving
    // all eleven entries; six were wrong.
    //
    // The arm is in `app::dispatch`; the verb is `app::actions::pages::merge_into`.
    (
        "edit.objects",
        "★ P3 — NO RECORDED REASON ANYWHERE. It appears in a test list and in an argument \
         about its LABEL (the abbreviation `Obj` being illegible), and nowhere else. It is \
         plausibly the third member of the deferred text/authoring phase above, but nothing \
         in the repository says so, and inferring a deferral is not the same as recording \
         one.",
    ),
    // ★★★ `edit.form_manage_fields` was HERE until 2026-08-28, and its entry is
    // **deleted rather than reworded** — the third such deletion in two days
    // and the one whose reason was most thoroughly hollow. It read:
    //
    //   "The same structural gate as the entry above, cited at the same place,
    //   and the same backlog row: `FEATURES.md` records forms as 'fill ✅ …;
    //   create field, flatten and FDF/XFDF/CSV still ⬜'. Its dialog does not
    //   exist either."
    //
    // **Every clause failed, in a different way, which is why it is worth
    // spelling out:**
    //
    // * *"the same structural gate as the entry above"* — a **dangling
    //   back-reference**. The entry that carried the gate left the list when
    //   `edit.form_create_field` was wired, so "the entry above" is now
    //   `edit.objects`, which records no gate at all. And the gate never
    //   existed: `FEATURES.md` calls it this project's **fourth stale
    //   blocker**, disproved by probing the engine for two minutes.
    // * *"the same backlog row"* — a **citation of a citation**. The row no
    //   longer says what is quoted, and its current wording is *also* stale,
    //   contradicted four times over in its own file.
    // * *"its dialog does not exist either"* — true, and irrelevant. It does
    //   not need one.
    //
    // ⇒ Wired as a **second route**, not a third surface. Every operation the
    // tooltip promises — list, rename, retype, remove — is already reachable in
    // the Forms panel and the Properties pane, so the arm raises
    // `Action::Command("view.panel_forms")`. `format.properties` set that
    // precedent and its own docs give the rule: *"it exists so a second route
    // to an existing command cannot become a second implementation of it."*
    // Building a manage-fields dialog would have been a third surface for verbs
    // that already have two.
    // ★★★ `edit.form_flatten` was HERE until 2026-08-27, and its entry is
    // **deleted rather than reworded** — which is what
    // `no_scaffolded_entry_is_stale`'s middle assertion exists to force. It
    // read:
    //
    //   "The third of the unbuilt forms-authoring verbs, on `FEATURES.md`'s
    //   same row — and the one that is irreversible on the document, so it
    //   also needs the disclosure surface a destructive verb takes before it
    //   can honestly be offered."
    //
    // **Both halves had become false, and neither could fail a test.**
    //
    // *Unbuilt*: `EditSession::flatten_fields` exists and this shell has been
    // calling it since the Forms panel shipped. The `FEATURES.md` row it cites
    // was itself stale — field creation shipped as O39 on 2026-08-26 — so this
    // entry's reason was a citation of a citation, and nothing re-read either.
    //
    // *Irreversible*: it is one `EditSession` command and one `Ctrl+Z`, and
    // `text::forms`' `forms_flatten_tooltip` had already argued at length that
    // flatten APPENDS an overlay and leaves existing content byte-verbatim, so
    // under the default incremental save the prior revision still holds the
    // values. Its irreversibility is conditional on the save mode, not
    // structural — which is why the panel's own button is delete-shaped rather
    // than modal.
    //
    // ⇒ An entry that states a false reason is the failure this list's
    // staleness test is about, and this one survived because the test asks
    // whether the id has an arm, not whether the sentence is still true. There
    // is no mechanism for the second question; a reader is the only instrument.
    // The arm is `app::dispatch::forms::flatten`.
    // ★ `edit.redact` and `edit.redact_apply` were here until 2026-08-15, and
    // their entries are **deleted rather than reworded**, which is what
    // `no_scaffolded_entry_is_stale`'s middle assertion exists to force. The
    // pair read:
    //
    // edit.redact — "Salvage. `FEATURES.md`: 'Redaction — mark, review,
    // apply, with the true-removal proof that exists only in the old
    // shell.' `SALVAGE.md`'s row for `redact_apply.rs` is stronger still:
    // '★ This file is currently the ONLY place the proof exists.' Marking
    // without the proof would be the worse half to ship first."
    //
    // edit.redact_apply — "The other half of the one unsalvaged unit
    // above… This is the verb the true-removal proof belongs to, and the
    // proof is the part that has not crossed over."
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
    // ★★★ `tools.font_folders` was HERE until 2026-08-28. Its reason was
    // **accurate about the engine and wrong about this shell**, which is a
    // third distinct way for one of these to go stale and worth naming.
    //
    // It said the setting is session-scoped and reachable only through the old
    // shell's batch pane, *"and a list needs the pane it lives in"*. Every
    // clause about `--font-dir` and about the batch pane being unsalvaged was
    // **true and still is**. The false clause is the last one: nothing about a
    // list of directories needs a batch pane, and `dialogs::settings` has nine
    // modules, seven groups, and a stated subject of *settings that persist
    // across documents*.
    //
    // ⇒ **A blocker naming a missing HOST is weaker than one naming a missing
    // capability**, and it goes stale the moment any other host will do —
    // silently, because nothing about the shell changed to make it stale. Six
    // of eleven entries were wrong on the day this was found and this is the
    // only one whose falsity required no event at all.
    //
    // ★ It also concealed a **real** dependency in the entry below it:
    // `EmbedRequest::supplied` needs donor files and pdfce *"never goes
    // looking"*, so `tools.embed_fonts` was blocked on this and neither entry
    // said so.
    //
    // The list is `dialogs::settings::fonts`; the preference is
    // `app::prefs::fonts`; the arm raises `Action::Command("file.settings")`.
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
